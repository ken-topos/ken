# `RT-CONTSRC-PRODUCER-LOCAL` `D3b` — lowering consumption

Released by `evt_3bc4c518ejrh` (Runtime leader), under the Architect's gate
`evt_65xkzqppdqdaj`. Built over QA-approved `D4a` `ac897a08`.

Commits: `1fb3e72a` (the two arms) → `573c49f9` (sentinel inversion) →
`8056fc1c` (the consumer control) → `41362630` (retiring the pending seams).

`D4b`, the candidate, QA of the node, `D6` closure, `AC-4`,
`#27`/case-emission, the call-result SCC and downstream `D7` remain held.

## The shape: the seam resolves a PAIR

Both emission seams previously matched the coordinate domain and the
availability domain **separately**, refusing every producer-local arm of each.
`D3b` replaces both with one exhaustive match over the **product**, because the
two domains are not independent: the projection builds `EntryAbi` availability
only for an entry coordinate and the two producer-local availabilities only for
a producer-local one.

Six pairings exist. Three are lawful, three are crossed.

| coordinate | availability | emitter | resolves to |
|---|---|---|---|
| `EntryAbi` | `EntryAbi` | either | `immediate_slot`, unchanged from before `D3b` |
| `ProducerLocal` | `CurrentLexical` | predeclared only | `post_shift_index` |
| `ProducerLocal` | `GeneratedContextCapture` | specialization only | `immediate_capture_slot` |
| `EntryAbi` | either producer-local arm | — | reject |
| `ProducerLocal` | `EntryAbi` | — | reject |

Matching the halves separately would let a crossed pair through whenever both
halves are individually well-formed, which is exactly the shape that reads as
safe. A crossed pair is not an unhandled case; it is the projection and the seam
disagreeing about what the value is.

### `CurrentLexical`

Requires a predeclared emitter — a specialization emitter is refused **before**
any index is produced, because a generated context lowers a raw body and does
not stand in the producer's semantic environment, so a post-shift index there
counts binders of a scope that function never entered. Then the availability
must be keyed to this exact emission: emission origin, lexical-environment
origin, producer owner.

### `GeneratedContextCapture`

Requires a specialization emitter whose context id matches, the context's raw
owner to match, **full root-coordinate membership** in that context's declared
capture run (by whole coordinate, exactly once — never by owner or position
alone, so a local binding cannot satisfy an entry position by carrying the same
integer), and the declared slot to equal `parameters + position`.

## Why a check was needed, and not merely trust

`D4a` measured both of the shifted fixture's inputs carrying `ValueWord` /
`OwnedByFrame` / `ActivationFrame` and the same referent affinity, with both
operands lowering to a `HostResult` with the same constructor pair. **Every
incidental discriminator a consumer could rely on is equal across the positions
of one seat environment.** A consumer indexing with the wrong number would
obtain a well-formed operand of exactly the right contract and emit a call
carrying the wrong value, silently.

So the `CurrentLexical` arm verifies, against the planner's own forward walk,
that the coordinate really occupies the index it is about to read. The direction
matters: the index arrives from the projection and is **checked**; deriving it
from the environment here would be the reverse map `evt_609am4v7cdt5b` forbids.

THE GAP: this re-runs the planner's own walk, so it proves the consumer indexes
with the number the planner assigned — **not** that the assignment is right.
`D2b`'s discriminator and `D3a`'s validator own that half.

## The consumer proof

`d3b_the_consumer_refuses_an_index_the_emission_seat_does_not_hold`, against
`D4a`'s shifted fixture. Two committed compile-preserving mutations:

| mutation | effect |
|---|---|
| `ConsumeLocatorIndex` | consume the locator's scope-relative introduction index — the exact defect `D2b` reopened `D2` for, now at the consumption boundary |
| `ShiftProducerLocalSlot` | move the resolved slot by one; also perturbs an emission with a single producer-local input, where no collision exists to catch it |

Both are refused **at** the seat-consistency check, with a perturbation counter
confirming each fired, and the unmutated route asserted to pass that same check.

