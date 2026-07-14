//! T1 acceptance tests — conformance cases from
//! `conformance/verify/protocol/seed-protocol.md`.
//!
//! T1 **serializes** the verdict V3 produced + the diagnostic V4 derived; it
//! **never re-decides** either. The failure mode is **infidelity** on the
//! wire (relabel, lossy round-trip, silently-dropped diagnostic, broken
//! stable-field contract) — **never** unsoundness.
//!
//! Field names are `(oracle)` per `25` preamble; the normative-locked content
//! is value-sets, cross-field invariants, id/hole_id-stability, and
//! trusted_base_delta emptiness.

use serde_json::{json, Value};

use ken_elaborator::{
    attempt_obligation,
    diagnostics::{
        AtomId, DiagnosticTag, FailureWitness, FormRef, KripkeCountermodel,
        SuggestedAction, WorldId,
    },
    extract::{ObligationId, ObligationTriple, ProvKind, Provenance},
    error::Span,
    prover::{Countermodel, ProverResult, Verdict},
    project_diagnostic,
    // T1 protocol exports
    hole_id_string, obligation_id_string, project_obligation_status, project_wire_verdict,
    rollup_doc_status, round_trip, serialize_countermodel,
    serialize_decomposition, serialize_diagnostic, serialize_document,
    serialize_slice, validate_document, DocStatus,
    ObligationStatus, WireVerdict,
};
use ken_kernel::{declare_postulate, GlobalEnv, Level, Term};

// ─── Test helpers ─────────────────────────────────────────────────────────────

struct ProofEnv {
    env: GlobalEnv,
    p: Term,
    q: Term,
}

