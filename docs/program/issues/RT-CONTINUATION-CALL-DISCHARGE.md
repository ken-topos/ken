---
id: RT-CONTINUATION-CALL-DISCHARGE
title: "A planned continuation call is neither directly emitted nor compositionally consumed once the Active resume path goes live — attribution, not repair"
status: ready
owner: runtime
size: M
gate: none
depends_on: [RT-SPECIALIZED-ACTIVE-RESUME]
blocks: [RT-MATCH-RECURSOR-CONSUMERS]
github: null
origin: Architect ruling evt_vxqa83y4z3nt (2026-08-08) on the RT-SPECIALIZED-ACTIVE-RESUME D2/D3 sixth wall at exact d9175d05, with the Steward cut and release ruling evt_27jwdbz9h2t4c. Campaign docs/program/16-recursive-descent-retirement.md node #6h. Steward-filed (agents cannot create tracked work per COORDINATION §2).
---

> # THIS IS THE SIXTH WALL, AND IT IS THE FIRST PLANNER-POPULATION AUTHORITY.

The first four were `BoundaryCarrier` refusals about how a carried operand may
**cross or be consumed**. The fifth ([[RT-SPECIALIZED-ACTIVE-RESUME]]) was about
the **value shape** a scrutinee has after ordinary lowering. **This one is
neither.** It is about whether the **planned population was discharged** — a
question no earlier wall on this chain has asked.

**This node is an ATTRIBUTION node. It does not begin with a repair.** Which
side is wrong is exactly what is unknown, and the deliverable is a classification
backed by a trace.

## What it is

Owner: **`ContinuationClaimLedger::close`**, `lowering/units.rs:3311`, at the
set-equality check on `units.rs:3362`.

```
the discharged continuation call population is not the planned one: 1 planned
tokens were neither directly emitted nor compositionally consumed, and 0
discharged tokens were never planned. Direct: 0, composed: 0
```

The law is **exact set equality, not counts**:

```
planned = direct-emitted  ⊎  composed-consumed
```

The two forms **partition** the planned population — `close` refuses an identity
appearing in both, on the ground that one obligation was answered twice.

## Why it only appears now

Under the fifth wall this program **aborted before `close()` ever ran**.
[[RT-SPECIALIZED-ACTIVE-RESUME]]'s `D2` cleared that stop for the routed cell,
so the ledger became reachable for this shape **for the first time**.

⇒ **Campaign Trap 2 — a fail-closed invariant meeting a newly reachable
population. It is not a defect in that `D2`**, and this node does not reopen it
or any of the five landed repairs.

## The owned fact

For every measured member `active.pending` is **empty**, and
`resume_active_continuation` **returns its operand unchanged**. So the activated
path reaches an empty `Active` resume **with no call**, while the planner has
already minted the causal call.

**That proves a planner/lowering obligation mismatch. It does not say which side
is wrong.** Both readings are live, and `pending_len == 0` is evidence for
neither.

> ### DO NOT DISCHARGE THE TOKEN IN THE EMPTY RESUME
>
> **Architect, explicit** (`evt_vxqa83y4z3nt`). Recording a discharge there
> would be **false evidence**:
>
> - it is **not direct-emitted** — no specialization call instruction was
>   emitted and decoded; and
> - it is **not composed-consumed** — that set is fed only after a recorded raw
>   worker call is found in **finished CLIF**, checked against the planned
>   worker, operands and result, and shown to return downstream.
>
> The ledger's own comment at `units.rs:3059` states that `composed` is fed
> **from `function_local.composed_discharges` and from nothing else**, because
> a composed instruction targets the raw worker while the direct gate requires
> the recorded instruction to decode to `identity.target()`. An identity in both
> sets would mean one of the two gates had been loosened.
>
> **Do not weaken the law, bulk-claim the token, manufacture a composed
> discharge, or treat an identity return as a call.**

## `D1` classifies exactly one of three. It does not pick a favourite

Trace the exact missing `ContinuationCallIdentity` — construct origin,
continuation origin, alternative, recursive position, call-site sequence,
target, and emission owner — **through the same program in both lanes**, then
classify:

1. **A real direct obligation was skipped.** Repair the actual producer/call
   seat and **retain finished-CLIF verification**.
2. **A real composed consumption occurred but its evidence was lost.** Restore
   the verified composed relation. **Do not claim it from the resume.**
3. **The activated path has no causal call obligation.** Correct the planner's
   issuance/projection **at planner authority**, proving why this exact path is
   not a member. **Do not infer that from `pending_len == 0` alone.**

**The planner is implicated but not convicted.** It mints the causal call while
discovering a recursive closure position under a computational continuation,
once the source environment is available — and **that mint is independent of the
test-only lane exclusion.**

**One fact bears directly on option 3 and is already recorded in the code.**
`ContinuationClaimLedger::open` carries an honest note that `planned == resolved`
is **structural today**, because `resolve_continuation_targets` walks the same
projection; the two would separate only if resolution ever dropped or added a
key. So a projection-level correction is not a free relabelling — **it moves the
set that `close` is checking against.**

## The retained lane is the control, and the population floor is two

**The retained lane closes the same program.** Record whether the same identity
is discharged **directly or compositionally** there. That is the discriminator,
and it is a non-degenerate pair on a shared input rather than a counter.

The **two independent A rows are the population floor, not the perimeter.** This
campaign has read a small-witness result as a class-wide property four times and
every correction cost more than the census would have.

> ### THE HELD LANE-PAIR OBJECT IS THIS NODE'S END-STATE ACCEPTANCE CONTROL
>
> `65639a13` on `runtime-implementer/sar-lane-pair-evidence` is **evidence, not
> a candidate, and it is not published.** It was correctly held rather than
> committed: divergence is the campaign premise failing, so committing it red is
> impermissible and weakening it to pass would absorb the very stop it detected.
>
> **This node inherits it as the exact assertion it must satisfy.** The end
> state is that the activated lane closes the same program the retained lane
> closes.

## Scope

Gates completion of [[RT-MATCH-RECURSOR-CONSUMERS]] and its `AC-1`. Does **not**
reopen [[RT-SPECIALIZED-ACTIVE-RESUME]]'s accepted `D2`/`D3`, or any of the five
landed repairs. Does not touch rows 1-5 or the `LexicalCallArgumentRecursor`
population ([[RT-LEXICAL-RECURSOR-CONSUMERS]]).

Frame: `docs/program/wp/RT-CONTINUATION-CALL-DISCHARGE.md`.

## Sizing is provisional, and `D0`/`D1` may overturn it

**`M`, provisional.** The three-way classification is a measurement, and which
branch it lands on changes the work by more than a size step: option 3 is a
planner-authority correction, option 1 touches a producer/call seat, option 2 is
an evidence-plumbing repair. **Re-size on the `D0`/`D1` handback**, as #6f and
#6g both were.
