---
id: RT-FNSPLIT-B2F
title: "functionization and authority switch — per-static-origin Cranelift target functions, atomic with switch-over, equivalence evidence, and old-path removal"
status: ready
owner: runtime
size: L
gate: none
depends_on: [RT-FNSPLIT-B2A-S, RT-FNSPLIT-B2O, RT-FNSPLIT-B2R, RT-FNSPLIT-B2V, RT-FNSPLIT-C1]
blocks: [RT-SCALE-B]
github: null
origin: Architect ruling evt_6h5gw5c503n5z plus amendment evt_25ynt8615r9sk answering Steward Q1-Q3 on merits (2026-07-25), gated behind research advisory evt_4w1rf45d4fkv3. Replaces the D1/D2 half of the retired RT-FNSPLIT-B2A frame. RE-SLICED 2026-07-25 by Architect ruling evt_842spc7t6js1 on hard-stop #9 (research advisory evt_531c4k52mshrn) plus addendum evt_t4fykh52ncb: this node is NOT buildable as one unit and now depends on two inert prerequisites. Steward-filed; Steward owns the replacement frame and AC/control placement.
---

> ## ⛔ AMENDED 2026-07-27 — PREREQUISITE IS NOW `C1`, AND ONE RESIDUAL IS FALSE
>
> **Ruling `evt_7ay6s5s79awz8`, Decision `dec_45aa2gngjc79z` — resolved,
> verified from the object.** Read this before the `#11` block below it.
>
> ### 1. ✅ This node's PURPOSE and ATOMICITY are UNCHANGED
>
> Per-static-origin Cranelift target functions, atomic with switch-over,
> equivalence evidence, old-path removal. ⭐ **It was ruled that hard-stop `#11`
> is independent of the store/sharing contract**, so `SPEC-STORE-SPLIT` §7's
> instruction to retire this node — which rested on the causal claim the ruling
> found **over-broad** — does not apply to it. `RT-FNSPLIT-C1.md` carries the
> full reasoning for retiring `B2E` and keeping this.
>
> ### 2. ⛔ THE PREREQUISITE CHANGED — `B2E` is RETIRED
>
> **Sequence is now `B2O` → `B2R` → `B2V` → `C1` → `B2F`.** The release gate is
> the **closed `C1` artifact**, not merely `C1` being merged.
>
> ### 3. ⛔ ONE RESIDUAL STATED BELOW IS NOW FALSE — do not build it
>
> `B2E`'s ruling `R1` left this node exactly one residual: *"`B2F` **activates
> it** — loads the resolved **store-local** ID from the binding/table."*
> ⛔ **There is no store-local ID to load.** Constructor and field identity now
> come from **artifact/module semantic authority shared by producer and
> consumer** (`C1` `D1`/`D2`), and `C1` lands that authority **already
> executing** — so this node's residual is not "activate a prepared binding" but
> "route production traffic through an edge that already runs."
>
> ⚠ **`C1` is NOT inert in the old sense.** It lands a real, executable
> producer → validator → eliminator edge. What it defers is only the
> **production function routing switch**, which is this node. ⛔ Do not plan
> against a `C1` that shipped a representation with the consumers deferred —
> the ruling forbids exactly that shape.
>
> ### 4. Count and cadence
>
> **Hard-stop count of record stays `#11`** — the numbering does not reset and
> the re-put did not add a stop. The armed research-consult and symptom-inventory
> lines live in [`RT-NATIVE-FNSPLIT.md`](RT-NATIVE-FNSPLIT.md); ⛔ read them
> there, not from memory.


