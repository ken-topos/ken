//! V3 automated prover: obligation `Γ ⊢ φ` → verdict trichotomy
//! (`proved` | `disproved` | `unknown`) (`23 §1.2`, `21 §5.1`).
//!
//! **Soundness = kernel re-checks every certificate** (`23 §1.5`): `proved` is
//! returned **only** when `check(env, Γ, cert, φ)` accepts. A prover bug can
//! cause `unknown` (honest failure) but **never a false `proved`** — the kernel
//! is the sole authority (the de Bruijn criterion, `18 §4.5`).
//!
//! **Exhaustive by construction** (`23 §2.1`, `§7.4`): the classifier `match`
//! has **no `_ ⇒ skip`** arm — every obligation shape reaches an outcome.
//! HO is the always-applicable default: an unrecognized/future `φ` lands in
//! HO and is attempted (tactics, or honest typed hole), never silently dropped.
//! A never-routed obligation leaves no cert **and** no hole, so it escapes
//! `trusted_base()` and reads discharged though never attempted — the V2
//! omission hazard, one tier up (`22 §2.5`). Structural routing is the sole
//! backstop.
//!
//! **Backends at V3 (23 §6 / §9):**
//! - **IPC** (propositional skeleton, `23 §5`): Pi-intro, Sigma-intro, assumption
//!   lookup — decides intuitionistic propositional goals built from `Π`/`Σ`.
//! - **D / FO** (decidable / Kripke-embedding, `23 §3`/§4): structural scaffold
//!   in place; Z3 decision + Kripke embedding + adequacy proof are
//!   `[placeholder — reifies in V4]` pending backend infrastructure.
//! - **HO** (induction / tactics, `23 §5`): IPC tactic + honest `unknown` hole
//!   for goals outside the propositional fragment.
//!
//! `[placeholder — reifies in V4]` marks deferred decisions/backends.

use ken_kernel::{
    check, declare_postulate, subst::subst0, Context, GlobalEnv, GlobalId, Term,
};

use crate::extract::{ObligationId, ObligationTriple};

// ─── Route ──────────────────────────────────────────────────────────────────

/// Fragment route from the classifier (`23 §2`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Route {
    /// Decidable atoms — `φ ∨ ¬φ` holds; direct/reflective decision (`23 §3`).
    D,
    /// First-order intuitionistic — Kripke embedding → solver (`23 §4`).
    FO,
    /// Higher-order / inductive — IPC tactic + typed hole as default (`23 §5`).
    HO,
}

// ─── Verdict ────────────────────────────────────────────────────────────────

/// A Kripke countermodel witnessing that `φ` is forced nowhere.
///
/// Diagnostic shape is `(oracle)` — `24`'s structured schema has not landed;
/// the field carries a human-readable description pending `24`.
#[derive(Debug, Clone)]
pub struct Countermodel {
    pub description: String,
}

/// The verdict trichotomy (`23 §1.2`, `21 §5.1`).
///
/// **No fourth verdict, no `failure` catch-all** (`23 §1.2`): a search that
/// neither closes nor refutes `φ` is `Unknown`-with-hole (honest), never a
/// silent drop.
#[derive(Debug, Clone)]
pub enum Verdict {
    /// Certificate `cert : φ` accepted by `check(env, Γ, cert, φ)` (`18 §4.5`).
    Proved { cert: Term },
    /// `φ` is refutable; a Kripke model forces `¬φ` at some world (`24 §1`).
    /// Where backend yields `q : ¬φ`, it is also `check`-ed (`23 §1.2`).
    Disproved { countermodel: Countermodel },
    /// Neither proved nor refuted — a typed hole `?id : φ` admitted as
    /// `declare_postulate(φ)` and enumerated by `trusted_base()` (`23 §1.3`).
    Unknown { hole_id: GlobalId },
}

