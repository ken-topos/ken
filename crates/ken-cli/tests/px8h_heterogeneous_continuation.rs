fn output_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "ken-px8h-{name}-{}-{}",
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
fn retain_tree
  (tree : HostIO APartial (Result Int Bool))
  (_written : Result IOError Unit)
  : HostIO APartial (Result Int Bool) = tree

proc step (marker : Bytes) (tree : HostIO APartial (Result Int Bool))
  : HostIO APartial (Result Int Bool) visits [Console] =
  bind (Coproduct (FSOp APartial) AmbientOp)
    (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
    (Result IOError Unit) (Result Int Bool)
    (host_console APartial (Result IOError Unit) (write Stdout marker))
    (retain_tree tree)

fn zero_tree (_unit : Unit) : HostIO APartial (Result Int Bool) =
  Ret (Coproduct (FSOp APartial) AmbientOp)
    (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
    (Result Int Bool) (Ok Int Bool True)

proc multi_tree (_unit : Unit)
  : HostIO APartial (Result Int Bool) visits [Console] =
  step (bytes_encode "A")
    (step (bytes_encode "B")
      (step (bytes_encode "C")
        (Ret (Coproduct (FSOp APartial) AmbientOp)
          (resp_coproduct (FSOp APartial) AmbientOp
            (fs_resp APartial) ambient_resp)
          (Result Int Bool) (Err Int Bool (7 : Int)))))

fn exit_with (code : ExitCode) (_flushed : Result IOError Unit)
  : HostIO APartial ExitCode = host_exit APartial code

proc flush_then_exit (code : ExitCode) (_written : Result IOError Unit)
  : HostIO APartial ExitCode visits [Console] =
  bind (Coproduct (FSOp APartial) AmbientOp)
    (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
    (Result IOError Unit) ExitCode
    (host_console APartial (Result IOError Unit) (flush Stdout))
    (exit_with code)

proc finish (outcome : Result Int Bool)
  : HostIO APartial ExitCode visits [Console] =
  match outcome {
    Err value |-> bind (Coproduct (FSOp APartial) AmbientOp)
      (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
      (Result IOError Unit) ExitCode
      (host_console APartial (Result IOError Unit)
        (write Stdout (bytes_encode ":final-E")))
      (flush_then_exit (Failure 7));
    Ok value |-> bind (Coproduct (FSOp APartial) AmbientOp)
      (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
      (Result IOError Unit) ExitCode
      (host_console APartial (Result IOError Unit)
        (write Stdout (bytes_encode ":final-O")))
      (flush_then_exit Success)
  }

proc main (input : ProcessInput) (_caps : ProgramCaps APartial)
  : HostIO APartial ExitCode visits [Console] =
  match input {
    MkProcessInput arguments _environment _cwd |-> match arguments {
      Nil |-> host_exit APartial (Failure 99);
      Cons _argv0 rest |-> match rest {
        Nil |-> bind (Coproduct (FSOp APartial) AmbientOp)
          (resp_coproduct (FSOp APartial) AmbientOp
            (fs_resp APartial) ambient_resp)
          (Result Int Bool) ExitCode (zero_tree MkUnit) (\outcome. finish outcome);
        Cons _argument _tail |-> bind (Coproduct (FSOp APartial) AmbientOp)
          (resp_coproduct (FSOp APartial) AmbientOp
            (fs_resp APartial) ambient_resp)
          (Result Int Bool) ExitCode (multi_tree MkUnit) (\outcome. finish outcome)
      }
    }
  }
"#;

fn assert_case(arguments: &[&str], expected_stdout: &[u8], expected_exit: i32) {
    let name = if arguments.is_empty() {
        "zero"
    } else {
        "multi"
    };
    let dir = output_dir(name);
    let output = ken_cli::build_native_program(
        PROGRAM,
        ken_cli::SourceFormat::Ken,
        "px8h-heterogeneous-continuation",
        &dir,
    )
    .expect("resource-independent heterogeneous continuation compiles");
    let native = ken_runtime::run_bound_process_effect_observation(
        &output.artifact,
        &ken_runtime::NativeEffectRunOptionsV1 {
            arguments: arguments.iter().map(std::ffi::OsString::from).collect(),
            environment: Vec::new(),
            cwd: dir.clone(),
            plan_hash: output.plan_transport_hash,
        },
    )
    .expect("linked heterogeneous continuation runs");

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
    .expect("the interpreter runs the same checked source");

    assert_eq!(native, interpreted);
    assert_eq!(native.stdout, expected_stdout);
    assert_eq!(native.exit_status, expected_exit);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn zero_step_preserves_the_direct_payload_direction() {
    assert_case(&[], b":final-O", 0);
}

#[test]
fn multiple_steps_preserve_the_recursive_payload_direction() {
    assert_case(&["three"], b"ABC:final-E", 7);
}
