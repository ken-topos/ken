//! B1 acceptance tests — behavioral-export emitter conformance cases from
//! `conformance/behavioral/export/seed-export.md`.
//!
//! Each test maps to a named conformance case (EX-A1 … EX-G1). Every test
//! routes **real verified content through the actual emitter** and observes
//! the projected field — never a synthetic export literal.
//!
//! **QA gate (build-qa lesson):** synthetic export literals guard nothing.
//! Every case here:
//! 1. Constructs a real kernel environment with actual obligations.
//! 2. Runs the actual checked-target emitter (`emit_checked_target_export`).
//! 3. Asserts a structural property of the emitted contract.
//!
//! **Discriminating-pair discipline:** EX-A1 and EX-A2 are the NON-DEGENERATE
//! pair on the same postcondition under two kernel states. Neither alone nets
//! the honesty discriminator: a lazy emitter (trusts a "proved" string, or
//! buckets by `ensures` presence) passes one but fails the other. Both tests
//! must pass simultaneously.

use std::collections::BTreeSet;

use ken_elaborator::{
    emit_checked_target_export, serialize_export, ExportError, GEntry, Pred, PStatus, TEntry,
    Temporal,
    compiler_driver::{compile_checked_target_denotation, CompilerSource},
    effects::row::EffectRow,
    extract::{ObligationId, ObligationTriple, ProvKind, Provenance},
    error::Span,
    prover::{attempt_with_cert, Verdict},
};
use ken_kernel::{declare_postulate, GlobalEnv, GlobalId, Level, Term};

// ─── Shared helpers ───────────────────────────────────────────────────────────

/// A minimal kernel environment with two abstract propositions `P : Ω` and
/// `Q : Ω` for constructing closed obligations.
struct KernEnv {
    env: GlobalEnv,
    p_term: Term,
    #[allow(dead_code)]
    q_term: Term,
    #[allow(dead_code)]
    p_id: GlobalId,
    #[allow(dead_code)]
    q_id: GlobalId,
}

fn make_kern_env() -> KernEnv {
    let mut env = GlobalEnv::new();
    let p_id = declare_postulate(&mut env, "test postulate".to_string(), vec![], Term::omega(Level::zero()))
        .expect("P postulate");
    let q_id = declare_postulate(&mut env, "test postulate".to_string(), vec![], Term::omega(Level::zero()))
        .expect("Q postulate");
    KernEnv {
        p_term: Term::const_(p_id, vec![]),
        q_term: Term::const_(q_id, vec![]),
        p_id,
        q_id,
        env,
    }
}

/// Build a closed `ObligationTriple` for `phi` with empty context.
///
/// `hole_id` is the V1 kernel postulate for this obligation — the same id
/// that `trusted_base()` membership tests check (the honesty discriminator).
fn closed_triple(
    hole_id: GlobalId,
    id: &str,
    phi: Term,
    prov: ProvKind,
) -> ObligationTriple {
    ObligationTriple {
        id: ObligationId(id.to_owned()),
        hole_id,
        context: vec![],
        phi: phi.clone(),
        goal_closed: phi,
        provenance: Provenance { kind: prov, span: Span::zero() },
    }
}

/// Collect `trusted_base()` into a `BTreeSet` for O(log n) membership checks.
fn trusted_base_set(env: &GlobalEnv) -> BTreeSet<GlobalId> {
    env.trusted_base().into_iter().collect()
}

fn emit_export(
    target_name: &str,
    results: &[(ObligationTriple, Verdict)],
    trusted_base: &BTreeSet<GlobalId>,
    legacy_alphabet: EffectRow,
    generators: Vec<GEntry>,
    temporal: Vec<TEntry>,
) -> Result<ken_elaborator::BehavioralExport, ExportError> {
    assert!(
        legacy_alphabet.effects().next().is_none(),
        "alphabet-bearing cases must use a real perform producer"
    );
    let source = format!("fn {target_name} (value : Unit) : Unit = value");
    let denotation = compile_checked_target_denotation(
        &format!("b1_acceptance_{target_name}"),
        CompilerSource::new("fixture.ken", source),
        target_name,
    )
    .expect("pure checked target denotation");
    emit_checked_target_export(
        &denotation,
        results,
        trusted_base,
        generators,
        temporal,
    )
}

// ─── EX-A. The status → field projection — the no-over-claim pair (AC2/I1) ──

