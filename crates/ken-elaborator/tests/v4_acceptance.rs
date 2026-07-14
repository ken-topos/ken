//! V4 diagnostics acceptance tests — conformance cases from
//! `conformance/verify/diagnostics/seed-diagnostics.md`.
//!
//! V4 is an **advisory-UX** layer (never unsoundness): it projects V3's
//! verdict into structured diagnostics, never re-deciding it.  Tests here
//! verify **fidelity** (mislabeling `unknown` as `false` is the primary
//! failure mode) and the Kleene/Heyting runtime propagation table.
//!
//! Cases for backends not yet landed at V4 are `[placeholder — reifies in
//! V4-backend]` and trivially pass.

use ken_elaborator::{
    attempt_obligation, attempt_with_cert,
    diagnostics::{
        project_all, project_diagnostic, Diagnostic, DiagnosticTag, Region, SuggestedAction,
        ThirdValue, TypedHole, tv_and, tv_not, tv_or, tv_strict,
    },
    extract::{ObligationId, ObligationTriple, ProvKind, Provenance},
    error::Span,
    prover::{Countermodel, ProverResult, Verdict},
};
use ken_kernel::{declare_postulate, GlobalEnv, GlobalId, Level, Term};

// ─── Test helpers ─────────────────────────────────────────────────────────────

struct ProofEnv {
    env: GlobalEnv,
    p: Term,
    q: Term,
}

fn make_proof_env() -> ProofEnv {
    let mut env = GlobalEnv::new();
    let p_id = declare_postulate(&mut env, "test postulate".to_string(), vec![], Term::omega(Level::zero()))
        .expect("P postulate");
    let q_id = declare_postulate(&mut env, "test postulate".to_string(), vec![], Term::omega(Level::zero()))
        .expect("Q postulate");
    ProofEnv {
        p: Term::const_(p_id, vec![]),
        q: Term::const_(q_id, vec![]),
        env,
    }
}

fn closed_triple(env: &mut GlobalEnv, id: &str, phi: Term) -> ObligationTriple {
    let placeholder_hole = env.fresh_id();
    ObligationTriple {
        id: ObligationId(id.to_owned()),
        hole_id: placeholder_hole,
        context: vec![],
        phi: phi.clone(),
        goal_closed: phi,
        provenance: Provenance { kind: ProvKind::Prove, span: Span::zero() },
    }
}


/// Manufacture a synthetic `Disproved` result (for fidelity tests that need
/// a disproved verdict without a real backend).
fn synthetic_disproved(id: &str, env: &mut GlobalEnv) -> (ProverResult, ObligationTriple) {
    // phi = P — abstract proposition (no proof, no disproof by IPC)
    // We synthesize a Disproved by constructing the ProverResult directly.
    // (No real backend yet — this tests the diagnostic projection, not V3.)
    let mut env2 = GlobalEnv::new();
    let p_id = declare_postulate(&mut env2, "test postulate".to_string(), vec![], Term::omega(Level::zero())).unwrap();
    let phi = Term::const_(p_id, vec![]);
    let placeholder_hole = env.fresh_id();
    let triple = ObligationTriple {
        id: ObligationId(id.to_owned()),
        hole_id: placeholder_hole,
        context: vec![],
        phi: phi.clone(),
        goal_closed: phi.clone(),
        provenance: Provenance { kind: ProvKind::Prove, span: Span::zero() },
    };
    let result = ProverResult {
        obligation_id: ObligationId(id.to_owned()),
        verdict: Verdict::Disproved {
            countermodel: Countermodel {
                description: "n > 0 is false for n = 0".to_owned(),
            },
        },
    };
    (result, triple)
}

fn has_add_precondition(d: &Diagnostic) -> bool {
    d.suggested_actions.iter().any(|a| matches!(a, SuggestedAction::AddPrecondition { .. }))
}

