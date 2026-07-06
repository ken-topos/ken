//! K5 conformance tests — `Top`-intro + `Bottom`-elim (`16 §1.3`).
//!
//! `tt : Top` (a prelude constant, zero new `Term` syntax) and
//! `absurd C p : C` (`Term::Absurd`, ex-falso, for `C : Ω` or `C : Type`) — the
//! observational-fragment completion that unblocks `antisym`/`sound`/
//! `complete`-shaped law proofs (K4 alone only reaches `IsTrue`-shaped
//! conclusions).

use ken_kernel::env::Context;
use ken_kernel::obs::{bottom_term, tt_term};
use ken_kernel::term::{Level, Term};
use ken_kernel::{
    check, declare_def, declare_inductive, declare_postulate, declare_recursive_group, infer,
    CtorSpec, GlobalEnv, GlobalId, InductiveSpec, KernelError,
};

// ---------------------------------------------------------------------------
// Minimal environment: Bool.
// ---------------------------------------------------------------------------

struct B {
    bool_: GlobalId,
    true_: GlobalId,
    false_: GlobalId,
    nat: GlobalId,
    zero: GlobalId,
    suc: GlobalId,
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
    let nat = declare_inductive(&mut env, |nat| InductiveSpec {
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
                args: vec![Term::indformer(nat, vec![])],
                target_indices: vec![],
            },
        ],
    })
    .expect("Nat");
    let (zero, suc) = {
        let cs = &env.inductive(nat).unwrap().constructors;
        (cs[0].id, cs[1].id)
    };
    (
        env,
        B {
            bool_,
            true_,
            false_,
            nat,
            zero,
            suc,
        },
    )
}

fn bool_t(b: &B) -> Term {
    Term::indformer(b.bool_, vec![])
}
fn nat_t(b: &B) -> Term {
    Term::indformer(b.nat, vec![])
}
fn true_c(b: &B) -> Term {
    Term::constructor(b.true_, vec![])
}
fn false_c(b: &B) -> Term {
    Term::constructor(b.false_, vec![])
}
fn zero_c(b: &B) -> Term {
    Term::constructor(b.zero, vec![])
}
fn suc_c(b: &B, n: Term) -> Term {
    Term::app(Term::constructor(b.suc, vec![]), n)
}
fn eq_bool(b: &B, x: Term, y: Term) -> Term {
    Term::Eq(Box::new(bool_t(b)), Box::new(x), Box::new(y))
}
fn eq_nat(b: &B, x: Term, y: Term) -> Term {
    Term::Eq(Box::new(nat_t(b)), Box::new(x), Box::new(y))
}

// ---------------------------------------------------------------------------
// AC1a / AC5a (capability): `tt` proves a goal that whnf's to `Top` — the
// "same value" branch of an antisym-shaped proof (`Equal Bool True True`).
// Genuinely new capability: `tt`/`tt_id` do not exist before this WP (a
// fresh prelude constant, not a widened admission) — there is no runtime
// pre/post flip to stash-and-rerun the way K2c/K4 have one, since referring
// to `tt` at all is a compile-time capability gained by this WP. Verified by
// hand: reverting only `env.rs`+`obs.rs` (git stash) makes this test file
// fail to *compile* (`no method named tt_id`), which is the strongest
// possible "didn't exist before" proof.
// ---------------------------------------------------------------------------

#[test]
fn tt_proves_reduced_top_goal() {
    let (mut env, b) = mk_env();
    let goal = eq_bool(&b, true_c(&b), true_c(&b)); // whnf's to Top (same ctor)
    let tt = tt_term(&env);
    let id =
        declare_def(&mut env, vec![], goal, tt).expect("tt must prove a goal that reduces to Top");
    assert!(env.transparent_body(id).is_some());
}

