//! CAT-3 acceptance for the structural collection-law slice.
//!
//! This file checks the real package source, not hand-copied snippets. The D1
//! surface is deliberately bounded to structural list ops plus proof-returning
//! `take`/`drop`, `map` length, and `take` length/min laws.
//! D2 adds the verified `List Bool` insertion-sort/count-permutation slice.

use ken_elaborator::{foreign::trusted_base_delta, ElabEnv};
use ken_kernel::Decl;

const TRANSPORT_KEN_MD: &str = include_str!("../../../catalog/packages/Core/Transport.ken.md");
const COLLECTIONS_KEN_MD: &str = include_str!("../../../catalog/packages/Data/Collections/Collections.ken.md");

fn mk_env() -> ElabEnv {
    let mut env = ElabEnv::new().expect("base env");
    env.elaborate_ken_md_file(TRANSPORT_KEN_MD)
        .expect("catalog/packages/Core/Transport.ken must elaborate");
    env.elaborate_ken_md_file(COLLECTIONS_KEN_MD)
        .expect("catalog/packages/Data/Collections/Collections.ken.md must elaborate");
    env
}

#[test]
fn cat3_d1_structural_collections_package_elaborates_zero_delta() {
    let env = mk_env();

    for name in [
        "map",
        "filter",
        "mem",
        "length",
        "min",
        "take_drop_decomposition",
        "map_length",
        "length_take_min",
        "bool_and",
        "bool_leq",
        "eq_from_ord",
        "count",
        "Perm",
        "insert",
        "sort",
        "insert_true_bool",
        "sort_bool",
        "sort_bool_sorted",
        "sort_bool_perm",
        "id_bool",
        "fst_pair_bool_bool",
        "set_fst_pair_bool_bool",
        "fst_lens_get_set",
        "fst_lens_set_get",
        "fst_lens_set_set",
        "bool_iso_to",
        "bool_iso_from",
        "bool_iso_to_from",
        "bool_iso_from_to",
        "true_refinement_project",
        "bool_pair_index_project",
        "id_bool_respects",
    ] {
        let id = env
            .globals
            .get(name)
            .copied()
            .unwrap_or_else(|| panic!("{name} should be exported by collections.ken"));
        match env.env.lookup(id) {
            Some(Decl::Transparent { .. }) => {}
            other => panic!("{name} must be a transparent checked definition, got {other:?}"),
        }
        let delta = trusted_base_delta(&env.env, id);
        assert!(
            delta.is_empty(),
            "{name} must add zero trusted_base delta, got {delta:?}"
        );
    }

    for name in [
        "View",
        "Lens",
        "Iso",
        "Representation",
        "RefinementView",
        "IndexedView",
        "SetoidMorphism",
    ] {
        let id = env
            .globals
            .get(name)
            .copied()
            .unwrap_or_else(|| panic!("{name} should be exported by collections.ken"));
        match env.env.lookup(id) {
            Some(Decl::Transparent { .. }) => {}
            other => panic!("{name} must be a transparent checked record type, got {other:?}"),
        }
        assert!(
            !env.env.trusted_base().contains(&id),
            "{name}'s own class-type id must never enter trusted_base()"
        );
    }
}

#[test]
fn cat3_d1_law_surfaces_are_proof_returning_not_prop_wrappers() {
    assert!(
        COLLECTIONS_KEN_MD.contains("fn take_drop_decomposition")
            && COLLECTIONS_KEN_MD
                .contains(": Equal (List a) (list_append a (take a n xs) (drop a n xs)) xs"),
        "take/drop decomposition must be a proof-returning Equal surface"
    );
    assert!(
        COLLECTIONS_KEN_MD.contains("fn map_length")
            && COLLECTIONS_KEN_MD.contains(": Equal Nat (length b (map a b f xs)) (length a xs)"),
        "map length preservation must be a proof-returning Equal surface"
    );
    assert!(
        COLLECTIONS_KEN_MD.contains("fn length_take_min")
            && COLLECTIONS_KEN_MD
                .contains(": Equal Nat (length a (take a n xs)) (min n (length a xs))"),
        "take length/min law must be a proof-returning Equal surface"
    );
    assert!(
        !COLLECTIONS_KEN_MD.contains(": Prop = Equal"),
        "CAT-3 D1 laws must not be `fn law : Prop = Equal ...` wrappers"
    );
    assert!(
        !COLLECTIONS_KEN_MD.contains("= Axiom"),
        "collections CAT-3 slice must not use Axiom"
    );
    assert!(
        !COLLECTIONS_KEN_MD.contains("data Perm"),
        "CAT-3 D2 permutation must be count equality, not a raw proof-relevant data family"
    );
    assert!(
        COLLECTIONS_KEN_MD.contains("fn Perm (a : Type) (eqf : a → a → Bool)")
            && COLLECTIONS_KEN_MD.contains(
                "(x : a) → Equal Nat (count a eqf x xs) (count a eqf x ys)"
            ),
        "CAT-3 D2 Perm must be the comparator-indexed count/multiset equality surface"
    );
    assert!(
        COLLECTIONS_KEN_MD.contains("fn eq_from_ord")
            && COLLECTIONS_KEN_MD.contains("bool_and (le x y) (le y x)"),
        "eq_from_ord must be the pinned bool_and (le x y) (le y x) definition"
    );
    assert!(
        COLLECTIONS_KEN_MD.contains("class View A")
            && COLLECTIONS_KEN_MD.contains("class Lens A")
            && COLLECTIONS_KEN_MD.contains("class SetoidMorphism A")
            && COLLECTIONS_KEN_MD.contains("project : Bool → Bool"),
        "CAT-3 D3 must expose capitalized View/Lens records and a setoid-morphism project field"
    );
    assert!(
        !COLLECTIONS_KEN_MD.contains("class view")
            && !COLLECTIONS_KEN_MD.contains("fn view")
            && !COLLECTIONS_KEN_MD.contains("const view")
            && !COLLECTIONS_KEN_MD.contains("\nview "),
        "CAT-3 D3 must not introduce a lowercase `view` identifier or retired view declaration"
    );
}

