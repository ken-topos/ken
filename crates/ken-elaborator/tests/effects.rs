//! L5 conformance cases — row lattice, ITree denotation, row-poly.
//!
//! Covers:
//! - EFF1 (row inference + escape), EFF3 (capability gating), EFF4
//!   (type-level space + handler), EFF5 (pure/impure boundary) — from L5-build.
//! - EFF2 (ITree denotation structure) — now runnable (K1.5 merged).
//! - Row-poly (higher-order parameter row inference) — L5-denotation.
//!
//! Every negative case is **discriminating**: verdict FLIPS between the correct
//! variant and the targeted-bug variant (COORDINATION §7).

use std::collections::HashMap;
use std::rc::Rc;

use ken_elaborator::effects::{
    bind, check_capabilities_no_handler, check_cross_space, check_escape,
    check_higher_order_guard, check_row_poly_escape, check_tail_resumptive,
    handler_fold, perform, infer_all, infer_row_poly,
    CapParam, CrossSpaceAccess, EffectDecl, EffectError, EffectName, EffectRow,
    HandlerCase, ITree, ResumeKind, RowSubst, RowType, RowVar, WitnessMap,
};

// ============================================================
// EFF1 — effect row: transitive inference + static check
// ============================================================

/// `surface/effects/eff-row-inferred-transitively` (oracle)
///
/// Structural assertion: `infer_row` returns exactly `[FS]` for a function
/// that calls only `read_config` (row `[FS]`). A bug that fails to release
/// `read_config`'s latent `[FS]` infers `[]` — the asserted row catches it
/// independently of any accept/reject verdict.
#[test]
fn eff_row_inferred_transitively() {
    // Seed: leaf primitive `read_config` has declared row [FS].
    let seed: HashMap<String, EffectRow> = [(
        "read_config".to_string(),
        EffectRow::singleton("FS"),
    )]
    .into();

    // `setup` calls only `read_config`; no declared row, no direct performs.
    let setup = EffectDecl::new("setup").with_callee("read_config");
    let rows = infer_all(&seed, &[setup]);

    assert_eq!(
        rows["setup"],
        EffectRow::singleton("FS"),
        "inferred row must be exactly [FS] — a miss gives [] (bug)"
    );
}

/// `surface/effects/eff-row-union-two-effects` (oracle)
///
/// `boot` calls both `read_config` (FS) and `now` (Clock); inferred row must
/// be the join `[Clock, FS]`. ≥2 distinct effects. A bug taking only the
/// first/last call's effect gives `[FS]` or `[Clock]` — the join assertion
/// flips the structural check.
#[test]
fn eff_row_union_two_effects() {
    let seed: HashMap<String, EffectRow> = [
        ("read_config".to_string(), EffectRow::singleton("FS")),
        ("now".to_string(), EffectRow::singleton("Clock")),
    ]
    .into();

    let boot = EffectDecl::new("boot")
        .with_callee("read_config")
        .with_callee("now");
    let rows = infer_all(&seed, &[boot]);

    let expected =
        EffectRow::from_effects(["FS".to_string(), "Clock".to_string()]);
    assert_eq!(
        rows["boot"],
        expected,
        "inferred row must be [Clock, FS] (join); [FS] or [Clock] alone is a bug"
    );
}

/// `surface/effects/eff-undeclared-escapes-rejected` (oracle) — **escape guard**
///
/// `logged` declares `visits [Console]`; its body calls `greet` (Console) and
/// `now` (Clock), so `ρ_inf = {Console, Clock}`. `Clock ∉ ρ_decl` → escape
/// error naming `Clock` with witness `now`.
///
/// This is the **single soundness-relevant gate** of the row system (§1.4).
/// Verdict FLIPS against `eff-declared-matches-used-accepted` below.
#[test]
fn eff_undeclared_escapes_rejected() {
    let seed: HashMap<String, EffectRow> = [
        ("greet".to_string(), EffectRow::singleton("Console")),
        ("now".to_string(), EffectRow::singleton("Clock")),
    ]
    .into();

    let mut witnesses = WitnessMap::new();
    witnesses.insert("Console".to_string(), "greet".to_string());
    witnesses.insert("Clock".to_string(), "now".to_string());

    let logged = EffectDecl::new("logged")
        .with_declared_row(EffectRow::singleton("Console"))
        .with_callee("greet")
        .with_callee("now");

    let rows = infer_all(&seed, &[logged.clone()]);
    let inferred = &rows["logged"];

    // Inferred must be {Console, Clock}
    assert!(
        inferred.contains("Console") && inferred.contains("Clock"),
        "inferred row must include both Console and Clock"
    );

    // Escape check: Clock escapes
    let err = check_escape(&logged, inferred, &witnesses)
        .expect_err("Clock escapes declared [Console] — must reject");
    match err {
        EffectError::EffectEscapes { witnesses: ws, .. } => {
            let escaping_effects: Vec<&EffectName> =
                ws.iter().map(|(e, _)| e).collect();
            assert!(
                escaping_effects.iter().any(|e| *e == "Clock"),
                "error must name Clock as escaping"
            );
            // The witness must be the `now` call site
            let clock_witness = ws
                .iter()
                .find(|(e, _)| e == "Clock")
                .map(|(_, site)| site.as_str());
            assert_eq!(
                clock_witness,
                Some("now"),
                "Clock witness must be 'now'"
            );
        }
        other => panic!("expected EffectEscapes, got {:?}", other),
    }
}

