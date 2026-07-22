---
id: CI-SKIPPED-NATIVE-TESTS
title: "Restore rt_parity_native — dedicated CI job, outlier not fixed"
status: active
owner: verify
size: S
gate: none
depends_on: []
blocks: []
github: null
origin: docs/program/11-test-suite-and-ci-remediation.md §3 (Track C, C2)
---

**Updated 2026-07-22: all three restored — see "Closed" below.**

| Binary | Tests | Measured | State |
|---|---:|---:|---|
| `ken-cli/tests/rt_parity_native.rs` | 7 | ~266.7s wall (250.5s outlier test) | ✅ restored, `native-rt-parity` job |
| `ken-cli/tests/px8f_buffer_native.rs` | 1 | 5m10s | ✅ restored, `native-buffer` job |
| `ken-verify/tests/px8f_write_partition.rs` | 1 | 5m09s | ✅ restored, `native-write-partition` job |

Each runs in its own parallel job, where it costs no wall clock: the worst
shard is ~471s and each of these jobs is ~250-374s, so they finish first and
never set the pace — that headroom is what let all three land without
touching the sharded lane's critical path.

**`rt_parity_native` was previously believed to be a one-test-fix-away
problem** (bring the 221s outlier down to sibling range, ~90s total, and it
fits inside the shard). Re-measured fresh against current `main` for this
WP: the outlier is still present (**250.5s**, if anything slightly worse
after intervening native-lowering commits) and this WP does **not** treat
it as a bug to fix. `fs_write_at_malformed_offset_narrows_to_invalid_offset`
is the only one of the seven that opens **two nested resource brackets**
where every sibling test opens exactly one — a topology fact that is
load-bearing to the property that test isolates (dispatch-skip under a
rights-clean write, distinct from its sibling's rights-fault-overlap case).
That topology is correlated with the timing outlier, but nothing here
isolates it as the *cause* — see "Closed" below for the unisolated-
hypothesis framing. **This WP takes Option 2 below instead: a dedicated
job**, mirroring `native-buffer`/`native-write-partition` — the binary's
own ~266.7s total fits the same headroom pattern as its two siblings
without needing the outlier fixed, or its cause established, at all.

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

**This was a speed trade, not a defect finding.** The three binaries were
not believed broken or flaky; they were cut for gate run time under
explicit operator direction. While skipped, CI no longer verified:

- interpreter/native execution parity for the checked-buffer-IO
  narrowing cases (`rt_parity_native.rs`) — the differential oracle
  from `RT-PARITY` (`docs/program/issues/RT-PARITY.md`)
- the PX8F buffer allocation/read/write native fast-path
  (`px8f_buffer_native.rs`)
- the PX8F write-partition native fast-path (`px8f_write_partition.rs`)
  in `ken-verify`

on any PR or push to `main`. A regression in any of these three areas would
not have been caught except by running them by hand or locally
(`scripts/ken-cargo -p ken-cli --test rt_parity_native`, etc.). **All three
are now restored to CI** — see "Closed" below; this section describes the
gap that existed before this WP (and its two predecessor WPs), not the
current state.

## How to close

Either of:

1. Rework the binary for speed, then remove the `-E` filter from the sharded
   `Test` step in `.github/workflows/ci.yml` and let it run inside a shard.
2. **[TAKEN, this WP]** Give it a dedicated job, same pattern as its two
   siblings — the binary's own wall clock fits the same headroom as
   `native-buffer`/`native-write-partition` without needing the outlier
   fixed. It stays excluded from the sharded lane (so it is not duplicated
   there) and runs in its own `native-rt-parity` job instead.

## Undo

Each of the three binaries is named in exactly four places, all in
`.github/workflows/ci.yml`: the `-E` exclusion in the sharded lane's `Test`
step, its own dedicated job (`native-rt-parity` / `native-buffer` /
`native-write-partition`), `build-test`'s `needs:` list, and the pass/fail
check loop. All four must stay complementary — but they do **not** fail the
same way when one drifts from the others. Removing each alone (leaving the
other three) reasoned through GitHub Actions' `needs:`/context semantics
below, not exhaustively executed against a live run:

- **Drop only the exclusion** (job/`needs:`/check loop untouched): the
  binary now runs in *both* the sharded lane and its own job — silently
  duplicated, no failure signal, just wasted CI compute.
- **Drop only the dedicated job** (`needs:`/check loop still name it):
  `needs:` referencing an undefined job is a workflow *syntax* error —
  GitHub Actions rejects the whole workflow file before any job runs. Loud
  and immediate, not silent.
- **Drop only the `needs:` entry** (job and check loop untouched): the
  check loop's `${{ needs.native-rt-parity.result }}` reference resolves to
  empty for a job outside the current job's own `needs:` list, which fails
  the loop's `!= "success"` comparison, so `build-test` reports red. This
  case also loses the ordering guarantee — `build-test` no longer waits for
  the job — so a red result here may not reflect the job's real outcome.
  Either way: red, not a silent green.
- **Drop only the check-loop entry** (job and `needs:` untouched): this is
  the dangerous one. `build-test` still waits for the job via `needs:`, but
  the loop never inspects its result, so a genuine failure in that job can
  leave `build-test` green regardless. This is the only one of the four
  that reproduces "a dropped test still shows a green gate."

So a simpler universal claim ("any partial removal duplicates or silently
drops the test, and either way `build-test` still reports green") would
overstate it: only the last case above is actually silent-green; the other
three fail loud or merely waste compute. To fully undo the restoration for
one binary, remove it from all four places together.

## Closed 2026-07-22

All three binaries now run in CI on every PR/push, each in its own
dedicated job (`native-rt-parity`, `native-buffer`, `native-write-partition`),
none on the sharded lane's critical path. `rt_parity_native`'s outlier test
is grounded but not fixed — see the finding above. **What's measured:** the
outlier test is the only one of the seven with a unique two-nested-resource-
bracket topology among the comparable FS cases, and it is a ~7x timing
outlier against its structurally nearest sibling. **What's not established:**
neither fact isolates bracket-nesting depth as the *cause* of the timing, or
shows the scaling is superlinear in it — that remains an unisolated
hypothesis, not a finding. Chasing the cause is out of Verify's lane in
either case, and not necessary to close this WP, since the dedicated-job
approach doesn't need the outlier gone. Recorded for the runtime team /
Architect as a follow-on hypothesis, not filed as its own tracker issue (not
currently blocking anything).

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
