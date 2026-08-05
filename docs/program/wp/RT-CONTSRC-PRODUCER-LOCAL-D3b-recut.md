# `RT-CONTSRC-PRODUCER-LOCAL` `D3b` (re-cut) — consumer-specific availability

Status: **partial, with one precise hard stop.** Not a candidate.

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
consumer needs the post-shift **lexical** index and the capture consumer needs
the **entry-run** position, and `D3c` measured those two numbers diverging. One
`availability` field repaired for either consumer silently mis-serves the other.

⛔ `predeclared_entry_frame_slot` returns `None` — no capture claim — when the
frame declares no member. A `ProducerLocal` coordinate is a mid-body value with
no position in any entry run, so the boundary **fails closed**; `D4b` owns making
such a value capturable.

## THE HARD STOP — a forwarded parameter occupies two lexical positions

The ruling states that at predeclared direct emission either root uses
`CurrentLexical` **only when the forward lexical walk finds the full coordinate
exactly once**. Three planner rows now fail because that precondition is
**unsatisfiable for a program class that works today**.

**Measured**, exactly:

```
coordinate = EntryAbi { source_owner: 0, source_abi_position: 1, source: Parameter }
present at lexical index 0 AND index 2, in a 3-element seat environment
```

**Cause, read off the walk rather than inferred.** `walk_continuation_value_environment`'s
`Let` arm mints a producer-local value only when the bound expression is an
`Effect`; otherwise it pushes **the bound expression's own value authority**. So
`let y = x`, forwarding a parameter, puts that parameter's identical `EntryAbi`
coordinate at both its entry position and the binder position. This is not a
double-count in the seat construction — the two are legitimately different
positions holding one **identity**, which is exactly what a root coordinate is.

The old law never met this: an entry root took its ABI position and never
walked.

**Why this is not mine to settle.** The obvious resolution is *innermost
occurrence* — index 0 is innermost, and de Bruijn lookup already designates it,
so it is semantics rather than a search convenience. Relaxing to innermost was
measured: the three rows pass and **nothing else regresses** (719 → 722). But:

- it is textually "take the first", which the release **bans**; the ban exists
  because a first match among candidates that might name *different* values is
  unsound, and whether that applies here is the ruling;
- the two positions need not hold the same **SSA value**. For `let y = x` they
  do. But the walk's `If` arm joins its branches (`then_value.join(else_value)`),
  so a binder can carry a coordinate whose value is a *different* SSA value with
  the same root identity. Indexing either still yields that identity, but not
  necessarily the same ownership or lifetime.

⚠ **NOT MEASURED:** that the two positions hold the same SSA value at lowering.
No lowering fixture in the corpus reaches an emission seat with a duplicate —
the three affected rows are planner-only — so there was no instrument to put on
it. The structural argument for `let y = x` is strong; the `If`-join case is the
one that needs the ruling.

**The concrete edge, as the release asks for it:** predeclared direct emission,
coordinate `EntryAbi { source_owner: 0, source_abi_position: 1, Parameter }`,
seat environment length 3, occurrences at indices 0 and 2.

## What is NOT built

The **two-stage generated `EntryFrame` construction** is not implemented. The
frame identity is still the provisional pair `(enclosing, worker_body_origin)`,
resolved to a `ContinuationContextId` **at the consumer** rather than stamped
into an immutable claim by a second planner phase.

⛔ This is a deferral, not a silent partial. The ruling requires finalization to
resolve the pair to exactly one `(ContinuationContextId,
ContinuationSpecializationId)`, refusing on zero or multiple, and to never expose
a half-stamped claim. Today the same zero/multiple refusal exists but fires at
first use, which means a plan carrying an unresolvable frame is **accepted by
planning** and only refused if and when something reaches it.

⚠ **A sequencing fact, measured, that bears on when this should land.** The path
the two-stage machinery governs is **planned but not consumed**. The planner
takes the generated-context emitter arm 78 times, so those specializations do
carry `EntryFrame{GeneratedContext}` claims — but across both lowering seams,
**0 of 60** observations held a `Specialization` emission owner; every consumer
held a predeclared frame. So the machinery would land **unexercised by any
behavioural test**, and its zero/multiple-match refusals could only be reached by
a directly constructed planner row. That is an argument about *validation
strength*, not about whether to build it: a guard nothing reaches is exactly the
kind that rots. Worth the Architect's attention when sequencing it against `D4b`,
which is what would make generated-context emission reachable.

The design is settled and the blast radius is small: projections keep a draft
typed with a `ContinuationFrameRequirement`; a pass after
`plan_continuation_contexts` builds the finalized views into per-unit and
per-context slices; `continuation_input_view` — the **single** conversion both
populations go through — becomes fallible and errors when the finalized entry is
absent. That gate is what makes a half-stamped claim unreachable rather than
merely unwritten.

## Suite

`ken-runtime` lib: **722 passed / 10 failed / 1 ignored**.

- **7 baseline reds, unchanged** — the two standing `D0` reds plus the five
  former `D4a` reds at their downstream `Var: no runtime binding` boundary.
- **3 reds, all one issue** — the hard stop above.

Both counts were taken against the same commit's tree; the 7 are the same seven
named in the `D3c` record at `f5e4fa9f`. The workspace build, the `--locked`
gate and conformance are CI's.
