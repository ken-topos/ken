//! RT-PARITY executable interpreter/native exact-variant differential.
//!
//! Each case is its own `#[test]` so that every constructible overlap reaches
//! independently: a composite driver aborts on the first failing arm and leaves
//! the later arms unexecuted, which cannot demonstrate a per-arm pre-fix ->
//! post-fix flip.
//!
//! Every *narrowing* case asserts two independent discriminators. The one
//! non-narrowing case asserts neither by design: the producer-closure case is a
//! source-scope check that runs no fixture.
//!
//! 1. **Exact variant.** The Ken fixture matches the one expected
//!    `ResourceError` constructor and exits `0`; every other error constructor
//!    takes a distinct non-zero exit. Both executors must exit `0`, so the
//!    assertion is on the exact public variant rather than on `is_err`.
//! 2. **Dispatch skip.** Narrowing now rejects at the consuming operation, so
//!    neither executor records a canonical effect event for that operation.
//!    Before the repair the interpreter substituted a sentinel and entered
//!    shared dispatch, recording an event native never had.
//!
//! Carrying both axes is what makes every *narrowing* case discriminating. On
//! the variant axis alone the `u64::MAX`-sentinel single-fault cases are
//! green-vs-green:
//! shared dispatch rejects a `u64::MAX` argument with the very same
//! `InvalidOffset`/`InvalidBounds` the repair produces, so no single-fault
//! input can separate the implementations for those consumers. The dispatch-
//! skip axis separates them anyway, because pre-fix the interpreter still
//! entered dispatch and recorded an event native never had.
//!
//! Measured pre-fix (this suite against `origin/main` interpreter production).
//! All six *narrowing* cases fail pre-fix; the one non-narrowing case is
//! deliberately fix-independent and is never cited as flip evidence:
//!
//! | Case | Pre-fix | Discriminating axis |
//! |---|---|---|
//! | `buffer_allocate_malformed_capacity` | FAILS | variant (`BufferLimit`) |
//! | `fs_read_at_malformed_offset_without_read_right` | FAILS | variant (`RightNotHeld`) |
//! | `fs_write_at_malformed_offset_without_write_right` | FAILS | variant (`RightNotHeld`) |
//! | `fs_read_at_malformed_offset` | FAILS | dispatch skip |
//! | `fs_read_at_malformed_window` | FAILS | dispatch skip |
//! | `fs_write_at_malformed_offset` | FAILS | dispatch skip |
//! | `buffer_freeze_malformed_span_is_unconstructible...` | passes | none -- source-scope pin, not interpreter behaviour |
//!
//! `BufferFreeze` has no *narrowing* case here by structural necessity, not by
//! omission: no malformed span is constructible from checked source on the
//! landed sealed-producer closure. See
//! `buffer_freeze_malformed_span_is_unconstructible_on_landed_producer_closure`
//! for what that rests on and what it does not claim. Its narrowing guards stay
//! covered at the dispatch boundary in `ken-interp`.

#![cfg(target_os = "linux")]

fn output_dir(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "ken-rt-parity-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&path).unwrap();
    path
}

const RT_PARITY_SOURCE: &str = r#"program capabilities FS AFull
fn rt_body_ok (_buffer : Resource Buffer)
  : HostIO AFull (ResourceBodyResult Unit Unit) =
  Ret (Coproduct (FSOp AFull) AmbientOp)
(resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
(ResourceBodyResult Unit Unit) (ResourceBodyOk Unit Unit MkUnit)

fn rt_expect_invalid_offset (outcome : Result ResourceError ReadProgress)
  : HostIO AFull (ResourceBodyResult Unit Unit) =
  match outcome {
Err InvalidOffset |-> Ret (Coproduct (FSOp AFull) AmbientOp)
  (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
  (ResourceBodyResult Unit Unit) (ResourceBodyOk Unit Unit MkUnit);
Err error |-> Ret (Coproduct (FSOp AFull) AmbientOp)
  (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
  (ResourceBodyResult Unit Unit) (ResourceBodyErr Unit Unit MkUnit);
Ok progress |-> Ret (Coproduct (FSOp AFull) AmbientOp)
  (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
  (ResourceBodyResult Unit Unit) (ResourceBodyErr Unit Unit MkUnit)
  }

fn rt_expect_invalid_bounds (outcome : Result ResourceError ReadProgress)
  : HostIO AFull (ResourceBodyResult Unit Unit) =
  match outcome {
Err InvalidBounds |-> Ret (Coproduct (FSOp AFull) AmbientOp)
  (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
  (ResourceBodyResult Unit Unit) (ResourceBodyOk Unit Unit MkUnit);
Err error |-> Ret (Coproduct (FSOp AFull) AmbientOp)
  (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
  (ResourceBodyResult Unit Unit) (ResourceBodyErr Unit Unit MkUnit);
Ok progress |-> Ret (Coproduct (FSOp AFull) AmbientOp)
  (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
  (ResourceBodyResult Unit Unit) (ResourceBodyErr Unit Unit MkUnit)
  }

fn rt_expect_write_invalid_offset (outcome : Result ResourceError WriteProgress)
  : HostIO AFull (ResourceBodyResult Unit Unit) =
  match outcome {
Err InvalidOffset |-> Ret (Coproduct (FSOp AFull) AmbientOp)
  (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
  (ResourceBodyResult Unit Unit) (ResourceBodyOk Unit Unit MkUnit);
Err error |-> Ret (Coproduct (FSOp AFull) AmbientOp)
  (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
  (ResourceBodyResult Unit Unit) (ResourceBodyErr Unit Unit MkUnit);
Ok progress |-> Ret (Coproduct (FSOp AFull) AmbientOp)
  (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
  (ResourceBodyResult Unit Unit) (ResourceBodyErr Unit Unit MkUnit)
  }

fn rt_bracket_done
  (outcome : Result FileError (ResourceBracketResult Unit Unit))
  : HostIO AFull ExitCode =
  match outcome {
Ok (ResourceBracketOk unit) |-> host_exit AFull Success;
Ok bracket |-> host_exit AFull (Failure 51);
Err error |-> host_exit AFull (Failure 52)
  }

fn rt_buffer_bracket_done
  (outcome : Result ResourceError (ResourceBracketResult Unit Unit))
  : HostIO AFull ExitCode =
  match outcome {
Ok (ResourceBracketOk unit) |-> host_exit AFull Success;
Ok bracket |-> host_exit AFull (Failure 53);
Err error |-> host_exit AFull (Failure 54)
  }

fn rt_inner_bracket_result
  (outcome : Result ResourceError (ResourceBracketResult Unit Unit))
  : HostIO AFull (ResourceBodyResult Unit Unit) =
  match outcome {
Ok (ResourceBracketOk unit) |-> Ret (Coproduct (FSOp AFull) AmbientOp)
  (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
  (ResourceBodyResult Unit Unit) (ResourceBodyOk Unit Unit MkUnit);
Ok bracket |-> Ret (Coproduct (FSOp AFull) AmbientOp)
  (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
  (ResourceBodyResult Unit Unit) (ResourceBodyErr Unit Unit MkUnit);
Err error |-> Ret (Coproduct (FSOp AFull) AmbientOp)
  (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
  (ResourceBodyResult Unit Unit) (ResourceBodyErr Unit Unit MkUnit)
  }

fn rt_allocate_done
  (outcome : Result ResourceError (ResourceBracketResult Unit Unit))
  : HostIO AFull ExitCode =
  match outcome {
Err InvalidBounds |-> host_exit AFull Success;
Err error |-> host_exit AFull (Failure 41);
Ok bracket |-> host_exit AFull (Failure 42)
  }

proc rt_allocate_stage (cap : Cap AFull)
  : HostIO AFull ExitCode visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
(resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
(Result ResourceError (ResourceBracketResult Unit Unit)) ExitCode
(withBuffer AFull Unit Unit (sub_int 0 1) rt_body_ok)
(\outcome. rt_allocate_done outcome)

proc rt_read_offset_body (file : Resource FsHandle) (buffer : Resource Buffer)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
(resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
(Result ResourceError ReadProgress) (ResourceBodyResult Unit Unit)
(readAt AFull file (sub_int 0 1) buffer (MkBufferWindow (0 : Int) (1 : Int)))
(\outcome. rt_expect_invalid_offset outcome)

proc rt_read_offset_file (file : Resource FsHandle)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
(resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
(Result ResourceError (ResourceBracketResult Unit Unit))
(ResourceBodyResult Unit Unit)
(withBuffer AFull Unit Unit (1 : Int) (rt_read_offset_body file))
(\outcome. rt_inner_bracket_result outcome)

proc rt_read_offset_stage (cap : Cap AFull)
  : HostIO AFull ExitCode visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
(resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
(Result FileError (ResourceBracketResult Unit Unit)) ExitCode
(withResource AFull Unit Unit cap (bytes_encode "source")
  ResourceRead rt_read_offset_file)
(\outcome. rt_bracket_done outcome)

proc rt_read_window_body (file : Resource FsHandle) (buffer : Resource Buffer)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
(resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
(Result ResourceError ReadProgress) (ResourceBodyResult Unit Unit)
(readAt AFull file (0 : Int) buffer
  (MkBufferWindow (sub_int 0 1) (1 : Int)))
(\outcome. rt_expect_invalid_bounds outcome)

proc rt_read_window_file (file : Resource FsHandle)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
(resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
(Result ResourceError (ResourceBracketResult Unit Unit))
(ResourceBodyResult Unit Unit)
(withBuffer AFull Unit Unit (1 : Int) (rt_read_window_body file))
(\outcome. rt_inner_bracket_result outcome)

proc rt_read_window_stage (cap : Cap AFull)
  : HostIO AFull ExitCode visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
(resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
(Result FileError (ResourceBracketResult Unit Unit)) ExitCode
(withResource AFull Unit Unit cap (bytes_encode "source")
  ResourceRead rt_read_window_file)
(\outcome. rt_bracket_done outcome)

proc rt_read_norights_stage (cap : Cap AFull)
  : HostIO AFull ExitCode visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
(resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
(Result FileError (ResourceBracketResult Unit Unit)) ExitCode
(withResource AFull Unit Unit cap (bytes_encode "sink")
  (ResourceWriteCreate CreateOrTruncate) rt_read_offset_file)
(\outcome. rt_bracket_done outcome)

proc rt_write_after_read
  (file : Resource FsHandle) (buffer : Resource Buffer)
  (outcome : Result ResourceError ReadProgress)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  match outcome {
Ok (ReadSome span count) |-> bind (Coproduct (FSOp AFull) AmbientOp)
  (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
  (Result ResourceError WriteProgress) (ResourceBodyResult Unit Unit)
  (writeAt AFull file (sub_int 0 1) buffer span)
  (\written. rt_expect_write_invalid_offset written);
Ok ReadEof |-> Ret (Coproduct (FSOp AFull) AmbientOp)
  (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
  (ResourceBodyResult Unit Unit) (ResourceBodyErr Unit Unit MkUnit);
Err error |-> Ret (Coproduct (FSOp AFull) AmbientOp)
  (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
  (ResourceBodyResult Unit Unit) (ResourceBodyErr Unit Unit MkUnit)
  }

proc rt_write_body (file : Resource FsHandle) (buffer : Resource Buffer)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
(resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
(Result ResourceError ReadProgress) (ResourceBodyResult Unit Unit)
(readAt AFull file (0 : Int) buffer (MkBufferWindow (0 : Int) (1 : Int)))
(\outcome. rt_write_after_read file buffer outcome)

proc rt_write_file (file : Resource FsHandle)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
(resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
(Result ResourceError (ResourceBracketResult Unit Unit))
(ResourceBodyResult Unit Unit)
(withBuffer AFull Unit Unit (1 : Int) (rt_write_body file))
(\outcome. rt_inner_bracket_result outcome)

proc rt_write_readonly_stage (cap : Cap AFull)
  : HostIO AFull ExitCode visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
(resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
(Result FileError (ResourceBracketResult Unit Unit)) ExitCode
(withResource AFull Unit Unit cap (bytes_encode "source")
  ResourceRead rt_write_file)
(\outcome. rt_bracket_done outcome)

fn rt_file_bracket_result
  (outcome : Result FileError (ResourceBracketResult Unit Unit))
  : HostIO AFull (ResourceBodyResult Unit Unit) =
  match outcome {
Ok (ResourceBracketOk unit) |-> Ret (Coproduct (FSOp AFull) AmbientOp)
  (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
  (ResourceBodyResult Unit Unit) (ResourceBodyOk Unit Unit MkUnit);
Ok bracket |-> Ret (Coproduct (FSOp AFull) AmbientOp)
  (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
  (ResourceBodyResult Unit Unit) (ResourceBodyErr Unit Unit MkUnit);
Err error |-> Ret (Coproduct (FSOp AFull) AmbientOp)
  (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
  (ResourceBodyResult Unit Unit) (ResourceBodyErr Unit Unit MkUnit)
  }

proc rt_write_pair_after
  (sink : Resource FsHandle) (buffer : Resource Buffer)
  (outcome : Result ResourceError ReadProgress)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  match outcome {
Ok (ReadSome span count) |-> bind (Coproduct (FSOp AFull) AmbientOp)
  (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
  (Result ResourceError WriteProgress) (ResourceBodyResult Unit Unit)
  (writeAt AFull sink (sub_int 0 1) buffer span)
  (\written. rt_expect_write_invalid_offset written);
Ok ReadEof |-> Ret (Coproduct (FSOp AFull) AmbientOp)
  (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
  (ResourceBodyResult Unit Unit) (ResourceBodyErr Unit Unit MkUnit);
Err error |-> Ret (Coproduct (FSOp AFull) AmbientOp)
  (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
  (ResourceBodyResult Unit Unit) (ResourceBodyErr Unit Unit MkUnit)
  }

proc rt_write_pair_buffer
  (source : Resource FsHandle) (sink : Resource FsHandle)
  (buffer : Resource Buffer)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
(resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
(Result ResourceError ReadProgress) (ResourceBodyResult Unit Unit)
(readAt AFull source (0 : Int) buffer (MkBufferWindow (0 : Int) (1 : Int)))
(\outcome. rt_write_pair_after sink buffer outcome)

proc rt_write_pair_sink
  (source : Resource FsHandle) (sink : Resource FsHandle)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
(resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
(Result ResourceError (ResourceBracketResult Unit Unit))
(ResourceBodyResult Unit Unit)
(withBuffer AFull Unit Unit (1 : Int) (rt_write_pair_buffer source sink))
(\outcome. rt_inner_bracket_result outcome)

proc rt_write_pair_source (cap : Cap AFull) (source : Resource FsHandle)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
(resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
(Result FileError (ResourceBracketResult Unit Unit))
(ResourceBodyResult Unit Unit)
(withResource AFull Unit Unit cap (bytes_encode "sink")
  (ResourceWriteCreate CreateOrTruncate) (rt_write_pair_sink source))
(\outcome. rt_file_bracket_result outcome)

proc rt_write_writable_stage (cap : Cap AFull)
  : HostIO AFull ExitCode visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
(resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
(Result FileError (ResourceBracketResult Unit Unit)) ExitCode
(withResource AFull Unit Unit cap (bytes_encode "source")
  ResourceRead (rt_write_pair_source cap))
(\outcome. rt_bracket_done outcome)

proc main (_input : ProcessInput) (caps : ProgramCaps AFull)
  : HostIO AFull ExitCode visits [FS] =
  match caps {
MkProgramCaps cap |-> __RT_PARITY_ENTRY__ cap
  }
"#;

/// One case's differential outcome, used for the exact-variant assertions.
struct Differential {
    interpreted: ken_runtime::EffectObservation,
    native: ken_runtime::EffectObservation,
}

/// Compile the fixture at `entry` to a linked native artifact, run it, then run
/// the identical source through the reference interpreter against the same
/// root, and return both canonical observations.
fn differential(case: &str, entry: &str) -> Differential {
    let root = output_dir(case);
    std::fs::write(root.join("source"), b"ab").unwrap();
    let source = RT_PARITY_SOURCE.replace("__RT_PARITY_ENTRY__", entry);

    let output = ken_cli::build_native_program(
        &source,
        ken_cli::SourceFormat::Ken,
        &format!("rt_parity_{}", case.replace('-', "_")),
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
        &source,
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

fn operation_events(
    observation: &ken_runtime::EffectObservation,
    operation: ken_runtime::HostOpV1,
) -> Vec<(
    ken_runtime::CanonicalRequestV1,
    ken_runtime::CanonicalOutcomeV1,
)> {
    observation
        .effect_trace
        .iter()
        .filter(|event| event.operation == operation)
        .map(|event| (event.request.clone(), event.outcome.clone()))
        .collect()
}

/// Assert both discriminators for one narrowing case.
///
/// `operation` is the consuming host operation whose narrowing rejects the
/// malformed argument; after the repair neither executor dispatches it.
fn assert_narrowed_alike(
    case: &str,
    entry: &str,
    operation: ken_runtime::HostOpV1,
    expected_variant: &str,
) {
    let Differential {
        interpreted,
        native,
    } = differential(case, entry);

    // Axis 1 -- exact public variant. The fixture exits 0 only on
    // `expected_variant`; any other `ResourceError` constructor exits non-zero.
    assert_eq!(
        interpreted.exit_status, 0,
        "{case}: interpreter must observe exactly {expected_variant}; got {interpreted:?}"
    );
    assert_eq!(
        native.exit_status, 0,
        "{case}: native must observe exactly {expected_variant}; got {native:?}"
    );
    assert_eq!(interpreted.terminal_error, None, "{case}: interpreter");
    assert_eq!(native.terminal_error, None, "{case}: native");
    assert_eq!(
        interpreted.terminal_exit, native.terminal_exit,
        "{case}: terminal exit class must agree across executors"
    );

    // Axis 2 -- dispatch skip. Narrowing rejects at the consuming operation, so
    // the malformed request never reaches shared dispatch in either executor.
    let interpreted_events = operation_events(&interpreted, operation);
    let native_events = operation_events(&native, operation);
    assert_eq!(
        interpreted_events, native_events,
        "{case}: canonical {operation:?} events must agree across executors"
    );
    assert!(
        interpreted_events.is_empty(),
        "{case}: a narrowed {operation:?} must not enter shared dispatch; got {interpreted_events:?}"
    );
}

fn in_large_stack_thread(name: &'static str, body: fn()) {
    std::thread::Builder::new()
        .name(name.to_string())
        .stack_size(256 * 1024 * 1024)
        .spawn(body)
        .expect("spawn large-stack RT-PARITY fixture")
        .join()
        .expect("RT-PARITY fixture thread");
}

// -- BufferAllocate ------------------------------------------------------
//
// Single fault only. `BufferAllocate` consumes no resource, so no
// liveness/rights fault can coincide with the malformed capacity; the
// overlapping-fault obligation is structurally unreachable and is reported
// rather than silently dropped (frame AC-5).
//
// This consumer's pre-repair sentinel was `0`, not `u64::MAX`: a lawful
// capacity. It failed closed only because the resource table rejects a
// zero-capacity request as `BufferLimit` -- the wrong public variant, but not a
// silent success (frame AC-4).

#[test]
fn buffer_allocate_malformed_capacity_narrows_to_invalid_bounds() {
    in_large_stack_thread("rt-parity-allocate", || {
        assert_narrowed_alike(
            "buffer-allocate-single",
            "rt_allocate_stage",
            ken_runtime::HostOpV1::BufferAllocate,
            "InvalidBounds",
        )
    });
}

// -- FsReadAt ------------------------------------------------------------

#[test]
fn fs_read_at_malformed_offset_narrows_to_invalid_offset() {
    in_large_stack_thread("rt-parity-read-offset", || {
        assert_narrowed_alike(
            "fs-read-at-offset-single",
            "rt_read_offset_stage",
            ken_runtime::HostOpV1::FsReadAt,
            "InvalidOffset",
        )
    });
}

#[test]
fn fs_read_at_malformed_window_narrows_to_invalid_bounds() {
    in_large_stack_thread("rt-parity-read-window", || {
        assert_narrowed_alike(
            "fs-read-at-window-single",
            "rt_read_window_stage",
            ken_runtime::HostOpV1::FsReadAt,
            "InvalidBounds",
        )
    });
}

/// Overlapping fault: the same malformed offset as
/// `fs_read_at_malformed_offset_narrows_to_invalid_offset`, against a handle
/// opened write-only so the *read* right is not held. The two cases are a
/// non-degenerate pair -- identical program and identical malformed offset,
/// differing only in whether the coincident resource fault exists -- so a
/// narrowing that ran in the wrong order would fail exactly one of them.
///
/// Before the repair the sentinel entered dispatch and rights won, surfacing
/// `RightNotHeld`; native synthesised `InvalidOffset`.
///
/// The coincident fault here is a *rights* fault rather than a liveness one
/// because the liveness shape is not compilable: constructing a closed-but-
/// referenced resource requires escaping it from its bracket, and escaping a
/// second `Resource` through a bracket currently fails native lowering with
/// `OrientedSubcontinuationPlanV1: checked Runtime frame marker was consumed
/// more than once`. That is a pre-existing native lowering limitation, not an
/// RT-PARITY regression, and is reported rather than worked around; the
/// rights fault discriminates the same narrowing-order property.
#[test]
fn fs_read_at_malformed_offset_without_read_right_narrows_to_invalid_offset() {
    in_large_stack_thread("rt-parity-read-norights", || {
        assert_narrowed_alike(
            "fs-read-at-offset-overlap",
            "rt_read_norights_stage",
            ken_runtime::HostOpV1::FsReadAt,
            "InvalidOffset",
        )
    });
}

// -- FsWriteAt -----------------------------------------------------------
//
// Only `file_offset` is source-controllable: `writeAt` takes a `BufferSpan`,
// whose constructor is prelude-private, so the `buffer_start`/`length`
// narrowings cannot be reached from checked source. Their coverage is the
// interpreter-level dispatch test, not this differential.

#[test]
fn fs_write_at_malformed_offset_narrows_to_invalid_offset() {
    in_large_stack_thread("rt-parity-write-offset", || {
        assert_narrowed_alike(
            "fs-write-at-offset-single",
            "rt_write_writable_stage",
            ken_runtime::HostOpV1::FsWriteAt,
            "InvalidOffset",
        )
    });
}

/// Overlapping fault: the same malformed offset against a file opened
/// read-only, so the write right is not held. Before the repair the sentinel
/// entered dispatch and rights won, surfacing `RightNotHeld`; native
/// synthesised `InvalidOffset`.
#[test]
fn fs_write_at_malformed_offset_without_write_right_narrows_to_invalid_offset() {
    in_large_stack_thread("rt-parity-write-readonly", || {
        assert_narrowed_alike(
            "fs-write-at-offset-overlap",
            "rt_write_readonly_stage",
            ken_runtime::HostOpV1::FsWriteAt,
            "InvalidOffset",
        )
    });
}

// -- BufferFreeze --------------------------------------------------------

const SPAN_PROBE: &str = r#"program capabilities FS AFull
const rt_probe_span : BufferSpan = __RT_PARITY_SPAN__

proc main (_input : ProcessInput) (caps : ProgramCaps AFull)
  : HostIO AFull ExitCode visits [FS] =
  match caps {
MkProgramCaps cap |-> host_exit AFull Success
  }
"#;

fn elaborates(span_expression: &str, result_type: &str) -> Result<(), ken_cli::RunError> {
    let source = SPAN_PROBE
        .replace("__RT_PARITY_SPAN__", span_expression)
        .replace(": BufferSpan", &format!(": {result_type}"));
    ken_cli::run_program_effect_observation(
        &source,
        ken_cli::SourceFormat::Ken,
        &[],
        &[],
        b".",
        &mut ken_interp::CaptureHost::new(Vec::new()),
    )
    .map(|_| ())
}

/// `BufferFreeze` has no executable *narrowing* case, because no malformed
/// span is constructible from checked source.
///
/// The basis is the **landed sealed-producer closure**, not constructor-name
/// privacy. Privacy of a name does not by itself establish that no public
/// producer exists -- an earlier revision of this test argued exactly that
/// fallacy and was blocked for it (Architect, on `2b55706a`), because
/// `write_all_advance_span` was then a public `BufferSpan` transform.
///
/// That helper has since been sealed, and the property is now asserted
/// directly rather than argued: `ken-elaborator`'s
/// `px8f_buffer_io_surface` **derives** the set of public globals whose result
/// type is `BufferSpan` and asserts it is empty, along three axes --
///
/// 1. modulo definitional equality, so a transparent alias cannot smuggle one
///    past a syntactic head-match
///    (`buffer_span_producer_closure_reduces_transparent_type_aliases`);
/// 2. over declarations **and** constructors, so a public constructor is not
///    silently dropped
///    (`buffer_span_producer_closure_resolves_public_constructors`); and
/// 3. with a loud failure for any public id in neither category
///    (`buffer_span_producer_closure_rejects_unknown_public_ids`).
///
/// With that closure empty, every `BufferSpan` reaching `freeze` is host-minted
/// by a successful `readAt`, so its start and budget are already host-width
/// valid and the narrowing guards cannot be reached from source.
///
/// **This is a test-derived property, not a proof, and it is stated that way on
/// purpose.** The closure computation was wrong more than once before it
/// landed -- a syntactic head-match missed transparent aliases, and a lookup
/// silently dropped every constructor -- so the honest claim is "structurally
/// unconstructible on the landed closure, as asserted by
/// `px8f_buffer_io_surface` along those three axes", never "impossible". If
/// that closure is broken or weakened, this exemption goes with it and
/// `BufferFreeze` owes a single-fault and an overlap differential case.
///
/// What *this* test adds is the source-level pin at the differential layer:
/// the private names must stay unnameable from checked source. The narrowing
/// guards themselves remain covered at the dispatch boundary by
/// `eval::px5b_effect_observation_tests::rt_parity_buffer_freeze_*` and
/// `rt_parity_malformed_freeze_bounds_precede_closed_resource`.
///
/// **On the `TransferCount` pin below -- verified but ungated.** `TransferCount`
/// has no public producer empirically at `cd4184b8`: every public declaration
/// mentioning it consumes one, and `PrivateTransferCount` is sealed. That is a
/// grep-verified fact with **no oracle behind it** -- the landed closure covers
/// `BufferSpan` only, so nothing would catch a future public `TransferCount`
/// producer. The missing gate is tracked as `SEAL-2` and is deliberately not
/// built here.
///
/// It is called out rather than quietly relied on, but note it is **not
/// load-bearing for this claim**: with the `BufferSpan` producer closure empty,
/// every span is host-minted regardless of what counts are constructible, since
/// no public route turns a count into a span. The pin is defense in depth, and
/// it is what would fail first if a new span transform were ever introduced
/// that consumed a count.
#[test]
fn buffer_freeze_malformed_span_is_unconstructible_on_landed_producer_closure() {
    // Both private span producers are unnameable from checked source, so the
    // only route to a span is the host-minted one the closure leaves standing.
    for forged in [
        "PrivateBufferSpan (sub_int 0 1) Zero",
        "PrivateBufferSpan (0 : Int) Zero",
    ] {
        let error = elaborates(forged, "BufferSpan")
            .err()
            .unwrap_or_else(|| panic!("a source-forged BufferSpan must not elaborate: {forged}"));
        assert!(
            matches!(error, ken_cli::RunError::Elaboration(_)),
            "PrivateBufferSpan must be absent from source scope; got {error:?}"
        );
    }
    let error = elaborates(
        "write_all_advance_span rt_seed_span rt_seed_count",
        "BufferSpan",
    )
    .err()
    .expect("the sealed span transform must not elaborate");
    assert!(
        matches!(error, ken_cli::RunError::Elaboration(_)),
        "write_all_advance_span must stay sealed: it was the public transform \
         that defeated the earlier privacy argument; got {error:?}"
    );
    // Defense in depth, not load-bearing: the span closure already settles the
    // claim. Verified but ungated -- see SEAL-2 in the doc comment.
    let error = elaborates("PrivateTransferCount Zero Zero", "TransferCount")
        .err()
        .expect("a source-forged TransferCount must not elaborate");
    assert!(
        matches!(error, ken_cli::RunError::Elaboration(_)),
        "PrivateTransferCount must be absent from source scope; got {error:?}"
    );

    // Control: a public constructor of the same shape does elaborate, so the
    // rejections above are about scope and not about the probe's own form.
    elaborates("MkBufferWindow (sub_int 0 1) (1 : Int)", "BufferWindow")
        .expect("control: the public window constructor elaborates from source");
}
