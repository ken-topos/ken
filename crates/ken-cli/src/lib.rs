//! Public application runner shared by the ken binary and deterministic tests.

use std::rc::Rc;

/// Selects the source front end used by run_program.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceFormat {
    Ken,
    LiterateKen,
}

/// The ordinary process outcome returned after the host tree is fully driven.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProgramOutcome {
    pub exit_status: i32,
}

/// Build one checked Program I source into a linked native process artifact.
/// All process observations remain supplied by the linked artifact at call
/// time; this library operation reads no ambient argv, environment, or cwd.
pub fn build_native_program(
    source: &str,
    format: SourceFormat,
    package_name: &str,
    output_dir: impl AsRef<std::path::Path>,
) -> Result<
    ken_elaborator::compiler_driver::NativeProgramBuildOutput,
    ken_elaborator::compiler_driver::NativeProgramBuildError,
> {
    let name = match format {
        SourceFormat::Ken => "src/main.ken",
        SourceFormat::LiterateKen => "src/main.ken.md",
    };
    ken_elaborator::compiler_driver::compile_native_program_sources(
        package_name,
        vec![ken_elaborator::compiler_driver::CompilerSource::new(
            name, source,
        )],
        output_dir,
    )
}

/// A failure before an ordinary Ken ProgramOutcome can be produced.
#[derive(Debug)]
pub enum RunError {
    Initialization(ken_elaborator::ElabError),
    Elaboration(ken_elaborator::ElabError),
    DuplicateEntrypoint,
    MissingEntrypoint,
    EntrypointAbiUnavailable,
    MissingCapability { effect: String },
    InvalidEntrypoint { authority: &'static str },
    ConsoleAbiUnavailable,
    RootExecutionObservationUnavailable,
    CapabilityRoot(std::io::Error),
    Io(ken_interp::RunIoError),
}

/// Elaborates and runs one Ken application using only injected process data and
/// the supplied host.
///
/// The host mints declaration-bounded capabilities rooted in its own identity;
/// the program's effects are then driven by the interpreter's real run_io path.
pub fn run_program<H: ken_interp::HostHandler>(
    source: &str,
    format: SourceFormat,
    arguments: &[Vec<u8>],
    environment: &[(Vec<u8>, Vec<u8>)],
    cwd: &[u8],
    host: &mut H,
) -> Result<ProgramOutcome, RunError> {
    let effective_uid = ken_runtime::observe_effective_uid_v1()
        .map_err(|_| RunError::RootExecutionObservationUnavailable)?;
    run_program_inner(
        source,
        format,
        arguments,
        environment,
        cwd,
        host,
        false,
        effective_uid,
    )
    .map(|(outcome, _)| outcome)
}

/// Elaborate and run one Ken application while producing the interpreter's
/// canonical host-effect observation from the real dispatch seam.
///
/// The returned filesystem delta is empty because the descriptor-only
/// interpreter host interface exposes no root snapshot. A differential harness
/// may fill that field from its independently captured before/after root state.
pub fn run_program_effect_observation_v1<H: ken_interp::HostHandler>(
    source: &str,
    format: SourceFormat,
    arguments: &[Vec<u8>],
    environment: &[(Vec<u8>, Vec<u8>)],
    cwd: &[u8],
    host: &mut H,
) -> Result<ken_runtime::EffectObservationV1, RunError> {
    let effective_uid = ken_runtime::observe_effective_uid_v1()
        .map_err(|_| RunError::RootExecutionObservationUnavailable)?;
    run_program_inner(
        source,
        format,
        arguments,
        environment,
        cwd,
        host,
        true,
        effective_uid,
    )
    .map(|(_, observation)| observation.expect("observation requested"))
}

fn run_program_inner<H: ken_interp::HostHandler>(
    source: &str,
    format: SourceFormat,
    arguments: &[Vec<u8>],
    environment: &[(Vec<u8>, Vec<u8>)],
    cwd: &[u8],
    host: &mut H,
    observe_effects: bool,
    effective_uid: ken_runtime::EffectiveUidSnapshotV1,
) -> Result<(ProgramOutcome, Option<ken_runtime::EffectObservationV1>), RunError> {
    let mut elab_env = ken_elaborator::ElabEnv::new().map_err(RunError::Initialization)?;
    let elaborated = match format {
        SourceFormat::Ken => elab_env.elaborate_file(source),
        SourceFormat::LiterateKen => elab_env.elaborate_ken_md_file(source),
    };
    if let Err(error) = elaborated {
        return match error {
            ken_elaborator::ElabError::DuplicateDefinition { ref name, .. } if name == "main" => {
                Err(RunError::DuplicateEntrypoint)
            }
            other => Err(RunError::Elaboration(other)),
        };
    }

    let admitted = ken_elaborator::program_admission::admit_checked_main(&elab_env)
        .map_err(map_program_admission_error)?;
    if ken_runtime::admit_root_execution(effective_uid, admitted.allow_root_execution).is_err() {
        let exit_status =
            ken_runtime::process_exit_status(ken_runtime::ProcessExitCode::Failure(0)).status;
        return Ok((
            ProgramOutcome { exit_status },
            observe_effects.then_some(ken_runtime::EffectObservationV1 {
                stdout: Vec::new(),
                stderr: Vec::new(),
                filesystem_delta: Vec::new(),
                terminal_error: Some(ken_runtime::TerminalErrorV1::RootExecutionDenied),
                effect_trace: Vec::new(),
                exit_status,
            }),
        ));
    }
    let main_id = admitted.main;
    let get = |name: &str| -> Option<ken_kernel::GlobalId> { elab_env.globals.get(name).copied() };
    let declared_fs = DeclaredFsAuthority {
        name: match admitted.authority_name.as_str() {
            "ANone" => "ANone",
            "APartial" => "APartial",
            "AFull" => "AFull",
            _ => unreachable!("admission returned an unknown authority"),
        },
        authority: admitted.authority,
        constructor_id: admitted.authority_constructor,
    };

    let console_ids =
        ken_interp::ConsoleIds::from_elab(&elab_env).ok_or(RunError::ConsoleAbiUnavailable)?;
    let fs_ids = ken_interp::FSIds::from_elab(&elab_env);
    let clock_ids = ken_interp::ClockIds::from_elab(&elab_env);
    let cap = ken_interp::EvalVal::Cap(
        host.mint_fs_cap_for_root(declared_fs.authority, &admitted.fs_root_spec)
            .map_err(RunError::CapabilityRoot)?,
    );
    let mut store = build_eval_store(&elab_env);
    let tree = apply_entrypoint(
        &elab_env,
        main_id,
        arguments,
        environment,
        cwd,
        declared_fs,
        cap,
        &mut store,
    );
    let coproduct_ids = ken_interp::CoproductIds {
        inl_id: elab_env.prelude_env.inl_id,
        inr_id: elab_env.prelude_env.inr_id,
    };
    let success_id = get("Success").expect("Success registered");
    let failure_id = get("Failure").expect("Failure registered");
    if observe_effects {
        let observation = ken_interp::run_io_effect_observation_v1(
            tree,
            host,
            &console_ids,
            fs_ids.as_ref(),
            clock_ids.as_ref(),
            Some(&coproduct_ids),
            &elab_env.env,
            &mut store,
            success_id,
            failure_id,
        );
        Ok((
            ProgramOutcome {
                exit_status: observation.exit_status,
            },
            Some(observation),
        ))
    } else {
        let final_value = ken_interp::run_io(
            tree,
            host,
            &console_ids,
            fs_ids.as_ref(),
            clock_ids.as_ref(),
            Some(&coproduct_ids),
            &elab_env.env,
            &mut store,
        )
        .map_err(RunError::Io)?;
        Ok((
            ProgramOutcome {
                exit_status: exit_status(&final_value, success_id, failure_id),
            },
            None,
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DeclaredFsAuthority {
    name: &'static str,
    authority: ken_elaborator::capabilities::Authority,
    constructor_id: ken_kernel::GlobalId,
}

fn map_program_admission_error(
    error: ken_elaborator::program_admission::ProgramAdmissionError,
) -> RunError {
    use ken_elaborator::program_admission::ProgramAdmissionError;
    match error {
        ProgramAdmissionError::MissingMain => RunError::MissingEntrypoint,
        ProgramAdmissionError::MissingProgramBoundary => RunError::EntrypointAbiUnavailable,
        ProgramAdmissionError::MissingAbiDeclaration { .. } => RunError::EntrypointAbiUnavailable,
        ProgramAdmissionError::MissingCapability { effect }
        | ProgramAdmissionError::DuplicateCapability { effect } => RunError::MissingCapability {
            effect: effect.to_string(),
        },
        ProgramAdmissionError::UnsupportedEffectRow => RunError::InvalidEntrypoint {
            authority: "declared",
        },
        ProgramAdmissionError::InvalidMainAbi { authority } => RunError::InvalidEntrypoint {
            authority: match authority.as_str() {
                "ANone" => "ANone",
                "APartial" => "APartial",
                "AFull" => "AFull",
                _ => "declared",
            },
        },
    }
}

fn constructor_value(
    id: ken_kernel::GlobalId,
    args: Vec<ken_interp::EvalVal>,
) -> ken_interp::EvalVal {
    ken_interp::EvalVal::Ctor {
        id,
        args: Rc::new(args),
        slot: 0,
    }
}

fn apply_entrypoint(
    elab_env: &ken_elaborator::ElabEnv,
    main_id: ken_kernel::GlobalId,
    arguments: &[Vec<u8>],
    environment: &[(Vec<u8>, Vec<u8>)],
    cwd: &[u8],
    declared_fs: DeclaredFsAuthority,
    cap: ken_interp::EvalVal,
    store: &mut ken_interp::EvalStore,
) -> ken_interp::EvalVal {
    let get = |name: &str| {
        *elab_env
            .globals
            .get(name)
            .unwrap_or_else(|| panic!("entrypoint ABI global '{name}' missing"))
    };
    let mut tree = ken_interp::eval(
        &[],
        &ken_kernel::Term::const_(main_id, vec![]),
        &elab_env.env,
        store,
    );
    tree = ken_interp::apply(
        tree,
        process_input_value(elab_env, arguments, environment, cwd),
        &elab_env.env,
        store,
    );

    // SECURITY: the FS authority comes only from the parsed root declaration.
    // AFull remains coarse and is not path-confined until CA4/I-5. Console is
    // ambient process context (stdio), so ProgramCaps deliberately has no
    // Console field and the runner mints no Console capability.
    let mut caps = ken_interp::eval(
        &[],
        &ken_kernel::Term::constructor(get("MkProgramCaps"), vec![]),
        &elab_env.env,
        store,
    );
    caps = ken_interp::apply(
        caps,
        constructor_value(declared_fs.constructor_id, vec![]),
        &elab_env.env,
        store,
    );
    caps = ken_interp::apply(caps, cap, &elab_env.env, store);
    ken_interp::apply(tree, caps, &elab_env.env, store)
}

fn list_value(
    nil_id: ken_kernel::GlobalId,
    cons_id: ken_kernel::GlobalId,
    values: impl IntoIterator<Item = ken_interp::EvalVal>,
) -> ken_interp::EvalVal {
    let values: Vec<_> = values.into_iter().collect();
    values.into_iter().rev().fold(
        constructor_value(nil_id, vec![ken_interp::EvalVal::Unknown]),
        |tail, head| constructor_value(cons_id, vec![ken_interp::EvalVal::Unknown, head, tail]),
    )
}

fn process_input_value(
    elab_env: &ken_elaborator::ElabEnv,
    arguments: &[Vec<u8>],
    environment: &[(Vec<u8>, Vec<u8>)],
    cwd: &[u8],
) -> ken_interp::EvalVal {
    let get = |name: &str| {
        elab_env
            .globals
            .get(name)
            .copied()
            .unwrap_or_else(|| panic!("entrypoint ABI global '{name}' missing"))
    };
    let arguments = list_value(
        elab_env.prelude_env.nil_id,
        elab_env.prelude_env.cons_id,
        arguments.iter().cloned().map(ken_interp::EvalVal::Bytes),
    );
    let environment = environment.iter().cloned().map(|(key, value)| {
        constructor_value(
            elab_env.prelude_env.mkprod_id,
            vec![
                ken_interp::EvalVal::Unknown,
                ken_interp::EvalVal::Unknown,
                ken_interp::EvalVal::Bytes(key),
                ken_interp::EvalVal::Bytes(value),
            ],
        )
    });
    let environment = list_value(
        elab_env.prelude_env.nil_id,
        elab_env.prelude_env.cons_id,
        environment,
    );
    constructor_value(
        get("MkProcessInput"),
        vec![
            arguments,
            environment,
            ken_interp::EvalVal::Bytes(cwd.to_vec()),
        ],
    )
}

fn exit_status(
    value: &ken_interp::EvalVal,
    success_id: ken_kernel::GlobalId,
    failure_id: ken_kernel::GlobalId,
) -> i32 {
    let exit_code = match value {
        ken_interp::EvalVal::Ctor { id, .. } if *id == success_id => {
            ken_runtime::ProcessExitCode::Success
        }
        ken_interp::EvalVal::Ctor { id, args, .. } if *id == failure_id => match args.first() {
            Some(ken_interp::EvalVal::Int(code)) => ken_runtime::ProcessExitCode::Failure(*code),
            _ => ken_runtime::ProcessExitCode::MalformedFailure,
        },
        _ => ken_runtime::ProcessExitCode::Malformed,
    };
    let mapped = ken_runtime::process_exit_status(exit_code);
    if let Some(report) = mapped.trap_report {
        eprintln!("ken run: {report}");
    }
    mapped.status
}

fn lit_to_eval(
    v: &ken_elaborator::NumericLitVal,
    mkdecimalpair_id: ken_kernel::GlobalId,
) -> ken_interp::EvalVal {
    use ken_elaborator::NumericLitVal;
    match v {
        NumericLitVal::Int(n) => ken_interp::EvalVal::from(*n),
        NumericLitVal::Float(f) => ken_interp::EvalVal::Float(*f),
        NumericLitVal::Float32(f) => ken_interp::EvalVal::Float32(*f),
        NumericLitVal::Decimal { coeff, exp } => {
            ken_interp::decimal_value(mkdecimalpair_id, *coeff, *exp)
        }
        NumericLitVal::Str(s) => ken_interp::EvalVal::Str(s.clone()),
    }
}

/// Build an `EvalStore` pre-wired with everything `ken-interp` needs beyond
/// the eliminator/reduction machinery itself: the elaborator's numeric-
/// literal map and the prelude's List-constructor ids
/// (`string_to_list_char`/`list_char_to_string`). Every acceptance test wires
/// these fields by hand; a production entry point (`run_file`, the REPL's
/// `Session`) that forgets one doesn't crash — the affected op just degrades
/// to `Neutral` (`ken-interp`'s "never silently wrong" default) — so the gap
/// is easy to miss. This is the second such gap VAL2 surfaced (`console_ids`
/// → console-harvest-fix; now `list_char_ids` → this WP); one shared builder
/// for every production call site means a third forgotten field can't recur
/// (subsume-don't-proliferate, `docs/PRINCIPLES.md`). Console IDs are
/// deliberately NOT included here — they're `run_file`-specific harvested
/// state with their own "Language layer pending" failure mode, not a plain
/// store field every caller needs.
#[doc(hidden)]
pub fn build_eval_store(elab_env: &ken_elaborator::ElabEnv) -> ken_interp::EvalStore {
    let mut store = ken_interp::EvalStore::new();
    let mkdecimalpair_id = elab_env.prelude_env.mkdecimalpair_id;
    for (id, lit) in &elab_env.num_values {
        let val = lit_to_eval(lit, mkdecimalpair_id);
        store.num_values.insert(*id, val);
    }
    store.list_char_ids = Some(ken_interp::eval::ListCharIds {
        nil_id: elab_env.prelude_env.nil_id,
        cons_id: elab_env.prelude_env.cons_id,
    });
    store
}

#[cfg(test)]
mod entrypoint_tests {
    use super::*;

    fn elaborate_program(source: &str) -> ken_elaborator::ElabEnv {
        let mut env = ken_elaborator::ElabEnv::new().expect("prelude");
        env.elaborate_file(source).expect("program elaborates");
        env
    }

    fn coproduct_ids(env: &ken_elaborator::ElabEnv) -> ken_interp::CoproductIds {
        ken_interp::CoproductIds {
            inl_id: env.prelude_env.inl_id,
            inr_id: env.prelude_env.inr_id,
        }
    }

    #[test]
    fn declared_afull_mints_runner_caps_and_writes_through_capture_host() {
        let env = elaborate_program(
            r#"program capabilities FS AFull
proc main (_input : ProcessInput) (caps : ProgramCaps AFull)
  : HostIO AFull ExitCode visits [FS] =
  match caps {
    MkProgramCaps cap |->
      bind (Coproduct (FSOp AFull) AmbientOp)
           (resp_coproduct (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp)
           (Result FileError Unit) ExitCode
        (inject_l (FSOp AFull) AmbientOp (fs_resp AFull) ambient_resp
          (Result FileError Unit)
          (writeFile cap (bytes_encode "root/out") CreateNew (bytes_encode "lit")))
        (\_. host_exit AFull Success)
  }"#,
        );
        let admitted = ken_elaborator::program_admission::admit_checked_main(&env)
            .expect("checked main admitted");
        let declared = DeclaredFsAuthority {
            name: "AFull",
            authority: admitted.authority,
            constructor_id: admitted.authority_constructor,
        };
        let main_id = *env.globals.get("main").expect("main");
        assert_eq!(admitted.main, main_id);

        let console = ken_interp::ConsoleIds::from_elab(&env).expect("Console ABI");
        let fs = ken_interp::FSIds::from_elab(&env).expect("FS ABI");
        let mut host = ken_interp::CaptureHost::new(Vec::new());
        host.insert_directory(b"root".to_vec());
        let cap = ken_interp::EvalVal::Cap(host.mint_fs_cap(declared.authority));
        let mut store = build_eval_store(&env);
        let tree = apply_entrypoint(&env, main_id, &[], &[], b"", declared, cap, &mut store);
        let result = ken_interp::run_io(
            tree,
            &mut host,
            &console,
            Some(&fs),
            None,
            Some(&coproduct_ids(&env)),
            &env.env,
            &mut store,
        )
        .expect("declared AFull write runs");
        assert!(matches!(
            result,
            ken_interp::EvalVal::Ctor { id, .. }
                if id == *env.globals.get("Success").expect("Success")
        ));
        assert_eq!(
            host.fs_trace(),
            &[ken_interp::FsTrace::WriteFile {
                path: b"root/out".to_vec(),
                policy: ken_interp::HostCreatePolicy::CreateNew,
                bytes: b"lit".to_vec(),
            }]
        );
    }

    #[test]
    fn fs_effect_without_declared_family_is_named_static_rejection() {
        let mut env = ken_elaborator::ElabEnv::new().expect("prelude");
        let error = env
            .elaborate_file(
                r#"program
proc main (_input : ProcessInput) (caps : ProgramCaps APartial)
  : HostIO APartial ExitCode visits [FS] =
  match caps {
    MkProgramCaps cap |->
      bind (Coproduct (FSOp APartial) AmbientOp)
           (resp_coproduct (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp)
           (Result FileError Bytes) ExitCode
        (inject_l (FSOp APartial) AmbientOp (fs_resp APartial) ambient_resp
          (Result FileError Bytes)
          (readFile APartial cap (bytes_encode "missing")))
        (\_. host_exit APartial Success)
  }"#,
            )
            .expect_err("an FS effect requires a declared FS capability");
        match error {
            ken_elaborator::ElabError::MissingCapability { effect, .. } => {
                assert_eq!(effect, "FS")
            }
            other => panic!("expected MissingCapability {{ effect = FS }}, got {other:?}"),
        }
    }

    #[test]
    fn runner_has_no_implicit_fs_authority_default() {
        let env = elaborate_program("program\n");
        assert!(
            env.boundary_header()
                .and_then(|header| header.capabilities.as_ref())
                .is_none_or(|caps| caps.iter().all(|cap| cap.family != "FS"))
        );
    }

    #[test]
    fn apartial_write_returns_named_denial_before_capture_host_syscall() {
        let env = elaborate_program("program capabilities FS APartial\n");
        let console = ken_interp::ConsoleIds::from_elab(&env).expect("Console ABI");
        let fs = ken_interp::FSIds::from_elab(&env).expect("FS ABI");
        let mut host = ken_interp::CaptureHost::new(Vec::new());
        host.insert_directory(b"root".to_vec());
        let cap =
            ken_interp::EvalVal::Cap(host.mint_fs_cap(ken_elaborator::capabilities::AUTH_PARTIAL));
        let mut store = build_eval_store(&env);
        let mut tree = ken_interp::eval(
            &[],
            &ken_kernel::Term::const_(
                *env.globals.get("write_file").expect("raw I-3 producer"),
                vec![],
            ),
            &env.env,
            &mut store,
        );
        for argument in [
            constructor_value(env.globals["APartial"], vec![]),
            cap,
            ken_interp::EvalVal::Bytes(b"root/denied".to_vec()),
            constructor_value(fs.create_new_id, vec![]),
            ken_interp::EvalVal::Bytes(b"no".to_vec()),
        ] {
            tree = ken_interp::apply(tree, argument, &env.env, &mut store);
        }
        let result = ken_interp::run_io(
            tree,
            &mut host,
            &console,
            Some(&fs),
            None,
            None,
            &env.env,
            &mut store,
        )
        .expect("denial is a total FileError result");
        let kind = match result {
            ken_interp::EvalVal::Ctor { id, args, .. } if id == console.err_id => {
                match args.get(2) {
                    Some(ken_interp::EvalVal::Ctor { id, args, .. })
                        if *id == fs.mk_file_error_id =>
                    {
                        match args.get(2) {
                            Some(ken_interp::EvalVal::Ctor { id, .. }) => *id,
                            other => panic!("FileError kind missing: {other:?}"),
                        }
                    }
                    other => panic!("expected FileError payload: {other:?}"),
                }
            }
            other => panic!("expected Err(FileError): {other:?}"),
        };
        assert_eq!(kind, console.capabilitydenied_id);
        assert!(host.fs_trace().is_empty(), "denial must precede syscall");
        assert!(!host.fs_nodes().contains_key(b"root/denied".as_slice()));
    }

    #[test]
    fn anone_readfile_reaches_named_denial_before_capture_host_syscall() {
        let env = elaborate_program("program capabilities FS ANone\n");
        let console = ken_interp::ConsoleIds::from_elab(&env).expect("Console ABI");
        let fs = ken_interp::FSIds::from_elab(&env).expect("FS ABI");
        let mut host = ken_interp::CaptureHost::new(Vec::new());
        host.insert_directory(b"root".to_vec());
        host.insert_file(b"root/input".to_vec(), b"secret".to_vec());
        let cap =
            ken_interp::EvalVal::Cap(host.mint_fs_cap(ken_elaborator::capabilities::AUTH_NONE));
        let mut store = build_eval_store(&env);
        let mut tree = ken_interp::eval(
            &[],
            &ken_kernel::Term::const_(
                *env.globals.get("readFile").expect("typed read wrapper"),
                vec![],
            ),
            &env.env,
            &mut store,
        );
        for argument in [
            constructor_value(env.globals["ANone"], vec![]),
            cap,
            ken_interp::EvalVal::Bytes(b"root/input".to_vec()),
        ] {
            tree = ken_interp::apply(tree, argument, &env.env, &mut store);
        }
        let result = ken_interp::run_io(
            tree,
            &mut host,
            &console,
            Some(&fs),
            None,
            None,
            &env.env,
            &mut store,
        )
        .expect("ANone denial is a total FileError result");
        let kind = match result {
            ken_interp::EvalVal::Ctor { id, args, .. } if id == console.err_id => {
                match args.get(2) {
                    Some(ken_interp::EvalVal::Ctor { id, args, .. })
                        if *id == fs.mk_file_error_id =>
                    {
                        match args.get(2) {
                            Some(ken_interp::EvalVal::Ctor { id, .. }) => *id,
                            other => panic!("FileError kind missing: {other:?}"),
                        }
                    }
                    other => panic!("expected FileError payload: {other:?}"),
                }
            }
            other => panic!("expected Err(FileError): {other:?}"),
        };
        assert_eq!(kind, console.capabilitydenied_id);
        assert!(host.fs_trace().is_empty(), "denial must precede syscall");
    }

    #[test]
    fn apartial_cannot_apply_afull_writefile_wrapper() {
        let mut env = ken_elaborator::ElabEnv::new().expect("prelude");
        let error = env
            .elaborate_file(
                r#"proc rejected (cap : Cap APartial)
  : FS AFull (Result FileError Unit) visits [FS] =
  writeFile cap (bytes_encode "out") CreateNew (bytes_encode "no")"#,
            )
            .expect_err("APartial must not satisfy writeFile's AFull argument");
        assert!(matches!(
            error,
            ken_elaborator::ElabError::KernelRejected {
                error: ken_kernel::KernelError::TypeMismatch { .. },
                ..
            }
        ));
    }

    #[test]
    fn exit_code_mapping_is_total_and_failure_zero_fails_closed() {
        let success = ken_kernel::GlobalId(1);
        let failure = ken_kernel::GlobalId(2);
        assert_eq!(
            exit_status(&constructor_value(success, vec![]), success, failure),
            0
        );
        for (payload, expected) in [(3, 3), (255, 255), (0, 1)] {
            assert_eq!(
                exit_status(
                    &constructor_value(failure, vec![ken_interp::EvalVal::Int(payload)]),
                    success,
                    failure,
                ),
                expected
            );
        }
    }

    fn root_program(allow_root: bool) -> String {
        let marker = if allow_root {
            ", RootExecution Allow"
        } else {
            ""
        };
        format!(
            r#"program capabilities FS APartial{marker}
proc main (_input : ProcessInput) (_caps : ProgramCaps APartial)
  : HostIO APartial ExitCode visits [Console] =
  bind (Coproduct (FSOp APartial) AmbientOp)
       (resp_coproduct (FSOp APartial) AmbientOp
         (fs_resp APartial) ambient_resp)
       (Result IOError Unit) ExitCode
       (host_console APartial (Result IOError Unit)
         (write Stdout (bytes_encode "admitted")))
       (\_ . host_exit APartial Success)
"#
        )
    }

    fn run_with_scripted_uid(
        effective_uid: u32,
        allow_root: bool,
        arguments: &[Vec<u8>],
        environment: &[(Vec<u8>, Vec<u8>)],
    ) -> (ken_runtime::EffectObservationV1, ken_interp::CaptureHost) {
        let mut host = ken_interp::CaptureHost::new(Vec::new());
        let (_, observation) = run_program_inner(
            &root_program(allow_root),
            SourceFormat::Ken,
            arguments,
            environment,
            b".",
            &mut host,
            true,
            ken_runtime::EffectiveUidSnapshotV1::scripted(effective_uid),
        )
        .expect("checked program reaches the real interpreter runner");
        (observation.expect("observation requested"), host)
    }

    #[test]
    fn root_admission_precedes_host_leaves_and_marker_flips_the_same_program() {
        let (denied, denied_host) = run_with_scripted_uid(0, false, &[], &[]);
        assert_eq!(denied.exit_status, 1);
        assert_eq!(
            denied.terminal_error,
            Some(ken_runtime::TerminalErrorV1::RootExecutionDenied)
        );
        assert!(denied.stdout.is_empty());
        assert!(denied.stderr.is_empty());
        assert!(denied.filesystem_delta.is_empty());
        assert!(denied.effect_trace.is_empty());
        assert!(denied_host.trace().is_empty());
        assert!(denied_host.fs_trace().is_empty());
        assert!(denied_host.clock_trace().is_empty());

        let (allowed, allowed_host) = run_with_scripted_uid(0, true, &[], &[]);
        assert_eq!(allowed.exit_status, 0);
        assert_eq!(allowed.terminal_error, None);
        assert_eq!(allowed.stdout, b"admitted");
        assert_eq!(allowed.effect_trace.len(), 1);
        assert_eq!(allowed_host.trace().len(), 1);
    }

    #[test]
    fn non_root_proceeds_and_process_input_cannot_forge_the_marker() {
        let (non_root, non_root_host) = run_with_scripted_uid(1000, false, &[], &[]);
        assert_eq!(non_root.exit_status, 0);
        assert_eq!(non_root.stdout, b"admitted");
        assert_eq!(non_root_host.trace().len(), 1);

        let arguments = vec![b"--allow-root".to_vec()];
        let environment = vec![(b"KEN_ALLOW_ROOT_EXECUTION".to_vec(), b"1".to_vec())];
        let (denied, denied_host) = run_with_scripted_uid(0, false, &arguments, &environment);
        assert_eq!(
            denied.terminal_error,
            Some(ken_runtime::TerminalErrorV1::RootExecutionDenied)
        );
        assert!(denied.effect_trace.is_empty());
        assert!(denied_host.trace().is_empty());
    }

    #[test]
    fn checked_admission_projects_root_marker_without_changing_program_caps() {
        let denied = elaborate_program(&root_program(false));
        let allowed = elaborate_program(&root_program(true));
        assert!(
            !ken_elaborator::program_admission::admit_checked_main(&denied)
                .unwrap()
                .allow_root_execution
        );
        assert!(
            ken_elaborator::program_admission::admit_checked_main(&allowed)
                .unwrap()
                .allow_root_execution
        );
        assert_eq!(
            denied
                .env
                .lookup(*denied.globals.get("ProgramCaps").unwrap()),
            allowed
                .env
                .lookup(*allowed.globals.get("ProgramCaps").unwrap())
        );
    }

    #[test]
    fn missing_and_duplicate_main_have_distinct_specific_errors() {
        let env = ken_elaborator::ElabEnv::new().expect("env");
        assert!(matches!(
            ken_elaborator::program_admission::admit_checked_main(&env),
            Err(ken_elaborator::program_admission::ProgramAdmissionError::MissingMain)
        ));

        let mut env = ken_elaborator::ElabEnv::new().expect("env");
        let error = env
            .elaborate_file("const main : Nat = Zero\nconst main : Nat = Zero")
            .expect_err("duplicate main must fail");
        assert!(matches!(
            error,
            ken_elaborator::ElabError::DuplicateDefinition { ref name, .. }
                if name == "main"
        ));
    }
}
