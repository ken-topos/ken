# Numbers and primitive scalars

> Status: **impl-ready (L1)**. Normative for the numeric model. Ken has `Int`
> from day one and a clear, honestly-typed scalar story: distinct numeric types,
> integer precision that never silently degrades above 2⁵³, and no universal
> `f64` carrier.
>
> **L1 scope (this WP).** L1 pins, to team-ready rigor: the **type set + exact
> lowerings** to `../40-runtime/41-values.md` (§2); the **fixed-width overflow
> obligation** — its VC structure, V2 routing, and degrade-to-runtime-check, as
> a *sealed* op-class dispatch with no silent-wrap default (§3); the **built-in
> literal default table** as a standalone elaborator rule (§4); explicit
> **conversions** (§5); and the **prelude laws as propositions**, not kernel
> rules (§6). **No kernel enlargement** — every numeric type is a `14 §5`
> primitive (opaque constant + registered, audited reduction); the
> non-definitional laws are prelude propositions.
>
> **Staging boundary — flag, do not reopen.** L1 ships the *built-in* default
> table (a fixed, form-keyed elaborator rule). The *polymorphic-over-user-types*
> story — a literal resolving to `fromInteger`/`fromDecimal` by **instance
> search** over user `Num`/`Integral`/`Fractional` instances — is gated on
> **L-classes** (the open class/instance mechanism, `33 §5`/`§7`, `39`), whose
> `Num`/`Integral`/`Fractional` classes are not yet specified (a `50-stdlib`
> follow-on). L1's deliverable is the standalone table; user-numeric-type
> instancing is the L-classes follow-on (§4.2).

## 1. The model, stated plainly

Ken's numeric model has **distinct, honestly-typed scalars**, not a single
universal numeric type. Values lower heterogeneously — `i1`/`i8`/`i64`/`f64`/
structs — as unboxed SSA values (`../40-runtime/41-values.md`). The design rests
on three commitments, each load-bearing for a *verified* language:

1. **Exactness by default.** The unsuffixed integer type `Int` is
   arbitrary-precision (§2.1), so `a + b` *means* addition and arithmetic specs
   (`a + b == b + a`) hold with **no overflow side-condition**.
2. **Honesty in the type.** `Float` is one honestly-named IEEE type among
   several, **not** the universal carrier and **not** the integer-literal
   default; `Decimal` is a first-class exact base-10 type. The type a value
   carries tells the truth about its representation and its equality.
3. **Partiality is marked, never silent.** Fixed-width overflow and
   division-by-zero are *obligation-generating* partial points (§3), surfaced in
   the obligation set — never a silent wrap or trap.

> There is no uniform-`f64` ontology and no 4×`f64` wire framing
> (`../40-runtime/41-values.md`). Lowering is heterogeneous and typed; the type
> determines the representation.

## 2. The numeric types

| Type | Meaning | Literal | Lowering (`41`) |
|---|---|---|---|
| **`Int`** | **arbitrary-precision** integer (default) | `42`, `0xFF` | small-int fast path `i64`, promotes to heap bignum (§2.1) |
| **`Int8 Int16 Int32 Int64`** | native signed fixed-width integers | `42 : Int32` | `i8`/`i16`/`i32`/`i64` immediates |
| **`UInt8 UInt16 UInt32 UInt64`** | native unsigned fixed-width integers | `0xFF : UInt8` | `u8`/`u16`/`u32`/`u64` immediates |
| **`Decimal`** | base-10 exact decimal (money) — **core type** | `3.14d` | inline `{i64 coeff, i32 exp}`, promotes to heap (§2.2) |
| **`Float`** | IEEE-754 binary64 | `3.14`, `1e-9` | `f64` immediate |
| `Float32` | IEEE-754 binary32 | `1.5f32` | `f32` immediate |
| `Bool` | boolean | `true`/`false` | `i1` |
| `Char` | Unicode scalar value | `'a'` | `u32` (U+0000–U+10FFFF, surrogates excluded) |

Every row is a **primitive type** in the kernel's sense (`14 §5`, §6): an opaque
type constant whose inhabitants are literals and the results of registered
primitive operations. The lowerings below are pinned to the **landed**
`41-values.md`, not paraphrased — `41` is the authority for representation.

### 2.1 `Int` — arbitrary-precision (`OQ-int` DECIDED)

