# Conformance — F1 genuine arbitrary-precision `Int` (retire the i128 ceiling)

Format: `../../README.md`. These pin **WP F1**
(`docs/program/wp/F1-bignum-int.md`) — the first Phase-2 BUILTINS tranche WP —
to executable, black-box cases: the interpreter's `Int` arithmetic (`add_int` /
`sub_int` / `mul_int` / `eq_int`) is genuinely **arbitrary-precision and
total**, matching the sealed `spec/10-kernel/18a §5.2` "NATIVE iff bignum"
boundary that Pat ratified (2026-07-02). Anchors: `18a §5.2` (the
`Int`-arithmetic floor rows), `18a §3` (the differential-oracle discipline — the
*sole* external net for a native op), `18a §2` (the partiality discipline), the
L1 numeric-tower surface (`35`), and the runtime store representation
(`ken-runtime` `Value::BigInt { sign, limbs }`, `canonical.rs`, `41 §5`).

## The non-reproduction: the i128 ceiling (distinct from AC1's f64 carrier)

This seed and `seed-numbers.md` pin **two different** numeric non-reproductions
of the prototype, needing **two different discriminating witnesses** — they are
not two homes for one property:

- **`seed-numbers.md` AC1 (`int-arbitrary-precision-above-2^53`)** targets the
  **f64-carrier** bug (an `Int` silently stored as an IEEE binary64). Its
  witness is `10²⁰ + 1` — off the f64 grid (ULP 2¹⁴ at 10²⁰), so the exact
  `…001` differs from the f64 image `…000`. But `10²⁰ ≈ 2⁶⁶` **fits `i128`**, so
  that witness is **green-vs-green under the *i128-ceiling* bug** — this is the
  OF1 blind-spot surfaced in the BUILTINS audit.
- **This seed (F1)** targets the **i128-ceiling** bug: the interpreter's `Int`
  is currently `EvalVal::BigInt(i128)` and `exact_int_binop` computes on `i128`
  (perishable anchors `ken-interp/src/eval.rs` — the `BigInt(i128)` variant and
  the `i128`-typed `op`), so a product crossing `2¹²⁷` **debug-panics or
  release-wraps**. The discriminating witness therefore lives **across the i128
  ceiling** (`2¹²⁷` / `2¹²⁸`), where `seed-numbers.md` AC1's `2⁶⁶` witness
  cannot reach. F1 delivers the "iff bignum" half of the ratified `18a §5.2`
  boundary.

Neither bug is a false kernel proof: the kernel keeps `Eq` at a primitive type
**neutral** (`ken-kernel/src/obs.rs:84`, no `eq→Eq` bridge, no `ken-interp`
dependency), so the i128 wrap is a **wrong value in the tested-not-trusted
interpreter ring** (`18a §4`), never a false proof — the F1 correctness AC is
the **precondition for the eventual K3 trusted-base promotion** of these ops (a
reduction that can produce a wrong value cannot be promoted to kernel-executed).

## Reading disciplines (what these cases pin, and how they flip)

- **AC1 is a structural VALUE assertion across the i128 ceiling, not a type
  check.** Each operand is chosen so its exact result is **outside `i128`**
  (`> 2¹²⁷ − 1`). Under the exact bug this targets — `i128`-width arithmetic —
  the operation panics (debug) or wraps to a wrong value (release); the correct
  bignum reduces to the exact result. The value is the discriminator, so the
  case **flips** green↔red under the ceiling bug
  ([[discriminating-conformance-verdict-must-flip]]).
- **AC2's oracle is the SOLE external net for a NATIVE floor op — and it is only
  a net if it is INDEPENDENT (`18a §3`).** `add_int` / `mul_int` / `eq_int` are
  **floor ops**: no lower Ken op to defer to, reduced in trusted Rust. Their one
  external check is the Rosetta differential against an **independent
  reference**. The reference must compute by a route **independent of the op
  under audit** — a genuinely distinct source (golden vectors from an external
  arbitrary-precision tool, or a distinct crate / schoolbook implementation) —
  **never the production bignum crate on both sides** (that is `18a §3`'s
  explicit green-vs-green anti-pattern: the same wrong code recomputed). Two
  internal cross-checks are therefore **NOT** admissible oracles here and this
  seed rejects them: (i) a "native vs the interpreter's own reduction"
  differential (they are the same path); (ii) an **algebraic-consistency** check
  like `mul` commutativity / `add` associativity computed **by the production
  crate itself** (it aliases the native path — a symmetric bug passes both
  sides). `mul_int` has **no cheap defining law** to make it non-circular
  internally (`18a §3`), so its net **must** be the across-2¹²⁷
  independent-bignum reference. See [[builtins-tcb-audit-disciplines]].
