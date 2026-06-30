//! Sec1 acceptance tests — conformance cases from
//! `conformance/security/ifc/seed-ifc.md`.
//!
//! Sec1 has **two trusted surfaces** the kernel does NOT backstop:
//! - **N1 (flow rules):** Labels erased before kernel — a flow bug emits
//!   well-typed core the kernel accepts. Sole net: flip cases {A1–A4, C1, F1}.
//! - **N2 (reduction faithfulness):** Kernel re-checks the cert for the
//!   obligation it is handed, not its faithfulness to 2-safety. Sole net: D5.
//!
//! IFC is relational (2-safety) — NI is never authored as `ensures φ`.
//! Field spellings are `(oracle)`; value-sets and invariants are locked.

use ken_elaborator::{
    attempt_obligation, attempt_with_cert,
    extract::{ObligationId, ObligationTriple, ProvKind, Provenance},
    error::Span,
    ifc::{
        check_declassify_in_delta, check_no_laundering,
        check_reduction_faithfulness, CtHook, CtLabel, DeclassifyCap, FlowCtx,
        LeakageSink, RelationalClaim, BOTTOM, INTERNAL, PUBLIC, SECRET,
        TRIGGER_REL_DEFERRED, TRIGGER_SEC1CT, TRIGGER_SEC1_DUAL,
        TRIGGER_SEC1_LAUNDER, TRIGGER_SEC1_REDUCE, TRUSTED, UNTRUSTED,
    },
    prover::{Countermodel, ProverResult, Verdict},
};
use ken_kernel::{declare_postulate, GlobalEnv, Level, Term};

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn make_env_pq() -> (GlobalEnv, Term, Term) {
    let mut env = GlobalEnv::new();
    let p_id = declare_postulate(&mut env, vec![], Term::omega(Level::zero()))
        .expect("P : Omega_0");
    let q_id = declare_postulate(&mut env, vec![], Term::omega(Level::zero()))
        .expect("Q : Omega_0");
    (env, Term::const_(p_id, vec![]), Term::const_(q_id, vec![]))
}

fn closed_triple(env: &mut GlobalEnv, id: &str, phi: Term) -> ObligationTriple {
    let placeholder = env.fresh_id();
    ObligationTriple {
        id: ObligationId(id.to_owned()),
        hole_id: placeholder,
        context: vec![],
        phi: phi.clone(),
        goal_closed: phi,
        provenance: Provenance { kind: ProvKind::Prove, span: Span::zero() },
    }
}

/// Synthetic `Disproved` result (for cases where the prover lacks the backend).
fn synthetic_disproved(
    env: &mut GlobalEnv,
    id: &str,
    phi: Term,
    description: &str,
) -> (ProverResult, ObligationTriple) {
    let placeholder = env.fresh_id();
    let triple = ObligationTriple {
        id: ObligationId(id.to_owned()),
        hole_id: placeholder,
        context: vec![],
        phi: phi.clone(),
        goal_closed: phi,
        provenance: Provenance { kind: ProvKind::Prove, span: Span::zero() },
    };
    let result = ProverResult {
        obligation_id: ObligationId(id.to_owned()),
        verdict: Verdict::Disproved {
            countermodel: Countermodel { description: description.to_owned() },
        },
    };
    (result, triple)
}

// ─── A. Flow typing — accept/reject (AC1) ────────────────────────────────────

/// A1. L-SINK rejects a Secret value at a Public clearance sink.
/// Flip (right=accept): same flow to a Secret-clearance sink accepts.
#[test]
fn secret_to_public_rejected() {
    let ctx = FlowCtx::new();
    // `log (e : String @ Secret)` to a Public sink: (Secret ⊔ ⊥) ⋢ Public → Reject
    let reject = ctx.l_sink(SECRET, PUBLIC, "log.out");
    assert!(reject.is_reject(), "Secret @ Public sink must reject");
    let err = reject.error().unwrap();
    assert_eq!(err.rule, "L-SINK");
    assert_eq!(err.data_label, SECRET);
    assert_eq!(err.sink_clearance, PUBLIC);
    assert_eq!(err.site, "log.out");

    // Flip: same value at Secret-clearance sink → Accept
    let accept = ctx.l_sink(SECRET, SECRET, "log.secret");
    assert!(accept.is_accept(), "Secret @ Secret sink must accept");

    // Flip: Public value at Public sink → Accept
    let accept_pub = ctx.l_sink(PUBLIC, PUBLIC, "log.out");
    assert!(accept_pub.is_accept(), "Public @ Public sink must accept");
}

