---
id: RT-CARRIED-ORDINARY-COMPOSITION
title: "Carried ordinary elimination consumes exactly one frame — a composed suffix behind an ordinary carried eliminator is refused rather than continued"
status: active
owner: runtime
size: M
gate: none
depends_on: [RT-CARRIED-CONTINUATION-RESUME]
blocks: [RT-SPECIALIZED-ACTIVE-RESUME, RT-MATCH-RECURSOR-CONSUMERS]
github: null
origin: Architect fourth-wall ruling evt_63ae56tttz9pq (2026-08-08) on the RT-CARRIED-CONTINUATION-RESUME D2 armed stop evt_7qcgfbwgxh0qf, measured at checkpoint cc736aaf. Handle assigned by the Architect in that ruling. Campaign docs/program/16-recursive-descent-retirement.md node #6f. Steward-filed (agents cannot create tracked work per COORDINATION §2).
---

> # THIS IS A COMPLETENESS SUCCESSOR TO A MERGED NODE, NOT A DEFECT IN IT.

The guard this node repairs was landed **deliberately** by
[[RT-PRODUCER-MATCH-PORT]] `D2`, which documented it in the code as a shape it
had not ported and predicted it would go live at retirement **with no
shape-reaching control**. A shape now reaches it. **That node is merged and is
not reopened.**

Three repairs already landed on this chain and **none is reopened here**:

- [[RT-RECURSOR-TRANSPORT]]'s `D2` at `resume_active_continuation`.
- [[RT-MATCH-RECURSOR-CONSUMERS]]'s `D2` at `carried_join_arm`, merged in
  `3061a645`.
- [[RT-CARRIED-CONTINUATION-RESUME]]'s `D2` routing `Carried x Active` into
  `resume_active_continuation`, measured correct at `cc736aaf`.

**This node exists because the third repair worked.** The continuation-frame
refusal is gone from both A rows, and they now fail further in at a fourth
owner.

## What it is

At `cc736aaf`, under **A**-only exclusion, both A rows fail identically:

```
Unsupported(BoundaryCarrier, "a carried producer-call scrutinee reached an
ordinary eliminator with further composed eliminators behind it; the carried
elimination consumes exactly one frame, so the remainder would be silently
dropped")
```

The owner is the **`Carried x Ordinary` pre-delegation guard family**, the three
guards standing in front of the delegation to `lower_carried_match`. *(At
`cc736aaf` the trailing-suffix guard sat near `core.rs:3761`. **The function and
the guard's own text are the handles** — this node moves that file, so any line
number written here rots against its own deliverable.)*

## Why it is a sibling and not the same authority

The Architect's separation is by **owned fact and required action**, not by
message or file:

| | [[RT-CARRIED-CONTINUATION-RESUME]] | this node |
|---|---|---|
| cell | `Carried x Active` | `Carried x Ordinary x nonempty remaining eliminators` |
| owned fact | the live carrier and the active pending head | the composed eliminator **suffix** |
| required action | route the carrier into the active continuation and **preserve** its pending suffix | eliminate one ordinary carried frame **and then continue the composed suffix** |
| status at `cc736aaf` | **done, measured** | **missing** |

⇒ `resume_active_continuation` **has already done its job** at the point of this
refusal. It takes the active pending head and preserves the tail as a successor
`Active` frame. The ordinary arm then sees that successor in `eliminators[1..]`
and refuses, because its delegation to `lower_carried_match` can express only
cases / default / origin / env and **consumes exactly one frame**.

**The missing capability is suffix-preserving carried ordinary elimination, not
carrier resumption.**

## The two suffix sources must not be conflated

**Architect, explicit, and this is the sharpest input on the node.** Two
different tails reach one guard with one message:

1. The **explicit outer eliminator tail** guarded by the new `Active` arm that
   `cc736aaf` added. **It did not fire.**
2. The **successor frame rebuilt from `active.pending`** by
   `resume_active_continuation`. **This is the one that fired.**