- **AC2 must feed the BOUNDARY, not the interior — this is the OF1 correction.**
  The existing `seed-numbers.md` AC1 nets at `2⁶⁶`, which fits `i128` and is
  green on the ceiling bug. The oracle operand set is pinned **across the real
  ceiling** (`2⁶³`, `2⁶⁴`, `2¹²⁷`, `2¹²⁸`, and a `> 2¹⁰⁰⁰` chain), plus the sign
  boundary (negatives, zero, mixed-sign `mul`) per `18a §2`/`§3` — an oracle
  that samples only the total interior cannot catch a silent out-of-domain
  result.
- **AC3 asserts a STRUCTURAL / trace output (byte-identity + content-hash),
  never a value.** The store round-trip property is not observable in an `Int`
  value — two encodings of the same integer print the same. So the case asserts
  the **canonical byte encoding** (`canonical.rs`, `encode_canonical`) and the
  content-address are **identical** across
  `eval → Value::BigInt { sign, limbs } → eval`, and that `minimal_limbs` (strip
  trailing-zero limbs; zero keeps one zero limb — `canonical.rs:61`) holds. A
  value-only assertion here would be green-vs-green
  ([[soundness-AC-static-vs-runtime-face]] structural face; the X1
  structural-output discipline).
- **AC3 is an ESTABLISH, not a preserve — and it must DRIVE the real eval→store
  producer, never hand-feed a `Value::BigInt`.** Verified at source
  (`ken-interp/src/eval.rs:212` `to_rt`): the eval→store conversion has arms for
  `Bool`/`Int`/`Ctor`/`Pair`/`Closure`/`Bytes`/`Str` but **no `BigInt` arm**, so
  an `EvalVal::BigInt` falls to the catch-all `None` and **cannot intern today**
  — the round-trip does not merely round to `i128`, it **does not exist**. F1
  **establishes** it (adds the `BigInt`→`Value::BigInt` arm), so AC3 asserts the
  round-trip *holds post-F1*, not that it is unchanged from a prior working
  state. The trap this creates: `ken-runtime` **already** round-trips a
  **hand-built** `Value::BigInt` (`store.rs:485`, `canonical.rs:360` are green
  today with zero eval-path bignum) — so a case that constructs a
  `Value::BigInt` directly and checks `canonical.rs` re-validates the
  **pre-existing** store consumer with **zero F1**, the
  [[conformance-hand-feeds-the-deliverable]] green-vs-green pattern. AC3's value
  **must arise from an evaluator arithmetic op** (a `> i128` `mul_int` result)
  and be interned **through `to_rt`** — the driver is the real producer F1
  wires, verified by grepping `to_rt`, not the hand-fed binding.
- **AC4/AC5 are landing-discipline / build-artifact ACs, not black-box
  behavioral cases.** AC4 pins the **no-regression gate shape**
  (workspace-green, the K7 reduction-value-change lesson) and names the
  migration surface; AC5 pins the **crate-vetting checklist** (`63` + ADR 0009).
  Both are **hard ACs the merge Decision verifies**, stated here so the build
  frame and the reviewers share one contract — not "given/expect/why" reduction
  cases.

**Tags.** **(soundness)** = a correctness / TCB commitment that must never
regress: the no-wrap totality across the i128 ceiling (AC1), the
independent-oracle discipline (AC2), the store-round-trip content-address
stability (AC3), and the interp-local / no-`trusted_base()`-promotion boundary.
**(oracle)** = a value confirmed against Ken's reference interpreter once
available; before then, grounded against `18a` + `35` + `41` + first principles.
**(hard-AC)** = a build-gate obligation the merge Decision checks (AC2
independence, AC4 workspace-green, AC5 dependency-delta) rather than a value the
interpreter emits.

## AC1 — no-wrap totality across the i128 ceiling  (soundness)

Each result is **exactly `2¹²⁸`** (= `340282366920938463463374607431768211456`)
or larger — strictly **outside `i128`** (`i128::MAX = 2¹²⁷ − 1`), so the current
`i128` arithmetic cannot represent it.

