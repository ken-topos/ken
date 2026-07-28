# `RT-FNSPLIT-B2F` `S6` — the switch-over, built and measured

**Author:** `runtime-implementer` · **Base:**
`wp/RT-FNSPLIT-B2F-functionization-live` at `fc32e15d` ·
**Suite at that base:** `ken-runtime` 498 + 26 + 14, zero failures.

⛔ **This records an experiment that is NOT on the branch.** The switch-over was
built, run, classified, and then removed; what landed is only the part that is
sound and consumed today (`ArtifactHelpers`). ⭐ **The reason to write this down
is that the experiment's result is the deliverable** — it converts *"S6 is next"*
into a measured statement about what S6 is blocked on, and by how much.

## What was built

`define_unit_body` constructing its **own** `Lowering` state over its **own**
`Function`, exactly as scoped:

- `plan_unit_emissions` projects `emittable_units()` into owned per-unit
  requests, which releases the plan's borrow of `Lowering` so the emission loop
  can take `&mut compiler`. ⚠ A projection, not a second table: every field is
  read off one `EmittableUnit`, and the `AbiSlot` run itself is not copied.
- `ArtifactHelpers::declare_in_func` re-resolves every module-level identity into
  the unit's `Function`; the result is swapped into `compiler.function_local` for
  the duration of the body and swapped back after.
- The body's de Bruijn environment is built **solely** from the unit's declared
  `Parameter` and `Capture` slots, loaded at `B2R`'s offsets as `Carried`
  boundary words, in the slot run's own order. ⛔ The caller's environment is
  **not** appended — that omission *is* the switch-over.
- The body is resolved through `retained_body_occurrence(unit.origin())`, the
  backend's sole `origin -> expression` route, so `AC-4`'s route count stays 1.
- The result crosses through `transfer_into_carrier` and is written to the
  declared `Result` slot and returned.

Nothing called the units: the root still lowered the whole tree inline. So this
measured **whether a unit body can be emitted standalone at all**, which is the
first question S6 has to answer and the cheapest one to get wrong by argument.

## Measurement 1 — one call parameter

`ken-runtime --lib`: **397 passed, 101 failed.**

| n | class |
|---|---|
| 50 | `"…has no invocation arena"` — `PrimitiveCall`, `BoundaryCarrier`, `RuntimeValue::Int` |
| 23 | `"process effect lowering owns an invocation pointer"` |
| 5 | `Var: no runtime binding for index N` |
| 7 | `PlannerInvariant("static origin has no atom of that kind at that occurrence")` |
| 4 | `Trap: protocol machinery is never a source value at a boundary` |
| 2 | pre-existing source-text pins reacting to a visibility widening |
| 11 | unclassified at this pass |

⭐ **78 of 101 were one cause.** `unit_signature` declared `fn(frame_ptr) -> i64`.
The int arena and the process invocation pointer are **dataflow results, not
identities** — `FunctionLocalRefs`' two `ir::Value` fields — so a unit handed no
invocation pointer cannot derive them, and every exact-`Int` operation, every
boundary-carrier allocation and every process effect fails closed.

## The fix that was in scope, and why it was not an escalation

A **second fixed parameter** carrying the invocation context.

⛔ **This does not widen `B2R`'s frame contract**, which is why it did not need a
ruling: `AbiSlotKind`, `AbiCarrier`, `CONVENTION_SLOTS` and every declared width
are untouched. The invocation pointer is **host activation context, not a Ken
value crossing a boundary** — routing it through a frame slot would subject it to
the transfer-admissibility rules (`AC-11`) that it must *not* be subject to.

⚠ And the guarantee `unit_signature`'s own doc protects is intact: the signature
still takes **no program-derived parameter**. Both parameters are fixed for every
unit in every artifact, so making a unit's signature vary with the program still
requires a visible change at that one site.

## Measurement 2 — two call parameters

`ken-runtime --lib`: **404 passed, 94 failed.** Both invocation classes are gone.
The residue re-classifies onto a single dominant cause:

