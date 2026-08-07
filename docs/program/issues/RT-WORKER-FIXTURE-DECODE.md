---
id: RT-WORKER-FIXTURE-DECODE
title: "the worker fixture cannot run (Backend NativeResultDecode token 9), so AC-5's target-redirect detector dies before reaching any of its three capture-order comparisons"
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
Backend(NativeResultDecode { token: 9 })
```

Panics at `crates/ken-runtime/src/cranelift_backend/lowering/core/tests/
constructors.rs`, in `run_worker_fixture`, at
`.expect("the worker fixture runs")` — the `.run(None)` step, **not** the
compile step above it, which succeeds.

## Provenance, measured at both ends

`scripts/ken-cargo test -p ken-runtime --lib --no-fail-fast`:

| ref | result |
|---|---|
| base `21fd46dc` | 778 passed / 2 failed / 1 ignored — **this row fails** |
| candidate `fb99d0fc` | 783 passed / 2 failed / 1 ignored — **this row fails** |
| same, `--features px8-ds-test-support` | identical at both ends, same signature |

Base-fail and candidate-fail with an identical signature, in every
configuration measured. The reported line moves `5759 -> 5761` purely because
the candidate adds two lines above it.

## THIS IS NOT AN ASSERTION FAILURE, AND THAT IS THE WHOLE POINT

`two_same_shape_workers_are_distinguished` is documented as `D5` and as
**`AC-5`'s target-redirect red**: two same-shape workers must be genuinely
distinguished, so that swapping a body or a capture order moves the linked
result. It carries three `assert_ne!` comparisons to prove it.

**It reaches none of them.** Its very first statement is
`let baseline = run_worker_fixture(...)`, and that call panics. The three
comparisons are dead code at both refs.

Two consequences, and the second is what the frame must act on:

1. **The `D1` regression hypothesis is falsified, for the right reason.** A
   capture-order regression from `D1`'s `reverse(Parameter run) ++ Capture run`
   would present as **two configurations comparing equal** — an `assert_ne!`
   firing. It cannot present as a fixture that will not execute, and it
   certainly cannot do so **at a base that predates the change**.
2. **The detector has not been measuring its property.** `AC-5`'s
   target-redirect red is currently asserted by a test that cannot reach its
   assertions. Annotating this row therefore switches off nothing that was
   working — but it also means **un-ignoring the row is not the deliverable.**

## What the frame owes

- **Restore the fixture, then re-arm the detector.** Un-ignoring a row whose
  fixture panics just restores a red. The deliverable is that
  `run_worker_fixture` runs and the three `assert_ne!` comparisons execute.
- **Prove the detector is live once restored, not merely green.** It went
  green-by-not-asking for an unknown span; a passing row is exactly what a
  still-dead fixture would produce if someone weakened the `expect`. Show a
  deliberate mutation (swap a body, swap a capture order) reddening it, then
  restore. Without that, this node closes by re-creating the condition it was
  filed for.
- **Decide what `NativeResultDecode { token: 9 }` means.** The compile step
  succeeds and the run step fails to decode a native result. Name whether the
  defect is in the fixture's expectations or in the decode path itself; a
  fixture-only repair that leaves a real decode bug is the cheap false fix
  here.
- **Say whether `AC-5` was ever discharged.** If the target-redirect red was
  claimed on this row, that claim rested on a test that could not execute.
  Establish what, if anything, covers the property today — and route the answer
  rather than absorbing it.
