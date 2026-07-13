//! The `ken` command-line driver.

mod repl;

use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::rc::Rc;

fn main() {
    let args: Vec<OsString> = std::env::args_os().collect();
    match args.get(1).and_then(|s| s.to_str()).unwrap_or("") {
        "repl" => repl::run(),
        "run" => match parse_run_invocation(&args[2..]) {
            Ok(invocation) => run_file(invocation.path.as_os_str(), &invocation.arguments),
            Err(RunArgumentError::MissingPath) => {
                eprintln!("ken run: missing <file> argument");
                eprintln!("Usage: ken run <file.ken> [-- <arguments>...]");
                std::process::exit(1);
            }
            Err(RunArgumentError::UnexpectedBeforeSeparator(argument)) => {
                eprintln!("ken run: unexpected argument before '--': {:?}", argument);
                std::process::exit(1);
            }
        },
        "check" => check_file(args.get(2).map(OsString::as_os_str)),
        "fmt" => format_files(&args[2..]),
        "version" | "--version" | "-V" => {
            println!(
                "ken {} — verified topos-oriented language",
                env!("CARGO_PKG_VERSION")
            );
            println!("kernel {}", ken_kernel::version());
            println!("{}", ken_interp::describe());
        }
        "" | "--help" | "-h" | "help" => print_help(),
        unknown => {
            eprintln!("ken: unknown subcommand '{}' — try 'ken help'", unknown);
            std::process::exit(1);
        }
    }
}

/// `ken fmt [--check] <paths...>` — the thin CLI over the landed formatter.
fn format_files(args: &[OsString]) {
    let mut check = false;
    let mut paths = Vec::new();
    for arg in args {
        let Some(arg) = arg.to_str() else {
            eprintln!("ken fmt: path is not valid UTF-8: {:?}", arg);
            std::process::exit(1);
        };
        if arg == "--check" {
            check = true;
        } else if arg.starts_with('-') {
            eprintln!("ken fmt: unknown option '{arg}'");
            std::process::exit(1);
        } else {
            paths.push(arg);
        }
    }
    if paths.is_empty() {
        eprintln!("ken fmt: missing <paths...> argument");
        eprintln!("Usage: ken fmt [--check] <paths...>");
        std::process::exit(1);
    }

    let mut failed = false;
    for path in paths {
        let source = match std::fs::read_to_string(path) {
            Ok(source) => source,
            Err(error) => {
                eprintln!("ken fmt: cannot read '{path}': {error}");
                failed = true;
                continue;
            }
        };
        let formatted = if path.ends_with(".ken.md") {
            ken_elaborator::format_ken_md(&source)
        } else if path.ends_with(".ken") {
            ken_elaborator::layout::format_ken(&source)
        } else {
            eprintln!("ken fmt: unsupported path '{path}' (expected .ken or .ken.md)");
            failed = true;
            continue;
        };
        let formatted = match formatted {
            Ok(formatted) => formatted,
            Err(error) => {
                eprintln!("ken fmt: formatting error in '{path}': {error:?}");
                failed = true;
                continue;
            }
        };

        if check {
            if formatted != source {
                eprintln!("ken fmt --check: non-canonical: {path}");
                failed = true;
            }
        } else if formatted != source {
            if let Err(error) = std::fs::write(path, formatted) {
                eprintln!("ken fmt: cannot write '{path}': {error}");
                failed = true;
            }
        }
    }

    if failed {
        std::process::exit(1);
    }
}

#[derive(Debug, PartialEq, Eq)]
enum RunArgumentError {
    MissingPath,
    UnexpectedBeforeSeparator(OsString),
}

struct RunInvocation {
    path: PathBuf,
    arguments: Vec<Vec<u8>>,
}

fn parse_run_invocation(args: &[OsString]) -> Result<RunInvocation, RunArgumentError> {
    let Some(path) = args.first() else {
        return Err(RunArgumentError::MissingPath);
    };
    let rest = &args[1..];
    let program_args = match rest.first() {
        None => &[][..],
        Some(separator) if separator == "--" => &rest[1..],
        Some(unexpected) => {
            return Err(RunArgumentError::UnexpectedBeforeSeparator(
                unexpected.clone(),
            ));
        }
    };
    Ok(RunInvocation {
        path: PathBuf::from(path),
        arguments: program_args.iter().map(|arg| os_bytes(arg)).collect(),
    })
}

#[cfg(unix)]
fn os_bytes(value: &OsStr) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt;
    value.as_bytes().to_vec()
}

