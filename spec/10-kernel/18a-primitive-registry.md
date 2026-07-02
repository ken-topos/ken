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
  checked-Ken stdlib; the class laws gain provability — **zero-delta** on an
  inductive carrier, **zero-NEW-delta** over an opaque floor like `Int` — but a
  **non-canonical** DEMOTE carrier can leave an `Equal`-tying class law
  **undeliverable**, e.g. `DecEq Decimal`, §5.6.1(4)).
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
  **AC: add the reduce arm, bignum-correct across the F1 boundary — delivered by
  the `Decimal`/`Char` demote tranche (§5.2.2) as a derived-def prerequisite,
  per the (A) ruling, not a later successor.**

**Structural closing net** (conformance, P1-precise): a whole-class
producer-grep of `prim_reduce`, not per-op sampling. Since
`checked`/`saturating` DEMOTE (§5.3, F-new), the sanctioned classes reduce to
one — **`wrapping_*` may appear only inside the explicit `+%` class, and
`saturating_*` may appear NOWHERE at all.** Any `wrapping_*` outside `+%` (a
bare op, the legacy path) or **any** `saturating_*` (the F4 `add_decimal` path)
is the violation; their absence elsewhere is the structural guarantee that no
bare/legacy/`Decimal` op silently wraps or saturates.

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

**Ordering-reduction placement (corrected).** The Bool-valued `Int` ordering
reduction (`leq_int`'s `prim_reduce` arm, §5.2.2) is a **prerequisite of the
Decimal/`Char` demote**, not a later successor: the derived `Decimal` exponent
alignment (§5.6.1) and the `Char` scalar-range check (§5.9.1) are inherently
ordering tests, so the arm lands **with** that tranche. Scheduling it later
rested on the premise that `Decimal` alignment needs only `mul`; the landed
`ea<eb` alignment branches disprove it. Pulling the arm up **resolves finding
F5** (§4, `leq_int` unreduced) within this tranche; the separate **conversions**
tranche (`checked_*`/`saturating_*`/`neg_int`, the `IntN↔Int` floor) is
unaffected — none of it the comparison arm needs.

## 5. The registry

Registrars: `ken-elaborator/src/{numbers,bytes,prelude}.rs` (assembled in
`ElabEnv::new`); reductions in `ken-interp/src/eval.rs::prim_reduce`. Kernel
admission: `declare_primitive` (`check.rs`), tag
`PrimReduction = OpaqueType | Op { symbol }` (`env.rs`).

### 5.1 Opaque primitive types

`Int`, `Int8/16/32/64`, `UInt8/16/32/64`, `Float`, `Float32`, `Bytes`, `String`
— opaque `Type 0` constants (`14 §5`), inhabited by literals + op results, in
`trusted_base()`. `Bool` is an **inductive** (`34`), *not* a primitive (its
GlobalId is reused, not re-declared). **`Decimal` and `Char` currently register
as primitive types but DEMOTE at the type level** (each a TCB removal):
`Decimal` → `(coeff : Int, exp : Int)` (§5.6), `Char` → the refinement
`{ c : Int | isScalar c }` (§5.9). `Map`/`Set` are abstract types (`37 §3.3`),
consumed through a library interface, no primitive ops (§5.10).

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

### 5.2.1 F1 delivery contract — the "iff bignum" half

The §5.2 verdicts read **NATIVE iff bignum**: `add_int` / `sub_int` / `mul_int`
/ `eq_int` / `leq_int` are trusted native reductions **only once** their
reduction is genuine arbitrary-precision — a fixed-width intermediate that wraps
or panics is the F1 wrong value, and a wrong value forecloses the eventual K3
promotion (a reduction that can produce a wrong value cannot be promoted to
kernel-executed). **F1's reducing scope is the built floor ops** — `add_int` /
`sub_int` / `mul_int` (arithmetic) and `eq_int` (comparison, reduced today).
**`leq_int` is out of F1's reducing scope**: F1 delivers only the
arbitrary-precision *representation* `leq_int` compares over — never its
reduction. Its `prim_reduce` arm is delivered by the **`Decimal`/`Char` demote
tranche** (§5.2.2), as a *prerequisite* of the derived `Decimal` exponent
alignment (§5.6.1) and the `Char` scalar-range check (§5.9.1) — **not** a later
successor tranche (finding F5's earlier scheduling rested on a premise the
landed `ea<eb` alignment branches disproved; §4.1, §5.2.2). WP F1
(`docs/program/wp/F1-bignum-int.md`) delivers the representation half; the §5.4
row records the arm. This subsection is the
**normative contract** the delivery satisfies; it fixes *what the reduction must
guarantee*, not the Rust that guarantees it (the interpreter line anchors are
perishable build detail, carried in the WP brief).

**(1) Totality — no fixed-width intermediate on the arithmetic path.** For
`add_int` / `sub_int` / `mul_int` the reduction computes the **exact**
mathematical integer for every operand pair, with **no** `i64` / `i128` (or any
fixed-width) value anywhere on the arithmetic path — not as an accumulator, not
as an intermediate, not as a fast-path result that later overflows. A
small-integer fast path is permitted **only** as a representation optimisation
that widens to the arbitrary-precision type *before* any operation that could
exceed its range, so it is never the path that wraps. **`eq_int` (in F1)**
compares over the **true** integers, never over truncated fixed-width images:
two distinct integers that share a fixed-width residue must **not** compare
equal.
**`leq_int` inherits this same arbitrary-precision representation; its reduce
arm is delivered by the `Decimal`/`Char` demote tranche (§5.2.2)** — F1
guarantees only that the comparison, once its arm lands, is over true integers
(never fixed-width residues), *not* that F1 reduces `leq_int` at all. This is
the `18 §5` clause-(2) "correct partial function on
literals" made total for the F1 floor ops (`add_int` / `sub_int` / `mul_int` /
`eq_int`).