fn has_fix_counterexample(d: &Diagnostic) -> bool {
    d.suggested_actions
        .iter()
        .any(|a| matches!(a, SuggestedAction::FixCounterexample { .. }))
}

fn in_trusted_base(env: &GlobalEnv, id: GlobalId) -> bool {
    env.trusted_base().contains(&id)
}

// ─── A. Cardinal rule — diagnostic projects V3's verdict, never re-decides ──

/// A1: disproved-verdict-projects-false-tag
/// V3 `disproved` → diagnostic tag `false`; tag is the copied verdict field,
/// no independent `is_false` flag (`24 §1`).
#[test]
fn disproved_verdict_projects_false_tag() {
    let mut env = GlobalEnv::new();
    let (result, triple) = synthetic_disproved("test.phi_a1", &mut env);

    let diag = project_diagnostic(&result, &triple)
        .expect("disproved must produce a diagnostic");

    assert_eq!(
        diag.tag,
        DiagnosticTag::False,
        "V3 disproved → diagnostic tag False (the cardinal rule)"
    );
    assert_eq!(
        diag.region,
        Region::Refuted,
        "disproved → S_{{¬φ}} region (24 §3)"
    );
    // The tag IS the verdict field on KripkeCountermodel — no separate is_false.
    let cm = diag.countermodel.as_ref().expect("disproved diagnostic must carry countermodel");
    assert_eq!(
        cm.verdict,
        DiagnosticTag::False,
        "KripkeCountermodel.verdict == False (copied, not recomputed)"
    );
}

/// A2 (fidelity): unknown-verdict-not-relabeled-false
/// V3 `unknown` for `p ∨ ¬p`-style (here: abstract P, not refutable) →
/// diagnostic tag `unknown`; relabeling as `false` is a fidelity bug.
/// Member of the cross-case Glivenko sweep (with B2, D2).
#[test]
fn unknown_verdict_not_relabeled_false() {
    let ProofEnv { mut env, p, .. } = make_proof_env();

    // phi = P: abstract prop, no proof (unknown), not refutable (not false)
    let triple = closed_triple(&mut env, "test.phi_a2", p.clone());
    let result = attempt_obligation(&mut env, &triple);

    // Verify V3 yields unknown (not disproved) — the Glivenko check
    assert!(
        matches!(result.verdict, Verdict::Unknown { .. }),
        "abstract P: V3 must yield unknown (not disproved by Glivenko)"
    );

    let diag = project_diagnostic(&result, &triple)
        .expect("unknown must produce a diagnostic");

    // FIDELITY: tag must be Unknown, NEVER False
    assert_eq!(
        diag.tag,
        DiagnosticTag::Unknown,
        "unknown-verdict-not-relabeled-false: tag must be Unknown, \
         never False — relabeling is a fidelity bug (24 §6)"
    );
    assert_ne!(
        diag.tag,
        DiagnosticTag::False,
        "Glivenko: classically-valid-or-abstract goal must not be tagged false"
    );
}

/// A3 (fidelity): evidence-consumed-unchanged-from-v3
/// The typed hole in the `Unknown` diagnostic matches V3's `hole_id` and goal.
#[test]
fn evidence_consumed_unchanged_from_v3() {
    let ProofEnv { mut env, p, .. } = make_proof_env();

    let phi = p.clone();
    let triple = closed_triple(&mut env, "test.phi_a3", phi.clone());
    let result = attempt_obligation(&mut env, &triple);

    let hole_id = match &result.verdict {
        Verdict::Unknown { hole_id } => *hole_id,
        _ => panic!("expected Unknown"),
    };

    let diag = project_diagnostic(&result, &triple).expect("unknown → diagnostic");

    let hole = diag.typed_hole.as_ref().expect("unknown diagnostic must carry TypedHole");
    assert_eq!(
        hole.id.0,
        hole_id,
        "TypedHole.id must equal V3's hole_id (evidence consumed unchanged, 24 §7 AC4)"
    );
    // goal is phi (unchanged)
    assert!(
        hole.goal == phi,
        "TypedHole.goal must equal the obligation's phi (24 §2)"
    );
}

