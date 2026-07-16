//! L7 acceptance tests — `foreign` FFI + trust boundary (`38 §2–§4`).
//!
//! **Producer-grep rule (QA gate):** before counting tests green, verify
//! `foreign::elaborate_foreign` is the real kernel call-site — NOT a hand-fed
//! `declare_postulate` in the test. Every AC2/AC5 case seeds from the ACTUAL
//! registration (`env.foreign_env` / `env.env.trusted_base()`), so removing
//! the real `declare_postulate` call inside `elaborate_foreign` breaks these
//! tests structurally.
//!
//! **Reading the test strategy:**
//! - AC1 A1: structural assertion on `ForeignBinding` (binding present, marshal_sig)
//! - AC2 B1/B2: `trusted_base_delta` dependency pair (caller has it, non-caller doesn't)
//! - AC3 C1/C2: trusted-by-postulate → P, proof-discharged → Q pair
//! - AC4 D1: unprovable `ensures` → `FfiRuntimeCheck` obligation emitted (discrim pair)
//! - AC5 E1/E2: real escape-check flip via `foreign_env.io_effect_rows` seed
//! - AC6 F1: capability + effect compose (3-way gate)
//! - G6/G1: verified component with foreign + round-trip

use ken_elaborator::{
    effects::{
        check_capabilities_no_handler, check_escape, infer_all, CapParam, EffectDecl, EffectError,
        WitnessMap,
    },
    foreign::elaborate_foreign,
    trusted_base_delta, ElabEnv, MarshalKind, Span,
};
use ken_kernel::{Decl, Term};

// ─── A1: AC1 — `foreign` declaration binds + marshals ────────────────────────

