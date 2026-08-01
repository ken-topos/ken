---
id: RT-CONTSPEC-PLANNER
title: "ContinuationSpecialization slice 1 — land the planner closure DORMANT: exact ordered projection, full-key interning before discovery, exact causal edge tokens, finite recursion"
status: draft
owner: runtime
size: M
gate: none
depends_on: []
blocks: [RT-CONTSPEC-ABI]
github: null
origin: "Architect second WIP audit evt_4t09329vdrf (2026-08-01) returned outcome (c) — RT-RECURSOR-TRANSPORT + RT-DECL-CLOSURE-PORT D7 is mis-sized as one delivery. Steward-authored recut under playbook §5a-iii; the Architect diagnoses sizing, the Steward cuts. No semantic recut: the ruling at evt_7dhwrk26ks9m0 remains binding."
---

> # ⏳ `draft` FOR EXACTLY ONE REASON — one fixed input is still open
>
> ⛔ **`status: draft` here does NOT mean the frame is thin.** The frame is
> written and shovel-ready except for **the base this slice branches from**,
> which is a fork the Steward routed to the Architect at `evt_1bh3p4wx76wtv`:
> `origin/main` re-derives the planner without the unmerged `D7` base beneath
> it; the proved base `93746ada` drags all of `D7`'s unmerged, un-QA'd content
> into a PR whose stated subject is the planner. ⇒ **The moment that lands, this
> flips to `ready` and is released.**
>
> ⛔ **Do not pick this up meanwhile** — a base you choose yourself is the one
> input the frame does not fix.

> # ⭐ SLICE 1 OF 3 — A DELIVERY SHAPE, NOT A NEW DESIGN
>
> The mechanism is already ruled.
>
> ⛔ **Nothing about the semantics is open here.** The causal
> `ContinuationSpecialization` mechanism was ruled at `evt_7dhwrk26ks9m0` and
> that ruling stands. What was wrong was asking **one turn** to close the
> planner, the ABI, the lowering, six ledger families, nested recursion, and the
> real three-way witness together — a cumulative **10 files, +10,193/−2,047**
> with no candidate and no proved checkpoint at any point along the way.
>
> | slice | lands | activates? |
> |---|---|---|
> | **1 — this node** | planner closure | ⛔ **no** — dormant |
> | 2 — [[RT-CONTSPEC-ABI]] | unit/descriptor + ABI/lifetime/affinity gates | ⛔ **no** — still dormant |
> | 3 — [[RT-CONTSPEC-LOWER]] | branch lowering, nested recursion, ledgers, witness, CI | ✅ **yes** |
>
> ⭐ **Each slice must be independently reviewable and must either land or hard
> stop inside the one-hour turn target.** If slice 1 cannot, that is a hard stop
> to route — ⛔ not a long silent run to push through.

## ▶ THE FRAME IS WRITTEN

`docs/program/wp/RT-CONTSPEC-PLANNER.md`. ⭐ Read it before touching code; it
fixes the base, the prototype reference, the deliverables, and the banned scope.

## The two objects this slice sits between

| object | what it is | what it is NOT |
|---|---|---|
| `93746ada…c243` | ⭐ **the proved semantic base** — build on this | — |
| `465fab90…720b` | the frozen prototype, on `origin/preserved/rt-recursor-freeze-465fab90` | ⛔ **not a green checkpoint, not acceptance evidence** |

⭐ **The prototype is the most valuable input to this slice and the most
dangerous.** The Architect's audit confirmed its projection schema, its
full-key `intern_specialization`, its call-token schema and plumbing, and its
explicit unit arm are all **directionally correct** — *"do not throw this design
work away."* ⛔ **But it carries no proof.** Read it, re-derive the parts this
slice needs, and land them with their own controls. ⛔ Do **not** port it
wholesale and treat the Architect's approval of its direction as acceptance of
its content.

## Why this node exists at all

⚠ **It is not a new semantic node, a carrier lane, a disposition, or a new
participant.** It is an implementation slice of the **existing**
[[RT-RECURSOR-TRANSPORT]] + [[RT-DECL-CLOSURE-PORT]] `D7` atomic mechanism, cut
so that a reviewer can see one increment at a time. The constraint that grounds
it is an Architect ruling plus a measured code surface, not a preference for a
tidier graph.
