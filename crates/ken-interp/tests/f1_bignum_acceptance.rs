//! F1 acceptance tests (`conformance/surface/numbers/seed-f1-bignum-int.md`).
//!
//! Covers AC1 (no-wrap totality across the i128 ceiling) and AC2 (the
//! independent differential oracle) against `ken_interp::eval::prim_reduce`
//! directly — the surface lexer's `NumericLitVal::Int(i128)` cap is out of
//! F1's scope, so operands beyond `i128` are constructed in Rust, not parsed
//! from `.ken` source text. AC3 (store round-trip) needs the crate-private
//! `to_rt`/`intern` producer and lives in `src/eval.rs`'s own test module.

use ken_interp::eval::{prim_reduce, EvalVal};
use num_bigint::BigInt;

/// Construct `2^n` as an `EvalVal` via `Shl` — a test-input constructor,
/// never the `add_int`/`sub_int`/`mul_int` reduction under audit.
fn pow2(n: u32) -> EvalVal {
    big(BigInt::from(1u8) << n)
}

fn big(n: BigInt) -> EvalVal {
    match n.to_string().parse::<i64>() {
        Ok(i) => EvalVal::Int(i),
        Err(_) => EvalVal::BigInt(n),
    }
}

/// Extract the base-10 string of an Int/BigInt result via `Display` — a
/// formatting call, never the arithmetic op under audit.
fn to_decimal_string(v: &EvalVal) -> String {
    match v {
        EvalVal::Int(n) => n.to_string(),
        EvalVal::BigInt(n) => n.to_string(),
        other => panic!("expected an Int/BigInt result, got {:?}", other),
    }
}

// ── AC1 — no-wrap totality across the i128 ceiling (soundness) ─────────────

/// surface/numbers/f1-mul-int-crosses-i128-ceiling (soundness)
#[test]
fn f1_mul_int_crosses_i128_ceiling() {
    let result = prim_reduce("mul_int", &[pow2(127), EvalVal::Int(2)]);
    assert_eq!(
        to_decimal_string(&result),
        "340282366920938463463374607431768211456", // 2^128, golden (independent computation)
        "2^127 * 2 must reduce to the exact 2^128, no panic/wrap"
    );
}

/// surface/numbers/f1-mul-int-2^64-squared (soundness)
#[test]
fn f1_mul_int_2_64_squared() {
    let result = prim_reduce("mul_int", &[pow2(64), pow2(64)]);
    assert_eq!(
        to_decimal_string(&result),
        "340282366920938463463374607431768211456", // 2^128
        "2^64 * 2^64 must reduce exactly even though each operand fits i128"
    );
}

/// surface/numbers/f1-add-int-crosses-i128-ceiling (soundness)
#[test]
fn f1_add_int_crosses_i128_ceiling() {
    // 2^128 - 1, built via Sub (test-input construction, not the op under audit).
    let two_128_minus_1 = pow2(128);
    let two_128_minus_1 = match two_128_minus_1 {
        EvalVal::BigInt(n) => EvalVal::BigInt(n - BigInt::from(1u8)),
        other => panic!("expected BigInt, got {:?}", other),
    };
    let result = prim_reduce("add_int", &[two_128_minus_1, EvalVal::Int(1)]);
    assert_eq!(
        to_decimal_string(&result),
        "340282366920938463463374607431768211456", // 2^128
        "(2^128 - 1) + 1 must reduce to the exact 2^128, no wrap"
    );
}

/// surface/numbers/f1-product-chain-exceeds-2^1000 (soundness)
#[test]
fn f1_product_chain_exceeds_2_1000() {
    let factor = pow2(128);
    let mut acc = EvalVal::Int(1);
    for _ in 0..8 {
        acc = prim_reduce("mul_int", &[acc, factor.clone()]);
    }
    // golden: (2^128)^8 = 2^1024, precomputed independently.
    let golden = "179769313486231590772930519078902473361797697894230657273430081157732675805500963132708477322407536021120113879871393357658789768814416622492847430639474124377767893424865485276302219601246094119453082952085005768838150682342462881473913110540827237163350510684586298239947245938479716304835356329624224137216";
    assert_eq!(
        to_decimal_string(&acc),
        golden,
        "eight-factor 2^128 chain must reduce to the exact 2^1024 compositionally"
    );
}

