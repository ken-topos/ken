# RT-FNSPLIT-C1 — the operational carrier and its three executable eliminators

> **Owner:** Team Runtime · **Size:** L · **Risk:** ★★★ · **Branch:**
> `wp/RT-FNSPLIT-C1-operational-carrier` · **Replaces:** the retired
> `RT-FNSPLIT-B2E` · **Blocks:** `RT-FNSPLIT-B2F`.
>
> **Base for every measurement below: `origin/main = a31bb7b6`.** Verify each
> fixed input against the tree at pickup; anything that does not survive contact
> is an **in-scope expansion to report**, not a stop.

## ⭐ READ THIS FIRST — what makes this node different from the three before it

`B2O`, `B2R` and `B2V` each landed a representation and deferred its consumer.
Each one's residual was found by the node **downstream** of it. The Architect has
now closed that pattern by ruling (`dec_45aa2gngjc79z`, `evt_7ay6s5s79awz8`):

> *"A prerequisite may be inert **only** in the sense that production function
> routing has not switched to it yet. Its **producer → validator → eliminator
> edge must nevertheless be real and executable.** A representation-only
> artifact with the semantic consumers deferred **does not discharge `#11`**."*

⛔ **So "inert" is no longer available as cover for a deferred consumer.** This
node lands an edge that **executes end to end** — a value is produced into the
carrier, validated, and *eliminated back out* by all three eliminators — while
production function routing stays on the old path. Those are two different
claims, and only the second one is deferred.

⇒ **The single question a reviewer should ask of every deliverable here:
*what executes it?*** If the answer is "`B2F` will", the deliverable is not done.

## 1. Fixed inputs — settled, do not reopen

| input | authority |
|---|---|
| **Hard-stop `#11` still binds.** The compile-time-template wall is **independent** of persistent-store / sharing policy. | Architect ruling `evt_7ay6s5s79awz8`, Decision `dec_45aa2gngjc79z` **resolved** |
| The `SPEC-STORE-SPLIT` §1 claim that the store/sharing conflation is *"why every eliminator needed a compile-time template"* is **over-broad**: the conflation **enlarged** the old prerequisite, it did **not cause** the template requirement. | same ruling; correction landed in `SPEC-STORE-SPLIT.md` §1 |
| **No old escape reopens.** Caller specialization violates compile-once authority (`D1`); scalar/static-template coexistence creates two representation authorities (rejected at `#9`); compile-time rehydration violates `D6`. | same ruling |
| Constructor and field identity come from **artifact/module semantic authority shared by producer and consumer** — ⛔ **not** from persistent-store identity. | same ruling, lever requirement 3 |
| `B2V`'s tagged-word interface is the **sole** decode authority. ⛔ No second decoder, no second value taxonomy. | `B2V` as merged; `RT-FNSPLIT-B2E` R3 (the part that survives) |
| Ordinary closures stay **runtime-local and live-domain only**; ⛔ the re-cut must **not** restore a durable `PersistentClosure` lane. | same ruling, §6 |
| `SPEC-CLOSURE-BOUNDARY`'s six ruled clauses stand. | untouched by the store split |

### 1a. What the store split DID remove from this node's contract

⭐ **This is the whole delta from the retired `B2E`, and it is worth stating
positively rather than as an absence.** `SPEC-STORE-SPLIT` removed obligations
that were *accidentally attached to storage*: stable `SlotId`, mandatory
interning/sharing, canonical-byte adoption, and store-local identity/name
binding. `B2E`'s ruling `R1` had routed the node's **name authority** through
exactly that substrate — *"one artifact-static name reference resolved through
the producer's **store-local interning authority**"* — and that clause is now
**dead**, because the identity a compiled-once body compares against may not
depend on any store's insertion order.

⛔ **`R1`'s conclusion is retired. Its measurement is not.** `R1` correctly
established that there is no artifact-static `u64` name ID sitting ready to be
used; what it got wrong was reaching for the store to supply one. §2c below
names where the artifact-static identity actually lives.

## 2. Measured substrate at `a31bb7b6` — a checklist, NOT a bound

### 2a. The wall, exactly as it stands

All three eliminators refuse a non-template scrutinee, in the same shape:

