# `RT-FNSPLIT-B2E` — semantic boundary-value elimination

**Node:** `docs/program/issues/RT-FNSPLIT-B2E.md` · **Owner:** Runtime · **Size:**
`L` · **Sequence:** `B2O` → `B2R` → `B2V` → **`B2E`** → `B2F`

**Authority:** Architect ruling `evt_35p5ancbdmzr7`, Decision
`dec_43h1rggqxcf1a` — `resolved`, `resolved_by=agt_37reqftfe6g00`, verified from
the object. The node carries the full transcription; ⛔ **if this frame and the
node disagree, the node's ruling text wins and you say so.**

---

## ⛔ READ THIS FIRST — it decides whether you build the right thing

**`B2V` already landed the decoder.** Class, scalar, tag, field-count, field,
record-field, construction and their helpers are on `main`, emitted from a live
site. **What is missing is the semantic elimination bridge above them.**

⇒ ⛔ **`B2E` is NOT "write a boundary-value decoder."** If you find yourself
adding a second way to read a tagged word, you have inverted this node — the
ruling forbids *"a second live decoder or value taxonomy"* in as many words.
`B2E` teaches the **existing** `Lowered` consumers to eliminate an **opaque**
boundary value **through `B2V`'s interface**, and nothing else.

⚠ The hard-stop report that produced this node called the gap *"a representation
with no consumer."* **The Architect corrected that shorthand**, and the
correction is why this paragraph is at the top: the short version under-describes
what exists and over-describes the gap. ⛔ Do not carry the short version forward
into your own notes or your handoff.

---

## 1. The obstruction, in the terms you will meet it

A compiled-once body can *receive* a boundary word. It cannot *eliminate* one,
because the `Lowered` lattice and all three ordinary eliminators require a
**compile-time constructor/record template**:

| eliminator | site | refusal |
|---|---|---|
| `Match` | `core.rs:4697` | `unsupported("Match", "scrutinee is not a constructor value")` |
| `ComputationalMatch` | `core.rs:1387` | `unsupported("ComputationalMatch", "scrutinee is not a constructor value after ordinary expression lowering")` |
| `Project` | `core.rs:4754` | `unsupported("Project", "record projection needs a record value")` |

Each is a `let Lowered::Constructor { … } = … else { return Err(…) }` — a
**pattern match against a compile-time-structured variant.** A landed test
already pins the `Match` refusal as intended, so it is a contract the suite
**defends**, not a gap.

Meanwhile **every `LexicalClosure` body is its own unit**
(`static_transition.rs:961` emits `EdgeKind::StaticBody` for every one), so under
`D1` its arguments arrive through `Parameter`/`ValueWord` slots. ⇒ The bodies that
most need to eliminate are exactly the ones that will receive opaque words.

⛔ **Three escapes are closed by settled authority. Do not re-propose them:**

| escape | closed by |
|---|---|
| per-caller specialization | defeats `D1` (compiled once per static origin) |
| scalar-only coexistence | rejected at hard-stop #9 |
| compile-time rehydration of the template | violates `D6` |

---

## 2. The inertness envelope — what `B2E` may and may not touch

✅ **May:** add an opaque boundary-value inhabitant to the `Lowered` lattice, and
teach semantic consumers to eliminate it through `B2V`'s helpers.

⛔ **Must land with:**

- **zero `B2F` target population** — no compiled-once target functions;
- **zero cross-owner call switch** — no production traffic re-routed;
- **zero old-authority removal** — nothing deleted that `B2F` is scoped to delete;
- **no second live decoder or value taxonomy.**

`B2F` remains the **atomic** node that creates compiled-once units and routes
production boundary traffic. ⛔ **`B2E` does not take a bite out of it.** If a
deliverable below looks like it needs one, that is a hard stop (`#12`), not a
judgement call.

---

## 3. ⭐⭐ THE BINDING ARTIFACT — a closed ledger, not a list of eliminators

> ⛔ **Do not frame this node as `Match` / `ComputationalMatch` / `Project`.**
> Those three are where the wall was *found*. They are not the surface.

The deliverable is **one `operation × boundary-class` disposition ledger over
every `Lowered` consumer reachable from a transferred value**, classifying:

| axis | requirement |
|---|---|
| structural elimination | via `B2V` as the **sole** decode authority |
| scalar use | classified |
| **callable invocation** | classified — see §3.3 |
| host-result use | classified |
| carry-through / merge / result paths | classified |
| unsupported | ⛔ **exact fail-closed, per class** |

