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
> ## CORRECTED 2026-08-06 ~18:0xZ — THIS NODE IS THE PUBLISH BLOCKER
>
> **This block previously read "off the critical path; it gates nothing;
> `blocks` is empty." All three were FALSE and all three were mine.**
>
> **Measured at candidate tip `b914c7ff`** (`evt_2h8wm2ff99ayq`), suite
> `px4b_native_production` at **14 passed / 5 failed**, down from 8/11 at
> `fc758323`. **Four of the five remaining failures are this node's subject,
> verbatim:**
>
> | count | seat |
> |---|---|
> | 2 | `FsWriteFile Argument(0)` needs `BytesPointerLength`, cannot observe in `CarriedWord` |
> | 1 | `ConsoleWrite Argument(1)` — same need, same phase |
> | 1 | `FsReadFile Argument(0)` — same need, same phase |
>
> The fifth is `ken native trap: explicit entry trap`, provenance under
> measurement.
>
> ⇒ **This node gates the merge of 212 reviewed commits with six nodes behind
> them.** It is the critical path until something measured says otherwise.
>
> ### The evidence route matters — do not cite the wrong one
>
> **Do NOT justify this node from the historical `c7410b79` `BoundaryCarrier:
> a host-effect operand is a specialized-only surface` signature.** The
> Architect refuted that attribution and the refutation stands
> (`evt_7v61ed5pn9q3t`): `claim_host_effect_seat` **did not exist** at that
> commit, the refusal fires in `specialized_at` before any seat key or need is
> consulted, and the wording is the **generic** phase-boundary vocabulary every
> specialized-only leaf uses. The Steward matched generic words against a
> function that did not yet exist — an invalid inference that happened to point
> the right way.
>
> **What establishes this node is the FOUR typed per-seat refusals measured at
> the tip**, naming exact operation, argument index, need and phase. The
> Architect predicted exactly this: *"after the bulk boundary is removed, this
> source shape can expose the real byte-span gap — a later seat-specific
> dependency, not what raised the historical signature."*
>
> ### The `depends_on` is NOT a cycle, and that needs a ruling not a default
>
> This node's `depends_on` names `RT-CONTSRC-PRODUCER-LOCAL`, whose branch
> cannot merge without this node. **That is only circular if this node must
> wait for a merge — it does not.** Its inputs are that branch's code, which is
> present on the branch. **The sound reading is that byte-span is the next unit
> ON that branch, not a successor awaiting a merge.** Steward to rule formally
> once the fifth failure's provenance returns.

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
> **THIS NODE IS ONE OF THE TWO VISIBLE ACTIVATION ROUTES.** It removes the
> first downstream blocker on the existing `rt_parity` route. **The moment a
> carried source-Match path becomes successfully executable, the dormancy
> argument expires.**
>
> ⇒ **This node may NOT merge as an activation without carrying forward the
> still-BLOCKED control families from `RT-CONTSRC-PRODUCER-LOCAL`** — their
> completing producer **and** their red-before-green controls.
>
> ### THE GATE IS ONE PRODUCER, NOT FIVE OBLIGATIONS
>
> **Measured per-family register `evt_5tzqtkgw02gxg`, disposition
> `evt_5p3hcgng950pw`.** Four of the five non-buildable families name the
> **same** first missing producer:
>
> > a **cross-unit carried word** reaching a source-machine `Match` **in a unit
> > that does not feed a byte-span effect seat**.
>
> | family | status | producer |
> |---|---|---|
> | 1 nontrivial continuation | BLOCKED | the producer above |
> | 2b distinct predecessors + exact-once | BLOCKED | the producer above |
> | 6 HostResult Ok/Err | BLOCKED | the producer above, word classed `HostResult` |
> | 3 identity / arity / field-order | BLOCKED | the producer above **plus** a `RuntimeExpr`-level route to a malformed case table that still reaches the seat (`rt_parity` is typed Ken, so the elaborator refuses a wrong constructor before lowering) |
> | 7 borrowed route | BLOCKED | the producer above, word classed `BorrowedOpaque`, whole case set the borrowed family |
> | 8 wrong-class | NOT RELEASABLE | `mismatch_block` measured reached **zero** times |
>
> **Family 7 is UNREACHED, not UNREACHABLE.** It was never measured
> unreachable. **Do not compress that** — unreached is a producer gap someone
> closes; unreachable is a closed question, and they license opposite decisions.
>
> **Why the obvious substitutes do not work, so nobody re-spends the turn:** the
> `rt_parity` carried producer is a **declared function unit's parameter
> carrying a cross-unit runtime value.** A closure parameter bound to a
> `Construct`, a `RuntimeDeclaration` parameter fed a `Construct`, and the
> borrowed process input **all arrive `Specialized`** — each completes and
> returns a plausible value **from a route that is not this one.**
>
> **Abort boundary, exact:** Phases 0–2 (descriptors, block allocation, whole
> selector graph) are decided; `PHASE 2 selector graph COMPLETE` fires,
> **`ALL LEAVES LOWERED` never does.** Phase 3 leaf lowering and Phase 4 join
> completion are after the abort.
>
> ### SECOND, INDEPENDENT GATE ENTRY — this node does NOT remove it
>
> **Measured `evt_eb1deg5r0j0r`, ruled `evt_7rdd0jgtg6zwh`.** Family
> **2a-acquisition** (inherited join acquisition), and possibly **family 5**,
> are blocked on a producer that is **not** the shared cross-unit carried word:
>
> > the `#[cfg(test)]` control seam must be **VISIBLE TO THE BUILD THAT REACHES
> > THE ARM.** It exists only in the `ken-runtime` **lib** build; the rig that
> > exercises the carried route — `rt_parity_native` — lives in **`ken-cli`**
> > and links a **non-`cfg(test)`** build.
>
> **Byte-span observation does not fix this.** Two independent blockers, kept
> separate deliberately so a successor cannot read one discharge as both.
> Feature-gating the seam across that boundary is a real change to what ships
> and belongs to the gate — **not to a bounded control child.**
>
> **Family 2a ships a TRANSITION SENTINEL, not a discharge:** it asserts
> `applications == 0` and **reddens the moment the arm becomes reachable under
> `cfg(test)`** — exactly when 2a becomes writable. **Prefer this shape for
> every remaining gate entry where it is cheap:** a blocked family that asserts
> its own unreachability converts a promise into a mechanism that fires by
> itself, instead of a note someone must read at the right moment.
>
> ### THE GOVERNING HAZARD — state the POPULATION with every claim
>
> Three times on this node a **true** fact about one population was transferred
> to a neighbouring one. None was carelessness; each read as obviously
> transferable:
>
> | true of | falsely transferred to |
> |---|---|
> | the route is **entered** 10 times on `rt_parity` | a **control** can be written on it |
> | a closure parameter is carried at an **effect argument** | carried at a **Match scrutinee** |
> | the mutation reddens on **`rt_parity`** | it reddens in the **lib build where the seam lives** |
>
> ⇒ **"Buildable" is meaningless without "in which build, on which rig."** Say
> the population in the same sentence as the claim.
>
> ### GATE-SHAPED is not GATE-SATISFYING
>
> **A control whose red can only be demonstrated by an UNCOMMITTED production
> mutation is gate-SHAPED.** The gate must be re-runnable by someone who was not
> present for the demonstration — that is the only moment it exists for. A red
> observed once in a turn and not committed discharges *that day's* question and
> guards nothing here.
>
> **Removing the first blocker is NOT a promise that byte-span alone
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
> reaches the same abort. Route entry, an emitted counter, or plausible IR is
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
> **It is NOT "HostResult payloads require this."**
> **Do NOT claim the probe identified the concrete runtime class of the
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
> **THE SCOPE BOUNDARY, and it is the sentence that sizes this node.** The
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
> "make `CarriedWord` observable" and **not** a phase relaxation. A frame
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