> ## ⛔ HELD AT HARD-STOP #11 — RULED: `RT-FNSPLIT-B2E` IS INSERTED BEFORE THIS NODE
>
> **Architect ruling `evt_35p5ancbdmzr7`, Decision `dec_43h1rggqxcf1a`** —
> `resolved`, `resolved_by=agt_37reqftfe6g00`, verified from the object. **The stop
> is ACCEPTED.** Full transcription lives in
> **`docs/program/issues/RT-FNSPLIT-B2E.md`**; the parts that bind *this* node:
>
> - **Sequence is now `B2O` → `B2R` → `B2V` → `B2E` → `B2F`.** ⛔ This node does
>   **not** resume until the closed `B2E` artifact lands. Its release gate depends
>   on that artifact, not merely on `B2E` being merged.
> - ✅ **`B2F` resumes UNCHANGED in purpose and atomicity.** It remains the atomic
>   node that creates compiled-once units and routes production boundary traffic.
>   ⛔ `B2E` does **not** take a bite of it: `B2E` must land with **zero `B2F`
>   target population, zero cross-owner call switch, zero old-authority removal.**
> - **Count #11 stands** — the numbering does not reset. ⛔ **No research pull is
>   due until #12.**
> - ⛔ **The three escapes are closed by settled authority**, so do not re-propose
>   them at resume: **caller specialization** (defeats `D1`), **scalar-only
>   coexistence** (rejected at #9), **compile-time rehydration** (violates `D6`).
> - ⚠ **The stop's own shorthand was corrected by the ruling:** `B2V` *did* land
>   the low-level tagged-word interface. What is missing is the **semantic
>   elimination bridge** above it. ⛔ Do not carry *"B2V has no consumer"* forward
>   as the description — it under-describes what exists and over-describes the gap.
>
> ### ▶ THE STOP AS RAISED — retained because the gate record is real
>
> **2026-07-26 ~13:00Z, `runtime-implementer`, raised before any production edit.**
> `crates/` is **byte-identical to `bb3e58ea`** — **there is nothing to unwind.**
> Evidence **`d1abbc79`** = `docs/program/rt-fnsplit-b2f-hardstop-11-evidence.md`,
> durable on `origin` at `preserved/rt-fnsplit-b2f-hardstop-11-evidence`.
>
> ⛔ **`d1abbc79`, NOT `a376bf65`.** The ref was first pushed at `a376bf65` and was
> **stale by exactly one additive commit** — the implementer's addendum landed after
> the push and names **`Project` (`core.rs:4754`) as a THIRD eliminator** with the
> same compile-time-template wall. ⇒ **The elimination surface `B2E` must cover is
> 3, not 2**, and a reader who fetched only the first ref would have under-scoped it.
> Fast-forwarded (`a376bf65..d1abbc79`, ancestry verified before the move).
> ⭐ The implementer flagged the gap itself, mention-free so as not to
> double-deliver into the Architect's live turn.
> ✅ **The ruling has LANDED** — see the block at the top of this file. ⛔ **Do not
> resume `D1`–`D8` construction until the closed `B2E` artifact lands and the
> Steward re-releases this node.**
>
> **The stop, in one line:** a value can be *written* into a tagged boundary word;
> nothing can *read* one back into a `Lowered` that the lowering can eliminate.
> Every aggregate `Lowered` carries compile-time structure, **all three eliminators**
> require that template (`Match` `core.rs:4697`, `ComputationalMatch` `core.rs:1387`,
> `Project` `core.rs:4754`) and a landed test
> **defends** the refusal, while every `LexicalClosure` body is its own unit
> (`static_transition.rs:961`) — so under `D1` its arguments arrive through
> `Parameter`/`ValueWord` slots, and **31 of the 47 transfers are `Constructor`**.
> Falsified by stripping the template at all 11 cross-owner sites: `args` reddens,
> the tag reddens, `HostResult.{ok,error}` is **444/0 green** ⇒ `HostResult` is
> measurably **not** implicated, narrowing #10's pairing.
>
> ⛔ **This is NOT a defect in `B2V`.** Its scope was *"INERT but EXECUTABLE"* and
> its own source records that a compiled-once callee cannot consume a `HostResult`
> today. **The consumer side was never in anyone's scope** — which is why it
> survived a full ring and two reviews.
>
> ⭐ **Third instance of one pattern in this chain:** `B2O` shipped a partition and
> could not check one-for-one consumption; `B2R` declared ownership modes and could
> not check obedience; `B2V` landed a representation and cannot check consumption.
> **Each node's residual is exactly the half its own inertness made unverifiable,
> and each was found by the node downstream.** ⇒ Binding on the next frame written
> in this chain: **when a node ships a representation, name who eliminates it.**
>
> ⚠ The census amendment below (**47 events / 10 positions**) is **unaffected** —
> the stale item was the implementer's *"no hard stop"* verdict, which was scoped
> to three measured axes, not the measurement. ⛔ **A clearance's scope is not
> visible in its wording**; both the leader's relay and the Steward's frame
> amendment were built on it in good faith.
>
> ## ✅ HARD-STOP #10 IS CLEARED — `B2V` MERGED (2026-07-26)
>
> ⛔ **THIS BLOCK RELEASED THE NODE AND `#11` ABOVE HAS SINCE RE-HELD IT.** #10
> **is** cleared and stays cleared — that part is durable. But the release it
> granted is **spent**: read the `#11` block above for the live state. ⛔ Do not
> cite this heading as permission to resume `D1`–`D8`.
>
> **The #10 hold below is SPENT. Both of its conditions are met:** `B2V`'s frame is
> `origin/main` and `B2V` itself is **merged** (`a5c8ba73`, PR #1014, retros in),
> and this node has been **explicitly kicked** by the Steward. ⛔ **Do not cite the
> hold text below to refuse work on this node** — it is retained because the gate
> record is real, not because it is still operative.
>
> ⭐ **The `AC-11` re-scoping in this block IS still operative, and it is now folded
> into the frame** (`docs/program/wp/RT-FNSPLIT-B2F-functionization.md`), which
> previously did not mention `B2V` at all. Bind the frame; this node is the record.
>
> ⛔ **THE CENSUS FIGURES QUOTED IN THE HISTORICAL RULING BELOW ARE SUPERSEDED.**
> The ruling's *"`Constructor` (29 `Parameter` transfers)"* and *"~33 of 41"* were
> taken pre-`B2V` against a top-level-shape proxy. Re-measured 2026-07-26 by the
> Runtime ring at the actual transfer boundary (`call_env == args ++ captures`):
> **47 events / 10 distinct positions — `Constructor` 31, `Int` 8, `HostResult` 4,
> `CapabilityToken` 2, `BorrowedNativeValue` 2**, and **0 of 47** reach a
> fail-closed disposition. The quoted numbers are left in place because the ruling
> record is verbatim; the **frame** carries the live operand and the provenance.
> ⛔ The re-measurement does **not** narrow `AC-11` — see the frame's census block.
>
> ### Historical — the ruling that inserted `B2V`
>
> ## ⛔ HELD AT HARD-STOP #10 — a NEW PREREQUISITE `RT-FNSPLIT-B2V` IS INSERTED
>
> **Architect ruling `evt_28cnmxf6ncghn`, 2026-07-25.** This node was kicked
> (`evt_3q00bkdra1vca`), ran ~25 minutes, and hard-stopped at **#10 before any
> production code was written**. `crates/` is byte-identical to `1e09a30a`;
> **there is nothing to unwind.** Evidence: `49e24b59..1b789817` on `origin`
> `wp/RT-FNSPLIT-B2F-functionization`.
>
> **The stop is VALID and STRUCTURAL.** `B2O`/`B2R` give static code ownership,
> unit population, slot order/width, and declared ownership. **They never
> defined what the bits of `ValueWord`/`ResultWord` MEAN**, nor how compiled
> code inspects a dynamic aggregate. Measured `Constructor` (29 `Parameter`
> transfers) and `HostResult` (4) have no executable word representation, so a
> fail-closed guard would reject **~33 of 41** source-valued transfers —
> incompatible with `D6` and `D7`.
>
> ⇒ **`RT-FNSPLIT-B2V` is inserted between `B2R` and this node.** Sequence is
> `B2O` → `B2R` → **`B2V`** → `B2F`. ⛔ **Runtime does not resume `B2F`
> construction until `B2V`'s frame is fetchable on `origin/main` AND explicitly
> kicked.**
>
> ⭐ **`AC-11` IS RE-SCOPED BY THIS RULING.** It becomes **enforcement of `B2V`
> on every `Parameter`/`Capture`/`Result` transfer** — **not** rejection of
> common aggregates, and **not** inheritance from `C4`. The `Parameter` +
> `Capture` + `Result` transfer set stands; what changes is that the correct
> response to an aggregate is now *represent it*, not *refuse it*.
>
> ### Historical — the release block, kept because the gate record is real
>
> **The frame is re-anchored and the ring is building against it.** Read
> `docs/program/wp/RT-FNSPLIT-B2F-functionization.md` **as it stands on
> `origin/main` = `1e09a30a` or later** — an earlier revision of that file
> described `RT-FNSPLIT-B2R` as a prerequisite that is *"missing and unowned"*,
> which is precisely what `B2R` landed.
>
> ⛔ **The build branch is `wp/RT-FNSPLIT-B2F-functionization`, cut fresh from
> `origin/main`.** The stale ref that previously held that name (`fbe206a7` —
> the hard-stop-#9 evidence commit, pushed for durability under the build
> branch's name) was moved to
> `refs/heads/preserved/rt-fnsplit-b2f-hardstop-9-evidence` and the `wp/` name
> deleted, so the ring's first push cannot be rejected. **That evidence doc is
> measured at `3891b7aa` and its §2 partition is superseded by `B2O`'s owner
> map — it is not a build input.**
>
> **Steward, 2026-07-25.** The #9 re-slice is complete on `main`:
>
> | | landed | PR |
> |---|---|---|
> | `RT-FNSPLIT-B2O` — the validated `SemanticOwner` partition | `origin/main` = `e470ab65` | **#963** |
> | `RT-FNSPLIT-B2R` — the representation and call-ABI contract | `origin/main` = `c986d0a3` | **#967** |
>
> **The shovel-ready frame is `docs/program/wp/RT-FNSPLIT-B2F-functionization.md`,
> re-anchored at `origin/main` = `bd24422b`.** ⛔ **Read the frame, not the prose
> below it in this file** — everything under *"Superseded header"* was written
> against a base four merges stale and several of its anchors have moved.
>
> **What the re-anchor changed, so a reader can tell what is new:**
>
> - the `HELD AT HARD-STOP #9` block became a **discharged-hold** block; the
>   "prerequisite is missing and unowned" language is gone and names `B2O`/`B2R`;
> - the landed-surface tables were **re-measured**, including the correction that
>   the unit seed set is `plan.entries` ∪ every `StaticBody` **target** — the
>   frame previously said "root ∪ `ClosureBody` heads", which is wrong;
> - `D1`–`D3` were re-cut from **construct** to **consume-and-enforce**, with the
>   one genuinely new obligation named: **minting the artifact-static seed
>   material**, which `B2R` declared and deliberately did not create;
> - new **`AC-11`** (boundary-slot representability — the Adversary's `C4` finding
>   on `abi.rs`, which `B2F` would otherwise inherit as its calling convention)
>   and **`AC-12`** (the declared ownership modes are *obeyed*, not re-read);
> - `AC-G0` is recorded as **answered** — 6 definitions / 8 declarations, Θ(1).
>
> ✅ **Released.** The §2c handoff gate ran clean before the kickoff: `B2R`
> retros 3/3 posted, zero proposed Decisions and zero open questions across the
> space, all three Runtime seats quiescent, and the ring compacted at the `B2R`
> close-out seam with the drops verified (`runtime-implementer` ctx 0%;
> `runtime-leader` and `runtime-qa` both showing `• Context compacted`).
> Contention check: the only other `active` node is the `RT-NATIVE-FNSPLIT`
> parent umbrella — same ring, not a competing WP.
>
> ---
>
> ## Historical — the #9 ruling that produced the re-slice (2026-07-25)
>
> **Architect ruling `evt_842spc7t6js1`, addendum `evt_t4fykh52ncb`.** The
> implementer raised #9 **before writing any code**; the branch
> `wp/RT-FNSPLIT-B2F-functionization` stayed clean at `3891b7aa` (0 commits, 0
> dirty) apart from one doc-only evidence commit, so the re-slice cost nothing.
>
> **The obstruction:** one closed callable unit per static origin requires
> configuration-independent compilation, but the emitted signature is
> `(pointer) -> i64`, `Lowered` is a compile-time specialization lattice rather
> than a value domain, `CaptureSlot` carries only an ordinal, and
> `PredeclaredFunction` has no signature, slot layout, ownership or calling
> convention. **What must exist first is a stable executable representation
> contract for every value crossing a generated-function boundary — NOT
> necessarily one universal boxed `Value`.**
>
> ⭐ **The frame's atomicity is what converted "hard" into "unsatisfiable as
> framed":** the one buildable increment — functionize scalar-parameter origins,
> keep specialization for the rest — is **exactly** what `AC-1` and `D6` forbid.
> That is a tension between two correct requirements, not a defect in either.
>
> **`AC-1` and `D6` are NOT amended.** Bounded coexistence was rejected — not as
> intrinsically unsound, but as the wrong *permanent* architecture for the ruled
> all-origin/Θ(n) objective, because retaining whole-configuration specialization
> for the aggregate complement **preserves the exact super-linear authority this
> chain exists to remove**, and because "scalar on this walk" is an observation
> about current values, not a static classification theorem. It would require
> revising `D1`, `D8`, `AC-1`/`AC-6` and the parent's total-O(n) claim; it is not
> an implementation of the current gate.
>
> ### Two inert prerequisites, then this node unchanged
>
> 1. **`RT-FNSPLIT-B2O`** — static body ownership: a total, validated
>    occurrence → `PredeclaredFunction` mapping.
> 2. **`RT-FNSPLIT-B2R`** — the representation and call-ABI contract.
> 3. **This node** — same shape, same atomic live switch, now depending on both.
>
> **Ownership precedes representation** because the ownership mapping *defines
> the cut*, and "every value that crosses a generated-function boundary" cannot
> be enumerated before the boundary is known.
>
> ### What survives, unchanged
>
> **The all-origin shape and the atomic live switch remain intact; only their
> missing dependency is made explicit.** After the substrate lands, this node is
> still **one atomic candidate**: forward-declare the complete bundle from the
> validated ABI descriptors; define one target per predeclared static function;
> switch the synthesized root to the root target; switch every cross-owner edge
> to a static direct call; transport frame/store state; prove the differential;
> and remove all cross-body whole-configuration re-emission. **At every landed
> point there is exactly one production authority.**
>
> ### ⭐ `D6`'s structural exhibit — use this, not the census
>
> `lower_source_declaration_call` (`…/lowering/core.rs:4034-4050`),
> non-recursive branch: it emits **no call**. It builds
> `call_env = args ++ captures ++ env` and continues the source machine with
> `expr: body` in that environment. **That is the authority being removed, in
> four lines.** The final pin must prove that cross-owner application emits a
> static call with **only the declared frame**, while intra-owner syntax
> traversal stays local. ⚠ **A census alone is supporting evidence, never the
> mechanism.**
>
> ### Corrected accounting (ruling § "Corrected D5/D6 and accounting")
>
> - The population is **59 tokenized calls** into `lower_expr`, **not 58 `self.`
>   spellings** — it includes the synthesized root `compiler.lower_expr` at
>   `core.rs:188`, which *seeds* the descent and cannot be classified as
>   traversal.
> - **Disposition is per occurrence ownership and reaching path**, not one row
>   per source site. ⛔ The five provenance classes are **evidence inputs, not
>   the authority partition** — this is the Architect confirming the Steward's
>   withdrawal of the site-keyed `AC-5`.
> - `D6` removes **retained-body cross-owner re-emission**; ordinary recursive
>   traversal within the current body owner remains.
> - `AC-G0` is **6 definitions / 8 declarations** per native module, Θ(1), and
>   accounted **separately** from the per-static-function population.
> - Growth verdict unchanged: total target units may be Θ(n) while each function
>   is bounded by its own static body/transition contract.
>
> ### Adversary P2 — closed, NOT adopted
>
> ⛔ **Do not add a container-spelling blacklist** for the entry-keyed-store
> residual. That arm stays review-enforced unless the prerequisites' new closed
> ABI/body-owner structures make an allowed-inventory structural pin possible
> **with a positive control**. **`B2F` must not absorb another forbidden-spelling
> scan.**
>
> ---
>
> ## Superseded header — retained for lineage (was `ready`, 2026-07-25)
>
> **The frame is
> `docs/program/wp/RT-FNSPLIT-B2F-functionization.md`.** ⚠ ~~It is now the frame
> of a **`draft`** node and omits the prerequisite above; it must be re-cut after
> `B2O` and `B2R` land.~~ ✅ **DONE — both landed and the frame was re-cut and
> re-anchored at `bd24422b`; see the `ready` block at the top of this file.**
> This file remains the durable home of the Architect's mechanism rulings (a
> ruling that lives only in a channel thread is not a deliverable).
>
> **The predecessor is clear:** `RT-FNSPLIT-B2A-S` merged at `origin/main` =
> `145fe915`, tree byte-identical to the approved `82356022`. This node closes
> symptom-inventory **entry 2 — the last open entry**, so it is still the node
> that closes `RT-NATIVE-FNSPLIT` — now behind two prerequisites.
>
> ⛔ **EVERY ANCHOR IN THE PROSE BELOW IS STALE — use the frame's.** Re-derived
> on `0aa9e53f`: `lower_expr` is at `core.rs:4333` (**not** `:3847`; it has moved
> `:3847 → :4255 → :4333` across three re-frames), and the real production call
> count is ~~**58** `self.lower_expr(` sites spanning `:310`–`:6743`~~ — ⛔ **that
> 58 is ITSELF superseded: the count of record is 59 tokenized calls, and the
> `:310` span start excluded the root at `:188` by construction.** See "Corrected
> accounting" above.
>
> ⚠ **Three framing findings the draft below does not contain:**
>
> 1. **The pin `B2F` breaks first is already committed** —
>    `correspondence_adds_no_emitted_unit_to_the_production_census`
>    (`lowering/core/tests/control.rs:3336` → ⚠ **`:3337`** at `bd24422b`) asserts
>    an *exact* emitted-unit
>    census (`core.rs` 1 builder / 1 definition / 2 declarations; four other
>    files all zero). `B2F` must **re-baseline it to a PREDICTED number, not to
>    the observed output**, and must not weaken it — escape-clause condition (1)
>    rests on it.
> 2. ⭐ **The census population is narrower than "the backend."**
>    `crates/ken-runtime/src/native_int_clif.rs` is **production** (un-gated at
>    `lib.rs:23`) and holds ~~**5** `FunctionBuilder::new` sites~~ — ⛔ **that 5
>    is a SOURCE-SITE count and was the wrong population; the emitted-unit
>    constant is 6 definitions / 8 declarations** — with its own declare/define
>    helpers, yet is in **neither** the N1 census nor
>    `BACKEND_PRODUCTION_SOURCES`. The landed pins are correctly scoped and say
>    so — **but this node owns a scaling verdict, and a verdict whose denominator
>    silently excludes a sibling production emitter measures the wrong
>    population.** **AC-G0** requires the denominator be named and every
>    exclusion justified. ✅ **It is ANSWERED — Θ(1) per native module, the 6
>    already pinned as `LOCAL_HELPER_COUNT`; see the frame.**
> 3. ⚠ **`BACKEND_PRODUCTION_SOURCES` is now 13 files, not 12** — `B2R` added
>    `planning/static_transition/abi.rs`. The five-row N1 census did **not**
>    grow with it, so the two populations have diverged; `AC-2` now requires the
>    census to state which files it covers and why.
> 3. **`B2A-S`'s AC-4 pins the `origin -> expression` lookup count at exactly
>    one.** A second consumer reddens it **correctly**; route through
>    `retained_body_occurrence` or re-baseline explicitly.
>
> ⚠ **Not yet released.** Fleet is single-threaded; run the §2c gate before
> kicking.

## This is a CONSTRUCTION. The frame must say so in those words.

The retired `RT-FNSPLIT-B2A` called this a behaviour-preserving **port** because
its `Retain` list was inherited from `b077eb7a`, a branch that **never landed**.
On the real base there is **one** production Cranelift function and **no**
emitted-unit population to re-key. ⛔ **The frame must describe the target units
as NEW, never as retained ones.**

## ✅ Q1 RULED — shape (a), per-static-origin Cranelift functions, ON MERITS

**One closed Cranelift target function per static planned function/origin,
forward-declared as a bundle, with the fixed explicit activation frame.**

⚠ **This is explicitly NOT carried from `b077eb7a` or from the invalid frame** —
the Architect re-decided it from scratch, and the held branch contributes **no
authority**. The four stated merits, transcribed because they are the reason this
choice is not re-openable on taste:

1. **The operator gate is PER-FUNCTION growth.** Ken's original failure was one
   Cranelift `Function` accumulating the whole program's lowering state. A
   direct-label/CFG machine inside one Cranelift function **still** grows that
   function's IR/VReg population with every static transition — it changes the
   control representation without establishing a bounded per-function unit.
2. A data-driven bytecode/instruction VM *could* keep one interpreter function
   bounded, but needs new instruction semantics, a decoder/dispatcher, a code
   store, and a runtime machine — **a larger new abstraction** than the
   already-planned semantic programs plus Cranelift module declarations, and it
   moves execution off the backend's current direct-code contract **with no
   demonstrated need**.
3. **`cranelift_module::Module` already supplies the right bundle boundary** —
   declare all signatures/IDs first, then define each body. The landed plane
   already has exact static origins, program IDs, capture layouts, and
   `PredeclaredFunction` records. ⚠ **Those records are evidence of FIT, not proof
   that functions already exist** — they make the function bundle the *smaller*
   construction.
4. One closed function per static unit gives the wanted invariant directly:
   dynamic environment/control/store state crosses a **fixed ABI**, code identity
   is **static**, each body has **bounded helper vocabulary**, and **total units
   may be Θ(n) while each function is bounded by its own static
   body/transition contract.** ★ That last clause is the precise scaling claim —
   the frame must state it this way and not as a blanket bound.

## ✅ Q3 RULED — ONE atomic review/merge boundary

**Functionization + live switch-over + differential equivalence + removal of the
old authority are ONE boundary.** ⛔ The ring's proposed live `ii`/`iii` split is
**rejected**: it would leave two live production authorities, which is what
"carrier and removal land together" exists to prevent. **At every landed point
there is exactly one production authority.**

The boundary must include the whole connected mechanism: target code-unit
population · declarations/signatures · the fixed dynamic-frame ABI · persistent-
store transport · static dispatch/call edges · behaviour-equivalence evidence ·
switch-over of **every** live consumer · **removal** of the recursive
whole-configuration body-emission authority.

### ⭐ The ONE permitted escape, as a checkable graph property

A preparatory merge is acceptable **only** when unreachability is mechanically
shown by **all four**:

1. Production still has **exactly** the pre-existing one `FunctionBuilder::new`
   and one root `define_function` path; **no** new production
   `Module::declare_function` / `define_function`, indirect call, dispatch, or
   compiled-module output is reachable.
2. Executable scaffold is **`#[cfg(test)]` only**; production additions are
   **declarative types / validation / data layout only**.
3. **No** feature flag, runtime branch, optional callback, unused function
   pointer, or alternate entry can activate it.
4. The compile-entry reachability census has **zero** production references to
   the scaffold consumer, and a **committed structural test/grep pins that zero
   edge** plus the unchanged one-function census.

⛔ **If preparation needs a production call edge, or emits even one callable
target unit, it is not scaffold** and must travel in the atomic live boundary.
★ **This makes unreachability a checkable graph property, not prose** — and the
committed pin in (4) is what stops it decaying into an assertion.

⚠ Note the cfg(test) asymmetry cuts both ways: a `#[cfg(test)]`-only scaffold is
invisible to a production build *and* a production-only path is invisible to a
test build. Whatever pins condition (4) must be verified in **both**
configurations.

## What this node inherits from the retired frame — decided, not copied

Old `AC-1` (D4 five-category differential suite), old `AC-2` (old-path removal,
⚠ re-scoped: the "whole-configuration emission path" is **not** a separable path
— it is `lower_expr`'s entire recursive-descent inliner, `core.rs:3847`, 60 call
sites), old `AC-3` (the four D3 width invariants, each independently falsifiable),
and old `AC-8` (**no growth claim** — superseded here, since this node *is* where
the scaling verdict belongs) all land on **this** node.

⚠ Old `AC-7`'s scope was right and stays: the **full** `scripts/ken-cargo test -p
ken-runtime`, no filter. ⛔ Workspace, `--locked`, and conformance are CI's.

## Open: what remains of RT-FNSPLIT-B2B

`RT-FNSPLIT-B2B` was framed as "the full emission census + finite differences +
explicit growth verdict." With the scaling verdict now belonging to this node's
atomic boundary, **B2B must be re-derived or subsumed** — do not release it
against its current frame. ⚠ Its premise ("a census taken while the emitter is
still moving measures a moving target") is still sound; what changed is *which*
node the verdict attaches to.

## ⛔⛔ RE-HOMED FROM `B2O` — 2026-07-25 (Architect `evt_5yxjd1zqnyvcq`)

**`B2O`'s `D6` source-route oracle was ruled OUT.** Its structural obligation is
re-homed into `B2R` (ABI/layout population) and here. This section discharges
the Steward's condition on that split: *narrowing a claim MOVES its acceptance
criteria.*

### The boundary-disposition view — derived, not hand-authored

> `B2F` derives a boundary-disposition view **from validated graph facts** rather
> than a parallel hand-authored table.

**The classification is total, and the fourth arm is a rejection:**

| graph fact | disposition |
|---|---|
| `StaticBody` edge between **distinct** function owners | **cross-owner call** |
| same-owner ordinary edge | **local traversal** |
| function edge to a **terminal** owner | **shared exit** |
| any other combination | ⛔ **REJECT** |

⚠ **The reject arm is load-bearing.** A taxonomy with no cell for the honest
answer reads as complete while silently absorbing the cases it cannot classify.

> ### ⛔⛔ THESE FOUR ARE **INVARIANTS `B2F` RELIES ON — NOT ACs `B2F`
> ### DISCHARGES.** AN AC ASSERTING THEM WOULD BE **VACUOUS**.
>
> Raised by `runtime-implementer` (`evt_2m655yqt94ycg`) while held, and
> **verified independently by the Steward against `96627f2a`**, not taken on
> report:
>
> **`validate_function_units`** (`planning/static_transition/semantic_ir.rs`,
> ~140 lines) **already enforces all four as `return Err` arms** in the
> production bytes `B2O` landed:
>
> - `match descriptor.owner` over `Function` / `Terminal` / `TrapTerminal` with
>   **no `_ =>` arm** — so a new `SemanticOwner` variant is a **compile error**,
>   not a silent fall-through.
> - `SemanticOwner::Function(to_unit) if to_unit == from_unit => {}` — the
>   same-owner ordinary edge is the *accepting* arm; `Function(_) =>` rejects.
> - `"static body edge targets a shared exit"` and `"shared exit has an outgoing
>   transfer edge"` are explicit rejects.
>
> ⇒ **Planning REFUSES TO CONSTRUCT a violating graph.** A `B2F` control
> asserting one of these laws is green on **every input that can reach `B2F`**,
> because no violating input can be built to reach it. **It would read as
> coverage and test nothing** — the vacuity failure this chain has already been
> caught by once (the withdrawn `AC-5`).
>
> ⇒ **Cite them to `B2O` as inherited invariants. Do NOT re-assert them here.**
>
> ★ **The obligation that actually survives the re-home is `1.` below —
> one-for-one consumption — which inert `B2O` cannot check and never could.**
> When a claim moves between nodes, the part that survives is the part the
> source node was structurally unable to verify. Everything else was already
> discharged where it was written.

**`B2F` must then:**

1. **Consume that view one-for-one** — no second table, no re-derivation.
2. **Emit each cross-owner transfer with its `B2R` descriptor.**
3. **Preserve intra-owner locality.**
4. **Enter the root unit.**
5. **Remove the old cross-owner whole-configuration re-emission path.**

### ★★ THE CONTROL INVERTS

> *"A Rust wrapper or nested function relocation must remain **GREEN** for
> semantic boundary classification, proving source topology is not authority."*

⛔ Structured controls mutate **graph/owner axes**, never source text. A pin that
reddens on a Rust refactor is measuring the wrong thing. **Four candidate SHAs
were spent on this in `B2O`; do not re-derive it.**

### If sequencing needs a separate durability node

It **may expose only an inert derived semantic-disposition view.** It **must
not** become a second authority table, and it gates **`B2F` switch-over — never
`B2O` production landing.**

⛔ **Do NOT add `syn`, any new dependency, or a source-parsing oracle.**
