# Surface conformance — numbers & primitive scalars (L1 seed)

Format: `../../README.md`. These pin the **L1 numeric model**
(`spec/30-surface/35-numbers.md`, impl-ready) to executable, black-box cases:
the arbitrary-precision `Int`, the fixed-width **obligation-generating**
overflow discipline, `Decimal` exactness, honestly-named `Float`, typed literal
defaulting, explicit conversions, and the kernel-primitive / prelude-law
boundary. Every case is a **verdict- or structure-flip** (right=accept /
wrong=reject, or a structural output the targeted bug would change) per the
conformance discipline (`35 §7`).

## Reading disciplines (what these cases pin, and how they flip)

- **The observable is the *structural* output, never "it compiles."** For the
  overflow obligation (AC3), the net is that the obligation is *emitted* and
  appears in the diagnostic / obligation set — driven by a **real** fixed-width
  `+`, not a synthetic flag. This is the **untrusted-layer net** (`22` preamble,
  `35 §3.2`): V2 is untrusted, the kernel only ever re-checks the certificates
  V2 *chose to emit*, so a **missed** obligation is **not backstopped**. A case
  that only asserts "compiles" would pass vacuously against a V2 that silently
  dropped the overflow burden.
- **Discriminating witnesses must be off the bug's fixed points.** AC1's witness
  is deliberately **off-grid**: the bare literal `100000000000000000000` (10²⁰)
  is *itself* exactly representable in IEEE binary64 (10²⁰ = 5²⁰·2²⁰, and
  5²⁰ < 2⁵³), so asserting *its* stored value alone is **green-vs-green** under
  an f64-carrier bug. The discriminator is `… + 1`: the ULP at 10²⁰ is 2¹⁴, so
  10²⁰+1 rounds to 10²⁰ under f64 — the exact `Int` value `…001` then *differs*
  from the f64 image `…000`. See
  [[discriminating-conformance-verdict-must-flip]].
- **The same operation under two contexts is the net for a classification map.**
  AC3 (`+ : Int32` undischarged→degrade vs discharged→total) and AC4 (bare `+`
  vs explicit `+%`, same overflowing operands) are **non-degenerate pairs on a
  shared op shape**: the *context* (the in-scope range refinements; the op class
  chosen) flips the verdict. A single positive case is green-vs-green under the
  realistic bug (a silent-wrap default arm; a missed obligation). This is the
  sealed-dispatch / projection net B1 and Sec1ct also use.
- **Locked vs deferred spelling.** `35` locks the **concepts** and the
  **value-sets**: the literal default *table* (form→type), the four-way op-class
  *partition* (obligation / wrapping / checked / saturating) with **no
  silent-wrap default**, the lowerings to `41 §5`, and the conversion *surface*
  as a closed named set. The **literal token spellings** of the explicit ops
  (`wrapping_add` / `+%` / `Wrapping[T]`, `checked_add`, `saturating_add`), the
  conversion-function names (`Int.toInt64`), and the exact emitted-obligation
  diagnostic field names are surface syntax not yet frozen by a harness — pinned
  to the spec's reference spelling and tagged **(oracle)**; the *behavior* each
  case nets is spelling-agnostic. See
  [[conformance-assert-at-locked-granularity]].
- **Lowerings are pinned to the landed `41 §5`**, not paraphrased: `Int` =
  inline `i64` fast path → heap bignum (tag `0x01`); fixed-width = inline
  machine immediates; `Decimal` = inline `{i64 coeff, i32 exp}` → heap (`0x0A`);
  `Float`→`f64`, `Float32`→`f32`, `Bool`→`i1`, `Char`→`u32` (surrogates
  excluded). `41` is the representation authority.

**Tags.** **(soundness)** here = a correctness / TCB commitment that must never
regress: the f64 non-reproduction (exactness), the **no-silent-wrap** seal, the
**no-missed-obligation** untrusted-layer net, and the **no-new-kernel-rules**
boundary (laws stay prelude propositions). **(oracle)** = a surface token /
diagnostic spelling pinned to the spec's reference form, to be confirmed against
Ken's reference interpreter / the harness once fixed; the netted behavior does
not depend on the spelling.

## AC1 — `Int` exactness above 2⁵³

### surface/numbers/int-arbitrary-precision-above-2^53  (soundness)
- spec: `35 §1`, `§2.1`, `41 §5` ("Small `Int`")
- given: `100000000000000000000 + 1 : Int`  (10²⁰ + 1, an off-grid witness)
- expect: **reduces-to** the exact value `100000000000000000001` — stored as a
  minimal-limb heap bignum (tag `0x01`), not an `f64`-rounded double.
