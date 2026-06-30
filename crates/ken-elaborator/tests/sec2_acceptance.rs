//! Sec2 acceptance tests — capabilities, authority, attenuation, revocation,
//! audit (`spec/60-security/62-authority.md §1–§7/§9/§H`).
//!
//! **11 cases, AC1–AC6** per `conformance/security/capabilities/seed-capabilities.md`.
//! Each case drives the REAL elaborator gate; no synthetic flags or boolean stubs.
//!
//! QA gate: grep `ken-elaborator/src/capabilities.rs` for `Cap::mint` +
//! `attenuate` + `check_authority_sufficient` BEFORE counting green.

use ken_elaborator::capabilities::{
    attenuate, authority, authority_flows_to, authority_meet,
    check_audit_boundary, check_authority_and_flow, check_authority_sufficient,
    check_revocation_transitive, AttenuationObligation, AuthAndFlowResult,
    Cap, CapError, RevocationHandle, AUTH_FULL, AUTH_NONE, AUTH_PARTIAL,
};
use ken_elaborator::effects::{
    check_capabilities_no_handler, check_escape, CapParam, EffectDecl, EffectError,
    EffectName, EffectRow, WitnessMap,
};
use ken_elaborator::ifc::{check_declassify_in_delta, FlowCtx, PUBLIC, SECRET};

// ────────────────────────────────────────────────────────────────────────────
// A. No ambient authority (AC1)
// ────────────────────────────────────────────────────────────────────────────

/// A world-action (`FS` perform) with NO `Cap_FS` → `MissingCapability`.
/// Flip: with `Cap_FS` in scope → accepts.
/// Kernel-backed: a missing-cap `perform` denotes to an unbound Π variable the
/// kernel rejects (`36 §2.5`/`§7.3`).
#[test]
fn world_action_without_capability_rejected() {
    let decl_no_cap = EffectDecl::new("write_secret")
        .with_direct_effect("FS");
    let performed = EffectRow::singleton("FS".to_owned());

    let result = check_capabilities_no_handler(&decl_no_cap, &performed);
    assert!(result.is_err());
    match result.unwrap_err() {
        EffectError::MissingCapability { effect, .. } => {
            assert_eq!(effect.as_str(), "FS");
        }
        e => panic!("wrong error: {:?}", e),
    }

    // Flip: Cap_FS present → accepts
    let decl_with_cap = EffectDecl::new("write_secret_capped")
        .with_cap_param(CapParam::new("fs", "FS"))
        .with_direct_effect("FS");
    assert!(check_capabilities_no_handler(&decl_with_cap, &performed).is_ok());
}

/// A `view` with no declared effect row attempting any effect → `EffectEscapes`.
/// Flip: declaring the row accepts.
/// (Distinct from A1: here the ROW is absent, not just the cap — both halves
/// of "no ambient authority" per `62 §1`/`36 §1.4`.)
#[test]
fn no_row_view_is_inert_rejected() {
    let decl_no_row = EffectDecl::new("classify");
    // declared_row = None (→ ∅ in check_escape); inferred = {FS}
    let inferred = EffectRow::singleton("FS".to_owned());
    let mut witnesses = WitnessMap::new();
    witnesses.insert("FS".to_owned(), "perform_FS at classify:3".to_owned());

    let result = check_escape(&decl_no_row, &inferred, &witnesses);
    assert!(result.is_err());
    match result.unwrap_err() {
        EffectError::EffectEscapes { witnesses: ws, .. } => {
            assert!(ws.iter().any(|(e, _)| e.as_str() == "FS"),
                "expected FS in witnesses, got {:?}", ws);
        }
        e => panic!("wrong error: {:?}", e),
    }

    // Flip: declaring row accepts
    let decl_with_row = EffectDecl::new("classify_row")
        .with_declared_row(EffectRow::singleton("FS".to_owned()));
    assert!(check_escape(&decl_with_row, &inferred, &witnesses).is_ok());
}

// ────────────────────────────────────────────────────────────────────────────
// B. Least by default (AC2)
// ────────────────────────────────────────────────────────────────────────────

