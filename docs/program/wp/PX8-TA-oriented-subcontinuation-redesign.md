# PX8-TA (redesign) — oriented-subcontinuation machine for native nested-bracket terminal answers

> ## ▶ KICK-READY (2026-07-19) — supersedes the single-file PX8-TA brief
>
> **This brief SUPERSEDES `PX8-TA-terminal-answer-authority.md`.** That brief's
> terminal-authority-token mechanism was implemented faithfully and hit **seven
> consecutive, minimized, genuinely-distinct mechanism hard-stops**; the seventh
> (an order-inversion) falsified the Architect's own sixth-ruling convergence
> claim and was ruled a **complete-mechanism redesign**, not another incremental
> cut. The old brief is retained for provenance only; **build to THIS brief.**
>
> **Owning team:** Runtime (under the NC5 erasure-boundary ownership precedent —
> the elaborator edit is only the checked-metadata *producer*). **Reviewers:**
> Runtime QA → **Architect §14 only**. **NO CV lane** — the change is
> compiler-private native lowering; it touches **no** `spec/` or `conformance/`
> path and changes no public/observable semantics. **Size:** L (cross-seam:
> elaborator metadata producer + Runtime plan/codec/backend). **Risk:** high
> (redesign of the trusted native nested-bracket lowering path; touches the
> checked→Runtime erasure boundary; fix-forward, no rollback).
>
> **Base / branch:** cut `wp/px8-ta-oriented-subcontinuation` from current
> `origin/main @ 49755e7b50a32ed7d54e18bd7ac5fea68d0879d4` (the PX8 code
> merge-base `a9db5a17` plus two disjoint doc-only commits; never stale local
> `main`). One WP, one branch, one §14 Decision.
>
> **This is a BLOCKING prerequisite for PX8-F.** PX8-F is HELD at preserved
> `origin/wp/px8f-buffer-io-surface @ eb8e5596` and resumes only when this lands.
> Do not touch PX8-F's branch or fixtures from this WP.
>
> **Authority (fixed inputs; execute, do not re-derive or re-open):**
> - Architect **redesign ruling** `evt_7411g73ptm34p` (thread `thr_70a0am0j923da`,
>   2026-07-19 18:51Z) — the oriented-subcontinuation carrier spec + evidence bar,
>   reproduced in §3/§7.
> - Architect **fence determination** `evt_47e91w1hxm4pp` (2026-07-19 19:02Z) —
>   the cross-seam production surface + cross-seam ACs, reproduced in §4/§7.
> - Research **prior-art advisory** `evt_40tfb9c45bfs9` (2026-07-19 18:46Z) — the
>   minimal lawful carrier + discriminating evidence net, reproduced in §5/§7.
> - Operator (Pat) **dispositioned option B** (2026-07-19 ~18:58Z): proceed with
>   the Architect + Research redesign path.
>
> Questions on the mechanism → Architect; scope/process → Steward. Do **not**
> re-open the classification, resume from the preserved probe, or invent a
> Runtime-inferred substitute for the checked metadata.

---

## 1. Objective (one line)

Lower a native nested checked-bracket continuation as **one compiler-private,
owned, oriented subcontinuation segment per prompt region**, whose *semantic*
frame-composition order (the noncommuting typed eliminators) is carried
explicitly from the checked, pre-erasure boundary and is **kept strictly
independent** of the *control* return-delimiter order — so terminal-answer
authority mints only when both projections are exhausted at the checked root,
and a returned aggregate owned by an outer bracket is never misdecoded as a
malformed process `ExitCode`.

## 2. Why a redesign — the two-axis contradiction (fixed input, do not re-open)

The predecessor mechanism carried a single terminal-authority token and then, in
successive hard-stops, a producer-hole marker, a phase-typed unwind, a
`SelectedCaseOccurrenceId` occurrence-cut, and a marker-local region stack. Each
closed one contradiction and narrowed the trace; the sixth ruling claimed the
region relation was "total and inductive… no seventh hard-stop expected."

The preserved probe **falsified that in 14 minutes** with an **order-inversion**
that no ownership assignment can repair. On the real two-level reaching path:

- dynamic return / control-delimiter order: `o0 → o4 → o3 → terminal`;
- same-occurrence executable-layer placement: `p0/o0 → p1/o4 → p2/o3`;
- the only **type-correct** value-transformer composition: `p2/o3 → p1/o4 → p0/o0`.

These eliminators **do not commute**: `p0` maps `ITree → ExitCode`; the next
layer `p1` still requires `ITree::{Ret,Vis}`, so same-position execution traps.
The lesson (Architect, confirmed by Research against delimited-control prior art):
**exact occurrence ownership can be wholly correct while exact same-position
execution is ill-typed.** Marker ownership + occurrence cancellation + parent
return order form a total *ownership/control-return* relation **only**; they do
**not** determine semantic continuation orientation. Those are two independent
axes, and the present execution representation conflates them. That is the
representational gap this redesign closes.

**Evidence preserved (do not resume from either; they are witnesses):**
- clean WP hold — `origin/preserved/px8ta-hold-9a41deb4 @
  9a41deb41bdfeb048ceb885c003b4faed3badd0c` (the sixth-ruling implementation,
  clean, green fixed-point, red public discriminator).
- causal probe — `origin/preserved/px8ta-region-splice-probe @
  b38de27ba97427c881ec1ba5ac16e7baab24071e` (the seventh-ruling probe: exact
  occurrence IDs on selected returns, H3/H4/H5 no longer flattened, delimiter-local
  K — surfacing the order-inversion decisively). Its debug discriminator prints
  `splice return o0 local=p0`, then `o4 local=p1`, then `o3 local=p2`, then
  `p1/o4 value=ExitCode::Failure` against `ITree`.

## 3. The required mechanism — oriented-subcontinuation machine (Architect `evt_7411g73ptm34p`, fixed input)

The lawful design is **one compiler-private owned, oriented subcontinuation
segment per prompt region**, with **two distinct projections that must never be
collapsed**:

1. **Semantic projection** — an *oriented* sequence of noncommuting computational
   frames in **checked composition order**.
2. **Control projection** — the producer-hole / selected-return spine and
   occurrence ledger governing dynamic ownership, restoration, and one-shot
   crossing.

> An occurrence identity proves **which control occurrence owns** a frame or
> witness. It does **not** select that frame's **execution position**.

### 3.1 Carrier (minimum fields — Architect)

The redesign carrier must contain, at minimum:

- exact producer-hole identity and exact parent region / root;
- an **input answer interface** and **output answer interface** for the whole
  segment;
- an **oriented sequence of frames**, each carrying its exact input/output answer
  interfaces and its computational payload;
- an exact **occurrence ledger** mapping each frame to one delimiter/root witness
  **for validation and bijection only**;
- the existing exact cursor / activation evidence needed at splice;
- one **affine splice/resume capability** and explicit **open/consumed** state.

The answer interfaces are **compile-time checked descriptors** — *not* owned
`Lowered` values, runtime constructor probes, scalar tokens, or source-syntax
snapshots. They are derived at the checked continuation/case boundary **before
erasure** and transported through compiler-private metadata (§4). Runtime
lowering may **validate** those interfaces but may **not invent** them from
`ITree`, `Result`, `ExitCode`, provenance, occurrence, or trial execution.

### 3.2 Composition (structural — Architect)

Define composition structurally. For segments `S : A → B` and `T : B → C`, splice
is defined **only** as the one-shot oriented composition `T ∘ S : A → C`,
preserving the stored frame order and consuming both capabilities. In the reaching
case, the semantic sequence is `p2 → p1 → p0` **regardless of** the delimiter
traversal `o0 → o4 → o3`. Endpoint mismatch, wrong orientation, stale ownership,
or reuse **rejects before CFG emission or any consumer runs**.

### 3.3 Projection discipline (Architect)

- The **control projection remains control-only.** It may cross `o0 → o4 → o3`,
  restore selected cursors, and validate the frame↔delimiter bijection; it must
  **never apply a computational frame merely because that delimiter is next**.
- Conversely, the **semantic segment may apply a frame only at the next checked
  answer interface**; it must **never choose ownership or a return target**.