**(2) The reduction interface is FROZEN.** The symbol-keyed primitive
registrations — the `add_int` / `sub_int` / `mul_int` / `eq_int` / `leq_int`
symbols, their arity, and their `PrimReduction::Op` entries in the elaborator's
number-primitive registry — are **unchanged** by F1. F1 replaces the
*representation and the arithmetic behind* the symbols, never the surface a term
elaborates against: a term that reduced through these symbols before F1 reduces
through the identical symbols after, and only the value it reaches changes (from
a wrapped/panicking one to the exact one). Renaming, re-arity-ing, or
re-registering any of them is out of scope and a break.

**(3) Store round-trip — a contract F1 ESTABLISHES, not merely preserves.** The
run-time store already carries the arbitrary-precision representation:
`Value::BigInt { sign, limbs }` — sign-magnitude, little-endian `limbs`, under
the `minimal_limbs` canonical invariant (no trailing zero limbs; a single zero
limb for `0`), content-addressed. The interpreter's evaluator value does **not**
currently populate it — there is no big-integer arm on the eval→store
conversion today, so a value beyond the fixed-width ceiling cannot intern at all
— so this is a bijection F1 **establishes**, not one it preserves. F1 fixes the
lossless conversion at the store boundary in **both** directions: every
evaluator arbitrary-precision integer converts to
`Value::BigInt { sign, limbs }` and back **byte-identically**, the
`minimal_limbs` invariant is preserved by
construction (a freshly reduced value is canonicalised, never emitted with a
non-minimal limb vector), and content-addressing (`canonical.rs`) stays stable —
equal integers intern to the same content hash regardless of the arithmetic
route that produced them. This is a **testable obligation**: round-trip
byte-identity + minimal-limb canonicity + cross-boundary hash stability.

**(4) Crate-vetting — the ADR 0009 rubric-step-1 gate, made concrete.** F1 is
the first Phase-2 dogfood of ADR 0009's **curate-before-construct** rubric: the
arbitrary-precision arithmetic is **sourced from a battle-tested external crate,
not constructed in-tree** (in-tree / proved construction is the deferred K3 —
ADR 0009 tier-c — question, not this WP). The concrete vetting gate the curated
crate must clear — the `63` re-check discipline applied to a tool-chain build
dependency:

- **Pure safe Rust.** No `unsafe` on the arithmetic path — either the crate is
  `#![forbid(unsafe_code)]`, or every `unsafe` block is audited and the audit is
  recorded. (Default candidate `num-bigint`; fall back to a forbid-unsafe
  equivalent — `ibig` / `dashu` — if the default carries un-audited `unsafe`.)
- **Permissive, non-copyleft licence.** MIT / Apache-2.0 / BSD class —
  clean-room-compatible; **no** GPL / AGPL / CeCILL (a copyleft dependency is a
  clean-room violation, not merely a licence preference).
- **Actively maintained, widely adopted.** The "earned industry trust" that is
  the ADR 0009 selection criterion (rubric step 1) — a maintained,
  broadly-depended-on crate, not an abandoned or niche one.
- **Vendored + version-pinned + dependency-delta recorded.** The exact version
  is pinned, the crate (and its transitive additions) vendored, and the
  dependency delta documented per `63` + ADR 0009 — licence, `unsafe`-status,
  version, and transitive-crate additions all recorded, so this addition to the
  tool-chain's own trusted computing base is legible and re-checkable on update.

**(5) Trust level — tier-b tested-not-trusted, NOT a `trusted_base()` line.**
The curation move (rubric step 1) sources the crate, but the resulting trust
posture is **tier-b (tested-not-trusted)**, not tier-a: F1 adds **nothing** to
`trusted_base()` and touches **no** `ken-kernel` file. The whole `prim_reduce`
path is the interpreter's outer, tested-not-trusted ring — structurally gated
out of every proof-relevant position (the kernel keeps `Eq` at a primitive type
**neutral**; there is no `eq → Eq` reflection bridge and no evaluator dependency
in the kernel), so a curated-crate bug is a **wrong value, never a false
proof**. The single external net for these floor reductions is the **§3
differential oracle** against an *independent* arbitrary-precision reference —
never the production crate on both sides (that is green-vs-green against the
very bug). The **tier-c proved-native promotion** (K3, kernel-executed
reductions) is a **separate, later** WP that re-decides the *trusted*
representation behind this same frozen interface; F1 neither performs nor
presumes it.

