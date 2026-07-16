//! Reference interpreter: core terms → values (`spec/40-runtime/42-evaluation.md`).
//!
//! Strategy: environment-based CBV with sharing via the K3 content-addressed
//! store. Reduction rules per §1: β, Σ-β, ι, δ, obs (cast/Eq/quotient), prim.
//! Branch laziness: eliminator methods held unevaluated; only the scrutinee-
//! selected method (ι) fires.
//!
//! # EvalVal variants
//! - Scalar immediates (`Bool`, `Int`, `Float`, `Float32`) — not K3-interned
//!   (they are K3 immediates: `RtValue::Bool`/`SmallInt`/…). `Decimal` is
//!   DEMOTE→derived (`18a §5.6.1`) — a `Ctor` over two `Int` fields, not a
//!   scalar immediate of its own.
//! - `BigInt` — an eval-level immediate (arbitrary-precision, `18a §5.2.1`),
//!   but its store image (`Value::BigInt { sign, limbs }`) is a K3-interned
//!   compound (content-addressed like `Ctor`/`Pair`), not an immediate; `to_rt`
//!   bridges the two representations.
//! - Compound data (`Ctor`, `Pair`, `Closure`) — K3-interned, carry a `SlotId`.
//! - Type-former values (`TypeUniverse`, `OmegaUniverse`, `PiTy`, `SigmaTy`,
//!   `IndFormerVal`) — not K3-interned; irreducible at the value layer (G1 scope).
//! - `CtorPending` — accumulates positional args before the constructor saturates.
//! - `Unknown` — open-hole residue (propagates strictly through all positions).
//! - `Neutral` — stuck on an opaque constant or open variable (closed ground
//!   programs never reach this per canonicity).

use std::collections::{BTreeMap, HashMap};
use std::io::{self, IsTerminal, Read, Write};
use std::rc::Rc;

use ken_elaborator::capabilities;
use ken_host::{OpenRequest as HostOpenRequest, PathComponent, RemoveKind, RootedHandle};

pub const INTERPRETER_TARGET_ABI_MANIFEST_HASH: [u8; 32] = ken_host::TARGET_ABI_MANIFEST_HASH;

fn assert_interpreter_target_abi_hash(hash: [u8; 32]) -> io::Result<()> {
    ken_host::assert_target_abi_identity(hash)
        .map_err(|error| io::Error::new(io::ErrorKind::Unsupported, error))
}

#[cfg(test)]
mod target_abi_tests {
    use super::*;

    #[cfg(target_os = "linux")]
    #[test]
    fn mismatch_stops_before_interpreter_host_entry() {
        let mut mismatch = INTERPRETER_TARGET_ABI_MANIFEST_HASH;
        mismatch[0] ^= 1;
        let mut entered = false;
        let result = assert_interpreter_target_abi_hash(mismatch).and_then(|()| {
            entered = true;
            Ok(())
        });
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::Unsupported);
        assert!(
            !entered,
            "mismatched artifact must not enter the host boundary"
        );

        assert_interpreter_target_abi_hash(INTERPRETER_TARGET_ABI_MANIFEST_HASH)
            .expect("matching interpreter artifact proceeds");
    }
}
use ken_kernel::env::{Decl, GlobalEnv, PrimReduction};
use ken_kernel::term::{GlobalId, Level, Term};
use ken_runtime::{InternResult, Sign as RtSign, Store, Value as RtValue};
use num_bigint::{BigInt, BigUint, Sign as NumSign};
use num_traits::ToPrimitive;

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
    /// Generic `List` constructor IDs needed by the `String ↔ List Char`
    /// and `Bytes ↔ List UInt8` primitive views (`37 §2.3/§2.6`). The
    /// historical field/type name is retained for API compatibility; `Nil` and
    /// `Cons` are polymorphic, so the same ids construct both element types.
    pub list_char_ids: Option<ListCharIds>,
}

