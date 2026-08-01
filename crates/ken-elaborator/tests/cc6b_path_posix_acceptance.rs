//! CC6b `Capability.Filesystem.Path.Posix` acceptance.

use std::collections::BTreeSet;

use ken_elaborator::{ElabEnv, NumericLitVal};
use ken_interp::eval::{apply, eval, EvalStore, EvalVal, ListCharIds};
use ken_kernel::{Decl, GlobalId};

const TRANSPORT: &str = include_str!("../../../catalog/packages/Core/Logic/Transport.ken.md");
const COLLECTIONS: &str =
    include_str!("../../../catalog/packages/Data/Collections/Derived.ken.md");
const LAWFUL_CLASSES: &str = include_str!("../../../catalog/packages/Core/Classes/LawfulClasses.ken.md");
const LAWFUL_FUNCTORS: &str = include_str!("../../../catalog/packages/Core/Classes/LawfulFunctors.ken.md");
const BYTES_KEYS: &str =
    include_str!("../../../catalog/packages/Data/Binary/BytesKeys.ken.md");
const PATH_POSIX: &str = include_str!("../../../catalog/packages/Capability/Filesystem/Path/Posix.ken.md");

fn dependency_env() -> ElabEnv {
    let mut env = ElabEnv::new().expect("prelude bootstrap");
    for (name, source) in [
        ("Core.Logic.Transport", TRANSPORT),
        ("Data.Collections", COLLECTIONS),
        ("Core.Classes.LawfulClasses", LAWFUL_CLASSES),
        ("Core.Classes.LawfulFunctors", LAWFUL_FUNCTORS),
        ("Data.Binary.BytesKeys", BYTES_KEYS),
    ] {
        env.elaborate_ken_md_file(source)
            .unwrap_or_else(|error| panic!("{name} must elaborate: {error}"));
    }
    env
}

fn full_env() -> ElabEnv {
    let mut env = dependency_env();
    env.elaborate_ken_md_file(PATH_POSIX)
        .expect("Capability.Filesystem.Path.Posix must elaborate after its declared dependencies");
    env
}

fn literal_value(value: &NumericLitVal, mkdecimalpair_id: GlobalId) -> EvalVal {
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
            .insert(*id, literal_value(value, env.prelude_env.mkdecimalpair_id));
    }
    store.list_char_ids = Some(ListCharIds {
        nil_id: env.prelude_env.nil_id,
        cons_id: env.prelude_env.cons_id,
    });
    store
}

fn resync_literals(env: &ElabEnv, store: &mut EvalStore) {
    for (id, value) in &env.num_values {
        store
            .num_values
            .entry(*id)
            .or_insert_with(|| literal_value(value, env.prelude_env.mkdecimalpair_id));
    }
}

fn eval_global(env: &ElabEnv, store: &mut EvalStore, name: &str) -> EvalVal {
    match env.env.lookup(env.globals[name]) {
        Some(Decl::Transparent { body, .. }) => eval(&[], body, &env.env, store),
        other => panic!("`{name}` must be transparent, got {other:?}"),
    }
}

