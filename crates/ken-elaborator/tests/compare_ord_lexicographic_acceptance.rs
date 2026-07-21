//! Acceptance coverage for the derived three-way comparator and the
//! lexicographic `Ord (Pair a b)` / `Ord (List a)` instances.
//!
//! This deliberately drives the real catalog packages.  The concrete
//! examples discriminate all three `OrdResult` outcomes, both strict-negative
//! soundness directions, Pair head/tail lexicography, List prefix/head
//! lexicography, and every law field on nontrivial structural values.

use ken_elaborator::{trusted_base_delta, ElabEnv};
use ken_kernel::env::Decl;

const TRANSPORT_KEN_MD: &str = include_str!("../../../catalog/packages/Core/Logic/Transport.ken.md");
const COLLECTIONS_KEN_MD: &str =
    include_str!("../../../catalog/packages/Data/Collections/Derived.ken.md");
const LAWFUL_CLASSES_KEN_MD: &str =
    include_str!("../../../catalog/packages/Core/Classes/LawfulClasses.ken.md");

fn mk_env() -> ElabEnv {
    let mut env = ElabEnv::new().expect("base env construction failed");
    env.elaborate_ken_md_file(TRANSPORT_KEN_MD)
        .expect("Core/Logic/Transport.ken must elaborate");
    env.elaborate_ken_md_file(COLLECTIONS_KEN_MD)
        .expect("Data/Collections/Derived.ken must elaborate");
    env.elaborate_ken_md_file(LAWFUL_CLASSES_KEN_MD)
        .expect("Core/Classes/LawfulClasses.ken must elaborate after its dependencies");
    env
}

fn assert_bool_reduces(env: &mut ElabEnv, name: &str, expression: &str, expected: &str) {
    env.elaborate_decl(&format!("const {name} : Bool = {expression}"))
        .unwrap_or_else(|e| panic!("{name} must elaborate: {e}"));
    env.elaborate_decl(&format!(
        "lemma {name}_reduces : Equal Bool {name} {expected} = Proved"
    ))
    .unwrap_or_else(|e| panic!("{name} must reduce to {expected}: {e}"));
}

fn assert_ord_result_reduces(env: &mut ElabEnv, name: &str, expression: &str, expected: &str) {
    env.elaborate_decl(&format!("const {name} : OrdResult = {expression}"))
        .unwrap_or_else(|e| panic!("{name} must elaborate: {e}"));
    env.elaborate_decl(&format!(
        "lemma {name}_reduces : Equal OrdResult {name} {expected} = Proved"
    ))
    .unwrap_or_else(|e| panic!("{name} must reduce to {expected}: {e}"));
}

fn assert_decl(env: &mut ElabEnv, declaration: &str) {
    env.elaborate_decl(declaration)
        .unwrap_or_else(|e| panic!("declaration must elaborate:\n{declaration}\nerror: {e}"));
}

#[test]
fn raw_compare_discriminates_all_results_and_strict_negatives() {
    let mut env = mk_env();

    assert_ord_result_reduces(
        &mut env,
        "raw_eq",
        "compare_raw Bool bool_leq True True",
        "ord_eq",
    );
    assert_ord_result_reduces(
        &mut env,
        "raw_lt",
        "compare_raw Bool bool_leq False True",
        "ord_lt",
    );
    assert_ord_result_reduces(
        &mut env,
        "raw_gt",
        "compare_raw Bool bool_leq True False",
        "ord_gt",
    );

    assert_decl(
        &mut env,
        "lemma raw_eq_positive : Equal Bool True True = compare_raw::eq_sound Bool bool_leq (Ord_instance_Bool).antisym True True Proved",
    );
    assert_decl(
        &mut env,
        "lemma raw_lt_positive : Equal Bool (bool_leq False True) True = compare_raw::lt_sound Bool bool_leq False True Proved",
    );
    assert_decl(
        &mut env,
        "lemma raw_gt_positive : Equal Bool (bool_leq False True) True = compare_raw::gt_sound Bool bool_leq (Ord_instance_Bool).total True False Proved",
    );
    assert_decl(
        &mut env,
        "lemma raw_lt_reverse_negative : Equal Bool (bool_leq True False) False = compare_raw::lt_reverse_false Bool bool_leq False True Proved",
    );
    assert_decl(
        &mut env,
        "lemma raw_gt_forward_negative : Equal Bool (bool_leq True False) False = compare_raw::gt_forward_false Bool bool_leq True False Proved",
    );
}

#[test]
fn pair_and_list_instances_compute_lexicographically() {
    let mut env = mk_env();
    let pair_ord = "Ord_instance_Pair Bool Bool Ord_instance_Bool Ord_instance_Bool";
    let list_ord = "Ord_instance_List Bool Ord_instance_Bool";

    assert_bool_reduces(
        &mut env,
        "pair_head_lt",
        &format!("({pair_ord}).leq (mk_pair Bool Bool False True) (mk_pair Bool Bool True False)"),
        "True",
    );
    assert_bool_reduces(
        &mut env,
        "pair_head_gt",
        &format!("({pair_ord}).leq (mk_pair Bool Bool True False) (mk_pair Bool Bool False True)"),
        "False",
    );
    assert_bool_reduces(
        &mut env,
        "pair_equal_head_tail_lt",
        &format!("({pair_ord}).leq (mk_pair Bool Bool True False) (mk_pair Bool Bool True True)"),
        "True",
    );
    assert_bool_reduces(
        &mut env,
        "pair_equal_head_tail_gt",
        &format!("({pair_ord}).leq (mk_pair Bool Bool True True) (mk_pair Bool Bool True False)"),
        "False",
    );

    assert_bool_reduces(
        &mut env,
        "list_prefix_lt",
        &format!(
            "({list_ord}).leq (Cons Bool False (Nil Bool)) (Cons Bool False (Cons Bool True (Nil Bool)))"
        ),
        "True",
    );
    assert_bool_reduces(
        &mut env,
        "list_prefix_gt",
        &format!(
            "({list_ord}).leq (Cons Bool False (Cons Bool True (Nil Bool))) (Cons Bool False (Nil Bool))"
        ),
        "False",
    );
    assert_bool_reduces(
        &mut env,
        "list_head_lt",
        &format!(
            "({list_ord}).leq (Cons Bool False (Cons Bool True (Nil Bool))) (Cons Bool True (Nil Bool))"
        ),
        "True",
    );
    assert_bool_reduces(
        &mut env,
        "list_head_gt",
        &format!(
            "({list_ord}).leq (Cons Bool True (Nil Bool)) (Cons Bool False (Cons Bool True (Nil Bool)))"
        ),
        "False",
    );
}

