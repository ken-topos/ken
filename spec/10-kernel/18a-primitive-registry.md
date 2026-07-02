# The primitive-operation registry (BUILTINS Phase 1 audit)

> Status: **Phase 1 — registry audit** (WS-K / BUILTINS). This is the concrete
> enumeration behind `18 §5` clause (2): every **native** (`declare_primitive`,
> `14 §5`) operation, re-adjudicated **adversarially** against the surface
> chapters (`35`/`37`/`38`). Scope is the **non-effectful** primitive layer;
> effects (`send`/`recv`/`read_bytes`/…), security, and eval-order are **out of
> scope** (noted, not re-adjudicated). Each native op is ratified **conditional
> on its correctness-AC** — Pat ratifies "`X` NATIVE **iff** [AC]", never
> as-currently-built where the build diverges from the seal.

The trusted base (`18 §5`) rests on the primitive reductions being **correct
partial functions on literals**. This registry makes that surface auditable:
what is native, why it *earns* native status, what class laws it forecloses, and
the single external net (the differential oracle) that checks it.

## 1. Schema

One row per operation:

| column | content |
|---|---|
| **symbol** | the registered reduction symbol |
| **signature** | argument types → result type |
| **current-state** | `BUILT` (registered today) · `GAP` (spec-expected, not built) · `LEGACY` (dead-but-live / to remove) |
| **reduction-semantics** | what it computes, **+ the partiality face** (§2) for any op that is not total |
| **oracle-ref** | the **live** differential net (§3); `N/A` for non-native verdicts |
| **burden-of-proof** | the adversarial case: the in-Ken derived alternative + why native is necessary (underivable / a real perf cliff), or why it demotes |
| **provability-consequence** | which class laws (`Num`/`Ord`/`Integral`/…) become **postulate-only (audited-delta)** because the op is opaque, vs the derived counterfactual that proves them **zero-delta** — the trade |
| **verdict** | `NATIVE` · `DEMOTE→derived` · `RETIRE` (adjudicated by the Architect; oracle+burden by the conformance-validator; the two gates are a **conjunction** — neither alone admits a NATIVE op to the TCB) |

**The `current-state × verdict` pair is the TCB delta** Pat reads:

- `BUILT × NATIVE` — stays trusted (ratified, conditional on its AC).
- `BUILT × DEMOTE→derived` — **removed from the TCB** (a trusted op becomes
  checked-Ken stdlib; also gains zero-delta provable laws).
- `GAP × NATIVE` — a **new trusted addition** (spec-mandated, currently
  missing).
- `LEGACY × RETIRE` — a latent-hazard reduction deleted.

## 2. The partiality discipline (`(C)` — a hard soundness AC)

A **partial** primitive — one whose result is undefined on part of its signature
domain (fixed-width overflow, `div`/`mod` by zero, a narrowing conversion, a
Unicode-invalid decode) — **must never reduce an out-of-domain input to a silent
value or to UB.** That is the same soundness shape as a silent wrap: *a wrong
value the kernel trusts.* The discipline is **face-independent** — a partial op
satisfies it in exactly **one** of three sound ways, and its row **names
which**:

1. **static** — the domain proof lives in the **signature**:
   `div : Int → (y : Int) → NonZero y → Int` (the `{ d | d ≠ 0 }` refinement,
   `34 §5`). An out-of-domain application is **rejected at elaboration** (no
   `NonZero 0` inhabitant).
2. **runtime** — a total-looking signature that **emits an obligation**:
   `div : Int → Int → Int` where `div x 0` yields an unsatisfiable `y ≠ 0`
   obligation (`22 §2.4`) that **degrades to panic / `Unknown`** at the
   unguarded site (`43 §2`), **never a value**. (This is the `35 §3.2`
   no-overflow discipline, generalized.)
3. **total-into-`Option`** — the **result type** carries the partiality:
   `Int.toInt32 : Int → Option Int32` (`None` out of range),
   `Float.toDecimal : Float → Option Decimal` (`None` on NaN/∞). The
   out-of-domain case is **representable and observably not-a-value**.

