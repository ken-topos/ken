---
id: RT-ESCAPE
title: escaping a second Resource through a bracket fails native lowering
status: draft
owner: runtime
size: TBD
gate: none
depends_on: []
blocks: []
github: null
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

Filed, unsized. Also carries the adversary's untested finding R2. Needs
Architect input on which layer owns the defect (native lowering vs. a shared
runtime frame-marker discipline) before it can be sized into the queue; not
currently blocking anything else.
