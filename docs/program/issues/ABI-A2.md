---
id: ABI-A2
title: "promote FsAppendFile, FsMetadata, FsRename to NativeTested"
status: draft
owner: runtime
size: M
gate: none
depends_on: [ABI-REVOKE]
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

Promote `FsAppendFile`, `FsMetadata`, `FsRename` to `NativeTested`.

## Why this is its own slice

Split by evidence shape: this is **metadata/rename**, whose distinguishing
difficulty is **path-policy interaction** (scoped roots, rights, symlink policy,
no-follow resolution) rather than nondeterminism or partial failure.
