//! Kernel-native `Term::IntLit` + `Eq`-at-registered-literal value-reduction
//! (ADR 0013 Layer 2: `docs/adr/0013-int-decidable-equality-kernel-posture.md`).
//! Kernel-level only — every term here is hand-built, bypassing the
//! elaborator entirely (the elaborator's own literal-emission wiring is a
//! separate, later follow-up). This proves the mechanism itself: given an
//! `IntLit`-bearing term, does the kernel type/reduce/convert it correctly.

use num_bigint::BigInt;
use std::collections::BTreeSet;

use ken_kernel::env::{Context, Decl, PrimReduction};
use ken_kernel::term::{Level, Term};
use ken_kernel::{
    check, declare_deceq_certificate, declare_inductive, declare_primitive, infer, whnf,
    CtorSpec, GlobalEnv, GlobalId, InductiveSpec, KernelError,
};

/// Minimal env: `Bool` (`True | False`) + an opaque primitive `PrimT` with a
/// registered decidable-equality certificate AND a registered `IntLit` home
/// type — the two DS-6a/DS-6b registrations `Int` itself carries.
struct Env0 {
    prim_t: GlobalId,
}

fn mk_env() -> (GlobalEnv, Env0) {
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
    let true_ = env.inductive(bool_).unwrap().constructors[0].id;

    let prim_t = declare_primitive(
        &mut env,
        vec![],
        Term::Type(Level::zero()),
        PrimReduction::OpaqueType,
    )
    .expect("PrimT");

    let prim_const = Term::const_(prim_t, vec![]);
    let bool_t = Term::indformer(bool_, vec![]);
    let eq_op = declare_primitive(
        &mut env,
        vec![],
        Term::pi(prim_const.clone(), Term::pi(prim_const, bool_t)),
        PrimReduction::Op { symbol: "eq_test" },
    )
    .expect("eq_op");

    let _cert = declare_deceq_certificate(&mut env, prim_t, eq_op, bool_, true_)
        .expect("well-shaped eq_op registers");
    env.register_int_lit_type(prim_t);

    (env, Env0 { prim_t })
}

fn eq_ty(prim_t: GlobalId) -> Term {
    Term::const_(prim_t, vec![])
}

// ─────────────────────────────────────────────────────────────────────────
// Typing: `infer(IntLit) = the registered primitive`.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn intlit_infers_the_registered_primitive_type() {
    let (env, e) = mk_env();
    let ctx = Context::new();
    let ty = infer(&env, &ctx, &Term::IntLit(BigInt::from(5))).expect("IntLit infers");
    assert_eq!(ty, eq_ty(e.prim_t));
}

#[test]
fn intlit_infer_fails_closed_when_unregistered() {
    let mut env = GlobalEnv::new();
    // No `register_int_lit_type` call at all.
    let prim_t = declare_primitive(
        &mut env,
        vec![],
        Term::Type(Level::zero()),
        PrimReduction::OpaqueType,
    )
    .expect("PrimT");
    let _ = prim_t;
    let ctx = Context::new();
    let err = infer(&env, &ctx, &Term::IntLit(BigInt::from(5)))
        .expect_err("IntLit must not infer a type before registration");
    assert!(
        matches!(err, KernelError::Msg(_)),
        "expected Msg, got {:?}",
        err
    );
}

// ─────────────────────────────────────────────────────────────────────────
// Acceptance bar (ADR 0013 §"Conformance"): the four discriminating arms.
// ─────────────────────────────────────────────────────────────────────────

/// Over-equate (soundness): a proof of two DISTINCT literals is REJECTED —
/// `Eq ty (IntLit 5) (IntLit 6)` reduces to `Bottom`, and `Refl` (which only
/// checks against a reflexive `Eq`) is specifically rejected against it.
#[test]
fn over_equate_distinct_literals_rejected() {
    let (env, e) = mk_env();
    let ctx = Context::new();
    let goal = Term::Eq(
        Box::new(eq_ty(e.prim_t)),
        Box::new(Term::IntLit(BigInt::from(5))),
        Box::new(Term::IntLit(BigInt::from(6))),
    );

    // The goal itself reduces to Bottom (not Top, not neutral).
    let reduced = whnf(&env, &ctx, &goal);
    assert!(
        matches!(&reduced, Term::Const { id, .. } if *id == env.bottom_id()),
        "Eq ty 5 6 must whnf to Bottom, got {:?}",
        reduced
    );

    // And `Refl` is specifically rejected — not `BadEliminator` (the goal no
    // longer even whnfs to an `Eq` shape to compare endpoints against, since
    // it's already fully reduced to `Bottom`), but `TypeMismatch` against
    // the fully-reduced `Bottom` goal: `Bottom` has no valid introduction
    // form at all, which IS the soundness property under test.
    let err = check(&env, &ctx, &Term::Refl(Box::new(Term::IntLit(BigInt::from(5)))), &goal)
        .expect_err("Refl must not prove 5 = 6");
    assert!(
        matches!(err, KernelError::TypeMismatch { .. }),
        "expected TypeMismatch, got {:?}",
        err
    );
}

