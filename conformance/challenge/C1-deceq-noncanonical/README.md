# C1 — `DecEq` over a non-canonical carrier (the Bottom edge)

**Axis:** lawful typeclasses + the canonical-carrier soundness precondition.
**Flavor:** A (soundness-boundary — the unsound arm must REJECT; acceptance =
a hole). **Couples with C7** (the quotient is the principled fix).

## Why this is a blind spot

`DecEq a`'s law `sound : (x y : a) → IsTrue (eq x y) → Equal a x y` makes `eq`
**decide the kernel's propositional equality**. That is a *theorem* only when
the carrier is **canonical** — one representation per semantic value. Over a
non-canonical carrier the statement is **false**, and a postulated `sound`
inhabits `Bottom` (ADR 0010; the Decimal/Char DEMOTE arc). VAL2 used `Ord`/
`DecEq` as comparators but never stressed the canonicity precondition — and the
existing machinery (`51 §5`, "laws PROVED not postulated") **guards deception,
not falsehood**: it checks that a postulate's proof-chain reaches an honest
audited leaf, never whether the *statement being postulated is even true*. So a
naive lawful `DecEq Decimal { sound = Axiom }` is exactly the construction that
slips through.

## The pair

- **Sound arm — `sound-deceq-char.ken` — should-PASS.** `DecEq Char` over the
  canonical carrier `Char = {c : Int | isScalar c}` (one Int per codepoint under
  erasure). `sound = Axiom` here is an **honest, true** postulate: `Equal Char`
  coincides with `Equal Int`, and `eqChar` really does decide it. Behaves
  exactly like the landed `DecEq Int` (also `Axiom`, also sound — `Int` is a
  primitive with no induction principle, so its true laws are honest visible
  postulates).
- **Unsound arm — `unsound-deceq-decimal.ken` — should-REJECT.** `DecEq Decimal`
  over `Decimal = MkDecimalPair Int Int` (non-canonical: `MkDecimalPair 10 (-1)`
  and `MkDecimalPair 1 0` both denote `1`). `decimalEq` returns `True` on those
  two structurally-distinct pairs, so `sound` would yield
  `Equal Decimal (MkDecimalPair 10 (-1)) (MkDecimalPair 1 0)`; `MkDecimalPair`
  injectivity (K7 no-confusion) refutes it: `Equal Int 10 1 → Bottom`. The arm
  **includes the exploit** (derives `Bottom` from the instance's `sound`) so the
  hole is concrete, not hypothetical.

## Expected behavior (exact)

- Sound arm: **PASS** — `DecEq Char` elaborates; `sound`/`complete` are honest
  visible `Axiom`s over a canonical carrier (grounded: `lawful_classes.ken`
  `DecEq Int` is the identical honest-`Axiom` shape; `Char` canonicity per ADR
  0010 §2).
- Unsound arm: **should-REJECT** — the correct outcome is that the lawful
  `DecEq Decimal` instance is **not deliverable** (ADR 0010 decision pt1/§90
  `OQ-decimal-eq`). **Predicted actual result on a first run: ACCEPT** (the
  instance elaborates and the exploit type-checks), because the machinery guards
  deception not falsehood (`51 §5`) — the `es4_classes` precedent shows this
  class of Bottom-inhabiting law was caught by *review*, then re-deferred, **not
  by a machinery check.** **If it accepts, that is the headline finding** — a
  `declare_postulate`-able false law is a soundness hole the canonicity gate
  (ADR 0010 pt4: "construct the two-representations counterexample before
  accepting the instance") is meant to close.

## Discriminates

Does the lawful-class machinery **enforce carrier canonicity**, or admit a
Bottom-inhabiting `sound = Axiom` over a non-canonical carrier? Char (canonical,
PASS) vs Decimal (non-canonical, must REJECT) is the flip — a single accept on
Char alone would be green-vs-green.

## Surface-expressibility note

The `sound` field type (`IsTrue (eq x y) → Equal a x y`) and `Axiom` fields are
landed surface (`lawful_classes.ken`). The exploit's final step needs
`MkDecimalPair` injectivity (K7 no-confusion, landed) and `Bottom`-elim
(`absurd`, K5, landed). If the surface cannot yet *state* the injectivity
projection, document that step as reached-but-not-surface-expressible — the
instance's acceptance is already the finding; the exploit only sharpens it.
