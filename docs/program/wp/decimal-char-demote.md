# WP: Decimal / Char DEMOTE→derived — the second Phase-2 tranche

- **WP id:** decimal-char-demote
- **Branch:** `wp/decimal-char-demote` (off `origin/main` `3e30e4c`, clean FF)
- **Team:** Runtime (interp + elaborator + prelude), then the enclave gates.
- **Priority:** Phase-2 tranche #2 (rides F1, now landed `bb40654` / PR #213).
- **Cites:** ADR 0009 (capability supply — the *adversarial-burden migration*,
  §5 rubric + Consequences "native/trusted is the exception that must be
  earned"); `18a §5.6` (Decimal), `18a §5.9` (Char), `18a §4.1` (the F1
  cross-AC dependency); PRINCIPLES §5 (small TCB), §8 (honesty about the
  boundary), §12 (typed trust choice).

## Objective

Remove `Decimal` and `Char` from `trusted_base()` at the **type level** (each a
TCB removal, `18a §5.1`), replacing the opaque primitives with **derived Ken
definitions over F1's landed bignum `Int`**:

- **`Decimal` → `(coeff : Int, exp : Int)`** (`18a §5.6`) — exact base-10 with
  arithmetic derived in-Ken (align-exponents + bignum ops), retiring the native
  `add/sub/mul/eq_decimal` primitives (the **F4** saturating-and-false-`Eq`
  bugs — `mul_decimal` uses `saturating_mul`, `decimal_eq` saturates so two
  distinct decimals can compare `True`; `eval.rs` on main).
- **`Char` → `{ c : Int | isScalar c }`** (`18a §5.9`) — the Unicode-scalar
  refinement over `Int`, with derived equality/conversions and **two
  load-bearing soundness pins**.

The demote is not a bug-patch: it makes the soundness posture **strictly
better** — `Num Decimal` / `DecEq`/`Ord Char` laws become **zero-delta
provable** over the derived reps instead of postulate-only, and the F4
false-`Eq`-proof hole **vanishes by construction** (no trusted `eq_decimal`
exists to be wrong).

## Why now (dependency fact, not preference)

`18a §4.1`: `Decimal`'s derived `(coeff, exp)` is native-necessary **only**
while `Int` is i128-capped; F1's genuine bignum `Int` (landed `bb40654`)
un-gates the derivation. This WP is the direct consumer of the F1 keystone —
the first payoff of "the dependency root is in." Tranche order (`18a §4.1`,
Steward-ratified): **F1 → Decimal/`Char` demote → F2/F3 → F5 → conversions.**

## Scope — precisely these two type-level demotes

**IN:**
- `Decimal` type demote + derived `add/sub/mul/eq` over `(coeff:Int, exp:Int)`;
  removal of the `add_decimal`/`sub_decimal`/`mul_decimal`/`eq_decimal`
  primitives and the `Decimal` primitive **type** registration.
- `Char` type demote to the refinement + derived `eq_char` (⇒ `eq_int ∘ proj`),
  `Char.toInt` (= `proj`), `Int.toChar` (= refinement-intro, `Option`,
  face-(c)); removal of the `Char` primitive **type** registration.

**OUT (explicit — do not scope-creep; each rides a later tranche):**
- **`leq_char` / `Ord Char` completeness.** The derived `leq_char ⇒ leq_int ∘
  proj`, but `leq_int` is **F5** (registered-but-unreduced, stuck-*safe*;
  `18a §5.9`/`§5.2`). There is **no** currently-computing native `leq_char` on
  main (grep `eval.rs::prim_reduce`: no `leq_char`/`eq_char` arm exists) — so
  routing `leq_char` through `leq_int` is **net-new, not a regression**. Deliver
  the derived *definition* now; its runtime completeness lands with **F5**.
  `DecEq Char` (via `eq_int`, built) **is** fully unblocked now — deliver it.
- **`checked_*` / `saturating_*` / `neg_int` demotes.** These gate on the
  *complete* `IntN↔Int` conversion floor (`18a §5.3`/`§5.7`) → the
  **conversions** tranche, not this one.
- **`Float.toDecimal` / `Decimal.toFloat`.** Conversions tranche (`§5.7`);
  `Decimal.toFloat` stays NATIVE (correct-rounding cliff).

