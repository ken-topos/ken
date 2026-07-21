//! PX8-F checked positioned-buffer surface and real structural proof terms.

use std::collections::BTreeSet;

use ken_elaborator::{ElabEnv, ElabError};
use ken_kernel::{Decl, Term};

const BUFFER_KEN_MD: &str =
    include_str!("../../../catalog/packages/Capability/System/Buffer.ken.md");
const IO_KEN_MD: &str = include_str!("../../../catalog/packages/Capability/System/IO.ken.md");

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
  (buffer : Resource Buffer) (window : BufferWindow)
  : HostIO a (Result ResourceError ReadProgress) visits [FS] =
  readAt a file offset buffer window

proc px8f_exact_write
  (a : Auth) (file : Resource FsHandle) (offset : Int)
  (buffer : Resource Buffer) (span : BufferSpan)
  : HostIO a (Result ResourceError WriteProgress) visits [FS] =
  writeAt a file offset buffer span

proc px8f_exact_freeze
  (a : Auth) (buffer : Resource Buffer) (span : BufferSpan)
  : HostIO a (Result ResourceError Bytes) visits [FS] =
  freeze a buffer span

proc px8f_exact_write_all
  (a : Auth) (file : Resource FsHandle) (offset : Int)
  (buffer : Resource Buffer) (span : BufferSpan)
  : HostIO a (Result ResourceError Unit) visits [FS] =
  writeAll a file offset buffer span

proc px8f_readsome_public_consumers
  (a : Auth) (file : Resource FsHandle) (offset : Int)
  (buffer : Resource Buffer) (window : BufferWindow)
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
    let buffer_span = env.globals["BufferSpan"];

    let producers = env
        .globals
        .iter()
        .filter_map(|(name, id)| {
            let decl = env.env.lookup(*id)?;
            let mut ty = match decl {
                Decl::Transparent { ty, .. }
                | Decl::Opaque { ty, .. }
                | Decl::Primitive { ty, .. } => ty,
                Decl::Inductive(inductive) => &inductive.former_type,
            };
            while let Term::Pi(_, codomain) = ty {
                ty = codomain;
            }
            matches!(ty, Term::IndFormer { id, .. } if *id == buffer_span).then_some(name.as_str())
        })
        .collect::<BTreeSet<_>>();

    assert_eq!(producers, BTreeSet::new());
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

#[test]
fn positioned_operations_keep_the_landed_wire_identities() {
    assert_eq!(ken_host::HostOpV1::FsReadAt as u16, 0x030D);
    assert_eq!(ken_host::HostOpV1::FsWriteAt as u16, 0x030E);
    assert_eq!(ken_host::HostOpV1::BufferFreeze as u16, 0x0403);
}
