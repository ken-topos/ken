use std::process::Command;

const PURE_PROGRAM: &str = r#"program capabilities FS APartial
fn unused_sibling (_input : ProcessInput) : ExitCode = Success
fn main (_input : ProcessInput) (_caps : ProgramCaps APartial)
  : HostIO APartial ExitCode = host_exit APartial Success
"#;

const PROCESS_BYTES_PROGRAM: &str = r#"program capabilities FS APartial
fn process_discriminator (input : ProcessInput) : UInt8 =
  match input {
    MkProcessInput arguments environment cwd |->
      match arguments {
        Nil |-> 1 ;
        Cons _argv0 rest |-> match rest {
          Nil |-> 2 ;
          Cons argument _more |-> match environment {
            Nil |-> 3 ;
            Cons binding bindings |-> match bindings {
              Cons _ _ |-> 8 ;
              Nil |-> match binding {
              MkProd key value |-> match bytes_at argument 0 {
                None |-> 4 ;
                Some argument_byte |-> match bytes_at key 0 {
                  None |-> 5 ;
                  Some key_byte |-> match bytes_at value 0 {
                    None |-> 6 ;
                    Some value_byte |-> match bytes_at cwd 0 {
                      None |-> 7 ;
                      Some cwd_byte |->
                        match eq_int (uint8_to_int argument_byte) 255 {
                          False |-> argument_byte ;
                          True |-> match eq_int (uint8_to_int key_byte) 75 {
                            False |-> key_byte ;
                            True |-> match eq_int (uint8_to_int value_byte) 254 {
                              False |-> value_byte ;
                              True |-> match eq_int (uint8_to_int cwd_byte) 47 {
                                False |-> cwd_byte ;
                                True |-> value_byte
                              }
                            }
                          }
                        }
                    }
                  }
                }
              }
              }
            }
          }
        }
      }
  }
fn main (input : ProcessInput) (_caps : ProgramCaps APartial)
  : HostIO APartial ExitCode =
  host_exit APartial (Failure (process_discriminator input))
"#;

fn output_dir(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "ken-px4b-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&path).unwrap();
    path
}

#[cfg(target_os = "linux")]
#[test]
fn linked_home_root_uses_only_production_account_database_boundary() {
    let dir = output_dir("px16-home-root");
    let source = r#"program capabilities FS APartial "~/", RootExecution Allow
fn main (_input : ProcessInput) (_caps : ProgramCaps APartial)
  : HostIO APartial ExitCode = host_exit APartial Success
"#;
    let output =
        ken_cli::build_native_program(source, ken_cli::SourceFormat::Ken, "px16-home-root", &dir)
            .expect("checked ~/ root reaches a linked artifact");
    let observation = ken_runtime::run_bound_process_effect_observation(
        &output.artifact,
        &ken_runtime::NativeEffectRunOptionsV1 {
            arguments: Vec::new(),
            environment: Vec::new(),
            cwd: dir.clone(),
            plan_hash: output.plan_transport_hash,
        },
    )
    .expect("linked child emits a complete startup observation");
    match observation.terminal_error {
        None => assert_eq!(observation.exit_status, 0),
        Some(ken_runtime::TerminalErrorV1::HomeRootResolutionFailed(
            ken_runtime::HomeRootResolutionFailureV1::NoAccountRecord,
        )) => assert_ne!(observation.exit_status, 0),
        other => panic!("unexpected production account-database result: {other:?}"),
    }
    assert!(observation.stdout.is_empty());
    assert!(observation.stderr.is_empty());
    assert!(observation.filesystem_delta.is_empty());
    assert!(observation.effect_trace.is_empty());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn real_source_builds_one_identity_bound_linked_process_artifact() {
    let dir = output_dir("pure");
    let output =
        ken_cli::build_native_program(PURE_PROGRAM, ken_cli::SourceFormat::Ken, "px4b-pure", &dir)
            .expect("checked source reaches native artifact");

    assert_eq!(
        output.package.core_semantic_hash,
        output.runtime_program.core_semantic_hash
    );
    assert_eq!(
        output.package.artifact_hash,
        output.runtime_program.artifact_hash
    );
    assert_eq!(
        output.artifact.runtime_artifact.core_semantic_hash,
        output.package.core_semantic_hash
    );
    assert!(matches!(
        output.report.runtime_lowering,
        ken_elaborator::compiler_driver::ReportFact::Emitted
    ));
    assert!(matches!(
        output.report.native_artifact,
        ken_elaborator::compiler_driver::ReportFact::Emitted
    ));
    assert!(!output.plan.main().to_string().contains("prelude::"));

    let reported = &output.closure.reachable_declarations;
    let executable = &output.executable_closure;
    let metadata = &output
        .runtime_program
        .erased_core
        .metadata
        .runtime_declaration_targets;
    let declarations = output
        .runtime_program
        .declarations
        .iter()
        .map(|declaration| declaration.symbol.clone())
        .collect::<std::collections::BTreeSet<_>>();
    let reported_runtime = reported
        .iter()
        .map(ToString::to_string)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(reported, executable);
    assert_eq!(&reported_runtime, metadata);
    assert_eq!(reported_runtime, declarations);
    assert!(reported
        .iter()
        .all(|symbol| !symbol.to_string().contains("unused_sibling")));

    let mut stale_plan = output.package.clone();
    let plan_bytes = stale_plan
        .artifact
        .semantic
        .metadata
        .values_mut()
        .find(|bytes| bytes.starts_with(b"NativeEntrypointPlanV1\0"))
        .expect("plan is contained in semantic inputs");
    plan_bytes.push(0xff);
    assert!(matches!(
        ken_elaborator::checked_core::validate_checked_core_package(&stale_plan),
        Err(ken_elaborator::checked_core::CheckedCorePackageError::SemanticHashMismatch { .. })
    ));
    stale_plan.core_semantic_hash =
        ken_elaborator::checked_core::semantic_fingerprint(&stale_plan.artifact.semantic);
    assert!(matches!(
        ken_elaborator::checked_core::validate_checked_core_package(&stale_plan),
        Err(ken_elaborator::checked_core::CheckedCorePackageError::ArtifactHashMismatch { .. })
    ));

    let mut command = Command::new(&output.artifact.executable_path);
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        command.arg(std::ffi::OsString::from_vec(vec![0xff]));
    }
    #[cfg(not(unix))]
    command.arg("raw-argv");
    let ran = command
        .env("KEN_PX4B_RAW", "bytes")
        .current_dir(&dir)
        .output()
        .expect("linked process artifact runs with fresh process data");
    assert_eq!(ran.status.code(), Some(0), "stderr: {:?}", ran.stderr);
    let _ = std::fs::remove_dir_all(dir);
}

