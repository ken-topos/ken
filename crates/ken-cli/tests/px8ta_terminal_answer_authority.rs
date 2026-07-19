//! PX8-TA terminal-answer authority through real nested resource brackets.

fn output_dir(label: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "ken-px8ta-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&path).unwrap();
    path
}

const COMMON: &str = r#"program capabilities FS AFull
fn px8ta_body_ok (_resource : Resource FsHandle)
  : HostIO AFull (ResourceBodyResult Unit Unit) =
  Ret (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (ResourceBodyResult Unit Unit)
    (ResourceBodyOk Unit Unit MkUnit)

const px8ta_body_failure
  : HostIO AFull (ResourceBodyResult Unit Unit) =
  Ret (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (ResourceBodyResult Unit Unit)
    (ResourceBodyErr Unit Unit MkUnit)

const px8ta_body_success
  : HostIO AFull (ResourceBodyResult Unit Unit) =
  Ret (Coproduct (FSOp AFull) AmbientOp)
    (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
    (ResourceBodyResult Unit Unit)
    (ResourceBodyOk Unit Unit MkUnit)

fn px8ta_bracket_to_body
  (outcome : Result FileError (ResourceBracketResult Unit Unit))
  : HostIO AFull (ResourceBodyResult Unit Unit) =
  match outcome {
    Err _ |-> px8ta_body_failure;
    Ok bracket |-> match bracket {
      ResourceBracketOk _ |-> px8ta_body_success;
      ResourceBracketBodyError _ |-> px8ta_body_failure;
      ResourceBracketReleaseError _ |-> px8ta_body_failure;
      ResourceBracketBodyAndReleaseError _ _ |-> px8ta_body_failure
    }
  }

fn px8ta_bracket_to_exit
  (outcome : Result FileError (ResourceBracketResult Unit Unit))
  : HostIO AFull ExitCode =
  match outcome {
    Err _ |-> host_exit AFull (Failure 91);
    Ok bracket |-> match bracket {
      ResourceBracketOk _ |-> host_exit AFull Success;
      ResourceBracketBodyError _ |-> host_exit AFull (Failure 92);
      ResourceBracketReleaseError _ |-> host_exit AFull (Failure 93);
      ResourceBracketBodyAndReleaseError _ _ |-> host_exit AFull (Failure 94)
    }
  }
"#;

const INNER: &str = r#"
proc main (_input : ProcessInput) (caps : ProgramCaps AFull)
  : HostIO AFull ExitCode visits [FS] =
  match caps {
    MkProgramCaps cap |->
      bind (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (Result FileError (ResourceBracketResult Unit Unit)) ExitCode
        (withResource AFull Unit Unit cap (bytes_encode "inner.bin")
          ResourceMetadata px8ta_body_ok)
        (\outcome. px8ta_bracket_to_exit outcome)
  }
"#;

const TWO_LEVEL: &str = r#"
proc main (_input : ProcessInput) (caps : ProgramCaps AFull)
  : HostIO AFull ExitCode visits [FS] =
  match caps {
    MkProgramCaps cap |->
      bind (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (Result FileError (ResourceBracketResult Unit Unit)) ExitCode
        (withResource AFull Unit Unit cap (bytes_encode "outer.bin")
          ResourceRead
          (\_outer.
            bind (Coproduct (FSOp AFull) AmbientOp)
              (resp_coproduct (FSOp AFull) AmbientOp
                (fs_resp AFull) ambient_resp)
              (Result FileError (ResourceBracketResult Unit Unit))
              (ResourceBodyResult Unit Unit)
              (withResource AFull Unit Unit cap (bytes_encode "inner.bin")
                ResourceMetadata px8ta_body_ok)
              (\outcome. px8ta_bracket_to_body outcome)))
        (\outcome. px8ta_bracket_to_exit outcome)
  }
"#;

const THREE_LEVEL: &str = r#"
proc main (_input : ProcessInput) (caps : ProgramCaps AFull)
  : HostIO AFull ExitCode visits [FS] =
  match caps {
    MkProgramCaps cap |->
      bind (Coproduct (FSOp AFull) AmbientOp)
        (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
        (Result FileError (ResourceBracketResult Unit Unit)) ExitCode
        (withResource AFull Unit Unit cap (bytes_encode "outer.bin")
          ResourceRead
          (\_outer.
            bind (Coproduct (FSOp AFull) AmbientOp)
              (resp_coproduct (FSOp AFull) AmbientOp
                (fs_resp AFull) ambient_resp)
              (Result FileError (ResourceBracketResult Unit Unit))
              (ResourceBodyResult Unit Unit)
              (withResource AFull Unit Unit cap (bytes_encode "middle.bin")
                ResourceRead
                (\_middle.
                  bind (Coproduct (FSOp AFull) AmbientOp)
                    (resp_coproduct (FSOp AFull) AmbientOp
                      (fs_resp AFull) ambient_resp)
                    (Result FileError (ResourceBracketResult Unit Unit))
                    (ResourceBodyResult Unit Unit)
                    (withResource AFull Unit Unit cap (bytes_encode "inner.bin")
                      ResourceMetadata px8ta_body_ok)
                    (\outcome. px8ta_bracket_to_body outcome)))
              (\outcome. px8ta_bracket_to_body outcome)))
        (\outcome. px8ta_bracket_to_exit outcome)
  }
"#;

fn run(label: &str, tail: &str) -> ken_runtime::EffectObservation {
    let dir = output_dir(label);
    std::fs::write(dir.join("outer.bin"), b"outer").unwrap();
    std::fs::write(dir.join("middle.bin"), b"middle").unwrap();
    std::fs::write(dir.join("inner.bin"), b"inner").unwrap();
    let source = format!("{COMMON}{tail}");
    let output = ken_cli::build_native_program(
        &source,
        ken_cli::SourceFormat::Ken,
        &format!("px8ta-{label}"),
        &dir,
    )
    .expect("checked nested bracket reaches the linked artifact");
    let observation = ken_runtime::run_bound_process_effect_observation(
        &output.artifact,
        &ken_runtime::NativeEffectRunOptionsV1 {
            arguments: Vec::new(),
            environment: Vec::new(),
            cwd: dir.clone(),
            plan_hash: output.plan_transport_hash,
        },
    )
    .expect("linked nested bracket produces an observation");
    let _ = std::fs::remove_dir_all(dir);
    observation
}

#[test]
fn linked_nested_brackets_drain_before_terminal_answer_authority() {
    let inner = run("inner", INNER);
    assert_eq!(inner.exit_status, 0);
    assert_eq!(
        inner
            .effect_trace
            .iter()
            .map(|event| event.operation)
            .collect::<Vec<_>>(),
        vec![
            ken_runtime::HostOpV1::FsOpen,
            ken_runtime::HostOpV1::ResourceRelease,
        ]
    );

    let two = run("two", TWO_LEVEL);
    assert_eq!(two.exit_status, 0, "{two:#?}");
    assert_eq!(
        two.effect_trace
            .iter()
            .map(|event| event.operation)
            .collect::<Vec<_>>(),
        vec![
            ken_runtime::HostOpV1::FsOpen,
            ken_runtime::HostOpV1::FsOpen,
            ken_runtime::HostOpV1::ResourceRelease,
            ken_runtime::HostOpV1::ResourceRelease,
        ]
    );

    let three = run("three", THREE_LEVEL);
    assert_eq!(three.exit_status, 0, "{three:#?}");
    assert_eq!(
        three
            .effect_trace
            .iter()
            .map(|event| event.operation)
            .collect::<Vec<_>>(),
        vec![
            ken_runtime::HostOpV1::FsOpen,
            ken_runtime::HostOpV1::FsOpen,
            ken_runtime::HostOpV1::FsOpen,
            ken_runtime::HostOpV1::ResourceRelease,
            ken_runtime::HostOpV1::ResourceRelease,
            ken_runtime::HostOpV1::ResourceRelease,
        ]
    );
}