/// `surface/effects/eff-declared-matches-used-accepted` (oracle)
///
/// Same body as `eff-undeclared-escapes-rejected`, but declares
/// `visits [Console, Clock]`. Accepts — makes the previous case discriminating.
/// Verdict FLIPS: both declared → accept; one missing → reject.
#[test]
fn eff_declared_matches_used_accepted() {
    let seed: HashMap<String, EffectRow> = [
        ("greet".to_string(), EffectRow::singleton("Console")),
        ("now".to_string(), EffectRow::singleton("Clock")),
    ]
    .into();

    let logged = EffectDecl::new("logged")
        .with_declared_row(EffectRow::from_effects([
            "Console".to_string(),
            "Clock".to_string(),
        ]))
        .with_callee("greet")
        .with_callee("now");

    let rows = infer_all(&seed, &[logged.clone()]);
    let inferred = &rows["logged"];

    check_escape(&logged, inferred, &WitnessMap::new())
        .expect("Console+Clock declared — must accept");
}

/// `surface/effects/eff-overdeclared-upper-bound-accepted` (oracle)
///
/// Body uses `{Console, Clock}`; declares `visits [Console, Clock, Net]`.
/// `ρ_inf ⊆ ρ_decl` (⊆, not =) → accepts. Flips against a bug that checks
/// exact equality: that bug would reject this legal over-declaration.
#[test]
fn eff_overdeclared_upper_bound_accepted() {
    let seed: HashMap<String, EffectRow> = [
        ("greet".to_string(), EffectRow::singleton("Console")),
        ("now".to_string(), EffectRow::singleton("Clock")),
    ]
    .into();

    let logged = EffectDecl::new("logged")
        .with_declared_row(EffectRow::from_effects([
            "Console".to_string(),
            "Clock".to_string(),
            "Net".to_string(),
        ]))
        .with_callee("greet")
        .with_callee("now");

    let rows = infer_all(&seed, &[logged.clone()]);
    let inferred = &rows["logged"];

    // ρ_inf = {Console, Clock} ⊆ {Console, Clock, Net} = ρ_decl → Ok
    check_escape(&logged, inferred, &WitnessMap::new())
        .expect("overdeclared [Console, Clock, Net] must accept (⊆, not =)");
}

/// `surface/effects/eff-pure-default-is-effect-free` (oracle)
///
/// `double` has no effectful calls and no row annotation; inferred row = ∅.
/// A bug that infers a spurious effect for pure code is caught by the asserted
/// empty row.
#[test]
fn eff_pure_default_is_effect_free() {
    let seed: HashMap<String, EffectRow> = HashMap::new();
    let double = EffectDecl::new("double");
    let rows = infer_all(&seed, &[double.clone()]);

    assert!(
        rows["double"].is_empty(),
        "pure view with no calls must have inferred row ∅"
    );

    // Escape check with ρ_decl = ∅ (no annotation) also passes
    check_escape(&double, &rows["double"], &WitnessMap::new())
        .expect("pure view: ∅ ⊆ ∅ must accept");
}

// ============================================================
// EFF2 — pure ITree denotation (K1.5-DEFERRED)
// ============================================================
//
// EFF2 cases (`eff-denotes-to-interaction-tree`, `eff-bind-is-tree-grafting`,
// `eff-kernel-checks-denotation-pure`, `eff-itree-level-forced`) require the
// `ITree` inductive, which is gated on K1.5 (`check_no_pi_bound_recursive`
// currently rejects the Π-bound W-style `Vis` argument, §7.0). These cases
// are noted here and will be added when K1.5 lands.

// ============================================================
// EFF3 — capabilities gate effectful ops
// ============================================================

/// `surface/effects/cap-op-without-token-rejected` (oracle)
///
/// `dump` performs `FS` but has no `Cap FS` in scope → `MissingCapability`.
/// The **denial path** (§2.5, §7.3.2).
#[test]
fn cap_op_without_token_rejected() {
    let dump = EffectDecl::new("dump")
        // declared visits [FS] — but no cap param
        .with_declared_row(EffectRow::singleton("FS"))
        .with_direct_effect("FS");

    let performed = EffectRow::singleton("FS");
    let err = check_capabilities_no_handler(&dump, &performed)
        .expect_err("no Cap FS in scope — must reject");

    match err {
        EffectError::MissingCapability { effect, .. } => {
            assert_eq!(effect, "FS", "error must name FS as missing cap");
        }
        other => panic!("expected MissingCapability, got {:?}", other),
    }
}

/// `surface/effects/cap-op-with-token-accepted` (oracle)
///
/// Same op, but `dump` takes `using fs : FsCap` → accepts.
/// Verdict FLIPS: with the token accepts, without rejects.
#[test]
fn cap_op_with_token_accepted() {
    let dump = EffectDecl::new("dump")
        .with_declared_row(EffectRow::singleton("FS"))
        .with_cap_param(CapParam::new("fs", "FS"))
        .with_direct_effect("FS");

    let performed = EffectRow::singleton("FS");
    check_capabilities_no_handler(&dump, &performed)
        .expect("Cap FS in scope via using-param — must accept");
}