**Face selection (Architect-ruled).** The faces are not equal in soundness
posture. **(c) is the default-preferred** — the partiality lives in the return
type, with **zero reliance on a trusted backstop** (no obligation to discharge,
no degrade path the kernel must trust). Use **(b)** only when the domain
condition is *overwhelmingly static-dischargeable* **and** `Option`-threading
would be prohibitively viral — `div`/`mod` and fixed-width overflow qualify
(divisors and in-range values are usually statically known; an `Option` on every
arithmetic op is unusable). Use **(a)** rarely (when totality-in-the-type is
worth threading a proof at every call). A row lands on (b)/(a) only by **earning
an exception to the (c) default**, so the *trusted backstop* is the quantity
minimized. Reflect-don't-extend: (b) reuses the audited `22 §2.4` obligation
machinery already sealed for overflow rather than adding a parallel path.

A partial row using **none** of the three (a silent value / UB) is **blocked**.
The oracle for a partial row **must feed its domain boundary** (the zero
divisor, the overflow edge, the narrowing edge, the sign edge) — an oracle that
samples only the total interior cannot catch a silent out-of-domain result.

## 3. The differential oracle (the sole external net for a native op)

A native op is **trusted** — reduced in Rust, outside the term language. The one
external check is the **Rosetta differential**: the native reduction vs an
**independent reference**. `18 §5` clause (2) ("a correct partial function on
literals") is exactly what the oracle nets. Two properties are **jointly**
required for a NATIVE verdict (either alone is vacuous):

- **Independent reference (non-circular).** The reference path must compute by a
  route **independent of the op under audit** — a spec-defined algorithm
  (schoolbook bignum) or an independent implementation, **never the
  interpreter's own native reduction.** A "native vs interpreter" differential
  where the interpreter *is* the native path is **green-vs-green** against the
  very bug (the same wrong code on both sides). *Floor ops — `add_int`,
  `eq_int`, `leq_int` — have no lower Ken op to reference, so they ground here.*
- **Boundary operands.** The discriminating inputs must include the domain
  boundary (§2), not just the interior — otherwise a silent out-of-domain result
  is never exercised.

**A defining law beats a reference implementation (oracle selection).** Where an
op has a **defining algebraic law** — the div-mod identity
`a = (a div b)·b + (a mod b)`, a widening round-trip `narrow ∘ widen = id`, a
`neg` involution — **that law is the preferred oracle: it is non-circular by
construction.** A reference *implementation* can alias the native path
(green-vs-green — the same wrong code recomputed); a *law* is checked against
the op's **output algebra**, so it cannot alias the reduction it audits. Fall
back to independent-reference + boundary-operands only where **no defining law
pins the op** — e.g. `mul_int` has no cheap defining law, so it needs the
across-2¹²⁷ independent-bignum reference.

For a `DEMOTE→derived`/`DERIVED` op (checked Ken, **not** trusted) or a `RETIRE`
op (removed), oracle-ref is **`N/A`** — neither is a trusted reduction, so
neither needs the net.

## 4. Findings — the build diverges from the sealed spec (hard correctness-ACs)

The adversarial re-adjudication found the current build **non-reproducing five
sealed commitments** of `35` (`seed-numbers.md` is the net; these ACs restore
compliance).

**Trust-level (Steward-grounded, `@18aeee7`) — these are wrong values in the
tested-not-trusted `ken-interp` evaluator ring; the kernel's trusted checker is
intact.** `Eq` at a primitive type stays **neutral** (`ken-kernel/src/obs.rs`,
"no `primEq` reduction yet"), there is **no primitive `eq → Eq` reflection
lemma** in the elaborator (the only `declare_postulate`s are bytes-round-trip +
capabilities-auth), and **`ken-kernel` does not depend on `ken-interp`** — so a
wrong `eval.rs` value **cannot** inhabit a false kernel proof (no
`refl : Eq Decimal a b` for `a ≠ b`) and nothing transports. The precise,
generalizing defect is two-fold: **the evaluator computes wrong values, AND the
conformance net meant to backstop them is green-vs-green (§3, OF1/OF2) — the
tested-not-trusted posture is *incomplete* for these ops.** High-priority (they
are reachable on `main`), but **not a kernel-soundness emergency**. Severity by
evaluator-ring reachability (Steward-ordered):

**F1 ≈ F4 (live wrong values) > F2 (backstop-missing) > F3 (latent) > F5 (safe
gap).**

- **F1 — `Int` is i128-capped, not arbitrary-precision.**
  `EvalVal::BigInt(i128)` (`ken-interp/src/eval.rs:140`); `exact_int_binop`
  computes raw `op(i128, i128)` (`eval.rs:673-675`) before range-bucketing, so
  `mul_int` of two ~2⁶⁴ values overflows i128 → **debug panics / release
  silently wraps** — a wrong evaluator value (tested-not-trusted ring; kernel
  checker unaffected). Violates `35 §3.1` ("`+`/`-`/`*` total, no obligation",
  sound **only** if `Int` is genuinely unbounded). **AC: genuine
  arbitrary-precision reduction, no i128 ceiling.** (Adding an overflow
  obligation to `Int` instead is *not* spec-compliant — it contradicts §3.1 and
  defeats the unbounded type.)
- **F2 — bare fixed-width silently wraps (runtime face non-compliant).** The
  **static** face is present — `elab.rs:976-989` emits a `NoOvf a b : Ω₀`
  obligation per bare fixed-width op. But the **runtime** face violates
  `35 §3.2`: `eval.rs:682-697` reduce via `iN::wrapping_*` **unconditionally**,
  ignoring discharge, where §3.2 seals *undischarged ⇒ degrade, never wrap*. If
  the static gate is ever bypassed (open-term eval, a discharge-propagation bug,
  an eval-before-prove mode) the wrapping reducer makes a wrong evaluator value
  with no backstop. **AC: bare `add/sub/mul_intN` reduce via CHECKED arithmetic
  (overflow → panic/`Unknown`), NEVER `wrapping_*`; wrapping is reserved to the
  explicit `wrapping_*`/`+%` class alone.** (Under a discharged obligation the
  checked branch is unreachable — a pure backstop.)
- **F3 — legacy i64 `add`/`sub`/`mul`.** `eval.rs:744-746` wrap with no
  obligation and **zero registration** (dead-but-live arms). **AC: RETIRE —
  delete the three arms + a guard-test that `"add"/"sub"/"mul"` are unregistered
  and unreduced (stuck/`Unknown`),** so the hazard cannot silently return.
- **F4 — `Decimal` silently saturates.** `add_decimal` uses `saturating_add/mul`
  + a `.min(18)` shift cap (`eval.rs:606-635`) → a **wrong evaluator value** on
  large magnitude, violating `35 §2.3` exact base-10; **`decimal_eq` also
  saturates** (`eval.rs:624-635`), so two distinct decimals can compare `True`
  in the `ken-interp` surface `==`. This stays in the tested-not-trusted ring —
  `Eq Decimal` is kernel-**neutral** and `eq_decimal : … → Bool` has **no
  `eq → Eq` bridge**, so **no false kernel proof is inhabitable** (the earlier
  "false-proof / `refl`-inhabits" reading was over-classified; grounded above).
  **AC: all Decimal ops (incl. `eq_decimal`/`decimal_eq`) reduce EXACTLY —
  bignum coefficient, no `saturating_*`, no `.min(18)` — or emit an overflow
  obligation on a bounded rep; never silent-saturate.** *This AC is **subsumed**
  by the Decimal DEMOTE (§5.6): a derived `Decimal` has no trusted `eq_decimal`
  at all — equality becomes structural, kernel-re-checked — so the wrong-value
  path is **removed by construction**, not merely fixed.*
- **F5 — `leq_int` registered but unreduced (safe GAP).** `numbers.rs:233`
  registers it; `eval.rs:661-811` has **no comparison-order arm** → stuck on
  literals, `Ord Int` cannot compute. Stuck is *safe* (incomplete, not wrong).
  **AC: add the reduce arm, bignum-correct across the F1 boundary.**

**Structural closing net** (conformance): after F2's degrade-not-wrap and F3's
retirement, `wrapping_*`/`saturating_*` appear in `prim_reduce` **only** inside
the sanctioned `+%`/`wrapping_*` class — their **absence** elsewhere is the
structural guarantee that no bare or legacy op silently wraps (a whole-class
producer-grep, not per-op sampling).

### 4.1 Cross-AC derivability (burden judged against the *fixed* language)

The adversarial burden (§1) is evaluated **"derivable *given the other ratified
ACs*", not "derivable on the build as-is".** A row can look native-necessary on
today's broken build yet be derivable once a sibling AC lands: `Decimal` is
native-necessary *only* because F1's `Int` is capped — F1's bignum fix un-gates
the `(coeff, exp)` derivation, so `Decimal` **DEMOTEs**. This imposes a
**dependency order**: a row whose derivability turns on another AC is
adjudicated *after*, or explicitly conditional on, that AC (the adversarial
default only bites if run to conclusion against the *post-fix* language).

**The F1 dependency root.** Three ratified ACs ride F1 (genuine bignum `Int`):
(i) F1 itself (`Int` totality); (ii) the `Decimal` DEMOTE (the derived `coeff`
is itself capped until `Int` is real bignum); (iii) `Float.toDecimal`
exact-for-finite (needs the derived-exact `Decimal`). F1-first is therefore a
**dependency fact, not a preference** — these cannot land before it (fixing the
Steward's post-ratification tranche order: **F1 → Decimal/`Char` demote → F2/F3
→ F5 → conversions**).

## 5. The registry

Registrars: `ken-elaborator/src/{numbers,bytes,prelude}.rs` (assembled in
`ElabEnv::new`); reductions in `ken-interp/src/eval.rs::prim_reduce`. Kernel
admission: `declare_primitive` (`check.rs`), tag
`PrimReduction = OpaqueType | Op { symbol }` (`env.rs`).

### 5.1 Opaque primitive types

`Int`, `Int8/16/32/64`, `UInt8/16/32/64`, `Float`, `Float32`, `Char`, `Bytes`,
`String` — opaque `Type 0` constants (`14 §5`), inhabited by literals + op
results, in `trusted_base()`. `Bool` is an **inductive** (`34`), *not* a
primitive (its GlobalId is reused, not re-declared). `Decimal` is currently a
primitive type but **demotes** with its ops (§5.6). `Map`/`Set` are abstract
types (`37 §3.3`), consumed through a library interface, no primitive ops.

### 5.2 `Int` arithmetic — floor + bignum cliff

| symbol | signature | current-state | reduction / partiality | oracle-ref | burden | provability | verdict |
|---|---|---|---|---|---|---|---|
| `add_int` | `Int → Int → Int` | BUILT (F1-broken) | bignum add, **total** (AC: real bignum, no i128) | schoolbook-bignum ref, operands **across 2¹²⁷** (indep., not the interp path) | floor — opaque `Int`, no lower op; underivable | `Num Int` `+`-comm/assoc **postulate-only**; on inductive `Nat` **zero-delta** — trade: bignum speed ↔ provable `+` laws | **NATIVE** iff bignum |
| `sub_int` | `Int → Int → Int` | BUILT (F1-broken) | bignum sub, total | as `add_int` | floor | `Num Int` postulate-only; `Nat` zero-delta | **NATIVE** iff bignum |
| `mul_int` | `Int → Int → Int` | BUILT (F1-broken) | bignum mul, total | indep. bignum, across-2¹²⁷ (`2¹⁰⁰ × 2¹⁰⁰`) | repeated-`add` is **O(value) = exponential in bit-length** — a real cliff | `Num Int` mul-assoc/comm/distrib postulate-only; `Nat` zero-delta | **NATIVE** iff bignum |
| `neg_int` | `Int → Int` | GAP | `neg x ≡ sub_int 0 x` | N/A (derived) | **derivable, no cliff** — one-liner over `sub_int` (must type-check vs `35`) | — (derived → zero-delta) | **DEMOTE→derived** |
| `div_int` `mod_int` | `Int → Int → Int` | GAP | **face (b) runtime obligation** (a *justified* (c)-exception — divisors overwhelmingly static-nonzero, `Option`/divide viral; reuses the sealed `22 §2.4` overflow machinery): `div x 0` ⇒ unsatisfiable `NonZeroDivisor y` obligation ⇒ degrade-to-panic/`Unknown`, **never a value** (`div x 0 → 0` is the F1/F4 shape, blocked). **Negative-`mod` PINNED truncated** (`(-7) mod 3 ≡ -1`, machine `%`). `divExact : Int → {y//y≠0} → Int` derives on top | **the div-mod identity `a = (a div b)·b + (a mod b)` IS the oracle** (non-circular law); boundary = zero divisor + negative-dividend (trunc≠floor) | repeated-`sub` = O(quotient) = exp-in-bit-length cliff; opaque `Int` | `Integral Int` div-mod-identity postulate-only; `Nat` zero-delta | **NATIVE** face-(b) iff degrade-not-wrap |

> **`neg_intN` (fixed-width) does NOT demote** — `neg(MIN_intN)` overflows the
> asymmetric two's-complement range → **face (b) obligation class** (`NoOvfNeg`
> obligation), NATIVE. Only bignum `neg_int` (symmetric range, no overflow)
> demotes to the `sub_int 0 x` one-liner.