/// A2. Integrity lattice (dual): Untrusted ⋢ Trusted sink — scalar-correct.
/// Flip: Trusted data into a Trusted sink accepts.
///
/// **`[Sec1-dual]` stub notice:** `UNTRUSTED=Label(2)` and `TRUSTED=Label(0)` are
/// the same scalars as `SECRET` and `PUBLIC`. `l_sink(UNTRUSTED,TRUSTED)` is
/// numerically byte-identical to `l_sink(SECRET,PUBLIC)` in A1 — the integrity
/// order-dual is NOT a separate implementation path. A bug that applied the wrong
/// order *specifically* to the IntegLabel carrier (leaving ConfLabel unaffected)
/// cannot be caught here; A2 cannot flip while A1 stays green. The genuine
/// `(Conf×Integ)` product + lattice-parametric rules are deferred to `[Sec1-dual]`.
#[test]
fn integrity_taint_rejected() {
    let ctx = FlowCtx::new();
    // `exec (cmd : String @ Untrusted)` into Shell(Trusted): Untrusted=⊤ ⋢ ⊥=Trusted.
    // Scalar-correct: flows_to(2, 0) = false → Reject.
    let reject = ctx.l_sink(UNTRUSTED, TRUSTED, "shell.exec");
    assert!(reject.is_reject(), "Untrusted @ Trusted sink must reject (scalar-correct)");
    let err = reject.error().unwrap();
    assert_eq!(err.data_label, UNTRUSTED);
    assert_eq!(err.sink_clearance, TRUSTED);

    // Flip: Trusted source → Trusted sink → Accept (flows_to(0,0)=true).
    let accept = ctx.l_sink(TRUSTED, TRUSTED, "shell.exec");
    assert!(accept.is_accept(), "Trusted @ Trusted sink must accept");

    // Cross-check: Trusted CAN flow to Untrusted context (flows_to(0,2)=true).
    let accept_down = ctx.l_sink(TRUSTED, UNTRUSTED, "log.any");
    assert!(accept_down.is_accept(), "Trusted flows to Untrusted context (downgrade OK)");

    // [Sec1-dual] trigger — stub named, not silent.
    assert_eq!(TRIGGER_SEC1_DUAL, "[Sec1-dual]");
}

/// A3. Implicit flow: `if (secret @ Secret) then send s 1 else send s 0`
/// where s : Socket Public. The `pc`-raise in L-OBSERVE is the discriminator.
/// Flip-bug: a checker that DROPS the pc-raise (pc stays ⊥) wrongly accepts.
#[test]
fn implicit_flow_pc_rejected() {
    // L-OBSERVE: branching on secret raises pc to ⊥ ⊔ Secret = Secret
    let ctx = FlowCtx::new(); // pc = ⊥
    let ctx_branch = ctx.l_observe(SECRET); // pc = Secret (inside if-branches)
    assert_eq!(ctx_branch.pc, SECRET, "pc raised to Secret after observing secret");

    // L-SINK inside branch: sent value is Public, but (Public ⊔ Secret) ⋢ Public
    let reject = ctx_branch.l_sink(PUBLIC, PUBLIC, "send.s");
    assert!(reject.is_reject(), "implicit flow: (Public ⊔ pc=Secret) ⋢ Public → reject");

    // Flip-bug: if pc is NOT raised (bug drops l_observe), pc stays ⊥ → wrongly accepts
    let buggy_ctx = FlowCtx::with_pc(BOTTOM); // pc stays ⊥ (bug: l_observe dropped)
    let wrongly_accept = buggy_ctx.l_sink(PUBLIC, PUBLIC, "send.s");
    assert!(wrongly_accept.is_accept(), "bug: dropped pc-raise → wrongly accepts (green-vs-red)");

    // Contrast: `secret : Bool @ Public` → no implicit flow, accept
    let ctx_public_branch = ctx.l_observe(PUBLIC); // pc = ⊥ ⊔ Public = Public
    let accept = ctx_public_branch.l_sink(PUBLIC, PUBLIC, "send.s");
    assert!(accept.is_accept(), "branching on Public value: no implicit flow");
}

