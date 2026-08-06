---
id: RT-CARRIER-BYTESPAN-OBSERVE
title: "Carrier byte-span observation capability — every BytesPointerLength seat is SPECIALIZED_ONLY and the carrier has no total emitted byte-span observer, so a carried host result cannot satisfy a byte-span effect seat"
status: draft
owner: runtime
size: L
gate: none
depends_on: [RT-CONTSRC-PRODUCER-LOCAL]
blocks: []
github: null
origin: Architect capability disposition evt_4c26q24rp7xqb (2026-08-06) — no sound in-node repair exists under the current capability. Steward ruling evt_3pr04vk7zrd7c recut AC-1 clause 1 out of RT-CONTSRC-PRODUCER-LOCAL into this node. Steward-filed (agents cannot create tracked work per COORDINATION §2).
---

> ## THE FRAME IS OWED. This node is `draft` and is NOT startable.
>
> What follows is the Architect's ruled **shape**, captured while fresh. It is
> not a frame: it has no fixed inputs measured at a named SHA, no acceptance
> criteria with controls, and no contention check. **The Steward owes those
> before this flips `ready`.**
>
> This node is **off the critical path**. It gates nothing — `blocks` is empty.
> It exists because a real capability gap was measured, not to make a graph
> tidy.

> ## ACTIVATION GATE — THIS NODE INHERITS AN OBLIGATION IT DID NOT CREATE
>
> **Architect `evt_7qfayjcebxv5y`, ratified by the Steward `evt_4nabbpm2crz82`.**
>
> `RT-CONTSRC-PRODUCER-LOCAL` lands `lower_source_carried_match` as a
> **reviewed DORMANT partial mechanism with an unmeasured runtime residual.**
> That was allowed **only because no completing Runtime rig executes the carried
> source-Match route at all** — which bounds the present risk and the present
> claim together.
>
> ⛔ **THIS NODE IS ONE OF THE TWO VISIBLE ACTIVATION ROUTES.** It removes the
> first downstream blocker on the existing `rt_parity` route. **The moment a
> carried source-Match path becomes successfully executable, the dormancy
> argument expires.**
>
> ⇒ **This node may NOT merge as an activation without carrying forward the
> still-BLOCKED control families from `RT-CONTSRC-PRODUCER-LOCAL`** — their
> completing producer **and** their red-before-green controls. Read that node's
> per-family register (BUILDABLE / BLOCKED, each BLOCKED family naming its exact
> first missing producer) and discharge what activation makes reachable.
>
> ⛔ **Removing the first blocker is NOT a promise that byte-span alone
> completes every row.** It may reveal a later one. Treat the register as the
> checklist, not this paragraph.
>
> **The other activation route, if someone builds it instead:** a typed-unit /
> closure-parameter producer delivering a genuine carried word into
> source-machine `Match` under a planned scalar cut. **It carries the identical
> obligation** — the gate is on *activation*, not on this node's id.
>
> **The admissibility rule any such control must meet:** the entire claimed
> property is decided **before** any independent abort, and a mutation of that
> property has been observed to make the exact assertion **RED** before the run
> reaches the same abort. ⛔ Route entry, an emitted counter, or plausible IR is
> **not** evidence.

> ## THE ATTRIBUTION — write the frame from THIS SENTENCE, not a paraphrase
>
> **Architect `evt_2qzwanx82m06r`, and it CORRECTS an earlier reading of mine.**
> The gap is:
>
> > The reachable `Constructor` predecessor of AC-1 match origin `268` projects
> > a carried child into effect origin `264`; that child's
> > `BytesPointerLength` seat lacks carried availability.
>
> ⛔ **It is NOT "HostResult payloads require this."**
> ⛔ **Do NOT claim the probe identified the concrete runtime class of the
> child, or a `HostResult`-selected path. It identified neither.**
>
> **Why the obvious reading is wrong.** The joint-keyed probe reported
> `leaf=(match_origin 268, CARRIED, case 0, reps={Constructor})`. That
> `reps` component is a **compile-time selector predecessor** — the only
> selector edge that can enter physical carried case 0 — **not an observed
> runtime class.** The diagnostic is emitted while the compiler lowers a
> selector graph.
>
> Under `Open` / `OpaqueIngress` the planner maps **every** case to
> `Reachable` and validation forbids eliminating one, so **compilation must
> lower the reachable `Constructor` leaf even if a later execution would select
> `HostResult`.** A different runtime arm cannot rescue a compile-time failure
> in an independently reachable leaf. ⇒ **Lawful decode, not a class gap.**
>
> ⭐ **THE SCOPE BOUNDARY, and it is the sentence that sizes this node.** The
> *same leaf*, *same `CarriedWord` phase*, a *different slot of the same
> operation* is **SATISFIED**:
>
> ```
> Argument(0)  need=BytesPointerLength    avail={specialized:true, carried:FALSE} REFUSED
> Capability   need=CapabilityTokenScalar avail={specialized:true, carried:true}  SATISFIED
> ```
>
> ⇒ **Per-seat availability, never a blanket phase ban.** This node is *give
> `BytesPointerLength` a total emitted observer over the carrier*. It is **not**
> "make `CarriedWord` observable" and **not** a phase relaxation. ⛔ A frame
> opening *"carried words cannot satisfy this operation"* would be **FALSE**.

