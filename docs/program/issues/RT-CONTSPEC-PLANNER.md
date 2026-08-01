---
id: RT-CONTSPEC-PLANNER
title: "ContinuationSpecialization slice 1 — land the planner closure DORMANT: exact ordered projection, full-key interning before discovery, exact causal edge tokens, finite recursion"
status: merged
owner: runtime
size: M
gate: none
depends_on: [RT-CONTSPEC-SUBSTRATE]
blocks: [RT-CONTSPEC-ABI]
github: null
origin: "Architect second WIP audit evt_4t09329vdrf (2026-08-01) returned outcome (c) — RT-RECURSOR-TRANSPORT + RT-DECL-CLOSURE-PORT D7 is mis-sized as one delivery. Steward-authored recut under playbook §5a-iii; the Architect diagnoses sizing, the Steward cuts. No semantic recut: the ruling at evt_7dhwrk26ks9m0 remains binding."
---

> # ✅ MERGED 2026-08-01 — PR #1298, `origin/main = 04cd9259`, CI GREEN
>
> Exact candidate `28bc225ba0756ff3095da32a1d3e6afd0505fb6e`, tree
> `2139ffc7c8dd66c775e93e1b4df70cc2c1776bbd`. Runtime QA `evt_ahgj4w4f54q7`;
> Architect approved `evt_2d5fd5csn19nv`; Decision `dec_124dnas2ffnjn` resolved.
> All three predicted post-conditions exact — landed tree
> `0cb30ed3539fc48e52d115f444b2b37ff92f242b`, blob
> `6af866f454f8fc89993f1dd2378335c80eb2f555` at the one planner path, and that
> path the only one changed.
>
> ## ⭐⭐ IT TOOK FOUR REVIEW ROUNDS, AND THE SHAPE IS THE LESSON FOR SLICE 2
>
> Three Architect rejects preceded approval — `dec_3ptbrzxz3cgyh`,
> `dec_7cy23z13jf0t8`, `dec_1xv1c0yaeabc`. ⛔ **That was NOT a sizing failure and
> the node was not recut.** The accepted surface grew monotonically: R1 named two
> D1 blockers; R2 ratified the ordinary-prefix/worker-envelope half; R3 ratified
> the semantic source-provenance half; R4 closed the input-population half.
> ⇒ ⭐ **The discriminator for a mis-sized WP is a FLAT accepted surface, not a
> reject count.**
>
> ⚠⚠ **Two of the three rejects turned on a DEGENERATE CONTROL, not on wrong
> production code** — R1's fixture had one worker and one capture, so both wrong
> answers equalled `1` (green-vs-green); R3's fixture had a `unit()` case body,
> so descriptor-count truncation was invisible to it. ⇒ ⭐⭐ **Slice 2 should
> budget its review effort on whether each control can DISCRIMINATE, not on
> whether the production code reads correctly.** The final fixture is the model:
> `Var(4)` makes ordinal 2 load-bearing, so exact production gives `[1,0,1]`
> while truncation gives `[1,0]` and descriptor restatement gives `[0,1]` —
> three distinguishable wrong answers, each named.
>
> ⚠ The specialization is **dormant**: no lowering, no unit emission, no ABI
> activation. [[RT-CONTSPEC-ABI]] is slice 2.

> # ✅ THE BASE FORK IS ANSWERED — `ready`, and gated on slice 0
>
> The Steward's fork at `evt_1bh3p4wx76wtv` was answered by the Architect at
> `evt_6wkw2c7ykjxsy` with a **third option**: neither `main` nor the proved
> oracle `93746ada` was a lawful base, so a **dormant slice 0**
> ([[RT-CONTSPEC-SUBSTRATE]]) lands the `D7` substrate first.
>
> ⇒ **This slice branches from `origin/main` AFTER slice 0 lands.** ⛔ Not from
> `93746ada`, ⛔ not from a preservation ref, ⛔ not from slice 0's branch.
>
> ⚠ **`depends_on` names [[RT-CONTSPEC-SUBSTRATE]] and that is a genuine RELEASE
> gate**, not a merge-ordering note: this slice cannot start until slice 0 is on
> `main`, because the authorities it consumes do not exist until then.

> # ⭐ SLICE 1 OF 4 — A DELIVERY SHAPE, NOT A NEW DESIGN
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
> | 0 — [[RT-CONTSPEC-SUBSTRATE]] | dormant `D7` substrate | ⛔ **no** |
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