#[cfg(target_os = "linux")]
#[test]
fn public_source_observes_raw_argv_environment_cwd_bytes_in_field_order() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let dir = output_dir("process-bytes");
    let output = ken_cli::build_native_program(
        PROCESS_BYTES_PROGRAM,
        ken_cli::SourceFormat::Ken,
        "px4b-process-bytes",
        &dir,
    )
    .expect("checked byte discriminator reaches native artifact");
    let observation_dir = output_dir("process-observation");
    let observation_path = observation_dir.join("process.observation");

    let run = |argument: u8| {
        Command::new(&output.artifact.executable_path)
            .arg(OsString::from_vec(vec![argument]))
            .env_clear()
            .env("K", OsString::from_vec(vec![0xfe]))
            .env("KEN_HOST_OBSERVATION_PATH", &observation_path)
            .current_dir(&dir)
            .output()
            .expect("linked artifact observes fresh raw process bytes")
    };

    // The first arm verifies argv=0xff, key='K', value=0xfe, and cwd='/' before
    // returning the raw environment byte. Changing argv reaches the distinct
    // fallback, so no field can be dropped, substituted, or reordered.
    let first = run(0xff);
    let second = run(0xfd);
    assert_eq!(first.status.code(), Some(254), "stderr: {:?}", first.stderr);
    assert_eq!(
        second.status.code(),
        Some(253),
        "stderr: {:?}",
        second.stderr
    );
    assert_ne!(first.status.code(), second.status.code());
    assert!(observation_path.is_file());
    let _ = std::fs::remove_dir_all(observation_dir);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn authority_mismatch_fails_before_any_artifact_is_written() {
    let dir = output_dir("mismatch");
    let source = PURE_PROGRAM.replace("ProgramCaps APartial", "ProgramCaps AFull");
    let error =
        ken_cli::build_native_program(&source, ken_cli::SourceFormat::Ken, "px4b-mismatch", &dir)
            .expect_err("declared/type authority mismatch must reject");
    assert!(matches!(
        error,
        ken_elaborator::compiler_driver::NativeProgramBuildError::Admission(
            ken_elaborator::program_admission::ProgramAdmissionError::InvalidMainAbi { .. }
        )
    ));
    assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn one_vis_reaches_the_px5_native_host_dispatch() {
    let dir = output_dir("vis");
    let source = r#"program capabilities FS APartial
proc main (_input : ProcessInput) (_caps : ProgramCaps APartial)
  : HostIO APartial ExitCode visits [Console] =
  host_program APartial (print_line "px5")
"#;
    let output = ken_cli::build_native_program(source, ken_cli::SourceFormat::Ken, "px5-vis", &dir)
        .expect("checked Vis reaches the PX5 artifact lane");
    let ran = Command::new(&output.artifact.executable_path)
        .output()
        .expect("linked host-effect artifact runs");
    assert_eq!(ran.status.code(), Some(0), "stderr: {:?}", ran.stderr);
    assert_eq!(ran.stdout, b"px5\n");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn two_vis_nodes_resume_once_in_source_order() {
    let dir = output_dir("two-vis");
    let source = r#"program capabilities FS APartial
proc main (_input : ProcessInput) (_caps : ProgramCaps APartial)
  : HostIO APartial ExitCode visits [Console] =
  host_program_then APartial
    (bind ConsoleOp console_resp Unit Unit
      (print_line "one")
      (\_. print_line "two"))
    Success
"#;
    let output =
        ken_cli::build_native_program(source, ken_cli::SourceFormat::Ken, "px5-two-vis", &dir)
            .expect("two checked Vis nodes reach one artifact");
    let native = ken_runtime::run_bound_process_effect_observation(
        &output.artifact,
        &ken_runtime::NativeEffectRunOptionsV1 {
            arguments: Vec::new(),
            environment: Vec::new(),
            cwd: dir.clone(),
            plan_hash: output.plan_transport_hash,
        },
    )
    .expect("the production launcher decodes the complete observation");

    let mut host = ken_interp::CaptureHost::new(Vec::new());
    let interpreted = ken_cli::run_program(
        source,
        ken_cli::SourceFormat::Ken,
        &[b"ken".to_vec()],
        &[],
        b"/",
        &mut host,
    )
    .expect("the same checked source runs through the interpreter");
    let expected_trace = [b"one\n".as_slice(), b"two\n".as_slice()]
        .into_iter()
        .enumerate()
        .map(|(sequence, bytes)| ken_runtime::EffectEvent {
            sequence: sequence as u64,
            operation: ken_runtime::HostOpV1::ConsoleWrite,
            capability: None,
            resource_bindings: Vec::new(),
            request: ken_runtime::CanonicalRequestV1::ConsoleWrite {
                stream: ken_runtime::ConsoleStreamV1::Stdout,
                bytes: bytes.to_vec(),
            },
            outcome: ken_runtime::CanonicalOutcomeV1::Success(ken_runtime::CanonicalReplyV1::Unit),
        })
        .collect();
    let interpreted = ken_runtime::EffectObservation {
        stdout: host.stdout().to_vec(),
        stderr: host.stderr().to_vec(),
        filesystem_delta: Vec::new(),
        terminal_error: None,
        effect_trace: expected_trace,
        terminal_exit: ken_runtime::TerminalExitClass::NormalReturn,
        exit_status: interpreted.exit_status,
    };
    assert_eq!(native, interpreted);

    let mut mutations = Vec::new();
    let mut changed = native.clone();
    changed.stdout.push(1);
    mutations.push(changed);
    let mut changed = native.clone();
    changed.stderr.push(1);
    mutations.push(changed);
    let mut changed = native.clone();
    changed
        .filesystem_delta
        .push(ken_runtime::FsDeltaV1::Created {
            relative_path: b"mutation".to_vec(),
            node: ken_runtime::FsNodeObservationV1 {
                kind: ken_runtime::FsNodeKindV1::File,
                file_bytes: Some(Vec::new()),
                symlink_target: None,
                mode: Some(0o644),
            },
        });
    mutations.push(changed);
    let mut changed = native.clone();
    changed.terminal_error = Some(ken_runtime::TerminalErrorV1::DriverFailure);
    mutations.push(changed);
    let mut changed = native.clone();
    changed.effect_trace.pop();
    mutations.push(changed);
    let mut changed = native.clone();
    changed.exit_status ^= 1;
    mutations.push(changed);
    for changed in mutations {
        assert_ne!(changed, interpreted);
    }
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn host_reply_selects_the_continuation_outcome() {
    let dir = output_dir("reply-dependent");
    let source = r#"program capabilities FS APartial
proc main (_input : ProcessInput) (_caps : ProgramCaps APartial)
  : HostIO APartial ExitCode visits [Console] =
  bind (Coproduct (FSOp APartial) AmbientOp)
    (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
    Bool ExitCode
    (host_console APartial Bool (is_terminal Stdout))
    (\terminal. match terminal {
      False |-> host_exit APartial (Failure 23) ;
      True |-> host_exit APartial (Failure 24)
    })
"#;
    let output = ken_cli::build_native_program(
        source,
        ken_cli::SourceFormat::Ken,
        "px5-reply-dependent",
        &dir,
    )
    .expect("checked response-dependent continuation reaches the artifact");
    let ran = Command::new(&output.artifact.executable_path)
        .output()
        .expect("response-dependent artifact runs");
    assert_eq!(ran.status.code(), Some(23), "stderr: {:?}", ran.stderr);
    let _ = std::fs::remove_dir_all(dir);
}

#[cfg(target_os = "linux")]
#[test]
fn linked_console_broken_pipe_reaches_ken_instead_of_signal_termination() {
    use std::os::unix::ffi::OsStringExt;

    let dir = output_dir("broken-pipe");
    let source = r#"program capabilities FS APartial
proc main (input : ProcessInput) (_caps : ProgramCaps APartial)
  : HostIO APartial ExitCode visits [Console] =
  match input {
    MkProcessInput arguments _environment _cwd |-> match arguments {
      Nil |-> host_exit APartial (Failure 60) ;
      Cons _argv0 rest |-> match rest {
        Nil |-> host_exit APartial (Failure 60) ;
        Cons payload _ |->
          bind (Coproduct (FSOp APartial) AmbientOp)
            (resp_coproduct (FSOp APartial) AmbientOp
              (fs_resp APartial) ambient_resp)
            (Result IOError Unit) ExitCode
            (host_console APartial (Result IOError Unit) (write Stdout payload))
            (\written. match written {
              Ok _ |-> host_exit APartial (Failure 62) ;
              Err error |-> match error {
                CapabilityDenied |-> host_exit APartial (Failure 63) ;
                BrokenPipe |-> host_exit APartial (Failure 61) ;
                NotFound |-> host_exit APartial (Failure 63) ;
                PermissionDenied |-> host_exit APartial (Failure 63) ;
                Interrupted |-> host_exit APartial (Failure 63) ;
                AlreadyExists |-> host_exit APartial (Failure 63) ;
                InvalidInput |-> host_exit APartial (Failure 63) ;
                IsDirectory |-> host_exit APartial (Failure 63) ;
                NotDirectory |-> host_exit APartial (Failure 63) ;
                NotEmpty |-> host_exit APartial (Failure 63) ;
                Unsupported |-> host_exit APartial (Failure 63) ;
                Other _ |-> host_exit APartial (Failure 63)
              }
            })
      }
    }
  }
"#;
    let output =
        ken_cli::build_native_program(source, ken_cli::SourceFormat::Ken, "px5-broken-pipe", &dir)
            .expect("checked BrokenPipe observer reaches the linked artifact");

    let mut child = Command::new(&output.artifact.executable_path)
        .arg(std::ffi::OsString::from_vec(vec![b'x'; 96 * 1024]))
        .env(
            "KEN_HOST_OBSERVATION_PATH",
            dir.join("broken-pipe.observation"),
        )
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("BrokenPipe artifact starts");
    drop(child.stdout.take().expect("stdout reader was piped"));
    let status = child.wait().expect("BrokenPipe artifact terminates");
    assert_eq!(status.code(), Some(61));
    let observation = std::fs::read(dir.join("broken-pipe.observation")).unwrap();
    let observation = ken_runtime::decode_linked_effect_trace(&observation).unwrap();
    assert_eq!(observation.effect_trace.len(), 1);
    assert_eq!(
        observation.effect_trace[0].outcome,
        ken_runtime::CanonicalOutcomeV1::Error(ken_runtime::SemanticErrorV1::Io(
            ken_runtime::IoErrorIdentityV1::BrokenPipe,
        ))
    );

    let starter_source = include_str!("../../ken-runtime/src/object_linker_packaging.rs");
    for forbidden in ["SIGPIPE", "SIG_IGN", "sigaction", "signal("] {
        assert!(!starter_source.contains(forbidden));
    }
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn fs_write_and_read_resume_through_the_native_capability() {
    let dir = output_dir("fs-roundtrip");
    let source = r#"program capabilities FS AFull
proc main (input : ProcessInput) (caps : ProgramCaps AFull)
  : HostIO AFull ExitCode visits [FS, Console] =
  match input {
    MkProcessInput arguments _environment _cwd |-> match arguments {
      Nil |-> host_exit AFull (Failure 30) ;
      Cons _argv0 rest |-> match rest {
        Nil |-> host_exit AFull (Failure 31) ;
        Cons path more |-> match more {
          Nil |-> host_exit AFull (Failure 32) ;
          Cons contents _ |-> match caps {
            MkProgramCaps cap |->
              bind (Coproduct (FSOp AFull) AmbientOp)
                (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
                (Result FileError Unit) ExitCode
                (inject_l (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp
                  (Result FileError Unit)
                  (writeFile cap path CreateNew contents))
                (\written. match written {
                  Err _ |-> host_exit AFull (Failure 34) ;
                  Ok _ |->
                    bind (Coproduct (FSOp AFull) AmbientOp)
                      (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
                      (Result FileError Bytes) ExitCode
                      (inject_l (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp
                        (Result FileError Bytes) (readFile AFull cap path))
                      (\read. match read {
                        Err _ |-> host_exit AFull (Failure 35) ;
                        Ok bytes |->
                          bind (Coproduct (FSOp AFull) AmbientOp)
                            (resp_coproduct (FSOp AFull) AmbientOp
                              (fs_resp AFull) ambient_resp)
                            (Result IOError Unit) ExitCode
                            (host_console AFull (Result IOError Unit) (flush Stdout))
                            (\_. match bytes_at bytes 0 {
                              None |-> host_exit AFull (Failure 36) ;
                              Some byte |-> host_exit AFull (Failure byte)
                            })
                      })
                })
          }
        }
      }
    }
  }
"#;
    let output =
        ken_cli::build_native_program(source, ken_cli::SourceFormat::Ken, "px5-fs-roundtrip", &dir)
            .expect("FS Vis nodes reach the native capability lane");
    let observation = ken_runtime::run_bound_process_effect_observation(
        &output.artifact,
        &ken_runtime::NativeEffectRunOptionsV1 {
            arguments: vec!["px5.bin".into(), "retained".into()],
            environment: Vec::new(),
            cwd: dir.clone(),
            plan_hash: output.plan_transport_hash,
        },
    )
    .expect("the production launcher returns the complete FS observation");
    assert_eq!(observation.exit_status, b'r' as i32);
    assert_eq!(std::fs::read(dir.join("px5.bin")).unwrap(), b"retained");
    assert_eq!(observation.effect_trace.len(), 3);
    assert_eq!(
        observation.effect_trace[0]
            .capability
            .as_ref()
            .map(|identity| identity.0.as_str()),
        Some("FS")
    );
    assert_eq!(
        observation.effect_trace[1]
            .capability
            .as_ref()
            .map(|identity| identity.0.as_str()),
        Some("FS")
    );
    assert_eq!(observation.effect_trace[2].capability, None);
    assert_eq!(
        observation.filesystem_delta,
        vec![ken_runtime::FsDeltaV1::Created {
            relative_path: b"px5.bin".to_vec(),
            node: ken_runtime::FsNodeObservationV1 {
                kind: ken_runtime::FsNodeKindV1::File,
                file_bytes: Some(b"retained".to_vec()),
                symlink_target: None,
                mode: Some(0o644),
            },
        }]
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn canonical_fs_identity_exactly_matches_across_real_producers_and_drift_fails() {
    let dir = output_dir("fs-identity-cross-lane");
    let path = b"shared.bin";
    let contents = vec![17, 0xff, 0];
    std::fs::write(dir.join("shared.bin"), &contents).unwrap();
    let source = r#"program capabilities FS APartial
proc main (input : ProcessInput) (caps : ProgramCaps APartial)
  : HostIO APartial ExitCode visits [FS, Console] =
  match input {
    MkProcessInput arguments _environment _cwd |-> match arguments {
      Nil |-> host_exit APartial (Failure 40) ;
      Cons _argv0 rest |-> match rest {
        Nil |-> host_exit APartial (Failure 41) ;
        Cons path _ |-> match caps {
          MkProgramCaps cap |->
            bind (Coproduct (FSOp APartial) AmbientOp)
              (resp_coproduct (FSOp APartial) AmbientOp
                (fs_resp APartial) ambient_resp)
              (Result FileError Bytes) ExitCode
              (inject_l (FSOp APartial) AmbientOp
                (fs_resp APartial) ambient_resp
                (Result FileError Bytes) (readFile APartial cap path))
              (\read. match read {
                Err _ |-> host_exit APartial (Failure 42) ;
                Ok bytes |-> match bytes_at bytes 0 {
                  None |-> host_exit APartial (Failure 43) ;
                  Some byte |-> host_exit APartial (Failure byte)
                }
              })
        }
      }
    }
  }
"#;

    let output =
        ken_cli::build_native_program(source, ken_cli::SourceFormat::Ken, "px5c-fs-identity", &dir)
            .expect("same checked source reaches the native producer");
    let native = ken_runtime::run_bound_process_effect_observation(
        &output.artifact,
        &ken_runtime::NativeEffectRunOptionsV1 {
            arguments: vec![std::ffi::OsString::from("shared.bin")],
            environment: Vec::new(),
            cwd: dir.clone(),
            plan_hash: output.plan_transport_hash,
        },
    )
    .expect("native producer returns its real observation");

    let mut host = ken_interp::CaptureHost::new(Vec::new());
    host.insert_file(path.to_vec(), contents);
    let interpreted = ken_cli::run_program_effect_observation(
        source,
        ken_cli::SourceFormat::Ken,
        &[b"ken".to_vec(), path.to_vec()],
        &[],
        b"/",
        &mut host,
    )
    .expect("interpreter producer returns its real observation");

    assert_eq!(interpreted, native);
    assert_eq!(interpreted.effect_trace.len(), 1);
    assert_eq!(
        interpreted.effect_trace[0]
            .capability
            .as_ref()
            .map(|identity| identity.0.as_str()),
        Some("FS")
    );

    for drift in ["interpreter:FS", "declared:FS", "other:FS"] {
        let mut interpreter_drift = interpreted.clone();
        interpreter_drift.effect_trace[0]
            .capability
            .as_mut()
            .expect("successful FS event has an identity")
            .0 = drift.to_string();
        assert_ne!(
            interpreter_drift, native,
            "interpreter seed drift must fail"
        );

        let mut native_drift = native.clone();
        native_drift.effect_trace[0]
            .capability
            .as_mut()
            .expect("successful FS event has an identity")
            .0 = drift.to_string();
        assert_ne!(interpreted, native_drift, "native seed drift must fail");
    }

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn fs_scope_denial_reaches_ken_as_the_named_error() {
    let dir = output_dir("fs-denial");
    let source = r#"program capabilities FS AFull
proc main (input : ProcessInput) (caps : ProgramCaps AFull)
  : HostIO AFull ExitCode visits [FS, Console] =
  match input {
    MkProcessInput arguments _environment _cwd |-> match arguments {
      Nil |-> host_exit AFull (Failure 40) ;
      Cons _argv0 rest |-> match rest {
        Nil |-> host_exit AFull (Failure 41) ;
        Cons path _ |-> match caps {
          MkProgramCaps cap |->
            bind (Coproduct (FSOp AFull) AmbientOp)
              (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
              (Result FileError Unit) ExitCode
              (inject_l (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp
                (Result FileError Unit) (writeFile cap path CreateNew path))
              (\written. match written {
                Ok _ |-> host_exit AFull (Failure 43) ;
                Err error |-> match error {
                  MkFileError _operation _path cause |-> match cause {
                    CapabilityDenied |-> host_exit AFull (Failure 44) ;
                    NotFound |-> host_exit AFull (Failure 45) ;
                    PermissionDenied |-> host_exit AFull (Failure 46) ;
                    BrokenPipe |-> host_exit AFull (Failure 47) ;
                    Interrupted |-> host_exit AFull (Failure 48) ;
                    AlreadyExists |-> host_exit AFull (Failure 49) ;
                    InvalidInput |-> host_exit AFull (Failure 50) ;
                    IsDirectory |-> host_exit AFull (Failure 51) ;
                    NotDirectory |-> host_exit AFull (Failure 52) ;
                    NotEmpty |-> host_exit AFull (Failure 53) ;
                    Unsupported |-> host_exit AFull (Failure 54) ;
                    Other _ |-> host_exit AFull (Failure 55)
                  }
                }
              })
        }
      }
    }
  }
"#;
    let output =
        ken_cli::build_native_program(source, ken_cli::SourceFormat::Ken, "px5-fs-denial", &dir)
            .expect("checked FS denial program reaches the artifact");
    let ran = Command::new(&output.artifact.executable_path)
        .arg("../escape")
        .current_dir(&dir)
        .output()
        .expect("FS denial artifact runs");
    assert_eq!(ran.status.code(), Some(44), "stderr: {:?}", ran.stderr);
    assert!(!dir.parent().unwrap().join("escape").exists());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn native_build_subcommand_reaches_the_same_public_producer() {
    let dir = output_dir("cli");
    let source_path = dir.join("main.ken");
    let artifact_dir = dir.join("artifact");
    std::fs::write(&source_path, PURE_PROGRAM).unwrap();
    let built = Command::new(env!("CARGO_BIN_EXE_ken"))
        .arg("native-build")
        .arg(&source_path)
        .arg(&artifact_dir)
        .output()
        .expect("native-build command runs");
    assert_eq!(built.status.code(), Some(0), "stderr: {:?}", built.stderr);
    let executable = String::from_utf8(built.stdout).unwrap();
    let executable = std::path::PathBuf::from(executable.trim());
    assert!(executable.is_file());
    let ran = Command::new(executable)
        .output()
        .expect("CLI artifact runs");
    assert_eq!(ran.status.code(), Some(0), "stderr: {:?}", ran.stderr);
    let _ = std::fs::remove_dir_all(dir);
}

/// Compile `snippet` as a standalone crate against a built `ken_runtime`
/// rlib, and report whether it compiled together with rustc's stderr.
///
/// `ken-cli` is a different crate from `ken-runtime`, so this is the real
/// cross-crate question rather than a proxy for it.
fn compile_probe_against_ken_runtime(
    rlib: &std::path::Path,
    deps: &std::path::Path,
    snippet: &str,
) -> (bool, String) {
    let dir = output_dir("vis-probe");
    let source = dir.join("probe.rs");
    std::fs::write(&source, snippet).expect("probe source is written");
    let compiled = Command::new(std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string()))
        .args(["--edition", "2021", "--crate-type", "lib"])
        .arg("--extern")
        .arg(format!("ken_runtime={}", rlib.display()))
        .arg("-L")
        .arg(format!("dependency={}", deps.display()))
        .arg("--out-dir")
        .arg(&dir)
        .arg(&source)
        .output()
        .expect("rustc runs");
    let stderr = String::from_utf8_lossy(&compiled.stderr).into_owned();
    let succeeded = compiled.status.success();
    let _ = std::fs::remove_dir_all(&dir);
    (succeeded, stderr)
}

/// The `ken-runtime` source tree, resolved at run time rather than baked in
/// by `include_str!`. A macro reads the file at *compile* time, so a change to
/// `ken-runtime` that does not force this crate to rebuild leaves the baked
/// copy stale; reading from disk when the assertion runs cannot go stale.
const KEN_RUNTIME_SRC: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../ken-runtime/src");

/// Every `.rs` file under `ken-runtime`'s source tree.
fn ken_runtime_source_files() -> Vec<std::path::PathBuf> {
    fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("ken-runtime source directory is readable") {
            let path = entry.expect("directory entry is readable").path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                out.push(path);
            }
        }
    }
    let mut files = Vec::new();
    walk(std::path::Path::new(KEN_RUNTIME_SRC), &mut files);
    files.sort();
    assert!(
        !files.is_empty(),
        "no .rs files under {KEN_RUNTIME_SRC}, so any source-derived assertion \
         below would pass vacuously"
    );
    files
}

fn modified_at(path: &std::path::Path) -> std::time::SystemTime {
    path.metadata()
        .and_then(|meta| meta.modified())
        .unwrap_or_else(|error| panic!("modification time of {}: {error}", path.display()))
}

/// Pick the built `ken_runtime` rlib that reflects CURRENT source, and prove
/// the probe harness resolves against it before any negative probe is read as
/// evidence.
///
/// Returns the rlib, its `deps` directory, and a human-readable account of
/// which candidate was chosen -- the account is returned rather than merely
/// printed so a failing assertion can name the artifact it measured.
fn select_current_ken_runtime_rlib(
    control: &str,
) -> (std::path::PathBuf, std::path::PathBuf, String) {
    // An integration test binary lives in `target/<profile>/deps`, alongside
    // the rlibs it was linked against.
    let deps = std::env::current_exe()
        .expect("test binary path")
        .parent()
        .expect("test binary has a parent directory")
        .to_path_buf();
    let mut candidates = std::fs::read_dir(&deps)
        .expect("deps directory is readable")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map_or(false, |name| {
                    name.starts_with("libken_runtime-") && name.ends_with(".rlib")
                })
        })
        .collect::<Vec<_>>();
    assert!(
        !candidates.is_empty(),
        "no libken_runtime-*.rlib under {}",
        deps.display()
    );
    // ⛔ NEWEST FIRST, and this is a correctness requirement rather than a
    // tidiness one. `deps` accumulates an rlib per build over the life of the
    // target directory -- there were 15 here when this was written, spanning
    // one day -- and every stale one still compiles the positive control.
    // Ordering by anything else (the filename hash, say) selects an arbitrary
    // rlib, and the probe then reports on source that may be hours old while
    // every signal looks healthy. Cargo has brought the dependency up to date
    // before this test runs, so the most recently written rlib is the current
    // one; stale ones are strictly older.
    //
    // The positive control cannot catch this on its own: it proves the harness
    // WORKS, never that the harness is looking at the CURRENT code. Freshness
    // is a separate axis, and the direction-1 mutation proof is what exercises
    // it -- a stale selection makes that proof fail to fail.
    candidates.sort_by_key(|path| {
        std::cmp::Reverse(
            path.metadata()
                .and_then(|meta| meta.modified())
                .expect("rlib modification time is readable"),
        )
    });

    // The positive control both validates the harness and SELECTS the rlib.
    // Several may be present -- `ken-cli` builds `ken-runtime` twice, once
    // with the `px8-ds-test-support` dev-dependency feature -- and a
    // "did not compile" result only means something against an rlib this
    // probe form can actually resolve against. Without this, a misconfigured
    // rustc invocation would make every negative probe fail for its own
    // reasons and the whole check would pass while testing nothing.
    let mut control_failures = Vec::new();
    let mut selected = None;
    for candidate in &candidates {
        let (compiled, stderr) = compile_probe_against_ken_runtime(candidate, &deps, control);
        if compiled {
            selected = Some(candidate.clone());
            break;
        }
        control_failures.push(format!("{}:\n{stderr}", candidate.display()));
    }
    let rlib = selected.unwrap_or_else(|| {
        panic!(
            "the positive control failed to compile against every candidate ken_runtime \
             rlib, so this harness is what broke, not the property under test:\n{}",
            control_failures.join("\n")
        )
    });

    // This loop carries TWO invariants -- use the freshest rlib, and use one
    // the control resolves against -- and where they conflict the second wins
    // silently. That fallback is not hypothetical: the control fails against a
    // candidate BY DESIGN (that is why the loop exists), and on the machine
    // where this was written 11 of 15 accumulated rlibs failed it, with the
    // next control-resolving candidate a full day old. So the selection is
    // reported rather than left implicit.
    let mut selection = format!("ken_runtime rlib: {}", rlib.display());
    if rlib != candidates[0] {
        selection.push_str(&format!(
            "  [NOT the newest candidate -- {} newer candidate(s) were skipped \
             because the positive control did not resolve against them, newest \
             being {}]",
            control_failures.len(),
            candidates[0].display()
        ));
    }
    eprintln!("{selection}");

    // ...and the fallback is made SAFE rather than merely visible, by a
    // post-condition on the artifact that was actually selected instead of a
    // guard keyed to how it got selected: whatever the loop chose must be at
    // least as new as every `ken-runtime` source file. A probe compiled
    // against an rlib older than the source it claims to report on is
    // measuring code that no longer exists, and every other signal here --
    // the positive control included -- stays green while it does. Cargo brings
    // the dependency up to date before this test runs, so the current rlib
    // always satisfies this; only a stale selection does not.
    let newest_source = ken_runtime_source_files()
        .iter()
        .map(|path| (modified_at(path), path.clone()))
        .max_by_key(|(time, _)| *time)
        .expect("ken-runtime has at least one source file");
    assert!(
        modified_at(&rlib) >= newest_source.0,
        "{selection}\nis OLDER than {}, so this probe would report on stale source \
         while the positive control still passes. Suspect the selection, not the \
         property under test.",
        newest_source.1.display()
    );

    (rlib, deps, selection)
}

/// Run each `probe` against `rlib` and require it to fail for a resolution or
/// visibility reason specifically.
fn assert_probes_do_not_resolve(
    rlib: &std::path::Path,
    deps: &std::path::Path,
    selection: &str,
    probes: &[&str],
) {
    for probe in probes {
        let (compiled, stderr) = compile_probe_against_ken_runtime(rlib, deps, probe);
        assert!(
            !compiled,
            "`{probe}` COMPILED against {selection}, so the test-only helper is \
             reachable as public production API"
        );
        // Failing is not enough: a `compile_fail`-shaped check passes when the
        // snippet fails for ANY reason, a typo included. Requiring a
        // resolution/visibility error code is what makes the negative result
        // evidence for the property. Error codes are stable rustc surface;
        // the diagnostic prose deliberately is not asserted on.
        assert!(
            ["E0432", "E0433", "E0603"]
                .iter()
                .any(|code| stderr.contains(code)),
            "`{probe}` did not compile, but not with a resolution or visibility error, \
             so its failure does not establish the property:\n{stderr}"
        );
    }
}

/// Assert that the test-only cranelift helper is not reachable as public API
/// from outside `ken-runtime`, by asking the compiler rather than by matching
/// the declaration's text.
///
/// Both externally reachable paths are probed. `ken-runtime`'s `lib.rs` has
/// `pub use cranelift_backend::*`, a public glob, so a widened item surfaces
/// at the crate root as well as under the module path; a check on one path
/// alone would miss the other, and neither is visible to a text match.
///
/// Each probe is a bare `use ... as _;`. That form resolves a path and checks
/// its visibility and nothing else -- no call, no argument types, no
/// inference. This is load-bearing rather than stylistic: the helper takes
/// `impl Into<String>`, so a probe that *used* the item would fail to compile
/// on type inference whether or not the path resolved, and would keep
/// reporting success even after the helper was made public.
fn assert_helper_is_not_reachable_from_outside_ken_runtime() {
    const CONTROL: &str =
        "use ken_runtime::cranelift_backend::emit_runtime_ir_object_with_cranelift as _;";
    const PROBES: [&str; 2] = [
        "use ken_runtime::cranelift_backend::emit_process_entrypoint_object_with_cranelift as _;",
        "use ken_runtime::emit_process_entrypoint_object_with_cranelift as _;",
    ];

    let (rlib, deps, selection) = select_current_ken_runtime_rlib(CONTROL);
    assert_probes_do_not_resolve(&rlib, &deps, &selection, &PROBES);
}

/// The same compiler-backed question for the packaging half:
/// `build_process_starter_executable_artifact` must not be reachable from
/// outside `ken-runtime`.
///
/// `lib.rs` declares `pub mod object_linker_packaging` AND
/// `pub use object_linker_packaging::*`, so both the module path and the crate
/// root are reachable, exactly as for the cranelift half.
fn assert_packaging_helper_is_not_reachable_from_outside_ken_runtime() {
    const CONTROL: &str =
        "use ken_runtime::object_linker_packaging::package_starter_executable_artifact as _;";
    const PROBES: [&str; 2] = [
        "use ken_runtime::object_linker_packaging::build_process_starter_executable_artifact as _;",
        "use ken_runtime::build_process_starter_executable_artifact as _;",
    ];

    let (rlib, deps, selection) = select_current_ken_runtime_rlib(CONTROL);
    assert_probes_do_not_resolve(&rlib, &deps, &selection, &PROBES);
}

/// ⛔ DELIBERATE SOURCE-TEXT RESIDUE -- read before "finishing the conversion".
///
/// The probe above cannot express this property, and no amount of care makes
/// it able to. `build_process_starter_executable_artifact` is bare-private,
/// not `pub(crate)`, and from `ken-cli` those two are THE SAME OBSERVATION:
/// both are unreachable, and both fail with `E0432`/`E0433`/`E0603`. Asserting
/// on error codes rather than diagnostic prose is correct, and it is precisely
/// what removes the power to tell them apart. So converting this conjunct to a
/// compile probe would not merely weaken it -- it would DROP a property the
/// cross-crate mechanism cannot state, silently, leaving a widening to
/// `pub(crate)` caught by nothing.
///
/// Expressing it with the compiler needs a probe from INSIDE `ken-runtime`,
/// where `pub(crate)` resolves and bare-private does not. That is a different
/// harness in a different crate; until one exists, this conjunct stays text,
/// and the reason is written here so the next reader knows it is residue by
/// decision rather than by oversight.
///
/// Two things make this a materially better pin than the one it replaces.
/// It scans the WHOLE crate rather than one hard-coded file, so relocating the
/// helper cannot make it pass vacuously; and it inspects only the `fn` line
/// itself, so attributes above the declaration may be reordered freely. It
/// also asserts the declaration is FOUND -- a negative text check passes
/// happily when its subject has been renamed out from under it.
fn assert_packaging_helper_is_declared_module_private() {
    const DECLARATION: &str = "fn build_process_starter_executable_artifact(";

    let mut declarations = Vec::new();
    for file in ken_runtime_source_files() {
        let text = std::fs::read_to_string(&file)
            .unwrap_or_else(|error| panic!("reading {}: {error}", file.display()));
        for (index, line) in text.lines().enumerate() {
            // `DECLARATION` starts with `fn `, so call sites do not match.
            let Some(at) = line.find(DECLARATION) else {
                continue;
            };
            declarations.push((file.clone(), index + 1, line[..at].trim().to_string()));
        }
    }

    assert!(
        !declarations.is_empty(),
        "no declaration of `{DECLARATION}` found anywhere under {KEN_RUNTIME_SRC}. \
         The helper was renamed or removed, and this assertion has been passing \
         vacuously rather than checking anything."
    );
    for (file, line, visibility) in &declarations {
        assert!(
            !visibility.contains("pub"),
            "{}:{line} declares the test-only packaging helper as `{visibility} \
             {DECLARATION}`. It must stay private to its own module: `pub` makes it \
             public production API, and `pub(crate)` widens it to the whole crate. \
             Neither is observable from outside `ken-runtime`, which is why this \
             check reads the declaration instead of asking the compiler.",
            file.display()
        );
    }
}

#[test]
fn naked_process_ir_helpers_are_not_public_production_api() {
    // `emit_process_entrypoint_object_with_cranelift` is checked by compiling
    // probes against the built `ken_runtime` rlib, below. The assertions that
    // used to live here matched a literal declaration string, so they tracked
    // attribute placement, visibility spelling, whitespace, and file location
    // — none of which are the property — and they broke on every relocation of
    // the helper, twice during `CB-HYGIENE` alone.
    assert_helper_is_not_reachable_from_outside_ken_runtime();

    // The packaging half, in two parts because the mechanism splits there.
    // Reachability from outside the crate is the compiler's question and is
    // asked as one; whether the declaration is bare-private or `pub(crate)` is
    // invisible from this crate at any level of care, so it stays a text check
    // -- see that function's comment for why, and why it is not a conversion
    // someone forgot to finish.
    assert_packaging_helper_is_not_reachable_from_outside_ken_runtime();
    assert_packaging_helper_is_declared_module_private();

    // The remaining assertions are about generated C source text, which is
    // genuinely what they are checking; they are not visibility oracles.
    let packaging = include_str!("../../ken-runtime/src/object_linker_packaging.rs");
    assert!(!packaging.contains(".capability = ((uint64_t)1 << 32)"));
    assert!(packaging.contains(".capability = host_init.capability"));
    assert!(packaging.contains("host_init.capability == 0"));
    assert!(packaging.contains("process_environment_count"));
}
