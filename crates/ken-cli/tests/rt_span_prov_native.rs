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
        let obs = interpret_only("sp-a-write-foreign", SP_A_WRITE_FOREIGN);
        let writes = write_outcomes(&obs);
        assert_eq!(
            writes,
            vec![ken_runtime::CanonicalOutcomeV1::Error(
                ken_runtime::SemanticErrorV1::Resource(ken_runtime::ResourceErrorV1::InvalidBounds),
            )],
            "interp: foreign-acquisition write must be exactly one InvalidBounds, zero backend write"
        );
    });
}
