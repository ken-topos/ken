//! PX8-F reaching write-partition differential over the real checked `writeAll` path.

#![cfg(target_os = "linux")]

use ken_host::{
    CanonicalOutcomeV1, CanonicalReplyV1, EffectObservation, HostOpV1, IoErrorIdentityV1,
    ResourceErrorV1, SemanticErrorV1, WriteProgressV1,
};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

const WRITE_ALL_PARTITION: &str = r#"program capabilities FS AFull
fn body_from_write (outcome : Result ResourceError Unit)
  : ResourceBodyResult Unit Unit =
  match outcome {
    Err error |-> ResourceBodyErr Unit Unit MkUnit;
    Ok value |-> ResourceBodyOk Unit Unit MkUnit
  }

fn after_write (outcome : Result ResourceError Unit)
  : HostIO AFull (ResourceBodyResult Unit Unit) =
  Ret (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (ResourceBodyResult Unit Unit) (body_from_write outcome)

fn read_error_body (error : ResourceError)
  : ResourceBodyResult Unit Unit =
  ResourceBodyErr Unit Unit MkUnit

fn read_eof_body (_unit : Unit)
  : ResourceBodyResult Unit Unit =
  ResourceBodyErr Unit Unit MkUnit

proc after_read
  (output : Resource FsHandle) (buffer : Resource Buffer)
  (outcome : Result ResourceError ReadProgress)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  match outcome {
    Err error |-> Ret (Coproduct (FSOp AFull) AmbientOp)
      (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
      (ResourceBodyResult Unit Unit) (read_error_body error);
    Ok progress |-> match progress {
      ReadEof |-> Ret (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (ResourceBodyResult Unit Unit) (read_eof_body MkUnit);
      ReadSome span count |->
        bind (Coproduct (FSOp AFull) AmbientOp)
          (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
          (Result ResourceError Unit) (ResourceBodyResult Unit Unit)
          (writeAll AFull output (10 : Int) buffer span)
          (\written. after_write written)
    }
  }

proc buffer_body
  (input : Resource FsHandle) (output : Resource FsHandle)
  (buffer : Resource Buffer)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError ReadProgress) (ResourceBodyResult Unit Unit)
    (readAt AFull input (0 : Int) buffer (MkBufferWindow (0 : Int) (8 : Int)))
    (\outcome. after_read output buffer outcome)

fn buffer_bracket_body
  (outcome : Result ResourceError (ResourceBracketResult Unit Unit))
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

fn after_buffer
  (outcome : Result ResourceError (ResourceBracketResult Unit Unit))
  : HostIO AFull (ResourceBodyResult Unit Unit) =
  Ret (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (ResourceBodyResult Unit Unit) (buffer_bracket_body outcome)

proc output_body
  (input : Resource FsHandle) (output : Resource FsHandle)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError (ResourceBracketResult Unit Unit))
    (ResourceBodyResult Unit Unit)
    (withBuffer AFull Unit Unit (8 : Int) (buffer_body input output))
    (\outcome. after_buffer outcome)

fn file_bracket_body
  (outcome : Result FileError (ResourceBracketResult Unit Unit))
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

fn after_output
  (outcome : Result FileError (ResourceBracketResult Unit Unit))
  : HostIO AFull (ResourceBodyResult Unit Unit) =
  Ret (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (ResourceBodyResult Unit Unit) (file_bracket_body outcome)

proc input_body (cap : Cap AFull) (input : Resource FsHandle)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result FileError (ResourceBracketResult Unit Unit))
    (ResourceBodyResult Unit Unit)
    (withResource AFull Unit Unit cap (bytes_encode "output.bin")
      (ResourceWriteCreate CreateOrTruncate) (output_body input))
    (\outcome. after_output outcome)

fn finish (outcome : Result FileError (ResourceBracketResult Unit Unit))
  : HostIO AFull ExitCode =
  match outcome {
    Err error |-> host_exit AFull (Failure 81);
    Ok bracket |-> match bracket {
      ResourceBracketOk value |-> host_exit AFull Success;
      ResourceBracketBodyError error |-> host_exit AFull (Failure 82);
      ResourceBracketReleaseError error |-> host_exit AFull (Failure 83);
      ResourceBracketBodyAndReleaseError body_error release_error |->
        host_exit AFull (Failure 84)
    }
  }

proc main (_input : ProcessInput) (caps : ProgramCaps AFull)
  : HostIO AFull ExitCode visits [FS] =
  match caps {
    MkProgramCaps cap |->
      bind (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (Result FileError (ResourceBracketResult Unit Unit)) ExitCode
        (withResource AFull Unit Unit cap (bytes_encode "input.bin")
          ResourceRead (input_body cap))
        (\outcome. finish outcome)
  }
"#;

const INTERPOSER_SOURCE: &str = r#"#define _GNU_SOURCE
#include <dlfcn.h>
#include <errno.h>
#include <fcntl.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>
#include <unistd.h>

static unsigned long call_index = 0;

static void record_call(unsigned long call, off64_t offset, size_t count,
                        ssize_t result) {
  const char *path = getenv("KEN_PX8F_CALL_LOG");
  if (!path) return;
  int fd = open(path, O_WRONLY | O_CREAT | O_APPEND, 0600);
  if (fd < 0) return;
  char line[160];
  int length = snprintf(line, sizeof(line), "%lu %lld %zu %zd\n", call,
                        (long long)offset, count, result);
  if (length > 0) (void)write(fd, line, (size_t)length);
  (void)close(fd);
}

static ssize_t scripted_pwrite64(int fd, const void *buf, size_t count,
                                 off64_t offset,
                                 ssize_t (*next)(int, const void *, size_t,
                                                 off64_t)) {
  const char *mode = getenv("KEN_PX8F_WRITE_SCRIPT");
  unsigned long call = ++call_index;
  if (mode && strcmp(mode, "zero") == 0 && call == 2) {
    record_call(call, offset, count, 0);
    return 0;
  }
  if (mode && strcmp(mode, "error") == 0 && call == 2) {
    errno = EINTR;
    record_call(call, offset, count, -1);
    return -1;
  }
  size_t limit = count;
  if (mode && (strcmp(mode, "zero") == 0 || strcmp(mode, "progress") == 0) &&
      call == 1 && limit > 3) {
    limit = 3;
  } else if (mode && strcmp(mode, "short") == 0) {
    if (call == 1 && limit > 3) limit = 3;
    if (call == 2 && limit > 2) limit = 2;
  } else if (mode && strcmp(mode, "error") == 0 && call == 1 && limit > 2) {
    limit = 2;
  }
  ssize_t result = next(fd, buf, limit, offset);
  record_call(call, offset, count, result);
  return result;
}

ssize_t pwrite64(int fd, const void *buf, size_t count, off64_t offset) {
  static ssize_t (*next)(int, const void *, size_t, off64_t) = 0;
  if (!next) next = dlsym(RTLD_NEXT, "pwrite64");
  return scripted_pwrite64(fd, buf, count, offset, next);
}
"#;

#[derive(Clone, Copy, Debug)]
enum ExpectedOutcome {
    Wrote(u64),
    NoProgress,
    Interrupted,
}

#[derive(Clone, Copy, Debug)]
struct ExpectedWrite {
    file_offset: u64,
    buffer_start: u64,
    length: u64,
    outcome: ExpectedOutcome,
}

struct RunResult {
    observation: EffectObservation,
    sink: Vec<u8>,
    syscall_log: Vec<String>,
}

fn build_interposer(dir: &Path) -> PathBuf {
    let source = dir.join("px8f_write_script.c");
    let library = dir.join("libpx8f_write_script.so");
    std::fs::write(&source, INTERPOSER_SOURCE).expect("write pwrite64 interposer source");
    let status = std::process::Command::new("cc")
        .args(["-shared", "-fPIC", "-o"])
        .arg(&library)
        .arg(&source)
        .arg("-ldl")
        .status()
        .expect("compile pwrite64 interposer");
    assert!(status.success(), "pwrite64 interposer compilation failed");
    library
}

fn run_script(
    build: &ken_elaborator::compiler_driver::NativeProgramBuildOutput,
    preload: &Path,
    root: &Path,
    script: &str,
) -> RunResult {
    std::fs::create_dir_all(root).expect("create scenario root");
    std::fs::write(root.join("input.bin"), b"ABCDEFGH").expect("write scenario input");
    let log = root.join("pwrite64.log");
    let observation = ken_runtime::run_bound_process_effect_observation(
        &build.artifact,
        &ken_runtime::NativeEffectRunOptionsV1 {
            arguments: Vec::new(),
            environment: vec![
                ("LD_PRELOAD".into(), preload.as_os_str().to_os_string()),
                ("KEN_PX8F_WRITE_SCRIPT".into(), OsString::from(script)),
                ("KEN_PX8F_CALL_LOG".into(), log.as_os_str().to_os_string()),
            ],
            cwd: root.to_path_buf(),
            plan_hash: build.plan_transport_hash,
        },
    )
    .expect("real checked writeAll native execution");
    let syscall_log = std::fs::read_to_string(&log)
        .expect("interposer wrote its reaching call log")
        .lines()
        .map(str::to_owned)
        .collect();
    RunResult {
        observation,
        sink: std::fs::read(root.join("output.bin")).expect("read positioned-write sink"),
        syscall_log,
    }
}

fn assert_exact_sink(result: &RunResult, expected_prefix: &[u8]) {
    assert_eq!(
        result.sink,
        [vec![0; 10], expected_prefix.to_vec()].concat(),
        "the positioned sink contains only the exact prefix at offset 10"
    );
}

fn assert_write_trace(result: &RunResult, expected_exit: i32, expected: &[ExpectedWrite]) {
    assert_eq!(result.observation.exit_status, expected_exit);
    assert_eq!(result.observation.terminal_error, None);
    let writes: Vec<_> = result
        .observation
        .effect_trace
        .iter()
        .filter(|event| event.operation == HostOpV1::FsWriteAt)
        .collect();
    assert_eq!(writes.len(), expected.len(), "exact writeAt call count");
    for (event, expected) in writes.iter().zip(expected) {
        let ken_host::CanonicalRequestV1::FsWriteAt {
            file_offset,
            buffer_start,
            length,
        } = event.request
        else {
            panic!("FsWriteAt event carries its canonical request");
        };
        assert_eq!(
            (file_offset, buffer_start, length),
            (expected.file_offset, expected.buffer_start, expected.length)
        );
        match (&event.outcome, expected.outcome) {
            (
                CanonicalOutcomeV1::Success(CanonicalReplyV1::WriteProgress(
                    WriteProgressV1::Wrote(count),
                )),
                ExpectedOutcome::Wrote(expected_count),
            ) => assert_eq!(count.get(), expected_count),
            (
                CanonicalOutcomeV1::Error(SemanticErrorV1::Resource(ResourceErrorV1::NoProgress)),
                ExpectedOutcome::NoProgress,
            ) => {}
            (
                CanonicalOutcomeV1::Error(SemanticErrorV1::Io(IoErrorIdentityV1::Interrupted)),
                ExpectedOutcome::Interrupted,
            ) => {}
            (actual, expected) => panic!("wrong write outcome: {actual:?}, expected {expected:?}"),
        }
    }
}

#[test]
fn checked_write_all_reaches_full_short_zero_progress_flip_and_error_prefixes() {
    std::thread::Builder::new()
        .name("px8f-write-partition".to_string())
        .stack_size(256 * 1024 * 1024)
        .spawn(run_write_partition)
        .expect("spawn large-stack PX8-F differential")
        .join()
        .expect("PX8-F differential thread");
}

fn run_write_partition() {
    let dir = std::env::temp_dir().join(format!("ken-px8f-write-partition-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create PX8-F differential root");
    let preload = build_interposer(&dir);
    let build = ken_cli::build_native_program(
        WRITE_ALL_PARTITION,
        ken_cli::SourceFormat::Ken,
        "px8f_write_partition",
        dir.join("build"),
    )
    .expect("the real checked writeAll program builds to a linked native artifact");

    let full = run_script(&build, &preload, &dir.join("full"), "full");
    assert_exact_sink(&full, b"ABCDEFGH");
    assert_write_trace(
        &full,
        0,
        &[ExpectedWrite {
            file_offset: 10,
            buffer_start: 0,
            length: 8,
            outcome: ExpectedOutcome::Wrote(8),
        }],
    );
    assert_eq!(full.syscall_log, ["1 10 8 8"]);

    let short = run_script(&build, &preload, &dir.join("short"), "short");
    assert_exact_sink(&short, b"ABCDEFGH");
    assert_write_trace(
        &short,
        0,
        &[
            ExpectedWrite {
                file_offset: 10,
                buffer_start: 0,
                length: 8,
                outcome: ExpectedOutcome::Wrote(3),
            },
            ExpectedWrite {
                file_offset: 13,
                buffer_start: 3,
                length: 5,
                outcome: ExpectedOutcome::Wrote(2),
            },
            ExpectedWrite {
                file_offset: 15,
                buffer_start: 5,
                length: 3,
                outcome: ExpectedOutcome::Wrote(3),
            },
        ],
    );
    assert_eq!(short.syscall_log, ["1 10 8 3", "2 13 5 2", "3 15 3 3"]);

    let zero = run_script(&build, &preload, &dir.join("zero"), "zero");
    assert_exact_sink(&zero, b"ABC");
    assert_write_trace(
        &zero,
        82,
        &[
            ExpectedWrite {
                file_offset: 10,
                buffer_start: 0,
                length: 8,
                outcome: ExpectedOutcome::Wrote(3),
            },
            ExpectedWrite {
                file_offset: 13,
                buffer_start: 3,
                length: 5,
                outcome: ExpectedOutcome::NoProgress,
            },
        ],
    );
    assert_eq!(zero.syscall_log, ["1 10 8 3", "2 13 5 0"]);

    let progress = run_script(&build, &preload, &dir.join("progress"), "progress");
    assert_exact_sink(&progress, b"ABCDEFGH");
    assert_write_trace(
        &progress,
        0,
        &[
            ExpectedWrite {
                file_offset: 10,
                buffer_start: 0,
                length: 8,
                outcome: ExpectedOutcome::Wrote(3),
            },
            ExpectedWrite {
                file_offset: 13,
                buffer_start: 3,
                length: 5,
                outcome: ExpectedOutcome::Wrote(5),
            },
        ],
    );
    assert_eq!(progress.syscall_log, ["1 10 8 3", "2 13 5 5"]);

    let error = run_script(&build, &preload, &dir.join("error"), "error");
    assert_exact_sink(&error, b"AB");
    assert_write_trace(
        &error,
        82,
        &[
            ExpectedWrite {
                file_offset: 10,
                buffer_start: 0,
                length: 8,
                outcome: ExpectedOutcome::Wrote(2),
            },
            ExpectedWrite {
                file_offset: 12,
                buffer_start: 2,
                length: 6,
                outcome: ExpectedOutcome::Interrupted,
            },
        ],
    );
    assert_eq!(error.syscall_log, ["1 10 8 2", "2 12 6 -1"]);

    let _ = std::fs::remove_dir_all(dir);
}