### 5.3 Fixed-width arithmetic — the four op-classes (`35 §3.2`)

| symbol | signature | current-state | reduction / partiality | oracle-ref | burden | provability | verdict |
|---|---|---|---|---|---|---|---|
| `add_intN` `sub_intN` `mul_intN` (bare, N∈8/16/32/64; signed+unsigned) | `T → T → T` | BUILT (F2-broken) | **obligation class**: emits `φ_no_ovf` (ℤ-domain, `35 §3.2`); **reduce via CHECKED arith — overflow → panic/`Unknown`, never `wrapping_*`** | bare op at overflow boundary, obligation **undischarged** ⇒ degrades **and ≠ wrapped value** | opaque fixed-width type; obligation-class op is the verification differentiator, underivable | fixed-width `Num` laws postulate-only | **NATIVE** (obligation) iff checked-not-wrap |
| `wrapping_add/sub/mul_*` `+%` | `T → T → T` | BUILT | **modular `mod 2ᴺ`, total** (no obligation — sanctioned by `35 §3.2` for hashing/crypto) | modular-boundary operands vs indep. `mod 2ᴺ` ref | explicit modular semantics, opaque type | modular-ring **(ℤ/2ᴺ) laws postulate-only**; zero-delta only on inductive `Fin 2ᴺ` | **NATIVE** (modular) |
| `checked_add/…` | `T → T → Option T` | GAP (spec `35 §3.2`) | **total-into-`Option`** — `None` on overflow | range-edge → `None` (never silent-`Some`) | opaque type | — | **NATIVE** |
| `saturating_add/…` | `T → T → T` | GAP (spec `35 §3.2`) | total by clamp to `T_MIN`/`T_MAX` | range-edge clamp vs indep. ref | opaque type | — | **NATIVE** |
| `add`/`sub`/`mul` (legacy i64) | — | **LEGACY** (F3) | wrapping, no obligation, unregistered | N/A | zero-benefit latent hazard | N/A | **RETIRE** |

