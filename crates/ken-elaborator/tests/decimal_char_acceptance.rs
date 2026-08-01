//! Decimal/Char DEMOTE→derived acceptance tests
//! (`conformance/surface/numbers/seed-decimal-char-demote.md`).
//!
//! Covers AC-D1/D2 (Decimal exact derivation, the F4 flip), AC-C1/C2/C3
//! (Char refinement, derived ops, surrogate/OOR rejection), and pin-1 (the
//! `isScalar` Ω-encoding's codepoint-collapse). AC-D3 (`Num`/`DecEq Decimal`
//! law instances) and `Ord Char` antisymmetry are re-homed to the
//! lawful-classes lane (Steward ruling) — not covered here. Pin-2 (`String`
//! → `Char` extraction computing the scalar proof) is deferred to the
//! extraction feature (Architect ruling, `evt_164tmsbevyk51`) — not covered
//! here either.
//!
//! Every case drives the REAL derived producer (`decimalAdd`/`decimalMul`/
//! `decimalEq`/`eqChar`/`leqChar`/`intToChar`, elaborated Ken definitions in
//! `ken-elaborator/src/decimal_char.rs`) via `elaborate_decl_v1` + `eval` —
//! never a hand-fed expected value
//! ([[conformance-hand-feeds-the-deliverable]]).

use ken_elaborator::{ElabEnv, NumericLitVal};
use ken_interp::eval::{eval, EvalStore, EvalVal};
use ken_kernel::Decl;

/// Elaborate and evaluate a single top-level `view` declaration, seeding the
/// literal side-table from the elaborator's own `num_values` (never a
/// hand-built literal value).
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

/// Extract `(coeff, exp)` from a `Decimal` `Ctor` value — never constructs
/// one, only inspects the real producer's output.
fn as_decimal(v: &EvalVal) -> (i64, i64) {
    match v {
        EvalVal::Ctor { args, .. } => {
            let coeff = match &args[0] {
                EvalVal::Int(n) => *n,
                other => panic!("expected Int coeff, got {:?}", other),
            };
            let exp = match &args[1] {
                EvalVal::Int(n) => *n,
                other => panic!("expected Int exp, got {:?}", other),
            };
            (coeff, exp)
        }
        other => panic!("expected a Decimal Ctor, got {:?}", other),
    }
}

// ── AC-D1/D2 — Decimal derivation is exact; the F4 flip (soundness) ────────

/// surface/numbers/decimal-mul-exact-flips-vs-saturating (soundness)
#[test]
fn decimal_mul_exact_flips_vs_saturating() {
    // Decimal(coeff=10^10, exp=0) * Decimal(coeff=10^10, exp=0):
    // coeff product 10^20 > i64::MAX (~9.22e18) — the old native
    // `mul_decimal` used `saturating_mul`, clamping to i64::MAX. The
    // derived `decimalMul` runs bignum `mul_int`, exact.
    let result = eval_view(
        "const t = decimalMul (MkDecimalPair 10000000000 0) (MkDecimalPair 10000000000 0)",
    );
    match &result {
        EvalVal::Ctor { args, .. } => {
            match &args[0] {
                EvalVal::BigInt(n) => {
                    assert_eq!(n.to_string(), "100000000000000000000", "exact 10^20, no saturation");
                }
                EvalVal::Int(n) => panic!("10^20 must widen to BigInt, got Int({})", n),
                other => panic!("expected Int/BigInt coeff, got {:?}", other),
            }
            match &args[1] {
                EvalVal::Int(0) => {}
                other => panic!("expected exp Int(0), got {:?}", other),
            }
        }
        other => panic!("expected a Decimal Ctor, got {:?}", other),
    }
}

/// surface/numbers/decimal-eq-distinct-flips-vs-false-true (soundness) —
/// the F4 closure: two DISTINCT decimals that the old saturating
/// `decimal_eq` compared (wrongly) `True`.
#[test]
fn decimal_eq_distinct_flips_vs_false_true() {
    // a = Decimal(coeff=i64::MAX, exp=0); b = Decimal(coeff=i64::MAX, exp=1)
    // (b = 10 * a exactly: 92233720368547758070 vs 9223372036854775807).
    // Old `decimal_eq` aligned b's coeff via `saturating_mul(10)`, which
    // saturates to i64::MAX, then compared `i64::MAX == i64::MAX` -> true
    // (the F4 hole: two distinct decimals comparing equal). The derived
    // `decimalEq` aligns exactly via `decimalPow10`/`mul_int`, so this must
    // reduce to `false`.
    let result = eval_view(
        "const t = decimalEq (MkDecimalPair 9223372036854775807 0) \
                             (MkDecimalPair 9223372036854775807 1)",
    );
    assert_eq!(
        result,
        EvalVal::Bool(false),
        "distinct decimals must compare unequal — the F4 saturating false-True hole must not recur"
    );
}