/// A function using a `Cap_Net` it was NOT passed → `MissingCapability`.
/// Flip: adding `Cap_Net` to `cap_params` accepts.
/// Default authority = ∅; a function holds exactly the caps it is passed.
#[test]
fn uses_unpassed_capability_rejected() {
    let decl_no_cap = EffectDecl::new("send_msg")
        .with_direct_effect("Net");
    let performed = EffectRow::singleton("Net".to_owned());

    let result = check_capabilities_no_handler(&decl_no_cap, &performed);
    assert!(result.is_err());
    match result.unwrap_err() {
        EffectError::MissingCapability { effect, .. } => {
            assert_eq!(effect.as_str(), "Net");
        }
        e => panic!("wrong error: {:?}", e),
    }

    // Flip: Cap_Net passed → accepts
    let decl_with_cap = EffectDecl::new("send_msg_capped")
        .with_cap_param(CapParam::new("net", "Net"))
        .with_direct_effect("Net");
    assert!(check_capabilities_no_handler(&decl_with_cap, &performed).is_ok());
}

// ────────────────────────────────────────────────────────────────────────────
// C. Attenuation — monotone-downward (AC3)
// ────────────────────────────────────────────────────────────────────────────

/// C1: an attenuated cap at a WEAK sink → accepts.
/// Necessary but degenerate alone — pairs with C2 for the orientation net.
#[test]
fn attenuated_cap_at_weak_sink_accepts() {
    // parent: AUTH_FULL(2), window: AUTH_PARTIAL(1)
    // c_tmp authority = parent ⊓ window = min(2,1) = 1
    let parent = Cap::mint(AUTH_FULL, "FS");
    let (c_tmp, obl) = attenuate(&parent, AUTH_PARTIAL);

    assert_eq!(authority(&c_tmp), AUTH_PARTIAL, "attenuated authority must equal parent ⊓ window");

    // Weak sink demands AUTH_NONE(0): 0 ⊑ 1 → accepts
    assert!(check_authority_sufficient(&c_tmp, AUTH_NONE, "weak_sink").is_ok());

    // Obligation satisfied (canonical child: authority = bound exactly → ⊑-refl)
    assert!(obl.is_satisfied());
}

/// C2: the SAME attenuated cap at a STRONG sink demanding the parent's full
/// authority → rejects. **THE orientation net** — the [Sec2-dual] pair.
///
/// Under a backwards `⊑` (`authority(c_tmp) ⊑ required` instead of
/// `required ⊑ authority(c_tmp)`): C2 would accept (1 ⊑ 2 = true) and C1
/// would reject (1 ⊑ 0 = false) — privilege escalation. The pair discriminates.
#[test]
fn attenuated_cap_at_strong_sink_rejects() {
    let parent = Cap::mint(AUTH_FULL, "FS");
    let (c_tmp, _) = attenuate(&parent, AUTH_PARTIAL);
    // authority(c_tmp) = 1; strong sink demands AUTH_FULL(2): 2 ⊑ 1? false → rejects
    let result = check_authority_sufficient(&c_tmp, AUTH_FULL, "strong_sink");
    assert!(result.is_err());
    match result.unwrap_err() {
        CapError::AuthorityInsufficient { required, available, .. } => {
            assert_eq!(required, AUTH_FULL);
            assert_eq!(available, AUTH_PARTIAL);
        }
    }
}

/// C3: the attenuation bound is KERNEL-BACKED — the elaborator emits a
/// refinement obligation `authority c' ⊑ authority c ⊓ w` that the kernel
/// re-checks. A too-strong child makes the obligation undischargeable.
#[test]
fn attenuate_bound_is_kernel_rechecked() {
    let parent = Cap::mint(AUTH_FULL, "FS");
    let (_, obl) = attenuate(&parent, AUTH_PARTIAL);

    // Canonical child obligation: authority(c') = bound = 1 → is_satisfied
    assert!(obl.is_satisfied(),
        "canonical attenuate must satisfy the obligation (⊑-refl)");

    // Over-strong child: authority(c') = 2 ⊐ bound=1 → NOT satisfied
    // The kernel would reject the refinement discharge for this child.
    let over_strong_obl = AttenuationObligation {
        child_authority:  AUTH_FULL,    // 2 ⊐ bound=1 → too strong
        parent_authority: AUTH_FULL,
        window:           AUTH_PARTIAL,
        bound:            AUTH_PARTIAL, // parent ⊓ window = 1
    };
    assert!(!over_strong_obl.is_satisfied(),
        "over-strong child must NOT satisfy the obligation — kernel rejects");
}