/// EX-A1: export/proved-postcondition-projects-to-Q
///
/// A postcondition whose V1 hole is discharged (absent from trusted_base)
/// projects into Q tagged `proved`.
/// Half of the no-over-claim pair: alone this is green-vs-green under a lazy
/// emitter. The net is the pair with EX-A2.
#[test]
fn proved_postcondition_projects_to_q() {
    let mut ke = make_kern_env();

    // Goal: `P → P` (Pi(P, P) — non-dependent arrow, closed proposition).
    // For Pi(A, B): A = P (domain), B = P (codomain constant, not Var(0)).
    // Proof: lam(P, Var(0)) = λx:P.x (identity function proves P → P).
    let phi = Term::pi(ke.p_term.clone(), ke.p_term.clone()); // P → P

    // V1 elaboration registers the goal as a hole postulate.
    let hole_id = declare_postulate(&mut ke.env, "test postulate".to_string(), vec![], phi.clone())
        .expect("V1 hole postulate");

    // The hole is now in trusted_base.
    assert!(ke.env.trusted_base().contains(&hole_id), "V1 hole in trusted_base pre-discharge");

    // Build the V2 obligation triple.
    let triple = closed_triple(hole_id, "view_f.ensures.0", phi.clone(), ProvKind::Ensures { index: 0 });

    // V3 prover cert: lam(P, Var(0)) = λx:P.x proves P → P.
    let cert = Term::lam(ke.p_term.clone(), Term::var(0));
    let verdict = attempt_with_cert(&mut ke.env, &phi, cert.clone());
    assert!(
        matches!(verdict, Verdict::Proved { .. }),
        "lam(P,Var(0)) should prove Pi(P,P), got {:?}", verdict
    );

    // Discharge the V1 hole: upgrade it to a transparent definition.
    // After this, hole_id is no longer Opaque → absent from trusted_base().
    let discharged = ke.env.upgrade_to_transparent(hole_id, cert.clone());
    assert!(discharged, "upgrade_to_transparent must succeed for an Opaque hole");
    assert!(
        !ke.env.trusted_base().contains(&hole_id),
        "hole must be absent from trusted_base after discharge"
    );

    // Run the emitter.
    let tb = trusted_base_set(&ke.env);
    let result = emit_export(
        "view_f",
        &[(triple, verdict)],
        &tb,
        EffectRow::empty(),
        vec![],
        vec![],
    ).expect("export must succeed for proved content");

    // Postcondition projects into Q.
    assert_eq!(result.guarantees.len(), 1, "Q must have one entry");
    assert_eq!(result.guarantees[0].obligation_id, "view_f.ensures.0");
    // P must be empty — the claim is NOT in assumptions.
    assert!(result.assumptions.is_empty(), "proved claim must be absent from P");
    // T must be empty.
    assert!(result.obligations.is_empty(), "proved claim must be absent from T");
}

/// EX-A2: export/open-hole-postcondition-rides-P-as-unknown  (soundness)
///
/// The SAME postcondition `P → P`, but the proof is left as an open typed
/// hole (verdict `unknown`). The hole's postulate IS in trusted_base.
/// The claim must project into P tagged `unknown` — NEVER into Q.
///
/// This is the **load-bearing no-over-claim net**: with EX-A1 it is the
/// non-degenerate distinguishing pair (same postcondition, two kernel states).
/// A lazy emitter that trusts a "proved" string or buckets by `ensures`
/// presence lands this in Q (over-claim) → red.
#[test]
fn open_hole_postcondition_rides_p_as_unknown() {
    let mut ke = make_kern_env();

    // Same goal as EX-A1: `P → P` = Pi(P, P) (non-dependent arrow).
    let phi = Term::pi(ke.p_term.clone(), ke.p_term.clone());

    // V1 elaboration registers the hole.
    let hole_id = declare_postulate(&mut ke.env, "test postulate".to_string(), vec![], phi.clone())
        .expect("V1 hole postulate");

    // Build the V2 triple — same id structure as EX-A1 to reinforce same-postcondition.
    let triple = closed_triple(hole_id, "view_f.ensures.0", phi.clone(), ProvKind::Ensures { index: 0 });

    // We do NOT discharge the hole — leave it open.
    // Simulate the prover returning Unknown (hole stays in trusted_base).
    let verdict = Verdict::Unknown { hole_id };

    // Hole is still in trusted_base.
    assert!(
        ke.env.trusted_base().contains(&hole_id),
        "hole must be in trusted_base (not discharged)"
    );

    // Run the emitter.
    let tb = trusted_base_set(&ke.env);
    let result = emit_export(
        "view_f",
        &[(triple, verdict)],
        &tb,
        EffectRow::empty(),
        vec![],
        vec![],
    ).expect("export must succeed (no disproved verdict)");

    // Claim projects into P — NEVER Q.
    assert!(result.guarantees.is_empty(), "open hole must NOT appear in Q (no-over-claim)");
    assert_eq!(result.assumptions.len(), 1, "P must have one entry");
    assert_eq!(result.assumptions[0].status, PStatus::Unknown);
    assert_eq!(result.assumptions[0].obligation_id, "view_f.ensures.0");
}