/// A same-value check on the same vector, aligned the other direction, to
/// confirm `decimalEq` isn't just accidentally always false.
#[test]
fn decimal_eq_same_value_both_alignments_true() {
    let result = eval_view(
        "const t = decimalEq (MkDecimalPair 9223372036854775807 1) \
                             (MkDecimalPair 92233720368547758070 0)",
    );
    assert_eq!(
        result,
        EvalVal::Bool(true),
        "9223372036854775807 * 10^1 (as coeff 9223372036854775807, exp 1) \
         must equal the same value spelled as (coeff 92233720368547758070, exp 0)"
    );
}

/// Sanity: `decimalAdd` at the same exponent needs no alignment.
#[test]
fn decimal_add_same_exponent() {
    let result = eval_view("const t = decimalAdd (MkDecimalPair 1 0) (MkDecimalPair 2 0)");
    assert_eq!(as_decimal(&result), (3, 0));
}

// ── AC-C1 — Char refinement (soundness) ─────────────────────────────────────

/// surface/numbers/char-is-isscalar-refinement (soundness)
#[test]
fn char_is_isscalar_refinement() {
    // `Char` erases to `Int` (refinement-erasure) — a value of type `Char`
    // reduces to a plain `Int`, never a `List`/`u32`-tagged carrier.
    let result = eval_view("const t : Char = 65");
    assert_eq!(result, EvalVal::Int(65));
}

// ── AC-C2 — derived Char ops over the projection (soundness) ───────────────

/// surface/numbers/char-eq-and-ord-on-projection (soundness)
#[test]
fn char_eq_and_ord_on_projection() {
    assert_eq!(eval_view("const t = eqChar 65 65"), EvalVal::Bool(true));
    assert_eq!(eval_view("const t = eqChar 65 66"), EvalVal::Bool(false));
    // The order PAIR (accept a<=b while reject b<=a) — a single accept is
    // green-vs-green under an orientation flip.
    assert_eq!(eval_view("const t = leqChar 65 66"), EvalVal::Bool(true));
    assert_eq!(eval_view("const t = leqChar 66 65"), EvalVal::Bool(false));
    assert_eq!(eval_view("const t = charToInt 65"), EvalVal::Int(65));
}

// ── AC-C3 — surrogate/OOR reject, flips vs isScalar:=true (soundness) ──────

/// surface/numbers/int-to-char-rejects-surrogate-and-oor (soundness)
#[test]
fn int_to_char_rejects_surrogate_and_oor() {
    // The non-degenerate PAIR: reject surrogate/OOR *while* a valid scalar
    // accepts — a single valid-accept case is green-vs-green under a stub
    // `isScalar := true` ([[two-arm-producer-needs-a-case-per-arm]]).
    let surrogate = eval_view("const t = intToChar 55296"); // 0xD800
    let oor = eval_view("const t = intToChar 1114112"); // 0x110000
    let valid = eval_view("const t = intToChar 65"); // 'A'

    let (surrogate_id, oor_id, valid_id) = match (&surrogate, &oor, &valid) {
        (
            EvalVal::Ctor { id: s, .. },
            EvalVal::Ctor { id: o, .. },
            EvalVal::Ctor { id: v, .. },
        ) => (*s, *o, *v),
        other => panic!("expected Ctor (Option) results, got {:?}", other),
    };
    assert_eq!(surrogate_id, oor_id, "surrogate and OOR must both reduce to the same ctor (None)");
    assert_ne!(valid_id, surrogate_id, "a valid scalar must reduce to a DIFFERENT ctor (Some)");
}

// ── Char pin 1 — the Ω-encoding is structural, not a naive disjunction ─────

/// surface/numbers/char-deceq-collapses-on-codepoint (soundness, hard-AC) —
/// structural half: `isScalar`'s definition head is `IsTrue (<computed
/// Bool>)`, never a raw `∨`/`∃`/multi-ctor form. Grepped directly against
/// the producer source (not a value witness — no value can prove a sort is
/// absent).
#[test]
fn char_deceq_pin1_structural_encoding() {
    let src = include_str!("../src/decimal_char.rs");
    assert!(
        src.contains("fn isScalar (c : Int) : Prop = IsTrue (inRangeBool c)"),
        "isScalar's definition head must be `IsTrue (<computed Bool>)` — \
         never a raw `∨`/`∃`/multi-ctor form as its own Ω-sort (`16 §1.3`); \
         the required value-level `or_bool`/`and_bool` inside `inRangeBool` \
         is a distinct, permitted layer (composing the Bool computation \
         that IsTrue then wraps), not the forbidden sort-level disjunction"
    );
}

/// Value-consequence of pin 1: `eqChar` (which routes through `eq_int` on
/// the projection, per pin 1's payoff) agrees with direct `Int` equality —
/// observable only because `IsTrue` is proof-irrelevant so no separate
/// scalar-proof component can desync the comparison.
#[test]
fn char_deceq_pin1_value_consequence() {
    assert_eq!(eval_view("const t = eqChar 97 97"), EvalVal::Bool(true));
}
