//! Canonical Nat arithmetic and free-law acceptance.

use ken_elaborator::ElabEnv;

const TRANSPORT_KEN_MD: &str = include_str!("../../../catalog/packages/Core/Logic/Transport.ken.md");
const NAT_ARITH_KEN_MD: &str = include_str!("../../../catalog/packages/Data/Numeric/Nat/Arithmetic.ken.md");

fn base_env() -> ElabEnv {
    let mut env = ElabEnv::empty().expect("prelude bootstrap");
    env.elaborate_ken_md_file(TRANSPORT_KEN_MD)
        .expect("Core/Logic/Transport.ken.md must elaborate");
    env
}

#[test]
fn all_laws_kernel_check_before_literate_examples() {
    let extracted = ken_elaborator::literate::extract_ken_md(NAT_ARITH_KEN_MD)
        .expect("Arithmetic.ken.md must extract");
    let mut env = base_env();
    env.elaborate_file(&extracted.source)
        .expect("all general arithmetic laws must kernel-check");
}

#[test]
fn entry_elaborates_and_registers_the_free_laws() {
    let mut env = base_env();
    env.elaborate_ken_md_file(NAT_ARITH_KEN_MD)
        .expect("Data/Numeric/Nat/Arithmetic.ken.md must elaborate and kernel-check");
    for name in [
        "add",
        "mul",
        "add::assoc",
        "add::comm",
        "mul::assoc",
        "mul::comm",
        "mul_add_distrib_l",
        "mul_add_distrib_r",
    ] {
        assert!(
            env.globals.contains_key(name),
            "{name} must be a checked global"
        );
    }
}

#[test]
fn canonical_operations_compute_on_concrete_naturals() {
    let mut env = base_env();
    env.elaborate_ken_md_file(NAT_ARITH_KEN_MD)
        .expect("Data/Numeric/Nat/Arithmetic.ken.md must elaborate");
    env.elaborate_decl(
        "lemma add_two_three_check : Equal Nat (add (Suc (Suc Zero)) (Suc (Suc (Suc Zero)))) (Suc (Suc (Suc (Suc (Suc Zero))))) = Proved",
    )
    .expect("add 2 3 must compute to 5");
    env.elaborate_decl(
        "lemma mul_two_three_check : Equal Nat (mul (Suc (Suc Zero)) (Suc (Suc (Suc Zero)))) (Suc (Suc (Suc (Suc (Suc (Suc Zero)))))) = Proved",
    )
    .expect("mul 2 3 must compute to 6");
}

#[test]
fn tangled_source_stays_free_of_numeric_classes_and_trusted_declarations() {
    let extracted = ken_elaborator::literate::extract_ken_md(NAT_ARITH_KEN_MD)
        .expect("Arithmetic.ken.md must extract");
    for forbidden in ["Axiom", "postulate", "class", "instance", "sorry"] {
        assert!(
            !extracted.source.contains(forbidden),
            "NatArith checked source must not contain {forbidden}",
        );
    }
}

#[test]
fn trusted_base_delta_is_empty_across_the_entry() {
    let mut env = base_env();
    let before: std::collections::BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    env.elaborate_ken_md_file(NAT_ARITH_KEN_MD)
        .expect("Data/Numeric/Nat/Arithmetic.ken.md must elaborate");
    let after: std::collections::BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    assert_eq!(before, after, "NatArith must add no trusted-base entries");
}
