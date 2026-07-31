//! CC8 ordered shared-environment acceptance for Schema's two real clients.

use std::collections::BTreeSet;
use std::rc::Rc;

use ken_elaborator::{ElabEnv, NumericLitVal};
use ken_interp::eval::{apply, eval, EvalStore, EvalVal, ListCharIds};
use ken_kernel::{Decl, GlobalId};

const TRANSPORT: &str = include_str!("../../../catalog/packages/Core/Logic/Transport.ken.md");
const COLLECTIONS: &str = include_str!("../../../catalog/packages/Data/Collections/Derived.ken.md");
const LAWFUL_CLASSES: &str =
    include_str!("../../../catalog/packages/Core/Classes/LawfulClasses.ken.md");
const BYTES_KEYS: &str = include_str!("../../../catalog/packages/Data/Binary/BytesKeys.ken.md");
const LAWFUL_FUNCTORS: &str =
    include_str!("../../../catalog/packages/Core/Classes/LawfulFunctors.ken.md");
const EFFECTFUL_CLASSES: &str =
    include_str!("../../../catalog/packages/Core/Classes/EffectfulClasses.ken.md");
const NONEMPTY: &str = include_str!("../../../catalog/packages/Data/Collections/NonEmpty.ken.md");
const VALIDATION: &str = include_str!("../../../catalog/packages/Data/Sums/Validation.ken.md");
const DIAGNOSTIC: &str =
    include_str!("../../../catalog/packages/Capability/Diagnostics/Core.ken.md");
const CURSOR: &str = include_str!("../../../catalog/packages/Capability/Parsing/Cursor.ken.md");
const DECODER: &str = include_str!("../../../catalog/packages/Capability/Parsing/Decoder.ken.md");
const STRING_BIJECTION: &str =
    include_str!("../../../catalog/packages/Data/Text/StringBijection.ken.md");
const STRING_KEYS: &str = include_str!("../../../catalog/packages/Data/Text/StringKeys.ken.md");
const CODEC: &str = include_str!("../../../catalog/packages/Data/Text/Codec.ken.md");
const NUMERIC: &str = include_str!("../../../catalog/packages/Capability/Parsing/Numeric.ken.md");
const PRETTY: &str = include_str!("../../../catalog/packages/Capability/Formatting/Doc.ken.md");
const ARGUMENTS: &str =
    include_str!("../../../catalog/packages/Capability/Process/Arguments.ken.md");
const ENVIRONMENT: &str =
    include_str!("../../../catalog/packages/Capability/Process/Environment.ken.md");
const EXIT: &str = include_str!("../../../catalog/packages/Capability/Process/Exit.ken.md");
const DIAGNOSTIC_RENDER: &str =
    include_str!("../../../catalog/packages/Capability/Diagnostics/Render.ken.md");
const SCHEMA: &str = include_str!("../../../catalog/packages/Application/Input/Schema.ken.md");
const ARGPARSE: &str =
    include_str!("../../../catalog/packages/Application/CommandLine/ArgParse.ken.md");
const CONFIG_DECODER: &str =
    include_str!("../../../catalog/packages/Application/Configuration/Decoder.ken.md");
const EXAMPLE: &str = include_str!("../../../catalog/examples/CommandLine/Forge.ken.md");

