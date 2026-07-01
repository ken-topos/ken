//! Reference interpreter: core terms → values (`spec/40-runtime/42-evaluation.md`).
//!
//! Strategy: environment-based CBV with sharing via the K3 content-addressed
//! store. Reduction rules per §1: β, Σ-β, ι, δ, obs (cast/Eq/quotient), prim.
//! Branch laziness: eliminator methods held unevaluated; only the scrutinee-
//! selected method (ι) fires.
//!
//! # EvalVal variants
//! - Scalar immediates (`Bool`, `Int`, `BigInt`, `Float`, `Float32`,
//!   `DecimalVal`) — not K3-interned (they are K3 immediates).
//! - Compound data (`Ctor`, `Pair`, `Closure`) — K3-interned, carry a `SlotId`.
//! - Type-former values (`TypeUniverse`, `OmegaUniverse`, `PiTy`, `SigmaTy`,
//!   `IndFormerVal`) — not K3-interned; irreducible at the value layer (G1 scope).
//! - `CtorPending` — accumulates positional args before the constructor saturates.
//! - `Unknown` — open-hole residue (propagates strictly through all positions).
//! - `Neutral` — stuck on an opaque constant or open variable (closed ground
//!   programs never reach this per canonicity).

use std::collections::HashMap;
use std::rc::Rc;

use ken_kernel::env::{Decl, GlobalEnv, PrimReduction};
use ken_kernel::term::{GlobalId, Level, Term};
use ken_runtime::{InternResult, Store, Value as RtValue};

// Re-export the slot-id type used by the K3 store.
pub type SlotId = u64;
const NULL_SLOT: SlotId = 0;

/// Evaluation-time store: wraps the K3 content-addressed heap with a
/// `code_id` side table so distinct closure bodies get distinct, collision-free
/// integer ids (the F4 lesson: closure equality is memcmp-exact, never a digest).
pub struct EvalStore {
    /// The underlying K3 content-addressed heap.
    pub k3: Store,
    /// Maps closure body Term (by content equality) to a sequential `code_id`.
    /// Same body Term → same code_id; distinct bodies → distinct ids, no collisions.
    code_ids: HashMap<Term, u64>,
    next_code_id: u64,
    /// Numeric literal values registered by the elaborator.
    /// Maps opaque postulate GlobalId → the EvalVal it represents.
    /// Filled by tests/driver that have access to ElabEnv.num_values.
    pub num_values: HashMap<GlobalId, EvalVal>,
    /// Propagates a `CapacityExhausted` error from the intern helper (`44 §2`).
    /// Set when the store's soft limit is hit; callers must not silently drop it.
    pub capacity_error: Option<(u64, u64)>,
}

impl EvalStore {
    pub fn new() -> Self {
        EvalStore {
            k3: Store::new(),
            code_ids: HashMap::new(),
            next_code_id: 1,
            num_values: HashMap::new(),
            capacity_error: None,
        }
    }

    /// Create a store with a soft capacity limit (for AC2 testing).
    pub fn with_capacity_limit(limit: u64) -> Self {
        EvalStore {
            k3: Store::with_capacity_limit(limit),
            code_ids: HashMap::new(),
            next_code_id: 1,
            num_values: HashMap::new(),
            capacity_error: None,
        }
    }

    /// Consume and return any recorded capacity error (`44 §2` loud propagation).
    pub fn take_capacity_error(&mut self) -> Option<(u64, u64)> {
        self.capacity_error.take()
    }

    /// Assign (or retrieve) a unique sequential `code_id` for a closure body.
    /// Same body Term (by structural equality) always maps to the same id.
    fn assign_code_id(&mut self, body: &Term) -> u64 {
        if let Some(&id) = self.code_ids.get(body) {
            return id;
        }
        let id = self.next_code_id;
        self.next_code_id += 1;
        self.code_ids.insert(body.clone(), id);
        id
    }

    /// Forward to the K3 store for slot statistics.
    pub fn stats(&self) -> ken_runtime::StoreStats {
        self.k3.stats()
    }

