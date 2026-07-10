//! `foreign` FFI bindings + trust-boundary discipline (`38 §2–§4`, L7).
//!
//! A `foreign f : T = "symbol" "library" [pure] [FS]` declaration elaborates
//! to a `declare_postulate` (opaque constant, `11 §4`) whose type is assumed
//! and whose id appears in `trusted_base()` (`18 §5`). This is exactly the
//! foreign-as-listed-postulate discipline (`38 §3.1`): **the kernel cannot
//! check C, so the type is assumed and the assumption is visible.**
//!
//! The `pure` annotation is recorded as a *claim*, never a kernel check —
//! it projects to *trusted* (`P`), never `Q` (AC3, `38 §3.2`).
//!
//! Effect rows for effect-tracked foreigners are stored in `ForeignEnv`
//! alongside the postulate id, so AC5 conformance tests derive the seed
//! from the actual L7 binding (not a hand-fed literal).
//!
//! Marshalling: Bytes arguments → `(ptr, len)` pair; scalars → machine types
//! (`41 §1`). The `MarshalSig` is computed structurally from the Ken type.
//!
//! Runtime checks (AC4): a foreign `ensures` clause that is statically
//! unprovable emits a `FfiRuntimeCheck` obligation — a `tested` entry in
//! the assumption boundary (`21 §5.2`). The clause is lowered to a
//! fail-fast assertion at the call boundary.

use std::collections::{HashMap, HashSet};

use ken_kernel::{declare_postulate, GlobalEnv, GlobalId};

use crate::effects::row::EffectRow;
use crate::error::ElabError;

// ─── Marshalling record (AC1) ─────────────────────────────────────────────────

/// How one argument (or the return) of a `foreign` function is marshalled
/// (`38 §2.2`, `41 §1`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarshalKind {
    /// A scalar type (Int, Float, …) → its machine equivalent.
    Scalar(String),
    /// `Bytes` → `(ptr: *const u8, len: usize)` pair (`41 §1`, `38 §1.1`).
    BytesPtr,
}

/// Structural marshalling signature for a `foreign` declaration.
///
/// Derived from the Ken type of the foreign — never from the C source.
/// Asserted in AC1 conformance cases.
#[derive(Debug, Clone)]
pub struct MarshalSig {
    pub params: Vec<MarshalKind>,
    pub result: MarshalKind,
}

/// Classify one type position (a Const id) into a `MarshalKind`.
fn classify_marshal(id: GlobalId, bytes_id: GlobalId) -> MarshalKind {
    if id == bytes_id {
        MarshalKind::BytesPtr
    } else {
        MarshalKind::Scalar(format!("{:?}", id))
    }
}

// ─── Runtime check record (AC4) ──────────────────────────────────────────────

/// A boundary contract (`requires`/`ensures`) on a `foreign` that is
/// statically unprovable and is lowered to a runtime-checked assertion
/// (`21 §5.2`, `38 §3.3`).
///
/// Emitted by `elaborate_foreign_env` for each `ensures` clause.
/// Status = `tested` in the assumption boundary.
#[derive(Debug, Clone)]
pub struct FfiRuntimeCheck {
    /// The postulate id (in `trusted_base()`) for this runtime-check slot.
    pub hole_id: GlobalId,
    /// The clause kind: "requires" or "ensures".
    pub clause_kind: &'static str,
    /// String representation of the clause proposition (for display).
    pub clause_str: String,
}

// ─── Foreign binding ─────────────────────────────────────────────────────────

/// The elaborated result of a `foreign` declaration (`38 §2`).
///
/// The `postulate_id` is the kernel's opaque constant for this binding;
/// it is always in `trusted_base()` (the foreign type is assumed).
/// The `pure` flag is a **claim**, not a kernel check — it does NOT cause
/// the binding to leave `trusted_base()` (`38 §3.2`).
#[derive(Debug, Clone)]
pub struct ForeignBinding {
    /// Kernel id — always an opaque postulate in `trusted_base()`.
    pub postulate_id: GlobalId,
    /// C symbol name (e.g. `"write"`).
    pub symbol: String,
    /// Library name (e.g. `"c"`).
    pub library: String,
    /// `pure` claim — asserted, not kernel-checked (`38 §3.2`).
    pub is_pure: bool,
    /// Declared effect row (`36 §1`). Empty iff `is_pure`.
    pub effect_row: EffectRow,
    /// Structural marshalling record (`41 §1`).
    pub marshal_sig: Option<MarshalSig>,
    /// Emitted runtime checks for statically-unprovable boundary contracts
    /// (`21 §5.2`, `38 §3.3`).
    pub runtime_checks: Vec<FfiRuntimeCheck>,
}

