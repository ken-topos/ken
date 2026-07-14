//! I-7 pure `ProcessInput` environment and working-directory projectors.

use std::collections::BTreeSet;
use std::rc::Rc;

use ken_elaborator::{ElabEnv, NumericLitVal};
use ken_interp::eval::{apply, eval, EvalStore, EvalVal};
use ken_kernel::{Decl, GlobalId};

const ENVIRONMENT_KEN_MD: &str =
    include_str!("../../../catalog/packages/Process/Environment.ken.md");
const WORKING_DIRECTORY_KEN_MD: &str =
    include_str!("../../../catalog/packages/Process/WorkingDirectory.ken.md");

fn full_env() -> ElabEnv {
    let mut env = ElabEnv::empty().expect("prelude bootstrap");
    env.elaborate_ken_md_file(ENVIRONMENT_KEN_MD)
        .expect("Process.Environment must elaborate over the prelude");
    env.elaborate_ken_md_file(WORKING_DIRECTORY_KEN_MD)
        .expect("Process.WorkingDirectory must elaborate over the prelude");
    env
}

fn lit_to_eval(value: &NumericLitVal, mkdecimalpair_id: GlobalId) -> EvalVal {
    match value {
        NumericLitVal::Int(n) => EvalVal::from(*n),
        NumericLitVal::Float(f) => EvalVal::Float(*f),
        NumericLitVal::Float32(f) => EvalVal::Float32(*f),
        NumericLitVal::Decimal { coeff, exp } => {
            ken_interp::decimal_value(mkdecimalpair_id, *coeff, *exp)
        }
        NumericLitVal::Str(s) => EvalVal::Str(s.clone()),
    }
}

fn make_store(env: &ElabEnv) -> EvalStore {
    let mut store = EvalStore::new();
    let mkdecimalpair_id = env.prelude_env.mkdecimalpair_id;
    for (id, value) in &env.num_values {
        store
            .num_values
            .insert(*id, lit_to_eval(value, mkdecimalpair_id));
    }
    store
}

fn constructor_value(id: GlobalId, args: Vec<EvalVal>) -> EvalVal {
    EvalVal::Ctor {
        id,
        args: Rc::new(args),
        slot: 0,
    }
}

fn eval_global(env: &ElabEnv, store: &mut EvalStore, name: &str) -> EvalVal {
    let id = env.globals[name];
    match env.env.lookup(id) {
        Some(Decl::Transparent { body, .. }) => eval(&[], body, &env.env, store),
        other => panic!("`{name}` must be transparent, got {other:?}"),
    }
}

fn call_global(
    env: &ElabEnv,
    store: &mut EvalStore,
    name: &str,
    arguments: impl IntoIterator<Item = EvalVal>,
) -> EvalVal {
    let mut value = eval_global(env, store, name);
    for argument in arguments {
        value = apply(value, argument, &env.env, store);
    }
    value
}

fn list_value(env: &ElabEnv, values: impl IntoIterator<Item = EvalVal>) -> EvalVal {
    let values: Vec<_> = values.into_iter().collect();
    values.into_iter().rev().fold(
        constructor_value(env.prelude_env.nil_id, vec![EvalVal::Unknown]),
        |tail, head| constructor_value(env.prelude_env.cons_id, vec![EvalVal::Unknown, head, tail]),
    )
}

fn environment_value(
    env: &ElabEnv,
    pairs: impl IntoIterator<Item = (Vec<u8>, Vec<u8>)>,
) -> EvalVal {
    list_value(
        env,
        pairs.into_iter().map(|(key, value)| {
            constructor_value(
                env.prelude_env.mkprod_id,
                vec![
                    EvalVal::Unknown,
                    EvalVal::Unknown,
                    EvalVal::Bytes(key),
                    EvalVal::Bytes(value),
                ],
            )
        }),
    )
}

fn process_input(
    env: &ElabEnv,
    arguments: impl IntoIterator<Item = Vec<u8>>,
    environment: impl IntoIterator<Item = (Vec<u8>, Vec<u8>)>,
    working_directory: Vec<u8>,
) -> EvalVal {
    constructor_value(
        env.globals["MkProcessInput"],
        vec![
            list_value(env, arguments.into_iter().map(EvalVal::Bytes)),
            environment_value(env, environment),
            EvalVal::Bytes(working_directory),
        ],
    )
}

fn process_input_fields<'a>(env: &ElabEnv, value: &'a EvalVal) -> &'a [EvalVal] {
    match value {
        EvalVal::Ctor { id, args, .. }
            if *id == env.globals["MkProcessInput"] && args.len() == 3 =>
        {
            args.as_slice()
        }
        other => panic!("expected a fully applied MkProcessInput, got {other:?}"),
    }
}

fn list_bytes(env: &ElabEnv, value: &EvalVal) -> Vec<Vec<u8>> {
    let mut current = value;
    let mut result = Vec::new();
    loop {
        match current {
            EvalVal::Ctor { id, .. } if *id == env.prelude_env.nil_id => return result,
            EvalVal::Ctor { id, args, .. } if *id == env.prelude_env.cons_id => {
                match &args[1] {
                    EvalVal::Bytes(bytes) => result.push(bytes.clone()),
                    other => panic!("expected Bytes list element, got {other:?}"),
                }
                current = &args[2];
            }
            other => panic!("expected List Bytes, got {other:?}"),
        }
    }
}