## Current state (grounded on `origin/main`, for the elaborator/builder)

- **Decimal — native ops exist, must be REMOVED.** `EvalVal::DecimalVal {
  coeff: i64, exp: i32 }` (`eval.rs`); `mul_decimal` = `ca.saturating_mul(cb)`
  (F4); `decimal_eq` saturates (F4). Registered in `numbers.rs`
  (`reg_ty!("Decimal")`, `reg_binop!` for add/sub/mul, `reg_cmpop!` for eq;
  `add_table`/`eq_table` entries). The demote **deletes** all of these.
- **Char — type-only, NO computing ops.** `reg_ty!("Char")` registers the
  opaque type (`numbers.rs`); `eval.rs::prim_reduce` has **no** `eq_char` /
  `leq_char` / `Char.toInt` / `Int.toChar` arm (`string_to_list_char` is a
  `Neutral` stub). So the Char demote is **type-conversion + net-new derived
  ops**, not the removal of working native ops.

## Acceptance criteria (each a hard build-AC; discriminating, not rubber-stamp)

The three soundness contours the Architect pre-committed to gating (offered at
closeout of the prior tranche) are baked in as **AC-G**, **AC-D2**, **AC-C3**:

**AC-G — `trusted_base()` shrinks for REAL, by removal not shadowing.**
Producer-grep proof: the `declare_primitive`/`reg_ty!`/`reg_binop!`/`reg_cmpop!`
calls for the `Decimal` type + `add/sub/mul/eq_decimal` and the `Char` type are
**gone** from `numbers.rs`/`prelude.rs`; the derived defs **reuse F1's landed
exact-`Int` arithmetic**, introduce **no new kernel flag / `Decl` variant**. A
demote that leaves a primitive registered *and* adds a derived def **grows**
surface — that is the failure. (Sibling of the abstraction/visibility
"reuse the existing constant, never a new flag" gate.)

**AC-D1 — Decimal arithmetic is exact (F4 killed).** Derived `add` =
align-exponents (`coeff × 10^Δexp` via bignum `mul`) then bignum `add`; `mul` =
bignum-`mul` coeffs + `add` exps; `eq` = normalize + bignum compare. **No
`saturating_*`, no `.min(18)`, no i64 coeff** — the coeff is F1 bignum `Int`.

**AC-D2 — the discriminating Decimal test FAILS against a saturating/lossy
stub.** Drive the **real derived** arithmetic on a case where the *old*
saturating `mul_decimal`/`decimal_eq` gave a **wrong** value and the *new*
exact-Int-derived one is correct (e.g. a coefficient product that overflows i64
today → saturates → false `eq`). Green-vs-green dies only if that case flips.
Do **not** hand-feed the derived `Decimal` value and test a downstream consumer
(the hand-feeds-the-deliverable trap).

**AC-D3 — `Num Decimal` / `DecEq Decimal` laws are zero-delta.** Equality is
**structural, kernel-re-checked** over `(coeff, exp)` — no trusted `eq_decimal`
in `trusted_base()`; the F4 false-`Eq` path is removed *by construction*, not
patched. Pin at least one law (`+`-comm or the normalize-then-compare `eq`
reflexivity) that was postulate-only pre-demote and is now proved zero-delta.

**AC-C1 — `Char` is the refinement `{ c : Int | isScalar c }`**, with `isScalar`
pinned to the **Bool-decidable reflection** (soundness pin 1, below), not a
naive disjunction.

**AC-C2 — derived Char ops over the projection.** `eq_char ⇒ eq_int ∘ proj`
(`DecEq Char` zero-delta); `Char.toInt = proj`; `Int.toChar : Int → Option Char`
= refinement-intro with the decidable check (**face-(c)**: `None` on
surrogate/out-of-range, never a silent `Some`).

**AC-C3 — the discriminating Char test: a surrogate/out-of-range `Int` is
REJECTED.** `Int.toChar 0xD800` (surrogate) and `Int.toChar 0x110000`
(out-of-range) reduce to `None`; a valid scalar reduces to `Some`. This must
**fail against a stub `isScalar := true`** (the predicate-definedness dual) —
i.e. the refinement obligation **actually reduces**, not name-matches.

