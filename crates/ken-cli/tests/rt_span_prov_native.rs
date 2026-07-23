//! PX8-SPAN-PROV Phase 2 linked-native provenance discriminator (SP-A).
//!
//! A `BufferSpan` is bound to the exact buffer *acquisition* that minted it
//! (`38 §1.7.1`). `freeze`/`spanBytes` admits a span only against that
//! acquisition; a foreign-acquisition span returns the existing `InvalidBounds`
//! **before** exposing any bytes, even when capacity, start, length, and the
//! live-window shape all match. Only the acquisition differs.
//!
//! The program installs an identical numeric window `[2,6)` in two capacity-8
//! buffers A and B, with distinct bytes (`AAAA` in A, `BBBB` in B), retains
//! `span_a` (origin A) and `span_b` (origin B), then performs `freeze B span_a`
//! (foreign) followed by `freeze B span_b` (own-span control). It discards both
//! results in-program (keeping the compiled function small) and the test asserts
//! the exact canonical outcomes from the effect trace, on **both** executors:
//! the foreign freeze is `InvalidBounds` and exposes no bytes; the own freeze
//! returns exactly `BBBB`. Reverting the dispatcher's `span_origin == target`
//! check flips the foreign trace to `Success(Bytes …)` and reddens this (AC-8).

#[cfg(target_os = "linux")]
struct Differential {
    interpreted: ken_runtime::EffectObservation,
    native: ken_runtime::EffectObservation,
}

#[cfg(target_os = "linux")]
fn output_dir(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "ken-spanprov-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&path).unwrap();
    path
}

/// Compile `source` to a linked native artifact, run it, then run the identical
/// source through the reference interpreter against the same root. A
/// `spanseed.bin` file holding `AAAABBBB` is present so buffer A installs `AAAA`
/// (from offset 0) and buffer B installs `BBBB` (from offset 4) at window
/// `[2,6)`.
#[cfg(target_os = "linux")]
fn differential(case: &str, source: &str) -> Differential {
    let root = output_dir(case);
    std::fs::write(root.join("spanseed.bin"), b"AAAABBBB").unwrap();

    let output = ken_cli::build_native_program(
        source,
        ken_cli::SourceFormat::Ken,
        &format!("rt_span_prov_{}", case.replace('-', "_")),
        &root,
    )
    .unwrap_or_else(|error| panic!("{case}: reaches linked native lowering: {error:?}"));
    let native = ken_runtime::run_bound_process_effect_observation(
        &output.artifact,
        &ken_runtime::NativeEffectRunOptionsV1 {
            arguments: Vec::new(),
            environment: Vec::new(),
            cwd: root.clone(),
            plan_hash: output.plan_transport_hash,
        },
    )
    .unwrap_or_else(|error| panic!("{case}: linked artifact runs: {error:?}"));

    let mut host = ken_interp::PosixHost::new_at(&root);
    let interpreted = ken_cli::run_program_effect_observation(
        source,
        ken_cli::SourceFormat::Ken,
        &[],
        &[],
        root.as_os_str().as_encoded_bytes(),
        &mut host,
    )
    .unwrap_or_else(|error| panic!("{case}: source runs in interpreter: {error:?}"));

    std::fs::remove_dir_all(&root).unwrap();
    Differential {
        interpreted,
        native,
    }
}

/// Run `source` through the reference interpreter only (no native lowering).
/// Used where the end-to-end program requires four nested resource brackets
/// (readable source + writable dest + two buffers) — which currently exceeds the
/// Cranelift backend's per-function code-size limit ("Code for function is too
/// large"), so the native executor cannot lower it. Three-bracket programs
/// (e.g. SP-A freeze) lower and run on both executors.
#[cfg(target_os = "linux")]
fn interpret_only(case: &str, source: &str) -> ken_runtime::EffectObservation {
    let root = output_dir(case);
    std::fs::write(root.join("spanseed.bin"), b"AAAABBBB").unwrap();
    let mut host = ken_interp::PosixHost::new_at(&root);
    let interpreted = ken_cli::run_program_effect_observation(
        source,
        ken_cli::SourceFormat::Ken,
        &[],
        &[],
        root.as_os_str().as_encoded_bytes(),
        &mut host,
    )
    .unwrap_or_else(|error| panic!("{case}: interpreter run: {error:?}"));
    std::fs::remove_dir_all(&root).unwrap();
    interpreted
}

/// Like `interpret_only`, but reads the named files from the run root *before*
/// teardown so a test can assert backend effects (destination contents). A
/// missing file reads as empty — the row's "zero backend / destination empty"
/// observation for a rejected write. `spanseed.bin` (AAAABBBB) is present.
#[cfg(target_os = "linux")]
fn interpret_reading(
    case: &str,
    source: &str,
    reads: &[&str],
) -> (ken_runtime::EffectObservation, Vec<Vec<u8>>) {
    let root = output_dir(case);
    std::fs::write(root.join("spanseed.bin"), b"AAAABBBB").unwrap();
    let mut host = ken_interp::PosixHost::new_at(&root);
    let interpreted = ken_cli::run_program_effect_observation(
        source,
        ken_cli::SourceFormat::Ken,
        &[],
        &[],
        root.as_os_str().as_encoded_bytes(),
        &mut host,
    )
    .unwrap_or_else(|error| panic!("{case}: interpreter run: {error:?}"));
    let contents = reads
        .iter()
        .map(|name| std::fs::read(root.join(name)).unwrap_or_default())
        .collect();
    std::fs::remove_dir_all(&root).unwrap();
    (interpreted, contents)
}

/// The ordered canonical outcomes of every `BufferFreeze` in an observation.
#[cfg(target_os = "linux")]
fn buffer_freeze_outcomes(
    observation: &ken_runtime::EffectObservation,
) -> Vec<ken_runtime::CanonicalOutcomeV1> {
    observation
        .effect_trace
        .iter()
        .filter(|event| event.operation == ken_runtime::HostOpV1::BufferFreeze)
        .map(|event| event.outcome.clone())
        .collect()
}

/// The ordered canonical outcomes of every `FsWriteAt` in an observation.
#[cfg(target_os = "linux")]
fn write_outcomes(
    observation: &ken_runtime::EffectObservation,
) -> Vec<ken_runtime::CanonicalOutcomeV1> {
    observation
        .effect_trace
        .iter()
        .filter(|event| event.operation == ken_runtime::HostOpV1::FsWriteAt)
        .map(|event| event.outcome.clone())
        .collect()
}

