---
id: RT-CONTSPEC-ABI
title: "ContinuationSpecialization slice 2 — land the explicit unit/descriptor projection and the ABI, owner/lifetime/affinity and zero-allocation negative gates, still DORMANT"
status: draft
owner: runtime
size: M
gate: none
depends_on: [RT-CONTSPEC-PLANNER]
blocks: [RT-CONTSPEC-LOWER]
github: null
origin: "Architect second WIP audit evt_4t09329vdrf (2026-08-01), outcome (c). Slice 2 of the Steward's staged recut of RT-RECURSOR-TRANSPORT + RT-DECL-CLOSURE-PORT D7."
---

> # ⏳ SLICE 2 OF 4 — `draft` UNTIL ITS FRAME IS WRITTEN
>
> ⛔ **`status: draft` here means the FRAME is owed, not that the work is
> unclear.** The Steward writes it while [[RT-CONTSPEC-PLANNER]] is in flight,
> per the standing rule that every immediate successor of an in-flight node is
> fully framed before that node merges. ⛔ Do not pick this up from the node
> text; wait for `docs/program/wp/RT-CONTSPEC-ABI.md` and the release post.

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
