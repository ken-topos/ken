//! CC3 (`Parsing.Cursor` + `Parsing.Decoder`) ordered shared-environment acceptance.

use std::collections::BTreeSet;

use ken_elaborator::{ElabEnv, NumericLitVal};
use ken_interp::eval::{eval, EvalStore, EvalVal};
use ken_kernel::{Decl, GlobalId};

const TRANSPORT_KEN_MD: &str = include_str!("../../../catalog/packages/Core/Transport.ken.md");
const COLLECTIONS_KEN_MD: &str =
    include_str!("../../../catalog/packages/Data/Collections/Collections.ken.md");
const LAWFUL_CLASSES_KEN_MD: &str =
    include_str!("../../../catalog/packages/Core/LawfulClasses.ken.md");
const CURSOR_KEN_MD: &str = include_str!("../../../catalog/packages/Parsing/Cursor.ken.md");
const DECODER_KEN_MD: &str = include_str!("../../../catalog/packages/Parsing/Decoder.ken.md");
const PARSING_KEN_MD: &str =
    include_str!("../../../catalog/packages/Capability/Parsing/Parsing.ken.md");

fn dependency_env() -> ElabEnv {
    let mut env = ElabEnv::empty().expect("prelude bootstrap");
    env.elaborate_ken_md_file(TRANSPORT_KEN_MD)
        .expect("Transport must elaborate first");
    env.elaborate_ken_md_file(COLLECTIONS_KEN_MD)
        .expect("Collections must elaborate second");
    env.elaborate_ken_md_file(LAWFUL_CLASSES_KEN_MD)
        .expect("LawfulClasses must elaborate third");
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

fn ctor_args<'a>(env: &ElabEnv, value: &'a EvalVal, name: &str) -> &'a [EvalVal] {
    let expected = env.globals[name];
    match value {
        EvalVal::Ctor { id, args, .. } if *id == expected => args.as_ref().as_slice(),
        other => panic!("expected `{name}`, got {other:?}"),
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

fn list_count(env: &ElabEnv, value: &EvalVal) -> u64 {
    match value {
        EvalVal::Ctor { id, .. } if *id == env.globals["Nil"] => 0,
        EvalVal::Ctor { id, args, .. } if *id == env.globals["Cons"] && args.len() >= 3 => {
            1 + list_count(env, &args[2])
        }
        other => panic!("expected List, got {other:?}"),
    }
}

fn nat_expr(n: usize) -> String {
    let mut result = "Zero".to_string();
    for _ in 0..n {
        result = format!("Suc ({result})");
    }
    result
}

fn neutralize_fixture_proofs(env: &ElabEnv, store: &mut EvalStore, names: &[&str]) {
    for name in names {
        let id = env
            .globals
            .get(*name)
            .copied()
            .unwrap_or_else(|| panic!("{name} should be in scope"));
        store.num_values.insert(id, EvalVal::Neutral);
    }
}

#[test]
fn ordered_dependency_closure_elaborates_cursor_then_decoder() {
    let mut env = dependency_env();
    env.elaborate_ken_md_file(CURSOR_KEN_MD)
        .expect("Parsing.Cursor must elaborate after the core closure");
    assert_transparent_globals(
        &env,
        &[
            "cursor_remaining",
            "cursor_peek",
            "cursor_advance",
            "cursor_locate",
            "arg_length::valid",
            "arg_cursor_remaining",
            "arg_cursor_peek",
            "arg_cursor_advance",
            "arg_cursor_locate",
            "arg_cursor_ops",
            "CursorLaws",
        ],
    );

    env.elaborate_ken_md_file(DECODER_KEN_MD)
        .expect("Parsing.Decoder must elaborate after Parsing.Cursor");
    assert_transparent_globals(
        &env,
        &[
            "decoder_error_location",
            "decoder_pure",
            "decoder_fail",
            "decoder_map",
            "decoder_bind",
            "decoder_seq",
            "decoder_alt",
            "decoder_satisfy",
            "decoder_token",
            "decoder_many_fuel",
            "decoder_many",
            "decoder_some",
            "decoder_recursive_fuel",
            "decoder_recursive",
            "DecoderProgress",
            "DecoderRejectsOnlyAtEnd",
            "DecoderManyConsumesAllLaw",
        ],
    );

    env.elaborate_ken_md_file(PARSING_KEN_MD)
        .expect("Capability.Parsing must elaborate after Parsing.Decoder");
    assert_transparent_globals(
        &env,
        &[
            "byte_cursor_remaining",
            "byte_cursor_peek",
            "byte_cursor_advance",
            "byte_cursor_locate",
            "byte_cursor_ops",
            "parser_from_decoder",
            "parser_pure",
            "parser_fail",
            "byte_code_decoder",
            "spaces_decoder",
            "bool_decoder_layer",
            "bool_expression_decoder",
            "complete_bool_decoder",
            "parse_bool_expr",
        ],
    );
}

#[test]
fn cc3_checked_code_has_zero_axiom_and_zero_trusted_base_delta() {
    for (name, source) in [
        ("Cursor.ken.md", CURSOR_KEN_MD),
        ("Decoder.ken.md", DECODER_KEN_MD),
        ("Capability/Parsing.ken.md", PARSING_KEN_MD),
    ] {
        let extracted =
            ken_elaborator::literate::extract_ken_md(source).expect("CC3 source must extract");
        assert!(
            !extracted.source.contains("Axiom"),
            "{name}'s checked code must contain no Axiom"
        );
    }

    let mut env = dependency_env();
    let before: BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    env.elaborate_ken_md_file(CURSOR_KEN_MD)
        .expect("Parsing.Cursor must elaborate");
    env.elaborate_ken_md_file(DECODER_KEN_MD)
        .expect("Parsing.Decoder must elaborate");
    env.elaborate_ken_md_file(PARSING_KEN_MD)
        .expect("Capability.Parsing must elaborate");
    let after: BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    assert_eq!(before, after, "CC3 must add zero trusted-base entries");
}

#[test]
fn repetition_progress_and_arg_locations_are_discriminating() {
    std::thread::Builder::new()
        .name("cc3-progress-discriminators".into())
        .stack_size(256 * 1024 * 1024)
        .spawn(repetition_progress_and_arg_locations_impl)
        .expect("spawn CC3 large-stack test")
        .join()
        .expect("CC3 progress discriminator thread");
}

fn repetition_progress_and_arg_locations_impl() {
    let mut env = full_env();
    let long_len = 96;
    let long_nat = nat_expr(long_len);
    let long_bytes = "x".repeat(long_len);
    env.elaborate_file(&format!(
        r#"
        data LongArg = MkLongArg
        data ArgZero = MkArgZero
        data ArgOne = MkArgOne
        data ArgTwo = MkArgTwo

        const long_arg_bytes : Bytes = bytes_encode "{long_bytes}"
        lemma long_arg_length_valid : ArgByteLength long_arg_bytes ({long_nat}) = Axiom
        instance ArgBytes LongArg {{
          arg_bytes_field = long_arg_bytes ;
          arg_length_field = ({long_nat}) ;
          arg_length_valid_field = long_arg_length_valid
        }}
        const long_arg : ArgBytes = ArgBytes_instance_LongArg
        const long_args : List ArgBytes = Cons ArgBytes long_arg (Nil ArgBytes)
        const long_start : ArgCursor = arg_cursor_start long_args

        fn arg_code_decoder (code : Int) : Decoder ArgCursor ArgLocation UInt8 =
          decoder_satisfy
            ArgCursor
            UInt8
            ArgLocation
            arg_cursor_ops
            (\byte. eq_int (uint8_to_int byte) code)

        const long_x_step : Decoder ArgCursor ArgLocation UInt8 = arg_code_decoder (120 : Int)
        const long_many : Decoder ArgCursor ArgLocation (List UInt8) =
          decoder_many ArgCursor UInt8 ArgLocation UInt8 arg_cursor_ops long_x_step
        const long_many_result : DecoderResult ArgCursor ArgLocation (List UInt8) =
          long_many long_start

        const zero_progress_many : Decoder ArgCursor ArgLocation (List Bool) =
          decoder_many
            ArgCursor
            UInt8
            ArgLocation
            Bool
            arg_cursor_ops
            (decoder_pure ArgCursor ArgLocation Bool True)
        const zero_progress_result : DecoderResult ArgCursor ArgLocation (List Bool) =
          zero_progress_many long_start

        const arg_zero_bytes : Bytes = bytes_encode "a"
        lemma arg_zero_length_valid : ArgByteLength arg_zero_bytes (Suc Zero) = Axiom
        instance ArgBytes ArgZero {{
          arg_bytes_field = arg_zero_bytes ;
          arg_length_field = Suc Zero ;
          arg_length_valid_field = arg_zero_length_valid
        }}

        const arg_one_bytes : Bytes = bytes_encode "bb"
        lemma arg_one_length_valid : ArgByteLength arg_one_bytes (Suc (Suc Zero)) = Axiom
        instance ArgBytes ArgOne {{
          arg_bytes_field = arg_one_bytes ;
          arg_length_field = Suc (Suc Zero) ;
          arg_length_valid_field = arg_one_length_valid
        }}

        const arg_two_bytes : Bytes = bytes_encode "cccX"
        lemma arg_two_length_valid
          : ArgByteLength arg_two_bytes (Suc (Suc (Suc (Suc Zero)))) = Axiom
        instance ArgBytes ArgTwo {{
          arg_bytes_field = arg_two_bytes ;
          arg_length_field = Suc (Suc (Suc (Suc Zero))) ;
          arg_length_valid_field = arg_two_length_valid
        }}

        const multi_args : List ArgBytes =
          Cons
            ArgBytes
            ArgBytes_instance_ArgZero
            (Cons
              ArgBytes
              ArgBytes_instance_ArgOne
              (Cons ArgBytes ArgBytes_instance_ArgTwo (Nil ArgBytes)))

        const location_probe : Decoder ArgCursor ArgLocation UInt8 =
          decoder_seq
            ArgCursor
            ArgLocation
            UInt8
            UInt8
            (arg_code_decoder (97 : Int))
            (decoder_seq
              ArgCursor
              ArgLocation
              UInt8
              UInt8
              (arg_code_decoder (98 : Int))
              (decoder_seq
                ArgCursor
                ArgLocation
                UInt8
                UInt8
                (arg_code_decoder (98 : Int))
                (decoder_seq
                  ArgCursor
                  ArgLocation
                  UInt8
                  UInt8
                  (arg_code_decoder (99 : Int))
                  (decoder_seq
                    ArgCursor
                    ArgLocation
                    UInt8
                    UInt8
                    (arg_code_decoder (99 : Int))
                    (decoder_seq
                      ArgCursor
                      ArgLocation
                      UInt8
                      UInt8
                      (arg_code_decoder (99 : Int))
                      (arg_code_decoder (122 : Int)))))))

        const location_probe_result : DecoderResult ArgCursor ArgLocation UInt8 =
          location_probe (arg_cursor_start multi_args)
        "#,
    ))
    .expect("CC3 progress and ArgCursor fixtures must elaborate");

    let mut store = make_store(&env);
    neutralize_fixture_proofs(
        &env,
        &mut store,
        &[
            "record_nil_val",
            "long_arg_length_valid",
            "arg_zero_length_valid",
            "arg_one_length_valid",
            "arg_two_length_valid",
        ],
    );

    let long_result = eval_global(&env, &mut store, "long_many_result");
    let long_args = ctor_args(&env, &long_result, "Decoded");
    assert_eq!(list_count(&env, &long_args[3]), long_len as u64);
    let long_end = ctor_args(&env, &long_args[4], "MkArgCursor");
    assert_eq!(
        (nat_count(&env, &long_end[1]), nat_count(&env, &long_end[2])),
        (1, 0),
        "long repetition must consume all 96 bytes and land exactly after the final argument"
    );

    let zero_progress = eval_global(&env, &mut store, "zero_progress_result");
    let zero_failed = ctor_args(&env, &zero_progress, "DecoderFailed");
    let zero_error = ctor_args(&env, &zero_failed[3], "DecoderZeroProgress");
    let zero_at = ctor_args(&env, &zero_error[1], "MkArgLocation");
    assert_eq!(
        (nat_count(&env, &zero_at[0]), nat_count(&env, &zero_at[1])),
        (0, 0),
        "many of a zero-consumption decoder must fail on the named zero-progress variant"
    );

    let location_result = eval_global(&env, &mut store, "location_probe_result");
    let location_failed = ctor_args(&env, &location_result, "DecoderFailed");
    let rejected = ctor_args(&env, &location_failed[3], "DecoderRejected");
    let at = ctor_args(&env, &rejected[1], "MkArgLocation");
    assert_eq!(
        (
            nat_count(&env, &at[0]),
            nat_count(&env, &at[1]),
            nat_count(&env, &at[2]),
        ),
        (2, 3, 3),
        "ArgCursor rejection must retain non-zero argument index and non-zero byte offset"
    );
}

#[allow(dead_code)]
fn full_env() -> ElabEnv {
    let mut env = dependency_env();
    env.elaborate_ken_md_file(CURSOR_KEN_MD)
        .expect("Parsing.Cursor must elaborate");
    env.elaborate_ken_md_file(DECODER_KEN_MD)
        .expect("Parsing.Decoder must elaborate");
    env.elaborate_ken_md_file(PARSING_KEN_MD)
        .expect("Capability.Parsing must elaborate after Decoder");
    env
}
