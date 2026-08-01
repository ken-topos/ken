# WP frame — `RT-CONTSPEC-SUBSTRATE` (slice 0 of 4)

**Owner:** team Runtime · **Size:** M · **Node:**
`docs/program/issues/RT-CONTSPEC-SUBSTRATE.md`

> ## ⛔⛔ READ FIRST
>
> The `ContinuationSpecialization` mechanism is **already ruled**
> (`evt_7dhwrk26ks9m0`) and ⛔ **nothing semantic is open.** This slice exists
> because the Architect ruled (`evt_6wkw2c7ykjxsy`) that neither `main` nor the
> proved oracle `93746ada` was a lawful base for the planner slice. ⭐ **Its job
> is to put the authorities later slices consume onto `main`, dormant.**

## Fixed inputs

| input | value |
|---|---|
| **base** | ⭐ **`origin/main` as of when you cut the branch — whatever SHA that is, provided it contains this frame.** ⛔ Not `93746ada`, ⛔ not a preservation ref, ⛔ not another slice's branch. |
| semantic oracle (read-only) | `93746adaaef845f6c857b6c007aeac336c6c800c` — ⛔ **an oracle, not a base;** six Runtime paths at +2,467/−526 |
| prototype reference | `origin/preserved/rt-recursor-freeze-465fab90` (`465fab90`, patch `d04a64a7…`) — ⛔ **no acceptance transfer** |
| binding rulings | `evt_7dhwrk26ks9m0` (mechanism) · `evt_4t09329vdrf` (sizing) · `evt_6wkw2c7ykjxsy` (this slice) |

## Deliverables

**D1 — closed case-emission reachability facts.** Re-derived and asserted, not
imported.

**D2 — exact occurrence / owner / lifetime authority.** The per-occurrence
authority later slices consume when they build projections and units.

**D3 — pre-allocation closure over D1 and D2.**

⭐ **All three land DORMANT.** They may be computed and asserted; ⛔ nothing on a
live path may consume them.

## Acceptance criteria

**AC-1 — D1–D3 implemented and independently gated**, each with its own
assertion, not only as an input to something else.

**AC-2 — extraction is claim-by-claim, and each claim is RE-PROVED.**
⛔ **The single most likely way this slice fails is a wholesale port from
`465fab90` or `93746ada` with the Architect's "retainable" finding cited as
evidence.** That finding was pinned to the **named design structures**, ⛔ never
to any exact patch, and it carries **no acceptance transfer.** Each claim taken
across needs its own control here.

**AC-3 — dormancy, behaviourally.** With this slice landed, observable program
behaviour is unchanged: the existing native and interp suites give the same
results as at the base.

**AC-4 — mutation controls.** For each of D1/D2/D3, one mutation that makes a
test produce a **wrong answer** — a wrong occurrence attributed, a wrong owner
or lifetime, a closure that admits an unreachable case.
⛔ A mutation that only breaks **compilation** is not a control. Name which test
reddened and **what wrong answer** it gave.

**AC-5 — no regression, green in CI on GitHub.** ⛔ Not a local `--workspace`
run; local work is `scripts/ken-cargo` scoped to the crate touched
(`COORDINATION §12`).

## ⛔ Banned scope

- ⛔ **No cherry-pick or wholesale landing of `93746ada`.**
- ⛔ **No dynamic continuation transport activation.**
- ⛔ **No post-join / static-worker mechanism** — that arm is rejected.
- ⛔ **No planner key/projection work** — that is slice 1 and it must be able to
  fail on its own.
- ⛔ **No semantic probes, no ungated prints, no formatter sweep.** The literal
  210/211 returns, `CONTSPEC_*` markers, the bare `eprintln!` at
  `planning/static_transition.rs:8549`, and the 177-file churn must not appear.
  ⚠ A probe that changes what the program returns is not a diagnostic.

## Contention

**None.** Runtime holds this alone; the doc track runs concurrently and touches
`library/`, never `crates/`.

## Hard stops

⭐ **Route a hard stop; do not push through one.** If the substrate cannot be
re-derived without activating something banned here, ⛔ **do not widen the slice
to make its own AC reachable** — say so and it will be re-cut.

⏱ **Target: complete or hard-stop inside one turn.** A Steward sizing target,
⛔ not an acceptance criterion and ⛔ not something QA checks.