## The gap, as measured rather than described

`RT-CONTSRC-PRODUCER-LOCAL`'s `AC-1` row reaches a host effect seat and refuses:

```text
Effect: seat Argument(0) of FsReadFile needs BytesPointerLength,
        which it cannot observe in CarriedWord
```

Exact `PlannedCaseEmission` authority on both of that row's carried source
matches, measured at `c2ae3eed` (`evt_657rvy4d1m4k9`):

```text
origin 268 index 0  status=Reachable  producers=Open
  scrutinee=267  producer_origins=0
  flow=[Environment:271->267, OpaqueIngress:271->271]
origin 271 index 0  status=Reachable  producers=Open
  scrutinee=270  producer_origins=0
  flow=[Environment:12->270,  OpaqueIngress:12->12]
```

**Zero producer origins does not make it closed.** The `OpaqueIngress`
self-edge is *positive fail-closed authority*: the scrutinee arrives over the
host boundary, so no result route is statically known and every case is
genuinely `Reachable`. Case-emission pruning structurally cannot reach that
seat.

⇒ The refusal is real. Nothing may be pruned, inferred from the operation
catalog, or rewritten to `Closed`.

## Why there is no in-node repair (Architect, `evt_4c26q24rp7xqb`)

Every `BytesPointerLength` seat is presently `SPECIALIZED_ONLY`, and the
carrier has no total emitted byte-span observer. Setting this seat — or only
`FsReadFile` — to `EITHER_PHASE` would **assert a capability that does not
exist**. `OpaqueIngress` describes planner flow, not the runtime
tag/class/owner, so it is also insufficient to choose a representation row.

## The ruled shape — five steps, and step 1 comes first

1. **Measure before freezing anything.** Enumerate every legal runtime
   representation that can reach every `BytesPointerLength` seat, and measure
   the exact tag / class / owner / extent shape of each.
2. **Normalize at the producer.** Convert invocation-owned byte sources into a
   self-evidencing bytes representation at their producer. Freeze the exact
   invocation-owned `Bytes` row or explicit byte-span subtype **only after** the
   step-1 measurement.
3. **One emitted helper.** Add a `bytes_view`-style carrier helper, analogous to
   the existing integer view, returning pointer and length **only after**
   tag/class/owner/extent and arena-bounds checks.
4. **One lowering observer.** It consumes the exact planned effect-seat record
   and emits that helper call, returning SSA pointer/length. It never constructs
   `Lowered::Bytes` and never decodes at Rust/JIT time.
5. **Flip the phase last.** Only after producer and reader close together may
   the complete applicable `BytesPointerLength` seat population become
   `EITHER_PHASE`, and **every excluded seat needs an explicit proof**.

## Banned

- **This is NOT `Carried -> Lowered`.** That inverse is withheld by design and
  reintroducing it here is the wall wearing a different name.
- **Do not dereference an arbitrary `BorrowedOpaque` scalar.** That class also
  represents capability and resource tokens; a byte-span reader that accepts
  the class rather than a measured row is a confused-deputy hole.
- No widening of `Avail`, no descriptor weakening, no planner/ABI authority
  change ahead of step 5.
- No representation frozen ahead of the step-1 measurement.

## Evidence the Architect named as required

Persistent **and** invocation-owned `Bytes`; phase equivalence; wrong
tag/class/owner/extent refusal **with zero host dispatch**; an exact seat
ledger; and a mutation that restores the specialized-only refusal.

## What this node does NOT own

The carried source-match **class dispatch** correction
(`Constructor`/`HostResult`/`BorrowedOpaque`, every other class fails closed)
stays in `RT-CONTSRC-PRODUCER-LOCAL`. The two are technically distinct units,
and the predecessor is approvable as semantic partial progress without this
node existing (`evt_3pr04vk7zrd7c`).
