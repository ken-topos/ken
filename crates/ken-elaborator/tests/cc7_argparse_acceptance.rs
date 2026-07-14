//! CC7 (`ArgParse`) ordered shared-environment acceptance.

use std::collections::BTreeSet;
use std::rc::Rc;

use ken_elaborator::{ElabEnv, NumericLitVal};
use ken_interp::eval::{apply, eval, EvalStore, EvalVal, ListCharIds};
use ken_kernel::{Decl, GlobalId, Term};

const TRANSPORT_KEN_MD: &str = include_str!("../../../catalog/packages/Core/Transport.ken.md");
const COLLECTIONS_KEN_MD: &str =
    include_str!("../../../catalog/packages/Data/Collections/Collections.ken.md");
const LAWFUL_CLASSES_KEN_MD: &str =
    include_str!("../../../catalog/packages/Core/LawfulClasses.ken.md");
const LAWFUL_FUNCTORS_KEN_MD: &str =
    include_str!("../../../catalog/packages/Core/LawfulFunctors.ken.md");
const EFFECTFUL_CLASSES_KEN_MD: &str =
    include_str!("../../../catalog/packages/Core/EffectfulClasses.ken.md");
const NONEMPTY_KEN_MD: &str =
    include_str!("../../../catalog/packages/Data/NonEmpty/NonEmpty.ken.md");
const VALIDATION_KEN_MD: &str =
    include_str!("../../../catalog/packages/Data/Validation/Validation.ken.md");
const DIAGNOSTIC_KEN_MD: &str = include_str!("../../../catalog/packages/Diagnostic/Core.ken.md");
const CURSOR_KEN_MD: &str = include_str!("../../../catalog/packages/Parsing/Cursor.ken.md");
const DECODER_KEN_MD: &str = include_str!("../../../catalog/packages/Parsing/Decoder.ken.md");
const STRING_BIJECTION_KEN_MD: &str =
    include_str!("../../../catalog/packages/Data/Collections/StringBijection.ken.md");
const STRING_KEYS_KEN_MD: &str =
    include_str!("../../../catalog/packages/Text/StringKeys/StringKeys.ken.md");
const CODEC_KEN_MD: &str = include_str!("../../../catalog/packages/Text/Codec/Codec.ken.md");
const NUMERIC_KEN_MD: &str = include_str!("../../../catalog/packages/Text/Numeric/Numeric.ken.md");
const PRETTY_KEN_MD: &str = include_str!("../../../catalog/packages/Pretty/Doc.ken.md");
const ARGUMENTS_KEN_MD: &str = include_str!("../../../catalog/packages/Process/Arguments.ken.md");
const EXIT_KEN_MD: &str = include_str!("../../../catalog/packages/System/Exit.ken.md");
const DIAGNOSTIC_RENDER_KEN_MD: &str =
    include_str!("../../../catalog/packages/Diagnostic/Render.ken.md");
const ARGPARSE_KEN_MD: &str = include_str!("../../../catalog/packages/ArgParse/ArgParse.ken.md");
const EXAMPLE_KEN_MD: &str = include_str!("../../../catalog/packages/ArgParse/Example.ken.md");

fn dependency_env() -> ElabEnv {
    let mut env = ElabEnv::empty().expect("prelude bootstrap");
    for (source, label) in [
        (TRANSPORT_KEN_MD, "Core.Transport"),
        (COLLECTIONS_KEN_MD, "Data.Collections"),
        (LAWFUL_CLASSES_KEN_MD, "Core.LawfulClasses"),
        (LAWFUL_FUNCTORS_KEN_MD, "Core.LawfulFunctors"),
        (EFFECTFUL_CLASSES_KEN_MD, "Core.EffectfulClasses"),
        (NONEMPTY_KEN_MD, "Data.NonEmpty"),
        (VALIDATION_KEN_MD, "Data.Validation"),
        (DIAGNOSTIC_KEN_MD, "Diagnostic.Core"),
        (CURSOR_KEN_MD, "Parsing.Cursor"),
        (DECODER_KEN_MD, "Parsing.Decoder"),
        (STRING_BIJECTION_KEN_MD, "Data.Collections.StringBijection"),
        (STRING_KEYS_KEN_MD, "Text.StringKeys"),
        (CODEC_KEN_MD, "Text.Codec"),
        (NUMERIC_KEN_MD, "Text.Numeric"),
        (PRETTY_KEN_MD, "Pretty.Doc"),
        (ARGUMENTS_KEN_MD, "Process.Arguments"),
        (EXIT_KEN_MD, "System.Exit"),
    ] {
        env.elaborate_ken_md_file(source)
            .unwrap_or_else(|err| panic!("{label} must elaborate in dependency order: {err:?}"));
    }
    env
}

