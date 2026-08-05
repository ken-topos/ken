---
id: RT-CONTSRC-PRODUCER-LOCAL
title: "Producer-local continuation source coordinate — a mid-body value is a third availability class with no ABI seat, so continuation specialization cannot name its environment"
status: active
owner: runtime
size: L
gate: none
depends_on: [RT-DECL-CLOSURE-PORT]
blocks: []
github: null
origin: Steward ruling 2026-08-05 (RT-DECL-CLOSURE-PORT checkpoint 1f) on the D7 1d/1e measurements (evt_5kws532ac99c9, evt_5ngh190h9b1k5) and the Architect representation gate evt_75k8cydbj5127. Steward-filed (agents cannot create tracked work per COORDINATION §2).
---

## Why this is a node and not a fold

Two prior attempts to fold this into [[RT-DECL-CLOSURE-PORT]] were refused by
measurement, and the refusals are the grounding:

- **`1d`** asked whether an existing upstream authority already proves the
  linked edge mandatory and closes its environment. **No** — 1110 candidate
  records, nine authorities, none qualifying (`evt_5kws532ac99c9`).
- **`1e`** authorized the minimal one-slot representation. **Falsified** — the
  effect-result-only population is **zero**, so it closes no edge, and the
  variant has no lawful ABI position (`evt_5ngh190h9b1k5`).
- The **Architect gate** rejected the one-slot design and named the boundary:
  this is a representation and population boundary, not a missing enum arm
  (`evt_75k8cydbj5127`).

⇒ The constraint is a **measured capability gap plus an Architect ruling** —
both grounded sources. This is not a node cut on Steward prose, which is
precisely what the earlier `1d` node claim was and why it was refused.

## The gap, exactly

`ContinuationInputSource` (`planning/static_transition.rs:410`) has three
variants — `Parameter`, `LexicalCapture`, `SeedCapture` — and its enclosing
record requires an **entry-ABI coordinate**. `continuation_owner_entry_sources`
enumerates exactly `parameters + captures`, and every carrier, ownership,
storage-owner, affinity and equality check derives from that exact `AbiSlot`.

A value created **mid-body** — a host-effect result, or a `Match` case binder —
is neither a parameter/capture nor the unit's outgoing `Result` convention slot.
The emission seam confirms the same boundary independently: its exhaustive
two-class resolution locates an entry value in its root owner, or a value
captured by a generated context. **A producer-local value is a third
availability class, and no authority for it exists at `179af863`.**

Measured closure-edge census at that base: **34 case-binder-only, 4
effect-result-plus-case-binder, 1 `Construct`-only.** The four mixed edges span
all six failing `D0` rows in [[RT-DECL-CLOSURE-PORT]].

## What it must deliver

Separate the two coordinate domains, per the Architect's boundary:

1. **Entry ABI source** — the existing owner + parameter/capture position and
   its slot-derived contract, **unchanged**.
2. **Producer-local source** — an exact structural binding identity in the
   producer body, with planner-derived carrier / ownership / storage /
   affinity, and an exact emission-time locator into the environment that
   actually contains it.

The source-position type is a **closed sum**, so an entry position cannot be
mistaken for a local binding. Validation re-derives the same arm and contract;
generated-context capture lookup compares the **full** source coordinate; the
emission resolver handles the local arm **explicitly**, with no default arm and
no exemption.

Producer-local coverage includes **both** the host-effect result and the exact
`Match` case binder. They are distinct structural bindings even if a later
common local-binding representation subsumes them.

## Scope ruling: BROAD admission

Every exact producer-local value is represented, and **all** newly representable
candidates may lawfully intern — not the four `D0` mixed edges alone.

Interning only those four while leaving the 34 same-shaped case-binder
candidates declined would require a **real edge-selection authority**, and
corpus, closure identity, first-`Open` reason and planned-member status are all
forbidden substitutes. Minting one to justify treating identical shapes
differently is a manufactured discriminator. Broad admission dissolves the
route-modality question rather than answering it: **no route-modality authority
is implied or authorized.**

Consequence, stated rather than discovered: roughly 34 additional edges newly
intern, changing emitted code on programs green today. That is the correct
outcome — they were declined only because the representation could not name
their environment — and the per-row baselines are the control.

## The standing methodological requirement

**Validate the full required environment as a vector. First-`Open`
classification is not a population oracle.** Reading a first-failure census as a
requirement census is what produced `1e`'s false minimality ruling: "6 effect
edges equal the 6 failing rows" was a pair count short-circuited at the first
`Open` position, in a different unit from the 161 it was compared against. Any
census this node produces states its unit and answers *what does this edge
require*, never *where did it first stop*.