### 3.1 ⭐ The design call the enclave owes you: BOTH axes must be enumerable

**A ledger a human maintains is a document. A ledger the compiler maintains is a
check.** The ruling requires that *"addition of a lowering inhabitant or
consumer must be completeness-critical"* and that *"wildcard fallthrough is not
closure."* That has a concrete meaning on each axis:

**The value axis already exists and is the precedent.** `LoweredVariant`
(`lowering/mod.rs:529`) is the `Lowered` **tag without a value** — **21 arms
today** — and both `Lowered::variant` and `LoweredVariant::boundary_disposition`
are `match`es with **no `_` arm**, so a 22nd variant is a **compile error in
both**. `BoundaryDisposition` has 4 arms (`RepresentedImmediate`,
`RepresentedHandle`, `ProtocolOnly`, `FailClosedForbidden`).

⇒ ⛔ **The operation axis must be given the same shape.** An enumerable
operation tag set, matched with **no `_` arm**, so that adding a consumer is a
**compile error until it is classified**. If the operation axis is a
`Vec<&'static str>`, a doc table, or a test fixture, **the ledger is not closed**
and `AC-E2` will fail — a list can be short and still typecheck, which is the
whole failure mode this node exists to prevent one level up.

⚠ **I am not naming the enum for you** — its right shape depends on the consumer
population you derive in §3.2, and that is a design call inside the node's scope.
⛔ What is *not* negotiable: **the closure mechanism is the type system, not
diligence.**

### 3.2 ⛔ The consumer population is YOURS to derive — here is the surface

I am **not** handing you a hand-counted consumer list. An inventory is bounded by
an unwritten notion of the surface, and two exhaustive searches can each be
complete against a different notion while the site sits in neither. So the frame
gives you the **definition** and requires you to report the **derivation**:

> **A consumer is in the population if a value that arrived through a
> `Parameter`, `Capture`, or `Result` transfer slot can reach it** — directly, or
> transitively through any `Lowered`-valued binding, field, `Box`, or `Vec`
> element that such a value can occupy.

⚠ **Measured, labelled as a Steward count so you can contradict it:**
`Lowered::` occurrences are **207** in `lowering/mod.rs` and **193** in
`lowering/core.rs` (non-test), plus 16 in the test tree. ⛔ **That is an
occurrence count, not a consumer count** — it is the *upper bound on where to
look*, and I am giving it to you as a search budget, **not** as a population.
**Derive the population from the definition above, state the reading you used,
and if your derivation disagrees with anything in this frame, yours wins and you
say so.**

⭐ **The reason this matters more than usual here:** if the population is
under-derived, the ledger still closes — over a smaller surface — and looks
green. **A ledger's closure is a property of its enumeration, not of its rows.**

### 3.3 The four specific constraints, and what each one forbids

**(a) `B2V` is the sole decode authority.** Runtime tag/name comparison and child
projection come from that interface (`boundary_value.rs`:
`tag`/`payload`/`immediate_value_class`/`handle_class_relation`/`storage_shape`/
`boundary_relation_admits`; `boundary_value_clif.rs`: the emitted helper graph,
`record_field` at `:154` taking `(arena, word, name_id, out) -> status`).

⛔ **Projected children remain OPAQUE boundary words and retain the required
region/lifetime context.** A projection that materializes a `Lowered::Constructor`
or `Lowered::Record` from a boundary word **is** compile-time rehydration — the
`D6` escape — and it is the precise wall #11 found. If projection is easier that
way, that is the signal you are about to violate the ruling.

**(b) ONE artifact-static name reference RESOLVED THROUGH the producer's
store-local interning authority. No parallel name authority.**