/// `surface/effects/cap-two-distinct-caps-each-gated` (oracle)
///
/// `exfil` performs `FS` and `Net`; three variants:
/// (a) both caps → accept; (b) only `fs` → reject `NetCap`;
/// (c) only `net` → reject `FsCap`. ≥2 distinct caps, each independently
/// gated. A bug checking only the first cap misses (c); only the last misses (b).
#[test]
fn cap_two_distinct_caps_each_gated() {
    let performed =
        EffectRow::from_effects(["FS".to_string(), "Net".to_string()]);

    // (a) both caps in scope → accept
    {
        let exfil = EffectDecl::new("exfil")
            .with_cap_params(vec![
                CapParam::new("fs", "FS"),
                CapParam::new("net", "Net"),
            ])
            .with_direct_effect("FS")
            .with_direct_effect("Net");
        check_capabilities_no_handler(&exfil, &performed)
            .expect("(a) both caps — must accept");
    }

    // (b) only `fs` in scope → rejects MissingCapability(Net)
    {
        let exfil = EffectDecl::new("exfil")
            .with_cap_param(CapParam::new("fs", "FS"))
            .with_direct_effect("FS")
            .with_direct_effect("Net");
        let err = check_capabilities_no_handler(&exfil, &performed)
            .expect_err("(b) missing Net cap — must reject");
        match err {
            EffectError::MissingCapability { effect, .. } => {
                assert_eq!(effect, "Net", "(b) must name Net as missing");
            }
            other => panic!("(b) expected MissingCapability, got {:?}", other),
        }
    }

    // (c) only `net` in scope → rejects MissingCapability(FS)
    {
        let exfil = EffectDecl::new("exfil")
            .with_cap_param(CapParam::new("net", "Net"))
            .with_direct_effect("FS")
            .with_direct_effect("Net");
        let err = check_capabilities_no_handler(&exfil, &performed)
            .expect_err("(c) missing FS cap — must reject");
        match err {
            EffectError::MissingCapability { effect, .. } => {
                assert_eq!(effect, "FS", "(c) must name FS as missing");
            }
            other => panic!("(c) expected MissingCapability, got {:?}", other),
        }
    }
}

// ============================================================
// EFF4 — `space` state + tail-resumptive handlers (type-level)
// ============================================================
//
// The RUNTIME execution of `runState` (asserting final-state value `(2,2)`)
// requires the `ITree` denotation and is K1.5-deferred. The K1-buildable
// assertions are the **static row-level** properties:
// - `becomes` desugars to a `State S` effect (the row contains `Counter`)
// - `runState` eliminates `State S` from the row (type-level plumbing)
// - Cross-space aliasing is rejected (shared-nothing, §4.4)
// - Multi-shot handlers are rejected (OQ-9, §5.2)

/// `surface/effects/space-becomes-threads-state` (type-level, oracle)
///
/// `inc` and `get` each carry row `[Counter]` (the `State Counter` effect).
/// A compound expression calling both also has row `[Counter]`.
/// `runState` eliminates `Counter` from the row → the residual row is ∅.
///
/// (The runtime result assertion `(2, 2)` is K1.5-deferred — requires ITree.)
#[test]
fn space_becomes_threads_state_type_level() {
    // `inc` and `get` have row [Counter] (space op → State Counter effect).
    let seed: HashMap<String, EffectRow> = [
        ("inc".to_string(), EffectRow::singleton("Counter")),
        ("get".to_string(), EffectRow::singleton("Counter")),
    ]
    .into();

    // A compound body `{ inc() ; inc() ; get() }` calls all three.
    let body = EffectDecl::new("body")
        .with_callee("inc")
        .with_callee("inc")  // same callee twice — join is idempotent
        .with_callee("get");
    let rows = infer_all(&seed, &[body]);

    assert_eq!(
        rows["body"],
        EffectRow::singleton("Counter"),
        "body row must be [Counter] (union of [Counter] × 3 = [Counter])"
    );

    // runState eliminates Counter → residual row = ∅.
    // Type-level: `runState` discharges the State effect (§4.2 row plumbing).
    let before_runstate = rows["body"].clone();
    let discharged = EffectRow::singleton("Counter");
    let after_runstate = before_runstate.minus(&discharged);
    assert!(
        after_runstate.is_empty(),
        "runState must discharge Counter — residual row must be ∅"
    );
}

/// `surface/effects/space-old-scoped-to-ensures` (type-level, oracle)
///
/// `inc` carries row `[Counter]` and an ensures `n == old(n) + 1`.
/// The K1-buildable assertion: the row analysis sees this as a `State Counter`
/// effect and the ensures predicate has the correct structure (pre/post).
///
/// The proof that `s.n+1 == s.n+1` closes by `refl` is verified by the
/// kernel (not this pass); we assert only the row and effect presence here.
#[test]
fn space_old_scoped_to_ensures_type_level() {
    let seed: HashMap<String, EffectRow> = [(
        "inc".to_string(),
        EffectRow::singleton("Counter"),
    )]
    .into();

    let inc = EffectDecl::new("inc")
        .with_declared_row(EffectRow::singleton("Counter"))
        .with_direct_effect("Counter");
    let rows = infer_all(&seed, &[inc.clone()]);

    // Row is [Counter]
    assert_eq!(rows["inc"], EffectRow::singleton("Counter"));

    // Escape check passes (Counter declared and used)
    check_escape(&inc, &rows["inc"], &WitnessMap::new())
        .expect("inc declares and uses [Counter] — must accept");
}

