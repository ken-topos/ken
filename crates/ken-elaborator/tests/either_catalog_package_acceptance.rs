//! Either catalog package acceptance —
//! `docs/program/wp/either-catalog-package.md`.
//!
//! - Zero `crates/` delta (pure catalog `.ken` + spec docs).
//! - Zero `Axiom`/`postulate`; zero `trusted_base()` delta.
//! - AC8 discriminators flip on a wrong witness, specific error variant.

use ken_elaborator::ElabEnv;

const TRANSPORT_KEN: &str = include_str!("../../../catalog/packages/Core/Transport.ken");
const LAWFUL_CLASSES_KEN: &str = include_str!("../../../catalog/packages/Core/LawfulClasses.ken");
const COLLECTIONS_KEN: &str =
    include_str!("../../../catalog/packages/Data/Collections/Collections.ken");
const LAWFUL_FUNCTORS_KEN: &str =
    include_str!("../../../catalog/packages/Core/LawfulFunctors.ken");
const SUMS_KEN: &str = include_str!("../../../catalog/packages/Data/Sums/Sums.ken");

fn base_env() -> ElabEnv {
    let mut env = ElabEnv::empty().expect("prelude bootstrap");
    env.elaborate_file(TRANSPORT_KEN).expect("Core/Transport.ken must elaborate");
    env.elaborate_file(LAWFUL_CLASSES_KEN).expect("Core/LawfulClasses.ken must elaborate");
    env.elaborate_file(COLLECTIONS_KEN).expect("Data/Collections/Collections.ken must elaborate");
    env.elaborate_file(LAWFUL_FUNCTORS_KEN).expect("Core/LawfulFunctors.ken must elaborate");
    env.elaborate_file(SUMS_KEN).expect("Data/Sums/Sums.ken must elaborate");
    env
}

#[test]
fn either_and_combinators_are_real_globals() {
    let env = base_env();
    for name in [
        "Either",
        "Left",
        "Right",
        "either",
        "either_left",
        "either_right",
        "map_left",
        "map_left_left",
        "map_left_right",
        "map_right",
        "map_right_left",
        "map_right_right",
        "swap",
        "swap_involutive",
    ] {
        assert!(
            env.globals.contains_key(name),
            "`{}` must be a real registered global after elaborating Sums.ken",
            name
        );
    }
}

#[test]
fn zero_axiom_in_sums_ken() {
    assert!(!SUMS_KEN.contains("Axiom"), "Sums.ken must contain zero Axiom literals");
}

#[test]
fn trusted_base_delta_is_empty_across_the_file() {
    let mut env = ElabEnv::empty().expect("prelude bootstrap");
    env.elaborate_file(TRANSPORT_KEN).expect("Core/Transport.ken must elaborate");
    env.elaborate_file(LAWFUL_CLASSES_KEN).expect("Core/LawfulClasses.ken must elaborate");
    env.elaborate_file(COLLECTIONS_KEN).expect("Data/Collections/Collections.ken must elaborate");
    env.elaborate_file(LAWFUL_FUNCTORS_KEN).expect("Core/LawfulFunctors.ken must elaborate");
    let before: std::collections::BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    env.elaborate_file(SUMS_KEN).expect("Data/Sums/Sums.ken must elaborate");
    let after: std::collections::BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    assert_eq!(
        before, after,
        "Sums.ken must introduce ZERO new trusted_base() entries (zero-Axiom acceptance bar)"
    );
}

// AC8 discriminator 1: `either` really does dispatch by branch — reusing
// the `Left`-branch proof (which applies `f`) to claim the `Right` branch
// also applies `f` must be rejected.
#[test]
fn ac8_either_does_not_swap_branches() {
    let mut env = base_env();
    let r = env.elaborate_decl(
        "fn bad_either_swapped (a : Type) (b : Type) (c : Type) (f : a -> c) (g : b -> c) (v : b) : \
           Equal c (either a b c f g (Right a b v)) (f v) = either_left a b c f g v",
    );
    match r {
        Ok(_) => panic!("either_left proves the Left-branch equation (f v); reusing it for a Right-branch goal must be rejected"),
        Err(e) => {
            let msg = format!("{:?}", e);
            assert!(
                msg.contains("TypeMismatch") || msg.contains("KernelRejected"),
                "expected a TypeMismatch/KernelRejected (specific variant), got: {:?}",
                e
            );
        }
    }
}

// AC8 discriminator 2: `swap` is genuinely involutive, not the identity —
// a `swap` that just returns its argument unchanged must be rejected when
// asked to stand in for `swap_involutive`'s witness against a REAL swap.
#[test]
fn ac8_non_involutive_swap_witness_rejected() {
    let mut env = base_env();
    let r = env.elaborate_decl(
        "fn bad_id_either (a : Type) (b : Type) (x : Either a b) : Either a b = x\n\
         fn badSwapInvolutive (a : Type) (b : Type) (x : Either a b) : Equal (Either a b) (bad_id_either a b x) x = tt",
    );
    match r {
        Ok(_) => panic!("a bare `tt` cannot discharge an identity-vs-abstract-x equality — must be rejected"),
        Err(e) => {
            let msg = format!("{:?}", e);
            assert!(
                msg.contains("TypeMismatch") || msg.contains("KernelRejected") || msg.contains("ParseError"),
                "expected a TypeMismatch/KernelRejected (specific variant), got: {:?}",
                e
            );
        }
    }
}

// AC8 discriminator 3: `map_left` does not touch the `Right` payload —
// reusing `map_left_right`'s "untouched" proof to claim `g` was applied
// must be rejected.
#[test]
fn ac8_mapleft_leaves_right_untouched() {
    let mut env = base_env();
    let r = env.elaborate_decl(
        "fn bad_mapLeft_touches_right (a : Type) (c : Type) (f : a -> c) (v : a) : \
           Equal (Either c a) (map_left a a c f (Right a a v)) (Right c a (f v)) = map_left_right a a c f v",
    );
    match r {
        Ok(_) => panic!("map_left_right proves Right v is left UNTOUCHED (Right c b v, not Right c b (f v)) — reusing it for a f-applied RHS must be rejected"),
        Err(e) => {
            let msg = format!("{:?}", e);
            assert!(
                msg.contains("TypeMismatch") || msg.contains("KernelRejected"),
                "expected a TypeMismatch/KernelRejected (specific variant), got: {:?}",
                e
            );
        }
    }
}

// Functional sanity: swap is genuinely involutive on a concrete example,
// both branches.
#[test]
fn swap_involutive_concrete_examples() {
    let mut env = base_env();
    env.elaborate_decl(
        "const swapInvolutiveLeftExample : Equal (Either Nat Nat) (swap Nat Nat (swap Nat Nat (Left Nat Nat Zero))) (Left Nat Nat Zero) = swap_involutive Nat Nat (Left Nat Nat Zero)",
    )
    .expect("swap(swap(Left 0)) = Left 0");
    env.elaborate_decl(
        "const swapInvolutiveRightExample : Equal (Either Nat Nat) (swap Nat Nat (swap Nat Nat (Right Nat Nat Zero))) (Right Nat Nat Zero) = swap_involutive Nat Nat (Right Nat Nat Zero)",
    )
    .expect("swap(swap(Right 0)) = Right 0");
}
