---
id: RT-NATIVE-FNSPLIT
title: "Native backend: bound per-function lowering growth to O(n) — helper identity is a variable-width whole-configuration key (orig. single-Function VReg::MAX, since fixed)"
status: active
owner: runtime
size: TBD
gate: none
depends_on: [RT-SCALE-B]
blocks: [NATIVE-HANDLE-CARRIER]
github: null
origin: PX8-SPAN-PROV Phase 2 native reachability wall (runtime-implementer measured repro evt_7qhtk8w489am4; CV option-(c) ruling evt_77q2tc5dh1kzj; Steward scope ruling evt_7c160ej3bwz4; Architect means/layer ruling evt_7gkn3g4tsvgb9, 2026-07-23). Steward-filed (agents cannot create tracked work per COORDINATION §2).
---

> ## ⭐⭐⭐ 2026-07-28 — THIS NODE NOW GATES EVERY REMAINING `PX8` BLOCKER
>
> **Steward sequencing call.** `PX8` gates 15 of the ABI program's 19 nodes, and
> as of PR #1142 its blocker set is:
>
> | `PX8` blocker | state |
> |---|---|
> | [[PX8-WROTE-ABS]] | ✅ **MERGED** PR #1142 |
> | [[PX8-F-CAP-41]] Ph2 (folded into [[NATIVE-HANDLE-CARRIER]]) | ⛔ blocked on **this node** |
> | [[PX8-ERRID-ALLOC]] → [[PX8-ERRID-SCOPE]] | ⛔ blocked on **this node** (new edge, 2026-07-28) |
>
> ⇒ ⭐ **Every remaining path to `PX8` runs through here.**
>
> **Why the new edge:** `PX8-ERRID-ALLOC` is built, QA-approved and
> Architect-approved, but PR #1141 died on
> `Cranelift backend failure: Code for function is too large`
> (`crates/ken-cli/tests/rt_parity_native.rs:370`, ObjectEmission). Foundation
> measured that **both** the original and the wire-corrected candidate fail
> identically against an **unchanged fixture blob** ⇒ allocation growth crossed
> the wall, not the mapping. The only mapping-preserving reduction
> (`e65c81b5`, five tags factored into one generated-tag `require_one_of_i64`)
> was built, controlled, and **still fails**.
>
> ⭐ **The ceiling is NOT general** — `PX8-WROTE-ABS` passed the same job. It is
> specific to per-function native lowering growth, which is exactly this node's
> subject.

## ⛔⛔ GOVERNING RULING 2026-07-27 — `#11` STILL BINDS, and the lever is NAMED

**Decision `dec_45aa2gngjc79z` — RESOLVED.** Architect ruling `evt_7ay6s5s79awz8`,
on the Steward's re-put (`evt_70jp2sk4by7t8`) required by `SPEC-STORE-SPLIT` §7
item 2. **Transcribed here because an in-thread ruling is not a durable
deliverable — and this chain has already lost three rulings to the channel.**

⭐ **Read this before anything below it.** It supersedes the re-cut's *shape*,
not the viability ruling's retain/replace lists.

### 1. ⛔ The answer: `#11` still binds. The relaxation did NOT dissolve it

> *"The compile-time-template wall is **independent of persistent-store/sharing
> policy**. `SPEC-STORE-SPLIT` removes obligations that were **accidentally
> attached to storage** — stable `SlotId`, mandatory interning/sharing,
> canonical-byte adoption, and store-local identity/name binding — but it does
> **not** remove the semantic specialization in `Lowered`."*

Grounded on the preserved evidence at exact `d1abbc79`. The three eliminators
consume **recursive compile-time structure**: `Match` (the `Lowered::Constructor`
arm) selects a constructor case and binds its arguments; `ComputationalMatch`
selects a case and recursive positions; `Project` (`Lowered::Record`) selects a
named field.

⭐ **The reason, in one sentence:** *"none of those decisions is obtained from a
slot, canonical bytes, interning, hashing, or a sharing rule. A body compiled
once may receive different constructors and records on different invocations,
and changing whether those values are copied, shared, interned, arena-held, or
privately handled cannot make their invocation-specific shape
compile-time-known."*

### 2. ⭐⭐ THE ASSERTED ROOT CAUSE WAS TESTED AND IS OVER-BROAD

`SPEC-STORE-SPLIT` §1 claims the conflation is *"why every eliminator needed a
compile-time template."* **Ruled:** *"store/sharing conflation **enlarged** the
old prerequisite, but it did **not cause** the template requirement itself."*

⚠ **The ruling cites this sentence as `RT-VALUE-TOTALITY` §1; it is
`SPEC-STORE-SPLIT` §1.** The claim ruled on is the one quoted above — a
mis-citation of location, not of content.

⛔ **This is why the re-put was framed as a test rather than a premise.** Had the
node's causal sentence been carried in as given, the answer would have been
decided before the question was asked, and the re-cut would have been built on a
dissolution that never happened.

### 3. ⛔ No old escape reopens

Caller specialization still violates compile-once authority; static-template /
scalar coexistence still creates two representation authorities; compile-time
rehydration still requires invocation-specific shape and violates the same
boundary. ⇒ `D1`, the `#9` coexistence rejection, and `D6` all stand.

### 4. ▶ THE LEVER IS KNOWN — a runtime-general OPERATIONAL CARRIER

The re-cut needs *"a runtime-general operational carrier at the
`Lowered`/lowering boundary, **distinct from durable canonical storage**, and
**executable** semantic consumers for all three eliminators"*:

1. `Match` and `ComputationalMatch` **discriminate runtime constructor identity
   against the artifact-static case set**, then project children back into **the
   same operational carrier**.
2. `Project` selects a runtime record field using **artifact-static field
   identity** and returns **that same carrier**.
3. Constructor and field identity come from **artifact/module semantic
   authority shared by producer and consumer** — ⛔ **not** from persistent-store
   identity.
4. Every reachable consumer outcome is **structurally closed**; unsupported
   forms **fail closed at the typed boundary**.

### 5. ⭐⭐⭐ THE INERTNESS RULE THAT ENDS THE B2O→B2R→B2V PATTERN

> *"A prerequisite may be inert **only** in the sense that production function
> routing has not switched to it yet. Its **producer → validator → eliminator
> edge must nevertheless be real and executable.** A representation-only
> artifact with the semantic consumers deferred **does not discharge `#11`**."*

⛔ **This is binding on every node of the re-cut.** `B2O` shipped a partition it
could not check consumption of; `B2R` declared modes it could not check
obedience to; `B2V` landed a representation nothing consumes — **three nodes,
each residual found by the node downstream.** ⇒ The standing lesson
`a-representation-node-must-name-who-eliminates-it` now has a **ruled**
formulation, and "inert" is no longer available as cover for a deferred
consumer.

### 6. `RT-VALUE-TOTALITY` P2 — not causal, but integration-relevant

⛔ **P2 is NOT on this critical path and cannot dissolve `#11`.** This ruling did
not wait on it. But the eventual carrier **must respect P2's canonical /
operational split**, ordinary closures stay runtime-local and live-domain only,
and ⛔ **the re-cut must not restore a durable `PersistentClosure` lane.**

⭐ That independently confirms the scoping in
`wp/RT-VALUE-TOTALITY-P2-representation-split.md` §5, which routes B2V's
`PersistentClosure` lane out of P2 and to this re-cut.

### 7. ▶ What the Steward owes from this ruling

Preserve `#11` **as the prerequisite boundary**, but **re-cut it around the
operational carrier plus the three semantic eliminators**, and **remove the old
store-identity/adoption substrate from the prerequisite contract.** ⛔ The
re-slice and its transcription are the Steward's; the Architect ruled the
mechanism, not a work plan.

> ### ✅ DISCHARGED 2026-07-27 — the re-cut is authored
>
> | | |
> |---|---|
> | **new prerequisite** | [`RT-FNSPLIT-C1`](RT-FNSPLIT-C1.md) — the operational carrier plus its three **executable** eliminators, grounded on artifact-static semantic identity. Frame: [`wp/RT-FNSPLIT-C1-operational-carrier.md`](../wp/RT-FNSPLIT-C1-operational-carrier.md), `ready`. |
> | **retired** | [`RT-FNSPLIT-B2E`](RT-FNSPLIT-B2E.md) → `closed`. Both its premises died: the *inert closed-ledger* contract (killed by the inertness rule) and its *store-local* name authority (killed by lever requirement 3). |
> | **amended, NOT retired** | [`RT-FNSPLIT-B2F`](RT-FNSPLIT-B2F.md) — purpose and atomicity unchanged; dependency `B2E` → `C1`; the `R1` residual *"loads the resolved store-local ID"* is now **false**. |
> | **sequence** | `B2O` → `B2R` → `B2V` → **`C1`** → `B2F` |
>
> ⭐ **Why `B2F` survives while `B2E` does not:** `SPEC-STORE-SPLIT` §7 directed
> that both be retired, on the ground that they are *"built around the constraint
> being removed."* The re-put put that ground under test and the Architect ruled
> it **over-broad** — the conflation *enlarged* the old prerequisite, it did not
> *cause* the template requirement. `B2E`'s contract genuinely descended from the
> removed substrate; `B2F`'s functionization purpose never touched it.
> Full reasoning in `RT-FNSPLIT-C1.md`.
>
> ⛔ **`SPEC-STORE-SPLIT` §7 items 3, 4 and 5 are unaffected by this and remain
> owed** — the `SPEC-ALIGN-A1` stop-list re-read, the `e1b540e2` salvage
> decision, and (item 5, ✅ done) the `RT-VALUE-TOTALITY` P2 frame.