/// Prover output keyed by obligation `id` for `21 §5.3` status projection.
#[derive(Debug)]
pub struct ProverResult {
    /// Stable obligation id from V2 (`22 §1`).
    pub obligation_id: ObligationId,
    /// The verdict for this obligation.
    pub verdict: Verdict,
}

// ─── Classifier ─────────────────────────────────────────────────────────────

/// Classify one obligation by syntactic shape of `φ` (`23 §2` / §2.1).
///
/// **Exhaustive — NO `_ ⇒ skip`** (`23 §2.1`): HO is the always-applicable
/// default arm. An unrecognized or future formula shape routes to HO, is
/// attempted (IPC, or left an honest typed hole), and is **never** silently
/// dropped. This makes completeness-of-routing a compile-time structural
/// property, not a runtime check — the V3 analog of V2's `exhaustive-by-
/// construction` (`22 §2.5`).
pub fn classify(phi: &Term) -> Route {
    if is_ground_decidable(phi) {
        // Closed ground atoms — φ ∨ ¬φ holds (23 §3).
        Route::D
    } else if is_first_order_intuit(phi) {
        // First-order connective structure over decidable atoms (23 §4).
        Route::FO
    } else {
        // HO: the default. Everything outside D/FO lands here. (23 §5)
        // NO `_ ⇒ skip` — "HO with hole" is always a legal outcome.
        Route::HO
    }
}

/// A ground-decidable atom: no free variables, built from constants only.
/// Conservative: only closed constant applications claimed as D.
fn is_ground_decidable(phi: &Term) -> bool {
    !has_free_vars(phi, 0) && is_const_atom(phi)
}

/// True if `φ` has a first-order connective structure: Pi/Sigma/App over
/// bound variables and Omega-typed predicates, with no type quantification
/// or inductive eliminators.
fn is_first_order_intuit(phi: &Term) -> bool {
    match phi {
        Term::Pi(a, b) | Term::Sigma(a, b) => {
            is_first_order_intuit(a) && is_first_order_intuit(b)
        }
        Term::App(f, a) => is_first_order_intuit(f) && is_first_order_intuit(a),
        Term::Omega(_) => true,
        Term::Var(_) => true,
        Term::Const { .. } => true,
        _ => false, // Lam, Pair, Proj, Cast, Eq, Trunc, etc. → HO
    }
}

/// A closed constant atom: constant applied to closed arguments (no free vars).
fn is_const_atom(phi: &Term) -> bool {
    match phi {
        Term::Const { .. } => true,
        Term::App(f, _) => is_const_atom(f),
        _ => false,
    }
}

/// True if `t` contains any free `Var` with index ≥ `depth`.
fn has_free_vars(t: &Term, depth: usize) -> bool {
    match t {
        Term::Var(i) => *i >= depth,
        Term::Pi(a, b) | Term::Lam(a, b) | Term::Sigma(a, b) | Term::Pair(a, b) => {
            has_free_vars(a, depth) || has_free_vars(b, depth + 1)
        }
        Term::App(f, a) => has_free_vars(f, depth) || has_free_vars(a, depth),
        Term::Proj1(t) | Term::Proj2(t) => has_free_vars(t, depth),
        Term::Eq(ty, a, b) => {
            has_free_vars(ty, depth) || has_free_vars(a, depth) || has_free_vars(b, depth)
        }
        _ => false,
    }
}

// ─── Main entry point ────────────────────────────────────────────────────────

/// Attempt one obligation; emit the verdict trichotomy (`23 §1.2`).
///
/// **Cardinal rule** (`23 §1.5`): `proved` is only returned when
/// `check(env, Γ, cert, φ)` accepts. The prover cannot forge `proved`.
///
/// Route selection is **exhaustive**: every obligation is attempted (§2.1).
pub fn attempt_obligation(env: &mut GlobalEnv, triple: &ObligationTriple) -> ProverResult {
    let route = classify(&triple.phi);
    let ctx = context_from_triple(triple);
    let verdict = match route {
        Route::D => attempt_d(env, &ctx, &triple.phi, &triple.goal_closed),
        Route::FO => attempt_fo(env, &ctx, &triple.phi, &triple.goal_closed),
        // HO: the default — every unrecognized shape also lands here.
        // NO `_ ⇒ skip`: this arm is always present and always attempts.
        Route::HO => attempt_ho(env, &ctx, &triple.phi, &triple.goal_closed),
    };
    ProverResult { obligation_id: triple.id.clone(), verdict }
}

