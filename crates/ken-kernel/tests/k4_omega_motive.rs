//! K4 conformance tests — Ω-motive elimination (`14 §3`, `16 §1.1`).
//!
//! Extends `Term::Elim`'s motive codomain from `Type ℓ'`-only to also admit
//! `Ω_ℓ'` (a per-branch-varying proposition proved by case-split on a relevant
//! scrutinee), via `infer_motive_level`/`motive_expected_type` in `check.rs`.
//!
//! The propositions here are **postulated** (`P : Bool -> Ω_0`, plus a proof
//! postulate per branch) rather than built from `Eq`/`Refl` — `Eq` at an
//! inductive with matching nullary constructors observationally reduces past
//! itself (`obs::eq_at_inductive`, same-ctor zero-field ⇒ `Top`), which is a
//! separate K2 mechanism this WP doesn't touch. A postulated family keeps
//! these tests scoped to the elim/motive admission being changed here.

use ken_kernel::env::Context;
use ken_kernel::term::{Level, Term};
use ken_kernel::{
    check, convert, declare_def, declare_inductive, declare_postulate, whnf, CtorSpec, GlobalEnv,
    GlobalId, InductiveSpec, KernelError,
};

// ---------------------------------------------------------------------------
// Minimal environment: Bool + a postulated Ω-valued family P : Bool -> Ω_0
// with a proof postulate per branch.
// ---------------------------------------------------------------------------

struct B {
    bool_: GlobalId,
    true_: GlobalId,
    false_: GlobalId,
    p: GlobalId,       // P : Bool -> Ω_0
    p_true: GlobalId,  // p_true : P true
    p_false: GlobalId, // p_false : P false
}

fn mk_env() -> (GlobalEnv, B) {
    let mut env = GlobalEnv::new();
    let bool_ = declare_inductive(&mut env, |_| InductiveSpec {
        level_params: vec![],
        params: vec![],
        indices: vec![],
        level: Level::zero(),
        constructors: vec![
            CtorSpec {
                args: vec![],
                target_indices: vec![],
            },
            CtorSpec {
                args: vec![],
                target_indices: vec![],
            },
        ],
    })
    .expect("Bool");
    let (true_, false_) = {
        let cs = &env.inductive(bool_).unwrap().constructors;
        (cs[0].id, cs[1].id)
    };
    let bool_t = Term::indformer(bool_, vec![]);
    let true_c = Term::constructor(true_, vec![]);
    let false_c = Term::constructor(false_, vec![]);

    let p = declare_postulate(&mut env, vec![], Term::pi(bool_t.clone(), Term::Omega(Level::zero())))
        .expect("P : Bool -> Ω_0");
    let p_app = |x: Term| Term::app(Term::Const { id: p, level_args: vec![] }, x);
    let p_true = declare_postulate(&mut env, vec![], p_app(true_c)).expect("p_true : P true");
    let p_false = declare_postulate(&mut env, vec![], p_app(false_c)).expect("p_false : P false");

    (
        env,
        B {
            bool_,
            true_,
            false_,
            p,
            p_true,
            p_false,
        },
    )
}

fn bool_t(b: &B) -> Term {
    Term::indformer(b.bool_, vec![])
}
fn true_c(b: &B) -> Term {
    Term::constructor(b.true_, vec![])
}
fn false_c(b: &B) -> Term {
    Term::constructor(b.false_, vec![])
}
fn p_const(b: &B) -> Term {
    Term::Const {
        id: b.p,
        level_args: vec![],
    }
}
fn p_true_const(b: &B) -> Term {
    Term::Const {
        id: b.p_true,
        level_args: vec![],
    }
}
fn p_false_const(b: &B) -> Term {
    Term::Const {
        id: b.p_false,
        level_args: vec![],
    }
}
fn p_app(b: &B, x: Term) -> Term {
    Term::app(p_const(b), x)
}

/// `elim_Bool motive true_case false_case scrut` (0 indices, 2 nullary ctors;
/// constructor order from `mk_env` is `[true_, false_]`).
fn bool_elim(b: &B, motive: Term, true_case: Term, false_case: Term, scrut: Term) -> Term {
    Term::Elim {
        fam: b.bool_,
        level_args: vec![],
        params: vec![],
        motive: Box::new(motive),
        methods: vec![true_case, false_case],
        indices: vec![],
        scrut: Box::new(scrut),
    }
}

// ---------------------------------------------------------------------------
// AC1 (capability) / AC5 (discriminating flip): an excluded_middle-shaped def
// admits an Ω-motive Elim.
//
// excluded_middle_shaped : (c:Bool) -> P c
//   := λc. elim_Bool P p_true p_false c
//
// Pre-fix: `infer_motive_level`'s final match accepted only `Term::Type(l)`,
// so the motive's `Ω_0` codomain was rejected — `Err(BadEliminator("motive
// result is not a type (Type ℓ')"))`. Post-fix: `Term::Omega(l)` is admitted
// alongside `Term::Type(l)` (the `Sort` enum already covered this — only
// `infer_motive_level`/`motive_expected_type` were Type-only). Flip verified
// by stashing the `check.rs` fix and rerunning (K2c-style): this test fails
// on the pre-fix tip and passes with the fix.
// ---------------------------------------------------------------------------