impl EvalStore {
    pub fn new() -> Self {
        EvalStore {
            k3: Store::new(),
            code_ids: HashMap::new(),
            next_code_id: 1,
            num_values: HashMap::new(),
            capacity_error: None,
            list_char_ids: None,
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
            list_char_ids: None,
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
    BigInt(BigInt), // Int values > i64::MAX or < i64::MIN (arbitrary-precision, `18a §5.2.1`)
    Float(f64),     // IEEE 754 double
    Float32(f32),   // IEEE 754 single
    /// A real, opaque capability token (fs-read-file-lines-flip D3,
    /// Architect ruling `evt_35knjqv2k941h` §D3 — structural self-evidence
    /// over a positional-scalar `EvalVal::Int(level)`). The *sole* producer
    /// reaching the driver is the CLI mint (`ken-cli/src/main.rs::run_file`);
    /// `Cap` is a surface-unconstructible `OpaqueType` postulate, so no
    /// surface term ever synthesizes one. `authorizes` (below) fail-closes
    /// on any other `EvalVal` shape.
    Cap(capabilities::Cap),
    // `Decimal` is DEMOTE→derived (`18a §5.6.1`): a `Ctor{id:mkdecimalpair_id}`
    // value over two `Int`/`BigInt` fields, not a scalar immediate — no
    // `DecimalVal` case here anymore (the native primitive was removed).

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
    /// K1.5 W-style (Π-bound) recursive-position IH, deferred until applied to
    /// its `nb` branch arguments. The kernel's term-level IH for such a
    /// position is `λb̄. elim_D … (a_j b̄)` (`ken-kernel/src/inductive.rs`
    /// `recursive_args`/`iota_reduct`) — a curried function of arity `nb`
    /// with no source `Term` at the value layer (`a_j` is already an
    /// `EvalVal`, e.g. a `Vis` continuation `Closure`), so it can't be built
    /// as an ordinary `Term`-bodied `Closure`. Applying all `nb` branch args
    /// threads them into `rec_field`, then folds the result through
    /// `elim_reduce` — the State-effect-build `run_state`/`elim_ITree` fold
    /// over `Vis` nodes (`36 §4.2`).
    IhClosure {
        rec_field: Rc<EvalVal>,
        fam: GlobalId,
        methods: Rc<[Term]>,
        ih_env: Rc<Env>,
        nb: usize,
        applied: Rc<Vec<EvalVal>>,
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
        EvalVal::BigInt(n) => Some(bigint_to_rt(n)),
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

#[derive(Clone, Copy)]
enum Projection {
    Fst,
    Snd,
}

fn project_value(mut val: EvalVal, path: &[Projection]) -> EvalVal {
    for projection in path {
        val = match (projection, val) {
            (Projection::Fst, EvalVal::Pair { fst, .. }) => (*fst).clone(),
            (Projection::Snd, EvalVal::Pair { snd, .. }) => (*snd).clone(),
            (_, EvalVal::Unknown) => return EvalVal::Unknown,
            _ => return EvalVal::Neutral,
        };
    }
    val
}

fn eval_projection_path(
    env: &[EvalVal],
    term: &Term,
    globals: &GlobalEnv,
    store: &mut EvalStore,
    path: &[Projection],
) -> EvalVal {
    if path.is_empty() {
        return eval(env, term, globals, store);
    }

    match term {
        Term::Ascript(inner, _) => eval_projection_path(env, inner, globals, store, path),
        Term::Proj1(inner) => {
            let mut inner_path = Vec::with_capacity(path.len() + 1);
            inner_path.push(Projection::Fst);
            inner_path.extend_from_slice(path);
            eval_projection_path(env, inner, globals, store, &inner_path)
        }
        Term::Proj2(inner) => {
            let mut inner_path = Vec::with_capacity(path.len() + 1);
            inner_path.push(Projection::Snd);
            inner_path.extend_from_slice(path);
            eval_projection_path(env, inner, globals, store, &inner_path)
        }
        Term::Pair(fst, snd) => {
            let selected = match path[0] {
                Projection::Fst => fst,
                Projection::Snd => snd,
            };
            if path.len() == 1 {
                eval(env, selected, globals, store)
            } else {
                eval_projection_path(env, selected, globals, store, &path[1..])
            }
        }
        Term::Const { id, .. } => {
            if let Some(Decl::Transparent { body, .. }) = globals.lookup(*id) {
                eval_projection_path(&[], body, globals, store, path)
            } else {
                project_value(eval(env, term, globals, store), path)
            }
        }
        _ => project_value(eval(env, term, globals, store), path),
    }
}

fn projection_path_from_var0(term: &Term) -> Option<Vec<Projection>> {
    match term {
        Term::Var(0) => Some(Vec::new()),
        Term::Proj1(inner) => {
            let mut path = projection_path_from_var0(inner)?;
            path.push(Projection::Fst);
            Some(path)
        }
        Term::Proj2(inner) => {
            let mut path = projection_path_from_var0(inner)?;
            path.push(Projection::Snd);
            Some(path)
        }
        Term::Ascript(inner, _) => projection_path_from_var0(inner),
        _ => None,
    }
}

fn projection_accessor_path(term: &Term, globals: &GlobalEnv) -> Option<Vec<Projection>> {
    match term {
        Term::Lam(_, body) => {
            let path = projection_path_from_var0(body)?;
            if path.is_empty() {
                None
            } else {
                Some(path)
            }
        }
        Term::Ascript(inner, _) => projection_accessor_path(inner, globals),
        Term::Const { id, .. } => match globals.lookup(*id) {
            Some(Decl::Transparent { body, .. }) => projection_accessor_path(body, globals),
            _ => None,
        },
        _ => None,
    }
}

fn eval_projection_accessor_app(
    env: &[EvalVal],
    fun: &Term,
    arg: &Term,
    globals: &GlobalEnv,
    store: &mut EvalStore,
) -> Option<EvalVal> {
    let path = projection_accessor_path(fun, globals)?;
    Some(eval_projection_path(env, arg, globals, store, &path))
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

/// Is constructor arg type `arg_ty` a recursive position for family `fam`,
/// direct **or** Π-bound (K1.5 W-style, `ken-kernel/src/inductive.rs`
/// `recursive_args`)? Peels leading `Term::Pi` domains (the branching
/// telescope `B₁ → … → B_nb → …`) and checks whether the remaining codomain
/// is headed by `fam`. Returns the branching arity `nb` (`0` = direct
/// occurrence, unchanged from pre-K1.5; `≥1` = W-style, e.g. `ITree`'s
/// `Vis : E → (Resp e → ITree r) → ITree r`, `nb = 1`).
fn recursive_arg_arity(arg_ty: &Term, fam: GlobalId) -> Option<usize> {
    let mut nb = 0;
    let mut t = arg_ty;
    loop {
        if is_recursive_arg(t, fam) {
            return Some(nb);
        }
        match t {
            Term::Pi(_, cod) => {
                t = cod;
                nb += 1;
            }
            _ => return None,
        }
    }
}

/// Peel `n` outer `Term::Lam` binders, returning the remaining inner term.
/// Returns `t` unchanged if fewer than `n` `Lam`s are found (should not
/// happen for elaborator-produced eliminator methods — the pattern-matrix
/// compiler always wraps ctor-field + IH columns as a fixed run of `Lam`s,
/// `build_ctor_buckets`/`compile_match_matrix`); the caller treats an
/// unexpected shape by falling back to always-compute, never a soundness gap.
fn peel_lams(mut t: &Term, mut n: usize) -> &Term {
    while n > 0 {
        match t {
            Term::Lam(_, body) => {
                t = body;
                n -= 1;
            }
            _ => break,
        }
    }
    t
}

/// Is `Var(target)` free in `t` (de Bruijn, `target` counted from `t`'s own
/// root)? Mirrors `ken_kernel::subst::shift`'s binder bookkeeping exactly —
/// same one-per-binder cutoff increments — but checks instead of rewriting.
/// Used only to decide whether a recursive-position IH is dead in the
/// selected method's body (`elim_reduce`'s eager-IH fix, RTP1 (B')); a
/// false-negative here would be a soundness/perf-regression risk, so every
/// binder-introducing variant increments `target`, matching `shift` variant
/// for variant.
fn term_var_free(t: &Term, target: usize) -> bool {
    match t {
        Term::Var(i) => *i == target,
        Term::Pi(a, b) | Term::Sigma(a, b) => {
            term_var_free(a, target) || term_var_free(b, target + 1)
        }
        Term::Lam(a, body) => term_var_free(a, target) || term_var_free(body, target + 1),
        Term::Let { ty, val, body } => {
            term_var_free(ty, target)
                || term_var_free(val, target)
                || term_var_free(body, target + 1)
        }
        Term::App(f, a) => term_var_free(f, target) || term_var_free(a, target),
        Term::Pair(a, b) => term_var_free(a, target) || term_var_free(b, target),
        Term::Proj1(p) | Term::Proj2(p) => term_var_free(p, target),
        Term::Ascript(t2, a) => term_var_free(t2, target) || term_var_free(a, target),
        Term::Eq(a, l, r) => {
            term_var_free(a, target) || term_var_free(l, target) || term_var_free(r, target)
        }
        Term::Refl(t2) => term_var_free(t2, target),
        Term::Cast(a, b, e, t2) => {
            term_var_free(a, target)
                || term_var_free(b, target)
                || term_var_free(e, target)
                || term_var_free(t2, target)
        }
        Term::J(m, d, e) => {
            term_var_free(m, target) || term_var_free(d, target) || term_var_free(e, target)
        }
        Term::Quot(a, r) => term_var_free(a, target) || term_var_free(r, target),
        Term::QuotClass(t2) => term_var_free(t2, target),
        Term::Trunc(a) => term_var_free(a, target),
        Term::TruncProj(t2) => term_var_free(t2, target),
        Term::QuotElim {
            motive,
            method,
            respect,
            scrut,
        } => {
            term_var_free(motive, target)
                || term_var_free(method, target)
                || term_var_free(respect, target)
                || term_var_free(scrut, target)
        }
        Term::Elim {
            params,
            motive,
            methods,
            indices,
            scrut,
            ..
        } => {
            params.iter().any(|p| term_var_free(p, target))
                || term_var_free(motive, target)
                || methods.iter().any(|m| term_var_free(m, target))
                || indices.iter().any(|i| term_var_free(i, target))
                || term_var_free(scrut, target)
        }
        Term::Absurd(motive, proof) => {
            term_var_free(motive, target) || term_var_free(proof, target)
        }
        Term::Type(_)
        | Term::Omega(_)
        | Term::Const { .. }
        | Term::IndFormer { .. }
        | Term::Constructor { .. }
        | Term::IntLit(_) => false,
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
        // `eq_int`/`leq_int`/`not_bool`/`and_bool`/`or_bool`/`eq_float(32)`
        // return the interpreter's native `EvalVal::Bool` immediate, but
        // `Bool` is a REAL inductive (`data Bool = True | False`) and
        // `Term::Elim` dispatches by constructor INDEX — a bare `True`/
        // `False` literal instead reduces to `EvalVal::Ctor{id:true/false_id}`
        // (`make_ctor`), which the arm below already handles. `True`/`False`
        // are declared in that exact index order (0/1) with arity 0 (no
        // constructor-specific args), so the selected method needs no
        // argument application — this is the zero-arg `Ctor` case, just
        // reached from the other value representation of the same `Bool`.
        // Without this arm, any `match` scrutinizing a *computed* `Bool`
        // (as opposed to a literal `True`/`False`) falls through to the
        // catch-all below and gets stuck at `Neutral` — the bug this fixes.
        EvalVal::Bool(b) => {
            let k = if b { 0 } else { 1 };
            eval(env, &methods[k], globals, store)
        }
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
            // Each entry pairs the field index with its branching arity `nb`
            // (`0` = direct, `≥1` = K1.5 W-style/Π-bound — `recursive_arg_arity`).
            let rec_positions: Vec<(usize, usize)> = ctor_decl
                .args
                .iter()
                .enumerate()
                .filter_map(|(i, ty)| recursive_arg_arity(ty, fam).map(|nb| (i, nb)))
                .collect();

            // Evaluate ONLY the selected method (the others are never touched).
            let mut mval = eval(env, &methods[k], globals, store);

            // Apply all ctor-specific args left-to-right.
            for arg in ctor_specific {
                mval = apply(mval, arg.clone(), globals, store);
            }

            // Apply IH values for recursive positions (in order). RTP1 (B'):
            // the pattern-matrix compiler (`build_ctor_buckets`) always wraps
            // one `Lam` per recursive position, whether or not the arm body
            // actually references it (`ColKind::Ih` columns are unconditional
            // per constructor, not per-use) — a plain self-recursive `view`
            // compiles its own recursion as an ordinary `Term::Const`
            // δ-unfold, so this IH binder is very often DEAD. Eagerly
            // computing it here was a redundant full recursive `elim_reduce`
            // walk whose result `apply`'s catch-all silently drops when
            // unused — the confirmed root cause of the exponential blowup
            // (D1: `single`/`doubleLet` both 2.00×/+depth with nothing to
            // share; `natGcd`'s own large-fuel recursion pays the same tax
            // independent of any downstream consumer).
            //
            // Fix: a pure static free-variable check on the UNEVALUATED
            // method term — dead-code elimination, not laziness/memoisation.
            // `ih_region` is what remains of `methods[k]` after peeling the
            // `ctor_specific.len()` field-lambdas already applied above;
            // `body_only` is what remains after further peeling all
            // `rec_positions.len()` IH-lambdas. Bound-variable index for the
            // `j`-th IH (0 = outermost) inside `body_only` is
            // `rec_positions.len() - 1 - j` (standard de Bruijn: the
            // innermost binder is `Var(0)`). A used slot still costs exactly
            // one `elim_reduce` call, applied once — unchanged from before
            // (this is dead-code skip, not memoisation; nothing here needed
            // sharing, per D1's `doubleLet` finding).
            let ih_region = peel_lams(&methods[k], ctor_specific.len());
            let body_only = peel_lams(ih_region, rec_positions.len());
            let p = rec_positions.len();
            for (j, (rec_pos, nb)) in rec_positions.iter().enumerate() {
                let used = term_var_free(body_only, p - 1 - j);
                let ih = if !used {
                    // Provably dead — any value is behaviorally inert here;
                    // skip the recursive walk entirely (RTP1 (B') fix).
                    EvalVal::Unknown
                } else {
                    let rec_arg = ctor_specific[*rec_pos].clone();
                    if *nb == 0 {
                        // Direct recursive position — unchanged from before.
                        elim_reduce(env, fam, methods, rec_arg, globals, store)
                    } else {
                        // K1.5 W-style: the IH is `λb̄. elim_D … (a_j b̄)`
                        // (`iota_reduct`) — cannot be computed until the `nb`
                        // branch args are known, so defer as an `IhClosure`
                        // rather than eagerly folding here.
                        EvalVal::IhClosure {
                            rec_field: Rc::new(rec_arg),
                            fam,
                            methods: Rc::from(methods.to_vec().into_boxed_slice()),
                            ih_env: Rc::new(env.to_vec()),
                            nb: *nb,
                            applied: Rc::new(Vec::new()),
                        }
                    }
                };
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

/// Narrow a `BigInt` arithmetic result to the `Int` fast-path representation
/// when it fits in `i64`; otherwise keep it as `BigInt`. Purely a
/// representation choice — the value entering here is already the exact
/// arbitrary-precision result, so this narrowing never wraps (`18a §5.2.1(1)`).
fn bigint_to_int_val(n: BigInt) -> EvalVal {
    match n.to_i64() {
        Some(i) => EvalVal::Int(i),
        None => EvalVal::BigInt(n),
    }
}

impl From<i128> for EvalVal {
    fn from(n: i128) -> Self {
        bigint_to_int_val(BigInt::from(n))
    }
}

/// Build a `Decimal` value — `Ctor{id:mkdecimalpair_id, args:[coeff, exp]}`
/// (`18a §5.6.1`) — from a `(coeff, exp)` pair. Used by literal-conversion
/// call sites outside this crate (`ken-cli`, elaborator test drivers) that
/// turn an elaborated `NumericLitVal::Decimal` into its `EvalVal`; not
/// interned here (callers intern via the store when needed).
pub fn decimal_value(mkdecimalpair_id: GlobalId, coeff: i64, exp: i32) -> EvalVal {
    EvalVal::Ctor {
        id: mkdecimalpair_id,
        args: Rc::new(vec![EvalVal::Int(coeff), EvalVal::Int(exp as i64)]),
        slot: NULL_SLOT,
    }
}

fn eval_to_bigint(v: &EvalVal) -> Option<BigInt> {
    match v {
        EvalVal::Int(n) => Some(BigInt::from(*n)),
        EvalVal::BigInt(n) => Some(n.clone()),
        _ => None,
    }
}

/// Total, arbitrary-precision binary `Int` op (`add_int`/`sub_int`/`mul_int`,
/// `18a §5.2.1(1)`) — no fixed-width intermediate anywhere on this path; `op`
/// runs entirely over `BigInt`, and the result only narrows to `Int` (never
/// widens/wraps) after being computed exactly.
fn exact_int_binop(a: &EvalVal, b: &EvalVal, op: impl Fn(BigInt, BigInt) -> BigInt) -> EvalVal {
    match (eval_to_bigint(a), eval_to_bigint(b)) {
        (Some(av), Some(bv)) => bigint_to_int_val(op(av, bv)),
        _ => EvalVal::Neutral,
    }
}

/// Convert an evaluator `BigInt` to its store representation
/// (`Value::BigInt { sign, limbs }`) — the forward half of the F1 store
/// round-trip (`18a §5.2.1(3)`). `to_u64_digits` is minimal by construction
/// (no leading-zero limb), and zero's empty digit vector maps to the single
/// canonical zero limb `canonical.rs` expects.
fn bigint_to_rt(n: &BigInt) -> RtValue {
    let (sign, digits) = n.to_u64_digits();
    let limbs = if digits.is_empty() { vec![0] } else { digits };
    let rt_sign = match sign {
        NumSign::Minus => RtSign::Negative,
        NumSign::NoSign | NumSign::Plus => RtSign::NonNegative,
    };
    RtValue::BigInt {
        sign: rt_sign,
        limbs,
    }
}

/// Convert a stored `Value::BigInt { sign, limbs }` back to an evaluator
/// value — the reverse half of the F1 store round-trip (`18a §5.2.1(3)`).
/// No production call site reads a slot back today (the K3 store is
/// currently write/dedup-only, `store.rs`); this establishes the conversion
/// F1's contract requires and is exercised by the round-trip test below.
#[allow(dead_code)]
fn bigint_from_rt(sign: RtSign, limbs: &[u64]) -> EvalVal {
    let u32_digits: Vec<u32> = limbs
        .iter()
        .flat_map(|&limb| [(limb & 0xFFFF_FFFF) as u32, (limb >> 32) as u32])
        .collect();
    let magnitude = BigUint::from_slice(&u32_digits);
    let nb_sign = match sign {
        RtSign::Negative => NumSign::Minus,
        RtSign::NonNegative => NumSign::Plus,
    };
    bigint_to_int_val(BigInt::from_biguint(nb_sign, magnitude))
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
            EvalVal::from(r)
        }
        _ => EvalVal::Neutral,
    }
}

/// Checked fixed-width `iN`/`uN` binop (`18a §5 F2`, `35 §3`
/// degrade-not-wrap) — the bare arithmetic op's runtime face. On overflow
/// `op` returns `None` and the arm degrades to a stuck `EvalVal::Neutral`
/// (never the wrapped value, mirroring `bytes_at` OOB and Decimal's
/// align-beyond-`MAX_SHIFT`) — `Unknown` is reserved for open-hole residue
/// and must not be conflated with a runtime arithmetic fault.
fn checked_binop_i8(a: &EvalVal, b: &EvalVal, op: fn(i8, i8) -> Option<i8>) -> EvalVal {
    match (a, b) {
        (EvalVal::Int(x), EvalVal::Int(y)) => match op(*x as i8, *y as i8) {
            Some(r) => EvalVal::Int(r as i64),
            None => EvalVal::Neutral,
        },
        _ => EvalVal::Neutral,
    }
}

fn checked_binop_i16(a: &EvalVal, b: &EvalVal, op: fn(i16, i16) -> Option<i16>) -> EvalVal {
    match (a, b) {
        (EvalVal::Int(x), EvalVal::Int(y)) => match op(*x as i16, *y as i16) {
            Some(r) => EvalVal::Int(r as i64),
            None => EvalVal::Neutral,
        },
        _ => EvalVal::Neutral,
    }
}

fn checked_binop_i32(a: &EvalVal, b: &EvalVal, op: fn(i32, i32) -> Option<i32>) -> EvalVal {
    match (a, b) {
        (EvalVal::Int(x), EvalVal::Int(y)) => match op(*x as i32, *y as i32) {
            Some(r) => EvalVal::Int(r as i64),
            None => EvalVal::Neutral,
        },
        _ => EvalVal::Neutral,
    }
}

fn checked_binop_i64(a: &EvalVal, b: &EvalVal, op: fn(i64, i64) -> Option<i64>) -> EvalVal {
    match (a, b) {
        (EvalVal::Int(x), EvalVal::Int(y)) => match op(*x, *y) {
            Some(r) => EvalVal::Int(r),
            None => EvalVal::Neutral,
        },
        _ => EvalVal::Neutral,
    }
}

fn checked_binop_u8(a: &EvalVal, b: &EvalVal, op: fn(u8, u8) -> Option<u8>) -> EvalVal {
    match (a, b) {
        (EvalVal::Int(x), EvalVal::Int(y)) => match op(*x as u8, *y as u8) {
            Some(r) => EvalVal::Int(r as i64),
            None => EvalVal::Neutral,
        },
        _ => EvalVal::Neutral,
    }
}

fn checked_binop_u16(a: &EvalVal, b: &EvalVal, op: fn(u16, u16) -> Option<u16>) -> EvalVal {
    match (a, b) {
        (EvalVal::Int(x), EvalVal::Int(y)) => match op(*x as u16, *y as u16) {
            Some(r) => EvalVal::Int(r as i64),
            None => EvalVal::Neutral,
        },
        _ => EvalVal::Neutral,
    }
}

fn checked_binop_u32(a: &EvalVal, b: &EvalVal, op: fn(u32, u32) -> Option<u32>) -> EvalVal {
    match (a, b) {
        (EvalVal::Int(x), EvalVal::Int(y)) => match op(*x as u32, *y as u32) {
            Some(r) => EvalVal::Int(r as i64),
            None => EvalVal::Neutral,
        },
        _ => EvalVal::Neutral,
    }
}

fn checked_binop_u64(a: &EvalVal, b: &EvalVal, op: fn(u64, u64) -> Option<u64>) -> EvalVal {
    match (a, b) {
        (EvalVal::Int(x), EvalVal::Int(y)) => match op(*x as u64, *y as u64) {
            Some(r) => EvalVal::from(r as i128),
            None => EvalVal::Neutral,
        },
        _ => EvalVal::Neutral,
    }
}

/// Checked fixed-width negation (`18a §5 neg_intN`, ~L256) — signed only.
/// `neg(MIN_intN)` has no representable positive counterpart in two's
/// complement, so `checked_neg` returns `None` and the arm degrades to a
/// stuck `Neutral`, never the wrapped value (F2-consistent).
fn checked_neg_i8(a: &EvalVal) -> EvalVal {
    match a {
        EvalVal::Int(x) => match (*x as i8).checked_neg() {
            Some(r) => EvalVal::Int(r as i64),
            None => EvalVal::Neutral,
        },
        _ => EvalVal::Neutral,
    }
}

fn checked_neg_i16(a: &EvalVal) -> EvalVal {
    match a {
        EvalVal::Int(x) => match (*x as i16).checked_neg() {
            Some(r) => EvalVal::Int(r as i64),
            None => EvalVal::Neutral,
        },
        _ => EvalVal::Neutral,
    }
}

fn checked_neg_i32(a: &EvalVal) -> EvalVal {
    match a {
        EvalVal::Int(x) => match (*x as i32).checked_neg() {
            Some(r) => EvalVal::Int(r as i64),
            None => EvalVal::Neutral,
        },
        _ => EvalVal::Neutral,
    }
}

fn checked_neg_i64(a: &EvalVal) -> EvalVal {
    match a {
        EvalVal::Int(x) => match x.checked_neg() {
            Some(r) => EvalVal::Int(r),
            None => EvalVal::Neutral,
        },
        _ => EvalVal::Neutral,
    }
}

/// Structural equality on `EvalVal` for equality-testing contexts (`L1 §4`).
pub fn eval_vals_eq(a: &EvalVal, b: &EvalVal) -> bool {
    match (a, b) {
        (EvalVal::Bool(x), EvalVal::Bool(y)) => x == y,
        (EvalVal::Int(x), EvalVal::Int(y)) => x == y,
        (EvalVal::BigInt(x), EvalVal::BigInt(y)) => x == y,
        (EvalVal::Int(x), EvalVal::BigInt(y)) => BigInt::from(*x) == *y,
        (EvalVal::BigInt(x), EvalVal::Int(y)) => *x == BigInt::from(*y),
        (EvalVal::Float(x), EvalVal::Float(y)) => x == y,
        (EvalVal::Float32(x), EvalVal::Float32(y)) => x == y,
        // `Decimal` is now `Ctor{id:mkdecimalpair_id, args:[Int/BigInt,Int/BigInt]}`
        // (`18a §5.6.1`) — not handled by this scalar-immediate helper; the
        // `Ctor` derived `PartialEq` (or the derived `decimalEq` reduction)
        // covers it elsewhere.
        _ => false,
    }
}

/// Primitive reduction for registered operations (`42 §3.3`, `14 §5`).
///
/// Covers `L1` numeric tower: Int (arbitrary-precision, `18a §5.2.1`),
/// fixed-width (checked, degrade-not-wrap), Decimal, Float, Float32, Bool.
/// The legacy unregistered `add`/`sub`/`mul` (wrapping i64) arms were
/// retired (`18a §5 F3`); those symbols now fall through to the catch-all
/// stuck arm.
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
        // ---- Int (arbitrary-precision, `18a §5.2.1`) ----
        ("add_int", [a, b]) => exact_int_binop(a, b, |x, y| x + y),
        ("sub_int", [a, b]) => exact_int_binop(a, b, |x, y| x - y),
        ("mul_int", [a, b]) => exact_int_binop(a, b, |x, y| x * y),
        ("eq_int", [a, b]) => match (eval_to_bigint(a), eval_to_bigint(b)) {
            (Some(av), Some(bv)) => EvalVal::Bool(av == bv),
            _ => EvalVal::Neutral,
        },
        // `leq_int` (`18a §5.2.2`) — bignum-correct total order, mirroring
        // `eq_int`'s non-circularity discipline. Already-registered symbol
        // (`numbers.rs:233`); this arm only wires its reduction.
        ("leq_int", [a, b]) => match (eval_to_bigint(a), eval_to_bigint(b)) {
            (Some(av), Some(bv)) => EvalVal::Bool(av <= bv),
            _ => EvalVal::Neutral,
        },

        // ---- Fixed-width (checked, degrade-not-wrap: `18a §5 F2`, `35 §3`) ----
        // Bare arms are the no-overflow obligation's RUNTIME face: on overflow
        // they degrade to stuck `Neutral`, never the wrapped value. The
        // sanctioned modular class (`wrapping_*_intN`/`+%`, below) is the
        // only path permitted to wrap — left untouched.
        ("add_int8", [a, b]) => checked_binop_i8(a, b, i8::checked_add),
        ("sub_int8", [a, b]) => checked_binop_i8(a, b, i8::checked_sub),
        ("mul_int8", [a, b]) => checked_binop_i8(a, b, i8::checked_mul),
        ("add_int16", [a, b]) => checked_binop_i16(a, b, i16::checked_add),
        ("sub_int16", [a, b]) => checked_binop_i16(a, b, i16::checked_sub),
        ("mul_int16", [a, b]) => checked_binop_i16(a, b, i16::checked_mul),
        ("add_int32", [a, b]) => checked_binop_i32(a, b, i32::checked_add),
        ("sub_int32", [a, b]) => checked_binop_i32(a, b, i32::checked_sub),
        ("mul_int32", [a, b]) => checked_binop_i32(a, b, i32::checked_mul),
        ("add_int64", [a, b]) => checked_binop_i64(a, b, i64::checked_add),
        ("sub_int64", [a, b]) => checked_binop_i64(a, b, i64::checked_sub),
        ("mul_int64", [a, b]) => checked_binop_i64(a, b, i64::checked_mul),
        ("add_uint8", [a, b]) => checked_binop_u8(a, b, u8::checked_add),
        ("add_uint16", [a, b]) => checked_binop_u16(a, b, u16::checked_add),
        ("add_uint32", [a, b]) => checked_binop_u32(a, b, u32::checked_add),
        ("add_uint64", [a, b]) => checked_binop_u64(a, b, u64::checked_add),

        // ---- Wrapping variants (explicit `+%`) ----
        ("wrapping_add_int8", [a, b]) => fixed_binop_i8(a, b, i8::wrapping_add),
        ("wrapping_sub_int8", [a, b]) => fixed_binop_i8(a, b, i8::wrapping_sub),
        ("wrapping_mul_int8", [a, b]) => fixed_binop_i8(a, b, i8::wrapping_mul),
        ("wrapping_add_int16", [a, b]) => fixed_binop_i16(a, b, i16::wrapping_add),
        ("wrapping_add_int32", [a, b]) => fixed_binop_i32(a, b, i32::wrapping_add),
        ("wrapping_sub_int32", [a, b]) => fixed_binop_i32(a, b, i32::wrapping_sub),
        ("wrapping_mul_int32", [a, b]) => fixed_binop_i32(a, b, i32::wrapping_mul),
        ("wrapping_add_int64", [a, b]) => fixed_binop_i64(a, b, i64::wrapping_add),
        ("wrapping_add_uint8", [a, b]) => fixed_binop_u8(a, b, u8::wrapping_add),
        ("wrapping_add_uint16", [a, b]) => fixed_binop_u16(a, b, u16::wrapping_add),
        ("wrapping_add_uint32", [a, b]) => fixed_binop_u32(a, b, u32::wrapping_add),
        ("wrapping_add_uint64", [a, b]) => fixed_binop_u64(a, b, u64::wrapping_add),

        // ---- `IntN<->Int` conversion floor (`18a §5.7`, NATIVE) ----
        // Widening `IntN.toInt` (total): every fixed-width value already
        // shares `Int`'s own value representation (`EvalVal::Int`/`BigInt`),
        // so the reduction is identity — only the KERNEL type changes
        // (`IntN -> Int`), never the value.
        (
            "int8_to_int" | "int16_to_int" | "int32_to_int" | "int64_to_int" | "uint8_to_int"
            | "uint16_to_int" | "uint32_to_int" | "uint64_to_int" | "usize_to_int" | "isize_to_int"
            | "cint_to_int",
            [a],
        ) => a.clone(),
        // Narrowing raw cast `Int -> IntN` (UNCHECKED — identity at the value
        // level, same representation-sharing as widening). Not part of the
        // public surface: only called internally by the derived `intToIntN`
        // (Ken view, `conversions.rs`) AFTER its own range check, and by the
        // `saturating*` family after clamping — never exposed un-guarded.
        (
            "int_to_int8_raw" | "int_to_int16_raw" | "int_to_int32_raw" | "int_to_int64_raw"
            | "int_to_uint8_raw" | "int_to_uint16_raw" | "int_to_uint32_raw" | "int_to_uint64_raw"
            | "int_to_usize_raw" | "int_to_isize_raw" | "int_to_cint_raw",
            [a],
        ) => a.clone(),

        // `neg_intN` (`18a §5`, ~L256) — fixed-width negation stays NATIVE
        // and checked (does NOT demote to `sub_int 0 x`, unlike bignum
        // `neg_int`): `neg(MIN_intN)` overflows the asymmetric two's-
        // complement range, degrading to stuck `Neutral` (F2-consistent),
        // never a wrapped value. Signed widths only — unsigned negation of
        // any nonzero value is out of range by construction and out of
        // scope (`18a` names no `neg_uintN`).
        ("neg_int8", [a]) => checked_neg_i8(a),
        ("neg_int16", [a]) => checked_neg_i16(a),
        ("neg_int32", [a]) => checked_neg_i32(a),
        ("neg_int64", [a]) => checked_neg_i64(a),

        // Decimal (`add_decimal`/`sub_decimal`/`mul_decimal`/`eq_decimal`) is
        // DEMOTE→derived (`18a §5.6.1`): no native `prim_reduce` arm here —
        // the elaborated `decimalAdd`/`decimalSub`/`decimalMul`/`decimalEq`
        // definitions (`ken-elaborator/src/decimal_char.rs`) reduce via
        // ordinary β/ι/δ evaluation over `add_int`/`mul_int`/`leq_int`.

        // ---- Float (IEEE 754 f64) ----
        ("add_float", [EvalVal::Float(a), EvalVal::Float(b)]) => EvalVal::Float(a + b),
        ("sub_float", [EvalVal::Float(a), EvalVal::Float(b)]) => EvalVal::Float(a - b),
        ("mul_float", [EvalVal::Float(a), EvalVal::Float(b)]) => EvalVal::Float(a * b),
        ("div_float", [EvalVal::Float(a), EvalVal::Float(b)]) => EvalVal::Float(a / b),
        ("eq_float", [EvalVal::Float(a), EvalVal::Float(b)]) => EvalVal::Bool(a == b),

        // ---- Float32 (IEEE 754 f32) ----
        ("add_float32", [EvalVal::Float32(a), EvalVal::Float32(b)]) => EvalVal::Float32(a + b),
        ("eq_float32", [EvalVal::Float32(a), EvalVal::Float32(b)]) => EvalVal::Bool(a == b),

        // ---- Bool ----
        ("not_bool", [EvalVal::Bool(b)]) => EvalVal::Bool(!b),
        ("and_bool", [EvalVal::Bool(a), EvalVal::Bool(b)]) => EvalVal::Bool(*a && *b),
        ("or_bool", [EvalVal::Bool(a), EvalVal::Bool(b)]) => EvalVal::Bool(*a || *b),

        // Legacy `add`/`sub`/`mul` (wrapping i64) retired (`18a §5 F3`):
        // unregistered (no `reg_binop!`/`declare_primitive` in
        // `ken-elaborator/src/numbers.rs`) dead-but-live wrap arms — no
        // surface program could reach them, and the arity entry below is
        // gone too, so `prim_reduce("add"/"sub"/"mul", ..)` now falls
        // through to the catch-all `_ => EvalVal::Neutral` at the bottom.

        // ── Bytes primitive ops (`38 §1.2`) ──────────────────────────────────
        ("bytes_length", [EvalVal::Bytes(b)]) => EvalVal::Int(b.len() as i64),

        ("bytes_concat", [EvalVal::Bytes(a), EvalVal::Bytes(b)]) => {
            let mut out = a.clone();
            out.extend_from_slice(b);
            EvalVal::Bytes(out)
        }

        // ── String ↔ Bytes encode/decode (`38 §1.4`) ─────────────────────────
        // encode: total — String is always valid UTF-8 at construction.
        ("bytes_encode", [EvalVal::Str(s)]) => EvalVal::Bytes(s.as_bytes().to_vec()),

        // The safe `bytes_at`, `bytes_slice`, and `bytes_decode` reductions
        // require their elaborated Option/Result constructor identities and
        // are therefore handled by `prim_reduce_elaborated`, not this
        // environment-free helper.

        // ── L3a String surface ops (`37 §2`) ───────────────────────────────
        // `byte_length s` — the stored UTF-8 byte count (`37 §2.2`). Real now
        // (NFC-independent: a CJK/non-combining witness differs from char count
        // regardless of normalization).
        ("byte_length", [EvalVal::Str(s)]) => EvalVal::Int(s.len() as i64),
        // `char_length s` — the Unicode scalar-value count (`37 §2.2`).
        ("char_length", [EvalVal::Str(s)]) => EvalVal::Int(s.chars().count() as i64),
        // Structural `String`/`Bytes` views need `store` + polymorphic `List`
        // constructor ids, unavailable to this pure fn. They are intercepted
        // in `apply`; direct calls stay Neutral (stuck), never silently wrong.
        ("string_to_list_char", [EvalVal::Str(_)]) => EvalVal::Neutral,
        ("list_char_to_string", [EvalVal::Ctor { .. }]) => EvalVal::Neutral,
        ("bytes_to_list", [EvalVal::Bytes(_)]) => EvalVal::Neutral,
        ("list_to_bytes", [EvalVal::Ctor { .. }]) => EvalVal::Neutral,

        // Partial or unrecognised primitive: neutral (stuck on non-literals).
        _ => EvalVal::Neutral,
    }
}

/// Derived strict order (`18a §5.2.2(2)`): `lt a b := ¬(leq_int b a)` —
/// Steward's locked minimal form (pure `leq`, no `eq`). No `lt_int` primitive
/// is registered; this composes the two already-reducing prims via
/// `prim_reduce` itself, so the composition is exercised end-to-end rather
/// than shortcut with a raw Rust `!`.
pub fn derived_lt_int(a: &EvalVal, b: &EvalVal) -> EvalVal {
    let leq_b_a = prim_reduce("leq_int", &[b.clone(), a.clone()]);
    prim_reduce("not_bool", std::slice::from_ref(&leq_b_a))
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

        // --- IntLit: a checked, already-canonical Int value
        // (`docs/adr/0013-int-decidable-equality-kernel-posture.md`
        // Layer 2) — narrows to the `Int`/`BigInt` fast-path split exactly
        // like every other BigInt-producing arithmetic result.
        Term::IntLit(n) => bigint_to_int_val(n.clone()),

        // --- Lambda: form a closure (body NOT reduced under binder) ---
        Term::Lam(_dom, body) => make_closure(Rc::new(*body.clone()), Rc::new(env.to_vec()), store),

        // --- Application: CBV — force operator then argument ---
        Term::App(f, u) => {
            if let Some(projected) = eval_projection_accessor_app(env, f, u, globals, store) {
                return projected;
            }
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
        Term::Proj1(p) => eval_projection_path(env, p, globals, store, &[Projection::Fst]),
        Term::Proj2(p) => eval_projection_path(env, p, globals, store, &[Projection::Snd]),

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
                    PrimReduction::Literal => EvalVal::Neutral,
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
                        // Structural String/Bytes views are intercepted here:
                        // they need `store` (make_ctor) and the polymorphic
                        // `Nil`/`Cons` ids, neither available to the pure
                        // `prim_reduce(symbol, args)` helper. They fall through
                        // to Neutral when the constructor ids are unwired.
                        if let Some(ids) = store.list_char_ids.clone() {
                            if *symbol == "string_to_list_char" {
                                if let [EvalVal::Str(s)] = args.as_slice() {
                                    return build_list_char(s, &ids, store);
                                }
                            } else if *symbol == "list_char_to_string" {
                                if let [v] = args.as_slice() {
                                    return list_char_to_evalval_string(v, &ids)
                                        .map(EvalVal::Str)
                                        .unwrap_or(EvalVal::Neutral);
                                }
                            } else if *symbol == "bytes_to_list" {
                                if let [EvalVal::Bytes(bytes)] = args.as_slice() {
                                    return build_list_uint8(bytes, &ids, store);
                                }
                            } else if *symbol == "list_to_bytes" {
                                if let [v] = args.as_slice() {
                                    return list_uint8_to_bytes(v, &ids)
                                        .map(EvalVal::Bytes)
                                        .unwrap_or(EvalVal::Neutral);
                                }
                            }
                        }
                        if matches!(*symbol, "bytes_at" | "bytes_slice" | "bytes_decode") {
                            return reduce_safe_bytes_primitive(id, symbol, &args, globals, store);
                        }
                        return prim_reduce(symbol, &args);
                    }
                }
                make_ctor(id, args, store)
            } else {
                EvalVal::CtorPending { id, args, need }
            }
        }

        // --- K1.5 W-style IH: accumulate branch args, fold when saturated ---
        EvalVal::IhClosure {
            rec_field,
            fam,
            methods,
            ih_env,
            nb,
            applied,
        } => {
            let mut applied2 = (*applied).clone();
            applied2.push(u);
            if applied2.len() >= nb {
                let mut branch_val = (*rec_field).clone();
                for a in applied2 {
                    branch_val = apply(branch_val, a, globals, store);
                }
                elim_reduce(&ih_env, fam, &methods, branch_val, globals, store)
            } else {
                EvalVal::IhClosure {
                    rec_field,
                    fam,
                    methods,
                    ih_env,
                    nb,
                    applied: Rc::new(applied2),
                }
            }
        }

        // --- Unknown: propagate strictly ---
        EvalVal::Unknown => EvalVal::Unknown,

        // --- Neutral: remain stuck ---
        _ => EvalVal::Neutral,
    }
}