**Guardrails (do-not-reopen, spec-level).** Interp-local; no `trusted_base()`
promotion (`declare_primitive` / `declare_postulate` untouched); no kernel file
changed (the neutral-`Eq`-at-primitive reduction stays neutral); the §2
partiality discipline and the numeric-tower surface (`§5.6`–`§5.9`) unchanged. A
kernel-touch is a **scope error → STOP and escalate**, not an F1 task. F1 is a
reduction-**value** change, so its no-regression gate is **workspace-green**
(the `18a` / K7 lesson): golden vectors and `.ken` artifacts riding the old
fixed-width behaviour migrate in the *same* green landing unit — never a
crate-only diff.

### 5.2.2 `Int` ordering (`leq_int`) — `Decimal`/`Char` prerequisite

The `Decimal`/`Char` demote (§5.6/§5.9) derives its arithmetic and its
refinement over F1's bignum `Int`, but two of those derived defs are inherently
**ordering** tests, not just `add`/`sub`/`mul`/`eq`: `Decimal` exponent
alignment (§5.6.1) picks the common exponent `min(ea, eb)`, and the `Char`
scalar-range check (§5.9.1) is a `≤`-bounded interval. So this tranche delivers
the `leq_int` `prim_reduce` arm as a **genuine prerequisite** of those defs —
resolving finding F5 (§4) here rather than as a later successor, where it was
scheduled on the premise (§4.1) that `Decimal` alignment needs only `mul` (the
landed `ea<eb` alignment branches disprove it). This subsection is the
**normative contract** that delivery satisfies; it fixes *what the reduction
must guarantee*, not the
Rust that guarantees it (the interpreter line anchors are perishable build
detail, carried in the WP brief).

**(1) Bignum-correct total order.** The reduction computes `a ≤ b` over the
**true** integers for every operand pair — never over fixed-width residues: two
integers whose fixed-width images misorder must still compare by true magnitude
(the boundary cases are mixed sign and operands beyond 2⁶³/2¹²⁷). It reduces to
a `Bool` **value**, total on all `Int × Int`, sharing F1's `Value::BigInt`
representation and the `eq_int` arm's arbitrary-precision discipline
(§5.2.1 (1),(3)).

**(2) Registration FROZEN; `<` DERIVED, not registered.** `leq_int` is
**already registered** (`Int → Int → Bool`, §5.4); this tranche adds **only**
its `prim_reduce` arm — no new symbol, no re-arity, no re-registration. Strict
ordering is **derived at the definition level** — `lt a b := not (leq_int b a)`
(pure `leq`, `¬(b ≤ a)`) — introducing **no** new registered primitive and
**no** new reduce arm (`<` reduces through the built `leq_int` + `not_bool`). A
direct `lt_int` primitive is a soundness-neutral ergonomics option, **out of
scope** here (keeps the primitive set flat).

**(3) Trust level — tier-b tested-not-trusted, zero `trusted_base()` delta.**
The arm is an interpreter `prim_reduce` reduction (`ken-interp/eval.rs`) —
**not** a kernel change and **not** a `declare_primitive`/`declare_postulate`:
it emits a `Bool` value and never touches definitional equality — the
kernel's
neutral-`Eq`-at-primitive and `conv.rs` stay **byte-untouched** (`git diff
--stat ken-kernel/` empty). Same outer, tested-not-trusted ring as `eq_int`,
structurally gated out of every proof-relevant position, so a bug is a **wrong
value, never a false proof**. Because `leq_int` is already registered, adding
the arm is **`trusted_base()`-neutral**.

**(4) Independent-oracle net (OF2, non-circular).** The single external net is
the §3 differential oracle: golden comparison vectors across the sign / 2⁶³ /
2¹²⁷ boundaries with mixed sign, or the order-defining law (`a ≤ b ⟺ ¬(b < a)`;
`a ≤ b ⟺ (a < b) ∨ (a = b)`) — **never** the production `num-bigint` crate's own
`Ord` on both sides (green-vs-green against the very bug). The exact
non-circularity discipline that nets `eq_int` (§5.2.1). This arm is what makes
`Ord Int` reduce (§5.4, previously postulate-only) and what the downstream
`Ord Char` (§5.9.1) and `Decimal` alignment (§5.6.1) reduce over.

### 5.3 Fixed-width arithmetic — the four op-classes (`35 §3.2`)

