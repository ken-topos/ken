//! K7 conformance tests — `obs.rs::eq_at_inductive` whnf-before-head-match
//! (`16 §2.2`, the "Inductive type" rule's neutral case: "not headed by a
//! constructor" is a whnf-relative notion, exactly as `eq_at_type`'s sibling
//! rule already treats "neutral"/"canonical").
//!
//! `eq_at_inductive` peeled its two operands with `peel_app` on the **raw**
//! terms, before ever reducing them — so an operation-wrapped literal
//! (`bool_leq True False`, a redex whose whnf is the constructor `False`)
//! reached the head-match as head-`Const`, fell through to the neutral arm,
//! and the `Eq` never collapsed. This is exactly the shape a case-split law
//! proof's contradictory branch produces: the hypothesis is `Eq Bool
//! (bool_leq x y) True` with `x`,`y` substituted to literal constructors, not
//! a bare variable. Fix: `whnf` both operands first, mirroring `eq_at_type`.

use ken_kernel::env::Context;
use ken_kernel::obs::{bottom_term, eq_reduce, top_term};
use ken_kernel::term::{Level, Term};
use ken_kernel::{
    check, declare_def, declare_inductive, declare_postulate, whnf, CtorSpec, GlobalEnv,
    GlobalId, InductiveSpec,
};

// ---------------------------------------------------------------------------
// Minimal environment: Bool, plus `bool_leq : Bool -> Bool -> Bool`, the
// operation whose wrapped output is the K7 repro shape:
//   bool_leq x y := elim_Bool (λ_. Bool) [true branch: y] [false branch: true] x
// i.e. `false <= y` always, `true <= y` iff `y`. `bool_leq true false ⇝ false`
// is the concrete redex that must whnf to a constructor before the `Eq`
// head-match sees it.
// ---------------------------------------------------------------------------

struct B {
    bool_: GlobalId,
    true_: GlobalId,
    false_: GlobalId,
    bool_leq: GlobalId,
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

    // bool_leq := λx. λy. elim_Bool (λ_. Bool) [y, true] x
    // Inside the elim: y is index 0 (innermost binder), x is index 1.
    // `infer_motive_level` calls `infer` on the motive, and bare lambdas
    // aren't inferable — ascribe it (`infer(Ascript(M, M_ty)) → check(M,
    // M_ty)`), mirroring the k1p5 W-style tests' motive-ascription pattern.
    let motive = Term::Ascript(
        Box::new(Term::lam(bool_t.clone(), bool_t.clone())),
        Box::new(Term::pi(bool_t.clone(), Term::Type(Level::zero()))),
    );
    let elim = Term::Elim {
        fam: bool_,
        level_args: vec![],
        params: vec![],
        motive: Box::new(motive),
        methods: vec![Term::var(0), true_c],
        indices: vec![],
        scrut: Box::new(Term::var(1)),
    };
    let body = Term::lam(bool_t.clone(), Term::lam(bool_t.clone(), elim));
    let ty = Term::pi(bool_t.clone(), Term::pi(bool_t.clone(), bool_t.clone()));
    let bool_leq = declare_def(&mut env, vec![], ty, body).expect("bool_leq : Bool -> Bool -> Bool");