/// Attempt a candidate certificate against the kernel.
///
/// **The `check_cert` path** (`23 §4` route (a), distinct from the kernel API
/// `check`): the kernel re-check `check(env, [], cert, phi_closed)` is the
/// SOLE reason `proved` is believed. This function is the soundness bridge —
/// nothing else here can break soundness if this call holds.
pub fn attempt_with_cert(env: &mut GlobalEnv, phi_closed: &Term, cert: Term) -> Verdict {
    match check(env, &Context::new(), &cert, phi_closed) {
        Ok(()) => Verdict::Proved { cert },
        Err(_) => emit_unknown_hole(env, phi_closed),
    }
}

// ─── Fragment D ─────────────────────────────────────────────────────────────

/// Fragment D: reflective decision for ground decidable atoms (`23 §3`).
///
/// For closed goals with a kernel-verified decision procedure `dec`, the kernel
/// evaluates `dec a` to `inl proof` or `inr refutation` by canonicity
/// (`16 §9`). The full decision procedure + Z3-backed search for open goals
/// (`23 §3.2`) is `[placeholder — reifies in V4]` pending backend infra.
/// Conservative fallback: honest `unknown` for atoms we cannot reduce.
fn attempt_d(
    env: &mut GlobalEnv,
    _ctx: &Context,
    _phi: &Term,
    phi_closed: &Term,
) -> Verdict {
    // [placeholder — reifies in V4]: kernel whnf + decision procedure (23 §3.1)
    // + Z3-backed arithmetic search + Decidable constructor extraction (23 §3.2).
    // Conservative: honest unknown until backend is in place.
    emit_unknown_hole(env, phi_closed)
}

// ─── Fragment FO ─────────────────────────────────────────────────────────────

/// Fragment FO: Kripke embedding → classical solver → reflective cert (`23 §4`).
///
/// The full Kripke embedding (`φ ↦ φ#`, `World` sort, adequacy lemma
/// `classically_valid(φ#) → φ`, `check_cert` soundness) is
/// `[placeholder — reifies in V4]` pending backend infrastructure.
/// Falls back to the IPC propositional skeleton for the connective structure.
fn attempt_fo(
    env: &mut GlobalEnv,
    ctx: &Context,
    phi: &Term,
    phi_closed: &Term,
) -> Verdict {
    // The FO propositional structure can be handled by the IPC tactic for the
    // connective skeleton. The Kripke embedding for quantified FO goals is
    // [placeholder — reifies in V4].
    attempt_ipc(env, ctx, phi, phi_closed)
}

// ─── Fragment HO ─────────────────────────────────────────────────────────────

/// Fragment HO: IPC reflective tactic + honest typed hole fallback (`23 §5`).
///
/// Handles: Pi-intro (⇒/∀ intro → λ-abstract), Sigma-intro (∧ intro → pair),
/// context assumption lookup (hyp), Sigma-elim (∧ elim → Proj1/Proj2).
/// Induction tactics and full higher-order proving are
/// `[placeholder — reifies in V4]` (`23 §5`).
fn attempt_ho(
    env: &mut GlobalEnv,
    ctx: &Context,
    phi: &Term,
    phi_closed: &Term,
) -> Verdict {
    attempt_ipc(env, ctx, phi, phi_closed)
}

// ─── IPC proof search ────────────────────────────────────────────────────────

