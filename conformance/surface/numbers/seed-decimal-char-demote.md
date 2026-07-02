# Conformance — Decimal / Char DEMOTE→derived (Phase-2 tranche #2)

Format: `../../README.md`. These pin **WP decimal-char-demote**
(`docs/program/wp/decimal-char-demote.md`) — the second Phase-2 BUILTINS
tranche, riding F1's landed bignum `Int` (`bb40654`). Both `Decimal` and `Char`
are **removed from `trusted_base()` at the type level** (`18a §5.1`) and
replaced by **derived Ken definitions over exact-`Int` arithmetic**:

- **`Decimal → (coeff : Int, exp : Int)`** (`18a §5.6`) — exact base-10,
  arithmetic derived in-Ken, retiring the native saturating
  `add/sub/mul/eq_decimal` primitives (the **F4** bug: `mul_decimal` uses
  `saturating_mul`, `decimal_eq` saturates so two *distinct* decimals compare
  `True`).
- **`Char → { c : Int | isScalar c }`** (`18a §5.9`) — the Unicode-scalar
  refinement over `Int`, with derived equality/ordering/conversions and **two
  load-bearing soundness pins** (the `isScalar` Ω-encoding; extraction computes
  the scalar proof).

**Ordering prerequisite (Steward ruling (A), thr_34jhda3bdrs8a).** The derived
`add`/`sub`/`eq_decimal` (exponent alignment) and Char `isScalar` (range checks)
need an `Int` **ordering** reduction; on main only `eq_int` reduces (`leq_int`
is registered-but-unreduced, `lt_int` unregistered). This tranche therefore
**pulls the `leq_int` `prim_reduce` arm up from F5** as a genuine prerequisite
(a bignum `≤` mirroring the landed `eq_int` arm) — **`trusted_base`-neutral**
(`leq_int` is already registered; wiring its outer-ring reduction adds no kernel
primitive) and **netted by an independent differential oracle** (**AC-L**, the
F1 AC2 discipline extended to comparison). **`<` / `min` / `|ea−eb|` are
DERIVED** from `leq_int` at the derived-op level — **no `lt_int` primitive**
(canonical `a < b := ¬ (leq b a)`, Steward's locked minimal form — pure `leq`,
no `eq`; `min`/`|ea−eb|` via `leq`+`sub`). With `leq_int` reducing, **`Ord Char`
lands this tranche** (`leq_char ⇒ leq_int ∘ proj` reduces) — the brief's earlier
"Ord Char rides F5" carve-out was downstream of the disproven premise and is
dropped.

Anchors: `18a §5.6.1`/`§5.9.1` (the landed delivery contracts), `18a §5.2.2`
(the `leq_int` prerequisite), `18a §5.6`/`§5.9` (the DEMOTE verdict rows),
`18a §5.1` (the type-level TCB removal), `18a §4` (F4/F5 findings + the
structural closing net), `18a §4.1` (the tranche order — corrected by this WP to
move the ordering reduction up), ADR 0009 (the adversarial-burden migration —
"native is the exception that must be earned"), `16 §1.3` (the `Bool → Ω` trap),
PRINCIPLES §5/§8/§12. The demote makes the soundness posture **strictly
better**: the laws become **zero-delta provable** and the F4 wrong-value hole
(the saturating false-`True` `eq_decimal`) **vanishes by construction**.
Trust-level precision (§4 F4): that false-`True` is a **wrong Bool value in the
tested-not-trusted `ken-interp` ring**, **not** an inhabitable false kernel
proof — `Eq Decimal` is kernel-neutral and `eq_decimal : … → Bool` has no
`eq → Eq` bridge, so no `refl : Eq Decimal a b` for `a ≠ b` is inhabitable (the
"refl-inhabits" reading is **over-classified**, §4 F4). The demote removes the
wrong-value path because no trusted `eq_decimal` exists to be wrong.

## Relationship to `seed-numbers.md` (single-home discipline)

`seed-numbers.md` pins the **surface semantics** — AC6
(`decimal-exact-while-float-honest`: `Decimal` is exact base-10, not an f64) and
`char-excludes-surrogates` (the surrogate block is not a `Char`). Those stay
authoritative for the *observable numeric model*. **This seed** pins the
**DEMOTE mechanism** — that the derivation genuinely computes (the F4 flip), the
TCB actually shrinks (removal, not shadowing), the laws are zero-delta, and the
`Char` refinement is sound (Ω-encoding + computed extraction proof). The
`leq_int` reduce-arm oracle (**AC-L**) homes here (this WP delivers it),
extending F1's `eq_int` oracle family (`seed-f1-bignum-int.md` AC2) to
comparison. Distinct properties, one home each; a one-line cross-reference is
added to `seed-numbers.md` AC6 + `char-excludes-surrogates`.

