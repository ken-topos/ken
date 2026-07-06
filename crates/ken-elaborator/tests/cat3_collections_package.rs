//! CAT-3 acceptance for the structural collection-law slice.
//!
//! This file checks the real package source, not hand-copied snippets. The D1
//! surface is deliberately bounded to structural list ops plus proof-returning
//! `take`/`drop`, `map` length, and `take` length/min laws.
//! D2 adds the verified `List Bool` insertion-sort/count-permutation slice.

use ken_elaborator::{foreign::trusted_base_delta, ElabEnv};
use ken_kernel::Decl;

const TRANSPORT_KEN: &str = include_str!("../../../packages/transport/transport.ken");
const COLLECTIONS_KEN: &str = include_str!("../../../packages/collections/collections.ken");

fn mk_env() -> ElabEnv {
    let mut env = ElabEnv::new().expect("base env");
    env.elaborate_file(TRANSPORT_KEN)
        .expect("packages/transport/transport.ken must elaborate");
    env.elaborate_file(COLLECTIONS_KEN)
        .expect("packages/collections/collections.ken must elaborate");
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
        "boolLeq",
        "eqFromOrd",
        "count",
        "Perm",
        "insert",
        "sort",
        "insertTrueBool",
        "sortBool",
        "sortBoolSorted",
        "sortBoolPerm",
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
}

#[test]
fn cat3_d1_law_surfaces_are_proof_returning_not_prop_wrappers() {
    assert!(
        COLLECTIONS_KEN.contains("fn take_drop_decomposition")
            && COLLECTIONS_KEN
                .contains(": Equal (List a) (list_append a (take a n xs) (drop a n xs)) xs"),
        "take/drop decomposition must be a proof-returning Equal surface"
    );
    assert!(
        COLLECTIONS_KEN.contains("fn map_length")
            && COLLECTIONS_KEN.contains(": Equal Nat (length b (map a b f xs)) (length a xs)"),
        "map length preservation must be a proof-returning Equal surface"
    );
    assert!(
        COLLECTIONS_KEN.contains("fn length_take_min")
            && COLLECTIONS_KEN
                .contains(": Equal Nat (length a (take a n xs)) (min n (length a xs))"),
        "take length/min law must be a proof-returning Equal surface"
    );
    assert!(
        !COLLECTIONS_KEN.contains(": Prop = Equal"),
        "CAT-3 D1 laws must not be `fn law : Prop = Equal ...` wrappers"
    );
    assert!(
        !COLLECTIONS_KEN.contains("= Axiom"),
        "collections CAT-3 slice must not use Axiom"
    );
    assert!(
        !COLLECTIONS_KEN.contains("data Perm"),
        "CAT-3 D2 permutation must be count equality, not a raw proof-relevant data family"
    );
    assert!(
        COLLECTIONS_KEN.contains("fn Perm (a : Type) (eqf : a → a → Bool)")
            && COLLECTIONS_KEN.contains(
                "(x : a) → Equal Nat (count a eqf x xs) (count a eqf x ys)"
            ),
        "CAT-3 D2 Perm must be the comparator-indexed count/multiset equality surface"
    );
    assert!(
        COLLECTIONS_KEN.contains("fn eqFromOrd")
            && COLLECTIONS_KEN.contains("bool_and (le x y) (le y x)"),
        "eqFromOrd must be the pinned bool_and (le x y) (le y x) definition"
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
           : isSorted Bool boolLeq (sortBool {sample}) = sortBoolSorted {sample}"
    ))
    .expect("sortBoolSorted should prove the sortedness surface");

    env.elaborate_decl(&format!(
        "const cat3_sort_bool_perm_sample \
           : Perm Bool (eqFromOrd Bool boolLeq) {sample} (sortBool {sample}) = \
             sortBoolPerm {sample}"
    ))
    .expect("sortBoolPerm should prove count/multiset equality");
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
               : isSorted Bool boolLeq (Cons Bool True (Cons Bool False (Nil Bool))) = tt",
        )
        .expect_err("descending Bool list must not satisfy isSorted");
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
               : Perm Bool (eqFromOrd Bool boolLeq) \
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