| n | class | owner |
|---|---|---|
| **69** | **the boundary-carrier producer is incomplete** | ⛔ `RT-FNSPLIT-C1` |
| 9 | `Var: no runtime binding` — free variable / ambient env | `B2R` + `S7` |
| 7 | double consumption of per-occurrence records | ⭐ `S7` |
| 2 | source-text pins vs the visibility widening | maintenance |
| 7 | unclassified | — |

## ⛔⛔ Finding 1 — `S6` is blocked on `C1`, and the blocker says so itself

**73% of the residue is one gap, and it is named in its own error text:**

> `the carrier producer does not yet emit a spillable immediate: its disposition
> carries `spill: Some(_)`, which needs a runtime magnitude test and a two-way
> branch, not a single `make_immediate``

> `the carrier producer does not yet emit a byte-bodied handle: the content needs
> the `store_bytes_len` / `store_byte` sequence, which is a distinct claim-then-fill
> protocol`

Classes with no producer today: `Int`-with-spill, `Bytes`, `ProcessExitStatus`,
`HostResult`, `Trap`.

⭐ **Why functionization is what exposes this, when the inliner never did.** Under
the inliner a closure body's result never crosses anything — it stays a
compile-time `Lowered` in the caller's own dataflow. A **unit's** result crosses a
function boundary, so it must be produced into a carrier. ⇒ `S6` converts every
result of every unit into a boundary transfer, and the producer's coverage stops
being a `C1`-internal completeness question and becomes `B2F`'s critical path.

⛔ **This is not a `B2F` implementation choice to route around.** `§2h` ¶4 makes
`boundary_disposition` the **sole** authority for how a value is represented. A
short cut — *"a single-word `Lowered` may return its own word without a
carrier"* — would be a second representation authority deciding the same
question, which is exactly the defect this program has spent nodes removing.

### ⚠ The one fork worth naming before anyone budgets `C1` work

`AbiCarrier::ResultWord` is documented only as *"the activation's single result
word."* It is **not** stated to be a boundary carrier. So there are two readings,
and they differ by ~69 tests:

- **(i) a unit result is a boundary transfer** — what was built. Blocks on `C1`'s
  producer for five value classes.
- **(ii) a unit result is a `ResultWord`** whose interpretation is static, because
  caller and callee are generated from the same static plan and the caller
  already knows statically what the callee returns.

⛔ **I did not choose (ii)**, because choosing it *is* deciding a representation
question, and that authority is `§2h` ¶4's. ⇒ Routed as a fork, not resolved.

## ⭐ Finding 2 — `S6` and `S7` are ONE edit. Measured, not forecast

The 7 `PlannerInvariant("static origin has no atom of that kind at that
occurrence")` failures are the decisive ones. The plan's per-occurrence atom
records are **single-consumption**. Emitting a body as a unit *while the root
still inlines that same body* consumes each record twice, and the second consumer
finds it gone.

