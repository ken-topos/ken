Lands `RT-DECL-CLOSURE-PORT` and `RT-CONTSRC-PRODUCER-LOCAL` — 212 reviewed
commits, three days in flight, with six nodes blocked behind them.

## What this is

The continuation-source work: a producer-local continuation source coordinate,
the typed-unit port, and the carried source-`Match` lowering. Every commit
carries a live checkpoint approval; the merge Decision was approved by Runtime
QA and by the Architect, who resolved it on cast as the sole required vote.

## Five tests ship marked `#[ignore]`, and that is the operator's ruling

Operator, 2026-08-06: *"mark the tests to skip (add comment why), land the PR,
and continue work, restoring the tests as work allows."*

At the candidate tip the `px4b_native_production` suite was **14 passed / 5
failed**. All five failures are **branch-introduced** — absent at merge-base
`e6b4a13b` and at `main` `3015aafd` — and all five are **attributed and owned**.
They are not repaired here. They are annotated and parked.

| test | exact observed signature | owner |
|---|---|---|
| `fs_write_and_read_resume_through_the_native_capability` | seat `Argument(0)` of `FsWriteFile` needs `BytesPointerLength`, cannot observe in `CarriedWord` | `RT-CARRIER-BYTESPAN-OBSERVE` |
| `fs_scope_denial_reaches_ken_as_the_named_error` | seat `Argument(0)` of `FsWriteFile` — same need, same phase | `RT-CARRIER-BYTESPAN-OBSERVE` |
| `canonical_fs_identity_exactly_matches_across_real_producers_and_drift_fails` | seat `Argument(0)` of `FsReadFile` — same need, same phase | `RT-CARRIER-BYTESPAN-OBSERVE` |
| `linked_console_broken_pipe_reaches_ken_instead_of_signal_termination` | seat `Argument(1)` of `ConsoleWrite` — same need, same phase | `RT-CARRIER-BYTESPAN-OBSERVE` |
| `public_source_observes_raw_argv_environment_cwd_bytes_in_field_order` | `ken native trap: explicit entry trap, exit Some(1)` where `Some(254)` is expected | `RT-ENTRY-TRAP-254` |

The fifth is a **runtime trap, not a lowering refusal** — a different failure
class from the four seat refusals. It is not folded into the byte-span node on
the strength of "bytes" appearing in its name; whether the two share a root
cause is unmeasured.

With the annotation the suite is **14 passed / 0 failed / 5 ignored**, and there
is no sixth failing row.

## What a reviewer should hold onto

**A skipped row measures nothing.** Greenness here is achieved by not asking the
question, not by answering it. That is the honest cost of this option and it is
why every comment names its owning node in the source, where whoever restores
the row will actually read it.

Each comment also records the **exact observed signature**. The Steward's
recommendation had been an exact-signature residual gate, so that a *different*
failure at one of these rows would still red; the operator chose the simpler
skip. Recording the signature preserves what that gate was protecting — the next
reader can tell recovery from a different failure wearing the same row.

**Restoration is owned, not hoped for.** `RT-CARRIER-BYTESPAN-OBSERVE` is
`ready` with a written frame, and un-skipping its four rows is an explicit
deliverable of it: its `D0` must un-skip them and record the live failure before
asserting anything, and a green suite still carrying the attributes has
discharged nothing.

## Provenance of the five, since it bears on trust

All three signature transitions on this branch land on commits that **announce
themselves** in their own subject lines — `c7410b79` *"exposes a RUNTIME failure
the compile evidence could not see"*, `9cea8a5e` *"(RED at this tip)"*. Nothing
was hidden, and the reason each row is red was written down by its author at the
time. That is a real property of how this branch was built. **It is not evidence
the failures are benign** — a declared defect is still a defect.