    /// Forward to the K3 store for value interning (used by tests and helpers).
    pub fn intern(&mut self, v: &RtValue) -> InternResult {
        self.k3.intern(v)
    }
}

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
    /// Immutable byte sequence (Bytes primitive, `38 §1.1`). Treated as a
    /// pseudo-immediate at the eval layer for simplicity; K3 interns as compound.
    Bytes(Vec<u8>),
    /// NFC-normalized UTF-8 string (for encode/decode boundary, `38 §1.4`).
    Str(String),
    BigInt(i128),                         // Int values > i64::MAX or < i64::MIN
    Float(f64),                           // IEEE 754 double
    Float32(f32),                         // IEEE 754 single
    DecimalVal { coeff: i64, exp: i32 },  // exact base-10: coeff × 10^exp

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
    /// `code_id` is assigned by `EvalStore::assign_code_id` — a sequential integer
    /// keyed on the body `Term` by structural equality, so distinct bodies always
    /// produce distinct ids with no collision (not a digest/hash of Debug output).
    Closure {
        body: Rc<Term>,
        captured: Rc<Env>,
        code_id: u64,
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

/// Sentinel `type_id` for `Pair` (Σ-intro) K3 records — disjoint from any
/// `GlobalId` produced by the kernel and from `QUOT_CLASS_TYPE_ID`.
const PAIR_TYPE_ID: u32 = u32::MAX;
/// Sentinel `type_id` for synthetic quotient-class K3 records.
/// Chosen one below `PAIR_TYPE_ID` so a future 2-field synthetic can't collide
/// with `Pair` (both use `Record` in the K3 store, distinguished by `type_id`).
const QUOT_CLASS_TYPE_ID: u32 = u32::MAX - 1;

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
        EvalVal::Closure {
            code_id, captured, ..
        } => {
            let cap_fields: Vec<RtValue> = captured.iter().filter_map(to_rt).collect();
            // Only intern if all captured values are representable.
            if cap_fields.len() == captured.len() {
                Some(RtValue::Closure {
                    code_id: *code_id,
                    captured: cap_fields,
                })
            } else {
                None
            }
        }
        EvalVal::Bytes(b) => Some(RtValue::Bytes(b.clone())),
        EvalVal::Str(s) => Some(RtValue::String(s.clone())),
        _ => None,
    }
}

/// Intern a K3-compatible `EvalVal` and return its slot id.
/// Returns `NULL_SLOT` if the value is not internable (type values, etc.).
/// On `CapacityExhausted`, records the error in `store.capacity_error` (44 §2
/// loud-never-silent) instead of silently collapsing to `NULL_SLOT`.
fn intern(val: &EvalVal, store: &mut EvalStore) -> SlotId {
    let rt = match to_rt(val) {
        Some(r) => r,
        None => return NULL_SLOT,
    };
    if !rt.is_compound() {
        return NULL_SLOT;
    }
    match store.k3.intern(&rt) {
        InternResult::New(s) | InternResult::Hit(s) => s,
        InternResult::CapacityExhausted { limit, current } => {
            store.capacity_error = Some((limit, current));
            NULL_SLOT
        }
    }
}

