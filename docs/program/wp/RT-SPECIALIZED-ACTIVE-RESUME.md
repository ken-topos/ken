# RT-SPECIALIZED-ACTIVE-RESUME — resume an Active frame over a live value

Owner: **runtime**. Node:
`docs/program/issues/RT-SPECIALIZED-ACTIVE-RESUME.md`. Campaign:
`docs/program/16-recursive-descent-retirement.md` node **#6g**.

> # WHAT THIS NODE IS FOR, IN ONE SENTENCE
>
> A consumer demands **constructor shape before it dispatches the eliminator**,
> so a live specialized value with an `Active` frame is refused even though
> resuming an `Active` continuation never needed a constructor. **Route the
> measured ordinary-live partition to its resume — or prove that pairing
> invalid and refuse it lawfully.** Both outcomes are legitimate and the second
> is not a lesser result.

## 1. Fixed inputs

Measured at the `RT-CARRIED-ORDINARY-COMPOSITION` `D2` candidate **`1f89a92b`**,
parent `147b239c`, over base `06e031de`.

| input | value |
|---|---|
| refusal | `Unsupported(ComputationalMatch, "scrutinee is not a constructor value after ordinary expression lowering")` |
| owner | `lower_computational_match_value_composed`, its `Lowered::Constructor` destructure |
| position | the destructure sits **before** the eliminator dispatch |
| production sites for the string | **exactly one** |
| committed pin | `core/tests/control.rs`, a **full-equality** assertion inside a suppression control |
| measured members | **two** A rows, `d8d` and `px8j_all_three_producer_paths_reach_real_consumers` |
| measured variant | `Specialized(Lowered::ProcessExitStatus)`, remaining stack exactly `[Active]` |
| `resume_active_continuation` entered first? | **no** — established from the trace, not asserted |
| evidence trace | `aa78c973`, evidence-only, **not a candidate and not merged** |

**Line numbers are deliberately absent from this table.** This node moves the
destructure, so any line it names rots against its own deliverable. Re-derive by
function name and by the `Lowered::Constructor` destructure. Two coordinate
failures on this campaign came from a line number that still resolved — to
different code.

## 2. What is owed

The Architect's ruling `evt_1pw1ng8448mef` establishes the authority and the
partition. **It does not supply the population.** Everything below turns on a
census that has not been run.

**`D0`/`D1` post as their own checkpoint before `D2` starts.** `M` is
provisional and that checkpoint is the Steward's re-size point — the predecessor
was re-sized `M` to `S` on exactly this evidence, and this node may go either
way.

## 3. Deliverables

### `D0` — close the population at the production boundary

Census **every `Specialized x first-Active` arrival** at
`lower_computational_match_value_composed`, grouped by:

- exact `LoweredVariant`;
- owner / route;
- `active.pending` length;
- frame kinds, with origins.

**With denominators and intersections**, and the instrument placed **above the
destructure** so an arrival is recorded before any guard can return. That
placement is the whole reason the predecessor's zeros meant *no members* rather
than *never reached*, and it is not optional here.

**Report the denominator as excluding committed controls.** The predecessor
learned this the expensive way: a control that arms the hook it observes is a
member of the population it measures.

### `D1` — partition before any repair

At least the five classes on the node, each with its measured member count:
ordinary-live, `Constructor` (plus the routed `BoundedNat` / `StructuralNat`
controls), `RecursiveBackedge`, `Trap`, and anything else the census finds.

**A shared refusal arm is not evidence of a shared mechanism.** If two classes
prove materially distinct authorities, that is a hard stop, not a two-case
`match`.

### `D2` — route only the measured ordinary-live partition

**Do not move `Active` dispatch wholesale above the shape and terminal guards**
(Architect, explicit). Route the ordinary-live partition ahead of the
constructor-only destructure and preserve the protocol and terminal laws:
`RecursiveBackedge` propagates, `Trap` seals or propagates, neither resumes.

**Repair only cells the census proves have members.** An empty cell keeps its
fail-closed refusal, recorded as a measured-at-base zero — the disposition
`PendingLet` received two nodes ago, for the same reason: a mechanism over an
empty population is a vacuous proof (Campaign Trap 3).

### `D3` — a discriminating advance control

Non-vacuous `ProcessExitStatus x Active` advance evidence, plus a mutation that
**restores the fifth refusal after proving the new detector was reached**.

## 4. Acceptance criteria

