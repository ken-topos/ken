# C7 — quotient type + the `respect` obligation

**Axis:** observational / quotient fragment (K2). **Flavor:** B (should-PASS if
the quotient surface is reachable, else a documented known-gap). **Couples with
C1** — the quotient is *how you soundly do* what C1's naive `DecEq Decimal`
does unsoundly.

## Why this is a blind spot

C1 shows that a value-equality over a non-canonical carrier cannot back a lawful
`DecEq` (it inhabits `Bottom`). The **principled resolution** ADR 0010 names is
a **quotient / setoid**: make the carrier canonical by quotienting the raw pairs
by denotational equality, `Decimal := DPair / denoteEq`. The quotient
eliminator `elim_/` then **demands a `respect` proof** — that any function you
lift out of the quotient gives equal results on related inputs. That obligation
is exactly the check C1's unsound instance skipped. VAL2 never touched the
observational/quotient fragment at all.

## The pair

- **Sound arm — `sound-quotient-respect.ken` — should-PASS (if reachable).**
  `Decimal := DPair / denoteEq`. Lift `isZero : Decimal → Bool` via `elim_/`
  with a **valid** `respect` proof (`isZeroPair` gives the same answer on any
  two pairs that denote the same value). This is the sound analogue of a
  `Decimal` decision procedure.
- **Unsound arm — `unsound-nonrespecting.ken` — should-REJECT.** Try to lift
  `coeff : Decimal → Int` (first projection) via `elim_/`. `coeff` does **not**
  respect `denoteEq`: `MkDPair 10 (-1)` and `MkDPair 1 0` denote the same value
  but have coefficients `10 ≠ 1`. The `respect` obligation is `denoteEq x y →
  Equal Int (coeffPair x) (coeffPair y)` — **unprovable** (it would give `Equal
  Int 10 1`, the same `Bottom` C1 reaches).

## Expected behavior (exact)

- Sound arm: **PASS** — `elim_/` with a discharged `respect` proof lifts
  `isZero` to the quotient. (Kernel support is present: `Term::Quot` /
  `Term::QuotElim { motive, method, respect, scrut }`, `foreign.rs`; surface
  `A / R`, `[t]`, `elim_/` reserved, `11-syntax.md`.)
- Unsound arm: **should-REJECT** — the `respect` obligation for `coeff` is
  **unprovable**; `elim_/` must refuse the lift. **If it accepts a non-
  respecting lift (or lets the `respect` slot be skipped/`Axiom`-ed), that is
  the finding** — the quotient's soundness *is* the `respect` gate, and it is
  the same door C1 exposes.

## Discriminates

Does `elim_/` **enforce** the `respect` obligation? A respecting lift (`isZero`,
PASS) vs a non-respecting one (`coeff`, must REJECT) is the flip. This closes
the C1↔C7 story: C1 = "the naive projection is unsound"; C7 = "the quotient
catches exactly that projection, because it demands `respect`."

## Surface-expressibility note

Quotients are **kernel-level** (`Term::Quot`/`QuotElim`) with **reserved**
surface (`A / R`, `[t]`, `elim_/`). Whether they are **surface-reachable** today
is the open question — the Steward flagged that "quotient/respect isn't yet
surface-expressible" is itself a **legitimate prepared depth-gap result**, not
wasted effort. If the surface can't yet write `A / R` or `elim_/`, record that
as the finding (the quotient fragment is landed in the kernel but not surfaced),
and note that C1's hole stays open until this fix is surface-reachable. Push the
encoding as far as the grammar allows before calling it a gap.
