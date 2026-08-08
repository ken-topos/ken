---
id: RT-RECURSOR-TRANSPORT
title: "Active-recursor transport — an active computational recursor's invocation-local scope/return-hole state cannot cross a functionized unit boundary, retaining two residual classes"
status: ready
owner: runtime
size: L
gate: none
depends_on: [RT-CONTSPEC-WITNESS]
blocks: [RT-DESCENT-RETIRE]
github: null
origin: Operator directive 2026-07-29 — prioritize replacement of RecursiveDescent, migrate the remaining residual classes, do not linger half-migrated. Campaign docs/program/16-recursive-descent-retirement.md. Steward-filed (agents cannot create tracked work per COORDINATION §2).
---

> # SUPERSEDED EDGE 2026-08-02 — READ THIS BEFORE ANY REFERENCE BELOW
>
> [[RT-CONTSPEC-LOWER]] is **`closed`**, superseded by a four-seam recut
> (Architect `evt_1yymw1gdszpbs`, outcome (c)). **Every reference to it below
> now means [[RT-CONTSPEC-WITNESS]]**, the terminal seam that carries the
> three-node closure. The seams are [[RT-CONTSPEC-ASSEMBLY]] ->
> [[RT-CONTSPEC-ACTIVATE]] -> [[RT-CONTSPEC-LEDGER]] -> [[RT-CONTSPEC-WITNESS]],
> each branching from `main` after its predecessor lands.
>
> The closure timing below is unchanged in kind: this node still closes when the
> terminal seam merges, in one tracker commit. Only the name of that seam moved.


> # ⛔⛔ RECUT 2026-08-01 — THIS NODE NO LONGER DELIVERS DIRECTLY
>
> It closes when [[RT-CONTSPEC-LOWER]] merges.
>
> **The Architect's second WIP audit (`evt_4t09329vdrf`) returned outcome (c):
> this contract is mis-sized as one implementer bite.** Two runs against it —
> the 30-hour first run and the corrected re-kickoff — produced a cumulative
> **10 files, +10,193/−2,047** with **no candidate, no QA route, and no proved
> checkpoint** at any point.
>
> ⛔ **The semantics are NOT recut.** The ruling at `evt_7dhwrk26ks9m0` stands
> unchanged, and the mechanism direction was confirmed correct in the same
> audit. **Only the delivery shape changed.** Work now runs through four
> staged slices, in order:
>
> | slice | node | activates? |
> |---|---|---|
> | 0 | [[RT-CONTSPEC-SUBSTRATE]] — dormant `D7` substrate | ⛔ no, dormant |
> | 1 | [[RT-CONTSPEC-PLANNER]] — planner closure | ⛔ no, dormant |
> | 2 | [[RT-CONTSPEC-ABI]] — unit/descriptor + ABI/lifetime/affinity gates | ⛔ no, dormant |
> | 3 | [[RT-CONTSPEC-LOWER]] — branch lowering, ledgers, witness, CI | ✅ **yes** |
>
> ⭐ **The frozen state is durable:** `465fab90767a808edac79e665a1055b81206720b`
> on `origin/preserved/rt-recursor-freeze-465fab90` (tree `aa7571a0`, parent
> `fbfa2403`, 173 files, +4267/−7885). It is a **prototype and reference**,
> ⛔ **not a green checkpoint and not acceptance evidence.**
>
> ⚠ **`depends_on` names [[RT-CONTSPEC-WITNESS]]** (it named
> [[RT-CONTSPEC-LOWER]] until the 2026-08-02 recut). That edge encodes *this
> node closes after the terminal seam*, which is exactly right. ⛔ It does **not** reopen
> the atomicity note below — see it for why [[RT-DECL-CLOSURE-PORT]] is still
> not a dependency.

> # ⛔ STILL ATOMIC WITH [[RT-DECL-CLOSURE-PORT]]
>
> ⛔ But **no longer one candidate.**
>
> ⚠ **It is not queued, and it is not "sixth".** Since 2026-07-29 it has been
> built together with [[RT-DECL-CLOSURE-PORT]]'s `D7` — the `D7` boundary-use
> contract and this node's capture transport are the **same mechanism**, so they
> cannot *land* apart. ⭐ **That is still true.** What changed on 2026-08-01 is
> that they are no longer *built* in one bite.
>
> | fact | value **(corrected 2026-08-01 — the recut changed every row but the last)** |
> |---|---|
> | branch | ⛔ **one branch per slice**, not `wp/RT-DECL-CLOSURE-PORT` |
> | PR | ⛔ **four** PRs — slices 0, 1 and 2 land dormant, slice 3 activates |
> | tracker flip | ⛔ **three** nodes flip `merged` in **one** commit when [[RT-CONTSPEC-LOWER]] merges: this one, [[RT-DECL-CLOSURE-PORT]], and the slice |
> | CI | `rt_parity_native` is this node's **own** job — ⚠ meaningful only at slice 3 |
>
> ⛔ **`depends_on` does NOT name [[RT-DECL-CLOSURE-PORT]], and that is
> deliberate — do not "fix" it.** A dependency edge encodes *after*; these two
> are **siblings in one atomic set**, not a sequence. The edge it *does* carry
> names [[RT-CONTSPEC-LOWER]], which is a genuine *after*.
>
> ⚠ **For whoever publishes:** ⛔ do not describe this node's recursor code as a
> `D7` deliverable in a PR body or a merge post — and at slices 1 and 2, ⛔ do
> not describe the PR as delivering **either** node. They deliver a slice.

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
**first** and its mechanism has been built against real refusals, so the risk
this section hedged against — learning of infeasibility after five nodes of
investment — is no longer the risk being run.
⚠ **It is no longer "in flight" either** — as of 2026-08-01 it is frozen and
delivers through the four slices in the top banner. ⛔ Read that banner, not
this paragraph, for the current delivery shape.

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

`docs/program/wp/RT-RECURSOR-TRANSPORT.md`. ⭐ Campaign context, the binding traps
that bind every node in this arc, and the full schedule:
`docs/program/16-recursive-descent-retirement.md` — **read it before the frame.**