/// Build a fully-applied `Ctor` value and intern it.
fn make_ctor(id: GlobalId, args: Vec<EvalVal>, store: &mut EvalStore) -> EvalVal {
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
fn make_pair(fst: EvalVal, snd: EvalVal, store: &mut EvalStore) -> EvalVal {
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

/// Build a `Closure` value, assign a unique `code_id` for its body Term, and
/// intern the result in the K3 store.
fn make_closure(body: Rc<Term>, captured: Rc<Env>, store: &mut EvalStore) -> EvalVal {
    let code_id = store.assign_code_id(&body);
    let slot = intern(
        &EvalVal::Closure {
            body: body.clone(),
            captured: captured.clone(),
            code_id,
            slot: NULL_SLOT,
        },
        store,
    );
    EvalVal::Closure {
        body,
        captured,
        code_id,
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
    store: &mut EvalStore,
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

// ── numeric helpers ───────────────────────────────────────────────────────────

fn i128_to_int_val(n: i128) -> EvalVal {
    if n >= i64::MIN as i128 && n <= i64::MAX as i128 {
        EvalVal::Int(n as i64)
    } else {
        EvalVal::BigInt(n)
    }
}

fn eval_to_i128(v: &EvalVal) -> Option<i128> {
    match v {
        EvalVal::Int(n) => Some(*n as i128),
        EvalVal::BigInt(n) => Some(*n),
        _ => None,
    }
}

fn exact_int_binop(a: &EvalVal, b: &EvalVal, op: impl Fn(i128, i128) -> i128) -> EvalVal {
    match (eval_to_i128(a), eval_to_i128(b)) {
        (Some(av), Some(bv)) => i128_to_int_val(op(av, bv)),
        _ => EvalVal::Neutral,
    }
}

fn fixed_binop_i8(a: &EvalVal, b: &EvalVal, op: fn(i8, i8) -> i8) -> EvalVal {
    match (a, b) {
        (EvalVal::Int(x), EvalVal::Int(y)) => EvalVal::Int(op(*x as i8, *y as i8) as i64),
        _ => EvalVal::Neutral,
    }
}

fn fixed_binop_i16(a: &EvalVal, b: &EvalVal, op: fn(i16, i16) -> i16) -> EvalVal {
    match (a, b) {
        (EvalVal::Int(x), EvalVal::Int(y)) => EvalVal::Int(op(*x as i16, *y as i16) as i64),
        _ => EvalVal::Neutral,
    }
}

fn fixed_binop_i32(a: &EvalVal, b: &EvalVal, op: fn(i32, i32) -> i32) -> EvalVal {
    match (a, b) {
        (EvalVal::Int(x), EvalVal::Int(y)) => EvalVal::Int(op(*x as i32, *y as i32) as i64),
        _ => EvalVal::Neutral,
    }
}

fn fixed_binop_i64(a: &EvalVal, b: &EvalVal, op: fn(i64, i64) -> i64) -> EvalVal {
    match (a, b) {
        (EvalVal::Int(x), EvalVal::Int(y)) => EvalVal::Int(op(*x, *y)),
        _ => EvalVal::Neutral,
    }
}

fn fixed_binop_u8(a: &EvalVal, b: &EvalVal, op: fn(u8, u8) -> u8) -> EvalVal {
    match (a, b) {
        (EvalVal::Int(x), EvalVal::Int(y)) => EvalVal::Int(op(*x as u8, *y as u8) as i64),
        _ => EvalVal::Neutral,
    }
}

fn fixed_binop_u16(a: &EvalVal, b: &EvalVal, op: fn(u16, u16) -> u16) -> EvalVal {
    match (a, b) {
        (EvalVal::Int(x), EvalVal::Int(y)) => EvalVal::Int(op(*x as u16, *y as u16) as i64),
        _ => EvalVal::Neutral,
    }
}

fn fixed_binop_u32(a: &EvalVal, b: &EvalVal, op: fn(u32, u32) -> u32) -> EvalVal {
    match (a, b) {
        (EvalVal::Int(x), EvalVal::Int(y)) => EvalVal::Int(op(*x as u32, *y as u32) as i64),
        _ => EvalVal::Neutral,
    }
}

fn fixed_binop_u64(a: &EvalVal, b: &EvalVal, op: fn(u64, u64) -> u64) -> EvalVal {
    match (a, b) {
        (EvalVal::Int(x), EvalVal::Int(y)) => {
            let r = op(*x as u64, *y as u64) as i128;
            i128_to_int_val(r)
        }
        _ => EvalVal::Neutral,
    }
}

fn add_decimal(ca: i64, ea: i32, cb: i64, eb: i32) -> EvalVal {
    if ea == eb {
        EvalVal::DecimalVal {
            coeff: ca.saturating_add(cb),
            exp: ea,
        }
    } else if ea < eb {
        let shift = (eb - ea).min(18) as u32;
        let factor = 10i64.saturating_pow(shift);
        EvalVal::DecimalVal {
            coeff: ca.saturating_add(cb.saturating_mul(factor)),
            exp: ea,
        }
    } else {
        let shift = (ea - eb).min(18) as u32;
        let factor = 10i64.saturating_pow(shift);
        EvalVal::DecimalVal {
            coeff: ca.saturating_mul(factor).saturating_add(cb),
            exp: eb,
        }
    }
}

fn decimal_eq(ca: i64, ea: i32, cb: i64, eb: i32) -> bool {
    if ea == eb {
        ca == cb
    } else if ea < eb {
        let shift = (eb - ea).min(18) as u32;
        let factor = 10i64.saturating_pow(shift);
        ca == cb.saturating_mul(factor)
    } else {
        let shift = (ea - eb).min(18) as u32;
        let factor = 10i64.saturating_pow(shift);
        ca.saturating_mul(factor) == cb
    }
}

/// Structural equality on `EvalVal` for equality-testing contexts (`L1 §4`).
pub fn eval_vals_eq(a: &EvalVal, b: &EvalVal) -> bool {
    match (a, b) {
        (EvalVal::Bool(x), EvalVal::Bool(y)) => x == y,
        (EvalVal::Int(x), EvalVal::Int(y)) => x == y,
        (EvalVal::BigInt(x), EvalVal::BigInt(y)) => x == y,
        (EvalVal::Int(x), EvalVal::BigInt(y)) => (*x as i128) == *y,
        (EvalVal::BigInt(x), EvalVal::Int(y)) => *x == (*y as i128),
        (EvalVal::Float(x), EvalVal::Float(y)) => x == y,
        (EvalVal::Float32(x), EvalVal::Float32(y)) => x == y,
        (EvalVal::DecimalVal { coeff: ca, exp: ea }, EvalVal::DecimalVal { coeff: cb, exp: eb }) => {
            decimal_eq(*ca, *ea, *cb, *eb)
        }
        _ => false,
    }
}

/// Primitive reduction for registered operations (`42 §3.3`, `14 §5`).
///
/// Covers `L1` numeric tower: Int (i128-exact), fixed-width, Decimal, Float,
/// Float32, Bool, plus legacy `add`/`sub`/`mul` (wrapping i64).
/// `L6` Bytes ops and encode/decode (`38 §1.2`, `38 §1.4`) are also grounded.
/// Division and fault-triggering operations are out of scope (`43 §2.2`).
/// Exposed `pub` for conformance tests in `ken-elaborator`.
pub fn prim_reduce(symbol: &str, args: &[EvalVal]) -> EvalVal {
    // Unknown operand: propagate strictly.
    if args.iter().any(|a| matches!(a, EvalVal::Unknown)) {
        return EvalVal::Unknown;
    }
    // Neutral operand: stuck.
    if args.iter().any(|a| matches!(a, EvalVal::Neutral)) {
        return EvalVal::Neutral;
    }

    match (symbol, args) {
        // ---- Int (arbitrary-precision, i128 exact) ----
        ("add_int", [a, b]) => exact_int_binop(a, b, |x, y| x + y),
        ("sub_int", [a, b]) => exact_int_binop(a, b, |x, y| x - y),
        ("mul_int", [a, b]) => exact_int_binop(a, b, |x, y| x * y),
        ("eq_int", [a, b]) => match (eval_to_i128(a), eval_to_i128(b)) {
            (Some(av), Some(bv)) => EvalVal::Bool(av == bv),
            _ => EvalVal::Neutral,
        },

        // ---- Fixed-width (wrapping, obligation-generating) ----
        ("add_int8",  [a, b]) => fixed_binop_i8(a, b, i8::wrapping_add),
        ("sub_int8",  [a, b]) => fixed_binop_i8(a, b, i8::wrapping_sub),
        ("mul_int8",  [a, b]) => fixed_binop_i8(a, b, i8::wrapping_mul),
        ("add_int16", [a, b]) => fixed_binop_i16(a, b, i16::wrapping_add),
        ("sub_int16", [a, b]) => fixed_binop_i16(a, b, i16::wrapping_sub),
        ("mul_int16", [a, b]) => fixed_binop_i16(a, b, i16::wrapping_mul),
        ("add_int32", [a, b]) => fixed_binop_i32(a, b, i32::wrapping_add),
        ("sub_int32", [a, b]) => fixed_binop_i32(a, b, i32::wrapping_sub),
        ("mul_int32", [a, b]) => fixed_binop_i32(a, b, i32::wrapping_mul),
        ("add_int64", [a, b]) => fixed_binop_i64(a, b, i64::wrapping_add),
        ("sub_int64", [a, b]) => fixed_binop_i64(a, b, i64::wrapping_sub),
        ("mul_int64", [a, b]) => fixed_binop_i64(a, b, i64::wrapping_mul),
        ("add_uint8",  [a, b]) => fixed_binop_u8(a, b, u8::wrapping_add),
        ("add_uint16", [a, b]) => fixed_binop_u16(a, b, u16::wrapping_add),
        ("add_uint32", [a, b]) => fixed_binop_u32(a, b, u32::wrapping_add),
        ("add_uint64", [a, b]) => fixed_binop_u64(a, b, u64::wrapping_add),

        // ---- Wrapping variants (explicit `+%`) ----
        ("wrapping_add_int8",  [a, b]) => fixed_binop_i8(a, b, i8::wrapping_add),
        ("wrapping_sub_int8",  [a, b]) => fixed_binop_i8(a, b, i8::wrapping_sub),
        ("wrapping_mul_int8",  [a, b]) => fixed_binop_i8(a, b, i8::wrapping_mul),
        ("wrapping_add_int16", [a, b]) => fixed_binop_i16(a, b, i16::wrapping_add),
        ("wrapping_add_int32", [a, b]) => fixed_binop_i32(a, b, i32::wrapping_add),
        ("wrapping_sub_int32", [a, b]) => fixed_binop_i32(a, b, i32::wrapping_sub),
        ("wrapping_mul_int32", [a, b]) => fixed_binop_i32(a, b, i32::wrapping_mul),
        ("wrapping_add_int64", [a, b]) => fixed_binop_i64(a, b, i64::wrapping_add),
        ("wrapping_add_uint8", [a, b]) => fixed_binop_u8(a, b, u8::wrapping_add),
        ("wrapping_add_uint16", [a, b]) => fixed_binop_u16(a, b, u16::wrapping_add),
        ("wrapping_add_uint32", [a, b]) => fixed_binop_u32(a, b, u32::wrapping_add),
        ("wrapping_add_uint64", [a, b]) => fixed_binop_u64(a, b, u64::wrapping_add),

        // ---- Decimal (exact base-10) ----
        ("add_decimal", [EvalVal::DecimalVal { coeff: ca, exp: ea }, EvalVal::DecimalVal { coeff: cb, exp: eb }]) => {
            add_decimal(*ca, *ea, *cb, *eb)
        }
        ("sub_decimal", [EvalVal::DecimalVal { coeff: ca, exp: ea }, EvalVal::DecimalVal { coeff: cb, exp: eb }]) => {
            add_decimal(*ca, *ea, -*cb, *eb)
        }
        ("mul_decimal", [EvalVal::DecimalVal { coeff: ca, exp: ea }, EvalVal::DecimalVal { coeff: cb, exp: eb }]) => {
            EvalVal::DecimalVal { coeff: ca.saturating_mul(*cb), exp: ea + eb }
        }
        ("eq_decimal", [EvalVal::DecimalVal { coeff: ca, exp: ea }, EvalVal::DecimalVal { coeff: cb, exp: eb }]) => {
            EvalVal::Bool(decimal_eq(*ca, *ea, *cb, *eb))
        }

        // ---- Float (IEEE 754 f64) ----
        ("add_float", [EvalVal::Float(a), EvalVal::Float(b)]) => EvalVal::Float(a + b),
        ("sub_float", [EvalVal::Float(a), EvalVal::Float(b)]) => EvalVal::Float(a - b),
        ("mul_float", [EvalVal::Float(a), EvalVal::Float(b)]) => EvalVal::Float(a * b),
        ("div_float", [EvalVal::Float(a), EvalVal::Float(b)]) => EvalVal::Float(a / b),
        ("eq_float",  [EvalVal::Float(a), EvalVal::Float(b)]) => EvalVal::Bool(a == b),

        // ---- Float32 (IEEE 754 f32) ----
        ("add_float32", [EvalVal::Float32(a), EvalVal::Float32(b)]) => EvalVal::Float32(a + b),
        ("eq_float32",  [EvalVal::Float32(a), EvalVal::Float32(b)]) => EvalVal::Bool(a == b),

        // ---- Bool ----
        ("not_bool", [EvalVal::Bool(b)]) => EvalVal::Bool(!b),
        ("and_bool", [EvalVal::Bool(a), EvalVal::Bool(b)]) => EvalVal::Bool(*a && *b),
        ("or_bool",  [EvalVal::Bool(a), EvalVal::Bool(b)]) => EvalVal::Bool(*a || *b),

        // ---- Legacy (existing, wrapping i64) ----
        ("add", [EvalVal::Int(a), EvalVal::Int(b)]) => EvalVal::Int(a.wrapping_add(*b)),
        ("sub", [EvalVal::Int(a), EvalVal::Int(b)]) => EvalVal::Int(a.wrapping_sub(*b)),
        ("mul", [EvalVal::Int(a), EvalVal::Int(b)]) => EvalVal::Int(a.wrapping_mul(*b)),

        // ── Bytes primitive ops (`38 §1.2`) ──────────────────────────────────
        ("bytes_length", [EvalVal::Bytes(b)]) => EvalVal::Int(b.len() as i64),

        // `at b i` — in-bounds: byte as Int; OOB: Neutral (no silent OOB read).
        ("bytes_at", [EvalVal::Bytes(b), EvalVal::Int(i)]) => {
            let idx = *i;
            if idx >= 0 && (idx as usize) < b.len() {
                EvalVal::Int(b[idx as usize] as i64)
            } else {
                EvalVal::Neutral
            }
        }

        // `slice b start len` — in-bounds: sub-slice as Bytes; OOB: Neutral.
        ("bytes_slice", [EvalVal::Bytes(b), EvalVal::Int(start), EvalVal::Int(len)]) => {
            let s = *start;
            let l = *len;
            if s >= 0 && l >= 0 {
                let s = s as usize;
                let l = l as usize;
                if s <= b.len() && l <= b.len() - s {
                    EvalVal::Bytes(b[s..s + l].to_vec())
                } else {
                    EvalVal::Neutral
                }
            } else {
                EvalVal::Neutral
            }
        }

        ("bytes_concat", [EvalVal::Bytes(a), EvalVal::Bytes(b)]) => {
            let mut out = a.clone();
            out.extend_from_slice(b);
            EvalVal::Bytes(out)
        }

        // ── String ↔ Bytes encode/decode (`38 §1.4`) ─────────────────────────
        // encode: total — String is always valid UTF-8 at construction.
        ("bytes_encode", [EvalVal::Str(s)]) => EvalVal::Bytes(s.as_bytes().to_vec()),

        // decode: partial — Neutral on invalid UTF-8 (represents Err(_)).
        ("bytes_decode", [EvalVal::Bytes(b)]) => match std::str::from_utf8(b) {
            Ok(s) => EvalVal::Str(s.to_string()),
            Err(_) => EvalVal::Neutral,
        },

        // ── L3a String surface ops (`37 §2`) ───────────────────────────────
        // `byte_length s` — the stored UTF-8 byte count (`37 §2.2`). Real now
        // (NFC-independent: a CJK/non-combining witness differs from char count
        // regardless of normalization).
        ("byte_length", [EvalVal::Str(s)]) => EvalVal::Int(s.len() as i64),
        // `char_length s` — the Unicode scalar-value count (`37 §2.2`).
        ("char_length", [EvalVal::Str(s)]) => EvalVal::Int(s.chars().count() as i64),
        // `string_to_list_char` / `list_char_to_string` are total-typed
        // (`37 §2.3`) but do not reduce at the interp layer in L3a (building a
        // `List Char` needs the ctor ids; the round-trip is L6's home). They
        // stay Neutral (stuck) — totality is asserted at the type level.
        ("string_to_list_char", [EvalVal::Str(_)]) => EvalVal::Neutral,
        ("list_char_to_string", [EvalVal::Ctor { .. }]) => EvalVal::Neutral,

        // Partial or unrecognised primitive: neutral (stuck on non-literals).
        _ => EvalVal::Neutral,
    }
}

// ── eval / apply ─────────────────────────────────────────────────────────────

/// `eval ρ t` — evaluate a core term in environment `ρ` (`42 §3.2`).
pub fn eval(env: &[EvalVal], term: &Term, globals: &GlobalEnv, store: &mut EvalStore) -> EvalVal {
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
        Term::Const { id, .. } => {
            // Numeric literal side table: opaque postulates representing literal values.
            if let Some(v) = store.num_values.get(id) {
                return v.clone();
            }
            match globals.lookup(*id) {
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
            }
        }

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
            // Represent [a] as a 1-arg constructor with a synthetic type_id.
            // Uses QUOT_CLASS_TYPE_ID (u32::MAX - 1) which is disjoint from both
            // real GlobalIds and PAIR_TYPE_ID (u32::MAX).
            EvalVal::Ctor {
                id: GlobalId(QUOT_CLASS_TYPE_ID),
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
pub fn apply(f: EvalVal, u: EvalVal, globals: &GlobalEnv, store: &mut EvalStore) -> EvalVal {
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

// ── effect driver (`42 §6`) ───────────────────────────────────────────────────

/// Constructor IDs for the `ITree` inductive, needed by `drive_h`.
///
/// Obtain these from the `GlobalEnv` after registering `ITree` via
/// `declare_inductive`. `params_len` is the number of *inductive* params
/// (type-level indices, e.g. `ρ` and `R`); ctor-specific args start at
/// `args[params_len]` in the `EvalVal::Ctor`.
pub struct ITreeIds {
    /// `GlobalId` of the `Ret` constructor (k = 0).
    pub ret_id: GlobalId,
    /// `GlobalId` of the `Vis` constructor (k = 1).
    pub vis_id: GlobalId,
    /// Number of inductive params (`ITree ρ R` has 2; a simplified test ITree
    /// may have 0). Ctor-specific args start at this offset in `EvalVal::Ctor.args`.
    pub params_len: usize,
}

/// `drive_H t = case whnf t of Ret r → r | Vis e k → drive_H (apply k (H e)) | unknown → unknown`
///
/// The effect driver (`42 §6.2`): `tree` is a fully-evaluated `ITree` value
/// (produced by `eval`; the denotation `⟦e⟧` from `36 §2.4` is a pure core
/// term `eval` already handles). The loop terminates because the `ITree` is
/// **finite** (K1.5 structural descent; no coinduction).
///
/// `handler` is the `36 §7.2` real-world-handler hook — **parametric** so
/// conformance can supply a deterministic mock while production supplies real
/// syscalls. It is `FnMut` because real I/O has side effects.
///
/// Exhaustiveness (`42 §6.5`, EFF7): the caller's `handler` must cover every
/// op-tag the open row admits — no catch-all `_ → skip`. A missing rule is a
/// build error in the handler, never a silent skip here.
pub fn drive_h<H>(
    mut tree: EvalVal,
    handler: &mut H,
    ids: &ITreeIds,
    globals: &GlobalEnv,
    store: &mut EvalStore,
) -> EvalVal
where
    H: FnMut(EvalVal) -> EvalVal,
{
    let m = ids.params_len;
    loop {
        let next = match tree {
            // §6.7: an open hole in the tree is strict — propagate unknown.
            EvalVal::Unknown => return EvalVal::Unknown,
            EvalVal::Ctor { id, args, .. } => {
                if id == ids.ret_id {
                    // Ret r → finished; return the result.
                    return args.get(m).cloned().unwrap_or(EvalVal::Unknown);
                } else if id == ids.vis_id {
                    // Vis e k → perform+observe (H e), resume (apply k resp), loop.
                    let e = args[m].clone();
                    let k = args[m + 1].clone();
                    let resp = handler(e);
                    apply(k, resp, globals, store)
                } else {
                    // Unrecognised constructor — stuck (should not happen for
                    // well-typed programs; closed ground ITree is either Ret or Vis).
                    return EvalVal::Neutral;
                }
            }
            // Any other value (closure, type-former, neutral) — stuck.
            _ => return EvalVal::Neutral,
        };
        tree = next;
    }
}

/// IDs for the `Console` effect driver (`42 §6.3`, `36 §2.1`).
///
/// Obtain by looking up the ITree/Console.Op inductives in the `GlobalEnv`
/// after Language registers them in `ElabEnv::new()`.
/// `params_len` is the number of ITree *type* params (2 for `ITree E R`;
/// 0 for the simplified 0-param test ITree).
pub struct ConsoleIds {
    /// `GlobalId` of the `ITree` inductive (for documentation; not used in the loop).
    pub itree_id: GlobalId,
    /// `GlobalId` of the `Ret` constructor (k = 0).
    pub ret_id: GlobalId,
    /// `GlobalId` of the `Vis` constructor (k = 1).
    pub vis_id: GlobalId,
    /// `GlobalId` of `Console.Op::Write` (k = 0, carries a `String` arg).
    pub write_id: GlobalId,
    /// `GlobalId` of the `Unit` constructor (response to `Write`).
    pub unit_id: GlobalId,
    /// Number of ITree type-level params. Ctor-specific args start at this offset.
    pub params_len: usize,
}

/// Error returned by `run_io` (`42 §6`).
#[derive(Debug)]
pub enum RunIoError {
    /// A `Vis` node carried an op-tag that is not `Console.Write`.
    UnknownEffect(EvalVal),
    /// The tree evaluated to `Unknown` (open hole, `42 §6.7`).
    UnknownTree,
    /// The tree is not an ITree `Ret`/`Vis` value.
    NotAnIOTree(EvalVal),
}

/// Console-effect driver (`42 §6.2`, `§6.3`): runs an `ITree Console Unit`
/// value to completion, printing each `Write s` op to stdout.
///
/// Dispatches exhaustively over `Console.Op` — no catch-all (`42 §6.5`): the
/// only op-tag is `Write`, and any other tag is `Err(UnknownEffect)`.
///
/// `ids.params_len` must equal the number of type-level params on `ITree`
/// (2 for the production `ITree Console Unit`; 0 for the test ITree).
pub fn run_io(
    mut tree: EvalVal,
    ids: &ConsoleIds,
    globals: &GlobalEnv,
    store: &mut EvalStore,
) -> Result<EvalVal, RunIoError> {
    let m = ids.params_len;
    loop {
        let next = match tree {
            EvalVal::Unknown => return Err(RunIoError::UnknownTree),
            EvalVal::Ctor { id, args, .. } => {
                if id == ids.ret_id {
                    // Ret r → done
                    return Ok(args.get(m).cloned().unwrap_or(EvalVal::Unknown));
                } else if id == ids.vis_id {
                    let op = args[m].clone();
                    let k = args[m + 1].clone();
                    // Dispatch on the Console.Op — exhaustive, no catch-all (42 §6.5).
                    let resp = match op {
                        EvalVal::Ctor { id: op_id, args: op_args, .. }
                            if op_id == ids.write_id =>
                        {
                            let maybe_s = match op_args.get(0) {
                                Some(EvalVal::Str(s)) => Some(s.clone()),
                                _ => None,
                            };
                            match maybe_s {
                                Some(s) => {
                                    println!("{}", s);
                                    make_ctor(ids.unit_id, vec![], store)
                                }
                                None => {
                                    return Err(RunIoError::UnknownEffect(EvalVal::Ctor {
                                        id: op_id,
                                        args: op_args,
                                        slot: NULL_SLOT,
                                    }))
                                }
                            }
                        }
                        other => return Err(RunIoError::UnknownEffect(other)),
                    };
                    apply(k, resp, globals, store)
                } else {
                    // Unrecognised ITree constructor
                    return Err(RunIoError::NotAnIOTree(EvalVal::Ctor {
                        id,
                        args,
                        slot: NULL_SLOT,
                    }));
                }
            }
            other => return Err(RunIoError::NotAnIOTree(other)),
        };
        tree = next;
    }
}

/// Instrumented variant of `drive_h` — emits a trace event at each `Vis` firing
/// (`73 §2`, TC2). Pure steps (`Ret`, β, ι) emit nothing.
///
/// `on_event` is called with `(space_id, effect_val, response_val, sequence_pos)`
/// **after** the handler responds (response is available) and **before** the
/// continuation resumes (sequential ordering preserved). The caller interprets
/// `effect_val` and `response_val` — no Ken-side decode.
///
/// **Instrumentation ONLY at the `Vis` site (TC2):** one callback per `Vis`
/// firing; no calls on `Ret` or pure reduction steps. Bounded overhead is
/// structural — the callback is at exactly the same location as `drive_h`'s
/// Vis branch.
///
/// **One-way (TC5):** `on_event` is a write-only side-channel (`FnMut`).
/// Its return type is `()` — there is no path from `on_event`'s output to the
/// ITree result or to Ken's epistemic status. Emitted events are witnesses, not
/// claims; a `delegated` T stays `delegated` regardless of what `on_event`
/// records.
pub fn drive_h_instrumented<H, S>(
    mut tree: EvalVal,
    handler: &mut H,
    ids: &ITreeIds,
    globals: &GlobalEnv,
    store: &mut EvalStore,
    space_id: &str,
    seq: &mut u64,
    on_event: &mut S,
) -> EvalVal
where
    H: FnMut(EvalVal) -> EvalVal,
    S: FnMut(String, EvalVal, EvalVal, u64), // (space_id, effect_val, resp_val, seq_pos)
{
    let m = ids.params_len;
    loop {
        let next = match tree {
            EvalVal::Unknown => return EvalVal::Unknown,
            EvalVal::Ctor { id, ref args, .. } => {
                if id == ids.ret_id {
                    return args.get(m).cloned().unwrap_or(EvalVal::Unknown);
                } else if id == ids.vis_id {
                    let e = args[m].clone();
                    let k = args[m + 1].clone();
                    let resp = handler(e.clone());
                    // INSTRUMENTATION POINT: one event per Vis firing (TC2).
                    // Emitted after handler responds (response is available).
                    let pos = *seq;
                    *seq += 1;
                    on_event(space_id.to_string(), e, resp.clone(), pos);
                    apply(k, resp, globals, store)
                } else {
                    return EvalVal::Neutral;
                }
            }
            _ => return EvalVal::Neutral,
        };
        tree = next;
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
        "and_bool" | "or_bool" => 2,
        "add_int" | "sub_int" | "mul_int" | "eq_int" => 2,
        "add_decimal" | "sub_decimal" | "mul_decimal" | "eq_decimal" => 2,
        "add_float" | "sub_float" | "mul_float" | "div_float" | "eq_float" => 2,
        "add_float32" | "eq_float32" => 2,
        s if s.starts_with("add_int") || s.starts_with("sub_int") || s.starts_with("mul_int") => 2,
        s if s.starts_with("add_uint") || s.starts_with("sub_uint") || s.starts_with("mul_uint") => 2,
        s if s.starts_with("wrapping_") => 2,
        // ── Bytes ops (`38 §1.2`, `38 §1.4`) ─────────────────────────────
        "bytes_length" | "bytes_encode" | "bytes_decode" => 1,
        "bytes_at" | "bytes_concat" => 2,
        "bytes_slice" => 3,
        // ── L3a String surface ops (`37 §2`) ──────────────────────────────
        "byte_length" | "char_length" | "string_to_list_char" | "list_char_to_string" => 1,
        _ => 1,
    }
}

// ── capacity conformance tests ────────────────────────────────────────────────

#[cfg(test)]
mod capacity_tests {
    use super::*;
    use ken_kernel::term::GlobalId;

    // conformance: runtime/capacity/loud-at-limit-raises-not-silent (interp layer)
    // The store's CapacityExhausted must propagate via store.capacity_error —
    // the silent NULL_SLOT collapse is the bug this guards against (44 §2).
    #[test]
    fn interp_loud_capacity_error_not_silent() {
        let mut store = EvalStore::with_capacity_limit(2);
        // Two distinct compound values fill the store.
        let v1 = EvalVal::Ctor {
            id: GlobalId(1),
            args: Rc::new(vec![EvalVal::Int(1)]),
            slot: NULL_SLOT,
        };
        let v2 = EvalVal::Ctor {
            id: GlobalId(1),
            args: Rc::new(vec![EvalVal::Int(2)]),
            slot: NULL_SLOT,
        };
        intern(&v1, &mut store);
        intern(&v2, &mut store);
        assert!(
            store.capacity_error.is_none(),
            "no error expected before limit"
        );

        // Third distinct value hits the limit.
        let v3 = EvalVal::Ctor {
            id: GlobalId(1),
            args: Rc::new(vec![EvalVal::Int(3)]),
            slot: NULL_SLOT,
        };
        intern(&v3, &mut store);
        let err = store.take_capacity_error();
        assert!(
            err.is_some(),
            "CapacityExhausted must be recorded, not silently dropped (44 §2)"
        );
        let (limit, current) = err.unwrap();
        assert_eq!(limit, 2);
        assert_eq!(current, 2);
    }

    // conformance: runtime/capacity/at-limit-repeat-does-not-trip (interp layer)
    // A repeat value must return Hit (not CapacityExhausted) even at the limit —
    // the dedup path short-circuits before the limit check (44 §2, §6).
    #[test]
    fn interp_at_limit_repeat_does_not_trip() {
        let mut store = EvalStore::with_capacity_limit(2);
        let v1 = EvalVal::Ctor {
            id: GlobalId(1),
            args: Rc::new(vec![EvalVal::Int(1)]),
            slot: NULL_SLOT,
        };
        let v2 = EvalVal::Ctor {
            id: GlobalId(1),
            args: Rc::new(vec![EvalVal::Int(2)]),
            slot: NULL_SLOT,
        };
        intern(&v1, &mut store);
        intern(&v2, &mut store);
        // At limit: re-interning an existing value must NOT trigger capacity error.
        intern(&v1, &mut store);
        assert!(
            store.capacity_error.is_none(),
            "repeat must not trip CapacityExhausted (44 §6 fixed-point partner)"
        );
    }
}