## Reading disciplines (what these cases pin, and how they flip)

- **The Decimal net is the discriminating FLIP, not a differential oracle
  (`18a §5.6`: oracle N/A — derived, not a trusted native op).** A derived op
  has no trusted reduction to Rosetta-differential against; the net is **AC-D2**
  — drive the **real derived** arithmetic on a vector where the *old* saturating
  `mul_decimal`/`decimal_eq` gave a **wrong** value and the *new* exact-`Int`
  one is correct, so the case **flips** green↔red on the demote. The flip vector
  is chosen to double as **F4's closure witness** (Architect): a real
  i64-saturation regime where `decimal_eq` returned a **false `True`**.
- **AC-D2 must DRIVE the derived producer, never hand-feed a `Decimal` value.**
  `Decimal` becomes a derived Ken type `(coeff:Int, exp:Int)`; the case applies
  the **derived `add`/`mul`/`eq`** to operands and checks the result — it does
  **not** construct the expected derived `Decimal` and test a downstream
  consumer ([[conformance-hand-feeds-the-deliverable]]). Verify by grepping the
  derived def is the producer, not a hand-fed binding.
- **AC-L is the pulled-up `leq_int` arm's INDEPENDENT differential oracle — the
  sole net for a native-tier reduction (`18a §3`, F1 AC2 discipline).** The
  `leq_int` bignum reduce arm is an outer-ring reduction; a wrong `≤` is a wrong
  value with no kernel backstop, so its net is a differential against an
  **independent** oracle — golden comparison verdicts hand-determined by the
  total order on ℤ, **never** `num_bigint`'s own `Ord` on both sides
  ([[soundness-AC-static-vs-runtime-face]] runtime face; the green-vs-green trap
  is using the production crate as its own oracle). Operands built via distinct
  paths (`Shl`/`Sub`/`Neg`), straddling the 2⁶³/2¹²⁷ boundaries and mixed sign.
- **AC-C3 is the predicate-definedness dual — the refinement obligation must
  actually REDUCE, not name-match.** `Int.toChar` on a surrogate / out-of-range
  `Int` must reduce to `None`; a valid scalar to `Some`. The case **flips
  against a stub `isScalar := true`** (which accepts everything → `Some`
  everywhere). A case that only checks a valid scalar accepts is green-vs-green
  under `isScalar := true`; the **non-degenerate pair** (reject surrogate/OOR
  *while* accept valid) is the net ([[two-arm-producer-needs-a-case-per-arm]]).
  This reduces **only** because ruling (A) pulled `leq_int` up — under the
  disproven premise it was stuck-neutral, not `None`.
- **The Ω-encoding pin is the `DecEq Char` SOUNDNESS check — a STRUCTURAL
  assertion on the `isScalar` def, not a value (Char pin 1).**
  `isScalar c := IsTrue (inRangeBool c)` where `inRangeBool c : Bool` is
  computed by the **decidable `Int` comparisons** — **value-level** `&&`/`||` on
  **`leq_int`** closed-interval bounds
  (`(0 ≤? c && c ≤? 0xD7FF) || (0xE000 ≤? c && c ≤? 0x10FFFF)`; the surrogate
  block `0xD800..0xDFFF` is excluded by the closed upper/lower bounds — **no
  strict `<` needed**), then bridged by the **sub-singleton**
  `IsTrue : Bool → Ω` (`IsTrue true ≡ Top`, `IsTrue false ≡ Bottom`). A naive
  `(…) ∨ (…) : Ω` is the **forbidden direction** — a raw disjunction is the sum
  `A + B`, a two-constructor **proof-relevant** type that cannot sit at Ω
  (`16 §1.3`, the `Bool → Ω` trap; range-disjointness does *not* rescue it, the
  injection tag stays). The soundness assertion is **structural**: the
  `isScalar` def head is `IsTrue (<computed Bool>)`, **never** a `∨`/`∃`/
  multi-ctor form at Ω. **Payoff (load-bearing):** Ω-PI makes `Char` equality
  reduce to **codepoint** equality — two `Char`s with the same codepoint but
  *distinct* scalar proofs are **equal by Ω-PI → zero-delta `DecEq Char`** —
  which holds **only** if `isScalar` is actually proof-irrelevant. See
  [[proof-relevant-inductive-cannot-be-declared-at-omega]].