### surface/numbers/f1-mul-int-crosses-i128-ceiling  (soundness)
- spec: `18a §5.2` (`mul_int` NATIVE iff bignum), `35 §1`, `41 §5`
- given: `mul_int (2^127) 2 : Int` (equivalently `2^127 * 2`)
- expect: **reduces-to** the exact value `2¹²⁸` — stored as a minimal-limb heap
  bignum (`Value::BigInt`, tag `0x01`), **no panic, no wrap**.
- why: `2¹²⁷ · 2 = 2¹²⁸` overflows `i128` (max `2¹²⁷ − 1`). Under the exact bug
  this targets — `i128`-width `mul` — the op **debug-panics**
  (`attempt to multiply with overflow`) or **release-wraps** to `0`/a wrong
  residue; the correct bignum reduces to `2¹²⁸`. Structural value flip.

### surface/numbers/f1-mul-int-2^64-squared  (soundness)
- spec: `18a §5.2` (`mul_int`), `35 §1`
- given: `mul_int (2^64) (2^64) : Int`
- expect: **reduces-to** `2¹²⁸` exactly — no wrap. (`2⁶⁴ · 2⁶⁴ = 2¹²⁸`; each
  operand fits `i128` but the **product** does not — the multiply is where the
  ceiling bites.)
- why: pins that the **operands fitting `i128` is not sufficient** — the product
  crossing the ceiling is the failure. A build that widens operand storage but
  keeps an `i128` multiply intermediate still flips here.

### surface/numbers/f1-add-int-crosses-i128-ceiling  (soundness)
- spec: `18a §5.2` (`add_int`), `35 §1`
- given: `add_int (2^128 - 1) 1 : Int`
- expect: **reduces-to** `2¹²⁸` exactly — no wrap. (`2¹²⁸ − 1` is already
  outside `i128`; this pins that even `add` — not just `mul` — is total across
  the ceiling, and that a build must migrate its operand representation before
  the sum, not only its product.)
- why: `add_int` is a floor op with the same "iff bignum" burden as `mul_int`
  (`18a §5.2`). A build that only fixes `mul` flips here.

### surface/numbers/f1-product-chain-exceeds-2^1000  (soundness)
- spec: `18a §5.2`, `35 §1`
- given: a product chain `mul_int` folded over `[2^128, 2^128, …]` (eight
  factors of `2¹²⁸`) `: Int` — the result is `2¹⁰²⁴ > 2¹⁰⁰⁰`.
- expect: **reduces-to** the exact `2¹⁰²⁴` — no panic, no wrap, no precision
  loss at any intermediate. Every intermediate product (`2²⁵⁶`, `2³⁸⁴`, …) is
  itself outside `i128`, so the totality must hold **compositionally**, not just
  for one op.
- why: pins **unbounded** precision, not merely "one op past the ceiling." A
  build with any residual fixed-width intermediate on the arithmetic path flips
  at the first `> i128` partial product. (`18a §5.2`: total, no `i128`
  intermediate anywhere on the arithmetic path.)

## AC2 — independent differential oracle  (soundness, hard-AC)

### surface/numbers/f1-oracle-independent-reference  (soundness, hard-AC)
- spec: `18a §3` (the differential oracle — sole net for a native floor op),
  `18a §2` (boundary operands), `18a §5.2`
- given: the boundary operand matrix, each op (`add_int`, `sub_int`, `mul_int`,
  `eq_int`) evaluated at:
  - **magnitude boundary:** operands and results straddling `2⁶³`, `2⁶⁴`,
    `2¹²⁷`, `2¹²⁸`, and a `> 2¹⁰⁰⁰` product;
  - **sign boundary:** negatives, zero, and **mixed-sign** `mul`
    (`(−2^127) · 2 = −2¹²⁸`, `(−a)·(−b) = a·b`, `a · 0 = 0`);
  - **`eq_int` at the boundary:** `eq_int (2^128) (2^128) ⇒ true`,
    `eq_int (2^128) (2^128 + 1) ⇒ false` — equality decided on the **bignum**
    representation, not an `i128` image that would collapse distinct `> i128`
    values.
- expect: for **every** operand, the interpreter's native reduction **agrees**
  with an **independent** arbitrary-precision reference. The reference is one
  of: (a) **golden vectors** precomputed by an external arbitrary-precision tool
  (e.g. an external CAS / a language with native bignums), checked into the
  corpus; or (b) a **distinct** implementation (a different crate, or a small
  schoolbook long-multiplication reference) — **never the production bignum
  crate invoked on both sides.**
