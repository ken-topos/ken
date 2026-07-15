//! `ken run` Console-execution regression test (`wp/console-harvest-fix`,
//! AC3). Runs the real `ken` binary as a subprocess against a Console/IO
//! program and asserts stdout — the exact path that regressed silently
//! (the Console-ID harvest looked up dotted/two-param names that were
//! never registered, so every `ken run` on an IO program hard-failed
//! with exit 2 before running anything). Reuses the canonical
//! `examples/rosetta/hello-world` fixture rather than a throwaway one.

use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn ken_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ken"))
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn ken_run_executes_console_program_and_prints_stdout() {
    let example = workspace_root().join("examples/rosetta/hello-world/hello-world.ken");
    let output = Command::new(ken_bin())
        .arg("run")
        .arg(&example)
        .output()
        .expect("failed to spawn `ken run`");

    assert!(
        output.status.success(),
        "ken run should exit 0, got {:?}; stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "Hello, World!\n",
        "ken run should print the Console program's output exactly"
    );
}

#[cfg(target_os = "linux")]
#[test]
fn closed_stdout_is_an_io_failure_not_sigpipe_termination() {
    assert!(
        !include_str!("../src/main.rs").contains("unix_sigpipe"),
        "the supported Rust entrypoint must not opt back into SIGPIPE termination"
    );
    let tmp_dir = workspace_root().join("target/px1-broken-pipe");
    std::fs::create_dir_all(&tmp_dir).expect("create broken-pipe fixture directory");
    let program = tmp_dir.join("broken-pipe.ken");
    let payload = "x".repeat(256 * 1024);
    let source = format!(
        "program capabilities FS APartial\n\
         proc main (_input : ProcessInput) (_caps : ProgramCaps APartial) \
           : HostIO APartial ExitCode visits [Console] = \
           bind (Coproduct (FSOp APartial) AmbientOp) \
                (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp) \
                (Result IOError Unit) ExitCode \
             (host_console APartial (Result IOError Unit) \
               (write Stdout (bytes_encode \"{payload}\"))) \
             (\\r. match r {{ \
               Err e |-> match e {{ \
                 BrokenPipe |-> host_exit APartial (Failure 17); \
                 NotFound |-> host_exit APartial (Failure 18); \
                 PermissionDenied |-> host_exit APartial (Failure 18); \
                 CapabilityDenied |-> host_exit APartial (Failure 18); \
                 Interrupted |-> host_exit APartial (Failure 18); \
                 AlreadyExists |-> host_exit APartial (Failure 18); \
                 InvalidInput |-> host_exit APartial (Failure 18); \
                 IsDirectory |-> host_exit APartial (Failure 18); \
                 NotDirectory |-> host_exit APartial (Failure 18); \
                 NotEmpty |-> host_exit APartial (Failure 18); \
                 Unsupported |-> host_exit APartial (Failure 18); \
                 Other errno |-> host_exit APartial (Failure 18) \
               }}; \
               Ok _ |-> host_exit APartial Success \
             }})\n"
    );
    std::fs::write(&program, source).expect("write broken-pipe Ken program");

    let mut child = Command::new(ken_bin())
        .arg("run")
        .arg(&program)
        .current_dir(workspace_root())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn `ken run` with piped stdout");
    let mut stdout = child.stdout.take().expect("piped stdout");
    let mut first = [0_u8; 1];
    stdout
        .read_exact(&mut first)
        .expect("read first output byte");
    drop(stdout);

    let output = child.wait_with_output().expect("wait for `ken run`");
    assert!(
        output.status.code().is_some(),
        "closed stdout must not terminate `ken` by SIGPIPE: {:?}",
        output.status
    );
    assert_eq!(
        output.status.code(),
        Some(17),
        "the EPIPE-derived Ken Err arm must select its ordinary exit code; stderr: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
}
