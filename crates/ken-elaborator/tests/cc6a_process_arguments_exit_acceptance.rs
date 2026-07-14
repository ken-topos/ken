//! CC6a (`Process.Arguments` + `System.Exit`) ordered shared-environment acceptance.

use std::collections::BTreeSet;

use ken_elaborator::{ElabEnv, NumericLitVal};
use ken_interp::eval::{apply, eval, EvalStore, EvalVal};
use ken_kernel::{Decl, GlobalId, Term};

const TRANSPORT_KEN_MD: &str = include_str!("../../../catalog/packages/Core/Transport.ken.md");
const COLLECTIONS_KEN_MD: &str =
    include_str!("../../../catalog/packages/Data/Collections/Collections.ken.md");
const LAWFUL_CLASSES_KEN_MD: &str =
    include_str!("../../../catalog/packages/Core/LawfulClasses.ken.md");
const DIAGNOSTIC_KEN_MD: &str = include_str!("../../../catalog/packages/Diagnostic/Core.ken.md");
const CURSOR_KEN_MD: &str = include_str!("../../../catalog/packages/Parsing/Cursor.ken.md");
const ARGUMENTS_KEN_MD: &str = include_str!("../../../catalog/packages/Process/Arguments.ken.md");
const EXIT_KEN_MD: &str = include_str!("../../../catalog/packages/System/Exit.ken.md");

fn dependency_env() -> ElabEnv {
    let mut env = ElabEnv::empty().expect("prelude bootstrap");
    env.elaborate_ken_md_file(TRANSPORT_KEN_MD)
        .expect("Core.Transport must elaborate first");
    env.elaborate_ken_md_file(COLLECTIONS_KEN_MD)
        .expect("Data.Collections must elaborate second");
    env.elaborate_ken_md_file(LAWFUL_CLASSES_KEN_MD)
        .expect("Core.LawfulClasses must elaborate third");
    env.elaborate_ken_md_file(DIAGNOSTIC_KEN_MD)
        .expect("Diagnostic.Core must elaborate fourth");
    env.elaborate_ken_md_file(CURSOR_KEN_MD)
        .expect("Parsing.Cursor must elaborate fifth");
    env
}

fn full_env() -> ElabEnv {
    let mut env = dependency_env();
    env.elaborate_ken_md_file(ARGUMENTS_KEN_MD)
        .expect("Process.Arguments must elaborate after Parsing.Cursor");
    env.elaborate_ken_md_file(EXIT_KEN_MD)
        .expect("System.Exit must elaborate after Process.Arguments");
    env
}

