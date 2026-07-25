---
id: ABI-A1
title: "promote ConsoleRead and ClockWallNow to NativeTested with differential evidence"
status: draft
owner: runtime
size: M
gate: none
depends_on: [ABI-REVOKE]
blocks: []
github: null
origin: docs/program/10-linux-abi-completion.md §4 (the ABI-completion program); node filed by the Steward 2026-07-25 on the operator's directive to frame the remaining program. Agents cannot create tracked work (COORDINATION §2).
---

> ## Authority: `10-linux-abi-completion.md` §4 — read that, not this
>
> ⛔ **This is a tracker/DAG node, NOT a shovel-ready WP frame.** A
> `docs/program/wp/` frame carrying deliverables, acceptance criteria, fixed
> inputs, negative controls, and a contention check **must be authored before
> release** (§2c front-load rule: the T1 enclave does the design judgment so the
> build ring executes mechanically). **Do not release this on the strength of
> this file.**

## Objective

Promote `ConsoleRead` and `ClockWallNow` to `NativeTested` with differential
evidence.

## Why this is its own slice

Track A is split **by evidence shape, not by count.** This slice is
console/clock: **nondeterministic observation**, so it needs a normalized
comparison rather than exact-output equality. That normalization is the whole
judgment content and does not generalize to the other two slices.