fn dependency_env() -> ElabEnv {
    let mut env = ElabEnv::empty().expect("prelude bootstrap");
    for (source, label) in [
        (TRANSPORT, "Core.Logic.Transport"),
        (COLLECTIONS, "Data.Collections"),
        (LAWFUL_CLASSES, "Core.Classes.LawfulClasses"),
        (BYTES_KEYS, "Data.Binary.BytesKeys"),
        (LAWFUL_FUNCTORS, "Core.Classes.LawfulFunctors"),
        (EFFECTFUL_CLASSES, "Core.Classes.EffectfulClasses"),
        (NONEMPTY, "Data.Collections.NonEmpty"),
        (VALIDATION, "Data.Sums.Validation"),
        (DIAGNOSTIC, "Capability.Diagnostics.Core"),
        (CURSOR, "Capability.Parsing.Cursor"),
        (DECODER, "Capability.Parsing.Decoder"),
        (STRING_BIJECTION, "Data.Text.StringBijection"),
        (STRING_KEYS, "Data.Text.StringKeys"),
        (CODEC, "Data.Text.Codec"),
        (NUMERIC, "Capability.Parsing.Numeric"),
        (PRETTY, "Capability.Formatting.Doc"),
        (ARGUMENTS, "Capability.Process.Arguments"),
        (ENVIRONMENT, "Capability.Process.Environment"),
        (EXIT, "Capability.Process.Exit"),
        (DIAGNOSTIC_RENDER, "Capability.Diagnostics.Render"),
    ] {
        env.elaborate_ken_md_file(source)
            .unwrap_or_else(|err| panic!("{label} must elaborate in dependency order: {err:?}"));
    }
    env
}

fn full_env() -> ElabEnv {
    let mut env = dependency_env();
    for (source, label) in [
        (SCHEMA, "Schema"),
        (ARGPARSE, "ArgParse"),
        (CONFIG_DECODER, "Application.Configuration.Decoder"),
        (EXAMPLE, "Application.CommandLine.Forge"),
    ] {
        env.elaborate_ken_md_file(source)
            .unwrap_or_else(|err| panic!("{label} must elaborate in dependency order: {err:?}"));
    }
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
    for (id, value) in &env.num_values {
        store
            .num_values
            .insert(*id, lit_to_eval(value, env.prelude_env.mkdecimalpair_id));
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

fn constructor_value(id: GlobalId, args: Vec<EvalVal>) -> EvalVal {
    EvalVal::Ctor {
        id,
        args: Rc::new(args),
        slot: 0,
    }
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
    environment: impl IntoIterator<Item = (Vec<u8>, Vec<u8>)>,
) -> EvalVal {
    constructor_value(
        env.globals["MkProcessInput"],
        vec![
            list_value(env, std::iter::empty()),
            environment_value(env, environment),
            EvalVal::Bytes(b"/cc8".to_vec()),
        ],
    )
}

fn ctor_args<'a>(env: &ElabEnv, value: &'a EvalVal, name: &str) -> &'a [EvalVal] {
    match value {
        EvalVal::Ctor { id, args, .. } if *id == env.globals[name] => args.as_slice(),
        other => panic!("expected `{name}`, got {other:?}"),
    }
}

fn list_elements<'a>(env: &ElabEnv, value: &'a EvalVal) -> Vec<&'a EvalVal> {
    let mut current = value;
    let mut result = Vec::new();
    loop {
        match current {
            EvalVal::Ctor { id, .. } if *id == env.prelude_env.nil_id => return result,
            EvalVal::Ctor { id, args, .. } if *id == env.prelude_env.cons_id => {
                result.push(&args[1]);
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
            EvalVal::Int(codepoint) => char::from_u32(*codepoint as u32).expect("valid Char"),
            other => panic!("expected Char, got {other:?}"),
        })
        .collect()
}

fn add_schema_fixtures(env: &mut ElabEnv) {
    env.elaborate_file(
        r#"
        const cc8_host_field : SchemaField =
          MkSchemaField "HOST" SchemaRequired SchemaBytes "service host"
        const cc8_token_field : SchemaField =
          MkSchemaField "TOKEN" SchemaRequired SchemaBytes "access token"
        const cc8_color_field : SchemaField =
          MkSchemaField "COLOR" SchemaOptional SchemaBytes "color mode"
        const cc8_base_schema : Schema = MkSchema
          "service"
          "service configuration"
          (Cons SchemaField cc8_host_field (Cons SchemaField cc8_token_field (Nil SchemaField)))
        const cc8_grown_schema : Schema = MkSchema
          "service"
          "service configuration"
          (Cons
            SchemaField
            cc8_host_field
            (Cons SchemaField cc8_token_field (Cons SchemaField cc8_color_field (Nil SchemaField))))
        const cc8_arg_color : OptionSpec =
          MkOptionSpec "color" (None String) ValueOption "color mode"
        const cc8_arg_before : CommandSpec =
          MkCommandSpec "serve" "serve" (Nil OptionSpec) (Nil PositionalSpec)
        const cc8_arg_after : CommandSpec =
          MkCommandSpec
            "serve"
            "serve"
            (Cons OptionSpec cc8_arg_color (Nil OptionSpec))
            (Nil PositionalSpec)
        "#,
    )
    .expect("CC8 behavioral fixtures must elaborate");
}