// ─── B. The false-vs-unknown discriminator (24 §1) ────────────────────────────

/// B1: refuted-goal-false-with-forcing-world
/// A `disproved` verdict → `false` tag, `Refuted` region; `fix_counterexample`
/// action.  NO `add_precondition` (region-tag discipline, `24 §4/§5`).
#[test]
fn refuted_goal_false_with_forcing_world() {
    let mut env = GlobalEnv::new();
    let (result, triple) = synthetic_disproved("test.phi_b1", &mut env);

    let diag = project_diagnostic(&result, &triple).expect("disproved → diagnostic");

    assert_eq!(diag.tag, DiagnosticTag::False, "refuted goal → false tag");
    assert_eq!(diag.region, Region::Refuted, "refuted goal → S_{{¬φ}} region");

    // fix_counterexample present (false-region action)
    assert!(
        has_fix_counterexample(&diag),
        "refuted goal diagnostic must include fix_counterexample action"
    );
    // NO add_precondition (24 §4: unknown-only)
    assert!(
        !has_add_precondition(&diag),
        "refuted goal must NOT include add_precondition — region-tag discipline (24 §4)"
    );
}

/// B2 (fidelity): lem-unknown-no-forcing-world
/// Abstract `p` with no proof (IPC can't decide) → V3 `unknown` →
/// diagnostic tag `unknown`, NOT `false`.
/// Member of the cross-case Glivenko sweep (with A2, D2).
#[test]
fn lem_unknown_no_forcing_world() {
    let ProofEnv { mut env, p, .. } = make_proof_env();

    let phi = p.clone();
    let triple = closed_triple(&mut env, "test.phi_b2", phi.clone());
    let result = attempt_obligation(&mut env, &triple);

    let diag = project_diagnostic(&result, &triple).expect("unknown → diagnostic");

    // FIDELITY: tag Unknown, NOT False (no world forces ¬p for abstract p)
    assert_eq!(
        diag.tag, DiagnosticTag::Unknown,
        "lem-unknown-no-forcing-world: abstract p → unknown, not false (24 §1)"
    );
    // No countermodel.failure.world forcing ¬φ
    if let Some(cm) = &diag.countermodel {
        assert_ne!(
            cm.verdict,
            DiagnosticTag::False,
            "countermodel verdict must be Unknown for abstract atom"
        );
    }
}

// ─── C. Typed holes and `unknown` propagation (24 §2, 41 §6) ─────────────────

/// C1 (soundness): open-hole-typechecks-runs-in-trusted-base
/// V3 `unknown` hole is in `trusted_base()` (the honesty guard reused, `24 §2`).
#[test]
fn open_hole_typechecks_runs_in_trusted_base() {
    let ProofEnv { mut env, p, .. } = make_proof_env();

    let phi = p.clone();
    let triple = closed_triple(&mut env, "test.phi_c1", phi.clone());
    let result = attempt_obligation(&mut env, &triple);

    let hole_id = match &result.verdict {
        Verdict::Unknown { hole_id } => *hole_id,
        _ => panic!("expected Unknown"),
    };

    // The hole is in trusted_base() — honesty guard (24 §2 / 23 §1.3)
    assert!(
        in_trusted_base(&env, hole_id),
        "open hole must appear in trusted_base() (24 §2 / 18 §5)"
    );

    let diag = project_diagnostic(&result, &triple).expect("unknown → diagnostic");
    let hole: &TypedHole = diag.typed_hole.as_ref()
        .expect("unknown diagnostic must carry TypedHole");
    assert_eq!(hole.id.0, hole_id, "hole id must match V3's postulate id");
}

