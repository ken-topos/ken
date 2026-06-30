//! Reference interpreter: core terms → values (`spec/40-runtime/42-evaluation.md`).
//!
//! Strategy: environment-based CBV with sharing via the K3 content-addressed
//! store. Reduction rules per §1: β, Σ-β, ι, δ, obs (cast/Eq/quotient), prim.
//! Branch laziness: eliminator methods held unevaluated; only the scrutinee-
//! selected method (ι) fires.
//!
//! # EvalVal variants
//! - Scalar immediates (`Bool`, `Int`) — not K3-interned (they are K3 immediates).
//! - Compound data (`Ctor`, `Pair`, `Closure`) — K3-interned, carry a `SlotId`.
//! - Type-former values (`TypeUniverse`, `OmegaUniverse`, `PiTy`, `SigmaTy`,
//!   `IndFormerVal`) — not K3-interned; irreducible at the value layer (G1 scope).
//! - `CtorPending` — accumulates positional args before the constructor saturates.
//! - `Unknown` — open-hole residue (propagates strictly through all positions).
//! - `Neutral` — stuck on an opaque constant or open variable (closed ground
//!   programs never reach this per canonicity).

use std::rc::Rc;

use ken_kernel::env::{Decl, GlobalEnv, PrimReduction};
use ken_kernel::term::{GlobalId, Level, Term};
use ken_runtime::{fnv1a_64, InternResult, Store, Value as RtValue};

// Re-export the slot-id type used by the K3 store.
pub type SlotId = u64;
const NULL_SLOT: SlotId = 0;

/// An evaluation environment: values indexed by de Bruijn depth.
/// `env[env.len() - 1 - i]` is the value of de Bruijn variable `i`.
pub type Env = Vec<EvalVal>;

fn env_lookup(env: &[EvalVal], i: usize) -> EvalVal {
    let n = env.len();
    if i < n {
        env[n - 1 - i].clone()
    } else {
        EvalVal::Neutral
    }
}

fn env_extend(env: &[EvalVal], val: EvalVal) -> Env {
    let mut e = env.to_vec();
    e.push(val);
    e
}

/// Runtime value — the output type of `eval` (`spec/40-runtime/41`, `42 §3.1`).
#[derive(Clone, Debug, PartialEq)]
pub enum EvalVal {
    // --- Scalar immediates (K3 stores these without interning) ---
    Bool(bool),
    Int(i64),

    // --- Compound data values (K3-interned; slot_id uniquely identifies content) ---
    /// Fully-applied constructor: `cₖ v̄`.  `args` holds ALL applied arguments
    /// (params then ctor-specific); `slot` is the K3 store slot id.
    Ctor {
        id: GlobalId,
        args: Rc<Vec<EvalVal>>,
        slot: SlotId,
    },
    /// Dependent pair `(v₁, v₂)` (Σ-type intro); K3-interned.
    Pair {
        fst: Rc<EvalVal>,
        snd: Rc<EvalVal>,
        slot: SlotId,
    },
    /// Closure `⟨λ(x:A).t ; ρ⟩`; K3-interned by `(code_id, captured_env_slots)`.
    Closure {
        body: Rc<Term>,
        captured: Rc<Env>,
        slot: SlotId,
    },

    // --- Constructor pending (arity not yet reached) ---
    /// A constructor partially applied — accumulates args until it saturates.
    CtorPending {
        id: GlobalId,
        args: Vec<EvalVal>,
        need: usize,
    },

    // --- Type-former values (carry the kernel's explicit levels; not K3-interned
    //     for G1 — type computation is out-of-scope for this pure-data release) ---
    TypeUniverse(Level),
    OmegaUniverse(Level),
    PiTy {
        dom: Rc<EvalVal>,
        cod: Rc<Term>,
        env: Rc<Env>,
    },
    SigmaTy {
        dom: Rc<EvalVal>,
        cod: Rc<Term>,
        env: Rc<Env>,
    },
    IndFormerVal {
        id: GlobalId,
    },
    /// Refl proof (used by cast C5).
    ReflVal {
        ty: Rc<EvalVal>,
        val: Rc<EvalVal>,
    },