#[test]
fn ordered_closure_elaborates_schema_before_both_clients() {
    let env = full_env();
    for name in [
        "schema_validate",
        "schema_help",
        "command_schema",
        "argparse_missing_positionals",
        "decode_process_environment",
        "decode_config_entries",
        "env_config_help",
    ] {
        let id = env.globals[name];
        assert!(
            env.env.transparent_body(id).is_some(),
            "`{name}` must be transparent and kernel checked"
        );
    }
}

#[test]
fn schema_help_growth_reaches_both_clients_behaviorally() {
    let mut env = full_env();
    add_schema_fixtures(&mut env);
    let mut store = make_store(&env);

    let width = constructor_value(env.globals["Zero"], vec![]);
    let base = eval_global(&env, &mut store, "cc8_base_schema");
    let grown = eval_global(&env, &mut store, "cc8_grown_schema");
    let base_doc = call_global(&env, &mut store, "env_config_help", [base]);
    let grown_doc = call_global(&env, &mut store, "env_config_help", [grown]);
    let base_text = list_char_text(
        &env,
        &call_global(&env, &mut store, "render", [width.clone(), base_doc]),
    );
    let grown_text = list_char_text(
        &env,
        &call_global(&env, &mut store, "render", [width.clone(), grown_doc]),
    );
    assert!(!base_text.contains("COLOR"));
    assert!(grown_text.contains("COLOR"));

    let before = eval_global(&env, &mut store, "cc8_arg_before");
    let after = eval_global(&env, &mut store, "cc8_arg_after");
    let before_doc = call_global(&env, &mut store, "command_help", [before]);
    let after_doc = call_global(&env, &mut store, "command_help", [after]);
    let before_text = list_char_text(
        &env,
        &call_global(&env, &mut store, "render", [width.clone(), before_doc]),
    );
    let after_text = list_char_text(
        &env,
        &call_global(&env, &mut store, "render", [width, after_doc]),
    );
    assert!(!before_text.contains("--color"));
    assert!(after_text.contains("--color"));
}

#[test]
fn invalid_utf8_environment_value_survives_the_full_pipeline() {
    let mut env = full_env();
    add_schema_fixtures(&mut env);
    let mut store = make_store(&env);
    let invalid = vec![0xff, 0xfe, 0x80, b'a'];
    let input = process_input(
        &env,
        [
            (b"HOST".to_vec(), invalid.clone()),
            (b"TOKEN".to_vec(), b"secret".to_vec()),
        ],
    );
    let schema = eval_global(&env, &mut store, "cc8_base_schema");
    let result = call_global(
        &env,
        &mut store,
        "decode_process_environment",
        [schema, input],
    );
    let valid = ctor_args(&env, &result, "Valid");
    let values = list_elements(&env, valid.last().expect("Valid payload"));
    assert_eq!(values[0], &EvalVal::Bytes(invalid));
}

