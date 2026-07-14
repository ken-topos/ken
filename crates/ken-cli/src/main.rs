//! The `ken` command-line driver.

mod repl;

use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

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
    let source = std::fs::read_to_string(path).unwrap_or_else(|error| {
        eprintln!("ken run: cannot read '{}': {error}", path.to_string_lossy());
        std::process::exit(1);
    });
    let format = if path.to_string_lossy().ends_with(".ken.md") {
        ken_cli::SourceFormat::LiterateKen
    } else {
        ken_cli::SourceFormat::Ken
    };
    let environment: Vec<_> = std::env::vars_os()
        .map(|(key, value)| (os_bytes(&key), os_bytes(&value)))
        .collect();
    let cwd = std::env::current_dir().unwrap_or_else(|error| {
        eprintln!("ken run: cannot read working directory: {error}");
        std::process::exit(1);
    });
    let mut host = ken_interp::PosixHost::new();
    match ken_cli::run_program(
        &source,
        format,
        arguments,
        &environment,
        &os_bytes(cwd.as_os_str()),
        &mut host,
    ) {
        Ok(outcome) => std::process::exit(outcome.exit_status),
        Err(ken_cli::RunError::DuplicateEntrypoint) => {
            eprintln!("ken run: duplicate entrypoint 'main'");
            std::process::exit(1);
        }
        Err(ken_cli::RunError::MissingEntrypoint) => {
            eprintln!(
                "ken run: missing entrypoint 'main' in '{}'",
                path.to_string_lossy()
            );
            std::process::exit(1);
        }
        Err(ken_cli::RunError::EntrypointAbiUnavailable) => {
            eprintln!("ken run: entrypoint ABI declarations are unavailable");
            std::process::exit(2);
        }
        Err(ken_cli::RunError::MissingCapability { effect }) => {
            eprintln!("ken run: MissingCapability {{ effect = {effect} }}");
            std::process::exit(1);
        }
        Err(ken_cli::RunError::InvalidEntrypoint { authority }) => {
            eprintln!(
                "ken run: invalid entrypoint 'main': expected ProcessInput -> \
                 ProgramCaps {authority} -> HostIO {authority} ExitCode"
            );
            std::process::exit(1);
        }
        Err(ken_cli::RunError::ConsoleAbiUnavailable) => {
            eprintln!("ken run: Console ABI declarations are unavailable");
            std::process::exit(2);
        }
        Err(ken_cli::RunError::Initialization(error)) => {
            eprintln!("ken run: elaborator init failed: {error:?}");
            std::process::exit(1);
        }
        Err(ken_cli::RunError::Elaboration(error)) => {
            eprintln!(
                "ken run: elaboration error in '{}': {error:?}",
                path.to_string_lossy()
            );
            std::process::exit(1);
        }
        Err(ken_cli::RunError::Io(ken_interp::RunIoError::UnknownTree)) => {
            eprintln!("ken run: program evaluated to an open hole (Unknown)");
            std::process::exit(1);
        }
        Err(ken_cli::RunError::Io(ken_interp::RunIoError::UnknownEffect(value))) => {
            eprintln!("ken run: unhandled effect: {value:?}");
            std::process::exit(1);
        }
        Err(ken_cli::RunError::Io(ken_interp::RunIoError::NotAnIOTree(value))) => {
            eprintln!("ken run: entrypoint did not return an IO tree: {value:?}");
            std::process::exit(1);
        }
    }
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
mod run_argument_tests {
    use std::ffi::OsString;

    use super::*;

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
