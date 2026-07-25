---
id: ABI-S3
title: "monotonic clocks, sleep/deadlines, and secure kernel entropy"
status: draft
owner: runtime
size: M
gate: none
depends_on: []
blocks: [PX12]
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

Monotonic clocks, sleep/deadlines, and secure kernel entropy.

## ⭐ Why this one is special — it is startable NOW

**`ABI-S3` and `ABI-R1` are the ONLY two nodes in this program with no
dependency on `PX8`.** Everything else in §5 descends from it. With the fleet
single-threaded on `RT-NATIVE-FNSPLIT`, these two are the only available parallel
ABI work.

⚠ **It is not isolated downstream, though — `ABI-S3` gates `PX12`.** Landing it
early removes one of the three inputs to the committed exit, so doing it now is
critical-path work, not filler.

⛔ **Monotonic is the point.** A deadline built on a wall clock is wrong across
adjustment; do not let `ClockWallNow` (`ABI-A1`) stand in for it.
