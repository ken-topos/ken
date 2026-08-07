---
id: RT-CONTSRC-PRODUCER-LOCAL
title: "Producer-local continuation source coordinate — a mid-body value is a third availability class with no ABI seat, so continuation specialization cannot name its environment"
status: merged
owner: runtime
size: L
gate: none
depends_on: [RT-DECL-CLOSURE-PORT]
blocks: []
github: null
origin: Steward ruling 2026-08-05 (RT-DECL-CLOSURE-PORT checkpoint 1f) on the D7 1d/1e measurements (evt_5kws532ac99c9, evt_5ngh190h9b1k5) and the Architect representation gate evt_75k8cydbj5127. Steward-filed (agents cannot create tracked work per COORDINATION §2).
---

> ## AC-1 SHIPS DORMANT, WITH AN UNMEASURED RUNTIME RESIDUAL AND A GATE ATTACHED
>
> **Architect `evt_7qfayjcebxv5y`, ratified Steward `evt_4nabbpm2crz82`,
> 2026-08-06.** Read this before concluding anything about `AC-1`'s controls.
>
> `lower_source_carried_match` lands as a **reviewed DORMANT partial
> mechanism**: structurally complete class/tag/arity selectors, valid IR,
> planner authority consumed exhaustively, mismatch separated from semantic
> default. **NOT tested functionality and NOT a completed `AC-1`.**
>
> **Why that was allowed, and it is one fact:** a census over 779 `ken-runtime`
> lib tests found the route **entered exactly once** — refusing at join
> acquisition before emitting any selector — and the whole-process `rt_parity`
> route stops at the independently-named byte-span seat before producing an
> executable result. **No completing rig executes this mechanism at all.** That
> bounds the present risk and the present claim simultaneously.
>
> **NARROWED by the Architect, `evt_m36y2zegby7m`, 2026-08-06.** Say **"no
> ENUMERATED completing rig executes `lower_source_carried_match`"**. The census
> is explicitly the `ken-runtime` crate plus `rt_parity`;
> `px4b_native_production` is a **third rig that completes lowering** and was in
> neither. It was checked and ruled a **negative by path** — a functionized
> declaration-unit call whose `Carried` scrutinee dispatches to the *generic*
> `lower_carried_match`, never constructing or resuming
> `SourceContinuation::MatchScrutinee`, which is `lower_source_carried_match`'s
> only caller on that path. **An enumerated negative, not a universal proof
> about every possible rig.** The activation gate has NOT fired.
>
> ⇒ **THE OWED CONTROLS DO NOT DISAPPEAR. THEY BECOME AN ACTIVATION GATE**,
> recorded on **`RT-CARRIER-BYTESPAN-OBSERVE`** and binding on **any** route
> that makes a carried source-Match path successfully executable — the gate is
> on *activation*, not on a node id.
>
> **`AC-1` clause 1 was already recut out** to that successor
> (`evt_3pr04vk7zrd7c`); clause 2 is discharged. **This node is NOT blocked
> on the controls** and must not be held for them.
>
> **Admissibility, for anything later claimed as a control here:** the entire
> claimed property is decided **before** any independent abort, **and** a
> mutation of it has been observed to make the exact assertion **RED** before
> the run reaches the same abort. Route entry, an emitted counter, or
> plausible IR is **not** evidence. Measured dead ends, so nobody re-spends
> them: the **capacity family reaches this route zero times** (a
> `Construct`-bound closure parameter stays `Specialized` and is selected at
> compile time — the doc comment claiming otherwise is true for an *effect
> argument*, false for a *match scrutinee*); the **mismatch block is reached
> zero times**, so the wrong-class control cannot be written today; and the
> `control.rs` inventory list is **`#[cfg(any())]` dead code** whose "repair"
> would resurrect a prohibited repository-text oracle — do not resurrect it.

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