/// `surface/effects/space-shared-nothing-no-cross-space-alias` (oracle)
///
/// (a) Space A directly reads/writes space B's `mut` cell → `CrossSpaceAlias`
///     (rejected, static error).
/// (b) Space A sends an immutable value to space B by message-passing → accepts.
///
/// Verdict FLIPS: (a) rejects, (b) accepts. A bug permitting cross-space
/// aliasing breaks shared-nothing isolation (§4.4) silently — each State type
/// is still well-typed; only this check catches it.
#[test]
fn space_shared_nothing_no_cross_space_alias() {
    // (a) direct aliasing → reject
    let accesses = vec![CrossSpaceAccess {
        from_space: "A".to_string(),
        to_space: "B".to_string(),
    }];
    let err = check_cross_space(&accesses)
        .expect_err("(a) direct cross-space alias must be rejected");
    match err {
        EffectError::CrossSpaceAlias { target_space, .. } => {
            assert_eq!(target_space, "B", "(a) target space must be B");
        }
        other => panic!("(a) expected CrossSpaceAlias, got {:?}", other),
    }

    // (b) message-passing (no direct cell access — A accesses only its own cells)
    let no_alias: Vec<CrossSpaceAccess> = vec![
        CrossSpaceAccess { from_space: "A".to_string(), to_space: "A".to_string() },
        CrossSpaceAccess { from_space: "B".to_string(), to_space: "B".to_string() },
    ];
    check_cross_space(&no_alias)
        .expect("(b) message-passing (own-space accesses only) must accept");
}

/// `surface/effects/handler-tail-resumptive-folds` (oracle)
///
/// (a) tail-resumptive handler (resumes once, in tail position) → accepts.
/// (b) multi-shot handler → `NonTailResumptive`.
/// Verdict FLIPS. OQ-9 exclusion guard (§5.2, §7.3.3).
#[test]
fn handler_tail_resumptive_folds() {
    // (a) tail-resumptive — accepts
    check_tail_resumptive("console_handler", ResumeKind::TailOnce)
        .expect("tail-resumptive handler must accept");

    // (b) multi-shot — rejects
    let err = check_tail_resumptive("bad_handler", ResumeKind::MultiShot)
        .expect_err("multi-shot handler must be rejected");
    assert!(
        matches!(err, EffectError::NonTailResumptive { .. }),
        "expected NonTailResumptive, got {:?}",
        err
    );
}

/// `surface/effects/handler-multishot-rejected` (oracle)
///
/// Non-tail-position resume → `NonTailResumptive`. Separate from the
/// multi-shot case — both are excluded by OQ-9 (§5.2).
#[test]
fn handler_multishot_rejected() {
    let err = check_tail_resumptive("nontail_handler", ResumeKind::NonTail)
        .expect_err("non-tail-position resume must be rejected");
    assert!(
        matches!(err, EffectError::NonTailResumptive { .. }),
        "expected NonTailResumptive, got {:?}",
        err
    );

    // Stop (no resume) is also permitted
    check_tail_resumptive("stop_handler", ResumeKind::Stop)
        .expect("no-resume (stop) handler must accept");
}

// ============================================================
// EFF5 — pure/impure boundary hook for L7 FFI
// ============================================================

/// `surface/effects/pure-view-usable-in-pure-context` (oracle)
///
/// `double` has inferred row ∅; it is usable where a pure function is
/// required (empty row is the certificate). The collapse `ITree 𝟘 R ≅ R` is
/// K1.5-deferred; the K1-buildable assertion is: inferred row is ∅ and
/// check_escape passes for `ρ_decl = ∅`.
#[test]
fn pure_view_usable_in_pure_context() {
    let seed = HashMap::new();
    let double = EffectDecl::new("double");
    let rows = infer_all(&seed, &[double.clone()]);

    assert!(rows["double"].is_empty(), "double: inferred row must be ∅");
    check_escape(&double, &rows["double"], &WitnessMap::new())
        .expect("pure view: ∅ ⊆ ∅ — must accept");
}

/// `surface/effects/impure-boundary-marker-exposed` (property, oracle)
///
/// A `foreign` with a non-empty row (`visits [Clock]`) has a non-empty
/// inferred row; a caller inherits `Clock` transitively (§1.2).
/// L5 pins that the **non-empty row is visible in the type** (the impure
/// marker, §7.2); L7 plugs in the interpreter. The assertion here is the
/// type-level propagation.
#[test]
fn impure_boundary_marker_exposed() {
    // `read_clock` is a foreign with row [Clock]
    let seed: HashMap<String, EffectRow> = [(
        "read_clock".to_string(),
        EffectRow::singleton("Clock"),
    )]
    .into();

    // A caller inherits Clock transitively
    let caller = EffectDecl::new("caller").with_callee("read_clock");
    let rows = infer_all(&seed, &[caller]);

    assert!(
        rows["caller"].contains("Clock"),
        "caller of impure foreign inherits Clock in its row"
    );
    assert!(
        !rows["caller"].is_empty(),
        "non-empty row is the impure marker (§7.2)"
    );
}