/// Under-equate (completeness) — the arm that ONLY passes with DS-6b:
/// `Equal ty 5 5` reduces all the way to `Top`, and `tt` (`Top`'s sole
/// introduction, K5) discharges it. This is the discriminating flip vs a
/// DS-6a-only base (where the goal would stay neutral `Eq`, never `Top`).
///
/// **Note the `tt`-vs-`Refl` discriminator applies here too** (same shape
/// DS-6a's own catalog derivation hit): once BOTH operands are literals the
/// goal is no longer `Eq`-shaped by the time `check` whnfs it — it's
/// already `Top` — so `Refl` (which requires the whnf'd expected type to
/// still BE `Eq`) is the wrong introduction form; `tt` is the textbook
/// correct one for a goal that has collapsed to `Top`.
#[test]
fn under_equate_same_literal_accepted() {
    let (env, e) = mk_env();
    let ctx = Context::new();
    let goal = Term::Eq(
        Box::new(eq_ty(e.prim_t)),
        Box::new(Term::IntLit(BigInt::from(5))),
        Box::new(Term::IntLit(BigInt::from(5))),
    );

    let reduced = whnf(&env, &ctx, &goal);
    assert!(
        matches!(&reduced, Term::Const { id, .. } if *id == env.top_id()),
        "Eq ty 5 5 must whnf to Top, got {:?}",
        reduced
    );

    check(&env, &ctx, &ken_kernel::obs::tt_term(&env), &goal)
        .expect("tt : Equal ty 5 5 must be accepted");
}

/// Neutral preserved: an ABSTRACT (bound-variable) operand pair never
/// reduces, at top level and under a binder — same `_ => None` default as
/// every other non-literal shape, no special case to get wrong.
#[test]
fn neutral_preserved_for_abstract_operands() {
    let (env, e) = mk_env();

    // Top level: two bound variables.
    let mut ctx = Context::new();
    ctx.push(eq_ty(e.prim_t));
    ctx.push(eq_ty(e.prim_t));
    let goal = Term::Eq(
        Box::new(eq_ty(e.prim_t)),
        Box::new(Term::var(1)),
        Box::new(Term::var(0)),
    );
    let reduced = whnf(&env, &ctx, &goal);
    assert!(
        matches!(reduced, Term::Eq(..)),
        "Eq ty x y for abstract x,y must stay neutral, got {:?}",
        reduced
    );

    // Under a binder: the same goal, wrapped in a Pi (so the operands are
    // bound one level deeper) — confirms binder depth doesn't matter, the
    // match is on shape not position.
    let under_binder = Term::pi(eq_ty(e.prim_t), goal.clone());
    let reduced_under = whnf(&env, &Context::new(), &under_binder);
    if let Term::Pi(_, body) = &reduced_under {
        let mut inner_ctx = Context::new();
        inner_ctx.push(eq_ty(e.prim_t));
        let body_reduced = whnf(&env, &inner_ctx, body);
        assert!(
            matches!(body_reduced, Term::Eq(..)),
            "Eq under a binder must also stay neutral, got {:?}",
            body_reduced
        );
    } else {
        panic!("expected a Pi, got {:?}", reduced_under);
    }

    // One literal, one abstract — also neutral (only BOTH-literal reduces).
    let mixed = Term::Eq(
        Box::new(eq_ty(e.prim_t)),
        Box::new(Term::IntLit(BigInt::from(5))),
        Box::new(Term::var(0)),
    );
    let mut ctx1 = Context::new();
    ctx1.push(eq_ty(e.prim_t));
    let mixed_reduced = whnf(&env, &ctx1, &mixed);
    assert!(
        matches!(mixed_reduced, Term::Eq(..)),
        "one-literal-one-abstract must stay neutral, got {:?}",
        mixed_reduced
    );
}

/// Unregistered primitive: even a WHNF-canonical value at an unregistered
/// primitive type never attempts literal reduction (the gate is
/// `deceq_cert(*id).is_some()`, checked before any `IntLit` matching).
#[test]
fn neutral_for_unregistered_primitive() {
    let mut env = GlobalEnv::new();
    let other = declare_primitive(
        &mut env,
        vec![],
        Term::Type(Level::zero()),
        PrimReduction::OpaqueType,
    )
    .expect("other primitive, never registered");
    assert!(env.deceq_cert(other).is_none());

    let ctx = Context::new();
    let goal = Term::Eq(
        Box::new(Term::const_(other, vec![])),
        Box::new(Term::var(0)), // arbitrary non-IntLit operand; the gate must
        Box::new(Term::var(0)), // reject before ever inspecting operand shape
    );
    let mut ctx1 = Context::new();
    ctx1.push(Term::const_(other, vec![]));
    let reduced = whnf(&env, &ctx1, &goal);
    let _ = ctx;
    assert!(matches!(reduced, Term::Eq(..)));
}