## ⭐⭐ Requirements (`RQ`) — retroactive, per `docs/program/15-*.md`

> **This program is the RETROFIT.** Operator, 2026-07-26: *"we should establish
> RQs retroactively for work in progress. completed work needn't have them, and
> all future work should."* This program qualifies (`B2E` `ready`, `B2F` `ready`)
> — and it is the one program whose **merged** nodes are included, because its
> residual chain was discovered by hand and is therefore **gradeable** (§ RQ-0).
>
> ⛔ **This section is the authority for RQ text.** A frame quoting an RQ is a
> convenience copy; if they disagree, this wins.

| id | kind | requirement | conformance rows it applies | discharged by |
|---|---|---|---|---|
| **`RT-NATIVE-FNSPLIT.RQ-1`** | functional | A cross-owner call transfers a boundary value whose **representation is a tagged word**, not a compile-time template. | `conformance/runtime/values/README.md` (value model), `conformance/runtime/seed-runtime.md` | ✅ `B2V.AC-*` (merged) |
| **`RT-NATIVE-FNSPLIT.RQ-2`** | functional | Ownership and region/lifetime context of a transferred value is **declared and obeyed** across the boundary. | `conformance/runtime/capacity/seed-capacity.md` (region/reclamation), `conformance/runtime/values/README.md` | ✅ `B2R.AC-*` (merged) |
| **`RT-NATIVE-FNSPLIT.RQ-3`** | functional | ⛔ **Boundary values are ELIMINATED — every semantic consumer reachable from a transferred value disposes of it, in PRODUCTION traffic.** | `conformance/runtime/evaluation/seed-evaluation.md`, `conformance/runtime/values/README.md` | ▶ **`C1` lands the executable elimination edge** (`B2E` retired); ⛔ **`B2F` is the only node that can discharge it in PRODUCTION traffic** |
| **`RT-NATIVE-FNSPLIT.RQ-4`** | functional | ⛔ **REWRITTEN 2026-07-27.** Name identity is **one authority**: constructor and field identity come from **artifact/module semantic authority shared by producer and consumer** — ⛔ **not** persistent-store identity. (Was: *"resolved through the producer's store-local interning"* — retired with `B2E`'s `R1`.) | `none` — no seed case asserts name-authority singularity. ⚠ **This absence is itself a finding**; see below. | ▶ `C1.D2` + `AC-C2` (the two-sided perturbation control) |
| **`RT-NATIVE-FNSPLIT.RQ-5`** | **non-functional** | Cross-owner call overhead does not regress against the pre-split baseline. | `none` — a complexity/perf contract, not a definitional row | ⛔ **UNREFERENCED — no AC in this program claims it** |
| **`RT-NATIVE-FNSPLIT.RQ-6`** | **non-functional** | Compiled-once function bodies are created once per unit, not per call site. | `none` — an NFR | ▶ `B2F` (its atomicity clause) |

### ⛔ TWO OPEN REQUIREMENTS, ONE OF THEM PREVIOUSLY INVISIBLE

- **`RQ-3`** — open by design; `B2F` is held at `#11` awaiting `B2E`. Expected.
- ⭐ **`RQ-5` is UNREFERENCED and nobody had noticed.** No AC in `B1R`, `B2A-C`,
  `B2A-S`, `B2O`, `B2R`, `B2V`, `B2E`, or `B2F` claims the overhead bound. ⇒ **The
  program could deliver every functional requirement and regress performance
  without a single red gate.** That is the check finding something on its first
  run, on a program I have been sequencing for weeks.
  ⚠ **I am not filing a WP for it in this turn** — whether it needs its own node
  or a clause in `B2F` is a scoping call, and `B2F` is held. Recorded as open.

⚠ **`RQ-4`'s `none` is a finding, not a clean bill.** Name-authority singularity
has no conformance row, so the *only* thing preventing a second name derivation is
`B2E.AC-E5`'s relocation control. ⛔ If that AC is weakened, nothing definitional
catches it. Sibling of the `B2V` lesson that two expressions of one authority
cannot disagree with each other.

## ⭐ `RQ-0` — THE GRADE: would the unreferenced-RQ check have caught hard-stop `#11`?

**Required by `15-*.md §6`, and it is designed to be able to say no.**

**Answer: PROBABLY, and the mechanism is Rule 1 at authoring time — not the check
itself.** Honest breakdown:

| | |
|---|---|
| ✅ **What it would have shown** | After `B2V` merged, `RQ-3` was **unreferenced by any AC**. Three consecutive green WPs with the program's central requirement unclaimed — visible continuously, in a table, instead of as a surprise at `B2F` pickup. |
| ⭐ **Where it actually bites** | **Not the report — Rule 1 applied when `B2F`'s frame was authored.** `B2F` would have had to name which of its ACs discharges `RQ-3`, and writing that reference forces the question *"does anything eliminate a boundary word?"* ⇒ That is precisely the question `#11` answered NO to, asked at framing rather than at build. |
| ⛔ **How it could still have failed** | The check reports; **it does not gate.** And Rule 1 is satisfiable by pointing a *weak* AC at `RQ-3` — which turns the check green without making the property true. **An advertised link is not an enforced one.** |
| ⚠ **What I already had and did not use** | `B2V`'s frame said in plain text that it *cannot check consumption*. The information existed; what was missing was anything that forced the **next** frame to name a discharger. |

⇒ **Verdict: the scheme keeps its main claim, but the load-bearing part is Rule 1
at authoring time, and the report is the backstop.** ⛔ Do not describe the check
as preventing this class of stop. It makes the gap continuously visible and gives
the framing question a place to be asked; a determined author can still satisfy it
vacuously. That residual is real and belongs in the doc, not in a footnote.


> ## ⛔ READ THE VIABILITY RULING BELOW BEFORE THIS SECTION
>
> **The original root cause described immediately below — one Cranelift
> `Function` per process, hitting `VReg::MAX` — WAS FIXED and is GONE as of
> `b077eb7a`.** It is retained only as the WP's origin history. The live defect
> is a **variable-width whole-configuration helper key**, ruled 2026-07-24
> (`evt_3m1g3v4m2bj51`). ⇒ **Do not implement against this paragraph.**

### Origin (historical — superseded)

Discovered while closing PX8-SPAN-PROV Phase 2's native conformance matrices. The
native backend (`build_native_program`) **inlines the whole process ITree into one
Cranelift function**, so program size scales with the nested resource-bracket
structure. Measured wall (minimal repro, not assumed): **any program with four
nested resource brackets** fails to compile with Cranelift's
`Code for function is too large`, *before* producing any runtime outcome.

- 3-bracket programs lower fine (e.g. SP-A **freeze**, `px8f_buffer_native`
  writeAll).
- The 4th bracket is the wall. A native end-to-end write / precedence / slot-reuse
  discriminator inherently needs four brackets (readable source to mint spans +
  writable dest + two buffers, or release/realloc), so it cannot compile today.

This is a **general native-backend limitation**, independent of provenance — it
blocks *any* 4-resource-bracket Ken program on the native path, not just
PX8-SPAN-PROV's tests.

**Root cause (Architect, `evt_7gkn3g4tsvgb9`, independently reproduced on
`b717bf64` — 175.72 s lowering then object-emission failure):**
`compile_expr_into_module` lowers the whole checked process entrypoint into a
single Cranelift `Function` and calls `define_function` once. Cranelift 0.113.1's
virtual-register allocator emits `Code for function is too large` when that one
function reaches `VReg::MAX` (2²¹−1 virtual registers). So a small, valid Ken
process expands past a backend implementation limit **before execution** — a
native-codegen completeness defect, not a span-provenance / ABI / conformance-
harness defect.

## ⚠ SCALING GATE — operator directive 2026-07-23 (evt_4btfhwqhah1ye)

> ### ▶ THE GATE NOW HAS TRACKED OWNERS — [[RT-SCALE-A]] and [[RT-SCALE-B]]
>
> **Filed 2026-07-26.** The three requirements below are unchanged and still the
> merge condition; what changed is that they are now **executable nodes with
> frames, acceptance criteria and an owner**, not prose in this file.
>
> | | node | frame |
> |---|---|---|
> | Boundary **A** — planner census, pre-lowering | [[RT-SCALE-A]] | `wp/RT-SCALE-A-planner-census.md` |
> | Boundary **B** — full emission + model + verdict | [[RT-SCALE-B]] | `wp/RT-SCALE-B-emission-scaling-verdict.md` |
>
> ⛔ **Requirements 1–3 below are the SOURCE, and the frames are what gets
> executed. If they ever disagree, this section wins and the frame is amended**
> — do not silently follow a frame that has drifted from the operator's words.
>
> ⭐ **Why this was filed at all:** a merge condition that lives only in prose is
> the `KW-THEOREM` failure shape — there, the frame *correctly named* the
> formatter's CI-only coupling and four exact-SHA reviews still approved a
> noncanonical corpus, because CI was the first operative control. All three
> rings' retros converged independently on the same fix: **a requirement only a
> distant gate can observe must become an executable step at the point of
> work.** Nothing sequenced this gate, nothing released it, and no team held it.

**"SP tests complete under a timeout" is NOT acceptance.** After the recut, a
4-resource-bracket program still costs ~103 CPU-s / ~4 GB to compile — the operator
ruled that unacceptable without understanding the scaling law. RT-NATIVE-FNSPLIT does
**not merge** until:
1. **Empirical scaling harness (Runtime, permanent tests):** minimal programs at
   **n = 3,4,5,6,7** nested resource brackets, each measured under a bounded
   harness (`prlimit`, fail-safe) for compile wall-time, peak RSS, and internal
   counts (distinct interned semantic states, defined helpers, total
   DFG/instr/blocks). Report the table + fitted growth curve.
2. **Analytical scaling model (Architect):** predicted order of growth vs. n;
   whether 103 s/4 GB @ n=4 is bad-constants-on-**O(n)** or residual
   super-linearity (→ further mechanism gap). Must be
   **research-grounded** (research dispatch `evt_62fqpe7pfvym4`).
3. **Verdict:** either (a) empirically+analytically **linear O(n)** + a plan
   to reduce the constants; or (b) a **research-supported** reason growth is
   inherently super-linear + an explicit operator ceiling/acceptability
   decision.

Gates the [[NATIVE-HANDLE-CARRIER]] fast-follow + [[PX8-F-CAP-41]] too.

> ### ⛔ DESIGN CONSTRAINT ON GATE REQUIREMENT 1 — added 2026-07-25
>
> **The n=3..7 harness MUST run its workers on the PRODUCT's stack (8 MiB /
> `ulimit -s`), NOT on the `crates/ken-cli/tests/` convention of
> `stack_size(256 * 1024 * 1024)`.**
>
> ⛔ **CORRECTION 2026-07-25 (adversary N2, `evt_7mve56d192pv6`) — THE STEWARD'S
> ORIGINAL WORDING HERE WAS WRONG AND IS FIXED BELOW.** I first wrote that
> threading added *"~128 KiB per recursive lowering frame."* **That is not what
> was measured.** The bisect measured a **~128 KiB shift in the TOTAL minimum
> stack** for a test driving bracket depths 2–3, across an **unknown number of
> recursive frames `k`**. Per-frame growth is ≈ `128/k` KiB and **`k` was never
> measured** — the recursion is over the *expression tree*, not the bracket depth,
> so `k` is not 2 or 3 either.
> ⚠ **Do not launder a total into a per-frame figure**, and do not claim it "errs
> safe": extrapolating at 128 KiB/frame is pessimistic only if `k > 1`, which is
> itself a claim about an unmeasured quantity. ⇒ **The honest statement is that
> per-frame growth is UNKNOWN.**
> ⭐ **`k` IS THE THING TO MEASURE** before this axis carries any weight in the
> analytical model — a per-frame number is exactly the operand a scaling
> projection consumes.
>
> **Why, measured:** B2A-C's correspondence threading shifted the total minimum
> stack by ~128 KiB, and CI went red on
> `ken-cli::px8ta_oriented_subcontinuation
> public_two_three_level_brackets_finish_and_release_lifo` with
> `fatal runtime error: stack overflow` (PR #940). Bisected at 64 KiB resolution:
>
> | commit | minimum passing stack |
> |---|---|
> | `70bd2c74` (base) | **> 1984 KiB, ≤ 2048 KiB** — cleared libtest's 2 MiB default by **< 64 KiB** |
> | `08633b3c` (candidate) | **> 2112 KiB, ≤ 2176 KiB** — did not fit; SIGABRT |
>
> ⇒ The remedy (`bb2242e8`) wrapped that one test at the repo's conventional
> **256 MiB** — correct and in-scope, but it means **that test can never detect
> stack growth again**, and every other `ken-cli` test was already blind for the
> same reason (5 pre-existing 256 MiB sites).
>
> ★ **The fleet spent its only accidental sentinel on the stack-growth axis, on
> the WP chain chartered to bound growth on that axis.** Acceptable there; ⛔ NOT
> acceptable in the harness that DISCHARGES the gate.
>
> ⚠ **A 256 MiB harness would report wall-time, RSS and internal counts while
> silently tolerating stack growth that kills the product at 8 MiB — i.e. it
> would measure the wrong machine and pass.** Same shape as
> `verify-the-mechanism-not-a-proxy`: the numbers would be real and the
> conclusion wrong.
>
> ⇒ **Stack exhaustion is a THIRD growth axis** alongside compile wall-time and
> peak RSS, and it is the one the current test convention hides. The harness
> should report it explicitly per n.
>
> ⛔ **AND IT NEEDS `k`, NOT A TOTAL.** The axis is only usable if the harness
> reports **recursion depth `k` alongside peak stack per n** — otherwise it
> produces another total with no per-frame operand, which is the exact defect
> corrected above. ⇒ Instrument `k` (max recursive depth reached in the lowering)
> as a first-class output of the n=3..7 harness.

## ⛔ ARMED §5a RESEARCH-CONSULT TRIGGER — the count of record

**Steward holds the authoritative count** (steward playbook §5a duty 1). The
Architect re-derives its own count across compactions; **on any disagreement
this line wins.** Re-read this line on every hard-stop.

> ### ✅✅ THE §5a-ii PREDICATE IS NAMED — Architect, 2026-07-28 (`evt_55bzwnhjpwjrs`)
>
> ⭐ **The predicate is `executable-boundary closure`.** `#9` and `#10` are **two
> observations of ONE predicate**, not two predicates. Recorded here because the
> ruling was issued in-thread and an in-thread ruling is not a durable
> deliverable.
>
> > *"A generated-function boundary is closed only when every admitted transfer
> > has both (1) a stable static schema for unit ownership, slot layout,
> > lifetime, and call ABI, and (2) executable constructor / projector /
> > eliminator semantics for the runtime bits carried by that schema. **A declared
> > or validated boundary description is not itself an executable value-transfer
> > mechanism.**"*
>
> | observation | which half failed |
> |---|---|
> | **`#9`** | ⛔ the **whole** predicate — one-function-per-origin had no configuration-independent executable representation contract at all |
> | **`#10`** | ⛔ the **dynamic half, after the static half landed** — `B2O`+`B2R` gave ownership, population, slot order/width and declared ownership, but no *meaning* for `ValueWord`/`ResultWord` bits and no emitted way to inspect a dynamic aggregate |
>
> ⇒ ⭐ **`#10` is `#9` recurring one representation layer down.** It is the second
> observation that forced the predicate to be stated with **both** halves.
> ⚠ **The defect was crediting the static half as "executable"** — which is
> exactly what this node's own `#9` discharge text already confessed when it said
> the discharge *"over-credited `B2R`"*.
>
> ### ⛔ WHAT THIS RULING DOES **NOT** DO — read before reaching for a recut
>
> **Structural closure:** retain `B2A-S`, `B2O`, `B2R`; require the executable
> half supplied by **`B2V` and `C1`**; let **`B2F`** consume that closed edge.
> ⭐ **That closure is ALREADY LANDED** — `C1` merged at PR #1156.
>
> ⛔ The ruling **names the already-executed recut.** Verbatim: it does *"not
> reopen `#9` or `#10`, add a prerequisite, reset the count, or hold the `B2F`
> build."*
>
> ⇒ ⭐ **No recut frame is owed.** This node's `#10` entry said *"if named, the
> RECUT FRAME IS THE STEWARD'S to author"* — that obligation is **discharged by
> the recut having already happened**, not by authoring a second one. ⛔ Do not
> read the naming as a trigger to re-cut work that is merged.
>
> ⚠ **Count unchanged: 11.** ⛔ `#10` is still **not** a fourth symptom-inventory
> entry; `ENTRIES` stays **3** and the next predicate check stays the **6th**.

> ### ⭐ SYMPTOM INVENTORY — armed 2026-07-24 (operator-directed)
>
> **The Architect appends one line per hard-stop, before it rules; at the 3rd
> entry it must answer whether the entries share a predicate** (architect
> playbook §1b, steward §5a-ii). This exists because *this* chain ran to 33
> hard-stops with nothing holding the pattern across them.
>
> ⛔ **BOTH counts below are on the LIVE RECUT chain. The original pre-recut
> chain is FROZEN at 33 hard-stops — do NOT resume that count**, and do not
> read a `#36` anchor from any older prose (the briefing carried one until
> 2026-07-25; an armed line with a stale anchor reads exactly like a working
> one). `RT-FNSPLIT-B2O` closed with **no hard-stop**, so neither count moved:
> a clean WP never advances them.
>
> ```text
> SYMPTOM INVENTORY (append only; never rewritten)
> NEXT PREDICATE CHECK = 6th entry   (3rd is CONSUMED — answered at entry 2)
> ENTRIES = 3  ← ANSWERED AT ENTRY 2; entry 3 gives the CAUSE. See below.
> 1. retained body selection — keyed on cloned RuntimeExpr pointer identity
> 2. lower_expr re-lowers each retained closure body AT EVERY CALL SITE, in
>    that call site's whole configuration (core.rs:4214/4229 clone the body
>    into Lowered::Closure; :4302 lowers the clone against
>    args ++ captures ++ env)  — keyed on runtime configuration
> 3. ⭐ THE CAUSE, not another instance: there is NO static key in scope at the
>    construction site to key on. lower_expr (core.rs:3847) takes only
>    (builder, expr: &RuntimeExpr, env: &[Lowered]); its RuntimeExpr::Closure /
>    LexicalClosure arms (:4211/:4226) build Lowered::Closure with
>    body: (**body).clone(). The planner walk preallocates StaticOriginId
>    (semantic_ir.rs:194/:231) but the LOWERING walk is an INDEPENDENT
>    traversal of the same source with NO carried correspondence — so the
>    occurrence being lowered has no static name. Every prior attempt reached
>    for a dynamic surrogate (pointer, then configuration) BECAUSE the static
>    one is absent, not because it was mis-chosen.
> ```
>
> ### ⭐ CLOSURE LEDGER — which entries are DISCHARGED (Steward bookkeeping)
>
> The inventory above is append-only and is **not** rewritten as entries close.
> This block is the closure record. ⛔ **Read it before claiming the parent is
> done: the parent stays `active` until all three are closed.**
>
> ```text
> entry 1  retained-body selection keyed on cloned RuntimeExpr pointer identity
>          ✅ CLOSED by RT-FNSPLIT-B2A-S — PR #944, origin/main = 145fe915
>             (2026-07-25). Selection is keyed by the planner's static origin;
>             a retained closure carries no term.
> entry 2  lower_expr re-lowers each retained body PER CALL SITE in that call
>          site's whole configuration
>          ⛔ OPEN — assigned to RT-FNSPLIT-B2F, which is BLOCKED behind THREE
>             inert prerequisites. Untouched by B2A-S and B2A-C; all three ring
>             seats and the Architect said so independently.
>          ⛔ This entry now takes FOUR nodes, not one:
>             RT-FNSPLIT-B2O -> RT-FNSPLIT-B2R -> RT-FNSPLIT-B2V -> B2F.
>          ⛔ RE-CUT 2026-07-27: the 4th node is RT-FNSPLIT-C1, not B2E.
>             B2E is RETIRED (closed); C1 replaces it and lands an EXECUTABLE
>             producer -> validator -> eliminator edge rather than an inert
>             ledger. Sequence: B2O -> B2R -> B2V -> C1 -> B2F. The entry stays
>             OPEN; the count of record stays 3 and the next predicate check
>             stays the 6th -- the re-put added no hard-stop.
>          ⭐ B2V was inserted 2026-07-25 by Architect ruling evt_28cnmxf6ncghn
>             on hard-stop #10. ⛔ #10 IS NOT A FOURTH SYMPTOM-INVENTORY ENTRY —
>             the ruling classified it explicitly as ANOTHER MISSING PREREQUISITE
>             UNDER THIS ENTRY, the same functionization obstruction ONE
>             REPRESENTATION LAYER BELOW #9. Do not widen the headline inventory;
>             ENTRIES stays 3 and the next predicate check stays the 6th.
> entry 3  THE CAUSE — no static key in scope at the construction site
>          ✅ CLOSED by RT-FNSPLIT-B2A-C — merged 2db29abe (2026-07-25).
>             plan↔lowering occurrence correspondence now exists.
> ```
>
> ### Adversary triage on the entry-1 close — both findings dispositioned
>
> The adversary hunted landed `145fe915` and reported the **mechanism held**;
> full disposition is in `RT-FNSPLIT-B2A-S.md`. Two carries for whoever frames
> next:
>
> - **P1 (admissibility premise of `B2A-C` now falsified) — ✅ no action.** The
>   ruling was re-derived on `B2A-S`'s own terms **before** the code, at framing
>   (`evt_1jdh8pn8y96z`) and again by ruling (a) (`evt_2eap269sgnavm`). The
>   replacement ground is **atomicity** — the retained-body carrier left in the
>   same diff that made the origin a selector, so the two authorities never
>   coexisted. ⭐ **A conditional ruling does not re-earn itself in the WP that
>   falsifies its condition — check for the re-derivation, do not assume it.**
> - **P2 (mechanize AC-5 residual arm 1) — ⚠ carried to `B2F` as an Architect
>   ruling candidate, NOT as proposed.** Review-enforcement does decay and arm 1
>   is the arm that matters, but the proposed container-spelling blacklist is the
>   same forbidden-spelling class retired during `B2A-S`. Mechanize it as an
>   **allowed inventory with a positive control**, or leave it review-enforced
>   and say so.
>
> ⚠ **Entry 2 is the one that carries the growth verdict**, so no per-function
> or scaling claim is established until B2F lands. Neither B2A-C nor B2A-S
> installs a target function, calling convention, dispatch, or emitted-code
> authority — do not read their closure as progress on the operator's
> per-function growth gate.
>
> ### ⛔⛔ ENTRY 2 WAS RE-SLICED AT HARD-STOP #9 — it now takes THREE nodes
>
> **Architect ruling `evt_842spc7t6js1`, addendum `evt_t4fykh52ncb`, on research
> advisory `evt_531c4k52mshrn` (2026-07-25).** `RT-FNSPLIT-B2F` was ruled **not
> buildable as one unit**: one closed callable unit per static origin requires a
> **stable executable representation contract for every value crossing a
> generated-function boundary**, and the current plane has none — emitted
> signature `(pointer) -> i64`, `Lowered` a compile-time specialization lattice
> rather than a value domain, `CaptureSlot` an ordinal, `PredeclaredFunction`
> with no signature or convention.
>
> ```text
> entry 2 now closes only after ALL THREE, in order:
>   RT-FNSPLIT-B2O  static body ownership — total validated
>                   occurrence -> PredeclaredFunction mapping        (INERT)
>   RT-FNSPLIT-B2R  representation + call-ABI contract               (INERT)
>   RT-FNSPLIT-B2F  the atomic live switch                          (LIVE)
> ```
>
> **Ownership precedes representation** because the ownership mapping *defines
> the cut*, and the cross-cut value population cannot be enumerated before the
> boundary is known.
>
> ⭐ **The prerequisite is NOT "one universal boxed `Value`".** That framing came
> from the two options the Steward routed; the advisory supplied a **third** that
> the ruling adopted, and it is materially smaller: pin the boundary *contract*,
> which may be satisfied by a family of statically typed per-origin layouts.
>
> ⛔ **Bounded coexistence was REJECTED and `AC-1`/`D6` are NOT amended.** Not
> because it is unsound — it is a known-sound architecture — but because
> retaining whole-configuration specialization for the aggregate complement
> **preserves the exact super-linear authority this chain exists to remove**, and
> because "scalar on this walk" is an observation about current values, not a
> static classification theorem. The newly grounded authority is **path-dependent
> and diffused through producer/eliminator-frame machinery**, so it cannot be
> bounded honestly by a call-site allowlist.
>
> ⭐ **The atomicity is what converted "hard" into "unsatisfiable as framed":**
> the one buildable increment — functionize scalar-parameter origins, keep
> specialization for the rest — is *exactly* what `AC-1` and `D6` forbid. Two
> correct requirements in tension, not a defect in either. The all-origin shape
> and the atomic live switch **survive unchanged**; only their missing dependency
> is made explicit.
>
> **P2 is now closed and NOT adopted** (superseding the carry above): the ruling
> directs **no container-spelling blacklist**. That arm stays review-enforced
> unless the prerequisites' closed ABI/body-owner structures admit an
> allowed-inventory structural pin **with a positive control**.
>
> ## ⭐ ENTRY 3 IS WHY THE PREDICATE KEEPS BEING VIOLATED
>
> Entries 1 and 2 are two *surrogates*; entry 3 is the *vacancy* that forces a
> surrogate. Hard-stop #5 ruled the origin **carrier** onto planner records and
> both the Architect and the Steward treated "the carrier exists" as sufficient.
> It is not: a field on a planner record does not put a value in `lower_expr`'s
> scope. ⇒ **The chain's next real deliverable is the plan↔lowering occurrence
> CORRESPONDENCE, not another consumer of an origin nothing produces.**
>
> ★ Steward frame defect, stated plainly: `RT-FNSPLIT-B2A-S` was sized by what
> **consumes** the tag (the selection table) and never by what must **produce**
> it at the construction site. Same class as
> `a-required-deliverable-can-transitively-require-the-frames-excluded-scope`.
>
> ## ✅ RESOLVED 2026-07-25 — entry 3 has an owner, and the RE-SLICE IS RULED
>
> **Census `TOTAL` + injective** (`runtime-implementer`, `evt_4tqj93ctj24z2`,
> type-driven over the `RuntimeExpr` declaration, not a grep) ⇒ **Architect
> ruled the third option CONFIRMED** (`evt_1jdh8pn8y96z`): correspondence
> **transports an already-settled fact to a site where it is out of scope** — it
> does not choose static identity, invent an identity space, or define behaviour
> for an unplanned occurrence. So it is **production plumbing, NOT Q3
> functionization authority**, and the Q3 atomic boundary stays intact.
>
> ```
> B1R → RT-FNSPLIT-B2A-C → RT-FNSPLIT-B2A-S → RT-FNSPLIT-B2F
>       correspondence      selection           functionization
>       (entry 3)           (entry 1, atomic)   (entry 2, atomic Q3)
> ```
>
> ⛔ **STATE THE INVENTORY CLOSURES SEPARATELY (ruled).** Closing the cause's
> transport seam is **not** closing either downstream symptom:
>
> | entry | closed by |
> |---|---|
> | **3** recoverability vacancy (the CAUSE) | `RT-FNSPLIT-B2A-C` |
> | **1** cloned-body / pointer identity | `RT-FNSPLIT-B2A-S` (complete selection) |
> | **2** whole-configuration specialization | `RT-FNSPLIT-B2F` (atomic switch) |
>
> ⚠ **The census carried a deliberate scoped `could_not_determine`:** the
> *partition* (which occurrences are machine-only) is program-dependent and not
> statically enumerable. **Totality is determined; the enumeration is not.**
> ⛔ Do not read "TOTAL" as "and here is the partition", and do not let any frame
> enumerate a guessed machine-only subset.
>
> ★ **The ring retracted its own convergence read, unprompted:** it had inferred
> "collapses into B2F" from the **size** of the carrier when the deciding
> property was its **totality**. **Size is not a boundary discriminator.**
>
> ## ⛔ DO NOT WAIT FOR THE 3rd ENTRY. Entries 1 and 2 ARE THE SAME PREDICATE.
>
> Entry 2 is *"a dynamic property naming static code"* — the chain's predicate,
> verbatim — and it is **also identical to HELD-CHAIN entry 1** (whole-
> configuration specialization). The runtime-implementer said so itself:
> `lower_expr`'s per-call-site re-lowering *"is simultaneously symptom-inventory
> entry #1 and whole-configuration specialization itself."*
>
> ★ **So the mechanism has already produced its answer at entry 2, and holding
> the question until a 3rd entry would be mechanical compliance defeating the
> mechanism's purpose.** The inventory exists to detect a shared predicate across
> stops, not to reach a count. **Two entries that reduce to one predicate is the
> finding.** "Check at the 3rd" is a *floor on when to ask*, never a *bar on
> answering early*.
>
> ⇒ **The recut did NOT eliminate whole-configuration specialization; it built a
> closed plane BESIDE it and never connected them.** Boundary A closed the plane,
> B1/B1R filled in its semantic material — and `core.rs:204` still drops the
> whole thing on the floor, leaving the original specializing inliner as the only
> live emission path. **That is why B2a is a construction and not a port**, and
> it is the same defect the recut was chartered to remove, one level up.
>
> ⚠ **This also means the frozen held-chain count of 33 was never as separable
> from the recut as the re-arm assumed.** The recut replaced the *plan*, not the
> *emitter* — and the emitter is where the predicate lives.
>
> ⛔ **Seeded, not empty of history.** The *held* chain's four entries are
> already known and were the input to the recut — keep them visible so a new
> entry can be compared against them rather than discovered fresh:
>
> ```text
> HELD CHAIN (closed, retained as the worked example)
> 1. whole-configuration specialization        — keyed on runtime configuration
> 2. vector-shaped / flattened residual keys   — keyed on residual contents
> 3. recursive Debug serialization as identity — keyed on serialized state
> 4. helper identity coupled to env/control/layout contents — keyed on contents
> PREDICATE (named at the recut) = a dynamic property must not name static code
> ```
>
> ★ **A recut-chain entry that reduces to the SAME predicate is not a new
> defect — it is evidence the recut is incomplete.** Say so rather than ruling
> it.

```text
RECUT CHAIN (live, from kickoff evt_2kgfmmeeh2x7w, 2026-07-24)
hard-stop count    = 8   ← #6's PULL FIRED AND WAS CONSUMED. NEXT PULL = #9.
                            ⛔ #9 IS THE VERY NEXT STOP AND IT FIRES A PULL.
                            #7 and #8 both went straight to the Architect.
  ⚠ THIS LINE READ "3" UNTIL 2026-07-25 AND WAS STALE BY TWO STOPS. Stops #4
    and #5 both happened and neither was posted here, so the authoritative
    count silently disagreed with reality in the one place designated to win
    that disagreement. Corrected by the Steward at the B2a kickoff.
    ★ The count is only authoritative if it is written at the stop, not at the
    next seam that happens to re-read it.
  #1 = Architect amendment ruling evt_6dpb96kn1583f (2026-07-24) — Phase 1's
       held-checkpoint premise is FALSE; census returns could_not_determine;
       empirical gate moves to the recut in two closed boundaries. Frame
       amended by the Steward in response. NO research pull due (< #3).
  #2 = Boundary B static-to-semantic bridge, raised by runtime-implementer on
       WIP d4df9278, ruled by the Architect at evt_2jt1s5r7c1g2z (2026-07-24) —
       Boundary A's plan is closed and constant-width but retains NO static
       helper -> semantic-body association, while the retained emitter still
       allocates FuncId from PartitionSemanticStateKey (vectors/strings/
       recursive keys). Ruling: extend A with an OUT-OF-LINE semantic
       descriptor plane keyed by the existing planned node/edge IDs; planned
       IDs are the sole code identity; no discovery-order or hashed-key
       fallback. NO research pull due (< #3).
  #3 = Boundary B grounding found GENUINELY UNREPRESENTED activation-
       independent semantic classes, raised by runtime-implementer at
       evt_21yr288qkpb92 on clean checkpoint ed54b17e (2026-07-24).
       SourceKont is not uniformly R (PartitionSourcePrefixKey carries LetBody,
       ApplyRecursorSelection, UnwindRecursorSegment, checked recursive/IH
       returns, selected-case return, terminal steps — these transform
       value/control and may own a body, so they are neither R nor
       authority-only edges). ProducerKont is not classifiable by action-name
       mapping (OrientedInvocationReturn, CheckedComputationalIHReturn,
       ScopeBodyReturn have independent control semantics; A has only R/W/T/C,
       and R is ruled to own no body). SourceArm bodies lose exact occurrence
       identity before reserve (cloned RuntimeExpr). Definition scheduling is
       still state-owned per whole semantic key.
       ⇒ This hits the ESCAPE HATCH in the Architect's hard-stop-#2 ruling:
       "add an explicit planner node/transition kind and RETURN BOUNDARY A for
       amended census and fresh review." The ring correctly refused to overload
       R/W/T/C, assign by discovery order, or retain first-activation body
       selection.
       ⇒ RESEARCH PULL FIRED (§5a). Architect ruling is gated BEHIND the
       advisory, at the implementer's own request.
  #4 = B2a hard-stopped BEFORE ANY CODE, raised by the Runtime ring at
       evt_6fm274bx4q6hb (2026-07-25). The Architect classified the cause as a
       REPRESENTATION DEFECT IN LANDED B1 rather than B2a plumbing
       (evt_7d5v99mh8n9cc) and ruled a RECUT AHEAD OF B2a — B1 counted
       occurrence-local semantic material it never stored. ⇒ RT-FNSPLIT-B1R was
       framed and B2a was flipped `active` → `ready` behind it.
       NO research pull due (< #6).
  #5 = B1R could not add the origin CARRIER without editing lowering/core.rs,
       raised at evt_3sx56kzx7z9q, Architect confirmed evt_37sc5gv2yfxr8
       (2026-07-25). Ruling: the carrier moves to B2a as D0 so one authority
       replaces another in a single reviewable diff, rather than two
       authorities coexisting across two WPs. Architect also settled the seam
       (widen StaticOriginId only to pub(in crate::cranelift_backend); field
       named static_origin; no mod.rs-only partial carrier).
       NO research pull due (< #6).
  #6 = B2a hard-stopped BEFORE ANY EDIT on a full fixed-input audit, raised by
       runtime-implementer at evt_3xzv4xn77na0d, leader confirmed
       evt_34y9pnbs8r330 (2026-07-25). 8 of 9 anchors held; the Retain line
       "exported root + bounded deferred Cranelift functions" is FALSE OF THE
       BASE. Steward independently re-verified all three deciding measurements:
         - partition.rs ABSENT from main; PartitionWorkItem = 0; work_?item = 0.
           They exist only on preserved/wp-...-b077eb7a, which is NOT an
           ancestor of main (merge-base 8ebe370a).
         - production lowering/ has exactly ONE FunctionBuilder::new
           (core.rs:140) and ONE define_function (core.rs:202); every other
           site is under core/tests/.
         - planner<->emitter coupling is one symbol: plan_static_transition_graph,
           built at core.rs:35, dropped at :204, zero refs to the plane types
           outside planning/.
       ⇒ B2a's Retain/Replace lists were INHERITED FROM THE HELD TREE and
       describe artifacts absent from B2a's base. The emitted units D2 would
       re-key DO NOT EXIST, so D1/D2 is a CONSTRUCTION (per-transition units, a
       real calling convention for the 8-field DynamicActivationFrame, and a
       persistent-store runtime, proven behaviour-preserving across the whole
       6201-line SCC) — not the behaviour-preserving port the frame asserts.
       ⛔ THIS IS A STEWARD FRAMING DEFECT, not an execution failure. The ring
       invoked the frame's own unreviewable-diff stop, exactly as instructed,
       and refused to reinterpret the deliverables to fit what was buildable.
       ⇒ RESEARCH PULL FIRED. Architect ruling was GATED BEHIND the advisory.
       ✅ ADVISORY DELIVERED evt_4w1rf45d4fkv3 (2026-07-25). GATE LIFTED.
  #7 = B2A-S hard-stopped AFTER D1-D3 landed, on grounding D4. Raised by
       runtime-leader at evt_2fvxkmfw8m1k8 (2026-07-25); implementer stopped
       clean at durable checkpoint 5c7eae26 with NO tag beside a retained body
       and no D5/D6/test work. Steward independently re-verified on origin/main
       = 70bd2c74 and CONFIRMS the measurement:
         - lower_expr (core.rs:3847) takes only (&mut self, builder,
           expr: &RuntimeExpr, env: &[Lowered]). No origin parameter; nothing
           origin-derivable in scope.
         - the ONLY two production Lowered::Closure constructions are its
           RuntimeExpr::Closure (:4211) and ::LexicalClosure (:4226) arms, both
           body: (**body).clone() — the exact carrier D4 orders removed.
           :532/:3551/:4261 are destructures, not constructions.
         - lower_expr is re-entered from source_call_state (:3542) via
           SourceMachineState::Eval (mod.rs:1962), so a threaded origin must
           also live in the state enum and both continuation enums.
       ⇒ With pointer identity, content, clone AND visit-order all prohibited by
       the frame's own D6 controls, NOTHING remains but a threaded parameter —
       which is the source-machine/static-authority scope B2A-S EXCLUDES.
       D4 is UNSATISFIABLE inside its own frame. The ring did not weaken it.
       ⛔ STEWARD FRAMING DEFECT (second in this chain, same author, DIFFERENT
       class from #6): #6 was a false premise inherited from a held tree; #7 is
       a boundary that excludes its own prerequisite. B2A-S was sized by what
       CONSUMES the tag and never by what must PRODUCE it. ⇒ Inventory entry 3.
       NO research pull due (count 7 < 9; #6's pull fired and was consumed).
       ROUTED TO ARCHITECT, gated behind ONE ring measurement: is the planned-
       origin population TOTAL over the closure occurrences reachable in
       lower_expr, including via the source-machine fallback? Totality decides
       whether the correspondence is mechanical threading (its own production
       unit) or drags in static-authority scope (collapse into B2F).
  #8 = B2A-C hard-stopped ON D3's OWN PROBE, at checkpoint 96e66c9f. Architect
       ruled evt_308azmr4cszd7 (2026-07-25); Steward CONFIRMED #8 and ruled the
       disposition = AMEND B2A-C IN PLACE (not a re-slice).
       ⭐ THE FINDING IS A CATEGORY ERROR, NOT AN ORDINAL DISAGREEMENT: one
       StaticNodeId is being made to mean two different things.
         - plan_expr's ENTRY = the first node the transfer graph schedules;
         - the expression's OCCURRENCE = the node on which
           SemanticSourceSeed::expression registered that RuntimeExpr, and from
           which its positional child-origin record is read.
       They coincide for ordinary forms and DELIBERATELY DO NOT for
       ComputationalMatch. Steward re-verified on exact 96e66c9f:
         :628  let resume = push_node(TransitionKind::SourceReturnResume, ...)
         :667  let scrutinee = self.plan_expr(scrutinee, ..., 0)?
         :672  self.expression_seed(resume, expr, &children)?   <- occurrence
         :673  Ok(scrutinee)                                    <- entry
       ⇒ Passing the scrutinee entry as the parent's child origin is a category
       error. ★ D3 FOUND IT BUT NAMED IT WRONGLY, which is the tell that the
       frame had ONE axis where it needed TWO.
       ⇒ Mechanism: plan_expr returns PlannedExpr { entry: StaticNodeId,
       occurrence: StaticOriginId } with DISJOINT consumers - transfer topology
       consumes only .entry (Boundary-A graph unchanged), source correspondence
       consumes only .occurrence. NO new node, origin, search, or arithmetic.
       ⇒ Classification (Architect): repairs the PRODUCER of correspondence, so
       it is inside B2A-C / entry 3. Entries 1 and 2 stay OPEN and the Q3 atomic
       boundary is unchanged. Architect explicitly did NOT authorize WP scope or
       the count - "Steward owns that formal call."
       ⇒ SIZE RAISED M -> L by the Steward, deliberately and in the open, rather
       than letting the unit grow silently (the #7 lesson).
       NO research pull due (count 8 < 9). ⛔ BUT #9 IS THE NEXT STOP AND FIRES
       ONE - dispatch research BEFORE the Architect rules on it.
          Independently grounded the stop on exact 7151ae58 and confirmed it
          "correct and structural". Three findings that bind the re-slice:
          (i)  A `static_origin` carrier CAN be an independent checkpoint, but
               ONLY as a COMPLETE tag-plus-SOLE-dispatch defunctionalization
               (Danvy/Millikin recognition criterion: the apply function is the
               sole point of consumption). Requires: every retained closure/work
               item carries the static tag + its dynamic env/state; raw cloned
               bodies/pointers ABSENT from that population; all applications go
               through ONE closed origin consumer; tag population = the closed
               static occurrence population, never inferred backward.
               ⇒ Honest milestone name is "DEFUNCTIONALIZE RETAINED BODY
               SELECTION", ⛔ NOT "the plane is load-bearing".
          (ii) StaticOriginId -> &RuntimeExpr is IDENTITY-CLEAN (a table indexed
               solely by preallocated id, with the borrow as payload only, does
               not use the pointer as identity) but is NOT PLANE-AUTHORITATIVE:
               it leaves RuntimeExpr owning semantics, i.e. an abstract machine
               over source terms rather than a virtual machine over compiled
               units (Ager et al.). ⇒ It closes SELECTION identity while
               deliberately leaving source-recursive semantics in authority.
               ⛔ Claiming it makes B1R's material load-bearing OVERCLAIMS.
         (iii) ⛔ Landing functionization as a LIVE SECOND PRODUCTION PATH with
               authority-removal postponed to a later slice has NO PRIOR-ART
               SUPPORT — it recreates two authorities. Functionization,
               switch-over, equivalence evidence, and old-path removal form ONE
               review boundary, unless the new path is mechanically unreachable
               non-authoritative scaffolding.
          ⚠ Prior art supports TWO coherent target shapes and does NOT choose:
             (a) closed target functions with explicit environment/frame
                 arguments (CertiCoq: after closure conversion + hoisting, all
                 functions closed in one bundle, forward-declare all, then one
                 target function per source function);
             (b) a compiled instruction/CFG machine with static labels and
                 explicit state.
          ★ The frame's "bounded deferred Cranelift functions" wording picked
            (a) BY INHERITANCE FROM A BRANCH THAT NEVER LANDED. The base
            implements NEITHER, so this is a genuinely OPEN Architect call to be
            made on merits, not carried forward.
          No local/refs/ or excluded-prototype material consulted; the held Ken
          branch was used only as repository-owned scale evidence.
NEXT RESEARCH PULL = hard-stop #11, then #15, #18, #21, …
   ⛔ CORRECTED 2026-07-25 (Steward, count of record). This line read #12 — the
      generic next-multiple-of-3 after the consumed #9. The steward playbook
      carries an OPERATOR OVERRIDE dated 2026-07-24 spelling the catch-up as
      "#11, then #15, #18, #21". The two cannot be reconciled from the dates, so
      it is settled by DOMINANCE, not by guess: a pull at #11 is REQUIRED under
      one reading and merely EARLY under the other, and early is explicitly fine
      (a cadence threshold is a floor on when to ask, not a bar on asking
      sooner). #12 is wrong under one reading; #11 is safe under both.
      ⚠ Occurrences of "#12" BELOW this line are append-only HISTORY — they
      record what was believed at the time and are deliberately NOT rewritten.
      This line is the operative anchor.
   ✅ #9 CONSUMED — raised 2026-07-25 on RT-FNSPLIT-B2F (evt_197xpdavdyrn0);
      research dispatched evt_63wjmry61vd89 BEFORE the Architect ruled, as
      armed. COUNT OF RECORD = 9. Obstruction: one-function-per-origin needs a
      uniform runtime value representation that does not exist, and building it
      is not among B2F's D1-D8. Two options were routed to the Architect —
      (i) a prerequisite unit for the representation + calling convention, or
      (ii) bounded coexistence, which requires AC-1 and D6 AMENDED.
      ✅ RULED evt_842spc7t6js1 + addendum evt_t4fykh52ncb: option (i),
         PREREQUISITE-FIRST. Coexistence rejected; AC-1/D6 NOT amended. The
         advisory (evt_531c4k52mshrn) supplied a THIRD framing that reshaped
         option (i) and was adopted: the prerequisite is a stable EXECUTABLE
         REPRESENTATION CONTRACT for every value crossing a generated-function
         boundary — NOT necessarily one universal boxed Value. B2F re-sliced
         into RT-FNSPLIT-B2O -> RT-FNSPLIT-B2R -> RT-FNSPLIT-B2F.
      ⛔ CORRECTED 2026-07-25 BY THE #10 RULING (evt_28cnmxf6ncghn) — THIS
         DISCHARGE OVER-CREDITED B2R. B2O+B2R delivered the STATIC half only:
         code ownership, unit population, slot order/width, declared ownership.
         They never defined what the bits of ValueWord/ResultWord MEAN, nor how
         compiled code inspects a dynamic aggregate. The word EXECUTABLE above
         was therefore NOT satisfied by B2R, and #10 is #9 recurring one
         representation layer down. The executable half is RT-FNSPLIT-B2V.
         ⭐ B2R is NOT defective within its declared slot-shape scope — the
         defect is in this discharge text, which credited it with a half it
         never claimed.
         ⚠ Advisory erratum evt_3k9xam3ws9pgz: its cited paths under
         crates/ken-backend-native/ do not exist (line numbers are accurate);
         real roots are crates/ken-runtime/src/cranelift_backend/{lowering/
         core.rs, lowering/mod.rs, planning/static_transition/semantic_ir.rs}.
   ⛔ #10 OPEN — raised 2026-07-25 on RT-FNSPLIT-B2F by runtime-implementer
      (evt_71d2jg83z2yt4), leader escalation evt_r7797bd7bzk3.
      COUNT OF RECORD = 10. NOT a research-pull stop: #9 consumed its pull and
      the next is #12, unchanged and armed. The Architect rules #10 unaided.
      Evidence 49e24b59 PUSHED by Steward to origin
      wp/RT-FNSPLIT-B2F-functionization (was ONE local ref); one doc only,
      crates/ byte-identical to 1e09a30a, nothing to unwind.
      Obstruction as reported by the ring, RELAYED NOT VERIFIED by Steward:
      D1+D2+D6+D7 jointly unsatisfiable inside B2F's boundary. The missing
      prerequisite is NOT B2R's frame shape (caller-env suffix probe green with
      a capture-withholding positive control) but an EXECUTABLE runtime-value
      representation plus emitted-code interface: measured Constructor (29
      Parameter transfers) and HostResult (4) cannot be represented by the
      declared 8-byte ValueWord, and lowering only constant-folds ground values
      with no dynamic aggregate projection. A fail-closed guard would reject
      ~33 of 41 measured source-valued transfers, incompatible with D6
      old-authority removal and D7 equivalence.
      ⭐ SURFACED BY THE AC-11 WIDENING (Architect evt_7ggqdk61pxzzf): 29 of the
      33 are PARAMETER transfers, the position the Steward's original AC-11
      omitted. The correction paid for itself within the hour.
      ⭐⭐ PREDICATE QUESTION IS LIVE — #9's obstruction was already "needs a
      uniform runtime value representation that does not exist", and B2R was
      the prerequisite ruled to supply it. B2R delivered a DECLARED and
      VALIDATED contract that verifies nothing about behaviour because nothing
      is emitted. Whether #9 and #10 are one predicate or two instances is the
      ARCHITECT's call to name (steward §5a-ii forbids the Steward naming it).
      If named, the RECUT FRAME IS THE STEWARD'S to author — retaining B2A-S,
      B2O, B2R, replacing only what the predicate names.

⛔ **B2a runs one stop away from a research pull.** The #3 pull is CONSUMED
   (advisory evt_rwqb8ear89wx — Danvy/Nielsen defunctionalization granularity;
   Agda TTerm / Lean FnBody / Cranelift IR as closed-IR precedent; Maranget on
   why small-n affine tables mislead). The next stop on this chain does NOT get
   an immediate ruling: it fires a research pull first, and the Architect's
   ruling is gated behind the advisory.
   ⚠ The B2a FRAME states "hard-stop count is 3; next pull is #6" — the count
   half is STALE (it is 5), the pull half is right by accident. This line wins.

⚠ BOUNDARY B1 KICKED 2026-07-24 (evt_784nkjqzzbxn) under the fork-(b) ruling;
   ring compacted, drops verified. RT-PLANNER-DIAGNOSTIC-K closed at 36dd61f6.

⚠ BOUNDARY A IS MERGED (647a2e5b, retros in) BUT THIS WP STAYS `active` —
   B1 is in flight and B2 has not started. Do not flip this parent to `merged`
   on a boundary landing; it closes only when B1 AND B2 land and the operator
   scaling gate above is satisfied.

HELD CHAIN (closed, historical — does NOT carry forward)
hard-stop count = 33            (FROZEN)
cadence           = SUSPENDED   (viability ruling 2026-07-24)
```

✅ **RE-ARMED 2026-07-24** when the recut frame was kicked to the Runtime ring.
The old count is **frozen and does not carry**: it counted the *held*
representation, and the recut replaces the machine those stops were about. The
recut chain starts at **zero** and pulls research on **every 3rd** hard-stop.

⛔ The frozen 33 is retained above as history only. #34 was raised but is
**evidence, not a ruled stop**, which is why the held count stayed at 33 rather
than advancing. Do not resume an every-3rd pull against the held
representation — that chain is closed.

**Advisories on record:** #24 `evt_5gshpmyb2ta79` · #27 `evt_7s6b3zg82n7n5` ·
#30 `evt_1stmfwh0tj5gm` · #33 `evt_3vr382mrv99pe` (requested by Architect
16:33:14Z; **transport-repaired by Steward `evt_d2b3vahe7khj`** — the request
carried no `mentions` array, so research was never notified; research ack
`evt_74gympwyk8q67`).

