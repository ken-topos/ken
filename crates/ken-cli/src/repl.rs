//! T2 REPL — the Little Prover interactive loop.
//!
//! Drives `ken-elaborator` (V0/V1), `ken-kernel` (check), the V-spine (V3
//! prover), and `ken-interp` (X1) as a front-end. Never reimplements them.
//!
//! # Multi-line rule
//! A declaration (`let`/`view`/`prove`/`law`/`space`-started input) spans
//! until an empty line. A `:command` or bare expression is always single-line.
//!
//! # Redefinition policy
//! A name re-entered shadows the previous binding in the elaboration env;
//! both the old and new `GlobalId` remain in the global env (kernel does not
//! retract). The `:list` command shows all names in entry order.

use std::io::{self, BufRead, Write};

use ken_elaborator::{
    ElabEnv, ElabError,
    extract::v2_extract,
    prover::{attempt_obligation, Countermodel, Verdict},
};
use ken_interp::eval::{eval, EvalStore, EvalVal};
use ken_kernel::Term;

// ── Session ───────────────────────────────────────────────────────────────────

struct Session {
    env: ElabEnv,
    store: EvalStore,
    /// Stable counter used to generate unique obligation ids for `:check`.
    check_seq: usize,
    /// Names registered in this session, in order (for `:list`).
    names: Vec<String>,
}

impl Session {
    fn new() -> Result<Self, ElabError> {
        Ok(Session {
            env: ElabEnv::new()?,
            store: EvalStore::new(),
            check_seq: 0,
            names: Vec::new(),
        })
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// True when `s` starts with a declaration keyword.
fn is_decl_start(s: &str) -> bool {
    let t = s.trim_start();
    t.starts_with("let ")
        || t.starts_with("view ")
        || t.starts_with("prove ")
        || t.starts_with("law ")
        || t.starts_with("space ")
        || t == "let"
        || t == "view"
        || t == "prove"
        || t == "law"
}

/// Render a `Verdict` for the user. `proved` only when the kernel certified it.
fn show_verdict(v: &Verdict) -> String {
    match v {
        Verdict::Proved { .. } => "proved (Q) — kernel-certified".to_owned(),
        Verdict::Disproved { countermodel: Countermodel { description } } => {
            format!("refuted — countermodel: {}", description)
        }
        Verdict::Unknown { .. } => "unknown — open goal".to_owned(),
    }
}

/// Render an `EvalVal` for the user.
fn show_val(v: &EvalVal) -> String {
    match v {
        EvalVal::Bool(b) => b.to_string(),
        EvalVal::Int(n) => n.to_string(),
        EvalVal::Ctor { id, args, .. } => {
            if args.is_empty() {
                format!("ctor({})", id)
            } else {
                let parts: Vec<_> = args.iter().map(show_val).collect();
                format!("ctor({}) {}", id, parts.join(" "))
            }
        }
        EvalVal::Pair { fst, snd, .. } => format!("({}, {})", show_val(fst), show_val(snd)),
        EvalVal::Closure { .. } => "<closure>".to_owned(),
        EvalVal::CtorPending { id, args, need } => {
            format!("<ctor({}) — {} args, need {}>", id, args.len(), need)
        }
        EvalVal::TypeUniverse(l) => format!("Type {}", l),
        EvalVal::OmegaUniverse(l) => format!("Prop {}", l),
        EvalVal::PiTy { .. } => "<Π-type>".to_owned(),
        EvalVal::SigmaTy { .. } => "<Σ-type>".to_owned(),
        EvalVal::IndFormerVal { id } => format!("<inductive {}>", id),
        EvalVal::ReflVal { .. } => "<refl>".to_owned(),
        EvalVal::Unknown => "<unknown>".to_owned(),
        EvalVal::Neutral => "<neutral>".to_owned(),
        EvalVal::Bytes(bs) => format!("{:?}", bs),
        EvalVal::Str(s) => format!("{:?}", s),
        EvalVal::BigInt(n) => format!("{}", n),
        EvalVal::Float(f) => format!("{}", f),
        EvalVal::Float32(f) => format!("{}", f),
        EvalVal::DecimalVal { coeff, exp } => format!("{}e{}", coeff, exp),
    }
}

/// Render a `Term` (used for type display). Uses Debug — a proper pretty-printer
/// is future work (G7 / later WS-T WP).
fn show_term(t: &Term) -> String {
    format!("{:?}", t)
}

// ── REPL actions ─────────────────────────────────────────────────────────────

/// Elaborate and register a declaration (`let`, `view`, `prove`, `law`).
/// On success: print the registered name. On error: print diagnostic, no registration.
fn do_def(session: &mut Session, src: &str) {
    match session.env.elaborate_decl(src.trim()) {
        Ok(_id) => {
            // Extract the declared name (second whitespace token after the keyword).
            let name = src
                .split_whitespace()
                .nth(1)
                .unwrap_or("?")
                .trim_end_matches(':')
                .to_owned();
            println!("  defined: {}", name);
            session.names.push(name);
        }
        Err(e) => println!("  error: {}", e),
    }
}

/// Verify a proposition through the V-spine (V3 prover).
/// The goal is wrapped in `prove repl_goal_N : <goal>` so V2 extraction
/// produces a proper `ObligationTriple` and the full pipeline is exercised.
fn do_check(session: &mut Session, goal_src: &str) {
    let id = format!("repl_goal_{}", session.check_seq);
    session.check_seq += 1;
    let src = format!("prove {} : {}", id, goal_src);

    match session.env.elaborate_decl_v1(src.trim()) {
        Ok(elab_result) => {
            let extraction = v2_extract(&elab_result);
            if extraction.obligations.is_empty() {
                // No obligations: the goal was trivially discharged at elaboration
                // (e.g., a tautology whose proof the elaborator inserted directly).
                println!("  proved (Q) — no obligations generated");
                return;
            }
            for triple in &extraction.obligations {
                let result = attempt_obligation(&mut session.env.env, triple);
                println!("  {}", show_verdict(&result.verdict));
            }
        }
        Err(e) => println!("  error: {}", e),
    }
}

/// Evaluate an expression through `ken-interp` (X1).
fn do_eval(session: &mut Session, expr_src: &str) {
    match session.env.elaborate_expr(expr_src.trim()) {
        Ok((term, ty)) => {
            let val = eval(&[], &term, &session.env.env, &mut session.store);
            println!("  {} : {}", show_val(&val), show_term(&ty));
        }
        Err(e) => println!("  error: {}", e),
    }
}

/// Infer and print the type of an expression.
fn do_type(session: &mut Session, expr_src: &str) {
    match session.env.elaborate_expr(expr_src.trim()) {
        Ok((_term, ty)) => println!("  : {}", show_term(&ty)),
        Err(e) => println!("  error: {}", e),
    }
}

fn print_help() {
    println!("  Commands:");
    println!("    :def <decl>         elaborate+check a declaration (let/view/prove/law)");
    println!("    :check <prop>       verify a proposition via the prover (alias: :prove)");
    println!("    :eval <expr>        evaluate an expression via ken-interp");
    println!("    :type <expr>        infer and print the type of an expression");
    println!("    :list               list names defined in this session");
    println!("    :reset              reset the session (clear all definitions)");
    println!("    :help               print this help");
    println!("    :quit               exit the REPL (alias: :q)");
    println!();
    println!("  Bare input:");
    println!("    let/view/prove/law/space …   treated as a declaration");
    println!("    anything else                evaluated as an expression");
    println!("    Multi-line: blank line ends a declaration block.");
}

// ── Read-eval-print loop ──────────────────────────────────────────────────────

pub fn run() {
    let mut session = match Session::new() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("repl: init failed: {}", e);
            return;
        }
    };

