---
id: RT-CONTSPEC-LEDGER
title: "ContinuationSpecialization seam 3 — retire the boundary-use schema: the four BoundaryUse axes are compile-time constants that no lowering, ABI, selection, lifetime, or emission consumer reads, so they are deleted from the continuation-specialization contract"
status: merged
owner: runtime
size: S
gate: none
depends_on: [RT-CONTSPEC-ACTIVATE]
blocks: [RT-CONTSPEC-WITNESS]
github: null
origin: "Architect ownership/sizing ruling evt_1yymw1gdszpbs (2026-08-02), outcome (c) on RT-CONTSPEC-LOWER, seam 3 of four. Recut to schema retirement by Architect ruling evt_1v9m7t4m9dmj7 (2026-08-08), sustaining hard stop 7. Steward-filed (agents cannot create tracked work per COORDINATION section 2)."
---

# Seam 3 — the boundary-use schema is unowned, so it is deleted

Measured at `main = 0fd9f6e8` in
`crates/ken-runtime/src/cranelift_backend/planning/static_transition.rs`, which
is the **entire** occurrence surface (27 `BoundaryUse` hits and 28
boundary-field hits across `crates/`, all in that one file):

- The four boundary-use enums (`BoundaryUsePhase` `:878`, `BoundaryUseOperation`
  `:885`, `BoundaryUseNeed` `:892`, `BoundaryUseAvail` `:899`) each have two
  variants, and in each the **second is `#[cfg(test)]`**.
- `ContinuationInputProjection` has **exactly one** construction site,
  `exact_continuation_projection` `:7491-7494`, which hardcodes all four to the
  one non-test variant.
- **No lowering, ABI, selection, lifetime, or emission consumer reads any of
  them.** Their only other behaviour is test mutation and copy-back in the
  interning-key omission harness.

## Recut 2026-08-08 — from population to deletion

The prior cut told the ring to make the four second variants
production-reachable and prove a distinct-tuple count above 1. Hard stop 7 fired
against it and the Architect **sustained** the stop (`evt_1v9m7t4m9dmj7`).

The fields are not a dormant authority awaiting a mapping. They are an unowned
schema fragment: changing a tuple would intern another specialization key whose
emitted semantics are identical. Populating them *"would manufacture
semantically duplicate units and then call the duplication evidence that the
ledger works."*

⇒ The seam keeps its position and its owner, and its deliverable becomes
deletion: the four enums, the four fields on `ContinuationInputProjection` and
`ContinuationInputView`, the constructor literals, the view-copy path, and the
corresponding omission variants, mutation cases, copy-back cases and four rows
of the key-discrimination control. Removing those rows retires tests for
distinctions production cannot make; it does not weaken a production guarantee.

## `RT-DECL-CLOSURE-PORT` is no longer a dependency authority

The same ruling corrected the Architect's earlier `evt_40ra70t92mjd2`. That
ruling stays correct in its negative parts — no mapping existed, the four
literals were not a classifier, coercion into the binary enums was forbidden —
but its ownership claim was wrong: **`RT-DECL-CLOSURE-PORT` `D7` did not owe a
global boundary-use record.** Its landed `PlannedEffectSeat` population is a
host-operation semantic seat with a deliberately separate vocabulary, and
reading it as a continuation projection would repeat the domain confusion `D7`
was built to prevent.

`RT-DECL-CLOSURE-PORT` is therefore **removed from `depends_on`** and is
historical context only. It is merged regardless, so this changes authority, not
gating.

## Sizing dropped from M to S

A single-file deletion with a closed census and no design left in it. One
checkpoint, not three: `D2` alone does not compile, because the mutation cases
reference the deleted variants.

Frame: `docs/program/wp/RT-CONTSPEC-LEDGER.md`. Branches from `main` and carries
only its own delta.
