//! DS-5b acceptance: dependent-match index refinement (constructor
//! injectivity + sibling convoy), `docs/program/wp/ds-5b-dependent-match-
//! refinement.md`.
//!
//! `check_match_dependent`'s motive recovery previously refined only the
//! scrutinee's own index (spec `34-data-match §3.2`); it could not (1)
//! re-type a branch's own peeled recursive field via constructor
//! injectivity, or (2) re-type an outer sibling binder sharing the same
//! index (the "convoy" case). Both are carried into the local context via
//! the kernel's own `Eq`/`J`/`Cast` (`16`) — never postulated.
//!
//! Coverage:
//! - AC-injectivity: `tail`-shaped peeled-recursive-field re-typing.
//! - AC-convoy: a sibling binder re-typed through a nested match.
//! - AC-goal: a branch that constructs a fresh family value against an
//!   index-dependent goal (needs its own goal refined, not a context
//!   variable — the third, narrower capability this WP's construction
//!   also required).
//! - AC8: an unlicensed equation is never fabricated — a genuinely
//!   ill-typed program stays kernel-rejected.
//! - Non-indexed inertness: `List`/`Bool` matches are unaffected (implicitly
//!   covered by the full pre-existing suite staying green; direct check
//!   here too).

use ken_elaborator::{ElabEnv, ElabError};
use ken_kernel::KernelError;
use std::collections::BTreeSet;

fn mk_env() -> ElabEnv {
    ElabEnv::new().expect("base env construction failed")
}

fn elab_ok(env: &mut ElabEnv, src: &str) {
    env.elaborate_decl(src)
        .unwrap_or_else(|e| panic!("elaboration failed: {}", e));
}

fn expect_err_val(env: &mut ElabEnv, src: &str) -> ElabError {
    env.elaborate_decl(src)
        .expect_err("declaration unexpectedly elaborated")
}

fn vec_env() -> ElabEnv {
    let mut env = mk_env();
    elab_ok(
        &mut env,
        "data Vec (A : Type) : Nat -> Type where { \
           VNil : Vec A 0; \
           VCons : (n : Nat) -> A -> Vec A n -> Vec A (n+1) \
         }",
    );
    env
}

/// AC-injectivity: `tail`'s `VCons` branch peels `Suc m = Suc n` (via the
/// kernel's own `eq_at_inductive` same-constructor case) to re-type the
/// recursive field `ys : Vec A m` up to the goal `Vec A n` — the exact
/// capability DS-5 named as blocked.
#[test]
fn tail_constructor_injectivity_retypes_peeled_recursive_field() {
    let mut env = vec_env();
    elab_ok(
        &mut env,
        "fn tail (A : Type) (n : Nat) (xs : Vec A (Suc n)) : Vec A n = \
         match xs { VCons m y ys => ys }",
    );
}

/// AC-convoy: matching `v : Vec Nat n` refines `n`; the sibling `w : Vec
/// Nat n` (an outer, independently-bound function parameter, never
/// destructured by the outer match) must refine in lockstep so the nested
/// match on `w` stays exhaustive without an explicit (impossible) `VNil`
/// arm. Un-refined, this is `ExhaustivenessError` on the omitted `VNil`.
#[test]
fn sibling_convoy_retypes_outer_binder_through_nested_match() {
    let mut env = vec_env();
    elab_ok(
        &mut env,
        "fn firstIsSecond (n : Nat) (v : Vec Nat n) (w : Vec Nat n) : Bool = \
         match v { \
           VNil => True; \
           VCons m a xs => match w { VCons _ b ys => True } \
         }",
    );
}

