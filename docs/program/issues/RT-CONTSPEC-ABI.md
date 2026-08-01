---
id: RT-CONTSPEC-ABI
title: "ContinuationSpecialization slice 2 — land the explicit unit/descriptor projection and the ABI, owner/lifetime/affinity and zero-allocation negative gates, still DORMANT"
status: ready
owner: runtime
size: M
gate: none
depends_on: [RT-CONTSPEC-PLANNER]
blocks: [RT-CONTSPEC-LOWER]
github: null
origin: "Architect second WIP audit evt_4t09329vdrf (2026-08-01), outcome (c). Slice 2 of the Steward's staged recut of RT-RECURSOR-TRANSPORT + RT-DECL-CLOSURE-PORT D7."
---

> # ▶ SLICE 2 OF 4 — THE FRAME IS WRITTEN; THIS IS NOW `ready`
>
> ⭐ **`docs/program/wp/RT-CONTSPEC-ABI.md`** (written 2026-08-01, while slice 0
> was in flight). ⛔ **Read the frame, not this node.**
>
> ⚠ **`ready` means shovel-ready, NOT startable now.** `depends_on` names
> [[RT-CONTSPEC-PLANNER]], and the chain is strict: slice 2 branches from
> `main` **after slice 1 lands**. ⛔ Do not cut a branch before then, and ⛔ do
> not start on a Steward post alone — wait for the release.

## Scope, as ruled

Land the explicit `ContinuationSpecialization` unit/descriptor projection and
the exact ordinary ABI, plus the owner/lifetime/affinity and **zero-allocation
negative gates**.

⛔ **Still no dynamic branch activation.** ⛔ **Zero callable/control identity in
runtime data** — that boundary is the point of keeping this slice dormant.

## Why it is separable from slice 3

The descriptor and ABI projection are **statically checkable** against the
planner closure that slice 1 lands. They do not need a lowered call to exist, so
they do not need the branch route, the join, or a runtime witness. ⇒ A defect
here surfaces as a failing gate rather than as an exit code from a native
binary, which is the whole reason the audit's finding was about *shape*.

Prototype reference (⛔ not acceptance evidence):
`origin/preserved/rt-recursor-freeze-465fab90`.
