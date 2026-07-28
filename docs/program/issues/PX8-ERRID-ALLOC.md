---
id: PX8-ERRID-ALLOC
title: "ResourceErrorV1 has no allocation-failure identity and buffer allocation is infallible, so PX8's allocation-distinct-from-BufferLimit row cannot be produced at all"
status: draft
owner: foundation
size: M
gate: none
depends_on: [RT-NATIVE-FNSPLIT]
blocks: [PX8-ERRID-SCOPE]
github: null
origin: "Architect ruling evt_6tzss92ckj2by (2026-07-27) on the Steward's PX8-ERRID-SCOPE partition question. Split out because the Architect ruled this row 'inside, but currently not representable' and named it a prerequisite to the evidence work."
---

> ## ⛔ PR #1141 CLOSED 2026-07-28 — the candidate must be RE-CUT, not re-polled
>
> **The work is not rejected and nothing is lost.** `wp/PX8-ERRID-ALLOC` remains
> on origin at `763f0a4424a02ccc66179cbf94f7ad9dc244af82`.
>
> PR #1141 had been open ~6h and could not merge. Three independent blockers,
> measured against `origin/main = 2de20719`:
>
> | # | blocker | measured |
> |---|---|---|
> | 1 | **CI red** | `native-slow (rt_parity_native)` FAILURE · `build + test` FAILURE |
> | 2 | **`COORDINATION §14` intersection NON-EMPTY** | `crates/ken-interp/src/eval.rs` (merge-base `5404108a`) ⇒ a rebase **is** required |
> | 3 | **enabler incomplete** | `C1` merged 2026-07-28 (PR #1156); ⛔ **`B2F` is what this node actually waits on** |
>
> ⚠ **The intersection has since GROWN.** This candidate touches
> `cranelift_backend/lowering/core.rs`, `lowering/mod.rs`, and
> `lowering/core/tests/effects.rs` — all three landed in `C1`. ⇒ A re-cut is
> required **regardless** of the CI result; ⛔ do not spend a cycle chasing
> `rt_parity_native` against this base.
>
> ⛔ **`dec_7jwry2zxze6qr` reads `resolved` / APPROVE for exact `763f0a44`, and
> that is NOT sufficient to merge.** It predates both the red CI and the
> divergence. A fresh candidate needs a **fresh exact-SHA Decision**.
> ⭐ An approved-but-unmergeable PR left open is a standing merge hazard — that
> is why it was closed rather than left to age.

> ## ⛔⛔ BLOCKED 2026-07-28 ON `RT-NATIVE-FNSPLIT` — Steward sequencing call
>
> **The work is BUILT, QA-approved, and Architect-approved. It cannot land: the
> native lowering it needs does not fit under Cranelift's per-function code-size
> limit.** `depends_on` is now `[RT-NATIVE-FNSPLIT]`; ⛔ no cycle (that node has
> `depends_on: []`).
>
> ### What happened, in order
>
> | # | event | outcome |
> |---|---|---|
> | 1 | `b117039f` — QA approved, Decision `dec_2qnf5j09rs5xt` | ⛔ **Architect REJECTED**: `AllocationFailed` inserted at a dynamic positional tag shifted four detail codes, so native observed `InvalidOffset` as `AllocationFailed`, etc. Existing green evidence could not catch it — the R2 Ken fixture accepted any `Err` while its Rust assertion read the host trace *before* the misprojected constructor. |
> | 2 | `763f0a44` — keyed on **generated wire identities**; QA + Architect approved (`dec_7jwry2zxze6qr`) | ⛔ **PR #1141 CI RED** — `crates/ken-cli/tests/rt_parity_native.rs:370`, `Cranelift backend failure: Code for function is too large` (ObjectEmission, `checked_process_object`). Not merged; `origin/main` unchanged. |
> | 3 | `e65c81b5` — the only mapping-preserving reduction | ⛔ **Still fails**, identically, 117.05s. |
>
> ### ⭐⭐ The mapping correction is EXONERATED
>
> **Both `b117039f` and `763f0a44` reproduce the same failure against an
> *unchanged fixture blob*.** ⇒ **allocation growth crossed the wall, not the
> wire-identity correction.** ⚠ Two independent defects lived in one candidate:
> the shifted-tag misprojection was real and step 1's rejection was right — it
> simply was never the cause of the size failure.
>
> ### ✅ The reduction was measured, not guessed — and is insufficient
>
> `e65c81b559eebcb93c258f1d7cee39e66e832466` (tree
> `102c54f888f8d661f4103a908e141e8d42614da9`, parent `763f0a44`) factors
> `BufferLimit`/`InvalidOffset`/`InvalidBounds`/`NoProgress`/`AllocationFailed`
> into one generated-tag `require_one_of_i64` plus one shared eight-zero payload
> check, **preserving every payload-bearing arm and every generated identity**.
> Projection 5/5 PASS; nonzero-payload negatives + unknown identity PASS; and a
> compile-preserving causal flip swapping `AllocationFailed` for a duplicate
> `NoProgress` went **RED specifically at `AllocationFailed`** (`left -1`,
> `right 79`), restored to PASS.
>
> ⇒ ⭐ **The remaining headroom is not in this WP's delta.** Any further
> reduction would have to come out of the mapping, and ⛔ that is banned — it
> would reinstate the step-1 defect.
>
> ### ⛔ Standing constraints on the successor
>
> - ⛔ **Never buy bytes with positional tags.** Generated wire-identity
>   selection is the ruled mechanism (`dec_2qnf5j09rs5xt`).
> - ⛔ `dec_7jwry2zxze6qr` is **spent**; a successor needs a fresh Decision.
> - ⚠ **Not a QA miss.** Local `-p ken-runtime` runs (468 green) never link
>   `ken-cli`'s native parity programs. The CI gate worked as designed.
>
> ### ⚠ DURABILITY EXPOSURE
>
> `763f0a44` is off-box — the publisher pushed `wp/PX8-ERRID-ALLOC` for PR #1141.
> ⛔ **`e65c81b5` is LOCAL-ONLY**, and this box holds the only copy of a measured,
> controlled result needed the moment FNSPLIT lands. ⛔ Do not reset, delete, or
> repoint that branch. ⚠ **A recorded SHA is not a copy.**
>
> ### Resume when FNSPLIT lands
>
> Re-derive against the new base, then re-run the decisive command
>
> ```
> scripts/ken-cargo test -p ken-cli --test rt_parity_native \
>   fs_write_at_malformed_offset_narrows_to_invalid_offset
> ```
>
> then fresh QA → fresh Architect Decision → publish. ⭐ Start from `e65c81b5`
> (the reduction is correct and independently controlled), not from scratch.

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