⭐ **The mutations sit inside the resolver, ahead of the verification, not after
it.** A mutation applied to the already-resolved index slips past the very check
meant to catch it, and the row would then prove only that the index changed —
which is `D4a`'s claim about the instrument, not `D3b`'s about the consumer.
This is the Architect's boundary: the `D4a` mutation proves the instrument, the
`D3b` mutation proves the consumer.

The positive control deliberately does **not** assert a successful compile; see
the boundary below. It asserts the discriminating fact instead — that the
unmutated failure is not this check.

## A law added, and one deliberately not added

**Injectivity, within the producer-local domain only.** Two distinct
producer-local coordinates of one emission must not resolve to one position.
This is the consumption-side dual of the planner's refusal of a binding present
at two positions of the seat environment: there ambiguity is one value in two
places, here it is two values claiming one place.

⛔ **It is deliberately not stated across domains, and that is a measurement.** A
first draft compared every resolved slot and **refused five lawful bracket
fixtures**, where a parameter at ABI position 0 and a case binder at lexical
position 0 legitimately carry the same integer. An entry-ABI `immediate_slot` is
a position in the entry ABI frame and a producer-local `post_shift_index` is a
position in the lexical frame; comparing the two integers is the cross-frame
conflation this node exists to forbid.

⚠ **That draft surfaced a real question which is NOT `D3b`'s to answer and is
recorded so it is not lost.** At a predeclared emitter both kinds of index are
used against the same `producer_env`, and `D4a` established that `producer_env`
is the lexical environment. An entry-ABI input's position in it is therefore its
lexical position, which equals its ABI position only at zero binder depth. Every
pre-`D3b` population was entry-ABI, so the two never had to be told apart. I
have not measured whether a reachable program makes them differ, and I did not
change the entry-ABI path.

## Sentinels retired

- The `D1` row asserting the emission resolver refuses **every** producer-local
  coordinate is **inverted, not deleted**: its perturbation injects a
  producer-local coordinate while leaving the entry-ABI availability in place,
  so what survives is the stronger permanent law that the resolver dispatches on
  the *pairing*. Reclassified transition sentinel → durable invariant.
- `entry_abi_pending_producer_local` and its `D3a` sentinel are **deleted**:
  once both lowering arms consume, its live enumeration is empty, which is the
  condition the release named and the event that row's own promise class named.
- `entry_abi_slot_pending_producer_local` is deleted too — it had zero callers
  of any kind, including tests.

## The boundary this checkpoint reaches, stated precisely

`ken-runtime` lib: **725 passed / 7 failed / 1 ignored**. Two are the standing
`D0` reds. **Five are `D4a`'s deliberate reds, and they have moved.** They no
longer fail at the emission seam — `D3b`'s arms consume their producer-local
inputs and lowering proceeds past it. They now fail *downstream*:

```
Var: no runtime binding for index 2
  index=2  env_len=2  defining=Predeclared(PredeclaredFunctionId(3))
```

`D4a`'s own shifted fixture reaches the identical boundary at `index=3`. So this
is one uniform gap, not five: admitting producer-local continuation inputs makes
a body reachable whose lowering environment is shorter than the body's own de
Bruijn depth.

⛔ **I did not widen into it.** The release scopes `D3b` to "exactly the two
lowering arms"; this is unit-body environment sizing, a different seam, and
fixing it would mean changing how unit frames are built — which is neither of
the two arms and is not authorized here. The five reds are therefore **still
red, at a new and better-characterised place**, and I am reporting that rather
than describing the checkpoint as closed.

Per `agent/COORDINATION.md §12` the workspace build, the `--locked` gate and the
conformance suite run in CI.

## Scope held

No selector, ABI or coordinate widening, fallback, or alternate lowering route.
Two accessors were added as reads of existing planner authority:
`ContinuationContextView::parameters`, so a consumer can check a capture slot
against the run it names, and `verify_current_lexical_availability`, which is
the fail-closed check above. The existing `D2b` fixture and `D4a`'s fixture are
unchanged.