`Int` is arbitrary-precision by default (not fixed-64). For a *verified*
language silent overflow is a correctness hazard; arbitrary precision makes
`a + b` mean addition, so `a + b == b + a` holds with no side-condition. The
representation is a **small-integer fast path** — a machine word `i64` for the
common case — that **promotes to a heap bignum** only when a value outgrows the
word (`41 §1`, `41 §5`). The heap form is sign-magnitude, minimal-limb (no
trailing-zero limbs), content-addressed with canonical-encoding tag `0x01`
(`41 §3a`); equality is therefore exact at every magnitude. This is the
2⁵³-exactness guarantee at the representation level: a 20-digit integer literal
is an exact bignum, never an `f64`-rounded double (AC1).

### 2.2 Fixed-width integers (`OQ-int` DECIDED)

The full signed `Int8/Int16/Int32/Int64` and unsigned
`UInt8/UInt16/UInt32/UInt64` set are **first-class native types**, the everyday
currency of bitfields, wire/byte layout, and C-ABI FFI interop (`38`). Each
lowers **directly to its machine width** as an unboxed immediate (`41 §5`,
"Immediate scalars" — stored inline, never interned). Their overflow semantics
are explicit and obligation-generating (§3) — the differentiator from a
silent-wrap fixed-width integer.

### 2.3 `Decimal` — exact base-10 (`OQ-int` DECIDED)

`Decimal` is a core, essential type — exact base-10 for money and any
computation where binary floating point is wrong by construction; **not** an
`f64` alias. A `Decimal` is a base-10 value `coeff × 10^exp` with an
**arbitrary-precision coefficient** and a bounded `i32` base-10 exponent. The
layout mirrors `Int`: an **inline `{i64 coeff, i32 exp}` fast path** that
**promotes to a heap big-`Decimal`** (canonical-encoding tag `0x0A`, `41 §3a`)
when the coefficient outgrows `i64` (`41 §5`, "Small `Decimal`"). Literal suffix
`d`. Because the coefficient is exact, `0.1d + 0.2d == 0.3d` holds exactly
(AC6) — the property the `Float` analog cannot offer.

> **Reconcile note (don't cite the frame's struct).** An earlier draft and the
> WP frame floated a flat `{i128 coeff, i32 exp}`. The **landed** `41 §5` pins
> the inline fast path as `{i64 coeff, i32 exp}` **with heap promotion** of the
> coefficient (parallel to `Int`), so the coefficient is arbitrary-precision,
> not capped at 128 bits. `41` is the lowering authority; this chapter follows
> it.

### 2.4 `Float`/`Float32`, `Bool`, `Char`

`Float`/`Float32` are IEEE-754 binary64/binary32, *honestly named*: **not** the
universal value carrier and **not** the default for integer literals (§4). ℝ
does not embed faithfully in `Float` — `Float` equality is the usual IEEE
minefield (NaN ≠ NaN, ±0, rounding) and Ken does not pretend otherwise. Proofs
about reals use `Decimal`/rationals or an explicit error-bound discipline, never
`Float ==`. `Bool` lowers to `i1`; `Char` is a **Unicode scalar value** lowering
to `u32` — its valid range is U+0000–U+10FFFF **excluding the surrogate block**
U+D800–U+DFFF (a refinement on the carrier, not all `u32` are `Char`).

## 3. Overflow and partiality

Partiality enters numeric code only at **marked** points, each generating an
obligation, never a silent trap (`../40-runtime/43 §2`, the partial-primitive
discipline).

### 3.1 `Int` — no overflow; division is the only partial point

`Int` is arbitrary-precision, so `+`/`-`/`*` are **total** (no overflow
obligation). The one partial point is **division/modulo by zero**: `div`/`mod`
either take a `{ d | d ≠ 0 }` refinement argument making them total (`34 §5` —
refinement coercion `{x:A|φ} ≤ A` emits the `φ` obligation at the use site), or
return `Option`/a checked error (`36`). A raw `/` or `%` on a possibly-zero
divisor emits a **non-zero side-condition obligation** at the operation site
(`22 §2.4`), not a silent trap.

### 3.2 Fixed-width — the no-overflow obligation (`OQ-1a` DECIDED)

A bare `+`/`-`/`*` on a fixed-width type is **obligation-generating**. This is
the verification-language differentiator; the rest of §3.2 pins it to
implementable rigor.

**The VC structure.** For a bare `a ⊕ b : T` with `⊕ ∈ {+, -, *}` and `T` a
fixed-width type (`Int8…Int64`/`UInt8…UInt64`), V2's extractor emits the
obligation triple (`22 §1`, `22 §2.4`)

```
  ⟨ id , Γ ⊢ φ_no_ovf , provenance ⟩
```

where the goal proposition is the **no-overflow predicate over the operands,
computed in the arbitrary-precision `Int` (ℤ) domain**:

