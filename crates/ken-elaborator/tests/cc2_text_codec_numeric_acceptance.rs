//! CC2 (`Data.Text.Codec` + `Capability.Parsing.Numeric`) acceptance —
//! `docs/program/wp/cc2-text-codec-numeric.md`.
//!
//! The packages consume catalog dependencies. Elaborate the dependency closure
//! in one ordered shared `ElabEnv`, including the separately-homed bijection
//! prerequisite, then elaborate every checked fence.

use std::collections::BTreeSet;

use ken_elaborator::{ElabEnv, NumericLitVal};
use ken_interp::eval::{eval, EvalStore, EvalVal, ListCharIds};
use ken_kernel::{Decl, GlobalId};

const TRANSPORT_KEN_MD: &str = include_str!("../../../catalog/packages/Core/Logic/Transport.ken.md");
const COLLECTIONS_KEN_MD: &str =
    include_str!("../../../catalog/packages/Data/Collections/Derived.ken.md");
const LAWFUL_CLASSES_KEN_MD: &str =
    include_str!("../../../catalog/packages/Core/Classes/LawfulClasses.ken.md");
const DIAGNOSTIC_KEN_MD: &str = include_str!("../../../catalog/packages/Capability/Diagnostics/Core.ken.md");
const STRING_BIJECTION_KEN_MD: &str =
    include_str!("../../../catalog/packages/Data/Text/StringBijection.ken.md");
const STRING_KEYS_KEN_MD: &str =
    include_str!("../../../catalog/packages/Data/Text/StringKeys.ken.md");
const CODEC_KEN_MD: &str = include_str!("../../../catalog/packages/Data/Text/Codec.ken.md");
const NUMERIC_KEN_MD: &str = include_str!("../../../catalog/packages/Capability/Parsing/Numeric.ken.md");
const NUMERIC_SEED: &str = include_str!("../../../conformance/stdlib/text/seed-text-numeric.md");

fn dependency_env() -> ElabEnv {
    let mut env = ElabEnv::empty().expect("prelude bootstrap");
    env.elaborate_ken_md_file(TRANSPORT_KEN_MD)
        .expect("Core/Logic/Transport.ken.md must elaborate first");
    env.elaborate_ken_md_file(COLLECTIONS_KEN_MD)
        .expect("Data/Collections/Derived.ken.md must elaborate second");
    env.elaborate_ken_md_file(LAWFUL_CLASSES_KEN_MD)
        .expect("Core/Classes/LawfulClasses.ken.md must elaborate third");
    env.elaborate_ken_md_file(DIAGNOSTIC_KEN_MD)
        .expect("Capability/Diagnostics/Core.ken.md must elaborate fourth");
    env
}

