//! Structural `DecEq` lifting acceptance: the real catalog package registers
//! proof-carrying `Pair` and `List` instances, computes on concrete values,
//! and keeps its neutral proof paths dictionary-directed.

use ken_elaborator::{trusted_base_delta, ElabEnv};
use ken_kernel::env::Decl;

const TRANSPORT_KEN_MD: &str = include_str!("../../../catalog/packages/Core/Transport.ken.md");
const COLLECTIONS_KEN_MD: &str =
    include_str!("../../../catalog/packages/Data/Collections/Collections.ken.md");
const LAWFUL_CLASSES_KEN_MD: &str =
    include_str!("../../../catalog/packages/Core/LawfulClasses.ken.md");

fn mk_env() -> ElabEnv {
    let mut env = ElabEnv::new().expect("base env construction failed");
    env.elaborate_ken_md_file(TRANSPORT_KEN_MD)
        .expect("Core/Transport.ken must elaborate");
    env.elaborate_ken_md_file(COLLECTIONS_KEN_MD)
        .expect("Data/Collections/Collections.ken must elaborate");
    env.elaborate_ken_md_file(LAWFUL_CLASSES_KEN_MD)
        .expect("Core/LawfulClasses.ken must elaborate after its declared dependencies");
    env
}

fn assert_bool_reduces(env: &mut ElabEnv, name: &str, expression: &str, expected: &str) {
    env.elaborate_decl(&format!("const {name} : Bool = {expression}"))
        .unwrap_or_else(|e| panic!("{name} must elaborate: {e}"));
    env.elaborate_decl(&format!("lemma {name}_reduces : Equal Bool {name} {expected} = Proved"))
        .unwrap_or_else(|e| panic!("{name} must reduce to {expected}: {e}"));
}

#[test]
fn structural_instances_are_checked_transparent_and_zero_delta() {
    let env = mk_env();
    for name in ["DecEq_instance_Pair", "DecEq_instance_List"] {
        let id = env.globals[name];
        assert!(
            matches!(env.env.lookup(id), Some(Decl::Transparent { .. })),
            "{name} must be a checked transparent instance"
        );
        let mut delta = trusted_base_delta(&env.env, id);
        delta.remove(&env.class_env.record_nil_val_id);
        assert!(delta.is_empty(), "{name} must add no trusted base entries: {delta:?}");
    }
}

#[test]
fn structural_instances_compute_positive_and_negative_bool_examples() {
    let mut env = mk_env();
    let pair_same = "(DecEq_instance_Pair Bool Bool DecEq_instance_Bool DecEq_instance_Bool).eq (mk_pair Bool Bool True False) (mk_pair Bool Bool True False)";
    let pair_distinct = "(DecEq_instance_Pair Bool Bool DecEq_instance_Bool DecEq_instance_Bool).eq (mk_pair Bool Bool True False) (mk_pair Bool Bool False False)";
    let list_same = "(DecEq_instance_List Bool DecEq_instance_Bool).eq (Cons Bool True (Cons Bool False (Nil Bool))) (Cons Bool True (Cons Bool False (Nil Bool)))";
    let list_distinct = "(DecEq_instance_List Bool DecEq_instance_Bool).eq (Cons Bool True (Cons Bool False (Nil Bool))) (Cons Bool False (Cons Bool False (Nil Bool)))";

    assert_bool_reduces(&mut env, "pair_same", pair_same, "True");
    assert_bool_reduces(&mut env, "pair_distinct", pair_distinct, "False");
    assert_bool_reduces(&mut env, "list_same", list_same, "True");
    assert_bool_reduces(&mut env, "list_distinct", list_distinct, "False");
}

#[test]
fn list_neutral_path_uses_case_eq_not_a_postulate() {
    let start = LAWFUL_CLASSES_KEN_MD
        .find("### 4.5 Structural `DecEq` liftings")
        .expect("structural lifting section must remain present");
    let section = &LAWFUL_CLASSES_KEN_MD[start..];
    let fence_start = section.find("\n```ken").expect("structural lifting fence must open") + 7;
    let code_end = section[fence_start..]
        .find("\n```")
        .expect("structural lifting fence must close")
        + fence_start;
    let code = &section[fence_start..code_end];
    for required in [
        "match (list_deceq_head_eq a da x y) eqn: h",
        "lemma list_deceq_sound_cons",
        "da.sound x y h",
        "da.complete x x Refl",
        "λp. absurd p",
    ] {
        assert!(code.contains(required), "missing required neutral-proof route: {required}");
    }
    for removed in ["bool_dichotomy", "list_deceq_sound_cons_dispatch"] {
        assert!(!code.contains(removed), "removed hand-written scaffolding remains: {removed}");
    }
    for forbidden in ["Axiom", "postulate", "sorry"] {
        assert!(!code.contains(forbidden), "structural lifting code must not contain {forbidden}");
    }
}
