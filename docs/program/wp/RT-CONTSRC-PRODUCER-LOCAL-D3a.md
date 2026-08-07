# RT-CONTSRC-PRODUCER-LOCAL D3a — every planner-side consumer assigns the producer-local domain

Deliverable record. Code `ec008478`, control `868fc0aa`, on
`wp/RT-DECL-CLOSURE-PORT-typed-units` over accepted D2b `7316e13a`
(record child `2bd724cd`).

Released by `evt_5wvankf3zpg0g`. The ABI representation — the closed tagged
provenance sum this deliverable implements — was ruled by the Architect at
`evt_3xyy1fq66tyvp`. The separate Architect confirming gate at
`evt_6p6vf0aqnjn3g` answers whether seam 1 lawfully refuses `CurrentLexical`
at a specialization emitter; it binds D3b only and D3a does not depend on it.

## Why there is a D3a at all

D3's original cut bundled two consumer groups with different measurability
into one all-or-nothing deliverable. The act-1 correspondence probe
(`evt_gp162jb84s8b`) measured that no reaching emission sits at nonzero binder
depth, so the lowering half consumes a population only admission creates. The
Steward named that a sizing defect in its own cut and redrew the boundary
(`evt_11esqaep9awbs`).

### The binding order is FOUR checkpoints

⛔ The Steward's two-checkpoint recut — D3a/D3b with D3b "with or after D4" —
is **superseded** by Architect `evt_7vc8zh0rvqyps`, acknowledged by the Steward
at `evt_78xj476p05zvj`. The two-checkpoint phrasing was directionally right and
under-specified in the one place that decides the work: D4 as a single unit
cannot both *create* the nonzero-depth population and *prove* the final
partition, so it never named what would produce D3b's evidence.

1. **D3a** — non-lowering closure. This record. Both lowering consumers
   explicitly refuse; the seam and the pending population stay visible.
2. **D4a** — bounded admission and measurement. Admits the census-bound `V`
   population under the existing authority while D3a's refusals stay in place,
   so real reaching producer-local emissions exist and nonzero-depth
   `CurrentLexical` correspondence can be measured. ⭐ It **may be deliberately
   red**: a red there is the instrument working, not a regression.
3. **D3b** — lowering closure, only after that evidence exists. The seam is
   deleted only when its closed population is empty.
4. **D4b** — admission closeout: `interned = V`, `declined = R`.

⛔ D3a precedes D4a and not the reverse: admission cannot safely run before the
lowering consumers are explicitly fail-closed.

## What landed

### 1. The ABI plane carries a tagged provenance sum

`AbiContinuationInputAuthority.source_owner: PredeclaredFunctionId` becomes
`provenance: AbiContinuationInputProvenance`, a closed sum:

| Arm | Field |
|---|---|
| `EntryAbi` | `source_owner` |
| `ProducerLocal` | `binding_owner` |

`ordinal` and `referent_affinity` are unchanged. Built by one exhaustive
coordinate match (`AbiContinuationInputProvenance::of`) with no wildcard,
default or fallback. Both push sites and the
`validate_continuation_specializations` cross-check route through it, and the
cross-check compares the **complete tagged value**.

The rejected alternative was a domain-total `provenance_owner()`. It is
lossy exactly once: `EntryAbi { source_owner: X }` and
`ProducerLocal { binding_owner: X }` become the same value, so an ABI
authority would accept either standing in for the other whenever ordinal,
owner and affinity agree. Nothing beside an owner encodes the domain — no
independent boolean or tag — because that shape can represent a combination no
coordinate can produce.

### 2. The validator re-derives producer-local sources

`validate_continuation_source_slot`'s `ProducerLocal` arm no longer refuses.
It re-runs the forward walk from `binding.binding_owner`'s own source root to
`locator.environment_origin`, and requires the value at
`locator.environment_index` to contain this exact source, whole.

The independent authority is the walk itself: it re-derives carrier,
ownership, storage owner and affinity through `producer_local_source`, rather
than trusting the fields the projection arrived carrying. Rooting at
`binding_owner` rather than the consumer or emitting owner is load-bearing —
a binding's scope belongs to the function whose body created it.

