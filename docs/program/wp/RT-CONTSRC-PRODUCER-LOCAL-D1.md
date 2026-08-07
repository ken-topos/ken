# `RT-CONTSRC-PRODUCER-LOCAL` `D1` — the two coordinate domains, separated

**What landed:** the source coordinate carried by a continuation input is now a
**closed sum** over two coordinate *spaces* rather than an entry-ABI triple
inlined into two records. The entry space is unchanged in content and meaning;
the producer-local space is representable for the first time.

⛔ Not a fourth `ContinuationInputSource` arm. That enum still labels provenance
*inside* the entry space, which is exactly what the Architect ruled at
`evt_75k8cydbj5127`: appending a case to an enum whose enclosing record still
requires an entry-ABI coordinate yields a truthful provenance label sitting
beside an untruthful `source_abi_position`.

## The representation

`ContinuationSourceCoordinate` (`static_transition.rs`):

- **`EntryAbi { source_owner, source_abi_position, source }`** — the three
  fields that were previously inline on `ContinuationSourceSlotAuthority`,
  `ContinuationInputProjection` and `ContinuationInputView`. Same components,
  same derivation, same slot-derived contract read off the exact `AbiSlot` by
  `continuation_owner_entry_sources`.
- **`ProducerLocal { binding, locator }`** — the new domain, with the three
  components the recut names:
  - `ProducerLocalBinding { binding_owner, binding_origin, binding_ordinal }` —
    the **exact structural binding identity**. The occurrence that introduces
    the value, plus which binding of that occurrence it is: a `Match` case
    introduces several at once, so an origin alone would name the set rather
    than the value.
  - `ProducerLocalLocator { environment_origin, environment_index }` — the
    **emission-time locator** into the environment that actually contains the
    binding. Its own type, not a second `u32` beside an ABI position, so that
    no consumer can read one space's number as the other's.
  - carrier / ownership / storage-owner / referent-affinity stay **beside** the
    coordinate on the enclosing record, where they already were. They are
    slot-derived for the entry arm and planner-derived for the local one; that
    is a difference in how they are obtained, not in what they mean, so
    duplicating them inside the arm would create a second authority for the
    same fact.

⛔ `D2` (which binding kinds are covered) and `D4` (broad admission) are not in
this deliverable. **Nothing in production constructs the `ProducerLocal` arm
yet.** That is the intended `D1` end state, not an omission.

## Every consumer, and what it does with the local domain

`D3` will teach these to assign the producer-local domain. Until it does, each
one **refuses**. ⛔ A refusal is the opposite of the exemption the node bans:
the banned thing is a consumer that skips the check, and these fail closed.

The full list is `grep`-able by the name of the seam function —
`entry_abi_pending_producer_local`, which exists to be enumerated and to be
deleted when the list empties:

| consumer | file | what it needs from the coordinate |
|---|---|---|
| `validate_continuation_source_slot` | `static_transition.rs` | the exact entry slot to re-derive and compare |
| `exact_continuation_projection`, `RootIsImmediate` | `static_transition.rs` | the entry position, which *is* the immediate slot there |
| `exact_continuation_projection`, `GeneratedContext` | `static_transition.rs` | **no refusal** — it now compares the **full** coordinate |
| `ContinuationContextView::captures` agreement | `static_transition.rs` | the entry owner recorded on the ABI plane |
| `ContinuationUnitView::continuation_inputs` agreement | `static_transition.rs` | the entry owner recorded on the ABI plane |
| `append_continuation_descriptor` | `static_transition/abi.rs` | the entry owner to record |
| `append_continuation_context_descriptor` | `static_transition/abi.rs` | the entry owner to record |
| `AbiPlane::validate_continuation_specializations` | `static_transition/abi.rs` | the entry owner to agree with |
| the specialization emission seam | `lowering/core.rs` | an index into the emitting environment |
| the generated-context capture seam | `lowering/core.rs` | an index into the emitting ABI operand run |

**The generated-context capture lookup is the one that changed rather than
refused.** It previously matched on the `(source_owner, source_abi_position)`
pair; it now compares the whole coordinate, which is what the recut asks for and
is strictly stronger. It cannot change any existing answer: within one owner the
entry position determines the remaining component, so the added comparison is
implied by the one it replaces.

**The node's `D3` says "the three consumers". The measured count is ten.** The
seven beyond the named three are the ABI plane's three sites and the two
view-agreement checks, which are entry-domain readers the recut's list does not
mention. Recorded here because it is a sizing input for `D3`, not because
anything is blocked: every one of them is handled explicitly and none is
exempted.

## What is measured, and what is not

`AC-2` asks that the closed sum be enforced by the type rather than by
convention: a new source kind must be unable to compile until every consumer
assigns it. **That property is compile-time and no runtime assertion can
observe it.** It is carried by there being no wildcard arm at any match on the
coordinate — verifiable by reading the ten sites above, not by a test.

⛔ And it must not be tested by asserting on the source text: a test that greps
for `_ =>` in this file would go red on an unrelated edit and would be measuring
the repository rather than the software.

What the three new controls do measure:

- **`contsrc_producer_local_coordinate_is_refused_by_both_planner_consumers`** —
  presents a producer-local coordinate to `validate_continuation_source_slot`
  and to `exact_continuation_projection`; each refuses with its own message.
  ⭐ Each row carries a **positive control**: the same record with its original
  entry coordinate validates and projects. Without that, `Err` would be
  satisfied by a record malformed for an unrelated reason.
- **`contsrc_the_two_coordinate_domains_never_compare_equal`** — the behaviour
  the type buys: no entry coordinate compares equal to a producer-local one, so
  the capture lookup cannot resolve one domain as the other.
- **`contsrc_the_emission_resolver_refuses_a_producer_local_coordinate`** —
  drives a real object emission with the new `PresentProducerLocalCoordinate`
  route mutation and asserts the refusal *and* that the perturbation fired.

The mutation perturbs the seam's **input** deliberately: no plan the planner
will build reaches these arms at `D1`, so an unperturbed run would leave all
three refusals as unmeasured code — which is indistinguishable from absent code.

**Promise classes.** The first two are durable invariants. The third is a
**transition sentinel**: `D3` replaces that refusal with a real resolution and
retires the row. It is named for the boundary rather than for a count, and its
doc comment names the event that retires it.

## Baselines

Per row and per suite, not as a total, measured on this commit against the `D0`
record at `179af863`:

- `ken-cli` `rt_parity_native`: unchanged — `buffer_freeze` green, the other six
  red with the same texts `D0` recorded, including the five-and-one split
  between the `ComputationalMatch` refusal and the `AC-1` row's `Match` one.
- `ken-runtime` lib: the `D0` baseline's 718 passed / 2 failed, plus the three
  controls above, and the same two standing reds.
- `ken-elaborator` lib: 108 passed / 0 failed.
- `1c`'s interned-to-member law and its four mutation controls are untouched by
  this deliverable; the `AC-2` omission matrix's three source-component rows now
  reach into the `EntryAbi` arm and **panic** rather than no-op if they ever
  meet a producer-local coordinate, because a silent no-op there would leave the
  two keys unequal and be read as proof that the field is load-bearing.
