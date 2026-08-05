# RT-CONTSRC-PRODUCER-LOCAL D2b — immediate availability

Deliverable D2b, released by `runtime-leader` at `evt_2d39r6ctx7n5t`,
transcribing the Architect's ruling `evt_44k69b55vhek2`. Built from exact
`e6d4f085f334e7b70f98243ddc2feffa9fd3ef05` on
`wp/RT-DECL-CLOSURE-PORT-typed-units`.

## The defect this corrects

`D1` promised "an exact emission-time locator into the environment that
actually contains it" and never said **which** environment. `D2` read that as
the semantic environment and populated a scope-relative
`(environment_origin, environment_index)`. The emission seam indexes a
different space — an ABI operand run. One load-bearing term spanned two
coordinate spaces, so `D2` could not be discharged as written.

The correction is not a new coordinate. Root identity
(`ProducerLocalBinding` plus its value contract) is preserved untouched and is
never rewritten as an ABI position. What is added is a **second, separate**
fact: where the emitting seat can reach the value right now.

## What landed

`ContinuationInputProjection.immediate_slot: u32` becomes
`availability: ContinuationImmediateAvailability`, a closed sum with no
wildcard at any consumer:

| Arm | Environment it names |
|---|---|
| `EntryAbi { immediate_slot }` | An ABI operand run. The entire pre-`D2b` population, carrying the identical number the bare field always held. |
| `CurrentLexical { emission_origin, lexical_environment_origin, post_shift_index }` | The semantic environment in force at one exact emission seat. |
| `GeneratedContextCapture { context, owner, immediate_capture_slot }` | A generated context's declared capture run. |

Both existing resolutions — root owner's entry run, and generated-context
capture lookup for an entry-ABI coordinate — land on `EntryAbi` unchanged, so
Entry ABI availability is untouched exactly as the release requires.

**Arm 1 needs no new authority.** `post_shift_index` is derived by pointing the
existing forward semantic-environment walk
(`walk_continuation_value_environment`) at the emission occurrence instead of
the continuation occurrence. The walk already returns the environment it holds
when it reaches its target, with every intervening `Let`, `Match` case and
`ComputationalMatch` case binder already pushed — which is precisely what
"post-shift" names. The binder-push rules are read off the walk rather than
restated.

**Arm 2's precondition is checkable.** The full root coordinate must be present
in the context's ordered capture projection (`enclosing_inputs`, which
`exact_continuation_projection` already holds and has compared on the whole
coordinate since `D1`), **and** the matched enclosing record must itself carry
a `CurrentLexical` availability — the caller's proof that the value existed at
that call seat. Without it the capture would be fabricated.

## Five fail-closed paths, each measured

| Path | Where | Mutation that reaches it |
|---|---|---|
| Wrong emission origin | `continuation_emission_seat_environment` | seat drawn from off the result edge |
| Wrong post-shift index | `current_lexical_availability` | coordinate absent from the seat environment |
| Missing full-coordinate capture membership | `GeneratedContext` arm | capture projection truncated |
| Wrong generated owner/context | `GeneratedContext` arm | resolution owner crossed with another unit's captures |
| Wrong immediate slot | `checked_add` on the capture run | `context_parameters = u32::MAX` |

Each is asserted with a positive control on the same record and a check on the
refusal's **own** message, so an `Err` arriving for an unrelated reason does not
satisfy the row.

## Consumers still refuse — `D3` is held

Both emission seams in `lowering/core.rs` now match the availability domain
exhaustively with no wildcard, refusing the two producer-local arms. That is
deliberate: `D2b` **projects** the availability, `D3` **assigns** it. Handing a
lexical environment index to an ABI operand run would name a different value.

The arms are unreachable from those seams today, because the coordinate match
above them has already refused every producer-local coordinate. The match is
written anyway — "unreachable" is a claim about the current projection, and an
emission seam must not be the place that discovers the claim was wrong.

## Discriminators

Production still declines every producer-local candidate at the `D2` gate,
*before* projection, so neither new arm is constructed by any fixture. Both
discriminators construct their arm directly, and both are mutation-proved.

**`contsrc_d2b_current_lexical_availability_counts_the_intervening_binder`.**
The host-effect result is introduced at index 0 of the `Let` body and emitted
past the enclosing `Match` case's binder, so the locator index and the
post-shift index are genuinely different numbers on that row. Two properties
make it non-vacuous: the seat is **searched for among real occurrences** rather
than hand-picked, so it must satisfy the same lawfulness check production
applies; and the row asserts the introduction index names a **different value**
at that seat, so passing the locator through is a *failing* answer rather than
an indistinguishable one.

**`contsrc_d2b_generated_context_capture_separates_root_from_immediate`.**
Introduction index 0, capture position 1, immediate slot 3 — pairwise distinct,
with a decoy ahead of the value in the capture projection so the position is
not zero.

### Mutation proofs

Run against the committed tree, one at a time, each reverted before the next.

| Mutation | Result |
|---|---|
| `current_lexical_availability` returns the `D1` locator index | Discriminator 1 RED (`post_shift_index: 0` vs `1`); discriminator 2 green |
| capture position used as the immediate slot | Discriminator 2 RED (`immediate_capture_slot: 1` vs `3`); discriminator 1 green |
| caller's current-lexical proof no longer required | Discriminator 2 RED; discriminator 1 green |
| emission-origin guard removed | Discriminator 1 RED; discriminator 2 green |

Each mutation reds exactly the discriminator that owns the property and leaves
the other green, so neither row is carrying the other's weight.

## Acceptance

`ken-runtime` lib: **726 passed / 2 failed**. The two failures are the standing
reds recorded at the `D0` baseline for this node
(`c2_ac4_runtime_host_result_selects_a_separately_generated_nested_payload`,
`two_same_shape_workers_are_distinguished`), which stood at 718 passed / 2
failed there. No new red; the +8 are `D1`/`D2`/`D2b`'s own rows.

Workspace build, `--locked` and conformance run in CI, not here.

## What this does NOT claim

- Nothing here **admits** a producer-local binding. The `D2` decline gate is
  untouched and every such candidate still declines.
- Nothing here lowers either arm. `D3` owns the emission consumers.
- The `D1` row
  `contsrc_producer_local_coordinate_is_refused_by_both_planner_consumers` was
  restated rather than deleted: `exact_continuation_projection` no longer
  refuses on the *domain*, because `D2b` gives that domain a real derivation.
  It refuses on the harder question — whether the coordinate is genuinely
  present at the emission seat — so a fabricated probe coordinate is still
  refused, now for the reason `D2b` owes.

## Held

`D3`, `D4` (`interned = V`, `declined = R` over the 83-instance census unit),
candidate, QA, `D6` closure, `AC-4`, `#27`/case-emission, the call-result SCC
and downstream `D7`.