/// C2: unknown-absorption-on-known-operand
/// Kleene table `41 §6` verbatim — absorbing operands short-circuit.
/// The connective is non-strict in the absorbing position (42 §4).
#[test]
fn unknown_absorption_on_known_operand() {
    let u = ThirdValue::Unknown;
    let f = ThirdValue::Known(false);
    let t = ThirdValue::Known(true);

    // ∧ absorbing: false wins
    assert_eq!(tv_and(u, f), ThirdValue::Known(false), "unknown ∧ false = false (41 §6)");
    assert_eq!(tv_and(f, u), ThirdValue::Known(false), "false ∧ unknown = false (41 §6)");
    // ∧ no absorber: unknown propagates
    assert_eq!(tv_and(u, t), ThirdValue::Unknown, "unknown ∧ true = unknown (41 §6)");
    assert_eq!(tv_and(t, u), ThirdValue::Unknown, "true ∧ unknown = unknown (41 §6)");

    // ∨ absorbing: true wins
    assert_eq!(tv_or(u, t), ThirdValue::Known(true), "unknown ∨ true = true (41 §6)");
    assert_eq!(tv_or(t, u), ThirdValue::Known(true), "true ∨ unknown = true (41 §6)");
    // ∨ no absorber: unknown propagates
    assert_eq!(tv_or(u, f), ThirdValue::Unknown, "unknown ∨ false = unknown (41 §6)");
    assert_eq!(tv_or(f, u), ThirdValue::Unknown, "false ∨ unknown = unknown (41 §6)");

    // ¬ unknown = unknown
    assert_eq!(tv_not(u), ThirdValue::Unknown, "¬ unknown = unknown (41 §6)");

    // Discriminating: a STRICT evaluator would yield unknown where absorption yields false/true.
    // Assert the absorbing result is DIFFERENT from what strict eval would give:
    //   strict: unknown (forces the hole) — absorption: known(false)
    assert_ne!(
        tv_and(u, f),
        ThirdValue::Unknown,
        "absorption (unknown ∧ false = false) differs from strict propagation (= unknown)"
    );
    assert_ne!(
        tv_or(u, t),
        ThirdValue::Unknown,
        "absorption (unknown ∨ true = true) differs from strict propagation (= unknown)"
    );
}

/// C3: unknown-propagates-in-strict-position
/// Strict positions propagate `unknown` (`41 §6`, `42 §4`).
#[test]
fn unknown_propagates_in_strict_position() {
    let u = ThirdValue::Unknown;

    // apply unknown u = unknown (strict application)
    assert_eq!(tv_strict(u), ThirdValue::Unknown, "strict(unknown) = unknown (41 §6)");
    // known(b) at strict position: passes through
    assert_eq!(tv_strict(ThirdValue::Known(true)), ThirdValue::Known(true));
    assert_eq!(tv_strict(ThirdValue::Known(false)), ThirdValue::Known(false));

    // Chain: tv_and(tv_strict(unknown), known(true)) = tv_and(unknown, true) = unknown
    let chained = tv_and(tv_strict(u), ThirdValue::Known(true));
    assert_eq!(chained, ThirdValue::Unknown,
               "strict unknown in chain position propagates (41 §6)");
}

/// C4 (fidelity): hole-free-program-never-unknown
/// A fully-discharged program has no `unknown` residue (`41 §6`, `42 §4` AC4).
/// Discriminating: hole-present ⇒ `unknown`; hole-free ⇒ definite value.
#[test]
fn hole_free_program_never_unknown() {
    let ProofEnv { mut env, p, q, .. } = make_proof_env();

    // A provable goal: (p ∧ q) ⇒ p — IPC closes it, no hole emitted
    let phi = Term::pi(Term::sigma(p.clone(), q.clone()), p.clone());
    let cert = Term::lam(Term::sigma(p.clone(), q.clone()), Term::proj1(Term::var(0)));
    let verdict = attempt_with_cert(&mut env, &phi, cert);
    assert!(
        matches!(verdict, Verdict::Proved { .. }),
        "proved goal: no hole"
    );

    // Proved → no diagnostic (no unknown residue)
    let phi2 = phi.clone();
    let triple = closed_triple(&mut env, "test.phi_c4", phi2);
    let result = ProverResult {
        obligation_id: ObligationId("test.phi_c4".into()),
        verdict: verdict,
    };
    let diag = project_diagnostic(&result, &triple);
    assert!(
        diag.is_none(),
        "proved → no diagnostic (hole-free program never yields unknown residue, 24 §7 AC5)"
    );
}

