---
id: CI-SKIPPED-NATIVE-TESTS
title: "Restore rt_parity_native — one test at 221s is the blocker"
status: ready
owner: verify
size: S
gate: none
depends_on: []
blocks: []
github: null
origin: docs/program/11-test-suite-and-ci-remediation.md §3 (Track C, C2)
---

**Updated 2026-07-21: one of the three is back. Two remain skipped.**

| Binary | Tests | Measured | State |
|---|---:|---:|---|
| `ken-cli/tests/rt_parity_native.rs` | 7 | 14m41s | **skipped** |
| `ken-cli/tests/px8f_buffer_native.rs` | 1 | 5m10s | ✅ restored, `native-buffer` job |
| `ken-verify/tests/px8f_write_partition.rs` | 1 | 5m09s | ✅ restored, `native-slow` job |

`px8f_write_partition` runs in its own parallel `native-slow` job, where it
costs no wall clock: the worst shard is ~471s and that job is ~374s, so it
finishes first and never sets the pace.

**`px8f_buffer_native` is now restored too** (own `native-buffer` job).
**Only `rt_parity_native` remains skipped.**

**`rt_parity_native` was measured directly** (experiment PR #808, closed —
see `docs/program/11-test-suite-and-ci-remediation.md` §1d). It parallelizes
fine under nextest: 7 tests, 266.7s wall against 470.6s of CPU. It does not
fit because of **one outlier test**:

| Test | Duration |
|---|---:|
| `fs_write_at_malformed_offset_narrows_to_invalid_offset` | **221.4s** |
| `fs_write_at_malformed_offset_without_write_right_...` | 42.2s |
| three `fs_read_at_*` | ~53s each |
| `buffer_allocate_malformed_capacity_...` | 45.3s |
| `buffer_freeze_malformed_span_...` | 1.2s |

**So this is a one-test problem, not a binary-wide one.** The 221s test is
5x its near-identical sibling and 4x the read-side equivalents, which looks
pathological rather than inherent. Bring it into sibling range and the
binary lands near 90s and fits with room to spare — 7 tests of coverage
restored for one test's worth of investigation.

**Do not simply re-enable it.** At ~470s job total against a ~471s critical
shard it fits by about a second, which is noise, not headroom.

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

## ⇒ REASSIGNED to Verify, 2026-07-22 — and BUDGET-EFF is why

**Owner moved `steward` → `verify`.** The skip stopped being a CI-hygiene chore
and became a **verification-integrity** problem, which is Verify's lane.

@adversary's BUDGET-EFF native finding: because `rt_parity_native.rs` does not
run, **a green CI on that WP carried no information** about whether the native
`remaining` defect was fixed — no test asserted it, and the binary that would
host one was skipped. The acceptance criterion had to be *"assert the numbers
and demonstrate fail-first"* precisely to route around this suite.

@verify-implementer then solved it for that WP by putting the new tests in
`-p ken-runtime --lib` instead — **closing the gap better than the fallback the
adversary proposed.** That is the ring that should own the general fix.

★ **The durable question this WP answers is not "why is a suite slow."** It is:
**which assertions currently have no running home, and what would a green CI
therefore fail to tell us?** Restoring the suite is one answer; relocating its
load-bearing assertions to a suite that runs may be a better one. **Both are in
scope — the WP is the question, not the restoration.**
