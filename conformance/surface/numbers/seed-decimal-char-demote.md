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
better**: the derived **ops** are **zero-delta computational** (no trusted
`*_decimal`, F4 removed) and the class **laws** become **zero-NEW-delta** —
provable adding no postulate beyond `Int`'s existing audited-delta `Axiom`s (the
law-carrying instances re-home to the lawful-classes lane, `18a §5.6.1(4)`/
`§5.9.1(3)`; see the deferred section); and the F4 wrong-value hole (the
saturating false-`True` `eq_decimal`) **vanishes by construction**. Trust-level
precision (§4 F4): that false-`True` is a **wrong Bool value in the
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
TCB actually shrinks (removal, not shadowing), the derived ops are zero-delta
computational (the laws are zero-NEW-delta over `Int`; the law-carrying
instances re-home to the lawful-classes lane), and the `Char` refinement is
sound (Ω-encoding + computed extraction proof). The `leq_int` reduce-arm oracle
(**AC-L**) homes here (this WP delivers it), extending F1's `eq_int` oracle
family (`seed-f1-bignum-int.md` AC2) to comparison. Distinct properties, one
home each; a one-line cross-reference is added to `seed-numbers.md` AC6 +
`char-excludes-surrogates`.

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
  *distinct* scalar proofs are **equal by Ω-PI** (a **zero-delta** kernel fact)
  — which holds **only** if `isScalar` is actually proof-irrelevant. (The
  `eq_char` **op** reduces via `eq_int`, zero-delta computational; the
  `DecEq Char` **law-carrying instance** — decidability sound/complete over the
  opaque `Int` projection — is **zero-NEW-delta**, re-homing to the
  lawful-classes lane, like `Ord Char`.) See
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
regress: the F4 flip (AC-D2), the TCB removal (AC-G), the `leq_int` independent
oracle (AC-L), the `isScalar` Ω-encoding + computed extraction proof (Char pins
1/2), the surrogate/OOR rejection (AC-C3). The zero-NEW-delta **law** cases
(`Ord Char` + `Num`/`DecEq Decimal`) re-home to the lawful-classes lane
(deferred section); Char pin 2's runtime face rides the extraction feature.
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

## AC-D3 — `Num`/`DecEq Decimal` laws  (RE-HOMED → lawful-classes lane)

The `Num`/`DecEq Decimal` **law-carrying instance** is **not** delivered by this
DEMOTE — it re-homes to the lawful-classes lane next to its `Ord Int`/`Num Int`
twin (Architect + Steward ruling; carrier-axis correction). The demote's Decimal
deliverable is the **computational** side (derived ops + trusted-primitive
removal — AC-G, AC-D1/D2), which is genuinely zero-delta. The **laws** are
**zero-NEW-delta**, not zero-delta: `Decimal = Prod Int Int` is inductive, so
`DecEq`/`Num Decimal` reflexivity/comm are a **real structural proof over the
pair** that bottoms out at the `DecEq Int`/`Num Int` audited-delta `Axiom`
**leaves** (`18a §5.2`/`§5.4`) — adding no NEW postulate, but not `Axiom`-free.
The corrected law case (honesty discriminator, below) is a forward conformance
obligation on that lawful-classes-lane WP — see the deferred section.

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
- why: AC-C2 — the derived `eq_char`/`leq_char` **ops** route through the
  projection to F1's `eq_int` and the pulled-up `leq_int`, **reducing this
  tranche** (zero-delta computational). `Char.toInt` is the projection itself
  (no runtime cost). The order **pair** (accept `'a' ≤ 'b'` *while* reject
  `'b' ≤ 'a'`) is the net — a single `≤`-accept is green-vs-green under an
  orientation-flip ([[taint-axis-orientation-needs-distinguishing-pair]]). This
  case pins the **ops**; the `DecEq`/`Ord Char` **law-carrying instances** (laws
  zero-NEW-delta over the opaque `Int` projection) re-home to the lawful-classes
  lane — see the deferred section.

### `Ord Char` laws  (RE-HOMED → lawful-classes lane)