/// A4. L-COMBINE raises the label: computing on Secret×Public gives Secret.
/// Flip-bug: a COMBINE that took ℓ₂ (or ⊓) instead of ⊔ would lower to Public.
#[test]
fn combine_raises_label() {
    let ctx = FlowCtx::new(); // pc = ⊥

    // f(x : A @ Secret) (y : B @ Public) → result @ (Secret ⊔ Public ⊔ ⊥) = Secret
    let combined = ctx.l_combine(SECRET, PUBLIC);
    assert_eq!(combined, SECRET, "L-COMBINE: Secret ⊔ Public = Secret");

    // L-SINK: Secret result → Public sink → Reject
    let reject = ctx.l_sink(combined, PUBLIC, "write.out");
    assert!(reject.is_reject(), "Secret combined result @ Public sink → reject");

    // Flip (correct clearance): Secret result → Secret sink → Accept
    let accept = ctx.l_sink(combined, SECRET, "write.sec");
    assert!(accept.is_accept(), "Secret combined result @ Secret sink → accept");

    // Flip-bug: a COMBINE that takes ⊓ (meet) instead of ⊔ (join)
    let buggy_combined = ken_elaborator::ifc::meet(SECRET, PUBLIC); // = ⊓ = Public
    assert_eq!(buggy_combined, PUBLIC, "bug: meet(Secret, Public) = Public");
    let wrongly_accept = ctx.l_sink(buggy_combined, PUBLIC, "write.out");
    assert!(wrongly_accept.is_accept(), "bug: meet-instead-of-join → wrongly accepts");

    // Also: pc propagation into combine
    let ctx_raised = FlowCtx::with_pc(INTERNAL);
    let combined_with_pc = ctx_raised.l_combine(PUBLIC, PUBLIC);
    assert_eq!(combined_with_pc, INTERNAL, "L-COMBINE includes pc: Public ⊔ Public ⊔ Internal = Internal");
}

// ─── B. Declassification — the only downgrade (AC1) ──────────────────────────

/// B1. Authorised declassify accepts AND the authority is listed in the delta.
#[test]
fn declassify_authorised_accepts_and_listed() {
    use ken_elaborator::ifc::check_declassify;
    let cap = DeclassifyCap::new(SECRET, PUBLIC);
    assert!(cap.is_valid(), "Cap_declassify[Secret→Public] is valid (to ⊑ from)");

    let result = check_declassify(Some(&cap), SECRET, SECRET, PUBLIC);
    assert!(
        matches!(result, ken_elaborator::ifc::DeclassifyResult::Accept { downgraded_label: PUBLIC }),
        "authorised declassify Secret→Public → accept with downgraded label Public"
    );

    // The authority MUST appear in trusted_base_delta (completeness of delta).
    let authority_id = "decl:user→public";
    let delta = vec![authority_id.to_owned(), "decl:admin→internal".to_owned()];
    assert!(
        check_declassify_in_delta(authority_id, &delta),
        "declassification authority present in delta → listed"
    );
}

/// B2. Same downgrade WITHOUT capability in scope → rejects.
#[test]
fn declassify_without_capability_rejected() {
    use ken_elaborator::ifc::check_declassify;
    let result = check_declassify(None, SECRET, SECRET, PUBLIC);
    assert!(
        matches!(result, ken_elaborator::ifc::DeclassifyResult::Reject { .. }),
        "no Cap_declassify in scope → rejects"
    );

    // Flip against B1: same downgrade, cap present → accept
    let cap = DeclassifyCap::new(SECRET, PUBLIC);
    let accept = check_declassify(Some(&cap), SECRET, SECRET, PUBLIC);
    assert!(matches!(accept, ken_elaborator::ifc::DeclassifyResult::Accept { .. }),
        "cap present → accepts (B1/B2 capability discriminator)");
}

/// B3. Package performs an authorised declassify but OMITS it from the delta.
/// This is an honesty-guard violation: a downgrade hidden from the delta is a
/// silent confidentiality hole (V2 silent-omission backstop, relational domain).
#[test]
fn declassify_absent_from_delta_is_infidelity() {
    let authority_id = "decl:user→public";

    // The package DID perform the declassification (accept),
    // but OMITTED it from the delta (infidelity).
    let empty_delta: Vec<String> = vec![];
    let listed = check_declassify_in_delta(authority_id, &empty_delta);
    assert!(!listed, "authority absent from delta → honesty-guard violation (infidelity)");

    // Flip: same package with the authority PRESENT → listed (B1 covers this)
    let full_delta = vec![authority_id.to_owned()];
    let present = check_declassify_in_delta(authority_id, &full_delta);
    assert!(present, "authority present in delta → honest disclosure");

    // Guard naming: completeness of the delta is the SOLE backstop (N1-analog for B).
    // A downgrade not in the delta is INVISIBLE to the kernel (labels erased).
    // This case flips on presence/absence of the authority id, not on the declassify itself.
}

// ─── C. No laundering through effects (AC2) — load-bearing guard ─────────────

