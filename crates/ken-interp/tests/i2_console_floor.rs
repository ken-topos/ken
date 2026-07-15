//! I-2 CA2 Console floor: exact host-seam and ordinary-package acceptance.

use ken_interp::{CaptureHost, ConsoleIds, ConsoleStream, ConsoleTrace, EvalStore, EvalVal};
use ken_kernel::{GlobalId, Term};

const CONSOLE_PACKAGE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../catalog/packages/Capability/Console/Text.ken.md"
));

struct ConsoleEnv {
    elab: ken_elaborator::ElabEnv,
    ids: ConsoleIds,
    print_line: GlobalId,
    eprint_line: GlobalId,
    read: GlobalId,
    write: GlobalId,
    flush: GlobalId,
    is_terminal: GlobalId,
}

fn setup() -> ConsoleEnv {
    let mut elab = ken_elaborator::ElabEnv::new().expect("prelude registers");
    elab.elaborate_ken_md_file(CONSOLE_PACKAGE)
        .expect("ordinary Console package elaborates");
    let get = |name: &str| -> GlobalId {
        *elab
            .globals
            .get(name)
            .unwrap_or_else(|| panic!("missing global {name}"))
    };
    let ids = ConsoleIds::from_elab(&elab).expect("complete Console ABI");
    ConsoleEnv {
        print_line: get("printLine"),
        eprint_line: get("eprintLine"),
        read: get("read"),
        write: get("write"),
        flush: get("flush"),
        is_terminal: get("is_terminal"),
        ids,
        elab,
    }
}

fn eval_global(env: &ConsoleEnv, id: GlobalId, store: &mut EvalStore) -> EvalVal {
    ken_interp::eval(&[], &Term::const_(id, vec![]), &env.elab.env, store)
}

fn eval_store(env: &ConsoleEnv) -> EvalStore {
    use ken_elaborator::NumericLitVal;
    let mut store = EvalStore::new();
    for (id, literal) in &env.elab.num_values {
        let value = match literal {
            NumericLitVal::Int(value) => EvalVal::from(*value),
            NumericLitVal::Float(value) => EvalVal::Float(*value),
            NumericLitVal::Float32(value) => EvalVal::Float32(*value),
            NumericLitVal::Decimal { coeff, exp } => {
                ken_interp::decimal_value(env.elab.prelude_env.mkdecimalpair_id, *coeff, *exp)
            }
            NumericLitVal::Str(value) => EvalVal::Str(value.clone()),
        };
        store.num_values.insert(*id, value);
    }
    store.list_char_ids = Some(ken_interp::eval::ListCharIds {
        nil_id: env.elab.prelude_env.nil_id,
        cons_id: env.elab.prelude_env.cons_id,
    });
    store
}

fn stream(env: &ConsoleEnv, id: GlobalId, store: &mut EvalStore) -> EvalVal {
    ken_interp::eval(
        &[],
        &Term::Constructor {
            id,
            level_args: vec![],
        },
        &env.elab.env,
        store,
    )
}

fn drive(
    env: &ConsoleEnv,
    host: &mut CaptureHost,
    tree: EvalVal,
    store: &mut EvalStore,
) -> EvalVal {
    ken_interp::run_io(tree, host, &env.ids, None, None, None, &env.elab.env, store)
        .expect("Console tree drives")
}

fn result_payload(value: &EvalVal, expected_ctor: GlobalId) -> &EvalVal {
    match value {
        EvalVal::Ctor { id, args, .. } if *id == expected_ctor => {
            args.get(2).expect("Result payload")
        }
        other => panic!("expected Result ctor {expected_ctor:?}, got {other:?}"),
    }
}

#[test]
fn console_surface_and_package_are_zero_trust_definitions() {
    let mut elab = ken_elaborator::ElabEnv::new().expect("prelude registers");
    for name in [
        "Stream",
        "ConsoleOp",
        "ReadResult",
        "console_resp",
        "read",
        "write",
        "flush",
        "is_terminal",
    ] {
        let id = *elab
            .globals
            .get(name)
            .unwrap_or_else(|| panic!("missing {name}"));
        assert!(
            !elab.env.trusted_base().contains(&id),
            "{name} must be ordinary kernel-checked Ken"
        );
    }
    let before: std::collections::BTreeSet<_> = elab.env.trusted_base().into_iter().collect();
    elab.elaborate_ken_md_file(CONSOLE_PACKAGE)
        .expect("ordinary Console package elaborates");
    let after: std::collections::BTreeSet<_> = elab.env.trusted_base().into_iter().collect();
    assert_eq!(
        before, after,
        "Console helpers must add zero trusted-base entries"
    );
    let extracted = ken_elaborator::literate::extract_ken_md(CONSOLE_PACKAGE)
        .expect("Console.ken.md must extract");
    assert!(
        !extracted.source.contains("Axiom"),
        "Console.ken code must declare no Axiom"
    );
}