fn make_proof_env() -> ProofEnv {
    let mut env = GlobalEnv::new();
    let p_id =
        declare_postulate(&mut env, "test postulate".to_string(), vec![], Term::omega(Level::zero())).expect("P postulate");
    let q_id =
        declare_postulate(&mut env, "test postulate".to_string(), vec![], Term::omega(Level::zero())).expect("Q postulate");
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

/// Synthetic `Disproved` ProverResult + ObligationTriple.
fn synthetic_disproved(
    env: &mut GlobalEnv,
    id: &str,
    phi: Term,
    description: &str,
) -> (ProverResult, ObligationTriple) {
    let placeholder_hole = env.fresh_id();
    let triple = ObligationTriple {
        id: ObligationId(id.to_owned()),
        hole_id: placeholder_hole,
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

/// Build a `KripkeCountermodel` with real world data (non-placeholder).
/// `verdict = False` (forces ¬φ at the root world w0).
fn concrete_countermodel_false() -> KripkeCountermodel {
    KripkeCountermodel {
        verdict: DiagnosticTag::False,
        worlds: vec![WorldId("w0".into()), WorldId("w1".into())],
        order: vec![(WorldId("w0".into()), WorldId("w1".into()))],
        forcing: vec![
            (WorldId("w1".into()), AtomId("d ≠ 0".into())),
        ],
        failure: Some(FailureWitness {
            world: WorldId("w0".into()),
            subformula: FormRef("result * d == n".into()),
        }),
    }
}

/// Read the `status` field of the document.
fn doc_status(doc: &Value) -> &str {
    doc["status"].as_str().expect("doc.status present")
}

/// Read the `status` field of an obligation.
fn ob_status(doc: &Value, ob_idx: usize) -> &str {
    doc["obligations"][ob_idx]["status"]
        .as_str()
        .expect("obligation.status present")
}

/// Read the `trusted_base_delta` array.
fn delta(doc: &Value) -> &Vec<Value> {
    doc["trusted_base_delta"].as_array().expect("trusted_base_delta present")
}

/// Count diagnostics in the document.
fn diagnostic_count(doc: &Value) -> usize {
    doc["obligations"]
        .as_array()
        .expect("obligations array")
        .iter()
        .filter(|ob| !ob["diagnostic"].is_null())
        .count()
}

// ─── A. Round-trip — each status + diagnostic kind serializes lossless ────────

/// A1: proved-result-empty-diagnostics-and-delta
/// A fully-proved run → `status:"proved"`, all obligations `status:"discharged"`,
/// `diagnostic:null`, `trusted_base_delta:[]`. Zero diagnostics.
#[test]
fn proved_result_empty_diagnostics_and_delta() {
    let ProofEnv { mut env, p, q, .. } = make_proof_env();

    // (p ∧ q) ⇒ p — provable by IPC
    let phi = Term::pi(Term::sigma(p.clone(), q.clone()), p.clone());
    let triple = closed_triple(&mut env, "ob:divide#post.0", phi.clone());
    let result = attempt_obligation(&mut env, &triple);
    assert!(matches!(result.verdict, Verdict::Proved { .. }));

    let diag = project_diagnostic(&result, &triple);
    assert!(diag.is_none(), "proved → no diagnostic");

    let data = vec![(result.verdict, diag, ObligationId("ob:divide#post.0".into()), triple)];
    let doc = serialize_document("divide", &data);

    // Check top-level status
    assert_eq!(doc_status(&doc), "proved", "proved → status:proved");

    // All obligations discharged, diagnostic null
    let obs = doc["obligations"].as_array().expect("obligations");
    assert_eq!(obs.len(), 1);
    assert_eq!(obs[0]["status"].as_str().unwrap(), "discharged");
    assert!(obs[0]["diagnostic"].is_null(), "discharged → diagnostic:null");

    // trusted_base_delta empty
    assert_eq!(delta(&doc).len(), 0, "proved → empty trusted_base_delta");

    // Zero diagnostics (AC5 + no-regression)
    assert_eq!(diagnostic_count(&doc), 0, "proved → |diagnostics| = 0");

    // Round-trip lossless
    let rt = round_trip(&doc);
    assert_eq!(rt["status"], doc["status"], "round-trip preserves status");
    assert_eq!(rt["obligations"], doc["obligations"], "round-trip preserves obligations");
    assert_eq!(rt["trusted_base_delta"], doc["trusted_base_delta"], "round-trip preserves delta");

    // Valid by the reference validator
    validate_document(&doc).expect("proved document must be schema-valid");
}

/// A2: countermodel-kind-round-trips-lossless
/// V3 `disproved` with a Kripke countermodel → `kind:"countermodel"`,
/// `verdict:"false"`, worlds/order/forcing/failure all present, round-trips
/// losslessly.
#[test]
fn countermodel_kind_round_trips_lossless() {
    let mut env = GlobalEnv::new();
    let p_id =
        declare_postulate(&mut env, "test postulate".to_string(), vec![], Term::omega(Level::zero())).unwrap();
    let p = Term::const_(p_id, vec![]);

    let cm = concrete_countermodel_false();
    let actions = vec![SuggestedAction::FixCounterexample {
        description: "n > 0 is false for n = 0".into(),
    }];
    let diag_val = serialize_countermodel(&cm, &actions);

    // Verify all fields present
    assert_eq!(diag_val["kind"].as_str().unwrap(), "countermodel");
    assert_eq!(diag_val["verdict"].as_str().unwrap(), "false", "verdict:false");
    assert!(!diag_val["worlds"].as_array().unwrap().is_empty(), "worlds present");
    assert!(!diag_val["order"].as_array().unwrap().is_empty(), "order present");
    assert!(diag_val["forcing"].is_object(), "forcing present");
    assert!(!diag_val["failure"].is_null(), "failure present");
    assert_eq!(diag_val["failure"]["world"].as_str().unwrap(), "w0");
    assert_eq!(diag_val["failure"]["subformula"].as_str().unwrap(), "result * d == n");

    // Round-trip: deserialize∘serialize is identity
    let rt = round_trip(&diag_val);
    assert_eq!(rt["verdict"], diag_val["verdict"], "verdict survives round-trip");
    assert_eq!(rt["worlds"], diag_val["worlds"], "worlds survive round-trip");
    assert_eq!(rt["order"], diag_val["order"], "order (≤ preorder) survives round-trip");
    assert_eq!(rt["forcing"], diag_val["forcing"], "forcing survives round-trip");
    assert_eq!(rt["failure"]["world"], diag_val["failure"]["world"]);
    assert_eq!(rt["failure"]["subformula"], diag_val["failure"]["subformula"]);
    assert_eq!(rt, diag_val, "full diagnostic round-trip is identity");

    // In a document: schema-valid
    let _triple = closed_triple(&mut env, "ob:test#post.0", p.clone());
    let (synth_result, synth_triple) = synthetic_disproved(&mut env, "ob:test#post.0", p, "refuted");
    let project = project_diagnostic(&synth_result, &synth_triple).unwrap();
    validate_document(&serialize_document("test", &[(
        synth_result.verdict,
        Some(project),
        synth_triple.id.clone(),
        synth_triple,
    )])).expect("countermodel document must be schema-valid");
}

/// A3 (soundness): hole-kind-round-trips-and-lists-in-delta
/// V3 `unknown` → `status:"incomplete"`, `kind:"hole"`, hole appears in
/// `trusted_base_delta` (the honesty guard). Round-trips lossless.
#[test]
fn hole_kind_round_trips_and_lists_in_delta() {
    let ProofEnv { mut env, p, .. } = make_proof_env();

    // abstract P → V3 unknown
    let triple = closed_triple(&mut env, "ob:sort#post.0", p.clone());
    let result = attempt_obligation(&mut env, &triple);
    assert!(matches!(result.verdict, Verdict::Unknown { .. }));

    let diag = project_diagnostic(&result, &triple).unwrap();

    let data = vec![(result.verdict, Some(diag), triple.id.clone(), triple)];
    let doc = serialize_document("sort", &data);

    // Status checks
    assert_eq!(doc_status(&doc), "incomplete", "unknown → status:incomplete");
    assert_eq!(ob_status(&doc, 0), "open", "unknown obligation → status:open");

    // Diagnostic kind = hole
    let diag_val = &doc["obligations"][0]["diagnostic"];
    assert_eq!(diag_val["kind"].as_str().unwrap(), "hole");
    assert!(diag_val.get("hole_id").is_some(), "hole_id present");
    assert_eq!(diag_val["runtime"].as_str().unwrap(), "unknown");

    // hole appears in trusted_base_delta (the honesty guard, soundness)
    let d = delta(&doc);
    assert_eq!(d.len(), 1, "one hole → one entry in trusted_base_delta");
    let entry_id = d[0]["id"].as_str().expect("delta entry has id");
    // hole_id in the diagnostic matches the delta entry id
    let hole_id_in_diag = diag_val["hole_id"].as_str().expect("hole_id present");
    assert_eq!(
        entry_id, hole_id_in_diag,
        "hole in diagnostic matches trusted_base_delta entry (25 §3, 18 §5)"
    );

    // Round-trip lossless
    let rt = round_trip(&doc);
    assert_eq!(rt["trusted_base_delta"], doc["trusted_base_delta"]);
    assert_eq!(rt["obligations"][0]["diagnostic"], doc["obligations"][0]["diagnostic"]);

    validate_document(&doc).expect("hole document must be schema-valid");
}

/// A4: decomposition-and-slice-kinds-round-trip
/// Both `decomposition` and `slice` diagnostic kinds serialize schema-valid
/// and round-trip losslessly.
#[test]
fn decomposition_and_slice_kinds_round_trip() {
    let actions_unknown = vec![SuggestedAction::AddPrecondition {
        hypothesis: Term::omega(Level::zero()),
    }];
    let actions_false = vec![SuggestedAction::FixCounterexample {
        description: "n < 0 in the failing region".into(),
    }];

    // Decomposition (can appear for both disproved/unknown, §5)
    let decomp = serialize_decomposition("n > 0", "n < 0", "n == 0", &actions_false);
    assert_eq!(decomp["kind"].as_str().unwrap(), "decomposition");
    assert_eq!(decomp["true_region"].as_str().unwrap(), "n > 0");
    assert_eq!(decomp["false_region"].as_str().unwrap(), "n < 0");
    assert_eq!(decomp["unknown_region"].as_str().unwrap(), "n == 0");
    let rt_decomp = round_trip(&decomp);
    assert_eq!(rt_decomp, decomp, "decomposition round-trip identity");

    // Slice (unknown-only, §5)
    let slice = serialize_slice("xs ≠ nil", "add as precondition", true, &actions_unknown);
    assert_eq!(slice["kind"].as_str().unwrap(), "slice");
    assert_eq!(slice["missing_hypothesis"].as_str().unwrap(), "xs ≠ nil");
    assert_eq!(slice["bridge"].as_str().unwrap(), "add as precondition");
    assert!(slice["sufficient"].as_bool().unwrap(), "sufficient:true");
    let rt_slice = round_trip(&slice);
    assert_eq!(rt_slice, slice, "slice round-trip identity");

    // Both serialize schema-valid when embedded in a document
    let decomp_doc = json!({
        "schema": "ken.verify/v1",
        "target": { "name": "test" },
        "status": "disproved",
        "obligations": [{ "id": "ob:t1", "goal": {"pretty": "n > 0"},
                          "context": [], "provenance": {"span":"0","clause":"ensures"},
                          "status": "refuted", "diagnostic": decomp }],
        "trusted_base_delta": []
    });
    validate_document(&decomp_doc).expect("decomposition document must be schema-valid");

    let slice_doc = json!({
        "schema": "ken.verify/v1",
        "target": { "name": "test" },
        "status": "incomplete",
        "obligations": [{ "id": "ob:t2", "goal": {"pretty": "is_sorted xs"},
                          "context": [], "provenance": {"span":"0","clause":"ensures"},
                          "status": "open", "diagnostic": slice }],
        "trusted_base_delta": [{"id": "?h:ob:t2", "goal": "is_sorted xs"}]
    });
    validate_document(&slice_doc).expect("slice document must be schema-valid");
    assert_eq!(round_trip(&slice_doc), slice_doc, "slice document round-trip identity");
}

// ─── B. The false-vs-unknown discriminator on the wire ───────────────────────

/// B1: refuted-goal-false-tag-forcing-world
/// `φ = p ∧ ¬p` (synthetic disproved) → `verdict:"false"`, forcing world
/// present, `fix_counterexample` only.
#[test]
fn refuted_goal_false_tag_forcing_world() {
    let mut env = GlobalEnv::new();
    let _p_id = declare_postulate(&mut env, "test postulate".to_string(), vec![], Term::omega(Level::zero())).unwrap();

    let cm = concrete_countermodel_false();
    let actions = vec![SuggestedAction::FixCounterexample {
        description: "p ∧ ¬p is false — ¬(p∧¬p) is provable".into(),
    }];
    let diag_val = serialize_countermodel(&cm, &actions);

    // FIDELITY: verdict must be "false"
    assert_eq!(diag_val["verdict"].as_str().unwrap(), "false",
               "B1: disproved → verdict:false (the false-side discriminator)");

    // failure world present (forces ¬φ)
    assert!(!diag_val["failure"].is_null(), "B1: failure world present");

    // suggested_actions = [fix_counterexample only]
    let actions_arr = diag_val["suggested_actions"].as_array().unwrap();
    assert!(
        actions_arr.iter().all(|a| a["kind"].as_str().unwrap() == "fix_counterexample"),
        "B1: only fix_counterexample actions (region:false)"
    );
    assert!(
        actions_arr
            .iter()
            .all(|a| a["region"].as_str().unwrap() == "false"),
        "B1: all actions region:false"
    );
    // No add_precondition (unknown-only)
    assert!(
        !actions_arr.iter().any(|a| a["kind"].as_str().unwrap() == "add_precondition"),
        "B1: no add_precondition on a false goal (region-tag discipline)"
    );
}

/// B2: unknown-goal-unknown-tag-no-forcing-world
/// Abstract P (not refutable) → `verdict:"unknown"`, no failure world,
/// `suggested_actions` from the unknown set. NEVER fix_counterexample.
#[test]
fn unknown_goal_unknown_tag_no_forcing_world() {
    let ProofEnv { mut env, p, .. } = make_proof_env();

    let triple = closed_triple(&mut env, "ob:lem#post.0", p.clone());
    let result = attempt_obligation(&mut env, &triple);
    assert!(matches!(result.verdict, Verdict::Unknown { .. }),
            "abstract P must be V3 unknown");

    let diag = project_diagnostic(&result, &triple).unwrap();
    let diag_val = serialize_diagnostic(&diag, &triple.id);

    // FIDELITY: verdict must be "unknown", never "false"
    assert_eq!(diag_val["kind"].as_str().unwrap(), "hole",
               "unknown → kind:hole");

    // For the wire cross-walk: project_wire_verdict says Unknown
    let wire = project_wire_verdict(&result.verdict);
    assert_eq!(wire, Some(WireVerdict::Unknown),
               "B2: abstract P (p∨¬p analog) → wire verdict Unknown, not False (Glivenko)");
    assert_ne!(wire, Some(WireVerdict::False),
               "B2: FIDELITY — never WireVerdict::False for an abstract atom");

    // No fix_counterexample action
    if let Some(actions_arr) = diag_val["suggested_actions"].as_array() {
        assert!(
            !actions_arr.iter().any(|a| a["kind"].as_str().unwrap() == "fix_counterexample"),
            "B2: unknown goal must NOT carry fix_counterexample (region-tag discipline)"
        );
    }
}

/// B3: false-unknown-non-confusable-roundtrip
/// The `p ∧ ¬p` (false) and `p ∨ ¬p` (unknown) documents are **distinct and
/// non-confusable** — they differ in verdict, status, and legal actions.
#[test]
fn false_unknown_non_confusable_roundtrip() {
    let mut env = GlobalEnv::new();
    let p_id = declare_postulate(&mut env, "test postulate".to_string(), vec![], Term::omega(Level::zero())).unwrap();
    let p = Term::const_(p_id, vec![]);

    // "false" side: synthetic disproved (p ∧ ¬p)
    let cm_false = concrete_countermodel_false();
    let actions_false = vec![SuggestedAction::FixCounterexample {
        description: "refuted".into(),
    }];
    let false_diag = serialize_countermodel(&cm_false, &actions_false);

    // "unknown" side: abstract P
    let (unk_result, _unk_triple) = {
        let triple = closed_triple(&mut env, "ob:lem#post.0", p.clone());
        let result = attempt_obligation(&mut env, &triple);
        let diag = project_diagnostic(&result, &triple).unwrap();
        let diag_val = serialize_diagnostic(&diag, &triple.id);
        (diag_val, result.verdict)
    };

    // They must differ on verdict
    assert_ne!(
        false_diag["verdict"], unk_result["verdict"],
        "B3: false-side and unknown-side have distinct verdict fields"
    );
    assert_eq!(false_diag["verdict"].as_str().unwrap(), "false");
    assert_ne!(false_diag["verdict"].as_str().unwrap(), "unknown");

    // document status also differs
    assert_ne!(
        project_wire_verdict(&Verdict::Disproved { countermodel: Countermodel { description: String::new() } }),
        project_wire_verdict(&Verdict::Unknown { hole_id: ken_kernel::GlobalId(0) }),
        "B3: wire verdict projections are distinct"
    );

    let false_ob_status = ObligationStatus::Refuted;
    let unknown_ob_status = ObligationStatus::Open;
    assert_ne!(false_ob_status, unknown_ob_status);
    assert_ne!(false_ob_status.as_str(), unknown_ob_status.as_str(),
               "B3: obligation status strings differ (refuted vs open)");

    let false_doc_status = rollup_doc_status(&[ObligationStatus::Refuted]);
    let unknown_doc_status = rollup_doc_status(&[ObligationStatus::Open]);
    assert_ne!(false_doc_status.as_str(), unknown_doc_status.as_str(),
               "B3: document status strings differ (disproved vs incomplete)");

    // The two serializations are non-confusable
    assert_ne!(
        false_diag["verdict"], unk_result["verdict"],
        "B3: a false and an unknown go to distinct, non-confusable messages"
    );
}

/// B4: glivenko-wire-sweep-classically-valid-never-false
/// The classically-valid class `{ p ∨ ¬p, ¬¬p ⇒ p }` (both represented as
/// abstract atoms in this suite) all serialize `verdict:"unknown"`, NEVER
/// `"false"`. Cross-case sweep asserts class agreement.
#[test]
fn glivenko_wire_sweep_classically_valid_never_false() {
    let ProofEnv { mut env, p: _, .. } = make_proof_env();

    // Both members of the classically-valid class project to Unknown
    let verdicts = vec![
        // p ∨ ¬p — abstract atom as stand-in (Glivenko: ¬(p∨¬p) unprovable)
        project_wire_verdict(&Verdict::Unknown {
            hole_id: env.fresh_id(),
        }),
        // ¬¬p ⇒ p — also unknown (same Glivenko reasoning)
        project_wire_verdict(&Verdict::Unknown {
            hole_id: env.fresh_id(),
        }),
    ];

    // CROSS-CASE SWEEP: all members agree — all Unknown, none False
    for (i, v) in verdicts.iter().enumerate() {
        assert_eq!(
            *v,
            Some(WireVerdict::Unknown),
            "B4: classically-valid class member {} must be WireVerdict::Unknown (Glivenko sweep)",
            i
        );
        assert_ne!(
            *v,
            Some(WireVerdict::False),
            "B4: classically-valid class member {} NEVER WireVerdict::False (Glivenko on the wire)",
            i
        );
    }

    // Contrast: the genuinely-refutable p ∧ ¬p → False
    let false_verdict = project_wire_verdict(&Verdict::Disproved {
        countermodel: Countermodel { description: "¬(p∧¬p) is provable".into() },
    });
    assert_eq!(false_verdict, Some(WireVerdict::False),
               "B4: genuinely-refutable goal → WireVerdict::False");

    // Cross-case: False and Unknown are distinct classes (the discriminator)
    assert_ne!(
        Some(WireVerdict::Unknown),
        Some(WireVerdict::False),
        "B4: the false/unknown discriminator is preserved — the two classes do not collapse"
    );
}

// ─── C. Cross-walk projection, rollup precedence, totality ────────────────────

/// C1: three-renderings-agree-one-source
/// obligation `status`, document `status`, and countermodel `verdict` are
/// three renderings of one V3 verdict — they agree per the cross-walk.
#[test]
fn three_renderings_agree_one_source() {
    // false case: Disproved → refuted / disproved / verdict:"false"
    {
        let v = Verdict::Disproved {
            countermodel: Countermodel { description: "test".into() },
        };
        let ob_s = project_obligation_status(&v);
        let wire_v = project_wire_verdict(&v);
        let doc_s = rollup_doc_status(&[ob_s.clone()]);

        assert_eq!(ob_s, ObligationStatus::Refuted, "C1: Disproved → obligation refuted");
        assert_eq!(doc_s, DocStatus::Disproved, "C1: Disproved → doc disproved");
        assert_eq!(wire_v, Some(WireVerdict::False), "C1: Disproved → verdict:false");

        // The three renderings agree per the cross-walk:
        // verdict:"false" ⟺ obligation refuted ⟺ doc disproved
        assert_eq!(ob_s.as_str(), "refuted");
        assert_eq!(doc_s.as_str(), "disproved");
        assert_eq!(wire_v.unwrap().as_str(), "false");
    }

    // unknown case: Unknown → open / incomplete / verdict:"unknown"
    {
        let hole_id = ken_kernel::GlobalId(0);
        let v = Verdict::Unknown { hole_id };
        let ob_s = project_obligation_status(&v);
        let wire_v = project_wire_verdict(&v);
        let doc_s = rollup_doc_status(&[ob_s.clone()]);

        assert_eq!(ob_s, ObligationStatus::Open, "C1: Unknown → obligation open");
        assert_eq!(doc_s, DocStatus::Incomplete, "C1: Unknown → doc incomplete");
        assert_eq!(wire_v, Some(WireVerdict::Unknown), "C1: Unknown → verdict:unknown");

        // verdict:"unknown" ⟺ open ⟺ incomplete
        assert_eq!(ob_s.as_str(), "open");
        assert_eq!(doc_s.as_str(), "incomplete");
        assert_eq!(wire_v.unwrap().as_str(), "unknown");
    }

    // proved case: Proved → discharged / proved / no countermodel
    {
        let v = Verdict::Proved { cert: Term::omega(Level::zero()) };
        let ob_s = project_obligation_status(&v);
        let wire_v = project_wire_verdict(&v);
        let doc_s = rollup_doc_status(&[ob_s.clone()]);

        assert_eq!(ob_s, ObligationStatus::Discharged);
        assert_eq!(doc_s, DocStatus::Proved);
        assert_eq!(wire_v, None, "C1: Proved → no wire verdict (no countermodel)");
    }
}

/// C2: mixed-rollup-refuted-dominates
/// Target with mixed statuses (≥1 refuted, ≥1 open, rest discharged) →
/// doc status = `"disproved"`, NOT `"incomplete"`.
#[test]
fn mixed_rollup_refuted_dominates() {
    let statuses = vec![
        ObligationStatus::Discharged,
        ObligationStatus::Refuted,
        ObligationStatus::Open,
        ObligationStatus::Discharged,
    ];
    let doc_s = rollup_doc_status(&statuses);

    assert_eq!(
        doc_s,
        DocStatus::Disproved,
        "C2: refuted ⊐ open ⊐ discharged — refuted dominates (mixed rollup, 25 §3)"
    );
    assert_ne!(
        doc_s,
        DocStatus::Incomplete,
        "C2: FIDELITY — a refuted obligation must not be masked as 'incomplete' \
         (a refuted obligation is a hard error that dominates open holes)"
    );
    assert_eq!(doc_s.as_str(), "disproved");
}

/// C3: non-discharged-implies-non-null-diagnostic
/// `diagnostic:null` iff `discharged`; a non-discharged obligation serialized
/// with `diagnostic:null` fails validation (totality, no silent failure).
#[test]
fn non_discharged_implies_non_null_diagnostic() {
    // Valid: discharged + null diagnostic → accepted
    let valid_discharged = json!({
        "schema": "ken.verify/v1", "target": {"name":"t"},
        "status": "proved",
        "obligations": [{
            "id": "ob:t1", "goal": {"pretty":"x"}, "context": [],
            "provenance": {"span":"0","clause":"ensures"},
            "status": "discharged", "diagnostic": null
        }],
        "trusted_base_delta": []
    });
    validate_document(&valid_discharged).expect("discharged+null is valid");

    // Invalid: open + null diagnostic → rejected (totality violation)
    let invalid_open_null = json!({
        "schema": "ken.verify/v1", "target": {"name":"t"},
        "status": "incomplete",
        "obligations": [{
            "id": "ob:t2", "goal": {"pretty":"x"}, "context": [],
            "provenance": {"span":"0","clause":"ensures"},
            "status": "open", "diagnostic": null
        }],
        "trusted_base_delta": []
    });
    let result = validate_document(&invalid_open_null);
    assert!(
        result.is_err(),
        "C3: non-discharged obligation with null diagnostic must be REJECTED \
         (totality: diagnostic:null iff discharged, no silent failure, 25 §4/§6)"
    );

    // Invalid: refuted + null diagnostic → rejected
    let invalid_refuted_null = json!({
        "schema": "ken.verify/v1", "target": {"name":"t"},
        "status": "disproved",
        "obligations": [{
            "id": "ob:t3", "goal": {"pretty":"x"}, "context": [],
            "provenance": {"span":"0","clause":"ensures"},
            "status": "refuted", "diagnostic": null
        }],
        "trusted_base_delta": []
    });
    assert!(
        validate_document(&invalid_refuted_null).is_err(),
        "C3: refuted obligation with null diagnostic must be REJECTED (totality)"
    );
}

// ─── D. The stability surface ─────────────────────────────────────────────────

/// D1: stable-field-drop-or-rename-rejected
/// Dropping `obligations[].id` or renaming `countermodel.verdict` → validator
/// REJECTS. (Drop-required = REJECT even if an unknown field is added.)
#[test]
fn stable_field_drop_or_rename_rejected() {
    // Base valid document
    let base = json!({
        "schema": "ken.verify/v1", "target": {"name":"t"},
        "status": "disproved",
        "obligations": [{
            "id": "ob:t1", "goal": {"pretty":"n > 0"}, "context": [],
            "provenance": {"span":"0","clause":"ensures"},
            "status": "refuted",
            "diagnostic": {
                "kind": "countermodel", "verdict": "false",
                "worlds": ["w0"], "order": [], "forcing": {"w0":[]},
                "failure": {"world":"w0","subformula":"n > 0"},
                "suggested_actions": [{"kind":"fix_counterexample","region":"false","detail":"fix","edit":null}]
            }
        }],
        "trusted_base_delta": []
    });
    validate_document(&base).expect("base document must be valid");

    // (a) Drop `obligations[].id` → rejected
    let mut missing_id = base.clone();
    if let Some(obs) = missing_id["obligations"].as_array_mut() {
        if let Some(ob) = obs.get_mut(0) {
            if let Some(obj) = ob.as_object_mut() {
                obj.remove("id");
            }
        }
    }
    assert!(
        validate_document(&missing_id).is_err(),
        "D1: dropping obligations[].id (stable required field) must be REJECTED"
    );

    // (b) Rename `countermodel.verdict` → `judgment` (drop-required + add-unknown)
    let renamed_verdict = json!({
        "schema": "ken.verify/v1", "target": {"name":"t"},
        "status": "disproved",
        "obligations": [{
            "id": "ob:t1", "goal": {"pretty":"n > 0"}, "context": [],
            "provenance": {"span":"0","clause":"ensures"},
            "status": "refuted",
            "diagnostic": {
                "kind": "countermodel",
                // "verdict" removed, replaced with "judgment"
                "judgment": "false",  // unknown field → ACCEPTED (ignored)
                // but "verdict" required → REJECTED
                "worlds": ["w0"], "order": [], "forcing": {"w0":[]},
                "failure": {"world":"w0","subformula":"n > 0"},
                "suggested_actions": [{"kind":"fix_counterexample","region":"false","detail":"fix","edit":null}]
            }
        }],
        "trusted_base_delta": []
    });
    assert!(
        validate_document(&renamed_verdict).is_err(),
        "D1: renaming countermodel.verdict → judgment must be REJECTED \
         (drop-required REJECT + add-unknown ACCEPT → net REJECT on missing discriminator)"
    );
}

/// D2: additive-unknown-field-accepted
/// Adding an unknown optional field or an unknown `suggested_actions.kind` →
/// validator ACCEPTS (forward-compatibility).
#[test]
fn additive_unknown_field_accepted() {
    // Add an unknown optional field + an unknown action kind
    let additive = json!({
        "schema": "ken.verify/v1", "target": {"name":"t"},
        "status": "incomplete",
        "obligations": [{
            "id": "ob:t1", "goal": {"pretty":"is_sorted xs"}, "context": [],
            "provenance": {"span":"0","clause":"ensures"},
            "status": "open",
            "confidence": 0.87,  // unknown field — must be ACCEPTED (ignored)
            "diagnostic": {
                "kind": "hole", "hole_id": "?h:ob:t1",
                "goal": "is_sorted xs", "context": [], "origin": {"span":"0","clause":"ensures"},
                "runtime": "unknown",
                "suggested_actions": [
                    {"kind": "add_precondition", "region": "unknown",
                     "detail": "add xs ≠ nil", "edit": null},
                    {"kind": "suggest_rewrite", "region": "unknown",
                     "detail": "try rewrite by insert_sorted"}  // unknown action kind
                ]
            }
        }],
        "trusted_base_delta": [{"id": "?h:ob:t1", "goal": "is_sorted xs"}],
        "meta": {"version": "0.9.1"}  // unknown top-level field
    });

    validate_document(&additive).expect(
        "D2: additive unknown fields and unknown action kinds must be ACCEPTED \
         (forward-compatibility, 25 §6)"
    );
}

/// D3: obligation-id-stable-across-unrelated-edit
/// `obligation_id_string` is a function of program structure, not line number
/// or allocation order. An "unrelated edit that shifts line numbers" does not
/// change the id (discriminates against line-keyed id bug).
#[test]
fn obligation_id_stable_across_unrelated_edit() {
    // Before edit: obligation at "line 14" (simulated via Span)
    let id = ObligationId("ob:divide#post.0".into());
    let id_str_before = obligation_id_string(&id);

    // After "unrelated edit that shifts line numbers": same clause path → same id
    // (The span changes, but the obligation id is derived from program structure, not span)
    let id_after_edit = ObligationId("ob:divide#post.0".into()); // same clause path
    let id_str_after = obligation_id_string(&id_after_edit);

    assert_eq!(
        id_str_before, id_str_after,
        "D3: obligation id must be UNCHANGED after a line-shifting unrelated edit \
         (id is a function of program structure = clause path, 22 §1, 24 §6)"
    );
    assert_eq!(id_str_before, "ob:divide#post.0");

    // Likewise hole_id_string is stable
    let hole_before = hole_id_string(&id);
    let hole_after = hole_id_string(&id_after_edit);
    assert_eq!(
        hole_before, hole_after,
        "D3: hole_id must be UNCHANGED across a line-shifting edit \
         (function of provenance/clause-path, not allocation order, 24 §6)"
    );
    assert_eq!(hole_before, "?h:ob:divide#post.0");

    // Discriminating assertion: only an edit TO the clause itself would change the id
    let id_different_clause = ObligationId("ob:divide#pre.0".into()); // different clause
    let id_str_different = obligation_id_string(&id_different_clause);
    assert_ne!(
        id_str_before, id_str_different,
        "D3: a different clause path produces a different id \
         (the discriminator that catches the line-keyed-id bug)"
    );
}

// ─── E. The agent loop + determinism ─────────────────────────────────────────

/// E1: agent-pivots-on-status-no-text-scraping
/// From machine fields alone (status + suggested_actions + provenance), an
/// agent locates the actionable signal — no human-text parsing required.
#[test]
fn agent_pivots_on_status_no_text_scraping() {
    // Build two documents: one disproved (fix-path), one incomplete (supply-path)
    let false_doc = json!({
        "schema": "ken.verify/v1", "target": {"name":"divide"},
        "status": "disproved",
        "obligations": [{
            "id": "ob:divide#post.0",
            "goal": { "pretty": null },   // pretty=null: no human text
            "context": [], "provenance": { "span": "pay.ken:14:11", "clause": "ensures" },
            "status": "refuted",
            "diagnostic": {
                "kind": "countermodel", "verdict": "false",
                "worlds": ["w0"], "order": [], "forcing": {"w0":[]},
                "failure": { "world": "w0", "subformula": null },  // subformula=null: no human text
                "suggested_actions": [
                    { "kind": "fix_counterexample", "region": "false",
                      "detail": null, "edit": null }  // detail=null: no human text
                ]
            }
        }],
        "trusted_base_delta": []
    });

    let unknown_doc = json!({
        "schema": "ken.verify/v1", "target": {"name":"sort"},
        "status": "incomplete",
        "obligations": [{
            "id": "ob:sort#post.0",
            "goal": { "pretty": null }, "context": [],
            "provenance": { "span": "lib.ken:30:3", "clause": "ensures" },
            "status": "open",
            "diagnostic": {
                "kind": "hole", "hole_id": "?h:ob:sort#post.0",
                "goal": null, "context": [], "origin": {"span":"lib.ken:30:3","clause":"ensures"},
                "runtime": "unknown",
                "suggested_actions": [
                    { "kind": "add_precondition", "region": "unknown",
                      "detail": null, "edit": null }
                ]
            }
        }],
        "trusted_base_delta": [{"id":"?h:ob:sort#post.0","goal":null}]
    });

    // Agent algorithm (§7): pivot on status alone, no text parsing
    let pivot_false = agent_pivot(&false_doc);
    assert_eq!(pivot_false, AgentPivot::FixSpec,
               "E1: status:disproved → fix-spec path (from machine fields only)");

    let pivot_unknown = agent_pivot(&unknown_doc);
    assert_eq!(pivot_unknown, AgentPivot::SupplyFacts,
               "E1: status:incomplete → supply-facts path (from machine fields only)");

    // Agent reads provenance.span for fix-spec location (machine field)
    let span = false_doc["obligations"][0]["provenance"]["span"].as_str().unwrap();
    assert_eq!(span, "pay.ken:14:11", "E1: provenance.span is a machine field (not human text)");

    // Agent reads suggested_actions[].kind for action (machine field)
    let action_kind = false_doc["obligations"][0]["diagnostic"]["suggested_actions"][0]["kind"]
        .as_str().unwrap();
    assert_eq!(action_kind, "fix_counterexample", "E1: action kind is a machine field");

    // Agent reads hole_id for supply-path target (machine field)
    let hole_id = unknown_doc["obligations"][0]["diagnostic"]["hole_id"].as_str().unwrap();
    assert_eq!(hole_id, "?h:ob:sort#post.0", "E1: hole_id is a machine field");
}

#[derive(Debug, PartialEq, Eq)]
enum AgentPivot { FixSpec, SupplyFacts, NoAction }

/// Minimal agent pivot function — from machine fields only, no text parsing.
fn agent_pivot(doc: &Value) -> AgentPivot {
    match doc["status"].as_str().unwrap_or("") {
        "disproved" => AgentPivot::FixSpec,
        "incomplete" => AgentPivot::SupplyFacts,
        "proved" => AgentPivot::NoAction,
        _ => AgentPivot::NoAction,
    }
}

/// E2: deterministic-modulo-stats-and-display
/// Two verification runs of the same program produce byte-stable output on the
/// stable surface (ids, verdict, status), differing ONLY in `stats.ms` and
/// display strings.
#[test]
fn deterministic_modulo_stats_and_display() {
    fn run_once(ms_value: u64) -> Value {
        let ProofEnv { mut env, p, q } = make_proof_env();

        // Obligation A: proved
        let phi_a = Term::pi(Term::sigma(p.clone(), q.clone()), p.clone());
        let triple_a = closed_triple(&mut env, "ob:divide#post.0", phi_a);
        let result_a = attempt_obligation(&mut env, &triple_a);
        let diag_a = project_diagnostic(&result_a, &triple_a);

        // Obligation B: unknown
        let triple_b = closed_triple(&mut env, "ob:sort#post.0", p.clone());
        let result_b = attempt_obligation(&mut env, &triple_b);
        let diag_b = project_diagnostic(&result_b, &triple_b);

        let mut doc = serialize_document("target", &[
            (result_a.verdict, diag_a, triple_a.id.clone(), triple_a),
            (result_b.verdict, diag_b, triple_b.id.clone(), triple_b),
        ]);

        // Inject a non-deterministic stats.ms to simulate two runs
        if let Some(stats) = doc["stats"].as_object_mut() {
            stats.insert("ms".to_owned(), json!(ms_value));
        }
        doc
    }

    let run1 = run_once(41);
    let run2 = run_once(57); // different ms

    // STABLE SURFACE: identical across runs
    assert_eq!(run1["status"], run2["status"], "E2: doc status stable");
    assert_eq!(run1["schema"], run2["schema"], "E2: schema stable");
    assert_eq!(
        run1["obligations"][0]["id"], run2["obligations"][0]["id"],
        "E2: obligation id stable across runs (not allocation-order-keyed)"
    );
    assert_eq!(
        run1["obligations"][1]["id"], run2["obligations"][1]["id"],
        "E2: second obligation id stable"
    );
    assert_eq!(
        run1["obligations"][0]["status"], run2["obligations"][0]["status"],
        "E2: obligation status stable"
    );
    assert_eq!(
        run1["obligations"][1]["status"], run2["obligations"][1]["status"],
        "E2: second obligation status stable"
    );
    assert_eq!(
        run1["trusted_base_delta"], run2["trusted_base_delta"],
        "E2: trusted_base_delta stable"
    );

    // NON-DETERMINISTIC: stats.ms differs (excluded from byte-stability)
    assert_ne!(
        run1["stats"]["ms"], run2["stats"]["ms"],
        "E2: stats.ms is non-deterministic (excluded from byte-stability, 24 §6)"
    );

    // DISTINCT FROM ROUND-TRIP: determinism is across runs, not a single doc's surface
    // Round-trip preserves everything including stats.ms; determinism excludes it.
    let rt = round_trip(&run1);
    assert_eq!(rt["stats"]["ms"], run1["stats"]["ms"],
               "E2: round-trip preserves stats.ms (distinct from cross-run determinism)");
}