### Two load-bearing Char soundness pins (hard, `18a §5.9`)

1. **`isScalar : Int → Ω` via the Bool-decidable reflection — pin the
   *encoding*, not just the sort.** `isScalar c := IsTrue (inRangeBool c)`, with
   `inRangeBool c : Bool = (0 ≤? c && c ≤? 0xD7FF) || (0xE000 ≤? c && c ≤?
   0x10FFFF)`. `IsTrue b` is a **genuine sub-singleton → proof-irrelevant → Ω**.
   A naive `(…) ∨ (…) : Ω` is the **forbidden direction** (a raw disjunction is
   the sum `A + B`, a two-constructor **proof-relevant** type that cannot sit at
   Ω directly — `16 §1.3`, the `Bool → Ω` trap; range-disjointness does *not*
   rescue it, the injection tag stays). Ω-admissible only via `IsTrue` (pinned)
   or explicit `‖A + B‖`. **Payoff (load-bearing):** Ω-PI makes `Char` equality
   reduce to **codepoint** equality (same codepoint, distinct scalar proofs →
   equal by Ω-PI → zero-delta `DecEq Char`) — this holds **only** if `isScalar`
   is *actually* proof-irrelevant, which the naive `∨` is not.
2. **String→`Char` extraction emits the canonical scalar proof.** `char_at` /
   `string_to_list_char` construct `(c, canonical_proof)`; sound because a
   valid-UTF-8 `String` only yields scalars, so `isScalar c` reduces to its
   canonical inhabitant. **No primitive may fabricate a non-scalar `Char`.**

## Guardrails

- **No kernel touch.** `ken-kernel` unchanged; no new `Decl` variant, no new
  `declare_primitive` flag. Verify: `git diff --stat crates/ken-kernel/` empty.
  The demote is an elaborator/interp/prelude change; the kernel **shrinks** its
  reachable primitive set, never grows.
- **Reuse F1's landed exact-`Int`.** The derived Decimal coeff arithmetic and
  the Char projection compare route through F1's `num_bigint`-backed reduction —
  no second bignum path, no re-introduced i64/i128 intermediate.
- **`leq_char`/`Ord Char` completeness is F5's, not this WP's** (deliver the
  derived def; runtime completeness rides F5). Do **not** pull the `leq_int`
  reduce arm into this WP.
- **Workspace-green landing (K7 lesson).** `cargo test --workspace` green — not
  a scoped `-p` subset; no golden/`.ken` fixture may ride the old saturating
  `Decimal` behavior (migrate any that assert the F4 wrong value).
- **Honesty about the boundary (§8).** State plainly which faces land now vs
  ride F5 (Ord Char) — no over-claim of a complete `Ord Char` this tranche.

## Conformance / oracle approach

- **Decimal:** oracle is **N/A-as-differential** (derived, not a trusted native
  op) — the net is the **discriminating AC-D2** (old-saturating-wrong vs
  new-exact-correct flip) + the zero-delta law pins (AC-D3). The structural
  closing net (`18a §4`): a whole-class `prim_reduce` producer-grep confirms
  **no `saturating_*` remains anywhere** and the `*_decimal` arms are gone.
- **Char:** the net is **AC-C3** (surrogate/out-of-range rejection, fails vs
  `isScalar := true`) + the **Ω-encoding pin** verification (the `DecEq Char`
  codepoint-collapse holds *because* `isScalar` is `IsTrue`-reflected, provably
  proof-irrelevant — not a naive `∨`).

## Handoff

Steward frames (this doc). **NEXT:** hand to **spec-leader** for elaboration —
spec-author authors the `§5.2.1`-style delivery-contract (the derived Decimal
arithmetic + Char refinement obligations); conformance-validator authors the
`/conformance` seed (AC-D2 flip vectors + AC-C3 surrogate-rejection + the
Ω-encoding pin as the `DecEq Char` soundness check). Then Team Runtime executes;
gates = **Architect soundness** (AC-G removal-not-shadowing + the Ω-encoding pin
+ no-kernel-touch) + **CV conformance** (AC-D2/D3/C2/C3 discriminating
coverage).

**Dormant carry (not this WP):** Architect's F1 `bigint_from_rt` re-validation
gates on a future K3 store slot-reader — unrelated to this tranche.