- why: arbitrary-precision `Int` is the prototype's real numeric
  non-reproduction. The `+ 1` is load-bearing: 10²⁰ alone is f64-exact, so a
  bare-literal value assertion would not flip under an f64-carrier bug; 10²⁰+1
  rounds to 10²⁰ under f64 (ULP 2¹⁴), so the exact result `…001` is the
  discriminator. Structural value assertion, not a type check (`35 §7` AC1). The
  **i128-ceiling** corpus (crossing 2¹²⁷/2¹²⁸, where this f64 witness — at
  2⁶⁶ — cannot reach) is a **distinct** non-reproduction, homed in
  `seed-f1-bignum-int.md` (WP F1, `18a §5.2.1`); this case stays authoritative
  for the f64-carrier property.

## AC2 — literal types are distinct (no universal f64)

### surface/numbers/literal-defaulting-distinct-types
- spec: `35 §4.1`, `31 §3` (lexical literal forms)
- given: the unconstrained literals `2`, `2.0`, `2.0d`
- expect: `2 : Int`, `2.0 : Float`, `2.0d : Decimal` — **three distinct types**
  by the built-in form-keyed default table (bare integer→`Int`, `.`-no-`d`→
  `Float`, `d`-suffix→`Decimal`). A program relying on `2 ≡ 2.0` (e.g. using `2`
  where a `Float` is required, with no explicit conversion) **rejects**
  (well-typed vs ill-typed flip).
- why: the f64-only ontology is not reproduced even at the lexer (`31 §3`
  critical rule: a bare integer literal is `Int`, never an f64). The default
  table is a **standalone elaborator rule** (`39 §2.5`), not instance search.

### surface/numbers/expected-type-overrides-default
- spec: `35 §4.1` (expected-type override)
- given: `view f (x : Int64) = x + 1` — the literal `1` in an `Int64`-typed
  position
- expect: `1` elaborates at the **expected type `Int64`**; the default table
  does **not** fire (no `Int`-default, no ambiguity error). The default applies
  **only** when a literal is otherwise unconstrained.
- why: the declared numeric default is the *one* place the elaborator defaults a
  literal without an ambiguity error (`39 §2.5`); a typed position pins the type
  instead. Pairs with the distinct-types case to show defaulting is
  context-sensitive, not a fixed coercion to `Int`.

## AC3 — fixed-width overflow emits a no-overflow obligation

### surface/numbers/fixed-width-overflow-emits-obligation  (soundness)
- spec: `35 §3.2`, `22 §1`, `§2.4`, `43 §2`
- given: a bare `a + b : Int32` on **unconstrained** `a b : Int32`
- expect: V2 **emits** the obligation triple `⟨id, Γ ⊢ φ_no_ovf, provenance⟩`
  with goal `φ_no_ovf ≡ −2147483648 ≤ (a +_ℤ b) ∧ (a +_ℤ b) ≤ 2147483647`
  (the no-overflow predicate in the ℤ domain, `Int32` bounds). Unprovable from
  `Γ` ⇒ an **open typed hole** `?id : φ_no_ovf` (`22 §1`, `24 §2`) — a **marked
  partial point** that degrades to a runtime check (panic / `unknown`, `43 §2`).
  Observe the **emitted obligation** in the diagnostic — not "it compiles".
  *(oracle: the diagnostic field names / hole-id format.)*
- why: the **untrusted-layer net** (`22` ★★ preamble, `35 §3.2`). A *missed*
  obligation is not backstopped by the kernel, so the net must be that the
  obligation is **emitted** by a real `+`, not a synthetic flag. Under a V2 that
  silently skipped the overflow site, the obligation set lacks `φ_no_ovf` and
  this case **flips** (no emitted goal to observe).

### surface/numbers/in-range-overflow-obligation-discharged
- spec: `35 §3.2`, `22 §2.4`, `34 §5` (refinement coercion discharges the goal)
- given: the **same** bare `a + b : Int32` with `a b : {x : Int32 | 0 ≤ x ∧
  x ≤ 1000}`
- expect: `φ_no_ovf` is the **same emitted goal**, now **dischargeable** from
  `Γ` (`a +_ℤ b ≤ 2000 ≤ 2³¹−1` and `≥ 0 ≥ −2³¹`) — the operation is **total and
  safe**, no residual runtime check.
