---
id: RT-AGG-COMPOSE
title: "escaping two Resources into one aggregate (Prod (Resource _) (Resource _)) fails at erasure — checked endpoints do not compose"
status: draft
owner: runtime
size: TBD
gate: none
depends_on: []
blocks: []
github: null
origin: runtime-leader evt_6wq2tp6txd5v8 (surfaced incidentally during RT-ESCAPE; agents cannot create tracked work per COORDINATION §2 — Steward-filed)
---

Pre-existing producer-layer defect surfaced during **RT-ESCAPE** and correctly
**filed rather than worked around** — it is upstream of RT-ESCAPE's native-
lowering fence (`lowering/core.rs`) and does not touch RT-ESCAPE's scope or
require reopening that fence.

## The defect

Escaping **two resources into one aggregate** — `Prod (Resource _) (Resource _)`
— fails at **erasure** (not native lowering) with:

```
oriented subcontinuation checked endpoints do not compose
```

Runtime-implementer's characterization: *"a separate pre-existing producer-layer
limitation, hard-stop-adjacent, unrelated to this native-lowering fix."*

## Why it is distinct — do not fold into RT-ESCAPE

- **Different layer.** RT-ESCAPE is a **native-lowering** completeness defect
  (the Cranelift `(invocation_id, frame_id)` consumed-set); this fails at
  **erasure**, upstream of lowering. RT-ESCAPE's Architect ruling explicitly
  fenced the erasure/producer layer *out* of that WP.
- **Different shape.** RT-ESCAPE = escaping a *second* Resource through a
  *bracket*. This = composing *two* Resources into one *aggregate* (`Prod`).
- **Distinct from the adversary's R2** (cross-buffer `freeze(buffer_b,
  span_from_a)` BufferFreeze reaching lane, carried in RT-ESCAPE AC-6). This is
  a checked-endpoint *composition* failure, not a bounds/freeze question.

## Before this can be sized

Like RT-ESCAPE, this needs an **Architect layer-ownership call** before it can
be sized and framed: which layer owns "checked endpoints do not compose" for an
aggregate of oriented-subcontinuation-carrying resources — the erasure pass, the
producer, or the checked-endpoint discipline shared across them. **Not routed to
the Architect yet** (low urgency; keep the Architect single-threaded through the
in-flight SEAL-2 / RT-ESCAPE reviews). Route the layer question when it rises in
priority and the Architect is free.

## Priority / relation to PX8

**Low urgency — pre-existing, blocks nothing today.** Relation to the PX8 closure
property (positioned/partial IO path reifies every value absolute-correct + co-
indexed on both backends) is **TBD**: it concerns Resource *aggregate*
composition, not obviously the positioned/partial IO reification the PX8 closure
set enumerates. Do **not** assume it is a PX8-closure gate without confirming
against `docs/program/issues/PX8.md`'s property. Verbatim detail:
`thr_1b4k2ba2d5j2v`.
