//! `Decimal`/`Char` DEMOTE→derived (`18a §5.6`/`§5.9`, Phase-2 tranche #2).
//!
//! Registers `Decimal`/`Char` as **derived** Ken definitions over F1's exact
//! `Int` plus the pulled-up `leq_int` ordering arm (`18a §5.2.2`) — not
//! primitive types. No `reg_ty!`/`reg_binop!`/`reg_cmpop!` for either type;
//! everything here is `declare_def`/`declare_inductive` (checked bodies),
//! per Architect's ruling (`evt_7dwtqbmka62bf`): zero new
//! `declare_postulate`/`declare_primitive` for the Decimal/Char surface
//! proper. The one exception is `decimal_pow10_unbounded` below — a
//! deliberately-never-reducing internal marker, not a Decimal/Char-facing
//! primitive (see its doc comment).
//!
//! Called from `prelude::register_prelude`, after `Equal`/`And`/`Prop`/`tt`
//! are declared (needed for `IsTrue`) and before the later `Char`-dependent
//! String-ops registration in that same function.

use ken_kernel::{declare_postulate, GlobalId, Term};

use crate::error::ElabError;
use crate::numbers::{AddEntry, EqEntry};
use crate::ElabEnv;

/// GlobalIds for the derived `Decimal`/`Char` surface.
pub struct DecimalCharEnv {
    pub decimal_id: GlobalId,
    pub mkdecimalpair_id: GlobalId,
    pub char_id: GlobalId,
}

/// Exact decimal literal `10^k` as a base-10 string (a leading `1` followed
/// by `k` zeros) — used to generate the bounded `decimalPow10` cascade.
fn pow10_literal(k: u32) -> String {
    let mut s = String::with_capacity(k as usize + 1);
    s.push('1');
    for _ in 0..k {
        s.push('0');
    }
    s
}

/// Exact, non-clamped for `k ∈ [0, MAX_SHIFT]` — generous (well beyond the
/// retired `.min(18)` i64 cap) while staying a FIXED, SCT-trivial
/// (non-recursive) unroll. Architect's ruling (`evt_7dwtqbmka62bf`, Q2): a
/// general Int-indexed loop fails SCT outright (no structural descent on
/// the opaque `Int`), so a genuinely unbounded `pow10` cannot be built this
/// tranche without inventing ad hoc termination machinery — out of scope.
/// Unbounded `Δexp` is a NAMED, tracked forward obligation instead (CV owns
/// the matching future conformance case).
///
/// Bounded above by the surface LEXER's own `i128` literal cap (a `10^k`
/// literal must itself parse — `10^38 < i128::MAX < 10^39`), independent of
/// F1's bignum `Int`, which only widens *computed* values, not literal
/// tokens. 30 stays comfortably inside that ceiling.
const MAX_SHIFT: u32 = 30;

/// The bounded `decimalPow10` cascade body: a fixed (non-self-referential)
/// chain of `eq_int`-on-Bool matches, one per `k ∈ [0, MAX_SHIFT]`, each
/// yielding the exact `10^k` literal. Beyond `MAX_SHIFT` it applies
/// `unbounded_name` — see `register_decimal_char`'s doc comment on that
/// marker. Because every branch here is a concrete literal (never
/// `saturating_*`/`.min(_)`/`clamp`), the align path this feeds is exact-or
/// -stuck, never wrong (Architect's hard-gated condition 1).
fn pow10_cascade_body(max_shift: u32, unbounded_name: &str) -> String {
    fn rec(k: u32, max_shift: u32, unbounded_name: &str) -> String {
        if k > max_shift {
            format!("{unbounded_name} k")
        } else {
            format!(
                "match (eq_int k {k}) {{ True => {lit} ; False => {rest} }}",
                k = k,
                lit = pow10_literal(k),
                rest = rec(k + 1, max_shift, unbounded_name),
            )
        }
    }
    rec(0, max_shift, unbounded_name)
}