Membership rather than equality at the position, because a joined position may
legitimately hold several sources; the member must still agree in every field.

### 3. Both view consumers compare the tagged value

`captures()` and `continuation_inputs()` compare
`of(projection.coordinate)` against `authority.provenance`.

### 4. Already delivered by D2b, verified unchanged

The generated-context full-coordinate lookup the release names is
`enclosing.coordinate == input.coordinate` in `exact_continuation_projection`,
landed by D2b. D3a does not touch it.

## The seam is retained, and that is compliance

`entry_abi_pending_producer_local` now has **zero callers** and is retained
under `#[allow(dead_code)]` with a rewritten doc naming its live enumeration:

- `lowering::core`, retained-frame seam — owes `CurrentLexical` consumption.
- `lowering::core`, declared-context seam — owes `GeneratedContextCapture`.

The release said delete it "only when its live enumeration is empty." It is
not empty. Deleting it would be the lie; D3b retires it with the debt it
names. Both lowering seams refuse in lowering's own vocabulary rather than
through this method — their message names lowering's environments, and routing
it through a planner-side helper would replace a message about the operand run
being indexed with one about a coordinate domain.

## Controls and mutation proofs

Four mutations, one at a time, each reverted before the next, all against the
committed tree.

| Mutation | Reds | Leaves green |
|---|---|---|
| Whole-record compare becomes coordinate-only | validator disc 3 (affinity) | disc 1, 2, 4 |
| Locator bounds guard accepts instead of refusing | validator disc 4 (past-end) | disc 1, 2, 3 |
| ABI cross-check compares owner only (the rejected design) | ABI domain-swap half | ABI wrong-owner half |
| `of()` collapses ProducerLocal to EntryAbi | domain-separation `assert_ne` | everything else |

Each reds exactly the discriminator that owns the property, attributed by
panic line, so no row carries another's weight.

### One control was wrong and the mutation caught it

Discriminator 3's first draft cleared the referent affinity. That passed
against a coordinate-only comparison, because the sibling `is_empty()` clause
refused it — the row named the whole-record comparison while measuring the
emptiness guard. It only became real when the corrupted affinity was made a
different **non-empty** value. Measured, not reasoned.

### Two existing rows restated, not deleted

`contspec_abi_refuses_owner_lifetime_and_affinity_disagreement` keeps its
wrong-owner half and gains a domain-swap half: the owner is carried across
unchanged and only the domain moves. Before D3a that substitution was **not
representable** — the record held one `PredeclaredFunctionId` and both domains
projected onto it.

`contsrc_producer_local_coordinate_is_refused_by_both_planner_consumers` had
its first half's reason moved, as D2b moved its second. The validator no
longer refuses on the domain; a fabricated probe coordinate is refused because
its `PredeclaredFunctionId(u32::MAX)` binding owner has no source root. That
refusal is real but **shallow** — it rejects the owner before any environment
is consulted, and the doc now says so. The deeper property is measured next
door with real owners and scopes.

## What this does NOT claim

- Nothing about lowering consuming either producer-local arm. Both emission
  seams still refuse; D3b owns them.
- Nothing about the depth-greater-than-zero correspondence between the
  lowering environment and the semantic seat environment. That remains
  **unmeasured**, per `evt_gp162jb84s8b`. D4a is the checkpoint that creates
  the emissions which would measure it.
- Every D3a arm is reached by direct construction, not by any fixture compile:
  production still declines every producer-local candidate at the D2 gate.
  The controls are built that way deliberately, with positive controls and
  oracles, but the arms are not exercised end-to-end.

## Acceptance

`ken-runtime` lib **729 passed / 2 failed**. The two are the standing D0
baseline reds (`c2_ac4_runtime_host_result_selects_a_separately_generated_
nested_payload`, `two_same_shape_workers_are_distinguished`), which stood at
718/2 at D0 and 726/2 at D2b. No new red. Workspace build, `--locked` and
conformance are CI's.

## Held

D4a, D3b, D4b, candidate, QA of the node, D6 closure, AC-4,
`#27`/case-emission, the call-result SCC and downstream D7.
