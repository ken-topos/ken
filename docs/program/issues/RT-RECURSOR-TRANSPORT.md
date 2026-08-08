---
id: RT-RECURSOR-TRANSPORT
title: "Retire the two live recursor residual classes — MatchScrutineeRecursor and LexicalCallArgumentRecursor — off the RecursiveDescent lane"
status: ready
owner: runtime
size: M
gate: none
depends_on: [RT-CONTSPEC-WITNESS]
blocks: [RT-DESCENT-RETIRE]
github: null
origin: Operator directive 2026-07-29 — prioritize replacement of RecursiveDescent, migrate the remaining residual classes, do not linger half-migrated. Campaign docs/program/16-recursive-descent-retirement.md. Steward-filed (agents cannot create tracked work per COORDINATION §2). Recut 2026-08-08 per Architect ruling evt_237tbdsacqbk4.
---

> # RECUT 2026-08-08 — THE WHOLE PRIOR CONTRACT IS WITHDRAWN, NOT AMENDED
>
> **Authority: Architect ruling `evt_237tbdsacqbk4`**, answering the Steward's
> re-derivation request `evt_4hr31qp6ab5xg`.
>
> Everything this node said before today was written against a world in which
> `RT-DECL-CLOSURE-PORT` `D7` had not landed and the ContinuationSpecialization
> seams did not exist. Both are now false. **Do not read the old contract for
> context; it is superseded, and it is wrong in the direction that costs the
> most — it describes work that no longer needs doing and a base that would
> destroy landed architecture.**
>
> Three specific withdrawals:
>
> **1. The global population-authority obligation is withdrawn.** The old text
> said this node owes *"one exact `BoundaryUse` record per static lowering
> event"*, replacing `D7`'s population authority in place. That sentence is
> **superseded, not an unfulfilled deliverable**. `BoundaryUse` has **zero hits
> in `crates/`**; the surviving references are historical docs. `D7`'s actual
> landed authority is `PlannedEffectSeat`, and it is **discharged for its own
> domain** — host-effect occurrences, with an intentionally effect-specific key,
> Need/Avail vocabulary and choke point.
>
> It does not extend to either residual class, and **this node must not widen it
> into a universal lowering-event record.** That is the exact domain conflation
> ruled out in `evt_1v9m7t4m9dmj7`.
>
> **Nor is there a missing universal authority to build.** Lowering deliberately
> uses separate exact authorities for separate semantic populations: host-effect
> seats, aggregate allocation occurrences, continuation source slots,
> continuation specializations and call identities, join plans, typed declared-
> unit calls. Security comes from the exact domain-specific producer plus its
> checked consumption boundary — **not from one global token vocabulary.**
>
> **2. The ordering rule is withdrawn.** *"Population authority FIRST, cell-level
> repair after"* is an artifact of the pre-`D7`, pre-CONTSPEC world. It is
> replaced by the three-step bounded order in the frame.
>
> **3. `07ce6ef1` is NOT the repair base.** The old text said it *"SURVIVES AND
> IS THE REPAIR BASE — do not reset it"*. It is **not an ancestor of `d9b2eb38`**
> and exists only on preserved and old `D7` branches. Its `StaticRecursorWorker`
> prototype has 36 crate hits there and **zero on current `main`**, while the
> four core files have diverged by roughly **+44,986/-16,942**. Continuing or
> cherry-picking it **would overwrite the landed continuation-specialization,
> ownership, ABI and ledger architecture.** Cite it as historical refusal and
> design evidence only; re-derive every mechanism claim on the new base.
>
> **Size withdrawn from `L` to a provisional `M`.** The hard internal mechanism
> the `L` assumed this node must invent has since landed.

> # THIS NODE DELIVERS DIRECTLY. IT DOES NOT MERELY CLOSE.
>
> **A prior banner said *"THIS NODE NO LONGER DELIVERS DIRECTLY — it closes when
> the terminal seam merges."* That is withdrawn and it did measurable harm.**
>
> On 2026-08-08 an implementer, correctly suspicious that closing an
> unimplemented node would be wrong, checked that suspicion **against this
> banner** and retired it. The banner answers *when* the node closes; it was
> read as evidence that closing it is *sound*. Both residual classes were live
> in production at the time, and the closure would have unblocked
> `RT-DESCENT-RETIRE` — the lane deletion — while two classes could still select
> the lane.
>
> ⇒ **An instruction to close is not evidence that the work behind it is done**,
> and a node asserting its own closure timing is the least independent source
> available for whether that closure is sound. The check that settles it: **are
> the classes this node owns still live in production?**

## What this node owns

Two `RecursiveDescentResidual` variants, both still live and both still
selecting the `RecursiveDescent` lane. Find them by name in
`crates/ken-runtime/src/cranelift_backend/lowering/core.rs` — do not trust a
line number in this file or the frame, including one written today.

**`MatchScrutineeRecursor`** — an ordinary `Match` whose scrutinee is a
`ComputationalMatch` carrying recursive positions.

**`LexicalCallArgumentRecursor`** — a `Call` whose callee is a `LexicalClosure`
and whose **argument** is such a recursor.

Three sibling classes are already retired: `TransparentDeclarationClosure`,
`SeedClosureCall`, `ProducerMatchCall`. These two are the remainder.

## They were folded as "one mechanism" — that claim is now conditional

The prior text folded them on the grounds that both fire on an active
computational recursor and *"differ only in the syntactic position it
occupies"*, so retiring one without the other would build the same transport
twice.

**That remains the working hypothesis and it is no longer an assumption you may
carry.** Per the ruling: if `D1` shows the two positions require materially
different transports, **hard stop and re-size or re-fold** — do not preserve the
"same mechanism" claim merely because both variants mention an active recursor.

## What remains owed

The frame carries the full contract. In outline, and in this order:

1. **`D0`/`D1` — re-census and activation probe on the post-WITNESS base.**
   Under a test-only per-variant selector exclusion, run one discriminating
   executable witness per position and record the first real functionized
   outcome. **This determines whether the landed continuation machinery already
   closes either class for free.**
2. **Only for a class that does not close for free, add the narrow consumer-port
   authority its failure proves necessary** — domain-specific, planner-owned,
   over the existing continuation machinery.
3. **Only after both executable positions are green** may the two variants and
   their test-only selector hooks retire.

The surviving invariant is outcome **(b)**: invocation-local activation, resume
and return-hole state never enters ABI data. Only ordinary typed values cross;
static continuation and callee identity stay planner- and compiler-owned; any
open, escaping or ambiguous case refuses **before** allocation or call emission.

## Base

**Branch after [[RT-CONTSPEC-WITNESS]] actually merges, from that then-current
`main`, and pin the new base at pickup.** Not `07ce6ef1`, not any preserved
freeze ref.

## Sequencing

Last migration before the capstone [[RT-DESCENT-RETIRE]], which owns the lane
itself and must not be closed by this node. By the time this runs, three classes
are retired, so program shapes that have **never** reached `FunctionizedUnits`
will reach it here for the first time — campaign Trap 2. **Expect a hard stop
and route it; that is the fail-closed machinery working, not a defect.**
[[RT-FNUNIT-RESULT-TOKEN]] is one such stop already routed.

## The frame is written

`docs/program/wp/RT-RECURSOR-TRANSPORT.md`. Campaign context, the traps binding
every node in this arc, and the schedule:
`docs/program/16-recursive-descent-retirement.md` — read it before the frame.
