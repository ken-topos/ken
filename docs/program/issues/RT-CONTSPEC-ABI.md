---
id: RT-CONTSPEC-ABI
title: "ContinuationSpecialization slice 2 — land the explicit unit/descriptor projection and the ABI, owner/lifetime/affinity and zero-allocation negative gates, still DORMANT"
status: merged
owner: runtime
size: M
gate: none
depends_on: [RT-CONTSPEC-PLANNER]
blocks: [RT-CONTSPEC-LOWER]
github: null
origin: "Architect second WIP audit evt_4t09329vdrf (2026-08-01), outcome (c). Slice 2 of the Steward's staged recut of RT-RECURSOR-TRANSPORT + RT-DECL-CLOSURE-PORT D7."
---

> # ✅ MERGED 2026-08-01 — PR #1303, `origin/main = f395ee5a`, CI GREEN
>
> Exact candidate `ed527eb774fa7f7ef9e274f8c8b428b980373563`, tree
> `e5f67676c432399c1adbbdc7174d7fcc209d7665`, base `139cf89b`, +663/−29 across
> exactly two files. QA `evt_74mmepexexppy`; Architect `evt_5396qdh7a7p32`;
> Decision `dec_77b33m1pbahng` resolved. All three predicted post-conditions
> exact — `abi.rs` blob `23b9f5d778bf98fbb2907cf087bf06da30d82e7d`,
> `static_transition.rs` blob `6725971de7e7a25c5bc81cab48baca6d65978f43`, and
> those the only two paths changed.
>
> ## ⭐⭐ APPROVED ON THE FIRST ROUND — AND THAT IS THE FINDING
>
> **Slice 1 took four review rounds; this took one, with nothing deferred.**
> ⛔ That is not slice 2 being easier in itself — it is what slices 0 and 1 had
> already fixed. ⭐ **The staged recut is paying for itself**, which is the
> argument for holding [[RT-CONTSPEC-LOWER]] to the same control standard rather
> than treating a cheap review as licence to relax.
>
> ## ⛔ WHAT THIS DID **NOT** ESTABLISH — measured for slice 3
>
> **Neither gate reaches a lowered call.** `validate_continuation_specializations`
> validates the descriptor **table** (count, `id == index`, arm identity). The
> `D4` zero-allocation gate is scoped to the **install** path in its own words
> (`abi.rs:597–603`) — it refuses *capacity growth while appending*, and is
> silent about allocation on a lowered call. ⇒ ⭐ **Slice 3 inherits nothing at
> the call site.** Recorded in that node and its frame.
>
> ⚠ The specialization remains **dormant**: constructed, projected and checked;
> nothing on a live path consumes it. [[RT-CONTSPEC-LOWER]] is the slice that
> activates.

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
