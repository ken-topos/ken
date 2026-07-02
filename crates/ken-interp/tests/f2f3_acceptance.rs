//! F2/F3 acceptance tests (`docs/program/wp/F2F3-reducer-degrade.md`,
//! `spec/10-kernel/18a-primitive-registry.md §5`,
//! `conformance/surface/numbers/seed-numbers.md` AC3/AC4).
//!
//! F2: bare fixed-width `add/sub/mul_intN`/`add_uintN` must degrade
//! (`EvalVal::Neutral`) on overflow, never wrap — the runtime face of the
//! no-overflow obligation. The sanctioned modular class (`wrapping_*_intN`)
//! is the only path permitted to wrap.
//! F3: the legacy unregistered `add`/`sub`/`mul` (wrapping i64) arms are
//! retired — unregistered and unreduced.

use ken_interp::eval::{prim_reduce, EvalVal};

// ── AC3/AC4 — bare op degrades, `+%` on the SAME operands still wraps ──────

/// surface/numbers/bare-overflow-never-silently-wraps (soundness, hard-AC).
/// `(100 : Int8) + (100 : Int8)` sums to 200 in ℤ, out of `Int8` range
/// (max 127). Must NOT silently produce the wrapped value `-56`.
#[test]
fn ac4_bare_add_int8_overflow_degrades_not_wraps() {
    let a = EvalVal::Int(100);
    let b = EvalVal::Int(100);
    let result = prim_reduce("add_int8", &[a, b]);
    assert_eq!(
        result,
        EvalVal::Neutral,
        "bare add_int8 on overflowing operands must degrade to Neutral, never yield -56"
    );
}

/// The discriminating partner: `+%` (`wrapping_add_int8`) on the identical
/// overflowing operands still wraps to the modular value -56. If this test
/// and the one above were both green under a broken conversion (e.g. the
/// bare arm still secretly wrapping), the pair would fail to flip — this
/// pins the flip.
#[test]
fn ac4_wrapping_add_int8_same_operands_still_wraps() {
    let a = EvalVal::Int(100);
    let b = EvalVal::Int(100);
    let result = prim_reduce("wrapping_add_int8", &[a, b]);
    assert_eq!(
        result,
        EvalVal::Int(-56),
        "wrapping_add_int8 (+%) must still wrap on the same overflowing operands"
    );
}

/// Non-overflowing bare arithmetic is unaffected (non-regression on the
/// total case): the checked path computes the same value as before.
#[test]
fn bare_add_int8_no_overflow_still_computes() {
    let result = prim_reduce("add_int8", &[EvalVal::Int(10), EvalVal::Int(20)]);
    assert_eq!(result, EvalVal::Int(30));
}

/// Every in-scope bare fixed-width op/width degrades on its own overflow
/// boundary — completeness across the whole F2 arm set, not just Int8/add.
#[test]
fn ac3_all_bare_fixed_width_arms_degrade_on_overflow() {
    let cases: &[(&str, i64, i64)] = &[
        ("add_int8", 127, 1),
        ("sub_int8", -128, 1),
        ("mul_int8", 127, 2),
        ("add_int16", i16::MAX as i64, 1),
        ("sub_int16", i16::MIN as i64, 1),
        ("mul_int16", i16::MAX as i64, 2),
        ("add_int32", i32::MAX as i64, 1),
        ("sub_int32", i32::MIN as i64, 1),
        ("mul_int32", i32::MAX as i64, 2),
        ("add_int64", i64::MAX, 1),
        ("sub_int64", i64::MIN, 1),
        ("mul_int64", i64::MAX, 2),
        ("add_uint8", 255, 1),
        ("add_uint16", 65535, 1),
        ("add_uint32", (u32::MAX) as i64, 1),
    ];
    for (op, a, b) in cases {
        let result = prim_reduce(op, &[EvalVal::Int(*a), EvalVal::Int(*b)]);
        assert_eq!(
            result,
            EvalVal::Neutral,
            "{op}({a}, {b}) must degrade to Neutral on overflow, never wrap"
        );
    }
}

// ── F3 — legacy `add`/`sub`/`mul` retired: unregistered AND unreduced ──────

/// surface/numbers/legacy-add-sub-mul-retired (hard-AC): after deletion,
/// `prim_reduce` no longer recognizes the legacy wrapping-i64 symbols —
/// they fall through to the generic stuck arm.
#[test]
fn f3_legacy_add_sub_mul_unreduced() {
    for op in ["add", "sub", "mul"] {
        let result = prim_reduce(op, &[EvalVal::Int(1), EvalVal::Int(2)]);
        assert_eq!(
            result,
            EvalVal::Neutral,
            "legacy '{op}' must be unreduced (stuck) post-retirement"
        );
    }
}

/// The other half of the F3 guard: `"add"`/`"sub"`/`"mul"` mint no primitive
/// in the elaborator's registration tables — grep the real producer source,
/// not just this test's own behavior (a mint elsewhere would make the arm
/// above reachable from real surface programs again).
#[test]
fn f3_legacy_add_sub_mul_unregistered_in_elaborator() {
    let numbers_src = include_str!("../../ken-elaborator/src/numbers.rs");
    for op in ["\"add\"", "\"sub\"", "\"mul\""] {
        assert!(
            !numbers_src.contains(&format!("reg_binop!({op}")),
            "elaborator must not register a reg_binop! for the legacy {op} symbol"
        );
    }
}