> Registration skew (reconcile): only bare `add_*` per width is registered
> today; `sub_intN`/`mul_intN` and most `wrapping_sub/mul_*` have `prim_reduce`
> arms but **no `declare_primitive`** — register the sanctioned ones, delete the
> rest under F3's scan.

### 5.4 Comparison & boolean

| symbol | signature | current-state | reduction / partiality | oracle-ref | burden | provability | verdict |
|---|---|---|---|---|---|---|---|
| `eq_int` | `Int → Int → Bool` | BUILT | bignum `=`, total (AC: bignum, not i128-truncated) | indep. bignum-compare, across-2¹²⁷ | opaque `Int` → no case-split → `DecEq` underivable | `DecEq Int` postulate-only; `Nat` zero-delta | **NATIVE** |
| `leq_int` | `Int → Int → Bool` | **GAP** (F5, registered/unreduced) | bignum `≤`, total | boundary/sign-edge pair flips | opaque `Int` → `Ord` underivable | `Ord Int` postulate-only (audited-delta) | **NATIVE** iff arm added |
| `not_bool` `and_bool` `or_bool` | `Bool[→Bool]→Bool` | BUILT | `Bool` logic, total | truth-table (small, exhaustive) | **`Bool` is inductive** → these **are derivable** by `match` | derived → zero-delta | **DEMOTE→derived** (candidate — `Bool` non-opaque; Architect to rule vs a short-circuit-eval cliff) |
| `eq_float` `eq_float32` | `FloatT → FloatT → Bool` | BUILT | IEEE `==`, total | IEEE `==` incl. NaN (`NaN ≠ NaN`), ±0 | opaque `Float` | **not a proof equality** (`35 §2.4`) — carries no `DecEq`/`Eq` law | **NATIVE** (honest IEEE, non-proof) |

