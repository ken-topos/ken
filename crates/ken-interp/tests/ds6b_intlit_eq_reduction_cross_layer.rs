//! Cross-layer same-decider pin (ADR 0013 Layer 2, the Architect's required
//! guard): the kernel's `Eq`-at-registered-literal reduction
//! (`ken_kernel::obs::eq_reduce`) and the interpreter's `eq_int` evaluation
//! (`ken_interp::eval::prim_reduce`) must decide the SAME relation —
//! `num_bigint::BigInt` value equality, the same crate/type/operator, not
//! two independently-written comparisons that happen to agree today. This
//! test constructs the identical `BigInt` values on both sides and asserts
//! they agree, so any future drift (a different comparison, a different
//! BigInt version) fails loud here rather than silently diverging.

use ken_interp::eval::{prim_reduce, EvalVal};
use ken_kernel::env::{Context, PrimReduction};
use ken_kernel::term::{Level, Term};
use ken_kernel::{declare_deceq_certificate, declare_inductive, declare_primitive, whnf, CtorSpec, GlobalEnv, InductiveSpec};
use num_bigint::BigInt;

/// Same minimal env shape as `ken-kernel/tests/ds6b_intlit_eq_reduction.rs`
/// — an opaque primitive with a registered cert + `IntLit` home type.
fn mk_env() -> (GlobalEnv, ken_kernel::GlobalId) {
    let mut env = GlobalEnv::new();
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
    let true_ = env.inductive(bool_).unwrap().constructors[0].id;

    let prim_t = declare_primitive(&mut env, vec![], Term::Type(Level::zero()), PrimReduction::OpaqueType)
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
    declare_deceq_certificate(&mut env, prim_t, eq_op, bool_, true_).expect("cert");
    env.register_int_lit_type(prim_t);
    (env, prim_t)
}

fn kernel_eq_int(env: &GlobalEnv, prim_t: ken_kernel::GlobalId, m: &BigInt, n: &BigInt) -> bool {
    let ctx = Context::new();
    let goal = Term::Eq(
        Box::new(Term::const_(prim_t, vec![])),
        Box::new(Term::IntLit(m.clone())),
        Box::new(Term::IntLit(n.clone())),
    );
    let reduced = whnf(env, &ctx, &goal);
    match &reduced {
        Term::Const { id, .. } if *id == env.top_id() => true,
        Term::Const { id, .. } if *id == env.bottom_id() => false,
        other => panic!("kernel eq_reduce did not decide (stayed neutral): {:?}", other),
    }
}

fn interp_eq_int(m: &BigInt, n: &BigInt) -> bool {
    match prim_reduce("eq_int", &[EvalVal::BigInt(m.clone()), EvalVal::BigInt(n.clone())]) {
        EvalVal::Bool(b) => b,
        other => panic!("interp eq_int did not decide: {:?}", other),
    }
}

#[test]
fn kernel_and_interp_agree_on_equal_bigints() {
    let (env, prim_t) = mk_env();
    let m = BigInt::from(123456789i64);
    let n = BigInt::from(123456789i64);
    assert!(kernel_eq_int(&env, prim_t, &m, &n));
    assert!(interp_eq_int(&m, &n));
}

#[test]
fn kernel_and_interp_agree_on_distinct_bigints() {
    let (env, prim_t) = mk_env();
    let m = BigInt::from(5);
    let n = BigInt::from(6);
    assert!(!kernel_eq_int(&env, prim_t, &m, &n));
    assert!(!interp_eq_int(&m, &n));
}

#[test]
fn kernel_and_interp_agree_on_negative_and_zero() {
    let (env, prim_t) = mk_env();
    for (m, n, expect_eq) in [
        (BigInt::from(-1), BigInt::from(-1), true),
        (BigInt::from(-1), BigInt::from(1), false),
        (BigInt::from(0), BigInt::from(0), true),
        (BigInt::from(0), BigInt::from(-0i8), true), // negative-zero edge case
    ] {
        assert_eq!(
            kernel_eq_int(&env, prim_t, &m, &n),
            expect_eq,
            "kernel disagreement on {}=={}",
            m,
            n
        );
        assert_eq!(
            interp_eq_int(&m, &n),
            expect_eq,
            "interp disagreement on {}=={}",
            m,
            n
        );
    }
}

/// Direct pin, exercising both sides on the SAME generated pairs in one
/// loop — the actual "same decider" property, not just "both correct
/// independently."
#[test]
fn kernel_and_interp_agree_across_a_spread_of_values() {
    let (env, prim_t) = mk_env();
    let values: Vec<BigInt> = vec![
        BigInt::from(0),
        BigInt::from(1),
        BigInt::from(-1),
        BigInt::from(i64::MAX),
        BigInt::from(i64::MIN),
        "1".repeat(50).parse().unwrap(),
        format!("-{}", "9".repeat(50)).parse().unwrap(),
    ];
    for m in &values {
        for n in &values {
            let k = kernel_eq_int(&env, prim_t, m, n);
            let i = interp_eq_int(m, n);
            assert_eq!(
                k, i,
                "kernel/interp eq_int disagreement on ({}, {}): kernel={} interp={}",
                m, n, k, i
            );
        }
    }
}