/// `surface/effects/impure-masquerading-as-pure-rejected` (oracle)
///
/// `safe` declares no row (ρ_decl = ∅, claims purity) but calls `read_clock`
/// (Clock). `ρ_inf = {Clock} ⊄ ∅` → `EffectEscapes(Clock)`.
///
/// Verdict FLIPS against `impure-boundary-marker-exposed` (which declares
/// [Clock] and accepts). Integrity of the pure/impure boundary (§7.2, §1.4).
#[test]
fn impure_masquerading_as_pure_rejected() {
    let seed: HashMap<String, EffectRow> = [(
        "read_clock".to_string(),
        EffectRow::singleton("Clock"),
    )]
    .into();

    let mut witnesses = WitnessMap::new();
    witnesses.insert("Clock".to_string(), "read_clock".to_string());

    // `safe` claims purity (no declared row) but calls impure `read_clock`
    let safe = EffectDecl::new("safe").with_callee("read_clock");
    // No declared_row → ρ_decl = ∅

    let rows = infer_all(&seed, &[safe.clone()]);
    assert!(
        rows["safe"].contains("Clock"),
        "inferred row must contain Clock"
    );

    let err = check_escape(&safe, &rows["safe"], &witnesses)
        .expect_err("Clock escapes pure declaration — must reject");
    match err {
        EffectError::EffectEscapes { witnesses: ws, .. } => {
            assert!(
                ws.iter().any(|(e, _)| e == "Clock"),
                "EffectEscapes must name Clock"
            );
        }
        other => panic!("expected EffectEscapes, got {:?}", other),
    }
}

// ============================================================
// Higher-order row release guard (Architect gap — §1.2 `f a` clause)
// ============================================================

/// `surface/effects/higher-order-param-undeclared-rejected` (oracle)
///
/// **The `apply_twice` gap.** A function `apply_twice (f : A →[ρ] A) (x : A) :
/// A = f (f x)` has a higher-order parameter `f` with a latent row variable
/// `ρ`. First-order `infer_row` cannot resolve `ρ` — it infers ∅, so the §1.4
/// escape check would silently pass even if the caller observes effects.
///
/// **Guard:** `check_higher_order_guard` requires the declared row to cover any
/// candidate effects from unknown higher-order parameters. Two cases:
/// (a) undeclared → `EffectEscapes` naming the candidate (reject).
/// (b) declared row covers the candidate → accept.
/// Verdict **flips**: (a) rejects, (b) accepts.
#[test]
fn higher_order_param_undeclared_rejected() {
    // (a) `apply_twice` with unknown param that may release `FS` —
    //     no declared row → guard fires.
    let apply_twice = EffectDecl::new("apply_twice")
        .with_unknown_param_effect("FS"); // f : A →[FS] A

    let err = check_higher_order_guard(&apply_twice)
        .expect_err("(a) higher-order FS param undeclared — guard must reject");
    match err {
        EffectError::EffectEscapes { witnesses, .. } => {
            assert!(
                witnesses.iter().any(|(e, _)| e == "FS"),
                "(a) EffectEscapes must name FS; got {:?}",
                witnesses
            );
            // Witness must identify it came from a higher-order param
            let site = witnesses
                .iter()
                .find(|(e, _)| e == "FS")
                .map(|(_, s)| s.as_str());
            assert_eq!(
                site,
                Some("<higher-order-param>"),
                "(a) witness site must be <higher-order-param>"
            );
        }
        other => panic!("(a) expected EffectEscapes, got {:?}", other),
    }
}

/// `surface/effects/higher-order-param-declared-accepted` (oracle)
///
/// Same `apply_twice` but declares `visits [FS]` — the guard sees that the
/// candidate `FS` is covered by `ρ_decl` and accepts.
/// Verdict FLIPS against `higher-order-param-undeclared-rejected`.
#[test]
fn higher_order_param_declared_accepted() {
    let apply_twice = EffectDecl::new("apply_twice")
        .with_declared_row(EffectRow::singleton("FS"))
        .with_unknown_param_effect("FS"); // f : A →[FS] A

    check_higher_order_guard(&apply_twice)
        .expect("(b) declared visits [FS] covers the FS param — guard must accept");
}

/// `surface/effects/higher-order-two-params-each-guarded` (oracle)
///
/// Two higher-order params with distinct candidate effects. Three variants:
/// (a) both declared → accept; (b) only FS declared → guard fires on Net;
/// (c) only Net declared → guard fires on FS.
/// Each candidate is guarded independently.
#[test]
fn higher_order_two_params_each_guarded() {
    // (a) both declared → accept
    {
        let f = EffectDecl::new("f")
            .with_declared_row(EffectRow::from_effects([
                "FS".to_string(),
                "Net".to_string(),
            ]))
            .with_unknown_param_effect("FS")
            .with_unknown_param_effect("Net");
        check_higher_order_guard(&f)
            .expect("(a) both declared — must accept");
    }

    // (b) only FS declared → rejects Net
    {
        let f = EffectDecl::new("f")
            .with_declared_row(EffectRow::singleton("FS"))
            .with_unknown_param_effect("FS")
            .with_unknown_param_effect("Net");
        let err = check_higher_order_guard(&f)
            .expect_err("(b) missing Net declaration — must reject");
        match err {
            EffectError::EffectEscapes { witnesses, .. } => {
                assert!(
                    witnesses.iter().any(|(e, _)| e == "Net"),
                    "(b) must name Net; got {:?}", witnesses
                );
                assert!(
                    !witnesses.iter().any(|(e, _)| e == "FS"),
                    "(b) FS is declared — must not appear in escaping set"
                );
            }
            other => panic!("(b) expected EffectEscapes, got {:?}", other),
        }
    }

    // (c) only Net declared → rejects FS
    {
        let f = EffectDecl::new("f")
            .with_declared_row(EffectRow::singleton("Net"))
            .with_unknown_param_effect("FS")
            .with_unknown_param_effect("Net");
        let err = check_higher_order_guard(&f)
            .expect_err("(c) missing FS declaration — must reject");
        match err {
            EffectError::EffectEscapes { witnesses, .. } => {
                assert!(
                    witnesses.iter().any(|(e, _)| e == "FS"),
                    "(c) must name FS; got {:?}", witnesses
                );
            }
            other => panic!("(c) expected EffectEscapes, got {:?}", other),
        }
    }
}

