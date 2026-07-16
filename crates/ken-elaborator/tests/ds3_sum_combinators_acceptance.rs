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

const TRANSPORT_KEN_MD: &str = include_str!("../../../catalog/packages/Core/Logic/Transport.ken.md");
const LAWFUL_CLASSES_KEN_MD: &str =
    include_str!("../../../catalog/packages/Core/Classes/LawfulClasses.ken.md");
const COLLECTIONS_KEN_MD: &str =
    include_str!("../../../catalog/packages/Data/Collections/Derived.ken.md");
const LAWFUL_FUNCTORS_KEN_MD: &str =
    include_str!("../../../catalog/packages/Core/Classes/LawfulFunctors.ken.md");
const SUMS_KEN_MD: &str = include_str!("../../../catalog/packages/Data/Sums/Combinators.ken.md");

fn base_env() -> ElabEnv {
    let mut env = ElabEnv::empty().expect("prelude bootstrap");
    env.elaborate_ken_md_file(TRANSPORT_KEN_MD).expect("Core/Logic/Transport.ken must elaborate");
    env.elaborate_ken_md_file(COLLECTIONS_KEN_MD).expect("Data/Collections/Derived.ken.md must elaborate");
    env.elaborate_ken_md_file(LAWFUL_CLASSES_KEN_MD).expect("Core/Classes/LawfulClasses.ken must elaborate");
    env.elaborate_ken_md_file(LAWFUL_FUNCTORS_KEN_MD).expect("Core/Classes/LawfulFunctors.ken.md must elaborate");
    env.elaborate_ken_md_file(SUMS_KEN_MD).expect("Data/Sums/Combinators.ken.md must elaborate");
    env
}

#[test]
fn all_combinators_and_laws_are_real_globals() {
    let env = base_env();
    for name in [
        "get_or_else",
        "get_or_else::none",
        "get_or_else::some",
        "is_some",
        "is_some::none",
        "is_some::some",
        "or_else",
        "or_else::none",
        "or_else::some",
        "or_else::none_rhs",
        "map_err",
        "map_err::ok",
        "map_err::err",
        "and_then",
        "and_then::ok",
        "and_then::err",
        "unwrap_or",
        "unwrap_or::ok",
        "unwrap_or::err",
    ] {
        assert!(
            env.globals.contains_key(name),
            "`{}` must be a real registered global after elaborating Combinators.ken.md",
            name
        );
    }
}

#[test]
fn zero_axiom_in_sums_ken() {
    // Scoped to the tangled code, not the raw `.ken.md` (which also carries
    // prose) — a literate file's prose can legitimately discuss the word
    // "Axiom" without that being a code-level regression; this must stay
    // false only if the checked/tangled fence itself contains one.
    let tangled = ken_elaborator::literate::extract_ken_md(SUMS_KEN_MD)
        .expect("Data/Sums/Combinators.ken.md must extract")
        .source;
    assert!(!tangled.contains("Axiom"), "Combinators.ken.md's tangled code must contain zero Axiom literals");
}

#[test]
fn trusted_base_delta_is_empty_across_the_file() {
    let mut env = ElabEnv::empty().expect("prelude bootstrap");
    env.elaborate_ken_md_file(TRANSPORT_KEN_MD).expect("Core/Logic/Transport.ken must elaborate");
    env.elaborate_ken_md_file(COLLECTIONS_KEN_MD).expect("Data/Collections/Derived.ken.md must elaborate");
    env.elaborate_ken_md_file(LAWFUL_CLASSES_KEN_MD).expect("Core/Classes/LawfulClasses.ken must elaborate");
    env.elaborate_ken_md_file(LAWFUL_FUNCTORS_KEN_MD).expect("Core/Classes/LawfulFunctors.ken.md must elaborate");
    let before: std::collections::BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    env.elaborate_ken_md_file(SUMS_KEN_MD).expect("Data/Sums/Combinators.ken.md must elaborate");
    let after: std::collections::BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    assert_eq!(
        before, after,
        "Combinators.ken.md must introduce ZERO new trusted_base() entries (zero-Axiom acceptance bar)"
    );
}

// AC8 discriminator 1: `get_or_else` returns the CONTAINED value on `Some`,
// not the default — reusing the `None`-case proof for the `Some` case
// must be rejected.
#[test]
fn ac8_getorelse_returns_contained_value_not_default() {
    let mut env = base_env();
    let r = env.elaborate_decl(
        "lemma bad_getOrElse_some_returns_default (a : Type) (d : a) (v : a) : Equal a (get_or_else a d (Some a v)) d = get_or_else::none a d",
    );
    match r {
        Ok(_) => panic!("get_or_else::none proves the None case (=d); reusing it for Some (=v) must be rejected"),
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

// AC8 discriminator 2: `map_err` maps the ERROR side, not the OK side —
// applying `g` under `Ok` must be rejected (field-order/constructor-role
// discriminator, directly testing the frame's Err-is-first caution).
#[test]
fn ac8_maperr_does_not_touch_ok_payload() {
    let mut env = base_env();
    let r = env.elaborate_decl(
        "lemma bad_mapErr_touches_ok (e : Type) (f : Type) (g : e -> f) (v : e) : Equal (Result f e) (map_err e f e g (Ok e e v)) (Ok f e (g v)) = map_err::ok e f e g v",
    );
    match r {
        Ok(_) => panic!("map_err::ok proves Ok v is left UNTOUCHED (Ok f a v, not Ok f a (g v)) — reusing it for a g-applied RHS must be rejected"),
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

// AC8 discriminator 3: `and_then` short-circuits on `Err`, never calling
// `k` — reusing the `Ok`-case proof (which DOES call `k`) for the `Err`
// case must be rejected.
#[test]
fn ac8_andthen_short_circuits_on_err() {
    let mut env = base_env();
    let r = env.elaborate_decl(
        "fn bad_andThen_err_calls_k (e : Type) (a : Type) (b : Type) (k : a -> Result e b) (u : e) (v : a) : \
           Equal (Result e b) (and_then e a b k (Err e a u)) (k v) = and_then::ok e a b k v",
    );
    match r {
        Ok(_) => panic!("and_then::ok proves the Ok case calls k; reusing it to claim the Err case also calls k must be rejected"),
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

// Functional sanity: `or_else` is genuinely left-biased on a concrete
// example (Some wins over Some, not just Some wins over None).
#[test]
fn orelse_left_biased_concrete_example() {
    let mut env = base_env();
    env.elaborate_decl(
        "lemma orElseLeftBiasedExample : Equal (Option Nat) (or_else Nat (Some Nat Zero) (Some Nat (Suc Zero))) (Some Nat Zero) = or_else::some Nat Zero (Some Nat (Suc Zero))",
    )
    .expect("or_else must prefer the left Some over a competing right Some");
}