fn assert_transparent_globals(env: &ElabEnv, names: &[&str]) {
    for name in names {
        let id = *env
            .globals
            .get(*name)
            .unwrap_or_else(|| panic!("expected checked global `{name}`"));
        assert!(
            env.env.transparent_body(id).is_some(),
            "`{name}` must be a real transparent, kernel-checked term"
        );
    }
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

fn eval_global(env: &ElabEnv, store: &mut EvalStore, name: &str) -> EvalVal {
    let id = env.globals[name];
    match env.env.lookup(id) {
        Some(Decl::Transparent { body, .. }) => eval(&[], body, &env.env, store),
        other => panic!("`{name}` must be transparent, got {other:?}"),
    }
}

fn eval_constructor(env: &ElabEnv, store: &mut EvalStore, name: &str) -> EvalVal {
    eval(
        &[],
        &Term::constructor(env.globals[name], vec![]),
        &env.env,
        store,
    )
}

fn apply_values(
    env: &ElabEnv,
    store: &mut EvalStore,
    mut function: EvalVal,
    arguments: impl IntoIterator<Item = EvalVal>,
) -> EvalVal {
    for argument in arguments {
        function = apply(function, argument, &env.env, store);
    }
    function
}

fn call_global(
    env: &ElabEnv,
    store: &mut EvalStore,
    name: &str,
    arguments: impl IntoIterator<Item = EvalVal>,
) -> EvalVal {
    let function = eval_global(env, store, name);
    apply_values(env, store, function, arguments)
}

fn list_value(env: &ElabEnv, store: &mut EvalStore, values: Vec<EvalVal>) -> EvalVal {
    let nil = eval_constructor(env, store, "Nil");
    let mut result = apply_values(env, store, nil, [EvalVal::Neutral]);
    for value in values.into_iter().rev() {
        let cons = eval_constructor(env, store, "Cons");
        result = apply_values(env, store, cons, [EvalVal::Neutral, value, result]);
    }
    result
}

fn nat_value(env: &ElabEnv, store: &mut EvalStore, count: usize) -> EvalVal {
    let mut result = eval_constructor(env, store, "Zero");
    for _ in 0..count {
        let suc = eval_constructor(env, store, "Suc");
        result = apply_values(env, store, suc, [result]);
    }
    result
}

fn list_bytes(env: &ElabEnv, value: &EvalVal) -> Vec<Vec<u8>> {
    let mut current = value;
    let mut result = Vec::new();
    loop {
        match current {
            EvalVal::Ctor { id, .. } if *id == env.globals["Nil"] => return result,
            EvalVal::Ctor { id, args, .. } if *id == env.globals["Cons"] => {
                match &args[1] {
                    EvalVal::Bytes(bytes) => result.push(bytes.clone()),
                    other => panic!("argv element must remain Bytes, got {other:?}"),
                }
                current = &args[2];
            }
            other => panic!("expected List Bytes, got {other:?}"),
        }
    }
}

fn nat_count(env: &ElabEnv, value: &EvalVal) -> u64 {
    match value {
        EvalVal::Ctor { id, args, .. } if *id == env.prelude_env.zero_id && args.is_empty() => 0,
        EvalVal::Ctor { id, args, .. } if *id == env.prelude_env.suc_id && args.len() == 1 => {
            1 + nat_count(env, &args[0])
        }
        other => panic!("expected Nat, got {other:?}"),
    }
}

fn ctor_args<'a>(env: &ElabEnv, value: &'a EvalVal, name: &str) -> &'a [EvalVal] {
    let expected = env.globals[name];
    match value {
        EvalVal::Ctor { id, args, .. } if *id == expected => args.as_ref().as_slice(),
        other => panic!("expected `{name}`, got {other:?}"),
    }
}

#[test]
fn ordered_dependency_closure_elaborates_both_packages() {
    let env = full_env();
    assert_transparent_globals(
        &env,
        &[
            "process_arguments",
            "replace_process_arguments",
            "process_arguments::round_trip",
            "process_argument_at",
            "argument_at",
            "argument_bytes_at",
            "argument_nat_leq",
            "argument_slice_location",
            "exit_success",
            "exit_failure",
            "exit_with",
            "exit_from_result",
        ],
    );
}

#[test]
fn process_arguments_round_trips_genuinely_invalid_utf8_byte_identically() {
    let env = full_env();
    let mut store = make_store(&env);
    let invalid = vec![0xff, 0xfe, 0x80, 0x61];
    let replacement = list_value(
        &env,
        &mut store,
        vec![
            EvalVal::Bytes(b"plain".to_vec()),
            EvalVal::Bytes(invalid.clone()),
        ],
    );
    let original = list_value(&env, &mut store, vec![EvalVal::Bytes(b"old".to_vec())]);
    let empty_environment = list_value(&env, &mut store, Vec::new());
    let mk_process_input = eval_constructor(&env, &mut store, "MkProcessInput");
    let input = apply_values(
        &env,
        &mut store,
        mk_process_input,
        [
            original,
            empty_environment,
            EvalVal::Bytes(b"/work".to_vec()),
        ],
    );

    let replaced = call_global(
        &env,
        &mut store,
        "replace_process_arguments",
        [replacement, input],
    );
    let projected = call_global(&env, &mut store, "process_arguments", [replaced.clone()]);
    assert_eq!(
        list_bytes(&env, &projected),
        vec![b"plain".to_vec(), invalid.clone()],
        "argv replacement/projection must preserve invalid UTF-8 without a String hop"
    );

    let one = nat_value(&env, &mut store, 1);
    let at_one = call_global(&env, &mut store, "process_argument_at", [one, replaced]);
    let some = ctor_args(&env, &at_one, "Some");
    assert_eq!(some.last(), Some(&EvalVal::Bytes(invalid)));
}

