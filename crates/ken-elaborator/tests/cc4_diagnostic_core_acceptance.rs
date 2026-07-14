//! CC4 (`Diagnostic.Core`) ordered shared-environment acceptance.

use std::collections::BTreeSet;

use ken_elaborator::{ElabEnv, ElabError, NumericLitVal};
use ken_interp::eval::{eval, EvalStore, EvalVal, ListCharIds};
use ken_kernel::{Decl, GlobalId};

const TRANSPORT_KEN_MD: &str = include_str!("../../../catalog/packages/Core/Transport.ken.md");
const COLLECTIONS_KEN_MD: &str =
    include_str!("../../../catalog/packages/Data/Collections/Collections.ken.md");
const LAWFUL_CLASSES_KEN_MD: &str =
    include_str!("../../../catalog/packages/Core/LawfulClasses.ken.md");
const DIAGNOSTIC_KEN_MD: &str = include_str!("../../../catalog/packages/Diagnostic/Core.ken.md");
const CURSOR_KEN_MD: &str = include_str!("../../../catalog/packages/Parsing/Cursor.ken.md");
const DECODER_KEN_MD: &str = include_str!("../../../catalog/packages/Parsing/Decoder.ken.md");
const PARSING_KEN_MD: &str =
    include_str!("../../../catalog/packages/Capability/Parsing/Parsing.ken.md");
const NUMERIC_KEN_MD: &str = include_str!("../../../catalog/packages/Text/Numeric/Numeric.ken.md");

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

