use std::collections::BTreeSet;

use ken_elaborator::temporal::{Pred, Temporal};
use ken_elaborator::{
    compiler_driver::{compile_checked_target_denotation, CompilerSource},
    emit_checked_target_export, serialize_export, try_serialize_export, BehavioralExport,
    ExportError, ResourceLifetimeObligationV1, TEntry,
};

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
fn checked_resource_producer_emits_exactly_one_pinned_correlated_t_body() {
    let export = checked_export(RESOURCE_PRODUCER, "px7f_export_resource", vec![]);
    assert!(export.alphabet.contains("FsOpen"));
    assert_eq!(
        export.resource_lifetime_obligation,
        Some(ResourceLifetimeObligationV1::pinned())
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
            "schema_version": 1,
            "body_kind": "ResourceLifetimeObligationV1",
            "obligation_id": "resource-lifetime-v1",
            "status": "delegated",
            "correlation": {
                "identity_type": "ResourceTraceIdentityV1",
                "event_field": "EffectEventV1.resource",
                "bind_at": "Successful(FsOpen)",
                "require_same_at": ["FsHandleMetadata", "ResourceRelease"],
            },
            "acquire_op": "FsOpen",
            "use_op": "FsHandleMetadata",
            "settle_op": "ResourceRelease",
            "monitor_template": {
                "successful_acquire_settles_exactly_once": true,
                "forbid_successful_use_after_settlement": true,
                "require_no_live_resource_on": [
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
    malformed.correlation.event_field = "EffectEventV1.operation";

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
    assert_eq!(obligations[1]["obligation_id"], "resource-lifetime-v1");
}