#[test]
fn package_helpers_route_exact_bytes_and_one_newline() {
    let env = setup();
    let mut host = CaptureHost::new(Vec::new());
    let mut store = eval_store(&env);

    let print = eval_global(&env, env.print_line, &mut store);
    let stdout_tree = ken_interp::apply(
        print,
        EvalVal::Str("alpha".into()),
        &env.elab.env,
        &mut store,
    );
    let stdout_result = drive(&env, &mut host, stdout_tree, &mut store);
    assert!(matches!(
        result_payload(&stdout_result, env.ids.ok_id),
        EvalVal::Ctor { id, .. } if *id == env.ids.unit_id
    ));

    let eprint = eval_global(&env, env.eprint_line, &mut store);
    let stderr_tree = ken_interp::apply(
        eprint,
        EvalVal::Str("beta\n".into()),
        &env.elab.env,
        &mut store,
    );
    drive(&env, &mut host, stderr_tree, &mut store);

    assert_eq!(host.stdout(), b"alpha\n");
    assert_eq!(host.stderr(), b"beta\n\n");
    assert_eq!(
        host.trace(),
        &[
            ConsoleTrace::Write {
                stream: ConsoleStream::Stdout,
                bytes: b"alpha\n".to_vec(),
            },
            ConsoleTrace::Write {
                stream: ConsoleStream::Stderr,
                bytes: b"beta\n\n".to_vec(),
            },
        ]
    );
}

#[test]
fn capture_read_is_bounded_and_eof_is_named_not_empty_chunk() {
    let env = setup();
    let mut host = CaptureHost::new(b"abcde".to_vec());
    let mut store = eval_store(&env);

    let mut read_once = |limit: i64| {
        let read = eval_global(&env, env.read, &mut store);
        let read = ken_interp::apply(
            read,
            stream(&env, env.ids.stdin_id, &mut store),
            &env.elab.env,
            &mut store,
        );
        let tree = ken_interp::apply(read, EvalVal::Int(limit), &env.elab.env, &mut store);
        drive(&env, &mut host, tree, &mut store)
    };

    let first = read_once(3);
    match result_payload(&first, env.ids.ok_id) {
        EvalVal::Ctor { id, args, .. } if *id == env.ids.chunk_id => {
            assert!(matches!(args.first(), Some(EvalVal::Bytes(bytes)) if bytes == b"abc"));
        }
        other => panic!("first read must be Chunk abc, got {other:?}"),
    }
    let second = read_once(3);
    match result_payload(&second, env.ids.ok_id) {
        EvalVal::Ctor { id, args, .. } if *id == env.ids.chunk_id => {
            assert!(matches!(args.first(), Some(EvalVal::Bytes(bytes)) if bytes == b"de"));
        }
        other => panic!("second read must be Chunk de, got {other:?}"),
    }
    let eof = read_once(3);
    assert!(matches!(
        result_payload(&eof, env.ids.ok_id),
        EvalVal::Ctor { id, args, .. } if *id == env.ids.eof_id && args.is_empty()
    ));
}

#[test]
fn broken_pipe_is_the_named_ioerror_value() {
    let env = setup();
    let mut host = CaptureHost::new(Vec::new());
    host.close(ConsoleStream::Stdout);
    let mut store = eval_store(&env);
    let write = eval_global(&env, env.write, &mut store);
    let write = ken_interp::apply(
        write,
        stream(&env, env.ids.stdout_id, &mut store),
        &env.elab.env,
        &mut store,
    );
    let tree = ken_interp::apply(
        write,
        EvalVal::Bytes(b"closed".to_vec()),
        &env.elab.env,
        &mut store,
    );
    let result = drive(&env, &mut host, tree, &mut store);
    assert!(matches!(
        result_payload(&result, env.ids.err_id),
        EvalVal::Ctor { id, .. } if *id == env.ids.brokenpipe_id
    ));
}

#[test]
fn flush_and_is_terminal_are_total_and_traced() {
    let env = setup();
    let mut host = CaptureHost::new(Vec::new());
    host.set_terminal(ConsoleStream::Stderr, true);
    let mut store = eval_store(&env);

    let flush = eval_global(&env, env.flush, &mut store);
    let flush_tree = ken_interp::apply(
        flush,
        stream(&env, env.ids.stderr_id, &mut store),
        &env.elab.env,
        &mut store,
    );
    let flushed = drive(&env, &mut host, flush_tree, &mut store);
    assert!(matches!(
        result_payload(&flushed, env.ids.ok_id),
        EvalVal::Ctor { id, .. } if *id == env.ids.unit_id
    ));

    let is_terminal = eval_global(&env, env.is_terminal, &mut store);
    let tty_tree = ken_interp::apply(
        is_terminal,
        stream(&env, env.ids.stderr_id, &mut store),
        &env.elab.env,
        &mut store,
    );
    let tty = drive(&env, &mut host, tty_tree, &mut store);
    assert!(matches!(tty, EvalVal::Ctor { id, .. } if id == env.ids.true_id));
    assert_eq!(
        host.trace(),
        &[
            ConsoleTrace::Flush {
                stream: ConsoleStream::Stderr
            },
            ConsoleTrace::IsTerminal {
                stream: ConsoleStream::Stderr
            },
        ]
    );
}
