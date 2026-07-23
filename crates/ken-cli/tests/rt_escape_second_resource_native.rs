//! RT-ESCAPE linked-native discriminator: an escaped resource consumed by a
//! host operation whose `Result` match fans out must reach native execution
//! with the same observable bracket semantics as the interpreter.
//!
//! ## The defect (native-lowering completeness, M1)
//!
//! Constructing a closed-but-still-referenced resource requires escaping it from
//! its bracket; using that escaped resource through a host op (`readAt`,
//! `writeAll`, metadata, release) whose `Result` match fans out used to fail
//! native lowering with
//! `OrientedSubcontinuationPlanV1: checked Runtime frame marker was consumed
//! more than once`. Escaping a resource *unused* downstream, or escaping a
//! resource plus a plain value, both lowered fine (`escape_one_used`,
//! `escape_resource_plus_plain` below) — the trip is the *use* site.
//!
//! Classification is **M1** (one checked occurrence revisited, not two occurrences
//! aliasing a shared id): a match on a dynamic value lowers its shared post-match
//! continuation once per mutually-exclusive arm (`ok_block`/`err_block` off one
//! `brif`), so a checked subcontinuation frame in that shared continuation is a
//! *distinct lawful activation per arm*. The single per-lowering
//! `consumed_subcontinuation_frames` set conflated the two arms. The repair forks
//! that set per mutually-exclusive branch (snapshot → reset-per-arm → union at
//! rejoin) in the dynamic-match arm lowerers (`lower_forked_branch`), preserving
//! the within-a-single-path affine rejection (a real double-consume on one path
//! still rejects — see `rt_escape_within_path_duplicate_still_rejects`).
//!
//! Each case runs the identical source through the linked native artifact and the
//! reference interpreter and asserts the canonical observations agree, so the
//! guard is a semantic equivalence, not merely "it lowers".

#[cfg(target_os = "linux")]
struct Differential {
    interpreted: ken_runtime::EffectObservation,
    native: ken_runtime::EffectObservation,
}

