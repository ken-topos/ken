//! Sec1ct acceptance tests — conformance cases from
//! `conformance/security/ct/seed-ct.md` (AC1–AC7, CT-A through CT-E).
//!
//! The `@ct` discipline: unary taint-by-typing for constant-time.
//! Observable: elaboration **accept/reject** — NEVER a V3 verdict (§5a.3).
//!
//! **N1 trust boundary (§5a.6/§9):** `@ct` labels are erased before the kernel;
//! the `L-CT-SINK` rule + `LeakSink` classification are TRUSTED. These flip
//! cases are the SOLE net — the kernel cannot catch a CT leak.
//!
//! Grounding: `61 §5a.1`–`§5a.6`, `§7`, `§9`, `§H`; `36 §3.1`.

use ken_elaborator::ifc::{
    check_ct_promise, check_declassify, check_declassify_in_delta, classify_vis_op,
    CtGuaranteeQ, CtHook, CtPromise, DeclassifyCap, DeclassifyResult,
    FlowCtx, FlowResult, LeakageSink, VisOpClass,
    CT_BOT, CT_TOP, SECRET,
    TRIGGER_SEC1_DUAL, TRIGGER_SEC1_LAUNDER, TRIGGER_SEC1_REDUCE, TRIGGER_WARD,
};

// ─── CT-A. @ct steers a leakage sink → reject (AC1, AC2, AC3) ───────────────

/// CT-A1. A `@ct` key steering a `BranchGuard` sink is a type error.
/// Verdict flip: the same branch shape on a `ct⊥` value accepts (AC1, AC4).
#[test]
fn ct_value_steers_branch_guard_rejected() {
    let ctx = FlowCtx::new(); // pc.ct = false (ct⊥)
    let k = CT_TOP; // @ct key: ct⊤

    // `cmp (k : Bytes @ ct) (g : Bytes)` — k steers a branch → reject.
    // L-CT-SINK: (k.ct ⊔ pc.ct) = ct⊤ ⋢ ct⊥.
    let reject = ctx.l_ct_sink(&k, &LeakageSink::BranchGuard, "cmp.branch_on");
    assert!(reject.is_reject(), "AC1: @ct at BranchGuard → reject");
    let err = reject.error().unwrap();
    assert_eq!(err.rule, "L-CT-SINK");
    assert!(err.data_label.ct, "error names data.ct = ct⊤");
    assert!(!err.pc_label.ct,  "error names pc.ct = ct⊥ (direct value, not pc)");
    assert_eq!(err.site, "cmp.branch_on");
    assert_eq!(err.sink_clearance, CT_BOT, "sink clearance is ct⊥");

    // Flip: same branch shape, but ct⊥ operand → accepts.
    // This is the [Sec1-dual] orientation pin (non-degenerate, not green-vs-green).
    let accept = ctx.l_ct_sink(&CT_BOT, &LeakageSink::BranchGuard, "cmp.branch_on");
    assert!(accept.is_accept(), "AC1 flip: ct⊥ at BranchGuard → accept");

    // Orientation bug: if a sink mistakenly demanded ct⊤ (flipped), this case
    // would ACCEPT while CT-B1 would REJECT — both fail simultaneously.
    // The pair {CT-A1 rejects, CT-B1 accepts} pins the orientation.
}

/// CT-A2. A `@ct` index feeding a `MemoryIndex` sink is a type error (AC2).
/// Distinct trigger from CT-A1 (data-dependent cache access, not a branch).
#[test]
fn ct_value_steers_memory_index_rejected() {
    let ctx = FlowCtx::new();
    let i = CT_TOP; // @ct index: ct⊤

    // `lookup (i : Nat @ ct) (t : Array A)` — t[i] is a MemoryIndex sink.
    let reject = ctx.l_ct_sink(&i, &LeakageSink::MemoryIndex, "lookup.t_index");
    assert!(reject.is_reject(), "AC2: @ct at MemoryIndex → reject");
    let err = reject.error().unwrap();
    assert_eq!(err.rule, "L-CT-SINK");

    // Flip: ct⊥ index, or a constant-time full-table scan primitive → accepts.
    let accept = ctx.l_ct_sink(&CT_BOT, &LeakageSink::MemoryIndex, "lookup.t_index");
    assert!(accept.is_accept(), "AC2 flip: ct⊥ index at MemoryIndex → accept");

    // Sealed-set coverage: MemoryIndex is distinct from BranchGuard (AC2 ≠ AC1 trigger).
    assert_ne!(
        classify_vis_op(VisOpClass::ArrayIndex),
        classify_vis_op(VisOpClass::ControlFlowBranch),
        "AC2: MemoryIndex and BranchGuard are distinct sealed-set members"
    );
}

