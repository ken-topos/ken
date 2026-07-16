//! PX7-F public checked-Ken linked-native discriminators.

fn output_dir(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "ken-px7f-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn run(name: &str, source: &str) -> ken_runtime::EffectObservationV1 {
    let dir = output_dir(name);
    std::fs::write(dir.join("held.bin"), b"held resource").unwrap();
    let output = ken_cli::build_native_program(source, ken_cli::SourceFormat::Ken, name, &dir)
        .expect("PX7-F checked program reaches the native resource lane");
    let observation = ken_runtime::run_bound_process_effect_observation_v1(
        &output.artifact,
        &ken_runtime::NativeEffectRunOptionsV1 {
            arguments: Vec::new(),
            environment: Vec::new(),
            cwd: dir.clone(),
            plan_hash: output.plan_transport_hash,
        },
    )
    .expect("linked PX7-F child emits its canonical observation");
    let _ = std::fs::remove_dir_all(dir);
    observation
}

const ESCAPE_CLOSED: &str = r#"program capabilities FS AFull
fn escape_body (resource : Resource FsHandle)
  : HostIO AFull (ResourceBodyResult Unit (Resource FsHandle)) =
  Ret (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (ResourceBodyResult Unit (Resource FsHandle))
    (ResourceBodyOk Unit (Resource FsHandle) resource)

proc after_escape (bracket : ResourceBracketResult Unit (Resource FsHandle))
  : HostIO AFull ExitCode visits [FS] =
  match bracket {
    ResourceBracketOk resource |->
      bind (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (Result ResourceError FileMetadata) ExitCode
        (resourceMetadata AFull resource)
        (\used. match used {
          Err Closed |-> host_exit AFull Success;
          Err error |-> host_exit AFull (Failure 91);
          Ok metadata |-> host_exit AFull (Failure 92)
        });
    ResourceBracketBodyError error |-> host_exit AFull (Failure 93);
    ResourceBracketReleaseError error |-> host_exit AFull (Failure 94);
    ResourceBracketBodyAndReleaseError body_error release_error |->
      host_exit AFull (Failure 95)
  }

proc after_outer
  (outcome : Result FileError (ResourceBracketResult Unit (Resource FsHandle)))
  : HostIO AFull ExitCode visits [FS] =
  match outcome {
    Err open_error |-> host_exit AFull (Failure 96);
    Ok bracket |-> after_escape bracket
  }

proc main (_input : ProcessInput) (caps : ProgramCaps AFull)
  : HostIO AFull ExitCode visits [FS] =
  match caps {
    MkProgramCaps cap |->
      bind (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (Result FileError (ResourceBracketResult Unit (Resource FsHandle))) ExitCode
        (withResource AFull Unit (Resource FsHandle)
          cap (bytes_encode "held.bin") ResourceMetadata
          (\resource. Ret (Coproduct (FSOp AFull) AmbientOp)
            (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
            (ResourceBodyResult Unit (Resource FsHandle))
            (ResourceBodyOk Unit (Resource FsHandle) resource)))
        (\outcome. after_outer outcome)
  }
"#;

const RIGHT_NOT_HELD: &str = r#"program capabilities FS AFull
fn metadata_after (outcome : Result ResourceError FileMetadata)
  : HostIO AFull (ResourceBodyResult ResourceError Unit) =
  match outcome {
    Err error |-> Ret (Coproduct (FSOp AFull) AmbientOp)
      (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
      (ResourceBodyResult ResourceError Unit)
      (ResourceBodyErr ResourceError Unit error);
    Ok metadata |-> Ret (Coproduct (FSOp AFull) AmbientOp)
      (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
      (ResourceBodyResult ResourceError Unit)
      (ResourceBodyOk ResourceError Unit MkUnit)
  }

proc metadata_body (resource : Resource FsHandle)
  : HostIO AFull (ResourceBodyResult ResourceError Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result ResourceError FileMetadata) (ResourceBodyResult ResourceError Unit)
    (resourceMetadata AFull resource) (\outcome. metadata_after outcome)

fn right_masks (error : ResourceError) : Bool =
  match error {
    ResourceHostIO io |-> False;
    Closed |-> False;
    MalformedResource |-> False;
    RightNotHeld required held |->
      match eq_int required 32 {
        True |-> eq_int held 1;
        False |-> False
      };
    ReleaseFailed kind identity io |-> False
  }

fn bracket_has_right_denial (bracket : ResourceBracketResult ResourceError Unit) : Bool =
  match bracket {
    ResourceBracketOk unit |-> False;
    ResourceBracketBodyError error |-> right_masks error;
    ResourceBracketReleaseError error |-> False;
    ResourceBracketBodyAndReleaseError body_error release_error |-> False
  }

fn after_right_outer
  (outcome : Result FileError (ResourceBracketResult ResourceError Unit))
  : HostIO AFull ExitCode =
  match outcome {
    Err open_error |-> host_exit AFull (Failure 81);
    Ok bracket |-> match bracket_has_right_denial bracket {
      True |-> host_exit AFull Success;
      False |-> host_exit AFull (Failure 82)
    }
  }

proc main (_input : ProcessInput) (caps : ProgramCaps AFull)
  : HostIO AFull ExitCode visits [FS] =
  match caps {
    MkProgramCaps cap |->
      bind (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (Result FileError (ResourceBracketResult ResourceError Unit)) ExitCode
        (withResource AFull ResourceError Unit
          cap (bytes_encode "held.bin") ResourceRead metadata_body)
        (\outcome. after_right_outer outcome)
  }
"#;

#[cfg(target_os = "linux")]
#[test]
fn linked_public_escape_is_exact_closed() {
    let observation = run("escape-closed", ESCAPE_CLOSED);
    assert_eq!(observation.exit_status, 0, "{observation:?}");
    assert_eq!(
        observation
            .effect_trace
            .iter()
            .map(|event| event.operation)
            .collect::<Vec<_>>(),
        vec![
            ken_runtime::HostOpV1::FsOpen,
            ken_runtime::HostOpV1::ResourceRelease,
            ken_runtime::HostOpV1::FsHandleMetadata,
        ]
    );
    assert!(matches!(
        observation.effect_trace[2].outcome,
        ken_runtime::CanonicalOutcomeV1::Error(ken_runtime::SemanticErrorV1::Resource(
            ken_runtime::ResourceErrorV1::Closed
        ))
    ));
}

#[cfg(target_os = "linux")]
#[test]
fn linked_public_right_denial_preserves_exact_masks() {
    let observation = run("right-denial", RIGHT_NOT_HELD);
    assert_eq!(observation.exit_status, 0, "{observation:?}");
    assert!(observation.effect_trace.iter().any(|event| matches!(
        event.outcome,
        ken_runtime::CanonicalOutcomeV1::Error(ken_runtime::SemanticErrorV1::Resource(
            ken_runtime::ResourceErrorV1::RightNotHeld {
                required: 32,
                held: 1
            }
        ))
    )));
}