fn primitive_result_type<'a>(id: GlobalId, globals: &'a GlobalEnv) -> Option<&'a Term> {
    let Decl::Primitive { ty, .. } = globals.lookup(id)? else {
        return None;
    };
    let mut result = ty;
    while let Term::Pi(_, codomain) = result {
        result = codomain;
    }
    Some(result)
}

fn type_application(term: &Term) -> Option<(GlobalId, Vec<&Term>)> {
    let mut args = Vec::new();
    let mut head = term;
    while let Term::App(function, argument) = head {
        args.push(argument.as_ref());
        head = function;
    }
    args.reverse();
    let Term::IndFormer { id, .. } = head else {
        return None;
    };
    Some((*id, args))
}

fn reduce_safe_bytes_primitive(
    primitive_id: GlobalId,
    symbol: &str,
    args: &[EvalVal],
    globals: &GlobalEnv,
    store: &mut EvalStore,
) -> EvalVal {
    let Some((result_family, result_params)) =
        primitive_result_type(primitive_id, globals).and_then(type_application)
    else {
        return EvalVal::Neutral;
    };
    let Some(result_decl) = globals.inductive(result_family) else {
        return EvalVal::Neutral;
    };
    let type_args = || vec![EvalVal::Neutral; result_params.len()];

    match (symbol, args) {
        ("bytes_at", [EvalVal::Bytes(bytes), EvalVal::Int(index)]) => {
            let Some(none) = result_decl.constructors.first() else {
                return EvalVal::Neutral;
            };
            let Some(some) = result_decl.constructors.get(1) else {
                return EvalVal::Neutral;
            };
            match usize::try_from(*index)
                .ok()
                .and_then(|index| bytes.get(index).copied())
            {
                Some(byte) => {
                    let mut ctor_args = type_args();
                    ctor_args.push(EvalVal::Int(i64::from(byte)));
                    make_ctor(some.id, ctor_args, store)
                }
                None => make_ctor(none.id, type_args(), store),
            }
        }
        ("bytes_slice", [EvalVal::Bytes(bytes), EvalVal::Int(start), EvalVal::Int(len)]) => {
            let Some(none) = result_decl.constructors.first() else {
                return EvalVal::Neutral;
            };
            let Some(some) = result_decl.constructors.get(1) else {
                return EvalVal::Neutral;
            };
            let slice = usize::try_from(*start)
                .ok()
                .zip(usize::try_from(*len).ok())
                .and_then(|(start, len)| {
                    start
                        .checked_add(len)
                        .filter(|end| *end <= bytes.len())
                        .map(|end| bytes[start..end].to_vec())
                });
            match slice {
                Some(slice) => {
                    let mut ctor_args = type_args();
                    ctor_args.push(EvalVal::Bytes(slice));
                    make_ctor(some.id, ctor_args, store)
                }
                None => make_ctor(none.id, type_args(), store),
            }
        }
        ("bytes_decode", [EvalVal::Bytes(bytes)]) => {
            let Some(err) = result_decl.constructors.first() else {
                return EvalVal::Neutral;
            };
            let Some(ok) = result_decl.constructors.get(1) else {
                return EvalVal::Neutral;
            };
            match std::str::from_utf8(bytes) {
                Ok(string) => {
                    let mut ctor_args = type_args();
                    ctor_args.push(EvalVal::Str(string.to_string()));
                    make_ctor(ok.id, ctor_args, store)
                }
                Err(_) => {
                    let Some(error_ty) = result_params.first() else {
                        return EvalVal::Neutral;
                    };
                    let Some((error_family, _)) = type_application(error_ty) else {
                        return EvalVal::Neutral;
                    };
                    let Some(error_ctor) = globals
                        .inductive(error_family)
                        .and_then(|decl| decl.constructors.first())
                    else {
                        return EvalVal::Neutral;
                    };
                    let error = make_ctor(error_ctor.id, vec![], store);
                    let mut ctor_args = type_args();
                    ctor_args.push(error);
                    make_ctor(err.id, ctor_args, store)
                }
            }
        }
        _ => EvalVal::Neutral,
    }
}

/// Reduce a registered primitive with access to its elaborated result type.
/// Safe Bytes primitives need that type to construct real `Option`/`Result`
/// values; the legacy pure `prim_reduce` helper deliberately has no global
/// environment and therefore cannot manufacture constructor identities.
pub fn prim_reduce_elaborated(
    symbol: &str,
    args: &[EvalVal],
    elab: &ken_elaborator::ElabEnv,
    store: &mut EvalStore,
) -> EvalVal {
    if matches!(symbol, "bytes_at" | "bytes_slice" | "bytes_decode") {
        let Some(id) = elab.globals.get(symbol).copied() else {
            return EvalVal::Neutral;
        };
        reduce_safe_bytes_primitive(id, symbol, args, &elab.env, store)
    } else {
        prim_reduce(symbol, args)
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
/// `params_len` is the number of ITree *type* params (1 for the landed
/// `ITree r`; 0 for the simplified 0-param test ITree).
#[derive(Clone)]
pub struct ConsoleIds {
    /// `GlobalId` of the `ITree` inductive (for documentation; not used in the loop).
    pub itree_id: GlobalId,
    /// `GlobalId` of the `Ret` constructor (k = 0).
    pub ret_id: GlobalId,
    /// `GlobalId` of the `Vis` constructor (k = 1).
    pub vis_id: GlobalId,
    pub read_id: GlobalId,
    pub write_id: GlobalId,
    pub flush_id: GlobalId,
    pub is_terminal_id: GlobalId,
    pub stdin_id: GlobalId,
    pub stdout_id: GlobalId,
    pub stderr_id: GlobalId,
    pub chunk_id: GlobalId,
    pub eof_id: GlobalId,
    pub true_id: GlobalId,
    pub false_id: GlobalId,
    pub ok_id: GlobalId,
    pub err_id: GlobalId,
    pub notfound_id: GlobalId,
    pub permissiondenied_id: GlobalId,
    pub capabilitydenied_id: GlobalId,
    pub brokenpipe_id: GlobalId,
    pub interrupted_id: GlobalId,
    pub alreadyexists_id: GlobalId,
    pub invalidinput_id: GlobalId,
    pub isdirectory_id: GlobalId,
    pub notdirectory_id: GlobalId,
    pub notempty_id: GlobalId,
    pub unsupported_id: GlobalId,
    pub other_id: GlobalId,
    /// `GlobalId` of the `Unit` constructor (response to `Write`).
    pub unit_id: GlobalId,
    /// Number of ITree type-level params. Ctor-specific args start at this offset.
    pub params_len: usize,
}

impl ConsoleIds {
    /// Harvest the complete Console ABI table from an elaboration environment.
    pub fn from_elab(elab: &ken_elaborator::ElabEnv) -> Option<Self> {
        let get = |name: &str| elab.globals.get(name).copied();
        Some(Self {
            itree_id: get("ITree")?,
            ret_id: get("Ret")?,
            vis_id: get("Vis")?,
            read_id: get("Read")?,
            write_id: get("Write")?,
            flush_id: get("Flush")?,
            is_terminal_id: get("IsTerminal")?,
            stdin_id: get("Stdin")?,
            stdout_id: get("Stdout")?,
            stderr_id: get("Stderr")?,
            chunk_id: get("Chunk")?,
            eof_id: get("Eof")?,
            true_id: get("True")?,
            false_id: get("False")?,
            ok_id: get("Ok")?,
            err_id: get("Err")?,
            notfound_id: get("NotFound")?,
            permissiondenied_id: get("PermissionDenied")?,
            capabilitydenied_id: get("CapabilityDenied")?,
            brokenpipe_id: get("BrokenPipe")?,
            interrupted_id: get("Interrupted")?,
            alreadyexists_id: get("AlreadyExists")?,
            invalidinput_id: get("InvalidInput")?,
            isdirectory_id: get("IsDirectory")?,
            notdirectory_id: get("NotDirectory")?,
            notempty_id: get("NotEmpty")?,
            unsupported_id: get("Unsupported")?,
            other_id: get("Other")?,
            unit_id: get("MkUnit")?,
            params_len: 3,
        })
    }
}

/// `List Char` constructor IDs needed by `string_to_list_char`/
/// `list_char_to_string` (`37 §2.3`). `List` has one type param, so `Nil`'s
/// `Ctor.args = [type_param]` and `Cons`'s `Ctor.args = [type_param, head,
/// tail]` (`prelude.rs`'s `cons_app` helper — `Cons` is always applied to the
/// element type first). The type-param slot carries no runtime information
/// (type parameters are carried as `EvalVal::Unknown` fillers).
#[derive(Clone)]
pub struct ListCharIds {
    pub nil_id: GlobalId,
    pub cons_id: GlobalId,
}

/// Decode a Rust `&str` into a `List Char` value (`string_to_list_char`,
/// `37 §2.3`). Witness mechanism (AC1): Rust's `char` is a hard language
/// invariant — every value is a Unicode Scalar Value, structurally excluding
/// the surrogate range `[0xD800,0xDFFF]` and bounded by `0x10FFFF` — exactly
/// Ken's own `isScalar`/`inRangeBool` range (`decimal_char.rs:225`:
/// `[0,55295]∪[57344,1114111]`). The `debug_assert` is a hardcoded copy of
/// `inRangeBool`'s range, checked against every decoded codepoint: it catches
/// decode-path drift (a future change sourcing codepoints from anything other
/// than scalar-guaranteed Rust `char`). It does NOT auto-detect a change to
/// `inRangeBool` itself (that would need re-entering the Ken reduction,
/// avoided here) — that agreement is a trusted bridge maintained by audit; if
/// `inRangeBool` is ever narrowed, this range must be updated in lockstep.
/// No precedent exists for a native prim re-invoking `eval` on elaborated
/// terms mid-reduction — native + checked, the `neg_intN` posture, not
/// demoted and not bare-trusted.
fn build_list_char(s: &str, ids: &ListCharIds, store: &mut EvalStore) -> EvalVal {
    let mut acc = make_ctor(ids.nil_id, vec![EvalVal::Unknown], store);
    for c in s.chars().rev() {
        let cp = c as u32;
        debug_assert!(
            cp <= 0xD7FF || (0xE000..=0x0010_FFFF).contains(&cp),
            "Rust char {:#x} outside Ken's isScalar range — invariant violated",
            cp
        );
        let elem = EvalVal::Int(cp as i64);
        acc = make_ctor(ids.cons_id, vec![EvalVal::Unknown, elem, acc], store);
    }
    acc
}

/// Walk a `List Char` value, decoding each element back to a `char` and
/// appending to a `String` (`list_char_to_string`, `37 §2.3`). Total (AC4):
/// relies on kernel soundness of `Char`'s refinement — only a valid-scalar
/// `Int` can be well-typed as `Char` — the same trust boundary already
/// accepted for `int_to_intN_raw` (conversions floor). `char::from_u32`'s
/// fallback is defensive dead code under that soundness guarantee, never
/// `Neutral`/panic, so totality holds even if it were ever reached. Returns
/// `None` only if `v` is not a well-formed `List Char` `Ctor` chain (neither
/// `Nil` nor `Cons` — the caller degrades to `Neutral`, never silently wrong).
///
/// The fallback value is `char::REPLACEMENT_CHARACTER` (U+FFFD) via safe
/// `char::from_u32` (never `_unchecked`/UB) — pinned and named here so AC4's
/// totality claim rests on a concrete value, not a bare "fallback." It is
/// unreachable under `Char`'s refinement soundness (only a valid-scalar `Int`
/// is ever well-typed `Char`); if that invariant were ever violated elsewhere,
/// `String` is bare-typed, so surfacing U+FFFD is soundness-inert regardless.
fn list_char_to_evalval_string(v: &EvalVal, ids: &ListCharIds) -> Option<String> {
    let mut out = String::new();
    let mut cur = v.clone();
    loop {
        match &cur {
            EvalVal::Ctor { id, .. } if *id == ids.nil_id => return Some(out),
            EvalVal::Ctor { id, args, .. } if *id == ids.cons_id => {
                let head = args.get(1)?;
                let cp = eval_to_bigint(head)?.to_u32()?;
                out.push(char::from_u32(cp).unwrap_or(char::REPLACEMENT_CHARACTER));
                let tail = args.get(2)?.clone();
                cur = tail;
            }
            _ => return None,
        }
    }
}

/// Expose an immutable byte buffer as its structural `List UInt8` view.
fn build_list_uint8(bytes: &[u8], ids: &ListCharIds, store: &mut EvalStore) -> EvalVal {
    let mut acc = make_ctor(ids.nil_id, vec![EvalVal::Unknown], store);
    for byte in bytes.iter().rev() {
        acc = make_ctor(
            ids.cons_id,
            vec![EvalVal::Unknown, EvalVal::Int(i64::from(*byte)), acc],
            store,
        );
    }
    acc
}

/// Rebuild a byte buffer from a well-typed `List UInt8`. The elaborator's
/// `UInt8` type guarantees every head is in range; the checked conversion is a
/// defensive fail-closed guard for malformed runtime values.
fn list_uint8_to_bytes(v: &EvalVal, ids: &ListCharIds) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut cur = v.clone();
    loop {
        match &cur {
            EvalVal::Ctor { id, .. } if *id == ids.nil_id => return Some(out),
            EvalVal::Ctor { id, args, .. } if *id == ids.cons_id => {
                let head = args.get(1)?;
                let byte = eval_to_bigint(head)?.to_u8()?;
                out.push(byte);
                cur = args.get(2)?.clone();
            }
            _ => return None,
        }
    }
}

/// IDs for the D3 coproduct peel (`effect-composition` D3, doc §D3.2/§D3.4).
/// Effect-blind: the peel matches ONLY on ctor identity (`inl_id`/`inr_id`),
/// never on which base effect the payload carries — no `ConsoleOp`/`FSOp`
/// literal anywhere in the peel (BV5).
#[derive(Clone)]
pub struct CoproductIds {
    pub inl_id: GlobalId,
    pub inr_id: GlobalId,
}

/// IDs for the ambient `Clock` effect driver.
#[derive(Clone)]
pub struct ClockIds {
    pub wall_now_id: GlobalId,
    pub mkinstant_id: GlobalId,
}

impl ClockIds {
    /// Harvest the complete Clock ABI table from an elaboration environment.
    pub fn from_elab(elab: &ken_elaborator::ElabEnv) -> Option<Self> {
        let get = |name: &str| elab.globals.get(name).copied();
        Some(Self {
            wall_now_id: get("WallNow")?,
            mkinstant_id: get("MkInstant")?,
        })
    }
}

/// The three process-owned streams exposed by the Console ABI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleStream {
    Stdin,
    Stdout,
    Stderr,
}