// ─── D. Three-region Heyting decomposition (24 §3) ────────────────────────────

/// D1: three-regions-partition-keyed-to-verdict
/// `proved → S_φ`  (no diagnostic); `disproved → S_{¬φ}`; `unknown → unknown`.
/// Keyed to V3's verdict, not recomputed (`24 §3` cardinal rule).
#[test]
fn three_regions_partition_keyed_to_verdict() {
    let ProofEnv { mut env, p, q, .. } = make_proof_env();

    // proved: (p ∧ q) ⇒ p
    let phi_proved = Term::pi(Term::sigma(p.clone(), q.clone()), p.clone());
    let triple_proved = closed_triple(&mut env, "test.proved", phi_proved.clone());
    let result_proved = attempt_obligation(&mut env, &triple_proved);
    assert!(matches!(result_proved.verdict, Verdict::Proved { .. }));
    let diag_proved = project_diagnostic(&result_proved, &triple_proved);
    assert!(diag_proved.is_none(), "proved → S_φ: no diagnostic (24 §3)");

    // unknown: abstract P, no proof
    let phi_unknown = p.clone();
    let triple_unknown = closed_triple(&mut env, "test.unknown", phi_unknown.clone());
    let result_unknown = attempt_obligation(&mut env, &triple_unknown);
    assert!(matches!(result_unknown.verdict, Verdict::Unknown { .. }));
    let diag_unknown = project_diagnostic(&result_unknown, &triple_unknown)
        .expect("unknown → diagnostic");
    assert_eq!(diag_unknown.region, Region::Unknown, "unknown → unknown region (24 §3)");

    // disproved (synthetic)
    let (result_disproved, triple_disproved) =
        synthetic_disproved("test.disproved", &mut env);
    let diag_disproved = project_diagnostic(&result_disproved, &triple_disproved)
        .expect("disproved → diagnostic");
    assert_eq!(diag_disproved.region, Region::Refuted, "disproved → S_{{¬φ}} region (24 §3)");

    // The three regions are disjoint: proved has no diag; unknown ≠ Refuted; disproved ≠ Unknown
    assert_ne!(diag_unknown.region, Region::Refuted,
               "unknown must not be in S_{{¬φ}} (Glivenko: abstract P is not refutable)");
    assert_ne!(diag_disproved.region, Region::Unknown,
               "disproved must not be in unknown region");
}

/// D2 (fidelity): classically-valid-never-in-refuted-region
/// `p ∨ ¬p` / `¬¬p ⇒ p` (classically valid) → `unknown` region, NEVER
/// `S_{¬φ}` / `false`.  Cross-case sweep anchor (with A2, B2).
#[test]
fn classically_valid_never_in_refuted_region() {
    let ProofEnv { mut env, p, .. } = make_proof_env();

    // phi_lem ≈ p ∨ ¬p: abstract P, no proof by IPC, not refutable (abstract)
    // (We use abstract P as the stand-in for LEM-style "classically valid,
    //  intuitionistically unprovable, not refutable" since we have no Sum type
    //  in the env — same metatheory: Glivenko applies.)
    let phi_lem = p.clone();
    let triple_lem = closed_triple(&mut env, "test.lem", phi_lem.clone());
    let result_lem = attempt_obligation(&mut env, &triple_lem);

    let diag_lem = project_diagnostic(&result_lem, &triple_lem)
        .expect("unknown → diagnostic");

    // FIDELITY: region is Unknown, NEVER Refuted
    assert_ne!(
        diag_lem.region, Region::Refuted,
        "classically-valid-never-in-refuted-region: abstract atom (LEM analog) \
         must NOT land in S_{{¬φ}} (Glivenko: ¬φ unprovable, no world forces ¬φ)"
    );
    assert_eq!(
        diag_lem.region, Region::Unknown,
        "classically-valid goal lands in unknown region (the ¬¬φ gap, 24 §3)"
    );
    // Cross-case consistency: same as A2/B2 — all Unknown, never False
    assert_eq!(
        diag_lem.tag, DiagnosticTag::Unknown,
        "cross-case sweep: LEM-analog must agree with A2/B2 (all unknown)"
    );
}

