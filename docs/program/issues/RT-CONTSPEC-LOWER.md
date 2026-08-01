---
id: RT-CONTSPEC-LOWER
title: "ContinuationSpecialization slice 3 — attach the token at each producer alternative, emit the direct call before the identity-erasing join, close nested recursion and the ledgers, then ACTIVATE"
status: draft
owner: runtime
size: L
gate: none
depends_on: [RT-CONTSPEC-ABI]
blocks: [RT-RECURSOR-TRANSPORT, RT-DECL-CLOSURE-PORT]
github: null
origin: "Architect second WIP audit evt_4t09329vdrf (2026-08-01), outcome (c). Slice 3 of the Steward's staged recut of RT-RECURSOR-TRANSPORT + RT-DECL-CLOSURE-PORT D7. This is the slice that activates."
---

> # ⚠ SLICE 3 OF 4 — THE ONLY SLICE THAT ACTIVATES
>
> ⏳ `draft` until its frame is written.
>
> ⛔ **Do not pick this up from the node text.** Its frame is written after
> [[RT-CONTSPEC-ABI]] is framed, and deliberately last: slices 1 and 2 fix
> exactly what this slice can assume, and a frame written now would be sized
> against a planner and ABI that do not exist yet.

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