#[cfg(not(unix))]
fn os_bytes(value: &OsStr) -> Vec<u8> {
    value.to_string_lossy().into_owned().into_bytes()
}

/// Read `<file>` and elaborate it (`` .ken.md `` via the literate path,
/// otherwise the plain `.ken` path) — the shared front half of both `ken run`
/// and `ken check`. Exits 1 on a missing argument, an unreadable file,
/// elaborator init failure, or an elaboration error, with a message prefixed
/// by `cmd` (`"run"`/`"check"`) so a user sees the subcommand they actually
/// typed, not a borrowed one.
fn elaborate_cli_file(
    cmd: &str,
    path: Option<&OsStr>,
) -> (PathBuf, ken_elaborator::ElabEnv, Vec<ken_kernel::GlobalId>) {
    let path = match path {
        Some(p) => p,
        None => {
            eprintln!("ken {cmd}: missing <file> argument");
            eprintln!("Usage: ken {cmd} <file.ken>");
            std::process::exit(1);
        }
    };

    let src = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ken {cmd}: cannot read '{}': {}", path.to_string_lossy(), e);
            std::process::exit(1);
        }
    };

    let mut elab_env = match ken_elaborator::ElabEnv::new() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("ken {cmd}: elaborator init failed: {:?}", e);
            std::process::exit(1);
        }
    };

    let ids_result = if path.to_string_lossy().ends_with(".ken.md") {
        elab_env.elaborate_ken_md_file(&src)
    } else {
        elab_env.elaborate_file(&src)
    };

    let ids = match ids_result {
        Ok(ids) => ids,
        Err(ken_elaborator::ElabError::DuplicateDefinition { name, .. })
            if cmd == "run" && name == "main" =>
        {
            eprintln!("ken run: duplicate entrypoint 'main'");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!(
                "ken {cmd}: elaboration error in '{}': {:?}",
                path.to_string_lossy(),
                e
            );
            std::process::exit(1);
        }
    };

    (PathBuf::from(path), elab_env, ids)
}

/// `ken check <file>` — FR-3 (`docs/program/wp/ds-1-findings-remediation.md`):
/// a library check-mode for pure-library catalog entries, which have no
/// natural IO `main` and so cannot satisfy `ken run`'s literal exit-0
/// contract. Runs the identical `elaborate_ken_md_file`/`elaborate_file` path
/// `ken run` calls before its own separate IO-execution step, then stops
/// before the IO-drive — no new checking logic, the fence-role verdicts
/// (`ken reject` must fail, `ken example` must elaborate) are already that
/// call's job. Exits 0 iff elaboration + every fence behaved; inherits the
/// shared front half's `Err -> exit 1` verbatim. Never drives IO, so a
/// runnable program's `main` is simply never executed here (`ken run` is
/// still how you run it) — `ken run` itself is unchanged, strict, and has no
/// auto-detect fallthrough to this mode.
fn check_file(path: Option<&OsStr>) {
    elaborate_cli_file("check", path);
}