// ─── Foreign environment ──────────────────────────────────────────────────────

/// Registry of all elaborated `foreign` bindings (`38 §2`).
pub struct ForeignEnv {
    /// Bindings keyed by the Ken name of the foreign.
    pub bindings: HashMap<String, ForeignBinding>,
    /// Effect rows keyed by op name — the seed for AC5 conformance tests.
    ///
    /// Removing a binding from here empties the seed for that op → AC5 E1
    /// test structurally fails (green-vs-green is impossible).
    pub io_effect_rows: HashMap<String, EffectRow>,
}

impl ForeignEnv {
    pub fn empty() -> Self {
        Self {
            bindings: HashMap::new(),
            io_effect_rows: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: String, binding: ForeignBinding) {
        if !binding.effect_row.is_empty() {
            self.io_effect_rows
                .insert(name.clone(), binding.effect_row.clone());
        }
        self.bindings.insert(name, binding);
    }
}

// ─── elaborate_foreign ───────────────────────────────────────────────────────

/// Elaborate a resolved `foreign` declaration into a kernel postulate.
///
/// Returns a `ForeignBinding` with the postulate id, marshalling record,
/// and runtime checks. Registers the binding name in `globals`.
///
/// **Trust accounting:** `declare_postulate` makes the binding opaque and
/// records it in `trusted_base()`. The `pure` flag is stored as a claim on
/// the binding — it does NOT discharge the postulate.
pub fn elaborate_foreign(
    env: &mut GlobalEnv,
    globals: &mut HashMap<String, GlobalId>,
    bytes_id: GlobalId,
    name: &str,
    ty_core: ken_kernel::Term,
    symbol: &str,
    library: &str,
    is_pure: bool,
    visits: &[String],
    ensures_clauses: &[String],
    span: &crate::error::Span,
) -> Result<ForeignBinding, ElabError> {
    // Compute the effect row from the visits list.
    let effect_row = if visits.is_empty() {
        EffectRow::empty()
    } else {
        EffectRow::from_effects(visits.iter().cloned())
    };

    // Compute the marshalling signature by walking the Ken type Pi-chain.
    let marshal_sig = compute_marshal_sig(&ty_core, bytes_id);

    // Declare the postulate — the foreign type is ASSUMED, not verified.
    let postulate_id = declare_postulate(env, vec![], ty_core)
        .map_err(|e| ElabError::KernelRejected {
            error: e,
            span: span.clone(),
        })?;
    globals.insert(name.to_string(), postulate_id);

    // Emit runtime checks for statically-unprovable boundary contracts (AC4).
    // Foreign `ensures` clauses are always unprovable (no C body for the
    // kernel to reason about) → they lower to `tested` runtime assertions.
    let mut runtime_checks = Vec::new();
    for clause in ensures_clauses {
        // Each runtime check gets its own postulate slot in trusted_base.
        let rc_ty = ken_kernel::Term::omega(ken_kernel::Level::Zero);
        let hole_id = declare_postulate(env, vec![], rc_ty).map_err(|e| {
            ElabError::KernelRejected { error: e, span: span.clone() }
        })?;
        runtime_checks.push(FfiRuntimeCheck {
            hole_id,
            clause_kind: "ensures",
            clause_str: clause.clone(),
        });
    }

    Ok(ForeignBinding {
        postulate_id,
        symbol: symbol.to_string(),
        library: library.to_string(),
        is_pure,
        effect_row,
        marshal_sig,
        runtime_checks,
    })
}

/// Walk the Ken type Pi-chain and produce a `MarshalSig`.
///
/// Recognises `Bytes` (maps to `BytesPtr`) and anything else (scalar).
/// Only `Const` nodes are classified — complex types default to scalar.
fn compute_marshal_sig(ty: &ken_kernel::Term, bytes_id: GlobalId) -> Option<MarshalSig> {
    let mut params = Vec::new();
    let mut cur = ty;
    loop {
        match cur {
            ken_kernel::Term::Pi(dom, cod) => {
                params.push(classify_const(dom, bytes_id));
                cur = cod;
            }
            ken_kernel::Term::Const { id, .. } => {
                return Some(MarshalSig {
                    params,
                    result: classify_marshal(*id, bytes_id),
                });
            }
            _ => return None,
        }
    }
}

fn classify_const(term: &ken_kernel::Term, bytes_id: GlobalId) -> MarshalKind {
    match term {
        ken_kernel::Term::Const { id, .. } => classify_marshal(*id, bytes_id),
        _ => MarshalKind::Scalar("?".to_string()),
    }
}

// ─── trusted_base_delta ───────────────────────────────────────────────────────

/// Compute the `trusted_base_delta` of a definition: the subset of
/// `trusted_base()` that is reachable from the definition's transparent body
/// (`25 §3`).
///
/// Walks the body term (via `env.transparent_body(def_id)`) collecting all
/// `Const` ids that appear in `trusted_base()`. This is the dependency cone
/// — what the artifact *relies on* (by call/use), not what is merely in scope.
///
/// Used in AC2 conformance: B1 asserts the foreign postulate IS in the cone
/// of a caller; B2 asserts it is ABSENT from a non-caller.
pub fn trusted_base_delta(env: &GlobalEnv, def_id: GlobalId) -> HashSet<GlobalId> {
    let tb: HashSet<GlobalId> = env.trusted_base().into_iter().collect();
    let mut result = HashSet::new();
    if let Some((_, body)) = env.transparent_body(def_id) {
        collect_consts_in_tb(&body, &tb, &mut result);
    }
    result
}

fn collect_consts_in_tb(
    term: &ken_kernel::Term,
    tb: &HashSet<GlobalId>,
    out: &mut HashSet<GlobalId>,
) {
    use ken_kernel::Term;
    match term {
        Term::Const { id, .. } => {
            if tb.contains(id) {
                out.insert(*id);
            }
        }
        Term::Var(_) | Term::Type(_) | Term::Omega(_) | Term::IntLit(_) => {}
        Term::Pi(a, b) | Term::Lam(a, b) | Term::Sigma(a, b) | Term::Pair(a, b)
        | Term::App(a, b) => {
            collect_consts_in_tb(a, tb, out);
            collect_consts_in_tb(b, tb, out);
        }
        Term::Proj1(t) | Term::Proj2(t) | Term::Trunc(t) | Term::TruncProj(t)
        | Term::Refl(t) => {
            collect_consts_in_tb(t, tb, out);
        }
        Term::Ascript(t, ty) => {
            collect_consts_in_tb(t, tb, out);
            collect_consts_in_tb(ty, tb, out);
        }
        Term::Eq(a, b, c) => {
            collect_consts_in_tb(a, tb, out);
            collect_consts_in_tb(b, tb, out);
            collect_consts_in_tb(c, tb, out);
        }
        Term::Cast(a, b, c, d) => {
            collect_consts_in_tb(a, tb, out);
            collect_consts_in_tb(b, tb, out);
            collect_consts_in_tb(c, tb, out);
            collect_consts_in_tb(d, tb, out);
        }
        Term::J(a, b, c) => {
            collect_consts_in_tb(a, tb, out);
            collect_consts_in_tb(b, tb, out);
            collect_consts_in_tb(c, tb, out);
        }
        Term::Quot(a, b) => {
            collect_consts_in_tb(a, tb, out);
            collect_consts_in_tb(b, tb, out);
        }
        Term::QuotClass(a) => {
            collect_consts_in_tb(a, tb, out);
        }
        Term::Let { ty, val, body } => {
            collect_consts_in_tb(ty, tb, out);
            collect_consts_in_tb(val, tb, out);
            collect_consts_in_tb(body, tb, out);
        }
        Term::Elim { params, motive, methods, indices, scrut, .. } => {
            for p in params { collect_consts_in_tb(p, tb, out); }
            collect_consts_in_tb(motive, tb, out);
            for m in methods { collect_consts_in_tb(m, tb, out); }
            for i in indices { collect_consts_in_tb(i, tb, out); }
            collect_consts_in_tb(scrut, tb, out);
        }
        Term::IndFormer { .. } | Term::Constructor { .. } => {}
        Term::QuotElim { motive, method, respect, scrut } => {
            collect_consts_in_tb(motive, tb, out);
            collect_consts_in_tb(method, tb, out);
            collect_consts_in_tb(respect, tb, out);
            collect_consts_in_tb(scrut, tb, out);
        }
        Term::Absurd(motive, proof) => {
            collect_consts_in_tb(motive, tb, out);
            collect_consts_in_tb(proof, tb, out);
        }
    }
}