#[test]
fn structural_ord_instances_and_all_laws_are_checked_zero_delta() {
    let mut env = mk_env();
    for name in ["Ord_instance_Pair", "Ord_instance_List"] {
        let id = env.globals[name];
        assert!(
            matches!(env.env.lookup(id), Some(Decl::Transparent { .. })),
            "{name} must be a checked transparent instance"
        );
        let mut delta = trusted_base_delta(&env.env, id);
        delta.remove(&env.class_env.record_nil_val_id);
        assert!(
            delta.is_empty(),
            "{name} must add no trusted base entries: {delta:?}"
        );
    }

    let pair_ord = "Ord_instance_Pair Bool Bool Ord_instance_Bool Ord_instance_Bool";
    let p0 = "(mk_pair Bool Bool False False)";
    let p1 = "(mk_pair Bool Bool False True)";
    let p2 = "(mk_pair Bool Bool True False)";
    let pair_leq = "pair_ord_leq Bool Bool Ord_instance_Bool Ord_instance_Bool";
    assert_decl(
        &mut env,
        &format!("lemma pair_refl_law : IsTrue ({pair_leq} {p1} {p1}) = ({pair_ord}).refl {p1}"),
    );
    assert_decl(
        &mut env,
        &format!("lemma pair_antisym_law : Equal (Pair Bool Bool) {p1} {p1} = ({pair_ord}).antisym {p1} {p1} Proved Proved"),
    );
    assert_decl(
        &mut env,
        &format!("lemma pair_trans_law : IsTrue ({pair_leq} {p0} {p2}) = ({pair_ord}).trans {p0} {p1} {p2} Proved Proved"),
    );
    assert_decl(
        &mut env,
        &format!("lemma pair_total_law : IsTrue (bool_or ({pair_leq} {p2} {p0}) ({pair_leq} {p0} {p2})) = ({pair_ord}).total {p2} {p0}"),
    );

    let list_ord = "Ord_instance_List Bool Ord_instance_Bool";
    let xs = "(Cons Bool False (Nil Bool))";
    let ys = "(Cons Bool False (Cons Bool True (Nil Bool)))";
    let zs = "(Cons Bool True (Nil Bool))";
    let list_leq = "list_ord_leq Bool Ord_instance_Bool";
    assert_decl(
        &mut env,
        &format!("lemma list_refl_law : IsTrue ({list_leq} {ys} {ys}) = ({list_ord}).refl {ys}"),
    );
    assert_decl(
        &mut env,
        &format!("lemma list_antisym_law : Equal (List Bool) {ys} {ys} = ({list_ord}).antisym {ys} {ys} Proved Proved"),
    );
    assert_decl(
        &mut env,
        &format!("lemma list_trans_law : IsTrue ({list_leq} {xs} {zs}) = ({list_ord}).trans {xs} {ys} {zs} Proved Proved"),
    );
    assert_decl(
        &mut env,
        &format!("lemma list_total_law : IsTrue (bool_or ({list_leq} {zs} {xs}) ({list_leq} {xs} {zs})) = ({list_ord}).total {zs} {xs}"),
    );
}

#[test]
fn list_instance_routes_the_canonical_compare_into_raw_list_compare() {
    // Rework (Q-RESIDUE, 2026-07-21): a source-text scan over the literate
    // `.ken.md` can be satisfied by a spelling that never actually executes,
    // and can't be fooled by dead comments either way -- neither proves
    // ROUTING. Instead evaluate `list_ord_leq` (the List Ord instance's own
    // `leq`, built from `list_compare a (compare a d)`) on two singleton
    // lists that differ only in their element: canonical Bool order is
    // False < True (see `raw_lt` above), so only the forward direction can
    // hold. An implementation that ignored the canonical element comparator
    // (e.g. compared only list length/shape) would wrongly agree on BOTH
    // directions, since both lists have the same length -- this pair is a
    // real, non-degenerate discriminator for "the canonical compare is what
    // actually drives list ordering", proven by kernel reduction rather than
    // by grepping either package's source text.
    let mut env = mk_env();
    let list_leq = "list_ord_leq Bool Ord_instance_Bool";
    assert_bool_reduces(
        &mut env,
        "list_leq_false_true",
        &format!("{list_leq} (Cons Bool False (Nil Bool)) (Cons Bool True (Nil Bool))"),
        "True",
    );
    assert_bool_reduces(
        &mut env,
        "list_leq_true_false",
        &format!("{list_leq} (Cons Bool True (Nil Bool)) (Cons Bool False (Nil Bool))"),
        "False",
    );
}
