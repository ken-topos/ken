//! V2 obligation extraction: obligation triples with stable ids, open context Γ,
//! and provenance (`22-obligations.md` §5/§1, §6).
//!
//! Consumes V1's `ElabResult` and produces `ExtractionResult` — the obligation
//! set keyed for the V3 prover (`22 §6`).
//!
//! **Absent-clause scan / exhaustiveness property (`22 §2.5`):** the `match` in
//! `lift_obligation` has **no catch-all `_ ⇒ skip` arm** — every `ObligationKind`
//! variant is handled explicitly. Adding a new variant without an arm is a
//! **compile error**, not a silent miss. This is what makes "a missing clause is
//! a visible gap, not a silent drop" concrete.

use ken_kernel::{GlobalId, Term};

use crate::elab::{ElabResult, Obligation, ObligationKind};
use crate::error::Span;

/// Stable obligation identifier (`22 §1`, `24 §6`).
///
/// Format: `"{def_name}.{kind}.{index}"` — stable across edits unrelated to
/// the clause (the def_name + kind + sequential index within the def are fixed
/// by the clause identity, not by global position).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObligationId(pub String);

/// Source clause kind for obligation provenance (`22 §2`, §1).
#[derive(Debug, Clone)]
pub enum ProvKind {
    /// `ensures ψ` or return-type refinement `{ x : A | φ }` (`22 §2.2`/§2.1).
    Ensures { index: usize },
    /// `prove name : φ` (`22 §2.4`).
    Prove,
    /// `law Name { field : φ }` field (`22 §2.4`).
    LawField { field_name: String },
    /// Precondition at a call site (`22 §2.3`).
    /// `[placeholder — reifies in V3]`: requires Eq + proof-arg tracking.
    CallPrecond,
    /// Partial-primitive side condition (`22 §2.4`, `35 §3`).
    /// `[placeholder — reifies in V4]`: requires runtime primitive recognition.
    PartialPrim,
}

/// Obligation provenance: source clause + originating span (`22 §1`).
#[derive(Debug, Clone)]
pub struct Provenance {
    pub kind: ProvKind,
    pub span: Span,
}

/// V2 obligation triple `⟨id, Γ ⊢ φ, provenance⟩` (`22 §1`).
///
/// The triple is the **V2→V3 interface unit** (`22 §6`): each maps to one
/// proof attempt → one verdict (`21 §5.1`).
///
/// **Context note (`22 §3/§4`):** `context` currently carries the
/// parameter telescope from V1's elaboration. Path-sensitive extensions —
/// `let`-equations, match scrutinee equations, `if`-branch conditions, and
/// induction hypotheses — require `Eq` (`16 §2`) and eliminator walking,
/// which are K2 prerequisites. Those extensions are
/// `[placeholder — reifies in V3]`.
#[derive(Debug, Clone)]
pub struct ObligationTriple {
    /// Stable identifier across unrelated edits (`22 §1`, `24 §6`).
    pub id: ObligationId,
    /// Kernel postulate id for the open hole (`21 §5.4`, `trusted_base()`).
    pub hole_id: GlobalId,
    /// Open hypothesis context Γ at the obligation site (`22 §3`).
    /// V1 level: parameter telescope. Path-sensitive extensions deferred to V3.
    pub context: Vec<Term>,
    /// Open goal `φ : Ω` in context Γ (`22 §1`). Body is substituted into
    /// the predicate (e.g., `ψ[b/result]` for an `ensures ψ` over body `b`).
    pub phi: Term,
    /// Pi-closed form: `Pi(context[0], Pi(context[1], ..., phi))`.
    /// The form submitted to the kernel for re-checking (`18 §4.5`).
    pub goal_closed: Term,
    /// Source clause provenance (`22 §1`).
    pub provenance: Provenance,
}

/// V2 extraction result: the obligation set for the V3 prover (`22 §6`).
///
/// **Completeness of extraction is the verification-soundness linchpin (`22
/// preamble`):** a missed obligation is **not** caught downstream — the V1
/// honesty guard catches generated-but-undischarged holes, not un-generated
/// sites. The absent-clause scan (§2.5) — and this extractor's exhaustive-
/// by-construction `match` — is the sole safeguard.
#[derive(Debug)]
pub struct ExtractionResult {
    pub obligations: Vec<ObligationTriple>,
}

/// Extract V2 obligation triples from a V1 `ElabResult` (`22 §5`).
///
/// Enriches V1's obligation holes with stable ids, open context Γ (recovered
/// by un-Pi-ing `goal_closed`), and provenance.
///
/// **Exhaustive by construction (§2.5):** every `ObligationKind` variant has
/// an explicit arm in `lift_obligation` — no catch-all skip. Path-sensitive
/// Γ accumulation for `Let`/`Elim`/`If` (§3/§4) is `[placeholder — reifies
/// in V3]` pending K2 `Eq`.
pub fn v2_extract(elab_result: &ElabResult) -> ExtractionResult {
    let def_name = &elab_result.name;
    let obligations = elab_result
        .obligations
        .iter()
        .map(|obl| lift_obligation(def_name, obl))
        .collect();
    ExtractionResult { obligations }
}

/// Lift a V1 `Obligation` to a V2 `ObligationTriple`.
///
/// Every `ObligationKind` arm is explicit — NO catch-all — satisfying the
/// §2.5 exhaustiveness property. Adding a new variant without an arm is a
/// compile error, making a silently-missed obligation kind impossible.
fn lift_obligation(def_name: &str, obl: &Obligation) -> ObligationTriple {
    let (context, phi) = unclose_goal(&obl.goal_closed);

    // Exhaustive match over ObligationKind — NO `_ =>` arm (§2.5).
    let (id_str, prov_kind) = match &obl.kind {
        ObligationKind::Ensures => (
            format!("{}.ensures.{}", def_name, obl.id),
            ProvKind::Ensures { index: obl.id as usize },
        ),
        ObligationKind::Prove => (
            format!("{}.prove", def_name),
            ProvKind::Prove,
        ),
        ObligationKind::LawField(field) => (
            format!("{}.law.{}", def_name, field),
            ProvKind::LawField { field_name: field.clone() },
        ),
    };

    ObligationTriple {
        id: ObligationId(id_str),
        hole_id: obl.hole_id,
        context,
        phi,
        goal_closed: obl.goal_closed.clone(),
        provenance: Provenance { kind: prov_kind, span: obl.span.clone() },
    }
}

/// Recover open context and open goal from a Pi-closed goal.
///
/// `Pi(T₀, Pi(T₁, ..., phi))` → `([T₀, T₁, ...], phi)`.
/// Inverse of `close_goal` in `elab.rs`.
fn unclose_goal(goal_closed: &Term) -> (Vec<Term>, Term) {
    let mut context = Vec::new();
    let mut cur = goal_closed.clone();
    loop {
        match cur {
            Term::Pi(ty, body) => {
                context.push(*ty);
                cur = *body;
            }
            other => return (context, other),
        }
    }
}