/// CT-A3. A `@ct` operand to a `var-time` primitive is a type error (AC3).
/// Discriminator: the `var-time` effect-signature flag, not the syntactic operator.
#[test]
fn ct_value_into_var_time_primitive_rejected() {
    let ctx = FlowCtx::new();
    let k = CT_TOP; // @ct operand

    // `naive_eq k g` — naive comparison is flagged var-time in its effect sig.
    let reject = ctx.l_ct_sink(&k, &LeakageSink::VarTimePrimitive, "naive_eq.operand");
    assert!(reject.is_reject(), "AC3: @ct at VarTimePrimitive → reject");
    let err = reject.error().unwrap();
    assert_eq!(err.rule, "L-CT-SINK");

    // Flip: same @ct operand into a CT-safe primitive (CtByteEq, not VarTimePrimitive).
    // classify_vis_op(CtByteEq) = None → no l_ct_sink call on that op → accepts.
    let ct_safe_class = classify_vis_op(VisOpClass::CtByteEq);
    assert!(ct_safe_class.is_none(), "AC3 flip: CtByteEq is not a LeakSink");

    // Exhaustive-by-construction: all three sealed-set members are classified
    // (no _ => None catch-all — the omission-hole discipline).
    assert_eq!(classify_vis_op(VisOpClass::ControlFlowBranch), Some(LeakageSink::BranchGuard));
    assert_eq!(classify_vis_op(VisOpClass::ArrayIndex),         Some(LeakageSink::MemoryIndex));
    assert_eq!(classify_vis_op(VisOpClass::VarTimePrimitive),   Some(LeakageSink::VarTimePrimitive));
}

/// CT-A4. A `ct⊥` inner op inside a `@ct`-guarded branch is rejected via
/// implicit `pc.ct` flow — the L-OBSERVE → L-CT-SINK composition (AC1 implicit).
#[test]
fn ct_guarded_branch_implicit_leak_rejected() {
    let ctx = FlowCtx::new(); // pc.ct = false

    // Scrutinee is @ct: `if (k[0] == g[0] : Bool @ ct) …`
    let scrutinee = CT_TOP;

    // L-OBSERVE raises pc.ct to ct⊤ in both branches.
    let ctx_branch = ctx.l_observe(scrutinee);
    assert!(ctx_branch.pc.ct, "AC4: pc.ct raised to ct⊤ after @ct branch");

    // Inner op: ct⊥ indices j, j' — but pc.ct = ct⊤ → caught anyway.
    // `if (k[0]==g[0] : Bool @ ct) then t[j] else t[j']`
    let j_label = CT_BOT; // ct⊥ index
    let reject = ctx_branch.l_ct_sink(&j_label, &LeakageSink::MemoryIndex, "t_index");
    assert!(reject.is_reject(), "AC4: ct⊥ inner op under ct⊤ pc → implicit CT leak → reject");
    let err = reject.error().unwrap();
    // The pc.ct is the culprit, not the operand's ct.
    assert!(!err.data_label.ct, "data is ct⊥");
    assert!(err.pc_label.ct,    "pc.ct = ct⊤ caused the rejection");

    // Flip-bug: if l_observe drops the pc.ct-raise, wrongly accepts.
    let buggy_ctx = FlowCtx::new(); // pc.ct stays false (bug: l_observe dropped)
    let wrongly_accept = buggy_ctx.l_ct_sink(&j_label, &LeakageSink::MemoryIndex, "t_index");
    assert!(
        wrongly_accept.is_accept(),
        "AC4 flip-bug: dropped pc.ct-raise → wrongly accepts (green-vs-red on pc.ct)"
    );
}