| symbol | signature | current-state | reduction / partiality | oracle-ref | burden | provability | verdict |
|---|---|---|---|---|---|---|---|
| `add_intN` `sub_intN` `mul_intN` (bare, N∈8/16/32/64; signed+unsigned) | `T → T → T` | BUILT (F2-broken) | **obligation class**: emits `φ_no_ovf` (ℤ-domain, `35 §3.2`); **reduce via CHECKED arith — overflow → panic/`Unknown`, never `wrapping_*`** | bare op at overflow boundary, obligation **undischarged** ⇒ degrades **and ≠ wrapped value** | opaque fixed-width type; obligation-class op is the verification differentiator, underivable | fixed-width `Num` laws postulate-only | **NATIVE** (obligation) iff checked-not-wrap |
| `wrapping_add/sub/mul_*` `+%` | `T → T → T` | BUILT | **modular `mod 2ᴺ`, total** (no obligation — sanctioned by `35 §3.2` for hashing/crypto) | modular-boundary operands vs indep. `mod 2ᴺ` ref | explicit modular semantics, opaque type | modular-ring **(ℤ/2ᴺ) laws postulate-only**; zero-delta only on inductive `Fin 2ᴺ` | **NATIVE** (modular) |
| `checked_add/…` | `T → T → Option T` | GAP | **total-into-`Option`** — `None` on overflow | N/A (derived) | **DERIVABLE** (F-new, both-sides-confirmed): `checked_add_intN a b = Int.toIntN (add_int (IntN.toInt a) (IntN.toInt b))` — the narrowing `Int.toIntN`'s `None` **IS** the overflow semantics; one `add` + two conversions, constant-factor, no cliff | — (derived → zero-delta) | **DEMOTE→derived** (F1 + the complete conversion floor §5.7) |
| `saturating_add/…` | `T → T → T` | GAP | total by clamp | N/A (derived) | **DERIVABLE** (F-new): widen → clamp-compare (`leq_int` vs `T_MIN`/`T_MAX`) → narrow, over bignum `Int` + conversions; constant-factor | — (derived) | **DEMOTE→derived** (F1 + conversions) |
| `add`/`sub`/`mul` (legacy i64) | — | **LEGACY** (F3) | wrapping, no obligation, unregistered | N/A | zero-benefit latent hazard | N/A | **RETIRE** |

> Registration skew (reconcile): only bare `add_*` per width is registered
> today; `sub_intN`/`mul_intN` and most `wrapping_sub/mul_*` have `prim_reduce`
> arms but **no `declare_primitive`** — register the sanctioned ones, delete the
> rest under F3's scan.

### 5.4 Comparison & boolean

| symbol | signature | current-state | reduction / partiality | oracle-ref | burden | provability | verdict |
|---|---|---|---|---|---|---|---|
| `eq_int` | `Int → Int → Bool` | BUILT | bignum `=`, total (AC: bignum, not i128-truncated) | indep. bignum-compare, across-2¹²⁷ | opaque `Int` → no case-split → `DecEq` underivable | `DecEq Int` postulate-only; `Nat` zero-delta | **NATIVE** |
| `leq_int` | `Int → Int → Bool` | **BUILT** (reduce arm delivered by the `Decimal`/`Char` demote, §5.2.2 — derived-def prerequisite; registered `numbers.rs:233`) | bignum `≤`, total | boundary/sign-edge pair flips (indep. oracle, never `num-bigint` `Ord` both-sides) | opaque `Int` → `Ord` underivable | `Ord Int` postulate-only (audited-delta) | **NATIVE** |
| `not_bool` `and_bool` `or_bool` | `Bool[→Bool]→Bool` | BUILT | the `Bool` **eliminator** (`and a b = match a {True⇒b; False⇒False}`, short-circuit inherent — the non-scrutinee arm isn't forced) | N/A (derived) | **`Bool` inductive → the ops ARE the eliminator**; strict-prim vs `match` observationally identical (`Bool` pure), constant-factor, no cliff; a native op must not shadow the eliminator (subsume-don't-proliferate) | derived → zero-delta `Bool`-algebra laws | **DEMOTE→derived** (R1, ruled) |
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
removal**, and the *better* soundness posture: (1) the derived ops are
**structural / zero-delta computational** (no trusted `*_decimal`). The
**`Num`/`DecEq Decimal` law-carrying instances are re-deferred, not delivered
here** — and, on the non-canonical `(coeff, exp)` carrier, are **not
structurally deliverable at all** (§5.6.1(4)): the value-equality `decimalEq` is
an `Eq`, not a `DecEq` for definitional `Equal`, so a `DecEq.sound` `Axiom`
would inhabit `Bottom`. The decide-once carrier call is `90 §OQ-decimal-eq`;
(2) the F4 **wrong-value** `eq_decimal` hazard **vanishes** — the derived,
checked value-equality, no trusted `eq_decimal` (§4 F4: a wrong `Bool` value in
the tested-not-trusted ring, never a false proof). Gated on F1's bignum `Int`;
oracle **N/A** (derived).

### 5.6.1 `Decimal` delivery contract — derived exact base-10 over F1 bignum

This subsection is the **normative contract** the §5.6 DEMOTE satisfies — what
the derived reduction must guarantee (representation-independent); the
interpreter line anchors are perishable build detail in the WP brief.