// ── AC2 — independent differential oracle (soundness, hard-AC) ─────────────
//
// The expected values below are golden vectors: exact powers of two and
// their sums/products, computed independently of `num-bigint` (elementary
// arithmetic identities — `2^127*2 = 2^128`, `(-a)*(-b) = a*b`, `a*0 = 0` —
// verified externally, not via the production crate's own multiply/add).
// Only `Display`/`to_string()` (base-10 formatting, a distinct code path
// from `Mul`/`Add`) is used to compare against the production reduction.

/// surface/numbers/f1-oracle-independent-reference (soundness, hard-AC)
#[test]
fn f1_oracle_magnitude_boundary_matrix() {
    let cases: &[(&str, EvalVal, EvalVal, &str)] = &[
        ("add_int", pow2(63), EvalVal::Int(1), "9223372036854775809"),
        ("add_int", pow2(64), EvalVal::Int(1), "18446744073709551617"),
        (
            "mul_int",
            pow2(63),
            EvalVal::Int(2),
            "18446744073709551616", // 2^64
        ),
        (
            "mul_int",
            pow2(127),
            EvalVal::Int(2),
            "340282366920938463463374607431768211456", // 2^128
        ),
        (
            "sub_int",
            pow2(128),
            EvalVal::Int(1),
            "340282366920938463463374607431768211455",
        ),
    ];
    for (op, a, b, expected) in cases {
        let result = prim_reduce(op, &[a.clone(), b.clone()]);
        assert_eq!(
            to_decimal_string(&result),
            *expected,
            "{op} boundary case must agree with the independent reference"
        );
    }
}

/// surface/numbers/f1-oracle-independent-reference — sign boundary (soundness, hard-AC)
#[test]
fn f1_oracle_sign_boundary() {
    // (-2^127) * 2 = -2^128 — mixed-sign mul.
    let neg_2_127 = match pow2(127) {
        EvalVal::BigInt(n) => EvalVal::BigInt(-n),
        other => panic!("expected BigInt, got {:?}", other),
    };
    let result = prim_reduce("mul_int", &[neg_2_127.clone(), EvalVal::Int(2)]);
    assert_eq!(
        to_decimal_string(&result),
        "-340282366920938463463374607431768211456",
        "(-2^127) * 2 must equal -2^128"
    );

    // (-a) * (-b) = a * b — both negative operands, positive result.
    let neg_2_64 = match pow2(64) {
        EvalVal::BigInt(n) => EvalVal::BigInt(-n),
        other => panic!("expected BigInt, got {:?}", other),
    };
    let result = prim_reduce("mul_int", &[neg_2_64.clone(), neg_2_64]);
    assert_eq!(
        to_decimal_string(&result),
        "340282366920938463463374607431768211456",
        "(-2^64) * (-2^64) must equal +2^128 (double-negative)"
    );

    // a * 0 = 0.
    let result = prim_reduce("mul_int", &[pow2(127), EvalVal::Int(0)]);
    assert_eq!(to_decimal_string(&result), "0", "a * 0 must equal 0");
}

/// surface/numbers/f1-oracle-independent-reference — `eq_int` at the boundary
/// (soundness, hard-AC): equality decided on the bignum representation, never
/// an `i128`/`i64` image that would collapse distinct `> i128` values.
#[test]
fn f1_oracle_eq_int_boundary() {
    let a = pow2(128);
    let b = pow2(128);
    assert_eq!(
        prim_reduce("eq_int", &[a, b]),
        EvalVal::Bool(true),
        "eq_int (2^128) (2^128) must be true"
    );

    let a = pow2(128);
    let b = match pow2(128) {
        EvalVal::BigInt(n) => EvalVal::BigInt(n + BigInt::from(1u8)),
        other => panic!("expected BigInt, got {:?}", other),
    };
    assert_eq!(
        prim_reduce("eq_int", &[a, b]),
        EvalVal::Bool(false),
        "eq_int (2^128) (2^128 + 1) must be false — no truncated-residue collapse"
    );
}
