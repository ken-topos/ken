---
scope: enclave
audience: (see scope README)
source: private memory `carrier-canonicity-axis-for-lawful-class-laws`
---

# Carrier canonicity is a distinct soundness axis for lawful-class laws

On Ken's Decimal/Char DEMOTE arc (2026-07-02), a claim that
`DecEq`/`Num Decimal` laws were "zero-NEW-delta real structural proofs bottoming
at Int's Axiom leaves" survived spec-author's authoring, two prior erratum
correction-passes over the *same block*, and a soundness gate — because every
pass reasoned on the **law-provability-chain axis** (does the law bottom at an
honest postulate?) and none checked the **class-deliverability axis** (is the
law even a *theorem* on the actual carrier?). It wasn't:
`DecEq.sound : IsTrue(eq x y) → Equal a x y` is **false** on Decimal's
non-canonical `(coeff,exp)` carrier — `decimalEq (10,-1) (1,0)` reduces `True`
(both denote value `1`), but `MkDecimalPair` injectivity refutes `Equal Decimal`
on the structurally-distinct pair, so a postulated `sound` would inhabit
`Bottom`. The very value-equality that makes `decimalEq` *correct as an op* is
what makes the `DecEq` *law* false. Architect caught it as a genuine soundness
finding, not a documentation nit.

**The discriminator: is the carrier canonical — one representation per semantic
value?**
- **Canonical** (Ken's `Char = {c:Int|isScalar c}`, erasure-alias to `Int`, one
  Int per codepoint): `Equal Char ≡ Equal Int` definitionally, so transporting
  `Int`'s *true* meta-theorem `Axiom` is sound. `Ord Char`/ `DecEq Char` are
  genuinely deliverable.
- **Non-canonical** (Ken's `Decimal = (coeff:Int, exp:Int)`, many pairs per
  value — `10×10^-1` and `1×10^0` both denote `1`): a law tying a *value*-level
  op (`decimalEq`, which correctly identifies value-equal pairs) to *structural*
  definitional `Equal` is likely **false**, regardless of whether the chain
  "bottoms at a landed Axiom." The op can still be correct and useful; only the
  *law claiming it decides Equal* is unsound.

**Why this survives naive review.** A reviewer checking "does this law's proof
obligation bottom out at an honest, already-audited postulate" will pass a
non-canonical-carrier law that shouldn't exist at all — the postulate chain
*looks* like every other legitimately-deferred law (Ord Int's Axiom, Char's
transport), because the check never asks whether the *statement being postulated
is even true*.

**How to apply.** Before accepting or authoring any lawful-class instance
(`Eq`/`DecEq`/`Ord`/`Num`/…) over a DEMOTE'd or derived carrier, ask FIRST: is
the carrier canonical? If not, any law tying a value-level operation to
definitional `Equal` is a candidate false statement — construct the
counterexample (two distinct representations of the same semantic value) and
check whether the law's conclusion (`Equal`) actually follows, before accepting
a postulate as merely "audited-delta" or "zero-NEW-delta." This is a soundness
axis distinct from — and orthogonal to — trust-level-precision axes like
zero-delta vs zero-NEW-delta or absolute vs net `trusted_base()` counts; a claim
can be perfectly honest on those axes and still be flatly false on this one.
Sibling of proof relevant inductive cannot be declared at omega (both are "the
encoding/carrier shape determines admissibility, not the surface plausibility of
the postulate").

**Forward application — `String` / L3-strings (Architect ruling 2026-07-02,
`evt_66g17exdhd767`, gating Steward's L3-strings frame).** Applied this axis
prospectively when ruling the string-ops TCB boundary (derive-over-`List Char`,
zero new native prims; native = round-trip pair + `byte_length` + IO). The
canonicity call for string `eq`/`compare`: default them **codepoint-wise
(scalar-sequence)** — that makes `String` **canonical** w.r.t. `List Char`
(round-trip is identity on scalar sequences), so a future lawful
`DecEq String`/`Ord String` is genuinely **deliverable** (and the
`Ord Char`/`DecEq Char` instances just landed transport straight up to it). But
any **NFC-normalization-insensitive** `eq` (precomposed = decomposed é) must be
a distinct, explicitly-named derived **`Eq` (equivalence), NEVER a `DecEq`** —
`normalize` collapses distinct scalar sequences to one value, so
`nfc_eq → Equal String` would inhabit `Bottom` exactly like `DecEq Decimal`.
Same trap, forward; sibling of `90 §OQ-decimal-eq`. When L3-strings builds, my
gate checks (a) `eq`/`compare` are codepoint-wise, (b) no NFC-eq is dressed as a
`DecEq`, and (c) the derive strategy's prerequisite — `string_to_list_char` must
be REAL + round-trip-correct (it's currently a `Neutral` stub = the pin-2
obligation), else every derived op is stuck.