> ### ⭐⭐ RULING `R1` — `D5` IS A TWO-STAGE SINGLE AUTHORITY (Architect, 2026-07-26)
>
> **Decision `dec_6r447gawdp6hy` — `resolved`.** Ruling `evt_5p1w8vq3b6q5s`,
> thread `thr_7ya91w7k5keyd`. Architect durable record `1d9a6f86` on
> `architect/work`, preserved off-box by the Steward. Transcribed here because
> **an in-thread ruling is not a durable deliverable.**
>
> ⛔ **THE FRAME'S PREMISE WAS WRONG AND THIS IS THE CORRECTION.** The phrase
> *"artifact-static name **ID**"* is superseded by *"artifact-static name
> **reference resolved through the producer's store-local interning
> authority**."* **The implementer's measurement is correct: there is no
> artifact-static `u64` name ID to share.** My text collapsed **two distinct
> identities**:
>
> | identity | what it is |
> |---|---|
> | **artifact-static name reference** | the existing semantic-plane name bytes/span (`SemanticPlane.names` / its `DenseRange`-equivalent) — stable in the artifact, ⛔ **not a store ID** |
> | **store-local name ID** | the dense ID minted by `BoundaryValueStore::intern_symbol` — insertion-order numbering and reverse lookup are part of the landed `B2V` representation |
>
> **The ruling:**
>
> - ✅ **Preserve dense interning.** ⛔ Do not replace it with a hash, do not
>   change the persistent name-ID space, do not break `symbol(id)` reverse
>   lookup.
> - ▶ **`B2E` lands ONE inert artifact/store binding path now.** It resolves the
>   artifact name reference through `intern_symbol` into an **opaque
>   `BoundaryNameId` whose only minting path is producer interning.** Producer
>   materialization **and** semantic elimination consumers both use that same
>   resolver/binding.
> - ▶ **`B2F` later ACTIVATES the prepared binding.** Emitted code loads the
>   resolved store-local ID from that binding/table. ⛔ It may not bake a numeric
>   `u64`, recompute a hash, or introduce a second name authority.
> - ✅ **Runtime owns the exact carrier and table spelling.** The contract is the
>   ownership and dataflow, ⛔ **not a mandated Rust layout.**
>
> ⛔ **A NEWTYPE ALONE DOES NOT SATISFY `D5`/`AC-E5`.** It is **necessary but not
> sufficient**: it prevents a Rust-side constructor, but it **does not connect an
> artifact reference to the runtime store**, and the CLIF ABI **remains forgeable
> as raw bits**. ⭐ **Carrying that missing bridge as a `B2F` residual would
> reproduce hard-stop `#11` one layer later** — which is the whole reason this
> node exists.
>
> ⚠ **Scope:** this is a **premise correction, not a `B2V` reopening.**
> Independent `D4`/`D6`/`D7`/`D8` work continues. ⛔ `D5`/`AC-E5` cannot bind
> terminally until this erratum is fetchable — it is, at the blob below.

