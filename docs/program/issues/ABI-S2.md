---
id: ABI-S2
title: "directory streaming — supersedes whole-directory read where streaming is the honest shape"
status: draft
owner: runtime
size: M
gate: none
depends_on: [ABI-A3]
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

Directory streaming. **Supersedes `ABI-A3`'s whole-directory read** where
streaming is the honest shape.

★ **This node exists because whole-directory read is a convenient lie for large
or concurrently-mutating directories.** `ABI-A3` promotes what exists; this
replaces the shape. Sequenced after `ABI-A3` so the replacement is measured
against a promoted, differentially-tested baseline.
