//! PX8-X sole static resource-lifetime export projection.

use std::collections::BTreeSet;

use ken_elaborator::compiler_driver::{compile_checked_target_denotation, CompilerSource};
use ken_elaborator::{
    emit_checked_target_export, serialize_export, try_serialize_export, BehavioralExport,
    ExportError, ResourceLifetimeBindingPoint, ResourceLifetimeObligation,
};
use ken_host::{HostOpV1, ResourceBindingRole, ResourceKindV1};

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
    // Frozen content-addressed fixture identity, not the live Rust test route.
    // The route rename must not perturb the canonical export hash.
    let denotation = compile_checked_target_denotation(
        "px8v_static_v2_export_projection",
        CompilerSource::new("buffer.ken", BUFFER_ONLY_PRODUCER),
        "px8v_buffer_only",
    )
    .expect("real checked Buffer bracket denotation");
    emit_checked_target_export(&denotation, &[], &BTreeSet::new(), Vec::new(), Vec::new())
        .expect("real checked denotation reaches the resource projection")
}

fn buffer_obligation(export: &BehavioralExport) -> &ResourceLifetimeObligation {
    export
        .resource_lifetime_obligation
        .as_ref()
        .expect("BufferAllocate must select the sole obligation")
}

#[test]
fn real_checked_buffer_denotation_emits_direct_delegated_body() {
    let first = real_buffer_export();
    let second = real_buffer_export();
    assert_eq!(
        first.alphabet,
        BTreeSet::from(["BufferAllocate".to_string(), "ResourceRelease".to_string()])
    );
    assert_eq!(first.hash, second.hash, "the real projection is stable");
    assert_eq!(
        first.hash, "ken-export-v0:47f2f35b7a825ca3",
        "PX8-X rebaseline for the direct buffer-only obligation body"
    );

    let obligation = buffer_obligation(&first);
    assert_eq!(obligation.status, "delegated");
    assert_eq!(obligation.plans.len(), 1);
    assert_eq!(obligation.plans[0].resource_kind, ResourceKindV1::Buffer);
    assert_eq!(
        obligation.plans[0].bind_at,
        ResourceLifetimeBindingPoint {
            operation: HostOpV1::BufferAllocate,
            role: ResourceBindingRole::Target,
        }
    );
    assert_eq!(
        obligation.plans[0].require_same_at,
        vec![ResourceLifetimeBindingPoint {
            operation: HostOpV1::ResourceRelease,
            role: ResourceBindingRole::Target,
        }]
    );

    let wire = serialize_export(&first);
    assert_eq!(wire["schema"], "ken.export/v0");
    assert!(wire.get("resource_lifetime_obligation").is_none());
    let entries = wire["obligations"].as_array().expect("existing T sequence");
    assert_eq!(entries.len(), 1, "one direct body");
    assert_eq!(entries[0]["body_kind"], "ResourceLifetimeObligation");
    // Deleted-family negative controls: the sole direct body must not regain a
    // version wrapper or either former variant arm.
    assert!(entries[0].get("schema_version").is_none());
    assert_eq!(entries[0]["status"], "delegated");
    assert!(entries[0].get("V1").is_none());
    assert!(entries[0].get("V2").is_none());
    assert!(entries[0].get("variant").is_none());
    let encoded = serde_json::to_string(&wire).expect("canonical JSON");
    assert!(!encoded.contains("runtime_identity"));
    assert!(!encoded.contains("ResourceTraceIdentityV1("));
}

#[test]
fn real_body_rejects_missing_plan_and_unreachable_operation_by_named_error() {
    let mut missing = real_buffer_export();
    missing
        .resource_lifetime_obligation
        .as_mut()
        .expect("resource obligation")
        .plans
        .clear();
    assert_eq!(
        try_serialize_export(&missing),
        Err(ExportError::MissingResourceLifetimePlan {
            resource_kind: ResourceKindV1::Buffer,
        })
    );

    let mut extra = real_buffer_export();
    extra
        .resource_lifetime_obligation
        .as_mut()
        .expect("resource obligation")
        .plans[0]
        .require_same_at
        .insert(
            0,
            ResourceLifetimeBindingPoint {
                operation: HostOpV1::FsWriteAt,
                role: ResourceBindingRole::Buffer,
            },
        );
    assert_eq!(
        try_serialize_export(&extra),
        Err(ExportError::ResourceLifetimeOperationOutsideAlphabet {
            operation: HostOpV1::FsWriteAt,
        })
    );
}

#[test]
fn direct_body_and_fixed_descriptor_fail_closed_before_wire_output() {
    let mut absent = real_buffer_export();
    absent.resource_lifetime_obligation = None;
    assert_eq!(
        try_serialize_export(&absent),
        Err(ExportError::MissingResourceLifetimePlan {
            resource_kind: ResourceKindV1::Buffer,
        })
    );

    // Deleted-family negative control: the former V2 obligation identity is
    // malformed under the sole direct schema.
    let mut wrong_id = real_buffer_export();
    wrong_id
        .resource_lifetime_obligation
        .as_mut()
        .expect("resource obligation")
        .obligation_id = "resource-lifetime-v2";
    assert_eq!(
        try_serialize_export(&wrong_id),
        Err(ExportError::InvalidResourceLifetimeObligation)
    );

    let mut wrong_correlation = real_buffer_export();
    wrong_correlation
        .resource_lifetime_obligation
        .as_mut()
        .expect("resource obligation")
        .correlation
        .event_field = "EffectEvent.resource";
    assert_eq!(
        try_serialize_export(&wrong_correlation),
        Err(ExportError::InvalidResourceLifetimeObligation)
    );
}
