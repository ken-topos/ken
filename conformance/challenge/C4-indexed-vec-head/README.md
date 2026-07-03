# C4 — length-indexed `Vec` with a total `head` (dependent elimination)

**Axis:** dependent types / indexed families / dependent elimination.
**Flavor:** B (capability-depth — should-PASS if landed, else a documented
known-gap). Sits **adjacent** to VAL2 finding #5 (the ≥2-recursive-field-`match`
bug): `Vec` has **one** recursive field, so it dodges that bug and probes the
*index* machinery specifically.

## Why this is a blind spot

VAL2 used simple `data` + `match` (`Tree`, `Shape`). It never reached an
**indexed family** — a `data` whose type depends on a value (`Vec a n`, the
length in the type) — nor **dependent elimination** where a `match` branch is
ruled **impossible by the index** and discharged by absurdity. This is the core
of a dependently-typed language and Ken's distinctive value.

## The pair

- **Sound direction — `vec-head.ken` — should-PASS (if landed).** `Vec a (Suc
  n)` is statically non-empty; `head : (a : Type) → (n : Nat) → Vec a (Suc n) →
  a` is **total** — the `VNil` case is **impossible by the index** (`VNil : Vec
  a Zero`, and `Zero ≢ Suc n`), so `match` omits it by absurdity. `head (VCons
  …)` reduces to the head element.
- **Reject direction — same file, final decl — should-REJECT (type error).**
  `head a Zero VNil` — applying `head` (which demands `Vec a (Suc n)`) to `VNil
  : Vec a Zero` is a **type error**: the index `Zero` cannot unify with `Suc n`.
  A statically-empty vector cannot be `head`ed.

## Expected behavior (exact)

- The `Vec` **declaration** (an indexed family) is expected to **elaborate** —
  indexed `data` is landed (L2).
- `head` on `Vec a (Suc n)` with the **impossible `VNil` arm omitted by
  absurdity**: **known-gap** — the "absurdity fill for index-impossible arms"
  is **DEFERRED** (`l2_acceptance.rs` AC5 `indexed-impossible-pair`, `34 §4.3`).
  So `head` as written (relying on omitting the impossible branch) is expected
  to be **rejected today with a "non-exhaustive / cannot omit `VNil`" error** —
  a *documented capability gap*, not a soundness failure. If it **does**
  elaborate, dependent elimination with absurdity fill is further along than the
  ledger records — a positive finding.
- `head a Zero VNil`: **should-REJECT (type error)** regardless — the index
  mismatch is a plain typing error and must fire even without the absurdity-fill
  machinery.

## Discriminates

Two things: (1) does the elaborator support **impossible-branch-by-absurdity**
in a dependent `match` (the deferred `34 §4.3` piece)? (2) does the **index**
statically forbid `head` on an empty vector (`Zero ≢ Suc n`)? The `VCons` pass
vs the `VNil` type-error is the flip; the impossible-arm omission is the
capability probe.

## Surface-expressibility note

Indexed `data` with an explicit index (`data Vec (a : Type) : Nat → Type = …`)
follows the `14`/`34` grammar. If the surface cannot yet write the index
signature `: Nat → Type`, that itself is the gap to record. The `head`
definition is written to **rely on** the deferred absurdity-fill precisely so a
first run pins exactly where the dependent-elimination frontier is.