### 5.5 `Float`/`Float32` arithmetic

| symbol | signature | current-state | reduction / partiality | oracle-ref | burden | provability | verdict |
|---|---|---|---|---|---|---|---|
| `add/sub/mul/div_float` | `Float → Float → Float` | BUILT (`add/div` only registered; `sub/mul` skew) | IEEE-754, **total** (overflow → ±∞, NaN — honest, no obligation, `35 §3.3`) | rounding-sensitive operands vs IEEE r-t-n-e | opaque `Float`; IEEE is the machine contract | no algebraic laws (non-associative FP) — none provable either way | **NATIVE** |
| `*_float32` | `Float32³` | GAP (only `add_float32`/`eq_float32` built) | as `Float`, binary32 | as `Float` | — | — | **NATIVE** (register the missing) |

### 5.6 `Decimal` — DEMOTE→derived (post-F1)

`add/sub/mul/eq_decimal` are `BUILT` (native, F4-broken). **Burden unmet →
DEMOTE.** Once F1 delivers genuine bignum `Int`, exact base-10 `Decimal` is
`(coeff : Int, exp : Int)` with arithmetic **derived in-Ken**: `add` =
align-exponents (`coeff × 10^Δexp`, bignum `mul`) then bignum `add`; `mul` =
bignum-`mul` coeffs + `add` exps; `eq` = normalize + bignum compare. Every op is
**O(bignum-arithmetic) — the same cost as the native reduction** (which only
does coeff arithmetic); **no perf cliff** (unlike `mul_int`, there is no
derived- Decimal blow-up). So verdict **`BUILT × DEMOTE→derived`** — a **TCB
removal**, and the *better* soundness posture: (1) `Num Decimal` laws become
**zero-delta provable** over `(coeff, exp)` instead of postulate-only; (2) the
F4 false-`Eq`-proof hole **vanishes** — structural kernel-re-checked `Eq`, no
trusted `eq_decimal`. Gated on F1's bignum `Int`; oracle **N/A** (derived).

