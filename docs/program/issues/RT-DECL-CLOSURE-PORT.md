---
id: RT-DECL-CLOSURE-PORT
title: "Transparent-declaration-closure emission port — a retained TransparentDeclarationClosure residual forces the whole object onto the monolithic RecursiveDescent root, which exceeds Cranelift's per-function ceiling"
status: merged
owner: runtime
size: L
gate: none
depends_on: []
blocks: [PX8-ERRID-ALLOC, RT-SEED-CALL-PORT, NATIVE-HANDLE-CARRIER, RT-RECURSOR-TRANSPORT]
github: null
origin: Architect ruling evt_3t7t27e3rv8cx (2026-07-29), outcome 2 on the Steward's PX8-ERRID-ALLOC wall discriminator (evt_s2kv0wttb5f7). Measured by the Architect in a detached scratch worktree with diagnostic-only labels against exact ad7298fb. Steward-filed (agents cannot create tracked work per COORDINATION §2).
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


> # ⛔⛔ RECUT 2026-08-01 — ITS `D7` NO LONGER DELIVERS DIRECTLY
>
> ⛔ Read this before anything below.
>
> **The Architect's second WIP audit (`evt_4t09329vdrf`) returned outcome (c):
> the `D7` + [[RT-RECURSOR-TRANSPORT]] contract is mis-sized as one implementer
> bite.** ⛔ **The semantics are NOT recut** — `evt_7dhwrk26ks9m0` stands and the
> mechanism direction was confirmed correct in the same audit. Only the delivery
> shape changed.
>
> ⇒ **`D7` now lands through four staged slices** —
> [[RT-CONTSPEC-SUBSTRATE]] (dormant `D7` substrate) →
> [[RT-CONTSPEC-PLANNER]] (planner closure, dormant) →
> [[RT-CONTSPEC-ABI]] (unit/descriptor + ABI gates, dormant) →
> [[RT-CONTSPEC-LOWER]] (branch lowering, ledgers, witness, CI — **activates**).
> **This node flips `merged` in one commit with the other two when slice 3
> merges.**
>
> ⚠ **Everything below this banner about `AC-1`, the three-candidate wall, and
> the `depends_on` edges is UNCHANGED and still binding.** ⛔ What is stale is
> any statement that this node's work is in flight as a single candidate on
> `wp/RT-DECL-CLOSURE-PORT` — it is not; that branch is free and the frozen
> prototype is `origin/preserved/rt-recursor-freeze-465fab90`.

