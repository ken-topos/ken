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
    // Q-CLAIM-COMPARE-ORD (2026-07-23): restore the two claims the original
    // block carried, which the Q-RESIDUE rework dropped when it collapsed to a
    // Bool-only `list_ord_leq` reduction. The original asserted two things by
    // grepping the `.ken.md` (a source scan, which proves neither -- a spelling
    // can be dead code, a comment can lie); both are restored here as kernel
    // reductions:
    //
    //   CLAIM 1 (ROUTING): the `Ord (List a)` instance routes the *canonical*
    //     derived element comparator `compare a d` into `list_compare` at the
    //     instance layer (LawfulClasses `list_ord_leq a d =
    //     ord_result_leq (list_compare a (compare a d) ...)`, projected here as
    //     the instance's own `.leq` field -- so the test exercises the routing
    //     literally through the instance, not a hand-picked helper).
    //   CLAIM 2 (PARAMETERIZATION): `list_compare` is raw-comparator
    //     parameterized -- it takes the element comparator `cmp` as a parameter
    //     and actually consults it, rather than hardcoding one element type.
    //
    // ⛔ THE ELEMENT TYPE MUST BE NON-`Bool`. `Bool` hides both claims: its
    // canonical `compare Bool Ord_instance_Bool` bottoms out in the primitive
    // `bool_leq`, so a `list_compare` that ignored `cmp` and hardcoded a Bool
    // comparison -- or an instance that fed a *constant* comparator instead of
    // the canonical `compare a d` -- would give identical answers on Bool-element
    // lists. Using `Pair Bool Bool` elements forces the element's own *derived*
    // lexicographic comparator (`pair_compare`) to be genuinely routed and
    // consulted: it is reachable by no non-routing / non-parameterized path, and
    // a `cmp` hardcoded to `Bool` would not even typecheck at `Pair Bool Bool`.
    let mut env = mk_env();

    // The real `Ord (List (Pair Bool Bool))` instance, projected through its
    // `.leq` field -- the instance layer whose routing is under test.
    let list_of_pair_leq = "(Ord_instance_List (Pair Bool Bool) \
         (Ord_instance_Pair Bool Bool Ord_instance_Bool Ord_instance_Bool)).leq";
    let singleton = |pair: &str| {
        format!("(Cons (Pair Bool Bool) (mk_pair Bool Bool {pair}) (Nil (Pair Bool Bool)))")
    };
    let ft = singleton("False True");
    let tf = singleton("True False");
    let tt = singleton("True True");

    // Head discriminator: the pairs differ in their FIRST component. Canonical
    // Bool order is False < True, so (False,True) < (True,False) lexicographically
    // -> the List instance must order [ (False,True) ] < [ (True,False) ]. Only
    // the forward direction can hold; a length/shape-only comparison would
    // wrongly agree BOTH ways (both lists are singletons of equal length).
    assert_bool_reduces(
        &mut env,
        "list_pair_head_lt",
        &format!("{list_of_pair_leq} {ft} {tf}"),
        "True",
    );
    assert_bool_reduces(
        &mut env,
        "list_pair_head_gt",
        &format!("{list_of_pair_leq} {tf} {ft}"),
        "False",
    );

    // Tail discriminator: the pairs SHARE their first component and differ only
    // in the SECOND. This proves the FULL element comparator is routed, not just
    // its head: (True,False) < (True,True) requires `pair_compare` to recurse
    // into the second component after the heads tie -- unreachable unless the
    // canonical element `compare` is genuinely driving the list order.
    assert_bool_reduces(
        &mut env,
        "list_pair_tail_lt",
        &format!("{list_of_pair_leq} {tf} {tt}"),
        "True",
    );
    assert_bool_reduces(
        &mut env,
        "list_pair_tail_gt",
        &format!("{list_of_pair_leq} {tt} {tf}"),
        "False",
    );

    // Bool-element base case, retained from the Q-RESIDUE rework: at the
    // primitive base the element (not length) drives order. Kept as a sanity
    // floor; the Pair cases above are what carry the routing/parameterization
    // claims that Bool cannot.
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
