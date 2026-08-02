---
id: RT-CONTSPEC-ACTIVATE
title: "ContinuationSpecialization seam 2 — lowering activation and exact-use consumption: direct call before the identity-erasing join, active emitted owner, affine call occurrence, JoinArm consumption, gating the 37-row lower-owned population"
status: draft
owner: runtime
size: L
gate: none
depends_on: [RT-CONTSPEC-ASSEMBLY, RT-WORKER-BIND]
blocks: [RT-CONTSPEC-LEDGER]
github: null
origin: "Architect ownership/sizing ruling evt_1yymw1gdszpbs (2026-08-02), outcome (c) on RT-CONTSPEC-LOWER, seam 2 of four. Steward-filed (agents cannot create tracked work per COORDINATION section 2)."
---

# Seam 2 — the activation, scoped to the population that actually owns it

> ## FROZEN 2026-08-02 — outcome (c), mis-sized at the target-binding boundary
>
> **Architect `evt_2anwskscqz5fg`, correcting its own `evt_2a6dgqcn0yss4`.** Do
> not start, resume, or QA this seam. `status` is `draft` and `depends_on` now
> names `RT-WORKER-BIND`, so it cannot enter the frontier by either route.
>
> **`dd0ca60e00baf9a413c397cf6892a8cf7fa23688` is preserved and
> preservation-only.** It is not a base. Do not delete the consumer route, add
> the four-field projection in isolation, fabricate a payload, begin D4/D5, or
> route QA.
>
> **Retained as proved:** the function-scoped `PredeclaredFunctionId` transport
> verified at `evt_53vaz7s3mg19r`, and the exact construct/alternative/sequence
> discriminator direction.
>
> **Falsified — do not inherit these:**
>
> - D2's `define_continuation_target` body treats function parameter 0 as the
>   payload, loads the `Result` slot before the caller initializes it, and XORs
>   that uninitialized word with every Parameter/Capture word. Exact source
>   falsifies it; **the accepted checkpoint label does not make it a
>   definition.**
> - the consumer-side `continuation_claim_for(static_origin, index, index)`
>   route — the consumer `ComputationalMatch` origin is not the producer
>   `Construct` origin, and case position is not sequence authority.
> - **the three-field binding key, which was mine.** The planner mints one call
>   token per ruled recursive position, so the key needs a fourth field,
>   `recursive_position`.
>
> **Why it is frozen rather than re-amended:** the remaining target cannot be
> bound through any existing route. Lowering cannot bind a worker's carried
> capture operands into the selected semantic body, and every discharge
> available today breaks already-landed law. That is a substrate absence, not
> an implementation correction, and it is `RT-WORKER-BIND`.
>
> ⇒ **Three of this seam's four stops trace to this frame. The fourth is the
> one that found the real defect**, and it was found because the implementer
> refused to invent the API rather than guessing one that type-checked.

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
