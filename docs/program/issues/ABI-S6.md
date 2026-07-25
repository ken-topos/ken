---
id: ABI-S6
title: "ordinary anonymous and file-backed mappings as opaque runtime-owned regions and bounded byte views"
status: draft
owner: runtime
size: L
gate: none
depends_on: [ABI-S1]
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

Ordinary anonymous and file-backed mappings as **opaque runtime-owned regions
and bounded byte views** — ⛔ **never Ken pointers.**

★ **This supplies the mapping/lifetime/bounded-access substrate that L2-8 MMIO
later builds on.** Getting the ownership and bounds shape right here is what
keeps MMIO from needing raw pointers in application Ken — which is a stated exit
condition of the whole program (§6).

⚠ Runtime + Foundation collaboration; recorded `owner: runtime` because a WP is
owned by a single team (§2).
