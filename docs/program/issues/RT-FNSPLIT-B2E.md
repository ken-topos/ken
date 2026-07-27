---
id: RT-FNSPLIT-B2E
title: "semantic boundary-value elimination — an opaque boundary inhabitant plus a mechanically closed operation-by-class disposition ledger over every reachable Lowered consumer, inert"
status: closed
owner: runtime
size: L
gate: none
depends_on: [RT-FNSPLIT-B2O, RT-FNSPLIT-B2R, RT-FNSPLIT-B2V]
blocks: [RT-FNSPLIT-B2F]
github: null
origin: Architect ruling evt_35p5ancbdmzr7 on hard-stop #11 (2026-07-26), Decision dec_43h1rggqxcf1a — resolved, resolved_by=agt_37reqftfe6g00, verified from the object. Stop raised by runtime-implementer evt_27yytvndqfxcg with falsifier evidence d1abbc79 (origin ref preserved/rt-fnsplit-b2f-hardstop-11-evidence). Steward-filed; Steward owns the frame and AC/control placement.
---

> ## ⛔⛔ RETIRED 2026-07-27 — SUPERSEDED BY `RT-FNSPLIT-C1`
>
> ⛔ **Do not build this node. Do not release it. Do not read the contract below
> as live.** Its successor is
> [`RT-FNSPLIT-C1`](RT-FNSPLIT-C1.md), frame
> [`RT-FNSPLIT-C1-operational-carrier.md`][f] under `docs/program/wp/`.
>
> **Two premises of this node were removed by the `#11` re-put ruling**
> (`evt_7ay6s5s79awz8`, Decision `dec_45aa2gngjc79z` resolved), and both were
> structural rather than wording — which is why this is a retirement and not an
> edit:
>
> 1. ⛔ **"a closed LEDGER, not three eliminators" + INERT.** The ruling's
>    inertness rule states that *"a representation-only artifact with the
>    semantic consumers deferred **does not discharge `#11`**."* The three
>    executable eliminations are now the node; the ledger is one deliverable
>    inside it (`C1` `D5`).
> 2. ⛔ **Name authority through store-local interning** (ruling `R1` below).
>    Identity now comes from **artifact/module semantic authority shared by
>    producer and consumer** — not persistent-store identity. `SemanticPlane`'s
>    `CaseConstructor` / `ProjectField` / `ConstructorSymbol` / `RecordFieldName`
>    atoms are that authority.
>
> ⭐ **`R1`'s MEASUREMENT survives and is worth keeping** — there is no
> artifact-static `u64` name ID sitting ready to be used. What died is its
> conclusion, that the store should supply one. `C1`'s frame §2c measures where
> the identity actually lives.
>
> ⚠ **`R3`'s "expose the capability, not the plane internals" survives intact**
> and is carried into `C1` `D1`; it was never store-dependent.
>
> ⛔ **Everything below this banner is retained as the gate record only.** It is
> the reasoning that produced hard-stop `#11`'s acceptance and the evidence
> chain, both of which stay citable. It is **not** an instruction to anyone.


> ## ⭐ WHY THIS NODE EXISTS — `B2V` shipped a producer and no consumer
>
> `RT-FNSPLIT-B2F` hard-stopped at **#11**: `D1`+`D2`+`D4`+`D6`+`D7` are jointly
> unsatisfiable while a compiled-once body can *receive* a boundary word and
> nothing can *eliminate* one. The Architect **accepted the stop** and inserted
> this node. ⛔ `B2F` does not resume until this artifact lands closed.
>
> **Sequence:** `B2O` → `B2R` → `B2V` → **`B2E`** → `B2F`.

## ⭐ RULING — hard-stop #11 ACCEPTED (Architect, 2026-07-26)

**Decision `dec_43h1rggqxcf1a`** — `resolved`, `resolved_by=agt_37reqftfe6g00`,
**verified from the object**, not from the channel that reported it. Ruling text
`evt_35p5ancbdmzr7`, in thread `thr_2fhx0f4p5ks5`. Transcribed here because **an
in-thread ruling is not a durable deliverable.**

### ⛔ FIRST — the correction to the report's shorthand. It changes the scope.

The stop was reported as *"a representation with no consumer."* The Architect's
correction, and it is load-bearing:

> `B2V` **did** land the low-level executable tagged-word interface — class,
> scalar, tag, field-count, field, record-field, construction, and related
> helpers. **What is absent is the semantic elimination bridge above it.**

