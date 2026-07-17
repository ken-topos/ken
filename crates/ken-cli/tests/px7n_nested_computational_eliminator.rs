fn output_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "ken-px7n-{name}-{}-{}",
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
proc wrap_result (as_ok : Bool)
  : HostIO APartial (Result Bytes Bytes) visits [Console] =
  bind (Coproduct (FSOp APartial) AmbientOp)
    (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
    (Result IOError Unit) (Result Bytes Bytes)
    (host_console APartial (Result IOError Unit)
      (write Stdout (bytes_encode "seed:")))
    (\written. match written {
      Err _ |-> Ret (Coproduct (FSOp APartial) AmbientOp)
        (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
        (Result Bytes Bytes) (Err Bytes Bytes (bytes_encode "host-error"));
      Ok _ |-> match as_ok {
        False |-> Ret (Coproduct (FSOp APartial) AmbientOp)
          (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
          (Result Bytes Bytes) (Err Bytes Bytes (bytes_encode "err-payload"));
        True |-> Ret (Coproduct (FSOp APartial) AmbientOp)
          (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
          (Result Bytes Bytes) (Ok Bytes Bytes (bytes_encode "ok-payload"))
      }
    })

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

proc wrap_again (body : Unit -> HostIO APartial (Result Bytes Bytes))
  : HostIO APartial ExitCode visits [Console] =
  bind (Coproduct (FSOp APartial) AmbientOp)
    (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
    (Result Bytes Bytes) ExitCode
    (relay body)
    finish

proc relay (body : Unit -> HostIO APartial (Result Bytes Bytes))
  : HostIO APartial (Result Bytes Bytes) visits [Console] =
  bind (Coproduct (FSOp APartial) AmbientOp)
    (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
    (Result Bytes Bytes) (Result Bytes Bytes)
    (body MkUnit)
    (\outcome. Ret (Coproduct (FSOp APartial) AmbientOp)
      (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
      (Result Bytes Bytes) outcome)

proc finish (outcome : Result Bytes Bytes)
  : HostIO APartial ExitCode visits [Console] =
  match outcome {
    Err bytes |-> write_then_exit bytes (Failure 7);
    Ok bytes |-> write_then_exit bytes Success
  }

proc main (input : ProcessInput) (_caps : ProgramCaps APartial)
  : HostIO APartial ExitCode visits [Console] =
  match input {
    MkProcessInput arguments _environment _cwd |-> match arguments {
      Nil |-> host_exit APartial (Failure 99);
      Cons _ tail |-> match tail {
        Nil |-> wrap_again (\_. wrap_result True);
        Cons _ _ |-> wrap_again (\_. wrap_result False)
      }
    }
  }
"#;

fn assert_case(arguments: &[&str], expected_stdout: &[u8], expected_exit: i32) {
    let name = if arguments.is_empty() { "ok" } else { "err" };
    let dir = output_dir(name);
    let output = ken_cli::build_native_program(
        PROGRAM,
        ken_cli::SourceFormat::Ken,
        "px7n-nested-computational-eliminator",
        &dir,
    )
    .expect("nested computational eliminators compose in the linked artifact");
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