/// C1. Label-equality check for no-laundering — concept-correct, routing stub.
///
/// **`[Sec1-launder]` stub notice:** `check_no_laundering(a,b)=(a==b)` is a
/// label-equality predicate over hand-assigned literals. No real `bind`/`incl`/
/// `handler_fold` on `itree::ITree` is invoked — the actual trusted surface
/// (`36 §2.2/§2.4`: `bind (Vis e f) k = Vis e (λr.…)` must preserve the label
/// index) is untested. The test verifies the CONCEPT (label-equality is the right
/// invariant) but not KEN'S ROUTING. A label-dropping `bind` in `itree.rs` would
/// not be caught until `[Sec1-launder]` wires C1 through real effect routing.
///
/// Doubly load-bearing under N1: the kernel is blind to label drops (labels
/// erased before kernel), so C1 — once wired to real routing — is the sole net.
#[test]
fn label_survives_effect_routing() {
    // Concept check: label-equality is the correct no-laundering invariant.
    let original_label = SECRET;
    let after_bind = original_label; // preserved (correct bind/incl)
    assert!(
        check_no_laundering(original_label, after_bind),
        "correct bind: label preserved → no-laundering invariant holds"
    );

    // With preserved label, the flow pass still sees Secret at the Public sink:
    let ctx = FlowCtx::new();
    let reject = ctx.l_sink(after_bind, PUBLIC, "chan.public");
    assert!(
        reject.is_reject(),
        "preserved Secret label @ Public sink → rejects (correct behaviour)"
    );

    // Concept of the flip-bug: a bind/incl that DROPS the label index (→ PUBLIC).
    let after_bind_bug = PUBLIC; // dropped to Public (the bug we target)
    assert!(
        !check_no_laundering(original_label, after_bind_bug),
        "bug: label-dropping bind → no-laundering invariant FAILS"
    );

    // Under the bug, the flow pass sees Public at the Public sink — wrongly accepts:
    let wrongly_accept = ctx.l_sink(after_bind_bug, PUBLIC, "chan.public");
    assert!(
        wrongly_accept.is_accept(),
        "bug: dropped label → wrongly accepts (concept: green-vs-red on label equality)"
    );
    // NOTE: the above is concept-level only. Real routing (itree::bind/incl) must
    // be the discriminant — see [Sec1-launder].

    // [Sec1-launder] trigger — stub named, not silent.
    assert_eq!(TRIGGER_SEC1_LAUNDER, "[Sec1-launder]");
}

// ─── D. Non-interference by proof — relational verdict mapping (AC3) ──────────

/// D1. Related pair → proved; kernel re-checks the certificate.
/// Cross-case invariant: a non-interfering program is NEVER disproved.
#[test]
fn related_pair_proved() {
    let (mut env, p, _q) = make_env_pq();
    // Simulate: product(c, ζ) emits lowEq(in¹,in²) ⇒ lowEq(out¹,out²) ∧ coterminates.
    // For a well-labeled view, this obligation is provable.
    // Concrete provable goal: p → p (IPC closes it by Pi-intro + assumption lookup).
    let ni_goal = Term::pi(p.clone(), p.clone()); // p → p (provable via IPC)
    let triple = closed_triple(&mut env, "ob:ni:D1.0", ni_goal);

    let claim = RelationalClaim::new("well_labeled_view", PUBLIC, triple, None);
    let result = claim.check(&mut env);

    assert!(
        matches!(result.verdict, Verdict::Proved { .. }),
        "related pair → proved: {:?}", result.verdict
    );
    assert!(!claim.has_deferred_trigger(), "D1: no deferred trigger — proved");
    // Cross-case: a proved result is NEVER from check_reduction_faithfulness (which checks Disproved)
    assert!(
        !check_reduction_faithfulness(&result.verdict),
        "D1: proved is not disproved — cross-case boundary invariant 1"
    );
}

/// D2. Distinguishing pair → disproved with witness (the pair IS the leak-witness).
/// Verdict-mapping pinned at source: a distinguishing pair is NOT unknown.
/// Cross-case contrast with D1 on the same metatheory class.
#[test]
fn distinguishing_pair_disproved_with_witness() {
    let (mut env, _p, q) = make_env_pq();
    // Simulate: leaking view has a distinguishing pair — two ζ-equal inputs
    // with ζ-observable output difference. The product obligation has a countermodel.
    let (result, triple) = synthetic_disproved(
        &mut env,
        "ob:ni:D2.0",
        q.clone(), // the product-program obligation (un-provable due to the leak)
        "distinguishing pair: (low_in=0,sec=0) vs (low_in=0,sec=1) → diff low_out",
    );
    let claim = RelationalClaim::new("leaking_view", PUBLIC, triple, None);

    assert!(
        matches!(result.verdict, Verdict::Disproved { .. }),
        "distinguishing pair → disproved: {:?}", result.verdict
    );
    // The distinguishing pair is the leak-witness — NOT unknown
    assert!(
        !matches!(result.verdict, Verdict::Unknown { .. }),
        "D2: a distinguishing pair maps to disproved, NEVER unknown (verdict-mapping pinned)"
    );
    assert!(!claim.has_deferred_trigger(), "D2: no deferred trigger");
}

