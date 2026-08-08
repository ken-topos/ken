---
id: RT-PRODUCER-MATCH-PORT
title: "Producer-match call port — an ordinary Match whose scrutinee is directly a Call routes the whole object to RecursiveDescent"
status: active
owner: runtime
size: M
gate: none
depends_on: [RT-SEED-CALL-PORT]
blocks: [RT-DESCENT-RETIRE]
github: null
origin: Operator directive 2026-07-29 — prioritize replacement of RecursiveDescent, migrate the remaining residual classes, do not linger half-migrated. Campaign docs/program/16-recursive-descent-retirement.md. Steward-filed (agents cannot create tracked work per COORDINATION §2).
---

## What it is

```rust
RuntimeExpr::Match { scrutinee, cases, .. } =>
    matches!(scrutinee.as_ref(), RuntimeExpr::Call { .. })
        .then_some(RecursiveDescentResidual::ProducerMatchCall)
```

⭐ **It is the *first* test in the `Match` arm** (`core.rs:92-95`), so it
short-circuits before `MatchScrutineeRecursor` and before any recursion into the
scrutinee or the case bodies. ⇒ While this class fires, the real population of
[[RT-RECURSOR-TRANSPORT]]'s classes is **partially masked**, which is why this
node is sequenced ahead of it and why every `D1` in this campaign must enumerate
*all* residuals rather than the reported first.

The mechanism gap: a producer `Call` in scrutinee position must deliver its
result across a callable-unit boundary into the match, rather than being
recursively lowered into the generated root with the match.

## Sequencing

**Fifth in Runtime's queue.** ⚠ The `depends_on` edge is file contention on
`crates/ken-runtime/src/cranelift_backend/lowering/core.rs`, not a mechanism
dependency — see the campaign doc's schedule section, which states the ordering
rationale so it can be reversed on evidence.

## ▶ THE FRAME IS WRITTEN

`docs/program/wp/RT-PRODUCER-MATCH-PORT.md`. ⭐ Campaign context, the binding traps
that bind every node in this arc, and the full schedule:
`docs/program/16-recursive-descent-retirement.md` — **read it before the frame.**