> ### ⭐⭐ RULING `R2` — `D5` NEEDS NO PRODUCTION CALLER (Architect, 2026-07-26)
>
> Ruling `evt_111gwqrdsm1n2`, thread `thr_7ya91w7k5keyd`; leader confirmation
> `evt_6mbp4rm0jvv5r`. **Transcribed here because an in-thread ruling is not a
> durable deliverable** — the same reason `R1` above is transcribed. ⚠ `R2` lived
> only in the channel for one review cycle while three WIP commits already cited
> it; the frame is where it becomes fetchable.
>
> ## ✅ THE `D5` SCOPE QUESTION IS SETTLED — read this with `R1`, not against it
>
> **The inert concrete `D5` binding IS SUFFICIENT without a `B2E` production
> traffic caller.** ⇒ The apparent tension in `R1` above — *"a newtype alone does
> not satisfy `D5`/`AC-E5`"* versus *"`B2E` lands ONE **inert** path"* — resolves
> this way:
>
> | question | answer |
> |---|---|
> | must `D5`'s binding be a real artifact→store mechanism, not a bare newtype? | ✅ **yes** — `R1` stands |
> | must a **production caller** invoke it inside `B2E`? | ⛔ **no** — `R2` settles this |
>
> ⇒ **`R1`'s "not sufficient" is about the MECHANISM's shape, not about call
> reachability.** The concrete path
> `artifact reference → intern_symbol → table → slot → load`, with **no numeric-ID
> bake**, must remain intact. ⛔ Do not re-litigate this; do not add a production
> switch to satisfy it.
>
> ## ⛔ BUT `46ed5c97` IS BLOCKED — on THREE independent `D4`/`D7` mechanism gaps
>
> ⚠ **This is the important part, and it is NOT about `D5` at all.** Inertness of
> the *name binding* does not license inertness of the *semantic consumer*.
>
> **1. No admitted opaque value is actually eliminated.** All five
> `Match`/`ComputationalMatch`/`Project` sites call `boundary_eliminate_or_refuse`;
> for an admitted `Lowered::BoundaryValue` the route resolves `Some(route)` and the
> function **immediately returns `unsupported(...)`**, so the caller never reaches
> its constructor/record destructuring. The only decode emitter,
> `emit_boundary_structural_decode`, is `#[allow(dead_code)]` **with no caller.**
> ⇒ ⭐ **`B2F` owns production-traffic ACTIVATION, not CONSTRUCTION of the semantic
> consumer `B2E` exists to supply.** Hard-stop `#11` inserted `B2E` to add an
> opaque inhabitant **and teach semantic consumers to eliminate it through `B2V`.`**
>
> ⚠ **The source said so truthfully** — *"B2E routes the ledger and emits no
> decode."* **A truthful residual record is still a missing deliverable.** ⛔ An
> honest comment does not convert an unbuilt mechanism into an authorized residual.
>
> **2. `Project` has a ledger row with no implementation.**
> `BoundaryDecodeFuncRefs` carries `record_field` and the ledger admits
> `RecordProjection` via `[Class, RecordField]`, but
> `emit_boundary_structural_decode` is **constructor-shaped unconditionally** and
> never calls `record_field`. ⇒ **A ledger cell naming a disposition with no
> corresponding consumer mechanism.** Wiring that emitter cannot implement the
> `Project` route.
>
> **3. The value-general recursive case is absent, and its residual is NOT
> authorized.** `projected_boundary_child` sets `declared_class: None`, and
> `boundary_elimination_route` rejects every such child. ⇒ That leaves **exactly
> the top-level-only / scalar-bounded coexistence shape rejected at `#11`.** The
> prerequisite was the value-general recursive case: project children as opaque
> boundary words, then permit later semantic elimination **without caller
> specialization or compile-time rehydration.**
>
> ## ▶ The five required successor items
>
> 1. Wire admitted `BoundaryValue` paths into **real `B2V`-backed semantic
>    elimination**, preserving `B2E`'s zero production-traffic switch.
> 2. Implement **both** constructor match **and** record projection — not merely
>    ledger rows.
> 3. Support **runtime class selection for projected opaque children**, so nested
>    aggregate elimination is value-general.
> 4. Retain the sound `R2` name binding / no-bake mechanism.
> 5. Correct the `bind_name` comment to say **both authorities converge on
>    `intern_symbol`**, ⛔ not that producer sites literally call `bind_name`.
>
> ⭐ **Tests must exercise the EMITTED CONSUMER PATH**, ⛔ not only the ledger and
> the dead emitter in isolation. **A test that drives a `#[allow(dead_code)]`
> helper directly measures the helper, not the consumer.**
>
> ⚠ **`46ed5c97` remains REJECTED** (`dec_3xnydcbcz4zm9` rejected; no live
> Decision). ⛔ Do not open a Decision until a fresh **complete** candidate is
> handed off.

