//! Checked-source, twin-real-root canonical differential runner.

use std::ffi::OsString;
use std::fmt;

#[cfg(unix)]
use std::os::unix::ffi::{OsStrExt, OsStringExt};

use ken_elaborator::capabilities::{Authority, RightSet, SymlinkPolicy, AUTH_FULL};
use ken_host::EffectObservationV1;
use ken_runtime::{
    BoundProcessExecutableArtifact, NativeEffectRunErrorV1, NativeEffectRunOptionsV1,
};

use crate::{
    canonical_filesystem_delta, compare_canonical_exact, AmbientScript, ExpectedFsEffect,
    LaneActionEvidence, ObservationMismatch, ScriptedPosixHost, SeedNode, TwinRealRoots,
    TwinRootError,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RawProcessInput {
    /// Arguments after argv[0]. The runner supplies the exact produced artifact
    /// path as argv[0] to both lanes.
    pub arguments: Vec<Vec<u8>>,
    pub environment: Vec<(Vec<u8>, Vec<u8>)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgramCapsShape {
    pub fs_authority: Authority,
    pub relative_root: Vec<u8>,
    pub rights: RightSet,
    pub symlink: SymlinkPolicy,
}

impl Default for ProgramCapsShape {
    fn default() -> Self {
        Self {
            fs_authority: AUTH_FULL,
            relative_root: Vec::new(),
            rights: RightSet::ALL,
            symlink: SymlinkPolicy::NoFollow,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedProgramEntry {
    pub identity: String,
    pub package_name: String,
    pub source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scenario {
    pub process_input: RawProcessInput,
    pub ambient: AmbientScript,
    pub program_caps: ProgramCapsShape,
    pub entry: CheckedProgramEntry,
    pub initial_filesystem: Vec<SeedNode>,
    /// Independent execution assertions for the interpreter descriptor calls.
    /// These values never author or alter an `EffectObservationV1` field.
    pub expected_fs: Vec<ExpectedFsEffect>,
}

#[derive(Debug)]
pub enum HarnessError {
    TwinRoots(TwinRootError),
    UnsupportedAmbient(&'static str),
    InvalidRawProcessInput(&'static str),
    Interpreter(String),
    NativeBuild(String),
    NativeRun(NativeEffectRunErrorV1),
    Observation(ObservationMismatch),
}

impl fmt::Display for HarnessError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TwinRoots(error) => error.fmt(formatter),
            Self::UnsupportedAmbient(field) => {
                write!(formatter, "native production lane cannot script {field}")
            }
            Self::InvalidRawProcessInput(field) => {
                write!(
                    formatter,
                    "raw ProcessInput field is not host-executable: {field}"
                )
            }
            Self::Interpreter(error) => write!(formatter, "interpreter lane failed: {error}"),
            Self::NativeBuild(error) => write!(formatter, "native artifact build failed: {error}"),
            Self::NativeRun(error) => error.fmt(formatter),
            Self::Observation(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for HarnessError {}

impl From<TwinRootError> for HarnessError {
    fn from(error: TwinRootError) -> Self {
        Self::TwinRoots(error)
    }
}

impl From<ObservationMismatch> for HarnessError {
    fn from(error: ObservationMismatch) -> Self {
        Self::Observation(error)
    }
}

/// A passing result owns the real roots and exact linked artifact so mutation
/// gates can alter real launcher inputs after the baseline run.
pub struct CanonicalDifferentialRun {
    pub scenario_identity: String,
    pub interpreter: EffectObservationV1,
    pub native: EffectObservationV1,
    pub interpreter_actions: LaneActionEvidence,
    pub native_actions: LaneActionEvidence,
    pub exact_artifact_executed: bool,
    pub process_input_arguments: Vec<Vec<u8>>,
    pub process_input_environment: Vec<(Vec<u8>, Vec<u8>)>,
    pub process_input_cwd: Vec<u8>,
    artifact: BoundProcessExecutableArtifact,
    plan_hash: u64,
    roots: TwinRealRoots,
}

impl CanonicalDifferentialRun {
    pub fn compare_exact(&self) -> Result<(), ObservationMismatch> {
        compare_canonical_exact(&self.interpreter, &self.native)
    }

    /// Mutate the real launch binding; the production decoder must fail closed.
    pub fn rejects_wrong_plan_binding(&self) -> bool {
        let options = NativeEffectRunOptionsV1 {
            arguments: self.process_input_arguments[1..]
                .iter()
                .map(|argument| raw_os_string(argument))
                .collect::<Result<Vec<_>, _>>()
                .expect("a completed run already validated arguments"),
            environment: self
                .process_input_environment
                .iter()
                .map(|(key, value)| Ok((raw_os_string(key)?, raw_os_string(value)?)))
                .collect::<Result<Vec<_>, HarnessError>>()
                .expect("a completed run already validated environment"),
            cwd: self.roots.native().to_path_buf(),
            plan_hash: self.plan_hash ^ 1,
        };
        matches!(
            ken_runtime::run_bound_process_effect_observation_v1(&self.artifact, &options),
            Err(NativeEffectRunErrorV1::BindingMismatch)
        )
    }
}

/// Execute the same checked program through the interpreter and the real
/// linked artifact. Canonical equality is part of the return gate.
pub fn run_scenario(scenario: &Scenario) -> Result<CanonicalDifferentialRun, HarnessError> {
    validate_native_ambient(&scenario.ambient)?;
    let roots = TwinRealRoots::create(&scenario.initial_filesystem)?;
    let build = ken_cli::build_native_program(
        &scenario.entry.source,
        ken_cli::SourceFormat::Ken,
        &scenario.entry.package_name,
        roots.artifacts(),
    )
    .map_err(|error| HarnessError::NativeBuild(error.to_string()))?;

    let argv0 = raw_path_bytes(&build.artifact.executable_path)?;
    let mut arguments = vec![argv0];
    arguments.extend(scenario.process_input.arguments.clone());
    let cwd = raw_path_bytes(roots.native())?;

    let interpreter_before = roots.snapshot_interpreter()?;
    let native_before = roots.snapshot_native()?;
    let mut host = ScriptedPosixHost::new_scoped(
        roots.interpreter(),
        scenario.ambient.clone(),
        scenario.program_caps.fs_authority,
        &scenario.program_caps.relative_root,
        scenario.program_caps.rights,
        scenario.program_caps.symlink,
        scenario.expected_fs.clone(),
    )
    .map_err(|error| HarnessError::Interpreter(error.to_string()))?;
    let mut interpreter = ken_cli::run_program_effect_observation_v1(
        &scenario.entry.source,
        ken_cli::SourceFormat::Ken,
        &arguments,
        &scenario.process_input.environment,
        &cwd,
        &mut host,
    )
    .map_err(|error| HarnessError::Interpreter(format!("{error:?}")))?;
    let interpreter_after = roots.snapshot_interpreter()?;
    host.finish_assertions()
        .map_err(HarnessError::Interpreter)?;
    if !interpreter.filesystem_delta.is_empty() {
        return Err(HarnessError::Interpreter(
            "PX5B interpreter producer unexpectedly authored filesystem_delta".to_string(),
        ));
    }
    interpreter.filesystem_delta =
        canonical_filesystem_delta(&interpreter_before, &interpreter_after);
    let interpreter_actions = LaneActionEvidence {
        root_before: interpreter_before,
        root_after: interpreter_after,
        fs_actions_after_resolve: Some(host.fs_actions_after_resolve()),
    };

    let options = NativeEffectRunOptionsV1 {
        arguments: scenario
            .process_input
            .arguments
            .iter()
            .map(|argument| raw_os_string(argument))
            .collect::<Result<Vec<_>, _>>()?,
        environment: scenario
            .process_input
            .environment
            .iter()
            .map(|(key, value)| Ok((raw_os_string(key)?, raw_os_string(value)?)))
            .collect::<Result<Vec<_>, HarnessError>>()?,
        cwd: roots.native().to_path_buf(),
        plan_hash: build.plan_transport_hash,
    };
    let native = ken_runtime::run_bound_process_effect_observation_v1(&build.artifact, &options)
        .map_err(HarnessError::NativeRun)?;
    let native_after = roots.snapshot_native()?;
    let native_actions = LaneActionEvidence {
        root_before: native_before,
        root_after: native_after,
        fs_actions_after_resolve: None,
    };

    compare_canonical_exact(&interpreter, &native)?;
    Ok(CanonicalDifferentialRun {
        scenario_identity: scenario.entry.identity.clone(),
        interpreter,
        native,
        interpreter_actions,
        native_actions,
        exact_artifact_executed: true,
        process_input_arguments: arguments,
        process_input_environment: scenario.process_input.environment.clone(),
        process_input_cwd: cwd,
        artifact: build.artifact,
        plan_hash: build.plan_transport_hash,
        roots,
    })
}

fn validate_native_ambient(ambient: &AmbientScript) -> Result<(), HarnessError> {
    if !ambient.stdin.is_empty() {
        return Err(HarnessError::UnsupportedAmbient("stdin"));
    }
    if ambient.stdin_is_terminal || ambient.stdout_is_terminal || ambient.stderr_is_terminal {
        return Err(HarnessError::UnsupportedAmbient(
            "terminal state other than piped/false",
        ));
    }
    if !ambient.wall_clock_nanoseconds.is_empty() {
        return Err(HarnessError::UnsupportedAmbient("clock script"));
    }
    Ok(())
}

#[cfg(unix)]
fn raw_os_string(bytes: &[u8]) -> Result<OsString, HarnessError> {
    if bytes.contains(&0) {
        return Err(HarnessError::InvalidRawProcessInput("embedded NUL"));
    }
    Ok(OsString::from_vec(bytes.to_vec()))
}

#[cfg(not(unix))]
fn raw_os_string(_bytes: &[u8]) -> Result<OsString, HarnessError> {
    Err(HarnessError::InvalidRawProcessInput(
        "raw bytes require Unix",
    ))
}

#[cfg(unix)]
fn raw_path_bytes(path: &std::path::Path) -> Result<Vec<u8>, HarnessError> {
    Ok(path.as_os_str().as_bytes().to_vec())
}

#[cfg(not(unix))]
fn raw_path_bytes(_path: &std::path::Path) -> Result<Vec<u8>, HarnessError> {
    Err(HarnessError::InvalidRawProcessInput(
        "raw paths require Unix",
    ))
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    use crate::{
        apply_canonical_mutation, confirm_native_tested_transition, denial_precedes_host_action,
        CanonicalMutation, NativeTestedEvidence, RunnerOnlyProxy, StatusTransitionError,
    };
    use ken_host::{
        dispatch_host_op_v1, program_caps_fs_trace_identity_v1, CanonicalOutcomeV1,
        CanonicalReplyV1, CanonicalRequestV1, CapabilityDeniedV1, CapabilityGrantV1,
        CapabilityTableV1, ConsoleStreamV1, CreatePolicyV1, FileErrorCauseV1, HostEffectBackendV1,
        HostOpAvailabilityV1, HostOpV1, IoErrorIdentityV1, SemanticErrorV1,
        PX5_PLANNED_NATIVE_TARGETS,
    };

    const FIVE_OP_SOURCE: &str = r#"program capabilities FS AFull
proc main (input : ProcessInput) (caps : ProgramCaps AFull)
  : HostIO AFull ExitCode visits [FS, Console] =
  match input {
    MkProcessInput arguments _environment _cwd |-> match arguments {
      Nil |-> host_exit AFull (Failure 20) ;
      Cons _argv0 rest |-> match rest {
        Nil |-> host_exit AFull (Failure 21) ;
        Cons path more |-> match more {
          Nil |-> host_exit AFull (Failure 22) ;
          Cons contents _ |-> match caps {
            MkProgramCaps cap |->
              bind (Coproduct (FSOp AFull) AmbientOp)
                (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
                (Result FileError Unit) ExitCode
                (inject_l (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp
                  (Result FileError Unit)
                  (writeFile cap path CreateNew contents))
                (\written. match written {
                  Err _ |-> host_exit AFull (Failure 23) ;
                  Ok _ |->
                    bind (Coproduct (FSOp AFull) AmbientOp)
                      (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
                      (Result FileError Bytes) ExitCode
                      (inject_l (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp
                        (Result FileError Bytes) (readFile AFull cap path))
                      (\read. match read {
                        Err _ |-> host_exit AFull (Failure 24) ;
                        Ok bytes |->
                          bind (Coproduct (FSOp AFull) AmbientOp)
                            (resp_coproduct (FSOp AFull) AmbientOp
                              (fs_resp AFull) ambient_resp)
                            (Result IOError Unit) ExitCode
                            (host_console AFull (Result IOError Unit)
                              (write Stdout bytes))
                            (\written_console. match written_console {
                              Err _ |-> host_exit AFull (Failure 25) ;
                              Ok _ |->
                                bind (Coproduct (FSOp AFull) AmbientOp)
                                  (resp_coproduct (FSOp AFull) AmbientOp
                                    (fs_resp AFull) ambient_resp)
                                  (Result IOError Unit) ExitCode
                                  (host_console AFull (Result IOError Unit) (flush Stdout))
                                  (\flushed. match flushed {
                                    Err _ |-> host_exit AFull (Failure 26) ;
                                    Ok _ |->
                                      bind (Coproduct (FSOp AFull) AmbientOp)
                                        (resp_coproduct (FSOp AFull) AmbientOp
                                          (fs_resp AFull) ambient_resp)
                                        Bool ExitCode
                                        (host_console AFull Bool (is_terminal Stdout))
                                        (\terminal. match terminal {
                                          False |-> host_exit AFull Success ;
                                          True |-> host_exit AFull (Failure 27)
                                        })
                                  })
                            })
                      })
                })
          }
        }
      }
    }
  }
"#;

    const DENIAL_SOURCE: &str = r#"program capabilities FS AFull
proc main (input : ProcessInput) (caps : ProgramCaps AFull)
  : HostIO AFull ExitCode visits [FS] =
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

    const RAW_COLLISION_SOURCE: &str = r#"program capabilities FS AFull
proc main (input : ProcessInput) (caps : ProgramCaps AFull)
  : HostIO AFull ExitCode visits [FS] =
  match input {
    MkProcessInput arguments _environment _cwd |-> match arguments {
      Nil |-> host_exit AFull (Failure 60) ;
      Cons _argv0 rest |-> match rest {
        Nil |-> host_exit AFull (Failure 61) ;
        Cons raw_path more |-> match more {
          Nil |-> host_exit AFull (Failure 62) ;
          Cons normalized_path _ |-> match caps {
            MkProgramCaps cap |->
              bind (Coproduct (FSOp AFull) AmbientOp)
                (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
                (Result FileError Bytes) ExitCode
                (inject_l (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp
                  (Result FileError Bytes) (readFile AFull cap raw_path))
                (\first. match first {
                  Err _ |-> host_exit AFull (Failure 63) ;
                  Ok _ |->
                    bind (Coproduct (FSOp AFull) AmbientOp)
                      (resp_coproduct (FSOp AFull) AmbientOp
                        (fs_resp AFull) ambient_resp)
                      (Result FileError Bytes) ExitCode
                      (inject_l (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp
                        (Result FileError Bytes) (readFile AFull cap normalized_path))
                      (\second. match second {
                        Err _ |-> host_exit AFull (Failure 64) ;
                        Ok _ |-> host_exit AFull Success
                      })
                })
          }
        }
      }
    }
  }
"#;

    fn five_op_scenario() -> Scenario {
        let path = b"dir/./px6.bin".to_vec();
        let bytes = vec![b'r', 0xff, b'x'];
        Scenario {
            process_input: RawProcessInput {
                arguments: vec![path.clone(), bytes.clone()],
                environment: vec![(b"PX6_ENV".to_vec(), vec![0xfe, b'v'])],
            },
            ambient: AmbientScript::default(),
            program_caps: ProgramCapsShape::default(),
            entry: CheckedProgramEntry {
                identity: "px6-five-op-real-artifact".to_string(),
                package_name: "px6-five-op-real-artifact".to_string(),
                source: FIVE_OP_SOURCE.to_string(),
            },
            initial_filesystem: vec![SeedNode {
                relative_path: b"dir".to_vec(),
                kind: crate::SeedNodeKind::Directory,
            }],
            expected_fs: vec![
                ExpectedFsEffect::WriteFile {
                    path: path.clone(),
                    create_policy: CreatePolicyV1::CreateNew,
                    bytes,
                },
                ExpectedFsEffect::ReadFile { path },
            ],
        }
    }

    fn denial_scenario() -> Scenario {
        let path = b"../escape".to_vec();
        Scenario {
            process_input: RawProcessInput {
                arguments: vec![path.clone()],
                environment: Vec::new(),
            },
            ambient: AmbientScript::default(),
            program_caps: ProgramCapsShape::default(),
            entry: CheckedProgramEntry {
                identity: "px6-denial-real-artifact".to_string(),
                package_name: "px6-denial-real-artifact".to_string(),
                source: DENIAL_SOURCE.to_string(),
            },
            initial_filesystem: Vec::new(),
            expected_fs: vec![ExpectedFsEffect::WriteFile {
                path: path.clone(),
                create_policy: CreatePolicyV1::CreateNew,
                bytes: path,
            }],
        }
    }

    fn raw_descriptor_collision_scenario() -> Scenario {
        let raw_path = b"dir/./x".to_vec();
        let normalized_path = b"dir/x".to_vec();
        Scenario {
            process_input: RawProcessInput {
                arguments: vec![raw_path.clone(), normalized_path.clone()],
                environment: Vec::new(),
            },
            ambient: AmbientScript::default(),
            program_caps: ProgramCapsShape::default(),
            entry: CheckedProgramEntry {
                identity: "px6-raw-descriptor-collision".to_string(),
                package_name: "px6-raw-descriptor-collision".to_string(),
                source: RAW_COLLISION_SOURCE.to_string(),
            },
            initial_filesystem: vec![SeedNode {
                relative_path: normalized_path.clone(),
                kind: crate::SeedNodeKind::File(b"same-node".to_vec()),
            }],
            expected_fs: vec![
                ExpectedFsEffect::ReadFile { path: raw_path },
                ExpectedFsEffect::ReadFile {
                    path: normalized_path,
                },
            ],
        }
    }

    #[derive(Default)]
    struct NoLeafBackend {
        calls: u64,
    }

    impl HostEffectBackendV1 for NoLeafBackend {
        fn console_write(
            &mut self,
            _stream: ConsoleStreamV1,
            _bytes: &[u8],
        ) -> Result<(), IoErrorIdentityV1> {
            self.calls += 1;
            Ok(())
        }

        fn console_flush(&mut self, _stream: ConsoleStreamV1) -> Result<(), IoErrorIdentityV1> {
            self.calls += 1;
            Ok(())
        }

        fn console_is_terminal(&mut self, _stream: ConsoleStreamV1) -> bool {
            self.calls += 1;
            false
        }

        fn fs_read_file(
            &mut self,
            _grant: &CapabilityGrantV1,
            _path: &[u8],
        ) -> Result<Vec<u8>, FileErrorCauseV1> {
            self.calls += 1;
            Ok(Vec::new())
        }

        fn fs_write_file(
            &mut self,
            _grant: &CapabilityGrantV1,
            _path: &[u8],
            _create_policy: CreatePolicyV1,
            _bytes: &[u8],
        ) -> Result<(), FileErrorCauseV1> {
            self.calls += 1;
            Ok(())
        }
    }

    #[test]
    fn real_artifact_five_op_observation_matches_interp_on_twin_roots() {
        let run = run_scenario(&five_op_scenario()).expect("real five-op differential");
        run.compare_exact().expect("six-field equality");
        assert!(run.exact_artifact_executed);
        assert_eq!(run.interpreter.stdout, vec![b'r', 0xff, b'x']);
        assert_eq!(run.interpreter.filesystem_delta.len(), 1);
        assert_eq!(
            run.interpreter_actions.root_after,
            run.native_actions.root_after
        );
        assert_eq!(
            run.interpreter
                .effect_trace
                .iter()
                .map(|event| event.operation)
                .collect::<Vec<_>>(),
            vec![
                HostOpV1::FsWriteFile,
                HostOpV1::FsReadFile,
                HostOpV1::ConsoleWrite,
                HostOpV1::ConsoleFlush,
                HostOpV1::ConsoleIsTerminal,
            ]
        );

        for operation in PX5_PLANNED_NATIVE_TARGETS {
            let evidence = NativeTestedEvidence::from_run(operation, &run);
            assert!(evidence.permits_confirmation());
            assert_eq!(
                confirm_native_tested_transition(operation, evidence),
                Ok(HostOpAvailabilityV1::NativeTested)
            );
        }
        assert!(run.rejects_wrong_plan_binding());
    }

    #[test]
    fn real_producers_preserve_raw_requests_across_descriptor_collision() {
        let run = run_scenario(&raw_descriptor_collision_scenario())
            .expect("real raw-descriptor collision differential");
        run.compare_exact().expect("six-field equality");
        assert!(run.exact_artifact_executed);
        assert_eq!(
            run.interpreter_actions.root_before,
            run.interpreter_actions.root_after
        );
        assert_eq!(
            run.native_actions.root_before,
            run.native_actions.root_after
        );

        let expected_identity = Some(program_caps_fs_trace_identity_v1());
        for observation in [&run.interpreter, &run.native] {
            let [raw, normalized] = observation.effect_trace.as_slice() else {
                panic!("collision program must emit exactly two FS events")
            };
            assert_eq!(raw.sequence, 0);
            assert_eq!(normalized.sequence, 1);
            assert_eq!(raw.capability, expected_identity);
            assert_eq!(normalized.capability, expected_identity);
            assert_eq!(
                raw.request,
                CanonicalRequestV1::FsReadFile {
                    path: b"dir/./x".to_vec(),
                }
            );
            assert_eq!(
                normalized.request,
                CanonicalRequestV1::FsReadFile {
                    path: b"dir/x".to_vec(),
                }
            );
            assert_eq!(
                raw.outcome,
                CanonicalOutcomeV1::Success(CanonicalReplyV1::Bytes(b"same-node".to_vec()))
            );
            assert_eq!(raw.outcome, normalized.outcome);
        }
    }

    #[test]
    fn wrong_token_malformed_identity_and_error_are_reply_owned() {
        let mut source_table = CapabilityTableV1::default();
        let wrong_token = source_table.insert(CapabilityGrantV1 {
            identity: program_caps_fs_trace_identity_v1(),
            capability: ken_elaborator::capabilities::Cap::mint(AUTH_FULL, "FS"),
        });
        let target_table = CapabilityTableV1::default();
        let request = CanonicalRequestV1::FsReadFile {
            path: b"raw/./identity".to_vec(),
        };
        let mut backend = NoLeafBackend::default();

        let reply = dispatch_host_op_v1(
            &mut backend,
            &target_table,
            HostOpV1::FsReadFile,
            Some(wrong_token),
            &request,
        )
        .expect("malformed token is a typed canonical reply");

        assert_eq!(backend.calls, 0, "denial must precede every host leaf");
        assert_eq!(reply.capability_identity, None);
        assert!(matches!(
            reply.outcome,
            CanonicalOutcomeV1::Error(SemanticErrorV1::File(ken_host::FileErrorIdentityV1 {
                operation: HostOpV1::FsReadFile,
                relative_path,
                cause: FileErrorCauseV1::Capability(CapabilityDeniedV1::MalformedCapability),
            })) if relative_path == b"raw/./identity"
        ));
    }

    #[test]
    fn real_captured_evidence_mutations_bite_while_return_proxy_stays_green() {
        let run = run_scenario(&five_op_scenario()).expect("real five-op differential");
        let proxy = RunnerOnlyProxy {
            scenario_identity: run.scenario_identity.clone(),
            returned_value: run.interpreter.exit_status,
        };
        for mutation in [
            CanonicalMutation::SilentSkip,
            CanonicalMutation::DuplicatedResume,
            CanonicalMutation::ReorderedEvents,
            CanonicalMutation::StdoutStderrSwap,
            CanonicalMutation::PathByteNormalization,
            CanonicalMutation::WrongCapabilityToken,
            CanonicalMutation::DeniedBeforeHostAction,
            CanonicalMutation::FilesystemMutationWithoutTrace,
            CanonicalMutation::TraceWithoutFilesystemMutation,
            CanonicalMutation::TargetEffectManifestMismatch,
            CanonicalMutation::OperationStatusTransition,
        ] {
            let mut changed = run.native.clone();
            apply_canonical_mutation(&mut changed, mutation)
                .unwrap_or_else(|error| panic!("real mutation {mutation:?}: {error:?}"));
            assert!(
                compare_canonical_exact(&run.interpreter, &changed).is_err(),
                "real captured mutation {mutation:?} must be rejected"
            );
            let unchanged_proxy = RunnerOnlyProxy {
                scenario_identity: run.scenario_identity.clone(),
                returned_value: run.interpreter.exit_status,
            };
            assert!(proxy.agrees(&unchanged_proxy));
        }
        let mut wrong_exit = run.native.clone();
        wrong_exit.exit_status ^= 1;
        assert!(compare_canonical_exact(&run.interpreter, &wrong_exit).is_err());
        let mut wrong_target = run.native.clone();
        wrong_target.terminal_error = Some(ken_host::TerminalErrorV1::TargetAbiMismatch);
        wrong_target.effect_trace.clear();
        wrong_target.filesystem_delta.clear();
        assert!(compare_canonical_exact(&run.interpreter, &wrong_target).is_err());
        assert!(proxy.agrees(&RunnerOnlyProxy {
            scenario_identity: run.scenario_identity.clone(),
            returned_value: run.interpreter.exit_status,
        }));
    }

    #[test]
    fn real_scope_denial_is_typed_and_precedes_any_host_action() {
        let run = run_scenario(&denial_scenario()).expect("real denial differential");
        run.compare_exact().expect("typed denial equality");
        assert_eq!(run.interpreter.exit_status, 44);
        assert!(run.interpreter.filesystem_delta.is_empty());
        assert!(denial_precedes_host_action(
            &run.interpreter_actions,
            &run.interpreter
        ));
        assert_eq!(
            run.interpreter_actions.root_before,
            run.interpreter_actions.root_after
        );
        assert_eq!(
            run.native_actions.root_before,
            run.native_actions.root_after
        );
        assert!(matches!(
            &run.interpreter.effect_trace[0].outcome,
            ken_host::CanonicalOutcomeV1::Error(SemanticErrorV1::File(error))
                if error.cause == FileErrorCauseV1::Capability(CapabilityDeniedV1::ScopeEscape)
        ));

        let proxy = RunnerOnlyProxy {
            scenario_identity: run.scenario_identity.clone(),
            returned_value: run.interpreter.exit_status,
        };
        let mut weakened = run.native.clone();
        apply_canonical_mutation(&mut weakened, CanonicalMutation::WeakenedErrorIdentity)
            .expect("real denial has an error identity");
        assert!(compare_canonical_exact(&run.interpreter, &weakened).is_err());
        assert!(proxy.agrees(&RunnerOnlyProxy {
            scenario_identity: run.scenario_identity,
            returned_value: run.interpreter.exit_status,
        }));
    }

    #[test]
    fn capture_host_is_explicitly_insufficient_negative_control() {
        let source = r#"program capabilities FS APartial
proc main (_input : ProcessInput) (_caps : ProgramCaps APartial)
  : HostIO APartial ExitCode visits [Console] =
  host_program APartial (print_line "capture-only")
"#;
        let mut host = ken_interp::CaptureHost::new(Vec::new());
        let outcome = ken_cli::run_program(
            source,
            ken_cli::SourceFormat::Ken,
            &[b"capture-control".to_vec()],
            &[],
            b"/",
            &mut host,
        )
        .expect("CaptureHost unit control runs");
        assert_eq!(host.stdout(), b"capture-only\n");

        let observation = EffectObservationV1 {
            stdout: host.stdout().to_vec(),
            stderr: host.stderr().to_vec(),
            filesystem_delta: Vec::new(),
            terminal_error: None,
            effect_trace: vec![ken_host::EffectEventV1 {
                sequence: 0,
                operation: HostOpV1::ConsoleWrite,
                capability: None,
                request: ken_host::CanonicalRequestV1::ConsoleWrite {
                    stream: ken_host::ConsoleStreamV1::Stdout,
                    bytes: b"capture-only\n".to_vec(),
                },
                outcome: ken_host::CanonicalOutcomeV1::Success(ken_host::CanonicalReplyV1::Unit),
            }],
            exit_status: outcome.exit_status,
        };
        let evidence = NativeTestedEvidence::unit_or_negative_control(
            HostOpV1::ConsoleWrite,
            &observation,
            &observation,
        );
        assert!(!evidence.permits_confirmation());
        assert_eq!(
            confirm_native_tested_transition(HostOpV1::ConsoleWrite, evidence),
            Err(StatusTransitionError::MissingExactArtifactEvidence(
                HostOpV1::ConsoleWrite
            ))
        );
    }
}