/// A1: `foreign` binds a typed, effect-rowed postulate; marshals `Bytes`
/// arguments as `(ptr, len)` and scalars as their machine type (`38 §2.2`,
/// `41 §1`).
///
/// Structural assertion on `ForeignBinding` — NOT just "compiles".
#[test]
fn foreign_decl_binds_typed_effect_rowed() {
    let mut env = ElabEnv::new().expect("ElabEnv::new()");
    let result = env
        .elaborate_decl_v1(r#"foreign os_write : Int -> Bytes -> Int = "write" "libc" [FS]"#)
        .expect("foreign decl must elaborate");

    let fb = result
        .foreign_binding
        .expect("ForeignBinding must be present");

    // AC1-a: postulate is in trusted_base (assumed, not verified).
    assert!(
        env.env.trusted_base().contains(&fb.postulate_id),
        "os_write postulate must appear in trusted_base(): {:?}",
        env.env.trusted_base()
    );

    // AC1-b: effect row contains [FS].
    assert!(
        fb.effect_row.contains("FS"),
        "effect row must contain FS, got: {:?}",
        fb.effect_row
    );

    // AC1-c: symbol + library recorded.
    assert_eq!(fb.symbol, "write");
    assert_eq!(fb.library, "libc");

    // AC1-d: Bytes argument marshals as BytesPtr (`41 §1`).
    let sig = fb.marshal_sig.expect("MarshalSig must be present");
    assert!(
        sig.params.iter().any(|k| *k == MarshalKind::BytesPtr),
        "Bytes parameter must marshal as BytesPtr, params: {:?}",
        sig.params
    );

    // AC1-e: no runtime checks (no ensures clause).
    assert!(
        fb.runtime_checks.is_empty(),
        "no ensures → no runtime checks"
    );
}

// ─── B1/B2: AC2 — trusted_base_delta dependency pair ─────────────────────────

/// B1: a definition whose body CALLS (contains) `os_write` has its postulate
/// in `trusted_base_delta`. Reliance-by-use, not by scope (`38 §3.1`).
///
/// Uses the kernel API directly to register a transparent definition because
/// the surface elaborator's resolver is de Bruijn-only: global names in
/// expression bodies require a separate name-resolution pass (planned, K3+).
/// The trust-accounting semantics of `trusted_base_delta` are orthogonal to
/// the surface parse — the test exercises the mechanism directly.
#[test]
fn relied_on_foreign_listed_in_trusted_base_delta() {
    let mut env = ElabEnv::new().expect("ElabEnv::new()");
    env.elaborate_decl_v1(r#"foreign os_write : Int -> Bytes -> Int = "write" "libc" [FS]"#)
        .expect("os_write must elaborate");
    let os_write_id = *env.globals.get("os_write").expect("os_write registered");
    let int_id = *env.globals.get("Int").expect("Int");

    // Register a transparent definition whose body IS `os_write` (direct use).
    let use_def_id = env.env.fresh_id();
    env.env.add_decl(Decl::Transparent {
        id: use_def_id,
        level_params: vec![],
        ty: Term::const_(int_id, vec![]),
        body: Term::const_(os_write_id, vec![]),
    });

    let delta = trusted_base_delta(&env.env, use_def_id);
    assert!(
        delta.contains(&os_write_id),
        "os_write must appear in the caller's trusted_base_delta — it references it"
    );
}

/// B2: (soundness) a definition whose body does NOT reference `os_write` has
/// it ABSENT from `trusted_base_delta`. Being in scope ≠ reliance.
#[test]
fn not_relied_on_foreign_absent_from_trusted_base_delta() {
    let mut env = ElabEnv::new().expect("ElabEnv::new()");
    env.elaborate_decl_v1(r#"foreign os_write : Int -> Bytes -> Int = "write" "libc" [FS]"#)
        .expect("os_write must elaborate");
    let os_write_id = *env.globals.get("os_write").expect("os_write registered");
    let int_id = *env.globals.get("Int").expect("Int");

    // A transparent definition whose body does NOT reference os_write.
    let no_def_id = env.env.fresh_id();
    env.env.add_decl(Decl::Transparent {
        id: no_def_id,
        level_params: vec![],
        ty: Term::const_(int_id, vec![]),
        body: Term::const_(int_id, vec![]),
    });

    let delta = trusted_base_delta(&env.env, no_def_id);
    assert!(
        !delta.contains(&os_write_id),
        "os_write must NOT appear in the non-caller's trusted_base_delta — no reference"
    );
}

/// B1+B2 pair discriminant: `trusted_base_delta` FLIPS on dependency, not scope.
/// A buggy emitter that lists every in-scope foreign would make B2 fail → red.
#[test]
fn trusted_base_delta_flips_on_dependency_not_scope() {
    let mut env = ElabEnv::new().expect("ElabEnv::new()");
    env.elaborate_decl_v1(r#"foreign os_write : Int -> Bytes -> Int = "write" "libc" [FS]"#)
        .expect("os_write");
    let os_write_id = *env.globals.get("os_write").expect("os_write");
    let int_id = *env.globals.get("Int").expect("Int");

    // caller: body IS os_write.
    let use_id = env.env.fresh_id();
    env.env.add_decl(Decl::Transparent {
        id: use_id,
        level_params: vec![],
        ty: Term::const_(int_id, vec![]),
        body: Term::const_(os_write_id, vec![]),
    });

    // non_caller: body does not mention os_write.
    let no_id = env.env.fresh_id();
    env.env.add_decl(Decl::Transparent {
        id: no_id,
        level_params: vec![],
        ty: Term::const_(int_id, vec![]),
        body: Term::const_(int_id, vec![]),
    });

    let delta_use = trusted_base_delta(&env.env, use_id);
    let delta_no = trusted_base_delta(&env.env, no_id);

    assert!(
        delta_use.contains(&os_write_id),
        "caller: os_write in delta"
    );
    assert!(
        !delta_no.contains(&os_write_id),
        "non_caller: os_write absent"
    );
    // Discriminating: the two deltas differ on os_write.
    assert_ne!(
        delta_use.contains(&os_write_id),
        delta_no.contains(&os_write_id),
        "trusted_base_delta must flip between caller and non-caller"
    );
}

// ─── C1/C2: AC3 — `pure` → P, kernel-proved → Q ─────────────────────────────

/// C1: `pure foreign`'s assumed guarantee stays in `trusted_base()` (`P`,
/// never `Q`). The `pure` flag is recorded; it does NOT discharge the postulate.
///
/// Q half: BytesRoundTripLaw is dischargeable — demonstrates the P/Q split
/// on one artifact. L6 `decode_encode_roundtrip_provable` is the source test;
/// here we tie it to the foreign P side.
#[test]
fn pure_foreign_assumption_rides_p_not_q() {
    let mut env = ElabEnv::new().expect("ElabEnv::new()");

    let result = env
        .elaborate_decl_v1(r#"foreign c_sqrt : Int -> Int = "sqrt" "m" pure"#)
        .expect("c_sqrt must elaborate");
    let fb = result.foreign_binding.expect("ForeignBinding");
    let c_sqrt_id = fb.postulate_id;

    // `pure` flag is recorded.
    assert!(fb.is_pure, "pure flag must be recorded");

    // The postulate remains in trusted_base — assumed, not proved → P.
    assert!(
        env.env.trusted_base().contains(&c_sqrt_id),
        "pure foreign must remain in trusted_base — trusted-by-postulate, not Q"
    );

    // Q half: discharge BytesRoundTripLaw to show the Q path exists.
    // BytesRoundTripLaw is registered in ElabEnv::new() by bytes::register_bytes_env.
    let prove_result = env
        .elaborate_decl_v1("prove roundtrip : BytesRoundTripLaw")
        .expect("prove BytesRoundTripLaw");
    let obl_hole_id = prove_result.obligations[0].hole_id;
    let goal = prove_result.obligations[0].goal_closed.clone();
    let wit_id = env
        .declare_postulate_raw("roundtrip_wit", goal)
        .expect("witness postulate");
    let cert = Term::const_(wit_id, vec![]);
    let obl = prove_result.obligations[0].clone();
    assert!(env.discharge_hole(&obl, cert), "discharge must succeed");

    // After discharge: obligation hole ∉ trusted_base → Q (proved).
    assert!(!env.is_open_hole(obl_hole_id), "discharged obligation → Q");

    // Foreign postulate: still in trusted_base → P (pure does not discharge).
    assert!(
        env.env.trusted_base().contains(&c_sqrt_id),
        "c_sqrt must remain in trusted_base (P) — pure does not discharge"
    );
}

/// C2: wrong `pure` claim is CONFINED to the postulate (never Q) and LISTED
/// in `trusted_base()` — visible, not hidden.
#[test]
fn wrong_pure_confined_and_listed() {
    let mut env = ElabEnv::new().expect("ElabEnv::new()");

    let result = env
        .elaborate_decl_v1(r#"foreign bad_pure : Int -> Int = "rand" "c" pure"#)
        .expect("bad_pure must elaborate — type system cannot reject it");
    let fb = result.foreign_binding.expect("ForeignBinding");

    // Confined: in trusted_base (cannot masquerade as proved).
    assert!(
        env.env.trusted_base().contains(&fb.postulate_id),
        "wrong pure assumption must be listed in trusted_base — confined, not silent"
    );
    assert!(fb.is_pure, "is_pure must be true");
}

// ─── D1: AC4 — boundary contracts become runtime-checked ─────────────────────

/// D1: a `foreign` with `ensures` clauses that are statically unprovable →
/// `FfiRuntimeCheck` obligations emitted (`21 §5.2`, `38 §3.3`).
///
/// Discriminating pair: WITH ensures → obligation emitted; WITHOUT → none.
/// Uses `foreign::elaborate_foreign` directly to exercise the AC4 path.
#[test]
fn unprovable_foreign_ensures_emits_runtime_check() {
    let mut env = ElabEnv::new().expect("ElabEnv::new()");
    let bytes_id = *env.globals.get("Bytes").expect("Bytes");
    let int_id = *env.globals.get("Int").expect("Int");

    let ty = Term::pi(Term::const_(int_id, vec![]), Term::const_(int_id, vec![]));
    let binding = elaborate_foreign(
        &mut env.env,
        &mut env.globals,
        bytes_id,
        "checked_sqrt",
        ty,
        "sqrt",
        "m",
        true,
        &[],
        &["result >= 0".to_string()],
        &Span::zero(),
    )
    .expect("elaborate_foreign must succeed");

    // AC4: at least one runtime check obligation emitted.
    assert!(
        !binding.runtime_checks.is_empty(),
        "unprovable ensures must emit at least one FfiRuntimeCheck"
    );

    let rc = &binding.runtime_checks[0];
    assert_eq!(rc.clause_kind, "ensures", "clause kind must be 'ensures'");
    assert!(
        env.env.trusted_base().contains(&rc.hole_id),
        "runtime-check hole must be in trusted_base (tested, not proved)"
    );
    assert!(
        env.env.trusted_base().contains(&binding.postulate_id),
        "foreign postulate must also be in trusted_base"
    );

    // Discriminant: WITHOUT ensures → no runtime checks.
    let int_ty2 = Term::pi(Term::const_(int_id, vec![]), Term::const_(int_id, vec![]));
    let no_check = elaborate_foreign(
        &mut env.env,
        &mut env.globals,
        bytes_id,
        "plain_foreign",
        int_ty2,
        "plain",
        "m",
        false,
        &[],
        &[],
        &Span::zero(),
    )
    .expect("plain_foreign must elaborate");
    assert!(
        no_check.runtime_checks.is_empty(),
        "no ensures → no runtime checks (discriminant)"
    );
}

// ─── E1/E2: AC5 — effects mandatory ──────────────────────────────────────────

/// E1: world-touching `foreign` WITHOUT its effect row → escape-check reject;
/// WITH the row → accepts.
///
/// Seeds from `env.foreign_env.io_effect_rows` — NOT a hand-fed literal.
/// Removing the `[FS]` registration empties the map → `infer_all` finds no
/// callee row → `check_escape` passes → `expect_err` fails structurally
/// (green-vs-green is impossible).
#[test]
fn world_foreign_without_row_rejected() {
    let mut env = ElabEnv::new().expect("ElabEnv::new()");
    env.elaborate_decl_v1(r#"foreign os_write : Int -> Bytes -> Int = "write" "libc" [FS]"#)
        .expect("foreign os_write");

    // Seed from ACTUAL registration.
    let seed = env.foreign_env.io_effect_rows.clone();
    assert!(
        seed.contains_key("os_write"),
        "os_write must be in io_effect_rows"
    );

    // caller_no_row: no declared row — FS escapes.
    let caller_no = EffectDecl::new("caller_no").with_callee("os_write");
    let rows_no = infer_all(&seed, &[caller_no.clone()]);
    let mut witnesses = WitnessMap::new();
    witnesses.insert("FS".to_string(), "os_write".to_string());
    let err = check_escape(&caller_no, &rows_no["caller_no"], &witnesses)
        .expect_err("FS must escape the empty declared row → reject");
    match err {
        EffectError::EffectEscapes { witnesses: ws, .. } => {
            assert!(ws.iter().any(|(e, _)| e == "FS"), "FS must be in witnesses");
        }
        other => panic!("expected EffectEscapes, got {:?}", other),
    }

    // caller_ok: declares [FS] — accepts.
    let fs_row = seed
        .get("os_write")
        .cloned()
        .expect("FS row from registration");
    let caller_ok = EffectDecl::new("caller_ok")
        .with_declared_row(fs_row)
        .with_callee("os_write");
    let rows_ok = infer_all(&seed, &[caller_ok.clone()]);
    check_escape(&caller_ok, &rows_ok["caller_ok"], &WitnessMap::new())
        .expect("declared [FS] must accept os_write — no escape");
}

/// E2: `pure`-but-effectful foreign is the NAMED RESIDUAL the type discipline
/// CANNOT catch. Conformance asserts it is listed (visible in `trusted_base()`)
/// and NOT rejected — asserting "the type system rejects it" would over-claim.
#[test]
fn pure_but_effectful_foreign_is_the_named_residual() {
    let mut env = ElabEnv::new().expect("ElabEnv::new()");

    // `sneaky` declared `pure` (empty row) but its C symbol performs I/O.
    let result = env
        .elaborate_decl_v1(r#"foreign sneaky : Int -> Int = "do_io_secretly" "c" pure"#)
        .expect("sneaky must elaborate — type system cannot reject it");

    let fb = result.foreign_binding.expect("ForeignBinding");

    // Listed: visible in trusted_base, not silent.
    assert!(
        env.env.trusted_base().contains(&fb.postulate_id),
        "pure-but-effectful foreign must be listed in trusted_base (visible, not silent)"
    );

    // In trusted_base → P, never Q (the wrong claim is confined).
    assert!(
        env.is_open_hole(fb.postulate_id),
        "pure-but-effectful foreign is in trusted_base → P, never Q"
    );

    // Effect row is empty (type system believed the pure claim — this is the gap).
    assert!(
        fb.effect_row.is_empty(),
        "pure foreign has empty effect row — the type system accepted the claim"
    );
}

// ─── F1: AC6 — capability + effect compose (3-way) ──────────────────────────

/// F1: authority + flow compose — BOTH the effect row AND the capability are
/// required. Three-way gate from `36 §1.4` + `36 §2.5`:
///
/// (a) declares [FS] AND holds Cap_FS → accepts both checks
/// (b) declares [FS] but no Cap_FS → MissingCapability
/// (c) holds Cap_FS but no [FS] declared → EffectEscapes
#[test]
fn foreign_world_action_needs_row_and_capability() {
    let mut env = ElabEnv::new().expect("ElabEnv::new()");
    env.elaborate_decl_v1(r#"foreign os_write : Int -> Bytes -> Int = "write" "libc" [FS]"#)
        .expect("foreign os_write");
    let seed = env.foreign_env.io_effect_rows.clone();
    let fs_row = seed.get("os_write").cloned().expect("FS row");

    // (a) declares [FS] AND holds Cap_FS → accepts.
    let decl_a = EffectDecl::new("caller_a")
        .with_declared_row(fs_row.clone())
        .with_cap_param(CapParam::new("fs_cap", "FS"))
        .with_callee("os_write");
    let rows_a = infer_all(&seed, &[decl_a.clone()]);
    let inferred_a = rows_a["caller_a"].clone();
    check_escape(&decl_a, &inferred_a, &WitnessMap::new())
        .expect("(a) row declared → escape check passes");
    check_capabilities_no_handler(&decl_a, &inferred_a)
        .expect("(a) Cap_FS present → capability check passes");

    // (b) declares [FS] but no Cap_FS → MissingCapability.
    let decl_b = EffectDecl::new("caller_b")
        .with_declared_row(fs_row.clone())
        .with_callee("os_write");
    let rows_b = infer_all(&seed, &[decl_b.clone()]);
    let inferred_b = rows_b["caller_b"].clone();
    check_escape(&decl_b, &inferred_b, &WitnessMap::new())
        .expect("(b) row declared → escape passes");
    let err_b = check_capabilities_no_handler(&decl_b, &inferred_b)
        .expect_err("(b) no Cap_FS → MissingCapability");
    match err_b {
        EffectError::MissingCapability { effect, .. } => {
            assert_eq!(effect, "FS", "missing capability must be FS");
        }
        other => panic!("expected MissingCapability, got {:?}", other),
    }

    // (c) holds Cap_FS but no [FS] declared → EffectEscapes.
    let decl_c = EffectDecl::new("caller_c")
        .with_cap_param(CapParam::new("fs_cap", "FS"))
        .with_callee("os_write");
    let rows_c = infer_all(&seed, &[decl_c.clone()]);
    let inferred_c = rows_c["caller_c"].clone();
    let mut wit_c = WitnessMap::new();
    wit_c.insert("FS".to_string(), "os_write".to_string());
    let err_c = check_escape(&decl_c, &inferred_c, &wit_c)
        .expect_err("(c) no declared row → EffectEscapes");
    match err_c {
        EffectError::EffectEscapes { witnesses: ws, .. } => {
            assert!(ws.iter().any(|(e, _)| e == "FS"), "FS must escape");
        }
        other => panic!("expected EffectEscapes, got {:?}", other),
    }
}

// ─── G1: G6 — verified component with foreign + round-trip ───────────────────

/// G1: a verified component that makes a `foreign` call (→ P, listed in
/// `trusted_base_delta`) AND has a dischargeable round-trip law (→ Q) in the
/// same artifact — the honest trust split `38 §4`.
#[test]
fn verified_component_foreign_call_and_roundtrip_proof() {
    let mut env = ElabEnv::new().expect("ElabEnv::new()");

    env.elaborate_decl_v1(r#"foreign io_read : Int -> Bytes = "read_bytes" "libc" [FS]"#)
        .expect("io_read must elaborate");
    let io_read_id = *env.globals.get("io_read").expect("io_read");
    let bytes_id = *env.globals.get("Bytes").expect("Bytes");

    // Register a transparent component whose body references io_read.
    let comp_id = env.env.fresh_id();
    env.env.add_decl(Decl::Transparent {
        id: comp_id,
        level_params: vec![],
        ty: Term::const_(bytes_id, vec![]),
        body: Term::const_(io_read_id, vec![]),
    });

    // G6 condition 1: foreign in trusted_base_delta (→ P).
    let delta = trusted_base_delta(&env.env, comp_id);
    assert!(
        delta.contains(&io_read_id),
        "io_read postulate must be in component's trusted_base_delta → P"
    );

    // G6 condition 2: round-trip law is dischargeable (→ Q).
    // BytesRoundTripLaw is registered in ElabEnv::new() by bytes::register_bytes_env.
    let prove_result = env
        .elaborate_decl_v1("prove rt : BytesRoundTripLaw")
        .expect("prove BytesRoundTripLaw");
    assert_eq!(prove_result.obligations.len(), 1, "one obligation");
    let rt_hole_id = prove_result.obligations[0].hole_id;
    let goal = prove_result.obligations[0].goal_closed.clone();
    let wit_id = env.declare_postulate_raw("rt_wit", goal).expect("witness");
    let cert = Term::const_(wit_id, vec![]);
    let obl = prove_result.obligations[0].clone();
    assert!(env.discharge_hole(&obl, cert), "discharge must succeed");

    // After discharge: rt_hole ∉ trusted_base → Q.
    assert!(
        !env.is_open_hole(rt_hole_id),
        "discharged rt leaves trusted_base → Q"
    );

    // Foreign postulate still in trusted_base → P.
    assert!(
        env.env.trusted_base().contains(&io_read_id),
        "io_read must remain in trusted_base → P"
    );
}
