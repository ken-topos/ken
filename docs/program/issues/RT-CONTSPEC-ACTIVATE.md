---
id: RT-CONTSPEC-ACTIVATE
title: "ContinuationSpecialization seam 2 — lowering activation and exact-use consumption: direct call before the identity-erasing join, active emitted owner, affine call occurrence, JoinArm consumption, gating the 37-row lower-owned population"
status: ready
owner: runtime
size: L
gate: none
depends_on: [RT-CONTSPEC-ASSEMBLY]
blocks: [RT-CONTSPEC-LEDGER]
github: null
origin: "Architect ownership/sizing ruling evt_1yymw1gdszpbs (2026-08-02), outcome (c) on RT-CONTSPEC-LOWER, seam 2 of four. Steward-filed (agents cannot create tracked work per COORDINATION section 2)."
---

# Seam 2 — the activation, scoped to the population that actually owns it

This is the seam that turns the mechanism on. Its population is the Architect's
**37 lower-owned rows**: inactive emitted owner, missing `JoinArm` use,
duplicate declared-unit call, and runtime discriminator failure.

## What it may not touch

- **No planner or ABI repair.** Those surfaces stay at their landed blobs. A
  planner/ABI-worded refusal here routes back as a new exact interface hard
  stop under seam 4's rule; it is not repaired in place.
- **No D7 population expansion.** Seam 3 owns the ledger and representation
  rows. Widening into them here rebuilds the mis-sizing that produced this recut.

## Why the row count is a scope oracle and not an estimate

The 37 rows come from the Architect's ownership matrix over the exact census run,
not from a plan. If this seam's work reaches rows outside that 37, the partition
is wrong and that is a hard stop, not a scope adjustment.

Branches from `main` after seam 1 lands, and carries only its own delta.

## Two things the frame settles that this node must not be read without

**The scope oracle is seam 1's `D4`, not the `46d29783` census.** That census
labels only 35 of the 37 lower-owned rows; the other 2 sit inside the 39 rows it
records as "ownership matrix pending." Selecting from it silently drops two rows.

**"37" names two disjoint populations.** Lower-owned is 37 rows; `producer
callable identity is not a Closure` is also 37 rows and belongs to the
**forbidden** slice-1 planner closure. Select by first-refusal kind and owner
label together, never by count.

The frame is `docs/program/wp/RT-CONTSPEC-ACTIVATE.md` — written 2026-08-02 while
seam 1 is in flight (section 2a-bis). Node is `ready`; release is gated on seam 1
merging, not on further framing.