    (
        env,
        B {
            bool_,
            true_,
            false_,
            bool_leq,
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
fn bool_leq_app(b: &B, x: Term, y: Term) -> Term {
    Term::app(
        Term::app(
            Term::Const {
                id: b.bool_leq,
                level_args: vec![],
            },
            x,
        ),
        y,
    )
}

// ---------------------------------------------------------------------------
// AC1 (capability): `Eq Bool (bool_leq true false) true` — the operand
// `bool_leq true false` is a redex (not already constructor-headed) whose
// whnf is the constructor `false`. Distinct-constructor comparison against
// `true` must now collapse to `Bottom`.
//
// Pre-fix: `eq_at_inductive` peels the raw redex, sees head `Const{bool_leq}`
// (not `Term::Constructor`), returns `None` — the `Eq` stays neutral.
// Post-fix: whnf resolves the redex to `false` first, then the existing
// distinct-constructor rule fires.
//
// Flip verified by git-stash-and-rerun on `obs.rs` (a genuine widened
// admission over the *same* file — not a new-syntax compile flip): stashing
// the two-line whnf addition reproduces `assert_eq!` failure (`None` instead
// of `Some(Bottom)`); restoring recovers `Some(Bottom)`.
// ---------------------------------------------------------------------------

#[test]
fn eq_at_inductive_collapses_operation_wrapped_distinct_constructors() {
    let (env, b) = mk_env();
    let ctx = Context::new();
    let a = bool_leq_app(&b, true_c(&b), false_c(&b)); // ⇝ false (a redex, not yet reduced)
    let ty = bool_t(&b);
    let reduct = eq_reduce(&env, &ctx, &ty, &a, &true_c(&b));
    assert_eq!(
        reduct,
        Some(bottom_term(&env)),
        "an operation-wrapped operand must whnf to its constructor before the head-match"
    );
}

// ---------------------------------------------------------------------------
// AC1 (capability, same-constructor side): `bool_leq true true ⇝ true` — the
// nullary same-constructor case must also collapse (to `Top`), not just the
// distinct-constructor case. Exercises the fix's other branch.
// ---------------------------------------------------------------------------

#[test]
fn eq_at_inductive_collapses_operation_wrapped_same_constructor() {
    let (env, b) = mk_env();
    let ctx = Context::new();
    let a = bool_leq_app(&b, true_c(&b), true_c(&b)); // ⇝ true
    let ty = bool_t(&b);
    let reduct = eq_reduce(&env, &ctx, &ty, &a, &true_c(&b));
    assert_eq!(reduct, Some(top_term(&env)));
}

// ---------------------------------------------------------------------------
// AC3 (no-regression): a genuinely-neutral operand — `bool_leq x false` under
// a *free* `x : Bool` — must stay neutral. `x` whnf's to itself (a bare
// variable), so the elim inside `bool_leq` can never ι-reduce; `eq_reduce`
// must still return `None`, exactly as before the fix.
// ---------------------------------------------------------------------------

#[test]
fn eq_at_inductive_stays_neutral_on_genuinely_neutral_operand() {
    let (env, b) = mk_env();
    let mut ctx = Context::new();
    ctx.push(bool_t(&b)); // x : Bool, free (no constructor head, ever)
    let x = Term::var(0);
    let a = bool_leq_app(&b, x, false_c(&b)); // scrutinee inside bool_leq is `x` — neutral
    let ty = bool_t(&b);
    let reduct = eq_reduce(&env, &ctx, &ty, &a, &true_c(&b));
    assert_eq!(
        reduct, None,
        "a scrutinee that never resolves to a constructor must leave Eq neutral"
    );
}

// ---------------------------------------------------------------------------
// AC4 (★ discriminating test, kickoff's literal shape): the antisym-style
// contradictory branch — `p : Eq Bool (bool_leq true false) true` is an
// impossible (Bottom-classifying) hypothesis once the operand whnfs, and
// `absurd` must discharge any goal from it. This is the actual failure mode
// `wp/ES4-lawproofs-remainder` hit.
// ---------------------------------------------------------------------------

#[test]
fn absurd_discharges_operation_wrapped_contradictory_hypothesis() {
    let (mut env, b) = mk_env();
    let hyp_ty = Term::Eq(
        Box::new(bool_t(&b)),
        Box::new(bool_leq_app(&b, true_c(&b), false_c(&b))),
        Box::new(true_c(&b)),
    );
    let p = declare_postulate(&mut env, vec![], hyp_ty)
        .expect("p : Eq Bool (bool_leq true false) true (an impossible, operation-wrapped hypothesis)");
    let p_const = Term::Const {
        id: p,
        level_args: vec![],
    };
    let ctx = Context::new();
    let goal = Term::Eq(
        Box::new(bool_t(&b)),
        Box::new(false_c(&b)),
        Box::new(false_c(&b)),
    ); // any Ω goal — ex-falso proves anything
    let absurd_proof = Term::Absurd(Box::new(goal.clone()), Box::new(p_const));
    assert!(
        check(&env, &ctx, &absurd_proof, &goal).is_ok(),
        "absurd must discharge from an operation-wrapped Bottom-classifying hypothesis"
    );
}

// ---------------------------------------------------------------------------
// AC3 (minimal-diff sanity, explicit): `Eq`-at-type (`eq_at_type`) and
// `Eq`-at-Pi/-Sigma/-Omega paths are untouched — a bare-constructor operand
// (the pre-existing, already-covered case) still reduces exactly as before.
// ---------------------------------------------------------------------------

#[test]
fn eq_at_inductive_already_constructor_headed_operand_unaffected() {
    let (env, b) = mk_env();
    let ctx = Context::new();
    let ty = bool_t(&b);
    let reduct = eq_reduce(&env, &ctx, &ty, &true_c(&b), &true_c(&b));
    assert_eq!(reduct, Some(top_term(&env)));
    let reduct2 = eq_reduce(&env, &ctx, &ty, &true_c(&b), &false_c(&b));
    assert_eq!(reduct2, Some(bottom_term(&env)));
}

// ---------------------------------------------------------------------------
// Sanity: `bool_leq` itself whnf's the way the repro assumes (independent of
// `Eq` machinery) — pins the fixture, not the fix.
// ---------------------------------------------------------------------------

#[test]
fn bool_leq_fixture_reduces_as_designed() {
    let (env, b) = mk_env();
    let ctx = Context::new();
    assert_eq!(
        whnf(&env, &ctx, &bool_leq_app(&b, true_c(&b), false_c(&b))),
        false_c(&b)
    );
    assert_eq!(
        whnf(&env, &ctx, &bool_leq_app(&b, false_c(&b), true_c(&b))),
        true_c(&b)
    );
}