/// AC-goal: a branch that constructs a FRESH family value (`VNil Nat`, the
/// base case a real `zip`-shaped function needs) has no existing context
/// binding for capability 1/2 to redirect — its natural type uses the
/// constructor's own target index, not the caller's un-refined index
/// variable. The checking goal itself must be refined (then the result
/// cast back up), not just a context variable.
#[test]
fn base_case_construction_retypes_the_checking_goal() {
    let mut env = vec_env();
    elab_ok(
        &mut env,
        "fn firstIsVNil (n : Nat) (v : Vec Nat n) (w : Vec Nat n) : Vec Nat n = \
         match v { VNil => VNil Nat; VCons m a xs => v }",
    );
}

/// AC8 (over-refinement discriminator): a goal that requires an equation
/// the branch does NOT license must stay rejected — `ys`'s only provable
/// re-typing (via the `Suc m = Suc n` premise) is `Vec Nat n`, never `Vec
/// Nat (Suc n)`. No cast is ever fabricated from thin air (every `Cast`
/// this WP builds carries a real `J`-derived proof of a real premise), so
/// this must still be a genuine kernel rejection, not a silent accept.
#[test]
fn over_refinement_stays_kernel_rejected() {
    let mut env = vec_env();
    let err = expect_err_val(
        &mut env,
        "fn wrongGoal (n : Nat) (xs : Vec Nat (Suc n)) : Vec Nat (Suc n) = \
         match xs { VCons m y ys => ys }",
    );
    assert!(
        matches!(
            &err,
            ElabError::KernelRejected {
                error: KernelError::TypeMismatch { .. },
                ..
            }
        ),
        "expected a kernel TypeMismatch rejection (the only equation the \
         Suc m = Suc n premise licenses is m = n, never m = Suc n), got: \
         {err:?}"
    );
}

/// Zero-`Axiom` acceptance bar (DS-2-style executable before==after
/// `trusted_base()` set-diff, mirroring `ds2_ord_nat_acceptance.rs`):
/// injectivity discharges through the kernel's own `Eq`/`J`/`Cast` (`16`)
/// — never a postulate — so elaborating all three DS-5b capabilities
/// together must introduce ZERO new `trusted_base()` entries. A `Cast`/`J`
/// construction can in principle require a postulate if a proof
/// obligation doesn't discharge cleanly; this is the machine-checked
/// claim that it didn't, not just an absence of `Axiom` in the diff.
#[test]
fn trusted_base_delta_is_empty_across_all_three_capabilities() {
    let mut env = vec_env();
    let before: BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    elab_ok(
        &mut env,
        "fn tail (A : Type) (n : Nat) (xs : Vec A (Suc n)) : Vec A n = \
         match xs { VCons m y ys => ys }",
    );
    elab_ok(
        &mut env,
        "fn firstIsSecond (n : Nat) (v : Vec Nat n) (w : Vec Nat n) : Bool = \
         match v { \
           VNil => True; \
           VCons m a xs => match w { VCons _ b ys => True } \
         }",
    );
    elab_ok(
        &mut env,
        "fn firstIsVNil (n : Nat) (v : Vec Nat n) (w : Vec Nat n) : Vec Nat n = \
         match v { VNil => VNil Nat; VCons m a xs => v }",
    );
    let after: BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    assert_eq!(
        before, after,
        "injectivity + convoy + goal-refinement must introduce ZERO new \
         trusted_base() entries (zero-Axiom acceptance bar)"
    );
}

/// Non-indexed inertness: a `List`/`Bool` match (no index to refine) must
/// still elaborate — the new equation-in-context machinery is gated on the
/// family actually having indices (`ind.indices.len() > 0` inside
/// `method_index_premise_pairs`) and must never fire on a non-indexed
/// family. This is the same guarantee the full pre-existing suite already
/// exercises broadly; pinned directly here too.
#[test]
fn non_indexed_match_stays_unaffected() {
    let mut env = mk_env();
    elab_ok(
        &mut env,
        "fn allTrue (xs : List Bool) : Prop = \
         match xs { Nil => Equal Bool True True ; \
                    Cons b bs => And (Equal Bool b True) (allTrue bs) }",
    );
}