- **Char pin 2 is the RUNTIME face — extraction must COMPUTE the scalar proof,
  not postulate it.** `String → Char` extraction (`char_at` /
  `string_to_list_char`) produces `(c, isScalar-witness)`; the witness must
  **reduce** from the `String`'s validity invariant (`inRangeBool c` computes to
  `true` → the proof is the canonical `tt`), **never** a `declare_postulate` /
  `Axiom` / hand-fed `sorry`. This is the static-face-vs-runtime-face split
  ([[soundness-AC-static-vs-runtime-face]]): the static face (the refinement in
  the type) is cheap; the **runtime face** (extraction actually discharging the
  obligation) is where a trusted-not-proved hole hides. Ruling (A) is what makes
  this face *deliverable* this tranche — under (B) extraction could only ship by
  **postulating** the witness (stuck `inRangeBool`), which is exactly this hole.
  The assertion greps the **producer** for the obligation *discharge* (reduces
  `tt`), not the type signature. No primitive may fabricate a non-scalar `Char`.
- **AC-G is removal-not-shadowing — a STRUCTURAL producer-grep, not a value.**
  The demote shrinks `trusted_base()` **only** if the `reg_ty!("Decimal")` +
  `reg_binop!`/`reg_cmpop!` for `add/sub/mul/eq_decimal` + `reg_ty!("Char")`
  registrations are **gone** from `numbers.rs`, the `*_decimal` arms gone from
  `eval.rs::prim_reduce`, and the derived defs introduce **no new kernel flag /
  `Decl` variant** (reuse F1's exact-`Int`). The pulled-up `leq_int` reduce arm
  is **`trusted_base`-neutral** — `leq_int` is *already registered*
  (`numbers.rs:233`), so wiring its outer-ring reduction adds **no** primitive/
  postulate; the net is still a **pure shrink** (four op removals + two type
  removals). A demote that leaves a primitive registered **and** adds a derived
  def **grows** the surface — that is the failure
  ([[abstraction-visibility-feature-soundness-gate]], "reuse the constant, never
  a new flag").
- **Honesty about the boundary (§8).** `DecEq Char` (via `eq_int`) **and**
  `Ord Char` (via `leq_int`) are **both delivered + reducing this tranche**
  under ruling (A). No `leq_char`/`eq_char` native arm exists on main today, so
  routing through `leq_int`/`eq_int` is **net-new, not a regression**. What
  stays OUT: `checked_*`/`saturating_*`/`neg_int` and the `IntN↔Int` conversion
  floor + `Float.toDecimal`/`Decimal.toFloat` — the later **conversions**
  tranche (`18a §5.3`/`§5.7`); the `leq_int` comparison arm needs none of them.

**Tags.** **(soundness)** = a TCB/correctness commitment that must never
regress: the F4 flip (AC-D2), the zero-delta laws (AC-D3), the TCB removal
(AC-G), the `leq_int` independent oracle (AC-L), the `isScalar` Ω-encoding +
computed extraction proof (Char pins 1/2), the surrogate/OOR rejection (AC-C3).
**(oracle)** = a value confirmed against the reference interpreter once
available; grounded meanwhile against `18a` + `35` + first principles.
**(hard-AC)** = a build-gate obligation the merge Decision verifies structurally
(AC-G removal, the AC-L differential independence, the Ω-encoding grep, the
computed-proof grep) rather than a value the interpreter emits.

## AC-G — TCB shrinks by REMOVAL, not shadowing  (soundness, hard-AC)

### surface/numbers/demote-removes-decimal-char-primitives  (soundness, hard-AC)
- spec: `18a §5.6.1(1)`/`§5.9.1(1)` (removal-not-shadowing), `18a §5.1`
  (type-level TCB removal), `18a §5.2.2(3)` (`leq_int` arm
  `trusted_base`-neutral), `docs/program/wp/decimal-char-demote.md` AC-G
- given: the post-demote elaborator/interp source.
- expect (producer-grep, structural): **gone** — `reg_ty!("Decimal")`, the
  `reg_binop!`/`reg_cmpop!` for `add_decimal`/`sub_decimal`/`mul_decimal`/
  `eq_decimal` (`numbers.rs`), the `("add_decimal", …)`/`("mul_decimal", …)`/
  `("eq_decimal", …)` arms + the `DecimalVal` variant's op handling
  (`eval.rs::prim_reduce`), and `reg_ty!("Char")`. **Present** — the derived
  `Decimal = (coeff:Int, exp:Int)` + `Char = {c:Int | isScalar c}` defs, reusing
  F1's exact-`Int` (`num_bigint`-backed) reduction. **`trusted_base`-neutral** —
  the newly-wired `leq_int` `prim_reduce` arm adds **no** `reg_*`/`declare_*`
  (already registered at `numbers.rs:233`). **No** new kernel flag / `Decl`
  variant (`git diff --stat crates/ken-kernel/` empty).
- why: a demote is a **TCB removal** only if the primitive is *deleted*, not
  shadowed by a derived def while the registration lingers. The failure mode is
  **surface growth**: a still-registered `Decimal` primitive **plus** a derived
  `Decimal` def = two things where there was one, `trusted_base()` unchanged or
  larger. The structural grep is the sole net (no value witnesses "a primitive
  is absent"); [[abstraction-visibility-feature-soundness-gate]]. The `leq_int`
  arm is outer-ring (tested-not-trusted, netted by AC-L), not `trusted_base`.

## AC-L — pulled-up `leq_int` arm + independent oracle  (soundness, hard-AC)

### surface/numbers/leq-int-bignum-differential-oracle  (soundness, hard-AC)
- spec: `18a §5.2.2` (the `leq_int` contract — (1) bignum-correct order, (2) `<`
  derived, (4) independent oracle), `18a §5.4` (the `leq_int` row, now BUILT),
  `18a §3` (differential-oracle discipline),
  `docs/program/wp/decimal-char-demote.md` (ruling (A))
- given: `leq_int a b` across an **independent** golden vector set — operands
  built via distinct paths (`Shl`/`Sub`/`Neg`, never the comparison itself): the
  2⁶³ boundary (`i64::MAX` vs `i64::MAX + 1`), the 2¹²⁷ boundary (`2¹²⁷ − 1` vs
  `2¹²⁷`), mixed sign (`-5 ≤ 3`, `3 ≤ -5`, `-5 ≤ -3`), and the equal boundary
  (`5 ≤ 5`).
- expect: `leq_int` **reduces-to** the Bool verdict hand-determined by the total
  order on ℤ — `true, true, true, false, true, true` respectively — each
  expected value **hand-authored from the order**, **never** computed by calling
  `num_bigint`'s `Ord`/`<=` (the green-vs-green trap: production crate as its
  own oracle). Derived `lt`/`min` route through this: canonically
  `lt a b ⇒ ¬ (leq b a)` (Steward's locked minimal form — pure `leq`, no `eq`;
  equivalent to `leq a b ∧ ¬ eq a b` on a total order, build's choice per
  Architect), so `lt 5 5 ⇒ false`, `lt (i64::MAX) (i64::MAX+1) ⇒ true` — **no
  `lt_int` primitive** (grep: `lt_int` unregistered; `<` is the derived
  composite).
- why: the `leq_int` reduce arm is the ordering **prerequisite** ruling (A)
  pulled into this tranche; it is a native-tier reduction with no kernel
  backstop, so an **independent** differential is its sole net — exactly the F1
  AC2 discipline (`seed-f1-bignum-int.md`) extended from `eq_int` to comparison.
  A differential that oracles against `num_bigint`'s own `Ord` is circular and
  green-vs-green. Verify the derived-`lt` composition (not a hidden primitive)
  by grepping the producer.

## AC-D1/D2 — Decimal derivation is exact; the F4 flip  (soundness)

### surface/numbers/decimal-mul-exact-flips-vs-saturating  (soundness)
- spec: `18a §5.6.1(2)` (exact arithmetic; `mul` ordering-free), `35 §2.3`,
  `docs/program/wp/decimal-char-demote.md` AC-D1/D2
- given: the **derived** `mul` applied to `2.0d`-shaped operands with a
  coefficient product that overflows `i64` — e.g.
  `Decimal(coeff = 10¹⁰, exp = 0) × Decimal(coeff = 10¹⁰, exp = 0)` (coeff
  product `10²⁰ > i64::MAX ≈ 9.22×10¹⁸`).
- expect: **reduces-to** the exact `Decimal(coeff = 10²⁰, exp = 0)` — the
  derived `mul` runs bignum-`mul` on the `Int` coefficients, **no saturation**.
  Under the *old* native `mul_decimal` (`coeff = ca.saturating_mul(cb)`,
  `eval.rs`) the product would **saturate to `i64::MAX`**
  (`9223372036854775807`) — a wrong value. The case **flips** on the demote.
  (Ordering-free — `mul` = `(ca·cb, ea+eb)` — so this vector reduced even under
  the disproven premise.)
- why: AC-D2's value-flip — drives the **real derived producer** (bignum `mul`
  on `Int` coeffs), and the exact result is the discriminator against the
  saturating stub. Not hand-fed: the operands are applied to the derived `mul`,
  not the expected `Decimal` constructed and a consumer tested.

### surface/numbers/decimal-eq-distinct-flips-vs-false-true  (soundness)
- spec: `18a §5.6.1(2)` (F4 discriminating closure — both halves), `18a §4` F4,
  `35 §2.3`, `docs/program/wp/decimal-char-demote.md` AC-D2 (F4 closure)
- given: two **distinct** decimals `a = Decimal(coeff = i64::MAX, exp = 0)` and
  `b = Decimal(coeff = i64::MAX, exp = 1)` — i.e. `b = 10 × a` (exactly
  `92233720368547758070` vs `9223372036854775807`), applied to the derived `eq`.
- expect: the derived `eq a b` **reduces-to `false`** (they are distinct — `b`
  is ten times `a`), decided by exponent alignment (**via the pulled-up
  `leq_int`** — min-exponent direction) then bignum compare. Under the *old*
  `decimal_eq` (`eval.rs`), aligning `b` scales `cb ×ₛₐₜ 10` which **saturates
  to `i64::MAX`**, so `ca == cb.saturating_mul(10)` compares
  `i64::MAX == i64::MAX` and returns a **false `True`** — the exact **F4**
  soundness hole (two distinct decimals comparing equal). The case **flips**.
- why: the sharpest AC-D2 witness — this is the **F4 closure**. The saturating
  `decimal_eq` returning `True` on `a ≠ b` is a **wrong Bool value in the
  tested-not-trusted `ken-interp` ring** — **not** an inhabitable false kernel
  proof: `Eq Decimal` is kernel-neutral and `eq_decimal : … → Bool` has no
  `eq → Eq` bridge, so no `refl : Eq Decimal a b` for `a ≠ b` is inhabitable
  (the "refl-inhabits" reading is **over-classified**, §4 F4). The demote
  removes the wrong-value path **by construction**: no trusted `eq_decimal`
  remains; equality is structural bignum compare over aligned `(coeff, exp)`.
  Verdict-flip (`false` correct vs `True` buggy),
  [[kernel-backed-claim-grep-the-emission-not-the-name]] (grep the trust level,
  not the name). Reduces **only** under ruling (A) — the alignment path is
  inherently ordering (different exponents).

## AC-D3 — `Num`/`DecEq Decimal` laws are zero-delta  (soundness)

### surface/numbers/decimal-eq-law-zero-delta-not-postulate  (soundness)
- spec: `18a §5.6.1(4)` (zero-delta structural laws),
  `docs/program/wp/decimal-char-demote.md` AC-D3
- given: a `DecEq Decimal` law — e.g. `eq`-reflexivity
  `(d : Decimal) → eq d d ≡ true` (same exponent ⇒ `ca == ca`, no alignment), or
  `+`-comm — established over the derived `(coeff:Int, exp:Int)` rep.
- expect: the law is a **real, kernel-re-checked, zero-delta proof** (the
  discriminating instance's `trusted_base()` delta is **empty** — no trusted
  `eq_decimal`, no `declare_postulate`), **not** postulate-only. The case
  **fails against a postulate stub** (a law asserted as an `Axiom` over an
  opaque `Decimal` — the pre-demote posture — has a non-empty delta and is not a
  real proof).
- why: the demote's payoff — equality is **structural** over the derived rep, so
  `Num`/`DecEq Decimal` laws that were postulate-only (opaque `Decimal` +
  trusted `eq_decimal`) become zero-delta provable. The discriminating flip is
  real-proof (empty delta) vs postulate (non-empty delta) —
  [[lawful-class-instances-must-carry-law-proofs]] (the discriminating test must
  FAIL against a law-less/postulated instance). The net is the delta, so this
  case held regardless of the ordering fork.

## AC-C1/C2/C3 — Char refinement + derived ops (eq + Ord) + the surrogate flip

### surface/numbers/char-is-isscalar-refinement  (soundness)
- spec: `18a §5.9.1(1)`/`(2)` (refinement + `isScalar` encoding), `18a §5.1`,
  `docs/program/wp/decimal-char-demote.md` AC-C1
- given: the `Char` type definition post-demote.
- expect: `Char = { c : Int | isScalar c }` with
  `isScalar c := IsTrue (inRangeBool c)` (the Ω-encoding pin below), **not** an
  opaque primitive type (`reg_ty!("Char")` gone, AC-G) and **not**
  `List`/`u32`-carrier.
- why: the refinement supplies the free projection `proj : Char → Int` and the
  decidable intro — the two things an opaque `Char` lacked, which is exactly why
  the ops could not derive before (`18a §5.9`). AC-C1 is the type-level demote.

### surface/numbers/int-to-char-rejects-surrogate-and-oor  (soundness)
- spec: `18a §5.9.1(3)` (`Int.toChar` face-(c), AC-C3), `35 §2.4`,
  `docs/program/wp/decimal-char-demote.md` AC-C3
- given: `Int.toChar` applied to `0xD800` (a surrogate), `0x110000`
  (out-of-range, `> 0x10FFFF`), and `0x41` (valid, `'A'`).
- expect: `Int.toChar 0xD800 ⇒ None`, `Int.toChar 0x110000 ⇒ None`,
  `Int.toChar 0x41 ⇒ Some 'A'` — the refinement-intro is **face-(c)** (`None`
  out of the scalar range, never a silent `Some`). The `None` results **reduce**
  (the decidable `inRangeBool` closed-interval `leq_int` check fires and
  rejects, under ruling (A)) — concretely for `0xD800`:
  `leq 0xD800 0xD7FF ⇒ false`, `leq 0xE000 0xD800 ⇒ false` →
  `or_bool false false ⇒ false` → `IsTrue false ≡ Bottom` → **rejected, and it
  *reduces*** (not a stuck neutral). The **non-degenerate pair** (surrogate/OOR
  reject *while* valid accept) **flips against a stub `isScalar := true`**
  (which would give `Some` for `0xD800`/`0x110000` too).
- why: AC-C3 — the refinement obligation must **actually reduce** (the decidable
  `inRangeBool` check fires and rejects), not name-match. A single valid-scalar
  `Some` case is green-vs-green under `isScalar := true`; the surrogate + OOR
  reject arms are the net ([[two-arm-producer-needs-a-case-per-arm]] — two
  reject arms, surrogate *and* range, each discriminating). Reduces only because
  ruling (A) pulled `leq_int` up — else these were stuck-neutral, not `None`.

### surface/numbers/char-eq-and-ord-on-projection  (soundness)
- spec: `18a §5.9.1(3)` (derived ops over projection, incl. `Ord Char`),
  `docs/program/wp/decimal-char-demote.md` AC-C2
- given: derived `eq_char a b`, derived `leq_char a b`, and `Char.toInt` on
  `Char` values.
- expect: `eq_char a b ⇒ eq_int (proj a) (proj b)` (`DecEq Char` = decidable
  `Int` equality on the projection); `leq_char a b ⇒ leq_int (proj a) (proj b)`
  (`Ord Char` on the projection, reducing under ruling (A));
  `Char.toInt = proj`. `eq_char 'a' 'a' ⇒ true`, `eq_char 'a' 'b' ⇒ false`;
  `leq_char 'a' 'b' ⇒ true`, `leq_char 'b' 'a' ⇒ false` (the order pair).
- why: AC-C2 — the derived ops route through the projection to F1's `eq_int` and
  the pulled-up `leq_int`, giving zero-delta `DecEq Char` **and** `Ord Char`
  this tranche. `Char.toInt` is the projection itself (no runtime cost). The
  order **pair** (accept `'a' ≤ 'b'` *while* reject `'b' ≤ 'a'`) is the net — a
  single `≤`-accept is green-vs-green under an orientation-flip
  ([[taint-axis-orientation-needs-distinguishing-pair]]).

### surface/numbers/char-ord-laws-carried-not-stubbed  (soundness)
- spec: `18a §5.9.1(3)` (`Ord Char` laws carried via `proj` injectivity),
  `51 §6` (lawful `Ord`), `docs/program/wp/decimal-char-demote.md` AC-C2 (Ord
  Char, ruling (A))
- given: the `Ord Char` instance and its **antisymmetry** law
  `(a b : Char) → leq_char a b → leq_char b a → eq_char a b`.
- expect: the instance **carries a real, derivable law proof** — antisymmetry
  **reduces** from `Int`'s total order **plus `proj` injectivity** (distinct
  scalars ⇒ distinct codepoints, so `leq_int (proj a) (proj b)` +
  `leq_int (proj b) (proj a)` ⇒ `proj a = proj b` ⇒ `a = b`). The
  `trusted_base()` delta is **empty** (no `Axiom`/`declare_postulate`). The case
  **fails against a law-less stub** (an `Ord Char` whose antisymmetry is
  postulated over an opaque carrier — non-empty delta, not a real proof).
