//! DS-3 (`Option`/`Result` combinators) acceptance —
//! `docs/program/wp/ds-3-sum-type-combinators.md` (lane a, the mechanical
//! combinator build — the `Either` design fork is a separate WP, not part
//! of this one, per the operator's later L5 ruling).
//!
//! - **AC1** — kernel-untouched, zero new elaborator capability, zero
//!   `trusted_base()` delta.
//! - **Zero `Axiom`/`postulate`/`sorry`** in any proved law.
//! - **AC8** — discriminators flip accept→reject on a wrong witness at
//!   the named law, asserted as the specific error variant.

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
fn all_combinators_and_laws_are_real_globals() {
    let env = base_env();
    for name in [
        "getOrElse",
        "getOrElse_none",
        "getOrElse_some",
        "isSome",
        "isSome_none",
        "isSome_some",
        "orElse",
        "orElse_none",
        "orElse_some",
        "orElse_none_rhs",
        "mapErr",
        "mapErr_ok",
        "mapErr_err",
        "andThen",
        "andThen_ok",
        "andThen_err",
        "unwrapOr",
        "unwrapOr_ok",
        "unwrapOr_err",
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

// AC8 discriminator 1: `getOrElse` returns the CONTAINED value on `Some`,
// not the default — reusing the `None`-case proof for the `Some` case
// must be rejected.
#[test]
fn ac8_getorelse_returns_contained_value_not_default() {
    let mut env = base_env();
    let r = env.elaborate_decl(
        "fn bad_getOrElse_some_returns_default (a : Type) (d : a) (v : a) : Equal a (getOrElse a d (Some a v)) d = getOrElse_none a d",
    );
    match r {
        Ok(_) => panic!("getOrElse_none proves the None case (=d); reusing it for Some (=v) must be rejected"),
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

// AC8 discriminator 2: `mapErr` maps the ERROR side, not the OK side —
// applying `g` under `Ok` must be rejected (field-order/constructor-role
// discriminator, directly testing the frame's Err-is-first caution).
#[test]
fn ac8_maperr_does_not_touch_ok_payload() {
    let mut env = base_env();
    let r = env.elaborate_decl(
        "fn bad_mapErr_touches_ok (e : Type) (f : Type) (g : e -> f) (v : e) : Equal (Result f e) (mapErr e f e g (Ok e e v)) (Ok f e (g v)) = mapErr_ok e f e g v",
    );
    match r {
        Ok(_) => panic!("mapErr_ok proves Ok v is left UNTOUCHED (Ok f a v, not Ok f a (g v)) — reusing it for a g-applied RHS must be rejected"),
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

// AC8 discriminator 3: `andThen` short-circuits on `Err`, never calling
// `k` — reusing the `Ok`-case proof (which DOES call `k`) for the `Err`
// case must be rejected.
#[test]
fn ac8_andthen_short_circuits_on_err() {
    let mut env = base_env();
    let r = env.elaborate_decl(
        "fn bad_andThen_err_calls_k (e : Type) (a : Type) (b : Type) (k : a -> Result e b) (u : e) (v : a) : \
           Equal (Result e b) (andThen e a b k (Err e a u)) (k v) = andThen_ok e a b k v",
    );
    match r {
        Ok(_) => panic!("andThen_ok proves the Ok case calls k; reusing it to claim the Err case also calls k must be rejected"),
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

// Functional sanity: `orElse` is genuinely left-biased on a concrete
// example (Some wins over Some, not just Some wins over None).
#[test]
fn orelse_left_biased_concrete_example() {
    let mut env = base_env();
    env.elaborate_decl(
        "const orElseLeftBiasedExample : Equal (Option Nat) (orElse Nat (Some Nat Zero) (Some Nat (Suc Zero))) (Some Nat Zero) = orElse_some Nat Zero (Some Nat (Suc Zero))",
    )
    .expect("orElse must prefer the left Some over a competing right Some");
}
