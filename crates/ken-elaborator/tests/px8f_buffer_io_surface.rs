//! PX8-F checked positioned-buffer surface and real structural proof terms.

use std::collections::BTreeSet;

use ken_elaborator::{ElabEnv, ElabError};
use ken_kernel::{whnf, Context, Decl, GlobalId, Term};

const BUFFER_KEN_MD: &str =
    include_str!("../../../catalog/packages/Capability/System/Buffer.ken.md");
const IO_KEN_MD: &str = include_str!("../../../catalog/packages/Capability/System/IO.ken.md");

fn result_head(env: &ken_kernel::GlobalEnv, ty: &Term) -> Term {
    let mut context = Context::new();
    let mut head = whnf(env, &context, ty);
    loop {
        match head {
            Term::Pi(domain, codomain) => {
                context.push(*domain);
                head = whnf(env, &context, &codomain);
            }
            result => return result,
        }
    }
}

fn public_buffer_span_producers(env: &ElabEnv) -> BTreeSet<String> {
    let buffer_span = env.globals["BufferSpan"];
    env.globals
        .iter()
        .filter_map(|(name, id)| {
            let ty = match env.env.lookup(*id) {
                Some(decl) => match decl {
                    Decl::Transparent { ty, .. }
                    | Decl::Opaque { ty, .. }
                    | Decl::Primitive { ty, .. } => ty,
                    Decl::Inductive(inductive) => &inductive.former_type,
                },
                None => match env.env.constructor(*id) {
                    Some((inductive, index)) => &inductive.constructors[index].type_,
                    None => {
                        panic!("public global `{name}` is neither a declaration nor a constructor")
                    }
                },
            };
            matches!(
                result_head(&env.env, ty),
                Term::IndFormer { id, .. } if id == buffer_span
            )
            .then(|| name.clone())
        })
        .collect()
}