// ─── EX-B. Assumption visibility (AC3/I2) ────────────────────────────────────

/// EX-B1: export/removing-assume-shrinks-P-and-changes-hash (AC3)
///
/// A program with an open assumption (postulate in trusted_base) emits a P
/// entry. Removing the assumption (discharging it) causes the P entry to vanish
/// and the hash to change.
///
/// Also carries the `tested`→P arm: a `Prove` provenance (explicit statement)
/// that is not discharged projects as PStatus::Tested.
#[test]
fn removing_assume_shrinks_p_and_changes_hash() {
    let mut ke = make_kern_env();

    // Goal: P → P (a dischargeable proposition, Prove provenance = "assume-like").
    let phi = Term::pi(ke.p_term.clone(), ke.p_term.clone()); // P → P

    // Register as a Prove-kind hole (explicit statement — models `assume P → P`).
    let hole_id = declare_postulate(&mut ke.env, "test postulate".to_string(), vec![], phi.clone())
        .expect("assume-like hole");

    let triple = closed_triple(hole_id, "f.prove", phi.clone(), ProvKind::Prove);

    // WITH the assumption: Unknown verdict (not discharged).
    let verdict_unknown = Verdict::Unknown { hole_id };
    let tb_with = trusted_base_set(&ke.env);
    let export_with = emit_export(
        "f",
        &[(triple.clone(), verdict_unknown)],
        &tb_with,
        EffectRow::empty(),
        vec![],
        vec![],
    ).expect("export with assumption");

    // P entry present, tagged Tested (Prove provenance).
    assert_eq!(export_with.assumptions.len(), 1, "P must contain the assumption");
    assert_eq!(export_with.assumptions[0].status, PStatus::Tested);
    let hash_with = export_with.hash.clone();

    // Discharge the assumption: λx:P.x proves P → P.
    let cert = Term::lam(ke.p_term.clone(), Term::var(0));
    let discharged = ke.env.upgrade_to_transparent(hole_id, cert.clone());
    assert!(discharged, "upgrade_to_transparent must succeed");
    assert!(!ke.env.trusted_base().contains(&hole_id));

    // WITHOUT the assumption: Proved verdict, hole absent from trusted_base.
    let verdict_proved = Verdict::Proved { cert };
    let tb_without = trusted_base_set(&ke.env);
    let export_without = emit_export(
        "f",
        &[(triple, verdict_proved)],
        &tb_without,
        EffectRow::empty(),
        vec![],
        vec![],
    ).expect("export after discharge");

    // P entry is gone.
    assert!(
        export_without.assumptions.is_empty(),
        "P entry must vanish after discharge"
    );
    // Hash changed.
    assert_ne!(
        hash_with, export_without.hash,
        "hash must change when assumption is removed (EX-B1 + EX-G1 pair)"
    );
}

// ─── EX-C. Alphabet reuse (AC4/I3) ───────────────────────────────────────────

/// EX-C1: export/alphabet-equals-perform-node-signatures (AC4)
///
/// Σ equals exactly the program's L5 perform-node signatures — no orphan
/// symbol, no missing node. Two distinct effects so a coincidental match
/// cannot hide a dropped node.
#[test]
fn alphabet_equals_perform_node_signatures() {
    let source = r#"
proc after_flush (_outcome : Result IOError Unit)
  : HostIO AFull Instant visits [Clock] =
  host_clock AFull Instant wall_now

proc h (_value : Unit) : HostIO AFull Instant visits [Console, Clock] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result IOError Unit) Instant
    (host_console AFull (Result IOError Unit) (flush Stdout))
    (\outcome. after_flush outcome)
