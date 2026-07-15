//! CC7 (`ArgParse`) ordered shared-environment acceptance.

use std::collections::BTreeSet;

use ken_elaborator::{ElabEnv, NumericLitVal};
use ken_interp::eval::{apply, eval, EvalStore, EvalVal, ListCharIds};
use ken_kernel::{Decl, GlobalId, Term};

const TRANSPORT_KEN_MD: &str = include_str!("../../../catalog/packages/Core/Logic/Transport.ken.md");
const COLLECTIONS_KEN_MD: &str =
    include_str!("../../../catalog/packages/Data/Collections/Derived.ken.md");
const LAWFUL_CLASSES_KEN_MD: &str =
    include_str!("../../../catalog/packages/Core/Classes/LawfulClasses.ken.md");
const LAWFUL_FUNCTORS_KEN_MD: &str =
    include_str!("../../../catalog/packages/Core/Classes/LawfulFunctors.ken.md");
const EFFECTFUL_CLASSES_KEN_MD: &str =
    include_str!("../../../catalog/packages/Core/Classes/EffectfulClasses.ken.md");
const NONEMPTY_KEN_MD: &str =
    include_str!("../../../catalog/packages/Data/Collections/NonEmpty.ken.md");
const VALIDATION_KEN_MD: &str =
    include_str!("../../../catalog/packages/Data/Sums/Validation.ken.md");
const DIAGNOSTIC_KEN_MD: &str = include_str!("../../../catalog/packages/Capability/Diagnostics/Core.ken.md");
const CURSOR_KEN_MD: &str = include_str!("../../../catalog/packages/Capability/Parsing/Cursor.ken.md");
const DECODER_KEN_MD: &str = include_str!("../../../catalog/packages/Capability/Parsing/Decoder.ken.md");
const STRING_BIJECTION_KEN_MD: &str =
    include_str!("../../../catalog/packages/Data/Text/StringBijection.ken.md");
const STRING_KEYS_KEN_MD: &str =
    include_str!("../../../catalog/packages/Data/Text/StringKeys.ken.md");
const CODEC_KEN_MD: &str = include_str!("../../../catalog/packages/Data/Text/Codec.ken.md");
const NUMERIC_KEN_MD: &str = include_str!("../../../catalog/packages/Capability/Parsing/Numeric.ken.md");
const PRETTY_KEN_MD: &str = include_str!("../../../catalog/packages/Capability/Formatting/Doc.ken.md");
const ARGUMENTS_KEN_MD: &str = include_str!("../../../catalog/packages/Capability/Process/Arguments.ken.md");
const EXIT_KEN_MD: &str = include_str!("../../../catalog/packages/Capability/Process/Exit.ken.md");
const DIAGNOSTIC_RENDER_KEN_MD: &str =
    include_str!("../../../catalog/packages/Capability/Diagnostics/Render.ken.md");
const SCHEMA_KEN_MD: &str = include_str!("../../../catalog/packages/Application/Input/Schema.ken.md");
const ARGPARSE_KEN_MD: &str = include_str!("../../../catalog/packages/Application/CommandLine/ArgParse.ken.md");
const EXAMPLE_KEN_MD: &str = include_str!("../../../catalog/examples/CommandLine/Forge.ken.md");

fn dependency_env() -> ElabEnv {
    let mut env = ElabEnv::empty().expect("prelude bootstrap");
    for (source, label) in [
        (TRANSPORT_KEN_MD, "Core.Logic.Transport"),
        (COLLECTIONS_KEN_MD, "Data.Collections"),
        (LAWFUL_CLASSES_KEN_MD, "Core.Classes.LawfulClasses"),
        (LAWFUL_FUNCTORS_KEN_MD, "Core.Classes.LawfulFunctors"),
        (EFFECTFUL_CLASSES_KEN_MD, "Core.Classes.EffectfulClasses"),
        (NONEMPTY_KEN_MD, "Data.Collections.NonEmpty"),
        (VALIDATION_KEN_MD, "Data.Sums.Validation"),
        (DIAGNOSTIC_KEN_MD, "Capability.Diagnostics.Core"),
        (CURSOR_KEN_MD, "Capability.Parsing.Cursor"),
        (DECODER_KEN_MD, "Capability.Parsing.Decoder"),
        (STRING_BIJECTION_KEN_MD, "Data.Text.StringBijection"),
        (STRING_KEYS_KEN_MD, "Data.Text.StringKeys"),
        (CODEC_KEN_MD, "Data.Text.Codec"),
        (NUMERIC_KEN_MD, "Capability.Parsing.Numeric"),
        (PRETTY_KEN_MD, "Capability.Formatting.Doc"),
        (ARGUMENTS_KEN_MD, "Capability.Process.Arguments"),
        (EXIT_KEN_MD, "Capability.Process.Exit"),
    ] {
        env.elaborate_ken_md_file(source)
            .unwrap_or_else(|err| panic!("{label} must elaborate in dependency order: {err:?}"));
    }
    env
}