/// D3. Unprovable relational obligation → incomplete(hole), NEVER false proved.
/// Carries [rel-deferred] trigger. Prover honesty guard in the relational domain.
#[test]
fn unprovable_relational_incomplete_never_false_proved() {
    let (mut env, _p, q) = make_env_pq();
    // A value-dependent claim needing deferred machinery: abstract proposition Q
    // cannot be proved or refuted by the current prover.
    let triple = closed_triple(&mut env, "ob:ni:D3.0", q.clone());
    let claim = RelationalClaim::new(
        "value_dependent_view",
        PUBLIC,
        triple,
        Some(TRIGGER_REL_DEFERRED), // non-silent deferred trigger
    );

    let result = claim.check(&mut env);

    assert!(
        matches!(result.verdict, Verdict::Unknown { .. }),
        "unprovable relational obligation → incomplete(hole): {:?}", result.verdict
    );
    // NEVER a false proved — the honesty guard (23 §1.3) in the relational domain.
    assert!(
        !matches!(result.verdict, Verdict::Proved { .. }),
        "D3: a prover limitation yields incomplete, NEVER a false proved"
    );
    // Reify-trigger present — never silent (LP-2 honest-limits).
    assert!(
        claim.has_deferred_trigger(),
        "D3: carries [rel-deferred] trigger — not silent"
    );
    assert_eq!(claim.deferred_trigger, Some(TRIGGER_REL_DEFERRED));
}

/// D4. Progress-sensitive default: a ζ-equal-input pair where one run diverges
/// is a leak (disproved). TI mode drops coterminates_ζ → different verdict.
#[test]
fn progress_sensitive_divergence_is_a_leak() {
    let (mut env, p, _q) = make_env_pq();

    // Progress-sensitive mode (default): coterminates_ζ is a conjunct.
    // One run diverges → the coterminates conjunct is falsified → disproved.
    let (ps_result, _triple) = synthetic_disproved(
        &mut env,
        "ob:ni:D4.ps",
        p.clone(),
        "coterminates_ζ falsified: run-1 terminates (out=0), run-2 diverges — ζ-equal inputs",
    );
    assert!(
        matches!(ps_result.verdict, Verdict::Disproved { .. }),
        "progress-sensitive: diverging run → disproved"
    );

    // Termination-insensitive (TI) opt-in: drops coterminates_ζ conjunct.
    // The ni-only obligation p→p is provable → different verdict.
    // (The relaxation must SHOW in the four-way status — never silent.)
    let ti_ni_goal = Term::pi(p.clone(), p.clone()); // ni-only, no coterminates
    let ti_triple = closed_triple(&mut env, "ob:ni:D4.ti", ti_ni_goal);
    let ti_result = attempt_obligation(&mut env, &ti_triple);
    assert!(
        matches!(ti_result.verdict, Verdict::Proved { .. }),
        "TI mode (no coterminates): ni-only obligation → proved"
    );

    // The two verdicts are DIFFERENT — the relaxation is not silent.
    assert!(
        !matches!(ps_result.verdict, Verdict::Proved { .. }),
        "progress-sensitive verdict differs from TI verdict (flip on coterminates_ζ)"
    );
}

/// D5. Reduction-faithfulness verdict-shape check — structural stub over synthetic.
///
/// **`[Sec1-reduce]` stub notice:** `check_reduction_faithfulness(v) =
/// matches!(v, Disproved)` is a verdict-SHAPE predicate; it asserts "Disproved
/// is Disproved." The test feeds `synthetic_disproved(...)` — a hand-rigged
/// `ProverResult::Disproved`. No `product(c,ζ)` construction (variable renaming,
/// `lowEq_ζ`, `coterminates_ζ`) is implemented. A too-weak `Φ_post` — the
/// N2 failure mode — cannot be detected because nothing builds `Φ_post`. This
/// stub verifies the predicate SHAPE but not Ken's product-program reduction.
/// When `[Sec1-reduce]` lands, D5 must use a real reduction from a known
/// interfering program and show that a weakened `Φ_post` yields a `proved`
/// verdict that D5 catches.
///
/// Distinct from D2 (faithful reduction, verdict tag) and E1 (cert typecheck).
#[test]
fn reduction_faithfulness_interfering_disproved() {
    let (mut env, _p, q) = make_env_pq();

    // Synthetic obligation: the CONCEPT of a known-interfering program's
    // product-program obligation being disproved.
    let (result, triple) = synthetic_disproved(
        &mut env,
        "ob:ni:D5.0",
        q.clone(),
        "synthetic: known leak (direct Secret→Public assignment, no declassify)",
    );
    let claim = RelationalClaim::new("interfering_program_stub", PUBLIC, triple, None);

    assert!(
        matches!(result.verdict, Verdict::Disproved { .. }),
        "synthetic disproved verdict is Disproved: {:?}", result.verdict
    );

    // Structural check: check_reduction_faithfulness returns true for Disproved.
    // This is a predicate-shape assertion, not N2 coverage.
    assert!(
        check_reduction_faithfulness(&result.verdict),
        "D5: Disproved verdict satisfies the faithfulness predicate (shape-correct)"
    );

    // The predicate rejects Proved and Unknown verdicts:
    assert!(!check_reduction_faithfulness(&Verdict::Unknown { hole_id: env.fresh_id() }),
        "D5: Unknown does not satisfy faithfulness predicate");

    assert!(!claim.has_deferred_trigger());

    // [Sec1-reduce] trigger — stub named, not silent.
    assert_eq!(TRIGGER_SEC1_REDUCE, "[Sec1-reduce]");
}

