---
id: PX8-ERRID-ALLOC
title: "ResourceErrorV1 has no allocation-failure identity and buffer allocation is infallible, so PX8's allocation-distinct-from-BufferLimit row cannot be produced at all"
status: ready
owner: foundation
size: M
gate: none
depends_on: []
blocks: [PX8-ERRID-SCOPE]
github: null
origin: "Architect ruling evt_6tzss92ckj2by (2026-07-27) on the Steward's PX8-ERRID-SCOPE partition question. Split out because the Architect ruled this row 'inside, but currently not representable' and named it a prerequisite to the evidence work."
---

**Frame:** `docs/program/wp/PX8-ERRID-ALLOC.md`, inputs pinned by blob at
`origin/main = e754508b`.

⭐ **On the Linux ABI I critical path.** `PX8` gates 15 of that program's 19
nodes; this is a prerequisite to [[PX8-ERRID-SCOPE]], one of `PX8`'s three
blockers.

## The measurement

`crates/ken-host/src/effect_v1.rs:592-613` — `ResourceErrorV1` is a **closed
sum** with no allocation-failure identity:

```
Closed · MalformedResource · ResourceKindMismatch · RightNotHeld
ReleaseFailed · BufferLimit · InvalidOffset · InvalidBounds · NoProgress
```

`:661` — allocation is **infallible**: `bytes: vec![0; capacity]`, which aborts
the process on exhaustion rather than returning an error.

`:829`, `:834` — `BufferLimit` is returned for **policy/width admission**, which
the Architect ruled is a different thing from allocator exhaustion.

⇒ **The row is not merely untested — it is unproducible.** There is no identity
to return and no fallible path to return it from.

## The Architect's ruling (`evt_6tzss92ckj2by`), verbatim on the constraints

> *policy refusal is not allocator exhaustion … this row is **not yet a
> tests-only WP**. It first needs one explicit, engine-neutral resource error
> identity (the direct shape is a nullary `AllocationFailed`, subject to the
> normal Spec/CV spelling lane) and fallible allocation that returns it before
> minting a resource or incrementing live capacity.*

⛔ Three named prohibitions:

1. ⛔ Do **not** encode allocator failure as `ResourceHostIO Other(errno)`.
2. ⛔ Do **not** alias it to `BufferLimit`.
3. ⛔ Do **not** test a synthetic error that production cannot emit.

⭐ **Precedence is ruled:** `BufferLimit` retains precedence for deterministic
policy/representability rejection; **only an admitted allocation that cannot
reserve storage** reaches allocation failure.

## ⚠ This is a closed-sum widening, so it has a spelling lane

Adding a variant to `ResourceErrorV1` changes the wire/ABI surface and needs a
checked-Ken binding (`:397` shows the existing
`generated_binding("error", "resource.BufferLimit")` pattern). ⇒ **Spec/CV own
the spelling**; the carrier and the fallible producer are the build work.

## What this does NOT cover

⛔ The production-reaching evidence for all five PR-C error identities is
[[PX8-ERRID-SCOPE]] and stays there. This node delivers only the identity and
the mechanism that can emit it.