- **AC-1** — the population is closed **by measurement** at the production
  boundary, with denominators and intersections, not by grep and not from the two
  known rows.
- **AC-2** — the five classes are partitioned on evidence, each with a member
  count. A class with zero members is reported as zero, not omitted.
- **AC-3** — every repaired cell has a measured member. No mechanism lands on an
  empty cell.
- **AC-4** — `RecursiveBackedge` still propagates and `Trap` still seals or
  propagates. Neither resumes.
- **AC-5** — **the committed full-equality suppression control still
  discriminates**: its suppressed `RecursiveBackedge` path still reaches the exact
  constructor refusal. If the repair changes the message, the control is updated
  in the same candidate and the update is justified against this AC, not merely
  made to pass.
- **AC-6** — `D3`'s control is proven able to fail **on its own candidate**, and
  does not key on a refusal string this node deletes.
- **AC-7** — the four landed repairs are intact:
  `resume_active_continuation`, `carried_join_arm`, the `Carried x Active` resume
  route, and the accepted `Carried x Ordinary` suffix continuation with its
  lexicographic measure and fail-closed depth bound.
- **AC-8** — zero added `#[ignore]`; no lane or variant retired; no tracker
  `status:` changed in a code candidate.
- **AC-9** — **green in CI**, never a local `--workspace` run
  (`COORDINATION §12`).

## 5. Banned scope

- Reopening any of the four landed repairs, or the accepted
  [[RT-CARRIED-ORDINARY-COMPOSITION]] `D2` mechanism.
- Hoisting `Active` dispatch above the shape and terminal guards.
- Rows 1-5 and the `LexicalCallArgumentRecursor` population.
- Retiring a lane or a residual variant; generalizing the per-variant exclusion
  hook; continuing `10369776`.
- Any planner or ABI population.
- Widening an interface to make the repair fit — that is a hard stop, below.

## 6. Hard stops

Stop and post **before coding** if:

1. the five classes prove to need **materially distinct mechanisms** rather than
   one routing decision;
2. the repair requires a **planner or ABI** population;
3. routing the ordinary-live partition requires **widening an interface** — a
   component-design call, the Architect's, never absorbed;
4. the census finds a variant the partition has no cell for;
5. **a sixth wall.**

> ### THIS IS THE FIFTH WALL ON ONE CHAIN. EXPECT A SIXTH, AND ROUTE IT.
>
> Five repairs, five correct, each revealing the next. **That is Campaign Trap 2
> — the fail-closed machinery working on a newly reachable population, not a
> defect in the node that finds it.**
>
> **Three walls have now been returned cleanly rather than absorbed**, which is
> what keeps the population visible in the tracker instead of buried in a sizing
> overrun. Absorbing one would hide it. **Route it.**

## 7. Sizing

**`M`, provisional.** The census is wider than the predecessor's — a production
boundary with five partitions rather than three ordered guards — but the repair
may prove to be a single routing decision over one class. `D0`/`D1` is the
re-size point and the Steward re-sizes on it, as with `M` to `S` on #6f.

Target the one-hour turn: a releasable increment or a genuine hard stop. Both
are good outcomes; the bad one is neither.

## 8. Base

**Cut from `origin/main` once
[[RT-CARRIED-ORDINARY-COMPOSITION]]'s `D2` has merged**, and take the merged
commit, not `1f89a92b`.

> ### CUT FROM THE OBJECT THAT WILL MERGE, NOT THE ONE THIS FRAME NAMES
>
> `1f89a92b` is the reviewed candidate; the repository **squash-merges**, so it
> **will not be an ancestor of `main`**. That has now been true for every accepted
> partial on this chain, twice predicted and twice confirmed.
>
> **Ground-truth the merge by content before building on it** — the route
> present, the guards intact, the file byte-identical — rather than on the
> publisher's word.
>
> The WP branch name has been deleted on origin by a squash merge several times.
> If you reuse a name, expect to prune; a targeted fetch reporting
> `couldn't find remote ref` means your tracking ref is **stale**, and it does
> not prune for you.

## 9. Contention

`crates/ken-runtime/src/cranelift_backend/lowering/core.rs` and its
`core/tests/control.rs`. **The whole carried chain contends here**, so this node
runs alone in that file — which the campaign's single-threaded posture already
guarantees.

The evidence trace `aa78c973` sits on `runtime-implementer/coc-evidence-wip`.
**It is evidence, not a candidate, and it does not merge.**
