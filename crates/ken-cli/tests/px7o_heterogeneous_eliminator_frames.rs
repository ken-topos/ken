fn output_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "ken-px7o-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

const PROGRAM: &str = r#"program capabilities FS APartial
proc write_then_exit (bytes : Bytes) (code : ExitCode)
  : HostIO APartial ExitCode visits [Console] =
  bind (Coproduct (FSOp APartial) AmbientOp)
    (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
    (Result IOError Unit) ExitCode
    (host_console APartial (Result IOError Unit) (write Stdout bytes))
    (\_. bind (Coproduct (FSOp APartial) AmbientOp)
      (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
      (Result IOError Unit) ExitCode
      (host_console APartial (Result IOError Unit) (flush Stdout))
      (\_. host_exit APartial code))

proc finish (outer : Result IOError (Result Bytes Bytes))
  : HostIO APartial ExitCode visits [Console] =
  match outer {
    Err _ |-> write_then_exit (bytes_encode "outer-error") (Failure 91);
    Ok inner |-> match inner {
      Err bytes |-> write_then_exit bytes (Failure 7);
      Ok bytes |-> write_then_exit bytes Success
    }
  }

proc run_case (as_ok : Bool) : HostIO APartial ExitCode visits [Console] =
  bind (Coproduct (FSOp APartial) AmbientOp)
    (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
    (Result IOError Unit) ExitCode
    (host_console APartial (Result IOError Unit)
      (write Stdout (bytes_encode "seed:")))
    (\written. finish (match written {
      Err io |-> Err IOError (Result Bytes Bytes) io;
      Ok _ |-> match as_ok {
        False |-> Ok IOError (Result Bytes Bytes)
          (Err Bytes Bytes (bytes_encode "err-payload"));
        True |-> Ok IOError (Result Bytes Bytes)
          (Ok Bytes Bytes (bytes_encode "ok-payload"))
      }
    }))

proc main (input : ProcessInput) (_caps : ProgramCaps APartial)
  : HostIO APartial ExitCode visits [Console] =
  match input {
    MkProcessInput arguments _environment _cwd |-> match arguments {
      Nil |-> host_exit APartial (Failure 99);
      Cons _ tail |-> match tail {
        Nil |-> run_case True;
        Cons _ _ |-> run_case False
      }
    }
  }
"#;

const IGNORED_PAYLOAD_PROGRAM: &str = r#"program capabilities FS APartial
proc main (_input : ProcessInput) (_caps : ProgramCaps APartial)
  : HostIO APartial ExitCode visits [Console] =
  bind (Coproduct (FSOp APartial) AmbientOp)
    (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
    (Result IOError Unit) ExitCode
    (host_console APartial (Result IOError Unit)
      (write Stdout (bytes_encode "ignored")))
    (\written. match written {
      Err _ |-> host_exit APartial (Failure 3);
      Ok _ |-> host_exit APartial Success
    })
"#;

fn assert_case(arguments: &[&str], expected_stdout: &[u8], expected_exit: i32) {
    let name = if arguments.is_empty() { "ok" } else { "err" };
    let dir = output_dir(name);
    let output = ken_cli::build_native_program(
        PROGRAM,
        ken_cli::SourceFormat::Ken,
        "px7o-heterogeneous-eliminator-frames",
        &dir,
    )
    .expect("heterogeneous eliminators reach the linked artifact");
    let native = ken_runtime::run_bound_process_effect_observation_v1(
        &output.artifact,
        &ken_runtime::NativeEffectRunOptionsV1 {
            arguments: arguments.iter().map(std::ffi::OsString::from).collect(),
            environment: Vec::new(),
            cwd: dir.clone(),
            plan_hash: output.plan_transport_hash,
        },
    )
    .expect("linked artifact returns its complete observation");

    let mut argv = vec![b"ken".to_vec()];
    argv.extend(
        arguments
            .iter()
            .map(|argument| argument.as_bytes().to_vec()),
    );
    let mut host = ken_interp::CaptureHost::new(Vec::new());
    let interpreted = ken_cli::run_program_effect_observation_v1(
        PROGRAM,
        ken_cli::SourceFormat::Ken,
        &argv,
        &[],
        b"/",
        &mut host,
    )
    .expect("same checked source runs through the interpreter");

    assert_eq!(native, interpreted);
    assert_eq!(native.exit_status, expected_exit);
    assert_eq!(native.stdout, expected_stdout);
    assert_eq!(
        native
            .effect_trace
            .iter()
            .map(|event| event.operation)
            .collect::<Vec<_>>(),
        vec![
            ken_runtime::HostOpV1::ConsoleWrite,
            ken_runtime::HostOpV1::ConsoleWrite,
            ken_runtime::HostOpV1::ConsoleFlush,
        ]
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn nested_ok_payload_reaches_both_real_executors() {
    assert_case(&[], b"seed:ok-payload", 0);
}

#[test]
fn nested_err_payload_reaches_both_real_executors() {
    assert_case(&["err"], b"seed:err-payload", 7);
}

#[test]
fn ignored_payload_twin_remains_an_opposite_only() {
    let dir = output_dir("ignored-payload");
    ken_cli::build_native_program(
        IGNORED_PAYLOAD_PROGRAM,
        ken_cli::SourceFormat::Ken,
        "px7o-ignored-payload-opposite",
        &dir,
    )
    .expect("the byte-near ignored-payload opposite still lowers");
    let _ = std::fs::remove_dir_all(dir);
}