// ─── CT-B. Axes are independent — the [Sec1-dual] distinguishing pair (AC4) ──

/// CT-B1. A `Secret`-but-not-`@ct` value branches freely — the timing adversary
/// sees nothing. The `Conf` and `CT` axes are orthogonal product factors (§5a.1).
///
/// This is the non-degenerate distinguishing pair with CT-A1:
/// - CT-A1 **rejects** on `BranchGuard` (k is `ct⊤`)
/// - CT-B1 **accepts** on the SAME `BranchGuard` (p is `Secret`, `ct⊥`)
/// A `CT`-order flip inverts **both** — so the pair, not either alone, pins
/// the orientation (`[Sec1-dual]` net).
#[test]
fn secret_not_ct_branches_freely_accepted() {
    let ctx = FlowCtx::new();

    // `route (p : Tag @ Secret) (g : Bytes)` — p is Secret but ct⊥.
    let p = SECRET; // SECRET = Label { conf: 2, integ: 0, ct: false }
    assert!(!p.ct, "AC4: Secret but not @ct (ct⊥)");

    // Same BranchGuard shape as CT-A1, different CT component.
    let accept = ctx.l_ct_sink(&p, &LeakageSink::BranchGuard, "route.if");
    assert!(accept.is_accept(), "AC4: Secret-not-@ct at BranchGuard → accept");

    // The SAME shape that CT-A1 rejects (k = CT_TOP):
    let k = CT_TOP;
    let reject = ctx.l_ct_sink(&k, &LeakageSink::BranchGuard, "route.if");
    assert!(reject.is_reject(), "AC4 pair: @ct at BranchGuard → reject (CT-A1)");

    // Only difference: p.ct = false vs k.ct = true. Axes are orthogonal (§5a.1).
    assert_eq!(p.conf, k.conf + 2, "demo: conf(Secret)=2, conf(CT_TOP)=0 — distinct components");
    // The pair makes the orientation non-degenerate and guards [Sec1-dual].
    assert_eq!(TRIGGER_SEC1_DUAL, "[Sec1-dual]");
}

// ─── CT-C. Declassify ends the @ct span — the sole terminator (AC5) ──────────

/// CT-C1. After an authorised `@ct → ct⊥` declassify, the formerly-@ct value
/// steers a sink freely. The cap appears in `trusted_base_delta` (AC5).
#[test]
fn declassified_ct_value_steers_sink_accepted_and_listed() {
    // `cmp_ok (k : Bytes @ ct) (d : Cap_declassify[ct⊤→ct⊥]) (g : Bytes)`
    // `let k' = declassify d (ct_eq k g)` → k' : Bool @ ct⊥
    let ct_cap = DeclassifyCap::new(CT_TOP, CT_BOT);
    assert!(ct_cap.is_valid(), "AC5: Cap_declassify[ct⊤→ct⊥] is valid");

    // Authorised downgrade: ct⊤ → ct⊥
    let result = check_declassify(Some(&ct_cap), CT_TOP, CT_TOP, CT_BOT);
    let k_prime = match result {
        DeclassifyResult::Accept { downgraded_label } => downgraded_label,
        other => panic!("AC5: expected Accept, got {:?}", other),
    };
    assert!(!k_prime.ct, "AC5: post-declassify label is ct⊥");

    // The @ct declassify authority must appear in trusted_base_delta.
    let authority_id = "decl:ct⊤→ct⊥";
    let delta = vec![authority_id.to_owned()];
    assert!(
        check_declassify_in_delta(authority_id, &delta),
        "AC5: @ct cap appears in delta"
    );

    // Now steer the ct⊥ value into the same BranchGuard → accepts.
    let ctx = FlowCtx::new();
    let accept = ctx.l_ct_sink(&k_prime, &LeakageSink::BranchGuard, "cmp_ok.branch_on");
    assert!(accept.is_accept(), "AC5: ct⊥ post-declassify steers BranchGuard → accept");

    // Flip vs CT-A1: same sink, same branch shape — only the authorised declassify
    // turns reject into accept. The declassify machinery is the sole span terminator.
    let reject = ctx.l_ct_sink(&CT_TOP, &LeakageSink::BranchGuard, "cmp_ok.branch_on");
    assert!(reject.is_reject(), "AC5 flip: pre-declassify @ct → same sink → reject");
}