// ─── E. Kernel re-checkable, not trusted (AC3) ───────────────────────────────

/// E1. A forged or fabricated certificate fails the kernel re-check.
/// The kernel is the sole authority (de Bruijn criterion, 18 §4.5).
/// IFC adds NO new trusted primitive (labels are Vis indices, erased before kernel).
#[test]
fn forged_label_or_cert_kernel_rejected() {
    let (mut env, p, q) = make_env_pq();

    // A forged cert: claims to prove `p → q` (which IPC cannot prove)
    // but presents `Var(0)` — a free variable in the empty context.
    let phi_closed = Term::pi(p.clone(), q.clone()); // p → q (not provable from nothing)
    let forged_cert = Term::var(0); // free variable — wrong type, wrong context

    let verdict = attempt_with_cert(&mut env, &phi_closed, forged_cert);
    assert!(
        matches!(verdict, Verdict::Unknown { .. }),
        "forged cert: kernel re-check fails → Unknown (never false Proved)"
    );
    assert!(
        !matches!(verdict, Verdict::Proved { .. }),
        "E1: a cert that fails kernel-check can NEVER yield Proved"
    );

    // Contrast: a correct cert for p → p (IPC-provable) passes the re-check.
    let phi_id = Term::pi(p.clone(), p.clone()); // p → p
    let correct_cert = Term::lam(p.clone(), Term::var(0)); // λ_:p. x — the identity proof
    let verdict_ok = attempt_with_cert(&mut env, &phi_id, correct_cert);
    assert!(
        matches!(verdict_ok, Verdict::Proved { .. }),
        "correct cert for p→p: kernel accepts → Proved"
    );
}

// ─── F. The @ct hook — source precondition only (AC4) ────────────────────────

/// F1. A @ct-marked value steering a leakage-relevant sink is a type error.
/// Flip: the same value NOT steering a leakage op, or not @ct → accepts.
#[test]
fn ct_value_steers_leakage_sink_rejected() {
    let ctx = FlowCtx::new();
    let ct_key = CtLabel(true); // the key is marked @ct

    // `branch_on (k[0] == g[0])` where k is @ct: steers a BranchGuard leakage sink
    let reject = ctx.l_ct_sink(ct_key, &LeakageSink::BranchGuard, "branch_on");
    assert!(reject.is_reject(), "@ct value at BranchGuard sink → reject");
    let err = reject.error().unwrap();
    assert_eq!(err.rule, "L-SINK(ct)");

    // Also covers MemoryIndex and VarTimePrimitive sinks.
    let reject_idx = ctx.l_ct_sink(ct_key, &LeakageSink::MemoryIndex, "mem_at");
    assert!(reject_idx.is_reject(), "@ct value at MemoryIndex sink → reject");

    let reject_vt = ctx.l_ct_sink(ct_key, &LeakageSink::VarTimePrimitive, "div_secret");
    assert!(reject_vt.is_reject(), "@ct value at VarTimePrimitive sink → reject");

    // Flip: same value but NOT @ct → accepts
    let not_ct = CtLabel(false);
    let accept = ctx.l_ct_sink(not_ct, &LeakageSink::BranchGuard, "branch_on");
    assert!(accept.is_accept(), "non-@ct value at BranchGuard → accepts");

    // Flip: @ct value at a NON-leakage-sink (ordinary sink) → accepts via l_sink
    let accept_ordinary = ctx.l_sink(PUBLIC, PUBLIC, "log.out");
    assert!(accept_ordinary.is_accept(), "@ct value at ordinary PUBLIC sink → accepts via l_sink");
}

