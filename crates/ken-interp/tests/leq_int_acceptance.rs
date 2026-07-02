//! AC-L acceptance tests (`conformance/surface/numbers/seed-decimal-char-demote.md`).
//!
//! Covers the pulled-up `leq_int` `prim_reduce` arm (`18a §5.2.2`) against
//! `ken_interp::eval::prim_reduce`/`derived_lt_int` directly. Golden verdicts
//! are hand-determined from the total order on ℤ — never computed by calling
//! `num-bigint`'s own `Ord`/`<=` on both sides (the F1 AC2 discipline,
//! extended from `eq_int` to comparison; green-vs-green is the trap this
//! guards against).

use ken_interp::eval::{derived_lt_int, prim_reduce, EvalVal};
use num_bigint::BigInt;

/// Construct `2^n` as an `EvalVal` via `Shl` — a distinct code path from the
/// comparison under audit.
fn pow2(n: u32) -> EvalVal {
    big(BigInt::from(1u8) << n)
}

fn big(n: BigInt) -> EvalVal {
    match n.to_string().parse::<i64>() {
        Ok(i) => EvalVal::Int(i),
        Err(_) => EvalVal::BigInt(n),
    }
}

/// `i64::MAX + 1` via `Sub`/`Add` on a `Shl`-built power of two — never the
/// comparison operator itself.
fn i64_max_plus_1() -> EvalVal {
    match pow2(63) {
        EvalVal::BigInt(n) => big(n),
        other => other,
    }
}

fn neg(v: EvalVal) -> EvalVal {
    match v {
        EvalVal::Int(n) => EvalVal::Int(-n),
        EvalVal::BigInt(n) => big(-n),
        other => panic!("expected Int/BigInt, got {:?}", other),
    }
}

// ── AC-L — pulled-up `leq_int` arm + independent oracle (soundness, hard-AC) ──

/// surface/numbers/leq-int-bignum-differential-oracle (soundness, hard-AC)
#[test]
fn leq_int_bignum_differential_oracle() {
    let i64_max = EvalVal::Int(i64::MAX);
    let two_127_minus_1 = match pow2(127) {
        EvalVal::BigInt(n) => big(n - BigInt::from(1u8)),
        other => panic!("expected BigInt, got {:?}", other),
    };
    let two_127 = pow2(127);

    let cases: &[(&str, EvalVal, EvalVal, bool)] = &[
        // the 2^63 boundary: i64::MAX <= i64::MAX + 1
        ("2^63 boundary", i64_max.clone(), i64_max_plus_1(), true),
        // the 2^127 boundary: 2^127 - 1 <= 2^127
        ("2^127 boundary", two_127_minus_1, two_127, true),
        // mixed sign
        ("-5 <= 3", EvalVal::Int(-5), EvalVal::Int(3), true),
        ("3 <= -5", EvalVal::Int(3), EvalVal::Int(-5), false),
        ("-5 <= -3", EvalVal::Int(-5), EvalVal::Int(-3), true),
        // equal boundary
        ("5 <= 5", EvalVal::Int(5), EvalVal::Int(5), true),
    ];

    for (label, a, b, expected) in cases {
        let result = prim_reduce("leq_int", &[a.clone(), b.clone()]);
        assert_eq!(
            result,
            EvalVal::Bool(*expected),
            "{label}: leq_int must agree with the independent order on ℤ"
        );
    }
}

/// surface/numbers/leq-int-bignum-differential-oracle — derived `lt` composite
/// (soundness, hard-AC): `lt a b := ¬(leq_int b a)`, no `lt_int` primitive.
#[test]
fn derived_lt_composes_leq_and_not() {
    assert_eq!(
        derived_lt_int(&EvalVal::Int(5), &EvalVal::Int(5)),
        EvalVal::Bool(false),
        "lt 5 5 must be false (not strictly less)"
    );
    let i64_max = EvalVal::Int(i64::MAX);
    assert_eq!(
        derived_lt_int(&i64_max, &i64_max_plus_1()),
        EvalVal::Bool(true),
        "lt (i64::MAX) (i64::MAX + 1) must be true"
    );
    assert_eq!(
        derived_lt_int(&neg(EvalVal::Int(5)), &EvalVal::Int(3)),
        EvalVal::Bool(true),
        "lt (-5) 3 must be true"
    );
    assert_eq!(
        derived_lt_int(&EvalVal::Int(3), &neg(EvalVal::Int(5))),
        EvalVal::Bool(false),
        "lt 3 (-5) must be false"
    );
}
