# RT-CONTSRC-PRODUCER-LOCAL D4a — bounded admission, and the measurement D3b needs

Deliverable record. Code `5d87299c` on `wp/RT-DECL-CLOSURE-PORT-typed-units`
over QA-approved D3a `14b111ae`. Released by `evt_5j8a7v9hrjdek`; checkpoint 2
of the binding order D3a → D4a → D3b → D4b (Architect `evt_7vc8zh0rvqyps`).

## Admission is a deletion, not a selector

The D2 transition sentinel that declined every producer-local candidate stood
in `exact_continuation_source_environment` and is now removed. Its own promise
class named D4 as the retiring event.

**Nothing replaces it, and that is the whole shape of the change.** The
declined set `R` is refused **upstream**, by the take-loop that D4a did not
touch, on authority that was always there:

| decline clause | census members it refuses |
|---|---|
| an `Open` value | `OPEN[ih-binder]`, `OPEN[let-value:Construct]` |
| more than one exact source at a position | `AMBIG2[let-value:If]` |

Those are exactly the census's three non-closed positions. So admitting `V` is
*removing* a filter, never adding a predicate. No corpus, closure identity,
planned-member status, first-`Open` classification or edge selector is
consulted. The full required vector — `required_input_count` positions, every
one closed and unambiguous — remains the sole authority.

The `ProducerLocal` arm's `dead_code` allowance is also removed: the arm is now
constructed and read on the ordinary planning path, not only by controls.

## The measurement — this is what the checkpoint is for

A correspondence probe was installed ahead of the lowering coordinate refusal
(that refusal is what these emissions now hit), run across the whole
`ken-runtime` lib corpus, and removed. 92 records.

### Positive, and new

**Five records carry `CurrentLexical` availability.** No producer-local
availability had ever reached lowering before — at act 1 all 85 records were
`EntryAbi`, because the D2 gate declined every candidate.

All five are one distinct emission (`construct=StaticOriginId(89)`, owner
`Predeclared(fn0)`), and for it:

| fact | value |
|---|---|
| entry width | 2 |
| planner seat environment length | 3 |
| **binder depth** | **1** |
| lowering `producer_env` length | 3 |
| verdict | `LEN-AGREE`, `SEAT-HOLDS-IT` |

So at binder depth **1** — the region that was entirely unmeasured at act 1,
where every observation sat at depth 0 — the lowering environment and the
planner's semantic seat environment have the **same length**, and the seat
environment holds this input's exact coordinate at its `post_shift_index`.

The act-1 emitter-class separation reproduces exactly: all 65 `LEN-AGREE`
records are predeclared emitters, all 27 `LEN-DISAGREE` are specialization
emitters. Zero exceptions, consistent with the Architect's `evt_6p6vf0aqnjn3g`
ruling that a specialization emitter must refuse `CurrentLexical`.

### What this measurement does NOT establish

- **It does not discriminate the post-shift walk from passing the locator
  index through.** On this emission `post_shift_index = 0` and
  `locator.environment_index = 0` — the same number. The environment grew, but
  this value's index did not move. A shifted emission, where the two differ, is
  **not among the reaching population**.
- **Length agreement is not value agreement.** The two planes hold different
  types; nothing here proves the lowering binding at index *i* is the same
  value as the seat environment's entry *i*. Length and planner-side placement
  are what was compared.
- **One distinct emission, at depth 1 only.** Not a population.
- One further record carries a `ProducerLocal` coordinate but is **not**
  production: its `binding_owner` is `PredeclaredFunctionId(u32::MAX)`, the D1
  `producer_local_probe` sentinel injected by a D5a route mutation. It is
  excluded from every count above.

## Deliberate red — authorized, and verified per row

`ken-runtime` lib **724 passed / 7 failed**. The frame authorizes this: *"This
checkpoint MAY BE DELIBERATELY RED. Its purpose is to produce the real reaching
producer-local emissions... A red here is the instrument working, not a
regression to chase."*

Two are the standing D0 baseline reds. **Five are new**, and each was checked
individually — a red for a different reason would be a regression, not
evidence. All five fail at the lowering coordinate seam refusing an admitted
producer-local emission, which is D3a's preserved refusal firing on the
population D4a exists to create:

- `control::a_discarded_visit_refuses_before_its_body_is_defined`
- `control::an_incomplete_duplicate_discarded_or_misobserved_visit_rejects`
- `control::erasing_a_seat_key_axis_or_collapsing_the_contract_rejects`
- `control::governed_nested_brackets_n3_through_n7_emit_complete_functionized_bundles`
- `control::rt_scale_b_governed_n3_through_n7_collect_every_d2_metric`

⚠ Note for D3b: the refusal fires at the **coordinate** match, not the
availability match. Lowering never reaches the availability domain for these
inputs today, so D3b must teach the coordinate seam first.

## The D2 sentinel row, inverted rather than deleted

`contsrc_d2_a_producer_local_environment_declines_the_candidate_not_the_program`
asserted that no interned specialization names a producer-local coordinate.
D4a is the event its promise class named, so the assertion is **inverted**, as
`contsrc_d4a_a_producer_local_environment_is_admitted_and_r_still_declines` —
keeping the transition measured rather than leaving a gap where a law used to
be. At least one interned specialization must name a producer-local
coordinate, asserted against a nonzero interned-input total so it cannot hold
vacuously.

⛔ It deliberately does **not** assert `R`'s decline. This fixture's
environment is fully closed — that is precisely why it is admitted — so nothing
in it reaches either decline clause, and a locally-constructed `Open` value
matched on locally would be a tautology about the enum rather than a
measurement of the take-loop. The corpus partition `interned = V` /
`declined = R` over all 83 instances is D4b's deliverable and needs the census
harness.

## What D4a does not do

No D3b lowering consumption. The pending seam is retained. No selector,
coordinate, ABI field or fallback added. Nothing inferred from depth-zero
correspondence. No claim of the final partition.

## Held

D3b, D4b, candidate, QA of the node, D6 closure, AC-4, `#27`/case-emission,
the call-result SCC and downstream D7.