⇒ **The root's recursive descent cannot survive alongside real unit bodies**, so
`D6`'s removal is not a follow-on to the switch-over — it is the same edit. The
forecast in the last handoff (*"`D6`'s removal may fall out of `S6`'s prerequisite
rather than follow it"*) is now measured, and it fell that way.

⛔ **Which triggers the work order's carry-over 4 unconditionally:** `AC-6`'s
removal pin and `AC-11` clause 3's invariant must both be in place **before** the
combined edit lands. ⚠ A pin authored after a removal cannot witness it, and the
tests a ban reddens on introduction never contain its witness — they exercise the
success path.

## Finding 3 — the ambient process environment has no declared carrier

9 `Var: no runtime binding for index N`. In process mode the root seeds its
environment with two entries before any body binding — the borrowed process input
and the capability token — and `B2R` declares **no slot for either**. A body that
reaches them resolves them today through the caller's appended environment, which
a unit does not have.

⚠ Stated as a partition, not an example: this class also contains genuinely free
variables reaching past a unit's declared slots for reasons unrelated to process
mode. ⛔ **It is not claimed that the process-ambient pair is the whole class** —
it is the part that was traced.

## Finding 4 — the `EmittableUnit::origin` dead-code oracle is now spent

Consuming `origin()` cleared its `dead_code` warning, exactly as the work order
predicted. ⭐ That warning could witness *"nobody consumes this"*; it can never
witness *"exactly one route consumes it."* ⇒ When the switch-over is re-applied,
`AC-4`'s route-count-stays-1 needs a real pin **in that same commit**.

⚠ The existing `exactly_one_plan_origin_to_expression_lookup_exists` does not
cover it: it constrains the identifier `source_occurrence` in one file and says
nothing about who may call `retained_body_occurrence` — whose visibility the
switch-over widened from private-to-`core` to all of `lowering`.

**The replacement designed but not yet built** (behavioural, not source-text): a
`#[cfg(test)]` differential between resolutions performed by
`StaticTransitionPlan::source_occurrence` and invocations of
`retained_body_occurrence` across one compile. Equal and non-zero ⇒ every
resolution went through the single route. Its positive control is that the
counters must both move on the fixture, and its compile-preserving evasion is the
mutation *"route the unit body through `plan.source_occurrence` directly"*, which
must make them diverge.

## What landed on the branch, and what did not

**Landed:** `ArtifactHelpers` + `declare_in_func`, replacing twenty inline
`declare_*_in_func` calls at the root with one operation. It has exactly one
caller today, and the doc comment says so rather than implying a population.

**Not landed:** everything else above. ⛔ The suite is green at 498 + 26 + 14 on
purpose — a red baseline makes every subsequent mutation experiment
uninterpretable in both directions, and the remaining `S6` work is mutation-heavy.

## Reproduction

From the seat's own worktree, against this document's base:

```
scripts/ken-cargo test -p ken-runtime --lib
```

⚠ The experimental patch is **not** in the tree. Re-deriving it means rebuilding
the four pieces listed under *What was built*; the counts above are only
comparable against a base whose own run is 498 + 26 + 14 green.

## ⭐ ADDENDUM — the 69 split by producer arm, and one class is not a producer gap

Re-derived from the same measurement-2 run, keyed on the producer's own refusal
text rather than on the test name:

| n | producer arm | disposition |
|---|---|---|
| **63** | **spillable immediate** — `Int`, `ProcessExitStatus`, `BoundedNat`, `StructuralNat` | `RepresentedImmediate { spill: Some(Int) }` |
| 4 | `Trap` | ⛔ **`ProtocolOnly` — see below** |
| 1 | `HostResult` handle | `RepresentedHandle` |
| 1 | byte-bodied handle — `String`, `Bytes` | `RepresentedHandle` |

⭐ **One arm is worth 63 of the 69.** The spillable immediate is not one of five
comparable pieces of work; it is the work. Its shape is already stated by the
refusal it raises: a runtime magnitude test and a two-way branch — the immediate
field when the payload fits, a `BoundaryClass::Int` handle when it does not —
never a bare `make_immediate`, which would silently truncate exactly the values
a bignum language exists to carry.

## ⛔⛔ `Trap` cannot be added to the producer, and those four reds are MY defect

The extension list names `Trap`. ⛔ **It is not extendable there**, and the
reason is structural rather than an ordering preference:

- `LoweredVariant::Trap`'s disposition is **`ProtocolOnly`** — *"a trap is
  written to the activation's trap word, which is a protocol carrier and not a
  source-expression result"* — and `boundary_disposition` is the **sole**
  representation authority (`§2h` ¶4).
- The producer's `Trap` arm sits in its **"REFUSED, not deferred"** section, not
  among the deferrals. ⚠ Those two sections mean different things and the arm
  says which it is.
- `B2R` gives a trap its own carrier and its own slot — `AbiSlotKind::Trap` /
  `AbiCarrier::TrapWord` — so a trap already has a declared lane that is not the
  result lane. ⭐ This is the same statement as `B2F`'s own `AC-11` correction
  that `result_carrier` is **not** the trap's producer.

⇒ Extending the producer for `Trap` would mean **re-deciding that a trap is a
source-expression result**, which contradicts three landed contracts at once.

✅ **The correct repair is in the switch-over, not in the producer.** Those four
reds come from `carry_unit_result` routing a trapping body through
`transfer_into_carrier`. A trapping unit body must instead write the declared
**`Trap`** slot and report through the declared **`Control`** slot — both of
which `B2R` already lays out and which the experimental emission ignored.

⇒ ⭐ **The producer extension is FOUR classes, not five**, and one of the five
was a defect in this node's own code wearing a dependency's clothing.

## ⭐ THE SPILLABLE-IMMEDIATE DISPATCH — the predicate already exists

Design for the 63-red arm, grounded rather than invented.

`ken_boundary_make_immediate_local` **already performs the magnitude test**, and
it already reports the answer distinguishably. Its own source says why the two
refusals are separate:

> *"A `Bool` that is not a bit is the wrong SHAPE; a magnitude past the field is
> out of BOUNDS. Distinct errors, so a control can tell which rule refused
> without reading the payload back."*

⇒ `BOUNDARY_ERR_BOUNDS` (`-3`) is returned **exactly** when a payload does not
fit the immediate field, computed from the one `BOUNDARY_IMMEDIATE_DOMAIN`
table.

⛔⛔ **So the producer must branch on that status, NOT on a magnitude test of its
own.** A shift-and-compare written at the emission site would be a **second
answer to a question that already has one**, and it would drift from the table
silently — the same defect class as a second representation authority, one layer
down. ⚠ The existing comment on `carrier_identity_immediate` makes the identical
point about `pack_identity`: *"a view over that one encoding, never a sibling
beside it."*

**The dispatch, as three outcomes rather than two:**

| status from `make_immediate` | emitted path |
|---|---|
| `BOUNDARY_OK` | use the returned word — today's behaviour, unchanged |
| `BOUNDARY_ERR_BOUNDS` | ⭐ **the spill** — allocate a `BoundaryClass::Int` handle and fill it |
| anything else | ⛔ trap — fail closed, exactly as today |

⚠ **Note what this changes about the existing emitter.** `emit_carrier_immediate`
currently ends in `require_i64(status, BOUNDARY_OK)`, which traps on *every*
non-OK status. The spill path replaces that unconditional assertion with the
three-way disposition above. ⛔ It must stay a **three**-way disposition:
collapsing "anything else" into the spill path would turn a shape error, a tag
error and a capacity error into a silent bignum allocation.

⛔⛔ **And the guard being replaced is load-bearing.** `carrier_immediate_tag`
refuses `spill: Some(_)` today, and that refusal is the only thing standing
between a spillable value and a truncating `make_immediate`. ⇒ It must be
**replaced by the dispatch, not deleted ahead of it**. A removed guard with a
half-built dispatch behind it is an unsound *accept*, not a stuck fallback —
and the values it would silently truncate are precisely the ones an
arbitrary-precision `Int` exists to carry.

**Controls this needs, none of which exist yet:**

1. a value that **fits** and one that **does not**, both transferred, both read
   back — ⛔ the second is the whole point and is the one a fits-only fixture
   would miss;
2. a **positive control** that the spill path is actually taken: the two
   fixtures must produce *different* emitted shapes, not merely the same answer;
3. a mutation replacing the status branch with a hand-written magnitude test,
   which must **still pass** the round-trip and so is caught by review rather
   than by the suite — ⚠ recorded here because that is the evasion this design
   is chosen to prevent, and it is **not** claimed to be mechanically detected.

## ✅ THE DISPATCH AS BUILT — and what its mutations measured

Landed at `eaf37513`+; `ken-runtime` **504 + 26 + 14** green. The design above
survived contact with the code; three things it did **not** anticipate are
recorded here rather than absorbed.

### The three-outcome shape, spelled so it cannot become two

```rust
let status = call make_immediate(tag, payload, out);
let fits   = icmp_eq(status, BOUNDARY_OK);
brif fits -> immediate_block, spill_block

immediate_block: word = load(out);              jump join(word)
spill_block:     require_i64(status, BOUNDARY_ERR_BOUNDS);   // ⭐ outcome 3
                 alloc(PersistentGround, spill_class, 0)
                 store_scalar(payload); store_int_tag(marker); jump join(word)
```

⭐ The third outcome is a **requirement on the not-OK edge**, not an `else`.
Written that way it cannot be reduced to a two-way branch by an edit that looks
like a simplification.

### ⚠ FINDING 5 — the third outcome is unreachable, by two tables

**A mutation deleting `require_i64(status, BOUNDARY_ERR_BOUNDS)` leaves every
behavioural row green.** ⛔ The tempting reading — *"the controls are weak"*
— is wrong, and the correct one is not visible from the dispatch alone:

- `make_immediate` answers `ERR_SHAPE` in exactly two situations: a **handle**
  tag, and a payload outside a **`Bit`** domain. Every other refusal is
  `ERR_BOUNDS`.
- The dispatch is reached only with a tag from a
  `RepresentedImmediate { spill: Some(_) }` disposition, and no such tag carries
  the `Bit` domain.

⇒ On this path `make_immediate` can answer only `OK` or `ERR_BOUNDS`. **No
fixture can drive the third arm without first changing one of those tables.**

✅ So the arm is pinned at its **premise** instead of by a fixture:
`b2f_d9_no_spillable_tag_can_make_the_immediate_producer_answer_shape`
quantifies over `LoweredVariant::ALL` and reads both authorities. ⛔ Its purpose
is to stop the backstop being deleted on the grounds that *"no test covers it"*
— the day the premise breaks, that test reddens and the branch needs a fixture.
Its own positive control is measured: marking the `Bit`-domain variant spillable
reddens it.

### ⚠ FINDING 6 — the spill arm covers a `Small`-marked magnitude only

`store_int_tag` admits a `NATIVE_INT_BIG_TAG_V1` marker only on a node the
**invocation arena** owns (`BOUNDARY_INT_MARKER_OWNER`), because that payload is
a slot in the invocation's `NativeIntArenaV1`. The spill target is
`PersistentGround` — the tag the ABI's own `ImmediateInt` doc names as the
overflow representation — whose owner is the persistent store.

⇒ A **region-limbed** magnitude would have to be *copied* into the node's own
region (`store_int_limbs` / `store_int_limb` / `seal_int`), which this arm does
not emit. Such a value **fails closed at `store_int_tag`'s own owner guard** —
⛔ but it is reported as `ERR_ESCAPE`, so a reader who meets it is told the
wrong thing about why. ⚠ **A residual, not coverage.**

### ⚠ FINDING 7 — the guard was replaced by a MIRROR, not by deletion

`carrier_immediate_tag`'s `spill: Some(_)` refusal **stays**. It did not become
unnecessary when the dispatch landed; it moved. `carrier_spillable_disposition`
is its mirror — refusing `spill: None` — and between them the two readers
partition `RepresentedImmediate`, so neither path is reachable for a value the
sole authority classified the other way. ⛔ Deleting the old arm would not
reintroduce a truncation *today*, which is exactly why it must stay: the next
`RepresentedImmediate` variant is added by someone copying the `Bool` arm.

### The controls, and what each mutation showed

⚠ All four rows drive **one compiled body with the payload as a run-time
parameter**. Two separate compilations cannot distinguish the dispatch from the
JIT-time specialization `AC-2` forbids — a body specializing on its constant
would produce the same two answers.

| mutation, at its production site | outcome | what it establishes |
|---|---|---|
| M1 delete the third-outcome requirement | **green** | Finding 5 — unreachable, pinned at its premise instead |
| M2 always take the immediate arm | red ×2 | rows 2 and 3 measure the spill, not merely "no error" |
| M3 always take the spill arm | red ×2 | row 1 measures the immediate arm |
| M4 spill stores a constant, not the payload | red ×2 | ⭐ the **operand** edge, not only the metadata |
| M5 record the wrong `NativeIntV1` marker | red ×2 | the spill records *how* the word is read |
| M6 allocate the spill with an undeclared class | red ×2 | the class comes from the disposition |
| M7 make the `Bit`-domain variant spillable | red ×1 | positive control for the premise pin |

⛔ **M4 is the row that would have been easiest to leave unattempted.** M2, M3
and M6 all redden through the *tag* of the returned word — bookkeeping the
spill arm writes on its way past. Only M4 substitutes the value itself, and
without it *"the spill carries the magnitude"* would have been a claim resting
on controls that never touched the magnitude.

⚠ **Unchanged from the design: the status-branch evasion is review-caught.**
Swapping the branch for a hand-written magnitude test still round-trips every
value, so nothing in this suite reddens. ⛔ Its absence from a green run is not
evidence about it.

## ✅ THE BYTE-BODIED HANDLE — built, and its class is the discriminator

Landed at `5f94d00e`; `ken-runtime` **506 + 26 + 14** green.

One emitter, driven with the class the disposition supplies. ⛔ Not two arms
with two bodies: `store_bytes_len` and `store_byte` **guard on the class**, so
it is the one path `String` and `Bytes` do not share, and a `Bytes`-only fixture
leaves `String`'s guard arm unreached. That is not hypothetical —
`boundary_value_clif` records a `class_guard` narrowed to `Bytes` alone staying
green because no test had ever asked emitted code to *build* a `String`.

| mutation | outcome | what it establishes |
|---|---|---|
| M8 write the length into every byte slot | red ×2 | the **content** edge, not the node's existence |
| M9 fill the span in reverse | red ×2 | order, not merely multiset |
| M10 hardcode the `Bytes` class for `String` | **red ×1 — the `String` row only** | ⭐ the `String` row genuinely adds the class axis |

⭐ **M10 is the row that justifies having two tests.** It reddens the `String`
row and leaves the `Bytes` row green — which is the precise claim *"these two
differ only in the axis the helpers guard on"*, demonstrated rather than
asserted.

⚠ Residual, on the emitter: the content is a **compile-time literal**. No
`Lowered` variant carries a runtime-computed byte body today, so ⛔ this is not
coverage of the byte-bodied class in general.

## ⛔ BLOCKED — `HostResult` cannot be built without a representation ruling

The last of the three producer mechanisms. ⛔ **I am not building it**, and the
obstruction is architectural rather than mechanical.

`Lowered::HostResult` is **synthesized by the effect lowering** from a host
reply (`core.rs:6828`) — its `ok` and `error` children are *constructed*, not
source subexpressions. In the `FsWriteAt` path, `ok` is a
`Lowered::Constructor` two levels deep.

⇒ The producer's recursion needs a `child_static_origin(origin, position)` for
each child, and a `Constructor` child then needs
`constructor_symbol_identity(child_origin)`. **A synthesized value has no source
occurrence, so there is no origin to ask at.** ⚠ The identity itself is
occurrence-*independent* — equal spellings intern to one canonical span — but
the **lookup is occurrence-keyed**, and asking at an occurrence that holds a
different atom is the hard `PlannerInvariant` failure this node already measured
seven of.

⛔ And the planner surface does not widen — that is ruled.

**A second, separate question in the same arm:** the ABI's `HostResult` node
records a discriminant plus two payload words (`store_scalar` / `store_field`
0=ok, 1=err). It has **no slot for `ok_constructor` / `err_constructor`**. So
either those identities live in the child words themselves, or a consumer
re-wraps from somewhere else. ⛔ `boundary_disposition` is the sole
representation authority and does not answer this; I will not decide it at an
emission site.

⇒ Escalated. It is **1 red of 69**, and it is the same lane as the region-limbed
`Int` question already routed. ⛔ Do not read the other two mechanisms landing
as `D9` complete.

## ⛔⛔ FINDING 6 IS RETRACTED — it was a false residual over a real corruption

⚠ **Read this instead of Finding 6 above.** Finding 6 claimed a region-limbed
`Int` *"fails closed at `store_int_tag`'s own owner guard."* **It never reaches
that guard.** The Architect found the reason (`evt_79xcj70p0qxjj`), and the
truth is worse than the residual it replaced.

`Lowered::Int`'s `value` is the **payload half of a `NativeIntV1` pair**, and
what that word *means* depends on the marker:

| marker | the payload word is |
|---|---|
| `NATIVE_INT_SMALL_TAG_V1` | the magnitude |
| `NATIVE_INT_BIG_TAG_V1` | ⛔ a **slot identity** in the invocation's native arena |

Slots begin at `1`. ⇒ Calling `make_immediate(ImmediateInt, payload)` on a `Big`
asks a **magnitude question about a slot number**, and a low slot *satisfies*
the immediate domain. The value crossed on the **apparent-success arm**, encoded
as the integer `1`.

⭐ **So the uncovered case was never a fail-closed residual.** It was a wrong
answer wearing the shape of a right one, and my residual paragraph is what would
have kept anyone from looking — ⚠ exactly the failure mode that paragraph exists
to avoid. **I wrote a residual describing the guard I expected to fire, without
tracing the path the value actually takes.**

## ✅ THE MARKER PARTITION, and the owned deep copy

The `Int` path now branches on the **canonical transport tag** before any
magnitude question. ⛔ Not a sibling magnitude predicate — within the `Small` arm
the ruled status-derived dispatch is untouched.

| marker | path |
|---|---|
| `Small` | the payload **is** the magnitude → the status dispatch, unchanged |
| `Big` | resolve, then an **owned deep copy** into `PersistentGround` |
| anything else | ⛔ fail closed |

The wide arm is: **allocate → region marker → claim → copy → seal**, writing
`BOUNDARY_INT_REGION_LIMBS` and ⛔ never the native `Big` marker, which names
storage that dies with the invocation. Because the copy is *owned*, no borrow
escapes and `ERR_ESCAPE` is not a terminal result for a valid value.

⭐ **The decode is `ken_native_int_resolve_local`'s.** It already yields
canonical `sign`, `len` and `limbs`; deriving them here would be a second
exact-integer decoder beside the first.

### ⚠ FINDING 8 — the carrier's arena source is conflated in production

The native arena for the decode is read from the **boundary** arena's own
binding slot (`ARENA_NATIVE_INT`), because `int_sign` / `int_len` / `int_limb`
decode with exactly that pointer — so producer and consumer agree **by
construction**.

⚠ **That choice also surfaced a pre-existing conflation, reported rather than
encoded.** `Lowering::carrier_arena()` returns `function_local.native_int_arena`
and its doc asserts the two are one pointer *"as a fact about the ABI"*. They are
not: `compiled.rs` passes a **`NativeIntArenaV1`** as parameter 0, and in process
mode the field is `invocation[24]` — the native arena either way. ⇒ Handing that
to the boundary allocator is wrong, and it has never fired only because the
carrier is inert. ⛔ Not fixed here: which pointer the carrier helpers take is an
ABI question, not an emission-site choice.

### The closing controls, and the five required mutations

⛔ **A synthetic `(Big, large_payload)` pair would not do** — it takes the bounds
edge and misses the low-slot path entirely. The pair here is minted by
`ken_native_int_intern_local` from **run-time limbs**, exactly as production
mints one. ⭐ And because `intern` trims leading zero limbs, `(x, 0)` returns
`Small` and `(x, 1)` returns `Big` **from the same call** — so one compiled body
answers both ways on a runtime operand, which is the marker partition as `AC-2`
requires it.

| required mutation | outcome |
|---|---|
| R1 pass the native `Big` slot to `make_immediate` | red — on the assertion naming the corruption |
| R2 persist the `Big` marker instead of region limbs | red |
| R3 substitute one copied limb | red |
| R4 change the sign | red |
| R5 omit `seal_int` | red |

⚠ Every `Small` / immediate / spill row stayed green under all five.

⭐ **R5 changed the test, not just confirmed it.** It first reddened the *limb*
assertion — `node_limbs` returns `None` for an unsealed node — so an omitted
seal was reported as a dropped limb: a true failure under a message naming the
wrong cause. The seal now has its **own** assertion, ahead of the limbs.