```
  φ_no_ovf  ≡   T_MIN ≤ (a ⊕_ℤ b)  ∧  (a ⊕_ℤ b) ≤ T_MAX        -- signed T
  φ_no_ovf  ≡   0     ≤ (a ⊕_ℤ b)  ∧  (a ⊕_ℤ b) ≤ 2^N − 1       -- unsigned T (width N)
```

`⊕_ℤ` is the operation in ℤ (no wrap), `T_MIN`/`T_MAX` the target width's
representable bounds (e.g. `Int32`: −2³¹ … 2³¹−1). `φ_no_ovf : Ω` and lives in
the local context `Γ` carrying any in-scope range refinements on `a`, `b` — so a
caller-supplied bound `{x : Int32 | x < 100}` discharges the obligation by
ordinary proof. The obligation routes through V2's machinery **exactly like a
refinement obligation** (`22 §2.4` lists it alongside §2.1's): it is a typed
hole of type `φ_no_ovf` (`22 §1`), discharged by supplying a proof term the
kernel re-checks.

**Degrade-to-runtime-check.** Discharged (proven in-range) ⇒ the operation is
**total and safe**, no residual runtime cost. Undischarged (an open hole) ⇒ a
**marked partial point** that degrades to a runtime check: at the unguarded use
it panics / yields `unknown` (`43 §2` case 2, `24 §2` — the hole is a listed
postulate). "Checked arithmetic" is therefore **subsumed** as the runtime face
of an *undischarged* obligation, not a separate mode the user selects.

**The op-class dispatch is sealed — no silent-wrap default.** The arithmetic
surface over a fixed-width type partitions into exactly **four explicit op
classes**, a closed match the elaborator dispatches with **no default arm**:

| Op class | Surface | Semantics | VC |
|---|---|---|---|
| **obligation** (the bare operator) | `a + b`, `a - b`, `a * b` | total-if-proven; degrades to runtime check | emits `φ_no_ovf` (§3.2) |
| **wrapping** | `wrapping_add`/`+%`, `Wrapping[T]` | modular `mod 2^N`, provably-modular | none (intended-modular) |
| **checked** | `checked_add : … → Option T` | `None` on overflow | none (result is `Option`) |
| **saturating** | `saturating_add` | clamp to `T_MIN`/`T_MAX` | none (total by clamp) |

The bare operator's image is **fixed to the obligation class**; there is no
fall-through to wrapping. A build team cannot reach for "wrap by default"
because the op classes are a **closed set** and the bare-op case is pinned — the
type-level exhaustiveness net (the same sealed-construction discipline B1's
no-measure `G` and Sec1ct's leak-sink use: forbid the unsound default by
construction, not by convention). Modular semantics is available **only** by
explicitly naming the wrapping class — visible and provably-modular, for domains
where wrap is the intent (hashing, crypto, checksums) — never silent.

> **Untrusted-layer net (★★, why AC3 is structural).** The obligation extractor
> (V2) is untrusted (`22` preamble): a *spurious* obligation is at worst a false
> `unknown`, but a **missed** obligation is **not** backstopped — the kernel
> only ever re-checks the certificates V2 chose to emit, and never sees the
> overflow site V2 skipped. So conformance for the overflow obligation must net
> that the obligation is **emitted** — drive a real fixed-width `+` and observe
> the emitted `φ_no_ovf` / the degrade-to-check — not merely assert "it
> compiles" or probe a synthetic flag (AC3; the QA gate, §7).

### 3.3 `Float` — IEEE semantics, exactly

`Float`/`Float32` carry IEEE-754 semantics exactly — NaN, ±∞, signed zero — with
no hidden "corrections." Overflow produces ±∞, not an obligation; this is the
honest IEEE behavior, and the reason `Float` is not the verification default.

## 4. Literals; built-in defaults and the L-classes boundary

A numeric literal's type is fixed by the **built-in default table** (§4.1) when
unconstrained, or by the **expected type** when there is one. Full polymorphism
over *user* numeric types via instance search is the **L-classes follow-on**
(§4.2) — explicitly out of L1's standalone deliverable.

### 4.1 The built-in default table (ships in L1, no L-classes)

An unconstrained numeric literal defaults by its **lexical form** (`31 §3`), a
fixed elaborator rule — *not* instance search:

| Literal form | Examples | Default type |
|---|---|---|
| bare integer | `42`, `0xFF`, `0b1010`, `0o17`, `1_000` | `Int` (arbitrary precision) |
| decimal-point / exponent, no `d` | `3.14`, `1e-9`, `0x1p-3` | `Float` |
| `d`-suffix | `19.99d`, `0.1d` | `Decimal` |
| `f32`-suffix | `1.5f32` | `Float32` |

```
view f (x : Int64) = x + 1      -- `1` checks against the expected Int64; no default
let big = 100000000000000000000 -- Int (arbitrary precision); not f64-rounded
let pi  = 3.14159               -- Float (has a `.`, no `d`)
let amt = 19.99d                -- Decimal
```

This is `39 §2.5`'s **declared default** — the one place the elaborator may
default a literal without an ambiguity error. **Expected-type override:** where
a literal appears in a typed position (`x : Int64` above, an annotated argument,
a field of known type), it elaborates at the **expected type** and the default
table does not fire; the default applies **only** when the literal is otherwise
unconstrained. `2` and `2.0` are **different types** under this rule (integer
vs. decimal-point form) — the `f64` non-reproduction at the type level (AC2).

### 4.2 Staging boundary — user-type overloading is gated on L-classes

The *polymorphic-over-numeric-classes* story — a literal elaborating to
`fromInteger`/`fromDecimal` resolved by **instance search** so user numeric
types join by instancing — requires the open class/instance mechanism. That
mechanism is **L-classes** (`33 §5` gives the general machinery —
typeclasses-as-subobjects, instance search via `39`; `33 §7` makes the
class/instance/`derive` system a WS-L deliverable). The specific
`Num`/`Integral`/`Fractional` classes and the `fromInteger`/`fromDecimal`
desugaring are **not yet specified** — only a placeholder row in
`50-stdlib/README.md` — and land with L-classes.

**Pinned boundary (flag, do not silently resolve):** **L1 delivers the built-in
default table of §4.1 standalone**, a fixed elaborator rule that does **not**
wait on instance search. **User-numeric-type instancing** (literals polymorphic
over user `Num` instances) is the **L-classes follow-on**, sequenced after
L-classes lands. Team Language builds §4.1 now; it does not block on, nor
pre-build, the §4.2 overloading. (This is the admittance-vs-staging discipline:
"literals are polymorphic via the typeclass mechanism" is true of the *eventual*
delivery, but L1's buildable-now subset is the fixed default table; the
overloading is blocked-on-stage.)

## 5. Conversions (explicit)

There are **no implicit numeric coercions** (a frequent bug source). Every
cross-type numeric move is a **named function** — the conversion surface is a
closed set with **no implicit-coercion arm**. Widening and narrowing are
explicit total/partial functions; lossy ones are visible in the type:

```
Int64.toInt     : Int64 → Int             -- total (widening into arbitrary precision)
Int.toInt64     : Int → Option Int64      -- partial (may not fit)
Int32.toInt64   : Int32 → Int64           -- total (widening, same signedness)
Int64.toInt32   : Int64 → Option Int32    -- partial (narrowing may not fit)
Int.toFloat     : Int → Float             -- total, documented-lossy above 2^53 (typed so)
Decimal.toFloat : Decimal → Float         -- documented-lossy; named
Float.toDecimal : Float → Option Decimal  -- partial (NaN/∞ have no Decimal)
```

The rule: a conversion that **always succeeds without loss** is `total`
(`Int64.toInt`, narrow→wide same-signedness); one that **may not fit** returns
`Option` (`Int.toInt64`, wide→narrow, signed↔unsigned); one that **always
succeeds but loses precision** is `total` and **documented-lossy** in its name
and type (`Int.toFloat` above 2⁵³). `Int + Int64` *without* an explicit
conversion is a **type error** (AC5) — there is no widening coercion to make the
operands agree.

> **Not a counterexample:** refinement coercion `{x:A|φ} ≤ A` (`34 §5`) is *not*
> an implicit numeric coercion — it is the identity on the same underlying type
> `A`, carrying a proof obligation, not a representation change between two
> numeric types. The "no implicit coercion" rule is about *numeric-type* moves.

## 6. Kernel view and prelude laws

### 6.1 Primitives with registered reductions (no kernel enlargement)

All numeric types are **primitive types** (`14 §5`): opaque type constants with
**registered, audited reductions** for the operations, so `2 + 3 ≡ 5 : Int`
computes *in the kernel's evaluator* and proofs reduce over literals (on
non-literal/stuck arguments a primitive op is a neutral term). The set of
primitive types and their reductions is **small, audited, and part of the
kernel's trusted base** (listed in `18 §5`); a wrong primitive reduction is a
soundness bug, so this is the one audited place computation enters the kernel
from outside the term language. **L1 adds no new kernel rules** — it specifies
the *surface and elaborator* face of these existing primitives; the registered
reductions are kernel primitives, not L1 inventions.

