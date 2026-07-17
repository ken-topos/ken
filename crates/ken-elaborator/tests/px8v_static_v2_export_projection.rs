//! PX8-V static V2 resource-lifetime export projection.

use std::collections::BTreeSet;

use ken_elaborator::compiler_driver::{compile_checked_target_denotation, CompilerSource};
use ken_elaborator::{
    emit_checked_target_export, serialize_export, try_serialize_export, BehavioralExport,
    ExportError, ResourceLifetimeBindingPointV2, ResourceLifetimeObligation,
    ResourceLifetimeObligationV1,
};
use ken_host::{HostOpV1, ResourceBindingRoleV2, ResourceKindV1};

const BUFFER_ONLY_PRODUCER: &str = r#"
fn px8v_buffer_body (_resource : Resource Buffer)
  : HostIO AFull (ResourceBodyResult Unit Unit) =
  Ret (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (ResourceBodyResult Unit Unit)
    (ResourceBodyOk Unit Unit MkUnit)

proc px8v_buffer_only (capacity : Int)
  : HostIO AFull (Result ResourceError (ResourceBracketResult Unit Unit))
    visits [FS, BufferAllocate, ResourceRelease] =
  withBuffer AFull Unit Unit capacity px8v_buffer_body
"#;

fn real_buffer_export() -> BehavioralExport {
    let denotation = compile_checked_target_denotation(
        "px8v_static_v2_export_projection",
        CompilerSource::new("buffer.ken", BUFFER_ONLY_PRODUCER),
        "px8v_buffer_only",
    )
    .expect("real checked Buffer bracket denotation");
    emit_checked_target_export(&denotation, &[], &BTreeSet::new(), Vec::new(), Vec::new())
        .expect("real checked denotation reaches V2 projection")
}

fn buffer_obligation(export: &BehavioralExport) -> &ken_elaborator::ResourceLifetimeObligationV2 {
    match export.resource_lifetime_obligation.as_ref() {
        Some(ResourceLifetimeObligation::V2(value)) => value,
        other => panic!("BufferAllocate must select exactly V2, got {other:?}"),
    }
}

#[test]
fn real_checked_buffer_denotation_emits_direct_delegated_v2_body() {
    let first = real_buffer_export();
    let second = real_buffer_export();
    assert_eq!(
        first.alphabet,
        BTreeSet::from(["BufferAllocate".to_string(), "ResourceRelease".to_string()])
    );
    assert_eq!(first.hash, second.hash, "the real V2 projection is stable");

    let obligation = buffer_obligation(&first);
    assert_eq!(obligation.schema_version, 2);
    assert_eq!(obligation.body_kind, "ResourceLifetimeObligationV2");
    assert_eq!(obligation.status, "delegated");
    assert_eq!(obligation.plans.len(), 1);
    assert_eq!(obligation.plans[0].resource_kind, ResourceKindV1::Buffer);
    assert_eq!(
        obligation.plans[0].bind_at,
        ResourceLifetimeBindingPointV2 {
            operation: HostOpV1::BufferAllocate,
            role: ResourceBindingRoleV2::Target,
        }
    );
    assert_eq!(
        obligation.plans[0].require_same_at,
        vec![ResourceLifetimeBindingPointV2 {
            operation: HostOpV1::ResourceRelease,
            role: ResourceBindingRoleV2::Target,
        }]
    );

    let wire = serialize_export(&first);
    assert_eq!(wire["schema"], "ken.export/v0");
    assert!(wire.get("resource_lifetime_obligation").is_none());
    let entries = wire["obligations"].as_array().expect("existing T sequence");
    assert_eq!(entries.len(), 1, "one body, not a parallel V1/V2 pair");
    assert_eq!(entries[0]["schema_version"], 2);
    assert_eq!(entries[0]["status"], "delegated");
    assert!(entries[0].get("V1").is_none());
    assert!(entries[0].get("V2").is_none());
    assert!(entries[0].get("variant").is_none());
    let encoded = serde_json::to_string(&wire).expect("canonical JSON");
    assert!(!encoded.contains("runtime_identity"));
    assert!(!encoded.contains("ResourceTraceIdentityV1("));
}

#[test]
fn real_v2_body_rejects_missing_plan_and_unreachable_operation_by_named_error() {
    let mut missing = real_buffer_export();
    match missing.resource_lifetime_obligation.as_mut() {
        Some(ResourceLifetimeObligation::V2(value)) => value.plans.clear(),
        other => panic!("expected V2, got {other:?}"),
    }
    assert_eq!(
        try_serialize_export(&missing),
        Err(ExportError::MissingResourceLifetimePlan {
            resource_kind: ResourceKindV1::Buffer,
        })
    );

    let mut extra = real_buffer_export();
    match extra.resource_lifetime_obligation.as_mut() {
        Some(ResourceLifetimeObligation::V2(value)) => {
            value.plans[0].require_same_at.insert(
                0,
                ResourceLifetimeBindingPointV2 {
                    operation: HostOpV1::FsWriteAt,
                    role: ResourceBindingRoleV2::Buffer,
                },
            );
        }
        other => panic!("expected V2, got {other:?}"),
    }
    assert_eq!(
        try_serialize_export(&extra),
        Err(ExportError::ResourceLifetimeOperationOutsideAlphabet {
            operation: HostOpV1::FsWriteAt,
        })
    );
}

#[test]
fn v2_selection_and_fixed_descriptor_fail_closed_before_wire_output() {
    let mut absent = real_buffer_export();
    absent.resource_lifetime_obligation = None;
    assert_eq!(
        try_serialize_export(&absent),
        Err(ExportError::MissingResourceLifetimePlan {
            resource_kind: ResourceKindV1::Buffer,
        })
    );

    let mut wrong_variant = real_buffer_export();
    wrong_variant.resource_lifetime_obligation = Some(ResourceLifetimeObligation::V1(
        ResourceLifetimeObligationV1::pinned(),
    ));
    assert_eq!(
        try_serialize_export(&wrong_variant),
        Err(ExportError::MissingResourceLifetimePlan {
            resource_kind: ResourceKindV1::Buffer,
        })
    );

    let mut wrong_version = real_buffer_export();
    match wrong_version.resource_lifetime_obligation.as_mut() {
        Some(ResourceLifetimeObligation::V2(value)) => value.schema_version = 1,
        other => panic!("expected V2, got {other:?}"),
    }
    assert_eq!(
        try_serialize_export(&wrong_version),
        Err(ExportError::InvalidResourceLifetimeObligation)
    );

    let mut wrong_correlation = real_buffer_export();
    match wrong_correlation.resource_lifetime_obligation.as_mut() {
        Some(ResourceLifetimeObligation::V2(value)) => {
            value.correlation.event_field = "EffectEventV1.resource";
        }
        other => panic!("expected V2, got {other:?}"),
    }
    assert_eq!(
        try_serialize_export(&wrong_correlation),
        Err(ExportError::InvalidResourceLifetimeObligation)
    );
}