/// First-order callee still inferred correctly alongside the higher-order guard.
///
/// A function with both a named callee (first-order, row known) and a
/// higher-order param (row unknown): `infer_row` picks up the named callee's
/// row; the guard then checks the param candidate against the declared row.
#[test]
fn higher_order_guard_coexists_with_first_order_callee() {
    let seed: HashMap<String, EffectRow> = [(
        "log".to_string(),
        EffectRow::singleton("Console"),
    )]
    .into();

    // `audit (f : A →[FS] A)` calls `log` (Console) and has an FS param.
    // declares `visits [Console, FS]` — both covered.
    let audit = EffectDecl::new("audit")
        .with_declared_row(EffectRow::from_effects([
            "Console".to_string(),
            "FS".to_string(),
        ]))
        .with_callee("log")
        .with_unknown_param_effect("FS");

    let rows = infer_all(&seed, &[audit.clone()]);

    // First-order: inferred row picks up Console from `log`
    assert!(
        rows["audit"].contains("Console"),
        "inferred row must include Console from callee `log`"
    );

    // Guard: FS param is covered by declared row → accepts
    check_higher_order_guard(&audit)
        .expect("both named callee (Console) and FS param declared — must accept");

    // Escape check passes too
    check_escape(&audit, &rows["audit"], &WitnessMap::new())
        .expect("Console inferred ⊆ [Console, FS] declared — must accept");
}

// ============================================================
// EFF2 — ITree denotation structure (K1.5 gate lifted, now runnable)
// ============================================================

/// `surface/effects/itree-ret-is-pure` (oracle)
///
/// `Ret r` is the pure-value constructor — no `Vis` nodes, no effects.
/// Structural assertion: `is_ret()`, `ret_value() == Some(r)`.
#[test]
fn itree_ret_is_pure() {
    let t = ITree::ret(42);
    assert!(t.is_ret(), "Ret must be identified as a Ret node");
    assert!(!t.is_vis(), "Ret must not be a Vis node");
    assert_eq!(t.ret_value(), Some(42), "Ret value must be recoverable");
}

/// `surface/effects/itree-perform-creates-vis` (oracle)
///
/// `perform e = Vis e (λr. Ret r)` (§2.2). Structural assertion: `is_vis()`,
/// `effect_name() == Some("FS")`, continuation at any response produces `Ret`.
/// Verdict FLIPS: `Ret 0` is NOT a `Vis` node (correct/buggy both return
/// something, but only Vis has an effect name — discriminating property).
#[test]
fn itree_perform_creates_vis_node() {
    let t = perform("FS");
    // Positive: correct
    assert!(t.is_vis(), "perform must produce a Vis node");
    assert_eq!(t.effect_name(), Some(&"FS".to_string()));

    // Structural: continuation maps any response to Ret r
    let cont_0 = t.apply_cont(0).unwrap();
    assert!(cont_0.is_ret(), "continuation at 0 must yield Ret");
    assert_eq!(cont_0.ret_value(), Some(0));
    let cont_99 = t.apply_cont(99).unwrap();
    assert_eq!(cont_99.ret_value(), Some(99));

    // Verdict flip: Ret is not a Vis (a buggy impl that returned Ret would fail this)
    let ret_t = ITree::ret(0);
    assert!(!ret_t.is_vis(), "Ret is not a Vis — verdict flips vs perform");
}

/// `surface/effects/itree-bind-ret-left-unit` (oracle)
///
/// `bind (Ret a) f = f a` (§2.2, left unit). Structural: the result is exactly
/// `f(a)`. Verdict FLIPS: `bind (Vis e k) f` is NOT `f a` (it's `Vis e …`).
#[test]
fn itree_bind_ret_left_unit() {
    let t = ITree::ret(5);
    let result = bind(t, Rc::new(|v| ITree::ret(v * 2)));
    assert_eq!(result.ret_value(), Some(10), "bind(Ret 5)(λv.Ret(v*2)) = Ret 10");

    // Verdict flip: bind of a Vis node gives Vis, not Ret v*2
    let vis_t = perform("FS");
    let vis_result = bind(vis_t, Rc::new(|v| ITree::ret(v * 2)));
    assert!(vis_result.is_vis(), "bind(Vis e k)(f) must give Vis, not Ret — flip");
}

/// `surface/effects/itree-bind-vis-distributes` (oracle)
///
/// `bind (Vis e k) f = Vis e (λr. bind (k r) f)` (§2.2). Structural:
/// (a) result is a Vis with same effect name.
/// (b) `(result.cont)(r)` = `bind (k r) f` — one more fold step.
/// Verdict FLIPS: `bind (Ret 5) f` is NOT a Vis node.
#[test]
fn itree_bind_vis_distributes() {
    // `perform "FS"` = Vis "FS" (λr. Ret r).
    let t = perform("FS");
    let f: Rc<dyn Fn(i64) -> ITree> = Rc::new(|v: i64| ITree::ret(v + 1));
    let result = bind(t, Rc::clone(&f));

    // (a) result is Vis "FS"
    assert!(result.is_vis(), "bind(Vis …)(f) must be a Vis node");
    assert_eq!(result.effect_name(), Some(&"FS".to_string()));

    // (b) applying the continuation: cont(7) = bind((Ret 7))(λv.Ret v+1) = Ret 8
    let inner = result.apply_cont(7).unwrap();
    assert_eq!(inner.ret_value(), Some(8),
        "cont(7) must be bind(Ret 7)(λv.Ret(v+1)) = Ret 8");

    // Verdict flip: bind(Ret 5)(f) is NOT a Vis
    let bind_ret = bind(ITree::ret(5), Rc::clone(&f));
    assert!(!bind_ret.is_vis(), "bind(Ret …)(f) is Ret — flips vs bind(Vis …)(f)");
}

