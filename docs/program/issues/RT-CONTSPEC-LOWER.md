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
> ⏳ **`draft` until the frame's four `▢ SLOT`s are filled**, which happens at
> [[RT-CONTSPEC-ABI]]'s merge.
>
> ⛔ **Do not pick this up from the node text** — and do not pick it up from the
> frame yet either.
>
> ⭐ **THE FRAME IS WRITTEN**: `docs/program/wp/RT-CONTSPEC-LOWER.md`, filed
> 2026-08-01 under operator standing policy §2a-bis (a node with a team must
> have a framed successor). Everything slice 2 **cannot** change is final in it
> — the risk posture, the six-item ruled scope, eight ACs, the banned scope, the
> 761 witness gate, the three-node tracker discipline, and the sizing analysis.
>
> ⚠ **What genuinely had to wait is marked `▢ SLOT` and is the only work owed at
> slice 2's merge:** the ABI surface slice 2 actually landed (A), which of its
> gates bind at the call site (B), the residual after its review (C), and a
> contention re-derivation (D). ⭐ Minutes of Steward work, not an hour.
>
> ⛔ The earlier instruction — *"frame this last or it will be sized against a
> planner and ABI that do not exist yet"* — was **sound for the interface facts
> and is preserved as those slots.** It was not grounded for anything else, and
> leaving the whole frame unwritten would have left the runtime ring with no
> framed successor the moment slice 2 merged.

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
