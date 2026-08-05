# `RT-CONTSRC-PRODUCER-LOCAL` `D3b` (re-cut) — consumer-specific availability

Status: **complete.** Test-green at the baseline. Candidate for QA.

Branch `wp/RT-DECL-CLOSURE-PORT-typed-units`, over the `D3c` record at
`f5e4fa9f`.

## What this replaces

`D3b` first landed a product over `(root coordinate, availability)` with three
lawful pairings and three crossed, resting on the premise that a value's root
provenance constrains where a consumer finds it. `D3c` measured that premise
false: at a predeclared seat under one intervening binder an entry root's ABI
position `0` is not its immediate position, which is `1`.

The re-cut keeps the root coordinate as **identity only** and makes availability
a **consumer-specific planner-issued claim** over two environments:

- `CurrentLexical` — the semantic environment in force at one exact predeclared
  emission occurrence, binders counted, obtained by the forward walk.
- `EntryFrame` — a declared slot in one exactly identified frame's operand run.

Both arms are open to **either** root. `RootIsImmediate`, the pairing table, the
equality `immediate_slot == source_abi_position`, and
`ContinuationImmediateResolution::root` are retired.
`GeneratedContextCapture` is **subsumed** into `EntryFrame`: a generated
context's capture run and a predeclared function's entry run are the same *kind*
of environment — a declared operand run — differing only in which frame declares
it. Two names for one environment class is what let the old law read a frame
identity off a root domain.

## The gating measurement: caller-frame multiplicity

The ruling made this decisive — if one target capture is consumed from more than
one lawful source frame, a single target-level claim is insufficient and
planning must issue claims per causal call edge, or hard-stop with the concrete
edge.

**Result: no multiplicity, for either consumer, structurally and by
measurement.**

### The first census was an artifact, and saying so is the point

Keyed on `ContinuationSpecializationId` alone, a corpus census reported
specialization `0` consumed from **three** different frames. That is not
multiplicity — `ContinuationSpecializationId` is **per-compile**, so id `0` in
one fixture and id `0` in another are different specializations, and the census
merged them. A count taken that way cannot answer this question at all.

### The collision-immune question

Ask instead, **within one plan**, whether the seam's frame is a function of the
target's own key:

| consumer | question | result |
|---|---|---|
| direct emission | `defining_owner == unit.emission_owner()`? | true, 40/40 |
| context capture | indexed frame = enclosing spec's emission owner? | true, 20/20 |
| context capture | indexed frame = that spec's own context? | false, 20/20 |

`emission_owner` is a **field of `ContinuationSpecializationKey`**. Two emitting
frames therefore give two keys and two distinct interned specializations, so one
specialization can never be emitted from two frames. For captures, the context
is interned on `(enclosing, worker_body_origin)`, `enclosing` determines the
enclosing unit, and that unit's key determines its `emission_owner` — so the
source frame is a **function of the context's own interning key**.

⭐ The structural argument is what carries this; the measurement is its positive
control, confirming the seam respects the key rather than reaching a frame the
key does not name. Neither alone would be enough: the argument could be about a
seam nobody takes, and 20 agreeing observations of one shape could be a corpus
accident.

## What the same measurement found, and it was a live defect

The frame whose `defining_abi_operands` the capture consumer indexes was, in
every observation, a **predeclared** function — never the enclosing
specialization's own generated context. So the capture consumer reads a
**predeclared entry ABI run**, and its claim is an entry-frame claim against
that frame, whose declared slot is the coordinate's position in that run.

The projection was withholding that view entirely — `context_capture: None` on
the predeclared arm — so **every** generated-context capture refused with *"a
generated context capture carries no context-capture availability claim"*. That
was 19 of the 33 mid-migration reds.

⭐ This is `D3c`'s two-environment result made concrete at one frame: the direct
consumer needs the nearest-exact-alias **lexical** index and the capture consumer
needs the **entry-run** position, and `D3c` measured those two numbers diverging.
One `availability` field repaired for either consumer silently mis-serves the
other.

⛔ `predeclared_entry_frame_slot` returns `None` — no capture claim — when the
frame declares no member. A `ProducerLocal` coordinate is a mid-body value with
no position in any entry run, so the boundary **fails closed**; `D4b` owns making
such a value capturable.

## The hard stop, and the law that replaced it

The exact-once `CurrentLexical` precondition was **unsatisfiable for a program
class that works today**, and the Architect accepted the stop
(`evt_cmcxf4h7v1st`).

**Measured**, exactly: coordinate
`EntryAbi { source_owner: 0, source_abi_position: 1, Parameter }` present at
lexical indices **0 and 2** of a 3-element seat environment. Cause, read off the
walk: `walk_continuation_value_environment`'s `Let` arm mints a producer-local
value only for an `Effect`; otherwise it pushes **the bound expression's own
authority**. So `let y = x` puts a parameter's identical coordinate at both its
entry position and the binder position.

The ruling identified the law as conflating two questions — *does this position
certainly hold `S`* and *is it the only position that does* — and `D3b` needs
only the first.

### The replacement: nearest exact alias

One total rule, `nearest_exact_alias`, shared by the planner that issues the
claim and the consumer that revalidates it:

1. eligibility is **exact equality of the complete requested source-slot
   authority** — coordinate, carrier, ownership, storage owner, referent
   affinity — against a position holding exactly `Closed([S])`;
2. among eligible positions, the **minimum de Bruijn index**.