#[test]
fn cat3_d1_positive_surfaces_check_against_real_package_defs() {
    let mut env = mk_env();
    env.elaborate_decl("fn cat3_to_true (x : Nat) : Bool = True")
        .expect("helper predicate should elaborate");
    env.elaborate_decl("fn cat3_nat_eq_all (x : Nat) (y : Nat) : Bool = True")
        .expect("helper equality predicate should elaborate");

    env.elaborate_decl(
        "const cat3_take_drop_sample \
           : Equal (List Bool) \
              (list_append Bool \
                (take Bool (Suc Zero) (Cons Bool True (Cons Bool False (Nil Bool)))) \
                (drop Bool (Suc Zero) (Cons Bool True (Cons Bool False (Nil Bool))))) \
              (Cons Bool True (Cons Bool False (Nil Bool))) \
           = take_drop_decomposition Bool (Suc Zero) (Cons Bool True (Cons Bool False (Nil Bool)))",
    )
    .expect("take/drop decomposition proof should check on a concrete list");

    env.elaborate_decl(
        "const cat3_map_length_sample \
           : Equal Nat \
              (length Bool (map Nat Bool cat3_to_true (Cons Nat Zero (Cons Nat (Suc Zero) (Nil Nat))))) \
              (length Nat (Cons Nat Zero (Cons Nat (Suc Zero) (Nil Nat)))) \
           = map_length Nat Bool cat3_to_true (Cons Nat Zero (Cons Nat (Suc Zero) (Nil Nat)))",
    )
    .expect("map length proof should check on a concrete list");

    env.elaborate_decl(
        "const cat3_length_take_min_sample \
           : Equal Nat \
              (length Nat (take Nat (Suc Zero) (Cons Nat Zero (Cons Nat (Suc Zero) (Nil Nat))))) \
              (min (Suc Zero) (length Nat (Cons Nat Zero (Cons Nat (Suc Zero) (Nil Nat))))) \
           = length_take_min Nat (Suc Zero) (Cons Nat Zero (Cons Nat (Suc Zero) (Nil Nat)))",
    )
    .expect("length/take/min proof should check on a concrete list");

    env.elaborate_decl(
        "const cat3_filter_mem_sample \
           : Equal Bool \
              (mem Nat cat3_nat_eq_all Zero (filter Nat cat3_to_true (Cons Nat (Suc Zero) (Nil Nat)))) \
              True \
           = tt",
    )
    .expect("filter and mem operations should reduce on concrete Bool decisions");
}

#[test]
fn cat3_d2_bool_sort_surfaces_check_against_real_package_defs() {
    let mut env = mk_env();
    let sample = "(Cons Bool True (Cons Bool False (Cons Bool True (Nil Bool))))";

    env.elaborate_decl(&format!(
        "const cat3_sort_bool_sorted_sample \
           : is_sorted Bool bool_leq (sort_bool {sample}) = sort_bool_sorted {sample}"
    ))
    .expect("sort_bool_sorted should prove the sortedness surface");

    env.elaborate_decl(&format!(
        "const cat3_sort_bool_perm_sample \
           : Perm Bool (eq_from_ord Bool bool_leq) {sample} (sort_bool {sample}) = \
             sort_bool_perm {sample}"
    ))
    .expect("sort_bool_perm should prove count/multiset equality");
}

