# RT-CONTSRC-CALLABLE-CONTRACT — the closed callable-contract arm

**`ContinuationSourceSlotAuthority` is unconditionally a value-slot contract:
carrier, ownership, storage owner, referent affinity. A recursive
induction-hypothesis binder is a compiler-only static worker with none of
those. The coordinate can name the IH; the enclosing contract has nowhere
lawful to write "static callable, no value carrier." This node splits the
contract beside the coordinate into a closed sum so that fact has a checked
home.**

**Owner:** Team Runtime. **Size:** M.
**Node:** `docs/program/issues/RT-CONTSRC-CALLABLE-CONTRACT.md`.
**Risk:** medium — it widens a production contract vocabulary, and the naive
version of that widening is already banned (see section 5).

**Authority:** Architect ruling `evt_4h2v01dc7g8s4` (2026-08-05) stating the
required component boundary, on the `D2` correction at exact `5377d2ab`. Sized
by the IH-requirement census at `evt_qttaeebtzjkt` (exact `e6d4f085`).

---

## 0. Priority, stated first so it is not mis-scheduled

**This node is off the critical path and must not be scheduled ahead of the
`RecursiveDescent` retirement campaign.** Measured at `e6d4f085`: it closes
**1 of 83 instances and 0 of the 6 failing `D0` rows.** All 17 parity edges —
the population behind those rows — are all-closed and IH-free.

It is real work on a real capability gap. It is not the thing that unblocks
anything. A reader looking for the next kickoff wants a
`RT-DECL-CLOSURE-PORT` successor — `RT-SEED-CALL-PORT`, `RT-DESCENT-RETIRE`,
`RT-CONTSPEC-LEDGER`, `NATIVE-HANDLE-CARRIER` or `PX8-ERRID-ALLOC` — not this.

⛔ **This sentence used to route the reader to `RT-UNIT-CLOSURE-CONVERT`. That
node is `closed`** (2026-08-05): its premise was measured false, nothing was
built, and nothing should be. Do not follow that pointer anywhere it survives.

## 0a. The standing perishability clause

**Treat every anchor in this frame as perishable. If a fixed input turns out
false against the landed code, say so and escalate — do not quietly build
around it.**

This frame is written by objective and acceptance for that reason. Its
current-state claims are measured at `e6d4f085`. **That anchor is now far
behind:** `RT-CONTSRC-PRODUCER-LOCAL` was recut past `D3c`/`D4b` onto a `D8`
series that has since run to `D8p`, so the distance is tens of checkpoints,
not several. **Verify every section-1 measurement against the landed code, not
against this line.**

> **Do not read the paragraph above for the predecessor's current position.**
> It is perishable under the very clause it states, and it has already been
> wrong once: it described `D8e` as open on a whole-node causal-projection hard
> stop, which was **ruled and discharged** at row 9 of that node's
> execution-order table. A reader who took it at face value would have priced
> the predecessor as possibly-not-delivering.
>
> ⇒ **The one live source is `RT-CONTSRC-PRODUCER-LOCAL`'s own execution-order
> table.** Read it at kickoff. This frame deliberately does not restate it,
> because a restatement is what went stale.

---

## 1. Base and fixed inputs

**Base:** whatever `RT-CONTSRC-PRODUCER-LOCAL` merges as. That node is
unmerged and its `D8` series has run to `D8p`. **Naming the exact base and
re-deriving the three measurements below is `D0` of this node, not this
section's claim** — which is why this section no longer enumerates what remains
ahead of the predecessor. That enumeration is what went stale in §0a; read it
from the node's execution-order table instead.

### The three measurements that establish the boundary, at `e6d4f085`

1. **No `ResultPhase` to `AbiCarrier` map exists anywhere.** The vocabularies
   are disjoint by construction — one records a representation phase, the other
   an ABI transport. `slot_referent_affinity`, the authority for which carriers
   a continuation source environment admits, accepts `ValueWord` and
   `GroundValueCarrier` and refuses the rest; nothing proves an IH is either.
2. **The IH's phase is not edge-local.** It is `carrier()` or `SPECIALIZED`
   depending on `functionized_units`, a whole-plan argument to
   `plan_static_transition_graph_with_symbols` that is **not a field of
   `StaticTransitionPlan`**. So `(case body, ordinal)` does not determine the
   IH contract, and the walk cannot reach the fact that would.
