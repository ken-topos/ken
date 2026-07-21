---
id: CI-SKIPPED-NATIVE-TESTS
title: "Restore three native-parity test binaries skipped from the CI gate (C2)"
status: ready
owner: steward
size: S
gate: none
depends_on: []
blocks: []
github: null
origin: docs/program/11-test-suite-and-ci-remediation.md §3 (Track C, C2)
---

`.github/workflows/ci.yml`'s `build + test` job now runs `cargo nextest run`
with an `-E` filter that excludes three test binaries from every CI run:

- `crates/ken-cli/tests/rt_parity_native.rs` (7 tests)
- `crates/ken-cli/tests/px8f_buffer_native.rs` (1 test)
- `crates/ken-verify/tests/px8f_write_partition.rs` (1 test)

## Why they were skipped

These nine tests measured at **56.5% of a 47-minute CI wall clock**
(14m41s + 5m10s + 5m09s of 44m14s test-execution time), per
`docs/program/11-test-suite-and-ci-remediation.md` §1/§3. `cargo test` runs
its 200 test binaries strictly one at a time, so these three binaries alone
were roughly half the gate. Adopting `cargo-nextest` (item C2 of that
program) fixes the scheduling problem for the other 197 binaries, but does
not shrink these three — they are slow on their own merits (native
Cranelift-JIT-backed parity/buffer/partition exercises), not slow because of
serial scheduling.

**This is a speed trade, not a defect finding.** The three binaries
are not believed broken or flaky; they were cut for gate run time
under explicit operator direction. Skipping them means CI no longer
verifies:

- interpreter/native execution parity for the checked-buffer-IO
  narrowing cases (`rt_parity_native.rs`) — the differential oracle
  from `RT-PARITY` (`docs/program/issues/RT-PARITY.md`)
- the PX8F buffer allocation/read/write native fast-path
  (`px8f_buffer_native.rs`)
- the PX8F write-partition native fast-path (`px8f_write_partition.rs`)
  in `ken-verify`

on every PR and every push to `main`. A regression in any of these three
areas will not be caught until someone runs them by hand or locally
(`scripts/ken-cargo -p ken-cli --test rt_parity_native`, etc.) or
until C4 (rework the slow tests using nextest's per-test timing)
lands and the skip is lifted.

## How to close

Either of:

1. **Rework the three binaries for speed** (this is exactly what C4 in
   `docs/program/11-test-suite-and-ci-remediation.md` is for —
   nextest's per-test timing is now available to ground that work),
   then remove the `-E` filter from the `Test` step in
   `.github/workflows/ci.yml`.
2. Move them to a separate, non-blocking scheduled/nightly job instead of
   dropping them outright, if reworking them for speed turns out not to be
   feasible.

## Undo

The skip is a single edit: delete the
`-E 'not (binary(rt_parity_native) or binary(px8f_buffer_native) or
binary(px8f_write_partition))'` argument from the `Test` step's
`run:` line in `.github/workflows/ci.yml`. No other file encodes the
exclusion.
