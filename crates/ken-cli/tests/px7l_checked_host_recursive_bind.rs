fn output_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "ken-px7l-{name}-{}-{}",
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
proc selected_body (terminal : Bool) (message : String)
  : Unit -> HostIO APartial Unit visits [Console] =
  \_. match terminal {
    False |-> host_console APartial Unit (print_line message) ;
    True |-> bind (Coproduct (FSOp APartial) AmbientOp)
      (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
      (Result IOError Unit) Unit
      (host_console APartial (Result IOError Unit) (flush Stdout))
      (\_. Ret (Coproduct (FSOp APartial) AmbientOp)
        (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
        Unit MkUnit)
  }

proc delayed (body : Unit -> HostIO APartial Unit)
  : HostIO APartial ExitCode visits [Console] =
  bind (Coproduct (FSOp APartial) AmbientOp)
    (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
    Unit ExitCode
    (body MkUnit)
    (\_. bind (Coproduct (FSOp APartial) AmbientOp)
      (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
      (Result IOError Unit) ExitCode
      (host_console APartial (Result IOError Unit) (flush Stdout))
      (\_. host_exit APartial Success))

proc main (_input : ProcessInput) (_caps : ProgramCaps APartial)
  : HostIO APartial ExitCode visits [Console] =
  bind (Coproduct (FSOp APartial) AmbientOp)
    (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
    Bool ExitCode
    (host_console APartial Bool (is_terminal Stdout))
    (\terminal. delayed (selected_body terminal "captured"))
"#;

const STATIC_DIRECT_VIS: &str = r#"program capabilities FS APartial
proc main (_input : ProcessInput) (_caps : ProgramCaps APartial)
  : HostIO APartial ExitCode visits [Console] =
  host_program APartial (print_line "static")
"#;

const CONSUMED_RUNTIME_RESPONSE: &str = r#"program capabilities FS APartial
proc selected_result (terminal : Bool) (message : String)
  : Unit -> HostIO APartial (Result IOError Unit) visits [Console] =
  \_. match terminal {
    False |-> host_console APartial (Result IOError Unit)
      (write Stdout (bytes_encode message)) ;
    True |-> host_console APartial (Result IOError Unit) (flush Stdout)
  }

proc delayed_result (body : Unit -> HostIO APartial (Result IOError Unit))
  : HostIO APartial ExitCode visits [Console] =
  bind (Coproduct (FSOp APartial) AmbientOp)
    (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
    (Result IOError Unit) ExitCode
    (body MkUnit)
    (\written. match written {
      Err _ |-> host_program APartial (print_line "write-error") ;
      Ok _ |-> host_program APartial (print_line "write-ok")
    })

proc main (_input : ProcessInput) (_caps : ProgramCaps APartial)
  : HostIO APartial ExitCode visits [Console] =
  bind (Coproduct (FSOp APartial) AmbientOp)
    (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
    Bool ExitCode
    (host_console APartial Bool (is_terminal Stdout))
    (\terminal. delayed_result (selected_result terminal "probe"))
"#;

fn contains_recursive_bind_ir(expr: &ken_runtime::RuntimeExpr) -> bool {
    use ken_runtime::RuntimeExpr;
    match expr {
        RuntimeExpr::ComputationalMatch { .. } | RuntimeExpr::LexicalClosure { .. } => true,
        RuntimeExpr::Let { value, body } => {
            contains_recursive_bind_ir(value) || contains_recursive_bind_ir(body)
        }
        RuntimeExpr::If {
            scrutinee,
            then_expr,
            else_expr,
        } => {
            contains_recursive_bind_ir(scrutinee)
                || contains_recursive_bind_ir(then_expr)
                || contains_recursive_bind_ir(else_expr)
        }
        RuntimeExpr::PrimitiveCall { args, .. } | RuntimeExpr::Construct { args, .. } => {
            args.iter().any(contains_recursive_bind_ir)
        }
        RuntimeExpr::Match {
            scrutinee, cases, ..
        } => {
            contains_recursive_bind_ir(scrutinee)
                || cases
                    .iter()
                    .any(|case| contains_recursive_bind_ir(&case.body))
        }
        RuntimeExpr::Record { fields } => fields
            .iter()
            .any(|(_, value)| contains_recursive_bind_ir(value)),
        RuntimeExpr::Project { record, .. } => contains_recursive_bind_ir(record),
        RuntimeExpr::Closure { body, .. } => contains_recursive_bind_ir(body),
        RuntimeExpr::Call { callee, args } => {
            contains_recursive_bind_ir(callee) || args.iter().any(contains_recursive_bind_ir)
        }
        RuntimeExpr::Effect {
            capability, args, ..
        } => {
            capability
                .as_ref()
                .is_some_and(|capability| contains_recursive_bind_ir(&capability.value))
                || args.iter().any(contains_recursive_bind_ir)
        }
        RuntimeExpr::Value(_)
        | RuntimeExpr::Var(_)
        | RuntimeExpr::DeclarationRef { .. }
        | RuntimeExpr::ImportedDeclarationRef { .. }
        | RuntimeExpr::Trap(_) => false,
    }
}

#[test]
fn delayed_capturing_generic_bind_agrees_across_real_executors() {
    let dir = output_dir("agreement");
    let output = ken_cli::build_native_program(
        PROGRAM,
        ken_cli::SourceFormat::Ken,
        "px7l-recursive-bind",
        &dir,
    )
    .expect("generic checked HostIO bind reaches the linked artifact");
    let native = ken_runtime::run_bound_process_effect_observation_v1(
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
    let interpreted = ken_cli::run_program_effect_observation_v1(
        PROGRAM,
        ken_cli::SourceFormat::Ken,
        &[b"ken".to_vec()],
        &[],
        b"/",
        &mut host,
    )
    .expect("same checked source runs through interpreter");
    assert_eq!(native, interpreted);
    assert_eq!(native.exit_status, 0);
    assert_eq!(native.stdout, b"captured\n");
    assert_eq!(native.effect_trace.len(), 3);
    assert_eq!(
        native
            .effect_trace
            .iter()
            .map(|event| event.operation)
            .collect::<Vec<_>>(),
        vec![
            ken_runtime::HostOpV1::ConsoleIsTerminal,
            ken_runtime::HostOpV1::ConsoleWrite,
            ken_runtime::HostOpV1::ConsoleFlush,
        ]
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn runtime_selected_non_unit_response_is_consumed_across_real_executors() {
    let dir = output_dir("consumed-response");
    let output = ken_cli::build_native_program(
        CONSUMED_RUNTIME_RESPONSE,
        ken_cli::SourceFormat::Ken,
        "px7l-consumed-runtime-response",
        &dir,
    )
    .expect("runtime-selected Result response reaches the linked artifact");
    let native = ken_runtime::run_bound_process_effect_observation_v1(
        &output.artifact,
        &ken_runtime::NativeEffectRunOptionsV1 {
            arguments: Vec::new(),
            environment: Vec::new(),
            cwd: dir.clone(),
            plan_hash: output.plan_transport_hash,
        },
    )
    .expect("linked artifact consumes its runtime-selected response");

    let mut host = ken_interp::CaptureHost::new(Vec::new());
    let interpreted = ken_cli::run_program_effect_observation_v1(
        CONSUMED_RUNTIME_RESPONSE,
        ken_cli::SourceFormat::Ken,
        &[b"ken".to_vec()],
        &[],
        b"/",
        &mut host,
    )
    .expect("interpreter consumes the same runtime-selected response");
    assert_eq!(native, interpreted);
    assert_eq!(native.exit_status, 0);
    assert_eq!(native.stdout, b"probewrite-ok\n");
    assert_eq!(
        native
            .effect_trace
            .iter()
            .map(|event| event.operation)
            .collect::<Vec<_>>(),
        vec![
            ken_runtime::HostOpV1::ConsoleIsTerminal,
            ken_runtime::HostOpV1::ConsoleWrite,
            ken_runtime::HostOpV1::ConsoleWrite,
        ]
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn static_direct_vis_retains_the_existing_lowering_path() {
    let dir = output_dir("static-direct-vis");
    let output = ken_cli::build_native_program(
        STATIC_DIRECT_VIS,
        ken_cli::SourceFormat::Ken,
        "px7l-static-direct-vis",
        &dir,
    )
    .expect("static direct Vis remains supported");
    let main = output
        .runtime_program
        .declarations
        .iter()
        .find(|declaration| declaration.symbol == output.plan.main().to_string())
        .expect("runtime program contains checked main");
    let ken_runtime::RuntimeDeclarationKind::Transparent { body } = &main.kind else {
        panic!("checked main remains transparent")
    };
    assert!(
        !contains_recursive_bind_ir(body),
        "static direct Vis must not be rerouted through PX7-L dynamic machinery"
    );
    let ran = std::process::Command::new(&output.artifact.executable_path)
        .output()
        .expect("static artifact runs");
    assert_eq!(ran.status.code(), Some(0));
    assert_eq!(ran.stdout, b"static\n");
    let _ = std::fs::remove_dir_all(dir);
}
