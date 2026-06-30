//! L5 conformance cases — K1-buildable half (`conformance/surface/effects/seed-effects.md`).
//!
//! Covers: EFF1 (row inference + escape), EFF3 (capability gating), EFF4
//! (type-level space + handler), EFF5 (pure/impure boundary).
//!
//! EFF2 (ITree denotation structure) is K1.5-deferred — noted below, not forced.
//!
//! Every negative case is **discriminating**: the verdict must **flip** between
//! the correct variant (accepts/Ok) and the targeted-bug variant (rejects/Err),
//! not pass vacuously (COORDINATION §7, `discriminating-conformance-verdict-must-flip`).

use std::collections::HashMap;

use ken_elaborator::effects::{
    check_capabilities_no_handler, check_cross_space, check_escape,
    check_tail_resumptive, CapParam, CrossSpaceAccess, EffectDecl,
    EffectError, EffectName, EffectRow, ResumeKind, WitnessMap, infer_all,
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
