---
id: RT-CONTSPEC-LEDGER
title: "ContinuationSpecialization seam 3 — make the boundary-use ledger record something: the four boundary-use axes are compile-time constants in production, so the ledger distinguishes no two continuation inputs"
status: ready
owner: runtime
size: M
gate: none
depends_on: [RT-CONTSPEC-ACTIVATE]
blocks: [RT-CONTSPEC-WITNESS]
github: null
origin: "Architect ownership/sizing ruling evt_1yymw1gdszpbs (2026-08-02), outcome (c) on RT-CONTSPEC-LOWER, seam 3 of four. Steward-filed (agents cannot create tracked work per COORDINATION section 2)."
---

# Seam 3 — the ledger's vocabulary exists only under `cfg(test)`

Measured at `main = cef564f1` in
`crates/ken-runtime/src/cranelift_backend/planning/static_transition.rs`:

- The four boundary-use enums (`BoundaryUsePhase` `:417`, `BoundaryUseOperation`
  `:424`, `BoundaryUseNeed` `:431`, `BoundaryUseAvail` `:438`) each have two
  variants, and in each the **second is `#[cfg(test)]`**.
- `ContinuationInputProjection` has **exactly one** construction site,
  `exact_continuation_projection` `:2754`, which hardcodes all four to the one
  non-test variant.

⇒ Every continuation input the planner produces carries the identical
boundary-use tuple. The ledger has one row shape and records no distinction.

## The finding worth carrying: the control is green and unreachable

`continuation_keys_equal_under_mutation` `:2832` already proves, per field, that
the interning key separates two units differing only in that field. That proof
is real and must keep passing. But `ContinuationProjectionOmission` `:531` is
itself `#[cfg(test)]`, and the harness reaches a distinct value by flipping to
the `#[cfg(test)]` variant.

⇒ **The discrimination is proved over a value production cannot construct.** The
seam's subject is not a missing mechanism — it is an instantiated key over an
uninstantiated vocabulary. A green boundary control today is not evidence the
ledger works.

## Recut 2026-08-02 — flipped `draft` to `ready`

The prior cut selected 17 D7-owned rows from the `46d29783` first-refusal
census. That census is a historical record from the held `1aef3192` lineage and
cannot name a current source authority; seam 2 was recut off it for the same
reason (`evt_2zhx69f2fw07w`, Architect confirmation `evt_66t42tapvdbsj`). The
census is no longer an input to any deliverable or AC.

The discriminator is now `D5`: distinct boundary-use tuples over a fixed
fixture, **exactly 1 on the base and greater than 1 on the candidate**. A
candidate census of 1 fails the seam.

Release is gated on seam 2 merging via `depends_on`, not on further framing.
Seam 2 may edit `static_transition.rs`, so the frame opens with a mandatory
re-measurement and a hard stop if it disagrees.

Frame: `docs/program/wp/RT-CONTSPEC-LEDGER.md`. Branches from `main` after seam 2
lands and carries only its own delta.