- why: `add_int` / `mul_int` / `eq_int` are **floor ops** — trusted Rust
  reductions with **no lower Ken op** to defer to (`18a §3`), so the
  differential is their **sole external net**. Independence is the whole net: a
  "native vs the interpreter's own path" differential, or an internal
  algebraic-consistency check (`mul` commutativity computed **by the production
  crate**), is **green-vs-green** — the same wrong code passes both sides
  (`18a §3`). `mul_int` has no cheap defining law, so it cannot self-net; the
  across-`2¹²⁷` independent reference is mandatory.
- **HARD-AC (flag if unmet):** if the corpus **cannot source an independent
  reference** (only the production crate is available on both sides), the AC is
  **unmet** and the build must be flagged — a same-crate differential does not
  satisfy AC2, however green it reads. The oracle-ref cell for these floor ops
  is **not** `N/A`; it is the independent reference, and its independence is
  verified at review.

## AC3 — store round-trip byte-identity + `minimal_limbs`  (soundness)

Structural / trace assertions on the canonical encoding — never a value.

### surface/numbers/f1-store-roundtrip-above-i128-byte-identical  (soundness)
- spec: `18a §5.2`, `ken-runtime` `Value::BigInt { sign, limbs }` (`values.rs`),
  `canonical.rs` (`encode_canonical`), `eval.rs:212` (`to_rt`), `41 §5`
- given: a `> i128` value **produced by the evaluator** — `mul_int (2^127) 4`
  (`= 2¹²⁹` exactly), interned **through `to_rt`** to
  `Value::BigInt { sign, limbs }` and read back — **not** a hand-constructed
  `Value::BigInt`.
- expect: the round-trip is **byte-identical** — `encode_canonical` produces the
  **same** bytes and the **same** content-address for the interned value and the
  value re-read into eval, and the reconstructed evaluator value reduces
  identically. (`2¹²⁹` requires ≥ 3 `u64` limbs, so the round-trip exercises
  real multi-limb storage.)
- why: **F1 establishes this round-trip; it does not preserve one.** `to_rt`
  (`eval.rs:212`) has **no `BigInt` arm** today, so an `EvalVal::BigInt` interns
  as `None` — the eval→store bignum path **does not exist** before F1. The case
  must therefore **drive the real producer**: the value **arises from an
  arithmetic op** and interns via `to_rt`, so it flips on the actual F1 wiring
  (`None` / no-intern before → byte-identical round-trip after). A case that
  hand-built a `Value::BigInt` and checked `canonical.rs` would pass **today
  with zero F1** (`store.rs:485` / `canonical.rs:360` already round-trip
  hand-built bignums) — the [[conformance-hand-feeds-the-deliverable]] trap.
  Structural (byte / content-hash) assertion, not a value.

### surface/numbers/f1-dedup-content-address-stable-across-paths  (soundness)
- spec: `canonical.rs:61` (`minimal_limbs`), `44` (content-addressed store /
  dedup), `18a §5.2`
- given: the **same** `> i128` integer `2¹²⁸` reached by **two distinct
  evaluator arithmetic paths** — `mul_int (2^64) (2^64)` and `mul_int (2^127) 2`
  — each interned via `to_rt`.
- expect: both intern to the **identical content-address** (one store slot —
  dedup holds), because `to_rt`'s `Value::BigInt` output is **canonical**
  (`minimal_limbs`-respecting) regardless of which arithmetic path produced it.
  A build whose `to_rt` emits a **non-canonical** bignum (e.g. carrying a
  crate-internal high-order zero limb on one path but not the other) yields
  **two** content-addresses for one value and **flips** (dedup breaks, the `44`
  store invariant).
- why: the F1-level face of the `minimal_limbs` invariant — asserted by
  **driving the real producer** (two eval paths → `to_rt`) and checking
  content-address **equality**, not by hand-feeding a non-minimal `[0,1,0]` limb
  vector to `canonical.rs` (which tests the **pre-existing** encoder with zero
  F1 — the hand-feed trap). Structural (content-hash) equality on two
  representations of one value — vacuous as a value assertion (both *are*
  `2¹²⁸`), discriminating as a store-address assertion
  ([[abstraction-visibility-feature-soundness-gate]] byte-identity discipline).
  `minimal_limbs`'s trailing-strip **rule itself** is a pre-existing
  `canonical.rs` unit-test's job; F1's obligation is that its produced bignums
  **feed** it canonically, which this content-address-equality case pins.

