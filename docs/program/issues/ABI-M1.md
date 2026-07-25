---
id: ABI-M1
title: "manifest v2 — family-scoped, versioned, generated from family schemas"
status: draft
owner: runtime
size: L
gate: none
depends_on: [ABI-R3]
blocks: [ABI-M2, ABI-S4, PX10, PX11]
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

A **family-scoped, versioned manifest generated from family schemas** rather
than one growing handwritten list: target identity (arch, pointer width,
endianness, C scalar widths/alignments), constants and record layouts per
enabled family, facility ABI versions, and canonical hashes per family
projection.

⚠ **Runtime + Foundation collaboration.** Recorded with `owner: runtime` because
a WP is owned by a single team (§2); Foundation participation is required and
must be named in the WP frame.

## Explicitly OUT of scope (§3 deferral)

⛔ Cross-target generation · signed or content-addressed manifests · CI
native-builder matrices. **Native-target only.**
