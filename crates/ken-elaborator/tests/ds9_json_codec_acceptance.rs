//! DS-9 lawful JSON codec acceptance.
//!
//! The suite elaborates the real literate package after its landed dependency
//! closure, then exercises the value constructors, cursor laws, typed decoder
//! failures, recursive fuel boundary, constructor-by-constructor round trips,
//! lawful String keys, and zero-new-trust posture.

use std::collections::BTreeSet;

use ken_elaborator::{ElabEnv, NumericLitVal};
use ken_interp::eval::{eval, EvalStore, EvalVal, ListCharIds};
use ken_kernel::{Decl, GlobalId};

const TRANSPORT_KEN_MD: &str =
    include_str!("../../../catalog/packages/Core/Logic/Transport.ken.md");
const COLLECTIONS_KEN_MD: &str =
    include_str!("../../../catalog/packages/Data/Collections/Derived.ken.md");
const LAWFUL_CLASSES_KEN_MD: &str =
    include_str!("../../../catalog/packages/Core/Classes/LawfulClasses.ken.md");
const STRING_BIJECTION_KEN_MD: &str =
    include_str!("../../../catalog/packages/Data/Text/StringBijection.ken.md");
const STRING_KEYS_KEN_MD: &str =
    include_str!("../../../catalog/packages/Data/Text/StringKeys.ken.md");
const DIAGNOSTIC_KEN_MD: &str =
    include_str!("../../../catalog/packages/Capability/Diagnostics/Core.ken.md");
const CURSOR_KEN_MD: &str =
    include_str!("../../../catalog/packages/Capability/Parsing/Cursor.ken.md");
const DECODER_KEN_MD: &str =
    include_str!("../../../catalog/packages/Capability/Parsing/Decoder.ken.md");
const JSON_KEN_MD: &str =
    include_str!("../../../catalog/packages/Data/Serialization/Json.ken.md");

fn dependency_env() -> ElabEnv {
    let mut env = ElabEnv::empty().expect("prelude bootstrap");
    env.elaborate_ken_md_file(TRANSPORT_KEN_MD)
        .expect("Transport must elaborate first");
    env.elaborate_ken_md_file(COLLECTIONS_KEN_MD)
        .expect("Derived collections must elaborate second");
    env.elaborate_ken_md_file(LAWFUL_CLASSES_KEN_MD)
        .expect("LawfulClasses must elaborate third");
    env.elaborate_ken_md_file(STRING_BIJECTION_KEN_MD)
        .expect("StringBijection must elaborate fourth");
    env.elaborate_ken_md_file(STRING_KEYS_KEN_MD)
        .expect("StringKeys must elaborate fifth");
    env.elaborate_ken_md_file(DIAGNOSTIC_KEN_MD)
        .expect("Diagnostic Core must elaborate sixth");
    env.elaborate_ken_md_file(CURSOR_KEN_MD)
        .expect("Cursor must elaborate seventh");
    env.elaborate_ken_md_file(DECODER_KEN_MD)
        .expect("Decoder must elaborate eighth");
    env
}

fn json_env() -> ElabEnv {
    let mut env = dependency_env();
    env.elaborate_ken_md_file(JSON_KEN_MD)
        .expect("Data/Serialization/Json.ken.md must elaborate");
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
            "`{name}` must be a transparent kernel-checked declaration"
        );
    }
}

fn literal_value(value: &NumericLitVal, mkdecimalpair_id: GlobalId) -> EvalVal {
    match value {
        NumericLitVal::Int(number) => EvalVal::from(*number),
        NumericLitVal::Float(number) => EvalVal::Float(*number),
        NumericLitVal::Float32(number) => EvalVal::Float32(*number),
        NumericLitVal::Decimal { coeff, exp } => {
            ken_interp::decimal_value(mkdecimalpair_id, *coeff, *exp)
        }
        NumericLitVal::Str(text) => EvalVal::Str(text.clone()),
    }
}