/// A total host read: an explicit chunk or an explicit end-of-file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostRead {
    Chunk(Vec<u8>),
    Eof,
}

/// Exact Console operations observed by the injectable host seam.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsoleTrace {
    Read {
        stream: ConsoleStream,
        limit: usize,
    },
    Write {
        stream: ConsoleStream,
        bytes: Vec<u8>,
    },
    Flush {
        stream: ConsoleStream,
    },
    IsTerminal {
        stream: ConsoleStream,
    },
}

/// Exact wall-clock reads observed by the injectable host seam.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClockTrace {
    WallNow { nanoseconds: BigInt },
}

/// Create policy carried by `WriteFile` after decoding its Ken constructor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostCreatePolicy {
    CreateNew,
    CreateOrTruncate,
    CreateOrKeep,
}

/// Stable, platform-independent file-kind projection exposed to Ken.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostFileKind {
    File,
    Directory,
    Symlink,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostFileMetadata {
    pub size: u64,
    pub kind: HostFileKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostDirEntry {
    pub name: Vec<u8>,
    pub kind: HostFileKind,
}

/// Exact FS operations observed at the injectable host seam.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FsTrace {
    ReadFile {
        path: Vec<u8>,
    },
    WriteFile {
        path: Vec<u8>,
        policy: HostCreatePolicy,
        bytes: Vec<u8>,
    },
    AppendFile {
        path: Vec<u8>,
        bytes: Vec<u8>,
    },
    Metadata {
        path: Vec<u8>,
    },
    ReadDirectory {
        path: Vec<u8>,
    },
    CreateDirectory {
        path: Vec<u8>,
        recursive: bool,
    },
    RemoveFile {
        path: Vec<u8>,
    },
    RemoveDirectory {
        path: Vec<u8>,
        recursive: bool,
    },
    Rename {
        from: Vec<u8>,
        to: Vec<u8>,
    },
}

/// Observable virtual-FS state used by `CaptureHost`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VirtualFsNode {
    File(Vec<u8>),
    Directory,
    Symlink(Vec<u8>),
}

pub type VfsNodeId = u64;

pub use ken_host::capability::{CapabilityDenied, FsCapabilityOperation as FsOpKind};

#[derive(Debug)]
pub enum ResolveError {
    Denied(CapabilityDenied),
    Io(io::Error),
}

impl From<io::Error> for ResolveError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Debug)]
pub enum Resolution<H> {
    Existing(H),
    Parent(H, Vec<u8>),
}

/// Provides host effects to `run_io`.
///
/// SECURITY: paths enter only through `fs_resolve`. Every operation below
/// consumes a host-owned descriptor/node id, so the trait has no byte-path
/// bypass that can re-resolve after authorization.
pub trait HostHandler {
    type Handle: Clone + std::fmt::Debug;

    /// Mints a cap rooted at the host's own resolved identity, bounded by the
    /// declared authority. This is runner-only and never reachable from Ken.
    fn mint_fs_cap(&self, authority: capabilities::Authority) -> capabilities::Cap;

    fn console_read(&mut self, stream: ConsoleStream, limit: usize) -> io::Result<HostRead>;
    fn console_write(&mut self, stream: ConsoleStream, bytes: &[u8]) -> io::Result<()>;
    fn console_flush(&mut self, stream: ConsoleStream) -> io::Result<()>;
    fn console_is_terminal(&mut self, stream: ConsoleStream) -> bool;

    /// Read wall-clock nanoseconds. This is ambient process context, carries
    /// no capability, and intentionally promises no ordering law.
    fn clock_wall_now(&mut self) -> BigInt;

    /// Observation hook for an exact pre-operation capability denial.
    fn fs_denied(&mut self, _denial: CapabilityDenied) {}

    /// Deterministic test seam after resolution and before handle operation.
    fn fs_after_resolve(&mut self) {}

    fn fs_resolve(
        &mut self,
        root: &capabilities::FsHandle,
        components: &[Vec<u8>],
        op: FsOpKind,
        symlink: capabilities::SymlinkPolicy,
    ) -> Result<Resolution<Self::Handle>, ResolveError>;
    fn fs_read_at(&mut self, handle: &Self::Handle) -> io::Result<Vec<u8>>;
    fn fs_write_at(
        &mut self,
        handle: &Self::Handle,
        policy: HostCreatePolicy,
        bytes: &[u8],
    ) -> io::Result<()>;
    fn fs_create_file_at(
        &mut self,
        parent: &Self::Handle,
        leaf: &[u8],
        policy: HostCreatePolicy,
        bytes: &[u8],
    ) -> io::Result<()>;
    fn fs_append_at(&mut self, handle: &Self::Handle, bytes: &[u8]) -> io::Result<()>;
    fn fs_create_append_at(
        &mut self,
        parent: &Self::Handle,
        leaf: &[u8],
        bytes: &[u8],
    ) -> io::Result<()>;
    fn fs_metadata_at(&mut self, handle: &Self::Handle) -> io::Result<HostFileMetadata>;
    fn fs_read_directory_at(&mut self, handle: &Self::Handle) -> io::Result<Vec<HostDirEntry>>;
    fn fs_create_directory_at(
        &mut self,
        parent: &Self::Handle,
        leaf: &[u8],
        recursive: bool,
    ) -> io::Result<()>;
    fn fs_remove_file_at(&mut self, parent: &Self::Handle, leaf: &[u8]) -> io::Result<()>;
    fn fs_remove_directory_at(
        &mut self,
        parent: &Self::Handle,
        leaf: &[u8],
        recursive: bool,
    ) -> io::Result<()>;
    fn fs_rename_at(
        &mut self,
        from_parent: &Self::Handle,
        from_leaf: &[u8],
        to_parent: &Self::Handle,
        to_leaf: &[u8],
    ) -> io::Result<()>;
}

#[cfg(any(test, not(target_os = "linux")))]
fn host_abi_unsupported() -> io::Error {
    io::Error::from(io::ErrorKind::Unsupported)
}

#[cfg(target_os = "linux")]
fn openat_handle(
    parent: &RootedHandle,
    leaf: &[u8],
    request: HostOpenRequest,
) -> io::Result<RootedHandle> {
    Ok(ken_host::open_at(
        parent,
        &PathComponent::new(leaf)?,
        request,
    )?)
}

#[cfg(target_os = "linux")]
fn readlinkat_bytes(parent: &RootedHandle, leaf: &[u8]) -> io::Result<Vec<u8>> {
    Ok(ken_host::readlink_at(parent, &PathComponent::new(leaf)?)?)
}

/// Real process stdio handler. Ken's supported standard-Rust binary entrypoint
/// inherits Rust's pre-`main` SIGPIPE-ignore contract, so writes report EPIPE.
pub struct PosixHost {
    #[cfg(target_os = "linux")]
    root: RootedHandle,
}

impl PosixHost {
    pub fn new() -> Self {
        Self::new_at(".")
    }

    pub fn new_at(path: impl AsRef<std::path::Path>) -> Self {
        #[cfg(target_os = "linux")]
        {
            assert_interpreter_target_abi_hash(INTERPRETER_TARGET_ABI_MANIFEST_HASH)
                .expect("interpreter target ABI identity");
            let path = ken_host::RootPath::new(path).expect("validate filesystem capability root");
            let root = ken_host::open_root(&path).expect("open filesystem capability root");
            Self { root }
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = path;
            Self {}
        }
    }

    pub fn mint_fs_cap(&self, authority: capabilities::Authority) -> capabilities::Cap {
        #[cfg(target_os = "linux")]
        {
            let metadata = ken_host::metadata(&self.root).expect("stat process working directory");
            let identity = capabilities::FsIdentity::Posix {
                device: metadata.identity.device,
                inode: metadata.identity.inode,
            };
            let rights = if authority == capabilities::AUTH_FULL {
                capabilities::RightSet::ALL
            } else if authority == capabilities::AUTH_PARTIAL {
                capabilities::RightSet::READ
                    .union(capabilities::RightSet::ENUMERATE)
                    .union(capabilities::RightSet::METADATA)
            } else {
                capabilities::RightSet::NONE
            };
            capabilities::Cap::mint_scoped(
                authority,
                "FS",
                capabilities::FsScope::root(
                    rights,
                    capabilities::FsHandle::Posix(self.root.clone()),
                    identity,
                    capabilities::SymlinkPolicy::NoFollow,
                ),
            )
        }
        #[cfg(not(target_os = "linux"))]
        {
            capabilities::Cap::mint(authority, "FS")
        }
    }

    pub fn mint_scoped_fs_cap(
        &self,
        authority: capabilities::Authority,
        relative_root: &[u8],
        rights: capabilities::RightSet,
        symlink: capabilities::SymlinkPolicy,
    ) -> io::Result<capabilities::Cap> {
        #[cfg(target_os = "linux")]
        {
            if relative_root.starts_with(b"/") {
                return Err(io::Error::from(io::ErrorKind::InvalidInput));
            }
            let root_metadata = ken_host::metadata(&self.root)?;
            let mut handle = self.root.clone();
            let mut lineage = vec![capabilities::FsIdentity::Posix {
                device: root_metadata.identity.device,
                inode: root_metadata.identity.inode,
            }];
            for component in relative_root.split(|byte| *byte == b'/') {
                if component.is_empty() || component == b"." {
                    continue;
                }
                if component == b".." {
                    return Err(io::Error::from(io::ErrorKind::InvalidInput));
                }
                handle = openat_handle(&handle, component, HostOpenRequest::ReadDirectory)?;
                let metadata = ken_host::metadata(&handle)?;
                lineage.push(capabilities::FsIdentity::Posix {
                    device: metadata.identity.device,
                    inode: metadata.identity.inode,
                });
            }
            Ok(capabilities::Cap::mint_scoped(
                authority,
                "FS",
                capabilities::FsScope {
                    rights,
                    root: capabilities::FsHandle::Posix(handle),
                    lineage,
                    symlink,
                    empty: false,
                },
            ))
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (relative_root, rights, symlink);
            Err(host_abi_unsupported())
        }
    }
}

impl Default for PosixHost {
    fn default() -> Self {
        Self::new()
    }
}

impl HostHandler for PosixHost {
    #[cfg(target_os = "linux")]
    type Handle = RootedHandle;
    #[cfg(not(target_os = "linux"))]
    type Handle = u64;

    fn mint_fs_cap(&self, authority: capabilities::Authority) -> capabilities::Cap {
        PosixHost::mint_fs_cap(self, authority)
    }