#[test]
fn checked_surface_is_public_but_proof_carrying_constructors_stay_private() {
    let mut env = ElabEnv::empty().expect("PX8-F prelude");
    env.elaborate_ken_md_file(BUFFER_KEN_MD)
        .expect("System.Buffer checked fences");
    env.elaborate_ken_md_file(IO_KEN_MD)
        .expect("System.IO checked fences and five law terms");

    env.elaborate_file(
        r#"
proc px8f_exact_read
  (a : Auth) (file : Resource FsHandle) (offset : Int)
  (buffer : BufferHandle) (window : BufferWindow)
  : HostIO a (Result ResourceError ReadProgress) visits [FS] =
  readAt a file offset buffer window

proc px8f_exact_write
  (a : Auth) (file : Resource FsHandle) (offset : Int)
  (buffer : BufferHandle) (span : BufferSpan)
  : HostIO a (Result ResourceError WriteProgress) visits [FS] =
  writeAt a file offset buffer span

proc px8f_exact_freeze
  (a : Auth) (buffer : BufferHandle) (span : BufferSpan)
  : HostIO a (Result ResourceError Bytes) visits [FS] =
  freeze a buffer span

proc px8f_exact_write_all
  (a : Auth) (file : Resource FsHandle) (offset : Int)
  (buffer : BufferHandle) (span : BufferSpan)
  : HostIO a (Result ResourceError Unit) visits [FS] =
  writeAll a file offset buffer span

proc px8f_readsome_public_consumers
  (a : Auth) (file : Resource FsHandle) (offset : Int)
  (buffer : BufferHandle) (window : BufferWindow)
  : HostIO a (Result ResourceError Unit) visits [FS] =
  bind
    (Coproduct (FSOp a) AmbientOp)
    (resp_coproduct (FSOp a) AmbientOp (fs_resp a) ambient_resp)
    (Result ResourceError ReadProgress)
    (Result ResourceError Unit)
    (readAt a file offset buffer window)
    (\read_outcome.
      match read_outcome {
        Err error ↦
          Ret
            (Coproduct (FSOp a) AmbientOp)
            (resp_coproduct (FSOp a) AmbientOp (fs_resp a) ambient_resp)
            (Result ResourceError Unit)
            (Err ResourceError Unit error);
        Ok progress ↦
          match progress {
            ReadEof ↦
              Ret
                (Coproduct (FSOp a) AmbientOp)
                (resp_coproduct (FSOp a) AmbientOp (fs_resp a) ambient_resp)
                (Result ResourceError Unit)
                (Ok ResourceError Unit MkUnit);
            ReadSome span count ↦
              bind
                (Coproduct (FSOp a) AmbientOp)
                (resp_coproduct (FSOp a) AmbientOp (fs_resp a) ambient_resp)
                (Result ResourceError WriteProgress)
                (Result ResourceError Unit)
                (writeAt a file (add_int offset (transfer_count_int count)) buffer span)
                (\_written.
                  bind
                    (Coproduct (FSOp a) AmbientOp)
                    (resp_coproduct (FSOp a) AmbientOp (fs_resp a) ambient_resp)
                    (Result ResourceError Bytes)
                    (Result ResourceError Unit)
                    (freeze a buffer span)
                    (\_frozen. writeAll a file offset buffer span))
          }
      })
"#,
    )
    .expect("exact PX8-F public signatures elaborate");

    for public in [
        "BufferHandle",
        "BufferWindow",
        "BufferSpan",
        "TransferCount",
        "ReadProgress",
        "WriteProgress",
        "readAt",
        "writeAt",
        "spanBytes",
        "freeze",
        "writeAll",
        "write_all_exact_prefix_prop",
        "write_all_exact_prefix_prop::exact_prefix",
    ] {
        assert!(env.globals.contains_key(public), "missing `{public}`");
    }
    for private in [
        "PrivateBufferHandle",
        "buffer_handle_resource",
        "buffer_handle_capacity",
        "PrivateBufferSpan",
        "write_all_advance_span",
        "PrivateTransferCount",
        "PrivateFsReadAt",
        "PrivateFsWriteAt",
        "PrivateBufferFreeze",
        "private_read_at_positive",
        "private_write_all_fuel",
    ] {
        assert!(!env.globals.contains_key(private), "`{private}` escaped");
    }
    for law in [
        "write_all_exact_prefix_prop",
        "write_all_exact_prefix_prop::exact_prefix",
        "write_all_terminates",
        "write_all_preserves_exact_prefix",
        "write_all_success_is_complete",
        "write_all_preserves_first_error",
        "write_all_all_success",
    ] {
        assert!(
            env.env.transparent_body(env.globals[law]).is_some(),
            "law `{law}` must be a real checked body"
        );
    }
    assert!(!BUFFER_KEN_MD.contains("Axiom"));
    assert!(!IO_KEN_MD.contains("Axiom"));
}

#[test]
fn buffer_span_producer_closure_is_derived_from_public_globals() {
    let mut env = ElabEnv::empty().expect("SPAN-SEAL prelude");
    env.elaborate_ken_md_file(BUFFER_KEN_MD)
        .expect("System.Buffer checked fences");
    env.elaborate_ken_md_file(IO_KEN_MD)
        .expect("System.IO checked fences");
    assert_eq!(public_buffer_span_producers(&env), BTreeSet::new());
}

#[test]
fn buffer_span_producer_closure_reduces_transparent_type_aliases() {
    let mut env = ElabEnv::empty().expect("SPAN-SEAL prelude");
    env.elaborate_file(
        r#"
def BufferSpanAlias = BufferSpan

fn escaped_result_alias (span : BufferSpan) : BufferSpanAlias = span

def BufferSpanFunctionAlias = BufferSpan → BufferSpan

const escaped_function_alias : BufferSpanFunctionAlias = λspan.span
"#,
    )
    .expect("transparent result and whole-function aliases elaborate");

    assert_eq!(
        public_buffer_span_producers(&env),
        BTreeSet::from([
            "escaped_function_alias".to_string(),
            "escaped_result_alias".to_string(),
        ])
    );
}

