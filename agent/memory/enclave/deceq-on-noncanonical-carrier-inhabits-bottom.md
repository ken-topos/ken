---
scope: enclave
audience: (see scope README)
source: private memory `deceq-on-noncanonical-carrier-inhabits-bottom`
---

# A DecEq instance over a non-canonical carrier is genuinely unsound

A lawful class whose law ties the op to the kernel's **propositional `Equal`** —
`DecEq.sound : (x y) → IsTrue(eq x y) → Equal a x y`, or
`Ord.antisym : … → Equal a x y` — is **only deliverable over a CANONICAL
carrier** (one carrier value per denoted value). Over a **non-canonical**
carrier (many representations per value) the law is a **false meta-theorem**, so
postulating it (`sound = Axiom`) **inhabits `Bottom`** — a real false-proof
hole, not a wrong value.

**The discriminator is carrier canonicity, per instance — not the type:**
- **`DecEq Decimal` UNSOUND.** `Decimal = MkDecimalPair Int Int` is
  non-canonical (`(10,-1)` and `(1,0)` both denote `1`). `decimalEq` is a
  value-**equivalence** *coarser* than `Equal Decimal`:
  `decimalEq (10,-1) (1,0)` reduces `True` on structurally-distinct pairs, while
  `MkDecimalPair` injectivity (no-confusion/K7) gives
  `Equal Decimal … → Equal Int 10 1 → Bottom`. So `sound` inhabits Bottom.
  `decimalEq` is an **`Eq`** (equivalence: refl/sym/trans as Bool-equations, NO
  `Equal`-tie), **never a `DecEq`**.
- **`DecEq Char` / `Ord Char` SOUND.** `Char = {c:Int|isScalar c}` is canonical
  (one Int per codepoint; `proj` = identity, `isScalar` Ω-irrelevant), so
  `Equal Char ≡ Equal Int` definitionally and `Ord Int`/`DecEq Int`'s Axioms
  transport soundly (zero-NEW-delta). Same *type family* (Char refinement) as
  Decimal's `Prod`, opposite verdict — **canonicity, not inductive-vs-opaque, is
  the axis here.**

**Key rule:** the "flips vs a deceptive stub, **never** against an honest
visible `Axiom`" discriminator (two arm producer needs a case per arm,
lawful-class honesty) has a **precondition**: the Axiom must be a **TRUE**
meta-theorem (like `Ord Int`'s laws). If the law is false on the carrier there
is **no** honest Axiom — the instance is not deliverable at all, and the fix is
re-defer to a **canonicalize-or-quotient/setoid** design call, NOT a
"bottoms-at-a-landed-floor" structural proof.

Live: Architect surfaced it on the lawful-lane Decimal WP; I independently
re-derived the Bottom witness and corrected my OWN just-landed AC-D3 over-claim
("DecEq Decimal bottoms at the landed DecEq Int Axiom") across 6 coupled seed
sites (`wp/num-landedness-erratum` Fix-2, `0198110`). No unsound code shipped —
the build correctly HELD before authoring the instance; only the doc over-claim
existed. Same absolute-honesty family as reconcile own over claim then grep
coupled and the carrier-axis line proof relevant inductive cannot be declared at
omega.
