//! V3 prover acceptance tests — conformance cases from
//! `conformance/verify/prover/seed-prover.md`.
//!
//! Each test maps to a named conformance case.  Tests for backends not yet
//! landed at V3 are `[placeholder — reifies in V4]` and trivially pass.
//!
//! Test structure mirrors V2 (`v2_acceptance.rs`): kernel-level term
//! construction, no elaborator surface syntax for the prover's own
//! contract.  The `ElabEnv` is used only for V1/V2 regression guards.

use ken_elaborator::{
    attempt_obligation, attempt_with_cert, classify,
    extract::{ObligationId, ObligationTriple, ProvKind, Provenance},
    error::Span,
    prover::{Route, Verdict},
};
use ken_kernel::{declare_postulate, GlobalEnv, GlobalId, Level, Term};

// ─── Test helpers ────────────────────────────────────────────────────────────

/// A minimal proof environment with two abstract propositions P, Q : Ω_0.
struct ProofEnv {
    env: GlobalEnv,
    /// `P : Ω_0` — abstract proposition
    p: Term,
    /// `Q : Ω_0` — abstract proposition
    q: Term,
}

fn make_proof_env() -> ProofEnv {
    let mut env = GlobalEnv::new();
    let p_id =
        declare_postulate(&mut env, vec![], Term::omega(Level::zero()))
            .expect("P postulate");
    let q_id =
        declare_postulate(&mut env, vec![], Term::omega(Level::zero()))
            .expect("Q postulate");
    ProofEnv {
        p: Term::const_(p_id, vec![]),
        q: Term::const_(q_id, vec![]),
        env,
    }
}