    println!(
        "ken {} repl — :help for commands, :quit to exit",
        env!("CARGO_PKG_VERSION")
    );
    println!("kernel {}", ken_elaborator::kernel_version());

    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    loop {
        print!("ken> ");
        io::stdout().flush().ok();

        let raw = match lines.next() {
            Some(Ok(l)) => l,
            Some(Err(_)) | None => break,
        };

        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }

        // ── `:command` dispatch ───────────────────────────────────────────
        if let Some(rest) = trimmed.strip_prefix(':') {
            let (cmd, args) = rest
                .split_once(char::is_whitespace)
                .map(|(c, a)| (c.trim(), a.trim()))
                .unwrap_or((rest.trim(), ""));

            match cmd {
                "quit" | "q" => break,

                "help" | "h" => print_help(),

                "list" => {
                    if session.names.is_empty() {
                        println!("  (no definitions in session)");
                    } else {
                        for name in &session.names {
                            println!("  {}", name);
                        }
                    }
                }

                "reset" => {
                    session = match Session::new() {
                        Ok(s) => s,
                        Err(e) => {
                            println!("  error resetting session: {}", e);
                            continue;
                        }
                    };
                    println!("  session reset");
                }

                "check" | "prove" => {
                    if args.is_empty() {
                        println!("  usage: :{} <proposition>", cmd);
                    } else {
                        do_check(&mut session, args);
                    }
                }

                "eval" => {
                    if args.is_empty() {
                        println!("  usage: :eval <expression>");
                    } else {
                        do_eval(&mut session, args);
                    }
                }

                "type" => {
                    if args.is_empty() {
                        println!("  usage: :type <expression>");
                    } else {
                        do_type(&mut session, args);
                    }
                }

                "def" => {
                    if args.is_empty() {
                        println!("  usage: :def <let/view/prove/law declaration>");
                    } else {
                        do_def(&mut session, args);
                    }
                }

                other => println!("  unknown command ':{}'  — try :help", other),
            }
            continue;
        }

        // ── Declaration (keyword-started) — reads until blank line ────────
        if is_decl_start(trimmed) {
            let mut block = raw.clone();
            block.push('\n');
            loop {
                print!("  | ");
                io::stdout().flush().ok();
                match lines.next() {
                    Some(Ok(l)) if l.trim().is_empty() => break,
                    Some(Ok(l)) => {
                        block.push_str(&l);
                        block.push('\n');
                    }
                    _ => break,
                }
            }
            do_def(&mut session, &block);
            continue;
        }

        // ── Bare expression — evaluate via ken-interp ─────────────────────
        do_eval(&mut session, trimmed);
    }

    println!("bye");
}