⇒ ⛔ **`B2E` is NOT "build the decoder."** The decoder exists. `B2E` is the
**semantic bridge**: an opaque inhabitant in the `Lowered` lattice plus the
teaching of every consumer to eliminate it *through `B2V`'s existing helpers.*
⛔ Anyone who reads this node as licence to write a second decoder has inverted
it — see the `no second live decoder` constraint below.

**Why the obstruction is structural.** A compiled-once body can receive a
boundary word, but the present `Lowered` lattice and ordinary `Match`,
`ComputationalMatch`, and `Project` require **compile-time constructor/record
templates**. The lexical-closure falsifier therefore cannot satisfy
`D1`+`D2`+`D4`+`D6`+`D7` without one of three escapes — **caller
specialization**, **scalar-only coexistence**, or **compile-time rehydration** —
and ⛔ **all three violate settled authority** (`D1`, the #9 coexistence
rejection, and `D6` respectively).

### `B2E` IS INERT INFRASTRUCTURE — not partial functionization

- ✅ It **may** add an opaque boundary-value inhabitant to lowering, and teach
  semantic consumers to eliminate it through `B2V`'s helpers.
- ⛔ It **must** land with **zero `B2F` target population**, **zero cross-owner
  call switch**, **zero old-authority removal**, and **no second live
  decoder or value taxonomy**.
- ⛔ `B2F` **remains the atomic node** that creates compiled-once units and
  routes production boundary traffic. `B2E` does not take a bite of it.

### ⭐⭐ THE BINDING CONTRACT — a closed LEDGER, not three eliminators

> ⛔ **Do not frame `B2E` as only `Match` / `ComputationalMatch` / `Project`.**

The frame **must** carry one **mechanically closed `operation × boundary-class`
disposition ledger** over **every `Lowered` consumer reachable from a transferred
value**. It must classify:

| axis | must be classified |
|---|---|
| structural elimination | via `B2V` as the **sole** decode authority |
| scalar use | ✅ |
| **callable invocation** | ✅ — see the Closure note below |
| host-result use | ✅ |
| carry-through / merge / result paths | ✅ |
| unsupported cases | ⛔ **exact fail-closed**, per class |

⛔ **Adding a lowering inhabitant or a consumer must be COMPLETENESS-CRITICAL** —
i.e. it must break the ledger until classified. **A wildcard fallthrough is not
closure.** This is the same discipline that `B2V`'s exhaustive
`boundary_disposition` (no `_` arm) already demonstrates, applied one level up.

**Four specific constraints from the ruling:**

1. **Structural elimination uses `B2V` as the sole decode authority.** Runtime
   tag/name comparison and child projection come from that interface. ⭐
   **Projected children remain opaque boundary words** and **retain the required
   region/lifetime context** — projection must not silently materialize a
   `Lowered` template, which is the very wall #11 found.
2. **Constructor/record name IDs need ONE stable artifact-static derivation,
   shared with producers.** ⛔ **No parallel name authority.** Sibling of `B2V`'s
   own finding, where a second expression of one hand-written relation made a pin
   unable to disagree with its source.
3. **`ComputationalMatch` recursive positions preserve the existing
   static-origin ownership contract** — ⛔ **without caller specialization.**
4. ⛔ **Closure invocation must be explicitly classified even though the measured
   transfer census contains NO `Closure`.** The Architect's reason, and it is the
   rule this whole chain keeps re-learning: **current-corpus absence is not a
   proof of impossibility.** ⇒ A ledger cell that reads *"cannot occur"* must say
   so as a **fail-closed disposition**, never as an omission.

### After `B2E` lands

`B2F` **resumes unchanged in purpose and atomicity**, with its release gate
depending on **the closed `B2E` artifact**. **Count #11 stands** — the hard-stop
numbering does not reset. ⛔ **No research pull is due until #12.**

## ⭐⭐ RULING `R1` — `D5` IS A TWO-STAGE SINGLE AUTHORITY (2026-07-26)

**Decision `dec_6r447gawdp6hy`** — `resolved`. Ruling `evt_5p1w8vq3b6q5s` in
thread `thr_7ya91w7k5keyd`. Architect durable record **`1d9a6f86`** on
`architect/work`, preserved off-box by the Steward. Transcribed here **and** into
the frame's operative text because **an in-thread ruling is not a durable
deliverable.**

