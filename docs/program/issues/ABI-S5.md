---
id: ABI-S5
title: "terminal basics and process signal disposition at the executable edge"
status: draft
owner: runtime
size: M
gate: none
depends_on: [PX9]
blocks: [PX10]
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

Terminal basics and **process signal disposition** needed at the executable edge.

⚠ **It gates `PX10`.** Signal disposition is part of what a spawned child
inherits, so processes cannot be modeled honestly until disposition is
expressible.
