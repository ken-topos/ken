//! `IntN<->Int` conversion floor + `checked_*`/`saturating_*` DEMOTE
//! acceptance tests (`docs/program/wp/conversions-intn-floor.md`,
//! `spec/10-kernel/18a-primitive-registry.md §5.7`,
//! `conformance/surface/numbers/seed-numbers.md` AC5).
//!
//! Every case drives the REAL producer (`ken-elaborator/src/conversions.rs`'s
//! elaborated `intToIntN`/`checked*IntN`/`saturating*IntN` views, and the
//! native `*_to_int`/`int_to_*_raw`/`neg_intN` primitives) via
//! `elaborate_decl_v1` + `eval` — never a hand-fed expected value. A bare
//! integer literal in an `IntN`-typed argument position elaborates AT that
//! type directly (expected-type literal defaulting, `35 §4.1`), so every
//! call below is a single top-level `view`, no helper wrapper needed;
//! negative operands (this grammar has no unary negation) are built via
//! `int_to_{snake}_raw (sub_int 0 <lit>)`, the same workaround
//! `decimal_char.rs` established.

use ken_elaborator::{ElabEnv, NumericLitVal};
use ken_interp::eval::{eval, eval_vals_eq, EvalStore, EvalVal};
use ken_kernel::Decl;

/// `w.max`/`w.min` as an `EvalVal` — `UInt64::MAX` (18446744073709551615)
/// exceeds `i64::MAX`, so it reduces to `EvalVal::BigInt`, not `Int`; a
/// plain `i64` literal comparison would panic/mismatch on that width.
fn expected_eval(n: i128) -> EvalVal {
    EvalVal::from(n)
}

/// Compare an `EvalVal` against an expected `i128`, correct across the
/// `Int`/`BigInt` representation boundary (`eval_vals_eq`, not `==`).
fn assert_eval_eq(actual: &EvalVal, expected: i128, msg: &str) {
    assert!(
        eval_vals_eq(actual, &expected_eval(expected)),
        "{}: got {:?}, expected {}",
        msg,
        actual,
        expected
    );
}

fn eval_view(src: &str) -> EvalVal {
    let mut env = ElabEnv::new().expect("prelude init");
    let r = env.elaborate_decl_v1(src).expect("elaborates");
    let mut store = EvalStore::new();
    let mkdecimalpair_id = env.prelude_env.mkdecimalpair_id;
    for (id, v) in &env.num_values {
        let val = match v {
            NumericLitVal::Int(n) => EvalVal::from(*n),
            NumericLitVal::Float(f) => EvalVal::Float(*f),
            NumericLitVal::Float32(f) => EvalVal::Float32(*f),
            NumericLitVal::Decimal { coeff, exp } => {
                ken_interp::decimal_value(mkdecimalpair_id, *coeff, *exp)
            }
            NumericLitVal::Str(s) => EvalVal::Str(s.clone()),
        };
        store.num_values.insert(*id, val);
    }
    match env.env.lookup(r.def_id) {
        Some(Decl::Transparent { body, .. }) => eval(&[], body, &env.env, &mut store),
        other => panic!("expected a checked Transparent def, got {:?}", other.map(|_| ())),
    }
}

/// Whether a single decl elaborates OK against a fresh prelude.
fn elaborates_ok(src: &str) -> bool {
    let mut env = ElabEnv::new().expect("prelude init");
    env.elaborate_decl(src).is_ok()
}

/// `Some`'s args are `[type-witness (Neutral), value]`; `None`'s is
/// `[type-witness]`. Returns `None` (Rust) for the `None` (Ken) case,
/// `Some(payload)` (the raw `EvalVal`, `Int` or `BigInt`) otherwise.
fn as_option_val(v: &EvalVal) -> Option<EvalVal> {
    match v {
        EvalVal::Ctor { args, .. } if args.len() == 2 => Some(args[1].clone()),
        EvalVal::Ctor { args, .. } if args.len() == 1 => None,
        other => panic!("expected an Option Ctor, got {:?}", other),
    }
}

fn assert_some_eq(v: &EvalVal, expected: i128, msg: &str) {
    match as_option_val(v) {
        Some(payload) => assert!(
            eval_vals_eq(&payload, &expected_eval(expected)),
            "{}: got Some({:?}), expected Some({})",
            msg,
            payload,
            expected
        ),
        None => panic!("{}: got None, expected Some({})", msg, expected),
    }
}

fn assert_none(v: &EvalVal, msg: &str) {
    assert!(as_option_val(v).is_none(), "{}: got Some, expected None", msg);
}

struct Width {
    name: &'static str,
    snake: &'static str,
    min: i128,
    max: i128,
    signed: bool,
}