### 5.7 Conversions (`35 §5` — closed named set, no implicit coercion)

All `GAP` (none built). Between opaque primitive types there is no shared
structure to recurse on → each is **NATIVE**. Faces per §2.

| symbol | signature | face | current-state | oracle boundary | verdict |
|---|---|---|---|---|---|
| `Int64.toInt` `Int32.toInt64` | `T → wider` | total (widening) | GAP | round-trip on `T_MAX` = identity | **NATIVE** |
| `Int.toInt64` `Int64.toInt32` | `T → Option narrower` | **Option** | GAP | just-above-`MAX` ⇒ `None`, **never silent `Some`** | **NATIVE** |
| `Int.toFloat` `Decimal.toFloat` | `T → Float` | total, **documented-lossy** | GAP | rounding-sensitive value = **defined IEEE r-t-n-e** (not arbitrary); row states "lossy" | **NATIVE** |
| `Float.toDecimal` | `Float → Option Decimal` | **Option** | GAP | `NaN`/`∞` ⇒ `None`; finite ⇒ `Some exact` | **NATIVE** |

Round-trip / conversion laws are postulate-only (opaque→opaque). **`toFloat`
ACs:** the rounding is **round-to-nearest-even** (IEEE default, pinned in the
reduction column) and the provability column **claims no exact round-trip**
(`toInt ∘ toFloat ≠ id`) — lossy-per-*defined*-semantics is correct under its
own semantics, unlike F1/F4 which are wrong under theirs. **`Float.toDecimal`
exact-for-finite** (every finite f64 is a terminating decimal) **depends on the
derived-exact `Decimal`**, so the chain **F1 bignum `Int` → derived-exact
`Decimal` → `Float.toDecimal` exact** rides F1 (a third cross-AC dependency,
§4.1). `Int + Int64` without an explicit conversion is a **type error**
(`35 §5`, no implicit coercion arm).

