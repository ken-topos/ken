use std::collections::BTreeSet;

use ken_elaborator::effects::row::EffectRow;
use ken_elaborator::{emit_export, serialize_export, ResourceLifetimeObligationV1};

fn export_with(alphabet: EffectRow) -> ken_elaborator::BehavioralExport {
    emit_export(
        "px7f-resource-target",
        &[],
        &BTreeSet::new(),
        alphabet,
        vec![],
        vec![],
    )
    .expect("resource export")
}

#[test]
fn fs_open_reachability_emits_exactly_one_pinned_correlated_obligation() {
    let export = export_with(EffectRow::from_effects([
        "FsOpen".to_string(),
        "FsHandleMetadata".to_string(),
        "ResourceRelease".to_string(),
    ]));
    assert_eq!(
        export.resource_lifetime_obligation,
        Some(ResourceLifetimeObligationV1::pinned())
    );

    let wire = serialize_export(&export);
    let obligations = wire["obligations"].as_array().expect("T array");
    assert_eq!(
        obligations.len(),
        1,
        "one target-level template, not three atoms"
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
fn no_reachable_acquire_emits_no_resource_lifetime_obligation() {
    let export = export_with(EffectRow::from_effects([
        "ConsoleWrite".to_string(),
        "FsHandleMetadata".to_string(),
        "ResourceRelease".to_string(),
    ]));
    assert!(export.resource_lifetime_obligation.is_none());
    assert!(serialize_export(&export)["obligations"]
        .as_array()
        .expect("T array")
        .is_empty());
}

#[test]
fn correlated_descriptor_participates_in_the_export_hash() {
    let without = export_with(EffectRow::from_effects([
        "FsHandleMetadata".to_string(),
        "ResourceRelease".to_string(),
    ]));
    let with = export_with(EffectRow::from_effects([
        "FsOpen".to_string(),
        "FsHandleMetadata".to_string(),
        "ResourceRelease".to_string(),
    ]));
    assert_ne!(without.hash, with.hash);

    let repeated = export_with(EffectRow::from_effects([
        "ResourceRelease".to_string(),
        "FsOpen".to_string(),
        "FsHandleMetadata".to_string(),
    ]));
    assert_eq!(with.hash, repeated.hash, "canonical ordering is stable");
}