- Both projections belong to the **same owned region object** so they cannot
  drift into independently inferred lists.

### 3.4 Terminal-answer authority (Architect)

Terminal-answer authority may mint **only** when **all** hold:

- the **semantic segment is exhausted** at the exact checked root answer interface;
- the **dynamic region/return spine is exhausted** at the distinguished root;
- the **affine splice capability has been consumed exactly once**; and
- the exact **outer cursor/root site still agrees**.

## 4. The cross-seam fence (Architect fence determination `evt_47e91w1hxm4pp`, fixed input)

**PINNED: the fence is cross-seam, still compiler-private, and Runtime-led.**
The answer-interface descriptors **cannot** be derived from the erased Runtime
graph and **must** be derived before erasure and transported through **new
compiler-private checked→Runtime metadata.** The decisive facts:

1. `CheckedCoreMatchView` still carries the checked motive, parameters, indices,
   and computational-recursion classification. At erasure,
   `checked_constant_motive_result_type` can recover an exact checked result type
   **before that information is discarded.**
2. The erased `RuntimeComputationalMatchCase` carries only `constructor`,
   `argument_binders`, `recursive_positions`, and `body` — **neither** the checked
   motive **nor** a frame input/output answer interface. Constructor identity,
   recursive positions, Runtime value shape, provenance, and trial lowering are
   **not substitutes** for that missing checked fact.
3. The existing `NativeJoinPlanV1` **proves the transport route already exists**:
   `erasure.rs` derives a checked result type + exact occurrence path, serializes a
   compiler-private plan into `RuntimeMetadata.checked_core.metadata`, and
   `cranelift_backend.rs` consumes it. **But** `NativeJoinAnswerKindV1` is
   deliberately only `Int | Bool | StructuralNat | ExitCode` — scalar CFG joins,
   **not** the heterogeneous `ITree → … → Result/resource → ExitCode` frame
   interfaces required here. `ScalarMergeKind` is **not** extensible authority for
   this redesign; **do not widen its enum.**

### 4.1 The new metadata (Architect — non-normative representation)

Require a **new exact oriented-subcontinuation plan**, emitted while the checked
case/continuation boundary is still present, carrying:

- a plan-owned table of **canonical checked answer-interface descriptors**, or
  stable IDs into that table;
- one exact **segment/site identity** bound to declaration plus checked occurrence;
- for **every semantic frame**: exact input-interface ID, output-interface ID, and
  its **position in checked composition order**;
- the segment's exact input/output interface IDs;
- exact **Runtime occurrence markers** consumed two-way by native lowering;
- prompt/parent and occurrence-ledger identities needed by the control projection.

**A bare type hash is not authority.** Canonical checked bytes (or an equivalently
closed plan-owned descriptor) are authority; hashes/fingerprints may validate
transport and erased-frame correspondence **only**. The backend may validate an
exact plan against the erased `RuntimeComputationalMatch`, but may **not mint**
endpoints or semantic order from constructor names, Runtime shapes, frame
fingerprints, occurrence order, or observed values.

### 4.2 The minimum honest production surface (Architect — this IS the fence)

This is the same architectural boundary as the checked scalar-join plan,
generalized to typed oriented segments rather than widening its scalar enum:

- `crates/ken-elaborator/src/checked_core.rs` — **only if** a checked
  endpoint/segment view must be exposed.
- `crates/ken-elaborator/src/erasure.rs` — derive and emit the plan **before
  erasure**.
- a compiler-private **plan/codec** and exact **marker surface** in `ken-runtime`
  (prefer a **dedicated module** plus the **minimal `ir.rs` marker**).
- `crates/ken-runtime/src/cranelift_backend.rs` — validate, consume, compose, and
  enforce one-shotness.
- exhaustive Runtime-expression **visitors/validators** and focused **tests**
  required by the marker addition.

Ownership remains **Runtime** under the existing **NC5 erasure-boundary ownership
precedent**, with the elaborator edit serving **only** as its checked-metadata
producer. It authorizes **no** kernel, public source syntax, public Runtime
semantics, host ABI/wire, spec, conformance, or PX8-F fixture change.