/// `surface/effects/handler-fold-discharges-effect` (oracle)
///
/// A tail-resumptive handler for `FS` applied to `perform "FS"`:
/// `Vis "FS" (λr. Ret r)` handled by FS (response 42) → `Ret 42`.
/// Verdict FLIPS: without the handler the `Vis` node remains unhandled.
#[test]
fn handler_fold_discharges_effect() {
    let t = perform("FS");
    let cases: Rc<[HandlerCase]> = vec![HandlerCase::new("FS", 42)].into();
    let result = handler_fold(t, cases);

    // Handler fires: Vis "FS" k → k(42) = Ret 42
    assert!(result.is_ret(), "FS handled by response 42 → Ret 42");
    assert_eq!(result.ret_value(), Some(42));
}

/// `surface/effects/handler-fold-passes-through-unhandled` (oracle)
///
/// A handler for `FS` applied to `perform "Net"` — `Net` is unhandled.
/// The `Vis "Net"` node passes through unchanged. Verdict FLIPS: a handler
/// that handles `FS` should KEEP `Net` nodes (not silently consume them).
#[test]
fn handler_fold_passes_through_unhandled() {
    let t = perform("Net");
    let cases: Rc<[HandlerCase]> = vec![HandlerCase::new("FS", 0)].into();
    let result = handler_fold(t, cases);

    assert!(result.is_vis(), "Net is unhandled — Vis node must pass through");
    assert_eq!(result.effect_name(), Some(&"Net".to_string()),
        "unhandled Vis must preserve the original effect name");
}

/// `surface/effects/handler-fold-tail-resumptive` (oracle)
///
/// Two sequential effects: `bind (perform "FS") (λ_. perform "FS")`.
/// Handler fires twice: both FS nodes consumed → `Ret 7`.
/// Tests that the fold recurses into the continuation (tail position, §5.2).
#[test]
fn handler_fold_tail_resumptive_chains() {
    // bind (Vis "FS" (λr. Ret r)) (λ_. Vis "FS" (λr. Ret r))
    // = Vis "FS" (λr. bind (Ret r) (λ_. perform "FS"))
    // = Vis "FS" (λr. perform "FS")  — by left-unit
    let t = bind(perform("FS"), Rc::new(|_| perform("FS")));

    let cases: Rc<[HandlerCase]> = vec![HandlerCase::new("FS", 7)].into();
    let result = handler_fold(t, cases);

    // Both FS nodes consumed, continuation responds with 7
    assert!(result.is_ret(),
        "both FS nodes folded → final Ret");
    assert_eq!(result.ret_value(), Some(7));
}

// ============================================================
// Row-polymorphism — infer_row_poly + check_row_poly_escape
// ============================================================

/// `surface/effects/row-poly-apply-twice-infers-row-var` (oracle)
///
/// `apply_twice (f : A →[ρ₀] A) : A →[ρ₀] A` — the canonical higher-order
/// row-poly example. `infer_row_poly` should return `RowType::Var(ρ₀)` (not
/// ∅ as the conservative guard approximated). The escape check `ρ₀ ⊆ ρ₀` passes.
/// Verdict FLIPS: no declared row → escape check rejects `ρ₀` (variant below).
#[test]
fn row_poly_apply_twice_infers_row_var() {
    let decl = EffectDecl::new("apply_twice")
        .with_param_row(RowVar(0))
        .with_declared_row_type(RowType::Var(RowVar(0)));

    let inferred = infer_row_poly(
        &HashMap::new(),
        &decl.direct_effects,
        &decl.callees,
        &decl.param_rows,
    );
    assert_eq!(inferred, RowType::Var(RowVar(0)),
        "apply_twice must infer the row variable of its param, not ∅");

    check_row_poly_escape(
        &decl.name,
        &inferred,
        decl.declared_row_type.as_ref(),
        decl.declared_row.as_ref(),
    )
    .expect("ρ₀ ⊆ ρ₀ — apply_twice with matching declared row var must accept");
}

/// `surface/effects/row-poly-undeclared-row-var-rejected` (oracle)
///
/// Same `apply_twice` but without `declared_row_type` (defaults to ∅).
/// The inferred row is `Var(ρ₀)` but declared row is ∅ → escape check fails.
/// Verdict FLIPS against `row_poly_apply_twice_infers_row_var` above.
#[test]
fn row_poly_undeclared_row_var_rejected() {
    let decl = EffectDecl::new("apply_twice_bad")
        .with_param_row(RowVar(0));
    // No declared_row_type → defaults to Concrete(∅)

    let inferred = infer_row_poly(
        &HashMap::new(),
        &decl.direct_effects,
        &decl.callees,
        &decl.param_rows,
    );
    assert_eq!(inferred, RowType::Var(RowVar(0)));

    let err = check_row_poly_escape(
        &decl.name,
        &inferred,
        decl.declared_row_type.as_ref(),
        decl.declared_row.as_ref(),
    )
    .expect_err("ρ₀ not covered by declared ∅ — must reject");

    assert!(matches!(err, EffectError::EffectEscapes { .. }),
        "expected EffectEscapes, got {:?}", err);
}

