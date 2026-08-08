---
id: RT-CENSUS-CAVEAT-GUARD
title: "The identifier-census caveat's staleness guard is an existence check standing in for a count check, so it cannot detect the drift it was written to catch"
status: draft
owner: runtime
size: S
gate: none
depends_on: []
blocks: []
github: null
origin: Reported by runtime-implementer during RT-RECURSOR-TRANSPORT D3 (evt_16cmej481q7ns family) as an incidental finding; Architect scoped it OUT of the hard-stop-4 repair nodes at evt_3r4j14fv1jtj2 as pre-existing prose debt. Steward-filed (agents cannot create tracked work per COORDINATION §2), and re-derived by the Steward at main 5d4c623e rather than inherited.
---

> ## STATUS CORRECTED `ready` TO `draft` — 2026-08-08, Steward
>
> **`ready` means shovel-ready: a written frame, dependencies merged.**
> This node was not, because
> no frame exists.
>
> **The correction is not a downgrade of the work.** A node advertising
> startable work it does not have makes the backlog read deeper than it
> is, and that depth is exactly what a Steward reads to decide whether a
> team is idle for want of work or for want of a lane.

## What it is

`crates/ken-runtime/src/cranelift_backend/lowering/core/tests/control.rs:5083`
carries an honest-limit caveat that says the identifier census does not
partition out **`core.rs`'s 22 inline `#[cfg(test)]` regions**.

**The number is wrong on every tree anyone has measured:**

| tree | inline `#[cfg(test)]` in `core.rs` |
|---|---|
| `main` `5d4c623e` (Steward, re-derived) | **180** |
| `8efdfdb3` (implementer) | **205** |
| `10369776` (implementer) | **200** |
| what the caveat claims | **22** |

It matched **no** tree, so it was **false when written**, not merely stale.

## The actual defect — and it is not the number

Immediately below the caveat sits a guard that exists specifically to catch the
caveat going stale:

```rust
assert!(
    core.contains("#[cfg(test)]"),
    "the caveat above describes inline cfg(test) regions that are no longer \
     present, so it has gone stale and must be re-derived"
);
```

**That guard fires only when the count reaches zero.** It is an **existence**
check standing in for a **count** check, so it passes identically at 1 region
and at 205. **It cannot detect the drift it was written to catch, and it never
could** — the failure is structural, not a case of the guard not having run yet.

⇒ **The number being wrong is a symptom. The guard being keyed on the wrong
property is the defect**, and it is the reusable one: a staleness guard must
assert the property whose drift it is guarding, not a weaker property that
happens to be true whenever the stronger one is.

**The guard also carries a false claim in its own message** — *"regions that are
no longer present"* describes the zero case, which is the only case it detects,
while the caveat above it is wrong in the opposite direction: far **more**
regions than claimed, not fewer.

## What survives, and what does not

**The safety direction survives.** The caveat says the unpartitioned regions err
toward a **false red, never a false green**, and that remains true — a call
added inside a `#[cfg(test)]` region is counted as production, which
over-reports. The census is still sound in the direction that matters.

**The magnitude does not.** A reader budgeting for 22 unpartitioned regions and
one budgeting for 180 are making materially different judgements about how much
slack this census has.

## Deliverables

1. **Re-derive the count at the candidate's own base** and correct the caveat.
   Do not copy any figure from this file — three of the four numbers above were
   measured on trees that no longer exist, and this node exists because a
   figure was inherited rather than derived.
2. **Replace the existence guard with one keyed on the count.** It must fail
   when the count drifts from the documented figure, in **either** direction,
   not only when it reaches zero.
3. **Correct the guard's own message**, which currently describes only the
   vanishing case.

## Acceptance criteria

- **AC-1 — the documented count equals the measured count at the final SHA.**
  *Control:* the guard asserts it; the handback states the command used.
- **AC-2 — the guard discriminates.** A deliberate off-by-one in the documented
  figure **reds** the control, and correcting it greens it.
  *Control:* a mutation proof, from the committed tree. **This is the AC that
  matters** — an existence check would pass the mutation, which is exactly how
  the current guard got here.
- **AC-3 — the caveat's direction claim is preserved.** The false-red-never-
  false-green statement stays; only the magnitude is corrected.
- **AC-4 — CI green.** Not a local `--workspace` run (`COORDINATION §12`).

## Scope

**One file, prose and one assertion.** No change to the census algorithm, and
no change to what the surrounding test measures. If the correct count turns out
to require partitioning the regions out of the census for the control to be
meaningful, **stop and route it** — that is a different and larger node.

## Contention

`core/tests/control.rs` — contended with [[RT-MATCH-RECURSOR-CONSUMERS]],
[[RT-LEXICAL-RECURSOR-CONSUMERS]] and [[RT-RECURSOR-TRANSPORT]] `D3`. **Schedule
it after that arc clears**, or accept a rebase. It blocks nothing, so there is
no reason to contend.
