# ADR 0010 — Lawful `DecEq`/`Ord` requires a canonical carrier; string equality is codepoint-wise, normalization-equality is `Eq`-not-`DecEq`

- **Status:** Accepted
- **Date:** 2026-07-02
- **Deciders:** Architect (soundness ruling), Steward (commissioned the record)

## Context

A lawful structure class whose law *concludes or hypothesises the kernel's
propositional equality* ties a value-level operation to that equality. In Ken
these are the **concrete-equality-conclusion laws** (`51 §2.2`, `§6`):

- `DecEq a`'s `sound : (x y : a) → IsTrue (eq x y) → Eq a x y` — `eq`
  **decides** the kernel `Eq a x y : Ω` (`10-kernel/15`, `16`; `Equal` in
  the implementation);
- `Ord a`'s `antisym : IsTrue (leq x y) → IsTrue (leq y x) → Eq a x y`;
- a lawful `Num a`'s commutativity/associativity when stated as a kernel
  equation.

These laws are only *theorems* when the value-level op agrees with the kernel's
**definitional** equality on the carrier. That holds automatically when the
carrier is **canonical** — exactly one representation per semantic value — and
can **fail outright** when it is not.

The BUILTINS / Decimal-Char DEMOTE arc (2026-07-02) made the failure concrete.
`Decimal` demoted to the derived pair `MkDecimalPair (coeff exp : Int)`, a
**non-canonical** carrier: `10 × 10⁻¹` and `1 × 10⁰` both denote `1` yet are
structurally distinct pairs. `decimalEq` is a *value*-equality (it aligns
exponents, then compares) — correct and useful **as an operation**. But
`DecEq Decimal.sound`, stated as `IsTrue (decimalEq x y) → Eq Decimal x y`, is
**false**: `decimalEq (MkDecimalPair 10 (-1)) (MkDecimalPair 1 0)` reduces to
`True`, so a postulated `sound` would yield `Eq Decimal` between the two, and
`MkDecimalPair` injectivity refutes it (`Eq Int 10 1 → Bottom`). Postulating the
law **inhabits `Bottom`** — a genuine soundness hole, not a trust-level nit, and
not rescued by the law "bottoming out at a landed, audited `Axiom`": the check
that a postulate's proof-chain reaches an honest audited leaf never asks whether
the *statement being postulated is even true* (`51 §5`, the "Laws PROVED, not
postulated" gate, guards deception, not falsehood). `Decimal`'s lawful equality
is therefore an **open decision** — `90 §OQ-decimal-eq` (canonicalise the
carrier vs. a setoid/quotient equality).

The same question now gates the string surface (the `L3-strings` frame, the
string half of Rosetta readiness). `String` round-trips to `List Char` via the
native `string_to_list_char` / `list_char_to_string` pair, and the intended
Rosetta string ops (`concat`, `slice`, `charAt`, `eq`, `compare`) derive over
`List Char`. Whether a lawful `DecEq String` / `Ord String` is *deliverable*
depends entirely on which equality `String` carries.

## Decision

1. **The canonicity invariant (general).** A lawful instance of a class whose
   law ties a value-level op to the kernel's propositional equality (`DecEq`'s
   `sound`/`complete`, `Ord`'s `antisym`, a kernel-equational `Num` law) is
   sound **only over a canonical carrier** — one representation per semantic
   value. Over a **non-canonical** carrier such a law is a false proposition and
   postulating it inhabits `Bottom`; only the weaker **`Eq`** class (a Boolean
   equivalence with **no** tie to kernel equality, `51 §2.1`) is sound there.

2. **String equality is codepoint-wise (scalar-sequence).** This is the default
   and normative choice for `eq`/`compare`. It makes `String` **canonical** with
   respect to `List Char` — the round-trip is the identity on scalar sequences —
   so `DecEq String` / `Ord String` are genuinely **deliverable**, by transport
   from the canonical `DecEq Char` / `Ord Char` (`Char = {c : Int | isScalar c}`
   is itself canonical: one carrier value per codepoint under erasure). The
   value-level `compare`/`leq`/`eq` on `String` are available now (derived over
   `List Char`); the lawful *instances* are both **pending** follow-ons — a
   lawful `Ord String` is the **nearer** one (its element order `Ord Char` is
   landed; it still needs the lexicographic law proofs), while a lawful `DecEq
   String` is **farther** (a lawful `DecEq Char` is not yet on `main`).

3. **Normalization-insensitive equality is `Eq`, never `DecEq`.** Any
   NFC/NFD-aware string equality (`nfc_eq = eq ∘ normalize`, treating a
   precomposed and a decomposed `é` as equal) is a **distinct, explicitly named
   equivalence relation**. `normalize` collapses distinct scalar sequences to
   one value, so `nfc_eq → Eq String` would inhabit `Bottom` — the `DecEq
   Decimal` failure verbatim. It may be published as an `Eq`/setoid; it must
   **never** be filed as a `DecEq`. The same rule governs any case-insensitive,
   quotient, or tolerance equality (float `≈`, set/multiset-as-list, …).

4. **Establish canonicity before authoring.** Before authoring or accepting any
   `DecEq`/`Ord`/`Num` instance over a demoted or derived carrier, construct the
   candidate counterexample — two distinct representations of one semantic value
   — and check whether the law's kernel-equality conclusion actually holds. This
   axis is **orthogonal** to the trust-level-precision axes (zero-delta vs
   zero-NEW-delta, absolute vs net `trusted_base()`): a claim can be perfectly
   honest on those and still be flatly false on this one.

## Consequences

- **Rosetta strings** use codepoint-wise `eq`/`compare` — the simplest, most
  portable, Unicode-correct default. The **value-level** `compare`/`leq`/`eq` on
  `String` are available now (derived over `List Char`, this WP); the lawful
  **instances** are **pending** follow-ons. A lawful `Ord String` is the
  **nearer** follow-on — its element order (`Ord Char`, leq-only) is landed on
  `main`, so it needs only the lexicographic law proofs; a lawful `DecEq String`
  is **farther**, gated on a lawful `DecEq Char` that is **not** yet on `main`
  (only the `eqChar` *view* + `Ord Char`-by-transport are). Codepoint-indexed
  `slice`/`charAt` (via `List Char`) are also the correct default, since a
  byte-indexed slice can split a multi-byte scalar.
- **The derive-over-`List Char` strategy has a hard prerequisite** the frame
  must sequence first: `string_to_list_char` and `list_char_to_string` are
  currently `EvalVal::Neutral` stubs (`eval.rs`; "totality asserted at the type
  level" but never decoded). Until the round-trip pair is a real,
  round-trip-correct native UTF-8 decode/encode (scalar witnesses on the
  `Char`s), **every** derived string op is stuck. Making the round-trip real is
  `L3-strings` deliverable #1; the
  derived surface (zero-`trusted_base()`-delta) falls out of it.
- **`Decimal`'s lawful equality stays deferred** behind `90 §OQ-decimal-eq`
  (canonicalise vs. setoid) — the same axis, tracked as the single decide-once
  gate for the future `class Num` + Decimal-equality lane. `Num Decimal` will
  hit the identical wall (commutativity-as-`Eq` fails on the non-canonical
  pairs).
- **A reusable pre-check for reviewers and authors:** the two-representations
  counterexample. It cleanly separates a carrier's *value-equality operation*
  (correct as a `Bool` op in the tested-not-trusted interpreter ring, "a wrong
  value, never a false proof") from any *law* that would have it decide kernel
  equality (sound only when the carrier is canonical).

## References

- `spec/50-stdlib/51-lawful-classes.md` — `§2.1` (`Eq`, the Boolean
  equivalence), `§2.2` (`DecEq`, `sound`/`complete` decide the kernel `Eq`),
  `§5` (laws proved not postulated), `§6` (the concrete-equality-conclusion
  laws: `antisym`/`sound`/`complete`).
- `spec/90-open-decisions.md §OQ-decimal-eq` — the Decimal decide-once basis
  (canonicalise the carrier vs. a setoid/quotient `Eq`), **OPEN**.
- `spec/10-kernel/18a-primitive-registry.md §5.6.1(4)` — Decimal: the
  value-equality op ships; the `DecEq`/`Num Decimal` law instances are not
  structurally deliverable on the non-canonical carrier.
- The Decimal/Char DEMOTE build and the `num-landedness` erratum — where the
  finding originated and was corrected across spec, conformance, and legend.
- ADR 0008 (typeclass coherence) — the coherence backdrop for lawful instances.