| eliminator | site | the refusal |
|---|---|---|
| `Match` | `cranelift_backend/lowering/core.rs:4696` | `let Lowered::Constructor { constructor, args } = lowered_scrutinee else { return Err(unsupported("Match", "scrutinee is not a constructor value")) }` |
| `ComputationalMatch` | `core.rs:1384` | `... else { return Err(unsupported("ComputationalMatch", "scrutinee is not a constructor value after ordinary expression lowering")) }` |
| `Project` | `core.rs:4754` | `let Lowered::Record { fields } = lowered_record else { return Err(unsupported("Project", "record projection needs a record value")) }` |

And what each does **after** the refusal is the substance of the wall, not the
refusal itself:

- `Match` (`:4700`–`:4712`) selects with `case.constructor == constructor` — a
  **compile-time name compare against the static case list** — then checks
  `case.binders != args.len()`, a **compile-time arity** check, then binds `args`
  (a compile-time `Vec<Lowered>`).
- `Project` (`:4761`–`:4765`) finds the field with `name == *field` over a
  compile-time `Vec<(String, Lowered)>`.

⇒ **The frame's target is not "make the `else` branch succeed."** It is to give
each eliminator a **second, executable route** in which the discrimination
happens in emitted code against the artifact-static case set, and the projected
children come back **in the same carrier**.

### 2b. The carrier that already exists — `B2V`, merged

`crates/ken-runtime/src/boundary_value.rs` (2844 lines) ·
`crates/ken-runtime/src/boundary_value_clif.rs` (8117 lines).

| landed | where |
|---|---|
| `BoundaryWord(pub u64)`, 8 tag bits + 56 payload bits | `boundary_value.rs:82`–`:343` |
| `BoundaryClass` — closed 9-member set, `from_bits` returns `None` outside it | `:359`–`:397` |
| `BoundaryTag` — closed 9-member set, `referent_owner`, `is_immediate` | `:101`–`:245` |
| node layout: `NodeField::{Class,Owner,Slot,TagId,Payload,FieldCount,FieldsAt,Extent,LimbsAt,LimbCount,IntSealed}`, 11 fields, `BOUNDARY_NODE_STRIDE` | `:915`–`:1013` |
| the emitted-code interface | `boundary_value_clif.rs:138` `BoundaryLocalFuncs`, `:243` `emit_boundary_value_local_graph` |

⛔ **The producer is emitted and the result is DISCARDED.** `core.rs:87`:

```rust
let boundary_plan = crate::boundary_value::BoundaryEmissionPlan::derive();
let _boundary_value_abi = crate::boundary_value_clif::emit_boundary_value_local_graph(
    &mut module, &native_int, &boundary_plan,
)?;
```

The leading underscore is the literal signature of the state the inertness rule
now forbids. ⭐ **This node's success is visible as that underscore going away**
— `BoundaryLocalFuncs` must be threaded to a consumer that calls the helpers.
⚠ That is a **necessary** condition, not a sufficient one: threading the handle
while nothing calls a helper reproduces the same defect one layer in. `AC-C7`
below is the version with teeth.

### 2c. ⭐ WHERE THE ARTIFACT-STATIC IDENTITY ACTUALLY LIVES — measured

The authority the ruling names **already exists** and is not store-derived:
`SemanticPlane`, in
`cranelift_backend/planning/static_transition/semantic_ir.rs:525`.

```rust
pub(super) struct SemanticPlane {
    ...
    /// Interned atom content (symbols, literal bytes, big-integer limbs).
    pub(super) names: Vec<u8>,
    ...
}
```

Its atom kinds (`:322`–`:349`) include exactly the four this node needs:
**`ConstructorSymbol`**, **`RecordFieldName`**, **`ProjectField`**,
**`CaseConstructor`**. Each is a fixed-width atom naming its own kind plus an
out-of-line content span into `names` (`:284`, `:366`–`:407`).

⇒ **That is the artifact/module semantic authority shared by producer and
consumer.** It is stable in the artifact, it is not a store ID, and it is
already validated as part of plane construction.

### 2d. ⛔ AND IT IS NOT REACHABLE FROM LOWERING. This is a hard design input.

**Measured visibility at `a31bb7b6`:**

| item | visibility | can `lowering` see it? |
|---|---|---|
| `SemanticPlane` | `pub(super)` in `planning::static_transition` | ⛔ **NO** — `super` is `planning` |
| `StaticTransitionPlan` | `pub(in crate::cranelift_backend)` | ✅ yes |
| `StaticOriginId` | `pub(in crate::cranelift_backend)` (`semantic_ir.rs:26`) | ✅ yes |