### 5.8 `String`/`Char` and `Bytes` (`37 §2.4`, `38 §1.2`) — tranche PENDING

> **This tranche + basic data structures are not yet adjudicated** — grounded
> here, but the **cross-AC derivability pass (§4.1) must run first**, then
> Architect native-vs-derived + CV oracle/face. **`Char` demote-candidate:** if
> `Char` is representable as `(codepoint : Int)` (a bignum-`Int`-backed scalar),
> its comparison/ordering ops **DEMOTE→derived** over the codepoint `Int` — the
> same un-gating as `Decimal` (do not reflexively file `Char` ops native). The
> `invalid-Char`/surrogate boundary is a face-(c)-vs-(b) fork.

Primitive registered reductions (compute over literals, neutral on stuck args):
`byteLength`/`charLength`/`++`/slice/index on `String`; `length`/`at`/`slice`/
`concat`/`empty` on `Bytes`; `bytes_encode`/`bytes_decode` at the text boundary.
`Char` is a **refined** carrier (Unicode scalar, surrogates excluded,
`35 §2.4`). The `String`/`Bytes` ops are `BUILT` and the partial ones are
already face-compliant (**total-into-neutral/`Option`**):

- `bytes_at`/`bytes_slice` — out-of-range ⇒ **neutral** (no silent OOB read),
  `Option` at the surface (`38 §1.2`); **compliant**.
- `bytes_decode`/`decode` — invalid UTF-8 ⇒ **neutral** / `Result … DecodeError`
  at the surface; **compliant** (the round-trip `decode ∘ encode ≡ Ok` is a
  provable law, `38 §1.5`).
- `string_to_list_char`/`list_char_to_string` — typed total, currently **stuck**
  (`eval.rs:805-806`, no arm) → `GAP` (add arms; safe-stuck today).

Non-definitional `String`/`Bytes` laws (`byteLength (s ++ t) ≡ …`,
`length (concat a b) ≡ …`) are **prelude propositions** (derivable), not kernel
reductions — they add nothing to `trusted_base()`.

## 6. Out of scope (noted, not re-adjudicated)

- **Effects** — `read_bytes`/`write_bytes`/`append`/`send`/`recv` are
  effect-tracked surface ops (`Vis` nodes, `36 §2`), **not** primitive
  reductions (they reduce to neutral, add nothing to the TCB). **X-series owns
  them.**
- **FFI** — a `foreign` decl is a `declare_postulate` (opaque, in
  `trusted_base()`), not a primitive reduction (`38 §2.3`); the postulate axis
  of `18 §5`, adjudicated there, not here.
- **Security, eval-order** — closed elsewhere.

## 7. Deliverable → Pat

The ratified native set, each op **conditional on its correctness-AC**, plus the
five findings framed **precisely**: **wrong runtime values in the
tested-not-trusted `ken-interp` ring with the conformance net incomplete
(green-vs-green) — the kernel checker is intact, no false proof transports**
(§4, 4-way source-verified). The five fixes are the **first, prioritized Phase-2
tranche** post-ratification, ordered by the F1 dependency root (§4.1): **F1 →
`Decimal`/`Char` demote → F2 + F3 → F5 → conversions**, each gated on the
independent-reference + boundary-operands oracle (§3). No drop-everything hotfix
(kernel intact); pulling F1+F3 into a pre-ratification correctness patch is the
Steward's call. The **TCB delta** Pat ratifies: `Decimal` + `neg_int` +
(candidate) `Char`-ordering + the `Bool` logic ops **leave** the trusted base
(DEMOTE — gaining **zero-delta-provable** laws in place of postulate-only);
`div`/`mod` + the conversion set + `checked`/`saturating` **enter** it (GAP →
NATIVE, spec-mandated); the legacy wrapping path is **deleted** (RETIRE); every
surviving native arithmetic op is ratified **iff** it reduces correctly (bignum
/ checked-not-wrap / exact).
