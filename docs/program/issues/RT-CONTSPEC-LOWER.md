---
id: RT-CONTSPEC-LOWER
title: "ContinuationSpecialization slice 3 — attach the token at each producer alternative, emit the direct call before the identity-erasing join, close nested recursion and the ledgers, then ACTIVATE"
status: ready
owner: runtime
size: L
gate: none
depends_on: [RT-CONTSPEC-ABI]
blocks: [RT-RECURSOR-TRANSPORT, RT-DECL-CLOSURE-PORT]
github: null
origin: "Architect second WIP audit evt_4t09329vdrf (2026-08-01), outcome (c). Slice 3 of the Steward's staged recut of RT-RECURSOR-TRANSPORT + RT-DECL-CLOSURE-PORT D7. This is the slice that activates."
---

> # ✅ READY 2026-08-01 — SLICE 3 OF 4, THE ONLY SLICE THAT ACTIVATES
>
> ⭐ **THE FRAME IS COMPLETE**: `docs/program/wp/RT-CONTSPEC-LOWER.md`. All four
> `▢ SLOT`s were filled from the merged slice 2 (PR #1303,
> `origin/main = 3cc4fa19`), measured **in the landed code**, not read off slice
> 2's frame. ⛔ **Read the frame, not this node.**
>
> ## ⛔⛔ SLOT B CAME BACK NEGATIVE — READ THIS BEFORE ANYTHING ELSE
>
> **Neither of slice 2's gates reaches a lowered call.** `D3`
> (`validate_continuation_specializations`) validates the descriptor **table** —
> count, `id == index`, arm identity. `D4` is scoped to the **install** path in
> its own words (`abi.rs:597–603`): it refuses *capacity growth while appending*,
> and says nothing about allocation on a lowered call.
>
> ⇒ ⭐ **This slice inherits NOTHING at the call site and must build its own
> owner / lifetime / affinity / allocation gates there.** An owner gate that
> holds for a descriptor and not for the call consuming it is exactly the hole
> this slice would otherwise ship.
>
> ⚠ `D4`'s positive control is `SKIP_CONTINUATION_ABI_PREFLIGHT`, a
> `#[cfg(test)]` thread-local. Sound for what it covers; ⛔ **not evidence about
> the lowered path.**
>
> ⭐ **Slice 2 was approved on the FIRST round** (QA `evt_74mmepexexppy`,
> Architect `evt_5396qdh7a7p32`, `dec_77b33m1pbahng`) against slice 1's four —
> nothing was deferred into this slice. That is the staging working, and it is
> the reason to hold this slice's controls to the same standard, not relax them.

## RECUT 2026-08-02 — read the frame's amendment before resuming

The Architect rejected exact `9d58df12` (`evt_70ssyb45tk3v3`,
`dec_1smwstxyhh1q5`). **The `CheckedFrameBranchScope` mechanism is accepted and
preserved unchanged**; the candidate is held, not published. Two blockers, both
repaired in the frame's `Amendment 2026-08-02` section:

1. **The candidate breaks its own in-crate test fixtures.** All four blobs under
   `lowering/core/tests/` are byte-identical to the green base, so all 49
   compile errors are candidate-attributable. They are `#[cfg(test)]`, so only a
   test compile sees them, and CI runs `cargo test --workspace --locked`.
   **Atomic repair with the capstone is forced** — the fixtures compile at the
   base, so nothing can land ahead of it. New `D8`, `AC-9`, `AC-10`.
2. **QA covered a fraction of the frame's surface.** No prior verdict transfers;
   a fresh SHA needs a fresh round giving one verdict per named obligation.

**49 is the floor, not the ceiling** — these tests have never run against this
candidate. The assertion failures behind the compile errors are unmeasured, and
a class-2 failure (an assertion the slice deliberately moved) is `AC-6`
activation evidence, not a fixture to edit.

## Scope, as ruled

Attach the exact call token at each producer alternative; emit the direct
call/return **before the identity-erasing join**; remove the dynamic case's
active-scalar/search route; close nested recursion and the ledgers; then run
E-1 / E-5 / E-7, the full 19-row population, and literal all-check CI.

## ⚠ This slice carries the campaign's real risk, and that is deliberate

Everything that needs a running native binary to prove is **here**, and nothing
that does not need one is. That is the split the audit's finding demands: the
first 30-hour run and the corrected run both hid their defects inside a breadth
where a planner error and a lowering error were indistinguishable from the
outside.

⭐ **When this merges it closes THREE nodes** — itself, [[RT-RECURSOR-TRANSPORT]]
and [[RT-DECL-CLOSURE-PORT]] — in ⛔ **one** tracker commit. Do not flip them
separately and do not describe this slice's recursor code as a `D7` deliverable.

⚠ **The 761 witness gate carries forward unchanged:**
`fs_read_at_malformed_offset_narrows_to_invalid_offset` must produce
`InvalidOffset`, and its sibling at `crates/ken-cli/tests/rt_parity_native.rs:544`
is covered by the same question — ⛔ did the trap become `InvalidOffset` because
the defect was **fixed**, or because the assertion **moved**?
