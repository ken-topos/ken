---
id: RT-SITEOP-CARRIED-WITNESS
title: "Site-bound operand reader cannot witness a carried value — a synthesized SiteOperand demands a compile-time Lowered template from the same seat byte-span activation wants carried"
status: draft
owner: runtime
size: L
gate: none
depends_on: [RT-CARRIER-BYTESPAN-OBSERVE]
blocks: []
github: null
origin: Hard stop returned by RT-CARRIER-BYTESPAN-OBSERVE D5, 2026-08-07, candidate 4244d082. The frame's own §1a recut clause fired — the 30 quarantined rows do not discharge from one mechanism. Steward-cut per that clause. Steward-filed (agents cannot create tracked work per COORDINATION §2).
---

> ## THIS NODE IS `draft` BECAUSE A DESIGN FORK IS OPEN, NOT BECAUSE IT IS UNSCOPED
>
> The gap is measured and confirmed twice, independently. **What is not settled
> is the mechanism**, and that is the Architect's call, not the Steward's. This
> node goes `ready` when the Architect has ruled the fork in §3 of its frame.
>
> **Do not read `draft` as "not yet investigated."** The investigation is done
> and is recorded below.

## The gap

Each `Fs*` path seat is consumed **twice**:

1. as a **wire span** — which `RT-CARRIER-BYTESPAN-OBSERVE`'s `D4` observer
   satisfies at every measured seat; and
2. as **`SiteOperand(0)`** of the synthesized `FileError`'s
   `Option::Some(<site path>)`, which demands a **compile-time `Lowered`
   template**.

Supplying (2) from a boundary word is the `Carried -> Lowered` inverse that §5
bans. So the same seat cannot be both `EITHER_PHASE` and a site-bound operand.

```rust
// lowering/mod.rs:11354-11362 — the sole template projection
fn site_operand_argument(&self, seat: StaticOriginId, index: u32,
                         seats: &ClaimedEffectSeats<'_>) -> Result<..> {
    let value = seats.specialized(EffectSeatSlot::Argument(index))?.clone();
    //                 ^^^^^^^^^^^ requires the compile-time template
```

`mod.rs:11650-11654` states the consequence in its own voice: a declared
`SiteOperand` whose claimed operand is carried *"refuses at that exact seat,
propagated from `specialized`. It does not reconstruct a template, widen the
carrier, borrow a sibling, or fall back — reconciliation needs a compile-time
witness, and there is none."*

## How it was established

**Two independent routes, which is why it is stated as measured rather than
diagnosed.**

- **Runtime implementer, stepwise at `4244d082`:** baseline refuses at
  `FsWriteFile Argument(0)`; flipping `Argument(0)` moves the refusal to
  `Argument(2)`, proving seat 5 is real; flipping both returns it to
  `Argument(0)`, now from the template projection, past the claim gate. All 26
  lowering refusals across ten files reduce to this one cause, with **zero
  failures of any other kind.**
- **Steward, structurally:** the two source sites above, read directly. Not a
  re-run of the implementer's measurement — a different route to the same
  place.

## What it owns

- **29 of the 30 `#[ignore]` rows** quarantined under
  [[RT-CARRIER-BYTESPAN-OBSERVE]], across 10 files.
- **The four seats left `SPECIALIZED_ONLY`** by that node's `D5`:
  `(FsReadFile, 0)`, `(FsWriteFile, 0)`, `(FsChangeMode, 0)`, `(FsOpen, 0)`.
- **The `D6` activation-gate discharge pass**, moved here because its premise
  is "the activation", and this node is where the activation completes.

## Frame

`docs/program/wp/RT-SITEOP-CARRIED-WITNESS.md`.
