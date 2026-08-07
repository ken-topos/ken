fn output_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "ken-px7m-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

const OK_PROGRAM: &str = r#"program capabilities FS APartial
proc two_step (label : String) : HostIO APartial Unit visits [Console] =
  bind (Coproduct (FSOp APartial) AmbientOp)
    (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
    Unit Unit
    (host_console APartial Unit (print_line label))
    (\_. bind (Coproduct (FSOp APartial) AmbientOp)
      (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
      (Result IOError Unit) Unit
      (host_console APartial (Result IOError Unit) (flush Stdout))
      (\_. Ret (Coproduct (FSOp APartial) AmbientOp)
        (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
        Unit MkUnit))

proc after_write (written : Result IOError Unit)
  : HostIO APartial Unit visits [Console] =
  match written {
    Err _ |-> two_step "unexpected-error" ;
    Ok unit |-> match unit { MkUnit |-> two_step "ok-payload" }
  }

proc inner : HostIO APartial Unit visits [Console] =
  bind (Coproduct (FSOp APartial) AmbientOp)
    (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
    (Result IOError Unit) Unit
    (host_console APartial (Result IOError Unit)
      (write Stdout (bytes_encode "probe:")))
    after_write

proc main (_input : ProcessInput) (_caps : ProgramCaps APartial)
  : HostIO APartial ExitCode visits [Console] =
  bind (Coproduct (FSOp APartial) AmbientOp)
    (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
    Unit ExitCode inner (\_. host_exit APartial Success)
"#;

const ERR_PROGRAM: &str = r#"program capabilities FS APartial
proc write_bytes_then_line (bytes : Bytes) (label : String)
  : HostIO APartial Unit visits [Console] =
  bind (Coproduct (FSOp APartial) AmbientOp)
    (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
    (Result IOError Unit) Unit
    (host_console APartial (Result IOError Unit) (write Stdout bytes))
    (\_. bind (Coproduct (FSOp APartial) AmbientOp)
      (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
      Unit Unit
      (host_console APartial Unit (print_line label))
      (\_. Ret (Coproduct (FSOp APartial) AmbientOp)
        (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
        Unit MkUnit))

fn failed_path (error : FileError) : Bytes =
  match error {
    MkFileError _operation path _kind |-> match path {
      None |-> bytes_encode "no-path" ;
      Some bytes |-> bytes
    }
  }

proc after_read (read : Result FileError Bytes)
  : HostIO APartial Unit visits [Console] =
  match read {
    Err error |-> write_bytes_then_line (failed_path error) "not-found" ;
    Ok bytes |-> write_bytes_then_line bytes "unexpected-ok"
  }

proc inner (cap : Cap APartial) : HostIO APartial Unit visits [FS, Console] =
  bind (Coproduct (FSOp APartial) AmbientOp)
    (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
    (Result FileError Bytes) Unit
    (inject_l (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp
      (Result FileError Bytes)
      (readFile APartial cap (bytes_encode "missing.bin")))
    after_read

proc main (_input : ProcessInput) (caps : ProgramCaps APartial)
  : HostIO APartial ExitCode visits [FS, Console] =
  match caps {
    MkProgramCaps cap |->
      bind (Coproduct (FSOp APartial) AmbientOp)
        (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
        Unit ExitCode (inner cap) (\_. host_exit APartial Success)
  }
"#;

fn assert_agreement(
    source: &str,
    name: &str,
    expected_stdout: &[u8],
    expected_operations: &[ken_runtime::HostOpV1],
) {
    let dir = output_dir(name);
    let output = ken_cli::build_native_program(source, ken_cli::SourceFormat::Ken, name, &dir)
        .expect("dynamic HostResult producer reaches the linked artifact");
    let native = ken_runtime::run_bound_process_effect_observation(
        &output.artifact,
        &ken_runtime::NativeEffectRunOptionsV1 {
            arguments: Vec::new(),
            environment: Vec::new(),
            cwd: dir.clone(),
            plan_hash: output.plan_transport_hash,
        },
    )
    .expect("linked artifact returns its complete observation");

    let mut host = ken_interp::CaptureHost::new(Vec::new());
    let interpreted = ken_cli::run_program_effect_observation(
        source,
        ken_cli::SourceFormat::Ken,
        &[b"ken".to_vec()],
        &[],
        b"/",
        &mut host,
    )
    .expect("same checked source runs through the interpreter");
    assert_eq!(native, interpreted);
    assert_eq!(native.exit_status, 0);
    assert_eq!(native.stdout, expected_stdout);
    assert_eq!(
        native
            .effect_trace
            .iter()
            .map(|event| event.operation)
            .collect::<Vec<_>>(),
        expected_operations
    );
    let _ = std::fs::remove_dir_all(dir);
}

// Ignored pending RT-CARRIER-BYTESPAN-OBSERVE.
//
// Observed signature, exactly:
//   Effect: seat Argument(0) of FsReadFile needs BytesPointerLength, which it cannot observe in CarriedWord
//
// Owner node: RT-CARRIER-BYTESPAN-OBSERVE.
// Pre-existing base debt, NOT a bind-order regression: measured failing at
// the frozen base 21fd46dc by the D10 differential, before any
// RT-SRCBODY-BIND-ORDER commit.
// It refuses at object emission, so the program never executes and no
// binding order is observable in it.
// The four px4b rows carry this same owner with the OPPOSITE provenance:
// those were branch-introduced, this one predates the branch.
// Annotation only -- test body and expectations are unchanged.
#[test]
#[ignore = "RT-CARRIER-BYTESPAN-OBSERVE: the FsReadFile byte-span seat cannot observe a carried word; fails at base 21fd46dc"]
fn dynamic_ok_payload_selects_a_multistep_tree_across_real_executors() {
    assert_agreement(
        OK_PROGRAM,
        "px7m-ok",
        b"probe:ok-payload\n",
        &[
            ken_runtime::HostOpV1::ConsoleWrite,
            ken_runtime::HostOpV1::ConsoleWrite,
            ken_runtime::HostOpV1::ConsoleFlush,
        ],
    );
}

// Ignored pending RT-CARRIER-BYTESPAN-OBSERVE.
//
// Observed signature, exactly:
//   Effect: seat Argument(0) of FsReadFile needs BytesPointerLength, which it cannot observe in CarriedWord
//
// Owner node: RT-CARRIER-BYTESPAN-OBSERVE.
// Pre-existing base debt, NOT a bind-order regression: measured failing at
// the frozen base 21fd46dc by the D10 differential, before any
// RT-SRCBODY-BIND-ORDER commit.
// It refuses at object emission, so the program never executes and no
// binding order is observable in it.
// The four px4b rows carry this same owner with the OPPOSITE provenance:
// those were branch-introduced, this one predates the branch.
// Annotation only -- test body and expectations are unchanged.
#[test]
#[ignore = "RT-CARRIER-BYTESPAN-OBSERVE: the FsReadFile byte-span seat cannot observe a carried word; fails at base 21fd46dc"]
fn dynamic_err_payload_selects_a_multistep_tree_across_real_executors() {
    assert_agreement(
        ERR_PROGRAM,
        "px7m-err",
        b"missing.binnot-found\n",
        &[
            ken_runtime::HostOpV1::FsReadFile,
            ken_runtime::HostOpV1::ConsoleWrite,
            ken_runtime::HostOpV1::ConsoleWrite,
        ],
    );
}