- why: the non-degenerate partner to the undischarged case — **same op shape,
  two `Γ` contexts** flip total↔degrade (the B1 same-input-two-states pattern,
  [[taint-axis-orientation-needs-distinguishing-pair]]). "Checked arithmetic" is
  *subsumed* as the runtime face of an *undischarged* obligation, not a separate
  mode the user selects (`35 §3.2`).

## AC4 — fixed-width overflow never silently wraps

### surface/numbers/bare-overflow-never-silently-wraps  (soundness)
- spec: `35 §3.2` (sealed op-class dispatch, no silent-wrap default)
- given: `(100 : Int8) + (100 : Int8)` — sums to 200 in ℤ, out of `Int8` range
  (`Int8` max 127)
- expect: does **NOT** silently produce the wrapped value `-56` (= 200 − 256).
  The bare `+` emits `φ_no_ovf ≡ −128 ≤ 200 ≤ 127` — **unsatisfiable** here — so
  the use is a marked partial point that obligation-checks / **panics**, never a
  silent wrap.
- why: the load-bearing non-reproduction (silent wrap is a correctness hazard in
  a verified language). The **guard** is the sealed four-way op-class dispatch
  with **no default arm**: the bare-op image is fixed to the obligation class
  (`35 §3.2`, the exhaustive-by-construction seal). Under the exact bug this
  targets — a silent-wrap default arm — this case would produce `-56` and
  **flip**. Guard-gated, not coincidental (absence-gate,
  [[conformance-reconcile-inherits-spec-metatheory-bugs]]).

### surface/numbers/explicit-wrapping-op-is-modular
- spec: `35 §3.2` (the wrapping op class)
- given: `(100 : Int8) +% (100 : Int8)` — the explicit wrapping operator
  *(oracle: `+%` / `wrapping_add` / `Wrapping[Int8]` spelling)*
- expect: **reduces-to** `-56 : Int8` (modular `mod 2⁸`, provably-modular), with
  **no** `φ_no_ovf` obligation (modular semantics is the *intent*).
- why: the partner to the no-silent-wrap case — the **same overflowing
  operands** yield a defined modular result **only** when the wrapping class is
  named explicitly. The pair pins the orientation: modular is reachable *only*
  by naming it; the bare operator never wraps. Wrap is visible and provable, for
  hashing / crypto / checksums (`35 §3.2`).

## AC5 — no implicit numeric coercion

### surface/numbers/no-implicit-cross-type-coercion
- spec: `35 §5`
- given: `(x : Int) + (y : Int64)` with **no** explicit conversion
- expect: **rejects** — a type error; the operands disagree and there is **no
  widening coercion** to make them agree.
- why: the conversion surface is a **closed named set with no implicit-coercion
  arm** (`35 §5`). Pairs with the explicit-conversion case (verdict flip:
  implicit rejects / explicit accepts). The cross-case sweep extends this to the
  whole class.

### surface/numbers/explicit-conversion-is-partial-option
- spec: `35 §5`
- given: `(x : Int).toInt64`  *(oracle: `Int.toInt64` spelling)*
- expect: **accepts** with type `Option Int64` — explicit, **partial** (a wide→
  narrow move may not fit). The widening direction `Int64.toInt : Int64 → Int`
  is **total**; the lossy `Int.toFloat : Int → Float` is total and
  **documented-lossy** in its type.
- why: lossiness/partiality is visible in the type, never silent — `Option` for
  may-fail, documented-lossy for `toFloat` above 2⁵³ (`35 §5`). The explicit
  conversion is the accept-side of the no-implicit-coercion flip.

## AC6 — `Decimal` exact; `Float` honest

### surface/numbers/decimal-exact-while-float-honest
- spec: `35 §2.3`, `§3.3`
- given: `0.1d + 0.2d == 0.3d` (`Decimal`) and `0.1 + 0.2 == 0.3` (`Float`)
- expect: the `Decimal` equality **reduces-to `true`** (exact base-10:
  coefficient is arbitrary-precision, `coeff × 10^exp`); the `Float` equality
  **reduces-to `false`** (IEEE binary64: `0.1 + 0.2` is `0.30000000000000004`,
  not `0.3`). The `Float` analog is honestly **not** asserted equal.
- why: the non-degenerate **honesty pair** — same arithmetic shape, `Decimal`
  accepts-equal **while** `Float` rejects-equal — proving `Decimal` is **not**
  an `f64` alias (`35 §2.3`) and `Float ==` is the IEEE minefield Ken does not
  paper over (`35 §3.3`). A `Decimal`-as-f64 bug collapses both to the same
  rounded result and **flips** the `Decimal` side.