And `StaticTransitionPlan`'s entire exported surface is four accessors plus the
entry point — `source_occurrence` (`:1111`), `child_static_origin` (`:1138`),
`root_static_origin` (`:1151`), `declaration_occurrence_origin` (`:1164`),
`plan_static_transition_graph` (`:1838`). ⛔ **None of them exposes a
case-constructor or field-name atom.** `core.rs` contains **zero**
`static_transition::` references.

⇒ **Deliverable `D1` below is therefore forced, and it is forced by the
visibility graph rather than by taste.** ⛔ Do **not** resolve this by making
`SemanticPlane` or `names` public: that widens the production surface to serve a
consumer, which is the move `RT-SCALE-A` was held for. Export a **capability**
keyed to the occurrence — the shape `B2E`'s `R3` already ruled and the one part
of `R3` that survives intact: *"expose the capability, not the plane
internals."*

### 2e. ⛔ THE PRODUCER CURRENTLY MINTS ITS IDENTITY FROM THE STORE

`boundary_value.rs:2642` `fn materialize` (production; reached from
`pub fn materialize_ground` at `:2633`) builds a constructor node with:

```rust
let tag_id = self.intern_symbol(constructor);      // :2733
```

and record field names the same way (`:2752`). `intern_symbol` (`:2500`) is
**dense insertion-order numbering** over one store instance.

⇒ **The same constructor gets a different `TagId` in a different store, or in
the same store reached by a different insertion order.** A compiled-once body
cannot compare against that — which is precisely the substrate the ruling
removes from the prerequisite contract. **`TagId` must be re-grounded on the
artifact-static atom from §2c.**

⚠ **Measured, and it is what makes this affordable:** `BoundaryValueStore` has
**no production construction site.** Every `BoundaryValueStore::new()` in the
tree is under `#[cfg(test)]` (`boundary_value_clif.rs:2680` onward). So this
re-grounding changes a producer whose only current callers are tests — ⛔ which
is *also* the warning: **the tests are the only thing that will notice, so they
must be strengthened to notice the right property**, not merely updated to pass.

### 2f. What `Lowered` is, so the carrier is not mistaken for a variant of it

`Lowered` is a **compile-time specialization lattice**, not a value domain — the
finding that produced `B2V` at hard-stop `#10`. Both
`Lowered::Constructor { constructor, args }` and `Lowered::Record { fields }`
carry compile-time structure by construction.

⛔ **Do not add a `Lowered::Boundary`-shaped inhabitant and call the node done.**
That is the `B2E` shape, and it is exactly what the inertness rule rejects: the
inhabitant is the easy half, the three executable eliminations are the node.

### 2g. ★ The `D3` carrier-representation ruling — Architect, transcribed verbatim

⭐ **This is the required interpretation of `§2f`.** Transcribed here 2026-07-28
because the ruling lived only in a channel thread: Decision
**`dec_4te25repm33ph`** (proposed and resolved by the Architect,
`2026-07-27T20:00:01Z`), against the question *"choose the typed representation
for an already-carried `BoundaryWord` without violating `§2f` or creating a
second decoder/value taxonomy."* ⛔ Do not re-derive it from a thread; bind it
from here. ⚠ If this text and any channel restatement disagree, **this text
governs** — it is the resolution field of the Decision object, copied without
edit:

> `D3` must use a closed phase sum, not `Lowered::Boundary`. Keep `Lowered` as
> the pre-boundary specialization lattice. Introduce one private lowering-flow
> wrapper (name may vary, shape may not), with exactly `Specialized(Lowered)`
> and `Carried(CarriedBoundaryWord)`; `CarriedBoundaryWord` contains the existing
> Cranelift SSA `BoundaryWord` only (plus no static constructor/field/body/
> template data and no parallel tag/class identity). Environments and result
> surfaces that can receive a transferred value carry this wrapper. The boundary
> producer has a one-way typed seam `Lowered -> CarriedBoundaryWord`; it consumes
> the sole `BoundaryLocalFuncs` authority. There is no inverse conversion and no
> compile-time rehydration. `Match`, `ComputationalMatch`, and `Project` have
> explicit `Carried` arms that call the emitted helpers; projected children
> remain `Carried`. Existing specialization paths remain explicit `Specialized`
> arms. Every other reachable consumer exhaustively classifies both phases with
> no wildcard, so a new flow inhabitant or consumer breaks compilation until
> classified. An already-carried word MUST NOT acquire a `LoweredVariant`,
> `BoundaryDisposition`, `AlreadyCarried` encoding policy, or pass through the
> producer again: those tables classify pre-boundary `Lowered`, while `Carried`
> records phase state, not a second value taxonomy. The helper `FuncId`s must be
> declared into each generated function, stored as callable refs, and actually
> called by the three elimination routes; discarded/underscore-only threading
> does not satisfy `D3`. This is the required interpretation of `§2f`: a real
> consumed carrier intermediate is required, but placing it inside `Lowered` is
> not permitted and the inhabitant alone never discharges the node.

