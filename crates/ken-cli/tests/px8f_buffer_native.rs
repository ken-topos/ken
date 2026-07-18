//! PX8-F linked checked `writeAll` reachability through PX8-N bounded Nat.

const WRITE_ALL: &str = r#"program capabilities FS AFull
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
  : HostIO AFull (ResourceBodyResult Unit Unit) =
  Ret (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (ResourceBodyResult Unit Unit) (ResourceBodyErr Unit Unit MkUnit)

fn read_eof_body (_unit : Unit)
  : HostIO AFull (ResourceBodyResult Unit Unit) =
  Ret (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (ResourceBodyResult Unit Unit) (ResourceBodyErr Unit Unit MkUnit)

proc after_read
  (output : Resource FsHandle) (buffer : Resource Buffer)
  (outcome : Result ResourceError ReadProgress)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  match outcome {
    Err error |-> read_error_body error;
    Ok progress |-> match progress {
      ReadEof |-> read_eof_body MkUnit;
      ReadSome span count |->
        bind (Coproduct (FSOp AFull) AmbientOp)
          (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
          (Result ResourceError Unit) (ResourceBodyResult Unit Unit)
          (writeAll AFull output (0 : Int) buffer span)
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
    (readAt AFull input (0 : Int) buffer (MkBufferWindow (0 : Int) (6 : Int)))
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
    (withBuffer AFull Unit Unit (6 : Int) (buffer_body input output))
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

#[cfg(target_os = "linux")]
fn build_short_pwrite_preload(dir: &std::path::Path) -> std::path::PathBuf {
    let source = dir.join("short_pwrite.c");
    let library = dir.join("libshort_pwrite.so");
    std::fs::write(
        &source,
        r#"#define _GNU_SOURCE
#include <dlfcn.h>
#include <stddef.h>
#include <sys/types.h>
#include <unistd.h>

ssize_t pwrite(int fd, const void *buf, size_t count, off_t offset) {
  static ssize_t (*next_pwrite)(int, const void *, size_t, off_t) = 0;
  if (!next_pwrite) {
    next_pwrite = dlsym(RTLD_NEXT, "pwrite");
  }
  size_t capped = count > 2 ? 2 : count;
  return next_pwrite(fd, buf, capped, offset);
}
"#,
    )
    .unwrap();
    let status = std::process::Command::new("cc")
        .args(["-shared", "-fPIC", "-o"])
        .arg(&library)
        .arg(&source)
        .arg("-ldl")
        .status()
        .expect("compile short-pwrite preload");
    assert!(status.success(), "short-pwrite preload compilation failed");
    library
}

#[cfg(target_os = "linux")]
#[test]
fn linked_checked_write_all_observes_short_progress_and_matches_interpreter() {
    std::thread::Builder::new()
        .name("px8f-write-all".to_string())
        .stack_size(256 * 1024 * 1024)
        .spawn(run_linked_checked_write_all)
        .expect("spawn large-stack PX8-F fixture")
        .join()
        .expect("PX8-F fixture thread");
}

#[cfg(target_os = "linux")]
fn run_linked_checked_write_all() {
    let dir = std::env::temp_dir().join(format!("ken-px8f-write-all-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("input.bin"), b"abcdef").unwrap();
    let preload = build_short_pwrite_preload(&dir);

    eprintln!("PX8-F: compiling checked writeAll fixture");
    let output = ken_cli::build_native_program(
        WRITE_ALL,
        ken_cli::SourceFormat::Ken,
        "px8f_write_all_native",
        &dir,
    )
    .expect("checked writeAll reaches linked native lowering");
    eprintln!("PX8-F: running linked fixture");
    let observation = ken_runtime::run_bound_process_effect_observation(
        &output.artifact,
        &ken_runtime::NativeEffectRunOptionsV1 {
            arguments: Vec::new(),
            environment: vec![("LD_PRELOAD".into(), preload.into_os_string())],
            cwd: dir.clone(),
            plan_hash: output.plan_transport_hash,
        },
    )
    .expect("linked checked writeAll runs");
    eprintln!("PX8-F: running interpreter fixture");

    assert_eq!(observation.exit_status, 0);
    assert_eq!(observation.terminal_error, None);
    assert_eq!(std::fs::read(dir.join("output.bin")).unwrap(), b"abcdef");
    let writes: Vec<_> = observation
        .effect_trace
        .iter()
        .filter(|event| event.operation == ken_runtime::HostOpV1::FsWriteAt)
        .collect();
    assert_eq!(writes.len(), 3, "short progress must recurse twice");
    for (event, expected) in writes.iter().zip([(0, 0, 6), (2, 2, 4), (4, 4, 2)]) {
        assert!(matches!(
            (&event.request, &event.outcome),
            (
                ken_runtime::CanonicalRequestV1::FsWriteAt {
                    file_offset,
                    buffer_start,
                    length,
                },
                ken_runtime::CanonicalOutcomeV1::Success(
                    ken_runtime::CanonicalReplyV1::WriteProgress(_)
                )
            ) if (*file_offset, *buffer_start, *length) == expected
        ));
    }

    let mut interpreter = ken_interp::CaptureHost::new(Vec::new());
    interpreter.insert_file(b"input.bin".to_vec(), b"abcdef".to_vec());
    let interpreted = ken_cli::run_program_effect_observation(
        WRITE_ALL,
        ken_cli::SourceFormat::Ken,
        &[],
        &[],
        b".",
        &mut interpreter,
    )
    .expect("the same checked writeAll runs in the interpreter");
    eprintln!("PX8-F: comparing observations");
    assert_eq!(interpreted.exit_status, observation.exit_status);
    assert_eq!(interpreted.terminal_error, observation.terminal_error);
    assert_eq!(
        interpreter.fs_nodes().get(b"output.bin".as_slice()),
        Some(&ken_interp::VirtualFsNode::File(b"abcdef".to_vec()))
    );

    let _ = std::fs::remove_dir_all(dir);
}
