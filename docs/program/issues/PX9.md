---
id: PX9
title: "cross-domain System.Error — semantic identity, raw errno, operation, resource, safe context, and honest retry classification"
status: draft
owner: foundation
size: L
gate: none
depends_on: [PX8, ABI-REVOKE]
blocks: [ABI-S1, ABI-S5, PX10, PX11]
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

## Objective — **the charter's own undelivered WP**

Cross-domain `System.Error`: semantic identity, raw errno where present,
operation, resource, and safe context; plus retry/interruption/transience
classification.

⛔ **The classification must NOT promise that retry is always safe.** An
error that is *transient* is not thereby *idempotent to retry* — conflating them
is how a retry loop corrupts state.

⛔ **It must reach BEYOND filesystem** — process, socket, and later completion
contexts. A filesystem-only error type is precisely the floor that already
exists and is what makes Track T impossible.

## ★ PX9 gates most of Track T — and that is a sequencing argument, not a preference

Sockets and processes need error context the **filesystem floor cannot express**.
Retrofitting it afterwards means **re-touching every operation added in
between** (`10-linux-abi-completion.md:131-133`, restated §7 as an operator/
Architect sequencing ruling: *"PX9 before PX10/PX11 … error context retrofitted
across two large surfaces"*).

⚠ **`ABI-REVOKE` is sequenced BEFORE PX9 deliberately** so PX9 absorbs the
distinct `revoked` identity rather than having it retrofitted (§7).