> ### ⭐⭐ RULING `R3` — THE PREPARATION SEAM (Architect, 2026-07-26)
>
> `dec_6cjcfms028q64` **resolved**, grounded on exact `c19625e8` against base
> `9410d7b8` and the governing `R1` frame/node. ⭐ **Transcribed here for the same
> reason as `R1` and `R2` — an in-thread ruling is not a durable deliverable, and
> this WP has now lost three of them to the channel.**
>
> ✅ **The measurement was correct; the "unsatisfiable" conclusion was too broad.**
> The three missing links are real: the semantic eliminators hold `RuntimeSymbol`
> / `String` rather than a `BoundaryNameReference`; the validated semantic name
> plane is private to planning and `compile_expr_into_module` builds it inside
> lowering preparation; and no production `BoundaryValueStore` is owned or
> threaded by lowering.
>
> ⛔ **The Architect WITHDREW its own prior instruction.** *"Make current
> production Lowering callers supply a live store so every admitted route
> immediately emits"* would make Lowering own store-local identity and would cross
> `B2E`'s inertness line into `B2F` activation.
>
> ⛔ **But item 1 is not deferrable wholesale** — `R1`/`R2` still require the
> **complete inert bridge** so `B2F` cannot reconstruct hard-stop `#11` one layer
> later. ⭐ **This stays inside `B2E`, and is not a new prerequisite node:** `D5`
> and `R1` already put the artifact/store binding path here. What is needed is a
> **two-phase preparation seam**, not a production traffic switch.
>
> | side | ruled boundary |
> |---|---|
> | **store ownership** | `BoundaryValueStore` is **caller-owned**. Lowering does not create, own, or retain it. `B2F` owns the production instance and lifetime. |
> | **planning** | Expose a **closed typed view of artifact-static name references**, keyed to the planned `Match` case constructor / `Project` field occurrence. ⛔ Do not publish raw `SemanticPlane.names`; ⛔ do not re-intern the `RuntimeExpr` string. The plane already carries `CaseConstructor` / `ProjectField` atoms and spans — **expose the capability, not the internals.** |
> | **preparation** | Consumes that typed view **plus a caller-owned store**, resolves **only** via `bind_name` → `intern_symbol`, and produces the prepared table plus the occurrence/case/field-to-slot mapping. |
> | **lowering** | Consumes **only an opaque prepared decode context** — `B2V` helper refs, table handle, plan-derived slot. ⛔ No plane bytes, no store ownership, no minting or rederiving a name ID. |
>
> **Activation.** Production callers stay **inactive / no-traffic** in `B2E`.
> ⛔ `B2F`'s residual is exactly to create/own the production store/context, pass
> it, and activate the already-built path — it may **not** invent the bridge,
> derive references, or add a second resolver.
>
> ⚠ **The evidence bar `c19625e8` did NOT clear.** Its JIT probe is valid and
> discriminating **emitter-level** evidence — real producer, and swapping the
> table row or neutering the tag comparison causally flips it. ⛔ **But it
> hand-feeds a reference/table.** Terminal item-6 evidence must execute a **real
> `Match` or `Project` semantic route whose slot comes from that route's
> plan-derived reference.**
>
> ⛔ **DO NOT land the "achievable half"** — a materialized table plus threaded
> helper refs while the slot stays unobtainable. ⭐ **It would read as bridge
> delivery while preserving the exact missing edge**, which is what `R3` exists to
> prevent.

⭐ **A dead-code allowance is a SUPPRESSED ORACLE.** `#[allow(dead_code)]` on the
only decode emitter is precisely the annotation that silences the compiler's own
answer to *"is this authority ever consumed?"* ⇒ ⛔ **When a node's deliverable is
"a consumer now eliminates X", an `#[allow(dead_code)]` on the eliminator is a
finding, not a tidiness measure.** The rustc warning was already computing finding
1 for free and had been turned off.

The producer already interns:
`boundary_value.rs:2579` —
`let constructor_id = self.intern_symbol(constructor) as u32;`
— and the CLIF helpers consume a `name_id`. ⇒ ⛔ **The consumer must go through
the same interning, not a second hash of the same string.** `boundary_code_id`
(`boundary_value.rs:886`) is the *shape* precedent and says why in its own
doc-comment: it uses **the crate's declared `fnv1a_64`** rather than a second
hash. ⚠ A second derivation that agrees today is worse than one that disagrees,
because it is **two expressions of one authority** and no pin over it can
disagree with its own source.

**(c) `ComputationalMatch` recursive positions preserve the existing
static-origin ownership contract — WITHOUT caller specialization.** `Closure`
and `DeclarationClosure` name their body by `StaticOriginId` and the term is
recovered from the plan by that name alone
(`Lowering::retained_body_occurrence`). ⛔ Do not reintroduce a term beside the
origin; the variant's own doc-comment explains that this is the two-authority
shape `RT-NATIVE-FNSPLIT` exists to remove.

**(d) ⛔ CLASSIFY CALLABLE INVOCATION EVEN THOUGH THE CENSUS CONTAINS NO
`Closure`.** The Architect's stated reason: **current-corpus absence is not a
proof of impossibility.** ⇒ A cell whose answer is *"cannot occur"* must appear
in the ledger as an **explicit fail-closed disposition with an exact error**,
never as an omission. ⚠ An omission and a classification read identically in a
green build — that is what makes this the cell most likely to be skipped.

---

## 4. Deliverables

- **`D1` — the opaque inhabitant.** Add the boundary-value inhabitant to
  `Lowered`. ⚠ **Expect the build to break in `Lowered::variant` and
  `LoweredVariant::boundary_disposition`** — that is `B2V`'s completeness
  mechanism **working**, and classifying the new variant in both is part of `D1`,
  not an obstacle to it.
- **`D2` — the operation axis.** Introduce the enumerable operation tag set with
  no `_` arm (§3.1), covering the derived consumer population (§3.2).