    // --- Special residues ---
    /// An open verification hole (`hole h`) or opaque postulate — the "unknown"
    /// truth value from `41 §6`.
    Unknown,
    /// A neutral head applied to values — only possible for open terms; closed
    /// ground programs never produce this per canonicity (`42 §3.6`).
    Neutral,
}

// ── K3 interning helpers ─────────────────────────────────────────────────────

/// The type_id used for Pair (Σ-intro) in the K3 store.
/// Not a valid GlobalId (GlobalIds start at 0 from declarations); this
/// sentinel sits past any reachable id space for the G1 corpus.
const PAIR_TYPE_ID: u32 = u32::MAX;

/// Convert an `EvalVal` to a K3 `Value` for interning.
/// Returns `None` if the value cannot be represented as a K3 compound
/// (type-former values, Unknown, Neutral, pending).
fn to_rt(val: &EvalVal) -> Option<RtValue> {
    match val {
        EvalVal::Bool(b) => Some(RtValue::Bool(*b)),
        EvalVal::Int(n) => Some(RtValue::SmallInt(*n)),
        EvalVal::Ctor { id, args, .. } => {
            let fields: Vec<RtValue> = args.iter().filter_map(to_rt).collect();
            if fields.len() == args.len() {
                Some(RtValue::Record {
                    type_id: id.0,
                    fields,
                })
            } else {
                None
            }
        }
        EvalVal::Pair { fst, snd, .. } => {
            let f = to_rt(fst)?;
            let s = to_rt(snd)?;
            Some(RtValue::Record {
                type_id: PAIR_TYPE_ID,
                fields: vec![f, s],
            })
        }
        EvalVal::Closure { body, captured, .. } => {
            let code_id = fnv1a_64(format!("{:?}", body).as_bytes());
            let cap_fields: Vec<RtValue> = captured.iter().filter_map(to_rt).collect();
            // Only intern if all captured values are representable.
            if cap_fields.len() == captured.len() {
                Some(RtValue::Closure {
                    code_id,
                    captured: cap_fields,
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Intern a K3-compatible `EvalVal` and return its slot id.
/// Returns `NULL_SLOT` if the value is not internable (type values, etc.).
fn intern(val: &EvalVal, store: &mut Store) -> SlotId {
    let rt = match to_rt(val) {
        Some(r) => r,
        None => return NULL_SLOT,
    };
    if !rt.is_compound() {
        return NULL_SLOT;
    }
    match store.intern(&rt) {
        InternResult::New(s) | InternResult::Hit(s) => s,
        InternResult::CapacityExhausted { .. } => NULL_SLOT,
    }
}

/// Build a fully-applied `Ctor` value and intern it.
fn make_ctor(id: GlobalId, args: Vec<EvalVal>, store: &mut Store) -> EvalVal {
    let slot = intern(
        &EvalVal::Ctor {
            id,
            args: Rc::new(args.clone()),
            slot: NULL_SLOT,
        },
        store,
    );
    EvalVal::Ctor {
        id,
        args: Rc::new(args),
        slot,
    }
}

/// Build a `Pair` value and intern it.
fn make_pair(fst: EvalVal, snd: EvalVal, store: &mut Store) -> EvalVal {
    let slot = intern(
        &EvalVal::Pair {
            fst: Rc::new(fst.clone()),
            snd: Rc::new(snd.clone()),
            slot: NULL_SLOT,
        },
        store,
    );
    EvalVal::Pair {
        fst: Rc::new(fst),
        snd: Rc::new(snd),
        slot,
    }
}

/// Build a `Closure` value and intern it.
fn make_closure(body: Rc<Term>, captured: Rc<Env>, store: &mut Store) -> EvalVal {
    let slot = intern(
        &EvalVal::Closure {
            body: body.clone(),
            captured: captured.clone(),
            slot: NULL_SLOT,
        },
        store,
    );
    EvalVal::Closure {
        body,
        captured,
        slot,
    }
}

// ── ι (eliminator) reduction ──────────────────────────────────────────────────

/// Fire the ι reduct for an eliminator (`42 §3.3`).
///
/// Only the constructor-selected method `methods[k]` is evaluated (branch
/// laziness). All other methods are discarded unevaluated.
///
/// Method application order (matching `iota_reduct` in `ken-kernel`):
///   1. Apply all constructor-specific args (args[m..] where m = params.len()).
///   2. Then apply the IH for each recursive position.
/// Returns whether a constructor arg type refers to the inductive type `fam`
/// at its head (strict positivity, simple case). This is used to identify
/// recursive positions because `ConstructorDecl.recursive_positions` is not
/// populated by the kernel for G1-scope inductives.
fn is_recursive_arg(arg_ty: &Term, fam: GlobalId) -> bool {
    match arg_ty {
        Term::IndFormer { id, .. } => *id == fam,
        // Descend into the operator of an application (e.g., I applied to
        // indices) — handles `I i₁ … iₙ` in indexed families.
        Term::App(f, _) => is_recursive_arg(f, fam),
        _ => false,
    }
}

fn elim_reduce(
    env: &[EvalVal],
    fam: GlobalId,
    methods: &[Term],
    scrut: EvalVal,
    globals: &GlobalEnv,
    store: &mut Store,
) -> EvalVal {
    match scrut {
        EvalVal::Unknown => EvalVal::Unknown,
        EvalVal::Neutral => EvalVal::Neutral,
        EvalVal::Ctor {
            id: ctor_id,
            ref args,
            ..
        } => {
            let (ind, k) = match globals.constructor(ctor_id) {
                Some(x) => x,
                None => return EvalVal::Neutral,
            };
            let m = ind.params.len();
            let ctor_decl = &ind.constructors[k];
            let ctor_specific: &[EvalVal] = &args[m..];

            // Compute recursive positions from arg types (kernel never populates
            // ConstructorDecl.recursive_positions for G1-scope inductives).
            let rec_positions: Vec<usize> = ctor_decl
                .args
                .iter()
                .enumerate()
                .filter(|(_, ty)| is_recursive_arg(ty, fam))
                .map(|(i, _)| i)
                .collect();

            // Evaluate ONLY the selected method (the others are never touched).
            let mut mval = eval(env, &methods[k], globals, store);

            // Apply all ctor-specific args left-to-right.
            for arg in ctor_specific {
                mval = apply(mval, arg.clone(), globals, store);
            }

            // Apply IH values for recursive positions (in order).
            for rec_pos in &rec_positions {
                let rec_arg = ctor_specific[*rec_pos].clone();
                let ih = elim_reduce(env, fam, methods, rec_arg, globals, store);
                mval = apply(mval, ih, globals, store);
            }

            mval
        }
        _ => EvalVal::Neutral,
    }
}

// ── observational reductions ─────────────────────────────────────────────────

/// `castReduce A B e a` — C5 regularity: `cast A A refl a → a`.
///
/// For this G1 release only C5 is grounded. The structural C6 push and the
/// `cast Type Type` edge cases are tagged `(oracle)` in `16 §9.1`; we return
/// `Unknown` for them (not locked, not an error).
fn cast_reduce(a_ty: EvalVal, b_ty: EvalVal, eq: EvalVal, val: EvalVal) -> EvalVal {
    if let EvalVal::Unknown = val {
        return EvalVal::Unknown;
    }
    // C5: cast A A refl a → a (regularity, `16 §3.2`).
    if eq_type_eq(&a_ty, &b_ty) {
        if matches!(eq, EvalVal::ReflVal { .. }) {
            return val;
        }
    }
    // All other cases are (oracle) — yield Unknown for the G1 scope.
    EvalVal::Unknown
}

/// `eqReduce A a b` — Eq-by-type (`16 §2.2`, C2–C4).
///
/// For same-head inductive constructors → conjunction of field equalities
/// (trivially `Top` for 0-field constructors like `true`/`false`).
/// Different constructors → `Bottom`. Types in Ω, proof-irrelevant at value
/// layer — the value IS the proposition type (`42 §3.3`, `16 §1.2`).
///
/// The exact form for multi-field same-ctor is `(oracle)`; we return `Unknown`
/// for that and for Π/Ω cases (C2/C3 are oracle-grounded, not locked here).
fn eq_reduce(a_ty: EvalVal, lhs: EvalVal, rhs: EvalVal, globals: &GlobalEnv) -> EvalVal {
    // Unknown operands propagate strictly.
    if matches!(a_ty, EvalVal::Unknown)
        || matches!(lhs, EvalVal::Unknown)
        || matches!(rhs, EvalVal::Unknown)
    {
        return EvalVal::Unknown;
    }

    // C4: Eq at an inductive type, same constructor (0-field → Top), diff → Bottom.
    // Both are represented as opaque IndFormerVal pointing to the prelude constants.
    if let (
        EvalVal::Ctor {
            id: l_id,
            args: l_args,
            ..
        },
        EvalVal::Ctor { id: r_id, .. },
    ) = (&lhs, &rhs)
    {
        if l_id == r_id {
            if l_args.is_empty() {
                // 0-field same-ctor: trivially equal proposition → Top.
                return EvalVal::IndFormerVal {
                    id: globals.top_id(),
                };
            }
            // Multi-field same-ctor: (oracle), not locked for G1.
            return EvalVal::Unknown;
        } else {
            // Different constructors → Bottom proposition.
            return EvalVal::IndFormerVal {
                id: globals.bottom_id(),
            };
        }
    }

    EvalVal::Unknown
}

/// Structural type equality (by value structure, not alpha-eq of closed types).
/// Used only by C5 cast-refl to confirm the source and target are the same type.
fn eq_type_eq(a: &EvalVal, b: &EvalVal) -> bool {
    match (a, b) {
        (EvalVal::TypeUniverse(la), EvalVal::TypeUniverse(lb)) => la.equiv(lb),
        (EvalVal::OmegaUniverse(la), EvalVal::OmegaUniverse(lb)) => la.equiv(lb),
        (EvalVal::IndFormerVal { id: ia }, EvalVal::IndFormerVal { id: ib }) => ia == ib,
        (
            EvalVal::Ctor {
                id: a, args: aa, ..
            },
            EvalVal::Ctor {
                id: b, args: ba, ..
            },
        ) => {
            a == b
                && aa.len() == ba.len()
                && aa.iter().zip(ba.iter()).all(|(x, y)| eq_type_eq(x, y))
        }
        _ => false,
    }
}

// ── prim reduction ────────────────────────────────────────────────────────────

/// Primitive reduction for registered operations (`42 §3.3`, `14 §5`).
///
/// Only `add`/`sub`/`mul` on `Int` literals are grounded for the G1 corpus.
/// Division and other operations that may fault are out of scope (§6 / `43 §2.2`).
fn prim_reduce(symbol: &str, args: &[EvalVal]) -> EvalVal {
    // Unknown operand: propagate strictly.
    if args.iter().any(|a| matches!(a, EvalVal::Unknown)) {
        return EvalVal::Unknown;
    }
    // Neutral operand: stuck.
    if args.iter().any(|a| matches!(a, EvalVal::Neutral)) {
        return EvalVal::Neutral;
    }

    match (symbol, args) {
        ("add", [EvalVal::Int(a), EvalVal::Int(b)]) => EvalVal::Int(a.wrapping_add(*b)),
        ("sub", [EvalVal::Int(a), EvalVal::Int(b)]) => EvalVal::Int(a.wrapping_sub(*b)),
        ("mul", [EvalVal::Int(a), EvalVal::Int(b)]) => EvalVal::Int(a.wrapping_mul(*b)),
        ("not_bool", [EvalVal::Bool(b)]) => EvalVal::Bool(!b),
        // Partial or unrecognised primitive: neutral (stuck on non-literals).
        _ => EvalVal::Neutral,
    }
}

// ── eval / apply ─────────────────────────────────────────────────────────────

/// `eval ρ t` — evaluate a core term in environment `ρ` (`42 §3.2`).
pub fn eval(env: &[EvalVal], term: &Term, globals: &GlobalEnv, store: &mut Store) -> EvalVal {
    match term {
        // --- Var: environment lookup ---
        Term::Var(i) => env_lookup(env, *i),

        // --- Type universe and Ω ---
        Term::Type(l) => EvalVal::TypeUniverse(l.clone()),
        Term::Omega(l) => EvalVal::OmegaUniverse(l.clone()),

        // --- Lambda: form a closure (body NOT reduced under binder) ---
        Term::Lam(_dom, body) => make_closure(Rc::new(*body.clone()), Rc::new(env.to_vec()), store),

        // --- Application: CBV — force operator then argument ---
        Term::App(f, u) => {
            let fv = eval(env, f, globals, store);
            let uv = eval(env, u, globals, store);
            apply(fv, uv, globals, store)
        }

        // --- Pi / Sigma type formers ---
        Term::Pi(a, b) => {
            let dom = eval(env, a, globals, store);
            EvalVal::PiTy {
                dom: Rc::new(dom),
                cod: Rc::new(*b.clone()),
                env: Rc::new(env.to_vec()),
            }
        }
        Term::Sigma(a, b) => {
            let dom = eval(env, a, globals, store);
            EvalVal::SigmaTy {
                dom: Rc::new(dom),
                cod: Rc::new(*b.clone()),
                env: Rc::new(env.to_vec()),
            }
        }

        // --- Pair intro / projections (Σ-β) ---
        Term::Pair(a, b) => {
            let av = eval(env, a, globals, store);
            let bv = eval(env, b, globals, store);
            if matches!(av, EvalVal::Unknown) || matches!(bv, EvalVal::Unknown) {
                return EvalVal::Unknown;
            }
            make_pair(av, bv, store)
        }
        Term::Proj1(p) => {
            let pv = eval(env, p, globals, store);
            match pv {
                EvalVal::Pair { fst, .. } => (*fst).clone(),
                EvalVal::Unknown => EvalVal::Unknown,
                _ => EvalVal::Neutral,
            }
        }
        Term::Proj2(p) => {
            let pv = eval(env, p, globals, store);
            match pv {
                EvalVal::Pair { snd, .. } => (*snd).clone(),
                EvalVal::Unknown => EvalVal::Unknown,
                _ => EvalVal::Neutral,
            }
        }

        // --- Let: strict binding, shared result ---
        Term::Let { val, body, .. } => {
            let vv = eval(env, val, globals, store);
            let env2 = env_extend(env, vv);
            eval(&env2, body, globals, store)
        }

        // --- Ascription: erased at runtime ---
        Term::Ascript(t, _) => eval(env, t, globals, store),

        // --- Const: δ-unfold transparent; postulate → Unknown; prim → pending ---
        Term::Const { id, .. } => match globals.lookup(*id) {
            Some(Decl::Transparent { body, .. }) => eval(&Vec::new(), body, globals, store),
            Some(Decl::Primitive { reduction, .. }) => match reduction {
                PrimReduction::OpaqueType => EvalVal::Neutral,
                PrimReduction::Op { symbol } => EvalVal::CtorPending {
                    id: *id,
                    args: vec![],
                    need: prim_arity(symbol),
                },
            },
            Some(Decl::Inductive(_)) => EvalVal::IndFormerVal { id: *id },
            // Opaque constant / postulate: no body → unknown (`42 §3.3`, `§4`).
            _ => EvalVal::Unknown,
        },

        // --- IndFormer: a type-former value ---
        Term::IndFormer { id, .. } => EvalVal::IndFormerVal { id: *id },

        // --- Constructor: return a pending or saturated ctor value ---
        Term::Constructor { id, .. } => {
            let arity = ctor_arity(*id, globals);
            if arity == 0 {
                make_ctor(*id, vec![], store)
            } else {
                EvalVal::CtorPending {
                    id: *id,
                    args: vec![],
                    need: arity,
                }
            }
        }

        // --- Elim: ι fires only the selected method (branch laziness) ---
        Term::Elim {
            fam,
            methods,
            scrut,
            ..
        } => {
            let sv = eval(env, scrut, globals, store);
            elim_reduce(env, *fam, methods, sv, globals, store)
        }

        // --- Refl: carry the type and value for cast C5 ---
        Term::Refl(t) => {
            let tv = eval(env, t, globals, store);
            EvalVal::ReflVal {
                ty: Rc::new(tv.clone()),
                val: Rc::new(tv),
            }
        }

        // --- Cast: observational regularity C5 + structural C6 ---
        Term::Cast(a, b, e, t) => {
            let av = eval(env, a, globals, store);
            let bv = eval(env, b, globals, store);
            let ev = eval(env, e, globals, store);
            let tv = eval(env, t, globals, store);
            cast_reduce(av, bv, ev, tv)
        }

        // --- Eq by type (`16 §2.2`, C2–C4) ---
        Term::Eq(a, l, r) => {
            let av = eval(env, a, globals, store);
            let lv = eval(env, l, globals, store);
            let rv = eval(env, r, globals, store);
            eq_reduce(av, lv, rv, globals)
        }

        // --- Quotient eliminator: C9 `elim_/ M f r [a] → f a` ---
        Term::QuotElim { method, scrut, .. } => {
            let sv = eval(env, scrut, globals, store);
            match sv {
                EvalVal::Unknown => EvalVal::Unknown,
                EvalVal::Ctor { args, .. } => {
                    // QuotClass constructor: apply f to the representative.
                    let fv = eval(env, method, globals, store);
                    // args[0] is the representative `a`.
                    if let Some(rep) = args.first() {
                        apply(fv, rep.clone(), globals, store)
                    } else {
                        EvalVal::Neutral
                    }
                }
                _ => EvalVal::Neutral,
            }
        }

        // --- QuotClass: wrap in a constructor-like value ---
        Term::QuotClass(t) => {
            let tv = eval(env, t, globals, store);
            // Represent [a] as a 1-arg constructor with id=0 (synthetic).
            // For the G1 scope, quotient classes are not interned.
            EvalVal::Ctor {
                id: GlobalId(u32::MAX), // synthetic quot-class id
                args: Rc::new(vec![tv]),
                slot: NULL_SLOT,
            }
        }

        // --- Open hole → unknown (`42 §4`) ---
        // (No Hole variant in Term — holes are represented as opaque Consts
        // with no body, handled in the Const case above.)

        // --- Remaining K2 forms: not reduced in the G1 scope ---
        _ => EvalVal::Neutral,
    }
}

/// `apply f u` — apply a value to an argument (`42 §3.2`).
pub fn apply(f: EvalVal, u: EvalVal, globals: &GlobalEnv, store: &mut Store) -> EvalVal {
    match f {
        // --- β: closure application extends the captured env ---
        EvalVal::Closure { body, captured, .. } => {
            let mut env2 = (*captured).clone();
            env2.push(u);
            eval(&env2, &body, globals, store)
        }

        // --- Constructor pending: collect args until saturated ---
        EvalVal::CtorPending { id, mut args, need } => {
            // Unknown propagates strictly through constructor arguments.
            // (A data constructor VALUE depends on all its fields; if a field
            // is unknown the whole constructor application is unknown.)
            if matches!(u, EvalVal::Unknown) {
                return EvalVal::Unknown;
            }
            args.push(u);
            if args.len() >= need {
                // Saturated — check if this is a prim or a data constructor.
                if let Some(Decl::Primitive { reduction, .. }) = globals.lookup(id) {
                    if let PrimReduction::Op { symbol } = reduction {
                        return prim_reduce(symbol, &args);
                    }
                }
                make_ctor(id, args, store)
            } else {
                EvalVal::CtorPending { id, args, need }
            }
        }

        // --- Unknown: propagate strictly ---
        EvalVal::Unknown => EvalVal::Unknown,

        // --- Neutral: remain stuck ---
        _ => EvalVal::Neutral,
    }
}

// ── utility helpers ───────────────────────────────────────────────────────────

/// Total arity of a constructor (params + ctor-specific args).
fn ctor_arity(id: GlobalId, globals: &GlobalEnv) -> usize {
    globals
        .constructor(id)
        .map(|(ind, k)| ind.params.len() + ind.constructors[k].args.len())
        .unwrap_or(0)
}

/// Arity of a known primitive operation.
fn prim_arity(symbol: &str) -> usize {
    match symbol {
        "add" | "sub" | "mul" => 2,
        "not_bool" => 1,
        _ => 1,
    }
}