    fn console_read(&mut self, stream: ConsoleStream, limit: usize) -> io::Result<HostRead> {
        if stream != ConsoleStream::Stdin {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "stream is not readable",
            ));
        }
        if limit == 0 {
            return Ok(HostRead::Chunk(Vec::new()));
        }
        let mut bytes = vec![0; limit];
        let count = io::stdin().lock().read(&mut bytes)?;
        if count == 0 {
            Ok(HostRead::Eof)
        } else {
            bytes.truncate(count);
            Ok(HostRead::Chunk(bytes))
        }
    }

    fn console_write(&mut self, stream: ConsoleStream, bytes: &[u8]) -> io::Result<()> {
        match stream {
            ConsoleStream::Stdout => io::stdout().lock().write_all(bytes),
            ConsoleStream::Stderr => io::stderr().lock().write_all(bytes),
            ConsoleStream::Stdin => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "stream is not writable",
            )),
        }
    }

    fn console_flush(&mut self, stream: ConsoleStream) -> io::Result<()> {
        match stream {
            ConsoleStream::Stdout => io::stdout().lock().flush(),
            ConsoleStream::Stderr => io::stderr().lock().flush(),
            ConsoleStream::Stdin => Ok(()),
        }
    }

    fn console_is_terminal(&mut self, stream: ConsoleStream) -> bool {
        match stream {
            ConsoleStream::Stdin => io::stdin().is_terminal(),
            ConsoleStream::Stdout => io::stdout().is_terminal(),
            ConsoleStream::Stderr => io::stderr().is_terminal(),
        }
    }

    fn clock_wall_now(&mut self) -> BigInt {
        match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(duration) => BigInt::from(duration.as_nanos()),
            Err(error) => -BigInt::from(error.duration().as_nanos()),
        }
    }

    fn fs_resolve(
        &mut self,
        root: &capabilities::FsHandle,
        components: &[Vec<u8>],
        op: FsOpKind,
        symlink: capabilities::SymlinkPolicy,
    ) -> Result<Resolution<Self::Handle>, ResolveError> {
        #[cfg(target_os = "linux")]
        {
            let capabilities::FsHandle::Posix(root) = root else {
                return Err(ResolveError::Denied(CapabilityDenied::ScopeEscape));
            };
            if components.is_empty() {
                return Ok(Resolution::Existing(root.clone()));
            }
            let mut stack = vec![root.clone()];
            let mut pending = components.to_vec();
            let mut index = 0;
            let mut symlink_hops = 0usize;
            while index < pending.len() {
                let component = pending[index].clone();
                if component.is_empty() || component == b"." {
                    index += 1;
                    continue;
                }
                if component == b".." {
                    return Err(ResolveError::Denied(CapabilityDenied::ScopeEscape));
                }
                let last = index + 1 == pending.len();
                let current = stack.last().expect("root handle").clone();
                if last && op.resolves_parent() {
                    if readlinkat_bytes(&current, &component).is_ok() {
                        if symlink == capabilities::SymlinkPolicy::NoFollow {
                            return Err(ResolveError::Denied(CapabilityDenied::SymlinkDenied));
                        }
                    }
                    return Ok(Resolution::Parent(current, component));
                }
                let request = if !last || op == FsOpKind::Enumerate {
                    HostOpenRequest::ReadDirectory
                } else if matches!(op, FsOpKind::Write | FsOpKind::Append) {
                    HostOpenRequest::ReadWrite
                } else {
                    HostOpenRequest::Read
                };
                match openat_handle(&current, &component, request) {
                    Ok(handle) => {
                        if last {
                            return Ok(Resolution::Existing(handle));
                        }
                        stack.push(handle);
                        index += 1;
                    }
                    Err(error) => match readlinkat_bytes(&current, &component) {
                        Ok(target) => {
                            if symlink == capabilities::SymlinkPolicy::NoFollow {
                                return Err(ResolveError::Denied(CapabilityDenied::SymlinkDenied));
                            }
                            symlink_hops += 1;
                            if symlink_hops > 40 {
                                return Err(ResolveError::Denied(CapabilityDenied::SymlinkDenied));
                            }
                            if target.starts_with(b"/") {
                                return Err(ResolveError::Denied(CapabilityDenied::ScopeEscape));
                            }
                            let mut replacement = Vec::new();
                            for part in target.split(|byte| *byte == b'/') {
                                if part.is_empty() || part == b"." {
                                    continue;
                                }
                                if part == b".." {
                                    if stack.len() == 1 {
                                        return Err(ResolveError::Denied(
                                            CapabilityDenied::ScopeEscape,
                                        ));
                                    }
                                    stack.pop();
                                } else {
                                    replacement.push(part.to_vec());
                                }
                            }
                            replacement.extend_from_slice(&pending[index + 1..]);
                            pending = replacement;
                            index = 0;
                        }
                        Err(_)
                            if error.kind() == io::ErrorKind::NotFound
                                && last
                                && matches!(op, FsOpKind::Write | FsOpKind::Append) =>
                        {
                            return Ok(Resolution::Parent(current, component));
                        }
                        Err(_) => return Err(ResolveError::Io(error)),
                    },
                }
            }
            Ok(Resolution::Existing(
                stack.last().expect("root handle").clone(),
            ))
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (root, components, op, symlink);
            Err(ResolveError::Io(host_abi_unsupported()))
        }
    }

    fn fs_read_at(&mut self, handle: &Self::Handle) -> io::Result<Vec<u8>> {
        #[cfg(target_os = "linux")]
        {
            Ok(ken_host::read(handle)?)
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = handle;
            Err(host_abi_unsupported())
        }
    }

    fn fs_write_at(
        &mut self,
        handle: &Self::Handle,
        policy: HostCreatePolicy,
        bytes: &[u8],
    ) -> io::Result<()> {
        #[cfg(target_os = "linux")]
        {
            if policy == HostCreatePolicy::CreateNew {
                return Err(io::Error::from(io::ErrorKind::AlreadyExists));
            }
            if policy == HostCreatePolicy::CreateOrKeep {
                return Ok(());
            }
            Ok(ken_host::replace(handle, bytes)?)
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (handle, policy, bytes);
            Err(host_abi_unsupported())
        }
    }

    fn fs_create_file_at(
        &mut self,
        parent: &Self::Handle,
        leaf: &[u8],
        policy: HostCreatePolicy,
        bytes: &[u8],
    ) -> io::Result<()> {
        #[cfg(target_os = "linux")]
        {
            let request = match policy {
                HostCreatePolicy::CreateNew => HostOpenRequest::CreateNew,
                HostCreatePolicy::CreateOrTruncate => HostOpenRequest::CreateOrTruncate,
                HostCreatePolicy::CreateOrKeep => HostOpenRequest::CreateOrKeep,
            };
            match openat_handle(parent, leaf, request) {
                Ok(handle) => Ok(ken_host::write_new(&handle, bytes)?),
                Err(error)
                    if policy == HostCreatePolicy::CreateOrKeep
                        && error.kind() == io::ErrorKind::AlreadyExists =>
                {
                    Ok(())
                }
                Err(error) => Err(error),
            }
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (parent, leaf, policy, bytes);
            Err(host_abi_unsupported())
        }
    }

    fn fs_append_at(&mut self, handle: &Self::Handle, bytes: &[u8]) -> io::Result<()> {
        #[cfg(target_os = "linux")]
        {
            Ok(ken_host::append(handle, bytes)?)
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (handle, bytes);
            Err(host_abi_unsupported())
        }
    }

    fn fs_create_append_at(
        &mut self,
        parent: &Self::Handle,
        leaf: &[u8],
        bytes: &[u8],
    ) -> io::Result<()> {
        #[cfg(target_os = "linux")]
        {
            let handle = openat_handle(parent, leaf, HostOpenRequest::AppendOrCreate)?;
            Ok(ken_host::append(&handle, bytes)?)
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (parent, leaf, bytes);
            Err(host_abi_unsupported())
        }
    }

    fn fs_metadata_at(&mut self, handle: &Self::Handle) -> io::Result<HostFileMetadata> {
        #[cfg(target_os = "linux")]
        {
            let metadata = ken_host::metadata(handle)?;
            Ok(HostFileMetadata {
                size: metadata.size,
                kind: match metadata.kind {
                    ken_host::FileKind::File => HostFileKind::File,
                    ken_host::FileKind::Directory => HostFileKind::Directory,
                    ken_host::FileKind::Symlink => HostFileKind::Symlink,
                    ken_host::FileKind::Other => HostFileKind::Other,
                },
            })
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = handle;
            Err(host_abi_unsupported())
        }
    }

    fn fs_read_directory_at(&mut self, handle: &Self::Handle) -> io::Result<Vec<HostDirEntry>> {
        #[cfg(target_os = "linux")]
        {
            Ok(ken_host::read_directory(handle)?
                .into_iter()
                .map(|entry| HostDirEntry {
                    name: entry.name,
                    kind: match entry.kind {
                        ken_host::FileKind::File => HostFileKind::File,
                        ken_host::FileKind::Directory => HostFileKind::Directory,
                        ken_host::FileKind::Symlink => HostFileKind::Symlink,
                        ken_host::FileKind::Other => HostFileKind::Other,
                    },
                })
                .collect())
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = handle;
            Err(host_abi_unsupported())
        }
    }

    fn fs_create_directory_at(
        &mut self,
        parent: &Self::Handle,
        leaf: &[u8],
        _recursive: bool,
    ) -> io::Result<()> {
        #[cfg(target_os = "linux")]
        {
            Ok(ken_host::create_directory(
                parent,
                &PathComponent::new(leaf)?,
            )?)
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (parent, leaf);
            Err(host_abi_unsupported())
        }
    }

    fn fs_remove_file_at(&mut self, parent: &Self::Handle, leaf: &[u8]) -> io::Result<()> {
        #[cfg(target_os = "linux")]
        {
            Ok(ken_host::remove(
                parent,
                &PathComponent::new(leaf)?,
                RemoveKind::File,
            )?)
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (parent, leaf);
            Err(host_abi_unsupported())
        }
    }

    fn fs_remove_directory_at(
        &mut self,
        parent: &Self::Handle,
        leaf: &[u8],
        recursive: bool,
    ) -> io::Result<()> {
        #[cfg(target_os = "linux")]
        {
            if recursive {
                return Ok(ken_host::remove_directory_tree(
                    parent,
                    &PathComponent::new(leaf)?,
                )?);
            }
            Ok(ken_host::remove(
                parent,
                &PathComponent::new(leaf)?,
                RemoveKind::Directory,
            )?)
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (parent, leaf, recursive);
            Err(host_abi_unsupported())
        }
    }

    fn fs_rename_at(
        &mut self,
        from_parent: &Self::Handle,
        from_leaf: &[u8],
        to_parent: &Self::Handle,
        to_leaf: &[u8],
    ) -> io::Result<()> {
        #[cfg(target_os = "linux")]
        {
            Ok(ken_host::rename(
                from_parent,
                &PathComponent::new(from_leaf)?,
                to_parent,
                &PathComponent::new(to_leaf)?,
            )?)
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (from_parent, from_leaf, to_parent, to_leaf);
            Err(host_abi_unsupported())
        }
    }
}

/// Deterministic in-memory Console provider used by tests and embedding.
pub struct CaptureHost {
    stdin: Vec<u8>,
    stdin_cursor: usize,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    terminals: [bool; 3],
    closed: [bool; 3],
    trace: Vec<ConsoleTrace>,
    clock_script: Vec<BigInt>,
    clock_cursor: usize,
    clock_trace: Vec<ClockTrace>,
    fs_nodes: BTreeMap<VfsNodeId, VirtualFsNode>,
    fs_entries: BTreeMap<(VfsNodeId, Vec<u8>), VfsNodeId>,
    fs_parents: BTreeMap<VfsNodeId, (VfsNodeId, Vec<u8>)>,
    next_fs_node: VfsNodeId,
    fs_trace: Vec<FsTrace>,
    fs_denials: Vec<CapabilityDenied>,
    fs_resolve_count: usize,
    after_resolve_replace: Option<CaptureAfterResolveReplace>,
}

struct CaptureAfterResolveReplace {
    from: Vec<u8>,
    to: Vec<u8>,
    replacement_file: Vec<u8>,
    replacement_bytes: Vec<u8>,
}

impl CaptureHost {
    pub fn new(stdin: Vec<u8>) -> Self {
        Self {
            stdin,
            stdin_cursor: 0,
            stdout: Vec::new(),
            stderr: Vec::new(),
            terminals: [false; 3],
            closed: [false; 3],
            trace: Vec::new(),
            clock_script: vec![BigInt::from(0)],
            clock_cursor: 0,
            clock_trace: Vec::new(),
            fs_nodes: [(0, VirtualFsNode::Directory)].into_iter().collect(),
            fs_entries: BTreeMap::new(),
            fs_parents: BTreeMap::new(),
            next_fs_node: 1,
            fs_trace: Vec::new(),
            fs_denials: Vec::new(),
            fs_resolve_count: 0,
            after_resolve_replace: None,
        }
    }

    pub fn set_terminal(&mut self, stream: ConsoleStream, value: bool) {
        self.terminals[stream_index(stream)] = value;
    }

    pub fn close(&mut self, stream: ConsoleStream) {
        self.closed[stream_index(stream)] = true;
    }

    pub fn stdout(&self) -> &[u8] {
        &self.stdout
    }

    pub fn stderr(&self) -> &[u8] {
        &self.stderr
    }

    pub fn trace(&self) -> &[ConsoleTrace] {
        &self.trace
    }

    /// Replace the deterministic wall-clock script. After the script is
    /// exhausted, reads remain fixed at its final value.
    pub fn set_clock_script(&mut self, nanoseconds: impl IntoIterator<Item = i128>) {
        self.clock_script = nanoseconds.into_iter().map(BigInt::from).collect();
        if self.clock_script.is_empty() {
            self.clock_script.push(BigInt::from(0));
        }
        self.clock_cursor = 0;
        self.clock_trace.clear();
    }

    /// Configure one fixed wall-clock value for every read.
    pub fn set_fixed_clock(&mut self, nanoseconds: i128) {
        self.set_clock_script([nanoseconds]);
    }

    pub fn clock_trace(&self) -> &[ClockTrace] {
        &self.clock_trace
    }

    pub fn insert_file(&mut self, path: impl Into<Vec<u8>>, bytes: impl Into<Vec<u8>>) {
        self.insert_virtual(path.into(), VirtualFsNode::File(bytes.into()));
    }

    pub fn insert_directory(&mut self, path: impl Into<Vec<u8>>) {
        self.insert_virtual(path.into(), VirtualFsNode::Directory);
    }

    pub fn insert_symlink(&mut self, path: impl Into<Vec<u8>>, target: impl Into<Vec<u8>>) {
        self.insert_virtual(path.into(), VirtualFsNode::Symlink(target.into()));
    }

    pub fn fs_nodes(&self) -> BTreeMap<Vec<u8>, VirtualFsNode> {
        self.fs_nodes
            .iter()
            .filter_map(|(id, node)| {
                if *id == 0 {
                    None
                } else {
                    Some((self.virtual_path(*id), node.clone()))
                }
            })
            .collect()
    }

    pub fn fs_trace(&self) -> &[FsTrace] {
        &self.fs_trace
    }

    pub fn fs_denials(&self) -> &[CapabilityDenied] {
        &self.fs_denials
    }

    pub fn fs_resolve_count(&self) -> usize {
        self.fs_resolve_count
    }

    pub fn replace_subtree_after_next_resolve(
        &mut self,
        from: impl Into<Vec<u8>>,
        to: impl Into<Vec<u8>>,
        replacement_file: impl Into<Vec<u8>>,
        replacement_bytes: impl Into<Vec<u8>>,
    ) {
        self.after_resolve_replace = Some(CaptureAfterResolveReplace {
            from: from.into(),
            to: to.into(),
            replacement_file: replacement_file.into(),
            replacement_bytes: replacement_bytes.into(),
        });
    }

    pub fn mint_fs_cap(&self, authority: capabilities::Authority) -> capabilities::Cap {
        self.mint_scoped_fs_cap(
            authority,
            b"",
            if authority == capabilities::AUTH_FULL {
                capabilities::RightSet::ALL
            } else if authority == capabilities::AUTH_PARTIAL {
                capabilities::RightSet::READ
                    .union(capabilities::RightSet::ENUMERATE)
                    .union(capabilities::RightSet::METADATA)
            } else {
                capabilities::RightSet::NONE
            },
            capabilities::SymlinkPolicy::NoFollow,
        )
        .expect("capture root exists")
    }

    pub fn mint_scoped_fs_cap(
        &self,
        authority: capabilities::Authority,
        path: &[u8],
        rights: capabilities::RightSet,
        symlink: capabilities::SymlinkPolicy,
    ) -> io::Result<capabilities::Cap> {
        let mut node = 0;
        let mut lineage = vec![capabilities::FsIdentity::Virtual(0)];
        for component in path
            .split(|byte| *byte == b'/')
            .filter(|part| !part.is_empty())
        {
            node = *self
                .fs_entries
                .get(&(node, component.to_vec()))
                .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
            lineage.push(capabilities::FsIdentity::Virtual(node));
        }
        Ok(capabilities::Cap::mint_scoped(
            authority,
            "FS",
            capabilities::FsScope {
                rights,
                root: capabilities::FsHandle::Virtual(node),
                lineage,
                symlink,
                empty: false,
            },
        ))
    }

    fn insert_virtual(&mut self, path: Vec<u8>, node: VirtualFsNode) -> VfsNodeId {
        let mut parent = 0;
        let components: Vec<_> = path
            .split(|byte| *byte == b'/')
            .filter(|part| !part.is_empty())
            .map(<[u8]>::to_vec)
            .collect();
        let (leaf, ancestors) = components.split_last().expect("virtual path is non-empty");
        for component in ancestors {
            parent = if let Some(id) = self.fs_entries.get(&(parent, component.clone())) {
                *id
            } else {
                let id = self.next_fs_node;
                self.next_fs_node += 1;
                self.fs_nodes.insert(id, VirtualFsNode::Directory);
                self.fs_entries.insert((parent, component.clone()), id);
                self.fs_parents.insert(id, (parent, component.clone()));
                id
            };
        }
        if let Some(id) = self.fs_entries.get(&(parent, leaf.clone())).copied() {
            self.fs_nodes.insert(id, node);
            id
        } else {
            let id = self.next_fs_node;
            self.next_fs_node += 1;
            self.fs_nodes.insert(id, node);
            self.fs_entries.insert((parent, leaf.clone()), id);
            self.fs_parents.insert(id, (parent, leaf.clone()));
            id
        }
    }

    fn virtual_path(&self, mut node: VfsNodeId) -> Vec<u8> {
        let mut parts = Vec::new();
        while let Some((parent, name)) = self.fs_parents.get(&node) {
            parts.push(name.clone());
            node = *parent;
        }
        parts.reverse();
        parts.join(&b'/')
    }

    fn parent_and_leaf(&self, path: &[u8]) -> Option<(VfsNodeId, Vec<u8>)> {
        let mut parts = path
            .split(|byte| *byte == b'/')
            .filter(|part| !part.is_empty())
            .peekable();
        let mut parent = 0;
        while let Some(part) = parts.next() {
            if parts.peek().is_none() {
                return Some((parent, part.to_vec()));
            }
            parent = *self.fs_entries.get(&(parent, part.to_vec()))?;
        }
        None
    }
}

impl HostHandler for CaptureHost {
    type Handle = VfsNodeId;

    fn mint_fs_cap(&self, authority: capabilities::Authority) -> capabilities::Cap {
        CaptureHost::mint_fs_cap(self, authority)
    }