⚠ **Why this section exists.** This chain ran to **10 hard-stops with zero
research pulls** (operator, 2026-07-24) because the count lived only as a prose
list of fork numbers in the Steward's resume state and in a status string —
never as an armed `next pull = N` line either party re-read. The Architect's
self-trigger lapsed across its compactions and the Steward backstop lapsed with
it. A deep chain with no advisories on it is itself the tell that **both**
mechanisms have silently lapsed.

⚠ **Transport lesson, #33 (2026-07-24):** the Architect's own advisory request
is not self-routing. A request posted with an empty `mentions` array reaches
**no one** — research is a no-poll seat, so it re-oriented to "awaiting
dispatch" while the Architect sat `blocked-on-Research` indefinitely. **Steward
duty 3 is not optional: after any pull, confirm the research pane actually went
`Working`.** Delivery ≠ engagement (COORDINATION §2, §13).


## ⛔⛔ VIABILITY RULING 2026-07-24 — HOLD + REPRESENTATION RECUT (`evt_3m1g3v4m2bj51`)

**Operator-directed viability review (`evt_98j3z2n49bpg`), Architect ruling on a
research advisory (`evt_7p40c3x8cnwtm`). The hard-stop cadence STOPS HERE.**
Runtime is **held at clean `b077eb7a`** — the semantic/diagnostic checkpoint —
**until the Steward authors the recut frame.** No #34/#35 option is implemented
in the current machine.