#[test]
fn omega_motive_excluded_middle_shaped_admitted() {
    let (mut env, b) = mk_env();
    let ty = Term::pi(bool_t(&b), p_app(&b, Term::var(0)));
    let body = Term::lam(
        bool_t(&b),
        bool_elim(
            &b,
            p_const(&b),
            p_true_const(&b),
            p_false_const(&b),
            Term::var(0),
        ),
    );
    let id = declare_def(&mut env, vec![], ty, body)
        .expect("Ω-motive elim (excluded_middle-shaped) must be admitted");
    assert!(env.transparent_body(id).is_some());
}

// ---------------------------------------------------------------------------
// AC3 (zero-regression, direct check): a Type-codomain motive is a strict
// subcase and still checks byte-identically (unaffected by threading `Sort`
// instead of `Level`).
// ---------------------------------------------------------------------------

#[test]
fn type_motive_regression_still_admitted() {
    let (env, b) = mk_env();
    let motive_ty = Term::pi(bool_t(&b), Term::Type(Level::zero()));
    let motive = Term::Ascript(
        Box::new(Term::lam(bool_t(&b), bool_t(&b))), // λ_. Bool : Bool -> Type 0
        Box::new(motive_ty),
    );
    let elim = bool_elim(&b, motive, true_c(&b), false_c(&b), true_c(&b));
    let ctx = Context::new();
    let result = check(&env, &ctx, &elim, &bool_t(&b));
    assert!(result.is_ok(), "Type-codomain motive must still be admitted");
}

// ---------------------------------------------------------------------------
// Architect ask #2: the Elim-into-Ω whnf/ι-reduction is the SAME
// constructor-selects-method rule as the Type-codomain case — no new
// reduction path, checked as an explicit property (not just true by
// construction). Both branches asserted, and cross-checked against each
// other, so the property is non-degenerate (not vacuously true because the
// gate always picks the same branch).
// ---------------------------------------------------------------------------

#[test]
fn omega_motive_iota_is_the_existing_constructor_selects_method_rule() {
    let (env, b) = mk_env();
    let ctx = Context::new();

    let elim_on_true = bool_elim(
        &b,
        p_const(&b),
        p_true_const(&b),
        p_false_const(&b),
        true_c(&b),
    );
    let elim_on_false = bool_elim(
        &b,
        p_const(&b),
        p_true_const(&b),
        p_false_const(&b),
        false_c(&b),
    );

    let ty_true = p_app(&b, true_c(&b));
    let ty_false = p_app(&b, false_c(&b));

    let reduced_true = whnf(&env, &ctx, &elim_on_true);
    let reduced_false = whnf(&env, &ctx, &elim_on_false);

    assert!(
        convert(&env, &ctx, &ty_true, &reduced_true, &p_true_const(&b)),
        "elim on true must ι-reduce to the true-branch method, unchanged rule"
    );
    assert!(
        convert(&env, &ctx, &ty_false, &reduced_false, &p_false_const(&b)),
        "elim on false must ι-reduce to the false-branch method, unchanged rule"
    );
    // Non-degenerate: the two branches reduce to genuinely distinct terms, so
    // the ι rule is actually dispatching on the constructor, not vacuously
    // returning one fixed answer regardless of scrutinee. This must be a raw
    // *structural* comparison, not `convert` — `convert` at an Ω-typed
    // ambient type is proof-irrelevant by design (Architect's "irrelevance
    // survives embedding" axis), so it would trivially call any two Ω proofs
    // equal regardless of which branch actually fired.
    assert_ne!(
        reduced_true, reduced_false,
        "true/false branches must reduce to syntactically distinct terms"
    );
}

// ---------------------------------------------------------------------------
// Rejection ground truth: an Ω-motive is rejected when it does NOT have a
// legal type/prop codomain (e.g. a value of the family's own carrier type),
// so the fix is a genuine *addition* (Type ∪ Ω), not a wildcard accept.
// ---------------------------------------------------------------------------

#[test]
fn non_type_non_omega_motive_still_rejected() {
    let (env, b) = mk_env();
    let ctx = Context::new();
    let bogus_motive_ty = Term::pi(bool_t(&b), bool_t(&b)); // Bool -> Bool, not a sort
    let motive = Term::Ascript(
        Box::new(Term::lam(bool_t(&b), true_c(&b))), // λ_. true : Bool -> Bool
        Box::new(bogus_motive_ty),
    );
    let elim = bool_elim(&b, motive, true_c(&b), false_c(&b), true_c(&b));
    let result = check(&env, &ctx, &elim, &bool_t(&b));
    assert!(
        result.is_err(),
        "a Bool-codomain motive must still be rejected"
    );
    assert!(matches!(result.unwrap_err(), KernelError::BadEliminator(_)));
}