    fn console_read(&mut self, stream: ConsoleStream, limit: usize) -> io::Result<HostRead> {
        self.trace.push(ConsoleTrace::Read { stream, limit });
        if stream != ConsoleStream::Stdin {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "stream is not readable",
            ));
        }
        if self.stdin_cursor >= self.stdin.len() {
            return Ok(HostRead::Eof);
        }
        let end = self
            .stdin_cursor
            .saturating_add(limit)
            .min(self.stdin.len());
        let bytes = self.stdin[self.stdin_cursor..end].to_vec();
        self.stdin_cursor = end;
        Ok(HostRead::Chunk(bytes))
    }

    fn console_write(&mut self, stream: ConsoleStream, bytes: &[u8]) -> io::Result<()> {
        self.trace.push(ConsoleTrace::Write {
            stream,
            bytes: bytes.to_vec(),
        });
        if self.closed[stream_index(stream)] {
            return Err(io::Error::from(io::ErrorKind::BrokenPipe));
        }
        match stream {
            ConsoleStream::Stdout => self.stdout.extend_from_slice(bytes),
            ConsoleStream::Stderr => self.stderr.extend_from_slice(bytes),
            ConsoleStream::Stdin => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "stream is not writable",
                ));
            }
        }
        Ok(())
    }

    fn console_flush(&mut self, stream: ConsoleStream) -> io::Result<()> {
        self.trace.push(ConsoleTrace::Flush { stream });
        if self.closed[stream_index(stream)] {
            Err(io::Error::from(io::ErrorKind::BrokenPipe))
        } else {
            Ok(())
        }
    }

    fn console_is_terminal(&mut self, stream: ConsoleStream) -> bool {
        self.trace.push(ConsoleTrace::IsTerminal { stream });
        self.terminals[stream_index(stream)]
    }

    fn clock_wall_now(&mut self) -> BigInt {
        let index = self.clock_cursor.min(self.clock_script.len() - 1);
        let nanoseconds = self.clock_script[index].clone();
        self.clock_cursor = self.clock_cursor.saturating_add(1);
        self.clock_trace.push(ClockTrace::WallNow {
            nanoseconds: nanoseconds.clone(),
        });
        nanoseconds
    }

    fn fs_denied(&mut self, denial: CapabilityDenied) {
        self.fs_denials.push(denial);
    }

    fn fs_after_resolve(&mut self) {
        let Some(replacement) = self.after_resolve_replace.take() else {
            return;
        };
        let (from_parent, from_leaf) = self
            .parent_and_leaf(&replacement.from)
            .expect("hook source parent exists");
        let (to_parent, to_leaf) = self
            .parent_and_leaf(&replacement.to)
            .expect("hook destination parent exists");
        let node = self
            .fs_entries
            .remove(&(from_parent, from_leaf))
            .expect("hook source exists");
        self.fs_entries.insert((to_parent, to_leaf.clone()), node);
        self.fs_parents.insert(node, (to_parent, to_leaf));
        self.insert_directory(replacement.from);
        self.insert_file(replacement.replacement_file, replacement.replacement_bytes);
    }

    fn fs_resolve(
        &mut self,
        root: &capabilities::FsHandle,
        components: &[Vec<u8>],
        op: FsOpKind,
        symlink: capabilities::SymlinkPolicy,
    ) -> Result<Resolution<Self::Handle>, ResolveError> {
        self.fs_resolve_count += 1;
        let capabilities::FsHandle::Virtual(root) = root else {
            return Err(ResolveError::Denied(CapabilityDenied::ScopeEscape));
        };
        let mut stack = vec![*root];
        let mut pending = components.to_vec();
        let mut index = 0;
        let mut symlink_hops = 0usize;
        while index < pending.len() {
            let part = pending[index].clone();
            if part.is_empty() || part == b"." {
                index += 1;
                continue;
            }
            if part == b".." {
                return Err(ResolveError::Denied(CapabilityDenied::ScopeEscape));
            }
            let current = *stack.last().expect("root node");
            let last = index + 1 == pending.len();
            if last && op.resolves_parent() {
                if let Some(id) = self.fs_entries.get(&(current, part.clone())) {
                    if matches!(self.fs_nodes.get(id), Some(VirtualFsNode::Symlink(_)))
                        && symlink == capabilities::SymlinkPolicy::NoFollow
                    {
                        return Err(ResolveError::Denied(CapabilityDenied::SymlinkDenied));
                    }
                }
                return Ok(Resolution::Parent(current, part));
            }
            let Some(child) = self.fs_entries.get(&(current, part.clone())).copied() else {
                if last && matches!(op, FsOpKind::Write | FsOpKind::Append) {
                    return Ok(Resolution::Parent(current, part));
                }
                return Err(ResolveError::Io(io::Error::from(io::ErrorKind::NotFound)));
            };
            match self.fs_nodes.get(&child) {
                Some(VirtualFsNode::Symlink(target)) => {
                    if symlink == capabilities::SymlinkPolicy::NoFollow {
                        return Err(ResolveError::Denied(CapabilityDenied::SymlinkDenied));
                    }
                    symlink_hops += 1;
                    if symlink_hops > 40 {
                        return Err(ResolveError::Denied(CapabilityDenied::SymlinkDenied));
                    }
                    if target.starts_with(b"/") {
                        return Err(ResolveError::Denied(CapabilityDenied::ScopeEscape));
                    }
                    let mut replacement = Vec::new();
                    for target_part in target.split(|byte| *byte == b'/') {
                        if target_part.is_empty() || target_part == b"." {
                            continue;
                        }
                        if target_part == b".." {
                            if stack.len() == 1 {
                                return Err(ResolveError::Denied(CapabilityDenied::ScopeEscape));
                            }
                            stack.pop();
                        } else {
                            replacement.push(target_part.to_vec());
                        }
                    }
                    replacement.extend_from_slice(&pending[index + 1..]);
                    pending = replacement;
                    index = 0;
                }
                Some(VirtualFsNode::Directory) if !last => {
                    stack.push(child);
                    index += 1;
                }
                Some(_) if last => return Ok(Resolution::Existing(child)),
                Some(_) => {
                    return Err(ResolveError::Io(io::Error::from(
                        io::ErrorKind::NotADirectory,
                    )))
                }
                None => return Err(ResolveError::Io(io::Error::from(io::ErrorKind::NotFound))),
            }
        }
        Ok(Resolution::Existing(*stack.last().expect("root node")))
    }

    fn fs_read_at(&mut self, handle: &Self::Handle) -> io::Result<Vec<u8>> {
        self.fs_trace.push(FsTrace::ReadFile {
            path: self.virtual_path(*handle),
        });
        match self.fs_nodes.get(handle) {
            Some(VirtualFsNode::File(bytes)) => Ok(bytes.clone()),
            Some(VirtualFsNode::Directory) => Err(io::Error::from(io::ErrorKind::IsADirectory)),
            _ => Err(io::Error::from(io::ErrorKind::NotFound)),
        }
    }

    fn fs_write_at(
        &mut self,
        handle: &Self::Handle,
        policy: HostCreatePolicy,
        bytes: &[u8],
    ) -> io::Result<()> {
        let path = self.virtual_path(*handle);
        self.fs_trace.push(FsTrace::WriteFile {
            path,
            policy,
            bytes: bytes.to_vec(),
        });
        match self.fs_nodes.get_mut(handle) {
            Some(VirtualFsNode::File(contents)) => match policy {
                HostCreatePolicy::CreateNew => Err(io::Error::from(io::ErrorKind::AlreadyExists)),
                HostCreatePolicy::CreateOrKeep => Ok(()),
                HostCreatePolicy::CreateOrTruncate => {
                    *contents = bytes.to_vec();
                    Ok(())
                }
            },
            Some(VirtualFsNode::Directory) => Err(io::Error::from(io::ErrorKind::IsADirectory)),
            _ => Err(io::Error::from(io::ErrorKind::NotFound)),
        }
    }

    fn fs_create_file_at(
        &mut self,
        parent: &Self::Handle,
        leaf: &[u8],
        policy: HostCreatePolicy,
        bytes: &[u8],
    ) -> io::Result<()> {
        let mut path = self.virtual_path(*parent);
        if !path.is_empty() {
            path.push(b'/');
        }
        path.extend_from_slice(leaf);
        self.fs_trace.push(FsTrace::WriteFile {
            path,
            policy,
            bytes: bytes.to_vec(),
        });
        if self.fs_entries.contains_key(&(*parent, leaf.to_vec())) {
            return if policy == HostCreatePolicy::CreateOrKeep {
                Ok(())
            } else {
                Err(io::Error::from(io::ErrorKind::AlreadyExists))
            };
        }
        let id = self.next_fs_node;
        self.next_fs_node += 1;
        self.fs_nodes
            .insert(id, VirtualFsNode::File(bytes.to_vec()));
        self.fs_entries.insert((*parent, leaf.to_vec()), id);
        self.fs_parents.insert(id, (*parent, leaf.to_vec()));
        Ok(())
    }

    fn fs_append_at(&mut self, handle: &Self::Handle, bytes: &[u8]) -> io::Result<()> {
        let path = self.virtual_path(*handle);
        self.fs_trace.push(FsTrace::AppendFile {
            path,
            bytes: bytes.to_vec(),
        });
        match self.fs_nodes.get_mut(handle) {
            Some(VirtualFsNode::File(contents)) => {
                contents.extend_from_slice(bytes);
                Ok(())
            }
            Some(VirtualFsNode::Directory) => Err(io::Error::from(io::ErrorKind::IsADirectory)),
            _ => Err(io::Error::from(io::ErrorKind::NotFound)),
        }
    }

    fn fs_create_append_at(
        &mut self,
        parent: &Self::Handle,
        leaf: &[u8],
        bytes: &[u8],
    ) -> io::Result<()> {
        self.fs_create_file_at(parent, leaf, HostCreatePolicy::CreateOrTruncate, bytes)
    }

    fn fs_metadata_at(&mut self, handle: &Self::Handle) -> io::Result<HostFileMetadata> {
        self.fs_trace.push(FsTrace::Metadata {
            path: self.virtual_path(*handle),
        });
        match self.fs_nodes.get(handle) {
            Some(VirtualFsNode::File(bytes)) => Ok(HostFileMetadata {
                size: bytes.len() as u64,
                kind: HostFileKind::File,
            }),
            Some(VirtualFsNode::Directory) => Ok(HostFileMetadata {
                size: 0,
                kind: HostFileKind::Directory,
            }),
            Some(VirtualFsNode::Symlink(target)) => Ok(HostFileMetadata {
                size: target.len() as u64,
                kind: HostFileKind::Symlink,
            }),
            None => Err(io::Error::from(io::ErrorKind::NotFound)),
        }
    }

    fn fs_read_directory_at(&mut self, handle: &Self::Handle) -> io::Result<Vec<HostDirEntry>> {
        self.fs_trace.push(FsTrace::ReadDirectory {
            path: self.virtual_path(*handle),
        });
        if !matches!(self.fs_nodes.get(handle), Some(VirtualFsNode::Directory)) {
            return Err(io::Error::from(io::ErrorKind::NotADirectory));
        }
        Ok(self
            .fs_entries
            .iter()
            .filter_map(|((parent, name), id)| {
                (*parent == *handle).then(|| HostDirEntry {
                    name: name.clone(),
                    kind: virtual_kind(self.fs_nodes.get(id).expect("entry node")),
                })
            })
            .collect())
    }

    fn fs_create_directory_at(
        &mut self,
        parent: &Self::Handle,
        leaf: &[u8],
        recursive: bool,
    ) -> io::Result<()> {
        let mut path = self.virtual_path(*parent);
        if !path.is_empty() {
            path.push(b'/');
        }
        path.extend_from_slice(leaf);
        self.fs_trace
            .push(FsTrace::CreateDirectory { path, recursive });
        if self.fs_entries.contains_key(&(*parent, leaf.to_vec())) {
            return Err(io::Error::from(io::ErrorKind::AlreadyExists));
        }
        let id = self.next_fs_node;
        self.next_fs_node += 1;
        self.fs_nodes.insert(id, VirtualFsNode::Directory);
        self.fs_entries.insert((*parent, leaf.to_vec()), id);
        self.fs_parents.insert(id, (*parent, leaf.to_vec()));
        Ok(())
    }

    fn fs_remove_file_at(&mut self, parent: &Self::Handle, leaf: &[u8]) -> io::Result<()> {
        let Some(id) = self.fs_entries.get(&(*parent, leaf.to_vec())).copied() else {
            return Err(io::Error::from(io::ErrorKind::NotFound));
        };
        let path = self.virtual_path(id);
        self.fs_trace.push(FsTrace::RemoveFile { path });
        if matches!(self.fs_nodes.get(&id), Some(VirtualFsNode::Directory)) {
            return Err(io::Error::from(io::ErrorKind::IsADirectory));
        }
        self.fs_entries.remove(&(*parent, leaf.to_vec()));
        self.fs_parents.remove(&id);
        self.fs_nodes.remove(&id);
        Ok(())
    }

    fn fs_remove_directory_at(
        &mut self,
        parent: &Self::Handle,
        leaf: &[u8],
        recursive: bool,
    ) -> io::Result<()> {
        let Some(id) = self.fs_entries.get(&(*parent, leaf.to_vec())).copied() else {
            return Err(io::Error::from(io::ErrorKind::NotFound));
        };
        let path = self.virtual_path(id);
        self.fs_trace
            .push(FsTrace::RemoveDirectory { path, recursive });
        if !matches!(self.fs_nodes.get(&id), Some(VirtualFsNode::Directory)) {
            return Err(io::Error::from(io::ErrorKind::NotADirectory));
        }
        let children: Vec<_> = self
            .fs_entries
            .iter()
            .filter_map(|((p, n), child)| (*p == id).then_some((n.clone(), *child)))
            .collect();
        if !recursive && !children.is_empty() {
            return Err(io::Error::from(io::ErrorKind::DirectoryNotEmpty));
        }
        fn remove_tree(host: &mut CaptureHost, id: VfsNodeId) {
            let children: Vec<_> = host
                .fs_entries
                .iter()
                .filter_map(|((p, n), child)| (*p == id).then_some((n.clone(), *child)))
                .collect();
            for (name, child) in children {
                host.fs_entries.remove(&(id, name));
                remove_tree(host, child);
            }
            host.fs_parents.remove(&id);
            host.fs_nodes.remove(&id);
        }
        self.fs_entries.remove(&(*parent, leaf.to_vec()));
        remove_tree(self, id);
        Ok(())
    }

    fn fs_rename_at(
        &mut self,
        from_parent: &Self::Handle,
        from_leaf: &[u8],
        to_parent: &Self::Handle,
        to_leaf: &[u8],
    ) -> io::Result<()> {
        let Some(id) = self.fs_entries.remove(&(*from_parent, from_leaf.to_vec())) else {
            return Err(io::Error::from(io::ErrorKind::NotFound));
        };
        let mut from = self.virtual_path(id); // parent map still names the old entry
        let mut to = self.virtual_path(*to_parent);
        if !to.is_empty() {
            to.push(b'/');
        }
        to.extend_from_slice(to_leaf);
        self.fs_trace.push(FsTrace::Rename {
            from: std::mem::take(&mut from),
            to,
        });
        self.fs_entries.insert((*to_parent, to_leaf.to_vec()), id);
        self.fs_parents.insert(id, (*to_parent, to_leaf.to_vec()));
        Ok(())
    }
}

fn virtual_kind(node: &VirtualFsNode) -> HostFileKind {
    match node {
        VirtualFsNode::File(_) => HostFileKind::File,
        VirtualFsNode::Directory => HostFileKind::Directory,
        VirtualFsNode::Symlink(_) => HostFileKind::Symlink,
    }
}

fn stream_index(stream: ConsoleStream) -> usize {
    match stream {
        ConsoleStream::Stdin => 0,
        ConsoleStream::Stdout => 1,
        ConsoleStream::Stderr => 2,
    }
}

/// Recursively strip `InL`/`InR` wrappers off an op value, returning the
/// innermost non-`Coproduct` base tag (`effect-composition` D3.2). `InL`/`InR`'s
/// `ctor_arity` = 2 params (`g,h`) + 1 arg (the payload) = 3, so the payload
/// sits at `op_args[2]` (this Coproduct-peel index is distinct from the FS arm's
/// own shifted `op_args[1]`/`[2]` — those index into the ALREADY-peeled base
/// op, not the `Coproduct` wrapper). Zero-wrapper trees (State/FS/Console alone)
/// pass through unchanged — a total no-op descent; `coproduct_ids = None` disables
/// peeling entirely (pre-composition callers, BV6).
fn peel_coproduct(mut op: EvalVal, coproduct_ids: Option<&CoproductIds>) -> EvalVal {
    let Some(coproduct_ids) = coproduct_ids else {
        return op;
    };
    loop {
        match &op {
            EvalVal::Ctor { id, args, .. }
                if *id == coproduct_ids.inl_id || *id == coproduct_ids.inr_id =>
            {
                match args.get(2) {
                    Some(payload) => op = payload.clone(),
                    // Malformed arity — leave as-is; the base-tag match below
                    // fails closed (UnknownEffect), never a panic.
                    None => return op,
                }
            }
            _ => return op,
        }
    }
}

/// Error returned by `run_io` (`42 §6`).
#[derive(Debug)]
pub enum RunIoError {
    /// A `Vis` node carried an op-tag outside the supported host algebra.
    UnknownEffect(EvalVal),
    /// The tree evaluated to `Unknown` (open hole, `42 §6.7`).
    UnknownTree,
    /// The tree is not an ITree `Ret`/`Vis` value.
    NotAnIOTree(EvalVal),
}

/// IDs for the `[FS]` effect driver arm (FS-driver-build D1/D2). Shares
/// `ConsoleIds`'s `itree_id`/`ret_id`/`vis_id`/`params_len` (one `ITree`,
/// reused — not a second effect system); this struct carries only the
/// FS-specific ctor ids the driver needs to decode the op and build the
/// `Result`/`IOError` response.
#[derive(Clone)]
pub struct FSIds {
    /// `GlobalId` of `FSOp::ReadFile` (carries `[Cap, Bytes]` — the
    /// capability + path, capability-*carrying*, not ambient authority).
    pub readfile_id: GlobalId,
    pub writefile_id: GlobalId,
    pub appendfile_id: GlobalId,
    pub metadata_id: GlobalId,
    pub readdirectory_id: GlobalId,
    pub createdirectory_id: GlobalId,
    pub removefile_id: GlobalId,
    pub removedirectory_id: GlobalId,
    pub rename_id: GlobalId,
    pub create_new_id: GlobalId,
    pub create_or_truncate_id: GlobalId,
    pub create_or_keep_id: GlobalId,
    pub mk_file_error_id: GlobalId,
    pub some_id: GlobalId,
    pub op_read_file_id: GlobalId,
    pub op_write_file_id: GlobalId,
    pub op_append_file_id: GlobalId,
    pub op_metadata_id: GlobalId,
    pub op_read_directory_id: GlobalId,
    pub op_create_directory_id: GlobalId,
    pub op_remove_file_id: GlobalId,
    pub op_remove_directory_id: GlobalId,
    pub op_rename_id: GlobalId,
    pub mk_file_metadata_id: GlobalId,
    pub mk_dir_entry_id: GlobalId,
    pub k_file_id: GlobalId,
    pub k_directory_id: GlobalId,
    pub k_symlink_id: GlobalId,
    pub k_other_id: GlobalId,
    pub nil_id: GlobalId,
    pub cons_id: GlobalId,
}