/// Assert one executor's freeze sequence is exactly `[foreign InvalidBounds, own
/// Bytes "BBBB"]`.
#[cfg(target_os = "linux")]
fn assert_freeze_sequence(case: &str, engine: &str, freezes: &[ken_runtime::CanonicalOutcomeV1]) {
    assert_eq!(
        freezes.len(),
        2,
        "{case}/{engine}: expected exactly two BufferFreeze events (foreign then own), got {freezes:?}"
    );
    match &freezes[0] {
        ken_runtime::CanonicalOutcomeV1::Error(ken_runtime::SemanticErrorV1::Resource(
            ken_runtime::ResourceErrorV1::InvalidBounds,
        )) => {}
        other => panic!(
            "{case}/{engine}: foreign-acquisition freeze must be InvalidBounds with no bytes, got {other:?}"
        ),
    }
    match &freezes[1] {
        ken_runtime::CanonicalOutcomeV1::Success(ken_runtime::CanonicalReplyV1::Bytes(bytes))
            if bytes.as_slice() == b"BBBB" => {}
        other => panic!("{case}/{engine}: own-acquisition freeze must return BBBB, got {other:?}"),
    }
}

// Two capacity-8 buffers with an identical numeric window [2,6) but distinct
// bytes; a foreign then an own freeze on B, both results discarded in-program.
#[cfg(target_os = "linux")]
const SP_A_FREEZE: &str = r#"program capabilities FS AFull
fn ok_body (unit : Unit) : ResourceBodyResult Unit Unit = ResourceBodyOk Unit Unit MkUnit

fn body_from_alloc (outcome : Result ResourceError (ResourceBracketResult Unit Unit))
  : ResourceBodyResult Unit Unit =
  match outcome {
    Err error |-> ResourceBodyErr Unit Unit MkUnit;
    Ok bracket |-> match bracket {
      ResourceBracketOk value |-> ResourceBodyOk Unit Unit MkUnit;
      ResourceBracketBodyError error |-> ResourceBodyErr Unit Unit MkUnit;
      ResourceBracketReleaseError error |-> ResourceBodyErr Unit Unit MkUnit;
      ResourceBracketBodyAndReleaseError body_error release_error |->
        ResourceBodyErr Unit Unit MkUnit
    }
  }