#[test]
fn buffer_span_producer_closure_resolves_public_constructors() {
    let mut env = ElabEnv::empty().expect("SPAN-SEAL prelude");
    let private_buffer_span = {
        let buffer_span = env
            .env
            .inductive(env.globals["BufferSpan"])
            .expect("BufferSpan inductive");
        assert_eq!(buffer_span.constructors.len(), 1);
        buffer_span.constructors[0].id
    };
    assert_eq!(
        private_buffer_span, env.prelude_env.private_buffer_span_id,
        "discriminator must re-expose the actual sealed constructor"
    );
    assert!(env.env.lookup(private_buffer_span).is_none());
    assert!(env.env.constructor(private_buffer_span).is_some());
    assert!(!env.globals.values().any(|id| *id == private_buffer_span));
    env.globals.insert(
        "escaped_private_buffer_span_constructor".to_string(),
        private_buffer_span,
    );

    assert_eq!(
        public_buffer_span_producers(&env),
        BTreeSet::from(["escaped_private_buffer_span_constructor".to_string()])
    );
}

#[test]
#[should_panic(
    expected = "public global `escaped_unknown_id` is neither a declaration nor a constructor"
)]
fn buffer_span_producer_closure_rejects_unknown_public_ids() {
    let mut env = ElabEnv::empty().expect("SPAN-SEAL prelude");
    env.globals
        .insert("escaped_unknown_id".to_string(), GlobalId(u32::MAX));
    let _ = public_buffer_span_producers(&env);
}

#[test]
fn checked_source_rejects_private_buffer_span_producers() {
    for (private, source) in [
        (
            "PrivateBufferSpan",
            "fn escaped (start : Int) (budget : Nat) : BufferSpan = \
             PrivateBufferSpan start budget",
        ),
        (
            "write_all_advance_span",
            "fn escaped (span : BufferSpan) (count : TransferCount) : BufferSpan = \
             write_all_advance_span span count",
        ),
    ] {
        let mut env = ElabEnv::empty().expect("SPAN-SEAL prelude");
        match (private, env.elaborate_file(source)) {
            ("PrivateBufferSpan", Err(ElabError::UnresolvedCon { name, .. })) => {
                assert_eq!(name, private)
            }
            ("write_all_advance_span", Err(ElabError::UnresolvedCon { name, .. })) => {
                assert_eq!(name, private)
            }
            other => panic!("`{private}` must reject as an unresolved private name, got {other:?}"),
        }
    }
}

/// Durable invariant (`38 §1.7.1`): the public handle is an ordinary product
/// internally, but checked source can neither forge it nor project either
/// acquisition-bound field.
#[test]
fn checked_source_cannot_forge_or_project_buffer_handles() {
    for (private, source) in [
        (
            "PrivateBufferHandle",
            "fn escaped (resource : Resource Buffer) : BufferHandle = \
             PrivateBufferHandle resource 8",
        ),
        (
            "buffer_handle_resource",
            "fn escaped (buffer : BufferHandle) : Resource Buffer = \
             buffer_handle_resource buffer",
        ),
        (
            "buffer_handle_capacity",
            "fn escaped (buffer : BufferHandle) : Int = \
             buffer_handle_capacity buffer",
        ),
    ] {
        let mut env = ElabEnv::empty().expect("PX8-F-CAP-41 prelude");
        match env.elaborate_file(source) {
            Err(ElabError::UnresolvedCon { name, .. }) => assert_eq!(name, private),
            other => panic!("`{private}` must reject as an unresolved private name, got {other:?}"),
        }
    }
}

#[test]
fn positioned_operations_keep_the_landed_wire_identities() {
    assert_eq!(ken_host::HostOpV1::FsReadAt as u16, 0x030D);
    assert_eq!(ken_host::HostOpV1::FsWriteAt as u16, 0x030E);
    assert_eq!(ken_host::HostOpV1::BufferFreeze as u16, 0x0403);
}