- **`D3` — the ledger.** The `operation × boundary-class` disposition function,
  mechanically closed on both axes, with every cell classified — including every
  fail-closed cell, each with an exact error in the existing
  `unsupported(op, why)` shape.
- **`D4` — structural elimination through `B2V`.** Tag/name comparison and child
  projection routed to `B2V`'s interface; projected children opaque, with
  region/lifetime context retained.
- **`D5` — one artifact-static name *reference*, resolved through the producer's
  store-local interning authority** (`RULING R1`, `dec_6r447gawdp6hy`). ⛔ **Not**
  "one name-ID derivation" — that premise was false; there is no artifact-static
  `u64` to share. `B2E` lands the inert binding path; `B2F` activates it. ⛔ A
  newtype alone does not discharge this.
- **`D6` — `ComputationalMatch` recursive positions** preserved against the
  static-origin ownership contract, no caller specialization.
- **`D7` — the three eliminators** (`Match`, `ComputationalMatch`, `Project`)
  route an opaque scrutinee/record through the ledger instead of refusing it —
  ⛔ while **retaining** the existing refusal for genuinely unsupported cases,
  because a landed test defends it.
- **`D8` — inertness evidence.** A positive demonstration that `B2F`'s target
  population is still **zero**, no cross-owner call is switched, and no
  old authority is removed.

---

## 5. Acceptance criteria

**`AC-E1` — every cell of the ledger is classified, and the classification is
executable.** ⛔ Not a doc table. A reviewer must be able to point at the code
that decides each cell.

**`AC-E2` — CLOSURE IS MECHANICAL ON BOTH AXES, proved by mutation.** Two
mutations, each of which **must break the build** (not a test — the *build*):

1. add a 22nd `Lowered` inhabitant and classify it **nowhere**;
2. add one operation to the operation axis and classify it **nowhere**.

⛔ **A green build under either mutation fails this AC outright**, and it fails it
in the *"the ledger is a document"* direction. ⭐ Note the direction of these
controls: they are **positive controls on the closure mechanism** — the expected
result is a **failure**, so record the exact compiler error, not "it broke."

**`AC-E3` — a fail-closed cell FIRES, and its error names the class.** For at
least one genuinely unsupported `(operation, class)` pair, exhibit the exact
error. ⚠ **A negative check passes for any reason** — so this AC needs its
positive counterpart: a *supported* pair through the same path that does **not**
error. ⛔ Without both, "fails closed" is unevidenced.

**`AC-E4` — the `Closure` invocation cell exists and is reachable in the
ledger.** ⛔ It may be classified fail-closed, but it must be **present as a
classification**. The control: a mutation that *deletes* that cell must break the
build (`AC-E2`'s mechanism, applied to the specific cell most likely to be
omitted).

**`AC-E5` — no parallel name authority.** ⭐ **Control shape SET BY `RULING R1`,
and it is stricter than what stood here:**

> A relocation of the canonical **artifact name reference** must move
> **producer materialization and consumer lookup together, through the one
> resolver.** A bypass that supplies a literal / store ID, or derives a second
> value, **must fail.**

⛔ The control that discriminates is a **relocation, not a removal** — change
what the producer interns to and show the consumer follows. A consumer that
computes an intersection with a hardcoded table passes every *removal* and fails
only a *move*.

⛔ **AND A NEWTYPE DOES NOT DISCHARGE `AC-E5`.** Per `R1`: necessary, not
sufficient. It blocks a Rust-side constructor while leaving the
artifact→store bridge unbuilt and the **CLIF ABI forgeable as raw bits**. ⇒ The
evidence must show the **binding path**, not just an opaque type.

⭐ **`B2F`'s remaining residual is now exactly one thing** — that production
emission actually **loads/calls the already-prepared binding**. `R1` deliberately
shrank it to that, because carrying the *bridge* as a `B2F` residual would
reproduce hard-stop `#11` one layer later.

**`AC-E6` — projection does not rehydrate.** Prove that a projected child is
still an opaque boundary word and still carries its region/lifetime context.
⛔ **The mutation that matters is not "does projection work"** — it is: make a
projected child be consumed as though it were a `Lowered` template and show that
path is unreachable or errors.

**`AC-E7` — INERTNESS, positively demonstrated.** `B2F` target population is
zero, no cross-owner call switched, no old authority removed. ⛔ An assertion of
absence needs a control that can *see* presence — a census that cannot observe a
target function cannot testify that there are none.