> # ▶ ACTIVE 2026-07-29 — REORDERED TO **NEXT**, AND IT NOW HOLDS **THREE** NODES
>
> **Steward disposition `evt_5mtkdft1nxmwp`.** [[NATIVE-HANDLE-CARRIER]] was
> released ahead of this node, rebased cleanly, and stopped at **11/12** on
> **this node's `AC-1` row** —
> `fs_write_at_malformed_offset_narrows_to_invalid_offset`, `Code for function is
> too large`, candidate-caused (the row passes `1/1` on detached
> `main = af056a78`). It is preserved at `85dcee25` and now `depends_on` this
> node.
>
> ⇒ **Three independent candidates reach this one ceiling on this one row:**
> Foundation's `ad7298fb`/`e65c81b5`, Runtime's `85dcee25`, and the row is
> already recorded as the only one of seven opening **two nested resource
> brackets** (`CI-SKIPPED-NATIVE-TESTS`). ⭐ One wall, three ways in.
>
> ### ⛔⛔ `AC-1` WAS AMENDED — IT NOW REQUIRES **TWO** DELTAS
>
> It read *"a tree carrying `ad7298fb`'s semantic delta"* — Foundation's only.
> **This node could have landed fully green and `NATIVE-HANDLE-CARRIER` would
> have resumed and still been red**, after the queue was reordered to fix exactly
> that. The row must now pass on **both** `ad7298fb`'s and `85dcee25`'s deltas.
> Read the frame, not this summary.
>
> ### ⭐ Held by this node
>
> [[PX8-ERRID-ALLOC]] (→ [[PX8-ERRID-SCOPE]] → `PX8`, which gates 15 of the ABI
> program's 19 nodes) · [[NATIVE-HANDLE-CARRIER]] (→ [[PX8-F-CAP-41]]) ·
> [[RT-SEED-CALL-PORT]] and the rest of the retirement campaign, whose shared
> residual enumerator this node's `D1` builds.

> # ⭐⭐ WHY THIS NODE EXISTS — an unmeasured sequencing premise held a ring
>
> **[[PX8-ERRID-ALLOC]] was held on [[RT-NATIVE-FNSPLIT]] from 2026-07-28 on the
> premise that closing the per-function growth gate would clear its code-size
> wall. That premise was a Steward scope inference. It was never measured, and
> on 2026-07-29 it was measured and found false.**
>
> Foundation rebased the preserved candidate onto current `main` and re-ran the
> exact gate: it **still fails**, identically. The Architect then reproduced it
> with diagnostic labels and ruled **outcome 2** — the arc is not defective, this
> fixture simply never enters it.
>
> ⛔ **[[RT-SCALE-B]]'s verdict is NOT falsified and must not be reopened.** It
> was bounded to representation growth over *governed recursive resource-bracket*
> populations and explicitly excluded the mutually exclusive `RecursiveDescent`
> root. It never claimed this fixture compiles. **The false claim was the
> Steward's sequencing edge, not the ring's measurement.**

## The measurement (Architect, `evt_3t7t27e3rv8cx`)

Bound exact `ad7298fb80128d43e430d427b71f8aa16a9336aa` / tree `77ece013`, base
`origin/main = eef0cb06`, protected `e65c81b5` / tree `102c54f8`. Run in a
**detached scratch worktree with diagnostic-only labels** at the selector and the
three possible definition seams — ⭐ no candidate or production change survived
that worktree.

```text
authority=RecursiveDescent
residual=TransparentDeclarationClosure
declaration=...::buffer_nat_to_int residual=TransparentDeclarationClosure
declaration=...::main             residual=TransparentDeclarationClosure
```

```text
PX8_ERRID_DIAGNOSTIC RecursiveDescent root:
Compilation error: Code for function is too large
```

⇒ The oversized function is **the `RecursiveDescent` root itself** — not a
functionized unit, not the functionized root adapter, not a fixed helper graph.
**`FunctionizedUnits` declares and defines *zero* semantic units on this route.**

## The mechanism constraint

**The selector is whole-program and all-or-nothing: *any* retained declaration
residual selects `RecursiveDescent` for the entire object.** Transparent
declarations whose bodies are closure seeds are deliberately one such retained
row. On that authority, declaration bodies are **recursively lowered into the
generated root** instead of being reached as separately owned callable units.
That root exceeds Cranelift's per-function ceiling once the `AllocationFailed`
projection is present.

## What this node must build

A Runtime-owned **transparent-declaration-closure emission port**:

1. **Planner-owned callable declaration units** — declarations become separately
   owned callable units rather than bodies inlined into the root.
2. **Typed capture / parameter / result / trap transport** across that boundary.
3. **`DeclarationRef` calls** to those units.
4. **Complete owner/phase validation** — this must be in place *before*
   `TransparentDeclarationClosure` may be removed from the retained residual.

⛔ **Two shortcuts are named and banned by the ruling.** Merely **deleting the
selector residual**, or **selectively inlining fewer declarations**, would be an
unproved shortcut. Neither is this node's deliverable.

## What is explicitly NOT authorized

- ⛔ **A second [[PX8-ERRID-ALLOC]] size reduction.** The feature delta is
  **exonerated**; shrinking its identity mapping would trade semantics for bytes.
  Foundation correctly stopped without attempting one.
- ⛔ **The `D4` constants-reduction follow-on is NOT this.** There is no
  functionized semantic function here to reduce.
- ⛔ **Reopening [[RT-NATIVE-FNSPLIT]].** It closed on its stated gate and that
  gate was met.

## Preserved refs — do not disturb

| ref | what |
|---|---|
| `ad7298fb80128d43e430d427b71f8aa16a9336aa` | the rebased semantic candidate, tree `77ece013` |
| `preserved/PX8-ERRID-ALLOC-e65c81b` = `e65c81b5` | the protected measured input, tree `102c54f8` |

**Foundation owes no restart until this port lands.**

## ▶ THE FRAME IS WRITTEN

`docs/program/wp/RT-DECL-CLOSURE-PORT.md` — read it, not this file, for scope and
acceptance. ⭐ Its `§3` carries the trap that decides the node: this residual is
**one of five**, the selector short-circuits at the first hit, so retiring it does
**not** entail the fixture reaches `FunctionizedUnits`. `AC-1` is therefore a
compile, not a code-shape assertion.

## Sequencing

Runtime-owned and **third** in Runtime's queue:
**[[RT-JOIN-DISPOSITION]] → [[NATIVE-HANDLE-CARRIER]] resume → this node.**

⭐⭐ **This node is now the keystone of a seven-node campaign
(operator, 2026-07-29): `docs/program/16-recursive-descent-retirement.md`.** It
retires the first of five residual classes **and builds the closure-seed →
callable-unit machinery that [[RT-SEED-CALL-PORT]] and
[[RT-PRODUCER-MATCH-PORT]] reuse.** ⇒ Its `D2`/`D3`/`D4` are not local to this
node — **build them for generality**, and if the transport turns out not to
generalize, that is a reportable finding for the successors, not a private
detail. Successor chain: [[RT-SEED-CALL-PORT]] → [[RT-PRODUCER-MATCH-PORT]] →
[[RT-RECURSOR-TRANSPORT]] → [[RT-DESCENT-RETIRE]].

⭐ **On the Linux ABI I critical path** — it is the sole blocker of
[[PX8-ERRID-ALLOC]], which blocks [[PX8-ERRID-SCOPE]], which blocks [[PX8]];
`PX8` gates 15 of that program's 19 nodes. ⚠ **So this ordering has a real
cost** and the frame states it plainly rather than burying it: see the frame's
status block for the grounded reason and for how to reverse the call.
