fn output_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "ken-px7p-{name}-{}-{}",
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

proc finish (outcome : Result Bytes Bytes)
  : HostIO APartial ExitCode visits [Console] =
  match outcome {
    Err bytes |-> write_then_exit bytes (Failure 7);
    Ok bytes |-> write_then_exit bytes Success
  }

proc produce (as_ok : Bool)
  : HostIO APartial (Result Bytes Bytes) visits [Console] =
  bind (Coproduct (FSOp APartial) AmbientOp)
    (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
    (Result IOError Unit) (Result Bytes Bytes)
    (host_console APartial (Result IOError Unit)
      (write Stdout (bytes_encode "seed:")))
    (\written. Ret (Coproduct (FSOp APartial) AmbientOp)
      (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
      (Result Bytes Bytes)
      (match written {
        Err _ |-> Err Bytes Bytes (bytes_encode "write-error");
        Ok _ |-> match as_ok {
          False |-> Err Bytes Bytes (bytes_encode "err-payload");
          True |-> Ok Bytes Bytes (bytes_encode "ok-payload")
        }
      }))

proc run_case (as_ok : Bool) : HostIO APartial ExitCode visits [Console] =
  bind (Coproduct (FSOp APartial) AmbientOp)
    (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
    (Result Bytes Bytes) ExitCode (produce as_ok) (\outcome. finish outcome)

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

const IGNORED_PROGRAM: &str = r#"program capabilities FS APartial
proc produce
  : HostIO APartial (Result Bytes Bytes) visits [Console] =
  bind (Coproduct (FSOp APartial) AmbientOp)
    (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
    (Result IOError Unit) (Result Bytes Bytes)
    (host_console APartial (Result IOError Unit)
      (write Stdout (bytes_encode "ignored")))
    (\written. Ret (Coproduct (FSOp APartial) AmbientOp)
      (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
      (Result Bytes Bytes)
      (match written {
        Err _ |-> Err Bytes Bytes (bytes_encode "ignored-error");
        Ok _ |-> Ok Bytes Bytes (bytes_encode "ignored-ok")
      }))

proc main (_input : ProcessInput) (_caps : ProgramCaps APartial)
  : HostIO APartial ExitCode visits [Console] =
  bind (Coproduct (FSOp APartial) AmbientOp)
    (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
    (Result Bytes Bytes) ExitCode produce
    (\_outcome. host_exit APartial Success)
"#;

const DYNAMIC_CARRIER_PROGRAM: &str = r#"program capabilities FS APartial
proc write_then_fail (bytes : Bytes)
  : HostIO APartial ExitCode visits [Console] =
  bind (Coproduct (FSOp APartial) AmbientOp)
    (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
    (Result IOError Unit) ExitCode
    (host_console APartial (Result IOError Unit) (write Stdout bytes))
    (\_. host_exit APartial (Failure 36))

proc finish_dynamic (outcome : Result Bytes Bytes)
  : HostIO APartial ExitCode visits [Console] =
  match outcome {
    Err bytes |-> write_then_fail bytes;
    Ok _ |-> host_exit APartial (Failure 83)
  }

proc produce_dynamic (cap : Cap APartial) (path : Bytes)
  : HostIO APartial (Result Bytes Bytes) visits [FS, Console] =
  bind (Coproduct (FSOp APartial) AmbientOp)
    (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
    (Result FileError Bytes) (Result Bytes Bytes)
    (inject_l (FSOp APartial) AmbientOp
      (fs_resp APartial) ambient_resp
      (Result FileError Bytes) (readFile APartial cap path))
    (\read. Ret (Coproduct (FSOp APartial) AmbientOp)
      (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
      (Result Bytes Bytes)
      (match read {
        Ok bytes |-> Ok Bytes Bytes bytes;
        Err error |-> match error {
          MkFileError _operation _path cause |-> match cause {
            NotFound |-> Err Bytes Bytes (bytes_encode "not-found");
            PermissionDenied |-> Err Bytes Bytes (bytes_encode "permission");
            CapabilityDenied |-> Err Bytes Bytes (bytes_encode "capability");
            BrokenPipe |-> Err Bytes Bytes (bytes_encode "broken-pipe");
            Interrupted |-> Err Bytes Bytes (bytes_encode "interrupted");
            AlreadyExists |-> Err Bytes Bytes (bytes_encode "exists");
            InvalidInput |-> Err Bytes Bytes (bytes_encode "invalid");
            IsDirectory |-> Err Bytes Bytes (bytes_encode "directory");
            NotDirectory |-> Err Bytes Bytes (bytes_encode "not-directory");
            NotEmpty |-> Err Bytes Bytes (bytes_encode "not-empty");
            Unsupported |-> Err Bytes Bytes (bytes_encode "unsupported");
            Other raw |-> match eq_int raw 36 {
              False |-> Err Bytes Bytes (bytes_encode "other");
              True |-> Err Bytes Bytes (bytes_encode "other-36")
            }
          }
        }
      }))

proc main (input : ProcessInput) (caps : ProgramCaps APartial)
  : HostIO APartial ExitCode visits [FS, Console] =
  match input {
    MkProcessInput arguments _environment _cwd |-> match arguments {
      Nil |-> host_exit APartial (Failure 81);
      Cons _argv0 rest |-> match rest {
        Nil |-> host_exit APartial (Failure 82);
        Cons path _ |-> match caps {
          MkProgramCaps cap |->
            bind (Coproduct (FSOp APartial) AmbientOp)
              (resp_coproduct (FSOp APartial) AmbientOp
                (fs_resp APartial) ambient_resp)
              (Result Bytes Bytes) ExitCode
              (produce_dynamic cap path)
              (\outcome. finish_dynamic outcome)
        }
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
        "px7p-constructor-field-composition",
        &dir,
    )
    .expect("constructor field composes through its selected consumer");
    let native = ken_runtime::run_bound_process_effect_observation(
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
    let interpreted = ken_cli::run_program_effect_observation(
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
fn selected_ok_field_reaches_both_real_executors() {
    assert_case(&[], b"seed:ok-payload", 0);
}

#[test]
fn selected_err_field_reaches_both_real_executors() {
    assert_case(&["err"], b"seed:err-payload", 7);
}

#[test]
fn ignored_field_twin_remains_green() {
    let dir = output_dir("ignored");
    ken_cli::build_native_program(
        IGNORED_PROGRAM,
        ken_cli::SourceFormat::Ken,
        "px7p-ignored-field-opposite",
        &dir,
    )
    .expect("the byte-near ignored-field opposite remains on ordinary lowering");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn dynamic_carrier_producer_payload_reaches_linked_process_exit() {
    let dir = output_dir("dynamic-carrier-producer");
    let output = ken_cli::build_native_program(
        DYNAMIC_CARRIER_PROGRAM,
        ken_cli::SourceFormat::Ken,
        "px7p-dynamic-carrier-producer",
        &dir,
    )
    .expect("the generic dynamic carrier preserves its producer continuation");
    let long_component = std::ffi::OsString::from("a".repeat(300));
    let native = ken_runtime::run_bound_process_effect_observation(
        &output.artifact,
        &ken_runtime::NativeEffectRunOptionsV1 {
            arguments: vec![long_component],
            environment: Vec::new(),
            cwd: dir.clone(),
            plan_hash: output.plan_transport_hash,
        },
    )
    .expect("linked artifact returns its complete dynamic-carrier observation");

    assert_eq!(native.exit_status, 36);
    assert_eq!(native.effect_trace.len(), 2);
    let ken_runtime::CanonicalOutcomeV1::Error(ken_runtime::SemanticErrorV1::File(error)) =
        &native.effect_trace[0].outcome
    else {
        panic!("expected one filesystem error observation");
    };
    assert_eq!(error.operation, ken_runtime::HostOpV1::FsReadFile);
    assert_eq!(format!("{:?}", error.cause), "Io(Other(36))");
    assert_eq!(
        native.effect_trace[1].operation,
        ken_runtime::HostOpV1::ConsoleWrite
    );
    assert!(matches!(
        &native.effect_trace[1].request,
        ken_runtime::CanonicalRequestV1::ConsoleWrite { bytes, .. }
            if bytes == b"other-36"
    ));
    let _ = std::fs::remove_dir_all(dir);
}