fn full_env() -> ElabEnv {
    let mut env = dependency_env();
    env.elaborate_ken_md_file(DIAGNOSTIC_RENDER_KEN_MD)
        .expect("Capability.Diagnostics.Render must elaborate after Capability.Diagnostics.Core and Capability.Formatting.Doc");
    env.elaborate_ken_md_file(SCHEMA_KEN_MD)
        .expect("Schema must elaborate before either decoder client");
    env.elaborate_ken_md_file(ARGPARSE_KEN_MD)
        .expect("ArgParse must elaborate after the complete substrate");
    env.elaborate_ken_md_file(EXAMPLE_KEN_MD)
        .expect("the separate forge client must elaborate after ArgParse");
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

fn nat_count(env: &ElabEnv, value: &EvalVal) -> usize {
    match value {
        EvalVal::Ctor { id, args, .. } if *id == env.globals["Zero"] => 0,
        EvalVal::Ctor { id, args, .. } if *id == env.globals["Suc"] => 1 + nat_count(env, &args[0]),
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

fn list_elements<'a>(env: &ElabEnv, value: &'a EvalVal) -> Vec<&'a EvalVal> {
    let mut current = value;
    let mut out = Vec::new();
    loop {
        match current {
            EvalVal::Ctor { id, .. } if *id == env.globals["Nil"] => return out,
            EvalVal::Ctor { id, args, .. } if *id == env.globals["Cons"] => {
                out.push(&args[1]);
                current = &args[2];
            }
            other => panic!("expected List, got {other:?}"),
        }
    }
}

fn list_char_text(env: &ElabEnv, value: &EvalVal) -> String {
    list_elements(env, value)
        .into_iter()
        .map(|value| match value {
            EvalVal::Int(codepoint) => {
                char::from_u32(*codepoint as u32).expect("valid rendered Char")
            }
            other => panic!("expected Char, got {other:?}"),
        })
        .collect()
}

fn add_argument_fixtures(env: &mut ElabEnv) {
    env.elaborate_file(
        r#"
        const cc7_build_bytes : Bytes = bytes_encode "build"
        const cc7_inspect_bytes : Bytes = bytes_encode "inspect"
        const cc7_verbose_bytes : Bytes = bytes_encode "--verbose"
        const cc7_output_bytes : Bytes = bytes_encode "--output"
        const cc7_output_value_bytes : Bytes = bytes_encode "out.bin"
        const cc7_input_bytes : Bytes = bytes_encode "input.ken"
        const cc7_bogus_bytes : Bytes = bytes_encode "--bogus"
        const cc7_wrong_bytes : Bytes = bytes_encode "--wrong"
        const cc7_invalid_seed_bytes : Bytes = bytes_encode "wxyz"
        "#,
    )
    .expect("raw Bytes fixtures must elaborate at the package boundary");
}

fn fixture(env: &ElabEnv, store: &mut EvalStore, name: &str) -> EvalVal {
    eval_global(env, store, name)
}

fn neutralize_fixture_proofs(env: &ElabEnv, store: &mut EvalStore) {
    store
        .num_values
        .insert(env.globals["record_nil_val"], EvalVal::Neutral);
}

fn parsed_arguments<'a>(env: &ElabEnv, result: &'a EvalVal) -> Vec<&'a EvalVal> {
    let valid = ctor_args(env, result, "Valid");
    let command = ctor_args(env, valid.last().expect("Valid payload"), "MkParsedCommand");
    list_elements(env, command.last().expect("parsed argument list"))
}

fn diagnostic_location(env: &ElabEnv, diagnostic: &EvalVal) -> (usize, usize, usize) {
    let diagnostic = ctor_args(env, diagnostic, "MkDiagnostic");
    let origin = ctor_args(env, &diagnostic[0], "ArgumentOrigin");
    let range = ctor_args(env, &origin[1], "MkByteRange");
    (
        nat_count(env, &origin[0]),
        nat_count(env, &range[0]),
        nat_count(env, &range[1]),
    )
}

fn invalid_diagnostics<'a>(env: &ElabEnv, result: &'a EvalVal) -> Vec<&'a EvalVal> {
    let invalid = ctor_args(env, result, "Invalid");
    let nonempty = ctor_args(
        env,
        invalid.last().expect("Invalid payload"),
        "NonEmptyCons",
    );
    let mut result = vec![&nonempty[1]];
    result.extend(list_elements(env, &nonempty[2]));
    result
}