fn full_env() -> ElabEnv {
    let mut env = dependency_env();
    env.elaborate_ken_md_file(DIAGNOSTIC_KEN_MD)
        .expect("Diagnostic.Core must elaborate fourth");
    env.elaborate_ken_md_file(CURSOR_KEN_MD)
        .expect("Parsing.Cursor must elaborate fifth");
    env.elaborate_ken_md_file(DECODER_KEN_MD)
        .expect("Parsing.Decoder must elaborate sixth");
    env.elaborate_ken_md_file(PARSING_KEN_MD)
        .expect("Capability.Parsing must elaborate seventh");
    env.elaborate_ken_md_file(NUMERIC_KEN_MD)
        .expect("Text.Numeric must elaborate eighth");
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

fn nat_count(env: &ElabEnv, value: &EvalVal) -> u64 {
    match value {
        EvalVal::Ctor { id, args, .. } if *id == env.prelude_env.zero_id && args.is_empty() => 0,
        EvalVal::Ctor { id, args, .. } if *id == env.prelude_env.suc_id && args.len() == 1 => {
            1 + nat_count(env, &args[0])
        }
        other => panic!("expected Nat, got {other:?}"),
    }
}

#[test]
fn ordered_dependency_closure_elaborates_all_cc4_clients() {
    let env = full_env();
    assert_transparent_globals(
        &env,
        &[
            "byte_range_start",
            "byte_range_end",
            "origin_source_id",
            "origin_argument_index",
            "origin_range_start",
            "origin_range_end",
            "environment_origin",
            "config_key_origin",
            "diagnostic_origin",
            "diagnostic_code",
            "ValidByteRange",
            "ValidConfigKeyPath",
            "ValidOrigin",
            "ValidDiagnostic",
            "arg_location_origin",
            "arg_location_origin_index_faithful",
            "arg_location_origin_start_faithful",
            "arg_location_origin_end_faithful",
            "span_to_byte_range",
            "span_origin",
            "span_to_byte_range_faithful",
            "span_origin_source_faithful",
            "numeric_error_code",
            "numeric_diagnostic",
            "numeric_argument_origin",
            "numeric_argument_origin_index_faithful",
            "numeric_argument_origin_start_faithful",
            "numeric_argument_origin_end_faithful",
        ],
    );

    for name in [
        "SourceId",
        "ByteRange",
        "Origin",
        "DiagnosticCode",
        "Diagnostic",
    ] {
        assert!(
            env.globals.contains_key(name),
            "expected checked data `{name}`"
        );
    }
    assert!(
        !env.globals.contains_key("NumericError"),
        "Text.Numeric must not retain its pre-CC4 carrier"
    );
    assert!(
        !PARSING_KEN_MD.contains("data SourceId ="),
        "SourceId must move down instead of surviving as a CAT-5 duplicate"
    );
    assert!(
        !DECODER_KEN_MD.contains("Diagnostic") && !DECODER_KEN_MD.contains("Origin"),
        "the location-generic Decoder must remain independent of Diagnostic.Core"
    );
}

#[test]
fn checked_cc4_chain_has_zero_axiom_and_zero_trusted_base_delta() {
    for (name, source) in [
        ("Diagnostic/Core.ken.md", DIAGNOSTIC_KEN_MD),
        ("Parsing/Cursor.ken.md", CURSOR_KEN_MD),
        ("Capability/Parsing.ken.md", PARSING_KEN_MD),
        ("Text/Numeric.ken.md", NUMERIC_KEN_MD),
    ] {
        let extracted =
            ken_elaborator::literate::extract_ken_md(source).expect("CC4 source must extract");
        assert!(
            !extracted.source.contains("Axiom"),
            "{name} must contain no checked Axiom"
        );
    }

    let mut env = dependency_env();
    let before: BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    env.elaborate_ken_md_file(DIAGNOSTIC_KEN_MD)
        .expect("Diagnostic.Core must elaborate");
    env.elaborate_ken_md_file(CURSOR_KEN_MD)
        .expect("Parsing.Cursor must elaborate");
    env.elaborate_ken_md_file(DECODER_KEN_MD)
        .expect("Parsing.Decoder must elaborate");
    env.elaborate_ken_md_file(PARSING_KEN_MD)
        .expect("Capability.Parsing must elaborate");
    env.elaborate_ken_md_file(NUMERIC_KEN_MD)
        .expect("Text.Numeric must elaborate");
    let after: BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    assert_eq!(before, after, "CC4 must add zero trusted-base entries");
}

#[test]
fn exact_non_degenerate_injections_preserve_every_location_field() {
    let mut env = full_env();
    env.elaborate_file(
        r#"
        const cc4_source_origin : Origin =
          span_origin
            (MkSourceId (Suc (Suc (Suc (Suc Zero)))))
            (MkSpan (Suc (Suc Zero)) (Suc (Suc (Suc (Suc (Suc Zero))))))

        const cc4_argument_origin : Origin =
          arg_location_origin
            (MkArgLocation
              (Suc (Suc Zero))
              (Suc (Suc (Suc Zero)))
              (Suc (Suc (Suc Zero))))

        lemma cc4_valid_range :
            ValidByteRange
              (MkByteRange
                (Suc (Suc Zero))
                (Suc (Suc (Suc (Suc (Suc Zero)))))) =
          Proved

        const cc4_environment_name : String = "PATH"

        lemma cc4_valid_environment :
            ValidOrigin (environment_origin cc4_environment_name) =
          Proved
        "#,
    )
    .expect("non-degenerate CC4 probes must elaborate");

    let invalid = env.elaborate_decl(
        "lemma cc4_invalid_range : ValidByteRange (MkByteRange (Suc Zero) Zero) = Proved",
    );
    assert!(
        matches!(invalid, Err(ElabError::KernelRejected { .. })),
        "start > end must fail specifically at kernel checking, got {invalid:?}"
    );
    let empty_config = env.elaborate_decl(
        "lemma cc4_empty_config : ValidOrigin (config_key_origin (Nil String)) = Proved",
    );
    assert!(
        matches!(empty_config, Err(ElabError::KernelRejected { .. })),
        "an empty config key path must fail at kernel checking, got {empty_config:?}"
    );

    let mut store = make_store(&env);

    let source = eval_global(&env, &mut store, "cc4_source_origin");
    let source_fields = ctor_args(&env, &source, "SourceOrigin");
    let source_id = ctor_args(&env, &source_fields[0], "MkSourceId");
    assert_eq!(nat_count(&env, &source_id[0]), 4);
    let source_range = ctor_args(&env, &source_fields[1], "MkByteRange");
    assert_eq!(nat_count(&env, &source_range[0]), 2);
    assert_eq!(nat_count(&env, &source_range[1]), 5);

    let argument = eval_global(&env, &mut store, "cc4_argument_origin");
    let argument_fields = ctor_args(&env, &argument, "ArgumentOrigin");
    assert_eq!(nat_count(&env, &argument_fields[0]), 2);
    let argument_range = ctor_args(&env, &argument_fields[1], "MkByteRange");
    assert_eq!(nat_count(&env, &argument_range[0]), 3);
    assert_eq!(nat_count(&env, &argument_range[1]), 3);

    let numeric = eval_global(&env, &mut store, "bad_digit_result");
    let diagnostic = ctor_args(&env, &numeric, "Err").last().unwrap();
    let diagnostic_fields = ctor_args(&env, diagnostic, "MkDiagnostic");
    let numeric_origin = ctor_args(&env, &diagnostic_fields[0], "ArgumentOrigin");
    assert_eq!(nat_count(&env, &numeric_origin[0]), 2);
    let numeric_range = ctor_args(&env, &numeric_origin[1], "MkByteRange");
    assert_eq!(nat_count(&env, &numeric_range[0]), 2);
    assert_eq!(nat_count(&env, &numeric_range[1]), 2);
    let code = ctor_args(&env, &diagnostic_fields[1], "MkDiagnosticCode");
    assert_eq!(
        code.last(),
        Some(&EvalVal::Str("text.numeric.invalid-digit".into()))
    );
}

#[test]
fn diagnostic_core_is_structured_and_render_free() {
    let extracted = ken_elaborator::literate::extract_ken_md(DIAGNOSTIC_KEN_MD)
        .expect("Diagnostic.Core must extract");
    let checked = extracted.source;
    assert!(checked.contains("data Diagnostic = MkDiagnostic Origin DiagnosticCode"));
    assert!(checked.contains("data Origin ="));
    for constructor in [
        "SourceOrigin SourceId ByteRange",
        "ArgumentOrigin Nat ByteRange",
        "EnvironmentOrigin String",
        "ConfigKeyOrigin (List String)",
    ] {
        assert!(checked.contains(constructor), "missing `{constructor}`");
    }
    for forbidden in ["fn show", "format", "render", "width", "layout", "message"] {
        assert!(
            !checked.contains(forbidden),
            "Diagnostic.Core must remain presentation-neutral: found `{forbidden}`"
        );
    }
}
