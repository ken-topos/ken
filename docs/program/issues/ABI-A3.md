---
id: ABI-A3
title: "promote FsReadDirectory, FsCreateDirectory, FsRemoveFile, FsRemoveDirectory to NativeTested"
status: draft
owner: runtime
size: M
gate: none
depends_on: [ABI-REVOKE, ABI-R3]
blocks: [ABI-S2]
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

Promote `FsReadDirectory`, `FsCreateDirectory`, `FsRemoveFile`,
`FsRemoveDirectory` to `NativeTested`.

## Why this is its own slice, and why it also depends on ABI-R3

Split by evidence shape: **directory mutation**, whose distinguishing difficulty
is **ordering and partial-failure semantics**.

★ **It additionally depends on `ABI-R3`** (`10-linux-abi-completion.md:106`) so
the promotions land against a **derived** inventory rather than a hand-edited
one — the promotion is exactly the moment a hand-maintained list would drift.

⚠ **`ABI-S2` supersedes this slices whole-directory read** where streaming is
the honest shape. Do not entrench whole-directory read as the contract.