- why: Architect's forward pin 1 for the now-in-scope `Ord Char`. Antisymmetry
  is the non-trivial law — it is the one that **needs `proj` injective**, so a
  stub that merely wraps `leq_int` without carrying the injectivity witness
  passes the operation cases (AC-C2) yet cannot discharge antisymmetry. The
  discriminating flip is real-proof (empty delta) vs postulate (non-empty delta)
  — [[lawful-class-instances-must-carry-law-proofs]] (the test must FAIL against
  a law-less instance). Reflexivity/transitivity/totality inherit directly from
  Int's order; antisymmetry is the carried net.

### surface/numbers/char-deceq-collapses-on-codepoint  (soundness, hard-AC)
- spec: `18a §5.9.1(2)` (pin 1, Ω-encoding), `18a §5.9` pin 1, `16 §1.3`,
  `docs/program/wp/decimal-char-demote.md` AC-C1
- given: the `isScalar` definition, and two `Char` values with the **same
  codepoint** but (hypothetically) **distinct scalar proofs**.
- expect (structural + value): (a) **structural (sort-not-token grep,
  Architect's Ω-encoding-lane hazard)** — `isScalar`'s **type** is
  `IsTrue <bool-expr>` (the sub-singleton bridge):
  `isScalar c := IsTrue (inRangeBool c)`, with `inRangeBool c` =
  `(0 ≤? c && c ≤? 0xD7FF) || (0xE000 ≤? c && c ≤? 0x10FFFF)` (value-level
  `&&`/`||`, i.e. `and_bool`/`or_bool`, over `leq_int`). The scalar set is
  **irreducibly two disjoint intervals** (`[0,0xD7FF] ∪ [0xE000,0x10FFFF]`), so
  `inRangeBool` **necessarily** contains **value-level** `or_bool`/`and_bool`/
  `not_bool` composing the `leq_int` results — these **reduce** and are
  **REQUIRED**; grepping them to zero **false-flags the only correct encoding**
  (the 44 §3 over-strip shape —
  [[grounding-a-fabricated-citation-two-failure-modes]]). The **FORBIDDEN** form
  is a raw `∨`/`∃`/multi-ctor `Or`/`Sum` **as the *type* (sort Ω) of
  `isScalar`** — the discriminator is the **sort** of the disjunction
  (Bool-value inside `IsTrue` = fine; Ω-prop as the predicate former = the
  trap), **not** the token. Uses **no `lt_int` primitive** (closed intervals).
  (b) **value** — `DecEq Char` on same-codepoint, distinct-proof `Char`s reduces
  to **equal** (by Ω-PI, since `IsTrue` is a sub-singleton → proof-irrelevant →
  the proofs collapse; routed through `eq_int` on the projection, not through
  `Int.toChar` construction). Under a naive `isScalar := (…) ∨ (…) : Ω` the
  disjunction is the **proof-relevant** `A + B`, the proofs do **not** collapse
  (distinct injection tags), the codepoint-collapse **fails**, and forcing
  `A + B` into Ω re-opens the `Bool → Ω` inconsistency — the case **flips**.
- why: Char pin 1 — the load-bearing `DecEq Char` soundness check. The
  zero-delta `DecEq Char` (Ω-PI codepoint-collapse) holds **only** if `isScalar`
  is actually proof-irrelevant, which the `IsTrue`-reflection guarantees and the
  naive `∨` does not ([[proof-relevant-inductive-cannot-be-declared-at-omega]]).
  The structural grep (`IsTrue`, not `∨`) is the primary net; the value
  assertion is its observable consequence.

### surface/numbers/char-extraction-computes-scalar-proof  (soundness, hard-AC)
- spec: `18a §5.9.1(4)` (pin 2, runtime face), `18a §5.9` pin 2, `37 §2`
  (`String` is NFC UTF-8), `docs/program/wp/decimal-char-demote.md` (Char pin 2)
- given: `String → Char` extraction (`char_at` / `string_to_list_char`) on a
  valid `String`.
- expect (producer-grep, structural): extraction constructs `(c, w)` where the
  `isScalar` witness `w` **reduces** from the `String`'s validity invariant —
  `inRangeBool c` computes to `true` (via the pulled-up `leq_int`), so `w` is
  the canonical `tt` (`IsTrue true ≡ Top`'s inhabitant). The producer
  **discharges** the obligation by reduction; it is **never** a
  `declare_postulate` / `Axiom` / hand-fed `sorry` / `Neutral`-stub asserting a
  scalar proof it did not compute.
- why: Char pin 2 — the **runtime face**
  ([[soundness-AC-static-vs-runtime-face]]). The static face (refinement in the
  type) is cheap; a trusted-not-proved hole hides in extraction *asserting* the
  scalar proof instead of computing it. Ruling (A) is what makes this face
  deliverable — under (B) extraction could only ship by postulating the witness
  (stuck `inRangeBool`), the exact hole this pins. The net greps the producer
  for the obligation **discharge** (a reduced `tt`), not the type signature.
  Sound because a valid-UTF-8 `String` only yields scalars, so `isScalar c`
  always reduces to its canonical inhabitant — **no primitive fabricates a
  non-scalar `Char`**.

## Coverage map (AC → cases)

- **AC-G** (TCB removal, not shadow; `leq_int` arm neutral) —
  `demote-removes-decimal-char-primitives` (hard-AC)
- **AC-L** (pulled-up `leq_int` arm, independent oracle; derived `lt`) —
  `leq-int-bignum-differential-oracle` (hard-AC)
- **AC-D1/D2** (exact derivation; F4 flip) —
  `decimal-mul-exact-flips-vs-saturating`,
  `decimal-eq-distinct-flips-vs-false-true` (F4 closure)
- **AC-D3** (zero-delta laws) — `decimal-eq-law-zero-delta-not-postulate`
- **AC-C1** (`Char` refinement) — `char-is-isscalar-refinement`
- **AC-C2** (derived Char eq + Ord over projection) —
  `char-eq-and-ord-on-projection`; `char-ord-laws-carried-not-stubbed` (Ord Char
  laws, Architect forward-pin 1)
- **AC-C3** (surrogate/OOR reject, flips vs `isScalar:=true`) —
  `int-to-char-rejects-surrogate-and-oor`
- **Char pin 1** (Ω-encoding → `DecEq Char` codepoint-collapse) —
  `char-deceq-collapses-on-codepoint` (hard-AC)
- **Char pin 2** (extraction computes the proof) —
  `char-extraction-computes-scalar-proof` (hard-AC)

## Cross-case sweep (`18a §4`)

- **No `saturating_*` / i64-coeff remains (Decimal).** The structural closing
  net (`18a §4`): a whole-class `prim_reduce`/`numbers.rs` producer-grep
  confirms the `*_decimal` arms are gone and **no `saturating_*` remains** on
  any `Decimal` path; the derived coeff arithmetic routes through F1's
  exact-`Int`, never a re-introduced i64/i128 intermediate. Every AC-D case
  drives the derived producer; none admits a saturating residue.
- **Ordering is `leq_int`-only; no `lt_int` primitive.** Every ordering use
  (Decimal exponent alignment, Char `inRangeBool`, derived `lt`/`min`) routes
  through the single pulled-up `leq_int` reduce arm; `lt`/`min`/`|·|` are
  derived composites (`leq`+`sub`+`eq`). A grep confirms `lt_int` stays
  **unregistered** and no new comparison primitive is added — the primitive set
  stays flat (AC-G neutrality). AC-L nets `leq_int` against an independent
  oracle.
- **`Char` refinement class agrees.** Every `Char`-forming path (`Int.toChar`,
  `String → Char` extraction, literals) yields a `{c:Int | isScalar c}` whose
  witness is a **computed** `IsTrue` inhabitant — never a postulated/fabricated
  scalar proof; `isScalar` is `IsTrue (<computed Bool>)` uniformly, never a `∨`
  at Ω. No path admits a non-scalar `Char`.
- **TCB strictly shrinks.** Two type-level removals (`Decimal` + `Char`) + four
  op removals (`add/sub/mul/eq_decimal`); the `leq_int` arm is neutral
  (already-registered); zero additions to `trusted_base()`, zero new kernel
  flags/`Decl` variants. The net delta is **negative** (a real TCB removal), the
  ADR 0009 adversarial-burden migration.

## Deferred to later tranches (not covered here — honesty about the boundary)

Per the brief's OUT scope (as corrected by ruling (A)) — flagged so no case
over-reaches:

- **`checked_*` / `saturating_*` / `neg_int` demotes** — gate on the complete
  `IntN↔Int` conversion floor (`18a §5.3`/`§5.7`) → the **conversions** tranche.
  The `leq_int` comparison arm pulled up here needs none of them.
- **`Float.toDecimal` / `Decimal.toFloat`** — conversions tranche;
  `Decimal.toFloat` stays NATIVE (the correct-rounding cliff, `18a §5.7`).

Note: `Ord Char` / `leq_char` is **no longer deferred** — ruling (A)'s `leq_int`
arm makes `leq_char ⇒ leq_int ∘ proj` reduce this tranche (AC-C2). The brief's
earlier "Ord Char rides F5" line is superseded.
