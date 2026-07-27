---
id: ABI-S3
title: "monotonic clocks, sleep/deadlines, and secure kernel entropy"
status: ready
owner: runtime
size: L
gate: none
depends_on: []
blocks: [PX12]
github: null
origin: docs/program/10-linux-abi-completion.md §4 (the ABI-completion program); node filed by the Steward 2026-07-25 on the operator's directive to frame the remaining program. Agents cannot create tracked work (COORDINATION §2).
---

> ## ✅ FRAMED AND RELEASABLE — 2026-07-27
>
> ⭐ **The shovel-ready frame is
> [`docs/program/wp/ABI-S3-monotonic-clocks-deadlines-entropy.md`](../wp/ABI-S3-monotonic-clocks-deadlines-entropy.md)
> — build from that, not from this node.** It carries fixed inputs measured on
> `origin/main = d359fb66`, the four front-loaded design judgments (D1–D4), six
> deliverables, six acceptance criteria with negative controls, the contention
> check, and the do-not-reopen list.
>
> Authority remains `10-linux-abi-completion.md` §4 Track S. The §2c front-load
> obligation is **discharged**: the previous banner here correctly refused
> release on the strength of this file alone, and the frame it required now
> exists.

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