#[test]
fn certified_slice_location_keeps_nonzero_argument_and_range() {
    let mut env = full_env();
    env.elaborate_file(
        r#"
        data CC6ArgZero = MkCC6ArgZero
        data CC6ArgOne = MkCC6ArgOne
        data CC6ArgTwo = MkCC6ArgTwo

        const cc6_arg_zero_bytes : Bytes = bytes_encode "a"
        lemma cc6_arg_zero_length : ArgByteLength cc6_arg_zero_bytes (Suc Zero) = Axiom
        instance ArgBytes CC6ArgZero {
          arg_bytes_field = cc6_arg_zero_bytes;
          arg_length_field = Suc Zero;
          arg_length_valid_field = cc6_arg_zero_length
        }

        const cc6_arg_one_bytes : Bytes = bytes_encode "bb"
        lemma cc6_arg_one_length : ArgByteLength cc6_arg_one_bytes (Suc (Suc Zero)) = Axiom
        instance ArgBytes CC6ArgOne {
          arg_bytes_field = cc6_arg_one_bytes;
          arg_length_field = Suc (Suc Zero);
          arg_length_valid_field = cc6_arg_one_length
        }

        const cc6_arg_two_bytes : Bytes = bytes_encode "abcdef"
        lemma cc6_arg_two_length
          : ArgByteLength
              cc6_arg_two_bytes
              (Suc (Suc (Suc (Suc (Suc (Suc Zero)))))) = Axiom
        instance ArgBytes CC6ArgTwo {
          arg_bytes_field = cc6_arg_two_bytes;
          arg_length_field = Suc (Suc (Suc (Suc (Suc (Suc Zero)))));
          arg_length_valid_field = cc6_arg_two_length
        }

        const cc6_arguments : List ArgBytes =
          Cons ArgBytes ArgBytes_instance_CC6ArgZero
            (Cons ArgBytes ArgBytes_instance_CC6ArgOne
              (Cons ArgBytes ArgBytes_instance_CC6ArgTwo (Nil ArgBytes)))

        const cc6_location : Option ArgLocation =
          argument_slice_location
            (Suc (Suc Zero))
            (Suc (Suc (Suc Zero)))
            (Suc (Suc (Suc (Suc (Suc Zero)))))
            cc6_arguments

        const cc6_out_of_bounds : Option ArgLocation =
          argument_slice_location
            (Suc (Suc Zero))
            (Suc (Suc (Suc Zero)))
            (Suc (Suc (Suc (Suc (Suc (Suc (Suc Zero)))))))
            cc6_arguments
        "#,
    )
    .expect("certified argv location probes must elaborate");

    let mut store = make_store(&env);
    for name in [
        "record_nil_val",
        "cc6_arg_zero_length",
        "cc6_arg_one_length",
        "cc6_arg_two_length",
    ] {
        store.num_values.insert(env.globals[name], EvalVal::Neutral);
    }
    let location = eval_global(&env, &mut store, "cc6_location");
    let some = ctor_args(&env, &location, "Some");
    let at = ctor_args(&env, some.last().expect("Some payload"), "MkArgLocation");
    assert_eq!(
        (
            nat_count(&env, &at[0]),
            nat_count(&env, &at[1]),
            nat_count(&env, &at[2]),
        ),
        (2, 3, 5),
        "location must retain both the non-zero argument index and byte range"
    );
    let out_of_bounds = eval_global(&env, &mut store, "cc6_out_of_bounds");
    assert_eq!(ctor_args(&env, &out_of_bounds, "None").len(), 1);
}