**Name adopted by the ring:** `LoweringOperand { Specialized(Lowered),
Carried(CarriedBoundaryWord) }`. ⭐ The ruling fixes the *shape* and leaves the
*name* free, so this spelling is the leader's choice and binds the
implementation, not the ruling.

⚠ **One clause is easy to lose, and it is the one a surface census needs:**
*"Environments and result surfaces that can receive a transferred value carry
this wrapper."* That is a **positive scope requirement** — it says *where* the
wrapper must appear, not merely what it may contain. ⛔ A census that enumerates
only the three eliminators under-covers it.

### 2h. ★★ The `D3` OBLIGATED-SURFACE ruling — Architect, transcribed verbatim

⭐ **`§2g` fixed the carrier's *shape*; this fixes *how far it propagates*.**
Decision **`dec_6fa4k28sp3y10`**, resolved by the Architect
`2026-07-28T01:20:34Z`, against the question of how far `§2g`'s clause
*"environments and result surfaces that can receive a transferred value"*
obligates the wrapper. ⛔ Transcribed at resolution time, per the lesson `§2g`
records. ⚠ **This text governs over any restatement**, including the summary
below it. Resolution field, copied without edit:

> Choose A at the semantic boundary: full compiler-enforced phase closure. The
> phrase "can receive a transferred value" denotes the transitive lowering
> dataflow reachable from the one-way producer — environment insertion, recursive
> call, branch/join forwarding, and result propagation — not the set of paths that
> happen to produce `Carried` in today's corpus. Once a projected `Carried` child
> enters `case_env` and that environment enters the mutually recursive lowering
> component, the shared environment spine and every result surface that can
> forward or return that operand MUST carry the closed
> `LoweringOperand::{Specialized(Lowered), Carried(CarriedBoundaryWord)}` sum.
> There is no manually maintained reachability whitelist and therefore no option-B
> boundary.
>
> This is full phase closure, not a blind replacement of all 622 `Lowered`
> mentions or an unproved claim that the textual count of 64 signatures is itself
> the closed population. A helper may retain raw `Lowered` parameters/results only
> behind an explicit typed phase boundary where its contract makes it structurally
> incapable of receiving, forwarding, or returning a `Carried` operand — for
> example, a leaf that constructs a fresh specialized value. Every edge from
> `LoweringOperand` into such a specialized-only helper must first exhaustively
> classify both variants with no wildcard; `Carried` must take its ruled
> emitted-helper route or fail closed. No conversion from `Carried` to `Lowered`
> is permitted. Thus the implementation must classify the 64-signature census by
> dataflow role, but all phase-bearing environment/result spine edges are
> obligated and mechanically enforced by Rust's type/exhaustiveness checks.
>
> Implement env/spine first, then propagate the wrapper through the recursive
> forwarding results and joins; let type errors enumerate the remaining
> phase-bearing edges. Controls must show a projected `Carried` child remains
> `Carried` through `case_env` and nested lowering, all
> `Match`/`ComputationalMatch`/`Project` consumers have explicit two-phase arms,
> and removing one required wrapper/exhaustive arm reddens compilation or a causal
> targeted control. The source-file size precedent is a reviewability caution, not
> authority to weaken closure; the `B2a` overlap is sequencing for the Steward and
> must be re-enumerated by the compiler rather than hand-patched.
>
> The producer reading the consumed `Lowered` value's existing wildcard-free
> `boundary_disposition()` to obtain its sole authoritative `(tag,class)` is
> correct and required. The prohibition applies to metadata on the resulting
> `Carried` phase: the `Carried` value contains only the SSA boundary word and
> acquires no `LoweredVariant`, `BoundaryDisposition`, or parallel encoding
> policy. `d9d4fb90` is measurement/WIP evidence only, not a reviewed candidate.