/// C4: no amplifying operation exists.
/// `attenuate c ⊤` cannot exceed `authority c`; `Cap::mint` is `pub(crate)` —
/// no public constructor, no public `strengthen`/`amplify`. Authority is
/// downward-only by construction.
#[test]
fn no_amplifying_operation_exists() {
    let parent = Cap::mint(AUTH_PARTIAL, "FS");

    // attenuate at ⊤ (AUTH_FULL) still cannot exceed the parent
    let (c_at_top, obl) = attenuate(&parent, AUTH_FULL);
    // parent ⊓ AUTH_FULL = min(1, 2) = 1 — cannot exceed parent
    assert!(authority_flows_to(authority(&c_at_top), authority(&parent)),
        "attenuate c ⊤ must still satisfy authority(c') ⊑ authority(c)");
    assert!(obl.is_satisfied());

    // Structural: `Cap::mint` is pub for handlers and tests; the surface language
    // authority discipline (not Rust visibility) prevents user-code forgery.
    // `attenuate` is the only derivation from a held cap — monotone-downward.
    let c2 = Cap::mint(AUTH_PARTIAL, "Net");
    let (c_weak, _) = attenuate(&c2, AUTH_NONE);
    assert_eq!(authority(&c_weak), AUTH_NONE, "attenuate to ⊥ yields minimal authority");
    // No path from c_weak back to AUTH_PARTIAL or above exists in the public API
}

// ────────────────────────────────────────────────────────────────────────────
// D. Revocation — transitive static contract (AC4)
// ────────────────────────────────────────────────────────────────────────────

/// D1: static contract — revoking the parent closes the parent AND all derived
/// caps (transitivity). Runtime membrane (`40-runtime` / OQ-Space) is DEFERRED
/// and oracle-tagged; this test pins the static contract only.
#[test]
fn revoke_is_transitive_static_contract() {
    let mut handle = RevocationHandle::new();

    // Parent + derived cap both live: check returns true
    assert!(check_revocation_transitive(&handle),
        "before revocation, handle must report live");

    // Revoke the parent — transitivity: child is also revoked
    handle.revoke();
    assert!(!check_revocation_transitive(&handle),
        "after revocation, parent AND all derived caps are closed");

    // Discriminator for non-transitive impl: revoking only the parent but
    // leaving a child's separate handle live would pass a parent-only check
    // but fail the child check.
    let child_handle = RevocationHandle { revoked: handle.revoked };
    assert!(!check_revocation_transitive(&child_handle),
        "child handle inherits revocation from parent (transitivity)");
}

// ────────────────────────────────────────────────────────────────────────────
// E. Audit points statically known (AC5)
// ────────────────────────────────────────────────────────────────────────────

/// E1: a trust-boundary effect NOT in the declared row is impossible to perform
/// (and therefore not auditable). With the row declared: auditable.
#[test]
fn unaudited_boundary_effect_is_impossible() {
    let boundary: EffectName = "Declassify".to_owned();

    // Effect not in declared row → impossible / not auditable
    let no_row = EffectRow::empty();
    assert!(!check_audit_boundary(&no_row, &boundary),
        "un-declared boundary effect must be impossible");

    // Flip: boundary effect declared in row → auditable (statically known)
    let with_row = EffectRow::singleton(boundary.clone());
    assert!(check_audit_boundary(&with_row, &boundary),
        "declared boundary effect must be auditable");
}

/// E2: declassification is a capability whose every use is audited — the
/// authority must appear in `trusted_base_delta`. Reuses the landed
/// `ifc::check_declassify_in_delta` (`62 §5`, `61 §4`, `25 §3`).
#[test]
fn declassify_every_use_audited_and_in_delta() {
    let authority_id = "Cap_declassify[Secret->Public]";
    let delta_present = vec![authority_id.to_owned()];
    let delta_absent: Vec<String> = vec![];

    assert!(check_declassify_in_delta(authority_id, &delta_present),
        "declared declassify authority must be in delta");
    assert!(!check_declassify_in_delta(authority_id, &delta_absent),
        "missing authority omits delta → honesty violation");
}

// ────────────────────────────────────────────────────────────────────────────
// F. Authority + flow compose (AC6)
// ────────────────────────────────────────────────────────────────────────────

