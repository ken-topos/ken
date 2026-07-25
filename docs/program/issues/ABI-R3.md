---
id: ABI-R3
title: "generated operation inventory derived from catalog structure — a new operation must be a build break"
status: draft
owner: runtime
size: M
gate: none
depends_on: [PX8]
blocks: [ABI-REVOKE, ABI-M1]
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

Generated inventory of **operation identity, availability, rights,
request/reply schema, and differential fixture per operation**, derived from the
catalogs own structure so that **adding an operation is a build break**.

## ★ This is the load-bearing node of Track R

It is the **same mechanism `SEAL-2` built for carrier producers**, applied to the
operation catalog: an enumeration **derived from structure** rather than restated
by hand. Every later track adds operations, and each one is a chance for a
hand-maintained list to drift.

⛔ **Tests assert NAMED memberships and properties, never total counts.** A count
is only as good as its window; a named membership survives the window changing.
