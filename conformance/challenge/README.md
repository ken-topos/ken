# Ken challenge suite — the distinctive-depth conformance instrument

> ⛔ **STAGED — DO NOT RUN until the operator is present.** This is a prepared
> instrument, **not** wired into CI and **not** part of the pass/fail
> conformance corpus. The exercises probe soundness/consistency/totality
> boundaries; a *surprising* result (a soundness-boundary probe that **passes**)
> is a design-level finding that is the operator's call. Author + expected
> behavior + staged = done. First execution waits for the operator.
> (Owner: conformance-validator. Frame:
> `docs/program/wp/CV-challenge-prep.md`. Findings/questions → Steward.)

## What this is (and is not)

VAL2 (the Rosetta pangram, 10 PASS / 6 KNOWN-GAP) validated the surface
**breadth** — output, loops, recursion, strings, ADTs+`match`, higher-order
functions, effects. It deliberately did **not** reach the **depth** of Ken's
*distinctive* value. This suite is the **blind-spot instrument** for that depth:
eight exercises on the axes VAL2's breadth pangram didn't touch, chosen so a
surprising result is **unambiguous** rather than green-vs-green.

It does **not** re-probe VAL2's five documented capability gaps (`Map`, `[FS]`,
`[State]`, mutual-recursion, ≥2-recursive-field-`match`). It stresses the
**adjacent depth** and the axes VAL2 skipped entirely.

## The two flavors (read this first — it calibrates every result)

Each exercise is a **discriminating pair**: a *sound* arm that should behave one
way and an *unsound/stub* arm that must behave the opposite way. That is what
keeps a surprising result from being green-vs-green.

- **Flavor A — soundness-boundary probes.** The **correct** behavior is
  **REJECT**. These construct a term the kernel/elaborator *should* refuse
  because accepting it inhabits `Bottom`, breaks consistency, or breaks
  totality. **If the unsound arm is ACCEPTED, that is a genuine hole** — the
  highest-value result, and the reason the operator wants to be present. (C1,
  C2, C3.)
- **Flavor B — capability-depth probes.** The **correct** behavior is
  **PASS** (if the capability is landed) or a **documented known-gap** (if it is
  not yet built). These reach past VAL2's surface for dependent elimination,
  verified programs, and law-carrying instances. A known-gap here is a valid
  prepared result, not a failure. (C4, C5, C6, C7, C8.)

## Expected-behavior legend

Every exercise's `README.md` states, per arm, exactly one of:

- **should-REJECT (reason)** — the elaborator/kernel must refuse it; the reason
  names the exact gate (an unprovable obligation, an Ω sort error, an absent
  former). Acceptance = a hole.
- **should-PASS** — it must elaborate and (where relevant) reduce to the stated
  value.
- **known-gap (reason)** — the capability is not yet landed; the reason names
  the deferred mechanism and its spec/ticket anchor. A clean, expected gap.

## The suite

Format per entry — **`slug` · axis · flavor** — sound arm → / unsound arm →.

- **C1 · `deceq-noncanonical` · lawful classes / canonical carrier · A** —
  `DecEq Char` PASS / `DecEq Decimal` + Bottom-exploit **REJECT**.
- **C2 · `proof-relevant-omega` · Ω / strict-prop boundary · A** —
  `‖Perm‖` / count-eq PASS / `data Perm : Ω` (4-ctor) **REJECT**.
- **C3 · `codata-totality` · coinduction under totality · A** — fuel-bounded
  `unfoldUpTo` PASS / `codata`/`cofix` stream **REJECT**.
- **C4 · `indexed-vec-head` · dependent / indexed families · B** — `head` on
  `Vec (Suc n)` (as far as landed) / impossible-`VNil` head **known-gap**.
- **C5 · `verified-sort` · proof-carrying programs · B** — refinement emits both
  conjuncts PASS / `const Nil` (isSorted-only) **REJECT**.
- **C6 · `lawful-ord-vs-stub` · law-carrying instances · B** — law-**proved**
  `Ord` (zero-delta) / `Axiom`-stub `Ord` (grows trust base).
- **C7 · `quotient-respect` · observational / quotient · B** — quotient elim
  with valid `respect` PASS / non-respecting elim **REJECT**.
- **C8 · `funext-definitional` · observational / funext · B** — pointwise-equal
  functions equal via `Eq`-at-Pi (`\x.x` ≡ `\x. and_bool x True`) PASS /
  same proof shape over functions that **differ at a point** (`\x.True` vs
  `\x.x`) **REJECT**.

C1 and C7 tell **one story**: the naive `DecEq` over a non-canonical carrier
(C1) is unsound; the quotient with a `respect` obligation (C7) is *how you do it
soundly*. C2 and C5 are coupled: C5's `Perm` must sit at the universe C2
establishes (`‖Perm‖`/count-equality, never a proof-relevant Ω inductive). C7
and C8 are the **two OTT faces** — C7 the *quotient* fragment (equality you
*impose*, gated by `respect`), C8 the *funext* fragment (equality that
*computes* pointwise) — Ken's signature observational equality from both sides.

## Grounding

Every expected-behavior is grounded against the landed code / spec at authoring
(`origin/main`), cited per exercise. Surface syntax follows the landed
`packages/lawful-classes/lawful_classes.ken` (classes/instances/`Axiom`/
`IsTrue`/`Equal`), the `es2_acceptance.rs` refinement form
(`{ ys : List a | And (isSorted a leq ys) (Perm a ys xs) }`), K5 `absurd`
(Bottom-elim), and the reserved quotient surface (`A / R`, `[t]`, `elim_/`,
`11-syntax.md`). Where an encoding pushes past the landed surface, that is
called out in the exercise as a **surface-expressibility note** — itself a
prepared depth result, not a defect in the instrument.

## How the operator runs it (when the time comes)

Each exercise dir carries its arm sources (`*.ken`) and a `README.md` with the
per-arm expected behavior + the one-line discriminator. Run each arm through the
`ken` CLI / elaborator and compare against the stated expectation. A
Flavor-A arm that **passes** (should have rejected) is the headline finding.