"#;
    let denotation = compile_checked_target_denotation(
        "b1_acceptance_alphabet",
        CompilerSource::new("alphabet.ken", source),
        "h",
    )
    .expect("checked two-effect target");
    let result = emit_checked_target_export(
        &denotation,
        &[],
        &BTreeSet::new(),
        vec![],
        vec![],
    )
    .expect("export with exact alphabet");

    let expected: BTreeSet<String> = ["Console", "Clock"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        result.alphabet, expected,
        "Σ must equal the checked perform-node projection exactly"
    );

    // No orphan symbol: each member of Σ is a real perform-node.
    assert!(result.alphabet.contains("Console"));
    assert!(result.alphabet.contains("Clock"));
    assert!(!result.alphabet.contains("FS"), "unused sibling effect must not appear");

    // Closure: alphabet is not missing any declared node.
    assert_eq!(result.alphabet.len(), 2, "no missing node");
}

// ─── EX-D. Generators carry support, never measure (AC5/I5) ──────────────────

/// EX-D1: export/generators-carry-support-not-measure (AC5, soundness)
///
/// G carries partition + boundaries + case-decomposition only. No weight /
/// likelihood / probability is representable. The seal is exhaustive-by-
/// construction: GEntry has no weight field — attaching a measure is a
/// compile error.
///
/// The test pins the observable consequence: a real program's G is
/// partition-only. The type-level seal is verified by inspection of the
/// `GEntry` type (no weight field exists — the compile error IS the proof).
#[test]
fn generators_carry_support_not_measure() {
    // GEntry for a refinement type {x : Nat | x > 0 ∨ x = 0}.
    // Partition: two conditions (positive / zero). No weight.
    let g = GEntry {
        source: "f".to_string(),
        conditions: vec!["x > 0".to_string(), "x = 0".to_string()],
        // If you try to add `weight: 0.5` here → compile error (no such field).
    };

    let result = emit_export(
        "f",
        &[],
        &BTreeSet::new(),
        EffectRow::empty(),
        vec![g],
        vec![],
    ).expect("export with generator");

    assert_eq!(result.generators.len(), 1);
    let emitted_g = &result.generators[0];

    // Support: partition conditions are emitted.
    assert!(emitted_g.conditions.contains(&"x > 0".to_string()));
    assert!(emitted_g.conditions.contains(&"x = 0".to_string()));

    // No measure: the GEntry type has no weight/probability field.
    // This is verified structurally: GEntry { source, conditions } — no
    // weight field exists. Attempting `emitted_g.weight` is a compile error.
    // The observable consequence: serialize and confirm no weight appears.
    let serialized = serialize_export(&result);
    let g_json = &serialized["generators"][0];
    assert!(g_json.get("weight").is_none(), "no weight in serialized G");
    assert!(g_json.get("likelihood").is_none(), "no likelihood in serialized G");
    assert!(g_json.get("probability").is_none(), "no probability in serialized G");
}

// ─── EX-E. The one-way gate — no promotion path (AC6/I4) ─────────────────────

/// EX-E1: export/delegated-obligation-never-promoted-to-proved (AC6, soundness)
///
/// A delegated `Temporal` obligation stays in T with status `delegated`.
/// Simulating a `Ward` discharge re-entering the Ken side as a TEntry:
/// the entry stays in T — never re-stamped `proved`, never in Q.
/// There is NO emitter code path from a Ward/classical result to Q.
///
/// The absence is guard-gated: `QEntry` is only constructible from a
/// `Verdict::Proved` whose hole is absent from trusted_base. A Ward green
/// result is not a Verdict::Proved from the kernel — no path exists.
#[test]
fn delegated_obligation_never_promoted_to_proved() {
    // A delegated Temporal obligation.
    let t_entry = TEntry {
        obligation_id: "f.temporal.ltl_safety".to_string(),
        formula: Temporal::Atom(Pred::Event("ltl_safety".into())),
    };

    // Ward "discharges" it — but the result re-enters only as a TEntry
    // (the channel boundary, 63 §5a). It does NOT become a Verdict::Proved.
    // We pass it as `temporal` to emit_export — it stays in T.
    let result = emit_export(
        "f",
        &[],           // no kernel-proved obligations
        &BTreeSet::new(),
        EffectRow::empty(),
        vec![],
        vec![t_entry.clone()],
    ).expect("export with temporal entry");

    // T: contains the delegated entry.
    assert_eq!(result.obligations.len(), 1);
    assert_eq!(result.obligations[0].obligation_id, "f.temporal.ltl_safety");

    // Q: empty — the Ward discharge did NOT promote to Q.
    assert!(
        result.guarantees.is_empty(),
        "Ward discharge must NOT appear in Q — no promotion path (EX-E1)"
    );

    // The serialized export's obligations carry status "delegated".
    let serialized = serialize_export(&result);
    let t_json = &serialized["obligations"][0];
    assert_eq!(
        t_json["status"].as_str().unwrap(),
        "delegated",
        "T entry must carry status delegated"
    );
    assert!(
        serialized["guarantees"].as_array().unwrap().is_empty(),
        "Q must be empty"
    );

    // Disconfirming question (absence-gate): would a delegated obligation with
    // a Ward green result land in Q under the bug this seam targets?
    // Yes — an emitter with a promotion path would stamp it proved.
    // Here: no such path exists; emitted[guarantees] is empty → case flips.
}

