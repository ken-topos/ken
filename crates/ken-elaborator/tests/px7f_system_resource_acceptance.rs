//! PX7-F checked surface and `ResourceHostIO` collision discriminator.

use ken_elaborator::ElabEnv;
use ken_kernel::{env::PrimReduction, Decl};

const RESOURCE_KEN_MD: &str =
    include_str!("../../../catalog/packages/Capability/System/Resource.ken.md");

#[test]
fn system_resource_is_opaque_and_only_the_bracket_mints_it() {
    let mut env = ElabEnv::empty().expect("PX7-F prelude bootstrap");
    env.elaborate_ken_md_file(RESOURCE_KEN_MD)
        .expect("System.Resource checked fences elaborate");

    let resource = env.globals["Resource"];
    assert!(matches!(
        env.env.lookup(resource),
        Some(Decl::Primitive {
            reduction: PrimReduction::OpaqueType,
            ..
        })
    ));
    for private in [
        "PrivateFsOpen",
        "PrivateFsHandleMetadata",
        "PrivateResourceRelease",
        "PrivateResourceTraceIdentity",
        "private_resource_acquire",
        "private_with_resource_after_open",
        "release_if_live",
    ] {
        assert!(
            !env.globals.contains_key(private),
            "private representation/protocol identity `{private}` escaped"
        );
    }
    for public in ["withResource", "resourceMetadata", "release"] {
        let id = env.globals[public];
        assert!(
            env.env.transparent_body(id).is_some(),
            "`{public}` must be an ordinary checked term"
        );
    }
}

#[test]
fn resource_host_io_preserves_the_existing_host_io_identity() {
    let mut env = ElabEnv::empty().expect("PX7-F prelude bootstrap");
    let host_io = env.globals["HostIO"];
    let resource_host_io = env.globals["ResourceHostIO"];
    assert_ne!(host_io, resource_host_io);
    assert!(env.env.transparent_body(host_io).is_some());
    assert!(env.env.constructor(resource_host_io).is_some());

    // Opposite mutation: the old spelling destructively replaces the live
    // former in the flat namespace. The candidate therefore fails the
    // two-identity closure even though the data declaration itself parses.
    env.elaborate_decl("data CollisionMutation = HostIO IOError")
        .expect("opposite spelling demonstrates the destructive collision");
    assert_ne!(env.globals["HostIO"], host_io);
    assert_eq!(env.globals["ResourceHostIO"], resource_host_io);
}

#[test]
fn shipped_source_states_the_runtime_honesty_boundary() {
    for statement in [
        "runtime-enforced and Ward-checked",
        "Ken does not make them affine",
        "escaped copy is legal",
        "every later use returns",
        "external process destruction",
    ] {
        assert!(
            RESOURCE_KEN_MD.contains(statement),
            "missing source-level honesty statement: {statement}"
        );
    }
    assert!(!RESOURCE_KEN_MD.contains("Axiom"));
    assert!(!RESOURCE_KEN_MD.contains("proved"));
}
