//! PX8-X sole observation schema through the real linked artifact.

fn output_dir() -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "ken-px8x-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&path).unwrap();
    path
}

const RESOURCE_PROGRAM: &str = r#"program capabilities FS AFull
fn px8x_body (_resource : Resource FsHandle)
  : HostIO AFull (ResourceBodyResult Unit Unit) =
  Ret (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (ResourceBodyResult Unit Unit)
    (ResourceBodyOk Unit Unit MkUnit)

fn px8x_after
  (outcome : Result FileError (ResourceBracketResult Unit Unit))
  : HostIO AFull ExitCode =
  match outcome {
    Err _ |-> host_exit AFull (Failure 41);
    Ok bracket |-> match bracket {
      ResourceBracketOk _ |-> host_exit AFull Success;
      ResourceBracketBodyError _ |-> host_exit AFull (Failure 42);
      ResourceBracketReleaseError _ |-> host_exit AFull (Failure 43);
      ResourceBracketBodyAndReleaseError _ _ |-> host_exit AFull (Failure 44)
    }
  }

proc main (_input : ProcessInput) (caps : ProgramCaps AFull)
  : HostIO AFull ExitCode visits [FS] =
  match caps {
    MkProgramCaps cap |->
      bind (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (Result FileError (ResourceBracketResult Unit Unit)) ExitCode
        (withResource AFull Unit Unit cap (bytes_encode "held.bin")
          ResourceMetadata px8x_body)
        (\outcome. px8x_after outcome)
  }
"#;

#[test]
fn linked_route_exposes_real_ordered_bindings_and_filters_reserved_input() {
    let dir = output_dir();
    std::fs::write(dir.join("held.bin"), b"held").unwrap();
    let output = ken_cli::build_native_program(
        RESOURCE_PROGRAM,
        ken_cli::SourceFormat::Ken,
        "px8x-single-schema-observation",
        &dir,
    )
    .expect("checked resource program reaches the linked artifact");
    let options = ken_runtime::NativeEffectRunOptionsV1 {
        arguments: Vec::new(),
        environment: vec![(
            "KEN_HOST_OBSERVATION_PATH".into(),
            "caller-controlled".into(),
        )],
        cwd: dir.clone(),
        plan_hash: output.plan_transport_hash,
    };
    let observation = ken_runtime::run_bound_process_effect_observation(&output.artifact, &options)
        .expect("sole route reads the role-labelled trace");

    assert_eq!(observation.exit_status, 0);
    assert_eq!(
        observation.terminal_exit,
        ken_runtime::TerminalExitClass::NormalReturn
    );
    assert_eq!(
        observation
            .effect_trace
            .iter()
            .map(|event| (event.sequence, event.operation))
            .collect::<Vec<_>>(),
        vec![
            (0, ken_runtime::HostOpV1::FsOpen),
            (1, ken_runtime::HostOpV1::ResourceRelease),
        ]
    );
    let acquired = observation.effect_trace[0].resource_bindings.as_slice();
    let released = observation.effect_trace[1].resource_bindings.as_slice();
    assert_eq!(acquired.len(), 1);
    assert_eq!(released.len(), 1);
    assert_eq!(acquired[0].0, ken_runtime::ResourceBindingRole::Target);
    assert_eq!(released[0].0, ken_runtime::ResourceBindingRole::Target);
    assert_eq!(acquired[0].1, released[0].1);

    let _ = std::fs::remove_dir_all(dir);
}
