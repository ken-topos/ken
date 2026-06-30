//! Σ-sort soundness erratum — `sort_sigma` split (`13 §4`/§5).
//!
//! Pins the both-directions discriminating pair: a `Σ` with a **relevant**
//! (`Type`-sorted) first component stays in `Type` (the soundness fix), while a
//! `Σ` with **both** Ω components stays in `Ω` (guards against over-correction).
//! Verifies that `Π` is still codomain-keyed (the split discriminates correctly).
//!
//! All cases are discriminating (verdict-flip on the sort rule).
//! Grounded in `spec/10-kernel/13-pi-sigma.md §4`/`§5`.
//! See `conformance/kernel/pi-sigma/seed-pi-sigma.md §A` for the conformance
//! narrative.

use ken_kernel::env::Context;
use ken_kernel::term::{Level, Term};
use ken_kernel::{
    convert, declare_inductive, infer, CtorSpec, GlobalEnv, GlobalId, InductiveSpec,
};

struct Std {
    bool_: GlobalId,
    true_: GlobalId,
    false_: GlobalId,
}

fn std_env() -> (GlobalEnv, Std) {
    let mut env = GlobalEnv::new();

    // data Bool : Type 0 where  true : Bool  ;  false : Bool
    let bool_ = declare_inductive(&mut env, |_| InductiveSpec {
        level_params: vec![],
        params: vec![],
        indices: vec![],
        level: Level::zero(),
        constructors: vec![
            CtorSpec { args: vec![], target_indices: vec![] },
            CtorSpec { args: vec![], target_indices: vec![] },
        ],
    })
    .expect("Bool");
    let (true_, false_) = {
        let cs = &env.inductive(bool_).unwrap().constructors;
        (cs[0].id, cs[1].id)
    };

    (env, Std { bool_, true_, false_ })
}

// ---------------------------------------------------------------------------
// A.1 — Σ(Bool, λ_. Top) : Type 0, NOT Ω  (the core fix)
//
// Before fix: sort_pi_sigma keys on codomain only → s2=Ω → Ω (wrong).
// After fix:  sort_sigma keys on both → s1=Type, s2=Ω → Type (correct).
// Verdict flips on the sort rule.
// ---------------------------------------------------------------------------
#[test]
fn sigma_subset_relevant_stays_type() {
    let (env, s) = std_env();
    let ctx = Context::new();
    let bool_t = Term::indformer(s.bool_, vec![]);
    let top_t = Term::Const { id: env.top_id(), level_args: vec![] };
    // Σ(Bool, λ_. Top) — carrier Bool : Type 0 (relevant), prop Top : Ω_0
    let sigma = Term::sigma(bool_t, top_t);
    assert_eq!(
        infer(&env, &ctx, &sigma),
        Ok(Term::Type(Level::zero())),
        "Σ(Bool, λ_.Top) must classify at Type 0, not Ω \
         (sort_sigma is both-components-keyed after the fix)"
    );
}

// ---------------------------------------------------------------------------
// A.2 — (true, t) ≢ (false, t) at Σ(Bool, λ_. Top)  (Ω-PI must not fire)
//
// Pre-fix: Σ(Bool,Top) : Ω → Ω-PI shortcut fires → pairs are convertible (bug).
// Post-fix: Σ(Bool,Top) : Type 0 → Ω-PI skipped → structural comparison →
//   first components: true vs false (distinct closed ctors) → NOT convertible.
// ---------------------------------------------------------------------------
#[test]
fn sigma_subset_pairs_not_proof_irrelevant() {
    let (env, s) = std_env();
    let mut ctx = Context::new();
    let bool_t = Term::indformer(s.bool_, vec![]);
    let top_t = Term::Const { id: env.top_id(), level_args: vec![] };
    ctx.push(top_t.clone()); // t : Top (var 0)
    let sigma = Term::sigma(bool_t, top_t.clone());
    let pair_true = Term::pair(
        Term::Constructor { id: s.true_, level_args: vec![] },
        Term::var(0),
    );
    let pair_false = Term::pair(
        Term::Constructor { id: s.false_, level_args: vec![] },
        Term::var(0),
    );
    // Ω-PI must NOT collapse (true, t) ≡ (false, t) — Σ(Bool,Top) is Type 0
    assert!(
        !convert(&env, &ctx, &sigma, &pair_true, &pair_false),
        "(true, t) must NOT be ≡ (false, t) at Σ(Bool, λ_.Top) : Type 0 \
         (Ω-PI proof-irrelevance must not fire on a Type-sorted Σ)"
    );
}