#[test]
fn exit_policy_is_total_explicit_and_keeps_uint8_payloads() {
    let mut env = full_env();
    env.elaborate_file(
        r#"
        data CC6Outcome = CC6Okay | CC6Usage | CC6Failed

        fn cc6_policy (outcome : CC6Outcome) : ExitCode =
          match outcome {
            CC6Okay ↦ exit_success;
            CC6Usage ↦ exit_failure (64 : UInt8);
            CC6Failed ↦ exit_failure (70 : UInt8)
          }

        const cc6_okay_exit : ExitCode = exit_with CC6Outcome cc6_policy CC6Okay
        const cc6_usage_exit : ExitCode = exit_with CC6Outcome cc6_policy CC6Usage
        const cc6_failed_exit : ExitCode = exit_with CC6Outcome cc6_policy CC6Failed

        data CC6Problem = CC6BadInput
        fn cc6_problem_code (problem : CC6Problem) : UInt8 = (65 : UInt8)
        const cc6_result_exit : ExitCode =
          exit_from_result
            CC6Problem
            Nat
            cc6_problem_code
            (Err CC6Problem Nat CC6BadInput)
        const cc6_result_okay_exit : ExitCode =
          exit_from_result
            CC6Problem
            Nat
            cc6_problem_code
            (Ok CC6Problem Nat Zero)
        "#,
    )
    .expect("explicit total exit-policy probes must elaborate");

    let mut store = make_store(&env);
    assert!(matches!(
        eval_global(&env, &mut store, "cc6_okay_exit"),
        EvalVal::Ctor { id, .. } if id == env.globals["Success"]
    ));
    assert!(matches!(
        eval_global(&env, &mut store, "cc6_result_okay_exit"),
        EvalVal::Ctor { id, .. } if id == env.globals["Success"]
    ));
    for (name, code) in [
        ("cc6_usage_exit", 64),
        ("cc6_failed_exit", 70),
        ("cc6_result_exit", 65),
    ] {
        let status = eval_global(&env, &mut store, name);
        let failure = ctor_args(&env, &status, "Failure");
        assert_eq!(failure.last(), Some(&EvalVal::Int(code)));
    }
}

#[test]
fn cc6a_has_zero_trust_delta_and_no_new_carrier_or_string_hop() {
    let arguments = ken_elaborator::literate::extract_ken_md(ARGUMENTS_KEN_MD)
        .expect("Process.Arguments must extract");
    let exit =
        ken_elaborator::literate::extract_ken_md(EXIT_KEN_MD).expect("System.Exit must extract");
    for (name, source) in [
        ("Process.Arguments", arguments.source.as_str()),
        ("System.Exit", exit.source.as_str()),
    ] {
        assert!(!source.contains("Axiom"), "{name} must contain no Axiom");
        assert!(
            !source.contains("primitive"),
            "{name} must add no primitive"
        );
        assert!(!source.contains("data ExitCode"));
        assert!(!source.contains("data ProcessInput"));
    }
    assert!(arguments.source.contains("List ArgBytes"));
    assert!(arguments.source.contains("Option ArgLocation"));
    assert!(!arguments.source.contains("class ArgBytes"));
    assert!(!arguments.source.contains("data ArgLocation"));
    assert!(!arguments.source.contains("String"));
    assert!(!arguments.source.contains("bytes_decode"));
    assert!(!arguments.source.contains("bytes_slice"));

    let mut env = dependency_env();
    let before: BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    env.elaborate_ken_md_file(ARGUMENTS_KEN_MD)
        .expect("Process.Arguments must elaborate in the shared environment");
    env.elaborate_ken_md_file(EXIT_KEN_MD)
        .expect("System.Exit must elaborate in the shared environment");
    let after: BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    assert_eq!(before, after, "CC6a must add zero trusted-base entries");
}