// ─── CT-D. CT-in-parameter promise + the Q export (AC6) ─────────────────────

/// CT-D1. A CT-in-parameter promise is **checked** (not a decoration): a clean
/// body emits a source-level `Q` clause; a leaking body is rejected (AC6).
#[test]
fn ct_in_parameter_promise_checked_emits_q() {
    // `ct_eq (k : Bytes @ ct) (g : Bytes) : Bool @ ct` — promises CT-in-k.
    let k_param = CT_TOP; // k bound at ct⊤ for the check
    let ctx = FlowCtx::new();

    let promise = CtPromise {
        param_name:   "k".to_owned(),
        source_level: true,
    };

    // Good body: `fold_and (map2 ct_byte_eq k g)` — ct_byte_eq is a PureOp/CtByteEq.
    // classify_vis_op(CtByteEq) = None → no LeakSink triggered → body = Accept.
    // (The test simulates the body-check result; the body does not route k to a sink.)
    let good_body = FlowResult::Accept;
    let q = check_ct_promise(&promise, good_body)
        .expect("AC6: clean body → Q emitted");
    assert_eq!(q.param_name, "k",  "AC6: Q names the parameter");
    assert!(q.source_level,         "AC6: Q is source-level precondition, NOT timing guarantee");

    // Structural assertion: Q is present + well-formed (guards the silent-omission hole).
    // A function accepted with NO Q clause would be an over-claim (the completion-hole
    // backstop from the V2 discipline, in the CT domain).
    let _: &CtGuaranteeQ = &q; // type-level: Q struct was emitted

    // Bad body: `branch_on k[0]` — k steers BranchGuard → reject.
    let bad_body = ctx.l_ct_sink(&k_param, &LeakageSink::BranchGuard, "branch_on");
    assert!(bad_body.is_reject(), "AC6 pre-check: leaking body rejects");
    let err = check_ct_promise(&promise, bad_body)
        .expect_err("AC6: leaking body → promise broken → rejected");
    assert_eq!(err.rule, "L-CT-SINK", "AC6: reject names L-CT-SINK");

    // (oracle) note: the literal field-token spelling of the Q clause is B1/71-deferred.
    // We assert the concept (present, named param, source-level) — not the token.
}

// ─── CT-E. Honest limits — no over-claim (AC7) ───────────────────────────────

/// CT-E1. No case in this corpus asserts Ken proves constant-time execution.
/// Ken proves only the source-level precondition `Q`. Timing is `[Ward]`'s.
/// The three `[Sec1-*]` triggers are present, not silent (AC7).
#[test]
fn timing_guarantee_delegated_not_claimed() {
    // The three kernel-blind surfaces remain named as scoped work (§H).
    assert_eq!(TRIGGER_SEC1_DUAL,    "[Sec1-dual]");
    assert_eq!(TRIGGER_SEC1_LAUNDER, "[Sec1-launder]");
    assert_eq!(TRIGGER_SEC1_REDUCE,  "[Sec1-reduce]");

    // The binary timing guarantee is delegated to [Ward].
    assert_eq!(TRIGGER_WARD, "[Ward]");

    // @ct hook carries [Ward] (Sec1ct now landed; timing deferred to Ward, not [Sec1ct]).
    let hook = CtHook::new(true);
    assert!(hook.has_reify_trigger(), "AC7: @ct hook carries a reify-trigger");
    assert_eq!(hook.deferred_timing, Some(TRIGGER_WARD), "AC7: trigger is [Ward]");

    // CT-in-param Q is a SOURCE-LEVEL precondition, not a timing guarantee.
    let q = CtGuaranteeQ { param_name: "k".to_owned(), source_level: true };
    assert!(q.source_level, "AC7: Q is source-level — Ken owns the precondition");
    // Ken never asserts "well-typed ⇒ constant-time execution."
    // That would over-claim past Ken's locked granularity (61 §5a.6/§H).

    // The [Sec1ct] trigger name is still accessible (it is the name of this WP's
    // delivered work, referenced from the Sec1 corpus for cross-linking).
    use ken_elaborator::ifc::TRIGGER_SEC1CT;
    assert_eq!(TRIGGER_SEC1CT, "[Sec1ct]");
}