3. **Production continuation inputs have no callable domain at all.** As
   measured when this frame was written, `BoundaryUseAvail::Callable` and
   `BoundaryUseNeed::PreserveCallableIdentity` existed solely as `#[cfg(test)]`
   mutations; every production projection was `Value` / `PreserveValue`.

   > **The instance is dated; the measurement is not.** `RT-CONTSPEC-LEDGER`
   > (recut 2026-08-08, Architect `evt_1v9m7t4m9dmj7`) **deletes those two
   > enums and their two siblings outright.** Measurement 3 then holds more
   > strongly than when it was written — there is no callable domain because
   > there is no domain. Re-derive it at your base per `D0` question 1, and
   > state which world you measured in.

**Measurement 3 is the one that decides the shape of the work**, and its force
does not depend on the enums surviving. A callable arm that exists only under
`cfg(test)` — or not at all — cannot be the authority for a production
contract; that would make the test suite the source of truth for what
production may represent. See AC-4.

### The witness edge

```
prog=10/2 consumer=fn0 cont=origin10 construct=origin19 pos=0 closure=origin18
  required=2   0:OPEN[ih-binder]   1:local[case-arg]
```

Exactly one edge of 83. **Its environment going all-closed is the deliverable.**

---

## 2. The expressibility audit, run and recorded

This node is the textbook case for `steward/frame-authoring.md`
§(b-triple-prime): *where does each obligation get written, and is that place
inside the shape's own checked vocabulary?*

| obligation | home in the shape as landed | status |
|---|---|---|
| this source has carrier C, ownership O, storage S | `ContinuationSourceSlotAuthority`'s value fields | has a home |
| this source is a static callable with **no** value carrier | none **on this surface** — every field is a value field | **no home here; this is the node** |
| its lawful consumer is callee-only | none **on this surface** in production; `cfg(test)` only | **no home here** |
| calling it yields representation R | deliberately **out of scope** — see section 5 | not this node's obligation |

**The failure is an expressibility gap, not a missing enum arm.** That
distinction is load-bearing: adding an arm to an existing value-shaped
authority would give the obligation a home that still says "value," which is
the shape the Architect's ruling forbids. The contract must split.

### The "no home" cells are scoped to this surface, and that now matters

**Two production closed sums with exactly this shape landed after this frame
was written** — on `RT-CONTSRC-PRODUCER-LOCAL`'s branch at `89e36ec1`, both in
`crates/ken-runtime/src/cranelift_backend/lowering/`:

| sum | non-value arm | what it establishes |
|---|---|---|
| `LoweringEnvironmentBinding` (`mod.rs`) | `StaticWorker(StaticWorkerBinding)` — "compiler-only... never becomes a runtime value"; no word, tag, layout, vtable, descriptor or environment pointer | a lexical binder can be a static callable with no value carrier, in production |
| `SourceCallee` (`mod.rs`) | `StaticWorker { worker, static_origin }` — a sum "rather than an operand, because a static worker **is not a value**" | a callee position can distinguish the two without widening the value slot |

⇒ **The obligation this node exists to house already has a checked production
home twice over — just not on the continuation-source surface.** Measurement 3
still holds where it is stated: at `89e36ec1` the only sites setting
`BoundaryUseAvail::Callable` / `BoundaryUseNeed::PreserveCallableIdentity` were
inside `mutate_projection_field`, a `cfg(test)` projection-mutation helper.
Verified there, not inherited from `e6d4f085`. **`RT-CONTSPEC-LEDGER` deletes
that helper's boundary cases along with the enums** — which removes the setters
rather than adding one, so the measurement survives the deletion.

**Why this changes how the node is built rather than whether it is.** `D1`
says the callable arm's identity comes from "the existing planned
static-worker/member authority." That instruction now has a concrete referent
to match rather than a description to satisfy, and the `SourceCallee` rationale
is the argument for a sum over a widened slot, already made and already landed.
**Diverging from those shapes on this surface is a choice that must be argued,
not a default.**

⚠ **What this is not.** It is not evidence the gap is closed, and it is not a
licence to reuse `StaticWorkerBinding` as the continuation-source contract —
that type carries no callable identity by deliberate design, which is the
opposite of what `D1`'s callable arm must carry. **Two sums of the same shape
on adjacent surfaces is a precedent, not a component.**

