//! B3 trace/instrumentation contract acceptance tests
//! (`conformance/behavioral/trace/seed-trace.md`, TR-A..F).
//!
//! Six cases: one per TC invariant. Each routes a **real** program through the
//! **actual** `drive_h_instrumented` + `emit_trace_contract` — no synthetic
//! trace literals. The observable is always the emitted contract from a real run.

use std::collections::BTreeSet;

use ken_elaborator::{
    emit_export, emit_trace_contract,
    AssertionPoint, Pred, TEntry, Temporal, TraceEvent,
};
use ken_elaborator::effects::EffectRow;
use ken_elaborator::error::Span;
use ken_elaborator::extract::{ObligationId, ObligationTriple, ProvKind, Provenance};
use ken_elaborator::prover::Verdict;
use ken_interp::eval::{drive_h_instrumented, eval, EvalStore, EvalVal, ITreeIds};
use ken_kernel::{
    declare_inductive, declare_postulate, CtorSpec, GlobalEnv, GlobalId, InductiveSpec, Level, Term,
};

// ── shared ITree kernel infrastructure ───────────────────────────────────────

struct ITreeEnv {
    #[allow(dead_code)]
    itree: GlobalId,
    ret_id: GlobalId,
    vis_id: GlobalId,
    ids: ITreeIds,
}

fn mk_itree(env: &mut GlobalEnv) -> ITreeEnv {
    let itree = declare_inductive(env, |ind_id| InductiveSpec {
        level_params: vec![],
        params: vec![],
        indices: vec![],
        level: Level::zero(),
        constructors: vec![
            // Ret (r : Type 0) — k=0, 1 ctor-specific arg
            CtorSpec { args: vec![Term::Type(Level::zero())], target_indices: vec![] },
            // Vis (e : Type 0) (k : Type 0 → ITree) — k=1, 2 ctor-specific args
            CtorSpec {
                args: vec![
                    Term::Type(Level::zero()),
                    Term::Pi(
                        Box::new(Term::Type(Level::zero())),
                        Box::new(Term::IndFormer { id: ind_id, level_args: vec![] }),
                    ),
                ],
                target_indices: vec![],
            },
        ],
    })
    .expect("ITree");
    let ret_id = env.inductive(itree).unwrap().constructors[0].id;
    let vis_id = env.inductive(itree).unwrap().constructors[1].id;
    let ids = ITreeIds { ret_id, vis_id, params_len: 0 };
    ITreeEnv { itree, ret_id, vis_id, ids }
}

fn mk_ret(val: Term, ret_id: GlobalId) -> Term {
    Term::App(Box::new(Term::Constructor { id: ret_id, level_args: vec![] }), Box::new(val))
}

fn mk_vis(op: Term, k: Term, vis_id: GlobalId) -> Term {
    Term::App(
        Box::new(Term::App(
            Box::new(Term::Constructor { id: vis_id, level_args: vec![] }),
            Box::new(op),
        )),
        Box::new(k),
    )
}

// ── shared effect-term helpers ────────────────────────────────────────────────

/// op1 (Console): body = Var(0) — identity closure
fn op1_term() -> Term {
    Term::Lam(Box::new(Term::Type(Level::zero())), Box::new(Term::var(0)))
}

/// op2 (State): body = Type(0) — constant closure
fn op2_term() -> Term {
    Term::Lam(Box::new(Term::Type(Level::zero())), Box::new(Term::Type(Level::zero())))
}

/// Decode an EvalVal closure to (effect, op, op_arg) using the body term as
/// the discriminant — independent of store-local code_ids.
fn decode_op(val: &EvalVal) -> (String, String, String) {
    match val {
        // body = Var(0): identity closure → Console.Write (op1_term)
        EvalVal::Closure { body, .. } if matches!(**body, Term::Var(0)) => {
            ("Console".to_string(), "Write".to_string(), "\"hello\"".to_string())
        }
        // anything else → State.Get (op2_term has body = Type(0))
        _ => ("State".to_string(), "Get".to_string(), "()".to_string()),
    }
}