#[test]
fn ordered_closure_elaborates_the_renderer_specialization_and_multifile_client() {
    let env = full_env();
    for name in [
        "diagnostic_to_doc",
        "argparse_name_decoder",
        "argparse_matches_chars",
        "argparse_find_option",
        "argparse_parse_tokens",
        "argparse_run",
        "command_help",
        "program_help",
        "forge_parse",
        "forge_help",
    ] {
        let id = env
            .globals
            .get(name)
            .copied()
            .unwrap_or_else(|| panic!("missing `{name}`"));
        assert!(
            env.env.transparent_body(id).is_some(),
            "`{name}` must be transparent and kernel checked"
        );
    }
}

#[test]
fn forge_parses_flags_raw_values_and_positionals_and_renders_derived_help() {
    let mut env = full_env();
    add_argument_fixtures(&mut env);
    let mut store = make_store(&env);
    neutralize_fixture_proofs(&env, &mut store);
    let arguments = [
        "cc7_build_bytes",
        "cc7_verbose_bytes",
        "cc7_output_bytes",
        "cc7_output_value_bytes",
        "cc7_input_bytes",
    ]
    .into_iter()
    .map(|name| fixture(&env, &mut store, name))
    .collect();
    let arguments = list_value(&env, &mut store, arguments);
    let parsed = call_global(&env, &mut store, "forge_parse", [arguments]);
    let values = parsed_arguments(&env, &parsed);
    assert_eq!(values.len(), 3);
    assert!(matches!(values[0], EvalVal::Ctor { id, .. } if *id == env.globals["ParsedFlag"]));
    let option = ctor_args(&env, values[1], "ParsedOption");
    assert_eq!(option.last(), Some(&EvalVal::Bytes(b"out.bin".to_vec())));
    let positional = ctor_args(&env, values[2], "ParsedPositional");
    assert_eq!(
        positional.last(),
        Some(&EvalVal::Bytes(b"input.ken".to_vec()))
    );

    let inspect = fixture(&env, &mut store, "cc7_inspect_bytes");
    let inspect_input = fixture(&env, &mut store, "cc7_input_bytes");
    let inspect_arguments = list_value(&env, &mut store, vec![inspect, inspect_input]);
    let inspect_parsed = call_global(&env, &mut store, "forge_parse", [inspect_arguments]);
    let inspect_command = ctor_args(
        &env,
        ctor_args(&env, &inspect_parsed, "Valid")
            .last()
            .expect("Valid payload"),
        "MkParsedCommand",
    );
    assert_eq!(inspect_command[0], EvalVal::Str("inspect".to_owned()));

    let width = nat_value(&env, &mut store, 0);
    let root_help = eval_global(&env, &mut store, "forge_help");
    let root_rendered = call_global(&env, &mut store, "render", [width.clone(), root_help]);
    let root_text = list_char_text(&env, &root_rendered);
    assert!(root_text.contains("build"));
    assert!(root_text.contains("inspect"));
    let build_spec = eval_global(&env, &mut store, "forge_build_spec");
    let build_help = call_global(&env, &mut store, "command_help", [build_spec]);
    let build_rendered = call_global(&env, &mut store, "render", [width, build_help]);
    let build_text = list_char_text(&env, &build_rendered);
    assert!(build_text.contains("--verbose"));
    assert!(build_text.contains("--output <value>"));
    assert!(build_text.contains("<input>"));
}

#[test]
fn two_independent_bad_arguments_accumulate_exact_nonzero_locations() {
    let mut env = full_env();
    add_argument_fixtures(&mut env);
    let mut store = make_store(&env);
    neutralize_fixture_proofs(&env, &mut store);
    let arguments = [
        "cc7_build_bytes",
        "cc7_bogus_bytes",
        "cc7_input_bytes",
        "cc7_wrong_bytes",
    ]
    .into_iter()
    .map(|name| fixture(&env, &mut store, name))
    .collect();
    let arguments = list_value(&env, &mut store, arguments);
    let parsed = call_global(&env, &mut store, "forge_parse", [arguments]);
    let diagnostics = invalid_diagnostics(&env, &parsed);
    assert_eq!(diagnostics.len(), 2, "Validation must not short-circuit");
    assert_eq!(diagnostic_location(&env, diagnostics[0]), (1, 2, 7));
    assert_eq!(diagnostic_location(&env, diagnostics[1]), (3, 2, 7));

    let first_doc = call_global(
        &env,
        &mut store,
        "diagnostic_to_doc",
        [diagnostics[0].clone()],
    );
    let zero = nat_value(&env, &mut store, 0);
    let rendered = call_global(&env, &mut store, "render", [zero, first_doc]);
    let rendered = list_char_text(&env, &rendered);
    assert!(rendered.contains("argument"));
    assert!(rendered.contains("unknown-option"));
}