**(1) Removal-not-shadowing — the TCB shrink is REAL.** The native
`add_decimal`/`sub_decimal`/`mul_decimal`/`eq_decimal` `prim_reduce` arms and
their `reg_binop!`/`reg_cmpop!` registrations, plus the primitive `DecimalVal`
representation path, are **deleted**. `Decimal` becomes the derived pair
`(coeff : Int, exp : Int)` over F1's landed bignum `Int` — **no** surviving
primitive `Decimal` op or type, **no** new kernel flag / `Decl` variant. A
registration that survives *alongside* a derived def **grows** the surface —
that is the failure; the removal, not a shadow, is the mechanism (AC-G).

**(2) Exact arithmetic over bignum — F4 killed by construction.** Every op
computes over F1's exact `Int`: **no** `saturating_*`, **no** `.min(18)` clamp,
**no** i64/fixed-width coeff anywhere on the arithmetic path.

- `mul` = (`mul_int` coeffs, `add_int` exps) — **ordering-free**, reduces
  directly.
- `add`/`sub` = align to the common exponent `min(ea, eb)`; the shift magnitude
  `|ea − eb|` and the direction (which coeff scales) are decided by `leq_int`
  (§5.2.2), the scale is `coeff × 10^|Δ|` via `mul_int`, then
  `add_int`/`sub_int` the aligned coeffs.
- `eq` = align (via `leq_int` + `mul_int`) then `eq_int` the aligned coeffs —
  reduces to a `Bool` value on the bounded-align path, **never** a saturating
  false `True`.

The exact scale `10^|Δ|` is delivered by a **bounded** unroll (a general
`Int`-recursive `pow10` fails size-change termination over opaque `Int`; the
fixed unroll is SCT-clean). Beyond that depth the scale is a **stuck-marker** —
the internal `decimalPow10Unbounded : Int → Int` postulate (function-typed ⇒
soundness-inert; never reduces) — so a large-`|Δexp|` `add`/`sub`/`eq` goes
**stuck: exact-or-stuck, never clamped** (§4 F4 discipline — a missing value,
never a wrong one). The unbounded-`Δexp` completion is a **named forward
obligation**, tracked in the conformance seed's deferred set.

Discriminating closure (AC-D2): a coefficient product that overflowed i64 and
**saturated** under the old `mul_decimal`/`decimal_eq` (a false `True` on
distinct decimals — the F4 hole) now reduces to the exact value / correct
`False`. **Both** halves of F4 close this tranche — the `mul`-saturation *and*
the sharp `eq` false-`True` on distinct decimals — a **wrong `Bool` value** in
the tested-not-trusted ring, **not** a false proof (`Eq Decimal` is
kernel-neutral, no `eq → Eq` bridge; §4 F4) — now that `leq_int` reduces.

**(3) User-facing surface preserved; only the MECHANISM moves.** The `Decimal`
type, its `Num`/`DecEq` instances, and the `+`/`-`/`*`/`=` surface at `Decimal`
are unchanged for terms; only the reduction moves from trusted native primitive
to **derived-over-bignum**. A term reduces to the **same** value after — except
where the old value was F4-wrong, which now reduces to the correct value.

**(4) Value-equality op ships; `DecEq`/`Num Decimal` law instances re-deferred,
NOT structurally deliverable.** The `decimalEq` **op** is **value-equality via
exponent alignment** (align then `eq_int`), reduces to a `Bool`, and removes the
F4 **wrong-value** trusted `eq_decimal` from `trusted_base()` *by construction*
(§4 F4: the hazard was a wrong `Bool` value in the tested-not-trusted ring,
never a false kernel proof) — the computational win, unchanged. But **because**
it is value-equality on the **non-canonical** `(coeff, exp)` carrier (many pairs
per value — `(10, -1)` and `(1, 0)` both denote `1.0`), it is an **`Eq`**
(equivalence), **not** a **`DecEq`** (a decision procedure for definitional
`Equal`, `51 §2.2`): `DecEq.sound : IsTrue(decimalEq x y) → Equal Decimal x y`
is **not a theorem** — `decimalEq (MkDecimalPair 10 (-1)) (MkDecimalPair 1 0)`
reduces `True`, so a postulated `sound` `Axiom` yields `Equal Decimal`, and
`MkDecimalPair` injectivity gives `Equal Int 10 1 → Bottom` (a **false proof**,
not a wrong value). So `DecEq`/`Num Decimal` as specified are **not**
zero-NEW-delta structural proofs — they are **not deliverable at all** on this
carrier (the landedness of `DecEq Int`'s `Axiom` is irrelevant: `DecEq Decimal`
is not a transport of it — `Int` is canonical, `Decimal` is not). The honest
target is `Eq Decimal` (no `Equal`-tie), which itself bottoms on missing `Int`
arithmetic lemmas — so the lawful-`Decimal` deliverable is **re-deferred**
pending the decide-once carrier/quotient call (`90 §OQ-decimal-eq`); `Ord Char`/
`DecEq Char` are **unaffected** (canonical carrier: one value per codepoint).