const WIDTHS: &[Width] = &[
    Width { name: "Int8",   snake: "int8",   min: -128,                 max: 127,                   signed: true },
    Width { name: "Int16",  snake: "int16",  min: -32768,                max: 32767,                 signed: true },
    Width { name: "Int32",  snake: "int32",  min: -2147483648,           max: 2147483647,            signed: true },
    Width { name: "Int64",  snake: "int64",  min: -9223372036854775808,  max: 9223372036854775807,   signed: true },
    Width { name: "UInt8",  snake: "uint8",  min: 0, max: 255,                        signed: false },
    Width { name: "UInt16", snake: "uint16", min: 0, max: 65535,                      signed: false },
    Width { name: "UInt32", snake: "uint32", min: 0, max: 4294967295,                 signed: false },
    Width { name: "UInt64", snake: "uint64", min: 0, max: 18446744073709551615,       signed: false },
];

/// A Ken-source expression for `w`'s exact `T_MIN`, typed `Int` (not `IntN`
/// — used where a bare narrowing check expects a raw `Int` argument).
fn min_int_expr(w: &Width) -> String {
    if w.min < 0 {
        format!("(sub_int 0 {})", -w.min)
    } else {
        w.min.to_string()
    }
}

/// A Ken-source expression producing `w.min` AS AN `IntN` VALUE (via the
/// raw narrowing cast) — used where an `IntN`-typed argument is expected
/// and the value is negative (no literal-defaulting path for negatives).
fn min_intn_expr(w: &Width) -> String {
    format!("(int_to_{}_raw {})", w.snake, min_int_expr(w))
}

// ── AC1 — the round-trip law `narrow ∘ widen = id` (soundness, hard-AC) ────

#[test]
fn ac1_round_trip_law_at_max_every_width() {
    for w in WIDTHS {
        let v = eval_view(&format!(
            "view t : Option {name} = intTo{name} ({snake}_to_int {max})",
            name = w.name,
            snake = w.snake,
            max = w.max,
        ));
        assert_some_eq(&v, w.max, &format!("{}: round-trip at T_MAX", w.name));
    }
}

#[test]
fn ac1_round_trip_law_at_min_every_signed_width() {
    for w in WIDTHS.iter().filter(|w| w.signed) {
        let v = eval_view(&format!(
            "view t : Option {name} = intTo{name} ({snake}_to_int {min_intn})",
            name = w.name,
            snake = w.snake,
            min_intn = min_intn_expr(w),
        ));
        assert_some_eq(&v, w.min, &format!("{}: round-trip at T_MIN", w.name));
    }
}

// ── AC2 — the narrowing boundary: exact edges, never a silent wrap ─────────

#[test]
fn ac2_narrowing_boundary_every_width() {
    for w in WIDTHS {
        let at_max = eval_view(&format!("view t : Option {name} = intTo{name} {max}", name = w.name, max = w.max));
        assert_some_eq(&at_max, w.max, &format!("{}: T_MAX", w.name));

        let above_max = eval_view(&format!(
            "view t : Option {name} = intTo{name} (add_int {max} 1)",
            name = w.name,
            max = w.max,
        ));
        assert_none(&above_max, &format!("{}: T_MAX+1", w.name));

        let at_min = eval_view(&format!(
            "view t : Option {name} = intTo{name} {min}",
            name = w.name,
            min = min_int_expr(w),
        ));
        assert_some_eq(&at_min, w.min, &format!("{}: T_MIN", w.name));

        let below_min = eval_view(&format!(
            "view t : Option {name} = intTo{name} (sub_int {min} 1)",
            name = w.name,
            min = min_int_expr(w),
        ));
        assert_none(&below_min, &format!("{}: T_MIN-1", w.name));
    }
}

// ── AC3 — seed AC5, the existing oracle: verdict must FLIP ─────────────────

/// surface/numbers/no-implicit-cross-type-coercion (soundness, hard-AC):
/// `(x:Int) + (y:Int64)` with no conversion rejects — no implicit widening.
#[test]
fn ac3_no_implicit_cross_type_coercion_rejects() {
    assert!(
        !elaborates_ok("view rejectMix (x : Int) (y : Int64) : Int = x + y"),
        "Int + Int64 with no explicit conversion must be a type error"
    );
}

/// surface/numbers/explicit-conversion-is-partial-option (soundness,
/// hard-AC): the explicit, partial narrowing conversion accepts, typed
/// `Option Int64` — the non-degenerate flip partner: same underlying types
/// (`Int`/`Int64`), explicit conversion accepts while implicit mixed-op
/// (above) rejects.
#[test]
fn ac3_explicit_conversion_accepts_as_option() {
    assert!(
        elaborates_ok("view acceptConv (x : Int) : Option Int64 = intToInt64 x"),
        "the explicit Int -> Option Int64 conversion must accept"
    );
}

// ── AC4 — demote is behavior-preserving at the overflow boundary ──────────