⭐ **The population is a DATAFLOW ROLE, not a text count.** A site is obligated
because the producer's output can reach it, ⛔ not because its signature spells
`Lowered`. ⇒ The 64/48/23 census is an **input to classification**, never the
answer; a `Lowered` mention on a specialized-only leaf stays `Lowered`, and the
**typed boundary in front of that leaf** is what must classify both phases.

⚠ **The escape hatch has a precondition that does the work:** a helper keeps raw
`Lowered` only where its contract makes it *structurally incapable* of receiving,
forwarding or returning a `Carried`. ⛔ "It does not today" is not that
property — `§2g`'s clause was written precisely to reject present-corpus
reasoning.

### 2h-i. Steward sequencing — the `B2a` overlap (assigned by `§2h`)

`C1-D3` and `RT-NATIVE-FNSPLIT-recut-B2a`'s `D0` (the origin carrier) both edit
`lowering/core.rs` and `lowering/mod.rs`, and both frames record their site lists
**by line number**.

⭐ **Whichever lands second MUST re-derive its list with the compiler
(`E0063`/`E0027`/exhaustiveness) and ⛔ MUST NOT hand-patch the recorded line
numbers.** Both radii were obtained that way originally, so regeneration is free
and hand-reconciliation is the only way the overlap becomes expensive.

⚠ **The recorded numbers are already stale**: `B1R` measured `core.rs` at
**6,201** lines; on `b62879b3` it is **6,899** (`mod.rs` is **7,471**). ⇒ Treat
every line citation in the `FNSPLIT` frame family as a **landmark, not a
boundary**.

⛔ **`core.rs` is NOT slated to move or be split.** No frame says so; `FNSPLIT`
splits the **emitted native function**, not the source file. ⚠ `B1R` named
editing `core.rs` a stop condition **for `B1R` alone** — its scope was
`planning/**` and nothing else — so that is a scope-boundary fact, ⛔ not a
standing bar on diffs to the file, and `§2h` confirms the size precedent is a
reviewability caution only.

## 3. Deliverables

**`D1` — the artifact-static semantic-identity capability.**
A typed, occurrence-keyed accessor exported from `planning::static_transition`
at `pub(in crate::cranelift_backend)`, which given the `StaticOriginId` of a
`Match` case / `ComputationalMatch` case / `Project` occurrence yields that
occurrence's artifact-static constructor or field identity. ⛔ Not raw
`SemanticPlane`, ⛔ not `names` bytes, ⛔ not a re-intern of the `RuntimeExpr`
string. The correspondence that makes the origin available at the lowering site
is `B2A-C`'s, already merged — ⛔ do not rebuild it.

**`D2` — one identity authority, shared by producer and consumer.**
Re-ground the carrier's `TagId` (and record field identity) on `D1`'s
artifact-static identity, so the producer and the
three eliminators derive the **same** identity from the **same** authority.
⛔ No second derivation, no parallel table, no hash substitute. ⚠ If `symbol(id)`
reverse lookup must survive for diagnostics, it survives as a **view over the
one authority**, never as a second source.