impl FSIds {
    pub fn from_elab(elab: &ken_elaborator::ElabEnv) -> Option<Self> {
        let get = |name: &str| elab.globals.get(name).copied();
        Some(Self {
            readfile_id: get("ReadFile")?,
            writefile_id: get("WriteFile")?,
            appendfile_id: get("AppendFile")?,
            metadata_id: get("Metadata")?,
            readdirectory_id: get("ReadDirectory")?,
            createdirectory_id: get("CreateDirectory")?,
            removefile_id: get("RemoveFile")?,
            removedirectory_id: get("RemoveDirectory")?,
            rename_id: get("Rename")?,
            create_new_id: get("CreateNew")?,
            create_or_truncate_id: get("CreateOrTruncate")?,
            create_or_keep_id: get("CreateOrKeep")?,
            mk_file_error_id: get("MkFileError")?,
            some_id: get("Some")?,
            op_read_file_id: get("OpReadFile")?,
            op_write_file_id: get("OpWriteFile")?,
            op_append_file_id: get("OpAppendFile")?,
            op_metadata_id: get("OpMetadata")?,
            op_read_directory_id: get("OpReadDirectory")?,
            op_create_directory_id: get("OpCreateDirectory")?,
            op_remove_file_id: get("OpRemoveFile")?,
            op_remove_directory_id: get("OpRemoveDirectory")?,
            op_rename_id: get("OpRename")?,
            mk_file_metadata_id: get("MkFileMetadata")?,
            mk_dir_entry_id: get("MkDirEntry")?,
            k_file_id: get("KFile")?,
            k_directory_id: get("KDirectory")?,
            k_symlink_id: get("KSymlink")?,
            k_other_id: get("KOther")?,
            nil_id: get("Nil")?,
            cons_id: get("Cons")?,
        })
    }
}

/// The authority a `read_bytes` sink demands (`62 §3.1`'s sink-sufficiency
/// check). `AUTH_PARTIAL` ("restricted, e.g. read-only, single dir") is the
/// least authority that authorizes a read; `AUTH_NONE` never suffices.
const READ_BYTES_REQUIRED_AUTHORITY: capabilities::Authority = capabilities::AUTH_PARTIAL;
const WRITE_BYTES_REQUIRED_AUTHORITY: capabilities::Authority = capabilities::AUTH_FULL;

/// Runtime capability gate (FS-driver-build D3, `FS-driver.md` D3, AC3's
/// runtime arm). Load-bearing — R2 flips on this returning `false`; a
/// no-op always-true `authorizes` is ambient authority and fails AC3.
///
/// **Representation (fs-read-file-lines-flip D3, Architect ruling
/// `evt_35knjqv2k941h`): a real opaque `EvalVal::Cap(capabilities::Cap)`,
/// NOT the earlier `EvalVal::Int(level)` positional-scalar projection.**
/// Structural self-evidence over a non-local type-gate+reachability argument
/// for the runtime net — reads the authority, rights, and scoped root off the
/// REAL minted struct, with no re-mint from a bare scalar. Scope and symlink
/// confinement are then enforced by the handle-only resolver/operate seam.
///
/// **Trust level (AC8): trusted Rust, conformance-netted, NOT kernel-backed**
/// — this calls `capabilities::check_authority_sufficient`, a plain Rust
/// `bool`-returning check, zero `declare_postulate`/`Obligation` emission.
/// Distinct from `attenuate`'s static refinement obligation: that emitted
/// `Eq`+`Refl` discharge mirrors the elaborator-selected product meet and is
/// not an independent kernel proof of the attenuation bound.
pub fn check_fs_capability<'a>(
    cap: &'a EvalVal,
    op: FsOpKind,
    required: capabilities::Authority,
    _operation: &str,
) -> Result<&'a capabilities::FsScope, CapabilityDenied> {
    let cap = match cap {
        EvalVal::Cap(cap) => cap,
        // Malformed/non-Cap value carries no recognizable authority — fail
        // closed (BV3: a wrong `op_args` index lands here, over-rejects,
        // never a soundness hole).
        _ => return Err(CapabilityDenied::MalformedCapability),
    };
    ken_host::capability::check_fs_capability(cap, op, required)
}

pub fn fs_target_components(path: &[u8]) -> Result<Vec<Vec<u8>>, CapabilityDenied> {
    if path.starts_with(b"/") {
        return Err(CapabilityDenied::ScopeEscape);
    }
    let mut components = Vec::new();
    for component in path.split(|byte| *byte == b'/') {
        if component.is_empty() || component == b"." {
            continue;
        }
        if component == b".." {
            return Err(CapabilityDenied::ScopeEscape);
        }
        if component.contains(&0) {
            return Err(CapabilityDenied::ScopeEscape);
        }
        components.push(component.to_vec());
    }
    Ok(components)
}

/// Map a `std::io::ErrorKind` to Ken's in-language `IOError` sum (D2, D5:
/// failure surfaces as a total `Result`, never a panic).
fn io_error_value(error: &io::Error, ids: &ConsoleIds, store: &mut EvalStore) -> EvalVal {
    let ctor = match error.kind() {
        std::io::ErrorKind::NotFound => ids.notfound_id,
        std::io::ErrorKind::PermissionDenied => ids.permissiondenied_id,
        std::io::ErrorKind::BrokenPipe => ids.brokenpipe_id,
        std::io::ErrorKind::Interrupted => ids.interrupted_id,
        std::io::ErrorKind::AlreadyExists => ids.alreadyexists_id,
        std::io::ErrorKind::InvalidInput => ids.invalidinput_id,
        std::io::ErrorKind::IsADirectory => ids.isdirectory_id,
        std::io::ErrorKind::NotADirectory => ids.notdirectory_id,
        std::io::ErrorKind::DirectoryNotEmpty => ids.notempty_id,
        std::io::ErrorKind::Unsupported => ids.unsupported_id,
        _ => ids.other_id,
    };
    let args = if ctor == ids.other_id {
        vec![EvalVal::Int(error.raw_os_error().unwrap_or(0) as i64)]
    } else {
        vec![]
    };
    make_ctor(ctor, args, store)
}

fn file_error_value(
    operation_id: GlobalId,
    path: &[u8],
    error: EvalVal,
    fs: &FSIds,
    store: &mut EvalStore,
) -> EvalVal {
    let operation = make_ctor(operation_id, vec![], store);
    let path = make_ctor(
        fs.some_id,
        vec![EvalVal::Unknown, EvalVal::Bytes(path.to_vec())],
        store,
    );
    make_ctor(fs.mk_file_error_id, vec![operation, path, error], store)
}

fn file_result(
    result: io::Result<EvalVal>,
    operation_id: GlobalId,
    path: &[u8],
    fs: &FSIds,
    ids: &ConsoleIds,
    store: &mut EvalStore,
) -> EvalVal {
    match result {
        Ok(value) => make_result(true, value, ids, store),
        Err(error) => {
            let kind = io_error_value(&error, ids, store);
            let file_error = file_error_value(operation_id, path, kind, fs, store);
            make_result(false, file_error, ids, store)
        }
    }
}

/// Build a `Result e a` response `EvalVal` (`Result`'s 2 type
/// params fill `args[0..2]` as `Unknown`, mirroring every other landed
/// prelude ctor's type-param-then-payload shape — `ctor_arity` = params.len()
/// + args.len()). Untyped at this layer regardless — `make_result` puts
/// `payload` at position 2 for both `Ok`/`Err` ctors, unaffected by which
/// static field type the surface ascription assigns.
fn make_result(ok: bool, payload: EvalVal, ids: &ConsoleIds, store: &mut EvalStore) -> EvalVal {
    let ctor_id = if ok { ids.ok_id } else { ids.err_id };
    make_ctor(
        ctor_id,
        vec![EvalVal::Unknown, EvalVal::Unknown, payload],
        store,
    )
}

fn decode_stream(value: &EvalVal, ids: &ConsoleIds) -> Option<ConsoleStream> {
    match value {
        EvalVal::Ctor { id, .. } if *id == ids.stdin_id => Some(ConsoleStream::Stdin),
        EvalVal::Ctor { id, .. } if *id == ids.stdout_id => Some(ConsoleStream::Stdout),
        EvalVal::Ctor { id, .. } if *id == ids.stderr_id => Some(ConsoleStream::Stderr),
        _ => None,
    }
}

fn read_limit(value: &EvalVal) -> Option<usize> {
    const MAX_CONSOLE_READ: usize = 64 * 1024;
    let n = eval_to_bigint(value)?;
    if n.sign() == NumSign::Minus {
        Some(0)
    } else {
        Some(n.to_usize().unwrap_or(usize::MAX).min(MAX_CONSOLE_READ))
    }
}

fn fs_kind_value(kind: HostFileKind, fs: &FSIds, store: &mut EvalStore) -> EvalVal {
    let id = match kind {
        HostFileKind::File => fs.k_file_id,
        HostFileKind::Directory => fs.k_directory_id,
        HostFileKind::Symlink => fs.k_symlink_id,
        HostFileKind::Other => fs.k_other_id,
    };
    make_ctor(id, vec![], store)
}

fn fs_denied(
    operation_id: GlobalId,
    path: &[u8],
    fs: &FSIds,
    ids: &ConsoleIds,
    store: &mut EvalStore,
) -> EvalVal {
    let kind = make_ctor(ids.capabilitydenied_id, vec![], store);
    let error = file_error_value(operation_id, path, kind, fs, store);
    make_result(false, error, ids, store)
}

fn fs_dispatch<H: HostHandler>(
    op_id: GlobalId,
    args: &[EvalVal],
    handler: &mut H,
    fs: &FSIds,
    ids: &ConsoleIds,
    store: &mut EvalStore,
) -> Option<Result<EvalVal, ()>> {
    let cap = args.get(1)?;
    let bytes_at = |index| match args.get(index) {
        Some(EvalVal::Bytes(bytes)) => Some(bytes.as_slice()),
        _ => None,
    };
    let unit = |store: &mut EvalStore| make_ctor(ids.unit_id, vec![], store);

    let (path, operation, required, operation_name, op_kind) = if op_id == fs.readfile_id {
        (
            bytes_at(2)?,
            fs.op_read_file_id,
            READ_BYTES_REQUIRED_AUTHORITY,
            "fs_driver::read_file",
            FsOpKind::Read,
        )
    } else if op_id == fs.writefile_id {
        (
            bytes_at(2)?,
            fs.op_write_file_id,
            WRITE_BYTES_REQUIRED_AUTHORITY,
            "fs_driver::write_file",
            FsOpKind::Write,
        )
    } else if op_id == fs.appendfile_id {
        (
            bytes_at(2)?,
            fs.op_append_file_id,
            WRITE_BYTES_REQUIRED_AUTHORITY,
            "fs_driver::append_file",
            FsOpKind::Append,
        )
    } else if op_id == fs.metadata_id {
        (
            bytes_at(2)?,
            fs.op_metadata_id,
            READ_BYTES_REQUIRED_AUTHORITY,
            "fs_driver::metadata",
            FsOpKind::Metadata,
        )
    } else if op_id == fs.readdirectory_id {
        (
            bytes_at(2)?,
            fs.op_read_directory_id,
            READ_BYTES_REQUIRED_AUTHORITY,
            "fs_driver::read_directory",
            FsOpKind::Enumerate,
        )
    } else if op_id == fs.createdirectory_id {
        (
            bytes_at(3)?,
            fs.op_create_directory_id,
            WRITE_BYTES_REQUIRED_AUTHORITY,
            "fs_driver::create_directory",
            FsOpKind::CreateDirectory,
        )
    } else if op_id == fs.removefile_id {
        (
            bytes_at(2)?,
            fs.op_remove_file_id,
            WRITE_BYTES_REQUIRED_AUTHORITY,
            "fs_driver::remove_file",
            FsOpKind::RemoveFile,
        )
    } else if op_id == fs.removedirectory_id {
        (
            bytes_at(3)?,
            fs.op_remove_directory_id,
            WRITE_BYTES_REQUIRED_AUTHORITY,
            "fs_driver::remove_directory",
            FsOpKind::RemoveDirectory,
        )
    } else if op_id == fs.rename_id {
        (
            bytes_at(2)?,
            fs.op_rename_id,
            WRITE_BYTES_REQUIRED_AUTHORITY,
            "fs_driver::rename",
            FsOpKind::RenameSource,
        )
    } else {
        return None;
    };

    let scope = match check_fs_capability(cap, op_kind, required, operation_name) {
        Ok(scope) => scope,
        Err(denial) => {
            handler.fs_denied(denial);
            return Some(Ok(fs_denied(operation, path, fs, ids, store)));
        }
    };
    let components = match fs_target_components(path) {
        Ok(components) => components,
        Err(denial) => {
            handler.fs_denied(denial);
            return Some(Ok(fs_denied(operation, path, fs, ids, store)));
        }
    };
    let resolved = match handler.fs_resolve(&scope.root, &components, op_kind, scope.symlink) {
        Ok(resolved) => resolved,
        Err(ResolveError::Denied(denial)) => {
            handler.fs_denied(denial);
            return Some(Ok(fs_denied(operation, path, fs, ids, store)));
        }
        Err(ResolveError::Io(error)) => {
            return Some(Ok(file_result(Err(error), operation, path, fs, ids, store)))
        }
    };
    handler.fs_after_resolve();

    let response = if op_id == fs.readfile_id {
        let Resolution::Existing(handle) = resolved else {
            return Some(Err(()));
        };
        file_result(
            handler.fs_read_at(&handle).map(EvalVal::Bytes),
            operation,
            path,
            fs,
            ids,
            store,
        )
    } else if op_id == fs.writefile_id {
        let policy = match args.get(3) {
            Some(EvalVal::Ctor { id, .. }) if *id == fs.create_new_id => {
                HostCreatePolicy::CreateNew
            }
            Some(EvalVal::Ctor { id, .. }) if *id == fs.create_or_truncate_id => {
                HostCreatePolicy::CreateOrTruncate
            }
            Some(EvalVal::Ctor { id, .. }) if *id == fs.create_or_keep_id => {
                HostCreatePolicy::CreateOrKeep
            }
            _ => return Some(Err(())),
        };
        let Some(contents) = bytes_at(4) else {
            return Some(Err(()));
        };
        let result = match resolved {
            Resolution::Existing(handle) => handler.fs_write_at(&handle, policy, contents),
            Resolution::Parent(parent, leaf) => {
                handler.fs_create_file_at(&parent, &leaf, policy, contents)
            }
        };
        file_result(
            result.map(|()| unit(store)),
            operation,
            path,
            fs,
            ids,
            store,
        )
    } else if op_id == fs.appendfile_id {
        let Some(contents) = bytes_at(3) else {
            return Some(Err(()));
        };
        let result = match resolved {
            Resolution::Existing(handle) => handler.fs_append_at(&handle, contents),
            Resolution::Parent(parent, leaf) => {
                handler.fs_create_append_at(&parent, &leaf, contents)
            }
        };
        file_result(
            result.map(|()| unit(store)),
            operation,
            path,
            fs,
            ids,
            store,
        )
    } else if op_id == fs.metadata_id {
        let Resolution::Existing(handle) = resolved else {
            return Some(Err(()));
        };
        let result = handler.fs_metadata_at(&handle).map(|metadata| {
            let kind = fs_kind_value(metadata.kind, fs, store);
            make_ctor(
                fs.mk_file_metadata_id,
                vec![EvalVal::BigInt(BigInt::from(metadata.size)), kind],
                store,
            )
        });
        file_result(result, operation, path, fs, ids, store)
    } else if op_id == fs.readdirectory_id {
        let Resolution::Existing(handle) = resolved else {
            return Some(Err(()));
        };
        let result = handler.fs_read_directory_at(&handle).map(|entries| {
            entries.into_iter().rev().fold(
                make_ctor(fs.nil_id, vec![EvalVal::Unknown], store),
                |tail, entry| {
                    let kind = fs_kind_value(entry.kind, fs, store);
                    let value = make_ctor(
                        fs.mk_dir_entry_id,
                        vec![EvalVal::Bytes(entry.name), kind],
                        store,
                    );
                    make_ctor(fs.cons_id, vec![EvalVal::Unknown, value, tail], store)
                },
            )
        });
        file_result(result, operation, path, fs, ids, store)
    } else if op_id == fs.createdirectory_id {
        let recursive = match args.get(2) {
            Some(EvalVal::Ctor { id, .. }) if *id == ids.true_id => true,
            Some(EvalVal::Ctor { id, .. }) if *id == ids.false_id => false,
            _ => return Some(Err(())),
        };
        let Resolution::Parent(parent, leaf) = resolved else {
            return Some(Err(()));
        };
        file_result(
            handler
                .fs_create_directory_at(&parent, &leaf, recursive)
                .map(|()| unit(store)),
            operation,
            path,
            fs,
            ids,
            store,
        )
    } else if op_id == fs.removefile_id {
        let Resolution::Parent(parent, leaf) = resolved else {
            return Some(Err(()));
        };
        file_result(
            handler
                .fs_remove_file_at(&parent, &leaf)
                .map(|()| unit(store)),
            operation,
            path,
            fs,
            ids,
            store,
        )
    } else if op_id == fs.removedirectory_id {
        let recursive = match args.get(2) {
            Some(EvalVal::Ctor { id, .. }) if *id == ids.true_id => true,
            Some(EvalVal::Ctor { id, .. }) if *id == ids.false_id => false,
            _ => return Some(Err(())),
        };
        let Resolution::Parent(parent, leaf) = resolved else {
            return Some(Err(()));
        };
        file_result(
            handler
                .fs_remove_directory_at(&parent, &leaf, recursive)
                .map(|()| unit(store)),
            operation,
            path,
            fs,
            ids,
            store,
        )
    } else {
        let Some(to) = bytes_at(3) else {
            return Some(Err(()));
        };
        let Resolution::Parent(from_parent, from_leaf) = resolved else {
            return Some(Err(()));
        };
        let to_components = match fs_target_components(to) {
            Ok(components) => components,
            Err(_) => return Some(Ok(fs_denied(operation, path, fs, ids, store))),
        };
        let to_resolution = match handler.fs_resolve(
            &scope.root,
            &to_components,
            FsOpKind::RenameDestination,
            scope.symlink,
        ) {
            Ok(resolution) => resolution,
            Err(ResolveError::Denied(_)) => {
                return Some(Ok(fs_denied(operation, path, fs, ids, store)))
            }
            Err(ResolveError::Io(error)) => {
                return Some(Ok(file_result(Err(error), operation, path, fs, ids, store)))
            }
        };
        let Resolution::Parent(to_parent, to_leaf) = to_resolution else {
            return Some(Err(()));
        };
        file_result(
            handler
                .fs_rename_at(&from_parent, &from_leaf, &to_parent, &to_leaf)
                .map(|()| unit(store)),
            operation,
            path,
            fs,
            ids,
            store,
        )
    };
    Some(Ok(response))
}

