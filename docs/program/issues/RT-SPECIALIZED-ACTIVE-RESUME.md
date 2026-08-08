---
id: RT-SPECIALIZED-ACTIVE-RESUME
title: "A live specialized value with an Active frame is refused by a constructor-only destructure — Active resume does not require constructor shape"
status: active
owner: runtime
size: S
gate: none
depends_on: [RT-CARRIED-ORDINARY-COMPOSITION]
blocks: [RT-MATCH-RECURSOR-CONSUMERS]
github: null
origin: Architect fifth-wall sibling-authority ruling evt_1pw1ng8448mef (2026-08-08) on the RT-CARRIED-ORDINARY-COMPOSITION D2 hard stop evt_5vs6jav0b9zws, discharged against the evidence-only trace aa78c973. Handle suggested by the Architect in the same ruling. Campaign docs/program/16-recursive-descent-retirement.md node #6g. Steward-filed (agents cannot create tracked work per COORDINATION §2).
---

> # THIS IS THE FIFTH WALL, AND THE FIRST ONE THAT IS NOT A CARRIER PROBLEM.

The previous four were `BoundaryCarrier` refusals about how a carried operand
may **cross or be consumed**. **This one is not carrier transport at all — the
carrier has already been eliminated.**

Four repairs landed on this chain and **none is reopened by this node**:

- [[RT-RECURSOR-TRANSPORT]]'s `D2` at `resume_active_continuation`.
- [[RT-MATCH-RECURSOR-CONSUMERS]]'s `D2` at `carried_join_arm`.
- [[RT-CARRIED-CONTINUATION-RESUME]]'s `D2` — the `Carried x Active` resume route.
- [[RT-CARRIED-ORDINARY-COMPOSITION]]'s `D2` — the suffix continuation, **which
  worked**: the trailing-suffix refusal is gone from both A rows, and the rows
  reaching a fifth authority is itself the proof the continuation ran.

## What it is

```
Unsupported(ComputationalMatch, "scrutinee is not a constructor value after
ordinary expression lowering")
```

Owner: **`lower_computational_match_value_composed`**, at its
`Lowered::Constructor` destructure. *(At the `D2` candidate `1f89a92b` that
destructure is `core.rs:3923` and the refusal `core.rs:3932`. **The function name
and the destructure are the handles** — this node moves that code, so line
numbers rot against its own deliverable. Re-derive by name.)*

## The owned fact, and why it is a distinct authority

**The destructure sits before the eliminator dispatch.** So the value's shape is
demanded before anyone asks what the eliminator needs, and **an `Active` frame
never reaches its resume when the value is an ordinary non-constructor.**

**Constructor shape is necessary for `Computational` and `Ordinary` elimination.
It is not a prerequisite for resuming an `Active` continuation** — the existing
producer and bounded/structural-Nat paths already resume `Active` over a
`LoweringOperand` without that premise (Architect, `evt_1pw1ng8448mef`).

Measured on the evidence-only trace `aa78c973`, identical index-for-index across
both A rows: the carried ordinary elimination **completes** and returns
`Specialized(Lowered::ProcessExitStatus)`; the remaining stack is exactly
`[Active]` — the bare tail of the pre-continuation frame list; and
`resume_active_continuation` **has not entered** before the refusal.

⇒ An ordinary live value meets a valid eliminator at a consumer that demands a
constructor first. **That is not a defect in the `D2` continuation.**

## Population

**Every `Specialized x first-Active` arrival at
`lower_computational_match_value_composed`**, grouped by exact `LoweredVariant`,
owner/route, `active.pending` length and frame kinds, with denominators and
intersections.

**The two measured `ProcessExitStatus` rows are the floor, not the perimeter.**
This campaign has now read a small-witness result as a class-wide property four
times, and every correction cost more than the census would have.

## The partition that must not be assumed

`D1` partitions **at least** these, and a shared refusal is not evidence they
share a mechanism:

1. **ordinary live values that may be resumed** — the repair population;
2. **`Constructor`**, plus the already-routed `BoundedNat` / `StructuralNat`
   controls;