fn full_env() -> ElabEnv {
    let mut env = dependency_env();
    env.elaborate_ken_md_file(DIAGNOSTIC_RENDER_KEN_MD)
        .expect("Diagnostic.Render must elaborate after Diagnostic.Core and Pretty.Doc");
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

fn replace_first_bytes(value: EvalVal, bytes: &[u8]) -> (EvalVal, bool) {
    match value {
        EvalVal::Bytes(_) => (EvalVal::Bytes(bytes.to_vec()), true),
        EvalVal::Ctor { id, args, slot } => {
            let mut replaced = false;
            let mut new_args = Vec::with_capacity(args.len());
            for argument in args.iter().cloned() {
                if replaced {
                    new_args.push(argument);
                } else {
                    let (next, did_replace) = replace_first_bytes(argument, bytes);
                    replaced = did_replace;
                    new_args.push(next);
                }
            }
            (
                EvalVal::Ctor {
                    id,
                    args: Rc::new(new_args),
                    slot,
                },
                replaced,
            )
        }
        EvalVal::Pair { fst, snd, slot } => {
            let (next_fst, replaced_fst) = replace_first_bytes((*fst).clone(), bytes);
            let (next_snd, replaced_snd) = if replaced_fst {
                ((*snd).clone(), false)
            } else {
                replace_first_bytes((*snd).clone(), bytes)
            };
            (
                EvalVal::Pair {
                    fst: Rc::new(next_fst),
                    snd: Rc::new(next_snd),
                    slot,
                },
                replaced_fst || replaced_snd,
            )
        }
        other => (other, false),
    }
}

fn add_argument_fixtures(env: &mut ElabEnv) {
    env.elaborate_file(
        r#"
        data CC7Build = MkCC7Build
        const cc7_build_bytes : Bytes = bytes_encode "build"
        lemma cc7_build_length : ArgByteLength cc7_build_bytes (Suc (Suc (Suc (Suc (Suc Zero))))) = Axiom
        instance ArgBytes CC7Build {
          arg_bytes_field = cc7_build_bytes;
          arg_length_field = Suc (Suc (Suc (Suc (Suc Zero))));
          arg_length_valid_field = cc7_build_length
        }

        data CC7Verbose = MkCC7Verbose
        const cc7_verbose_bytes : Bytes = bytes_encode "--verbose"
        lemma cc7_verbose_length : ArgByteLength cc7_verbose_bytes (Suc (Suc (Suc (Suc (Suc (Suc (Suc (Suc (Suc Zero))))))))) = Axiom
        instance ArgBytes CC7Verbose {
          arg_bytes_field = cc7_verbose_bytes;
          arg_length_field = Suc (Suc (Suc (Suc (Suc (Suc (Suc (Suc (Suc Zero))))))));
          arg_length_valid_field = cc7_verbose_length
        }

        data CC7Output = MkCC7Output
        const cc7_output_bytes : Bytes = bytes_encode "--output"
        lemma cc7_output_length : ArgByteLength cc7_output_bytes (Suc (Suc (Suc (Suc (Suc (Suc (Suc (Suc Zero)))))))) = Axiom
        instance ArgBytes CC7Output {
          arg_bytes_field = cc7_output_bytes;
          arg_length_field = Suc (Suc (Suc (Suc (Suc (Suc (Suc (Suc Zero)))))));
          arg_length_valid_field = cc7_output_length
        }

        data CC7OutputValue = MkCC7OutputValue
        const cc7_output_value_bytes : Bytes = bytes_encode "out.bin"
        lemma cc7_output_value_length : ArgByteLength cc7_output_value_bytes (Suc (Suc (Suc (Suc (Suc (Suc (Suc Zero))))))) = Axiom
        instance ArgBytes CC7OutputValue {
          arg_bytes_field = cc7_output_value_bytes;
          arg_length_field = Suc (Suc (Suc (Suc (Suc (Suc (Suc Zero))))));
          arg_length_valid_field = cc7_output_value_length
        }

        data CC7Input = MkCC7Input
        const cc7_input_bytes : Bytes = bytes_encode "input.ken"
        lemma cc7_input_length : ArgByteLength cc7_input_bytes (Suc (Suc (Suc (Suc (Suc (Suc (Suc (Suc (Suc Zero))))))))) = Axiom
        instance ArgBytes CC7Input {
          arg_bytes_field = cc7_input_bytes;
          arg_length_field = Suc (Suc (Suc (Suc (Suc (Suc (Suc (Suc (Suc Zero))))))));
          arg_length_valid_field = cc7_input_length
        }

        data CC7Bogus = MkCC7Bogus
        const cc7_bogus_bytes : Bytes = bytes_encode "--bogus"
        lemma cc7_bogus_length : ArgByteLength cc7_bogus_bytes (Suc (Suc (Suc (Suc (Suc (Suc (Suc Zero))))))) = Axiom
        instance ArgBytes CC7Bogus {
          arg_bytes_field = cc7_bogus_bytes;
          arg_length_field = Suc (Suc (Suc (Suc (Suc (Suc (Suc Zero))))));
          arg_length_valid_field = cc7_bogus_length
        }

        data CC7Wrong = MkCC7Wrong
        const cc7_wrong_bytes : Bytes = bytes_encode "--wrong"
        lemma cc7_wrong_length : ArgByteLength cc7_wrong_bytes (Suc (Suc (Suc (Suc (Suc (Suc (Suc Zero))))))) = Axiom
        instance ArgBytes CC7Wrong {
          arg_bytes_field = cc7_wrong_bytes;
          arg_length_field = Suc (Suc (Suc (Suc (Suc (Suc (Suc Zero))))));
          arg_length_valid_field = cc7_wrong_length
        }

        data CC7Invalid = MkCC7Invalid
        const cc7_invalid_seed_bytes : Bytes = bytes_encode "wxyz"
        lemma cc7_invalid_length : ArgByteLength cc7_invalid_seed_bytes (Suc (Suc (Suc (Suc Zero)))) = Axiom
        instance ArgBytes CC7Invalid {
          arg_bytes_field = cc7_invalid_seed_bytes;
          arg_length_field = Suc (Suc (Suc (Suc Zero)));
          arg_length_valid_field = cc7_invalid_length
        }
        "#,
    )
    .expect("certified ArgBytes fixtures must elaborate at the package boundary");
}

fn fixture(env: &ElabEnv, store: &mut EvalStore, name: &str) -> EvalVal {
    eval_global(env, store, name)
}

fn neutralize_fixture_proofs(env: &ElabEnv, store: &mut EvalStore) {
    for name in [
        "record_nil_val",
        "cc7_build_length",
        "cc7_verbose_length",
        "cc7_output_length",
        "cc7_output_value_length",
        "cc7_input_length",
        "cc7_bogus_length",
        "cc7_wrong_length",
        "cc7_invalid_length",
    ] {
        store.num_values.insert(env.globals[name], EvalVal::Neutral);
    }
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
        "ArgBytes_instance_CC7Build",
        "ArgBytes_instance_CC7Verbose",
        "ArgBytes_instance_CC7Output",
        "ArgBytes_instance_CC7OutputValue",
        "ArgBytes_instance_CC7Input",
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
        "ArgBytes_instance_CC7Build",
        "ArgBytes_instance_CC7Bogus",
        "ArgBytes_instance_CC7Input",
        "ArgBytes_instance_CC7Wrong",
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
    let seed = fixture(&env, &mut store, "ArgBytes_instance_CC7Invalid");
    let (invalid_argument, replaced) = replace_first_bytes(seed.clone(), &invalid);
    assert!(
        replaced,
        "fixture dictionary must expose its raw Bytes field, got {seed:?}"
    );
    let values = vec![
        fixture(&env, &mut store, "ArgBytes_instance_CC7Build"),
        fixture(&env, &mut store, "ArgBytes_instance_CC7Output"),
        invalid_argument,
        fixture(&env, &mut store, "ArgBytes_instance_CC7Input"),
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
    for source in [DIAGNOSTIC_RENDER_KEN_MD, ARGPARSE_KEN_MD, EXAMPLE_KEN_MD] {
        env.elaborate_ken_md_file(source)
            .expect("each CC7 file must elaborate in the ordered environment");
        let extracted = ken_elaborator::literate::extract_ken_md(source)
            .expect("CC7 literate source must extract");
        assert!(!extracted.source.contains("Axiom"));
        assert!(!extracted.source.contains("bytes_eq"));
        assert!(!extracted.source.contains("DecEq Bytes"));
        assert!(!extracted.source.contains("bytes_decode"));
        assert!(!extracted.source.contains("class ArgBytes"));
        assert!(!extracted.source.contains("data ArgCursor"));
        assert!(!extracted.source.contains("data DecoderError"));
    }
    let after: BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    assert_eq!(before, after, "CC7 must add zero trusted-base entries");

    let argparse =
        ken_elaborator::literate::extract_ken_md(ARGPARSE_KEN_MD).expect("ArgParse must extract");
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