⭐ **Why this is not the banned first-match.** The ban exists because choosing
among candidates never proved equivalent silently picks one of several different
values. Here every candidate is proved the same semantic value **before ordering
is consulted at all**: the discriminator is eligibility, not ordering. The proof
is the authority's own algebra — `join` unions and *deduplicates complete
records*, so `Closed([S])` means every represented path yields exactly `S`, and
`Closed([S, T])` is not an exact alias even though it contains `S`. My escalation
worried that an `If` join could carry the same identity with a different SSA
value; the ruling closed that directly — such a join is `Closed([S, T])` and is
ineligible, while an `If` whose branches both yield `S` stays `Closed([S])`,
which is exactly the proof needed. **No SSA-equality instrument was required.**

⛔ `min` is written as a fold over the whole eligible set rather than an early
break. The two agree today because the scan is ascending — which is precisely why
the total rule is spelled out: an early break would *read* as "take the first",
and a later reordering of the scan would silently change the answer.

⛔ Exact-once membership is **preserved** for ordered capture projections and
predeclared `EntryFrame` membership. Those are declared slot runs, not semantic
environments; the alias argument comes from `join` deduplicating in the semantic
environment and does not transfer.

Every "post-shift index" spelling is retired — 78 occurrences across four files,
including a test function name — because a reader who thinks of this number as a
shift will reconstruct the exactly-once law it replaces.

### The six required controls

1. the measured duplicate selects index 0 and the real consumer accepts it —
   `d3b_the_duplicated_entry_source_selects_the_nearest_alias`
2. perturbing that claim to index 2 is refused — same row
3. inner `Closed([S,T])` + outer `Closed([S])` selects the **outer** —
   `d3b_alias_eligibility_not_position_decides`
4. `Closed([S,T])` with no singleton refuses — same row
5. same coordinate, different contract does not qualify —
   `contspec_parameter_affinity_comes_from_its_exact_source_slot`
6. zero-depth and shifted-index discriminators stay live — `d3c_*`, `d4a_*`,
   `d5a_the_capture_*`, all green

⭐ **Controls 1 and 3 select opposite ends of the environment, and that pairing
is the point.** A suite carrying only the nearest-alias case passes just as well
under the banned positional shortcut, because there the first member *is* the
answer. **Mutation-proved**: replacing the rule with "first coordinate-containing
member" reds **only** control 3 (`left: 0, right: 2`) — controls 1 and 2 pass
under it. Replacing `min` with `max` reds controls 1 and 2. Neither mutation is
caught by the other's row.

Control 5 is discharged by a **real production perturbation on a landed
fixture** rather than a synthetic environment: `ResultLifetimeProxy` narrows an
input's affinity from `[NoReferent, PersistentStore, InvocationArena]` to
`[NoReferent, PersistentStore]`, which the row could previously only *watch*
reach the projection.

## Two-stage `EntryFrame` finalization — built

Stage 1 and stage 2 are **two types**, not one type with a sometimes-filled
field: `ContinuationFrameRequirement` (structural, keyed on
`(enclosing, worker_body_origin)`) and `ContinuationFrameIdentity` (exact,
carrying the resolved `ContinuationContextId` **alongside** the key it resolved
from). A requirement cannot be presented to a consumer — nothing converts one
into an identity except finalization.

`finalize_continuation_availability_plan` runs **once, whole-plan**, after every
context is minted, over every specialization input and every context capture.
⛔ Not lazily: lazy resolution leaves a plan carrying an unresolvable frame
*accepted*, refused only if something reaches it. ⚠ The `0/60` figure originally
cited here as making that gap invisible is **retracted** — see the note below and
`RT-CONTSRC-PRODUCER-LOCAL-D4b.md`. The argument for whole-plan finalization does
not rest on it: a claim nothing reaches today may be reached tomorrow, and
"accepted but unresolvable" is the wrong state to publish either way.

`continuation_input_view` is the **publication gate** — the single conversion
both populations pass through, now fallible, refusing when no finalized entry
exists.

The consumer revalidates all three sides: it re-resolves the recorded key against
the plan in hand and checks the answer agrees with the recorded id. ⛔ Not
redundant with finalization: finalization proves the key resolved uniquely *in
the plan it ran over*; this proves the consumer holds that same plan.

⚠ Two placement facts, both found by measurement. Finalization must run **after**
`validate_continuation_specialization_plan` — that validator re-derives the whole
plan and compares for exact equality, and a finalized sibling is state a
re-derivation cannot produce; stamping first reddened **83 tests, none about the
plan being wrong**. And it must run after the context ABI install, the earliest
point any view can be built.

⛔ Its control asserts **non-vacuity first**: a zero-or-multiple perturbation over
an empty requirement set succeeds trivially, so the count of generated
requirements is what distinguishes "the refusals fire" from "nothing to fire on".
The witness carries them.

⚠ **THE GAP AS FIRST WRITTEN — RETRACTED BY `D4b`.** This said no lowered
program consumes a generated frame identity (`0/60`), so the revalidation was
exercised by construction rather than execution. **That figure was wrong.** It
was measured while the capture-view defect was still live, so the probe recorded
the path refusing rather than the path being absent. `D4b` re-measured on the
repaired tree: the generated-frame arm is taken **30 times** across ordinary
lowering tests, including one that emits and executes a real object. See
`RT-CONTSRC-PRODUCER-LOCAL-D4b.md`.

## Suite

`ken-runtime` lib: **728 passed / 7 failed / 1 ignored**.

- **7 baseline reds, unchanged** — the two standing `D0` reds plus the five
  former `D4a` reds at their downstream `Var: no runtime binding` boundary.
- **No other failures.**

Both counts were taken against the same commit's tree; the 7 are the same seven
named in the `D3c` record at `f5e4fa9f`. The workspace build, the `--locked`
gate and conformance are CI's.