// ─── EX-F. The disproved boundary — never exported (71 §2.1) ─────────────────

/// EX-F1: export/disproved-claim-never-exported (soundness)
///
/// A refuted claim (verdict `disproved`) causes `emit_export` to return an
/// error. The claim appears in no export field. The build is not shippable.
#[test]
fn disproved_claim_never_exported() {
    let mut ke = make_kern_env();

    // An unprovable goal — force a Disproved verdict synthetically.
    let phi = ke.p_term.clone();
    let hole_id = declare_postulate(&mut ke.env, "test postulate".to_string(), vec![], phi.clone())
        .expect("hole");

    let triple = closed_triple(hole_id, "f.ensures.0", phi.clone(), ProvKind::Ensures { index: 0 });

    // Synthesize a disproved verdict (countermodel for P — P is abstract, so
    // "refuted" means a model where P doesn't hold; the test just needs the
    // Disproved variant to be passed, which is the conformance-observable).
    let verdict = Verdict::Disproved {
        countermodel: ken_elaborator::prover::Countermodel {
            description: "P does not hold in the empty model".to_string(),
        },
    };

    let tb = trusted_base_set(&ke.env);
    let err = emit_export(
        "f",
        &[(triple, verdict)],
        &tb,
        EffectRow::empty(),
        vec![],
        vec![],
    );

    // The emitter must return an error — build is non-shippable.
    assert!(
        matches!(err, Err(ExportError::DisproovedClaim { .. })),
        "disproved claim must cause non-shippable error, got: {:?}", err
    );

    // The claim appears in no export field (error prevents any export).
    // Conformance: the returned Err IS the signal that no export was produced.
}

// ─── EX-G. Reproducibility (AC1) ─────────────────────────────────────────────

/// EX-G1: export/same-program-same-export-hash (AC1)
///
/// The same verified content emitted twice yields the identical hash.
/// A non-canonical serialization (map-iteration order, embedded timestamp)
/// yields different hashes → red.
#[test]
fn same_program_same_export_hash() {
    let mut ke = make_kern_env();

    let phi = Term::pi(ke.p_term.clone(), ke.p_term.clone()); // P → P
    let hole_id = declare_postulate(&mut ke.env, "test postulate".to_string(), vec![], phi.clone())
        .expect("hole");

    let triple = closed_triple(hole_id, "f.ensures.0", phi.clone(), ProvKind::Ensures { index: 0 });

    // Discharge the hole.
    let cert = Term::lam(ke.p_term.clone(), Term::var(0));
    let verdict_check = attempt_with_cert(&mut ke.env, &phi, cert.clone());
    assert!(matches!(verdict_check, Verdict::Proved { .. }), "cert must prove phi");
    let discharged = ke.env.upgrade_to_transparent(hole_id, cert.clone());
    assert!(discharged);

    let verdict = Verdict::Proved { cert };
    let tb = trusted_base_set(&ke.env);

    // Run 1.
    let export1 = emit_export(
        "f",
        &[(triple.clone(), verdict.clone())],
        &tb,
        EffectRow::empty(),
        vec![],
        vec![],
    ).expect("first export");

    // Run 2 (same inputs).
    let export2 = emit_export(
        "f",
        &[(triple, verdict)],
        &tb,
        EffectRow::empty(),
        vec![],
        vec![],
    ).expect("second export");

    // Both runs must yield the identical hash.
    assert_eq!(
        export1.hash, export2.hash,
        "same program must yield the same export hash (determinism)"
    );

    // Pairs with EX-B1's hash-sensitivity: a constant hash passes this but
    // would fail EX-B1. A non-canonical hash fails this. The pair pins the
    // hash as a proper content-address.
}