### ⚠ THE ROOT CAUSE STATED AT THE TOP OF THIS FILE IS STALE — DO NOT BUILD ON IT

The "**inlines the whole process ITree into one Cranelift function** →
`VReg::MAX`" premise **is already fixed and gone.** At `b077eb7a`,
`core.rs:120–492` emits an exported root plus a **separate Cranelift function
per queued `PartitionWorkItem`**. **The 1,482-state / 1,525-edge red is NOT the
single-function failure.** Anyone reading this file top-down would otherwise
re-solve a solved problem.

### The ruling, in three parts

1. **Single-`Function` inlining — dead, already replaced.**
2. **Defunctionalized lowering into bounded helpers — VIABLE, and Θ(n) is
   reachable** for n nested well-bracketed scopes. Normal/abrupt control, affine
   authority, trap order, joins and cleanup add **constant-factor** node kinds;
   none forces super-linearity. A linked predecessor stack/DAG shares suffixes
   instead of enumerating paths.
3. **The whole-configuration specialization REPRESENTATION — no route to O(n)
   through more sealing.** Sealing is linear only in the graph it *receives*; it
   cannot undo state products or variable-width identity already materialized.

⇒ **The mechanism family is viable; this representation of it is not.**

### Why the representation cannot claim O(n)

One helper per distinct **composite semantic-state key**, where keys remain the
Cartesian tuple `(program point × environment × selected suffix × join/path ×
layout × control heads)` with **variable-width** members:
`PartitionSelectedScopeKey.outer_env` / selected `pending` are `Vec`s
(`partition.rs:1262–1348`); producer actions embed eliminator vectors
(`:1350–1401`); SourceArm/SourceKont keys combine residual body, env,
declaration stack, active recursion, control heads, selected state, cleanup,
field types and field maps (`:2700–3050`); `PartitionContinuationKey` carries
the exact checked join plus `field_types`/`field_map` (`:1413–1422`); the
interner exact-compares the **entire retained key** (`:3088–3148`, `:4500–4635`).