## 5. Prior-art grounding (Research advisory `evt_40tfb9c45bfs9` — reference, not a mandate)

Research confirmed the gap against established delimited-control representations,
all of which make the missing axis explicit (consult only as understanding; do
**not** vendor or copy — implementer builds from this brief, not from refs):

- **Dybvig–Peyton Jones–Sabry** — a continuation is a typed generalized sequence;
  a captured subcontinuation is an **oriented segment with input/output
  endpoints**; append is defined only when the intermediate endpoint matches, and
  prompt splitting is a separate operation.
- **Hillerström–Lindley–Atkey** — the generalized-continuation machine keeps a
  pure continuation inside each handler frame; return drains the pure frames first,
  so handler nesting and value-transformer composition are linked but are **not one
  traversal order**.
- **OCaml 5** — captured continuations are **one-shot stack segments** linked on
  resume, not a rediscovered flat frame list.
- **WasmFX** — a continuation carries **input and result stack types** and
  resume/bind consumes it **one-shot**.
- **Unison** (`splitCont`/`repush`) — splits at a prompt while building a captured
  continuation and later reconstructs its orientation; walking markers is **not**
  "apply every frame now."

Research's **minimal lawful carrier** (the five points it distilled — consistent
with §3.1): (1) exact producer-hole/prompt + parent-region identity; (2) a
structural oriented sequence of computational frames; (3) checked input/output
answer interfaces; (4) an occurrence ledger linking each frame to its delimiter
witness **for validation/bijection, not as execution position**; (5) a one-shot
state/token consumed by splice/resume. Bottom line: `SelectedCaseOccurrenceId` is
necessary ownership evidence, but the missing lawful evidence is a **typed/oriented,
one-shot subcontinuation segment**.

## 6. Mandated deliverable outline (each section ends in a concrete choice)

Author as a single Runtime-owned WP spanning the elaborator metadata producer and
the Runtime plan/codec/backend. Deliverable sections, in order:

1. **Checked answer-interface descriptor + producer (elaborator).** Define the
   canonical checked answer-interface descriptor (§4.1) and derive the oriented
   frame chain + segment endpoints at the checked case/continuation boundary in
   `erasure.rs`, using `checked_constant_motive_result_type` /
   `CheckedCoreMatchView` **before** erasure. Concrete choice: state the exact
   descriptor representation (canonical checked bytes vs. closed plan-owned
   descriptor table + stable IDs) and the exact `erasure.rs` emit site; expose a
   `checked_core.rs` view **only if** strictly required, and say why.
2. **Oriented-subcontinuation plan + codec (Runtime, dedicated module).**
   Introduce the compiler-private plan type (the §4.1 fields) and its
   serialize/deserialize codec into/out of `RuntimeMetadata.checked_core.metadata`,
   as a **new** plan alongside `NativeJoinPlanV1` — **not** a widening of
   `NativeJoinAnswerKindV1`. Concrete choice: name the new module and plan type;
   assert the scalar-join plan is untouched.
3. **Runtime marker surface (`ir.rs`, minimal).** Add the exact occurrence
   marker(s) consumed two-way by native lowering, plus the exhaustive
   Runtime-expression visitor/validator arms the new marker forces. Concrete
   choice: name the marker variant(s) and enumerate every visitor updated (the
   surface-enum-expansion consumer closure — see §9).
4. **Owned oriented segment + affine splice (backend).** In
   `cranelift_backend.rs`, build the owned region object holding both projections
   (§3), and implement splice as the one-shot oriented composition `T ∘ S : A → C`
   (§3.2) that preserves stored frame order and consumes both capabilities.
   Concrete choice: the exact type for the affine capability (move-only /
   non-`Clone`, open/consumed state) and the single function that performs the
   oriented compose.
5. **Terminal-authority mint (backend).** Mint terminal-answer authority **only**
   under all four §3.4 conditions. Concrete choice: name the single mint function
   and assert (test) no other site mints; retain `live_source_continuations` at
   most as a `debug_assert!` accounting invariant, never in the mint/classify path.