---

## 3. Deliverables

### `D0` — name the base and re-derive

Record the exact SHA, branch, and the state of the three section-1
measurements. State which survived and which moved. **If measurement 3 has
changed — if a production callable domain now exists — stop and hand back;
that changes the node.**

**Measurement 3 is surface-specific, and you must test the right surface.**
Section 2's table records two production static-callable sums that landed in
`lowering/` after this frame was written. **They do not falsify measurement 3**,
which is a claim about the continuation-source projection vocabulary. So `D0`
answers two questions, separately:

1. **On the projection surface** — is there a production callable domain? Ask
   it that way, because **the vocabulary the question used to name is being
   deleted.**

   > **Amended 2026-08-08 by the Steward.** `RT-CONTSPEC-LEDGER` was recut to
   > schema retirement (Architect `evt_1v9m7t4m9dmj7`) and **deletes
   > `BoundaryUseAvail`, `BoundaryUseNeed`, and their two sibling enums
   > outright**, along with the `mutate_projection_field` cases that flip them.
   > That node does not gate this one and the two may land in either order.
   >
   > **So answer by enumerating setters of a production callable domain on the
   > continuation-source projection surface, and record which world you are
   > in:**
   >
   > - **The enums still exist** (LEDGER has not landed): the original question
   >   stands unchanged. At `89e36ec1` the sole setters sat in
   >   `mutate_projection_field`, and a comment elsewhere *restated* the
   >   property without being a setter. **That is where to look first, not what
   >   to conclude** — re-enumerate at your actual base, since the `D8` series
   >   has worked the emission and projection machinery since.
   > - **The enums are gone** (LEDGER has landed): measurement 3 holds **a
   >   fortiori** — a domain that does not exist is not production-reachable.
   >   **Record it as discharged by absence and say so explicitly.** Do not
   >   report a zero-hit grep as if you had enumerated setters; those are
   >   different claims and only one of them is true here.
   >
   > **A grep hit on a comment that asserts the measurement is not a
   > measurement** — and after the deletion, neither is a grep miss.
2. **On the lowering surface** — are the two sums still as section 2 records
   them, and did either grow a value carrier, ownership or storage field?

Report both. **Only question 1 arms the hard stop.**

### `D1` — the closed sum

Preserve the coordinate and locator from `RT-CONTSRC-PRODUCER-LOCAL`'s `D1`
unchanged. Split the **contract beside it** into a closed sum:

1. **Value source** — the existing carrier / ownership / storage /
   referent-affinity contract, **unchanged**.
2. **Static-callable source** — an exact planner-owned callable/worker identity
   and its declared parameter/capture contract, with **no** value carrier,
   **no** ownership, and **no** storage projection.

The callable arm's identity comes from the **existing planned
static-worker/member authority** — never from `ResultPhase`, never from source
syntax, never from a new lookup in lowering.

### `D2` — exhaustive distinction at every consumer

Every downstream source, projection and view consumer distinguishes the two
contracts exhaustively. **No default arm, no wildcard.** Enumerate the consumer
set as part of the deliverable; the enumeration is itself reviewable output,
not a step.

### `D3` — fail closed on value use

A callable source creates **no ordinary ABI slot**. Its lawful consumer is
callee-only, and every value-producing use fails closed.

### `D4` — the production vocabulary states it

The production need/availability vocabulary can express "static callable, no
value carrier." The `cfg(test)` mutations of measurement 3 are **not** the
authority and are not promoted in place.

### `D5` — the witness closes

The single IH edge's required vector goes all-closed. **A report that the arm
exists without that edge closing does not discharge this node.**

---

## 4. Acceptance criteria

**AC-1 (`D5`, the witness).** The IH edge's vector is all-closed, measured
through the production planner path with no alternate route. *Control:* the
same census instrument that produced the `e6d4f085` numbers, re-run at the
node's base, showing that edge moving from `OPEN[ih-binder]` to closed **and no
other edge changing verdict**. A census that moves other edges is measuring
something else.

**AC-2 (`D2`, exhaustiveness).** Every consumer distinguishes the two
contracts. *Control:* the match is exhaustive with no wildcard — **the compiler
is the enforcing mechanism here and it is the strongest one available**, so do
not specify a test for what a non-exhaustive match already refuses. What needs
a test is the consumer set being complete: for each consumer in `D2`'s
enumeration, a mutation that routes a callable source into it must be refused.
Record the result **per consumer**, not "each consumer" as a quantifier the
reader resolves.

