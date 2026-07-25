---
id: ABI-R1
title: "correct stale filesystem capability prose — scoped roots, rights, symlink policy and no-follow resolution have landed"
status: draft
owner: foundation
size: S
gate: none
depends_on: []
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

`Capability/Filesystem/Errors.ken.md` still says filesystem authority is coarse
and not path-confined. **That is now false**: scoped roots, rights, symlink
policy, and no-follow resolution have landed.

## ⭐ Why this one is special — it is startable NOW

**`ABI-R1` and `ABI-S3` are the ONLY two nodes in this program with no
dependency on `PX8`.** Everything else in §5 descends from it. With the fleet
single-threaded on `RT-NATIVE-FNSPLIT`, these two are the only available parallel
ABI work.

⚠ **Documentation-only and `S`, but it is not busywork:** prose that contradicts
landed behavior is actively misleading, and this is the class of defect the
`DOC-W0` family and the withdrawn `ABI-R2` both came from — *a true statement
standing in for the property that mattered*. The judgment content is deciding
what the capability **now** guarantees, not find-and-replace.
