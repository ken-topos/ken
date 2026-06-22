# ADR 0002 — Teams wait idle for CI; no intra-team multitasking

- **Status:** Accepted
- **Date:** 2026-06-22
- **Deciders:** Pat Lasswell (operator)

## Context

After a team opens a PR, it must wait for CI (build + conformance + clean-room +
path-guard) before the work proceeds to review and merge. The question: should a
team stay idle during that wait, or keep busy — e.g. **stacked PRs** on a
dependent sequence, or starting a second independent work package to fill the gap?

## Decision

**Teams wait idle for their CI runs.** No stacked PRs; no starting a second task
to fill the wait. A team carries **one** task through its ring at a time
(reinforces COORDINATION §0).

## Rationale

1. **Agents multi-task poorly.** Splitting a team's attention across two in-flight
   branches costs the coherence the token-ring is designed to buy.
2. **Throughput already comes from cross-team parallelism.** The federation is a
   *ring of rings*: while team A waits on CI, teams B–F are working. Per-team idle
   is masked at the federation level, so intra-team pipelining adds little.
3. **Waiting is cheap.** Idle agents make no API calls → no token/$ cost. The cost
   is wall-clock, which (unlike with humans) we spend freely, and which cross-team
   parallelism largely hides.
4. **Keep-busy fights the compute budget.** A team that kept building during the
   wait would compete for the global build lock and the 16 GB box (COORDINATION
   §12 / `../ops/compute-budget.md`). A *waiting* team is a *non-competing* team —
   idle is load-friendly.
5. **Stacked PRs add churn.** If the base PR gets change requests, the stacked
   work must rework/rebase — exactly the bookkeeping agents handle badly.

## Consequences

- **Idle means quiescent, not resident.** A team waiting on CI pauses to free RAM
  (COORDINATION §12), especially on constrained hardware — it does not sit
  resident spinning.
- **The watchdog must not treat CI-wait as a stall.** "PR with a CI run in
  progress, team idle" is a *normal* state; only "CI finished with no follow-up
  action" is a stall (COORDINATION §13). The liveness watchdogs distinguish them.
- More wall-clock per work item — accepted.

## Revisit if

CI becomes slow enough (e.g. long conformance runs) **and** a single critical-path
team's idle starts to dominate the roadmap **and** no cross-team work absorbs the
slack. Even then, prefer **faster CI** (the compute levers, caching, splitting the
suite) or **re-phasing the work** over intra-team multitasking. These conditions
are anti-correlated (slow CI arrives once many teams are active, which restores
cross-team parallelism), so expect this to stay rare.
