use std::collections::BTreeSet;

use ken_elaborator::temporal::{Pred, Temporal};
use ken_elaborator::{
    compiler_driver::{compile_checked_target_denotation, CompilerSource},
    emit_checked_target_export, serialize_export, try_serialize_export, BehavioralExport,
    ExportError, ResourceLifetimeBindingPoint, TEntry,
};
use ken_host::{HostOpV1, ResourceBindingRole, ResourceKindV1};

const RESOURCE_PRODUCER: &str = r#"
fn px7f_after_metadata (outcome : Result ResourceError FileMetadata)
  : HostIO AFull (ResourceBodyResult Unit Unit) =
  match outcome {
    Err _ |-> Ret (Coproduct (FSOp AFull) AmbientOp)
      (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
      (ResourceBodyResult Unit Unit) (ResourceBodyOk Unit Unit MkUnit);
    Ok _ |-> Ret (Coproduct (FSOp AFull) AmbientOp)
      (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
      (ResourceBodyResult Unit Unit) (ResourceBodyOk Unit Unit MkUnit)
  }

proc px7f_export_body (resource : Resource FsHandle)
  : HostIO AFull (ResourceBodyResult Unit Unit)
    visits [FS, FsHandleMetadata] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError FileMetadata) (ResourceBodyResult Unit Unit)
    (resourceMetadata AFull resource) (\outcome. px7f_after_metadata outcome)

proc px7f_export_resource (cap : Cap AFull) (path : Bytes)
  : HostIO AFull (Result FileError (ResourceBracketResult Unit Unit))
    visits [FS, FsOpen, FsHandleMetadata, ResourceRelease] =
  withResource AFull Unit Unit cap path ResourceMetadata px7f_export_body
"#;

const ORDINARY_PRODUCER: &str = r#"
proc px7f_export_ordinary (_value : Unit)
  : HostIO AFull (Result IOError Unit) visits [Console] =
  host_console AFull (Result IOError Unit) (flush Stdout)
"#;

fn checked_export(source: &str, target: &str, temporal: Vec<TEntry>) -> BehavioralExport {
    let denotation = compile_checked_target_denotation(
        "px7f_resource_lifetime_export",
        CompilerSource::new("fixture.ken", source),
        target,
    )
    .expect("checked producer denotation");
    emit_checked_target_export(&denotation, &[], &BTreeSet::new(), vec![], temporal)
        .expect("checked producer export")
}

fn ordinary_temporal() -> TEntry {
    TEntry {
        obligation_id: "ordinary-temporal".to_string(),
        formula: Temporal::Atom(Pred::Event("ConsoleFlush".to_string())),
    }
}

fn resource_temporal() -> TEntry {
    TEntry {
        obligation_id: "ordinary-temporal".to_string(),
        formula: Temporal::Atom(Pred::Event("FsOpen".to_string())),
    }
}

#[test]
fn checked_resource_producer_emits_exactly_one_correlated_t_body() {
    let export = checked_export(RESOURCE_PRODUCER, "px7f_export_resource", vec![]);
    assert_eq!(
        export.hash, "ken-export-v0:1bf3cb3f5b648ea7",
        "PX8-X rebaseline for the direct file-only obligation body"
    );
    assert!(export.alphabet.contains("FsOpen"));
    let obligation = export
        .resource_lifetime_obligation
        .as_ref()
        .expect("resource lifetime body");
    assert_eq!(obligation.plans.len(), 1);
    assert_eq!(obligation.plans[0].resource_kind, ResourceKindV1::FsHandle);
    assert_eq!(
        obligation.plans[0].require_same_at,
        vec![
            ResourceLifetimeBindingPoint {
                operation: HostOpV1::FsHandleMetadata,
                role: ResourceBindingRole::Target,
            },
            ResourceLifetimeBindingPoint {
                operation: HostOpV1::ResourceRelease,
                role: ResourceBindingRole::Target,
            },
        ]
    );

    let wire = serialize_export(&export);
    let obligations = wire["obligations"].as_array().expect("T array");
    assert_eq!(
        obligations.len(),
        1,
        "one target-level template, not three synthetic atoms"
    );
    assert_eq!(
        obligations[0],
        serde_json::json!({
            "body_kind": "ResourceLifetimeObligation",
            "obligation_id": "resource-lifetime",
            "status": "delegated",
            "correlation": {
                "identity_type": "ResourceTraceIdentityV1",
                "event_field": "EffectEvent.resource_bindings",
                "role_type": "ResourceBindingRole",
                "canonical_order": "OperationDefined",
            },
            "plans": [{
                "resource_kind": "FsHandle",
                "bind_at": "Successful(FsOpen, Target)",
                "require_same_at": [
                    ["FsHandleMetadata", "Target"],
                    ["ResourceRelease", "Target"]
                ],
            }],
            "monitor_template": {
                "correlate_every_role_binding": true,
                "successful_acquire_settles_exactly_once": true,
                "forbid_successful_use_after_settlement": true,
                "require_no_live_bracket_owned_identity_on": [
                    "NormalReturn",
                    "ReturnedError",
                    "ControlledTrap"
                ],
                "retain_settlement_outcome": true,
            }
        })
    );

    let encoded = serde_json::to_string(&obligations[0]).expect("JSON");
    assert!(!encoded.contains("Pred::Event"));
    assert!(!encoded.contains("event(FsOpen)"));
    assert!(!encoded.contains("runtime_identity"));
    assert!(!encoded.contains("\"r\""));
}

#[test]
fn checked_no_acquire_producer_preserves_the_pre_px7f_t_hash_route() {
    let temporal = ordinary_temporal();
    let export = checked_export(
        ORDINARY_PRODUCER,
        "px7f_export_ordinary",
        vec![temporal.clone()],
    );
    assert!(!export.alphabet.contains("FsOpen"));
    assert!(export.resource_lifetime_obligation.is_none());
    assert_eq!(export.obligations, vec![temporal]);
    assert_eq!(
        export.hash, "ken-export-v0:6360c2cb74f78f7e",
        "fixed regression for the exact corrected B1 canonical input"
    );

    let wire = serialize_export(&export);
    let obligations = wire["obligations"].as_array().expect("T array");
    assert_eq!(obligations.len(), 1);
    assert_eq!(obligations[0]["obligation_id"], "ordinary-temporal");
    assert_eq!(wire["hash"], export.hash);
}

#[test]
fn independent_event_lookalike_is_rejected_before_t_or_hash_emission() {
    let mut export = checked_export(RESOURCE_PRODUCER, "px7f_export_resource", vec![]);
    let malformed = export
        .resource_lifetime_obligation
        .as_mut()
        .expect("resource body");
    malformed.correlation.event_field = "EffectEvent.operation";

    assert_eq!(
        try_serialize_export(&export),
        Err(ExportError::InvalidResourceLifetimeObligation),
        "RL-B returns no wire object, hence no T entry and no emitted hash"
    );
}

#[test]
fn correlated_body_is_one_member_of_the_same_t_sequence() {
    let export = checked_export(
        RESOURCE_PRODUCER,
        "px7f_export_resource",
        vec![resource_temporal()],
    );
    let wire = serialize_export(&export);
    assert!(wire.get("resource_lifetime_obligation").is_none());
    let obligations = wire["obligations"].as_array().expect("T array");
    assert_eq!(obligations.len(), 2);
    assert_eq!(obligations[0]["obligation_id"], "ordinary-temporal");
    assert_eq!(obligations[1]["obligation_id"], "resource-lifetime");
}