### surface/numbers/f1-zero-and-sign-canonical  (soundness)
- spec: `canonical.rs:61` (zero keeps exactly one zero limb), `values.rs`
  (`sign`), `18a §5.2`
- given: `0 : Int` (via `sub_int n n`) and a negative `−(2^128) : Int` (via
  `neg_int (2^128)` / `sub_int 0 (2^128)`), each round-tripped through the
  store.
- expect: `0` canonicalizes to **exactly one zero limb** (not zero limbs, not
  many) with a canonical `sign`; the negative round-trips with `sign`
  **preserved and distinct** from its positive (`−2¹²⁸` and `+2¹²⁸` have
  **different** content-addresses). A build that drops the sign, or emits `0`
  with a non-canonical limb count, **flips**.
- why: the sign and the zero-limb rule are the two edges of the `Value::BigInt`
  canonical form; the negative case is the **non-degenerate sign pair**
  (`+n`/`−n` must not collapse to one content-address —
  [[taint-axis-orientation-needs-distinguishing-pair]] applied to the sign
  axis).

## AC4 — workspace-green landing discipline (the K7 lesson)  (hard-AC)

- spec: `docs/program/wp/F1-bignum-int.md` AC4, `COORDINATION §7`
- **AC (build-gate, not a reduction case):** the no-regression gate is
  **`cargo test --workspace`**, **never** `-p ken-interp`. F1 changes reduction
  **values**, so its blast radius is **workspace-wide**: any downstream proof
  term, golden vector, or `.ken` artifact that encoded the old `i128`-ceiling
  behavior (a wrapped/panicking product, a `Value::BigInt(i128)` round-trip
  bound) **migrates in the same land-together green unit**. Asserting a
  `ken-interp`-only diff is the exact K7 mistake (a reduction-value change with
  a wider landing unit than the file it touches).
- **Migration surface to sweep (name-and-migrate, do not silently drop):** the
  interp eval tests (`ken-interp` numeric reductions), the store/canonical tests
  (`ken-runtime` `canonical.rs`, `store.rs` — which already round-trip
  `Value::BigInt`), and any `packages/*.ken` / golden numeric fixture asserting
  a result at or above the `i128` boundary. If a fixture rode the old wrap/panic
  behavior, it is **corrected in this unit** (its old expectation was the bug),
  not carried forward.
- why: [[check-main-via-git-object-store-not-find]] scope-honesty — the
  soundness surface is narrow (interp-only, kernel untouched, legitimately
  asserted) **but** the landing unit is wide; the two must be distinguished in
  the diff and the no-regression AC set to workspace scope.

## AC5 — dependency-delta recorded (§63 / ADR-0009 crate-vetting)  (hard-AC)

- spec: `spec/60-security/63` (dependency vetting),
  `docs/adr/0009-capability-supply-strategy.md` (curate-not-construct, tier-a),
  `docs/program/wp/F1-bignum-int.md` AC5
- **AC (build-gate, verified at the merge Decision):** the curated
  arbitrary-precision crate is: 1. **pure safe Rust** — `unsafe`-status
  **verified**: either the crate is `#![forbid(unsafe_code)]` / carries **no**
  `unsafe`, or every `unsafe` block is **audited and noted** in the
  dependency-delta record. (Default candidate `num-bigint`; fall back to a
  `forbid(unsafe_code)` equivalent — `ibig` / `dashu` — if `num-bigint` carries
  un-audited `unsafe`.) 2. **permissively licensed** — non-copyleft (MIT /
  Apache-2.0 / BSD), so it is **clean-room-compatible** (no GPL/AGPL/CeCILL
  contact — my copyleft-leakage lane; the vendored source is
  original-license-clean, no `local/refs/` provenance). 3. **version-pinned** —
  an exact version pin, not a range. 4. **dependency-delta recorded** — the
  addition (crate, version, license, `unsafe`-status, transitive-dependency
  delta) is **documented** per `63` + ADR 0009's curation rubric (ADR 0009
  step-1 "select an industry-trusted component" for an outer-ring capability).