// ---------------------------------------------------------------------------
// A.3 — Σ(Top, λ_. Top) : Ω  (both Ω → Ω; guards against over-correction)
//
// A fix that sent ALL Σ to Type would break conjunction proof-irrelevance.
// This case verifies sort_sigma is Ω when both components are propositions.
// ---------------------------------------------------------------------------
#[test]
fn sigma_conjunction_both_omega_stays_omega() {
    let env = GlobalEnv::new();
    let ctx = Context::new();
    let top_t = Term::Const { id: env.top_id(), level_args: vec![] };
    // Σ(Top, λ_. Top) = Top ∧ Top — both components in Ω_0
    let sigma = Term::sigma(top_t.clone(), top_t);
    assert_eq!(
        infer(&env, &ctx, &sigma),
        Ok(Term::Omega(Level::zero())),
        "Σ(Top, λ_.Top) must remain Ω — conjunction of propositions is a \
         proposition (sort_sigma both-keyed, both Ω → Ω)"
    );
}

// ---------------------------------------------------------------------------
// A.4 — (x : Bool) → Top : Ω  (Π stays codomain-keyed — the split is correct)
//
// Π is unchanged: codomain-keyed (if codomain : Ω then Π : Ω, regardless of
// the domain's sort). Paired with A.1 this pins that the split is Π vs Σ,
// not "all quantifiers changed".
// ---------------------------------------------------------------------------
#[test]
fn pi_into_prop_is_prop_codomain_keyed() {
    let (env, s) = std_env();
    let ctx = Context::new();
    let bool_t = Term::indformer(s.bool_, vec![]);
    let top_t = Term::Const { id: env.top_id(), level_args: vec![] };
    // (x : Bool) → Top — Π with relevant domain (Bool : Type 0), Ω codomain
    let pi = Term::pi(bool_t.clone(), top_t.clone());
    assert_eq!(
        infer(&env, &ctx, &pi),
        Ok(Term::Omega(Level::zero())),
        "(x:Bool)→Top must be Ω — sort_pi is still codomain-keyed (unchanged)"
    );
    // Contrast with Σ: same components, different sort (the discriminating split)
    let sigma = Term::sigma(bool_t, top_t);
    assert_eq!(
        infer(&env, &ctx, &sigma),
        Ok(Term::Type(Level::zero())),
        "Σ(Bool, λ_.Top) must be Type 0 (Π and Σ now differ — erratum split)"
    );
}

// ---------------------------------------------------------------------------
// Regression — Nat × Nat : Type 0 (both Type → Type; K1 unchanged)
// ---------------------------------------------------------------------------
#[test]
fn k1_sigma_nat_nat_regression() {
    let mut env = GlobalEnv::new();
    let nat = declare_inductive(&mut env, |nat| InductiveSpec {
        level_params: vec![],
        params: vec![],
        indices: vec![],
        level: Level::zero(),
        constructors: vec![
            CtorSpec { args: vec![], target_indices: vec![] },
            CtorSpec {
                args: vec![Term::indformer(nat, vec![])],
                target_indices: vec![],
            },
        ],
    })
    .expect("Nat");
    let ctx = Context::new();
    let nat_t = Term::indformer(nat, vec![]);
    let sigma = Term::sigma(nat_t.clone(), nat_t);
    assert_eq!(
        infer(&env, &ctx, &sigma),
        Ok(Term::Type(Level::zero())),
        "Nat × Nat must still be Type 0 (K1 regression)"
    );
}