// ---------------------------------------------------------------------------
// AC1b / AC5b (capability): `absurd` discharges a goal from an impossible
// (Bottom-typed) hypothesis — the "contradictory hypothesis" branch of an
// antisym-shaped proof. `p_false : Eq Bool true false` is POSTULATED here —
// asserting an impossible proposition is exactly how a genuinely-unreachable
// branch's hypothesis looks (Architect's safety invariant (i): a Bottom
// inhabitant only ever arises as a variable in an unreachable branch, never
// synthesized from nothing).
// ---------------------------------------------------------------------------

#[test]
fn absurd_discharges_goal_from_impossible_hypothesis() {
    let (mut env, b) = mk_env();
    let p_false_ty = eq_bool(&b, true_c(&b), false_c(&b)); // whnf's to Bottom
    let p_false = declare_postulate(&mut env, vec![], p_false_ty)
        .expect("p_false : Eq Bool true false (an impossible hypothesis)");
    let p_false_const = Term::Const {
        id: p_false,
        level_args: vec![],
    };

    let goal = eq_bool(&b, true_c(&b), true_c(&b)); // any Ω goal — ex-falso proves anything
    let body = Term::Absurd(Box::new(goal.clone()), Box::new(p_false_const));
    let id = declare_def(&mut env, vec![], goal, body).expect("absurd must discharge from Bottom");
    assert!(env.transparent_body(id).is_some());
}

// ---------------------------------------------------------------------------
// Type-valued ex-falso is allowed, but only from an actual Bottom proof. The
// proof must reduce to `Bottom`, not merely be Ω-typed — `tt : Top` is a valid
// Ω-proof but NOT of `Bottom`, so plugging it into `absurd`'s proof slot must be
// rejected too.
// ---------------------------------------------------------------------------

#[test]
fn absurd_discharges_type_motive_from_constructor_disjoint_hypothesis() {
    let (mut env, b) = mk_env();
    let n = declare_postulate(&mut env, vec![], nat_t(&b)).expect("n : Nat");
    let n_const = Term::Const {
        id: n,
        level_args: vec![],
    };
    let p_zero_suc_ty = eq_nat(&b, zero_c(&b), suc_c(&b, n_const));
    let p_zero_suc =
        declare_postulate(&mut env, vec![], p_zero_suc_ty).expect("p : Eq Nat Zero (Suc n)");
    let p_zero_suc_const = Term::Const {
        id: p_zero_suc,
        level_args: vec![],
    };
    let body = Term::Absurd(Box::new(bool_t(&b)), Box::new(p_zero_suc_const));
    let id = declare_def(&mut env, vec![], bool_t(&b), body)
        .expect("absurd must eliminate constructor-disjoint Bottom evidence into Bool");
    assert!(env.transparent_body(id).is_some());
}

#[test]
fn absurd_type_motive_can_mention_context_variables() {
    let (env, b) = mk_env();
    let mut ctx = Context::new();
    ctx.push(Term::Type(Level::zero())); // A : Type 0
    ctx.push(Term::var(0)); // x : A
    ctx.push(eq_bool(&b, true_c(&b), false_c(&b))); // p : Eq Bool True False ⇝ Bottom

    let motive = Term::var(2); // A, a Type-valued motive from the ordinary context.
    let body = Term::Absurd(Box::new(motive.clone()), Box::new(Term::var(0)));
    assert!(
        check(&env, &ctx, &body, &motive).is_ok(),
        "absurd must eliminate Bottom into a Type motive that depends on the context"
    );
}

#[test]
fn refl_does_not_inhabit_constructor_disjoint_equality() {
    let (env, b) = mk_env();
    let mut ctx = Context::new();
    ctx.push(nat_t(&b)); // n : Nat
    let disjoint = eq_nat(&b, zero_c(&b), suc_c(&b, Term::var(0)));
    let result = check(&env, &ctx, &Term::Refl(Box::new(zero_c(&b))), &disjoint);
    assert!(
        result.is_err(),
        "Refl must not prove a constructor-disjoint equality"
    );
}