3. **`RecursiveBackedge`** — protocol machinery, **must propagate, not resume**;
4. **`Trap`** — terminal, **must seal or propagate, not resume**;
5. any other represented or forbidden variant the census actually finds.

> ### DO NOT MOVE ACTIVE DISPATCH WHOLESALE ABOVE THE SHAPE AND TERMINAL GUARDS
>
> **Architect, explicit.** Route **only the measured ordinary-live partition**
> ahead of the constructor-only destructure, preserving the protocol and terminal
> laws. Hoisting the dispatch is the obvious repair and it is the wrong one:
> it would resume `RecursiveBackedge` and `Trap`, which must not resume.

> ### A COMMITTED FULL-EQUALITY CONTROL PINS THIS EXACT REFUSAL
>
> `core/tests/control.rs:11425` asserts the rendered `Unsupported` value in
> **full equality** — status, `construct`, and reason together — as the replay of
> a **deliberately suppressed** `RecursiveBackedge` propagation. There is exactly
> one production site for that string.
>
> ⇒ **A repair that changes or removes the message reds that assertion by
> design.** Meet it as a known consequence, not as an unexplained mid-turn red.
>
> **And it is more than an obstacle — it is a free discriminator.** The control's
> suppressed `RecursiveBackedge` path **must still reach the exact constructor
> refusal** after the repair. If it stops doing so, the partition leaked protocol
> machinery into the resume path, which is precisely the failure the
> do-not-hoist rule above is guarding against.

## Scope

Gates completion of [[RT-MATCH-RECURSOR-CONSUMERS]] and its `AC-1`. Does **not**
reopen the accepted [[RT-CARRIED-ORDINARY-COMPOSITION]] `D2` mechanism, or any of
the four landed repairs above. Does not touch rows 1-5 or the
`LexicalCallArgumentRecursor` population ([[RT-LEXICAL-RECURSOR-CONSUMERS]]).

Frame: `docs/program/wp/RT-SPECIALIZED-ACTIVE-RESUME.md`.

## `D0`/`D1` MEASURED — re-sized `M` to `S`

Steward, 2026-08-08, `evt_64f8gm80y2w0y`.

Checkpoint `f3be6476`, base `dcd6d84c`, one path `+257/-0`, `crates/`
byte-identical. **`AC-1` and `AC-2` discharged over the lib corpus.**