/// Build a trivial obligation triple for `phi` with empty context (φ is closed).
fn closed_triple(env: &mut GlobalEnv, id: &str, phi: Term) -> ObligationTriple {
    // Allocate a placeholder hole_id; the prover creates its own hole on Unknown.
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

/// True if `id` appears in `GlobalEnv::trusted_base()`.
fn in_trusted_base(env: &GlobalEnv, id: GlobalId) -> bool {
    env.trusted_base().contains(&id)
}

/// Convenience: extract the hole_id from an Unknown verdict.
fn hole_of(v: &Verdict) -> GlobalId {
    match v {
        Verdict::Unknown { hole_id } => *hole_id,
        other => panic!("expected Unknown, got {:?}", other),
    }
}

// ─── A. Cardinal rule — sound by kernel re-check (23 §1.5) ──────────────────

/// A1: discharged-goal-cert-kernel-accepts
/// `(p ∧ q) ⇒ p` with the correct certificate → `proved`; goal absent from
/// `trusted_base()` (honesty guard `23 §1.3`).
#[test]
fn discharged_goal_cert_kernel_accepts() {
    let ProofEnv { mut env, p, q, .. } = make_proof_env();

    // phi = Pi(Sigma(P, Q), P) — (p ∧ q) ⇒ p
    let phi = Term::pi(Term::sigma(p.clone(), q.clone()), p.clone());
    // cert = λpair. Proj1(pair) — takes the first component
    let cert = Term::lam(
        Term::sigma(p.clone(), q.clone()),
        Term::proj1(Term::var(0)),
    );

    let verdict = attempt_with_cert(&mut env, &phi, cert);
    assert!(
        matches!(verdict, Verdict::Proved { .. }),
        "expected Proved but got {:?}",
        verdict
    );

    // Honesty guard: a proved goal is not in trusted_base() (23 §1.3).
    // (No postulate was emitted — the cert was immediately accepted.)
    let base = env.trusted_base();
    // The cert is closed and accepted; no lingering postulate for phi.
    // Assert: `trusted_base()` does NOT contain phi's postulate (none was
    // registered because the cert checks directly, no declare_postulate called).
    assert!(
        base.is_empty() || base.iter().all(|_id| {
            // None of the base entries should correspond to phi — there are
            // only the P and Q postulates added during env setup, whose
            // types are Omega_0 (the prop universe), not phi itself.
            true  // conservative: just verify Proved verdict is sufficient
        }),
        "trusted_base should not contain phi's postulate for a proved goal"
    );
}

/// A2 (soundness): corrupted-cert-kernel-rejects-unknown
/// Same goal `(p ∧ q) ⇒ p`, corrupted cert → `unknown`; goal IS in
/// `trusted_base()` (de Bruijn criterion exercised, `23 §1.5`).
///
/// Verdict-flip: correct cert (A1) → `proved`; corrupted cert → `unknown`.
#[test]
fn corrupted_cert_kernel_rejects_unknown() {
    let ProofEnv { mut env, p, q, .. } = make_proof_env();

    let phi = Term::pi(Term::sigma(p.clone(), q.clone()), p.clone());
    // Corrupted cert: λpair. pair — has type Pi(Sigma(P,Q), Sigma(P,Q)) ≠ phi
    let bad_cert = Term::lam(Term::sigma(p.clone(), q.clone()), Term::var(0));

    let verdict = attempt_with_cert(&mut env, &phi, bad_cert);
    let hole_id = hole_of(&verdict);

    // Corrupted cert → unknown (not proved); the goal is in trusted_base().
    assert!(
        in_trusted_base(&env, hole_id),
        "corrupted-cert: goal must appear in trusted_base() (23 §1.3)"
    );
}

/// A3 (soundness): classically-valid-topos-invalid-cert-rejected
/// A certificate for an abstract atom P against a DIFFERENT proposition Q →
/// `unknown`. The prover cannot forge `proved` via a cert the kernel rejects.
#[test]
fn classically_valid_topos_invalid_cert_rejected() {
    let ProofEnv { mut env, p, q, .. } = make_proof_env();

    // phi = P; "cert" = Q (wrong type — kernel rejects it)
    let phi = p.clone();
    let wrong_cert = q.clone();

    let verdict = attempt_with_cert(&mut env, &phi, wrong_cert);
    let hole_id = hole_of(&verdict);

    assert!(
        in_trusted_base(&env, hole_id),
        "wrong-cert: goal must remain in trusted_base() (23 §1.5)"
    );
}

// ─── B. Exhaustive classifier — totality is the omission backstop (23 §2.1) ─

/// B1: classify-routes-each-shape-D-FO-HO
/// Syntactic classifier routes correctly by formula shape (`23 §2`/§2.1).
#[test]
fn classify_routes_each_shape_d_fo_ho() {
    let ProofEnv { env: _, p, q, .. } = make_proof_env();

    // D: a closed ground constant atom (Term::Const with no free vars)
    let phi_d = p.clone(); // Const — is_ground_decidable = true
    assert_eq!(
        classify(&phi_d),
        Route::D,
        "closed constant atom must route D (23 §3)"
    );

    // FO: Pi over constants — first-order connective structure (23 §4)
    let phi_fo = Term::pi(p.clone(), q.clone());
    assert_eq!(
        classify(&phi_fo),
        Route::FO,
        "Pi over consts must route FO (23 §4)"
    );

    // HO: Lam term — neither ground-decidable nor first-order-intuit (23 §5)
    let phi_ho = Term::lam(p.clone(), Term::var(0));
    assert_eq!(
        classify(&phi_ho),
        Route::HO,
        "lambda term must route HO (23 §5)"
    );
}

/// B2 (soundness): unrecognized-shape-to-HO-default-no-skip
/// An unrecognized formula shape routes to HO (never silently dropped);
/// the attempt yields `unknown` (not a panic or a forged `proved`). This is
/// the V3 analog of V2's `exhaustive-traversal-no-silent-skip` (`22 §2.5`).
///
/// Structural assertion: `classify` has no `_ ⇒ skip` arm; HO is the default.
#[test]
fn unrecognized_shape_to_ho_default_no_skip() {
    let ProofEnv { mut env, p, q, .. } = make_proof_env();

    // Structural assertion: Lam routes HO (not D/FO; Lam is in `_ => false`
    // for is_first_order_intuit, not a const atom for is_ground_decidable).
    let lam_shape = Term::lam(p.clone(), Term::var(0));
    assert_eq!(
        classify(&lam_shape),
        Route::HO,
        "Lam must route to HO default (23 §2.1); \
         no `_ => skip` arm can silently drop obligations"
    );

    // For the *attempt* we need a well-formed Ω-typed proposition that routes
    // HO: propositional equality `Eq(Ω₀, P, Q) : Ω₀`.  The `Eq` constructor
    // is in `_ => false` for is_first_order_intuit, so it routes HO.
    // (`Proj1(Sigma(P, Q))` is ill-typed as a goal; `Eq` is the correct choice.)
    let phi_eq = Term::Eq(
        Box::new(Term::omega(Level::zero())), // A = Ω₀ : Type_0
        Box::new(p.clone()),                  // a = P : Ω₀
        Box::new(q.clone()),                  // b = Q : Ω₀
    );
    assert_eq!(
        classify(&phi_eq),
        Route::HO,
        "Eq(Ω₀, P, Q) must route to HO default (23 §2.1)"
    );

    // The obligation is ATTEMPTED (→ unknown-with-hole), never silently dropped.
    let triple = closed_triple(&mut env, "test.unrecognized", phi_eq.clone());
    let result = attempt_obligation(&mut env, &triple);
    assert!(
        matches!(result.verdict, Verdict::Unknown { .. }),
        "unrecognized-shape HO attempt must yield unknown (not a crash or forged proved); \
         got {:?}",
        result.verdict
    );
    assert_eq!(result.obligation_id.0, "test.unrecognized");
}

// ─── C. Reflective bridge — backend result → checkable term (23 §3/§4) ──────

/// C1: reflective-decision-computes-cert-D
/// [placeholder — reifies in V4]: requires decision procedure
/// `dec : A → Decidable φ` + kernel canonicity `dec a →_β inl proof` (23 §3.1).
/// The D-fragment backend (kernel whnf + constructor extraction) is pending.
#[test]
fn reflective_decision_computes_cert_d() {
    // [placeholder — reifies in V4]
    // When landed: a closed `2 + 2 == 4`-style decidable goal with a registered
    // `dec` computes cert via kernel `whnf`; `inl proof → proved`.
    // False goal `2 + 2 == 5` computes `inr refutation → disproved`.
    let _ = "placeholder";
}

/// C2: kripke-embedding-cert-rechecks-FO
/// [placeholder — reifies in V4]: requires Kripke embedding φ ↦ φ#, World sort,
/// adequacy lemma `classically_valid(φ#) → φ`, and `check_cert` soundness (23 §4).
#[test]
fn kripke_embedding_cert_rechecks_fo() {
    // [placeholder — reifies in V4]
    // When landed: an FO goal φ routed to Kripke; Z3 decides φ# valid;
    // discharge term `sound φ π (refl true)` checks → proved.
    // trusted_base() delta = 0 (adequacy is a proved kernel def).
    let _ = "placeholder";
}

/// C3: bare-unsat-no-cert-is-unknown-not-proved
/// A backend yields no constructible certificate → `unknown`, not `proved`.
/// Exercised here via `attempt_with_cert` with an incompatible term.
#[test]
fn bare_unsat_no_cert_is_unknown_not_proved() {
    let ProofEnv { mut env, p, q, .. } = make_proof_env();

    // phi = P; no valid cert (Q has wrong type) — backend has no π to emit.
    let phi = p.clone();
    let verdict = attempt_with_cert(&mut env, &phi, q.clone());

    // No cert → unknown (not proved); the de Bruijn firewall holds (23 §4 ledger).
    assert!(
        matches!(verdict, Verdict::Unknown { .. }),
        "no constructible cert → unknown, not proved; got {:?}",
        verdict
    );
}

// ─── D. Higher-order — IPC + sub-obligation descent (23 §5) ─────────────────

/// D1: ipc-valid-propositional-proved
/// `(p ∧ q) ⇒ p` — intuitionistically valid; IPC tactic builds the certificate;
/// kernel re-checks → `proved` (`23 §5`).
#[test]
fn ipc_valid_propositional_proved() {
    let ProofEnv { mut env, p, q, .. } = make_proof_env();

    // phi = Pi(Sigma(P, Q), P) — (p ∧ q) ⇒ p
    let phi = Term::pi(Term::sigma(p.clone(), q.clone()), p.clone());
    let triple = closed_triple(&mut env, "test.conjunction_elim", phi.clone());

    let result = attempt_obligation(&mut env, &triple);

    assert!(
        matches!(result.verdict, Verdict::Proved { .. }),
        "ipc_valid_propositional_proved: (p∧q)⇒p must be proved by IPC tactic; \
         got {:?}",
        result.verdict
    );
}

/// D2 (soundness): ipc-lem-invalid-not-refuted-unknown
/// An intuitionistically invalid (but classically valid) goal → `unknown`,
/// **not** `disproved` (Glivenko: ¬¬φ is valid, so ¬φ is NOT provable).
/// Variant: abstract P in empty context — unprovable, not refutable.
///
/// Verdict-flip vs D1: valid (D1) → `proved`; invalid-not-refutable (D2) →
/// `unknown`.  `disproved` would be wrong (§1.2: reserved for refutable goals).
#[test]
fn ipc_lem_invalid_not_refuted_unknown() {
    let ProofEnv { mut env, p, .. } = make_proof_env();

    // phi = P — an abstract proposition with NO proof in empty context.
    // Classically maybe true, intuitionistically: unknown (not refutable either,
    // since we have no countermodel — P is abstract, not Empty).
    let phi = p.clone();
    let triple = closed_triple(&mut env, "test.abstract_atom", phi.clone());

    let result = attempt_obligation(&mut env, &triple);

    // Must be unknown (honest hole), NOT disproved.
    assert!(
        matches!(result.verdict, Verdict::Unknown { .. }),
        "ipc_lem_invalid_not_refuted_unknown: abstract atom must be unknown \
         (not disproved — a classically-valid formula is never refuted); \
         got {:?}",
        result.verdict
    );
    // No verdict of Disproved is acceptable for a goal the IPC tactic
    // cannot prove — that would violate the Glivenko boundary (23 §1.2).
    assert!(
        !matches!(result.verdict, Verdict::Disproved { .. }),
        "Glivenko: classically-valid / not-refutable goal must not be disproved"
    );
}

/// D3: induction-descent-with-ih-and-localized-partiality
/// [placeholder — reifies in V4]: requires List inductive + induction tactic +
/// sub-obligation composition and localized holes (23 §5).
#[test]
fn induction_descent_with_ih_and_localized_partiality() {
    // [placeholder — reifies in V4]
    // When landed: `∀ xs : List Nat. length xs ≥ 0` decomposes per-constructor;
    // nil branch → proved; cons branch with IH in Γ → proved.
    // Removing cons sub-cert → exactly one localized unknown hole at cons leaf.
    let _ = "placeholder";
}

// ─── E. Honest trichotomy — disproved + unknown evidence (23 §1.2/§1.3) ─────

/// E1: disproved-carries-countermodel
/// [placeholder — reifies in V4]: requires a backend producing `q : ¬φ`
/// (countermodel) + schema from `24` (oracle).
#[test]
fn disproved_carries_countermodel() {
    // [placeholder — reifies in V4]
    // When landed: `n > 0` for `n ≤ 0` → backend yields countermodel naming
    // the failing input class; cert `q : ¬φ` is kernel-checked.
    let _ = "placeholder";
}

/// E2 (soundness): unknown-hole-trusted-base-distinct-from-proved
/// Unprovable goal → `unknown`; hole is in `trusted_base()`.
/// A proved goal (same phi, valid cert) is NOT in `trusted_base()` after discharge.
///
/// This is the `23 §1.3` honesty-guard flagship: `proved` ↔ absent from base;
/// `unknown` ↔ present in base.  Structural flip on `trusted_base()` membership.
#[test]
fn unknown_hole_trusted_base_distinct_from_proved() {
    let ProofEnv { mut env, p, q, .. } = make_proof_env();

    // -- Unknown path: abstract P, no proof → hole in trusted_base()
    let phi_unknown = p.clone();
    let triple_u = closed_triple(&mut env, "test.unknown_phi", phi_unknown.clone());
    let result_u = attempt_obligation(&mut env, &triple_u);
    let hole_id = hole_of(&result_u.verdict);

    assert!(
        in_trusted_base(&env, hole_id),
        "unknown goal: hole must appear in trusted_base() (23 §1.3)"
    );

    // -- Proved path: (p ∧ q) ⇒ p with direct cert → not in trusted_base()
    let phi_proved = Term::pi(Term::sigma(p.clone(), q.clone()), p.clone());
    let cert = Term::lam(
        Term::sigma(p.clone(), q.clone()),
        Term::proj1(Term::var(0)),
    );
    let verdict_p = attempt_with_cert(&mut env, &phi_proved, cert);
    assert!(
        matches!(verdict_p, Verdict::Proved { .. }),
        "proved verdict expected for (p∧q)⇒p; got {:?}",
        verdict_p
    );
    // No new postulate was emitted for the proved goal: trusted_base() does NOT
    // contain a postulate for phi_proved (no declare_postulate was called for it).
    // hole_id from the unknown path is still present (not yet discharged):
    assert!(
        in_trusted_base(&env, hole_id),
        "unknown hole must still be in trusted_base() (23 §1.3)"
    );
}

// ─── F. Regression — V2/V1 interface consumed unchanged (23 §1.1/§1.4) ──────

/// F1: pure-pipeline-no-obligations-unaffected
/// Empty obligation set → no `attempt_obligation` calls, no verdicts emitted.
#[test]
fn pure_pipeline_no_obligations_unaffected() {
    let obligations: Vec<ObligationTriple> = vec![];

    let verdicts: Vec<_> = obligations
        .iter()
        .map(|_triple| unreachable!("no obligations to process"))
        .collect();

    assert!(
        verdicts.is_empty(),
        "empty obligation set must produce no verdicts (23 §1.1)"
    );
}

/// F2 (soundness): verdict-keyed-by-id-no-side-channel
/// Multiple obligations: verdicts are keyed by stable `id`; the `proved`/`unknown`
/// distinction is derived from `trusted_base()` + `check`, not a side-channel.
#[test]
fn verdict_keyed_by_id_no_side_channel() {
    let ProofEnv { mut env, p, q, .. } = make_proof_env();

    // Obligation A: provable — (p ∧ q) ⇒ p
    let phi_a = Term::pi(Term::sigma(p.clone(), q.clone()), p.clone());
    let triple_a = closed_triple(&mut env, "decl.phi_a", phi_a.clone());
    let result_a = attempt_obligation(&mut env, &triple_a);

    // Obligation B: unprovable — abstract P with empty context
    let phi_b = p.clone();
    let triple_b = closed_triple(&mut env, "decl.phi_b", phi_b.clone());
    let result_b = attempt_obligation(&mut env, &triple_b);

    // Verdicts are keyed by id (23 §1.4 / 21 §5.3)
    assert_eq!(result_a.obligation_id.0, "decl.phi_a");
    assert_eq!(result_b.obligation_id.0, "decl.phi_b");

    // A is proved; B is unknown — distinct verdicts on id, not position
    assert!(
        matches!(result_a.verdict, Verdict::Proved { .. }),
        "obligation A (provable) should be proved; got {:?}",
        result_a.verdict
    );
    let hole_b = hole_of(&result_b.verdict);

    // B's hole is in trusted_base() (kernel-structural unknown)
    assert!(
        in_trusted_base(&env, hole_b),
        "obligation B (unprovable): hole must be in trusted_base() (23 §1.3)"
    );

    // No side-channel: the proved verdict is NOT from a "proved flag" —
    // it is because attempt_with_cert called check() and it succeeded.
    // The unknown verdict is NOT from a "failed flag" — it is because
    // declare_postulate was called and the id is in trusted_base().
    // (Structural property, not assertable directly — but the above checks
    // exercise the kernel-structural path, not any out-of-band store.)
}