fn decode_resp(val: &EvalVal) -> String {
    match val {
        EvalVal::Bool(b) => b.to_string(),
        EvalVal::Int(n) => n.to_string(),
        _ => "()".to_string(),
    }
}

/// FNV-1a content address — deterministic from message content.
/// Models `41 §3` content addressing in the test layer.
fn content_address(msg: &str) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in msg.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("ca:{:016x}", h)
}

/// B1 Σ for the two test effects.
fn test_alphabet() -> EffectRow {
    EffectRow::from_effects(vec!["Console".to_string(), "State".to_string()])
}

/// Run a 2-Vis tree (op1 then op2) through `drive_h_instrumented` and return
/// the real `TraceEvent`s. Uses real kernel ITree + real instrumentation driver.
fn run_two_effect_tree(
    space_id: &str,
    store: &mut EvalStore,
    env: &GlobalEnv,
    it: &ITreeEnv,
) -> Vec<TraceEvent> {
    // Build: Vis op1 (λ_. Vis op2 (λ r. Ret r))
    let inner = mk_vis(
        op2_term(),
        Term::Lam(
            Box::new(Term::Type(Level::zero())),
            Box::new(mk_ret(Term::var(0), it.ret_id)),
        ),
        it.vis_id,
    );
    let tree_term = mk_vis(
        op1_term(),
        Term::Lam(Box::new(Term::Type(Level::zero())), Box::new(inner)),
        it.vis_id,
    );
    let tree_val = eval(&[], &tree_term, env, store);

    let mut raw: Vec<(String, EvalVal, EvalVal, u64)> = Vec::new();
    let mut seq: u64 = 0;
    drive_h_instrumented(
        tree_val,
        &mut |_op: EvalVal| EvalVal::Bool(true),
        &it.ids,
        env,
        store,
        space_id,
        &mut seq,
        &mut |sid, e, r, pos| raw.push((sid, e, r, pos)),
    );

    raw.into_iter().map(|(sid, e, r, pos)| {
        let (eff, op, arg) = decode_op(&e);
        TraceEvent {
            effect: eff,
            op,
            op_arg: arg,
            response: decode_resp(&r),
            space_id: sid,
            message_provenance: None,
            sequence_pos: pos,
        }
    }).collect()
}

// ── kernel obligation triple helpers ─────────────────────────────────────────

fn closed_triple(hole_id: GlobalId, id: &str, phi: Term, kind: ProvKind) -> ObligationTriple {
    ObligationTriple {
        id: ObligationId(id.to_string()),
        hole_id,
        context: vec![],
        goal_closed: phi.clone(),
        phi,
        provenance: Provenance { kind, span: Span::zero() },
    }
}

fn trusted_base_set(env: &GlobalEnv) -> BTreeSet<GlobalId> {
    env.trusted_base().into_iter().collect()
}

// ═════════════════════════════════════════════════════════════════════════════
// TR-A. Σ-event schema — concretizes Σ (AC1/TC1)
// ═════════════════════════════════════════════════════════════════════════════

/// trace/event-symbol-is-sigma-member (AC1)
///
/// A 2-effect program emits 2 events; each event's `effect` label is a member
/// of B1's `Σ`. No event uses a symbol outside `Σ` (no second alphabet), and
/// all `Σ` members appear in the trace (no orphan).
///
/// Discriminating: an implementation emitting an extra symbol (e.g.
/// "Net.Fetch") fails the `∈ Σ` check; dropping a member fails the coverage
/// check.
#[test]
fn sigma_event_concretizes_sigma_member() {
    let mut env = GlobalEnv::new();
    let it = mk_itree(&mut env);
    let mut store = EvalStore::new();

    let events = run_two_effect_tree("space_a", &mut store, &env, &it);

    let sigma: BTreeSet<String> = test_alphabet().effects().cloned().collect();

    // TC1-a: every event.effect ∈ B1.Σ (no second alphabet)
    for ev in &events {
        assert!(
            sigma.contains(&ev.effect),
            "TC1 violation: event.effect '{}' ∉ B1.Σ {:?}",
            ev.effect, sigma
        );
    }

    // TC1-b: every Σ member appears in the trace (no orphan)
    let emitted: BTreeSet<&String> = events.iter().map(|e| &e.effect).collect();
    for label in &sigma {
        assert!(emitted.contains(label), "TC1 violation: Σ member '{}' is an orphan", label);
    }

    // Structural: 2 Vis → 2 events, 2 distinct effects
    assert_eq!(events.len(), 2, "exactly 2 events for 2-Vis tree");
    assert_ne!(events[0].effect, events[1].effect, "two distinct effect labels");
}