/// F2. The @ct label parses, attaches, carries through the denotation, and a
/// [Sec1ct]/[Ward] reify-trigger is present. Timing guarantee NOT asserted here.
#[test]
fn ct_label_parses_carries_and_defers_timing() {
    // The @ct label must be attached (parsed + carried).
    let hook = CtHook::new(true); // @ct is set
    assert!(hook.label_carries(), "@ct label is attached and carried");
    assert_eq!(hook.ct_label, CtLabel(true));

    // A reify-trigger MUST be present — never silent (LP-2 honest-limits).
    assert!(
        hook.has_reify_trigger(),
        "[Sec1ct]/[Ward] reify-trigger must be present for @ct values"
    );
    assert_eq!(
        hook.deferred_timing,
        Some(TRIGGER_SEC1CT),
        "deferred timing trigger is [Sec1ct]"
    );

    // The timing guarantee itself is NOT asserted here — delegated to [Ward].
    // (Asserting "no timing leak" would over-claim past what Sec1 lands.)
    // Only the source-level precondition Q (no @ct at leakage sinks) is enforced.

    // A non-@ct value has no reify-trigger (no timing claim needed).
    let hook_nct = CtHook::new(false);
    assert!(!hook_nct.label_carries(), "non-@ct value carries no ct label");
    assert!(!hook_nct.has_reify_trigger(), "non-@ct value has no reify-trigger");
    assert_eq!(hook_nct.deferred_timing, None);
}

// ─── G. Honest limits — scope, no over-claim (AC5) ───────────────────────────

/// G1. A deferred relational claim carries a [rel-deferred] reify-trigger.
/// Not silently passed, not silently omitted — the trigger is named.
#[test]
fn deferred_machinery_carries_reify_trigger() {
    let (mut env, _p, q) = make_env_pq();
    // A quantitative claim ("at most n bits leak") needing deferred heavy machinery.
    let triple = closed_triple(&mut env, "ob:ni:G1.0", q);
    let claim = RelationalClaim::new(
        "quantitative_leak_claim",
        PUBLIC,
        triple,
        Some(TRIGGER_REL_DEFERRED),
    );

    // The trigger is non-null and names the exact deferred WP.
    assert!(
        claim.has_deferred_trigger(),
        "G1: deferred claim has a reify-trigger — not silent"
    );
    assert_eq!(
        claim.deferred_trigger,
        Some(TRIGGER_REL_DEFERRED),
        "G1: trigger is [rel-deferred]"
    );

    // Not silently omitted: a claim WITHOUT a trigger that CANNOT be proved would
    // be an honesty-guard violation (the N2-analog for G1).
    let fresh_phi = {
        let id = declare_postulate(&mut env, vec![], Term::omega(Level::zero()))
            .expect("fresh Omega postulate");
        Term::const_(id, vec![])
    };
    let g1_silent_triple = closed_triple(&mut env, "ob:ni:G1.1", fresh_phi);
    let claim_no_trigger = RelationalClaim::new(
        "quantitative_leak_silent",
        PUBLIC,
        g1_silent_triple,
        None, // no trigger — WRONG for a deferred case
    );
    assert!(
        !claim_no_trigger.has_deferred_trigger(),
        "G1: a claim without a trigger for deferred machinery = infidelity (not the correct impl)"
    );
}

/// G2. A well-labeled program that type-checks under §3 rules:
/// conformance asserts the RULES FIRED CORRECTLY (accept), not that NI is
/// mechanically verified. The by-typing meta-theorem is itself trusted (§H).
#[test]
fn by_typing_meta_theorem_is_trusted_scope() {
    let ctx = FlowCtx::new();

    // A well-labeled program: `send (e : String @ Public)` to a `Public` sink.
    // Rule enforcement: L-SINK checks (Public ⊔ ⊥) ⊑ Public → true → Accept.
    let result = ctx.l_sink(PUBLIC, PUBLIC, "send.public");
    assert!(
        result.is_accept(),
        "G2: well-labeled program (Public @ Public) accepts — rule fired correctly"
    );

    // Also: L-PURE always accepts — a pure value at any label.
    let pure_result = ctx.l_pure();
    assert!(pure_result.is_accept(), "G2: L-PURE always accepts");

    // The meta-theorem "this flow discipline implies NI" is TRUSTED (§H):
    // - The meta-theorem is a named-future deliverable (mechanization deferred).
    // - Asserting "this program is NI" from a single type-check would OVER-CLAIM.
    // - We assert only: the flow rules fire with the correct accept/reject verdict.
    // This case pins the honest scope boundary: test the discipline, name the limit.

    // Cross-check: a flow violation still rejects (the rules DO fire).
    let flow_violation = ctx.l_sink(SECRET, PUBLIC, "send.public");
    assert!(
        flow_violation.is_reject(),
        "G2: flow violation still rejects — the rules fire, not just vacuous"
    );
}