#[test]
fn neutral_index_equality_cannot_feed_type_absurd() {
    let (mut env, b) = mk_env();
    let x = declare_postulate(&mut env, vec![], nat_t(&b)).expect("x : Nat");
    let y = declare_postulate(&mut env, vec![], nat_t(&b)).expect("y : Nat");
    let x_const = Term::Const {
        id: x,
        level_args: vec![],
    };
    let y_const = Term::Const {
        id: y,
        level_args: vec![],
    };
    let p_neutral_ty = eq_nat(&b, x_const, y_const);
    let p_neutral = declare_postulate(&mut env, vec![], p_neutral_ty).expect("p : Eq Nat x y");
    let p_neutral_const = Term::Const {
        id: p_neutral,
        level_args: vec![],
    };
    let bad = Term::Absurd(Box::new(bool_t(&b)), Box::new(p_neutral_const));
    let result = infer(&env, &Context::new(), &bad);
    assert!(
        result.is_err(),
        "a neutral equality premise must not check as Bottom"
    );
}

#[test]
fn absurd_proof_must_actually_be_bottom() {
    let (env, b) = mk_env();
    let ctx = Context::new();
    let goal = eq_bool(&b, true_c(&b), true_c(&b));
    let bad = Term::Absurd(Box::new(goal.clone()), Box::new(tt_term(&env))); // tt : Top, not Bottom
    let result = infer(&env, &ctx, &bad);
    assert!(
        result.is_err(),
        "a non-Bottom proof (tt : Top) must be rejected"
    );
}

#[test]
fn absurd_type_motive_proof_must_actually_be_bottom() {
    let (env, b) = mk_env();
    let ctx = Context::new();
    let bad = Term::Absurd(Box::new(bool_t(&b)), Box::new(tt_term(&env))); // motive : Type, proof : Top
    let result = infer(&env, &ctx, &bad);
    assert!(
        result.is_err(),
        "Type-valued absurd still requires an actual Bottom proof"
    );
}

#[test]
fn absurd_motive_must_still_classify_as_a_sort() {
    let (mut env, b) = mk_env();
    let p_false_ty = eq_bool(&b, true_c(&b), false_c(&b));
    let p_false = declare_postulate(&mut env, vec![], p_false_ty).expect("p_false");
    let p_false_const = Term::Const {
        id: p_false,
        level_args: vec![],
    };
    let ctx = Context::new();
    let bad = Term::Absurd(Box::new(true_c(&b)), Box::new(p_false_const)); // true : Bool, not a sort.
    let result = infer(&env, &ctx, &bad);
    assert!(
        result.is_err(),
        "an absurd motive must still classify as Type or Omega"
    );
}

// ---------------------------------------------------------------------------
// AC3 (zero-regression, explicit): `trusted_base()` is unaffected by `tt` —
// the prelude constants (`Top`/`Bottom`/`tt`) stay kernel vocabulary, never
// user assumptions. A silently-missed `is_prelude` wire (the exact trap the
// kickoff flagged) would make `tt` appear as a spurious trusted_base entry.
// ---------------------------------------------------------------------------

#[test]
fn trusted_base_unaffected_by_prelude_tt() {
    let env = GlobalEnv::new();
    assert!(
        env.trusted_base().is_empty(),
        "a fresh env's trusted_base() must be empty — Top/Bottom/tt are prelude, not assumptions"
    );
}