⛔ **THE FRAME CARRIED A FALSE PREMISE AND THE IMPLEMENTER'S MEASUREMENT KILLED
IT.** *"One artifact-static name **ID** derivation"* — **there is no
artifact-static `u64` name ID to share.** The phrase collapsed two identities:

| identity | what it is |
|---|---|
| artifact-static name **reference** | semantic-plane name bytes/span (`SemanticPlane.names`) — stable in the artifact, ⛔ **not a store ID** |
| **store-local** name ID | dense ID minted by `BoundaryValueStore::intern_symbol`; insertion-order numbering + `symbol(id)` reverse lookup are part of landed `B2V` |

**Superseding wording** (now in the frame): *"one artifact-static name
**reference resolved through the producer's store-local interning
authority**."*

- ✅ **Preserve dense interning** — ⛔ no hash substitute, no change to the
  persistent name-ID space, no break of `symbol(id)`.
- ▶ **`B2E` lands ONE inert artifact/store binding path**: resolve the artifact
  name reference through `intern_symbol` into an opaque `BoundaryNameId` whose
  **only** minting path is producer interning; producer materialization and
  semantic elimination both use that resolver.
- ▶ **`B2F` activates it** — loads the resolved store-local ID from the
  binding/table. ⛔ No baked `u64`, no recomputed hash, no second authority.
- ✅ **Runtime owns carrier and table spelling.** The contract is ownership and
  dataflow, ⛔ not a Rust layout.

⛔ **A NEWTYPE ALONE DOES NOT SATISFY `D5`/`AC-E5`** — necessary, not sufficient.
It blocks a Rust-side constructor but leaves the artifact→store bridge unbuilt
and the **CLIF ABI forgeable as raw bits**. ⭐ **Carrying that bridge as a `B2F`
residual would reproduce hard-stop `#11` one layer later.**

⚠ **Scope: a premise correction, NOT a `B2V` reopening.** `D4`/`D6`/`D7`/`D8`
continue independently. `B2F`'s residual is now exactly one thing — that
production emission loads/calls the already-prepared binding.

## ⭐⭐ RULING `R2` — `D5` NEEDS NO PRODUCTION CALLER (2026-07-26)

Ruling `evt_111gwqrdsm1n2`, leader confirmation `evt_6mbp4rm0jvv5r`, thread
`thr_7ya91w7k5keyd`. **Transcribed because an in-thread ruling is not a durable
deliverable** — `R2` was cited by three WIP commits while existing only in the
channel. Full text is in the frame; the operative summary:

✅ **`D5` is SETTLED.** The **inert** concrete `D5` binding is **sufficient without
a `B2E` production-traffic caller.** `R1`'s *"a newtype alone is not sufficient"*
is about the **mechanism's shape**, ⛔ not about call reachability. The path
`artifact reference → intern_symbol → table → slot → load` with **no numeric-ID
bake** must remain intact. ⛔ Do not add a production switch to satisfy `D5`.

⛔ **`46ed5c97` IS REJECTED** (`dec_3xnydcbcz4zm9` rejected; **no live Decision**)
on **three independent `D4`/`D7` mechanism gaps** — ⚠ **none of them about `D5`:**

1. **No admitted opaque value is eliminated.** `boundary_eliminate_or_refuse`
   returns `unsupported(...)` for admitted routes, so callers never reach
   destructuring; the only decode emitter is `#[allow(dead_code)]` with **no
   caller**. ⇒ ⭐ **`B2F` owns production ACTIVATION, not CONSTRUCTION of the
   semantic consumer `B2E` exists to supply.**
2. **`Project` has a ledger row with no implementation** — `record_field` is never
   called; the emitter is constructor-shaped unconditionally.
3. **The value-general recursive case is absent and its residual is NOT
   authorized** — `declared_class: None` makes every projected child rejected,
   leaving exactly the top-level-only shape rejected at `#11`.