6. **Validation, fail-closed (backend + validators).** Enforce the frame↔delimiter
   bijection, endpoint/orientation checks, one-shotness, and metadata-presence
   before trace/splice/CFG-emission/consumer execution. Concrete choice: the exact
   rejection points and that all reject **before** any consumer runs.
7. **Shared primitive across all four consumers.** The real source path **and all
   three direct/non-source consumers** use the **same** oriented composition
   primitive and validator (no per-consumer bespoke path). Concrete choice: name
   the one primitive and grep-prove the four call sites route through it.
8. **Evidence suite.** Every AC in §7 as a real reaching test in the Runtime crate
   suite (targeted, per §8), plus the elaborator-side producer tests.

## 7. Acceptance criteria — merged evidence bar (all mandated; each a real reaching test)

### 7a. Redesign bar (Architect `evt_7411g73ptm34p`)

- **AC-R1 — exact counterexample passes:** delimiter trace `o0,o4,o3`, semantic
  application `p2,p1,p0`, **checked Success**, LIFO releases, and **no residual
  segment**.
- **AC-R2 — noncommuting typed chain:** a chain `A → B → C → D` where **every
  swap/reversal fails endpoint validation before CFG emission.**
- **AC-R3 — axis independence:** a **correct occurrence ledger + permuted semantic
  order FAILS**; a **correct oriented order + opposite delimiter order PASSES.**
- **AC-R4 — bijection:** exact frame↔delimiter/root bijection; **reject** missing,
  duplicate, stale, cross-region, or unmatched evidence.
- **AC-R5 — one-shotness:** double splice/resume, reuse-after-consume, and
  child-after-parent-consume all **fail before consumer execution.**
- **AC-R6 — independent nested/sibling regions:** no token theft across siblings;
  each child completes before composing into its parent.
- **AC-R7 — endpoint-corruption:** corrupting an answer endpoint **fails even when
  occurrence evidence remains exact.**
- **AC-R8 — mutation pair:** **same-ID colocation** reproduces the current
  `ExitCode`-to-`ITree` trap; **semantic flatten/reversal** reproduces the earlier
  duplicate-consumer / surviving-aggregate failures.
- **AC-R9 — one shared primitive** for the source path and all three direct
  consumers (§6.7).
- **AC-R10 — real bracket controls:** one-, two-, and three-level public bracket
  tests, the fixed-point cases, exact terminal answer, LIFO release trace, and
  bounded/no-fuel behavior — all green.

### 7b. Cross-seam ACs (Architect fence `evt_47e91w1hxm4pp`)

- **AC-X1** — the **checked producer emits the exact `p2 → p1 → p0` interface
  chain before erasure.**
- **AC-X2** — each marked Runtime frame and each plan entry **consume each other
  exactly once.**
- **AC-X3** — missing, duplicate, stale, transplanted, or endpoint-corrupted plan
  entries **reject before CFG emission.**
- **AC-X4** — preserving occurrence evidence while permuting semantic order
  **rejects** (the elaborator-transported twin of AC-R3).
- **AC-X5** — the opposite delimiter order `o0 → o4 → o3` **passes with the checked
  semantic order intact.**
- **AC-X6** — **deleting the metadata or replacing it with Runtime inference fails
  closed.**
- **AC-X7** — the **scalar join plan cannot be used as a fallback** for aggregate
  interfaces.

### 7c. Research discriminating evidence net (`evt_40tfb9c45bfs9` — fold in where not already covered)

Covered by 7a/7b above except make explicit: **(net-1)** preserve the exact
counterexample trace with **no residual continuation**; **(net-7)** corrupt **only**
an answer endpoint while retaining correct occurrence → validation fails **before
artifact execution**; **(net-6)** H3/H4/H5 stay **independently partitioned** and
siblings cannot consume each other's tokens.

> **All ACs are reaching tests, not prose claims.** No AC is satisfied by a
> green-vs-green: AC-R8 / AC-X6 mutations are the discriminators that prove the new
> machine — not the metadata's mere presence — is the net.

## 8. Build/test discipline (hard rules)

