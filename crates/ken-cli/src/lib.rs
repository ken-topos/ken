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

    let main_id = resolve_main(&elab_env).map_err(|_| RunError::MissingEntrypoint)?;
    let get = |name: &str| -> Option<ken_kernel::GlobalId> { elab_env.globals.get(name).copied() };
    let (process_input_id, program_caps_id, host_io_id, exit_code_id) = match (
        get("ProcessInput"),
        get("ProgramCaps"),
        get("HostIO"),
        get("ExitCode"),
    ) {
        (Some(a), Some(b), Some(c), Some(d)) => (a, b, c, d),
        _ => return Err(RunError::EntrypointAbiUnavailable),
    };
    let declared_fs = declared_fs_authority(&elab_env).map_err(|error| match error {
        ProgramValidationError::MissingCapability { effect } => {
            RunError::MissingCapability { effect }
        }
    })?;
    if !entrypoint_has_abi(
        &elab_env,
        main_id,
        process_input_id,
        program_caps_id,
        host_io_id,
        exit_code_id,
        declared_fs.constructor_id,
    ) {
        return Err(RunError::InvalidEntrypoint {
            authority: declared_fs.name,
        });
    }

    let console_ids =
        ken_interp::ConsoleIds::from_elab(&elab_env).ok_or(RunError::ConsoleAbiUnavailable)?;
    let fs_ids = ken_interp::FSIds::from_elab(&elab_env);
    let clock_ids = ken_interp::ClockIds::from_elab(&elab_env);
    let cap = ken_interp::EvalVal::Cap(host.mint_fs_cap(declared_fs.authority));
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
    Ok(ProgramOutcome {
        exit_status: exit_status(
            &final_value,
            get("Success").expect("Success registered"),
            get("Failure").expect("Failure registered"),
        ),
    })
}

#[derive(Debug, PartialEq, Eq)]
enum EntrypointResolutionError {
    MissingMain,
}

#[derive(Debug, PartialEq, Eq)]
enum ProgramValidationError {
    MissingCapability { effect: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DeclaredFsAuthority {
    name: &'static str,
    authority: ken_elaborator::capabilities::Authority,
    constructor_id: ken_kernel::GlobalId,
}

fn declared_fs_authority(
    elab_env: &ken_elaborator::ElabEnv,
) -> Result<DeclaredFsAuthority, ProgramValidationError> {
    use ken_elaborator::capabilities::{AUTH_FULL, AUTH_NONE, AUTH_PARTIAL};

    let declaration = elab_env
        .boundary_header()
        .and_then(|header| header.capabilities.as_ref())
        .and_then(|capabilities| capabilities.iter().find(|cap| cap.family == "FS"))
        .ok_or_else(|| ProgramValidationError::MissingCapability {
            effect: "FS".to_owned(),
        })?;
    let (name, authority) = match declaration.authority.as_str() {
        "ANone" => ("ANone", AUTH_NONE),
        "APartial" => ("APartial", AUTH_PARTIAL),
        "AFull" => ("AFull", AUTH_FULL),
        authority => unreachable!("parser admitted invalid FS authority {authority}"),
    };
    let constructor_id = *elab_env
        .globals
        .get(name)
        .expect("parsed Auth constructor registered by the prelude");
    Ok(DeclaredFsAuthority {
        name,
        authority,
        constructor_id,
    })
}

fn resolve_main(
    elab_env: &ken_elaborator::ElabEnv,
) -> Result<ken_kernel::GlobalId, EntrypointResolutionError> {
    elab_env
        .globals
        .get("main")
        .copied()
        .ok_or(EntrypointResolutionError::MissingMain)
}

fn entrypoint_has_abi(
    elab_env: &ken_elaborator::ElabEnv,
    main_id: ken_kernel::GlobalId,
    process_input_id: ken_kernel::GlobalId,
    program_caps_id: ken_kernel::GlobalId,
    host_io_id: ken_kernel::GlobalId,
    exit_code_id: ken_kernel::GlobalId,
    authority_id: ken_kernel::GlobalId,
) -> bool {
    use ken_kernel::{Decl, Term};

    if let Some(row) = elab_env.effect_rows.get("main") {
        let granted = ken_elaborator::effects::EffectRow::from_effects([
            "Console".to_string(),
            "Clock".to_string(),
            "FS".to_string(),
        ]);
        if !row.row_vars().is_empty() || !row.concrete_effects().is_subset_of(&granted) {
            return false;
        }
    }

    let actual = match elab_env.env.lookup(main_id) {
        Some(Decl::Transparent { ty, .. })
        | Some(Decl::Opaque { ty, .. })
        | Some(Decl::Primitive { ty, .. }) => ty,
        _ => return false,
    };
    let process_input = Term::indformer(process_input_id, vec![]);
    let authority = Term::constructor(authority_id, vec![]);
    let program_caps = Term::app(Term::indformer(program_caps_id, vec![]), authority.clone());
    let exit_code = Term::indformer(exit_code_id, vec![]);
    let host_exit = Term::app(
        Term::app(Term::const_(host_io_id, vec![]), authority),
        exit_code,
    );
    let expected = Term::pi(process_input, Term::pi(program_caps, host_exit));
    ken_kernel::convert_type(
        &elab_env.env,
        &ken_kernel::Context::new(),
        actual,
        &expected,
    )
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
        let declared = declared_fs_authority(&env).expect("FS AFull declared");
        assert_eq!(declared.name, "AFull");
        let main_id = *env.globals.get("main").expect("main");
        assert!(entrypoint_has_abi(
            &env,
            main_id,
            *env.globals.get("ProcessInput").expect("ProcessInput"),
            *env.globals.get("ProgramCaps").expect("ProgramCaps"),
            *env.globals.get("HostIO").expect("HostIO"),
            *env.globals.get("ExitCode").expect("ExitCode"),
            declared.constructor_id,
        ));

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
        assert_eq!(
            declared_fs_authority(&env),
            Err(ProgramValidationError::MissingCapability {
                effect: "FS".to_owned(),
            })
        );
    }

    #[test]
    fn apartial_write_returns_named_denial_before_capture_host_syscall() {
        let env = elaborate_program("program capabilities FS APartial\n");
        let declared = declared_fs_authority(&env).expect("FS APartial declared");
        let console = ken_interp::ConsoleIds::from_elab(&env).expect("Console ABI");
        let fs = ken_interp::FSIds::from_elab(&env).expect("FS ABI");
        let mut host = ken_interp::CaptureHost::new(Vec::new());
        host.insert_directory(b"root".to_vec());
        let cap = ken_interp::EvalVal::Cap(host.mint_fs_cap(declared.authority));
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
            constructor_value(declared.constructor_id, vec![]),
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
        let declared = declared_fs_authority(&env).expect("FS ANone declared");
        let console = ken_interp::ConsoleIds::from_elab(&env).expect("Console ABI");
        let fs = ken_interp::FSIds::from_elab(&env).expect("FS ABI");
        let mut host = ken_interp::CaptureHost::new(Vec::new());
        host.insert_directory(b"root".to_vec());
        host.insert_file(b"root/input".to_vec(), b"secret".to_vec());
        let cap = ken_interp::EvalVal::Cap(host.mint_fs_cap(declared.authority));
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
            constructor_value(declared.constructor_id, vec![]),
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

    #[test]
    fn missing_and_duplicate_main_have_distinct_specific_errors() {
        let env = ken_elaborator::ElabEnv::new().expect("env");
        assert_eq!(
            resolve_main(&env),
            Err(EntrypointResolutionError::MissingMain)
        );

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
