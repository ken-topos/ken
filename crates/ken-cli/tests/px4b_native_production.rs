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
    let native = ken_runtime::run_bound_process_effect_observation_v1(
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
        .map(|(sequence, bytes)| ken_runtime::EffectEventV1 {
            sequence: sequence as u64,
            operation: ken_runtime::HostOpV1::ConsoleWrite,
            capability: None,
            request: ken_runtime::CanonicalRequestV1::ConsoleWrite {
                stream: ken_runtime::ConsoleStreamV1::Stdout,
                bytes: bytes.to_vec(),
            },
            outcome: ken_runtime::CanonicalOutcomeV1::Success(ken_runtime::CanonicalReplyV1::Unit),
        })
        .collect();
    let interpreted = ken_runtime::EffectObservationV1 {
        stdout: host.stdout().to_vec(),
        stderr: host.stderr().to_vec(),
        filesystem_delta: Vec::new(),
        terminal_error: None,
        effect_trace: expected_trace,
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
    let observation = ken_runtime::decode_linked_effect_trace_v1(&observation).unwrap();
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
    let observation = ken_runtime::run_bound_process_effect_observation_v1(
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
        observation.filesystem_delta,
        vec![ken_runtime::FsDeltaV1::Created {
            relative_path: b"px5.bin".to_vec(),
            node: ken_runtime::FsNodeObservationV1 {
                kind: ken_runtime::FsNodeKindV1::File,
                file_bytes: Some(b"retained".to_vec()),
                symlink_target: None,
            },
        }]
    );
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

#[test]
fn naked_process_ir_helpers_are_not_public_production_api() {
    let cranelift = include_str!("../../ken-runtime/src/cranelift_backend.rs");
    let packaging = include_str!("../../ken-runtime/src/object_linker_packaging.rs");
    assert!(cranelift
        .contains("#[cfg(test)]\npub(crate) fn emit_process_entrypoint_object_with_cranelift("));
    assert!(!cranelift.contains("\npub fn emit_process_entrypoint_object_with_cranelift("));
    assert!(packaging.contains("#[cfg(test)]\nfn build_process_starter_executable_artifact("));
    assert!(!packaging.contains("\npub fn build_process_starter_executable_artifact("));
    assert!(!packaging.contains("\npub(crate) fn build_process_starter_executable_artifact("));
    assert!(!packaging.contains(".capability = ((uint64_t)1 << 32)"));
    assert!(packaging.contains(".capability = host_init.capability"));
    assert!(packaging.contains("host_init.capability == 0"));
    assert!(packaging.contains("process_environment_count"));
}