The `Ord Char` **law-carrying instance** is **not** delivered by this DEMOTE —
it re-homes to the lawful-classes lane next to `Ord Int` (Architect + Steward
ruling). **Correction (carrier-axis):** the original "antisymmetry is a real
zero-delta proof via `proj` injectivity, never `Axiom`" was **wrong** —
`Char ≡ Int` under refinement erasure (`21 §6.3`), so `proj` is the identity and
`Ord Char`'s laws **are** `Ord Int`'s laws, which are honest visible `Axiom`s
(`Int` is opaque, no induction principle to case-split — `lawful_classes.ken`).
Antisymmetry is **zero-NEW-delta by transport** (the instance's `antisym` field
references `Ord Int`'s existing `Axiom`, adding no new `Decl::Opaque`), NOT a
fresh proof. The corrected discriminator is **HONESTY, not zero-delta**: the
instance carries an **honest-visible** law (a `declare_def` that reduces on an
inductive carrier, OR a visible `Axiom`/transport on an opaque one) and flips
against a **deceptive empty/false stub** (claims proved, is empty) — **never**
against an honest visible `Axiom`. Forward conformance obligation on the
lawful-classes-lane WP — see the deferred section.

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
  **codepoint-collapse** (`Char` equality reduces to codepoint equality — a
  **zero-delta** Ω-PI fact) holds **only** if `isScalar` is actually
  proof-irrelevant, which the `IsTrue`-reflection guarantees and the naive `∨`
  does not ([[proof-relevant-inductive-cannot-be-declared-at-omega]]). The
  structural grep (`IsTrue`, not `∨`) is the primary net; the collapse is its
  observable consequence. (This pins the collapse fact + the `isScalar` shape,
  **not** the `DecEq Char` law-instance, whose decidability laws over the opaque
  `Int` projection are zero-NEW-delta and re-home to the lawful-classes lane.)

### surface/numbers/char-extraction-computes-scalar-proof  (soundness, deferred)
- spec: `18a §5.9.1(4)` (pin 2, runtime face), `18a §5.9` pin 2, `37 §2`
  (`String` is NFC UTF-8), `docs/program/wp/decimal-char-demote.md` (Char pin 2)
- status: **RUNTIME FACE DEFERRED** — `char_at` doesn't exist and
  `string_to_list_char` is a pre-existing `Neutral` stub (`eval.rs:870`); real
  UTF-8 `String → Char` extraction is a **new feature**, not a wire-up, out of
  this demote's scope. **Safe to defer:** while the stub is stuck, **no `Char`
  is ever constructed from a `String`** → no un-witnessed `Char` → **no hole**
  (Architect-ruled). The **static face** (the requirement below + no-postulate)
  stands now; the runtime face (extraction *computes* the `tt`) rides the
  extraction-feature WP as a forward obligation (deferred section), gated then.
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
- **AC-C1** (`Char` refinement) — `char-is-isscalar-refinement`
- **AC-C2** (derived Char eq + Ord **ops** over projection) —
  `char-eq-and-ord-on-projection`
- **AC-C3** (surrogate/OOR reject, flips vs `isScalar:=true`) —
  `int-to-char-rejects-surrogate-and-oor`
- **Char pin 1** (Ω-encoding → codepoint-collapse) —
  `char-deceq-collapses-on-codepoint` (hard-AC)
- **Char pin 2** (extraction computes the proof) —
  `char-extraction-computes-scalar-proof` (hard-AC, **runtime face deferred**)
- **RE-HOMED (lawful-classes lane, forward obligations):** the `Ord Char` +
  `Num`/`DecEq Decimal` **law-carrying instance** cases (were AC-D3 +
  `char-ord-laws-carried-not-stubbed`) — corrected to the honesty discriminator
  (zero-NEW-delta, not zero-delta); see the deferred section.

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

**Re-homed / deferred forward conformance obligations** (each gated on a
distinct future WP; flagged so none is silently dropped):

- **`Ord Char` + `Num`/`DecEq Decimal` law-carrying instances → the
  lawful-classes lane WP** (Steward frames post-merge). The corrected law cases
  pin the **HONESTY** discriminator (not zero-delta): the instance carries an
  **honest-visible** law — for `Char`, `antisym` is **zero-NEW-delta by
  transport** (references `Ord Int`'s visible `Axiom`, no new `Decl::Opaque`,
  since `Char ≡ Int` under erasure); for `Decimal`, a **real structural proof
  over `Prod Int Int`** bottoming out at the `DecEq Int`/`Num Int` audited-delta
  `Axiom` leaves — and **flips against a deceptive empty/false stub** (claims
  proved, is empty), **never** against an honest visible `Axiom`. Homed next to
  their `Int` twins (subsume-don't-proliferate); the demote here ships only the
  computational ops + primitive removal.
- **pin-2 `String → Char` extraction computes-the-witness (runtime face) → the
  extraction-feature WP.** `char_at`/`string_to_list_char` are unbuilt (a
  `Neutral` stub — no `Char` from a `String` ⇒ no hole); when real UTF-8
  extraction lands, verify it **reduces** the canonical `tt` scalar witness,
  never postulates it.
- **Unbounded-Δexp general Decimal `align` → the Int-recursion WP.** The demote
  ships a fixed-unrolled **exact-or-stuck** `align` (a general `10^|Δexp|`
  recursion over `Int` fails SCT — no structural descent on opaque `Int`); a
  large-Δexp case (exact reduction, no clamp/saturate) rides that WP.

Note: `Ord Char` / `leq_char` — the **op** is **no longer deferred** (ruling
(A)'s `leq_int` arm makes `leq_char ⇒ leq_int ∘ proj` reduce this tranche,
AC-C2; the brief's earlier "Ord Char rides F5" line is superseded); the
**law-carrying instance** re-homes to the lawful-classes lane (above).