proc do_freezes (buffer_b : Resource Buffer) (span_a : BufferSpan) (span_b : BufferSpan)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError Bytes) (ResourceBodyResult Unit Unit)
    (freeze AFull buffer_b span_a)
    (\ra. bind (Coproduct (FSOp AFull) AmbientOp)
      (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
      (Result ResourceError Bytes) (ResourceBodyResult Unit Unit)
      (freeze AFull buffer_b span_b)
      (\rb. Ret (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (ResourceBodyResult Unit Unit) (ok_body MkUnit)))

proc b_after_read (buffer_b : Resource Buffer) (span_a : BufferSpan)
  (outcome : Result ResourceError ReadProgress)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  match outcome {
    Err error |-> Ret (Coproduct (FSOp AFull) AmbientOp)
      (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
      (ResourceBodyResult Unit Unit) (ResourceBodyErr Unit Unit MkUnit);
    Ok progress |-> match progress {
      ReadEof |-> Ret (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (ResourceBodyResult Unit Unit) (ResourceBodyErr Unit Unit MkUnit);
      ReadSome span_b count |-> do_freezes buffer_b span_a span_b
    }
  }

proc b_body (file : Resource FsHandle) (span_a : BufferSpan) (buffer_b : Resource Buffer)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError ReadProgress) (ResourceBodyResult Unit Unit)
    (readAt AFull file (4 : Int) buffer_b (MkBufferWindow (2 : Int) (4 : Int)))
    (\outcome. b_after_read buffer_b span_a outcome)

proc a_after_read (file : Resource FsHandle) (span_a : BufferSpan)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError (ResourceBracketResult Unit Unit)) (ResourceBodyResult Unit Unit)
    (withBuffer AFull Unit Unit (8 : Int) (b_body file span_a))
    (\outcome. Ret (Coproduct (FSOp AFull) AmbientOp)
      (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
      (ResourceBodyResult Unit Unit) (body_from_alloc outcome))

proc a_body (file : Resource FsHandle) (buffer_a : Resource Buffer)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError ReadProgress) (ResourceBodyResult Unit Unit)
    (readAt AFull file (0 : Int) buffer_a (MkBufferWindow (2 : Int) (4 : Int)))
    (\outcome. match outcome {
      Err error |-> Ret (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (ResourceBodyResult Unit Unit) (ResourceBodyErr Unit Unit MkUnit);
      Ok progress |-> match progress {
        ReadEof |-> Ret (Coproduct (FSOp AFull) AmbientOp)
          (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
          (ResourceBodyResult Unit Unit) (ResourceBodyErr Unit Unit MkUnit);
        ReadSome span_a count |-> a_after_read file span_a
      }
    })

proc file_body (file : Resource FsHandle)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError (ResourceBracketResult Unit Unit)) (ResourceBodyResult Unit Unit)
    (withBuffer AFull Unit Unit (8 : Int) (a_body file))
    (\outcome. Ret (Coproduct (FSOp AFull) AmbientOp)
      (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
      (ResourceBodyResult Unit Unit) (body_from_alloc outcome))

fn finish (outcome : Result FileError (ResourceBracketResult Unit Unit)) : HostIO AFull ExitCode =
  match outcome {
    Err error |-> host_exit AFull (Failure 81);
    Ok bracket |-> match bracket {
      ResourceBracketOk value |-> host_exit AFull Success;
      ResourceBracketBodyError error |-> host_exit AFull (Failure 82);
      ResourceBracketReleaseError error |-> host_exit AFull (Failure 83);
      ResourceBracketBodyAndReleaseError body_error release_error |-> host_exit AFull (Failure 84)
    }
  }

proc main (_input : ProcessInput) (caps : ProgramCaps AFull)
  : HostIO AFull ExitCode visits [FS] =
  match caps {
    MkProgramCaps cap |->
      bind (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (Result FileError (ResourceBracketResult Unit Unit)) ExitCode
        (withResource AFull Unit Unit cap (bytes_encode "spanseed.bin") ResourceRead file_body)
        (\outcome. finish outcome)
  }
"#;

/// Nested resource brackets drive deep evaluator recursion during native
/// lowering; run under a large stack like the other linked-native buffer tests.
#[cfg(target_os = "linux")]
fn in_large_stack_thread(name: &'static str, body: fn()) {
    std::thread::Builder::new()
        .name(name.to_string())
        .stack_size(256 * 1024 * 1024)
        .spawn(body)
        .unwrap()
        .join()
        .unwrap();
}

#[cfg(target_os = "linux")]
#[test]
fn sp_a_foreign_span_freeze_rejects_own_span_succeeds_on_both_engines() {
    in_large_stack_thread("sp-a-freeze", || {
        let diff = differential("sp-a-freeze", SP_A_FREEZE);
        // Both executors settle their brackets (exit 0) and agree.
        assert_eq!(
            diff.native.exit_status, diff.interpreted.exit_status,
            "sp-a-freeze: exit status must agree; native={:?} interp={:?}",
            diff.native, diff.interpreted
        );
        assert_eq!(diff.interpreted.exit_status, 0, "sp-a-freeze: interpreter exits Success");
        assert_eq!(diff.native.exit_status, 0, "sp-a-freeze: native exits Success");
        // The load-bearing assertion: exact foreign/own freeze outcomes.
        let native = buffer_freeze_outcomes(&diff.native);
        let interp = buffer_freeze_outcomes(&diff.interpreted);
        assert_eq!(
            native, interp,
            "sp-a-freeze: native and interpreter freeze outcomes must agree"
        );
        assert_freeze_sequence("sp-a-freeze", "native", &native);
        assert_freeze_sequence("sp-a-freeze", "interpreter", &interp);
    });
}

// SP-A write consumer (foreign arm). A minimal 4-bracket program: read span_a
// from buffer A, then `writeAt dest B span_a` — a foreign-acquisition span on
// target B. B is never read: a foreign write is rejected on the shared-host
// provenance check (after host-width admission) BEFORE `initialized_slice` or any
// backend write, so B needs no live window. The trace must show exactly one
// FsWriteAt = InvalidBounds on both executors.
#[cfg(target_os = "linux")]
const SP_A_WRITE_FOREIGN: &str = r#"program capabilities FS AFull
fn ok_body (unit : Unit) : ResourceBodyResult Unit Unit = ResourceBodyOk Unit Unit MkUnit

fn from_buffer_alloc (outcome : Result ResourceError (ResourceBracketResult Unit Unit))
  : ResourceBodyResult Unit Unit =
  match outcome {
    Err error |-> ResourceBodyErr Unit Unit MkUnit;
    Ok bracket |-> match bracket {
      ResourceBracketOk value |-> ResourceBodyOk Unit Unit MkUnit;
      ResourceBracketBodyError error |-> ResourceBodyErr Unit Unit MkUnit;
      ResourceBracketReleaseError error |-> ResourceBodyErr Unit Unit MkUnit;
      ResourceBracketBodyAndReleaseError be re |-> ResourceBodyErr Unit Unit MkUnit
    }
  }

fn from_file_alloc (outcome : Result FileError (ResourceBracketResult Unit Unit))
  : ResourceBodyResult Unit Unit =
  match outcome {
    Err error |-> ResourceBodyErr Unit Unit MkUnit;
    Ok bracket |-> match bracket {
      ResourceBracketOk value |-> ResourceBodyOk Unit Unit MkUnit;
      ResourceBracketBodyError error |-> ResourceBodyErr Unit Unit MkUnit;
      ResourceBracketReleaseError error |-> ResourceBodyErr Unit Unit MkUnit;
      ResourceBracketBodyAndReleaseError be re |-> ResourceBodyErr Unit Unit MkUnit
    }
  }

proc b_body (dest : Resource FsHandle) (span_a : BufferSpan) (buffer_b : Resource Buffer)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError WriteProgress) (ResourceBodyResult Unit Unit)
    (writeAt AFull dest (0 : Int) buffer_b span_a)
    (\w. Ret (Coproduct (FSOp AFull) AmbientOp)
      (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
      (ResourceBodyResult Unit Unit) (ok_body MkUnit))

proc a_after_read (dest : Resource FsHandle) (span_a : BufferSpan)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError (ResourceBracketResult Unit Unit)) (ResourceBodyResult Unit Unit)
    (withBuffer AFull Unit Unit (8 : Int) (b_body dest span_a))
    (\outcome. Ret (Coproduct (FSOp AFull) AmbientOp)
      (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
      (ResourceBodyResult Unit Unit) (from_buffer_alloc outcome))

proc a_body (dest : Resource FsHandle) (source : Resource FsHandle) (buffer_a : Resource Buffer)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError ReadProgress) (ResourceBodyResult Unit Unit)
    (readAt AFull source (0 : Int) buffer_a (MkBufferWindow (2 : Int) (4 : Int)))
    (\outcome. match outcome {
      Err error |-> Ret (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (ResourceBodyResult Unit Unit) (ResourceBodyErr Unit Unit MkUnit);
      Ok progress |-> match progress {
        ReadEof |-> Ret (Coproduct (FSOp AFull) AmbientOp)
          (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
          (ResourceBodyResult Unit Unit) (ResourceBodyErr Unit Unit MkUnit);
        ReadSome span_a count |-> a_after_read dest span_a
      }
    })

proc source_body (dest : Resource FsHandle) (source : Resource FsHandle)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError (ResourceBracketResult Unit Unit)) (ResourceBodyResult Unit Unit)
    (withBuffer AFull Unit Unit (8 : Int) (a_body dest source))
    (\outcome. Ret (Coproduct (FSOp AFull) AmbientOp)
      (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
      (ResourceBodyResult Unit Unit) (from_buffer_alloc outcome))

proc dest_body (cap : Cap AFull) (dest : Resource FsHandle)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result FileError (ResourceBracketResult Unit Unit)) (ResourceBodyResult Unit Unit)
    (withResource AFull Unit Unit cap (bytes_encode "spanseed.bin") ResourceRead
      (source_body dest))
    (\outcome. Ret (Coproduct (FSOp AFull) AmbientOp)
      (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
      (ResourceBodyResult Unit Unit) (from_file_alloc outcome))

fn finish (outcome : Result FileError (ResourceBracketResult Unit Unit)) : HostIO AFull ExitCode =
  match outcome {
    Err error |-> host_exit AFull (Failure 81);
    Ok bracket |-> match bracket {
      ResourceBracketOk value |-> host_exit AFull Success;
      ResourceBracketBodyError error |-> host_exit AFull (Failure 82);
      ResourceBracketReleaseError error |-> host_exit AFull (Failure 83);
      ResourceBracketBodyAndReleaseError body_error release_error |-> host_exit AFull (Failure 84)
    }
  }

proc main (_input : ProcessInput) (caps : ProgramCaps AFull)
  : HostIO AFull ExitCode visits [FS] =
  match caps {
    MkProgramCaps cap |->
      bind (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (Result FileError (ResourceBracketResult Unit Unit)) ExitCode
        (withResource AFull Unit Unit cap (bytes_encode "dest.bin")
          (ResourceWriteCreate CreateOrTruncate) (dest_body cap))
        (\outcome. finish outcome)
  }
"#;

// Interpreter end-to-end for the write consumer's foreign arm. The equivalent
// native discriminator is blocked by the four-bracket code-size limit (see
// `interpret_only`); the write consumer's native span-origin ABI is exercised by
// the own-buffer `px8f_buffer_native`/`px8f_write_partition` writeAll tests, and
// the shared-host foreign-write rejection (+ zero backend) is proven in
// `ken_host::effect_v1::tests::foreign_acquisition_span_rejects_on_both_consumers_before_bytes_or_backend`.
#[cfg(target_os = "linux")]
#[test]
fn sp_a_foreign_span_write_rejects_before_backend_interp() {
    in_large_stack_thread("sp-a-write-foreign-interp", || {
        let (obs, files) =
            interpret_reading("sp-a-write-foreign", SP_A_WRITE_FOREIGN, &["dest.bin"]);
        assert_eq!(
            write_outcomes(&obs),
            vec![ken_runtime::CanonicalOutcomeV1::Error(
                ken_runtime::SemanticErrorV1::Resource(ken_runtime::ResourceErrorV1::InvalidBounds),
            )],
            "interp: foreign-acquisition write must be exactly one InvalidBounds"
        );
        assert!(
            files[0].is_empty(),
            "interp: foreign write must issue zero backend writes — destination stays empty, got {:?}",
            files[0]
        );
    });
}

// SP-A write consumer, own arm — interpreter e2e. A three-bracket program:
// readAt installs BBBB in B [2,6), then `writeAt dest 0 B span_b` (own span)
// succeeds. The test asserts the destination file contains exactly BBBB — the
// row's "one backend call at offset 0 carrying BBBB, destination BBBB". The
// foreign arm's zero-backend/empty-destination is asserted separately above.
#[cfg(target_os = "linux")]
const SP_A_WRITE_OWN: &str = r#"program capabilities FS AFull
fn ok_body (unit : Unit) : ResourceBodyResult Unit Unit = ResourceBodyOk Unit Unit MkUnit

fn from_buffer_alloc (outcome : Result ResourceError (ResourceBracketResult Unit Unit))
  : ResourceBodyResult Unit Unit =
  match outcome {
    Err error |-> ResourceBodyErr Unit Unit MkUnit;
    Ok bracket |-> match bracket {
      ResourceBracketOk value |-> ResourceBodyOk Unit Unit MkUnit;
      ResourceBracketBodyError error |-> ResourceBodyErr Unit Unit MkUnit;
      ResourceBracketReleaseError error |-> ResourceBodyErr Unit Unit MkUnit;
      ResourceBracketBodyAndReleaseError be re |-> ResourceBodyErr Unit Unit MkUnit
    }
  }

fn from_file_alloc (outcome : Result FileError (ResourceBracketResult Unit Unit))
  : ResourceBodyResult Unit Unit =
  match outcome {
    Err error |-> ResourceBodyErr Unit Unit MkUnit;
    Ok bracket |-> match bracket {
      ResourceBracketOk value |-> ResourceBodyOk Unit Unit MkUnit;
      ResourceBracketBodyError error |-> ResourceBodyErr Unit Unit MkUnit;
      ResourceBracketReleaseError error |-> ResourceBodyErr Unit Unit MkUnit;
      ResourceBracketBodyAndReleaseError be re |-> ResourceBodyErr Unit Unit MkUnit
    }
  }

proc do_write (dest : Resource FsHandle) (buffer_b : Resource Buffer) (span_b : BufferSpan)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError WriteProgress) (ResourceBodyResult Unit Unit)
    (writeAt AFull dest (0 : Int) buffer_b span_b)
    (\w. Ret (Coproduct (FSOp AFull) AmbientOp)
      (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
      (ResourceBodyResult Unit Unit) (ok_body MkUnit))

proc b_body (dest : Resource FsHandle) (source : Resource FsHandle) (buffer_b : Resource Buffer)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError ReadProgress) (ResourceBodyResult Unit Unit)
    (readAt AFull source (4 : Int) buffer_b (MkBufferWindow (2 : Int) (4 : Int)))
    (\outcome. match outcome {
      Err error |-> Ret (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (ResourceBodyResult Unit Unit) (ResourceBodyErr Unit Unit MkUnit);
      Ok progress |-> match progress {
        ReadEof |-> Ret (Coproduct (FSOp AFull) AmbientOp)
          (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
          (ResourceBodyResult Unit Unit) (ResourceBodyErr Unit Unit MkUnit);
        ReadSome span_b count |-> do_write dest buffer_b span_b
      }
    })

proc source_body (dest : Resource FsHandle) (source : Resource FsHandle)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError (ResourceBracketResult Unit Unit)) (ResourceBodyResult Unit Unit)
    (withBuffer AFull Unit Unit (8 : Int) (b_body dest source))
    (\outcome. Ret (Coproduct (FSOp AFull) AmbientOp)
      (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
      (ResourceBodyResult Unit Unit) (from_buffer_alloc outcome))

proc dest_body (cap : Cap AFull) (dest : Resource FsHandle)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result FileError (ResourceBracketResult Unit Unit)) (ResourceBodyResult Unit Unit)
    (withResource AFull Unit Unit cap (bytes_encode "spanseed.bin") ResourceRead
      (source_body dest))
    (\outcome. Ret (Coproduct (FSOp AFull) AmbientOp)
      (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
      (ResourceBodyResult Unit Unit) (from_file_alloc outcome))

fn finish (outcome : Result FileError (ResourceBracketResult Unit Unit)) : HostIO AFull ExitCode =
  match outcome {
    Err error |-> host_exit AFull (Failure 81);
    Ok bracket |-> match bracket {
      ResourceBracketOk value |-> host_exit AFull Success;
      ResourceBracketBodyError error |-> host_exit AFull (Failure 82);
      ResourceBracketReleaseError error |-> host_exit AFull (Failure 83);
      ResourceBracketBodyAndReleaseError body_error release_error |-> host_exit AFull (Failure 84)
    }
  }

proc main (_input : ProcessInput) (caps : ProgramCaps AFull)
  : HostIO AFull ExitCode visits [FS] =
  match caps {
    MkProgramCaps cap |->
      bind (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (Result FileError (ResourceBracketResult Unit Unit)) ExitCode
        (withResource AFull Unit Unit cap (bytes_encode "dest.bin")
          (ResourceWriteCreate CreateOrTruncate) (dest_body cap))
        (\outcome. finish outcome)
  }
"#;

#[cfg(target_os = "linux")]
#[test]
fn sp_a_own_span_write_succeeds_with_bytes_interp() {
    in_large_stack_thread("sp-a-write-own-interp", || {
        let (obs, files) = interpret_reading("sp-a-write-own", SP_A_WRITE_OWN, &["dest.bin"]);
        let writes = write_outcomes(&obs);
        assert_eq!(writes.len(), 1, "expected one FsWriteAt event, got {writes:?}");
        match &writes[0] {
            ken_runtime::CanonicalOutcomeV1::Success(
                ken_runtime::CanonicalReplyV1::WriteProgress(_),
            ) => {}
            other => panic!("interp: own write must succeed (Wrote), got {other:?}"),
        }
        assert_eq!(
            files[0], b"BBBB",
            "interp: own write must land exactly BBBB at offset 0 (one backend call), got {:?}",
            files[0]
        );
    });
}


// SP-C non-revival — interpreter e2e. Buffer A mints span_a and is released; B is
// then acquired (reusing A's vacated slot with a newer generation) and installs
// BBBB. The old acquisition's span (span_a) applied to B is rejected with
// InvalidBounds; a fresh span (span_b) from B succeeds. Native half is
// BLOCKED-ON-NATIVE-REACHABILITY. (Freeze arm shown; the write arm is covered by
// SP-A-write's foreign/own controls + the shared-host slot-reuse unit test.)
#[cfg(target_os = "linux")]
const SP_C_FREEZE: &str = r#"program capabilities FS AFull
fn ok_body (unit : Unit) : ResourceBodyResult Unit Unit = ResourceBodyOk Unit Unit MkUnit

fn escape_span (span : BufferSpan) : ResourceBodyResult Unit BufferSpan =
  ResourceBodyOk Unit BufferSpan span

fn from_span_bracket (inner : Result ResourceError (ResourceBracketResult Unit Unit))
  : ResourceBodyResult Unit Unit =
  match inner {
    Err error |-> ResourceBodyErr Unit Unit MkUnit;
    Ok bracket |-> match bracket {
      ResourceBracketOk value |-> ResourceBodyOk Unit Unit MkUnit;
      ResourceBracketBodyError error |-> ResourceBodyErr Unit Unit MkUnit;
      ResourceBracketReleaseError error |-> ResourceBodyErr Unit Unit MkUnit;
      ResourceBracketBodyAndReleaseError be re |-> ResourceBodyErr Unit Unit MkUnit
    }
  }

proc do_freezes (buffer_b : Resource Buffer) (span_a : BufferSpan) (span_b : BufferSpan)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError Bytes) (ResourceBodyResult Unit Unit)
    (freeze AFull buffer_b span_a)
    (\ra. bind (Coproduct (FSOp AFull) AmbientOp)
      (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
      (Result ResourceError Bytes) (ResourceBodyResult Unit Unit)
      (freeze AFull buffer_b span_b)
      (\rb. Ret (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (ResourceBodyResult Unit Unit) (ok_body MkUnit)))

proc b_body (source : Resource FsHandle) (span_a : BufferSpan) (buffer_b : Resource Buffer)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError ReadProgress) (ResourceBodyResult Unit Unit)
    (readAt AFull source (4 : Int) buffer_b (MkBufferWindow (2 : Int) (4 : Int)))
    (\outcome. match outcome {
      Err error |-> Ret (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (ResourceBodyResult Unit Unit) (ResourceBodyErr Unit Unit MkUnit);
      Ok progress |-> match progress {
        ReadEof |-> Ret (Coproduct (FSOp AFull) AmbientOp)
          (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
          (ResourceBodyResult Unit Unit) (ResourceBodyErr Unit Unit MkUnit);
        ReadSome span_b count |-> do_freezes buffer_b span_a span_b
      }
    })

proc after_a (source : Resource FsHandle)
  (inner : Result ResourceError (ResourceBracketResult Unit BufferSpan))
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  match inner {
    Err error |-> Ret (Coproduct (FSOp AFull) AmbientOp)
      (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
      (ResourceBodyResult Unit Unit) (ResourceBodyErr Unit Unit MkUnit);
    Ok bracket |-> match bracket {
      ResourceBracketOk span_a |->
        bind (Coproduct (FSOp AFull) AmbientOp)
          (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
          (Result ResourceError (ResourceBracketResult Unit Unit)) (ResourceBodyResult Unit Unit)
          (withBuffer AFull Unit Unit (8 : Int) (b_body source span_a))
          (\outcome. Ret (Coproduct (FSOp AFull) AmbientOp)
            (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
            (ResourceBodyResult Unit Unit) (from_span_bracket outcome));
      ResourceBracketBodyError error |-> Ret (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (ResourceBodyResult Unit Unit) (ResourceBodyErr Unit Unit MkUnit);
      ResourceBracketReleaseError error |-> Ret (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (ResourceBodyResult Unit Unit) (ResourceBodyErr Unit Unit MkUnit);
      ResourceBracketBodyAndReleaseError be re |-> Ret (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (ResourceBodyResult Unit Unit) (ResourceBodyErr Unit Unit MkUnit)
    }
  }

proc read_a (source : Resource FsHandle) (buffer_a : Resource Buffer)
  : HostIO AFull (ResourceBodyResult Unit BufferSpan) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError ReadProgress) (ResourceBodyResult Unit BufferSpan)
    (readAt AFull source (0 : Int) buffer_a (MkBufferWindow (2 : Int) (4 : Int)))
    (\outcome. match outcome {
      Err error |-> Ret (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (ResourceBodyResult Unit BufferSpan) (ResourceBodyErr Unit BufferSpan MkUnit);
      Ok progress |-> match progress {
        ReadEof |-> Ret (Coproduct (FSOp AFull) AmbientOp)
          (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
          (ResourceBodyResult Unit BufferSpan) (ResourceBodyErr Unit BufferSpan MkUnit);
        ReadSome span_a count |-> Ret (Coproduct (FSOp AFull) AmbientOp)
          (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
          (ResourceBodyResult Unit BufferSpan) (escape_span span_a)
      }
    })

proc source_body (source : Resource FsHandle)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError (ResourceBracketResult Unit BufferSpan)) (ResourceBodyResult Unit Unit)
    (withBuffer AFull Unit BufferSpan (8 : Int) (read_a source))
    (\outcome. after_a source outcome)

fn finish (outcome : Result FileError (ResourceBracketResult Unit Unit)) : HostIO AFull ExitCode =
  match outcome {
    Err error |-> host_exit AFull (Failure 81);
    Ok bracket |-> match bracket {
      ResourceBracketOk value |-> host_exit AFull Success;
      ResourceBracketBodyError error |-> host_exit AFull (Failure 82);
      ResourceBracketReleaseError error |-> host_exit AFull (Failure 83);
      ResourceBracketBodyAndReleaseError body_error release_error |-> host_exit AFull (Failure 84)
    }
  }

proc main (_input : ProcessInput) (caps : ProgramCaps AFull)
  : HostIO AFull ExitCode visits [FS] =
  match caps {
    MkProgramCaps cap |->
      bind (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (Result FileError (ResourceBracketResult Unit Unit)) ExitCode
        (withResource AFull Unit Unit cap (bytes_encode "spanseed.bin") ResourceRead
          source_body)
        (\outcome. finish outcome)
  }
"#;

#[cfg(target_os = "linux")]
#[test]
fn sp_c_released_span_not_revived_by_slot_reuse_interp() {
    in_large_stack_thread("sp-c-freeze-interp", || {
        let obs = interpret_only("sp-c-freeze", SP_C_FREEZE);
        let freezes = buffer_freeze_outcomes(&obs);
        assert_eq!(freezes.len(), 2, "expected two BufferFreeze events, got {freezes:?}");
        match &freezes[0] {
            ken_runtime::CanonicalOutcomeV1::Error(ken_runtime::SemanticErrorV1::Resource(
                ken_runtime::ResourceErrorV1::InvalidBounds,
            )) => {}
            other => panic!("interp: released-acquisition span must be InvalidBounds, got {other:?}"),
        }
        match &freezes[1] {
            ken_runtime::CanonicalOutcomeV1::Success(ken_runtime::CanonicalReplyV1::Bytes(b))
                if b.as_slice() == b"BBBB" => {}
            other => panic!("interp: fresh span must freeze to BBBB, got {other:?}"),
        }
    });
}

// SP-B precedence — interpreter e2e. A foreign-acquisition span on target B: with
// a valid file offset the span-validity relation returns InvalidBounds; changing
// only the file offset to -1 returns the already-locked InvalidOffset at
// host-width admission (which precedes the provenance check). Observed via exit
// code (the -1 offset fails at request narrowing, before dispatch, so it is not
// in the effect trace). Exit 0 iff [foreign@0 -> InvalidBounds, foreign@-1 ->
// InvalidOffset]. Native half is BLOCKED-ON-NATIVE-REACHABILITY.
#[cfg(target_os = "linux")]
const SP_B_PRECEDENCE: &str = r#"program capabilities FS AFull
fn is_invalid_bounds (e : ResourceError) : Bool =
  match e {
    ResourceHostIO io |-> False; Closed |-> False; MalformedResource |-> False;
    RightNotHeld required held |-> False; ReleaseFailed kind identity io |-> False;
    ResourceKindMismatch expected actual |-> False; BufferLimit |-> False;
    InvalidOffset |-> False; InvalidBounds |-> True; NoProgress |-> False
  }

fn is_invalid_offset (e : ResourceError) : Bool =
  match e {
    ResourceHostIO io |-> False; Closed |-> False; MalformedResource |-> False;
    RightNotHeld required held |-> False; ReleaseFailed kind identity io |-> False;
    ResourceKindMismatch expected actual |-> False; BufferLimit |-> False;
    InvalidOffset |-> True; InvalidBounds |-> False; NoProgress |-> False
  }

fn ok_code (code : ExitCode) : ResourceBodyResult Unit ExitCode =
  ResourceBodyOk Unit ExitCode code

fn from_buffer_alloc (outcome : Result ResourceError (ResourceBracketResult Unit ExitCode))
  : ResourceBodyResult Unit ExitCode =
  match outcome {
    Err error |-> ResourceBodyOk Unit ExitCode (Failure 80);
    Ok bracket |-> match bracket {
      ResourceBracketOk code |-> ResourceBodyOk Unit ExitCode code;
      ResourceBracketBodyError error |-> ResourceBodyOk Unit ExitCode (Failure 81);
      ResourceBracketReleaseError error |-> ResourceBodyOk Unit ExitCode (Failure 82);
      ResourceBracketBodyAndReleaseError be re |-> ResourceBodyOk Unit ExitCode (Failure 83)
    }
  }

fn from_file_alloc (outcome : Result FileError (ResourceBracketResult Unit ExitCode))
  : ResourceBodyResult Unit ExitCode =
  match outcome {
    Err error |-> ResourceBodyOk Unit ExitCode (Failure 84);
    Ok bracket |-> match bracket {
      ResourceBracketOk code |-> ResourceBodyOk Unit ExitCode code;
      ResourceBracketBodyError error |-> ResourceBodyOk Unit ExitCode (Failure 85);
      ResourceBracketReleaseError error |-> ResourceBodyOk Unit ExitCode (Failure 86);
      ResourceBracketBodyAndReleaseError be re |-> ResourceBodyOk Unit ExitCode (Failure 87)
    }
  }

proc verdict (dest : Resource FsHandle) (buffer_b : Resource Buffer) (span_a : BufferSpan)
  : HostIO AFull (ResourceBodyResult Unit ExitCode) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError WriteProgress) (ResourceBodyResult Unit ExitCode)
    (writeAt AFull dest (0 : Int) buffer_b span_a)
    (\valid. match valid {
      Ok progress |-> Ret (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (ResourceBodyResult Unit ExitCode) (ok_code (Failure 71));
      Err eb |-> match is_invalid_bounds eb {
        False |-> Ret (Coproduct (FSOp AFull) AmbientOp)
          (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
          (ResourceBodyResult Unit ExitCode) (ok_code (Failure 73));
        True |-> bind (Coproduct (FSOp AFull) AmbientOp)
          (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
          (Result ResourceError WriteProgress) (ResourceBodyResult Unit ExitCode)
          (writeAt AFull dest (sub_int 0 1) buffer_b span_a)
          (\neg. match neg {
            Ok progress |-> Ret (Coproduct (FSOp AFull) AmbientOp)
              (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
              (ResourceBodyResult Unit ExitCode) (ok_code (Failure 72));
            Err eo |-> match is_invalid_offset eo {
              True |-> Ret (Coproduct (FSOp AFull) AmbientOp)
                (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
                (ResourceBodyResult Unit ExitCode) (ok_code Success);
              False |-> Ret (Coproduct (FSOp AFull) AmbientOp)
                (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
                (ResourceBodyResult Unit ExitCode) (ok_code (Failure 74))
            }
          })
      }
    })

proc b_body (dest : Resource FsHandle) (span_a : BufferSpan) (buffer_b : Resource Buffer)
  : HostIO AFull (ResourceBodyResult Unit ExitCode) visits [FS] =
  verdict dest buffer_b span_a

proc a_after_read (dest : Resource FsHandle) (span_a : BufferSpan)
  : HostIO AFull (ResourceBodyResult Unit ExitCode) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError (ResourceBracketResult Unit ExitCode)) (ResourceBodyResult Unit ExitCode)
    (withBuffer AFull Unit ExitCode (8 : Int) (b_body dest span_a))
    (\outcome. Ret (Coproduct (FSOp AFull) AmbientOp)
      (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
      (ResourceBodyResult Unit ExitCode) (from_buffer_alloc outcome))

proc a_body (dest : Resource FsHandle) (source : Resource FsHandle) (buffer_a : Resource Buffer)
  : HostIO AFull (ResourceBodyResult Unit ExitCode) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError ReadProgress) (ResourceBodyResult Unit ExitCode)
    (readAt AFull source (0 : Int) buffer_a (MkBufferWindow (2 : Int) (4 : Int)))
    (\outcome. match outcome {
      Err error |-> Ret (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (ResourceBodyResult Unit ExitCode) (ok_code (Failure 88));
      Ok progress |-> match progress {
        ReadEof |-> Ret (Coproduct (FSOp AFull) AmbientOp)
          (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
          (ResourceBodyResult Unit ExitCode) (ok_code (Failure 89));
        ReadSome span_a count |-> a_after_read dest span_a
      }
    })

proc source_body (dest : Resource FsHandle) (source : Resource FsHandle)
  : HostIO AFull (ResourceBodyResult Unit ExitCode) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError (ResourceBracketResult Unit ExitCode)) (ResourceBodyResult Unit ExitCode)
    (withBuffer AFull Unit ExitCode (8 : Int) (a_body dest source))
    (\outcome. Ret (Coproduct (FSOp AFull) AmbientOp)
      (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
      (ResourceBodyResult Unit ExitCode) (from_buffer_alloc outcome))

proc dest_body (cap : Cap AFull) (dest : Resource FsHandle)
  : HostIO AFull (ResourceBodyResult Unit ExitCode) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result FileError (ResourceBracketResult Unit ExitCode)) (ResourceBodyResult Unit ExitCode)
    (withResource AFull Unit ExitCode cap (bytes_encode "spanseed.bin") ResourceRead
      (source_body dest))
    (\outcome. Ret (Coproduct (FSOp AFull) AmbientOp)
      (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
      (ResourceBodyResult Unit ExitCode) (from_file_alloc outcome))

fn finish (outcome : Result FileError (ResourceBracketResult Unit ExitCode)) : HostIO AFull ExitCode =
  match outcome {
    Err error |-> host_exit AFull (Failure 96);
    Ok bracket |-> match bracket {
      ResourceBracketOk code |-> host_exit AFull code;
      ResourceBracketBodyError error |-> host_exit AFull (Failure 93);
      ResourceBracketReleaseError error |-> host_exit AFull (Failure 94);
      ResourceBracketBodyAndReleaseError body_error release_error |-> host_exit AFull (Failure 95)
    }
  }

proc main (_input : ProcessInput) (caps : ProgramCaps AFull)
  : HostIO AFull ExitCode visits [FS] =
  match caps {
    MkProgramCaps cap |->
      bind (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (Result FileError (ResourceBracketResult Unit ExitCode)) ExitCode
        (withResource AFull Unit ExitCode cap (bytes_encode "dest.bin")
          (ResourceWriteCreate CreateOrTruncate) (dest_body cap))
        (\outcome. finish outcome)
  }
"#;

#[cfg(target_os = "linux")]
#[test]
fn sp_b_host_width_offset_precedes_provenance_interp() {
    in_large_stack_thread("sp-b-precedence-interp", || {
        let obs = interpret_only("sp-b-precedence", SP_B_PRECEDENCE);
        assert_eq!(
            obs.exit_status, 0,
            "interp: exit 0 == foreign write@0 -> InvalidBounds AND foreign write@-1 -> InvalidOffset (host-width precedes provenance); got {obs:?}"
        );
    });
}

// SP-B validity arms — interpreter e2e. Two rejections that intentionally share
// InvalidBounds: (1) a FOREIGN span_a (from A, window [2,6) — numerically equal
// to B's live window) on B; (2) a matching-acquisition STALE span_b0 (from B's
// own [0,2) window, made stale by a later readAt that moved B's live window to
// [2,6)). Both freeze (no bytes) and writeAt (zero backend / empty destination)
// reject with InvalidBounds. Order: freeze span_a, write span_a, freeze span_b0,
// write span_b0. Native half is BLOCKED-ON-NATIVE-REACHABILITY.
#[cfg(target_os = "linux")]
const SP_B_VALIDITY: &str = r#"program capabilities FS AFull
fn ok_body (unit : Unit) : ResourceBodyResult Unit Unit = ResourceBodyOk Unit Unit MkUnit

fn from_buffer_alloc (outcome : Result ResourceError (ResourceBracketResult Unit Unit))
  : ResourceBodyResult Unit Unit =
  match outcome {
    Err error |-> ResourceBodyErr Unit Unit MkUnit;
    Ok bracket |-> match bracket {
      ResourceBracketOk value |-> ResourceBodyOk Unit Unit MkUnit;
      ResourceBracketBodyError error |-> ResourceBodyErr Unit Unit MkUnit;
      ResourceBracketReleaseError error |-> ResourceBodyErr Unit Unit MkUnit;
      ResourceBracketBodyAndReleaseError be re |-> ResourceBodyErr Unit Unit MkUnit
    }
  }

fn from_file_alloc (outcome : Result FileError (ResourceBracketResult Unit Unit))
  : ResourceBodyResult Unit Unit =
  match outcome {
    Err error |-> ResourceBodyErr Unit Unit MkUnit;
    Ok bracket |-> match bracket {
      ResourceBracketOk value |-> ResourceBodyOk Unit Unit MkUnit;
      ResourceBracketBodyError error |-> ResourceBodyErr Unit Unit MkUnit;
      ResourceBracketReleaseError error |-> ResourceBodyErr Unit Unit MkUnit;
      ResourceBracketBodyAndReleaseError be re |-> ResourceBodyErr Unit Unit MkUnit
    }
  }

fn ret_body (b : ResourceBodyResult Unit Unit) : HostIO AFull (ResourceBodyResult Unit Unit) =
  Ret (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (ResourceBodyResult Unit Unit) b

proc four_ops (dest : Resource FsHandle) (buffer_b : Resource Buffer)
  (span_a : BufferSpan) (span_b0 : BufferSpan)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError Bytes) (ResourceBodyResult Unit Unit)
    (freeze AFull buffer_b span_a)
    (\f1. bind (Coproduct (FSOp AFull) AmbientOp)
      (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
      (Result ResourceError WriteProgress) (ResourceBodyResult Unit Unit)
      (writeAt AFull dest (0 : Int) buffer_b span_a)
      (\w1. bind (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (Result ResourceError Bytes) (ResourceBodyResult Unit Unit)
        (freeze AFull buffer_b span_b0)
        (\f2. bind (Coproduct (FSOp AFull) AmbientOp)
          (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
          (Result ResourceError WriteProgress) (ResourceBodyResult Unit Unit)
          (writeAt AFull dest (0 : Int) buffer_b span_b0)
          (\w2. ret_body (ok_body MkUnit)))))

proc b_rewindow (dest : Resource FsHandle) (source : Resource FsHandle)
  (span_a : BufferSpan) (span_b0 : BufferSpan) (buffer_b : Resource Buffer)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError ReadProgress) (ResourceBodyResult Unit Unit)
    (readAt AFull source (2 : Int) buffer_b (MkBufferWindow (2 : Int) (4 : Int)))
    (\outcome. match outcome {
      Err error |-> ret_body (ResourceBodyErr Unit Unit MkUnit);
      Ok progress |-> match progress {
        ReadEof |-> ret_body (ResourceBodyErr Unit Unit MkUnit);
        ReadSome span_new count |-> four_ops dest buffer_b span_a span_b0
      }
    })

proc b_body (dest : Resource FsHandle) (source : Resource FsHandle)
  (span_a : BufferSpan) (buffer_b : Resource Buffer)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError ReadProgress) (ResourceBodyResult Unit Unit)
    (readAt AFull source (0 : Int) buffer_b (MkBufferWindow (0 : Int) (2 : Int)))
    (\outcome. match outcome {
      Err error |-> ret_body (ResourceBodyErr Unit Unit MkUnit);
      Ok progress |-> match progress {
        ReadEof |-> ret_body (ResourceBodyErr Unit Unit MkUnit);
        ReadSome span_b0 count |-> b_rewindow dest source span_a span_b0 buffer_b
      }
    })

proc a_after_read (dest : Resource FsHandle) (source : Resource FsHandle) (span_a : BufferSpan)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError (ResourceBracketResult Unit Unit)) (ResourceBodyResult Unit Unit)
    (withBuffer AFull Unit Unit (8 : Int) (b_body dest source span_a))
    (\outcome. ret_body (from_buffer_alloc outcome))

proc a_body (dest : Resource FsHandle) (source : Resource FsHandle) (buffer_a : Resource Buffer)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError ReadProgress) (ResourceBodyResult Unit Unit)
    (readAt AFull source (0 : Int) buffer_a (MkBufferWindow (2 : Int) (4 : Int)))
    (\outcome. match outcome {
      Err error |-> ret_body (ResourceBodyErr Unit Unit MkUnit);
      Ok progress |-> match progress {
        ReadEof |-> ret_body (ResourceBodyErr Unit Unit MkUnit);
        ReadSome span_a count |-> a_after_read dest source span_a
      }
    })

proc source_body (dest : Resource FsHandle) (source : Resource FsHandle)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError (ResourceBracketResult Unit Unit)) (ResourceBodyResult Unit Unit)
    (withBuffer AFull Unit Unit (8 : Int) (a_body dest source))
    (\outcome. ret_body (from_buffer_alloc outcome))

proc dest_body (cap : Cap AFull) (dest : Resource FsHandle)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result FileError (ResourceBracketResult Unit Unit)) (ResourceBodyResult Unit Unit)
    (withResource AFull Unit Unit cap (bytes_encode "spanseed.bin") ResourceRead
      (source_body dest))
    (\outcome. ret_body (from_file_alloc outcome))

fn finish (outcome : Result FileError (ResourceBracketResult Unit Unit)) : HostIO AFull ExitCode =
  match outcome {
    Err error |-> host_exit AFull (Failure 81);
    Ok bracket |-> match bracket {
      ResourceBracketOk value |-> host_exit AFull Success;
      ResourceBracketBodyError error |-> host_exit AFull (Failure 82);
      ResourceBracketReleaseError error |-> host_exit AFull (Failure 83);
      ResourceBracketBodyAndReleaseError body_error release_error |-> host_exit AFull (Failure 84)
    }
  }

proc main (_input : ProcessInput) (caps : ProgramCaps AFull)
  : HostIO AFull ExitCode visits [FS] =
  match caps {
    MkProgramCaps cap |->
      bind (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (Result FileError (ResourceBracketResult Unit Unit)) ExitCode
        (withResource AFull Unit Unit cap (bytes_encode "dest.bin")
          (ResourceWriteCreate CreateOrTruncate) (dest_body cap))
        (\outcome. finish outcome)
  }
"#;

#[cfg(target_os = "linux")]
#[test]
fn sp_b_foreign_and_stale_window_reject_with_no_effect_interp() {
    in_large_stack_thread("sp-b-validity-interp", || {
        let (obs, files) = interpret_reading("sp-b-validity", SP_B_VALIDITY, &["dest.bin"]);
        let ib = ken_runtime::CanonicalOutcomeV1::Error(ken_runtime::SemanticErrorV1::Resource(
            ken_runtime::ResourceErrorV1::InvalidBounds,
        ));
        assert_eq!(
            buffer_freeze_outcomes(&obs),
            vec![ib.clone(), ib.clone()],
            "interp: foreign and stale-window freeze must both be InvalidBounds (no bytes)"
        );
        assert_eq!(
            write_outcomes(&obs),
            vec![ib.clone(), ib.clone()],
            "interp: foreign and stale-window write must both be InvalidBounds"
        );
        assert!(
            files[0].is_empty(),
            "interp: both foreign/stale writes must issue zero backend writes — destination empty, got {:?}",
            files[0]
        );
    });
}
