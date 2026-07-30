---
id: RT-RECURSOR-TRANSPORT
title: "Active-recursor transport — an active computational recursor's invocation-local scope/return-hole state cannot cross a functionized unit boundary, retaining two residual classes"
status: active
owner: runtime
size: L
gate: none
depends_on: []
blocks: [RT-DESCENT-RETIRE]
github: null
origin: Operator directive 2026-07-29 — prioritize replacement of RecursiveDescent, migrate the remaining residual classes, do not linger half-migrated. Campaign docs/program/16-recursive-descent-retirement.md. Steward-filed (agents cannot create tracked work per COORDINATION §2).
---

> # ⛔⛔ THIS NODE IS IN FLIGHT **NOW**, ATOMICALLY WITH [[RT-DECL-CLOSURE-PORT]]
>
> ⚠ **It is not queued, and it is no longer "sixth".** Since 2026-07-29 it is
> being built as **one candidate** with [[RT-DECL-CLOSURE-PORT]]'s `D7` — the
> `D7` boundary-use contract and this node's capture transport turned out to be
> the same mechanism, so they cannot land apart.
>
> | fact | value |
> |---|---|
> | branch | `wp/RT-DECL-CLOSURE-PORT` — ⚠ **one branch for two nodes** |
> | PR | **one** PR carries both |
> | tracker flip | ⛔ both nodes flip `merged` in **one** commit |
> | CI | `rt_parity_native` is this node's **own** job |
>
> ⛔ **`depends_on: []` is deliberate — do NOT "fix" it by adding
> [[RT-DECL-CLOSURE-PORT]].** A dependency edge encodes *after*; these are
> **siblings in one atomic set**, not a sequence. The edge that keeps this node
> off the releasable frontier is its `active` status, nothing else.
>
> ⚠ **For whoever publishes:** the branch name names only one of the two nodes.
> ⛔ Do not describe this node's recursor code as a `D7` deliverable in a PR body
> or a merge post.

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

## ⚠ This is the hard node, and the feasibility risk was retired EARLY

⚠ **This section used to read "and it is sixth", and to offer `D1` as a probe
that *could* be pulled forward.** Both are now spent: the node was pulled to
**first** and is in flight atomically with [[RT-DECL-CLOSURE-PORT]] (see the
banner above), so the risk this section hedged against — learning of
infeasibility after five nodes of investment — is no longer the risk being run.

⭐ **What actually happened is better than the mitigation.** The transport is
being built against real refusals on a live branch rather than probed in the
abstract, and the campaign's remaining nodes now depend on a mechanism that is
being *proven* rather than *assumed*. ⇒ The residual risk moved from *"can this
be done at all?"* to *"does this exact candidate go green?"* — which CI answers.

⛔ **What this does NOT license:** treating the later nodes as de-risked. `D1`'s
feasibility question is answered for **these two** residual classes only;
[[RT-SEED-CALL-PORT]] and [[RT-PRODUCER-MATCH-PORT]] still owe their own
measurements, and [[RT-SEED-CALL-PORT]] may yet close for free (its own node
says so). ⭐ A mechanism proven here is *preparation* for them, not a verdict.

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