// ─── Cross-case consistency sweep ─────────────────────────────────────────────

/// Cross-case: the @ct discipline is internally consistent.
///
/// - The reject class {A1, A2, A3, A4} all agree: any `ct⊤` at any sealed
///   `LeakSink` member (directly or via `pc.ct`) → reject; observable = accept/reject,
///   never a V3 verdict.
/// - The accept class {B1, C1, D1-accept-arm} all agree: `ct⊥` at sink → accept.
/// - The [Sec1-dual] distinguishing pair {A1, B1}: A1 rejects WHILE B1 accepts on
///   the SAME `BranchGuard` shape — the orientation pin. A CT-order flip inverts both.
/// - The N1 trust boundary {A1–A4, C1}: labels erased before kernel; these flip
///   cases, not the kernel, are the sole backstop for CT leaks.
#[test]
fn ct_discipline_cross_case_sweep() {
    let ctx = FlowCtx::new();

    // Reject class: all three sealed-set members, direct @ct.
    for (sink, site) in [
        (&LeakageSink::BranchGuard,     "sweep.branch"),
        (&LeakageSink::MemoryIndex,      "sweep.index"),
        (&LeakageSink::VarTimePrimitive, "sweep.vartime"),
    ] {
        let r = ctx.l_ct_sink(&CT_TOP, sink, site);
        assert!(r.is_reject(), "sweep reject: CT_TOP at {:?} → reject", sink);
    }

    // Accept class: ct⊥ at the same sinks.
    for (sink, site) in [
        (&LeakageSink::BranchGuard,     "sweep.branch"),
        (&LeakageSink::MemoryIndex,      "sweep.index"),
        (&LeakageSink::VarTimePrimitive, "sweep.vartime"),
    ] {
        let a = ctx.l_ct_sink(&CT_BOT, sink, site);
        assert!(a.is_accept(), "sweep accept: CT_BOT at {:?} → accept", sink);
    }

    // [Sec1-dual] orientation pin: A1 rejects WHILE B1 accepts on same shape.
    let branch_a1 = ctx.l_ct_sink(&CT_TOP,  &LeakageSink::BranchGuard, "pin.branch");
    let branch_b1 = ctx.l_ct_sink(&SECRET,  &LeakageSink::BranchGuard, "pin.branch");
    assert!(branch_a1.is_reject(), "pin: @ct at BranchGuard rejects (A1)");
    assert!(branch_b1.is_accept(), "pin: Secret-not-@ct at BranchGuard accepts (B1)");

    // Implicit pc.ct flow: ct⊥ inner op under ct⊤ pc → reject.
    let ctx_ct_pc = FlowCtx::new().l_observe(CT_TOP);
    assert!(ctx_ct_pc.pc.ct, "sweep: pc.ct raised after @ct l_observe");
    let implicit = ctx_ct_pc.l_ct_sink(&CT_BOT, &LeakageSink::MemoryIndex, "sweep.implicit");
    assert!(implicit.is_reject(), "sweep: implicit pc.ct flow → reject");

    // Mode discipline: the observable is elaboration accept/reject, NOT a V3 verdict.
    // (No Verdict import in this file is intentional — CT path never reaches the prover.)
    // A [Sec1-reduce] cite here would be a mode-confusion bug.
    assert_eq!(TRIGGER_SEC1_REDUCE, "[Sec1-reduce]"); // named, not confused with this path
}
