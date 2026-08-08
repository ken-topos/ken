---
id: RT-FNUNIT-RESULT-TOKEN
title: "Broad starter shapes fail the result-token table on the FunctionizedUnits lane — pre-existing, unmasked by retiring SeedClosureCall"
status: ready
owner: runtime
size: M
gate: none
depends_on: []
blocks: [RT-DESCENT-RETIRE]
github: null
origin: Measured 2026-08-08 by the Runtime ring during RT-SEED-CALL-PORT D3, evidence SHA d6fb593b. Campaign docs/program/16-recursive-descent-retirement.md Trap 2 — a newly reachable shape tripping a fail-closed invariant is routed as its own node. Steward-filed (agents cannot create tracked work per COORDINATION §2).
---

## What it is

`nc22_cranelift_agrees_with_runtime_ir_report_for_broad_starter_shapes`
(`crates/ken-runtime/src/cranelift_backend/artifact/api/tests.rs:95`) fails on
the `FunctionizedUnits` lane with:

```
native result token 265 is not in the result table
```

raised at `cranelift_backend/surface.rs:251` / `:315`.

## The attribution is measured, not argued

**This is NOT caused by the seed-closure port.** The Runtime implementer flipped
`nc22`'s callee from `RuntimeExpr::Closure` to `RuntimeExpr::LexicalClosure` —
the sibling arm live since [[RT-DECL-CLOSURE-PORT]], which
[[RT-SEED-CALL-PORT]]'s `D2` and `D3` never touched — and reproduced the
**identical** error.

⇒ **The shape was already unsupported on the functionized lane.**
`SeedClosureCall` was masking it: while the residual fired, the program routed
to the `RecursiveDescent` root and never reached the failing path. Retiring the
variant made it newly reachable.

**Discounted evidence, recorded so nobody re-counts it:** an earlier smaller
record-returning probe failed on both arms with a *different* error
(`BoundaryCarrier` unsupported). The implementer explicitly declined to
attribute this stop to it. Only the `nc22` callee flip attributes.

## Why it is its own node

Campaign Trap 2, verbatim: a newly reachable shape tripping a fail-closed
invariant is **expected** as classes retire, and is routed as its own node
rather than absorbed into the retiring node or worked around by adjusting the
lane.

## It owns the quarantine

`nc22` is **skipped** so [[RT-SEED-CALL-PORT]] `D3` can land (Steward ruling,
2026-08-08, under the operator's 2026-08-06 CI-gate policy). **This node owns
un-skipping it.** A skipped row measures nothing, so closing this node means
`nc22` runs green on the functionized lane, not that the skip is tidied.

## Why it blocks `RT-DESCENT-RETIRE`

[[RT-DESCENT-RETIRE]] **deletes the `RecursiveDescent` emission lane.** Any
shape supported only there stops being supported at all. This shape is
currently supported only there, so it must work on the functionized lane before
that lane is deleted — otherwise the retirement silently narrows what Ken can
compile.

## First questions for whoever picks this up

- **What is result token 265, and why is it absent from the table?** Start at
  `surface.rs:251`/`:315` and work back to who populates the table on the
  functionized path.
- **Is the gap the token's production or its registration?** Those route
  differently.
- **How wide is the shape class?** `nc22` is one fixture. **The `M` was set
  without that answer, and the answer cannot come from reading this corpus** —
  see below.

## THE SIZING QUESTION IS SCOPING, NOT MEASUREMENT

**Measured by the Adversary on the closing merge `91435b89`:**

1. **`nc22` is a single composite program, not a loop over shapes.** Its body is
   one nested `Let` / `Call{callee: Closure}` / `Match` / `Construct` / `Record`
   / `If` tree — "broad starter shapes" names breadth *within* one program. The
   skip suppresses **one row**, not a corpus.
2. **`nc22` is the ONLY one of 21 `nc` fixtures carrying a `Call` whose callee
   is a `Closure` or `LexicalClosure`.** It is alone.

⇒ **The family-width question is not merely unmeasured — it is
UNESTABLISHABLE from this corpus**, which holds exactly one instance of the
failing shape. Answering it **requires authoring fixtures that do not exist.**

⇒ **And the skip's consequence is wider than one lost row: this corpus now has
ZERO live coverage of that shape in either direction.** Nothing in it will
observe the wall move — **not a repair, and not a regression.**

**So the open question for whoever picks this up is scoping, not measurement:**
is authoring the missing fixtures inside the `M`, or was `M` sized for a repair
against one known fixture? **Answer that before starting.** If it is the
latter, the node is mis-sized and comes back to the Steward.