> #### ⭐⭐ AS-BUILT, measured 2026-07-28 — `D1` and `D2` ARE LARGELY BUILT ALREADY
>
> ⛔ **Do not build these from scratch. Verified on the branch, not reported.**
> In `planning/static_transition/semantic_ir.rs` at `790d69e4`:
>
> | what | where | visibility |
> |---|---|---|
> | `case_constructor_identity` (`D1`, `Match`/`ComputationalMatch`) | `:1145` | `pub(super)` |
> | `constructor_symbol_identity` (`D2` producer side) | `:1159` | `pub(super)` |
> | `project_field_identity` (`D1`, `Project`) | `:1172` | `pub(super)` |
> | `record_field_identity` (`D2` producer side) | `:1185` | `pub(super)` |
> | `ConstructorIdentity(DenseRange)` / `FieldIdentity(DenseRange)` | `:45` / `:53` | ✅ already `pub(in crate::cranelift_backend)` |
> | `pack_identity` / `unpack_identity` — `DenseRange` ⟷ `u64` | `:72` / `:89` | the encoding, **already present** |
>
> ⭐ **`D2`'s hard property is therefore already STRUCTURAL, not something to
> establish.** Exactly **one** `identity_span` routes both sides over one closed
> name arena. ⛔ There is no second derivation, no parallel table and no hash
> substitute *to remove* — the thing `D2` forbids was never created. ⇒ **The
> `D2` risk is no longer "build one authority", it is "do not add a second while
> finishing."**
>
> ⚠ **`pack_identity` already documents itself as "the **one** injective
> `DenseRange -> u64` encoding."** ⇒ If the ABI's opaque `tag_id`/`name_id`
> slots need `i64`, that is a **signedness bridge over the existing encoding**,
> ⛔ **never a second `DenseRange -> i64` beside it.** A sibling encoding is
> precisely the second authority `D2` exists to forbid, and it would be
> introduced *by the work that discharges `D2`.*
>
> **⇒ The actual residual of `D1`/`D2`:** widen those four accessors to
> `pub(in crate::cranelift_backend)` (which is what `AC-C1` asks for), settle the
> `u64`/`i64` slot question as a view over the one encoding, and **write the
> consumers.** ⛔ Not a new capability.
>
> ### ⛔⛔ TWO CORRECTIONS — read these before acting on the above
>
> 1. ⛔ **These accessors are NOT on `main`, and they are NOT `B1`'s.** They were
>    introduced by **`6d1dd77b` — "RT-FNSPLIT-C1 S1+S2 (WIP, pre-compile):
>    artifact-static identity capability"** — a **`C1`** commit on `C1`'s own
>    branch, ⛔ not reachable from `origin/main`. `B1` never touched this file;
>    the last three commits to reach it on `main` are `B2R`, the static-body
>    ownership change, and `B2A-C`.
>    ⇒ ⛔ **`C1` has NO dependency on `B1` for this**, and ⛔ nobody should wait
>    on `B1` to supply it. ⚠ An earlier report attributed them to `B1`; that
>    attribution is withdrawn here so it cannot become a phantom blocker.
> 2. ⚠ **A `grep` of `main` finds nothing, and that absence is NOT evidence to
>    rebuild.** This is `C1`'s own earlier phase, unconsumed on `C1`'s branch —
>    which is also why rustc's dead-code warnings surfaced it. ⭐ The
>    production-consumer oracle fired on **this WP's own** unfinished work, not a
>    sibling's.
>
> ⚠ **Stale citation retired:** this deliverable previously named *"the producer
> at `boundary_value.rs:2642`."* That file is
> `crates/ken-runtime/src/boundary_value.rs` (2,983 lines) and `:2642` is a **doc
> comment**, not a producer. Locate the producer by name, ⛔ not by line — see
> `§2h-i` on landmarks versus boundaries.

**`D3` — `Match` and `ComputationalMatch` eliminate a carried value.**
A second, executable route in which the scrutinee is a boundary word: emitted
code reads the runtime constructor identity out of the carrier, discriminates it
against the **artifact-static case set**, and projects the selected case's
children back **into the same carrier**. ⛔ Projection must not materialize a
`Lowered` template — that is the wall itself. ⭐ `ComputationalMatch`'s recursive
positions preserve the existing static-origin ownership contract, ⛔ without
caller specialization.

⭐ **The typed representation `D3` must use is ruled, not open: see `§2g`** —
a closed phase sum beside `Lowered`, never an inhabitant inside it. ⭐ **And how
far it propagates is ruled too: `§2h`** — full compiler-enforced phase closure
over the transitive dataflow reachable from the producer, ⛔ with no
reachability whitelist.

**`D4` — `Project` eliminates a carried value.**
Emitted code selects a runtime record field by **artifact-static field
identity** and returns **that same carrier**.

**`D5` — structural closure over every reachable consumer outcome.**
Every consumer reachable from a transferred value is classified, and
**unsupported forms fail closed at the typed boundary**. ⛔ A wildcard
fallthrough is not closure — adding an inhabitant or a consumer must **break the
classification until classified**, the discipline `B2V`'s exhaustive
`boundary_disposition` already demonstrates, applied one level up.

⛔ **`Closure` invocation must be explicitly classified even though the measured
transfer census contains none.** The Architect's standing reason, and this chain
keeps re-learning it: **current-corpus absence is not proof of impossibility.**
A cell reading *"cannot occur"* is a **fail-closed disposition**, never an
omission. ⚠ And per §1, its disposition may **not** be a durable
`PersistentClosure` lane.