### 6.2 Non-definitional laws are prelude propositions

The **definitional** facts — `2 + 3 ≡ 5 : Int`, literal computation — are the
registered `prim` reductions (§6.1); they hold by computation. The
**non-definitional** laws — commutativity `a + b == b + a`, associativity,
distributivity, the ring/field axioms — are **propositions in the prelude**
(`14 §5`, `50-stdlib/`), **not** new kernel reductions. They are discharged one
of two ways, kept small and visible (TCB discipline):

- **(a) proved against a reference model** — e.g. the bignum/`Int` semantics —
  so the law is a theorem, no trust added; or
- **(b) axiomatized as a small, visible interface** — the audited primitive-law
  set, listed alongside the primitives in `18 §5`.

The boundary is the point: **registered reductions are trusted kernel primitives
(audited); algebraic laws are prelude propositions (proved, untrusted).** L1
does not move a law into the kernel. Conformance nets that the registered
reductions **match the reference model** (the trusted-primitive obligation, §7).

## 7. What WS-L must deliver here (L1) + acceptance

**Deliverables.** `Int` (arbitrary precision, fast-path + bignum, §2.1) +
fixed-width integers with the obligation-generating overflow discipline (§2.2,
§3.2); `Decimal` (exact base-10, inline + heap, §2.3); honestly-named
`Float`/`Float32`, `Bool`, `Char` (§2.4); the built-in literal default table as
a standalone elaborator rule (§4.1) with the L-classes boundary pinned (§4.2);
explicit, visibly-lossy conversions as a closed named set (§5); the algebraic
laws as prelude propositions (§6.2). **No new kernel rules** (§6.1).

