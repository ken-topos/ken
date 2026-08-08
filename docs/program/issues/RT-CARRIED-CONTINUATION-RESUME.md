---
id: RT-CARRIED-CONTINUATION-RESUME
title: "A carried scrutinee reaching a continuation frame has no resume path — the carried elimination does not implement the Carried x {PendingLet, Active} arm"
status: merged
owner: runtime
size: M
gate: none
depends_on: []
blocks: [RT-CARRIED-ORDINARY-COMPOSITION, RT-MATCH-RECURSOR-CONSUMERS]
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

## Closed 2026-08-08 — all four deliverables landed, all eight ACs met

`D0`/`D1` census, `D2` route and `D3` control are on `main` at **`06e031de`**,
across PRs **#1623** (`cc736aaf`, the accepted partial) and **#1625**
(`752a7099`, the control). CI green on both.

| AC | discharge |
|---|---|
| AC-1 population closed by measurement | the four-cell census over **486 arrivals** with denominators, not a grep |
| AC-2 `PendingLet`/`Active` partitioned on evidence | discharged in a shape the AC did not anticipate — **one variant fires, the other has no members.** See the note in the frame |
| AC-3 committed discriminating control | `D3`, keyed on the **advance** and proven able to fail |
| AC-4 landed guards intact | `emit_carrier_transfer` byte-unchanged, `carried_join_arm` unchanged, `PendingLet` still fail-closed |
| AC-5 / AC-6 / AC-7 | zero added `#[ignore]`; no retirement or lane deletion; no tracker `status:` in either candidate |
| AC-8 CI green | #1623 and #1625 |

> ### THIS NODE CLOSING DOES NOT CLOSE A ROW, AND THAT IS NOT A SHORTFALL
>
> **`AC-1` here is *this node's* population closure and it is discharged.** The
> undischarged `AC-1` that appears throughout this node's handbacks belongs to
> [[RT-MATCH-RECURSOR-CONSUMERS]] — a different node's AC, now gated on
> [[RT-CARRIED-ORDINARY-COMPOSITION]]. **Do not read the two as one.** Both rows
> still refuse, one authority further out.
>
> The obligation here was *"resume a pending computation over a carrier value,
> or prove that frame/value pairing invalid."* The resume path was built and the
> carrier was **measured** to survive the composition. That obligation is met.

Frame: `docs/program/wp/RT-CARRIED-CONTINUATION-RESUME.md`.