**`AC-E8` — no-regression in CI.** The workspace build, `--locked`, and the
conformance suite are **CI's** job on GitHub. ⛔ Do **not** run
`--workspace`/`--locked` locally (COORDINATION §12). Test with `scripts/ken-cargo`
scoped to `-p ken-runtime`, plus the full `-p ken-interp` suite if you touch the
reifier.

---

## 6. The measured ground — labelled, and not mine to assert

From the Runtime ring's `B2F` grounding on bound code `bb3e58ea`. ⛔ **This is a
RING measurement, relayed on a fetchable ref — not Steward-re-derived.** If your
measurement disagrees, **yours wins and you say so.**

| class | events |
|---|---|
| `Constructor` | 31 |
| `Int` | 8 |
| `HostResult` | 4 |
| `CapabilityToken` | 2 |
| `BorrowedNativeValue` | 2 |
| **total** | **47 events / 10 distinct positions** |

Censused at `call_env == args ++ captures` — the actual transfer boundary, not a
top-level-shape proxy.

⭐ **`HostResult` is measurably NOT implicated in #11.** Stripping
`HostResult.{ok,error}` at all 11 cross-owner sites is **444 / 0 green**, while
`Constructor.args` and the constructor tag each redden. ⇒ Hard-stop #10 named
`Constructor` and `HostResult` together; **#11 splits them.** Do not inherit the
pairing.

⚠ **AND BOTH REDDENINGS ARE THE SAME SINGLE TEST OF 444** —
`constructors::heterogeneous_frame_environment_and_binder_order_are_preserved`,
a `LexicalClosure` matching on its own parameter `Var(0)`, applied to a
`Construct`. First-order Ken. ⛔ **Thin coverage is a hazard to widen, not a
result to lean on**: with one test standing between this node and a false green,
a partial or mis-scoped `B2E` would look correct. Treat widening that coverage as
in scope.

**Evidence refs on `origin`:**

| ref | commit |
|---|---|
| `preserved/rt-fnsplit-b2f-hardstop-11-evidence` | `d1abbc79` |
| `preserved/architect-state-b134f710` | `b134f710` |

⚠ `d1abbc79`, **not** `a376bf65` — the earlier push named only two eliminators
and would have under-scoped this node by one.

---

## 7. ⭐⭐ `B2E`'S OWN RESIDUAL — in the frame, where you are standing

This chain has produced the same shape three times:

| node | shipped | could not check | found by |
|---|---|---|---|
| `B2O` | a partition | one-for-one consumption | `B2R` |
| `B2R` | ownership modes | obedience to them | `B2V` |
| `B2V` | a representation | consumption of it | `B2F` (#11) |

**Each node's residual is exactly the half its own inertness made unverifiable,
and each was found by the node downstream.** `B2E` is also inert, so it will have
one too. ⛔ **Naming it is this frame's obligation, not `B2F`'s discovery:**

> **`B2E` closes a ledger of DISPOSITIONS. It cannot check that production
> traffic actually takes the classified path** — because by construction it
> switches no traffic. **That check is `B2F`'s**, and `B2F`'s release gate depends
> on the closed `B2E` artifact precisely so the two halves meet.

⇒ ⛔ **Do not cite a closed ledger as evidence that elimination works end-to-end
in production.** It is evidence that every reachable consumer has a *classified
disposition*. Those are different claims, and the difference is the whole reason
this node is inert.

⚠ **And a second, smaller residual, so `B2F` does not have to find it either:**
the ledger is closed over the population **you derive**. If the derivation misses
a consumer, the ledger closes over the smaller surface and reports green. ⇒ State
your derivation's reading explicitly in the handoff, so terminal QA can attack
the *notion of the surface* rather than only the rows.

---

## 8. Reporting discipline for this node

⛔ **A clearance must name the axes it covers.** The #11 stop exists partly
because a grounding report said *"no hard stop: this node is satisfiable"* — true
of the three axes it had measured, and read by two readers as a verdict on the
node. Nothing decayed and nothing was mis-worded; **the evidence base under the
clearance grew.**

⇒ When you report a partial result, **the axis list is a required field**, not a
stylistic choice. *"No hard stop on the axes measured so far: A, B, C"* is the
form. The reader's question is always *"is the node OK?"*; yours is only ever
*"did axis N hold?"* — and prose silently collapses the second into the first.

⛔ **Hard stops:** count **#11 stands**; the next one is **#12**, and **#12 is
where a research pull becomes due**, not before.