**`D6` — the executable edge, end to end.**
At least one test drives **producer → validator → eliminator** for each of the
three eliminators, on a value that never had a compile-time template. ⛔ This is
the node's reason to exist; see `AC-C7`.

## 4. Acceptance criteria

Each AC names its **positive control** — the thing that must fire if the work
were skipped. ⛔ An AC with no control is invisible to review: *"discharged"* and
*"never asked"* read identically.

- **`AC-C1`** — `D1`'s accessor exists at `pub(in crate::cranelift_backend)` and
  `SemanticPlane` / `names` remain `pub(super)`.
  **Control (AMENDED 2026-07-27 by the Steward, who authored the original and
  got it wrong — see the amendment note below):** the **positive** half is the
  mechanized pin — `lowering` calls the narrowly exported `D1` accessor, and
  compilation of `ken-runtime` proves it. The **negative** half is discharged by
  **compiler-enforced visibility**, recorded as such, ⛔ **not** as a running
  pin.

> #### ⛔ Amendment note — the original `AC-C1` control was UNRUNNABLE
>
> The original demanded a probe that *"fails to compile"* naming `SemanticPlane`
> from `lowering`. Runtime measured that no mechanism in this repo can execute
> it: **CI runs `nextest`, which does not run doctests at all**; an external
> `rlib` cannot distinguish `pub(super)` from crate visibility; `trybuild` would
> add a dependency; and a source scan is both outside the implementation lane and
> banned by the operator's test policy (2026-07-26).
>
> ⇒ As written it had **zero trust delta** — it could not have gone red.
>
> ⭐ **Compiler-enforced visibility is not a downgrade; on coverage it is
> strictly stronger.** A compile-fail pin tests **one spelling** at **one call
> site**. `pub(super)` is enforced by rustc over **every** name and **every**
> call site, including ones written after this WP. A pin naming `SemanticPlane`
> would have said nothing about `names`, `SemanticNameArena`, or any later
> spelling.
>
> ⚠ **The residual, stated in the direction it actually fails:** compiler
> enforcement is **conditional on the visibility annotation surviving**. If a
> later WP widens `pub(in crate::cranelift_backend)` → `pub(crate)`, the
> encapsulation evaporates and **nothing goes red**. That is a review-enforced
> property, not a mechanized one. ⛔ Report it as a named residual; `AC-C1` does
> **not** close it.
>
> ⛔ Do **not** add `trybuild`, and do **not** substitute a source-scanning
> oracle. Both were considered and rejected here.
- **`AC-C2`** — producer and eliminators derive constructor/field identity from
  one authority.
  **Control:** perturb the artifact-static atom for one constructor and show
  **both** the producer's emitted identity **and** the eliminator's comparison
  move together. ⛔ A test that only shows the producer moved cannot distinguish
  one authority from two that happen to agree.
- **`AC-C3`** — `Match` eliminates a carried value with no compile-time
  template, selecting the correct case.
  **Control:** a **negative** arm — a constructor outside the static case set —
  reaches the closed default, and the probe is shown to detect a *deliberately
  wrong* case selection (so a green result is not green-by-vacuity).
- **`AC-C4`** — `ComputationalMatch` likewise, with recursive positions.
  **Control:** as `AC-C3`, plus evidence the static-origin ownership contract is
  unchanged for the recursive positions.
- **`AC-C5`** — `Project` selects the correct field by artifact-static identity
  and returns the carrier.
  **Control:** a record whose fields are **reordered** relative to declaration
  yields the same projection, and a missing field fails closed.
- **`AC-C6`** — closure is structural: adding an inhabitant or a consumer breaks
  the classification.
  **Control:** a mutation that adds one and shows the build **red**; plus the
  inverse, that the unmutated tree is green. ⚠ **Check the redden is narrow** —
  an unexpectedly wide redden usually means the artifact stopped building, in
  which case you measured the compiler, not the classification.
- **⭐ `AC-C7`** — **the executable-edge AC, and the one that discharges `#11`.**
  For **each** of the three eliminators, a test drives producer → validator →
  eliminator on a value with no compile-time template, and **asserts the
  eliminated result's value**, not merely that no error was returned.
  **Control:** neuter the eliminator's emitted discrimination and show **each**
  of the three tests reddens. ⛔ Report **per eliminator**, not as one aggregate
  — an aggregate differential passes while one of three contributors defects.