- note: the **DEMOTE mechanism** behind this exactness — derived
  `(coeff : Int, exp : Int)` over F1 bignum, the F4 saturating-`mul`/`eq`
  closure, and the zero-delta `Num`/`DecEq Decimal` laws — is pinned in
  `seed-decimal-char-demote.md` (`18a §5.6.1`). This case owns the *observable*;
  that seed owns the *TCB-removal mechanism*.

## §3.1 — `Int` division by zero is an obligation, not a trap

### surface/numbers/int-div-by-zero-emits-obligation
- spec: `35 §3.1`, `22 §2.4`, `34 §5`
- given: `a / b : Int` with possibly-zero `b : Int`
- expect: V2 **emits** a non-zero side-condition obligation `b ≠ 0` at the
  operation site (`22 §2.4`); a raw `/` on a possibly-zero divisor is a marked
  partial point, **not** a silent trap. Supplying `b : {d : Int | d ≠ 0}`
  **discharges** it (refinement coercion `{x:A|φ} ≤ A`, `34 §5`) ⇒ `div` total.
- why: `Int` is total for `+`/`-`/`*` (arbitrary precision), and division/modulo
  by zero is its **one** partial point — surfaced as an obligation, parallel to
  the fixed-width overflow mechanism. Mechanism-consistency with AC3 (same
  obligation triple, same typed-hole/postulate degrade).

## §6 — kernel-primitive / prelude-law boundary (no kernel enlargement)

### surface/numbers/literal-reduces-in-kernel
- spec: `35 §6.1`, `14 §5`
- given: `2 + 3 : Int`
- expect: **reduces-to** `5` *in the kernel's evaluator* — the registered,
  audited `prim` reduction on literals (`14 §5`). On a non-literal / stuck
  argument the primitive op is a **neutral** term (no reduction fires).
- why: numeric ops are kernel **primitives** with registered reductions, so
  `2 + 3 ≡ 5 : Int` is **definitional** (holds by computation). The trusted-
  primitive obligation (`35 §6.2`, `35 §7`): the registered reduction must match
  the reference model — a wrong primitive reduction is a soundness bug (`14 §5`,
  `18 §5`).

### surface/numbers/algebraic-law-is-proposition-not-reduction  (soundness)
- spec: `35 §6.2`, `14 §5`
- given: abstract `a b : Int`; the conversion query `a + b ≟ b + a` and the
  prelude law `add_comm : (a b : Int) → a + b == b + a`
- expect: `a + b` and `b + a` are **NOT definitionally equal** — `a + b` is a
  **neutral** term on abstract operands (no reduction fires), so kernel
  conversion **rejects** `a + b ≡ b + a`. Commutativity holds only as the
  **propositional** equality `add_comm`, discharged by a proof term (a prelude
  proposition, proved against the reference model or in the audited
  primitive-law interface).
- why: **no new kernel rules** (`35 §6.1`, `14 §5`). The **guard**: conversion
  has **no** registered commutativity reduction; the definitional/non-
  definitional boundary is the TCB line (`35 §6.2`). Under the exact bug this
  targets — registering an algebraic law as a kernel reduction (or making
  conversion accept it) — `a + b ≡ b + a` would be **accepted** and this case
  **flips**. A wrong-but-convenient reduction here is a soundness/TCB-growth
  defect.

## §2.4 — `Char` is a Unicode scalar value (surrogates excluded)

### surface/numbers/char-excludes-surrogates
- spec: `35 §2.4`, `41 §5`
- given: the char literals `'a'` (U+0061) and a surrogate code point
  U+D800  *(oracle: the surrogate-literal spelling, e.g. `'\u{D800}'`)*
- expect: `'a' : Char` **accepts** (a valid Unicode scalar value); the surrogate
  U+D800 **rejects** — `Char`'s valid range is U+0000–U+10FFFF **excluding** the
  surrogate block U+D800–U+DFFF.
- why: `Char` is a **refinement** on the `u32` carrier (`35 §2.4`, `41 §5`): not
  all `u32` are `Char`. Verdict flips: valid scalar accepts, surrogate rejects —
  the non-reproduction of the "char = any 16/32-bit code unit" model.