★ **Hash-consing children is necessary but INSUFFICIENT while the outer key is
still the Cartesian tuple** — it shares equal subterms, it cannot merge two
different tuples merely because their components are shared. **Θ(n) states each
carrying Θ(n)-wide data already permits Θ(n²)** descriptor, comparison, frame
and emitted-interface work.

Two invariants are absent, and both are required for an analytical O(n) claim:
**(a)** a fixed **K** helpers/transitions per static source/control node;
**(b)** a **constant maximum key/frame width**, independent of nesting depth.

### ★ What n=4 does and does not prove — the honest reading

`(states, edges) = (1482, 1525)`; `E/S ≈ 1.029` says only that the realized
graph is **nearly chain-shaped on average**. **One point cannot establish an
exponent** — it cannot separate `370n` from `93n²` from a product that only
switches on at n=5. ⛔ **We do NOT claim n=4 proves quadratic growth.** The hold
rests on **code inspection rejecting an O(n) proof**, not on curve-fitting one
datum. Both things are true at once: the growth order is still unknown, *and*
more local sealing is the wrong next move.

### Retain vs replace (permanent architecture)

**Retain** — exported root + bounded deferred functions; the useful semantic
transition categories; and **all of #24–#33's proved semantics**: exact
normal/abrupt edges, trap sequencing, affine reservation/bind/spend authority,
graph sealing, completion witnesses, the **W/T producer-wrapper vs ultimate-tail
distinction**, linked cleanup/source topology.