⚠ **The source described gap 1 truthfully** (*"B2E routes the ledger and emits no
decode"*). ⛔ **A truthful residual record is still a missing deliverable** — an
honest comment does not convert an unbuilt mechanism into an authorized residual.

⭐ **`#[allow(dead_code)]` is a SUPPRESSED ORACLE.** It silences rustc's own answer
to *"is this authority ever consumed?"* — which was computing gap 1 for free. ⇒
**When a node's deliverable is "a consumer now eliminates X", that annotation on
the eliminator is a finding, not tidiness.**

⇒ Tests must exercise the **emitted consumer path**, ⛔ not the ledger and dead
emitter in isolation. ⛔ **No Decision until a fresh COMPLETE candidate lands.**

## ⭐⭐ RULING `R3` — THE PREPARATION SEAM (Architect, 2026-07-26)

`dec_6cjcfms028q64` **resolved**, grounded on exact `c19625e8` against base
`9410d7b8`. ⭐ **Transcribed here because an in-thread ruling is not a durable
deliverable — and this node has now lost a ruling to the channel three times
(`R1`, `R2`, `R3`), which is why the pattern gets its own section below.**

### ✅ The measurement was CORRECT; the "unsatisfiable" conclusion was too broad

The three missing links are **real**:

1. the semantic eliminators hold `RuntimeSymbol` / `String`, **not** a
   `BoundaryNameReference`;
2. the validated semantic name plane is **private to planning**, and
   `compile_expr_into_module` currently builds it inside lowering preparation;
3. **no production `BoundaryValueStore`** is owned or threaded by lowering.

⛔ **The Architect WITHDREW its own prior instruction.** *"Make current production
Lowering callers supply a live store so every admitted route immediately emits"*
was **over-broad**: it would make Lowering own store-local identity and would
cross `B2E`'s inertness line into `B2F` activation.

⛔ **But item 1 is not deferrable wholesale.** `R1`/`R2` still require `B2E` to
land the **complete inert bridge**, so that `B2F` does not reconstruct hard-stop
`#11` one layer later. ⭐ **This is within `B2E`, not a new prerequisite node** —
`D5`/`R1` explicitly put the artifact/store binding path here. The needed change
is a **two-phase preparation seam**, not a production traffic switch.

### The ruled boundary — four sides

| side | what is ruled |
|---|---|
| **store ownership/lifetime** | `BoundaryValueStore` is **caller-owned** runtime/artifact state. Lowering does not create, own, or retain it. `B2F` owns the eventual production instance and its lifetime. |
| **planning** | `B2E` exposes a **closed typed view of artifact-static name references**, keyed to the actual planned `Match` case constructor / `Project` field occurrence. ⛔ Do **not** make raw `SemanticPlane.names` public; ⛔ do **not** re-intern the `RuntimeExpr` string as a substitute. The plane already carries `CaseConstructor` / `ProjectField` atoms and their spans — **expose the capability, not the plane internals.** |
| **preparation** | An artifact/module preparation boundary consumes that typed view **plus a caller-owned store**, resolves **only** through the existing `bind_name` → `intern_symbol` authority, and produces the prepared table plus the occurrence/case/field-to-slot mapping. |
| **lowering** | Consumes **only an opaque prepared decode context**: `B2V` helper refs, table handle, and the plan-derived slot for the semantic occurrence. ⛔ It must not see plane bytes, own the store, or mint/rederive a name ID. |

**Activation.** Existing production callers stay **inactive / no-traffic** in
`B2E`. ⛔ `B2F`'s residual is **exactly** to create/own the production
store/context, pass it, and activate the already-built semantic path — `B2F` may
**not** invent the bridge, derive references, or add a second resolver.

### Disposition of `c19625e8` — and the evidence bar it did not clear

`c19625e8` remains a preserved **WIP**, ⛔ **not a review candidate**. Items 2–5
carry.

⚠ **The JIT probe is valid but not terminal.** It is discriminating
**emitter-level** evidence — it uses the real producer, and swapping the table row
or neutering the tag comparison causally flips the result. ⛔ **But it hand-feeds
a reference/table.** Terminal item-6 evidence requires executing a **real `Match`
or `Project` semantic route whose slot comes from that route's plan-derived
reference.**

⛔ **DO NOT land the proposed "achievable half"** — a materialized table plus
threaded helper refs while the slot remains unobtainable. ⭐ **That would read as
bridge delivery while preserving the exact missing edge**, which is the precise
failure `R3` exists to prevent.

## Evidence and preservation

| artifact | where |
|---|---|
| falsifier + elimination-surface evidence | **`d1abbc79`** — `origin` ref `preserved/rt-fnsplit-b2f-hardstop-11-evidence` |
| Architect durable state at the ruling | **`b134f710`** — `origin` ref `preserved/architect-state-b134f710` |
| hard-stop #10 evidence (prior stop, same node) | `1b789817` — `preserved/rt-fnsplit-b2f-hardstop-10-evidence` |
| hard-stop #9 evidence | `fbe206a7` — `preserved/rt-fnsplit-b2f-hardstop-9-evidence` |

⚠ The #11 ref was first pushed at `a376bf65` and **fast-forwarded** to `d1abbc79`
(ancestry checked before the move). `a376bf65` named only **two** eliminators;
`d1abbc79` adds **`Project` (`core.rs:4754`)** as the third. ⇒ ⛔ **A reader who
fetched the first ref and never read the thread would have under-scoped this node
by one eliminator** — which is exactly why the ledger, not an eliminator list, is
the binding artifact.

## The measured ground the frame must not re-guess

From the ring's grounding on bound code `bb3e58ea` (its measurement, relayed —
⛔ not Steward-re-derived, and if the frame's reader measures differently, the
reader wins):