/// `surface/effects/row-poly-concrete-caller-substitution` (oracle)
///
/// At a call site, the row variable is substituted with the concrete row of
/// the supplied argument: `apply_twice(read_config)` where `read_config : FS`.
/// After `RowVar(0) → Concrete({FS})`, inferred = declared = `{FS}`.
#[test]
fn row_poly_concrete_caller_substitution() {
    // ρ₀ → FS (caller supplies a concrete FS-effect function)
    let subst: RowSubst = [(RowVar(0), RowType::singleton("FS"))].into();

    let inferred = RowType::Var(RowVar(0)).apply_subst(&subst);
    let declared = RowType::Var(RowVar(0)).apply_subst(&subst);

    assert_eq!(inferred, RowType::concrete(EffectRow::singleton("FS")));
    assert!(inferred.is_subset_of(&declared),
        "after substitution FS ⊆ FS — caller with concrete arg passes");
}

/// `surface/effects/row-poly-pure-caller-substitution` (oracle)
///
/// `apply_twice(id)` where `id : A → A` (pure, row ∅). After substitution
/// `ρ₀ → ∅`, the whole expression is pure. Callers with a pure function
/// get a pure `apply_twice`.
#[test]
fn row_poly_pure_caller_is_pure() {
    let subst: RowSubst = [(RowVar(0), RowType::empty())].into();

    let inferred = RowType::Var(RowVar(0)).apply_subst(&subst);
    assert_eq!(inferred, RowType::empty(),
        "apply_twice(id) with pure id → row ∅");
    assert!(inferred.is_subset_of(&RowType::empty()),
        "pure inferred row ⊆ ∅ declared — accepts");
}

/// `surface/effects/row-poly-two-params-each-tracked` (oracle)
///
/// A function with two higher-order parameters: `compose (f : A →[ρ₀] B)
/// (g : B →[ρ₁] C) : A →[ρ₀ ⊕ ρ₁] C`. The inferred row is
/// `Join(Var(ρ₀), Var(ρ₁))`; declared row is the same join.
/// Three variants — both declared accepts; only ρ₀ declared rejects ρ₁.
#[test]
fn row_poly_two_params_each_tracked() {
    let inferred = infer_row_poly(
        &HashMap::new(),
        &[],   // no direct effects
        &[],   // no named callees
        &[RowVar(0), RowVar(1)],
    );
    // inferred = Join(Var(0), Var(1))
    assert!(inferred.row_vars().contains(&RowVar(0)));
    assert!(inferred.row_vars().contains(&RowVar(1)));

    // (a) declared = Join(Var(0), Var(1)) → accepts
    {
        let declared = RowType::Var(RowVar(0)).join(RowType::Var(RowVar(1)));
        assert!(inferred.is_subset_of(&declared),
            "(a) both row vars declared — must accept");
    }

    // (b) declared = Var(0) only → rejects ρ₁
    {
        let declared = RowType::Var(RowVar(0));
        assert!(!inferred.is_subset_of(&declared),
            "(b) only ρ₀ declared — ρ₁ escapes, must reject");
        let result = check_row_poly_escape(
            "compose_bad",
            &inferred,
            Some(&declared),
            None,
        );
        assert!(result.is_err(), "(b) ρ₁ not covered — check_row_poly_escape must reject");
    }
}

/// `surface/effects/row-poly-mixed-concrete-and-var` (oracle)
///
/// A function with a named callee (concrete row) and a higher-order param
/// (row var): `foo (f : A →[ρ₀] A)` calls `log : [Console]` and has param `f`.
/// Inferred row = `Join(Concrete({Console}), Var(ρ₀))`.
/// Declared `[Console, ρ₀]` (concrete + same var) → accepts.
#[test]
fn row_poly_mixed_concrete_and_var() {
    let env: HashMap<String, EffectRow> = [
        ("log".to_string(), EffectRow::singleton("Console")),
    ].into();

    let inferred = infer_row_poly(
        &env,
        &[],
        &["log".to_string()],
        &[RowVar(0)],
    );
    // Concrete Console + Var(0)
    assert!(inferred.concrete_effects().contains("Console"));
    assert!(inferred.row_vars().contains(&RowVar(0)));

    // Declared = Concrete({Console}) join Var(0) → accepts
    let declared = RowType::concrete(EffectRow::singleton("Console"))
        .join(RowType::Var(RowVar(0)));
    assert!(inferred.is_subset_of(&declared),
        "mixed concrete+var declared correctly — must accept");
}

// ============================================================
// Regression — existing elaboration invariants still green
// ============================================================

/// `surface/effects/existing-surface-invariants-still-green` (property)
///
/// The effects module is additive: importing it must not break the V0
/// pipeline. This test verifies that `ken_elaborator::ElabEnv` still works
/// (the V0 pure-elaboration path is untouched).
#[test]
fn existing_surface_invariants_still_green() {
    use ken_elaborator::ElabEnv;
    let mut env = ElabEnv::new().expect("base env failed");
    env.elaborate_decl("view id (A : Type) (x : A) : A = x")
        .expect("id elaboration must still pass after adding effects module");
}