#[test]
fn two_missing_fields_accumulate_exact_environment_origins() {
    let mut env = full_env();
    add_schema_fixtures(&mut env);
    let mut store = make_store(&env);
    let schema = eval_global(&env, &mut store, "cc8_base_schema");
    let input = process_input(&env, std::iter::empty());
    let result = call_global(
        &env,
        &mut store,
        "decode_process_environment",
        [schema, input],
    );
    let invalid = ctor_args(&env, &result, "Invalid");
    let errors = ctor_args(
        &env,
        invalid.last().expect("Invalid payload"),
        "NonEmptyCons",
    );
    let mut diagnostics = vec![&errors[1]];
    diagnostics.extend(list_elements(&env, &errors[2]));
    let origins: Vec<_> = diagnostics
        .into_iter()
        .map(|diagnostic| {
            let diagnostic = ctor_args(&env, diagnostic, "MkDiagnostic");
            let origin = ctor_args(&env, &diagnostic[0], "EnvironmentOrigin");
            match origin.last() {
                Some(EvalVal::Str(name)) => name.clone(),
                other => panic!("expected environment name, got {other:?}"),
            }
        })
        .collect();
    assert_eq!(origins, ["HOST", "TOKEN"]);
}

#[test]
fn config_failures_keep_config_key_origins_distinct_from_environment() {
    let mut env = full_env();
    add_schema_fixtures(&mut env);
    let mut store = make_store(&env);
    let schema = eval_global(&env, &mut store, "cc8_base_schema");
    let entries = environment_value(&env, std::iter::empty());
    let result = call_global(&env, &mut store, "decode_config_entries", [schema, entries]);
    let invalid = ctor_args(&env, &result, "Invalid");
    let errors = ctor_args(
        &env,
        invalid.last().expect("Invalid payload"),
        "NonEmptyCons",
    );
    let mut diagnostics = vec![&errors[1]];
    diagnostics.extend(list_elements(&env, &errors[2]));
    let paths: Vec<_> = diagnostics
        .into_iter()
        .map(|diagnostic| {
            let diagnostic = ctor_args(&env, diagnostic, "MkDiagnostic");
            let origin = ctor_args(&env, &diagnostic[0], "ConfigKeyOrigin");
            list_elements(&env, origin.last().expect("config key path"))
                .into_iter()
                .map(|part| match part {
                    EvalVal::Str(text) => text.clone(),
                    other => panic!("expected String config path part, got {other:?}"),
                })
                .collect::<Vec<_>>()
        })
        .collect();
    assert_eq!(paths, [vec!["HOST".to_owned()], vec!["TOKEN".to_owned()]]);
}

#[test]
fn cc8_adds_no_trust_and_declares_no_second_universe() {
    let mut env = dependency_env();
    let before_trust: BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    for (source, label) in [
        (SCHEMA, "Schema"),
        (ARGPARSE, "ArgParse"),
        (CONFIG_DECODER, "Application.Configuration.Decoder"),
    ] {
        let before_globals: BTreeSet<_> = env.globals.values().copied().collect();
        env.elaborate_ken_md_file(source)
            .unwrap_or_else(|err| panic!("{label} must elaborate: {err:?}"));
        let new_globals: BTreeSet<_> = env
            .globals
            .values()
            .copied()
            .filter(|id| !before_globals.contains(id))
            .collect();
        assert!(!new_globals.is_empty(), "{label} must emit declarations");
        for id in new_globals {
            assert!(
                !matches!(
                    env.env.lookup(id),
                    Some(Decl::Opaque { .. } | Decl::Primitive { .. })
                ),
                "{label} emitted a trusted declaration: {id:?}"
            );
        }
    }
    let after_trust: BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    assert_eq!(
        before_trust, after_trust,
        "CC8 must add zero trusted entries"
    );

    let checked = [SCHEMA, ARGPARSE, CONFIG_DECODER]
        .into_iter()
        .map(|source| {
            ken_elaborator::literate::extract_ken_md(source).expect("extract checked Ken")
        })
        .map(|extracted| extracted.source)
        .collect::<Vec<_>>()
        .join("\n");
    for forbidden in [
        "ArgBytes",
        "ArgByteLength",
        "bytes_eq",
        "bytes_length",
        "bytes_at",
        "data DecoderError",
        "data Doc",
        "Axiom",
    ] {
        assert!(
            !checked.contains(forbidden),
            "forbidden CC8 surface `{forbidden}`"
        );
    }
}