They arrive at the same line and render the same string. **Conflating them
misattributes the wall**, and the evidence must keep them distinct.

## Population

**The entire production `Carried x Ordinary` pre-delegation guard family**, not
only the cell that fired and not only the string that rendered:

1. `retained_scrutinee_index = Some(_)`;
2. `deferred_constructor_case = Some(_)`;
3. nonempty `eliminators[1..]`, **including exact suffix length, kinds and
   provenance**.

Census with **denominators and intersections**. **The two A rows with one
trailing `Active` frame are the floor, not the perimeter** — this campaign has
now three times read a small-witness result as a class-wide property, and every
correction cost more than the census would have.

**If only the trailing-suffix cell has members, repair that cell and leave the
other two as measured-at-base zeros** — the same disposition `PendingLet`
received at [[RT-CARRIED-CONTINUATION-RESUME]], and for the same reason: a
mechanism over an empty population is a vacuous proof (Campaign Trap 3).

## The partition that must not be assumed

The three guards share an arm and share a historical port. **Neither proves one
mechanism.** `D1` partitions their owned facts before any coding.

## Scope

Gates completion of [[RT-MATCH-RECURSOR-CONSUMERS]] and its `AC-1`. Does **not**
reopen [[RT-PRODUCER-MATCH-PORT]], [[RT-RECURSOR-TRANSPORT]] `D2`, or either
landed carried repair. Does not touch rows 1-5 or the
`LexicalCallArgumentRecursor` population ([[RT-LEXICAL-RECURSOR-CONSUMERS]]).

## `D0`/`D1` MEASURED 2026-08-08 — re-sized `M` to `S`, `D2` authorized

Checkpoint `147b239c` over `06e031de`, record-only. Steward ruling
`evt_ds4hwahvc5se`, thread `thr_pvxda1tcg20d`.

**14 arrivals, instrument above every guard, all three predicates per arrival.**
`retained_scrutinee_index` and `deferred_constructor_case` were false at every
arrival, and no arrival satisfied more than one guard — so the two zeros are
**no members**, not *never reached*. Only the trailing-suffix cell has members:
**3 under A-only exclusion, each `suffix_len=1`, `suffix_kinds=Active`.**

`AC-3` is discharged by measurement: **all three firing suffixes come from the
`active.pending`-rebuilt successor; zero from the explicit outer tail.**
Attribution is a take-and-clear flag set immediately before the composing call,
not read off the guard message.

**Re-sized to `S`** — one cell, one mechanism, two independent members (the
third firing member in the retained run is this chain's own `D3` control arming
the committed hook). `D2` repairs the trailing-suffix cell only; the other two
stay fail-closed as measured-at-base zeros.

**Hard stop 3 did not fire and is NOT discharged.** `lower_carried_match`
already returns a `LoweringOperand`, so the suffix is continuable without
widening the interface — **expressible, not proven**. It fires if `D2` finds the
re-entry needs more than cases / default / origin / env.

> ### `D2` OWES A DECREASING MEASURE, AND THE CENSUS CANNOT SUPPLY IT
>
> Every firing suffix comes from the `active.pending`-rebuilt successor — **the
> one source that can regenerate a suffix.** Composing the returned operand
> against `eliminators[1..]` and re-entering the composed consumer terminates
> only if each re-entry consumes from a strictly shorter list, and a rebuilt
> successor is not obviously drawn from that list.
>
> **Every measured member is `suffix_len=1`, so nothing exercises depth two.**
> State the measure and what guarantees it, or bound the re-entry depth and fail
> closed past the bound.

**Two notes on counting.** A census re-run after `D3` lands reads inflated —
state the denominator as excluding committed controls. And this node's `D3` must
not count itself as evidence its population exists.

**The predecessor's outer-tail guard has never fired and still has no witness.**
It stays, because fail-closed on an unported shape is correct, but its presence
is not evidence the shape exists.

Frame: `docs/program/wp/RT-CARRIED-ORDINARY-COMPOSITION.md`.
