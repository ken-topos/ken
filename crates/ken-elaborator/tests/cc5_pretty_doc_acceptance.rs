//! CC5 (`Pretty.Doc`) ordered shared-environment acceptance.

use std::collections::BTreeSet;

use ken_elaborator::{ElabEnv, NumericLitVal};
use ken_interp::eval::{eval, EvalStore, EvalVal, ListCharIds};
use ken_kernel::{Decl, GlobalId};

const TRANSPORT_KEN_MD: &str = include_str!("../../../catalog/packages/Core/Transport.ken.md");
const COLLECTIONS_KEN_MD: &str =
    include_str!("../../../catalog/packages/Data/Collections/Collections.ken.md");
const PRETTY_DOC_KEN_MD: &str = include_str!("../../../catalog/packages/Pretty/Doc.ken.md");

fn dependency_env() -> ElabEnv {
    let mut env = ElabEnv::empty().expect("prelude bootstrap");
    env.elaborate_ken_md_file(TRANSPORT_KEN_MD)
        .expect("Core.Transport must elaborate first");
    env.elaborate_ken_md_file(COLLECTIONS_KEN_MD)
        .expect("Data.Collections must elaborate second");
    env
}

fn full_env() -> ElabEnv {
    let mut env = dependency_env();
    env.elaborate_ken_md_file(PRETTY_DOC_KEN_MD)
        .expect("Pretty.Doc and every checked fence must elaborate third");
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

fn list_char_codepoints(env: &ElabEnv, value: &EvalVal) -> Vec<u32> {
    let nil_id = env.prelude_env.nil_id;
    let cons_id = env.prelude_env.cons_id;
    let mut out = Vec::new();
    let mut current = value.clone();
    loop {
        match &current {
            EvalVal::Ctor { id, .. } if *id == nil_id => return out,
            EvalVal::Ctor { id, args, .. } if *id == cons_id => {
                match &args[1] {
                    EvalVal::Int(n) => out.push(*n as u32),
                    other => panic!("Cons head must be an Int-typed Char, got {other:?}"),
                }
                current = args[2].clone();
            }
            other => panic!("expected a List Char constructor chain, got {other:?}"),
        }
    }
}

fn as_text(env: &ElabEnv, value: &EvalVal) -> String {
    list_char_codepoints(env, value)
        .into_iter()
        .map(|codepoint| char::from_u32(codepoint).expect("valid Char codepoint"))
        .collect()
}

fn add_render_probes(env: &mut ElabEnv) {
    env.elaborate_file(
        r#"
        const cc5_group_doc : Doc =
          Group
            (Concat
              (Text (string_to_list_char "ab"))
              (Nest
                (Suc (Suc Zero))
                (Concat Line (Text (string_to_list_char "cd")))))

        const cc5_alt_doc : Doc =
          Alt
            (Concat
              (Text (string_to_list_char "ab"))
              (Concat Line (Text (string_to_list_char "cd"))))
            (Concat
              (Text (string_to_list_char "ab"))
              (Nest
                (Suc (Suc Zero))
                (Concat Line (Text (string_to_list_char "cd")))))

        const cc5_group_below : List Char =
          render (Suc (Suc (Suc (Suc Zero)))) cc5_group_doc

        const cc5_group_boundary : List Char =
          render (Suc (Suc (Suc (Suc (Suc Zero))))) cc5_group_doc

        const cc5_group_above : List Char =
          render (Suc (Suc (Suc (Suc (Suc (Suc Zero)))))) cc5_group_doc

        const cc5_group_boundary_again : List Char =
          render (Suc (Suc (Suc (Suc (Suc Zero))))) cc5_group_doc

        const cc5_alt_below : List Char =
          render (Suc (Suc (Suc (Suc Zero)))) cc5_alt_doc

        const cc5_alt_boundary : List Char =
          render (Suc (Suc (Suc (Suc (Suc Zero))))) cc5_alt_doc

        const cc5_alt_above : List Char =
          render (Suc (Suc (Suc (Suc (Suc (Suc Zero)))))) cc5_alt_doc

        "#,
    )
    .expect("boundary and determinism probes must elaborate");
}

fn add_law_probes(env: &mut ElabEnv) {
    env.elaborate_file(
        r#"
        lemma cc5_group_valid : DocContentInvariant cc5_group_doc =
          and_intro
            Top
            (And Top Top)
            Proved
            (and_intro Top Top Proved Proved)

        lemma cc5_group_tokens_preserved :
            Equal
              (List Char)
              (render_content (Suc (Suc (Suc (Suc Zero)))) cc5_group_doc)
              (doc_content cc5_group_doc) =
          (proof preserves_text_tokens for render_content)
            (Suc (Suc (Suc (Suc Zero))))
            cc5_group_doc
            cc5_group_valid

        lemma cc5_group_width_independent :
            Equal
              (List Char)
              (render_content (Suc (Suc (Suc (Suc Zero)))) cc5_group_doc)
              (render_content (Suc (Suc (Suc (Suc (Suc Zero))))) cc5_group_doc) =
          (proof width_independent for render_content)
            (Suc (Suc (Suc (Suc Zero))))
            (Suc (Suc (Suc (Suc (Suc Zero)))))
            cc5_group_doc
            cc5_group_valid

        lemma cc5_group_render_fixed_point :
            Equal
              (List Char)
              (render
                (Suc (Suc (Suc (Suc Zero))))
                (Text (render (Suc (Suc (Suc (Suc Zero)))) cc5_group_doc)))
              (render (Suc (Suc (Suc (Suc Zero)))) cc5_group_doc) =
          (proof fixed_point for render) (Suc (Suc (Suc (Suc Zero)))) cc5_group_doc
        "#,
    )
    .expect("proof-consumption probes must elaborate");
}

#[test]
fn ordered_dependency_closure_elaborates_transparent_pretty_doc() {
    let env = full_env();
    assert!(env.globals.contains_key("Doc"));
    for constructor in ["Text", "Line", "Concat", "Nest", "Group", "Alt"] {
        assert!(
            env.globals.contains_key(constructor),
            "expected checked Doc constructor `{constructor}`"
        );
    }
    assert_transparent_globals(
        &env,
        &[
            "doc_content",
            "DocContentInvariant",
            "pretty_nat_add",
            "pretty_nat_leq",
            "pretty_repeat_char",
            "doc_flat_width",
            "doc_fits",
            "render_mode",
            "render",
            "render_content_mode",
            "render_content",
            "render_content_mode::preserves_text_tokens",
            "render_content::preserves_text_tokens",
            "render_content::width_independent",
            "render::fixed_point",
            "text_string",
            "render_string",
        ],
    );
}

#[test]
fn group_and_alt_flip_at_the_exact_fitting_boundary() {
    let mut env = full_env();
    add_render_probes(&mut env);
    let mut store = make_store(&env);

    for name in ["cc5_group_below", "cc5_alt_below"] {
        assert_eq!(
            as_text(&env, &eval_global(&env, &mut store, name)),
            "ab\n  cd"
        );
    }
    for name in [
        "cc5_group_boundary",
        "cc5_group_above",
        "cc5_alt_boundary",
        "cc5_alt_above",
    ] {
        assert_eq!(as_text(&env, &eval_global(&env, &mut store, name)), "ab cd");
    }

    let first = eval_global(&env, &mut store, "cc5_group_boundary");
    let second = eval_global(&env, &mut store, "cc5_group_boundary_again");
    assert_eq!(
        list_char_codepoints(&env, &first),
        list_char_codepoints(&env, &second),
        "same Doc and width must render byte-identically across runs"
    );
}

#[test]
fn all_three_laws_are_checked_and_consumable_as_proofs() {
    let mut env = full_env();
    add_render_probes(&mut env);
    add_law_probes(&mut env);
    assert_transparent_globals(
        &env,
        &[
            "cc5_group_tokens_preserved",
            "cc5_group_width_independent",
            "cc5_group_render_fixed_point",
        ],
    );
}

#[test]
fn cc5_has_zero_trust_delta_and_keeps_string_at_the_boundary() {
    let extracted = ken_elaborator::literate::extract_ken_md(PRETTY_DOC_KEN_MD)
        .expect("Pretty.Doc must extract");
    assert!(!extracted.source.contains("Axiom"));
    assert!(!extracted.source.contains("string_length"));
    assert!(!extracted.source.contains("Diagnostic"));
    assert!(extracted.source.contains("Text : List Char → Doc"));
    assert!(extracted.source.contains("fn text_string"));
    assert!(extracted.source.contains("fn render_string"));
    assert!(
        !extracted.source.contains("Equal String"),
        "verified CC5 laws must not cross the opaque String boundary"
    );

    let mut env = dependency_env();
    let before: BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    env.elaborate_ken_md_file(PRETTY_DOC_KEN_MD)
        .expect("Pretty.Doc must elaborate after its Core/Data closure");
    let after: BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    assert_eq!(before, after, "CC5 must add zero trusted-base entries");
}
