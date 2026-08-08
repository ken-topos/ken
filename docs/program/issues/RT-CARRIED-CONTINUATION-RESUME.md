---
id: RT-CARRIED-CONTINUATION-RESUME
title: "A carried scrutinee reaching a continuation frame has no resume path — the carried elimination does not implement the Carried x {PendingLet, Active} arm"
status: ready
owner: runtime
size: M
gate: none
depends_on: []
blocks: [RT-MATCH-RECURSOR-CONSUMERS]
github: null
origin: Architect sibling-authority ruling evt_2pt95vbja6447 (2026-08-08) on the RT-MATCH-RECURSOR-CONSUMERS D2 hard stop evt_397gfxdg45ncs, measured at checkpoint 50808c11. Campaign docs/program/16-recursive-descent-retirement.md node #6e. Steward-filed (agents cannot create tracked work per COORDINATION §2).
---

> # THIS IS A SIBLING AUTHORITY, NOT A DEFECT IN EITHER LANDED PORT.

Two repairs have already landed on this chain and **neither is reopened by this
node**:

- [[RT-RECURSOR-TRANSPORT]]'s `D2` at `resume_active_continuation` — sound,
  merged at `89aa1550`, **not** a completeness defect (Architect,
  `evt_2pt95vbja6447`).
- [[RT-MATCH-RECURSOR-CONSUMERS]]'s `D2` at `carried_join_arm` — built, correct,
  retained-green at `50808c11`, landing separately as an accepted partial.

**This node exists because the second repair worked.** The `RecursiveBackedge`
refusal is gone from both A rows, which is itself the proof the repaired arm was
reached — and the rows now fail further in, at a different owner.

## What it is

Under **A**-only exclusion at `50808c11`, both A rows fail **identically**:

```
Unsupported(BoundaryCarrier, "a carried scrutinee reached a continuation frame
that resumes a compile-time value rather than eliminating one")
```

The owner is **`lower_computational_match_value_composed`**, at its
`Active`/`PendingLet` arm. *(At `50808c11` those were `core.rs:3667` and
`core.rs:3793`. **The function name is the handle** — this node moves that file,
so the line numbers rot against its own deliverable. Re-derive by name.)*

## Why it is a sibling and not the same authority

The Architect's distinction is **semantic**, not a difference of function name or
refusal string:

| | `resume_active_continuation` (landed `D2`) | this arm |
|---|---|---|
| operand | `Specialized(RecursiveBackedge)` | `LoweringOperand::Carried(word)` |
| what it holds | a protocol marker; **the CFG edge has already left** | a **live dynamic value** |
| first eliminator | n/a | `PendingLet` or `Active` |
| the obligation | propagate the marker before cursor minting, successor-frame construction or eliminator work | resume a pending computation over a carrier value, **or prove that frame/value pairing invalid** |

⇒ **A pending suffix is shared *context*, not shared *authority*.** The owned
fact, the operand phase, the required action and the fail-closed boundary all
differ. Returning a backedge marker here is not available and **would answer a
different question**.

## Population

**The production arm `Carried(word)` x first eliminator `{PendingLet, Active}`
in `lower_computational_match_value_composed`.** The owner is the **carried
continuation-frame consumer** — not either discovering fixture.

**The two exposed A rows are the floor, not the perimeter.** This campaign has
now twice read a one-or-two-witness result as a class-wide property. Enumerate
from the production arm, per fixture, by measurement.

## The partition that must not be assumed

**`PendingLet` and `Active` are two frame variants sharing one refusal arm.** A
shared arm is **not** evidence they require one mechanism (Architect, explicit).
`D0`/`D1` census **both** variants and partition them **before** any repair; if
they prove materially distinct authorities, that is a hard stop, not a two-case
`match`.

## Scope

Gates completion of [[RT-MATCH-RECURSOR-CONSUMERS]] and its `AC-1`. Does **not**
reopen or reassign [[RT-RECURSOR-TRANSPORT]] `D2`. Does not touch rows 1-5 or the
`LexicalCallArgumentRecursor` population ([[RT-LEXICAL-RECURSOR-CONSUMERS]]).

Frame: `docs/program/wp/RT-CARRIED-CONTINUATION-RESUME.md`.
