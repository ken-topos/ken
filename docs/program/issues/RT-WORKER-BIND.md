---
id: RT-WORKER-BIND
title: "compiler-only static-worker binding and transport substrate — lowering cannot bind a worker's carried capture operands into a selected semantic body, and continuation specialization cannot emit a target without it"
status: draft
owner: runtime
size: L
gate: none
depends_on: [RT-CONTSPEC-ASSEMBLY]
blocks: [RT-CONTSPEC-ACTIVATE]
github: null
origin: "Architect outcome (c) at evt_2anwskscqz5fg (2026-08-02), correcting evt_2a6dgqcn0yss4. RT-CONTSPEC-ACTIVATE is mis-sized at the target-binding boundary: the remaining target cannot be bound through any existing route. Steward-filed (agents cannot create tracked work per COORDINATION section 2)."
---

# The substrate that continuation specialization turned out to need

> ## HELD AT `draft` ON PURPOSE — the frame is incomplete by design
>
> **Steward, 2026-08-02.** Two inputs are owed by the Architect before this can
> be released, requested in one pass at `evt_3ka1whhzj9z8x`:
>
> 1. **the substrate's own representation** — the ruling names precisely what
>    blocks, and deliberately does not name what should exist instead;
> 2. **the independent witness** — what proves this substrate *without*
>    continuation specialization as its consumer.
>
> The node is `draft` so it cannot enter the frontier with those open. This is
> the corrective for the pattern that produced three hard stops on
> `RT-CONTSPEC-ACTIVATE`: a frame that specifies one interface hop at a time
> costs one hard stop per hop. **Do not release this node by flipping `status`
> until both are ruled.**

## How this node came to exist

`RT-CONTSPEC-ACTIVATE` reached D3 and stopped four times. The first three stops
traced to the Steward's frame. **The fourth did not.** The implementer refused
to invent a binding API that the Architect's complete-interface ruling had
assumed existed (`evt_2y2586nt6xrtz`), the leader carried the refusal
(`evt_24m7c923c57bf`), and the Architect re-grounded on exact `dd0ca60e` and
corrected itself: there is no lawful producer binding under another name, and
the missing piece is not a lookup anyone failed to find.

⇒ **The refusal produced the finding.** A seat that had guessed an API here
would have buried a representation defect under a plausible implementation.

## What exists, exactly

The source-occurrence authority is present in `lowering/core.rs`:

- `retained_body_occurrence(StaticOriginId) -> SourceOccurrence` — the sole
  retained origin to expression route;
- `child_occurrence(parent, position, child)` — the exact planner-owned
  positional child origin;
- `case_body_occurrence(match_origin, case_index, body)`, which is exactly
  `child_occurrence(match_origin, 1 + case_index, body)`.

At the real `RuntimeExpr::Construct` arm of `lower_computational_producer_expr`
the lawful fields are already in hand: the producer Construct origin
(`static_origin`), the active continuation origin
(`eliminator.static_origin` for `EliminatorFrame::Computational`), the selected
alternative (the enumerated `case_index`), the ruled recursive positions
(`case.recursive_positions`), the selected return hole, the worker occurrence
and body, the lexical captures, and the caller-continuation operands
(`eliminator.env`).

**Those observations validate a planner decision. None may select a token by
worker shape or runtime value.**

## What does not exist

Two distinct absences, and only the second is hard.

**The projection.** There is no lawful producer binding API. The planner
projection must gain one read-only operation keyed

```
(producer_construct_origin, continuation_origin,
 producer_alternative, recursive_position) -> Option<ContinuationCallIdentity>
```

**Four fields, not three.** The planner mints one call token per ruled
recursive position, so a three-field key is not unique for a case with multiple
recursive positions — this corrects both the Architect's prior ruling and the
Steward's frame. `call_site_sequence` stays opaque inside the returned
identity; lowering neither supplies nor derives it.

**The target's semantic environment — the actual stop.**

- continuation ABI Parameter/Capture slots load as `LoweringOperand::Carried`;
- `Lowered::Closure` requires `captures: Vec<Lowered>`, so it cannot hold those
  operands without the forbidden carried to specialized inversion;
- `Lowered::ComputationalRecursorClosure` owns the sole licensed
  `LoweringOperand` residual, but current source gives it neither ordered
  carried worker captures nor a planned worker-environment aggregate;
- `recursive_position_unit_body` returns `None` for a lexical closure with
  captures, because the carried value exposes no capture operands;
- `call_declared_recursive_position_unit` can call a known body with operands,
  but no environment or binding API lets the exact selected case body denote
  and invoke that static worker.

⇒ `retained_body_occurrence` recovers the return-hole and worker **terms**, and
a completed planner view can project their **contract**, but **lowering cannot
bind the worker's carried capture operands into the selected semantic body.**

## Already-landed law this node may not violate

A raw frame pointer, a fabricated `Lowered::Closure`, an inverse carrier
conversion, source substitution, a new runtime callable word, or a second
affine ledger. Each would discharge the stop by breaking something already
proved.

## Scope the Architect named

The substrate must cover **multiple recursive positions** and **nested-worker
closure**, and it must be **independently provable**. That last word is
load-bearing: if the only demonstration runs through `RT-CONTSPEC-ACTIVATE`,
this is not a substrate but a cumulative branch, which is the exact shape that
killed `RT-CONTSPEC-LOWER`.

## The fork this node does not foreclose

The Architect offered two dispositions: this substrate, **or** a deliberate
reopening of the preallocated worker-environment representation. This node
frames the first. **If the substrate proves unbuildable against the landed
representation, that is a hard stop back to the Architect** and the reopening
becomes an explicit fork with operator visibility — not something the Steward's
sequencing quietly decided.

Frame: `docs/program/wp/RT-WORKER-BIND.md`, written when the two owed inputs
land.