- **`AC-C8`** — `boundary_value_clif::emit_boundary_value_local_graph`'s result
  is **consumed**, not bound to `_`.
  **Control:** this is a *necessary-not-sufficient* check and must be labelled
  so in the report; `AC-C7` is what makes it meaningful. ⛔ Do not report `AC-C8`
  alone as evidence the carrier is live.
- **`AC-C9`** — no second decoder, no second value taxonomy, no
  `PersistentClosure` lane restored.
  **Control:** name the single decode authority and show the candidate adds no
  other; for the closure lane, a probe that **would** find a durable closure
  path, demonstrated against a boundary class that **is** durable.
- **`AC-C10`** — **zero `B2F` activation.** No production function routing
  switches, no target-function population, no old-authority removal.
  **Control:** the production entrypoint's routing decision is byte-identical to
  `a31bb7b6`'s, shown by diff.

## 5. Residuals — stated, because an unstated residual reads as discharged

1. **`B2F` still owns functionization and the authority switch.** This node
   makes the edge executable; it does **not** create compiled-once units or move
   production traffic. ⛔ That is deliberate and is `AC-C10`.
2. **The `#11` safety net is ONE test.** Both reddenings recorded in
   `SPEC-STORE-SPLIT` §7a are the same single test out of 444. ⛔ Any claim in
   this node that leans on that test as *coverage* is leaning on one test. `D6`
   / `AC-C7` exist to replace it with real coverage — say explicitly, in the
   report, how many independent tests now discriminate.
3. **`RQ-5` remains unreferenced.** No AC in this program claims the
   cross-owner-call overhead bound. This node does not claim it either. ⛔ Do not
   silently let `D3`/`D4` imply a performance property; if the carrier route is
   measurably slower, report the number and let the Steward scope it.
4. **The scaling verdict is not touched.** `RT-SCALE-B` and the operator's
   n=3..7 gate are downstream of `B2F`, not of this node. ⛔ Do not report any
   per-function growth or scaling claim from this work.

## 6. Validation

⛔ **Targeted only — never `--workspace`** (`COORDINATION §12`, operator hard
rule). Use `scripts/ken-cargo -p ken-runtime`, and `--test <name>` for a single
suite. **The full-workspace build, the `--locked` gate and conformance run in CI
on GitHub**; the scripted publisher polls those checks before it merges. A
"no-regression" criterion here means **green in CI**, never a local
`--workspace` run.

⚠ This node touches the lowering core and the boundary carrier, so run the
**full** `-p ken-runtime` suite rather than a targeted `--test` for the final
candidate.

## 7. ⛔ CONTENTION — `boundary_value.rs` is shared with `RT-VALUE-TOTALITY` P2

**Measured, and it must be sequenced rather than discovered at merge.**

| WP | its sites in `crates/ken-runtime/src/boundary_value.rs` |
|---|---|
| `RT-VALUE-TOTALITY` P2 | `:122` `PersistentClosure = 6` · `:367` `BoundaryClass::Closure = 7` · `:655` tag↔class row · `:2400`–`:2413` adopt arm |
| **this node** | `:359`–`:397` `BoundaryClass` closure discipline (`D5`) · `:915`–`:1013` node layout (`D3`/`D4`) · `:2642`/`:2733`/`:2752` producer identity (`D2`) |

⇒ **They do not collide line-for-line, and that is exactly the hazard** — git
will merge a union of two independently-correct halves with nothing to complain
about. ⛔ **Sequence them; do not run them concurrently.** P2 is already queued
ahead of this node behind `ABI-S3`. After P2 lands, **re-derive §2's substrate
against the new `main`** — the class/tag rows this node's `D5` closes over are
the ones P2 edits.

⚠ **`ABI-S3` is contention-free with this node** (`ken-host`, `ken-elaborator`,
`ken-interp`; disjoint from `ken-runtime/cranelift_backend`) — but it is *not*
contention-free with P2, which also touches `ken-interp`. That ordering is the
Steward's and is already set.

## 8. Reporting

Report **per deliverable and per AC**, with the control's result beside each.
⛔ For `AC-C7`, report **three separate rows** — one per eliminator.

State plainly anything the frame got wrong against the tree. §2 is a **checklist,
not a bound**: a fixed input that does not survive contact is an in-scope
expansion to report and keep building, exactly as `ABI-S3`'s census corrections
were handled. ⛔ The one thing that is a **hard stop** is discovering that a
settled input in §1 is false — that goes to the Architect, not around it.