**Replace** — whole-configuration specialization; vector-shaped/flattened
residual keys; recursive `Debug` serialization as identity
(`partition.rs:153–163`); helper identity coupled to env/control/layout contents.

**The factored machine:** (1) static transition graph, one constant-width node
per syntax/control transition, one helper per static node/edge; (2) dynamic
activation passed through a **fixed ABI frame** — *dynamic activation identity
must not create code identity*; (3) persistent constant-width cons/DAG stores
(syntax, env extension, eliminator, selected context/lineage, source, cleanup,
continuation) — no flattened suffix/ancestry/declaration-stack/occurrence-path
in any helper key; (4) evidence attached **out of line** to stable node/edge IDs;
(5) normal and abrupt successors share one persistent continuation/cleanup graph
— mutually exclusive runtime paths must not become a static subset product.

`PartitionWorkItem` survives **only** if each item names one static transition.

### #34 is EVIDENCE, not work to finish

#34 confirms #33's semantic ruling worked: the path constructs `W = site 4`,
`T = site 2`, leaves W solely descriptor-owned, passes strict STOP, and reaches
the nested-exit resume seam. **Carry the invariant, do not patch it now.** In the
new graph this is an explicit **source-return-owned resume edge/node** — ⛔ do
**not** overload `Terminal` (which means *no continuation*, whereas this state
has a live continuation owned indirectly by an exact source-return descriptor).
Option 2's duplicate direct W is **rejected**.

