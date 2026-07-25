---
id: ABI-S4
title: "statx-shaped metadata with field-availability bits"
status: draft
owner: runtime
size: M
gate: none
depends_on: [ABI-M1]
blocks: []
github: null
origin: docs/program/10-linux-abi-completion.md §4 (the ABI-completion program); node filed by the Steward 2026-07-25 on the operator's directive to frame the remaining program. Agents cannot create tracked work (COORDINATION §2).
---

> ## Authority: `10-linux-abi-completion.md` §4 — read that, not this
>
> ⛔ **This is a tracker/DAG node, NOT a shovel-ready WP frame.** A
> `docs/program/wp/` frame carrying deliverables, acceptance criteria, fixed
> inputs, negative controls, and a contention check **must be authored before
> release** (§2c front-load rule). **Do not release this on the strength of this
> file.**

## Objective

`statx`-shaped metadata **with field-availability bits**.

★ **The availability bits are the deliverable.** `statx` returns a mask saying
which fields it actually filled; a binding that drops the mask silently converts
"not supplied" into a plausible zero. Depends on `ABI-M1` because the record
layout per enabled family is generated there, not restated here.
