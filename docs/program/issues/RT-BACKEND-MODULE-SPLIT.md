---
id: RT-BACKEND-MODULE-SPLIT
title: "Split the oversized ken-runtime backend files into modules — the follow-on to the recursive-descent retirement, not an interlude in it"
status: draft
owner: runtime
size: M
gate: none
depends_on: [RT-DESCENT-RETIRE]
blocks: [NATIVE-HANDLE-CARRIER]
github: null
origin: Operator directive 2026-07-31 — the ken-runtime backend files are oversized again; a previous interlude of this shape produced the cranelift_backend/ directory. Operator asked whether to repeat it now or after the campaign, and confirmed AFTER on the Steward's recommendation. Campaign docs/program/16-recursive-descent-retirement.md §4 node #8. Steward-filed (agents cannot create tracked work per COORDINATION §2).
---

> ## ⛔ DELIBERATELY UNFRAMED UNTIL [[RT-DESCENT-RETIRE]] MERGES
>
> This node is `draft` on purpose, and ⛔ it must **not** be flipped `ready`
> before the capstone lands. [[RT-DESCENT-RETIRE]] **deletes** the classifiers,
> `RecursiveDescentResidual`, `BodyEmissionAuthority::RecursiveDescent` and the
> whole recursive-descent emission lane across exactly the files this node
> splits. ⭐ **The deletion changes where the natural module seams are**, so a
> frame written now would be sized against a tree that is about to disappear.
>
> ⇒ The frame is owed **after** #7 merges, measured on the **post-retirement**
> tree. ⛔ Do not carry today's line counts into it.

## What it is

The `ken-runtime` backend has files well past the crate's ~2k-line average
(97,881 lines across 49 files). Measured on `main = 1e6eb5c6`:

| file | lines |
|---|---|
| `cranelift_backend/lowering/mod.rs` | 11,197 |
| `cranelift_backend/lowering/core/tests/control.rs` | 9,847 (test) |
| `cranelift_backend/lowering/core.rs` | 9,788 |
| `cranelift_backend/planning/static_transition.rs` | 9,034 |
| `boundary_value_clif.rs` | 8,691 |

⚠ `static_transition.rs` reaches **>20,858** in [[RT-RECURSOR-TRANSPORT]]'s
in-flight delta — the largest single file in the crate by a wide margin.

Subtree totals: `lowering/` 40,180 · `planning/` 13,364 · `artifact/` 2,220.

## ⭐ Why this is cheaper than the precedent it is modelled on

The original interlude **created** `cranelift_backend/` from a monolith. This one
does not have to invent a structure: `static_transition.rs` **already has** a
sibling `static_transition/` directory holding `semantic_ir.rs` (2,729) and
`abi.rs` (1,601), and `lowering/` is already a directory. ⇒ This node **extends
established seams** rather than designing new ones.

## Sequencing

**Node #8**, immediately after [[RT-DESCENT-RETIRE]]. The full ruling and its
three grounds are in `docs/program/16-recursive-descent-retirement.md` §4 — read
that before framing this. In brief:

1. ⭐⭐ #7 **subtracts** from exactly these files, so splitting first re-homes a
   lane that is then deleted out of its new home — paid twice.
2. The two remaining ports are **consumers** of the transport, not authors, and
   both frames ban building a second one ⇒ the size peak is roughly now.
3. ⛔ A split and the campaign **contend on the same files** and cannot run
   concurrently, so this is purely an ordering question.

## ⚠ The open question this node does NOT settle

Whether large files are themselves making the campaign work harder. No evidence
was found for it — [[RT-DECL-CLOSURE-PORT]]'s three hard stops were **semantic**,
not navigational — ⛔ but that is a Steward inference from reports, not a
measurement, and the ring is better placed to judge it.

⭐ **The cheap test, and it is owed before #5:** at [[RT-RECURSOR-TRANSPORT]]'s
merge, ask the Architect whether a **narrow** split of `static_transition.rs`
alone should ride ahead of [[RT-PRODUCER-MATCH-PORT]] — the only remaining node
that must do real work inside that file. ⛔ That is one exchange, not this node;
a "yes" carves a slice off #8 and does not reorder it.
