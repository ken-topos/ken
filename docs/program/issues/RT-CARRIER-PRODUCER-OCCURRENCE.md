---
id: RT-CARRIER-PRODUCER-OCCURRENCE
title: "a source aggregate reaches the carrier with no planner-issued producer occurrence, so the C2 edge refuses to emit and the nested-payload selection row never exercises its property"
status: draft
owner: runtime
size: TBD
gate: none
depends_on: [RT-SRCBODY-BIND-ORDER]
blocks: []
github: null
origin: Measured by the RT-SRCBODY-BIND-ORDER all-eight-package two-ended census (evt_ksrhrv82t5ae), after CI failed this row at candidate fb99d0fc. Fails identically at frozen base 21fd46dc, so it is pre-existing base debt and not a regression from D1. Fits no released owner; the ring stopped and reported rather than assigning a nearest fit. Steward-filed (agents cannot create tracked work per COORDINATION §2).
---

> ## THE FRAME IS OWED. `draft`, NOT startable.
>
> It exists so a skipped CI row has an owner. **A skipped row measures
> nothing; this node owns un-skipping it.** Size is `TBD` deliberately.

## Exact signature

```text
Unsupported(UnsupportedLowering {
    construct: "Constructor",
    reason: "a source aggregate reached the carrier with no planner-issued
             producer occurrence, so it would name no ownership record and
             could only be given the authority of wherever it happened to be
             transferred",
})
```

Panics at `crates/ken-runtime/src/cranelift_backend/lowering/core/tests/
constructors.rs`, in
`c2_ac4_runtime_host_result_selects_a_separately_generated_nested_payload`, at
`.expect("the C2 carrier edge emits")` — the `emit(...)` call itself refuses.

## Provenance, measured at both ends

`scripts/ken-cargo test -p ken-runtime --lib --no-fail-fast`:

| ref | result |
|---|---|
| base `21fd46dc` | fails, at `:2509:14` |
| candidate `fb99d0fc` | fails, at `:2511:14`, **identical signature** |
| same, `--features px8-ds-test-support` | identical at both ends |

Base-fail and candidate-fail. The two-line offset is the candidate's added
lines above the row, not a different failure.

## The refusal is upstream of the property under test

The row exists to prove that a runtime host result **selects a separately
generated nested payload**. It never gets to select anything: the carrier edge
declines to emit at all, so the assertion the test was written for is never
evaluated. As with [[RT-WORKER-FIXTURE-DECODE]], this is a **refusal to build
the fixture**, not a wrong answer — which is why neither row could have been a
`D1` capture-order regression, and why both fail at a base that predates `D1`.

**The refusal text is a design statement, and it should be read as one before
it is read as a bug.** It says an aggregate arriving without a planner-issued
producer occurrence would name no ownership record, and so could only inherit
the authority of wherever it happened to be transferred. That is the lowering
refusing to fabricate provenance. The open question is whether the **test rig**
is failing to issue the producer occurrence the planner would issue in
production, or whether a real path can reach the carrier in that state.

## What the frame owes

- **Answer the rig-versus-real question first, because it decides everything
  else.** If the rig simply skips a planner step, the repair is in the fixture.
  If a production path can reach the carrier with no producer occurrence, the
  refusal is protecting a real hole and the row is evidence, not debt. Do not
  fold these two outcomes into one deliverable.
- **Do not repair it by relaxing the carrier's refusal.** Making the emit
  succeed by accepting an aggregate with no ownership record is the cheapest
  available false fix, and it would grant exactly the unearned authority the
  refusal text names. If that is the proposed direction, it is a mechanism
  question and returns to the Architect.
- **Prove the row measures its property once restored.** A green row after the
  emit succeeds does not establish that the nested payload was *separately
  generated* and correctly selected; that is a distinct claim needing its own
  discriminating pair.
- **Check whether the same refusal reaches other rows.** This is one observed
  occurrence; the enumeration that found it was scoped to the failing set, not
  to this signature. Name the population before sizing the repair.