**Acceptance criteria (each a verdict- or structure-flip, per the conformance
discipline):**

- **AC1 (Int exactness)** Integer arithmetic above 2⁵³ is **exact** —
  `100000000000000000000 : Int` is an exact bignum, not `f64`-rounded. A
  **structural value assertion** (the stored value), not a type check (§2.1).
- **AC2 (literal types distinct)** `2 : Int`, `2.0 : Float`, `2.0d : Decimal`
  are **distinct types**; a program relying on `2 ≡ 2.0` is **rejected**
  (verdict flips: well-typed vs ill-typed) (§4.1).
- **AC3 (overflow obligation)** A bare `a + b : Int32` **emits a no-overflow
  obligation**: provably-in-range accepts as total; un-provable **degrades to a
  runtime-checked partial point**. Assert the obligation is **emitted**
  (structural — observe the emitted `φ_no_ovf` / the degrade-to-check), not just
  "compiles" (§3.2; untrusted-layer net).
- **AC4 (no silent wrap)** Fixed-width overflow **never silently wraps** — the
  only wrapping path is the explicit `wrapping_add`/`+%`/`Wrapping[T]`
  (provably-modular). A test that overflows *without* the explicit op must NOT
  produce a wrapped value silently — it obligation-checks / panics (§3.2).
- **AC5 (no implicit coercion)** `Int + Int64` without an explicit conversion is
  a **type error** (reject); the explicit `Int.toInt64` returns `Option` (§5).
- **AC6 (Decimal exact)** `0.1d + 0.2d == 0.3d` (exact base-10), while the
  `Float` analog is honestly **not** asserted equal (§2.3, §3.3).

**Conformance** (`../../conformance/surface/numbers/`): the `> 2^53` exactness
regression (AC1) + literal-defaulting cases (AC2) + the overflow-obligation
cases (AC3/AC4) + the no-implicit-coercion rejects (AC5) + the `Decimal`-exact
case (AC6). Per-case **verdict/structural-flip** (right=accept/wrong=reject, or
a structural output the bug would change) and the **cross-case sweep**: the
overflow-obligation class agrees (every bare fixed-width `+`/`-`/`*` emits the
obligation, none silently wraps), and the no-implicit-coercion reject class
agrees (every cross-type op without an explicit conversion rejects). **QA gate
(2-team build-qa lesson):** the overflow-obligation cases must route through the
**actual VC emission** — drive a real fixed-width `+` and observe the emitted
obligation / the degrade-to-check — never predicate about a synthetic flag.

**Staging note for the build team.** §4.1 (built-in default table) and §6.1 (no
new kernel rules) are buildable now. §4.2 (user-numeric-type instancing via
`Num`/`Integral`/`Fractional`) is **blocked on L-classes** (`33 §7`) — do not
build it under L1; it is the L-classes follow-on.