/// Register the derived `Decimal`/`Char` surface (`18a §5.6.1`/`§5.9.1`).
pub fn register_decimal_char(elab: &mut ElabEnv) -> Result<DecimalCharEnv, ElabError> {
    // ── Decimal := DecimalPair (a fresh 2-field inductive) ──────────────
    //
    // A dedicated `DecimalPair` (not the prelude's `Prod`) avoids an
    // ordering dependency on `Prod`'s own registration point in
    // `register_prelude`; it costs nothing (reuses the EXISTING `data`
    // machinery, no new kernel flag/`Decl` variant, `18a` AC-G). `Decimal`
    // itself is a TRANSPARENT ALIAS (`declare_def`, via `def Decimal =
    // DecimalPair`) rather than referencing the inductive directly, so
    // `decimal_id` stays `Term::Const`-shaped — matching
    // `classify_add`/`classify_eq`'s `Term::Const` dispatch and
    // `elab_num_lit_checked`'s decimal-literal check exactly like `IO :=
    // ITree` (`register_prelude`, above).
    elab.elaborate_decl("data DecimalPair = MkDecimalPair Int Int")
        .map_err(|e| ElabError::Internal(format!("DecimalPair failed: {}", e)))?;
    elab.elaborate_decl("def Decimal = DecimalPair")
        .map_err(|e| ElabError::Internal(format!("Decimal alias failed: {}", e)))?;
    let decimal_id = *elab
        .globals
        .get("Decimal")
        .ok_or_else(|| ElabError::Internal("Decimal not registered".into()))?;
    let mkdecimalpair_id = *elab
        .globals
        .get("MkDecimalPair")
        .ok_or_else(|| ElabError::Internal("MkDecimalPair not registered".into()))?;

    // Internal forward-obligation marker (Architect ruling
    // `evt_7dwtqbmka62bf`): applying it never reduces (a genuine opaque
    // postulate with no `PrimReduction`/body), so `decimalPow10` goes
    // STUCK — never a clamped/wrong value — beyond `MAX_SHIFT`. This is
    // NOT part of the Decimal/Char user-facing surface (never documented,
    // never reachable from `decimalAdd`/`decimalEq`'s public signatures);
    // it exists solely so "unbounded Δexp" is a structurally honest
    // incompleteness rather than a silently wrong result. There is no
    // top-level `postulate`/`axiom` surface keyword in this grammar, so
    // this one constant is declared directly via `ken_kernel::declare_postulate`.
    let int_id = *elab
        .globals
        .get("Int")
        .ok_or_else(|| ElabError::Internal("Int not registered".into()))?;
    let int_t = Term::const_(int_id, vec![]);
    let unbounded_ty = Term::pi(int_t.clone(), int_t.clone());
    let unbounded_id = declare_postulate(&mut elab.env, vec![], unbounded_ty)
        .map_err(|e| ElabError::Internal(format!("decimalPow10Unbounded failed: {}", e)))?;
    elab.globals
        .insert("decimalPow10Unbounded".to_string(), unbounded_id);

    let pow10_src = format!(
        "fn decimalPow10 (k : Int) : Int = {}",
        pow10_cascade_body(MAX_SHIFT, "decimalPow10Unbounded")
    );
    elab.elaborate_decl(&pow10_src)
        .map_err(|e| ElabError::Internal(format!("decimalPow10 failed: {}", e)))?;

    // Exact base-10 arithmetic over the derived `(coeff, exp)` pair
    // (`18a §5.6.1(2)`): `mul` is ordering-free; `add`/`sub`/`eq` align to
    // `min(ea, eb)` via the pulled-up `leq_int`, scale via `decimalPow10`
    // (`mul_int`-exact, never `saturating_*`), then combine via `add_int`/
    // `sub_int`/`eq_int`.
    elab.elaborate_decl(
        "fn decimalAdd (d1 : Decimal) (d2 : Decimal) : Decimal = \
         match d1 { MkDecimalPair ca ea => \
         match d2 { MkDecimalPair cb eb => \
           match (leq_int ea eb) { \
             True => MkDecimalPair (add_int ca (mul_int cb (decimalPow10 (sub_int eb ea)))) ea ; \
             False => MkDecimalPair (add_int (mul_int ca (decimalPow10 (sub_int ea eb))) cb) eb \
           } } }",
    )
    .map_err(|e| ElabError::Internal(format!("decimalAdd failed: {}", e)))?;

    elab.elaborate_decl(
        "fn decimalSub (d1 : Decimal) (d2 : Decimal) : Decimal = \
         match d1 { MkDecimalPair ca ea => \
         match d2 { MkDecimalPair cb eb => \
           match (leq_int ea eb) { \
             True => MkDecimalPair (sub_int ca (mul_int cb (decimalPow10 (sub_int eb ea)))) ea ; \
             False => MkDecimalPair (sub_int (mul_int ca (decimalPow10 (sub_int ea eb))) cb) eb \
           } } }",
    )
    .map_err(|e| ElabError::Internal(format!("decimalSub failed: {}", e)))?;

    elab.elaborate_decl(
        "fn decimalMul (d1 : Decimal) (d2 : Decimal) : Decimal = \
         match d1 { MkDecimalPair ca ea => \
         match d2 { MkDecimalPair cb eb => \
           MkDecimalPair (mul_int ca cb) (add_int ea eb) \
         } }",
    )
    .map_err(|e| ElabError::Internal(format!("decimalMul failed: {}", e)))?;

    elab.elaborate_decl(
        "fn decimalEq (d1 : Decimal) (d2 : Decimal) : Bool = \
         match d1 { MkDecimalPair ca ea => \
         match d2 { MkDecimalPair cb eb => \
           match (leq_int ea eb) { \
             True => eq_int ca (mul_int cb (decimalPow10 (sub_int eb ea))) ; \
             False => eq_int (mul_int ca (decimalPow10 (sub_int ea eb))) cb \
           } } }",
    )
    .map_err(|e| ElabError::Internal(format!("decimalEq failed: {}", e)))?;

    let decimalpair_id = *elab
        .globals
        .get("DecimalPair")
        .ok_or_else(|| ElabError::Internal("DecimalPair not registered".into()))?;

    let decimal_add_id = *elab.globals.get("decimalAdd").unwrap();
    let decimal_eq_id = *elab.globals.get("decimalEq").unwrap();
    // Keyed by `decimalpair_id`, not `decimal_id`: `whnf` fully unfolds the
    // transparent `Decimal := DecimalPair` alias, so `classify_add`/
    // `classify_eq` see `Term::IndFormer{id: decimalpair_id}` on a WHNF'd
    // `Decimal`-typed term, never `Term::Const{id: decimal_id}`.
    elab.numeric_env.set_add_entry(
        decimalpair_id,
        AddEntry {
            op_id: decimal_add_id,
            wrapping_id: None,
            no_ovf_id: None,
            result_id: decimal_id,
        },
    );
    elab.numeric_env
        .set_eq_entry(decimalpair_id, EqEntry { op_id: decimal_eq_id });
    elab.numeric_env.decimal_id = decimal_id;
    elab.numeric_env.decimalpair_id = decimalpair_id;

    // ── Char := { c : Int | isScalar c } (`18a §5.9.1`) ─────────────────
    //
    // `IsTrue` bridges a decidable `Bool` to the proof-irrelevant `Ω`
    // sub-singleton (`IsTrue true ≡ Top`, `IsTrue false ≡ Bottom`) —
    // established spelling, matching `catalog/packages/lawful-classes/
    // lawful_classes.ken`'s `fn IsTrue (b : Bool) : Prop = Equal Bool b
    // True` verbatim (not vendored — the one-line definition is restated
    // here since that package isn't part of the auto-loaded core prelude).
    elab.elaborate_decl("fn IsTrue (b : Bool) : Prop = Equal Bool b True")
        .map_err(|e| ElabError::Internal(format!("IsTrue failed: {}", e)))?;

    // Scalar bounds `[0, 0xD7FF] ∪ [0xE000, 0x10FFFF]` spelled in decimal
    // (this grammar has no hex-literal lexing): 55295 / 57344 / 1114111.
    // Closed-interval `leq_int` bounds exclude the surrogate block without
    // needing strict `<` (`18a §5.9.1(2)`); value-level `and_bool`/`or_bool`
    // (not the `Ω`-sort) compose the two disjoint intervals — required, not
    // forbidden (the forbidden form is a raw `∨`/`∃` as `isScalar`'s own
    // Ω-sort, which `IsTrue (<Bool>)` below never is).
    elab.elaborate_decl(
        "fn inRangeBool (c : Int) : Bool = \
         or_bool (and_bool (leq_int 0 c) (leq_int c 55295)) \
                 (and_bool (leq_int 57344 c) (leq_int c 1114111))",
    )
    .map_err(|e| ElabError::Internal(format!("inRangeBool failed: {}", e)))?;

    elab.elaborate_decl("fn isScalar (c : Int) : Prop = IsTrue (inRangeBool c)")
        .map_err(|e| ElabError::Internal(format!("isScalar failed: {}", e)))?;

    elab.elaborate_decl("def Char = { c : Int | isScalar c }")
        .map_err(|e| ElabError::Internal(format!("Char failed: {}", e)))?;
    let char_id = *elab
        .globals
        .get("Char")
        .ok_or_else(|| ElabError::Internal("Char not registered".into()))?;

    // Derived ops over the (erased-to-identity) projection (`18a §5.9.1(3)`).
    elab.elaborate_decl("fn eqChar (a : Char) (b : Char) : Bool = eq_int a b")
        .map_err(|e| ElabError::Internal(format!("eqChar failed: {}", e)))?;
    elab.elaborate_decl("fn leqChar (a : Char) (b : Char) : Bool = leq_int a b")
        .map_err(|e| ElabError::Internal(format!("leqChar failed: {}", e)))?;
    elab.elaborate_decl("fn charToInt (c : Char) : Int = c")
        .map_err(|e| ElabError::Internal(format!("charToInt failed: {}", e)))?;
    // `Int.toChar` — face-(c): `None` on surrogate/out-of-range, `Some` on a
    // valid scalar; the `inRangeBool` check REDUCES (via the pulled-up
    // `leq_int`), so this is not a stuck neutral on rejection (AC-C3).
    elab.elaborate_decl(
        "fn intToChar (n : Int) : Option Char = \
         match (inRangeBool n) { True => Some Char n ; False => None Char }",
    )
    .map_err(|e| ElabError::Internal(format!("intToChar failed: {}", e)))?;

    let eq_char_id = *elab.globals.get("eqChar").unwrap();
    elab.numeric_env
        .set_eq_entry(char_id, EqEntry { op_id: eq_char_id });
    elab.numeric_env.char_id = char_id;

    Ok(DecimalCharEnv {
        decimal_id,
        mkdecimalpair_id,
        char_id,
    })
}