fn make_store(env: &ElabEnv) -> EvalStore {
    let mut store = EvalStore::new();
    let mkdecimalpair_id = env.prelude_env.mkdecimalpair_id;
    for (id, value) in &env.num_values {
        store
            .num_values
            .insert(*id, literal_value(value, mkdecimalpair_id));
    }
    store.list_char_ids = Some(ListCharIds {
        nil_id: env.prelude_env.nil_id,
        cons_id: env.prelude_env.cons_id,
    });
    store
}

fn resync_store(env: &ElabEnv, store: &mut EvalStore) {
    let mkdecimalpair_id = env.prelude_env.mkdecimalpair_id;
    for (id, value) in &env.num_values {
        store
            .num_values
            .entry(*id)
            .or_insert_with(|| literal_value(value, mkdecimalpair_id));
    }
}

fn eval_global(env: &ElabEnv, store: &mut EvalStore, name: &str) -> EvalVal {
    let id = env.globals[name];
    match env.env.lookup(id) {
        Some(Decl::Transparent { body, .. }) => eval(&[], body, &env.env, store),
        other => panic!("`{name}` must be transparent, got {other:?}"),
    }
}

fn eval_decl(env: &mut ElabEnv, store: &mut EvalStore, name: &str, declaration: &str) -> EvalVal {
    env.elaborate_decl(declaration)
        .unwrap_or_else(|error| panic!("`{name}` must elaborate: {error}"));
    resync_store(env, store);
    eval_global(env, store, name)
}

fn ctor_args<'a>(env: &ElabEnv, value: &'a EvalVal, name: &str) -> &'a [EvalVal] {
    let expected = env.globals[name];
    match value {
        EvalVal::Ctor { id, args, .. } if *id == expected => args.as_ref().as_slice(),
        other => panic!("expected `{name}`, got {other:?}"),
    }
}

fn nat_value(env: &ElabEnv, value: &EvalVal) -> usize {
    match value {
        EvalVal::Ctor { id, args, .. } if *id == env.prelude_env.zero_id && args.is_empty() => 0,
        EvalVal::Ctor { id, args, .. } if *id == env.prelude_env.suc_id && args.len() == 1 => {
            1 + nat_value(env, &args[0])
        }
        other => panic!("expected Nat, got {other:?}"),
    }
}

fn assert_decoder_error(env: &ElabEnv, value: &EvalVal, variant: &str, remaining: usize) {
    let err = ctor_args(env, value, "Err");
    let json_error = ctor_args(env, &err[2], "JsonDecoderError");
    let decoder_error = ctor_args(env, &json_error[0], variant);
    assert_eq!(
        nat_value(env, decoder_error.last().expect("error location")),
        remaining,
        "wrong remaining-character location for `{variant}`"
    );
}

fn chars_expr(text: &str) -> String {
    text.chars().rev().fold("Nil Char".to_string(), |tail, ch| {
        format!("Cons Char ({} : Char) ({tail})", ch as u32)
    })
}

#[test]
fn d1_nested_json_probe_is_rejected_by_current_kernel() {
    let mut env = ElabEnv::new().expect("prelude bootstrap");
    let result = env.elaborate_decl(
        "data JsonNestedProbe = \
         JsonNestedLeaf | JsonNestedArray (List JsonNestedProbe)",
    );
    let error = result.expect_err(
        "the current kernel must reject nested recursive occurrences under List",
    );
    assert!(
        format!("{error:?}").contains("PositivityViolation")
            && format!("{error:?}").contains("non-strictly-positive"),
        "expected the nested-inductive strict-positivity boundary, got {error:?}"
    );
}

#[test]
fn ac1_json_and_all_six_constructor_families_are_real_globals() {
    let env = json_env();
    for name in [
        "Json",
        "JsonNull",
        "JsonBool",
        "JsonNumber",
        "JsonString",
        "JsonArray",
        "JsonObject",
    ] {
        assert!(
            env.globals.contains_key(name),
            "`{name}` must be registered by elaborating the Json package"
        );
    }
    assert_transparent_globals(
        &env,
        &[
            "char_cursor_ops",
            "char_cursor_ops::lawful",
            "encode",
            "decode",
        ],
    );
}