#[test]
fn cat3_d1_wrong_take_drop_witness_rejected() {
    let mut env = mk_env();
    let err = env
        .elaborate_decl(
            "const cat3_bad_take_drop \
               : Equal (List Bool) \
                  (list_append Bool \
                    (take Bool (Suc Zero) (Cons Bool True (Nil Bool))) \
                    (drop Bool (Suc Zero) (Cons Bool True (Nil Bool)))) \
                  (Nil Bool) \
               = tt",
        )
        .expect_err("wrong take/drop endpoint must not typecheck");
    let msg = format!("{err}");
    assert!(
        msg.contains("Type mismatch")
            || msg.contains("type mismatch")
            || msg.contains("Kernel rejected"),
        "wrong witness should reject during type/proof checking, got {msg}"
    );
}

#[test]
fn cat3_d2_bad_sorted_and_bad_perm_witnesses_rejected() {
    let mut env = mk_env();

    let err = env
        .elaborate_decl(
            "const cat3_bad_sorted_bool \
               : is_sorted Bool bool_leq (Cons Bool True (Cons Bool False (Nil Bool))) = tt",
        )
        .expect_err("descending Bool list must not satisfy is_sorted");
    let msg = format!("{err}");
    assert!(
        msg.contains("Type mismatch")
            || msg.contains("type mismatch")
            || msg.contains("Kernel rejected"),
        "bad sorted witness should reject during proof checking, got {msg}"
    );

    let err = env
        .elaborate_decl(
            "const cat3_bad_perm_bool \
               : Perm Bool (eq_from_ord Bool bool_leq) \
                   (Cons Bool True (Nil Bool)) \
                   (Nil Bool) = \
                 \\q. match q { False => tt ; True => tt }",
        )
        .expect_err("dropping True must not satisfy count-based Perm");
    let msg = format!("{err}");
    assert!(
        msg.contains("Type mismatch")
            || msg.contains("type mismatch")
            || msg.contains("Kernel rejected"),
        "bad permutation witness should reject during proof checking, got {msg}"
    );
}

#[test]
fn cat3_d3_view_lens_records_and_flavors_check_against_real_package_defs() {
    let mut env = mk_env();

    for class_name in [
        "View",
        "Lens",
        "Iso",
        "Representation",
        "RefinementView",
        "IndexedView",
        "SetoidMorphism",
    ] {
        assert!(
            env.class_env.classes.contains_key(class_name),
            "{class_name} should be registered as an ordinary class/record"
        );
    }
    assert!(
        env.class_env.classes["SetoidMorphism"]
            .field_names
            .iter()
            .any(|name| name == "project"),
        "setoid-morphism flavor must use field name `project`"
    );

    env.elaborate_decl(
        "const cat3_d3_get_set_sample \
           : Equal Bool \
               (fst_pair_bool_bool (set_fst_pair_bool_bool False (mk_pair Bool Bool True True))) \
               False \
           = fst_lens_get_set False (mk_pair Bool Bool True True)",
    )
    .expect("get-set lens law should be proof-returning and check");

    env.elaborate_decl(
        "const cat3_d3_set_get_sample \
           : Equal (Pair Bool Bool) \
              (set_fst_pair_bool_bool (fst_pair_bool_bool (mk_pair Bool Bool True False)) (mk_pair Bool Bool True False)) \
              (mk_pair Bool Bool True False) \
           = fst_lens_set_get (mk_pair Bool Bool True False)",
    )
    .expect("set-get lens law should be proof-returning and check as full pair equality");

    env.elaborate_decl(
        "const cat3_d3_set_set_sample \
           : Equal (Pair Bool Bool) \
              (set_fst_pair_bool_bool False (set_fst_pair_bool_bool True (mk_pair Bool Bool True False))) \
              (set_fst_pair_bool_bool False (mk_pair Bool Bool True False)) \
           = fst_lens_set_set True False (mk_pair Bool Bool True False)",
    )
    .expect("set-set lens law should be proof-returning and check as full pair equality");

    env.elaborate_decl(
        "const cat3_d3_indexed_project_sample \
           : Equal Bool \
               (bool_pair_index_project (mk_pair Bool Bool True False) True) \
               False \
           = tt",
    )
    .expect("indexed flavor should expose a concrete project operation");

    env.elaborate_decl(
        "const cat3_d3_setoid_project_sample \
           : Equal Bool (id_bool True) (id_bool True) = \
             id_bool_respects True True tt",
    )
    .expect("setoid-morphism respects law should check through project");
}

#[test]
fn cat3_d3_wrong_lens_endpoint_rejected() {
    let mut env = mk_env();
    let err = env
        .elaborate_decl(
            "const cat3_bad_lens_get_set \
               : Equal Bool \
                   (fst_pair_bool_bool (set_fst_pair_bool_bool False (mk_pair Bool Bool True True))) \
                   True \
               = fst_lens_get_set False (mk_pair Bool Bool True True)",
        )
        .expect_err("wrong get-set endpoint must not typecheck");
    let msg = format!("{err}");
    assert!(
        msg.contains("Type mismatch")
            || msg.contains("type mismatch")
            || msg.contains("Kernel rejected"),
        "wrong lens law endpoint should reject during proof checking, got {msg}"
    );
}