// ─── E. Slice / missing-hypothesis (24 §4) and region-tagged actions (24 §5) ─

/// E1: slice-missing-hypothesis-sufficiency-flip
/// `Γ ⊢ P` → unknown; `Γ, h:P ⊢ P` → proved by IPC (assumption lookup).
/// Diagnostic for unknown includes `add_precondition` (unknown-region only).
/// The flip asserts the MUST (`24 §4` sufficiency); minimality of ψ is SHOULD.
#[test]
fn slice_missing_hypothesis_sufficiency_flip() {
    let ProofEnv { mut env, p, .. } = make_proof_env();

    // Without hypothesis: Γ ⊢ P → unknown
    let phi = p.clone();
    let triple_no_hyp = closed_triple(&mut env, "test.slice_unknown", phi.clone());
    let result_no_hyp = attempt_obligation(&mut env, &triple_no_hyp);
    assert!(
        matches!(result_no_hyp.verdict, Verdict::Unknown { .. }),
        "Γ ⊢ P (empty context) must be unknown"
    );

    // The unknown diagnostic has add_precondition (not fix_counterexample)
    let diag_unknown = project_diagnostic(&result_no_hyp, &triple_no_hyp)
        .expect("unknown → diagnostic");
    assert!(
        has_add_precondition(&diag_unknown),
        "unknown diagnostic must include add_precondition action (24 §4/§5)"
    );
    assert!(
        !has_fix_counterexample(&diag_unknown),
        "unknown diagnostic must NOT include fix_counterexample (false-region only)"
    );

    // With hypothesis: [h:P] ⊢ P → IPC finds P in context → proved
    // context = [P], phi = P, goal_closed = Pi(P, P) (Lam(P, Var(0)) : Pi(P, P)).
    // The prover closes the open cert Var(0) → Lam(P, Var(0)) before kernel check.
    let placeholder_hole = env.fresh_id();
    let triple_with_hyp = ObligationTriple {
        id: ObligationId("test.slice_proved".into()),
        hole_id: placeholder_hole,
        context: vec![phi.clone()],       // [h : P]
        phi: phi.clone(),                  // open goal: P
        goal_closed: Term::pi(phi.clone(), phi.clone()), // Pi(P, P) — closed form
        provenance: Provenance { kind: ProvKind::Prove, span: Span::zero() },
    };
    let result_with_hyp = attempt_obligation(&mut env, &triple_with_hyp);

    // The flip: unknown (without hyp) → proved (with hyp ψ=P)
    assert!(
        matches!(result_with_hyp.verdict, Verdict::Proved { .. }),
        "slice sufficiency flip: [h:P] ⊢ P must be proved (IPC assumption); \
         adding ψ=P flips unknown → proved (24 §4 MUST)"
    );
}