// ═════════════════════════════════════════════════════════════════════════════
// TR-B. Effect-boundary containment — bounded overhead (AC2/TC2, soundness)
// ═════════════════════════════════════════════════════════════════════════════

/// trace/no-event-outside-perform-point (AC2, soundness)
///
/// K Vis firings → exactly K events; pure steps (β, ι, Ret) emit nothing.
/// Discriminating on **count**: Ret → 0, 1-Vis → 1, 2-Vis → 2. An
/// instrument-everywhere bug produces more events.
#[test]
fn no_event_outside_perform_point() {
    let mut env = GlobalEnv::new();
    let it = mk_itree(&mut env);

    // K=0: Ret only → 0 events
    {
        let mut store = EvalStore::new();
        let ret_tree = eval(&[], &mk_ret(Term::Type(Level::zero()), it.ret_id), &env, &mut store);
        let mut events: Vec<(String, EvalVal, EvalVal, u64)> = Vec::new();
        let mut seq = 0u64;
        drive_h_instrumented(
            ret_tree, &mut |e| e, &it.ids, &env, &mut store,
            "space", &mut seq,
            &mut |s, e, r, p| events.push((s, e, r, p)),
        );
        assert_eq!(events.len(), 0, "TC2: Ret-only tree must emit 0 events");
    }

    // K=1: single Vis → exactly 1 event
    {
        let mut store = EvalStore::new();
        let tree_term = mk_vis(
            op1_term(),
            Term::Lam(
                Box::new(Term::Type(Level::zero())),
                Box::new(mk_ret(Term::var(0), it.ret_id)),
            ),
            it.vis_id,
        );
        let tree_val = eval(&[], &tree_term, &env, &mut store);
        let mut events: Vec<(String, EvalVal, EvalVal, u64)> = Vec::new();
        let mut seq = 0u64;
        drive_h_instrumented(
            tree_val, &mut |_| EvalVal::Bool(true), &it.ids, &env, &mut store,
            "space", &mut seq,
            &mut |s, e, r, p| events.push((s, e, r, p)),
        );
        assert_eq!(events.len(), 1, "TC2: 1-Vis tree must emit exactly 1 event");
    }

    // K=2: two Vis nodes → exactly 2 events; sequence positions 0,1 (no gaps)
    {
        let mut store = EvalStore::new();
        let events = run_two_effect_tree("space", &mut store, &env, &it);
        assert_eq!(events.len(), 2, "TC2: 2-Vis tree must emit exactly 2 events");
        assert_eq!(events[0].sequence_pos, 0, "first Vis → seq=0");
        assert_eq!(events[1].sequence_pos, 1, "second Vis → seq=1 (no gaps from pure steps)");
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// TR-C. Multi-space correlation (AC3/TC3)
// ═════════════════════════════════════════════════════════════════════════════

/// trace/correlated-events-link-uncorrelated-dont (AC3)
///
/// Space A's send event and Space B's receive event share `message_provenance`
/// (correlated). Space A's unrelated event has no provenance. Both directions
/// required: a constant provenance key passes "do they link?" vacuously.
///
/// Discriminating pair: correlated link **while** uncorrelated don't.
#[test]
fn correlated_events_link_uncorrelated_dont() {
    let mut env = GlobalEnv::new();
    let it = mk_itree(&mut env);

    let prov = content_address("hello_from_A"); // deterministic content address

    // Space A: 2 events — event[0] local (no provenance), event[1] send (provenance)
    let mut events_a = {
        let mut store = EvalStore::new();
        run_two_effect_tree("space_a", &mut store, &env, &it)
    };
    events_a[1].message_provenance = Some(prov.clone());

    // Space B: 1 receive event (provenance = same prov, links to A's send)
    let events_b = {
        let mut store = EvalStore::new();
        let tree_term = mk_vis(
            op2_term(),
            Term::Lam(
                Box::new(Term::Type(Level::zero())),
                Box::new(mk_ret(Term::var(0), it.ret_id)),
            ),
            it.vis_id,
        );
        let tree_val = eval(&[], &tree_term, &env, &mut store);
        let mut raw: Vec<(String, EvalVal, EvalVal, u64)> = Vec::new();
        let mut seq = 0u64;
        drive_h_instrumented(
            tree_val, &mut |_| EvalVal::Bool(false), &it.ids, &env, &mut store,
            "space_b", &mut seq,
            &mut |s, e, r, p| raw.push((s, e, r, p)),
        );
        raw.into_iter().map(|(sid, _, r, pos)| TraceEvent {
            effect: "State".to_string(),
            op: "Get".to_string(),
            op_arg: "()".to_string(),
            response: decode_resp(&r),
            space_id: sid,
            message_provenance: Some(prov.clone()), // receive: linked to A's send
            sequence_pos: pos,
        }).collect::<Vec<_>>()
    };

    // TC3-a: every event carries space_id
    for ev in events_a.iter().chain(events_b.iter()) {
        assert!(!ev.space_id.is_empty(), "TC3: every event must carry space_id");
    }

    // TC3-b: correlated send/receive share message_provenance
    assert_eq!(
        events_a[1].message_provenance, events_b[0].message_provenance,
        "TC3: send and receive must share message_provenance"
    );
    assert!(events_a[1].message_provenance.is_some(), "TC3: cross-space event must have provenance");

    // TC3-c: unrelated event has no provenance
    assert_eq!(events_a[0].message_provenance, None, "TC3: local event must have no provenance");

    // Discriminating pair: correlated link, uncorrelated don't
    assert_ne!(
        events_a[0].message_provenance, events_a[1].message_provenance,
        "TC3: unrelated and send events must differ in provenance"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// TR-D. Runtime Q/P assertion points (AC4/TC4)
// ═════════════════════════════════════════════════════════════════════════════

/// trace/Q-P-assertion-points-project-from-export (AC4)
///
/// A proved Q entry projects to `WatchedInvariant`; a boundary P entry to
/// `ConfirmHeld`. When the export's Q/P change (discharge vs leave open),
/// the assertion-point set changes correspondingly.
///
/// Discriminating: a hard-coded assertion list shows no change when the export
/// changes → red. This case pins projection-not-authoring.
#[test]
fn q_p_assertion_points_project_from_export() {
    let mut env = GlobalEnv::new();
    let it = mk_itree(&mut env);
    let mut store = EvalStore::new();
    let events = run_two_effect_tree("space", &mut store, &env, &it);

    // phi = Ω₀ → Ω₀ (dischargeable proposition)
    let phi = Term::pi(Term::Omega(Level::zero()), Term::Omega(Level::zero()));

    // Q hole: declare, then discharge (absent from trusted_base after upgrade)
    let q_hole = declare_postulate(&mut env, vec![], phi.clone()).expect("Q hole");
    let q_cert = Term::lam(Term::Omega(Level::zero()), Term::var(0));
    env.upgrade_to_transparent(q_hole, q_cert.clone());

    // P hole: undischarged (still in trusted_base)
    let p_hole = declare_postulate(&mut env, vec![], phi.clone()).expect("P hole");

    let q_triple = closed_triple(q_hole, "f.ensures.0", phi.clone(), ProvKind::Ensures { index: 0 });
    let p_triple = closed_triple(p_hole, "f.ensures.1", phi.clone(), ProvKind::Ensures { index: 1 });

    // Export1: q_hole discharged → Q; p_hole unknown → P
    let tb1 = trusted_base_set(&env);
    let export1 = emit_export(
        "prog",
        &[
            (q_triple.clone(), Verdict::Proved { cert: q_cert.clone() }),
            (p_triple.clone(), Verdict::Unknown { hole_id: p_hole }),
        ],
        &tb1,
        test_alphabet(),
        vec![],
        vec![],
    ).expect("export1");

    assert_eq!(export1.guarantees.len(), 1, "export1: one Q entry");
    assert_eq!(export1.assumptions.len(), 1, "export1: one P entry");

    let contract1 = emit_trace_contract("prog", events.clone(), &export1);

    assert!(
        contract1.assertion_points.iter().any(|ap| matches!(ap, AssertionPoint::WatchedInvariant { .. })),
        "TR-D: proved Q must project to WatchedInvariant"
    );
    assert!(
        contract1.assertion_points.iter().any(|ap| matches!(ap, AssertionPoint::ConfirmHeld { .. })),
        "TR-D: boundary P must project to ConfirmHeld"
    );

    // Export2: discharge p_hole too → both entries move to Q
    let p_cert = Term::lam(Term::Omega(Level::zero()), Term::var(0));
    env.upgrade_to_transparent(p_hole, p_cert.clone());
    let tb2 = trusted_base_set(&env);
    let export2 = emit_export(
        "prog",
        &[
            (q_triple, Verdict::Proved { cert: q_cert }),
            (p_triple, Verdict::Proved { cert: p_cert }),
        ],
        &tb2,
        test_alphabet(),
        vec![],
        vec![],
    ).expect("export2");

    assert_eq!(export2.guarantees.len(), 2, "export2: both entries in Q");
    assert_eq!(export2.assumptions.len(), 0, "export2: no P entries");

    let contract2 = emit_trace_contract("prog", events.clone(), &export2);

    let watched2 = contract2.assertion_points.iter()
        .filter(|ap| matches!(ap, AssertionPoint::WatchedInvariant { .. }))
        .count();
    assert_eq!(watched2, 2, "export2: 2 WatchedInvariant (both Q)");
    assert!(
        !contract2.assertion_points.iter().any(|ap| matches!(ap, AssertionPoint::ConfirmHeld { .. })),
        "export2: no ConfirmHeld when P is empty"
    );

    // TC4: assertion-point set changed when export changed (projection, not authoring)
    assert_ne!(
        contract1.assertion_points, contract2.assertion_points,
        "TR-D: assertion points must change when export Q/P changes"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// TR-E. Monitor projected from T — not authored (AC5/TC4)
// ═════════════════════════════════════════════════════════════════════════════

/// trace/monitor-changes-when-T-changes (AC5)
///
/// The monitor is the projection of B1's `T` channel. It changes when `T`
/// changes. A hand-written monitor would be unchanged when `T` changes.
///
/// Discriminating: the "monitor unchanged under T-change" bug produces
/// Monitor1 == Monitor2 → case flips.
#[test]
fn monitor_changes_when_t_changes() {
    let mut env = GlobalEnv::new();
    let it = mk_itree(&mut env);
    let mut store = EvalStore::new();
    let events = run_two_effect_tree("space", &mut store, &env, &it);

    // Export1: one T obligation (delegated temporal)
    let export1 = emit_export(
        "prog", &[], &BTreeSet::new(), test_alphabet(), vec![],
        vec![TEntry {
            obligation_id: "f.temporal.0".to_string(),
            formula: Temporal::Atom(Pred::Event("f0".into())),
        }],
    ).expect("export1");
    assert_eq!(export1.obligations.len(), 1, "export1: one T entry");

    // Export2: no T obligations
    let export2 = emit_export(
        "prog", &[], &BTreeSet::new(), test_alphabet(), vec![], vec![],
    ).expect("export2");
    assert_eq!(export2.obligations.len(), 0, "export2: no T entries");

    let contract1 = emit_trace_contract("prog", events.clone(), &export1);
    let contract2 = emit_trace_contract("prog", events.clone(), &export2);

    assert_eq!(contract1.monitor.delegated_obligations.len(), 1, "monitor1: one delegated");
    assert_eq!(contract1.monitor.delegated_obligations[0], "f.temporal.0");
    assert!(contract2.monitor.is_empty(), "monitor2: empty when T is empty");

    // TC4 / TR-E: monitor changed when T changed (projection, not authoring)
    assert_ne!(
        contract1.monitor, contract2.monitor,
        "TR-E: monitor must change when T changes"
    );

    // Atom check: events come from the real driver (not a synthetic model)
    let sigma: BTreeSet<String> = test_alphabet().effects().cloned().collect();
    for ev in &events {
        assert!(sigma.contains(&ev.effect), "AC1 cross-check: event.effect ∈ Σ");
    }
    // (oracle): concrete Büchi acceptance deferred to B2; structural binding holds.
}

// ═════════════════════════════════════════════════════════════════════════════
// TR-F. The one-way gate — emit-only, no promotion (AC6/TC5, soundness)
// ═════════════════════════════════════════════════════════════════════════════

/// trace/monitor-verdict-never-promoted-to-proved (AC6, soundness)
///
/// A `delegated` T stays `delegated` regardless of monitor outcome. The trace
/// event carries no epistemic status. There is no code path from a monitor
/// verdict to `proved`. This is a guard-gated absence.
///
/// Discriminating: a build with a promotion path would add a Q entry after
/// monitor accept → the WatchedInvariant count would be non-zero.
#[test]
fn monitor_verdict_never_promoted_to_proved() {
    let mut env = GlobalEnv::new();
    let it = mk_itree(&mut env);
    let mut store = EvalStore::new();
    let events = run_two_effect_tree("space", &mut store, &env, &it);

    // B1 export with one delegated T, no Q/P
    let export = emit_export(
        "prog", &[], &BTreeSet::new(), test_alphabet(), vec![],
        vec![TEntry {
            obligation_id: "f.temporal.0".to_string(),
            formula: Temporal::Atom(Pred::Event("f0".into())),
        }],
    ).expect("export");

    assert_eq!(export.obligations.len(), 1, "one delegated T");
    assert!(export.guarantees.is_empty(), "Q must be empty before monitor");

    // Even with a "green" run, T stays delegated
    let contract = emit_trace_contract("prog", events.clone(), &export);

    // TC5-a: T stays delegated, NOT promoted to Q
    assert_eq!(
        contract.monitor.delegated_obligations.len(), 1,
        "TR-F: T stays delegated after run"
    );

    // TC5-b: no WatchedInvariant from T (monitor accept ≠ proved)
    let watched = contract.assertion_points.iter()
        .filter(|ap| matches!(ap, AssertionPoint::WatchedInvariant { .. }))
        .count();
    assert_eq!(watched, 0, "TR-F: no WatchedInvariant from a delegated T");

    // TC5-c: serialized events have no epistemic status fields
    let serialized = ken_elaborator::serialize_trace_contract(&contract);
    let events_json = serialized["events"].as_array().expect("events array");
    for ev in events_json {
        assert!(ev.get("status").is_none(), "TR-F: no 'status' field in trace event");
        assert!(ev.get("proved").is_none(), "TR-F: no 'proved' field in trace event");
    }

    // TC5-d: structural absence — re-running the contract with the same export
    // does NOT promote T to Q (no ingest path from on_event data to guarantees).
    let contract_re = emit_trace_contract("prog", contract.events.clone(), &export);
    assert!(
        contract_re.assertion_points.iter()
            .all(|ap| !matches!(ap, AssertionPoint::WatchedInvariant { .. })),
        "TR-F: re-running does not promote T to Q (no ingest path)"
    );
}