- note: post-**DEMOTE** the carrier is realized as `{ c : Int | isScalar c }`
  (`18a §5.9`) — the Int-refinement, superseding the `u32`-carrier framing for
  the *mechanism*; the observable surrogate-exclusion above is unchanged. The
  `isScalar := IsTrue(inRangeBool)` Ω-encoding, the *reducing* surrogate/OOR
  rejection (`Int.toChar 0xD800 ⇒ None`), and the derived
  `Ord Char`/`DecEq Char` **ops** are pinned in `seed-decimal-char-demote.md`
  (`18a §5.9.1`); their **law-carrying instances** re-home to the lawful-classes
  lane (zero-NEW-delta over the opaque `Int` projection).

## Coverage map (AC → cases)

- **AC1** (`Int` exact above 2⁵³) — `int-arbitrary-precision-above-2^53`
- **AC2** (literal types distinct; expected-type override) —
  `literal-defaulting-distinct-types`, `expected-type-overrides-default`
- **AC3** (overflow obligation emitted, structural) —
  `fixed-width-overflow-emits-obligation`,
  `in-range-overflow-obligation-discharged`
- **AC4** (no silent wrap; explicit wrap modular) —
  `bare-overflow-never-silently-wraps`, `explicit-wrapping-op-is-modular`
- **AC5** (no implicit coercion; explicit is `Option`) —
  `no-implicit-cross-type-coercion`, `explicit-conversion-is-partial-option`
- **AC6** (`Decimal` exact, `Float` honest) —
  `decimal-exact-while-float-honest`
- **§3.1** (`Int` div-by-zero obligation) — `int-div-by-zero-emits-obligation`
- **§6.1/§6.2** (primitive reduction vs prelude law) —
  `literal-reduces-in-kernel`,
  `algebraic-law-is-proposition-not-reduction`
- **§2.4** (`Char` surrogate exclusion) — `char-excludes-surrogates`

## Cross-case sweep (`35 §7`)

- **Overflow-obligation class agrees.** Every bare fixed-width `+`/`-`/`*` (over
  `Int8…Int64`, `UInt8…UInt64`) emits `φ_no_ovf` — signed with `T_MIN…T_MAX`
  bounds, unsigned with `0…2^N−1` — and **none silently wraps**. The cases
  instantiate `+` on `Int32` (emit/discharge) and `Int8` (no-wrap); the
  elaborator dispatches the bare-op arm uniformly, so it generalizes to `-`/`*`
  and every width. No case admits a bare-op result that bypasses the obligation.
- **No-implicit-coercion reject class agrees.** Every cross-type numeric op
  without an explicit conversion **rejects** (`Int + Int64`, `Int32 + Int64`,
  `Float + Int`, `Decimal + Float`, …); the only cross-type path is a named
  conversion function. No case accepts a mixed-type op.
- **Obligation-mechanism consistency.** The overflow obligation (AC3), the
  division obligation (§3.1), and the discharged/undischarged degrade share one
  shape: the triple `⟨id, Γ ⊢ φ, provenance⟩`, a typed hole `?id : φ`
  (`22 §1`, `24 §2`), discharged by proof or degraded to a listed postulate
  (`43 §2`). No case encodes a one-off "checked mode" distinct from this.
- **`Float`-honesty consistency.** `Float` is never the integer-literal default
  (AC2) and never asserted exact (AC6); no case treats `Float ==` as reliable or
  `Float` as the universal carrier.

## Subsumed bootstrap siblings

This seed is **authoritative** for the L1 numeric model. The two bootstrap cases
in `../seed-surface.md` — `surface/numbers/int-not-float` and
`surface/numbers/int-exact-above-2^53` — are the pre-L1 placeholders for AC2 and
AC1; they are **subsumed** here (expanded to the off-grid AC1 witness and the
full distinct-types / expected-type-override AC2 form) and retired to a pointer
in `seed-surface.md`, so each property has **one home** (subsume-don't-
proliferate; no contradictory siblings, see
[[conformance-reconcile-inherits-spec-metatheory-bugs]]).

## Build-sequencing note (the L-classes boundary)

These cases pin only the **L1 standalone** deliverable: the built-in,
form-keyed literal default table (§4.1) and the no-new-kernel-rules boundary
(§6.1) — **buildable now**. The **polymorphic-over-user-types** story (a literal
resolving to `fromInteger`/`fromDecimal` by **instance search** over user
`Num`/`Integral`/`Fractional` instances, §4.2) is **gated on L-classes**
(`33 §5`/`§7`, `39`) and is **not** covered here — it is the L-classes
follow-on.
No case asserts user-numeric-type instancing; the default table is the fixed
elaborator rule that does not wait on instance search. *(The WP frame's §2.5
floated instance-search defaulting as in-scope; spec `35 §4.2` correctly stages
it behind L-classes — these cases follow the elaborated spec.)*