#[cfg(target_os = "linux")]
fn output_dir(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "ken-rtescape-{name}-{}-{}",
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
/// source through the reference interpreter against the same root, and return
/// both canonical observations. A `held.bin` seed file is present in the root.
#[cfg(target_os = "linux")]
fn differential(case: &str, source: &str) -> Differential {
    let root = output_dir(case);
    std::fs::write(root.join("held.bin"), b"held resource").unwrap();

    let output = ken_cli::build_native_program(
        source,
        ken_cli::SourceFormat::Ken,
        &format!("rt_escape_{}", case.replace('-', "_")),
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

/// Native and interpreter must agree on exit, terminal class, and the exact
/// canonical effect-operation sequence.
#[cfg(target_os = "linux")]
fn assert_native_matches_interpreter(case: &str, diff: &Differential) {
    let Differential {
        interpreted,
        native,
    } = diff;
    assert_eq!(
        native.exit_status, interpreted.exit_status,
        "{case}: exit status must agree; native={native:?} interp={interpreted:?}"
    );
    assert_eq!(
        native.terminal_error, interpreted.terminal_error,
        "{case}: terminal error must agree"
    );
    assert_eq!(
        native.terminal_exit, interpreted.terminal_exit,
        "{case}: terminal exit class must agree"
    );
    let native_ops: Vec<_> = native
        .effect_trace
        .iter()
        .map(|event| event.operation)
        .collect();
    let interp_ops: Vec<_> = interpreted
        .effect_trace
        .iter()
        .map(|event| event.operation)
        .collect();
    assert_eq!(
        native_ops, interp_ops,
        "{case}: canonical effect-operation sequence must agree across executors"
    );
}

// (a) One escaped Resource, used once after its bracket settles. Always lowered
// (negative control): a single escaped-resource use consumes its checked frame
// exactly once, on one path.
#[cfg(target_os = "linux")]
const ESCAPE_ONE_USED: &str = r#"program capabilities FS AFull
proc after_escape (bracket : ResourceBracketResult Unit (Resource FsHandle))
  : HostIO AFull ExitCode visits [FS] =
  match bracket {
    ResourceBracketOk resource |->
      bind (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (Result ResourceError FileMetadata) ExitCode
        (resourceMetadata AFull resource)
        (\used. match used {
          Err Closed |-> host_exit AFull Success;
          Err error |-> host_exit AFull (Failure 91);
          Ok metadata |-> host_exit AFull (Failure 92)
        });
    ResourceBracketBodyError error |-> host_exit AFull (Failure 93);
    ResourceBracketReleaseError error |-> host_exit AFull (Failure 94);
    ResourceBracketBodyAndReleaseError body_error release_error |-> host_exit AFull (Failure 95)
  }

proc after_outer
  (outcome : Result FileError (ResourceBracketResult Unit (Resource FsHandle)))
  : HostIO AFull ExitCode visits [FS] =
  match outcome {
    Err open_error |-> host_exit AFull (Failure 96);
    Ok bracket |-> after_escape bracket
  }

proc main (_input : ProcessInput) (caps : ProgramCaps AFull)
  : HostIO AFull ExitCode visits [FS] =
  match caps {
    MkProgramCaps cap |->
      bind (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (Result FileError (ResourceBracketResult Unit (Resource FsHandle))) ExitCode
        (withResource AFull Unit (Resource FsHandle)
          cap (bytes_encode "held.bin") ResourceMetadata
          (\resource. Ret (Coproduct (FSOp AFull) AmbientOp)
            (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
            (ResourceBodyResult Unit (Resource FsHandle))
            (ResourceBodyOk Unit (Resource FsHandle) resource)))
        (\outcome. after_outer outcome)
  }
"#;

// (b) A Resource plus a plain value escaped as one aggregate. Always lowered
// (negative control): the aggregate carries one Resource, whose checked frame is
// consumed once.
#[cfg(target_os = "linux")]
const ESCAPE_RESOURCE_PLUS_PLAIN: &str = r#"program capabilities FS AFull
proc after_b
  (outcome : Result FileError (ResourceBracketResult Unit (Prod (Resource FsHandle) Unit)))
  : HostIO AFull ExitCode visits [FS] =
  match outcome {
    Err open_error |-> host_exit AFull (Failure 96);
    Ok bracket |-> match bracket {
      ResourceBracketOk pair |-> host_exit AFull Success;
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
        (Result FileError (ResourceBracketResult Unit (Prod (Resource FsHandle) Unit))) ExitCode
        (withResource AFull Unit (Prod (Resource FsHandle) Unit)
          cap (bytes_encode "held.bin") ResourceMetadata
          (\resource. Ret (Coproduct (FSOp AFull) AmbientOp)
            (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
            (ResourceBodyResult Unit (Prod (Resource FsHandle) Unit))
            (ResourceBodyOk Unit (Prod (Resource FsHandle) Unit)
              (MkProd (Resource FsHandle) Unit resource MkUnit))))
        (\outcome. after_b outcome)
  }
"#;

// (c) THE defect: escape the FILE out of its bracket, then `readAt` it (with a
// live buffer) after settlement. `readAt` returns `Result ResourceError
// ReadProgress`; its match fans out (Ok/Err), and the escaped file's checked
// frame lives in the shared post-match continuation. Pre-fix this failed native
// lowering with "checked Runtime frame marker was consumed more than once".
#[cfg(target_os = "linux")]
const ESCAPE_FILE_THEN_READAT: &str = r#"program capabilities FS AFull
proc read_body (file_closed : Resource FsHandle) (buffer : Resource Buffer)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError ReadProgress) (ResourceBodyResult Unit Unit)
    (readAt AFull file_closed (0 : Int) buffer (MkBufferWindow (0 : Int) (6 : Int)))
    (\outcome. Ret (Coproduct (FSOp AFull) AmbientOp)
      (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
      (ResourceBodyResult Unit Unit) (ResourceBodyOk Unit Unit MkUnit))

proc after_file_escape (file_closed : Resource FsHandle)
  : HostIO AFull ExitCode visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError (ResourceBracketResult Unit Unit)) ExitCode
    (withBuffer AFull Unit Unit (6 : Int) (read_body file_closed))
    (\outcome. host_exit AFull Success)

proc handle_outer (outcome : Result FileError (ResourceBracketResult Unit (Resource FsHandle)))
  : HostIO AFull ExitCode visits [FS] =
  match outcome {
    Err open_error |-> host_exit AFull (Failure 96);
    Ok bracket |-> match bracket {
      ResourceBracketOk file_closed |-> after_file_escape file_closed;
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
        (Result FileError (ResourceBracketResult Unit (Resource FsHandle))) ExitCode
        (withResource AFull Unit (Resource FsHandle)
          cap (bytes_encode "held.bin") ResourceRead
          (\resource. Ret (Coproduct (FSOp AFull) AmbientOp)
            (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
            (ResourceBodyResult Unit (Resource FsHandle))
            (ResourceBodyOk Unit (Resource FsHandle) resource)))
        (\outcome. handle_outer outcome)
  }
"#;

// Closure across resource *kinds*: the mirror of (c) with the escaped resource
// being a `Buffer` instead of an `FsHandle`. Escape the BUFFER out of its
// bracket, then `readAt` it with a still-live file. Same fan-out lowering, other
// kind — pre-fix this tripped the identical "consumed more than once".
#[cfg(target_os = "linux")]
const ESCAPE_BUFFER_THEN_READAT: &str = r#"program capabilities FS AFull
fn escape_buffer (buffer : Resource Buffer)
  : HostIO AFull (ResourceBodyResult Unit (Resource Buffer)) =
  Ret (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (ResourceBodyResult Unit (Resource Buffer))
    (ResourceBodyOk Unit (Resource Buffer) buffer)

proc read_with_escaped_buffer (file : Resource FsHandle) (buffer_closed : Resource Buffer)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError ReadProgress) (ResourceBodyResult Unit Unit)
    (readAt AFull file (0 : Int) buffer_closed (MkBufferWindow (0 : Int) (6 : Int)))
    (\outcome. Ret (Coproduct (FSOp AFull) AmbientOp)
      (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
      (ResourceBodyResult Unit Unit) (ResourceBodyOk Unit Unit MkUnit))

proc after_buffer_escape
  (file : Resource FsHandle)
  (inner : Result ResourceError (ResourceBracketResult Unit (Resource Buffer)))
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  match inner {
    Err allocate_error |-> Ret (Coproduct (FSOp AFull) AmbientOp)
      (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
      (ResourceBodyResult Unit Unit) (ResourceBodyErr Unit Unit MkUnit);
    Ok bracket |-> match bracket {
      ResourceBracketOk buffer_closed |-> read_with_escaped_buffer file buffer_closed;
      ResourceBracketBodyError error |-> Ret (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (ResourceBodyResult Unit Unit) (ResourceBodyErr Unit Unit MkUnit);
      ResourceBracketReleaseError error |-> Ret (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (ResourceBodyResult Unit Unit) (ResourceBodyErr Unit Unit MkUnit);
      ResourceBracketBodyAndReleaseError body_error release_error |-> Ret (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (ResourceBodyResult Unit Unit) (ResourceBodyErr Unit Unit MkUnit)
    }
  }

proc file_body (file : Resource FsHandle)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError (ResourceBracketResult Unit (Resource Buffer)))
    (ResourceBodyResult Unit Unit)
    (withBuffer AFull Unit (Resource Buffer) (6 : Int) escape_buffer)
    (\inner. after_buffer_escape file inner)

fn finish (outcome : Result FileError (ResourceBracketResult Unit Unit))
  : HostIO AFull ExitCode =
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
        (withResource AFull Unit Unit cap (bytes_encode "held.bin") ResourceRead file_body)
        (\outcome. finish outcome)
  }
"#;

// R2 reaching lane (AC-6): once two nested buffer resources compile, a
// `BufferSpan` obtained by reading into buffer_a (capacity 6, span length 6)
// applied to `freeze` on buffer_b (capacity 2) is the cross-buffer overlap
// fault. Statically predicted outcome: an `InvalidBounds` rejection (the span
// length exceeds buffer_b's capacity). The trace confirms `BufferFreeze` fails
// closed with `InvalidBounds` in both executors — the span length is bounded by
// the *target* buffer, so a span from a larger buffer cannot read a smaller one
// out of bounds. No distinct BufferFreeze defect; the obligation is discharged
// as a bounds rejection, not buried.
#[cfg(target_os = "linux")]
const R2_CROSS_BUFFER_FREEZE: &str = r#"program capabilities FS AFull
fn body_from_freeze (r : Result ResourceError Bytes) : ResourceBodyResult Unit Unit =
  match r {
    Ok bytes |-> ResourceBodyErr Unit Unit MkUnit;
    Err error |-> match error {
      InvalidBounds |-> ResourceBodyOk Unit Unit MkUnit;
      Closed |-> ResourceBodyErr Unit Unit MkUnit;
      InvalidOffset |-> ResourceBodyErr Unit Unit MkUnit;
      BufferLimit |-> ResourceBodyErr Unit Unit MkUnit;
      NoProgress |-> ResourceBodyErr Unit Unit MkUnit;
      MalformedResource |-> ResourceBodyErr Unit Unit MkUnit;
      RightNotHeld required held |-> ResourceBodyErr Unit Unit MkUnit;
      ResourceHostIO io |-> ResourceBodyErr Unit Unit MkUnit;
      ReleaseFailed kind identity io |-> ResourceBodyErr Unit Unit MkUnit;
      ResourceKindMismatch expected actual |-> ResourceBodyErr Unit Unit MkUnit
    }
  }

fn body_from_bracket (bracket : ResourceBracketResult Unit Unit) : ResourceBodyResult Unit Unit =
  match bracket {
    ResourceBracketOk value |-> ResourceBodyOk Unit Unit MkUnit;
    ResourceBracketBodyError error |-> ResourceBodyErr Unit Unit MkUnit;
    ResourceBracketReleaseError error |-> ResourceBodyErr Unit Unit MkUnit;
    ResourceBracketBodyAndReleaseError body_error release_error |-> ResourceBodyErr Unit Unit MkUnit
  }

fn body_from_alloc (outcome : Result ResourceError (ResourceBracketResult Unit Unit))
  : ResourceBodyResult Unit Unit =
  match outcome {
    Err error |-> ResourceBodyErr Unit Unit MkUnit;
    Ok bracket |-> body_from_bracket bracket
  }

proc after_read (buffer_b : Resource Buffer) (outcome : Result ResourceError ReadProgress)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  match outcome {
    Err error |-> Ret (Coproduct (FSOp AFull) AmbientOp)
      (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
      (ResourceBodyResult Unit Unit) (ResourceBodyErr Unit Unit MkUnit);
    Ok progress |-> match progress {
      ReadEof |-> Ret (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (ResourceBodyResult Unit Unit) (ResourceBodyErr Unit Unit MkUnit);
      ReadSome span_a count |->
        bind (Coproduct (FSOp AFull) AmbientOp)
          (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
          (Result ResourceError Bytes) (ResourceBodyResult Unit Unit)
          (freeze AFull buffer_b span_a)
          (\r. Ret (Coproduct (FSOp AFull) AmbientOp)
            (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
            (ResourceBodyResult Unit Unit) (body_from_freeze r))
    }
  }

proc buffer_b_body (file : Resource FsHandle) (buffer_a : Resource Buffer) (buffer_b : Resource Buffer)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError ReadProgress) (ResourceBodyResult Unit Unit)
    (readAt AFull file (0 : Int) buffer_a (MkBufferWindow (0 : Int) (6 : Int)))
    (\outcome. after_read buffer_b outcome)

proc buffer_a_body (file : Resource FsHandle) (buffer_a : Resource Buffer)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError (ResourceBracketResult Unit Unit)) (ResourceBodyResult Unit Unit)
    (withBuffer AFull Unit Unit (2 : Int) (buffer_b_body file buffer_a))
    (\outcome. Ret (Coproduct (FSOp AFull) AmbientOp)
      (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
      (ResourceBodyResult Unit Unit) (body_from_alloc outcome))

proc file_body (file : Resource FsHandle)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError (ResourceBracketResult Unit Unit)) (ResourceBodyResult Unit Unit)
    (withBuffer AFull Unit Unit (6 : Int) (buffer_a_body file))
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
        (withResource AFull Unit Unit cap (bytes_encode "held.bin") ResourceRead file_body)
        (\outcome. finish outcome)
  }
"#;

#[cfg(target_os = "linux")]
#[test]
fn escape_one_used_matches_interpreter() {
    let diff = differential("escape-one-used", ESCAPE_ONE_USED);
    assert_eq!(diff.native.exit_status, 0, "{:?}", diff.native);
    assert_native_matches_interpreter("escape-one-used", &diff);
}

#[cfg(target_os = "linux")]
#[test]
fn escape_resource_plus_plain_matches_interpreter() {
    let diff = differential("escape-res-plus-plain", ESCAPE_RESOURCE_PLUS_PLAIN);
    assert_eq!(diff.native.exit_status, 0, "{:?}", diff.native);
    assert_native_matches_interpreter("escape-res-plus-plain", &diff);
}

#[cfg(target_os = "linux")]
#[test]
fn escaped_resource_used_by_fanning_host_op_matches_interpreter() {
    // Pre-fix: this panicked in `build_native_program` with
    // "checked Runtime frame marker was consumed more than once". The fork/union
    // of `consumed_subcontinuation_frames` per mutually-exclusive arm makes it
    // reach native execution; the assertion below pins interpreter equivalence.
    let diff = differential("escape-file-then-readat", ESCAPE_FILE_THEN_READAT);
    assert_native_matches_interpreter("escape-file-then-readat", &diff);
}

#[cfg(target_os = "linux")]
#[test]
fn escaped_buffer_used_by_fanning_host_op_matches_interpreter() {
    // Closure across resource kinds: same fan-out defect with an escaped
    // `Buffer` rather than an escaped `FsHandle`. Also pre-fix "consumed more
    // than once"; now interpreter-equivalent.
    let diff = differential("escape-buffer-then-readat", ESCAPE_BUFFER_THEN_READAT);
    assert_native_matches_interpreter("escape-buffer-then-readat", &diff);
}

/// The nested three-resource R2 fixture needs a deep native stack, as the
/// oriented subcontinuation tests do.
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
fn buffer_freeze_outcome(
    observation: &ken_runtime::EffectObservation,
) -> ken_runtime::CanonicalOutcomeV1 {
    observation
        .effect_trace
        .iter()
        .find(|event| event.operation == ken_runtime::HostOpV1::BufferFreeze)
        .map(|event| event.outcome.clone())
        .expect("the cross-buffer freeze must reach dispatch as a BufferFreeze")
}

#[cfg(target_os = "linux")]
#[test]
fn r2_cross_buffer_freeze_fails_closed_with_invalid_bounds() {
    in_large_stack_thread("rt-escape-r2", || {
        // R2 reaching lane: two nested buffer resources compile and run; a span
        // from buffer_a (length 6) applied to freeze buffer_b (capacity 2) is
        // rejected with InvalidBounds in both executors. The span length is
        // bounded by the target buffer, so this is the statically-predicted
        // bounds rejection, not a distinct BufferFreeze semantic defect.
        let diff = differential("r2-cross-buffer-freeze", R2_CROSS_BUFFER_FREEZE);
        assert_native_matches_interpreter("r2-cross-buffer-freeze", &diff);
        let expected = ken_runtime::CanonicalOutcomeV1::Error(
            ken_runtime::SemanticErrorV1::Resource(ken_runtime::ResourceErrorV1::InvalidBounds),
        );
        assert_eq!(
            buffer_freeze_outcome(&diff.native),
            expected,
            "native: cross-buffer freeze must fail closed with InvalidBounds"
        );
        assert_eq!(
            buffer_freeze_outcome(&diff.interpreted),
            expected,
            "interpreter: cross-buffer freeze must fail closed with InvalidBounds"
        );
    });
}
