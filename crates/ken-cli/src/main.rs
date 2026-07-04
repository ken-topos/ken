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

    // Bare names, matching the landed prelude's registration (`prelude.rs`:
    // `data ITree r = Ret r | Vis ConsoleOp (Unit -> ITree r)` — one type
    // param, constructors registered under their bare (not dotted) names).
    let (itree_id, ret_id, vis_id, write_id, unit_id) = match (
        get("ITree"),
        get("Ret"),
        get("Vis"),
        get("Write"),
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
        params_len: 3, // ITree (E:Type)(Resp:E->Type)(R:Type) — 3 type params (State-effect-build lift)
    };

    // Harvest FS IDs (FS-driver-build D1/D2); absent on a program that never
    // registers `[FS]` (can't happen post-prelude, but degrade gracefully
    // rather than assume, matching the Console harvest's own style above).
    let fs_ids = match (
        get("ReadFile"),
        get("Ok"),
        get("Err"),
        get("NotFound"),
        get("PermissionDenied"),
        get("CapabilityDenied"),
        get("Other"),
    ) {
        (Some(readfile_id), Some(ok_id), Some(err_id), Some(notfound_id), Some(permissiondenied_id), Some(capabilitydenied_id), Some(other_id)) => {
            Some(ken_interp::FSIds {
                readfile_id,
                ok_id,
                err_id,
                notfound_id,
                permissiondenied_id,
                capabilitydenied_id,
                other_id,
            })
        }
        _ => None,
    };

    let mut store = build_eval_store(&elab_env);

    let main_term = ken_kernel::Term::const_(main_id, vec![]);
    let mut tree = ken_interp::eval(&[], &main_term, &elab_env.env, &mut store);

    // fs-read-file-lines-flip D2b — the manifest→mint-exactly→bind sequence
    // (the gap `FS-driver-conformance.md`/`fs-read-file-lines-flip.md` D2
    // names): read the authority `main`'s OWN TYPE declares on its `Cap`
    // param (a structural read — non-widenable by construction, the audit
    // point), mint EXACTLY that (never full, never ambient — the operator's
    // locked least-privilege ruling), and bind it to `main` before `run_io`.
    // A `main` with no FS cap param mints/binds nothing (`declared_fs_
    // authority` returns `None`) — unchanged from today's behavior.
    if let Some(authority) = declared_fs_authority(&elab_env, main_id) {
        let cap = ken_elaborator::capabilities::Cap::mint(authority, "FS");
        tree = ken_interp::apply(tree, ken_interp::EvalVal::Cap(cap), &elab_env.env, &mut store);
    }

    let sum_ids = ken_interp::SumIds {
        inl_id: elab_env.prelude_env.inl_id,
        inr_id: elab_env.prelude_env.inr_id,
    };

    match ken_interp::run_io(tree, &console_ids, fs_ids.as_ref(), Some(&sum_ids), &elab_env.env, &mut store) {
        Ok(final_val) => {
            // fs-read-file-lines-flip D4 (Option 3, Steward/Architect
            // ruling `evt_5a6kr3sgsmzp0`): `main`'s `[FS]` computation is
            // pure FS-read + parse, NOT Console-composed — the returned
            // `Result IOError (List String)` is rendered HERE, post-`run_io`,
            // not printed from within the Ken program itself. A non-FS
            // program's return value (`fs_ids` absent, or a value whose
            // ctor doesn't match `Ok`/`Err`) is left untouched — unchanged
            // behavior for every Console-only example.
            if let Some(fs) = fs_ids.as_ref() {
                render_fs_result(
                    &final_val,
                    fs,
                    elab_env.prelude_env.nil_id,
                    elab_env.prelude_env.cons_id,
                );
            }
        }
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

/// fs-read-file-lines-flip D4 (Option 3): render a `main` return value shaped
/// `Result IOError (List String)` — print each line on `Ok`, render the
/// failure and exit non-zero on `Err` (fail-closed: a `CapabilityDenied`/
/// `NotFound` must never be mistaken for success by anything diffing stdout).
/// A value whose ctor matches neither `ok_id` nor `err_id` is a non-FS
/// program's return value and is left alone.
fn render_fs_result(
    val: &ken_interp::EvalVal,
    fs: &ken_interp::FSIds,
    nil_id: ken_kernel::GlobalId,
    cons_id: ken_kernel::GlobalId,
) {
    match val {
        ken_interp::EvalVal::Ctor { id, args, .. } if *id == fs.ok_id => {
            if let Some(lines) = args.get(2) {
                print_string_list(lines, nil_id, cons_id);
            }
        }
        ken_interp::EvalVal::Ctor { id, args, .. } if *id == fs.err_id => {
            let name = match args.get(2) {
                Some(ken_interp::EvalVal::Ctor { id: err_id, .. }) if *err_id == fs.notfound_id => {
                    "NotFound"
                }
                Some(ken_interp::EvalVal::Ctor { id: err_id, .. })
                    if *err_id == fs.permissiondenied_id =>
                {
                    "PermissionDenied"
                }
                Some(ken_interp::EvalVal::Ctor { id: err_id, .. })
                    if *err_id == fs.capabilitydenied_id =>
                {
                    "CapabilityDenied"
                }
                Some(ken_interp::EvalVal::Ctor { id: err_id, .. }) if *err_id == fs.other_id => {
                    "Other"
                }
                other => {
                    eprintln!("ken run: IOError (unrecognized payload {:?})", other);
                    std::process::exit(1);
                }
            };
            eprintln!("ken run: IOError({})", name);
            std::process::exit(1);
        }
        _ => {}
    }
}

/// Print every element of a `List String` value, one per line (mirrors
/// `print_line`'s own `println!`). `Cons`'s args are `[type-param-filler,
/// head, tail]` (`ctor_arity = params.len() + args.len()`, `List`'s single
/// param `a` fills index 0); `Nil`'s sole arg is its own type-param filler
/// — distinguished by CTOR ID, not arg count (`Nil` is non-empty: `[filler]`).
fn print_string_list(val: &ken_interp::EvalVal, nil_id: ken_kernel::GlobalId, cons_id: ken_kernel::GlobalId) {
    let mut cur = val;
    loop {
        match cur {
            ken_interp::EvalVal::Ctor { id, .. } if *id == nil_id => break,
            ken_interp::EvalVal::Ctor { id, args, .. } if *id == cons_id => {
                match (args.get(1), args.get(2)) {
                    (Some(ken_interp::EvalVal::Str(s)), Some(tail)) => {
                        println!("{}", s);
                        cur = tail;
                    }
                    _ => break,
                }
            }
            _ => break,
        }
    }
}

/// Peel an application spine `f a₁ a₂ … aₙ` into `(f, [a₁, …, aₙ])`.
fn peel_app(t: &ken_kernel::Term) -> (&ken_kernel::Term, Vec<&ken_kernel::Term>) {
    let mut args = Vec::new();
    let mut cur = t;
    while let ken_kernel::Term::App(f, a) = cur {
        args.push(a.as_ref());
        cur = f.as_ref();
    }
    args.reverse();
    (cur, args)
}

/// D2b's manifest read: does `main`'s type declare an FS capability
/// parameter, and if so, at what authority level?
///
/// Walks `main`'s type to its FIRST Π-domain and peels its application
/// spine to the head (BV2: the enrichment changed a cap-param domain from
/// `Const(Cap)` to `App(Cap, a)`, so detection must key on the `Cap` HEAD
/// through the spine, not the domain as a whole). Returns `None` for a
/// `main` with no FS cap param (mint/bind nothing — unchanged behavior) —
/// this is a structural read of the type, never a computed/inflatable
/// value (Architect's non-widenable constraint).
fn declared_fs_authority(
    elab_env: &ken_elaborator::ElabEnv,
    main_id: ken_kernel::GlobalId,
) -> Option<ken_elaborator::capabilities::Authority> {
    use ken_elaborator::capabilities::{AUTH_FULL, AUTH_NONE, AUTH_PARTIAL};
    use ken_kernel::{Decl, Term};

    let g = &elab_env.globals;
    let cap_id = g.get("Cap").copied()?;
    let anone_id = g.get("ANone").copied()?;
    let apartial_id = g.get("APartial").copied()?;
    let afull_id = g.get("AFull").copied()?;

    let ty = match elab_env.env.lookup(main_id)? {
        Decl::Transparent { ty, .. } | Decl::Opaque { ty, .. } | Decl::Primitive { ty, .. } => ty,
        Decl::Inductive(_) => return None,
    };
    let dom = match ty {
        Term::Pi(dom, _) => dom.as_ref(),
        _ => return None,
    };
    let (head, args) = peel_app(dom);
    let (Term::Const { id, .. }, [auth_arg]) = (head, args.as_slice()) else {
        return None;
    };
    if *id != cap_id {
        return None;
    }
    match auth_arg {
        Term::Constructor { id, .. } if *id == anone_id => Some(AUTH_NONE),
        Term::Constructor { id, .. } if *id == apartial_id => Some(AUTH_PARTIAL),
        Term::Constructor { id, .. } if *id == afull_id => Some(AUTH_FULL),
        _ => None,
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

#[cfg(test)]
mod declared_fs_authority_tests {
    use super::declared_fs_authority;

    /// BV2 — positive: `(cap : Cap APartial)` (the enriched `App(Cap, a)`
    /// domain shape) IS detected as an FS cap param, and the declared
    /// authority is read correctly off the app spine.
    #[test]
    fn cap_apartial_detected_with_correct_authority() {
        let mut env = ken_elaborator::ElabEnv::new().expect("env");
        let main_id = env
            .elaborate_decl("view main (cap : Cap APartial) : Cap APartial = cap")
            .expect("elaborates");
        assert_eq!(
            declared_fs_authority(&env, main_id),
            Some(ken_elaborator::capabilities::AUTH_PARTIAL)
        );
    }

    /// SEAM-A: `(cap : Cap ANone)` is STILL detected (the cap param is kept,
    /// not absent) — it is minted+bound and denied at the driver, never at
    /// this manifest-read step.
    #[test]
    fn cap_anone_detected_with_correct_authority() {
        let mut env = ken_elaborator::ElabEnv::new().expect("env");
        let main_id = env
            .elaborate_decl("view main (cap : Cap ANone) : Cap ANone = cap")
            .expect("elaborates");
        assert_eq!(
            declared_fs_authority(&env, main_id),
            Some(ken_elaborator::capabilities::AUTH_NONE)
        );
    }

    /// A `main` with no FS cap param at all mints/binds nothing — the
    /// distinct `MissingCapability`-at-elaboration foil, not this gate.
    #[test]
    fn no_cap_param_detects_nothing() {
        let mut env = ken_elaborator::ElabEnv::new().expect("env");
        let main_id = env.elaborate_decl("view main : Nat = Zero").expect("elaborates");
        assert_eq!(declared_fs_authority(&env, main_id), None);
    }
}