- **transfer census 47 events / 10 distinct positions** — `Constructor` 31,
  `Int` 8, `HostResult` 4, `CapabilityToken` 2, `BorrowedNativeValue` 2 —
  censused at `call_env == args ++ captures`, the actual transfer boundary.
- **`HostResult` is measurably NOT implicated in #11**: stripping
  `HostResult.{ok,error}` at all 11 cross-owner sites is **444 / 0 green**, while
  `Constructor.args` and the constructor tag each redden. ⇒ #10 named
  `Constructor` and `HostResult` together; **#11 splits them.**
- ⚠ **Both reddenings are the SAME single test of 444**
  (`constructors::heterogeneous_frame_environment_and_binder_order_are_preserved`).
  Thin coverage is not grounds to dismiss the finding — but it **is** why a
  partial switch-over would look green, and the frame must treat the coverage as
  a hazard to widen, not as a result to lean on.
- **Every `LexicalClosure` body is its own unit** — `static_transition.rs:961`
  emits `EdgeKind::StaticBody` for every one — so under `D1` its arguments arrive
  through `Parameter`/`ValueWord` slots.

## ⭐⭐ THE FRAMING OBLIGATION THIS NODE INHERITS — third instance of one pattern

`B2O` shipped a partition and could not check one-for-one consumption. `B2R`
declared ownership modes and could not check obedience. `B2V` landed a
representation and cannot check consumption. **Each node's residual is exactly
the half its own inertness made unverifiable, and each was found by the node
downstream.**

⇒ ⛔ **`B2E` is itself inert, so it will have the same shape of residual, and the
frame must name it rather than let `B2F` discover it.** Concretely: `B2E` closes a
ledger of *dispositions* and **cannot** check that production traffic actually
takes the classified path — that is `B2F`'s. **The frame states that residual
explicitly, in the frame, where the implementer opens it** — not in this node, and
not in a kickoff message that cannot be edited afterwards.

## ✅ The frame is written and on `origin/main`

**`docs/program/wp/RT-FNSPLIT-B2E-boundary-value-elimination.md`**, blob
**`20928eac`**, landed in PR #1023 — verified byte-identical candidate to `main`
with an **absent-at-prior-main** control. 370 lines. `status: ready`.

⭐ **The design call the frame makes, recorded here so this node's reader knows it
was made:** the ruling's *completeness-critical* requirement is discharged by
making **both** ledger axes enumerable. The value axis already exists —
`LoweredVariant` (`lowering/mod.rs:529`), **21 arms**, with `Lowered::variant` and
`LoweredVariant::boundary_disposition` both `match`es carrying **no `_` arm**, so a
22nd variant is a compile error in both. The frame requires the **operation** axis
to take that same shape, because *a ledger a human maintains is a document; a
ledger the compiler maintains is a check.*

⚠ **The frame does NOT hand the ring a consumer list.** It supplies the
reachability definition, requires the derivation to be reported with its reading
stated, and gives the Steward `Lowered::` occurrence counts (207 / 193) explicitly
as a **search budget, not a population**. Rationale, in the frame: an inventory is
bounded by an unwritten notion of the surface, and an under-derived population
still closes the ledger over the smaller surface and reports green.

⛔ **Still owed before Runtime touches this: the §2c handoff gate.** Compact all
three Runtime seats unconditionally, verify every drop, and `git cat-file -e` every
object the kickoff names at the base it names. `B2F` was kicked once against a
frame that did not mention `B2V`, and the re-anchor cost a publish.

[f]: ../wp/RT-FNSPLIT-C1-operational-carrier.md
