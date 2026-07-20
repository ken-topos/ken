//! PX8-TA public checked-bracket oriented-subcontinuation controls.

fn output_dir(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "ken-px8ta-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&path).unwrap();
    path
}

const NESTED_BRACKET_PROGRAM: &str = r#"program capabilities FS AFull
fn leaf_body (_resource : Resource FsHandle)
  : HostIO AFull (ResourceBodyResult Unit Unit) =
  Ret (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (ResourceBodyResult Unit Unit)
    (ResourceBodyOk Unit Unit MkUnit)

fn body_result
  (outcome : Result FileError (ResourceBracketResult Unit Unit))
  : HostIO AFull (ResourceBodyResult Unit Unit) =
  match outcome {
    Ok (ResourceBracketOk unit) |->
      Ret (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (ResourceBodyResult Unit Unit)
        (ResourceBodyOk Unit Unit MkUnit);
    Ok bracket |->
      Ret (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (ResourceBodyResult Unit Unit)
        (ResourceBodyErr Unit Unit MkUnit);
    Err error |->
      Ret (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (ResourceBodyResult Unit Unit)
        (ResourceBodyErr Unit Unit MkUnit)
  }

proc level_one_body
  (cap : Cap AFull) (_resource : Resource FsHandle)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result FileError (ResourceBracketResult Unit Unit))
    (ResourceBodyResult Unit Unit)
    (withResource AFull Unit Unit cap (bytes_encode "held-1.bin")
      ResourceMetadata leaf_body)
    (\outcome. body_result outcome)

proc level_two_body
  (cap : Cap AFull) (_resource : Resource FsHandle)
  : HostIO AFull (ResourceBodyResult Unit Unit) visits [FS] =
  bind (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (Result FileError (ResourceBracketResult Unit Unit))
    (ResourceBodyResult Unit Unit)
    (withResource AFull Unit Unit cap (bytes_encode "held-1.bin")
      ResourceMetadata (\resource. level_one_body cap resource))
    (\outcome. body_result outcome)

fn after_root
  (outcome : Result FileError (ResourceBracketResult Unit Unit))
  : HostIO AFull ExitCode =
  match outcome {
    Ok (ResourceBracketOk unit) |-> host_exit AFull Success;
    Ok (ResourceBracketBodyError error) |-> host_exit AFull (Failure 81);
    Ok (ResourceBracketReleaseError error) |-> host_exit AFull (Failure 82);
    Ok (ResourceBracketBodyAndReleaseError body_error release_error) |->
      host_exit AFull (Failure 83);
    Err error |-> host_exit AFull (Failure 84)
  }

proc main (_input : ProcessInput) (caps : ProgramCaps AFull)
  : HostIO AFull ExitCode visits [FS] =
  match caps {
    MkProgramCaps cap |->
      bind (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (Result FileError (ResourceBracketResult Unit Unit)) ExitCode
        (withResource AFull Unit Unit cap (bytes_encode "held-0.bin")
          ResourceMetadata __ROOT_BODY__)
        (\outcome. after_root outcome)
  }
"#;

#[cfg(target_os = "linux")]
fn run_depth(depth: usize) -> (ken_runtime::EffectObservation, usize) {
    let body = match depth {
        1 => "leaf_body",
        2 => {
            r#"(\resource.
          bind (Coproduct (FSOp AFull) AmbientOp)
            (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
            (Result FileError (ResourceBracketResult Unit Unit))
            (ResourceBodyResult Unit Unit)
            (withResource AFull Unit Unit cap (bytes_encode "held-1.bin")
              ResourceMetadata leaf_body)
            (\outcome. body_result outcome))"#
        }
        3 => {
            r#"(\resource.
          bind (Coproduct (FSOp AFull) AmbientOp)
            (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
            (Result FileError (ResourceBracketResult Unit Unit))
            (ResourceBodyResult Unit Unit)
            (withResource AFull Unit Unit cap (bytes_encode "held-1.bin")
              ResourceMetadata (\inner_resource.
                bind (Coproduct (FSOp AFull) AmbientOp)
                  (resp_coproduct (FSOp AFull) AmbientOp
                    (fs_resp AFull) ambient_resp)
                  (Result FileError (ResourceBracketResult Unit Unit))
                  (ResourceBodyResult Unit Unit)
                  (withResource AFull Unit Unit cap (bytes_encode "held-2.bin")
                    ResourceMetadata leaf_body)
                  (\inner_outcome. body_result inner_outcome)))
            (\outcome. body_result outcome))"#
        }
        _ => panic!("PX8-TA public control supports depths one through three"),
    };
    let source = NESTED_BRACKET_PROGRAM.replace("__ROOT_BODY__", body);
    let dir = output_dir(&format!("depth-{depth}"));
    for index in 0..depth {
        std::fs::write(
            dir.join(format!("held-{index}.bin")),
            format!("held resource {index}"),
        )
        .unwrap();
    }
    let output = ken_cli::build_native_program(
        &source,
        ken_cli::SourceFormat::Ken,
        &format!("px8ta-depth-{depth}"),
        &dir,
    )
    .unwrap_or_else(|error| {
        panic!("depth {depth} checked nested bracket reaches native lowering: {error:?}")
    });
    let plan = output
        .runtime_program
        .erased_core
        .metadata
        .checked_core
        .metadata
        .values()
        .find(|bytes| bytes.starts_with(ken_runtime::ORIENTED_SUBCONTINUATION_PLAN_V1_HEADER))
        .and_then(|bytes| ken_runtime::OrientedSubcontinuationPlanV1::decode(bytes).ok())
        .expect("checked nested bracket transports its oriented answer plan");
    let observation = ken_runtime::run_bound_process_effect_observation(
        &output.artifact,
        &ken_runtime::NativeEffectRunOptionsV1 {
            arguments: Vec::new(),
            environment: Vec::new(),
            cwd: dir.clone(),
            plan_hash: output.plan_transport_hash,
        },
    )
    .expect("linked nested bracket emits its canonical observation");
    let _ = std::fs::remove_dir_all(dir);
    (observation, plan.frames.len())
}

#[cfg(target_os = "linux")]
#[test]
fn public_one_level_bracket_finishes_and_releases() {
    assert_depth_finishes_and_releases_lifo(1);
}

#[cfg(target_os = "linux")]
#[test]
fn public_two_three_level_brackets_finish_and_release_lifo() {
    for depth in 2..=3 {
        assert_depth_finishes_and_releases_lifo(depth);
    }
}

#[cfg(target_os = "linux")]
fn assert_depth_finishes_and_releases_lifo(depth: usize) {
    let (observation, planned_frames) = run_depth(depth);
    assert_eq!(observation.exit_status, 0, "depth {depth}: {observation:?}");
    assert_eq!(observation.terminal_error, None, "depth {depth}");
    assert!(
        planned_frames >= depth,
        "depth {depth} must retain every checked bracket continuation"
    );

    let opens = observation
        .effect_trace
        .iter()
        .filter(|event| event.operation == ken_runtime::HostOpV1::FsOpen)
        .map(|event| event.resource_bindings[0].1.clone())
        .collect::<Vec<_>>();
    let releases = observation
        .effect_trace
        .iter()
        .filter(|event| event.operation == ken_runtime::HostOpV1::ResourceRelease)
        .map(|event| event.resource_bindings[0].1.clone())
        .collect::<Vec<_>>();
    assert_eq!(opens.len(), depth, "depth {depth} acquisition count");
    assert_eq!(releases.len(), depth, "depth {depth} release count");
    assert_eq!(
        releases,
        opens.into_iter().rev().collect::<Vec<_>>(),
        "depth {depth} releases must be strict LIFO"
    );
}
