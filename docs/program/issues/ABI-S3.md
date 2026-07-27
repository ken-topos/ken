---
id: ABI-S3
title: "monotonic clocks, sleep/deadlines, and secure kernel entropy"
status: merged
owner: runtime
size: L
gate: none
depends_on: []
blocks: [PX12]
github: null
origin: docs/program/10-linux-abi-completion.md §4 (the ABI-completion program); node filed by the Steward 2026-07-25 on the operator's directive to frame the remaining program. Agents cannot create tracked work (COORDINATION §2).
---

> ## ✅ MERGED 2026-07-27 — `origin/main = 312a22dd`, PR #1073
>
> Candidate `e60ab3645f3a03719af1aeec13b7300d95030eb8`, published unchanged.
> **CI checks passed** and the currency checker is green on `origin/main` — the
> full-workspace / `--locked` / conformance gate, which only ever runs there.
> Landed content verified by **blob identity** (`ken-verify/src/host.rs`
> `8555aab2`, `ken-host/src/effect_v1.rs` `374356f3`,
> `wp/ABI-S3-report.md` `4fefd09a`), each with a discriminating pre-merge control.
>
> Decision `dec_7yh8pk6w77bm8` read `resolved` fresh from the object
> (`resolved_by=agt_37reqftfe6g00`, 07:00:57Z) before publishing.
>
> ⭐ **Two QA blocks, and the second found a production defect.**
> `decode_deadline` read `args.first()`, so a forbidden cancellation field on the
> surface `Deadline` was **discarded during decoding** — upstream of every control
> that existed, all of which stayed green. The decisive artifact is the *contrast*:
> the new elaborated-telescope control fails while the host request triad stays
> green. ⚠ `SleepUntil`'s and `RandomBytes`' operand reads had the identical
> `first()` shape, so the repair was made **as a class**, wider than the block.
>
> ⚠ **Root cause worth carrying:** the governing frame was amended on
> `origin/main` after the branch was cut, and the branch's own copy read as
> complete and self-consistent — no conflict, no error, no signal. Bind a frame
> **by blob from `origin/main`**, never by the worktree path.
>
> ⛔ Per operator direction (2026-07-27), no retro is owed and none was collected.

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
