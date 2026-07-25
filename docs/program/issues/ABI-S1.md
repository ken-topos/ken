---
id: ABI-S1
title: "descriptor completion — seek, truncate, sync/data-sync, flags, duplication under explicit inheritance policy"
status: draft
owner: runtime
size: M
gate: none
depends_on: [PX9]
blocks: [ABI-S6]
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

Descriptor completion: seek, truncate, sync/data-sync, flags, duplication
**under explicit inheritance policy**, and descriptor metadata.

⚠ Duplication without an explicit inheritance policy is how a descriptor leaks
across a boundary that was supposed to confine it — the policy is the deliverable,
not a parameter.