### ⛔ THE FIRST IMPLEMENTATION UNIT IS A PLANNER/CENSUS RECUT — NOT #35

Generate the minimal nested family for **n = 3…7 BEFORE lowering full bodies**,
and report **by state kind**: static nodes, edges, helpers, emitted CLIF
instructions/bytes; descriptor bytes constructed/retained; exact-comparison
bytes; total and maximum frame fields; maximum static-key bytes; maximum
env/pending/path lengths.

**Acceptance needs BOTH** — empirical **bounded first differences** (use first
*and second* finite differences, not ratios) **and** structural assertions:
fixed K transition/helper nodes per static source/control node; constant maximum
key and frame schema widths in n; all graph/code/descriptor totals affine in n.
Then port semantics and rerun the table **plus** exact normal return, every
abrupt exit, trap identity/order, joins, and affine single-spend differentials.
**n=4 alone never discharges the gate.**

**Falsifier for this hold:** the unchanged representation showing, across
n=3…7, constant key/frame/env/pending/path maxima, bounded K states per source
node, and stable first differences for graph, code **and descriptor** work —
*not merely state/edge counts*.

### ✅ THE CHECKPOINT IS DURABLE — `b077eb7a` is tagged ON ORIGIN

```
refs/tags/rt-native-fnsplit-checkpoint-b077eb7a  ->  b077eb7a   (verified by ls-remote)
```

