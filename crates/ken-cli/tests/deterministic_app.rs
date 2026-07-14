//! Copyable end-to-end example for deterministic Ken application tests.

use ken_cli::{run_program, ProgramOutcome, SourceFormat};
use ken_interp::{
    CaptureHost, ConsoleStream, ConsoleTrace, FsTrace, HostCreatePolicy, VirtualFsNode,
};

const APP: &str = r#"program capabilities FS AFull
proc main (input : ProcessInput) (caps : ProgramCaps AFull)
  : HostIO AFull ExitCode visits [FS, Console] =
  match input {
    MkProcessInput arguments environment cwd |->
      match arguments {
        Nil |-> host_exit AFull (Failure 2) ;
        Cons argument _ |->
          match environment {
            Nil |-> host_exit AFull (Failure 3) ;
            Cons entry _ |->
              match entry {
                MkProd _ value |->
                  match caps {
                    MkProgramCaps cap |->
                      bind (Coproduct (FSOp AFull) AmbientOp)
                           (resp_coproduct (FSOp AFull) AmbientOp
                             (fs_resp AFull) ambient_resp)
                           (Result FileError Unit) ExitCode
                        (inject_l (FSOp AFull) AmbientOp
                          (fs_resp AFull) ambient_resp
                          (Result FileError Unit)
                          (writeFile cap cwd CreateNew value))
                        (\_ .
                          bind (Coproduct (FSOp AFull) AmbientOp)
                               (resp_coproduct (FSOp AFull) AmbientOp
                                 (fs_resp AFull) ambient_resp)
                               (Result IOError Unit) ExitCode
                            (host_console AFull (Result IOError Unit)
                              (write Stdout argument))
                            (\_ . host_exit AFull Success))
                  }
              }
          }
      }
  }
"#;

#[derive(Debug, PartialEq, Eq)]
struct Snapshot {
    outcome: ProgramOutcome,
    stdout: Vec<u8>,
    console: Vec<ConsoleTrace>,
    filesystem: Vec<FsTrace>,
}

fn run_once() -> Snapshot {
    let mut host = CaptureHost::new(b"fixed stdin".to_vec());
    host.insert_directory(b"sandbox".to_vec());

    let outcome = run_program(
        APP,
        SourceFormat::Ken,
        &[b"fixed argv".to_vec()],
        &[(b"FIXED_KEY".to_vec(), b"fixed env value".to_vec())],
        b"sandbox/output",
        &mut host,
    )
    .expect("application runs through the public driver");

    assert_eq!(host.stdout(), b"fixed argv");
    assert_eq!(
        host.trace(),
        &[ConsoleTrace::Write {
            stream: ConsoleStream::Stdout,
            bytes: b"fixed argv".to_vec(),
        }]
    );
    assert_eq!(
        host.fs_trace(),
        &[FsTrace::WriteFile {
            path: b"sandbox/output".to_vec(),
            policy: HostCreatePolicy::CreateNew,
            bytes: b"fixed env value".to_vec(),
        }]
    );
    assert_eq!(
        host.fs_nodes().get(b"sandbox/output".as_slice()),
        Some(&VirtualFsNode::File(b"fixed env value".to_vec()))
    );

    Snapshot {
        outcome,
        stdout: host.stdout().to_vec(),
        console: host.trace().to_vec(),
        filesystem: host.fs_trace().to_vec(),
    }
}

#[test]
fn injected_application_run_is_byte_identical_and_drives_capture_host() {
    let first = run_once();
    let second = run_once();
    assert_eq!(first, second);
    assert_eq!(first.outcome.exit_status, 0);
}