/// Host-effect driver (`42 §6.2`, `§6.3`): runs an `ITree` value to
/// completion, dispatching `Vis` nodes to Console, Clock, and, when `fs_ids`
/// is supplied, FS — one driver with exhaustive arms, not parallel dispatchers
/// (all share `ids`'s
/// `ret_id`/`vis_id`/`params_len`, the same `ITree`).
///
/// Dispatches exhaustively — no catch-all (`42 §6.5`): any op-tag that
/// matches none of the supplied Console, Clock, or FS algebras is
/// `Err(UnknownEffect)`, never silently skipped.
///
/// `ids.params_len` must equal the number of type-level params on `ITree`
/// (3 for the lifted `ITree (E:Type)(Resp:E->Type)(R:Type)`; 0 for the
/// simplified 0-param test ITree).
pub fn run_io<H: HostHandler>(
    mut tree: EvalVal,
    handler: &mut H,
    ids: &ConsoleIds,
    fs_ids: Option<&FSIds>,
    clock_ids: Option<&ClockIds>,
    coproduct_ids: Option<&CoproductIds>,
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
                    // Guard args access — a malformed Vis returns Err rather than panic.
                    let op = match args.get(m).cloned() {
                        Some(v) => v,
                        None => {
                            return Err(RunIoError::NotAnIOTree(EvalVal::Ctor {
                                id,
                                args,
                                slot: NULL_SLOT,
                            }))
                        }
                    };
                    let k = match args.get(m + 1).cloned() {
                        Some(v) => v,
                        None => {
                            return Err(RunIoError::NotAnIOTree(EvalVal::Ctor {
                                id,
                                args: Rc::new(vec![op]),
                                slot: NULL_SLOT,
                            }))
                        }
                    };
                    // D3 coproduct peel: strip InL/InR down to the innermost
                    // base tag BEFORE dispatch — effect-blind, a no-op when
                    // `coproduct_ids` is absent or the op carries no wrapper.
                    let op = peel_coproduct(op, coproduct_ids);
                    // Dispatch on every constructor in the sealed host floor.
                    // Unknown tags fail loudly below.
                    let resp = match &op {
                        EvalVal::Ctor {
                            id: op_id,
                            args: op_args,
                            ..
                        } if *op_id == ids.read_id => {
                            let Some(stream) = op_args.first().and_then(|v| decode_stream(v, ids))
                            else {
                                return Err(RunIoError::UnknownEffect(op));
                            };
                            let Some(limit) = op_args.get(1).and_then(read_limit) else {
                                return Err(RunIoError::UnknownEffect(op));
                            };
                            match handler.console_read(stream, limit) {
                                Ok(HostRead::Chunk(bytes)) => make_result(
                                    true,
                                    make_ctor(ids.chunk_id, vec![EvalVal::Bytes(bytes)], store),
                                    ids,
                                    store,
                                ),
                                Ok(HostRead::Eof) => make_result(
                                    true,
                                    make_ctor(ids.eof_id, vec![], store),
                                    ids,
                                    store,
                                ),
                                Err(error) => make_result(
                                    false,
                                    io_error_value(&error, ids, store),
                                    ids,
                                    store,
                                ),
                            }
                        }
                        EvalVal::Ctor {
                            id: op_id,
                            args: op_args,
                            ..
                        } if *op_id == ids.write_id => {
                            let Some(stream) = op_args.first().and_then(|v| decode_stream(v, ids))
                            else {
                                return Err(RunIoError::UnknownEffect(op));
                            };
                            let Some(EvalVal::Bytes(bytes)) = op_args.get(1) else {
                                return Err(RunIoError::UnknownEffect(op));
                            };
                            match handler.console_write(stream, bytes) {
                                Ok(()) => make_result(
                                    true,
                                    make_ctor(ids.unit_id, vec![], store),
                                    ids,
                                    store,
                                ),
                                Err(error) => make_result(
                                    false,
                                    io_error_value(&error, ids, store),
                                    ids,
                                    store,
                                ),
                            }
                        }
                        EvalVal::Ctor {
                            id: op_id,
                            args: op_args,
                            ..
                        } if *op_id == ids.flush_id => {
                            let Some(stream) = op_args.first().and_then(|v| decode_stream(v, ids))
                            else {
                                return Err(RunIoError::UnknownEffect(op));
                            };
                            match handler.console_flush(stream) {
                                Ok(()) => make_result(
                                    true,
                                    make_ctor(ids.unit_id, vec![], store),
                                    ids,
                                    store,
                                ),
                                Err(error) => make_result(
                                    false,
                                    io_error_value(&error, ids, store),
                                    ids,
                                    store,
                                ),
                            }
                        }
                        EvalVal::Ctor {
                            id: op_id,
                            args: op_args,
                            ..
                        } if *op_id == ids.is_terminal_id => {
                            let Some(stream) = op_args.first().and_then(|v| decode_stream(v, ids))
                            else {
                                return Err(RunIoError::UnknownEffect(op));
                            };
                            let bool_id = if handler.console_is_terminal(stream) {
                                ids.true_id
                            } else {
                                ids.false_id
                            };
                            make_ctor(bool_id, vec![], store)
                        }
                        EvalVal::Ctor {
                            id: op_id,
                            args: op_args,
                            ..
                        } => {
                            if let Some(clock) = clock_ids
                                .filter(|clock| *op_id == clock.wall_now_id && op_args.is_empty())
                            {
                                make_ctor(
                                    clock.mkinstant_id,
                                    vec![bigint_to_int_val(handler.clock_wall_now())],
                                    store,
                                )
                            } else {
                                match fs_ids.and_then(|fs| {
                                    fs_dispatch(*op_id, op_args, handler, fs, ids, store)
                                }) {
                                    Some(Ok(response)) => response,
                                    Some(Err(())) | None => {
                                        return Err(RunIoError::UnknownEffect(op));
                                    }
                                }
                            }
                        }
                        _ => return Err(RunIoError::UnknownEffect(op)),
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
        "not_bool" => 1,
        "and_bool" | "or_bool" => 2,
        "add_int" | "sub_int" | "mul_int" | "eq_int" | "leq_int" => 2,
        "add_float" | "sub_float" | "mul_float" | "div_float" | "eq_float" => 2,
        "add_float32" | "eq_float32" => 2,
        s if s.starts_with("add_int") || s.starts_with("sub_int") || s.starts_with("mul_int") => 2,
        s if s.starts_with("add_uint")
            || s.starts_with("sub_uint")
            || s.starts_with("mul_uint") =>
        {
            2
        }
        s if s.starts_with("wrapping_") => 2,
        // ── Bytes ops (`38 §1.2`, `38 §1.4`) ─────────────────────────────
        "bytes_length" | "bytes_encode" | "bytes_decode" => 1,
        "bytes_at" | "bytes_concat" => 2,
        "bytes_slice" => 3,
        // ── L3a String surface ops (`37 §2`) ──────────────────────────────
        "byte_length"
        | "char_length"
        | "string_to_list_char"
        | "list_char_to_string"
        | "bytes_to_list"
        | "list_to_bytes" => 1,
        _ => 1,
    }
}

// ── capacity conformance tests ────────────────────────────────────────────────

#[cfg(test)]
mod px0_target_classification_tests {
    use super::*;

    fn console_ids(unsupported_id: GlobalId) -> ConsoleIds {
        let unused = GlobalId(0);
        ConsoleIds {
            itree_id: unused,
            ret_id: unused,
            vis_id: unused,
            read_id: unused,
            write_id: unused,
            flush_id: unused,
            is_terminal_id: unused,
            stdin_id: unused,
            stdout_id: unused,
            stderr_id: unused,
            chunk_id: unused,
            eof_id: unused,
            true_id: unused,
            false_id: unused,
            ok_id: unused,
            err_id: unused,
            notfound_id: unused,
            permissiondenied_id: unused,
            capabilitydenied_id: unused,
            brokenpipe_id: unused,
            interrupted_id: unused,
            alreadyexists_id: unused,
            invalidinput_id: unused,
            isdirectory_id: unused,
            notdirectory_id: unused,
            notempty_id: unused,
            unsupported_id,
            other_id: GlobalId(1),
            unit_id: unused,
            params_len: 3,
        }
    }

    #[test]
    fn px1_named_unavailable_lane_maps_to_ken_unsupported() {
        let unsupported_id = GlobalId(91);
        let ids = console_ids(unsupported_id);
        let mut store = EvalStore::new();
        let value = io_error_value(&host_abi_unsupported(), &ids, &mut store);
        assert!(matches!(
            value,
            EvalVal::Ctor { id, args, .. } if id == unsupported_id && args.is_empty()
        ));
    }

    #[cfg(not(target_os = "linux"))]
    #[test]
    fn px1_non_linux_fs_driver_fails_before_host_io() {
        fn assert_unsupported<T: std::fmt::Debug>(result: io::Result<T>) {
            assert_eq!(result.unwrap_err().kind(), io::ErrorKind::Unsupported);
        }

        let mut host = PosixHost::new();
        let handle = 0;
        assert_unsupported(host.mint_scoped_fs_cap(
            capabilities::AUTH_FULL,
            b".",
            capabilities::RightSet::ALL,
            capabilities::SymlinkPolicy::NoFollow,
        ));
        match host.fs_resolve(
            &capabilities::FsHandle::Virtual(0),
            &[],
            FsOpKind::Read,
            capabilities::SymlinkPolicy::NoFollow,
        ) {
            Err(ResolveError::Io(error)) => {
                assert_eq!(error.kind(), io::ErrorKind::Unsupported)
            }
            other => panic!("expected named non-Linux unavailable lane, got {other:?}"),
        }
        assert_unsupported(host.fs_read_at(&handle));
        assert_unsupported(host.fs_write_at(&handle, HostCreatePolicy::CreateOrTruncate, b"bytes"));
        assert_unsupported(host.fs_create_file_at(
            &handle,
            b"file",
            HostCreatePolicy::CreateNew,
            b"bytes",
        ));
        assert_unsupported(host.fs_append_at(&handle, b"bytes"));
        assert_unsupported(host.fs_create_append_at(&handle, b"file", b"bytes"));
        assert_unsupported(host.fs_metadata_at(&handle));
        assert_unsupported(host.fs_read_directory_at(&handle));
        assert_unsupported(host.fs_create_directory_at(&handle, b"dir", false));
        assert_unsupported(host.fs_remove_file_at(&handle, b"file"));
        assert_unsupported(host.fs_remove_directory_at(&handle, b"dir", false));
        assert_unsupported(host.fs_rename_at(&handle, b"from", &handle, b"to"));
    }
}

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

// ── F1 bignum conformance tests (`conformance/surface/numbers/seed-f1-bignum-int.md`) ──
//
// AC3 (store round-trip) needs the private `to_rt`/`intern` producer, so these
// live here rather than in an external `tests/` file (which can only see the
// `pub` surface). AC1/AC2 (no-wrap totality / independent oracle) are covered
// externally in `tests/f1_bignum_acceptance.rs` against `prim_reduce`.
#[cfg(test)]
mod f1_bignum_tests {
    use super::*;
    use ken_runtime::Canonical;

    /// Construct `2^n` as an `EvalVal` via `Shl` — a test-input constructor,
    /// never the `add_int`/`sub_int`/`mul_int` reduction under audit.
    fn pow2(n: u32) -> EvalVal {
        bigint_to_int_val(BigInt::from(1u8) << n)
    }

    fn as_bigint_rt(rt: &RtValue) -> (RtSign, Vec<u64>) {
        match rt {
            RtValue::BigInt { sign, limbs } => (*sign, limbs.clone()),
            other => panic!("expected Value::BigInt, got {:?}", other),
        }
    }

    // surface/numbers/f1-store-roundtrip-above-i128-byte-identical (soundness)
    #[test]
    fn f1_store_roundtrip_above_i128_byte_identical() {
        // given: mul_int(2^127, 4) = 2^129, produced by the real arithmetic
        // path (not a hand-fed Value::BigInt).
        let result = prim_reduce("mul_int", &[pow2(127), EvalVal::Int(4)]);
        let rt = to_rt(&result).expect("F1 establishes the BigInt to_rt arm");
        let (sign, limbs) = as_bigint_rt(&rt);
        assert!(limbs.len() >= 3, "2^129 requires >= 3 u64 limbs");

        // "and back": reconstruct the evaluator value from the stored
        // representation, then re-derive its store image.
        let reconstructed = bigint_from_rt(sign, &limbs);
        let rt_again = to_rt(&reconstructed).expect("reconstructed value must also intern");

        let mut bytes1 = Vec::new();
        let mut bytes2 = Vec::new();
        rt.encode_canonical(&mut bytes1);
        rt_again.encode_canonical(&mut bytes2);
        assert_eq!(
            bytes1, bytes2,
            "round-trip must be byte-identical (18a §5.2.1(3))"
        );

        let mut store = EvalStore::new();
        let (InternResult::New(slot1) | InternResult::Hit(slot1)) = store.k3.intern(&rt) else {
            panic!("expected successful intern, not capacity exhaustion");
        };
        let (InternResult::New(slot2) | InternResult::Hit(slot2)) = store.k3.intern(&rt_again)
        else {
            panic!("expected successful intern, not capacity exhaustion");
        };
        assert_eq!(
            slot1, slot2,
            "round-tripped value must content-address identically"
        );

        // "reduces identically": the reconstructed value behaves like the
        // original under further reduction.
        assert!(eval_vals_eq(&result, &reconstructed));
        assert_eq!(
            prim_reduce("eq_int", &[result, reconstructed]),
            EvalVal::Bool(true)
        );
    }

    // surface/numbers/f1-dedup-content-address-stable-across-paths (soundness)
    #[test]
    fn f1_dedup_content_address_stable_across_paths() {
        // given: the same 2^128 reached by two distinct arithmetic paths.
        let path1 = prim_reduce("mul_int", &[pow2(64), pow2(64)]);
        let path2 = prim_reduce("mul_int", &[pow2(127), EvalVal::Int(2)]);
        let rt1 = to_rt(&path1).expect("path1 must intern");
        let rt2 = to_rt(&path2).expect("path2 must intern");

        let mut store = EvalStore::new();
        let (InternResult::New(slot1) | InternResult::Hit(slot1)) = store.k3.intern(&rt1) else {
            panic!("expected successful intern, not capacity exhaustion");
        };
        let (InternResult::New(slot2) | InternResult::Hit(slot2)) = store.k3.intern(&rt2) else {
            panic!("expected successful intern, not capacity exhaustion");
        };
        assert_eq!(
            slot1, slot2,
            "two eval paths to one integer must dedup to one store slot (44)"
        );
    }

    // surface/numbers/f1-zero-and-sign-canonical (soundness)
    #[test]
    fn f1_zero_and_sign_canonical() {
        let n = pow2(128);

        // given: 0 via sub_int n n. This always narrows to the `Int` fast
        // path (`bigint_to_int_val`, `18a §5.2.1(1)`) — a `BigInt`-tagged
        // zero never reaches `to_rt` through real arithmetic. The canonical
        // zero-limb rule is therefore pinned directly against `to_rt`'s
        // `BigInt` arm (the real producer, just not gated behind narrowing),
        // and the arithmetic narrowing itself is asserted as a precondition.
        let zero = prim_reduce("sub_int", &[n.clone(), n.clone()]);
        assert_eq!(
            zero,
            EvalVal::Int(0),
            "zero must narrow to the Int fast path, never a BigInt tag"
        );
        let rt_zero = to_rt(&EvalVal::BigInt(BigInt::from(0))).expect("zero must intern");
        let (zero_sign, zero_limbs) = as_bigint_rt(&rt_zero);
        assert_eq!(
            zero_limbs,
            vec![0u64],
            "zero must canonicalize to exactly one zero limb"
        );
        assert_eq!(
            zero_sign,
            RtSign::NonNegative,
            "zero must have canonical sign"
        );

        // given: -(2^128) via sub_int 0 (2^128).
        let neg = prim_reduce("sub_int", &[EvalVal::Int(0), n.clone()]);
        let rt_neg = to_rt(&neg).expect("negative must intern");
        let rt_pos = to_rt(&n).expect("positive must intern");
        let (neg_sign, _) = as_bigint_rt(&rt_neg);
        assert_eq!(neg_sign, RtSign::Negative, "sign must be preserved");

        let mut store = EvalStore::new();
        let (InternResult::New(slot_neg) | InternResult::Hit(slot_neg)) = store.k3.intern(&rt_neg)
        else {
            panic!("expected successful intern, not capacity exhaustion");
        };
        let (InternResult::New(slot_pos) | InternResult::Hit(slot_pos)) = store.k3.intern(&rt_pos)
        else {
            panic!("expected successful intern, not capacity exhaustion");
        };
        assert_ne!(
            slot_neg, slot_pos,
            "+n and -n must have distinct content-addresses"
        );
    }
}
