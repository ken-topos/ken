//! PX8-P checked `withBuffer` producer and exact perform inventory.

use std::collections::BTreeSet;

use ken_elaborator::compiler_driver::{compile_checked_target_denotation, CompilerSource};
use ken_elaborator::{emit_checked_target_export, ElabEnv, ResourceLifetimeObligation};

const BUFFER_ONLY_PRODUCER: &str = r#"
fn px8p_buffer_body (_resource : Resource Buffer)
  : HostIO AFull (ResourceBodyResult Unit Unit) =
  Ret (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (ResourceBodyResult Unit Unit)
    (ResourceBodyOk Unit Unit MkUnit)

proc px8p_buffer_only (capacity : Int)
  : HostIO AFull (Result ResourceError (ResourceBracketResult Unit Unit))
    visits [FS, BufferAllocate, ResourceRelease] =
  withBuffer AFull Unit Unit capacity px8p_buffer_body
"#;

#[test]
fn with_buffer_has_the_ruled_surface_and_keeps_raw_allocation_private() {
    let mut env = ElabEnv::empty().expect("PX8-P prelude bootstrap");
    env.elaborate_file(
        r#"
proc exact_with_buffer_surface
  (a : Auth) (e r : Type) (capacity : Int)
  (body : Resource Buffer -> HostIO a (ResourceBodyResult e r))
  : HostIO a (Result ResourceError (ResourceBracketResult e r)) visits [FS] =
  withBuffer a e r capacity body
"#,
    )
    .expect("the exact ruled withBuffer signature elaborates");

    assert!(env
        .env
        .transparent_body(env.globals["withBuffer"])
        .is_some());
    assert!(
        env.env
            .constructor(env.prelude_env.private_buffer_allocate_id)
            .is_some(),
        "the private checked BufferAllocate constructor is admitted"
    );
    for private in [
        "PrivateBufferAllocate",
        "private_with_buffer_after_allocate",
        "resource_settle_result_for",
    ] {
        assert!(
            !env.globals.contains_key(private),
            "private protocol identity `{private}` escaped"
        );
    }
    for deferred in ["BufferFreeze", "FsReadAt", "FsWriteAt", "writeAll"] {
        assert!(
            !env.globals.contains_key(deferred),
            "PX8-P must not expose deferred Buffer use `{deferred}`"
        );
    }
    assert_eq!(ken_host::HostOpV1::BufferAllocate as u16, 0x0402);
}

#[test]
fn real_checked_buffer_bracket_emits_exact_allocate_and_release_sigma() {
    let denotation = compile_checked_target_denotation(
        "px8p_checked_buffer_producer",
        CompilerSource::new("producer.ken", BUFFER_ONLY_PRODUCER),
        "px8p_buffer_only",
    )
    .expect("real checked withBuffer denotation");
    let export =
        emit_checked_target_export(&denotation, &[], &BTreeSet::new(), Vec::new(), Vec::new())
            .expect("PX8-P producer reaches the existing export transaction");

    assert_eq!(
        export.alphabet,
        BTreeSet::from(["BufferAllocate".to_string(), "ResourceRelease".to_string(),])
    );
    let Some(ResourceLifetimeObligation::V2(obligation)) = export.resource_lifetime_obligation
    else {
        panic!("the combined PX8-P + PX8-V train must emit V2")
    };
    assert_eq!(obligation.status, "delegated");
    assert_eq!(obligation.plans.len(), 1);
    assert_eq!(
        obligation.plans[0].resource_kind,
        ken_host::ResourceKindV1::Buffer
    );
    assert_eq!(
        obligation.plans[0].require_same_at,
        vec![ken_elaborator::ResourceLifetimeBindingPointV2 {
            operation: ken_host::HostOpV1::ResourceRelease,
            role: ken_host::ResourceBindingRoleV2::Target,
        }]
    );
}