fn bool_value(env: &ElabEnv, value: &EvalVal) -> bool {
    match value {
        EvalVal::Ctor { id, .. } if *id == env.globals["True"] => true,
        EvalVal::Ctor { id, .. } if *id == env.globals["False"] => false,
        EvalVal::Bool(value) => *value,
        other => panic!("expected Bool, got {other:?}"),
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

fn path_parts<'a>(env: &ElabEnv, value: &'a EvalVal) -> (bool, &'a EvalVal) {
    match value {
        EvalVal::Ctor { id, args, .. } if *id == env.globals["MkPath"] => {
            let absolute = match &args[0] {
                EvalVal::Ctor { id, .. } if *id == env.globals["True"] => true,
                EvalVal::Ctor { id, .. } if *id == env.globals["False"] => false,
                EvalVal::Bool(value) => *value,
                other => panic!("Path absolute flag must be Bool, got {other:?}"),
            };
            (absolute, &args[1])
        }
        other => panic!("expected Path, got {other:?}"),
    }
}

fn list_segments(env: &ElabEnv, value: &EvalVal) -> Vec<Vec<u8>> {
    let mut segments = Vec::new();
    let mut current = value;
    loop {
        match current {
            EvalVal::Ctor { id, .. } if *id == env.globals["Nil"] => return segments,
            EvalVal::Ctor { id, args, .. } if *id == env.globals["Cons"] => {
                let mut bytes = Vec::new();
                let mut segment = &args[1];
                loop {
                    match segment {
                        EvalVal::Ctor { id, .. } if *id == env.globals["Nil"] => break,
                        EvalVal::Ctor { id, args, .. } if *id == env.globals["Cons"] => {
                            match &args[1] {
                                EvalVal::Int(byte) => {
                                    bytes.push(u8::try_from(*byte).expect("UInt8"))
                                }
                                other => panic!("segment byte must be UInt8, got {other:?}"),
                            }
                            segment = &args[2];
                        }
                        other => panic!("segment must be List UInt8, got {other:?}"),
                    }
                }
                segments.push(bytes);
                current = &args[2];
            }
            other => panic!("segments must be List (List UInt8), got {other:?}"),
        }
    }
}

#[test]
fn ordered_dependency_closure_elaborates_path_package() {
    let env = full_env();
    for name in [
        "Path",
        "path_parse",
        "path_render",
        "path_normalize",
        "path_join",
        "path_parent",
        "path_is_absolute",
        "path_valid",
        "path_split_render_segments",
        "path_parse_render_valid",
        "path_parse_valid",
        "path_parse_render_parse",
        "path_normalize_idempotent",
        "path_normalize_has_no_dot",
        "path_normalize_absolute_has_no_dotdot",
    ] {
        assert!(env.globals.contains_key(name), "missing `{name}`");
    }
}

#[test]
fn parse_render_and_normalize_are_byte_exact_at_posix_edges() {
    let env = full_env();
    let mut store = make_store(&env);
    let cases: &[(&[u8], bool, &[&[u8]], &[u8])] = &[
        (b"", false, &[], b""),
        (b"/", true, &[], b"/"),
        (b"a//b/", false, &[b"a", b"b"], b"a/b"),
        (b"/..", true, &[], b"/"),
        (b"../a", false, &[b"..", b"a"], b"../a"),
        (b"a/../../b", false, &[b"..", b"b"], b"../b"),
        (b"../..", false, &[b"..", b".."], b"../.."),
    ];
    for (raw, absolute, normalized_segments, rendered) in cases {
        let parsed = call_global(
            &env,
            &mut store,
            "path_parse",
            [EvalVal::Bytes(raw.to_vec())],
        );
        if *raw == b"a//b/" {
            let (_, parsed_segments) = path_parts(&env, &parsed);
            let first = match parsed_segments {
                EvalVal::Ctor { id, args, .. } if *id == env.globals["Cons"] => args[1].clone(),
                other => panic!("expected first segment, got {other:?}"),
            };
            let dot = eval_global(&env, &mut store, "path_dot_segment");
            let decision = call_global(&env, &mut store, "path_segment_eq", [first, dot]);
            assert!(
                !matches!(decision, EvalVal::Unknown),
                "segment equality became Unknown"
            );
        }
        let normalized = call_global(&env, &mut store, "path_normalize", [parsed]);
        assert!(
            !matches!(normalized, EvalVal::Unknown),
            "normalization became Unknown for {raw:?}"
        );
        let (actual_absolute, segments) = path_parts(&env, &normalized);
        assert_eq!(actual_absolute, *absolute, "case {raw:?}");
        assert_eq!(
            list_segments(&env, segments),
            normalized_segments
                .iter()
                .map(|segment| segment.to_vec())
                .collect::<Vec<_>>(),
            "case {raw:?}"
        );
        let renormalized = call_global(&env, &mut store, "path_normalize", [normalized.clone()]);
        let (renormalized_absolute, renormalized_segments) = path_parts(&env, &renormalized);
        assert_eq!(renormalized_absolute, actual_absolute, "case {raw:?}");
        assert_eq!(
            list_segments(&env, renormalized_segments),
            list_segments(&env, segments),
            "normalization must be idempotent for {raw:?}"
        );
        assert_eq!(
            call_global(&env, &mut store, "path_render", [normalized]),
            EvalVal::Bytes(rendered.to_vec()),
            "case {raw:?}"
        );
    }
}

#[test]
fn invalid_utf8_survives_the_structured_boundary_without_decoding() {
    let env = full_env();
    let mut store = make_store(&env);
    let invalid = vec![b'/', 0xff, 0xfe, b'/', 0x80, b'a'];
    let parsed = call_global(
        &env,
        &mut store,
        "path_parse",
        [EvalVal::Bytes(invalid.clone())],
    );
    let (absolute, segments) = path_parts(&env, &parsed);
    assert!(absolute);
    assert_eq!(
        list_segments(&env, segments),
        vec![vec![0xff, 0xfe], vec![0x80, b'a']]
    );
    assert_eq!(
        call_global(&env, &mut store, "path_render", [parsed]),
        EvalVal::Bytes(invalid)
    );
}

#[test]
fn validity_rejects_the_old_counterexamples_and_accepts_dotdot() {
    let mut env = full_env();
    env.elaborate_file(
        r#"
        const cc6b_invalid_empty : Path =
          MkPath False (Cons (List UInt8) (Nil UInt8) (Nil (List UInt8)))
        const cc6b_invalid_slash : Path =
          MkPath False
            (Cons
              (List UInt8)
              (Cons UInt8 (47 : UInt8) (Nil UInt8))
              (Nil (List UInt8)))
        const cc6b_valid_dotdot : Path =
          MkPath False
            (Cons
              (List UInt8)
              (Cons UInt8 (46 : UInt8) (Cons UInt8 (46 : UInt8) (Nil UInt8)))
              (Nil (List UInt8)))
        "#,
    )
    .expect("validity discriminator paths must elaborate");
    let mut store = make_store(&env);
    resync_literals(&env, &mut store);
    for (name, path_name, expected) in [
        ("empty", "cc6b_invalid_empty", false),
        ("slash-bearing", "cc6b_invalid_slash", false),
        ("valid but unnormalized dotdot", "cc6b_valid_dotdot", true),
    ] {
        let path = eval_global(&env, &mut store, path_name);
        assert_eq!(
            bool_value(
                &env,
                &call_global(&env, &mut store, "path_valid", [path.clone()])
            ),
            expected,
            "{name} validity"
        );
        if expected {
            let rendered = call_global(&env, &mut store, "path_render", [path.clone()]);
            let reparsed = call_global(&env, &mut store, "path_parse", [rendered]);
            assert_eq!(path_parts(&env, &reparsed).0, path_parts(&env, &path).0);
            assert_eq!(
                list_segments(&env, path_parts(&env, &reparsed).1),
                list_segments(&env, path_parts(&env, &path).1)
            );
        }
    }
}

#[test]
fn package_is_extracted_and_adds_zero_trust() {
    let extracted = ken_elaborator::extract_ken_md(PATH_POSIX).expect("Path.Posix extraction");
    let tokens: BTreeSet<_> = extracted
        .source
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .filter(|token| !token.is_empty())
        .collect();
    for forbidden in [
        "Axiom",
        "primitive",
        "postulate",
        "bytes_length",
        "bytes_slice",
        "bytes_at",
        "String",
    ] {
        assert!(
            !tokens.contains(forbidden),
            "forbidden `{forbidden}` in extracted Ken"
        );
    }
    let mut env = dependency_env();
    let before: BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    env.elaborate_ken_md_file(PATH_POSIX)
        .expect("Path.Posix package");
    let after: BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    assert_eq!(
        before, after,
        "Path.Posix must add zero trusted declarations"
    );
}