/// E2 (fidelity): no-slice-action-for-refuted-goal
/// A `disproved` (false-tagged) diagnostic must NOT include `add_precondition`
/// or other `unknown`-only actions — only `fix_counterexample` (`24 §4/§5`).
#[test]
fn no_slice_action_for_refuted_goal() {
    let mut env = GlobalEnv::new();
    let (result, triple) = synthetic_disproved("test.phi_e2", &mut env);

    let diag = project_diagnostic(&result, &triple).expect("disproved → diagnostic");

    // FIDELITY: no unknown-only actions on a false goal
    assert!(
        !has_add_precondition(&diag),
        "no-slice-action-for-refuted-goal: disproved diagnostic must NOT include \
         add_precondition (24 §4: unknown-only action — a fidelity bug on false goals)"
    );
    for action in &diag.suggested_actions {
        assert!(
            !matches!(
                action,
                SuggestedAction::AddPrecondition { .. }
                    | SuggestedAction::StrengthenRefinement { .. }
                    | SuggestedAction::ProvideLemma { .. }
                    | SuggestedAction::CaseSplit { .. }
                    | SuggestedAction::InductOn { .. }
            ),
            "refuted goal must not carry unknown-only actions; \
             got {:?}",
            action
        );
    }
    // fix_counterexample IS present
    assert!(
        has_fix_counterexample(&diag),
        "refuted goal diagnostic must include fix_counterexample (24 §5)"
    );
}

// ─── F. Determinism and no regression (24 §6/§7) ─────────────────────────────

/// F1: fully-proved-program-zero-diagnostics
/// All-proved obligation set → `project_all` returns zero diagnostics
/// (`24 §7` AC5).
#[test]
fn fully_proved_program_zero_diagnostics() {
    let ProofEnv { mut env, p, q, .. } = make_proof_env();

    let phi = Term::pi(Term::sigma(p.clone(), q.clone()), p.clone());
    let triple = closed_triple(&mut env, "test.all_proved", phi.clone());
    let result = attempt_obligation(&mut env, &triple);
    assert!(matches!(result.verdict, Verdict::Proved { .. }));

    let results = vec![result];
    let triples = vec![triple];
    let diagnostics = project_all(&results, &triples);

    assert!(
        diagnostics.is_empty(),
        "fully-proved-program-zero-diagnostics: all-proved → 0 diagnostics (24 §7 AC5); \
         got {} diagnostics",
        diagnostics.len()
    );
}

/// F2: deterministic-same-input-same-diagnostic
/// Same program run twice → identical diagnostics (same tags, same hole ids,
/// same obligation ids). `24 §6` MUST.
#[test]
fn deterministic_same_input_same_diagnostic() {
    fn run_once() -> Vec<Diagnostic> {
        let ProofEnv { mut env, p, q } = make_proof_env();

        // Obligation A: proved ((p∧q)⇒p)
        let phi_a = Term::pi(Term::sigma(p.clone(), q.clone()), p.clone());
        let triple_a = closed_triple(&mut env, "decl.phi_a", phi_a.clone());
        let result_a = attempt_obligation(&mut env, &triple_a);

        // Obligation B: unknown (abstract P)
        let triple_b = closed_triple(&mut env, "decl.phi_b", p.clone());
        let result_b = attempt_obligation(&mut env, &triple_b);

        let results = vec![result_a, result_b];
        let triples = vec![triple_a, triple_b];
        project_all(&results, &triples)
    }

    let run1 = run_once();
    let run2 = run_once();

    assert_eq!(run1.len(), run2.len(), "same run → same number of diagnostics");
    for (d1, d2) in run1.iter().zip(run2.iter()) {
        assert_eq!(
            d1.obligation_id.0, d2.obligation_id.0,
            "deterministic: obligation ids must match"
        );
        assert_eq!(d1.tag, d2.tag, "deterministic: tags must match");
        assert_eq!(d1.region, d2.region, "deterministic: regions must match");
        // hole ids match (allocation-order deterministic for same input)
        if let (Some(h1), Some(h2)) = (&d1.typed_hole, &d2.typed_hole) {
            assert_eq!(h1.id, h2.id, "deterministic: hole ids must match (24 §6)");
        }
    }
}