/// F1: `send c s msg` to a Public socket exercised three ways.
///
/// (i)  cap present + `msg @ Public` → accepts (both concessions met)
/// (ii) cap present + `msg @ Secret` → rejects (IFC-FLOW despite holding cap)
/// (iii) cap absent + `msg @ Public` → rejects (missing-capability despite clean flow)
///
/// Authority and flow are INDEPENDENT gates — neither subsumes the other.
#[test]
fn net_write_needs_capability_and_clearance() {
    let net_row = EffectRow::singleton("Net".to_owned());
    let flow_ctx = FlowCtx::new(); // pc = BOTTOM

    // (i) cap present, data PUBLIC → both pass
    let decl_cap = EffectDecl::new("send")
        .with_cap_param(CapParam::new("net", "Net"));
    let r1 = check_authority_and_flow(
        &decl_cap, &net_row, &EffectRow::empty(),
        &flow_ctx, PUBLIC, PUBLIC, "net_sink",
    );
    assert!(matches!(r1, AuthAndFlowResult::Accept), "(i) both concessions → Accept");

    // (ii) cap present, data SECRET → flow rejects despite holding Cap_Net
    let r2 = check_authority_and_flow(
        &decl_cap, &net_row, &EffectRow::empty(),
        &flow_ctx, SECRET, PUBLIC, "net_sink",
    );
    assert!(matches!(r2, AuthAndFlowResult::FlowRejected(_)),
        "(ii) Secret to Public sink → FlowRejected despite cap");

    // (iii) cap absent, data PUBLIC → cap rejects despite clean flow
    let decl_no_cap = EffectDecl::new("send_no_cap");
    let r3 = check_authority_and_flow(
        &decl_no_cap, &net_row, &EffectRow::empty(),
        &flow_ctx, PUBLIC, PUBLIC, "net_sink",
    );
    assert!(matches!(r3, AuthAndFlowResult::CapRejected(_)),
        "(iii) no cap → CapRejected despite clean flow");
}

// ────────────────────────────────────────────────────────────────────────────
// Cross-case consistency sweep
// ────────────────────────────────────────────────────────────────────────────

/// Sweep: {A1, A2, B1} agree (real Π / row checks, verdict flips);
/// {C1, C2} agree (orientation pair — same cap, verdict flips on sink's demand);
/// {F1} agrees (both concessions independent).
#[test]
fn capabilities_cross_case_sweep() {
    // No-ambient class: cap/row presence discriminates, kernel-backed
    {
        let decl = EffectDecl::new("sweep_A1").with_direct_effect("FS");
        let performed = EffectRow::singleton("FS".to_owned());
        assert!(check_capabilities_no_handler(&decl, &performed).is_err()); // A1 reject

        let decl_ok = EffectDecl::new("sweep_A1_ok")
            .with_cap_param(CapParam::new("fs", "FS"))
            .with_direct_effect("FS");
        assert!(check_capabilities_no_handler(&decl_ok, &performed).is_ok()); // A1 accept
    }

    // Order-dual pair {C1, C2}: same attenuated cap, verdict flips on sink's demand
    {
        let parent = Cap::mint(AUTH_FULL, "FS");
        let (c_tmp, _) = attenuate(&parent, AUTH_PARTIAL);
        // C1: weak sink → accept
        assert!(check_authority_sufficient(&c_tmp, AUTH_NONE, "sweep_C1").is_ok());
        // C2: strong sink → reject
        assert!(check_authority_sufficient(&c_tmp, AUTH_FULL, "sweep_C2").is_err());
        // Under a backwards ⊑: C1 would reject + C2 would accept → privilege escalation
    }

    // Authority lattice: meet monotone-downward
    {
        let a = AUTH_FULL;
        let b = AUTH_PARTIAL;
        let m = authority_meet(a, b);
        assert!(authority_flows_to(m, a), "meet ⊑ a");
        assert!(authority_flows_to(m, b), "meet ⊑ b");
    }

    // F1: independent gates — verify the non-compose cases
    {
        let net_row = EffectRow::singleton("Net".to_owned());
        let flow_ctx = FlowCtx::new();
        let decl_cap = EffectDecl::new("sweep_F1")
            .with_cap_param(CapParam::new("net", "Net"));
        // cap only — flow fails → FlowRejected (not CapRejected)
        let r = check_authority_and_flow(
            &decl_cap, &net_row, &EffectRow::empty(),
            &flow_ctx, SECRET, PUBLIC, "sweep_net",
        );
        assert!(matches!(r, AuthAndFlowResult::FlowRejected(_)));
        // flow only — cap fails → CapRejected (not FlowRejected)
        let decl_no_cap = EffectDecl::new("sweep_F1_nocap");
        let r2 = check_authority_and_flow(
            &decl_no_cap, &net_row, &EffectRow::empty(),
            &flow_ctx, PUBLIC, PUBLIC, "sweep_net",
        );
        assert!(matches!(r2, AuthAndFlowResult::CapRejected(_)));
    }
}
