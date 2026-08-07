---
id: RT-WORKER-FIXTURE-DECODE
title: "AC-5's target-redirect detector is dark — its expression dies at the run step with Backend NativeResultDecode token 9, before any of its three comparisons, while the fixture helper's other caller passes"
status: ready
owner: runtime
size: M
gate: none
depends_on: [RT-SRCBODY-BIND-ORDER]
blocks: []
github: null
origin: Measured by the RT-SRCBODY-BIND-ORDER all-eight-package two-ended census (evt_ksrhrv82t5ae), after CI failed this row at candidate fb99d0fc. Fails identically at frozen base 21fd46dc, so it is pre-existing base debt and not a regression from D1. Fits no released owner; the ring stopped and reported rather than assigning a nearest fit. Steward-filed (agents cannot create tracked work per COORDINATION §2).
---

> ## FRAMED — `ready`, size M
>
> Frame: [`wp/RT-WORKER-FIXTURE-DECODE.md`](../wp/RT-WORKER-FIXTURE-DECODE.md)
>
> **The frame governs; this file is the origin record.** Where the two differ,
> the frame is later and was ground against `origin/main` `89916fc1`.
>
> **Two claims below are corrected by the frame §1c/§1d and must not be built
> on as written here:**
>
> 1. **"The worker fixture cannot run" is false as a general statement.**
>    `run_worker_fixture` (`constructors.rs:5772`) has exactly two callers, and
>    the other one — `nested_worker_depends_on_both_levels` (`:5895`) — is
>    un-ignored and passing. The discriminator is the expression, not the
>    helper, and that live sibling is a working differential already in the
>    tree.
> 2. **`token` is the native return value, not an error code** —
>    `compiled.rs:132`. Eight sites across five decoder kinds raise this one
>    variant, so `token: 9` names no arm. Naming it is `D1`.
>
> `ready` does not mean released. The fleet is single-threaded and this node is
> sequenced behind `RT-CARRIER-BYTESPAN-OBSERVE`, which owns the same crate.

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

## The dark detector is the witness for an unmeasured capture-order axis

**Adversary `evt_2yxmdfhvt4fm0` (F1), verified by the Steward against
`b0a0a20c` 2026-08-07. This raises the node's stakes: the fixture is not just
dark, it is dark over a question `RT-SRCBODY-BIND-ORDER` left open.**

`RT-SRCBODY-BIND-ORDER` reversed the Parameter run and left the Capture run in
descriptor order
(`crates/ken-runtime/src/cranelift_backend/lowering/units.rs:4060-4067`):

```rust
let converts = source_body_binding_order(unit.definition);
if converts {
    parameters.reverse();
}
let mut env = parameters;
env.extend(captures);          // not reversed
```

`source_body_binding_order` (`:3689-3699`) returns `true` for
`CallableDeclaration` **and `ClosureBody`** — and `ClosureBody` is the unit kind
that carries a non-empty capture run. **So on exactly the units that have
captures, the Parameter run is reversed and the Capture run is not.** The same
shape appears again at `:2600-2605`. All four facts confirmed in source.

**The covering comment at `:4057-4059` is the part to read:**

> `validate_slot_run` proves the Parameter run is a contiguous prefix of the
> Capture run, so concatenating the two IS the descriptor order

**Descriptor order is precisely what that commit established is not the
semantic order for the sibling run.** The sentence justifies keeping the
capture run in the order the same commit deemed wrong next door.

**What is NOT established: that captures should be reversed.** That turns on
how the elaborator assigns de Bruijn indices across a closure environment,
which nobody has read. **Both answers are live**, and this node must not assume
either.

**Why this belongs here rather than in a node of its own.**
`two_same_shape_workers_are_distinguished` — the row this node exists to
restore — **is the direct discriminator**: two same-shape workers differing in
captured content, with three `assert_ne!` comparisons as the comparison. The
axis is unmeasured *because* this fixture is dark. Restoring it is already this
node's deliverable; answering the capture-order question is what the restored
detector is *for*.

⇒ **Two additions to what the frame owes:**

- **Once the detector is live, use it to decide the capture-order axis** —
  reversed or correct-by-construction — and state which, with the elaborator's
  de Bruijn assignment as the evidence, not the test's colour alone. A green
  row does not by itself distinguish "captures are already right" from
  "this fixture does not vary captures".
- **Whichever way it resolves, the comment at `:4057` must say so.** If
  captures are correct by construction, that fact belongs in the comment,
  because the current sentence does not say it — it appeals to descriptor
  order, which is the refuted ground.

**Not asserted as a bug, and do not repair it as one.** There is no repro. If
the measurement says the capture run is wrong, that is a lowering-semantics
change and it goes to the Architect, not into this node's fixture repair.