fn full_env() -> ElabEnv {
    let mut env = dependency_env();
    env.elaborate_ken_md_file(STRING_BIJECTION_KEN_MD)
        .expect("StringBijection prerequisite must elaborate");
    env.elaborate_ken_md_file(STRING_KEYS_KEN_MD)
        .expect("StringKeys must elaborate");
    env.elaborate_ken_md_file(CODEC_KEN_MD)
        .expect("Codec must elaborate");
    env.elaborate_ken_md_file(NUMERIC_KEN_MD)
        .expect("Numeric must elaborate");
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
    store.list_char_ids = Some(ListCharIds {
        nil_id: env.prelude_env.nil_id,
        cons_id: env.prelude_env.cons_id,
    });
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

fn small_int(value: &EvalVal) -> i64 {
    match value {
        EvalVal::Int(n) => *n,
        other => panic!("expected a small Int, got {other:?}"),
    }
}

fn bool_value(env: &ElabEnv, value: &EvalVal) -> bool {
    match value {
        EvalVal::Bool(value) => *value,
        EvalVal::Ctor { id, args, .. } if *id == env.globals["True"] && args.is_empty() => true,
        EvalVal::Ctor { id, args, .. } if *id == env.globals["False"] && args.is_empty() => false,
        other => panic!("expected a Bool, got {other:?}"),
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

#[test]
fn ordered_dependency_closure_elaborates_codec_then_numeric() {
    let mut env = dependency_env();

    env.elaborate_ken_md_file(STRING_BIJECTION_KEN_MD)
        .expect("StringBijection.ken.md must elaborate fourth as the prerequisite");
    env.elaborate_ken_md_file(STRING_KEYS_KEN_MD)
        .expect("Data/Text/StringKeys.ken.md must elaborate fifth");
    assert_transparent_globals(
        &env,
        &[
            "string_to_list_char_injective",
            "string_deceq_eq",
            "string_deceq_eq::sound",
            "string_deceq_eq::complete",
            "DecEq_instance_String",
            "string_ord_leq",
            "string_ord_leq::refl",
            "string_ord_leq::antisym",
            "string_ord_leq::trans",
            "string_ord_leq::total",
            "Ord_instance_String",
        ],
    );

    env.elaborate_ken_md_file(CODEC_KEN_MD)
        .expect("Data/Text/Codec.ken.md and every checked fence must elaborate sixth");
    assert_transparent_globals(
        &env,
        &[
            "decode_utf8",
            "byte_is_ascii",
            "ascii_view",
            "decode_utf8::definition",
            "codec_roundtrip_anchor",
            "ascii_view_none",
            "ascii_view_some",
        ],
    );

    env.elaborate_ken_md_file(NUMERIC_KEN_MD)
        .expect("Capability/Parsing/Numeric.ken.md and every checked fence must elaborate seventh");
    assert_transparent_globals(
        &env,
        &[
            "char_to_digit",
            "numeric_error_code",
            "numeric_diagnostic",
            "numeric_argument_origin",
            "numeric_argument_origin_index_faithful",
            "numeric_argument_origin_start_faithful",
            "numeric_argument_origin_end_faithful",
            "parse_digits_at",
            "parse_nat_chars",
            "parse_int_chars",
            "parse_nat",
            "parse_int",
            "decimal_digit_value",
            "decimal_digit_to_char",
            "decimal_digit_to_char::valid",
            "decimal_digit_values",
            "format_digits",
            "parse_formatted_digits",
            "format_digits_roundtrip",
            "show_digits",
        ],
    );
    for name in ["NumericErrorKind", "Diagnostic", "DecimalDigit"] {
        assert!(
            env.globals.contains_key(name),
            "expected checked data `{name}`"
        );
    }
    assert!(
        !env.globals.contains_key("NumericError"),
        "CC4 must remove the parallel NumericError carrier"
    );
}

#[test]
fn cc2_checked_code_has_zero_axiom_and_zero_trusted_base_delta() {
    for (name, source) in [
        ("StringKeys.ken.md", STRING_KEYS_KEN_MD),
        ("Codec.ken.md", CODEC_KEN_MD),
        ("Numeric.ken.md", NUMERIC_KEN_MD),
    ] {
        let extracted =
            ken_elaborator::literate::extract_ken_md(source).expect("CC2 source must extract");
        assert!(
            !extracted.source.contains("Axiom"),
            "{name}'s tangled checked code must contain no Axiom"
        );
        for range in extracted
            .example_ranges
            .iter()
            .chain(extracted.reject_ranges.iter())
        {
            assert!(
                !source[range.clone()].contains("Axiom"),
                "{name}'s checked example/reject fences must contain no Axiom"
            );
        }
    }

    let mut env = dependency_env();
    env.elaborate_ken_md_file(STRING_BIJECTION_KEN_MD)
        .expect("StringBijection prerequisite must elaborate");
    let before: BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    env.elaborate_ken_md_file(STRING_KEYS_KEN_MD)
        .expect("StringKeys.ken.md must elaborate");
    env.elaborate_ken_md_file(CODEC_KEN_MD)
        .expect("Codec.ken.md must elaborate");
    env.elaborate_ken_md_file(NUMERIC_KEN_MD)
        .expect("Numeric.ken.md must elaborate");
    let after: BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    assert_eq!(
        before, after,
        "CC2's own fences must add no primitive, opaque constant, postulate, or Axiom"
    );
}

#[test]
fn bijection_prerequisite_is_the_single_separately_homed_assumption() {
    let extracted = ken_elaborator::literate::extract_ken_md(STRING_BIJECTION_KEN_MD)
        .expect("StringBijection source must extract");
    assert_eq!(
        extracted
            .source
            .lines()
            .filter(|line| line.starts_with("axiom "))
            .count(),
        1,
        "the prerequisite must contain exactly one named assumption"
    );
    assert!(extracted
        .source
        .contains("axiom string_to_list_char_retraction"));
    assert!(extracted
        .source
        .contains("lemma string_to_list_char_injective"));

    let mut env = dependency_env();
    let before: BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    env.elaborate_ken_md_file(STRING_BIJECTION_KEN_MD)
        .expect("StringBijection prerequisite must elaborate");
    let after: BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    let delta: Vec<_> = after.difference(&before).copied().collect();
    assert_eq!(
        delta.len(),
        1,
        "the prerequisite must add exactly one trusted entry"
    );
    assert_transparent_globals(&env, &["string_to_list_char_injective"]);
}

#[test]
fn located_numeric_discriminators_and_codec_boundary_are_checked() {
    let numeric = ken_elaborator::literate::extract_ken_md(NUMERIC_KEN_MD)
        .expect("Numeric.ken.md must extract");
    for discriminator in [
        "parsed_decimal_result",
        "empty_input_result",
        "bad_digit_result",
        "parsed_negative_result",
        "digit_zero_result",
        "digit_nine_result",
        "letter_digit_result",
    ] {
        assert!(
            NUMERIC_KEN_MD[numeric.example_ranges[0].clone()].contains(discriminator),
            "checked examples must contain `{discriminator}`"
        );
    }

    assert!(CODEC_KEN_MD.contains("lemma ascii_view_none"));
    assert!(CODEC_KEN_MD.contains("lemma ascii_view_some"));
    assert!(CODEC_KEN_MD.contains("const ascii_a_view"));
    assert!(CODEC_KEN_MD.contains("const ascii_a_missing_view"));
    assert!(CODEC_KEN_MD.contains("const utf8_lead_view"));

    assert!(NUMERIC_SEED.contains("text/numeric/valid-decimal-parse"));
    assert!(NUMERIC_SEED.contains("text/numeric/empty-input-located-at-zero"));
    assert!(NUMERIC_SEED.contains("text/numeric/invalid-digit-exact-char-index"));

    let mut env = full_env();
    for declaration in [
        "const cc2_string_key_equal_compute : Bool = list_eq Char eqChar (string_to_list_char \"alpha\") (string_to_list_char \"alpha\")",
        "const cc2_string_key_distinct_compute : Bool = list_eq Char eqChar (string_to_list_char \"alpha\") (string_to_list_char \"beta\")",
        "const cc2_string_key_order_compute : Bool = ord_result_leq (list_compare Char compare_char (string_to_list_char \"alpha\") (string_to_list_char \"beta\"))",
    ] {
        env.elaborate_decl(declaration)
            .expect("equivalent String-key discriminator must elaborate");
    }
    let mut store = make_store(&env);

    assert!(
        bool_value(
            &env,
            &eval_global(&env, &mut store, "cc2_string_key_equal_compute")
        ),
        "equal String keys must compare equal"
    );
    assert!(
        !bool_value(
            &env,
            &eval_global(&env, &mut store, "cc2_string_key_distinct_compute")
        ),
        "distinct String keys must compare unequal"
    );
    assert!(
        bool_value(
            &env,
            &eval_global(&env, &mut store, "cc2_string_key_order_compute")
        ),
        "String ordering must be lexicographic"
    );

    let zero = eval_global(&env, &mut store, "digit_zero_result");
    assert_eq!(small_int(ctor_args(&env, &zero, "Some").last().unwrap()), 0);
    let nine = eval_global(&env, &mut store, "digit_nine_result");
    assert_eq!(small_int(ctor_args(&env, &nine, "Some").last().unwrap()), 9);
    let letter = eval_global(&env, &mut store, "letter_digit_result");
    ctor_args(&env, &letter, "None");

    let parsed = eval_global(&env, &mut store, "parsed_decimal_result");
    assert_eq!(
        small_int(ctor_args(&env, &parsed, "Ok").last().unwrap()),
        123
    );
    let negative = eval_global(&env, &mut store, "parsed_negative_result");
    assert_eq!(
        small_int(ctor_args(&env, &negative, "Ok").last().unwrap()),
        -42
    );

    for (name, expected_code, expected_position) in [
        ("empty_input_result", "text.numeric.empty-input", 0),
        ("bad_digit_result", "text.numeric.invalid-digit", 2),
    ] {
        let result = eval_global(&env, &mut store, name);
        let diagnostic = ctor_args(&env, &result, "Err").last().unwrap();
        let fields = ctor_args(&env, diagnostic, "MkDiagnostic");
        let origin = ctor_args(&env, &fields[0], "ArgumentOrigin");
        assert_eq!(nat_count(&env, &origin[0]), 2);
        let range = ctor_args(&env, &origin[1], "MkByteRange");
        assert_eq!(nat_count(&env, &range[0]), expected_position);
        assert_eq!(nat_count(&env, &range[1]), expected_position);
        let code = ctor_args(&env, &fields[1], "MkDiagnosticCode");
        assert_eq!(code.last(), Some(&EvalVal::Str(expected_code.into())));
    }

    let ascii = eval_global(&env, &mut store, "ascii_a_view");
    assert_eq!(
        ctor_args(&env, &ascii, "Some").last(),
        Some(&EvalVal::Bool(true))
    );
    let missing = eval_global(&env, &mut store, "ascii_a_missing_view");
    ctor_args(&env, &missing, "None");
    let non_ascii = eval_global(&env, &mut store, "utf8_lead_view");
    assert_eq!(
        ctor_args(&env, &non_ascii, "Some").last(),
        Some(&EvalVal::Bool(false))
    );
}

#[test]
fn verified_roundtrip_stays_structural_and_key_instances_stay_out_of_numeric() {
    let extracted = ken_elaborator::literate::extract_ken_md(NUMERIC_KEN_MD)
        .expect("Numeric.ken.md must extract");
    let checked = extracted.source;
    assert!(checked.contains("lemma format_digits_roundtrip"));
    assert!(!checked.contains("instance DecEq String"));
    assert!(!checked.contains("instance Ord String"));
    assert!(!checked.contains("Bytes↔List"));
    assert!(
        !checked.contains("Equal String"),
        "Numeric must not claim a universal String-crossing round trip"
    );
}
