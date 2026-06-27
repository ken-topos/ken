# ADR 0005 — Observational equality (OTT), not cubical

- **Status:** Accepted
- **Date:** 2026-06-27
- **Deciders:** the operator

## Context

The DRAFT spec (`OQ-4`) committed Ken's identity/equality layer to **full
cubical type theory** (interval, cofibrations, `transp`/`hcomp`/`comp`, `Glue`,
computational univalence, higher inductive types), because it makes `J` reduce
on non-`refl` paths and gives funext/univalence as computing theorems. That
fixed the prototype's "`J`-only-on-`refl`" gap, but it is the **largest single
piece of the trusted kernel**, and it provides power (univalence, higher HITs)
that **software** verification does not use.

A scoped research review (the operator's question + a web-research agent,
recorded in `local/`) compared **(A) full cubical**, **(B) modern Observational
Type Theory (OTT — Pujet–Tabareau `TTobs`; `CICobs` "Observational Equality
Meets CIC", TOPLAS 2025; `CCobs`)**, and **(C) middle grounds (XTT,
cubical-without-Glue, setoid TT)** against Ken's must-haves: computing
transport/rewrite, funext, set-quotients, propositional truncation,
proof-irrelevant propositions, canonicity, decidable checking — **without**
univalence or higher HITs. Findings:

- **OTT's design target *is* exactly that must-have list** — minus univalence,
  which it deliberately excludes (UIP-incompatible) and which Ken doesn't need.
- **Trusted-kernel economy:** OTT's equality is **defined by recursion on type
  structure** with a `cast` coercion + a proof-irrelevant `SProp` universe — far
  smaller and more auditable than cubical's interval/cofibration/`hcomp`/`Glue`
  engine.
- **Metatheory is settled:** OTT's canonicity and **decidable conversion** are
  proven (machine-checked / model-justified); cubical holds these too but via a
  much heavier argument.
- **Soundness track record favours OTT for an *adversarial* setting:** cubical
  has a documented history of **canonicity-breaking bugs accepted under
  `--safe`** (`transp`-over-`Glue` proving ⊥, HIT-indexed ⊥, a positivity hole
  giving Curry's paradox, `SProp`×cubical regressions, non-constructive
  regularity). These are exactly the dark corners **agent-generated proofs will
  probe**. OTT's known issues are engineering rough edges (fording for indexed
  inductives, rewrite-rule plumbing), not core canonicity failures.
- **Maturity caveat:** OTT has **no production kernel yet** (CICobs/CCobs are
  research prototypes); cubical Agda is mature. But Ken is **building its own
  small kernel**, so "mature off-the-shelf" is not the relevant axis — the
  relevant axis is "smallest, most auditable foundation that meets the
  requirements," and the references (CICobs, CCobs) are concrete blueprints.

## Decision

**Ken's identity/equality foundation is observational (OTT/`TTobs`-style), not
cubical.** Concretely:

1. **Propositional equality is observational `Eq A a b`**, a proposition
   computed **by recursion on the structure of `A`** (e.g. `Eq` of functions is
   pointwise `Eq` — *funext is definitional*; `Eq` of pairs is componentwise;
   `Eq` of inductives is structural). `refl a : Eq A a a`.
2. **`cast` (type coercion / transport)** `cast A B (e : Eq Type A B) : A → B`
   computes by recursion on the type structure and **reduces on reflexivity**
   (`cast A A refl a ≡ a`). It derives `subst`/`J`, which therefore **compute on
   non-`refl`** — closing the prototype's gap, now via observational equality
   rather than cubical paths. The equality eliminator **does not inspect the
   equality proof** (it computes from the endpoints), which is *why* OTT
   tolerates added consistent axioms without breaking canonicity.
3. **Ω is the strict, proof-irrelevant proposition universe (`SProp`).** `Eq`
   lands in Ω, so **proof irrelevance and UIP are definitional**, and Ken is a
   **set-level** theory (which is what software is). Ω keeps its Heyting
   structure.
4. **Native set-quotients** `A / R` (`R : A → A → Ω`) and **propositional
   truncation** `∥A∥` (a squash into Ω) provide the data-structure and logical
   needs HITs would have — at set level.
5. **Dropped:** the interval, cofibrations, partial elements, `transp`/`hcomp`/
   `comp`, `Glue`/`unglue`, **computational univalence**, and **higher**
   inductive types. (`PathP`, the interval grammar, the de Morgan/face machinery
   — all gone.)
6. **Foundation, not surface:** this lives in the kernel (`Eq`, `cast`, Ω,
   quotients are kernel primitives), but it is **smaller** than the cubical
   kernel it replaces — net reduction of the TCB.

## Consequences

- **Smaller, more auditable trusted kernel** — directly serves the tier-1
  security / small-TCB goal (ADR 0004, ADR 0001). Fewer canonicity-critical
  surfaces for agent-generated proofs to break.
- **`OQ-4` is resolved** (observational, not cubical) and ceases to be open.
- **`OQ-Prop` is revised.** It earlier chose "derived Ω, *propositional* proof
  irrelevance, no `SProp`" *in the cubical setting*. Under OTT, **Ω *is* a
  strict proof-irrelevant universe** — so proof irrelevance is now
  **definitional**, as a *foundational, free* part of the smaller OTT kernel (it
  even *helps* agent-generated proofs: equality goals discharge definitionally,
  so agents synthesise fewer coherence terms). Impredicativity stays **ruled
  out** (OTT's Ω is predicative).
- **`OQ-eval-strategy` is unaffected in spirit and better-fitting in fact:** the
  Lean-style lazy-WHNF + NbE engine stands; the value domain now computes
  **observational `Eq`/`cast`** (and SProp proof irrelevance) instead of cubical
  operations — and OTT is *closer* to Lean's (non-cubical) setting than cubical
  was.
- **No univalence.** Generic transport across *type equivalences* is not
  available; for set-level software this is a non-loss (it was the mathematics
  feature).
- **Spec edits:** rewrite `10-kernel/15` (identity) and `10-kernel/16` (cubical
  → **observational**, file renamed); update `11` (syntax), `12` (universes/Ω),
  `14` (indexed inductives via `cast`/fording; quotients), `17` (conversion),
  `README`/`18`; ripple terminology (Path→`Eq`, HITs→quotients, drop univalence)
  across `00`, `20`, `30`, `40`, `50`, `60`, the register, conformance, and the
  strategy/roadmap docs.

## Implementation blueprints

- **`CICobs`** ("Observational Equality Meets CIC", TOPLAS 2025) — decidable
  conversion, indexed inductive types via `cast`, quotients; a Coq-fork
  prototype.
- **`CCobs`** (TYPES 2024) — an OTT kernel with **NbE + a bidirectional
  checker** and quotient/inductive types — directly aligned with
  `OQ-eval-strategy`.
- **`TTobs`** (Pujet–Tabareau, POPL 2022) — the core metatheory (normalization,
  algorithmic canonicity, Agda-formalised).
- Watch **QITs-in-OTT** (Felicissimo–Tabareau, TYPES 2025) if richer quotients
  are wanted later.

## Revisit if

- A concrete need for **univalence / higher-dimensional** structure appears (it
  is not expected for software) — that would reopen the cubical-vs-observational
  question wholesale.
- OTT's metatheory for a feature Ken needs (e.g. a specific
  quotient/indexed-family pattern) turns out to be genuinely open at
  implementation time — fall back to the CICobs-proven subset and defer the
  rest.