- why: F1 is the **first Phase-2 dogfood of ADR 0009's supply strategy** —
  curation (not in-tree construction) is what reaches the trust an oracle-netted
  outer-ring evaluator needs; in-tree/proved construction is the **deferred K3
  question** (ADR 0009 tier-c), not this WP. The checklist is a **hard AC** the
  merge Decision verifies, because a curated dependency is only as trustworthy
  as its vetting record ([[builtins-tcb-audit-disciplines]] — the trust level is
  a typed, recorded choice, PRINCIPLES §12).

## Coverage map (AC → cases)

- **AC1** (no-wrap totality across i128) — `f1-mul-int-crosses-i128-ceiling`,
  `f1-mul-int-2^64-squared`, `f1-add-int-crosses-i128-ceiling`,
  `f1-product-chain-exceeds-2^1000`
- **AC2** (independent differential oracle) — `f1-oracle-independent-reference`
  (hard-AC)
- **AC3** (store round-trip byte-identity + `minimal_limbs`) —
  `f1-store-roundtrip-above-i128-byte-identical`,
  `f1-dedup-content-address-stable-across-paths`, `f1-zero-and-sign-canonical`
- **AC4** (workspace-green landing discipline) — landing-discipline AC (hard-AC)
- **AC5** (dependency-delta / crate-vetting) — checklist AC (hard-AC)

## Cross-case sweep (`18a §3` / `35 §7`)

- **Floor-op oracle class agrees.** Every native floor op (`add_int`, `sub_int`,
  `mul_int`, `eq_int`) is netted by the **same** independent-reference
  differential across the same boundary matrix; none is netted by an internal
  self-consistency check (which would alias the native path). No op admits a
  same-crate-on-both-sides differential as its net.
- **Ceiling-crossing totality class agrees.** Every `add`/`sub`/`mul` whose
  exact result exceeds `i128` reduces to the exact value with no panic/wrap; the
  cases instantiate `mul` (operand-fits/product-exceeds), `add` (`2¹²⁸−1 + 1`),
  and a compositional chain — the interpreter dispatches the `exact_int_binop`
  arm uniformly, so totality generalizes across the three ops and all
  magnitudes. No case admits a fixed-width intermediate on the arithmetic path.
- **Store-canonical class agrees.** Every `> i128` value round-trips
  byte-identically; `minimal_limbs` strips trailing zeros so non-minimal and
  minimal encodings of one value are content-address-equal; zero keeps exactly
  one zero limb; `sign` is preserved and `+n`/`−n` stay distinct. No case admits
  a raw-limb (unstripped) or sign-dropping encoding.
- **Kernel-untouched boundary holds.** No case asserts a **kernel** reduction of
  these ops — F1 is interp-local; `obs.rs:84` keeps `Eq` at a primitive type
  neutral, so `eq_int (2^128) (2^128)` reduces in the **interpreter** to `true`
  but the **kernel** does not reflect it into a definitional `Eq`. This is the
  no-`trusted_base()`-promotion / no-false-proof line (`18a §4`).

## Relationship to `seed-numbers.md` (single-home discipline)

This seed and `seed-numbers.md` AC1 pin **distinct** non-reproductions (i128
ceiling vs f64 carrier — see the top section), so there is **one home per
property**, not a duplicate. `seed-numbers.md` AC1 stays authoritative for the
**f64-carrier** surface property and its off-grid `10²⁰+1` witness; this seed is
authoritative for the **interpreter's bignum delivery across the i128 ceiling**
and the store round-trip. A one-line cross-reference is added to
`seed-numbers.md` AC1's `why` noting the ceiling corpus lives here (the OF1
blind-spot is closed here, not by loosening AC1's f64 witness).

## Deferred to sibling tranche WPs (not covered here)

Per `18a §4.1` / the F1 brief's OUT scope — flagged so no case over-reaches:

- **F5** — `leq_int` registered-but-unreduced: F1 guarantees only the
  representation `leq_int` will compare over; the reduce arm lands in F5.
- **F2** — bare fixed-width `*_intN` obligation emission; **F3** — retire the
  legacy wrapping arms.
- **`div_int` / `mod_int`** partiality (the face-(b) `NonZeroDivisor`
  obligation, negative-`mod` truncated) — `18a §2` / `35 §3.1`, already seeded
  in `seed-numbers.md §3.1`; F1's representation must **support** correct bignum
  div/mod when that WP lands, but F1 does not add the div/mod arm.
- **`Decimal` / `Char` demotes** and any **K3 trusted-base promotion** — ride F1
  but are their own frames.