/// `ken run <file>` — elaborate, evaluate, and drive a Console IO program.
///
/// Elaborates every declaration in `<file>`, resolves the ABI-shaped `main` by
/// name, supplies process input and capabilities, and drives its host tree.
///
/// Console IDs are harvested from the elaboration environment (`ElabEnv::globals`).
fn run_file(path: &OsStr, arguments: &[Vec<u8>]) {
    let (path, elab_env, _ids) = elaborate_cli_file("run", Some(path));
    let main_id = match resolve_main(&elab_env) {
        Ok(id) => id,
        Err(EntrypointResolutionError::MissingMain) => {
            eprintln!("ken run: missing entrypoint 'main' in '{}'", path.display());
            std::process::exit(1);
        }
    };

    // Harvest Console IDs from the elaboration environment.
    // These are registered by the prelude and harvested explicitly.
    let g = &elab_env.globals;
    let get = |name: &str| -> Option<ken_kernel::GlobalId> { g.get(name).copied() };

    let (process_input_id, program_caps_id, host_io_id, exit_code_id) = match (
        get("ProcessInput"),
        get("ProgramCaps"),
        get("HostIO"),
        get("ExitCode"),
    ) {
        (Some(a), Some(b), Some(c), Some(d)) => (a, b, c, d),
        _ => {
            eprintln!("ken run: entrypoint ABI declarations are unavailable");
            std::process::exit(2);
        }
    };
    if !entrypoint_has_abi(
        &elab_env,
        main_id,
        process_input_id,
        program_caps_id,
        host_io_id,
        exit_code_id,
    ) {
        eprintln!(
            "ken run: invalid entrypoint 'main': expected \
             ProcessInput -> ProgramCaps -> HostIO ExitCode"
        );
        std::process::exit(1);
    }

    // Bare names, matching the landed prelude's registration (`prelude.rs`:
    // Constructors are registered under their bare (not dotted) names.
    let (
        itree_id,
        ret_id,
        vis_id,
        read_id,
        write_id,
        flush_id,
        is_terminal_id,
        stdin_id,
        stdout_id,
        stderr_id,
        chunk_id,
        eof_id,
        unit_id,
        true_id,
        false_id,
        ok_id,
        err_id,
        notfound_id,
        permissiondenied_id,
        capabilitydenied_id,
        brokenpipe_id,
        interrupted_id,
        other_id,
    ) = match (
        get("ITree"),
        get("Ret"),
        get("Vis"),
        get("Read"),
        get("Write"),
        get("Flush"),
        get("IsTerminal"),
        get("Stdin"),
        get("Stdout"),
        get("Stderr"),
        get("Chunk"),
        get("Eof"),
        get("MkUnit"),
        get("True"),
        get("False"),
        get("Ok"),
        get("Err"),
        get("NotFound"),
        get("PermissionDenied"),
        get("CapabilityDenied"),
        get("BrokenPipe"),
        get("Interrupted"),
        get("Other"),
    ) {
        (
            Some(a),
            Some(b),
            Some(c),
            Some(d),
            Some(e),
            Some(f),
            Some(g),
            Some(h),
            Some(i),
            Some(j),
            Some(k),
            Some(l),
            Some(m),
            Some(n),
            Some(o),
            Some(p),
            Some(q),
            Some(r),
            Some(s),
            Some(t),
            Some(u),
            Some(v),
            Some(w),
        ) => (
            a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w,
        ),
        _ => {
            eprintln!("ken run: Console ABI declarations are unavailable");
            std::process::exit(2);
        }
    };

    let console_ids = ken_interp::ConsoleIds {
        itree_id,
        ret_id,
        vis_id,
        read_id,
        write_id,
        flush_id,
        is_terminal_id,
        stdin_id,
        stdout_id,
        stderr_id,
        chunk_id,
        eof_id,
        true_id,
        false_id,
        ok_id,
        err_id,
        notfound_id,
        permissiondenied_id,
        capabilitydenied_id,
        brokenpipe_id,
        interrupted_id,
        other_id,
        unit_id,
        params_len: 3, // ITree (E:Type)(Resp:E->Type)(R:Type) — 3 type params (State-effect-build lift)
    };

    // Harvest FS IDs (FS-driver-build D1/D2); absent on a program that never
    // registers `[FS]` (can't happen post-prelude, but degrade gracefully
    // rather than assume, matching the Console harvest's own style above).
    let fs_ids = match get("ReadFile") {
        Some(readfile_id) => Some(ken_interp::FSIds { readfile_id }),
        _ => None,
    };

    let mut store = build_eval_store(&elab_env);

    let main_term = ken_kernel::Term::const_(main_id, vec![]);
    let mut tree = ken_interp::eval(&[], &main_term, &elab_env.env, &mut store);
    let input = process_input_value(&elab_env, arguments);
    tree = ken_interp::apply(tree, input, &elab_env.env, &mut store);
    let cap =
        ken_elaborator::capabilities::Cap::mint(ken_elaborator::capabilities::AUTH_PARTIAL, "FS");
    let caps = constructor_value(
        get("MkProgramCaps").expect("ProgramCaps constructor registered"),
        vec![ken_interp::EvalVal::Cap(cap)],
    );
    tree = ken_interp::apply(tree, caps, &elab_env.env, &mut store);

    let coproduct_ids = ken_interp::CoproductIds {
        inl_id: elab_env.prelude_env.inl_id,
        inr_id: elab_env.prelude_env.inr_id,
    };

    let mut host = ken_interp::PosixHost::new();
    match ken_interp::run_io(
        tree,
        &mut host,
        &console_ids,
        fs_ids.as_ref(),
        Some(&coproduct_ids),
        &elab_env.env,
        &mut store,
    ) {
        Ok(final_val) => std::process::exit(exit_status(
            &final_val,
            get("Success").expect("Success registered"),
            get("Failure").expect("Failure registered"),
        )),
        Err(ken_interp::RunIoError::UnknownTree) => {
            eprintln!("ken run: program evaluated to an open hole (Unknown)");
            std::process::exit(1);
        }
        Err(ken_interp::RunIoError::UnknownEffect(v)) => {
            eprintln!("ken run: unhandled effect: {:?}", v);
            std::process::exit(1);
        }
        Err(ken_interp::RunIoError::NotAnIOTree(v)) => {
            eprintln!("ken run: entrypoint did not return an IO tree: {:?}", v);
            std::process::exit(1);
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum EntrypointResolutionError {
    MissingMain,
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
) -> bool {
    use ken_kernel::{Decl, Term};

    if let Some(row) = elab_env.effect_rows.get("main") {
        let granted = ken_elaborator::effects::EffectRow::from_effects([
            "Console".to_string(),
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
    let program_caps = Term::indformer(program_caps_id, vec![]);
    let exit_code = Term::indformer(exit_code_id, vec![]);
    let host_exit = Term::app(Term::const_(host_io_id, vec![]), exit_code);
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
    let environment = std::env::vars_os().map(|(key, value)| {
        constructor_value(
            elab_env.prelude_env.mkprod_id,
            vec![
                ken_interp::EvalVal::Unknown,
                ken_interp::EvalVal::Unknown,
                ken_interp::EvalVal::Bytes(os_bytes(&key)),
                ken_interp::EvalVal::Bytes(os_bytes(&value)),
            ],
        )
    });
    let environment = list_value(
        elab_env.prelude_env.nil_id,
        elab_env.prelude_env.cons_id,
        environment,
    );
    let cwd = std::env::current_dir().unwrap_or_else(|error| {
        eprintln!("ken run: cannot read working directory: {error}");
        std::process::exit(1);
    });
    constructor_value(
        get("MkProcessInput"),
        vec![
            arguments,
            environment,
            ken_interp::EvalVal::Bytes(os_bytes(cwd.as_os_str())),
        ],
    )
}

fn exit_status(
    value: &ken_interp::EvalVal,
    success_id: ken_kernel::GlobalId,
    failure_id: ken_kernel::GlobalId,
) -> i32 {
    match value {
        ken_interp::EvalVal::Ctor { id, .. } if *id == success_id => 0,
        ken_interp::EvalVal::Ctor { id, args, .. } if *id == failure_id => match args.first() {
            Some(ken_interp::EvalVal::Int(0)) => 1,
            Some(ken_interp::EvalVal::Int(code @ 1..=255)) => *code as i32,
            _ => {
                eprintln!("ken run: malformed ExitCode::Failure payload");
                1
            }
        },
        _ => {
            eprintln!("ken run: entrypoint returned a malformed ExitCode");
            1
        }
    }
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
fn build_eval_store(elab_env: &ken_elaborator::ElabEnv) -> ken_interp::EvalStore {
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

fn print_help() {
    println!(
        "ken {} — verified topos-oriented language",
        env!("CARGO_PKG_VERSION")
    );
    println!();
    println!("Usage: ken <subcommand>");
    println!();
    println!("Subcommands:");
    println!("  run <file>    Elaborate and run a Ken source file (Console IO)");
    println!("  check <file>  Elaborate a Ken source file and verify its fences,");
    println!("                without driving IO (for pure-library entries)");
    println!("  fmt [--check] <paths...>");
    println!("                Canonicalize Ken source, or check without writing");
    println!("  repl          Start the interactive REPL (the Little Prover loop)");
    println!("  version       Print version and kernel information");
    println!("  help          Print this message");
}

#[cfg(test)]
mod entrypoint_tests {
    use std::ffi::OsString;

    use super::*;

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

    #[test]
    fn unknown_argument_before_separator_is_a_specific_error() {
        let args = vec![OsString::from("app.ken"), OsString::from("--bad")];
        assert_eq!(
            parse_run_invocation(&args).map(|_| ()),
            Err(RunArgumentError::UnexpectedBeforeSeparator(OsString::from(
                "--bad"
            )))
        );
    }

    #[cfg(unix)]
    #[test]
    fn program_arguments_after_separator_preserve_non_utf8_bytes() {
        use std::os::unix::ffi::OsStringExt;

        let raw = vec![0xff, 0x00, b'a'];
        let args = vec![
            OsString::from("app.ken"),
            OsString::from("--"),
            OsString::from_vec(raw.clone()),
        ];
        let invocation = parse_run_invocation(&args).expect("valid invocation");
        assert_eq!(invocation.arguments, vec![raw]);
    }
}
