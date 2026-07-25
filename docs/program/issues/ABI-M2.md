---
id: ABI-M2
title: "runtime facility/operation probes, distinct from build-time facts"
status: draft
owner: runtime
size: M
gate: none
depends_on: [ABI-M1]
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

Runtime facility/operation **probes**, kept distinct from build-time facts.

## ★ The point being made

**A minimum kernel version is release metadata, not an availability contract.**
Backports and configuration mean support is **per-operation**, so availability
must be probed at runtime rather than inferred from a version number.

⛔ **Unavailability is a stable NAMED result, never a silent fallback.** A silent
fallback converts an absent facility into wrong behavior.