/// Intuitionistic propositional calculator: build a proof certificate from the
/// connective structure of `φ`.
///
/// The returned cert is **always kernel-checked** before `proved` is declared
/// — the cardinal rule (`23 §1.5`).
fn attempt_ipc(
    env: &mut GlobalEnv,
    ctx: &Context,
    phi: &Term,
    phi_closed: &Term,
) -> Verdict {
    match ipc_search(ctx, phi, 0) {
        Some(cert) => {
            // Cardinal rule: check the cert before claiming proved (23 §1.5).
            match check(env, &Context::new(), &cert, phi_closed) {
                Ok(()) => Verdict::Proved { cert },
                Err(_) => emit_unknown_hole(env, phi_closed),
            }
        }
        None => emit_unknown_hole(env, phi_closed),
    }
}

/// Recursive IPC proof search in open context `ctx`.
///
/// `depth` = number of Pi-intros performed (caps the recursion to avoid
/// divergence on cyclic/self-referential goals, limit = 32).
///
/// Returns a candidate cert in the open context; caller MUST kernel-check.
/// Returns `None` when the goal is outside the propositional fragment.
fn ipc_search(ctx: &Context, phi: &Term, depth: usize) -> Option<Term> {
    if depth > 32 {
        return None;
    }
    match phi {
        // Pi-intro: ⊢ Pi(A, B) via λx:A. proof(B) — `23 §5` ⇒/∀ intro.
        Term::Pi(a, b) => {
            let mut ext_ctx = ctx.clone();
            ext_ctx.push(*a.clone());
            let body = ipc_search(&ext_ctx, b, depth + 1)?;
            Some(Term::lam(*a.clone(), body))
        }

        // Sigma-intro: ⊢ Sigma(A, B) via pair(proof(A), proof(B[pA/x])).
        // Non-dependent approximation: substitutes the first proof into B.
        Term::Sigma(a, b) => {
            let p_a = ipc_search(ctx, a, depth)?;
            let b_sub = subst0(b, &p_a);
            let p_b = ipc_search(ctx, &b_sub, depth)?;
            Some(Term::pair(p_a, p_b))
        }

        // Context lookup + simple Sigma elimination.
        phi_goal => {
            for i in 0..ctx.len() {
                let hyp_ty = ctx.lookup(i).expect("valid index");

                // Direct assumption: hyp has exactly type phi_goal.
                if hyp_ty == phi_goal {
                    return Some(Term::var(i));
                }

                // Sigma-elim: hyp = Sigma(A, B); goal matches A → Proj1(hyp).
                if let Term::Sigma(a, _b) = hyp_ty {
                    if a.as_ref() == phi_goal {
                        return Some(Term::Proj1(Box::new(Term::var(i))));
                    }
                }

                // Sigma-elim: hyp = Sigma(A, B); goal matches B (non-dep) → Proj2(hyp).
                if let Term::Sigma(_a, b) = hyp_ty {
                    if b.as_ref() == phi_goal {
                        return Some(Term::Proj2(Box::new(Term::var(i))));
                    }
                }
            }
            None
        }
    }
}

// ─── Unknown hole ────────────────────────────────────────────────────────────

/// Emit an `Unknown` verdict — register a typed hole in `trusted_base()`.
///
/// Per `23 §1.3` / `18 §5`: `declare_postulate(phi_closed)` registers the goal
/// as an assumption, so its id appears in `trusted_base()`. This is what makes
/// `unknown` **kernel-structural** and **`trusted_base()`-distinct from `proved`**
/// (which retires the postulate on discharge).
fn emit_unknown_hole(env: &mut GlobalEnv, phi_closed: &Term) -> Verdict {
    let hole_id = declare_postulate(env, vec![], phi_closed.clone())
        .expect("declare_postulate for unknown hole must succeed");
    Verdict::Unknown { hole_id }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Build a kernel `Context` from the obligation triple's open context Γ.
/// Entries are pushed in order so that `context[0]` is the outermost binder.
fn context_from_triple(triple: &ObligationTriple) -> Context {
    let mut ctx = Context::new();
    for ty in &triple.context {
        ctx.push(ty.clone());
    }
    ctx
}