- **Targeted builds only** — `scripts/ken-cargo -p ken-elaborator` and
  `scripts/ken-cargo -p ken-runtime` (and `--test <name>` for a single suite);
  **never `--workspace`.** "Workspace-green" / no-regression = **green in CI**,
  verified by the publisher polling the CI checks — never a local workspace run
  (COORDINATION §12, CLAUDE.md hard rule). Because this is **cross-crate**, the
  full-workspace `--locked` build and any checked-program/prelude exhaustiveness
  consequences of the new `ir.rs` marker are caught **only in CI** — do not rely on
  a local per-crate green to prove no downstream consumer broke.
- `source scripts/ken-env.sh` first (shared sccache/CARGO_HOME).
- Run `ken fmt --check` / the frozen-corpus `ken_fmt` test and `rustfmt --check` on
  touched files, and `git diff --check`, before releasing the candidate.

## 9. Scope fence & do-not-reopen guards

- **Cross-seam but compiler-private and Runtime-owned.** Permitted surface is
  **exactly** §4.2: `erasure.rs` (+ a `checked_core.rs` view only if required) as
  the metadata **producer**, and the Runtime plan/codec/`ir.rs` marker/backend/
  validators/tests as the **consumer**. **NO** kernel, public source syntax, public
  Runtime semantics, host ABI/wire, `spec/`, `conformance/`, or PX8-F fixture
  change. Because the diff touches **no `spec/`/`conformance/` path and changes no
  observable semantics, there is NO CV lane** — reviewers are **Runtime QA →
  Architect §14** only.
- **New `ir.rs` marker = surface-enum expansion.** Enumerate and update **every**
  Runtime-expression visitor/validator/exhaustive-match consumer of the new marker
  in the **same** WP (the consumer-closure discipline); a checked-program /
  prelude-exhaustiveness break from a new enum ctor is caught **only** by
  full-workspace CI, so the AC bar leans on CI green, not local `--lib`.
- **Do NOT widen `NativeJoinAnswerKindV1` / reuse the scalar-join plan.** The new
  oriented-subcontinuation plan is a **separate** plan; AC-X7 forbids the scalar
  plan as an aggregate fallback (§4).
- **Do NOT collapse the two projections.** Occurrence/delimiter order must never
  select semantic execution position, and the semantic segment must never choose
  ownership or a return target (§3.3). This is the exact defect being fixed.
- **Do NOT invent endpoints in the backend.** Runtime lowering **validates** the
  checked descriptors; it may never mint them from `ITree`/`Result`/`ExitCode`/
  provenance/occurrence/shape/trial execution (§4).
- **Do NOT resume from the preserved probe or hold.** `9a41deb4` and `b38de27b`
  are **evidence witnesses**; the redesign is a fresh mechanism from the current
  base (§4.2 surface). No resumed incremental cut is authorized.
- **This is DISTINCT from PX8-J-ERR** (the non-blocking validator erratum that adds
  `validate_recursor_invocation_segment` at the top of `install_recursor_invocation`).
  Separate WP, separate branch, separate Decision; do not fold it in.
- **Do NOT special-case `withResource` / PX8-F or widen a match by nesting depth.**
  AC-R2/AC-X1 (the general noncommuting chain and the exact interface chain) exist
  precisely to catch a two-level special-case.

## 10. On landing

Merges via the publisher path on a **resolved Architect §14 Decision**
(`status=resolved`, `resolved_by=architect`), verified **by content** on `main`
(grep the new plan/marker symbols + blob compare), then PINNED. On its landing:

- **PX8-F resumes** from preserved `origin/wp/px8f-buffer-io-surface @ eb8e5596`
  (Foundation re-kicked by the Steward after a fresh handoff-gate) and owns the
  full linked `writeAll` trace/differential gate. **Notify the adversary**
  (`agt_37vnwmcdxhw00`) on the code merge (every code merge).
- **PX8-J-ERR** (Task #30) may run on Runtime in parallel with PX8-F or be
  sequenced by the Steward.
- Record at the post-PX8 debrief: the **7-hard-stop chain** that mapped a
  "predicate swap" into a **delimited-control machine**, and the honest redesign —
  operator-visibility material, not wasted motion.