⚠ **When found, `b077eb7a` lived on ONE local branch
(`wp/RT-NATIVE-FNSPLIT-native-partition`) with ZERO refs on origin** — no copy
anywhere off this box, in a repo where `handoff-gate-compact.sh` hard-resets
branches and where a `git branch -f` would have orphaned it silently. It carries
the proved semantics of #24–#33 that **Phase 1 must measure and Phases 2–3 must
port from**; losing it would have cost the recut its reference implementation.

★ **Same failure family as the QA bound-verdict attestations:** *a workaround (or
a hold) that leaves load-bearing state on one local ref in one clone.* A "frozen"
checkpoint is only frozen if something outside this machine holds it. ⛔ Do not
delete this tag.

### Steward duties from this ruling

- **Author the recut frame.** Runtime is held until it exists. ⇒ status is
  `active` but the ring is **parked**, not building.
- **Hard-stop cadence is SUSPENDED with the machine.** Count of record stays
  **33** (#34 was raised but is evidence, not a ruled stop). The every-3rd
  research cadence resumes **only** against the recut chain; re-arm the trigger
  line when the new frame opens.

## Contract (Architect-specified — state the NEED, do not freeze the mechanism)

Per `evt_7gkn3g4tsvgb9`, the follow-up must:
- make native compilation **accept the minimal four-bracket discriminator and the
  actual SP-A-write / SP-B / SP-C programs without source contortions**;
- **bound per-function lowering growth generically** — outlining,
  continuation/function partitioning, or an equivalent owner-chosen design is
  admissible — while **preserving process semantics, effect order, trap/error
  identity, join/subcontinuation accounting, and the public native ABI**;
- **prove the boundary** with the minimal three-vs-four-bracket reproducer, then
  run the exact native SP matrices currently blocked.

**⛔ Rejected non-solutions (Architect):** test-only special cases; merely raising
Cranelift's cap; disabling a verifier/check; interpreter fallback presented as
native execution; or asking Ken authors to reshape otherwise-valid source. The
mechanism ("function splitting" etc.) is the owning team's to choose — this WP
states the need, not the design.

## Why this is its own WP (not in the PX8-SPAN-PROV fence)

PX8-SPAN-PROV's fence is buffer-span provenance (elaborator/interp/runtime/host
admission). Native codegen function-splitting / process-ITree size relief is a
distinct backend-capability concern. Per the Steward scope ruling
(`evt_7c160ej3bwz4`), PX8-SPAN-PROV Phase 2 lands its **sound, mutation-proven
mechanism** now with **honest partial-status** native conformance rows
(interpreter GREEN; native SP-A-write / SP-B / SP-C marked
**BLOCKED-ON-NATIVE-REACHABILITY**, pointing here). This WP is the named
follow-up that lifts the wall.

✅ **Architect means/layer ruling delivered (`evt_7gkn3g4tsvgb9`): out-of-fence for
PX8-SPAN-PROV, a separate Runtime backend WP** — concurring with the Steward scope
split. Size is **TBD** because the repair mechanism is the owning team's to choose
(the Contract above admits outlining, continuation/function partitioning, or an
equivalent design), not because any ruling is outstanding.

## What "done" unblocks

Once the 4-bracket wall is lifted, the deferred native conformance matrices become
runnable:
- SP-A **write** absolute native+interpreter discriminator (foreign-write reject →
  `InvalidBounds`, zero backend; own-write success control);
- SP-B per-engine combined foreign / stale / overflow / negative-offset precedence
  arms, native end-to-end;
- SP-C old / foreign / fresh **write** controls (not just freeze), native
  end-to-end.

At that point CV flips the PX8-SPAN-PROV native SP rows from
BLOCKED-ON-NATIVE-REACHABILITY to GREEN on the landed capability (a small
conformance-only follow-up fold), completing the Phase-1-locked engine matrices.

## Sequencing (Steward)

**RELEASED + `active` 2026-07-23** — this is **Track 1** of two concurrent impl
tracks (operator "the plan sgtm" + returning to two active impl teams with the
codex reseat). Kicked to the Runtime ring (design-bearing: Architect design
consult before heavy implementation). Track 2 is [[PX8-F-CAP-41]] (Foundation) —
genuinely contention-free (disjoint crates: this = `ken-runtime`/Cranelift; A1 =
`ken-elaborator`/`ken-host`; disjoint ledger; different team). Size stays **TBD**
until the ring proposes its codegen approach in the design sketch.
Sibling of [[PX8-SPAN-PROV]] (whose native conformance completion it unblocks);
root [[PX8]].
