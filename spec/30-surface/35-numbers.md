# Numbers and primitive scalars

> Status: **DRAFT v0**. Normative for the numeric model (the single most
> important correction the reality-check made). The prototype's central defect —
> *one* surface numeric type `number` lowered to `f64`, silently losing integer
> precision above 2⁵³ — is **not reproduced**. Ken has `Int` from day one and a
> clear, honestly-typed scalar story.

## 1. The correction, stated plainly

The analysis claimed "every value is a uniform f64" was a foundational flaw. The
reality-check **refuted** the ontology (the prototype already lowers
heterogeneously — `i1`/`i8`/`i64`/`f64`/structs — scalars are unboxed SSA
values), but confirmed **one real, narrow defect: there is no distinct `Int`**;
the lone numeric type is `number → f64`. Ken's response is the small, contained
fix the reality-check prescribes, **not** the analysis's giant "abandon f64 /
typed handles" workstream (which targets a non-problem):

> Ken adds `Int` (and `Decimal`) as first-class types and keeps `Float` as one
> honestly-named numeric type among several. Use heterogeneous typed lowering
> (`../40-runtime/41-values.md`). Do not build Ken around a uniform-f64 ontology
> or a 4×f64 wire framing.

## 2. The numeric types

| Type | Meaning | Literal | Lowering (`41`) |
|---|---|---|---|
| **`Int`** | **arbitrary-precision** integer (default) | `42`, `0xFF` | small-int fast path `i64` + bignum overflow (§3) |
| `Int64`, `Int32`, `UInt64`, … | fixed-width machine integers | `42 : Int64` | the named machine width |
| **`Decimal`** | base-10 exact fixed/large decimal (money) | `3.14d` | a decimal struct (`{i128 coeff, i32 exp}` or sim.) |
| **`Float`** | IEEE-754 binary64 | `3.14`, `1e-9` | `f64` |
| `Float32` | IEEE-754 binary32 | `1.5f32` | `f32` |
| `Bool` | boolean | `true`/`false` | `i1` |
| `Char` | Unicode scalar value | `'a'` | `u32` |

- **`Int` is arbitrary-precision by default** (OQ-1 records the alternative of
  fixed-64 default). Rationale: for a *verified* language, silent overflow is a
  correctness hazard; arbitrary precision makes `a + b` mean addition, so
  arithmetic specs (`a + b == b + a`, `../20-verification/`) hold without
  overflow side-conditions. The implementation uses a small-integer fast path so
  the common case is a machine word (`41 §numbers`); only values exceeding the
  word grow.
- **Fixed-width** integers (`Int64`, `UInt32`, …) are available where machine
  semantics/representation are wanted (FFI, byte layout, performance); their
  **overflow semantics are explicit** (§3).
- **`Decimal`** is the money/exact-base-10 type (the prototype's `money` was an
  f64 alias — Ken makes it exact). Distinct literal suffix `d`.
- **`Float`** is IEEE-754, *honestly named*. It is **not** the universal value
  carrier and **not** the default for integer literals. Be explicit that ℝ does
  not embed faithfully in `Float` (the analysis's one fair caveat): `Float`
  equality is the usual IEEE minefield and Ken does not pretend otherwise —
  proofs about reals use `Decimal`/rationals or an explicit error-bound
  discipline, not `Float ==`.

## 3. Overflow and partiality

- **`Int`** (arbitrary precision): no overflow. Division/modulo by zero is the
  partial case — `div`/`mod` either require a `{ d | d ≠ 0 }` refinement (`34
  §5`, total) or return `Option`/a checked error (`36`); raw `/` on
  possibly-zero is an obligation, not a silent trap.
- **Fixed-width** integers: overflow is **explicit** via distinct operations —
  `wrapping_add` (modular), `checked_add : … → Option`, `saturating_add`. A bare
  `+` on fixed-width is **checked by default** (panics/`unknown` on overflow) so
  it never silently wraps; opting into wrapping is a named choice. (Exact
  default — checked vs. obligation-generating — is OQ-1a.)
- **`Float`**: IEEE semantics (NaN, ±∞, signed zero) exactly; no hidden
  "corrections."

## 4. Literals are polymorphic; defaults are typed

A numeric literal is **overloaded** over the numeric classes and resolves by
context, with the §2 defaults when unconstrained:

```
view f (x : Int64) = x + 1     -- `1` resolves to Int64
let big = 100000000000000000000 -- Int (arbitrary precision); no f64 rounding
let pi  = 3.14159              -- Float (has a `.`, no `d`)
let amt = 19.99d               -- Decimal
```

- Integer literals default to **`Int`**, decimal-point literals to **`Float`**,
  `d`-suffixed to **`Decimal`** (`31 §3`). This is the f64 non-reproduction *at
  the type level*: `2` and `2.0` are different types, not the same `f64`.
- Overloading is the typeclass/constraint mechanism (`33 §5`): `Num`/`Integral`/
  `Fractional`-style classes; literals elaborate to `fromInteger`/`fromDecimal`
  calls resolved by instance search (`39`). User numeric types join by
  instancing.

## 5. Conversions (explicit)

There are **no implicit numeric coercions** (a frequent bug source). Widening
and narrowing are explicit total/partial functions:

```
Int.toInt64    : Int → Option Int64      -- partial (may not fit)
Int64.toInt    : Int64 → Int             -- total (widening)
Int.toFloat    : Int → Float             -- total, lossy above 2^53 (typed so)
Decimal.toFloat: Decimal → Float         -- lossy; named
```

Lossy conversions are **named and visible** in the type (`Option` for may-fail,
documented-lossy for `toFloat`), so the precision pitfalls the analysis
attributed to f64 are surfaced as ordinary, checkable function boundaries —
never silent.

## 6. Kernel view

All of these are **primitive types** (`../10-kernel/14 §5`): opaque type
constants with registered, audited reductions for the operations (so `2 + 3 ≡ 5
: Int` computes in the kernel and proofs reduce over literals). Their
non-definitional laws (commutativity, associativity, ring axioms) are
**propositions** in the prelude, proved against a reference model or axiomatized
as a small, visible interface (`14 §5`, `../50-stdlib/`).

## 7. What WS-L must deliver here (L1)

`Int` (arbitrary precision, fast-path) + fixed-width integers with explicit
overflow; `Decimal`; honestly-named `Float`/`Float32`; `Bool`/`Char`; typed
literal defaulting (`2:Int`, `2.0:Float`, `2.0d:Decimal`); numeric classes for
literal overloading; and explicit, visibly-lossy conversions. Acceptance:
integer arithmetic above 2⁵³ is exact (the prototype's defect, fixed) and `2 :
Int` / `2.0 : Float` are distinct. Conformance:
`../../conformance/surface/numbers/` — the `> 2^53` exactness regression and the
literal-defaulting tests.
