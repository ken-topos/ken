---
id: RT-DESCENT-RETIRE
title: "Retire RecursiveDescent — delete the migration selector, the residual enum, the authority variant, and the recursive-descent emission lane"
status: ready
owner: runtime
size: M
gate: none
depends_on: [RT-DECL-CLOSURE-PORT, RT-SEED-CALL-PORT, RT-PRODUCER-MATCH-PORT, RT-RECURSOR-TRANSPORT, RT-FNUNIT-RESULT-TOKEN]
blocks: []
github: null
origin: Operator directive 2026-07-29 — "we should not let it linger in a half-migrated state. That just carries tech debt for no benefit." Campaign docs/program/16-recursive-descent-retirement.md. Steward-filed (agents cannot create tracked work per COORDINATION §2).
---

> # RETIRING ALL FIVE RESIDUAL CLASSES IS NOT THE FINISH LINE
>
> With every class retired, the selector still exists, still evaluates on every
> compilation, and the `RecursiveDescent` emission lane is still compiled in —
> **dead**. **That residue IS the tech debt the directive names.** So this is
> a required node, not a tidy-up, and it is the node that actually banks the
> efficiency win.
>
> **Done is:** `select_body_emission_authority`, `RecursiveDescentResidual`,
> `declaration_recursive_descent_residual`, `recursive_descent_residual`,
> `BodyEmissionAuthority::RecursiveDescent` and the recursive-descent emission
> lane are **deleted**, and every program compiles through `FunctionizedUnits`.

## Why it is its own node and not a coda on the last migration

**Because a deletion this wide has a different risk profile than a port**, and
folding it into [[RT-RECURSOR-TRANSPORT]] would let "the last class is retired"
be reported as "the lane is gone." Those are different claims, and only the
second is the directive.

The lane's surface at `origin/main = 14c3c5f7` spans **five production files** —
`lowering/core.rs`, `lowering/mod.rs`, `planning/static_transition.rs`,
`object_linker_packaging.rs`, and the `core/tests/` control modules. A
deletion that misses a file leaves a dead branch that still compiles.

## The dead-code oracle is spent by the commit that clears it

Once the last residual class is retired, **nothing in the tree can any longer
distinguish "the lane is unreachable" from "the lane was deleted."** The
evidence that the lane is dead exists only *before* this node lands.

⇒ **`D1` captures that evidence first**, while it is still capturable, and the
acceptance criteria are written against it. Do not start deleting and then try
to prove the lane was dead.

## Sequencing

**Last** in the campaign, gated on the four migration nodes **and on
[[RT-FNUNIT-RESULT-TOKEN]]**. This is the only node here whose `depends_on`
list is a genuine mechanism dependency rather than file contention — it cannot
land until every class is retired.

**The fifth edge is a different kind of dependency and was added 2026-08-08
by the Steward** (sequencing call; the node was filed that morning, after this
list and the campaign DAG were written). The four migration edges say *the lane
is no longer selected*. `RT-FNUNIT-RESULT-TOKEN` says *the lane is no longer
needed* — it owns `nc22`, currently the only program exercising a shape that
**only the `RecursiveDescent` lane supports**.

**Landing this node first would silently narrow what Ken can compile, and
nothing would fail.** `nc22` is `#[ignore]`d under that node's own quarantine,
so the one witness is already suppressed; deleting the lane under a skipped row
retires the fallback and the detector together. Un-skipping `nc22` green on the
functionized lane is that node's closure condition, and it is this node's
release gate.

## ▶ THE FRAME IS WRITTEN

`docs/program/wp/RT-DESCENT-RETIRE.md`. Campaign context, the binding traps that
bind every node in this arc, and the full schedule:
`docs/program/16-recursive-descent-retirement.md` — **read it before the frame.**