fn environment_pairs(env: &ElabEnv, value: &EvalVal) -> Vec<(Vec<u8>, Vec<u8>)> {
    let mut current = value;
    let mut result = Vec::new();
    loop {
        match current {
            EvalVal::Ctor { id, .. } if *id == env.prelude_env.nil_id => return result,
            EvalVal::Ctor { id, args, .. } if *id == env.prelude_env.cons_id => {
                match &args[1] {
                    EvalVal::Ctor { id, args, .. }
                        if *id == env.prelude_env.mkprod_id && args.len() == 4 =>
                    {
                        match (&args[2], &args[3]) {
                            (EvalVal::Bytes(key), EvalVal::Bytes(value)) => {
                                result.push((key.clone(), value.clone()));
                            }
                            other => panic!("expected Bytes environment pair, got {other:?}"),
                        }
                    }
                    other => panic!("expected Prod Bytes Bytes, got {other:?}"),
                }
                current = &args[2];
            }
            other => panic!("expected List (Prod Bytes Bytes), got {other:?}"),
        }
    }
}

#[test]
fn both_packages_elaborate_as_transparent_structural_projectors() {
    let env = full_env();
    for name in [
        "process_environment",
        "replace_process_environment",
        "process_environment::round_trip",
        "process_working_directory",
        "replace_process_working_directory",
        "process_working_directory::round_trip",
    ] {
        let id = *env
            .globals
            .get(name)
            .unwrap_or_else(|| panic!("expected checked global `{name}`"));
        assert!(
            env.env.transparent_body(id).is_some(),
            "`{name}` must be a transparent, kernel-checked term"
        );
    }
}

#[test]
fn invalid_environment_bytes_round_trip_and_preserve_argv_and_cwd() {
    let env = full_env();
    let mut store = make_store(&env);
    let invalid_key = vec![0xff, 0x80, b'K'];
    let invalid_value = vec![0xfe, 0xc0, b'V'];
    let source = process_input(
        &env,
        [b"source-argv".to_vec()],
        [(invalid_key.clone(), invalid_value.clone())],
        b"/source".to_vec(),
    );
    let projected = call_global(&env, &mut store, "process_environment", [source]);

    let target_argv = vec![b"target".to_vec(), vec![0x81, b'A']];
    let target_cwd = vec![b'/', 0x82, b'd'];
    let target = process_input(
        &env,
        target_argv.clone(),
        [(b"OLD".to_vec(), b"old".to_vec())],
        target_cwd.clone(),
    );
    let replaced = call_global(
        &env,
        &mut store,
        "replace_process_environment",
        [projected, target],
    );
    let projected_again = call_global(&env, &mut store, "process_environment", [replaced.clone()]);

    assert_eq!(
        environment_pairs(&env, &projected_again),
        vec![(invalid_key, invalid_value)],
        "invalid UTF-8 key and value must survive projection/replacement/projection"
    );
    let fields = process_input_fields(&env, &replaced);
    assert_eq!(list_bytes(&env, &fields[0]), target_argv);
    assert_eq!(fields[2], EvalVal::Bytes(target_cwd));
}

#[test]
fn working_directory_round_trip_preserves_argv_and_environment() {
    let env = full_env();
    let mut store = make_store(&env);
    let arguments = vec![b"app".to_vec(), vec![0x83, b'X']];
    let environment = vec![
        (b"A".to_vec(), b"one".to_vec()),
        (vec![0xff, b'B'], vec![0xfe, b't']),
    ];
    let replacement = vec![b'/', b'w', 0x80, b'd'];
    let input = process_input(
        &env,
        arguments.clone(),
        environment.clone(),
        b"/old".to_vec(),
    );
    let replaced = call_global(
        &env,
        &mut store,
        "replace_process_working_directory",
        [EvalVal::Bytes(replacement.clone()), input],
    );
    let projected = call_global(
        &env,
        &mut store,
        "process_working_directory",
        [replaced.clone()],
    );

    assert_eq!(projected, EvalVal::Bytes(replacement));
    let fields = process_input_fields(&env, &replaced);
    assert_eq!(list_bytes(&env, &fields[0]), arguments);
    assert_eq!(environment_pairs(&env, &fields[1]), environment);
}

#[test]
fn i7_adds_zero_trusted_base_and_no_forbidden_mechanism() {
    for (name, source) in [
        ("Process.Environment", ENVIRONMENT_KEN_MD),
        ("Process.WorkingDirectory", WORKING_DIRECTORY_KEN_MD),
    ] {
        let extracted = ken_elaborator::literate::extract_ken_md(source)
            .unwrap_or_else(|error| panic!("{name} must extract: {error}"));
        for forbidden in [
            "Axiom",
            "primitive",
            "postulate",
            "opaque",
            "bytes_decode",
            "lookup_env",
            "DecEq",
            "Clock",
            "HostHandler",
            "IOOp",
        ] {
            assert!(
                !extracted.source.contains(forbidden),
                "{name} must not contain `{forbidden}`"
            );
        }
        assert!(!extracted.source.contains("data ProcessInput"));
    }

    let mut env = ElabEnv::empty().expect("prelude bootstrap");
    let before: BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    env.elaborate_ken_md_file(ENVIRONMENT_KEN_MD)
        .expect("Process.Environment must elaborate");
    env.elaborate_ken_md_file(WORKING_DIRECTORY_KEN_MD)
        .expect("Process.WorkingDirectory must elaborate");
    let after: BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    assert_eq!(before, after, "I-7 must add zero trusted-base entries");
}
