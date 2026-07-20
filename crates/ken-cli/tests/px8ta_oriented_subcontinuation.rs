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

const PX8DS_SIBLING_RECURSION_PROGRAM: &str = r#"program capabilities FS APartial
proc countdown (fuel : Nat)
  : HostIO APartial (Result Int Bool) visits [Console] =
  match fuel {
    Zero |-> Ret (Coproduct (FSOp APartial) AmbientOp)
      (resp_coproduct (FSOp APartial) AmbientOp
        (fs_resp APartial) ambient_resp)
      (Result Int Bool) (Ok Int Bool True);
    Suc rest |-> bind (Coproduct (FSOp APartial) AmbientOp)
      (resp_coproduct (FSOp APartial) AmbientOp
        (fs_resp APartial) ambient_resp)
      Bool (Result Int Bool)
      (host_console APartial Bool (is_terminal Stdout))
      (\terminal. match terminal {
        False |-> countdown rest;
        True |-> countdown rest
      })
  }

fn after_countdown (_outcome : Result Int Bool)
  : HostIO APartial (ResourceBodyResult Unit Unit) =
  Ret (Coproduct (FSOp APartial) AmbientOp)
    (resp_coproduct (FSOp APartial) AmbientOp
      (fs_resp APartial) ambient_resp)
    (ResourceBodyResult Unit Unit) (ResourceBodyOk Unit Unit MkUnit)

proc after_first_countdown (_outcome : Result Int Bool)
  : HostIO APartial (ResourceBodyResult Unit Unit) visits [Console] =
  bind (Coproduct (FSOp APartial) AmbientOp)
    (resp_coproduct (FSOp APartial) AmbientOp
      (fs_resp APartial) ambient_resp)
    (Result Int Bool) (ResourceBodyResult Unit Unit)
    (countdown (Suc Zero))
    (\outcome. after_countdown outcome)

proc selected_buffer_body (terminal : Bool)
  : HostIO APartial (ResourceBodyResult Unit Unit) visits [Console] =
  match terminal {
    False |-> after_countdown (Ok Int Bool False);
    True |-> bind (Coproduct (FSOp APartial) AmbientOp)
      (resp_coproduct (FSOp APartial) AmbientOp
        (fs_resp APartial) ambient_resp)
      (Result Int Bool) (ResourceBodyResult Unit Unit)
      (countdown (Suc (Suc Zero)))
      (\outcome. after_first_countdown outcome)
  }

proc buffer_body (_buffer : Resource Buffer)
  : HostIO APartial (ResourceBodyResult Unit Unit) visits [Console] =
  bind (Coproduct (FSOp APartial) AmbientOp)
    (resp_coproduct (FSOp APartial) AmbientOp
      (fs_resp APartial) ambient_resp)
    Bool (ResourceBodyResult Unit Unit)
    (host_console APartial Bool (is_terminal Stdout))
    (\terminal. selected_buffer_body terminal)

fn finish_buffer
  (outcome : Result ResourceError (ResourceBracketResult Unit Unit))
  : HostIO APartial ExitCode =
  match outcome {
    Err _ |-> host_exit APartial (Failure 91);
    Ok (ResourceBracketOk _) |-> host_exit APartial Success;
    Ok _ |-> host_exit APartial (Failure 92)
  }

proc main (_input : ProcessInput) (_caps : ProgramCaps APartial)
  : HostIO APartial ExitCode visits [FS, Console] =
  bind (Coproduct (FSOp APartial) AmbientOp)
    (resp_coproduct (FSOp APartial) AmbientOp
      (fs_resp APartial) ambient_resp)
    (Result ResourceError (ResourceBracketResult Unit Unit)) ExitCode
    (withBuffer APartial Unit Unit (2 : Int) buffer_body)
    (\outcome. finish_buffer outcome)
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
#[test]
fn px8ds_real_same_depth_path_rejects_flat_order_and_runs_exact_edges() {
    std::thread::Builder::new()
        .name("px8ds-real-siblings".to_string())
        .stack_size(256 * 1024 * 1024)
        .spawn(run_px8ds_real_same_depth_path)
        .expect("spawn large-stack PX8-DS discriminator")
        .join()
        .expect("PX8-DS discriminator thread");
}

#[cfg(target_os = "linux")]
fn run_px8ds_real_same_depth_path() {
    let retired_dir = output_dir("px8ds-retired-flat");
    let retired = ken_runtime::with_px8ds_retired_flat_order(|| {
        ken_cli::build_native_program(
            PX8DS_SIBLING_RECURSION_PROGRAM,
            ken_cli::SourceFormat::Ken,
            "px8ds-retired-flat",
            &retired_dir,
        )
    })
    .expect_err("the retired cross-instance flat order must reproduce its false rejection");
    let retired = format!("{retired:?}");
    assert!(
        retired.contains("retired flat oriented splice answer endpoints do not compose"),
        "the real checked producer must reach the retired consumer: {retired}"
    );
    assert_eq!(
        retired.matches("depth=1").count(),
        2,
        "the reaching discriminator must reject two same-depth IH instances: {retired}"
    );
    let _ = std::fs::remove_dir_all(retired_dir);

    let exact_dir = output_dir("px8ds-exact-edges");
    let exact = ken_cli::build_native_program(
        PX8DS_SIBLING_RECURSION_PROGRAM,
        ken_cli::SourceFormat::Ken,
        "px8ds-exact-edges",
        &exact_dir,
    )
    .expect("exact dynamic edges compile the same checked source");
    let observation = ken_runtime::run_bound_process_effect_observation(
        &exact.artifact,
        &ken_runtime::NativeEffectRunOptionsV1 {
            arguments: Vec::new(),
            environment: Vec::new(),
            cwd: exact_dir.clone(),
            plan_hash: exact.plan_transport_hash,
        },
    )
    .expect("the exact-edge linked artifact runs");
    assert_eq!(observation.exit_status, 0, "{observation:?}");
    assert_eq!(observation.terminal_error, None);
    assert_eq!(
        observation
            .effect_trace
            .iter()
            .filter(|event| event.operation == ken_runtime::HostOpV1::ConsoleIsTerminal)
            .count(),
        1,
        "the live false branch must skip both recursive siblings"
    );
    assert_eq!(
        observation
            .effect_trace
            .iter()
            .map(|event| event.operation)
            .collect::<Vec<_>>(),
        vec![
            ken_runtime::HostOpV1::BufferAllocate,
            ken_runtime::HostOpV1::ConsoleIsTerminal,
            ken_runtime::HostOpV1::ResourceRelease,
        ]
    );
    let _ = std::fs::remove_dir_all(exact_dir);
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
