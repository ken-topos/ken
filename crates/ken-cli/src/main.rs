//! The `ken` command-line driver.

mod repl;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()).unwrap_or("") {
        "repl" => repl::run(),
        "run" => run_file(args.get(2).map(|s| s.as_str())),
        "version" | "--version" | "-V" => {
            println!("ken {} — verified topos-oriented language", env!("CARGO_PKG_VERSION"));
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

/// `ken run <file>` — elaborate, evaluate, and drive a Console IO program.
///
/// Elaborates every declaration in `<file>` in order, then evaluates the last
/// top-level definition and runs it through the Console effect driver (`42 §6`).
///
/// Console IDs are harvested from the elaboration environment (`ElabEnv::globals`).
/// Until the Language layer registers ITree/Console.Op, this returns an error.
fn run_file(path: Option<&str>) {
    let path = match path {
        Some(p) => p,
        None => {
            eprintln!("ken run: missing <file> argument");
            eprintln!("Usage: ken run <file.ken>");
            std::process::exit(1);
        }
    };

    let src = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ken run: cannot read '{}': {}", path, e);
            std::process::exit(1);
        }
    };

    let mut elab_env = match ken_elaborator::ElabEnv::new() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("ken run: elaborator init failed: {:?}", e);
            std::process::exit(1);
        }
    };

    let ids = match elab_env.elaborate_file(&src) {
        Ok(ids) => ids,
        Err(e) => {
            eprintln!("ken run: elaboration error in '{}': {:?}", path, e);
            std::process::exit(1);
        }
    };

    let main_id = match ids.last() {
        Some(&id) => id,
        None => {
            eprintln!("ken run: '{}' contains no declarations", path);
            std::process::exit(1);
        }
    };

    // Harvest Console IDs from the elaboration environment.
    // These are registered by the Language layer; until that lands the globals
    // map will not contain them and we surface a clear "not yet wired" message.
    let g = &elab_env.globals;
    let get = |name: &str| -> Option<ken_kernel::GlobalId> { g.get(name).copied() };

    let (itree_id, ret_id, vis_id, write_id, unit_id) = match (
        get("ITree"),
        get("ITree.Ret"),
        get("ITree.Vis"),
        get("Console.Op.Write"),
        get("Unit"),
    ) {
        (Some(a), Some(b), Some(c), Some(d), Some(e)) => (a, b, c, d, e),
        _ => {
            // Language layer not yet landed — normal during the Runtime-only build.
            eprintln!("ken run: Console not yet wired (Language layer pending)");
            std::process::exit(2);
        }
    };

    let console_ids = ken_interp::ConsoleIds {
        itree_id,
        ret_id,
        vis_id,
        write_id,
        unit_id,
        params_len: 2, // ITree E R — two type params in the production inductive
    };

    // Build the numeric-literal map from the elaborator.
    let mut store = ken_interp::EvalStore::new();
    for (id, lit) in &elab_env.num_values {
        let val = lit_to_eval(lit);
        store.num_values.insert(*id, val);
    }

    let main_term = ken_kernel::Term::const_(main_id, vec![]);
    let tree = ken_interp::eval(&[], &main_term, &elab_env.env, &mut store);

    match ken_interp::run_io(tree, &console_ids, &elab_env.env, &mut store) {
        Ok(_) => {}
        Err(ken_interp::RunIoError::UnknownTree) => {
            eprintln!("ken run: program evaluated to an open hole (Unknown)");
            std::process::exit(1);
        }
        Err(ken_interp::RunIoError::UnknownEffect(v)) => {
            eprintln!("ken run: unhandled effect: {:?}", v);
            std::process::exit(1);
        }
        Err(ken_interp::RunIoError::NotAnIOTree(v)) => {
            eprintln!("ken run: last definition is not an IO tree: {:?}", v);
            std::process::exit(1);
        }
    }
}

fn lit_to_eval(v: &ken_elaborator::NumericLitVal) -> ken_interp::EvalVal {
    use ken_elaborator::NumericLitVal;
    match v {
        NumericLitVal::Int(n) => {
            let n = *n;
            if n >= i64::MIN as i128 && n <= i64::MAX as i128 {
                ken_interp::EvalVal::Int(n as i64)
            } else {
                ken_interp::EvalVal::BigInt(n)
            }
        }
        NumericLitVal::Float(f) => ken_interp::EvalVal::Float(*f),
        NumericLitVal::Float32(f) => ken_interp::EvalVal::Float32(*f),
        NumericLitVal::Decimal { coeff, exp } => {
            ken_interp::EvalVal::DecimalVal { coeff: *coeff, exp: *exp }
        }
    }
}

fn print_help() {
    println!("ken {} — verified topos-oriented language", env!("CARGO_PKG_VERSION"));
    println!();
    println!("Usage: ken <subcommand>");
    println!();
    println!("Subcommands:");
    println!("  run <file>  Elaborate and run a Ken source file (Console IO)");
    println!("  repl        Start the interactive REPL (the Little Prover loop)");
    println!("  version     Print version and kernel information");
    println!("  help        Print this message");
}