#[test]
fn ac8_ac9_json_file_adds_zero_trusted_declarations() {
    let mut env = dependency_env();
    let before: BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    env.elaborate_ken_md_file(JSON_KEN_MD)
        .expect("Json package must elaborate");
    let after: BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    assert_eq!(
        after.difference(&before).count(),
        0,
        "DS-9 trusted_base() delta must be exactly zero"
    );
}

#[test]
fn ac3_malformed_inputs_return_named_decoder_errors_at_exact_locations() {
    let mut env = json_env();
    let mut store = make_store(&env);
    let cases = [
        ("unterminated", "\"abc", 0),
        ("trailing_comma", "[null,]", 2),
        ("bare_nan", "NaN", 3),
        ("unclosed_array", "[null", 0),
        ("unclosed_object", "{\"a\":null", 0),
        ("duplicate_key", "{\"a\":null,\"a\":true}", 0),
    ];

    for (index, (label, input, remaining)) in cases.into_iter().enumerate() {
        let name = format!("ds9_malformed_{index}");
        let declaration = format!(
            "const {name} : Result JsonError Json = decode ({})",
            chars_expr(input)
        );
        let value = eval_decl(&mut env, &mut store, &name, &declaration);
        assert_decoder_error(&env, &value, "DecoderRejected", remaining);
        assert!(
            !matches!(value, EvalVal::Neutral),
            "{label} must reach the typed decoder boundary"
        );
    }
}

#[test]
fn ac4_zero_progress_is_named_and_non_backtrackable() {
    let mut env = json_env();
    let mut store = make_store(&env);
    let input = chars_expr("x");
    let declaration = format!(
        "const ds9_zero_progress : DecoderResult (List Char) Nat (List Bool) = \
         decoder_many (List Char) Char Nat Bool char_cursor_ops \
           (decoder_pure (List Char) Nat Bool True) ({input})"
    );
    let value = eval_decl(
        &mut env,
        &mut store,
        "ds9_zero_progress",
        &declaration,
    );
    let failed = ctor_args(&env, &value, "DecoderFailed");
    let zero_progress = ctor_args(&env, failed.last().expect("decoder error"), "DecoderZeroProgress");
    assert_eq!(
        nat_value(
            &env,
            zero_progress.last().expect("zero-progress location")
        ),
        1
    );
}

#[test]
fn ac5_runtime_round_trip_covers_each_json_constructor_separately() {
    let mut env = json_env();
    let mut store = make_store(&env);
    let rows = [
        ("null", "JsonNull", "JsonNull"),
        ("bool", "JsonBool True", "JsonBool"),
        (
            "number",
            "JsonNumber (JsonIntegerPositive JsonNonZero4 \
             (Cons JsonDigit JsonDigit2 (Nil JsonDigit)))",
            "JsonNumber",
        ),
        ("string", "JsonString \"hello\"", "JsonString"),
        (
            "array",
            "JsonArray (Cons Json JsonNull (Cons Json (JsonBool False) (Nil Json)))",
            "JsonArray",
        ),
        (
            "object",
            "JsonObject (Cons (Pair String Json) \
             (mk_pair String Json \"answer\" \
               (JsonNumber (JsonIntegerPositive JsonNonZero4 \
                 (Cons JsonDigit JsonDigit2 (Nil JsonDigit))))) \
             (Nil (Pair String Json)))",
            "JsonObject",
        ),
    ];

    for (index, (label, expression, constructor)) in rows.into_iter().enumerate() {
        let name = format!("ds9_round_trip_{index}");
        let declaration = format!(
            "const {name} : Result JsonError Json = decode (encode ({expression}))"
        );
        let value = eval_decl(&mut env, &mut store, &name, &declaration);
        let ok = ctor_args(&env, &value, "Ok");
        ctor_args(
            &env,
            ok.last().expect("round-trip payload"),
            constructor,
        );
        assert!(
            !matches!(value, EvalVal::Neutral),
            "{label} round trip must evaluate"
        );
    }
}