// ---------------------------------------------------------------------------
// Architect's HARD gate item (`evt_76y734h71sv4h`): `sct.rs::collect_calls`
// MUST recurse into `Absurd`'s `motive` and `proof` — a new syntactic
// position the call-graph builder doesn't traverse is exactly the K2c
// unapplied-self-reference shape ([[sct-unapplied-self-reference-over-
// accepts]]): a transparent recursive def could launder its self-call
// through an `Absurd` subterm and escape the termination gate.
//
// loop : Bottom := absurd(Bottom, loop)  -- the self-reference sits in
// `Absurd`'s *proof* position (a bare, unapplied `Const{loop_id}` — the
// K2c-style bare-occurrence shape), not at the top level. Genuinely
// type-checks (Bottom classifies as Ω; `loop`'s pre-admitted opaque type IS
// Bottom, so the bare self-reference checks against the `Bottom` proof
// slot) — SCT is the only thing that can catch it.
//
// Flip verified directly (not by git-stash — `sct.rs` compiles identically
// either way, so this is the true apples-to-apples K2c discipline): edit
// the `Term::Absurd` arm in `collect_calls` to a no-recursion no-op, rerun
// — confirmed WRONGLY ADMITTED (`Ok`, transparent). Restored the real
// recursion — confirmed REJECTED (`NotTerminating`).
// ---------------------------------------------------------------------------

#[test]
fn sct_rejects_self_reference_laundered_through_absurd() {
    let mut env = GlobalEnv::new();
    let bottom_ty = bottom_term(&env);
    let result = declare_recursive_group(&mut env, vec![(vec![], bottom_ty.clone())], |ids| {
        let loop_id = ids[0];
        vec![Term::Absurd(
            Box::new(bottom_ty.clone()),
            Box::new(Term::Const {
                id: loop_id,
                level_args: vec![],
            }),
        )]
    });
    assert!(
        result.is_err(),
        "self-reference laundered through Absurd's proof position must be rejected"
    );
    assert!(matches!(
        result.unwrap_err(),
        KernelError::NotTerminating(_)
    ));
}

#[test]
fn sct_rejects_self_reference_laundered_through_type_absurd_motive() {
    let mut env = GlobalEnv::new();
    let bottom_ty = bottom_term(&env);
    let impossible =
        declare_postulate(&mut env, vec![], bottom_ty).expect("p : Bottom for unreachable code");
    let result =
        declare_recursive_group(&mut env, vec![(vec![], Term::Type(Level::zero()))], |ids| {
            let loop_id = ids[0];
            let motive = Term::Let {
                ty: Box::new(Term::Type(Level::zero())),
                val: Box::new(Term::Const {
                    id: loop_id,
                    level_args: vec![],
                }),
                body: Box::new(Term::Type(Level::zero())),
            };
            vec![Term::Absurd(
                Box::new(motive),
                Box::new(Term::Const {
                    id: impossible,
                    level_args: vec![],
                }),
            )]
        });
    assert!(
        result.is_err(),
        "self-reference laundered through Absurd's Type motive must be rejected"
    );
    let err = result.unwrap_err();
    assert!(
        matches!(err, KernelError::NotTerminating(_)),
        "expected NotTerminating, got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// AC5 composite: the literal antisym-shaped case-split — both branches of a
// two-constructor case analysis, one discharged by `tt`, the other by
// `absurd`, matching the actual ES4-lawproofs blocker shape end to end.
// ---------------------------------------------------------------------------

#[test]
fn antisym_shaped_case_split_both_branches() {
    let (mut env, b) = mk_env();
    // "same value" branch: goal reduces to Top, tt proves it.
    let same_goal = eq_bool(&b, true_c(&b), true_c(&b));
    let ctx = Context::new();
    assert!(check(&env, &ctx, &tt_term(&env), &same_goal).is_ok());

    // "contradictory hypothesis" branch: an impossible hypothesis, absurd proves anything.
    let p_false_ty = eq_bool(&b, true_c(&b), false_c(&b));
    let p_false = declare_postulate(&mut env, vec![], p_false_ty).expect("p_false");
    let p_false_const = Term::Const {
        id: p_false,
        level_args: vec![],
    };
    let other_goal = eq_bool(&b, false_c(&b), false_c(&b)); // a different Ω goal
    let absurd_proof = Term::Absurd(Box::new(other_goal.clone()), Box::new(p_false_const));
    assert!(check(&env, &ctx, &absurd_proof, &other_goal).is_ok());
}