`Specialized x first-Active` closes at **4 arrivals, 2 independent** (`d8d`,
`px8j`; `ccr_d3` and `coc_d3` are this chain's own committed controls, excluded).
Denominator 497 arrivals / 507 retained, one disposition each, zero orphans.
All four uniform: `ProcessExitStatus`, `pending_len=0`, `route=DirectScrutinee`,
`owner=Predeclared(0)`, disposition `ConstructorRefusal`. Every other class is
**zero**, and two of those zeros are stronger than "not observed":
`BoundedNat`/`StructuralNat` are **never reached from this function at all**, and
the variant axis is a **wildcard-free committed table**, so a new `Lowered`
variant cannot be silently absorbed.

> ### THE RESUME IS THE IDENTITY FOR THE WHOLE MEASURED POPULATION
>
> `resume_active_continuation` opens with
> `let Some((head, tail)) = active.pending.split_first() else { return Ok(value) };`,
> and **every measured member has `pending_len=0`**.
>
> ⇒ **Routing to the resume and simply skipping the guard are observationally
> identical on every member.** A `D3` control keyed on behaviour — refusal gone,
> value flows — **cannot discriminate the two implementations**; a guard-skip
> passes it identically, and the difference surfaces only when a
> non-empty-`pending` member appears and is silently mishandled.
>
> **`D3` must assert the ROUTE was taken**, not merely that the refusal
> disappeared. The mechanism is chosen for behaviour no measured member
> exercises, so that is a measured-at-base limitation, not a proven general
> resume.

> ### THE RETAINED LANE IS A FREE POSITIVE CONTROL, AND IT TESTS THE THESIS
>
> The **same two programs** arrive as `Constructor, pending_len=1, ACCEPTED` when
> retained and `ProcessExitStatus, pending_len=0, REFUSED` when activated, with
> the `Active` frame constant across both. **The retained lane already reaches
> the end state the activated lane should reach**, on the same program, and both
> lanes already run.
>
> That is a **non-degenerate pair on a shared input** (`COORDINATION §7b`) and is
> stronger than a counter: a counter says the route fired, the pair says the two
> lanes agree. **It is also a direct test of the campaign's thesis** that the
> residual lane is removable without changing behaviour — if the lanes diverge
> after `D2`, the premise failed and that is a stop-and-route.

**`AC-5` is discharged BY DISJOINTNESS AND THE DISCHARGE IS CONDITIONAL.** The
committed suppression control arrives as `Specialized(RecursiveBackedge)` with
first frame `Ordinary` — outside the repair cell on **both** axes. That holds
**for the narrow key only**. `D2` routes `ProcessExitStatus x first-Active` as
measured; any widening re-measures `AC-5` in the same candidate and is never
carried forward.

## The cross-crate census: TAKEN, and Campaign Trap 1 is RETIRED AS A QUESTION

Steward, 2026-08-08, `evt_27jwdbz9h2t4c`, on the `D2`/`D3` handback.

**The harness does carry the instrument** into `ken-cli` and `ken-verify` —
proven rather than assumed, with a positive control at
`select_body_emission_authority` so that "no census file" and "no arrivals"
cannot be confused. All three binaries emit `COMPILE` rows and **zero arrivals**
at the boundary.

> ### THE ZERO IS STRUCTURAL, SO NO CENSUS CAN EVER CLOSE TRAP 1 HERE
>
> `SELECTOR_VARIANT_EXCLUSION` and the whole activation seam are `#[cfg(test)]`,
> and those binaries build `ken-runtime` **without** it. ⇒ **A cross-crate
> census can only ever witness the RETAINED lane.** This node's population
> exists solely under A-only exclusion.
>
> **Trap 1 stays open for the activated population, and it is not closable by
> measurement.** Closing it would mean moving a test-only seam into production
> gating — banned, and out of scope for any node in this campaign.
>
> **This retires the question, not the trap.** Five nodes carried Trap 1 as an
> unexamined gap; it is now a named structural reason. Do not re-run this
> census expecting a different answer.

**The `--include-ignored` run splits a question that was being carried as one.**
The sole failure is [[RT-COMPMATCH-TREE-SCRUTINEE]] — *"a tree-producing match
scrutinee is neither Bool nor a constructor"* — **a real-program non-constructor
scrutinee in this wall's class, failing at a different consumer.**

⇒ *Do real programs exhibit this shape?* **Yes** — the class occurs in real
programs. *Is this specific cell reachable in production?* **No**, and that is a
property of the exclusion seam, **not of the language.** The two were previously
one worry, and only the first is what Trap 1 was really about.

## The sixth wall, and the disposition of this cut

`D2` clears the fifth refusal and the program reaches `lowering/units.rs`
`close()` — the continuation-call discharge ledger, whose law is
`planned = direct-emitted ⊎ composed-consumed`, **sets not counts**. One planned
token is neither directly emitted nor compositionally consumed. Every member has
`pending_len=0`, where the resume discharges nothing, so resolving it means
**assigning discharge ownership** — planner, resume, or a third party.

**A distinct planner-population authority: Trap 2, not a defect in `D2`.** Under
the fifth wall this program aborted before `close()` ever ran, so `D2` made the
ledger reachable for this shape for the first time. Routed, not absorbed; no
ownership assigned.

**The lane-pair control is what found it, and it is held, not committed**, at
`65639a13` on `runtime-implementer/sar-lane-pair-evidence`. Divergence is the
premise failing, so it is a stop-and-route: committing it red is impermissible
and weakening it to pass would absorb the stop it exists to detect. **Evidence,
never a candidate — do not publish it.**

`D2`/`D3` at exact `d9175d05` is **accepted partial work** by the same pattern
that governed #6f's `D2`. It does **not** close this node and does **not**
discharge [[RT-MATCH-RECURSOR-CONSUMERS]] `AC-1`.

No hard stop fired. Interface widening in particular did **not** fire:
`resume_active_continuation` (`core.rs:2059`) already takes a `LoweringOperand`.