The derived defs sit in the interpreter's
tested-not-trusted ring over F1's tier-b arithmetic (§5.2.1 (5)) and §5.2.2; the
demote touches **no** kernel file and **net-shrinks** `trusted_base()` — the
four `*_decimal` primitives + the `Decimal` type registration leave against
**one** honest-visible addition, the internal deferred-align postulate
`decimalPow10Unbounded : Int → Int` (§5.6.1(2)'s large-`|Δexp|` stuck-marker;
function-typed ⇒ soundness-inert). A **net shrink (−5** over the combined
`Decimal`/`Char` tranche**)**, **not** a zero-addition — the same
absolute-vs-net precision as the zero-NEW-delta `Char` laws (§5.9.1).

### 5.7 Conversions (`35 §5` — closed named set, no implicit coercion)

All `GAP` (none built). Between opaque primitive types there is no shared
structure to recurse on → each is **NATIVE**. Faces per §2. **★ The COMPLETE
`IntN↔Int` set** (every width `N∈{8,16,32,64}×{signed,unsigned}`) **is the
NATIVE floor** under `checked`/`saturating` (§5.3): those DEMOTE *given* the
full set, so completing it (beyond §5.5's `Int64`/`Int32` representatives) is a
spec-mandated GAP→NATIVE entry, and this floor does **not** itself demote
(Architect-ruled — nothing lower to derive it from).

| symbol | signature | face | current-state | oracle boundary | verdict |
|---|---|---|---|---|---|
| `IntN.toInt` (all N, widening) | `IntN → Int` | total | GAP | `Int.toIntN ∘ IntN.toInt = Some` on `T_MAX` (defining round-trip law) | **NATIVE** (floor) |
| `Int.toIntN` (all N, narrowing) | `Int → Option IntN` | **Option** | GAP | just-above-`MAX` ⇒ `None`, **never silent `Some`** | **NATIVE** (floor) |
| `Int.toFloat` | `Int → Float` | total, **documented-lossy** | GAP | rounding-sensitive value = **defined IEEE r-t-n-e**; opaque `Int`, direct contract | **NATIVE** |
| `Decimal.toFloat` | `Decimal → Float` | total, **documented-lossy** | GAP | **burden (re-run post-Decimal-DEMOTE):** naive `coeff.toFloat *. 10^exp` over derived `Decimal` **double-rounds** (two roundings compound → wrong last bit); **correctly-rounded decimal→binary is a real algorithm cliff** (David-Gay / Ryū-shaped, not a one-liner) → earns **NATIVE** | **NATIVE** (correct-rounding cliff) |
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

### 5.8 `String` and `Bytes` — NATIVE opaque buffers (`37 §2.4`, `38 §1.2`)

`String` (immutable UTF-8, content-addressed, NFC-normalized at construction)
and `Bytes` (immutable byte buffer) are **opaque primitive types**: their ops
act on the buffer with no case-split, and `String` earns native over a derived
`List Char` on a **real cliff** — **O(1) content-addressed equality** (slot-id
vs O(n) structural), **NFC-at-construction**, compact UTF-8 (`mul_int`-shaped,
not convenience). So
`byteLength`/`charLength`/`++`/`slice`/`index`/`encode`/`decode` (String) and
`length`/`at`/`slice`/`concat`/`empty` (Bytes) are **NATIVE**, partiality
already face-(c)-compliant:

- index/slice/`at` out-of-range ⇒ **neutral** (no silent OOB read), `Option` at
  the surface (`38 §1.2`).
- `decode` invalid UTF-8 ⇒ **neutral** / `Result … DecodeError`; round-trip
  `decode ∘ encode ≡ Ok` provable (`38 §1.5`).

**★ Pin: `String` equality is NFC-equality, not byte-equality** — the row must
state it (`"é"`-composed `≡` `"é"`-decomposed), the semantic-pin discipline of
truncated-`mod`. `DecEq String` is **postulate-only** (opaque buffer, like
`DecEq Int`) — but it **is** a real decidable equality (content-addressed ⇒
slot-id compare is structural, *unlike* the non-proof `eq_float`), so it can
back `DecEq String`, just audited-delta. Non-definitional `String`/`Bytes` laws
(`byteLength (s ++ t) ≡ …`) are prelude propositions (derived), adding nothing
to `trusted_base()`.

### 5.9 `Char` — DEMOTE→derived (refinement `{ c : Int | isScalar c }`)

**RULED refinement; TYPE + ops both demote (double TCB removal) — the fork was
forced.** An *opaque* `Char` has no projection to `Int` and no case-split, so
`eq_char`/`leq_char` could not derive (nothing to project) and would be NATIVE
by the *exact* argument that keeps `eq_int` native. The ops demote **iff**
`Char` is the refinement (which supplies the projection + the decidable intro) —
so {refinement-`Char` + demoted ops} (**zero-delta** computational ops,
**zero-NEW-delta** `DecEq`/`Ord Char` laws — no *new* postulate beyond `Int`'s)
is the coherent, strictly-better option over {opaque + native ops}
(Char-specific postulate-only, type stays in the TCB). Given bignum `Int` (F1):

- `eq_char`/`leq_char`/ordering ⇒ `eq_int`/`leq_int` on the free projection
  `proj : Char → Int` (CV-confirmed constant-factor) → **DEMOTE** — the **ops**
  reduce (zero-delta computational); the `DecEq`/`Ord Char` **law instances**
  are **zero-NEW-delta** over `Int` (re-home to the lawful-classes lane,
  §5.9.1(3)).
- `Char.toInt` = `proj` (derived); `Int.toChar : Int → Option Char` =
  refinement-intro with the decidable check → **face-(c)** (`None` on
  surrogate/out-of-range) → derived.

**Two load-bearing soundness pins** (the refinement is sound only with these):

1. **`isScalar : Int → Ω` via the Bool-decidable reflection** — pin the
   *encoding*, not just the sort: `isScalar c := IsTrue (inRangeBool c)`, with
   `inRangeBool c : Bool` =
   `(0 ≤? c && c ≤? 0xD7FF) || (0xE000 ≤? c && c ≤? 0x10FFFF)`.
   **`IsTrue b` is a genuine sub-singleton → proof-irrelevant → Ω.** A *naive*
   `(…) ∨ (…) : Ω` is the **forbidden direction**: a raw disjunction is the sum
   `A + B`, a two-constructor **proof-relevant** type that cannot sit at Ω
   directly (`16 §1.3`, the `Bool → Ω` trap). Range-disjointness does **not**
   rescue it — at most one summand is inhabited, but the *type* `A + B` still
   carries the injection tag, so it stays relevant. Ω-admissible only via the
   `IsTrue` form (pinned) or explicit truncation `‖A + B‖`. **Load-bearing:**
   pin 1's payoff — Ω-PI making `Char` equality reduce to **codepoint** equality
   (two `Char`s, same codepoint, distinct scalar proofs, equal by Ω-PI — a
   **zero-delta** kernel fact; the `DecEq Char` lawful instance itself is
   zero-NEW-delta) — holds only if `isScalar` is *actually*
   proof-irrelevant, which the naive-`∨` is not (and forcing an `A + B` into Ω
   would re-open the `Bool → Ω` inconsistency).
2. **String→`Char` extraction emits the canonical scalar proof** — `char_at` /
   `string_to_list_char` construct `(c, canonical_proof)`; sound because a
   valid-UTF-8 `String` only yields scalars, so `isScalar c` reduces to its
   canonical inhabitant. **No primitive can fabricate a non-scalar `Char`.**

`Char` literals (`'a' ↝ 97` + the scalar proof) are an elaborator concern.

### 5.9.1 `Char` delivery contract — refinement + derived ops (incl. `Ord Char`)

This subsection is the **normative contract** the §5.9 DEMOTE satisfies
(representation-independent); perishable anchors live in the WP brief.

**(1) Removal-not-shadowing — a double TCB removal.** The primitive `Char`
**type** registration (`reg_ty!("Char")`) is **deleted**; `Char` becomes the
refinement `{ c : Int | isScalar c }` over `Int`. `Char` has **no** native
computing ops on main today (no `eq_char`/`leq_char`/`Char.toInt`/`Int.toChar`
arm — `string_to_list_char` is a `Neutral` stub), so this is a type-conversion +
**net-new derived ops**, not the removal of working native ops. **No** surviving
primitive `Char` type; **no** new kernel flag / `Decl` variant (AC-G).

**(2) The refinement + `isScalar` Ω-encoding — soundness pin 1 (load-bearing).**
`isScalar c := IsTrue (inRangeBool c)`, with `inRangeBool c : Bool` computed by
**value-level** `Bool` `&&`/`||` over the decidable `Int` comparisons (`leq_int`
/ derived `<`, §5.2.2), then bridged to Ω by the **sub-singleton**
`IsTrue : Bool → Ω` (`IsTrue true ≡ Top`, `IsTrue false ≡ Bottom`). It is
**never** a naive `(…) ∨ (…) : Ω` — a raw disjunction is the proof-relevant sum
`A + B` (the `Bool → Ω` trap, `16 §1.3`); range-disjointness does **not** rescue
it (the injection tag stays). The build producer-greps that the definition is
`IsTrue(<computed Bool>)`, **not** a `∨`/`∃`/multi-constructor form at Ω.
**Payoff (load-bearing):** Ω-PI makes `Char` equality reduce to **codepoint**
equality (same codepoint, distinct scalar proofs → equal by Ω-PI — a
**zero-delta** kernel fact) — holds **only** because `isScalar` is genuinely
proof-irrelevant, which the naive `∨` is not. (The Ω-PI *collapse* is
zero-delta; the full `DecEq Char` **lawful instance** is **zero-NEW-delta**,
bottoming out at `DecEq Int`, and re-homes to the lawful-classes lane per (3).)

**(3) Derived ops over the projection — all reduce this tranche.** With
`proj : Char → Int` the free projection:

- `eq_char := eq_int` on the projections — the **op** reduces (built `eq_int`),
  zero-delta computational; the `DecEq Char` **lawful instance** is
  **zero-NEW-delta** (bottoms out at `DecEq Int`; re-homes to the lawful-classes
  lane, like `Ord Char`).
- `Char.toInt := proj` — reduces.
- `Int.toChar : Int → Option Char` = refinement-intro guarded by the decidable
  check (**face-(c)**): `isScalar c` reduces (via `leq_int`, §5.2.2) to
  `Top`/`Bottom`, so `Int.toChar` reduces to `Some ⟨c, proof⟩` on a scalar and
  `None` on a surrogate/out-of-range `Int`, **never** a silent `Some`.
  Discriminating (AC-C3): `Int.toChar 0xD800` (surrogate) and
  `Int.toChar 0x110000` (out-of-range) **reduce to `None`**; a valid scalar
  reduces to `Some` — and this **fails against a stub `isScalar := true`** (the
  obligation actually reduces, not name-matches).
- `Ord Char`: `leq_char a b := leq_int (proj a) (proj b)` — the **op** reduces
  (via `leq_int`, §5.2.2), computational and unaffected; its completeness rides
  `leq_int` reducing (lands this tranche → **no F5 dependency remains for
  `Char`**). The `Ord Char` **law-carrying instance** homes in the
  **lawful-classes lane** (next to `Ord Int`), **not** this demote: after
  refinement erasure `Char ≡ Int` (`21 §6.3`, `proj` the identity), so its `Ord`
  laws (reflexivity / antisymmetry / transitivity / totality) **are**
  `Ord Int`'s laws — established by **transporting** `Ord Int`'s visible
  audited-delta
  `Axiom`, **zero-NEW-delta** (no *new* postulate beyond `Int`'s; `§5.4`
  `Ord Int` postulate-only), **not** independently `Axiom`-free via `proj`
  injectivity. The honesty discriminator is a **declared-visible** law (the
  transported `Ord Int` `Axiom`) vs a **deceptive empty/false stub**, not
  zero-delta-vs-postulate.

**(4) String→`Char` extraction COMPUTES the canonical scalar proof — the runtime
face (soundness pin 2).** `char_at` / `string_to_list_char` construct
`(c, canonical_proof)` where the `isScalar c` witness is **reduced** from the
`String`'s UTF-8 validity invariant — a valid `String` yields only scalars, so
`inRangeBool c` reduces to `true` and the proof is the canonical inhabitant
`tt : Top`. The witness is **computed**, never `declare_postulate` / `Axiom` /
`sorry` / hand-fed: an extraction path that *asserts* the scalar proof instead
of reducing it is the **trusted-not-proved hole** this pin exists to catch (the
static-vs-runtime-face discipline — grep the producer for obligation discharge,
not the type signature). **No primitive may fabricate a non-scalar `Char`.** The
refinement + derived ops sit in the tested-not-trusted ring over F1 + §5.2.2;
the demote removes the `Char` primitive type and adds **no** `trusted_base()`
line **of its own** and **no** kernel touch — a double net shrink (type +
would-be native ops). The one postulate the combined `Decimal`/`Char` tranche
adds is `Decimal`'s deferred-align `decimalPow10Unbounded` (§5.6.1), **not** a
`Char` line — so the tranche net is **−5**, a shrink, not a zero-addition.

### 5.10 Basic data structures — no primitive reductions

`List`/`Option`/`Result` are transparent inductive `data` (`34`) → **derived**,
consumed by `elim` — no primitive ops. `Array`/`Map`/`Set` are **abstract
types** (`33 §4`, `37 §3`) with **library-level** ops (`get`/`lookup`/`insert`
over the interface), **not `PrimReduction`s**. The audit adds **nothing to the
primitive-reduction axis** here.

**Precision (so the TCB claim is exact):** "no primitive ops" ≠ "no trust." The
operation-trust of `Map`/`Set` — whether `lookup`/`insert` are `foreign`
postulates in `trusted_base()` or derived Ken over a representation — lives on
the **FFI / library axis (§6)**, adjudicated there, not in this
primitive-reduction registry. "Confirms the TCB" here means *adds no primitive
reduction*, not *trust-free*.

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
Steward's call. The **TCB delta** Pat ratifies — the audit **net-shrinks** the
trusted base (the `Decimal`/`Char` demote is a **net −5**: six primitive/type
registrations leave against one honest-visible deferred-align postulate
`decimalPow10Unbounded` — a net shrink, **not** a zero-addition). **Leave**
(DEMOTE→derived — the trusted op removed and its class
laws gained: **zero-delta** on an inductive carrier like the `Bool` logic ops,
**zero-NEW-delta** over an opaque floor like `Int`/fixed-width for the rest —
except where a **non-canonical** carrier leaves an `Equal`-tying class law
undeliverable (`DecEq`/`Num Decimal`, §5.6.1(4)) — in
place of type-specific postulates): `Decimal` (type + ops), **`Char`** (type +
ops, a *double* removal), `neg_int`, the `Bool` logic ops, and
`checked`/`saturating`
(all fixed-width). **Enter** (GAP→NATIVE, spec-mandated): `div`/`mod` and the
**completed `IntN↔Int` conversion floor** (plus the `Int`/`Float`/`Decimal`
conversions). **Deleted** (RETIRE): the legacy wrapping path. Every surviving
native op is ratified **iff** it reduces correctly (bignum / checked-not-wrap /
exact / NFC / Ω-scalar-proof).
