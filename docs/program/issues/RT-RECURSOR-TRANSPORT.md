---
id: RT-RECURSOR-TRANSPORT
title: "Active-recursor transport — an active computational recursor's invocation-local scope/return-hole state cannot cross a functionized unit boundary, retaining two residual classes"
status: ready
owner: runtime
size: L
gate: none
depends_on: [RT-PRODUCER-MATCH-PORT]
blocks: [RT-DESCENT-RETIRE]
github: null
origin: Operator directive 2026-07-29 — prioritize replacement of RecursiveDescent, migrate the remaining residual classes, do not linger half-migrated. Campaign docs/program/16-recursive-descent-retirement.md. Steward-filed (agents cannot create tracked work per COORDINATION §2).
---

> # ⭐⭐ THIS NODE RETIRES **TWO** RESIDUAL CLASSES, BECAUSE THEY ARE ONE MECHANISM
>
> `MatchScrutineeRecursor` and `LexicalCallArgumentRecursor` both fire on an
> **active computational recursor** — a `ComputationalMatch` with a case whose
> `recursive_positions` is non-empty — and differ only in the syntactic position
> it occupies. **The code says so itself**, in
> `LexicalCallArgumentRecursor`'s own doc comment (`core.rs:47-52`):
>
> > *"The recursive result still carries invocation-local scope/return-hole
> > state. Passing it through a separately declared lexical unit is not one of
> > the completed functionized ports, so the established recursive descent lane
> > retains the whole call."*
>
> ⇒ ⛔ **Retiring one without the other would build the same transport twice.**
> Folded per `docs/PRINCIPLES.md` *subsume-don't-proliferate*.

## What they are

**`MatchScrutineeRecursor`** (`core.rs:96-105`) — an ordinary `Match` whose
scrutinee is a `ComputationalMatch` carrying recursive positions.

**`LexicalCallArgumentRecursor`** (`core.rs:125-136`) — a `Call` whose callee is
a `LexicalClosure` and whose **argument** is such a recursor.

**The shared gap:** the recursive result carries **invocation-local
scope/return-hole state**. Functionizing means that state must be transported
across a separately owned unit boundary — or be shown not to need to cross.

## ⚠ This is the hard node, and it is sixth

If this proves infeasible as scoped, we learn it after five nodes of investment,
and *half-migrated* is exactly the state the directive rules out.

⭐ **The mitigation is a deliverable, not a hope.** `D1` is a **feasibility
probe that can be pulled forward and run at any time**, independently of this
node's queue position — it needs no code change and no branch of its own. If the
Architect or the ring wants the risk retired early, run it during
[[RT-DECL-CLOSURE-PORT]] and re-cut the schedule on the result. ⛔ Do not reorder
the *build* work to chase it; the transport machinery built earlier in the
campaign is real preparation for this node.

## Sequencing

**Sixth in Runtime's queue**, and the last migration before the capstone
[[RT-DESCENT-RETIRE]]. ⚠ By the time this runs, three classes are retired, so
program shapes that have **never** reached `FunctionizedUnits` will reach it here
for the first time — see the campaign doc's Trap 2. **Expect a hard stop and
route it; it is the fail-closed machinery working, not a defect in this node.**

## ▶ THE FRAME IS WRITTEN

`docs/program/wp/RT-RECURSOR-TRANSPORT.md`. ⭐ Campaign context, the three traps
that bind every node in this arc, and the full schedule:
`docs/program/16-recursive-descent-retirement.md` — **read it before the frame.**