#[test]
fn ac4_checked_add_boundary_every_width() {
    for w in WIDTHS {
        // In-range at the very edge: (T_MAX - 1) + 1 = T_MAX -> Some(T_MAX).
        let in_range = eval_view(&format!(
            "view t : Option {name} = checkedAdd{name} {near_max} 1",
            name = w.name,
            near_max = w.max - 1,
        ));
        assert_some_eq(&in_range, w.max, &format!("{}: checkedAdd in-range edge", w.name));

        // Overflow: T_MAX + 1 -> None, never a wrapped/truncated Some.
        let overflow = eval_view(&format!("view t : Option {name} = checkedAdd{name} {max} 1", name = w.name, max = w.max));
        assert_none(&overflow, &format!("{}: checkedAdd T_MAX+1", w.name));
    }
}

#[test]
fn ac4_checked_sub_underflow_every_width() {
    for w in WIDTHS {
        let underflow = eval_view(&format!(
            "view t : Option {name} = checkedSub{name} {min} 1",
            name = w.name,
            min = if w.signed { min_intn_lit_arg(w) } else { "0".to_string() },
        ));
        assert_none(&underflow, &format!("{}: checkedSub T_MIN-1", w.name));
    }
}

/// `checkedSub{Name}` takes an `IntN`-typed first argument; a negative
/// `T_MIN` needs the raw-cast form since it can't be spelled as a literal.
fn min_intn_lit_arg(w: &Width) -> String {
    min_intn_expr(w)
}

#[test]
fn ac4_saturating_add_clamps_at_boundary_every_width() {
    for w in WIDTHS {
        // T_MAX + T_MAX always overflows every width (including UInt64) ->
        // clamps to T_MAX, never wraps.
        let v = eval_view(&format!("view t : {name} = saturatingAdd{name} {max} {max}", name = w.name, max = w.max));
        assert_eval_eq(&v, w.max, &format!("{}: saturatingAdd(T_MAX,T_MAX) clamp", w.name));
    }
}

#[test]
fn ac4_saturating_sub_clamps_at_lower_boundary_every_width() {
    for w in WIDTHS {
        // T_MIN - T_MAX always underflows (or, for unsigned, 0 - MAX) ->
        // clamps to T_MIN.
        let v = eval_view(&format!(
            "view t : {name} = saturatingSub{name} {min} {max}",
            name = w.name,
            min = if w.signed { min_intn_lit_arg(w) } else { "0".to_string() },
            max = w.max,
        ));
        assert_eval_eq(&v, w.min, &format!("{}: saturatingSub(T_MIN,T_MAX) clamp", w.name));
    }
}

#[test]
fn ac4_saturating_add_no_overflow_still_exact() {
    // Non-regression: the interior (no clamp needed) still computes the
    // exact sum, not a clamped or wrapped value.
    let v = eval_view("view t : Int8 = saturatingAddInt8 10 20");
    assert_eq!(v, EvalVal::Int(30));
}

// ── AC5 — neg_intN stays native, checked (degrades, never wraps) ──────────

#[test]
fn ac5_neg_intn_degrades_on_min_every_signed_width() {
    for w in WIDTHS.iter().filter(|w| w.signed) {
        let v = eval_view(&format!(
            "view t : {name} = neg_{snake} {min_intn}",
            name = w.name,
            snake = w.snake,
            min_intn = min_intn_expr(w),
        ));
        assert_eq!(v, EvalVal::Neutral, "{}: neg(MIN) must degrade to Neutral, never wrap", w.name);
    }
}

#[test]
fn ac5_neg_intn_computes_in_range_every_signed_width() {
    for w in WIDTHS.iter().filter(|w| w.signed) {
        let v = eval_view(&format!("view t : {name} = neg_{snake} 5", name = w.name, snake = w.snake));
        assert_eq!(v, EvalVal::Int(-5), "{}: neg(5) must be -5", w.name);
    }
}

/// `neg_uintN` is out of scope (`18a` names no such op — unsigned negation
/// of any nonzero value is out of range by construction). Verify by
/// absence: the symbol was never registered.
#[test]
fn neg_uintn_out_of_scope_unregistered() {
    for w in WIDTHS.iter().filter(|w| !w.signed) {
        assert!(
            !elaborates_ok(&format!("view t : {name} = neg_{snake} 5", name = w.name, snake = w.snake)),
            "{}: neg_{} must not be registered (unsigned negation is out of scope)",
            w.name,
            w.snake,
        );
    }
}

// ── out of scope, verified by absence ──────────────────────────────────────

#[test]
fn float_conversion_floor_out_of_scope() {
    assert!(
        !elaborates_ok("view t : Float = int_to_float 5"),
        "Int.toFloat/the Float conversion arms are a separate tranche, not built here"
    );
}