**AC-3 (`D3`, fail closed).** Every value-producing use of a callable source
fails closed. *Control:* per value-producing site, a mutation supplying a
callable source must fail **at that site's own guard**, with a perturbation
counter confirming the mutation fired. A failure downstream of the site does
not discharge it — that measures the next guard, not this one.

**AC-4 (`D4`, the production vocabulary).** The callable domain is production,
not `cfg(test)`. *Control:* build with `cfg(test)` disabled and confirm the
production path can still represent and refuse a callable source. **If the only
witness is a test-gated enum arm, this AC is not met**, however green the
suite.

**AC-5 — the value arm is untouched.** *Control:* the value-source contract's
fields and its `slot_referent_affinity` behaviour are byte-identical to the
base, or every difference is enumerated and justified.

**AC-6 — no regression.** The base's red rows are unchanged. Workspace-green,
`--locked` and conformance are **CI's**, never a local `--workspace` run
(`agent/COORDINATION.md §12`).

**AC-7 — the residual is stated, not implied.** This node closes one of `D4`'s
three declined causes. **State plainly in the handoff that `let-value:Construct`
and `let-value:If` remain declined and are outside this contract domain.** They
are outside-this-domain residuals, not unrepresentable
(Architect, `evt_38yd5sd1ht0kk`).

---

## 5. Banned scope

- **Whole-plan result representation** — what calling the worker *returns* — is
  a separate later decision. It must not be folded into coordinate identity,
  and it must not be used to manufacture an input carrier.
- **No `ResultPhase` to `AbiCarrier` bridge.** The vocabularies are disjoint;
  minting a map between them is the unlawful shape, not the missing feature.
- **No new lookup in lowering** to recover a callable identity the planner did
  not issue.
- **Do not promote the `cfg(test)` callable arm in place** as the production
  authority.
- **Do not claim `D4` closure.** `RT-CONTSRC-PRODUCER-LOCAL`'s `D4` is stated
  as **set equality** — `interned = V`, `declined = R` — precisely because
  "every environment closed" is unreachable by any node currently in the graph,
  with or without this one. Closing the IH cause does not convert it.

---

## 6. Sequencing and contention

**Depends on:** `RT-CONTSRC-PRODUCER-LOCAL` — this node splits the contract
enclosing that node's coordinate, so it cannot precede it.

**Contention:** one branch, one team.

⛔ **This section used to sequence this node behind `RT-UNIT-CLOSURE-CONVERT`
— "the unit-closure node goes first." That node is `closed` and will never
run, so read literally the clause held this node forever.** Deleted rather
than annotated: a sequencing constraint whose predecessor cannot execute is
not a weaker constraint, it is an unsatisfiable one.

The live contention is with `RT-CONTSRC-PRODUCER-LOCAL` itself, which is the
`depends_on` edge below and is handled by waiting for its merge.

---

## 7. Hard stop

Stop and hand back, without repairing, if:

- `D0` finds measurement 3 changed **on the projection surface** — a
  production callable domain already existing there changes what this node is.
  **The two `lowering/` sums in section 2 do not trip this**; they were
  measured and are expected. Tripping it means **any production-reachable
  callable domain on the continuation-source projection surface**, by whatever
  name.

  > **Amended 2026-08-08 by the Steward, and the amendment is the point.** This
  > stop previously read *"a non-`cfg(test)` setter of
  > `BoundaryUseAvail::Callable` or `BoundaryUseNeed::PreserveCallableIdentity`."*
  > `RT-CONTSPEC-LEDGER` deletes both enums, after which **no such setter can
  > exist and the stop could never fire** — it would pass for the reason it was
  > written to detect, and read as "measured and clear" when it was
  > unmeasurable.
  >
  > **Stated by property it still fires**, on any successor vocabulary as well
  > as on the deleted one. If you discharge it by absence, say absence — see
  > `D0` question 1;
- closing the IH edge requires a `ResultPhase`-to-carrier bridge, or any of
  section 5's banned shapes;
- the consumer enumeration in `D2` cannot be shown complete — an unbounded
  consumer set makes AC-2 undischargeable and the cut is the Steward's;
- closing this edge moves any other edge's verdict.
