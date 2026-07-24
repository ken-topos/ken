---
id: RT-ESCAPE
title: escaping a second Resource through a bracket fails native lowering
status: merged
owner: runtime
size: M
gate: none
depends_on: []
blocks: []
github: "PR #911 @ 238a5c5d (origin/main 4ac9141e, CI green)"
origin: steward (filed 2026-07-21 ~03:53Z, STEWARD-DECISION-LOG.md; agents cannot create tracked work per COORDINATION §2)
---

Pre-existing native-lowering defect surfaced by `RT-PARITY`, correctly **not**
fixed there (the implementer filed rather than fixed it, per that WP's "if
native looks wrong, file it, don't fix it here" guardrail). Constructing a
closed-but-still-referenced resource needs it escaped from its bracket;
escaping a **second** `Resource` through a bracket fails native lowering with
`OrientedSubcontinuationPlanV1: checked Runtime frame marker was consumed
more than once`. Escaping a resource plus a plain value lowers fine, so this
is specific to a second *Resource*.

**Architect layer ruling (`thr_65k4`):** this is a **native-lowering
completeness defect, Runtime-owned, sized M** — the interpreter stays out of
the production fix. The `(invocation_id, frame_id)` consumed-set and the
rejection both live in the Cranelift lowerer; `Resource`-ness is the
discriminator that exposes the traversal defect, not authority for a
Resource-specific rule. The layer is settled; the exact mechanism (one
occurrence revisited vs. two lawful activations aliasing) is **not** — the WP
instruments and classifies before repairing. Carries the adversary's untested
finding R2 as a post-repair reaching lane; not currently blocking anything else.

Full shovel-ready brief (ACs, fence, hard-stops):
[`docs/program/wp/RT-ESCAPE-second-resource-native-lowering.md`](../wp/RT-ESCAPE-second-resource-native-lowering.md).