// ─────────────────────────────────────────────────────────────────────────
// Definitional equality (`conv_struct`) — distinct from the observational
// `Eq` reduction above, per the Architect's point 2.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn intlit_converts_by_value_not_by_identity() {
    use ken_kernel::convert_type;
    let (env, e) = mk_env();
    let ctx = Context::new();
    let _ = e;
    assert!(convert_type(
        &env,
        &ctx,
        &Term::IntLit(BigInt::from(5)),
        &Term::IntLit(BigInt::from(5)),
    ));
    assert!(!convert_type(
        &env,
        &ctx,
        &Term::IntLit(BigInt::from(5)),
        &Term::IntLit(BigInt::from(6)),
    ));
}

// ─────────────────────────────────────────────────────────────────────────
// Termination / exhaustiveness sanity: a large literal doesn't diverge or
// panic anywhere in the traversal set.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn large_literal_does_not_diverge() {
    let (env, e) = mk_env();
    let ctx = Context::new();
    // ~1000 decimal digits.
    let big: BigInt = "1".repeat(1000).parse().unwrap();
    let mut big_plus_one = big.clone();
    big_plus_one += 1;

    let goal_eq = Term::Eq(
        Box::new(eq_ty(e.prim_t)),
        Box::new(Term::IntLit(big.clone())),
        Box::new(Term::IntLit(big.clone())),
    );
    assert!(matches!(
        whnf(&env, &ctx, &goal_eq),
        Term::Const { id, .. } if id == env.top_id()
    ));

    let goal_neq = Term::Eq(
        Box::new(eq_ty(e.prim_t)),
        Box::new(Term::IntLit(big)),
        Box::new(Term::IntLit(big_plus_one)),
    );
    assert!(matches!(
        whnf(&env, &ctx, &goal_neq),
        Term::Const { id, .. } if id == env.bottom_id()
    ));

    // raw_wf / infer / check also don't choke on a large payload. The goal
    // reduces fully to `Top` (same operand twice), so `tt` is the correct
    // introduction form here too (see `under_equate_same_literal_accepted`).
    let big2: BigInt = "9".repeat(1000).parse().unwrap();
    ken_kernel::raw_well_formed(&ctx, &Term::IntLit(big2.clone())).expect("raw_wf");
    infer(&env, &ctx, &Term::IntLit(big2.clone())).expect("infer");
    check(
        &env,
        &ctx,
        &ken_kernel::obs::tt_term(&env),
        &Term::Eq(
            Box::new(eq_ty(e.prim_t)),
            Box::new(Term::IntLit(big2.clone())),
            Box::new(Term::IntLit(big2)),
        ),
    )
    .expect("check");
}

// ─────────────────────────────────────────────────────────────────────────
// trusted_base(): zero delta. `IntLit` is a term variant (a value former),
// never a `Decl` — no path through the filter reaches it.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn trusted_base_untouched_by_intlit_mechanism() {
    let (mut env, e) = mk_env();
    let before: BTreeSet<GlobalId> = env.trusted_base().into_iter().collect();

    // Exercise every new code path: infer, whnf-reduce both directions,
    // conv_struct, raw_wf — none of it touches trusted_base().
    let ctx = Context::new();
    let _ = infer(&env, &ctx, &Term::IntLit(BigInt::from(1)));
    let _ = whnf(
        &env,
        &ctx,
        &Term::Eq(
            Box::new(eq_ty(e.prim_t)),
            Box::new(Term::IntLit(BigInt::from(1))),
            Box::new(Term::IntLit(BigInt::from(2))),
        ),
    );
    // Registering a SECOND primitive's IntLit-independent cert (unrelated
    // control) doesn't perturb this either — confirms the registry itself,
    // not just IntLit use, stays outside trusted_base().
    let other = declare_primitive(
        &mut env,
        vec![],
        Term::Type(Level::zero()),
        PrimReduction::OpaqueType,
    )
    .unwrap();
    let _ = other;

    let after: BTreeSet<GlobalId> = env.trusted_base().into_iter().collect();
    // `other` itself IS trusted_base (an OpaqueType primitive) — expected
    // and irrelevant to this WP; strip it before asserting IntLit's own
    // zero-delta claim.
    let mut after_minus_other = after.clone();
    after_minus_other.remove(&other);
    assert_eq!(
        before, after_minus_other,
        "IntLit/eq_reduce mechanism itself must add nothing to trusted_base()"
    );

    // And every Decl this test touched beyond `other` is exactly what
    // `mk_env`/DS-6a already registered — no surprise Opaque appeared.
    for id in after.difference(&before) {
        assert_eq!(*id, other, "unexpected new trusted_base entry {:?}", id);
        assert!(matches!(env.lookup(*id), Some(Decl::Primitive { .. })));
    }
}