#[test]
fn invalid_utf8_option_value_survives_byte_identically() {
    let mut env = full_env();
    add_argument_fixtures(&mut env);
    let mut store = make_store(&env);
    neutralize_fixture_proofs(&env, &mut store);
    let invalid = vec![0xff, 0xfe, 0x80, 0x61];
    let invalid_argument = EvalVal::Bytes(invalid.clone());
    let values = vec![
        fixture(&env, &mut store, "cc7_build_bytes"),
        fixture(&env, &mut store, "cc7_output_bytes"),
        invalid_argument,
        fixture(&env, &mut store, "cc7_input_bytes"),
    ];
    let arguments = list_value(&env, &mut store, values);
    let parsed = call_global(&env, &mut store, "forge_parse", [arguments]);
    let values = parsed_arguments(&env, &parsed);
    let option = ctor_args(&env, values[0], "ParsedOption");
    assert_eq!(option.last(), Some(&EvalVal::Bytes(invalid)));
}

#[test]
fn adding_one_option_to_the_spec_changes_help_without_a_second_help_edit() {
    let mut env = full_env();
    env.elaborate_file(
        r#"
        const cc7_color_option : OptionSpec =
          MkOptionSpec "color" (None String) FlagOption "color"

        const cc7_help_probe_without_color : CommandSpec =
          MkCommandSpec "probe" "probe" (Nil OptionSpec) (Nil PositionalSpec)

        const cc7_help_probe_with_color : CommandSpec =
          MkCommandSpec
            "probe"
            "probe"
            (Cons OptionSpec cc7_color_option (Nil OptionSpec))
            (Nil PositionalSpec)

        const cc7_help_without_color : List Char =
          render Zero (command_help cc7_help_probe_without_color)

        const cc7_help_with_color : List Char =
          render Zero (command_help cc7_help_probe_with_color)
        "#,
    )
    .expect("single-source-of-truth help-growth probe must elaborate");
    let mut store = make_store(&env);
    let before = eval_global(&env, &mut store, "cc7_help_without_color");
    let after = eval_global(&env, &mut store, "cc7_help_with_color");
    assert!(!list_char_text(&env, &before).contains("--color"));
    assert!(list_char_text(&env, &after).contains("--color"));
}

#[test]
fn cc7_is_a_zero_trust_specialization_with_no_second_universe() {
    let mut env = dependency_env();
    let before: BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    for source in [
        DIAGNOSTIC_RENDER_KEN_MD,
        SCHEMA_KEN_MD,
        ARGPARSE_KEN_MD,
        EXAMPLE_KEN_MD,
    ] {
        env.elaborate_ken_md_file(source)
            .expect("each CC7 file must elaborate in the ordered environment");
        let extracted = ken_elaborator::literate::extract_ken_md(source)
            .expect("CC7 literate source must extract");
        assert!(!extracted.source.contains("Axiom"));
        assert!(!extracted.source.contains("bytes_eq"));
        assert!(!extracted.source.contains("DecEq Bytes"));
        assert!(!extracted.source.contains("bytes_decode"));
        assert!(!extracted.source.contains("class ArgBytes"));
        assert!(!extracted.source.contains("ArgByteLength"));
        assert!(!extracted.source.contains("data ArgCursor"));
        assert!(!extracted.source.contains("data DecoderError"));
        let emitted_names: BTreeSet<_> = extracted
            .source
            .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
            .filter(|token| !token.is_empty())
            .collect();
        for forbidden in ["bytes_length", "bytes_slice", "bytes_at"] {
            assert!(
                !emitted_names.contains(forbidden),
                "CC7 structural consumer path must not name `{forbidden}`"
            );
        }
    }
    let after: BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    assert_eq!(before, after, "CC7 must add zero trusted-base entries");

    let argparse =
        ken_elaborator::literate::extract_ken_md(ARGPARSE_KEN_MD).expect("ArgParse must extract");
    assert!(
        argparse.source.contains(
            "fn argparse_byte_matches_char (actual : UInt8) (expected : Char) : Bool =\n  eq_int (uint8_to_int actual) (charToInt expected)"
        ),
        "SUB-2 must leave the byte-comparison path unchanged"
    );
    for required in [
        "Decoder ArgCursor ArgLocation",
        "arg_cursor_ops",
        "Validation (NonEmpty Diagnostic)",
        "Semigroup_instance_NonEmpty Diagnostic",
        "ArgumentOrigin",
        "Bytes",
        "Doc",
    ] {
        assert!(
            argparse.source.contains(required),
            "ArgParse must visibly consume `{required}`"
        );
    }
}
