---
id: RT-SEED-CALL-PORT
title: "Seed-closure call port — a Call whose callee is the retained non-lexical closure form routes the whole object to RecursiveDescent"
status: active
owner: runtime
size: M
gate: none
depends_on: [RT-DECL-CLOSURE-PORT]
blocks: [RT-PRODUCER-MATCH-PORT]
github: null
origin: Operator directive 2026-07-29 — prioritize replacement of RecursiveDescent, migrate the remaining residual classes, do not linger half-migrated. Campaign docs/program/16-recursive-descent-retirement.md. Steward-filed (agents cannot create tracked work per COORDINATION §2).
---

> ## THIS NODE MAY CLOSE FOR FREE, AND THAT IS A SUCCESS
>
> `SeedClosureCall` is a `Call` whose **callee** is the retained non-lexical
> closure form (`core.rs:121-124`). [[RT-DECL-CLOSURE-PORT]] builds the
> closure-seed → callable-unit machinery (`D2`/`D3`/`D4`), so this class may be
> **largely or wholly subsumed** by the time this node starts.
>
> **That is a prediction, not a measurement**, which is exactly why this is a
> node rather than a fold. Folding on an unmeasured prediction is the error that
> held a ring for a day on 2026-07-28. **If `D1` reports the class no longer
> fires on any measured program, close this node and move on** — do not
> manufacture work to justify it.

## What it is

```rust
RuntimeExpr::Call { callee, args } =>
    matches!(callee.as_ref(), RuntimeExpr::Closure { .. })
        .then_some(RecursiveDescentResidual::SeedClosureCall)
```

The callee is a **non-lexical `Closure`** — a closure seed in call position
rather than in a declaration body. [[RT-DECL-CLOSURE-PORT]] retires the same
seed form when it is a *transparent declaration body*; this node retires it at
the *call site*.

## Sequencing

**Fourth in Runtime's queue**, after [[RT-DECL-CLOSURE-PORT]] whose machinery it
reuses. The `depends_on` edge is genuine on two grounds: the deliverable
reuses that node's typed capture/parameter/result/trap transport, **and** both
edit `crates/ken-runtime/src/cranelift_backend/lowering/core.rs`, which cannot
be rewritten concurrently.

## THE FRAME IS WRITTEN

`docs/program/wp/RT-SEED-CALL-PORT.md`. Campaign context, the binding traps that
bind every node in this arc, and the full schedule:
`docs/program/16-recursive-descent-retirement.md` — **read it before the frame.**