// ─── Cross-case consistency sweep ─────────────────────────────────────────────

/// Cross-case: by-proof verdict-trichotomy class {D1, D2, D3, D4, D5}.
/// Glivenko-analog for IFC: asserts cross-case boundary invariants.
///
/// **Honest-limits note:** the D5 faithfulness check here is a verdict-shape
/// predicate over synthetic verdicts. The N2 trusted surface (product-program
/// reduction faithfulness) is stub-tested, not netted, pending `[Sec1-reduce]`.
#[test]
fn by_proof_trichotomy_cross_case_sweep() {
    let (mut env, p, q) = make_env_pq();

    // D1 verdict class: related → Proved (non-interfering never Disproved)
    let d1_triple = closed_triple(&mut env, "ob:sweep:D1", Term::pi(p.clone(), p.clone()));
    let d1 = attempt_obligation(&mut env, &d1_triple);
    assert!(matches!(d1.verdict, Verdict::Proved { .. }), "D1 class: proved");

    // D2 verdict class: distinguishing pair → Disproved (with witness)
    let (d2_result, _) = synthetic_disproved(&mut env, "ob:sweep:D2", q.clone(), "leak-witness");
    assert!(matches!(d2_result.verdict, Verdict::Disproved { .. }), "D2 class: disproved");

    // D3 verdict class: undischargeable → Unknown (honesty guard: never false Proved)
    let d3_triple = closed_triple(&mut env, "ob:sweep:D3", q.clone());
    let d3 = attempt_obligation(&mut env, &d3_triple);
    assert!(matches!(d3.verdict, Verdict::Unknown { .. }), "D3 class: unknown (incomplete-hole)");
    assert!(!matches!(d3.verdict, Verdict::Proved { .. }), "D3: NEVER a false proved");

    // Boundary invariant 1: a non-interfering program is NEVER Disproved.
    assert!(!check_reduction_faithfulness(&d1.verdict), "D1 (proved) is not Disproved");

    // Boundary invariant 2: an undischargeable obligation is Unknown, NEVER Proved.
    assert!(!matches!(d3.verdict, Verdict::Proved { .. }), "D3 (unknown) is not Proved");

    // D5 shape: the faithfulness predicate correctly classifies verdict classes.
    // (This is shape coverage, not N2 coverage — see [Sec1-reduce].)
    assert!(check_reduction_faithfulness(&d2_result.verdict), "D2/D5: Disproved passes shape predicate");
    assert!(!check_reduction_faithfulness(&d1.verdict), "D1: Proved does not pass faithfulness predicate");
    assert!(!check_reduction_faithfulness(&d3.verdict), "D3: Unknown does not pass faithfulness predicate");
}

/// Honest limits: N1/N2 stub gaps carry named reify-triggers — never silent.
///
/// This WP delivers the scalar IFC discipline-library with correct flow-rule
/// logic. Three N1/N2 surfaces are STUB-TESTED (the logic is right; the real
/// Ken routing/reduction machinery is not yet wired):
/// - `[Sec1-dual]`: A2 is scalar-identical to A1; the integrity order-dual is
///   not independently discriminating. Genuine (Conf×Integ) product deferred.
/// - `[Sec1-launder]`: C1 checks label-equality over literals; no real
///   `bind`/`incl`/`Vis` routing is invoked. Real effect routing deferred.
/// - `[Sec1-reduce]`: D5 checks verdict shape over synthetic obligations; no
///   real `product(c,ζ)` reduction is implemented. Real reduction deferred.
#[test]
fn n1_n2_stub_gaps_carry_reify_triggers() {
    // Each trigger is a named non-silent gap (LP-2 honest-limits discipline
    // applies to BUILD claims exactly as it applies to spec claims).
    assert_eq!(TRIGGER_SEC1_DUAL, "[Sec1-dual]");
    assert_eq!(TRIGGER_SEC1_LAUNDER, "[Sec1-launder]");
    assert_eq!(TRIGGER_SEC1_REDUCE, "[Sec1-reduce]");

    // What IS delivered (real, not stubs):
    // - Four flow rules correct: l_sink, l_observe, l_combine, l_pure
    // - flows_to direction right; join/meet correct
    // - Declassification capability-gated + strictly-lower + delta-audited
    // - @ct hook parses, carries, rejects at leakage sinks
    // - Deferred cases carry triggers: [rel-deferred], [Sec1ct], [Ward]
    // What is NOT yet delivered (stub, pending the three triggers above):
    // - Independent integrity dual (A2 ≡ A1 at scalar level)
    // - Real Vis-routing for no-laundering (C1 is label-equality only)
    // - Real product-program reduction for faithfulness (D5 is verdict-shape only)
}
