# RT-SPLIT — decompose `cranelift_backend.rs`

**Owner:** Team Runtime · **Size:** L (a slice series) · **Risk:** medium
(large diff, low semantic content) · **Gate:** none — maintainability work,
feeds no G-gate · **Deps:** none (PX8 series complete and landed)

**Status:** frame authored; **Phase 0 ruling DELIVERED and transcribed below
(§10) — it is the binding decomposition.** Execution waits only on the Runtime
ring becoming free (it is sequential and currently on RT-PARITY).

## 1. Objective

`crates/ken-runtime/src/cranelift_backend.rs` is **22,081 lines** in a single
flat module. Decompose it into coherent submodules **without changing any
behavior**. The value is entirely in future legibility and review cost: every
Runtime WP for the last several series has landed in this one file, and both
the implementer and the §14 reviewer now pay a 22k-line orientation tax on
every change.

This WP is a **pure move**. It buys no features, fixes no defects, and changes
no semantics.

## 2. Flow — and why the spec enclave is not in it

Normal WP release runs Steward-frame → spec-enclave elaboration → build team
(`steward.md §2c`). **This WP skips the enclave step deliberately:** it touches
no `spec/` and no `conformance/` path, changes no behavioral contract, and asks
no "what must this do to be correct?" question — the artifact it changes is
module structure. What it *does* need is a component-design ruling, which is
the `any → Architect` edge (COORDINATION §9). The operator directed this
explicitly: *"architect to rule on the breakdown."*

So the flow is:

```mermaid
flowchart LR
  A[Steward: this frame] --> B[Phase 0: Architect rules the decomposition]
  B --> C[Phase 1..N: Runtime executes, one module per slice]
  C --> D[Gates: Runtime QA + Architect §14 — NO CV lane]
  D --> E[Publisher path]
```

**No CV vote** — the combined diff-scope touches no `spec/`/`conformance/`
path. If a slice somehow does, that slice pulls a Spec vote (§14 diff-scope);
that would itself be a signal the slice is out of scope.

## 3. Phase 0 — the Architect's ruling (blocking, and the real design work)

> **✅ DELIVERED — see §10 for the binding ruling.** This section states what
> was *asked for*; §10 is what was *ruled*. Where they differ, §10 governs.

**Runtime does not choose the seams.** The Architect rules, and the ruling
must pin four things:

1. **The module list** — the target set of submodules, each with a one-line
   charter, and a ceiling on any single resulting module.
2. **The assignment** — which items land in which module. The Architect need
   not enumerate all 191 top-level items individually; a rule per cluster plus
   the disposition of the ambiguous ones is enough.
3. **The dependency order** — the modules must form a DAG. `mod` cycles are
   legal in Rust, so nothing mechanically stops a tangled cut; the ruling is
   what prevents it.
4. **The visibility policy** — see §4, which is the constraint that actually
   decides whether a proposed cut is good.

**Slice order** is also the Architect's to set, and should be *leaf-first*: the
modules nothing else depends on move first, so each slice shrinks the residual
file without re-touching an already-moved one.

## 4. ★ The constraint that decides the cut: visibility widening

This is the non-obvious part and the reason a ruling is needed rather than a
tidy-up.

In a flat module every item is mutually accessible with **no visibility
annotation at all**. The instant the file is split, every reference that
crosses a new module boundary needs `pub(crate)` or `pub(super)`. And
`crates/ken-runtime/src/lib.rs:39` does:

```rust
pub use cranelift_backend::*;
```

— a **glob re-export**. So an item promoted to bare `pub` to satisfy a cut
does not merely become crate-visible; it lands in `ken_runtime`'s **public
API**. A cut can therefore silently widen the crate's public surface while
every test stays green, because nothing in-repo fails when a surface grows.

⇒ **The count of items that must be visibility-widened is the objective
quality metric of a proposed decomposition.** A cut that follows topic labels
but severs a tightly-coupled cluster will need hundreds of `pub(crate)`s; a
cut that follows the actual dependency structure will need few. **Prefer the
cut that minimizes widening, not the one with the prettiest module names.**

This is the expressibility shape from `steward.md §2c (b‴)`: the obligation
*"this item is an internal detail"* needs a home the compiler checks. In a
flat module the home is "it's private to the file." After a split, a bare
`pub` is a **reach outside the checked vocabulary** — the guarantee stops
being enforced and survives only as intent. `pub(crate)` keeps it enforced.
Hence AC-7.

## 5. Fixed inputs — settled, do not reopen

1. **Pure move.** No logic changes. No renames. No signature changes. No
   clippy/format drive-bys, no "while I'm here" cleanups, no dead-code
   removal. A one-line semantic change hidden in a 20k-line move diff is
   effectively invisible to review — that is the entire risk of this WP, and
   it is bought down only by the diff being *provably* a move.
2. **The crate's exported name set is invariant.** Every name reachable as
   `ken_runtime::<name>` before must be reachable after, unchanged. 25 test
   files across the workspace consume `ken_runtime::` paths.
3. **Tests move with their subject.** ~7,067 lines (32% of the file) sit in 25
   `#[cfg(test)]` blocks that exercise **private** internals. Each moves with
   the items it tests. None may be deleted, `#[ignore]`d, or weakened to make
   a move compile.
4. **CI is the venue for workspace-green** (COORDINATION §12, operator hard
   rule). Local work is `scripts/ken-cargo -p ken-runtime` only. **Never** a
   local `--workspace` — it OOMs the box and stalls the fleet.
5. **No ABI, wire-format, or codegen change.** A pure move must not perturb
   emitted code.

## 6. Grounded inventory (measured at `origin/main @ c4f55c19`)

**⚠ Perishable — re-verify against the landed code at pickup; do not trust
these numbers if the file has moved under this frame.**

| Property | Value |
|---|---|
| Total lines | 22,081 |
| Free functions (`^fn` / `^pub fn`) | 93 |
| `impl` blocks | 17 |
| Structs | 51 |
| Enums | 30 |
| `#[cfg(test)]` blocks | 25 (≈ 7,067 lines) |
| Section-comment banners | **none** — the file is flat |

The absence of banners matters: there is **no existing authorial seam to
follow**, so the decomposition must be derived from the dependency graph, not
recovered from comments.

**In-crate consumers** of `cranelift_backend::` paths:
`object_linker_packaging.rs` (11 references), `native_int_clif.rs` (1),
`lib.rs` (the glob re-export).

**Apparent clusters — a starting signal for Phase 0, NOT a pinned
decomposition.** The Architect owns the actual cut and should feel free to
discard this grouping entirely:

> **⛔ SUPERSEDED — and the ruling did discard it. DO NOT BUILD FROM THIS
> LIST.** §10.0 found that the `Lowering` impl's call graph contains a
> 29-method strongly connected component, and that splitting it along exactly
> the "control / source / host / value" lines suggested below **would
> manufacture a module cycle and is forbidden.** The list is retained only as
> a record of the pre-ruling signal. **The binding decomposition is §10.**

- **Errors / reports** — `CraneliftBackendError`, `ValidatedNativeRunError`,
  `UnsupportedLowering`, `BackendFailure`, `CraneliftRunReport`,
  `NativeDifferentialReport`, `NativeTrustReport`, `NativeToolchainReport`,
  `NativeRuntimeIrComparisonReport`, `InterpreterOracleObservation`,
  `NativeRunEvidence`, `NativeArtifactIdentity`.
- **Oriented-subcontinuation / recursor control** (the PX8-TA/DS/J surface) —
  `DynamicSpliceEdge{,Id}`, `AffineSpliceCapability`,
  `RecursorInvocationSegment`, `{Owned,Installed}OrientedSubcontinuationSegment`,
  `OwnedSelectedScope`, `RecursorUnwindStack`, `RecursorFrameProvenance`,
  `RecursorProducerOriginId`, `CheckedOrientedMarkerSets`,
  `OrientedControlLedgerEntry`, `ComputationalRecursorLayer`,
  `ComputationalRecursorFramePayload`, `SourceComputationalAnswerRoute`,
  `SelectedCaseReturnDelimiter`, `{Root,}TerminalAnswerAuthority`,
  `OpenControlObligation`, `SourceControl`, `SourceBranchFanout`,
  `SourceJoinTarget`, `SourcePredecessorEdge`, `SourceSelectedContinuation`.
- **Continuation frames / eliminators** — `ActiveContinuationFrame`,
  `ComputationalEliminatorFrame`, `OrdinaryEliminatorFrame`,
  `PendingLetContinuationFrame`, `DeferredConstructorCaseEnvironment`,
  `Continuation{Activation,Cursor}Id`, `ArmedInvocation`.
- **Value / numeric lowering** — `BoundedNatV1`, `StructuralNatV1`,
  `NativeScalarPairV1`, `DynamicConstructor{V1,AlternativeV1}`.
- **Compilation / JIT / artifact** — `CompiledModule`,
  `CraneliftObjectArtifact`, `NativeSeedEnvironment`, `Lowering`.
- **Recursive declarations** — `ActiveRecursiveDeclarationV1`,
  `CheckedRecursiveInvocationInstance`.

Note the second cluster is both the largest and the most recently churned (the
whole PX8 chain landed there). It is the highest-value extraction and probably
the hardest — the Architect should decide whether it leads or trails the
series.

## 7. Acceptance criteria

Each is checkable by a reviewer; AC-2/3/7 are the ones that make the "pure
move" claim auditable rather than asserted.

> **★ AC-2 and AC-3 were REWRITTEN 2026-07-22, after slice 1 merged**, on the
> adversary's post-merge findings and the Architect's ruling
> (`evt_1y255ges6mftc`). Slice 1's verdict is **unchanged and clean** — it was
> established by an independent line-multiset check, **not** by AC-2. What
> changed is the *evidence contract for slices 2–7*: AC-2 was carrying a claim
> it had never established, and AC-3 named a mechanism loosely enough that a
> multiset could be read as discharging it. **If you cut a slice against the
> pre-2026-07-22 wording of these two ACs, re-read them now.**

1. **Decomposition matches the ruling.** The resulting modules are exactly the
   Architect's ruled list, and no module exceeds the ruled ceiling.
2. **Module-level exported-NAME identity.** The set of *module-level item
   names* reachable as `ken_runtime::<name>` is **identical** before and
   after — verified by a sorted symbol dump taken at the merge-base and at the
   candidate tip, diffed to **empty**. The command used goes in the PR body.

   > ⛔ **State this check at its measured strength and no further** (Architect
   > ruling `evt_1y255ges6mftc`, 2026-07-22, on the adversary's measurement).
   > `cargo doc --no-deps` + hrefs from `all.html` enumerates **module-level
   > item names only.** Fields, enum variants, inherent methods, and trait
   > impls are **not names in that namespace**, so they cannot move the diff.
   > Four real public-surface mutations were run against the landed oracle and
   > **all four went undetected** (baseline 14 hrefs → mutated 14 hrefs, diff
   > empty): private field → `pub`; new public enum **variant**; new public
   > inherent **method**; **deleted `impl Display`**.
   >
   > **AC-2 is necessary, never sufficient.** Cite it in a PR body as *"no
   > module-level item name changed"* — that sentence, not "no public surface
   > change." The whole-public-surface claim is carried by **AC-3**, which is
   > where impls, methods, variants, and fields are actually held.
   >
   > This is the same defect as `DOC-VALIDATION-BINDING`, one day later in
   > another team: **an enumeration checked against another enumeration of the
   > same kind, while the property that matters lives outside the domain
   > either one iterates.**

3. **Move-purity — ORDERED item-level identity.** For each slice, every moved
   production **declaration, function/method body, trait impl, and macro
   invocation** is compared against its source as an **ordered token
   sequence**, and the only permitted deltas are enumerated and reviewed
   **separately**: module/import paths, namespace wiring, and the exact AC-7
   visibility ledger. State the mechanism in the PR body.

   > ⛔ **Order is load-bearing; a multiset is not enough.** A normalized line
   > multiset is excellent as a **second inventory net** — it exposes
   > dropped/added lines and it *produces* the AC-7 ledger as a measurement
   > rather than as an author's enumeration confirmed after the fact — but it
   > **discards order and context.** Swapping two effectful statements
   > preserves the multiset and changes behavior. Use it; do not let it stand
   > in for ordered identity.
   >
   > ⛔ **"The tests pass" is not evidence of move-purity.** It is evidence of
   > behavior on the paths the tests reach. Retain pre-move coverage
   > measurement plus targeted green as the **behavior** net — especially for
   > trait impls and macro-generated behavior — but **never substitute tests
   > for ordered move identity.**
   >
   > ⛔ **There is no "restructuring class" that relaxes this.** Every RT-SPLIT
   > **production** slice remains move-pure under this AC and §10.5. The word
   > *restructuring* may describe namespace scaffolding, imports, test-file
   > redistribution, or the final facade — it **does not authorize production
   > logic or content change** in any slice. Applied to the ruled order:
   > slices **2** (`planning`), **3** (`compiled`), **5** (lowering support),
   > and **6** (`artifact`) are predominantly ordered item moves; slice **4**
   > (`lowering::core`) adds hierarchy and test scaffolding and slice **7**
   > adds `artifact::api` plus the explicit facade — for those two, the **new
   > wiring** gets its own separate review pass, and the moved production text
   > is still held to ordered identity.
   >
   > ⛔ **And nothing in this AC is discharged by a byte-identity check of
   > whole files.** Byte-identity goes red on lawful import and visibility
   > churn — it was ruled out (operator, 2026-07-22) and stays out. The unit
   > is the **moved item**, not the file.
4. **Test preservation.** All 25 `#[cfg(test)]` blocks compile and pass. The
   total test-function count is unchanged; no test is deleted, `#[ignore]`d,
   or has an assertion weakened. Report the before/after count.
5. **Targeted green locally, workspace-green in CI.**
   `scripts/ken-cargo test -p ken-runtime` green on the candidate; the full
   `--workspace --locked` run is **CI's**, polled by the publisher path.
6. **Codegen unperturbed.** The native differential and any frozen native
   fixture remain green and **unmodified**. If a frozen fixture needs an
   update, the move was not pure — stop and escalate rather than
   re-baselining.
7. **No public-surface widening.** No item gains bare `pub` that did not have
   it. Every new cross-module visibility is `pub(crate)` or `pub(super)`, and
   the **count** of items so widened is reported per slice in the PR body
   (it is the metric from §4).

## 8. Guardrails — do not reopen

- **Do not redesign the backend.** If you find something that looks wrong
  while moving it, **move it unchanged and file it separately.** A defect
  found during a refactor is a follow-up WP, never a bundled fix.
- **Do not widen visibility to make a test reach its subject.** If a test
  cannot reach what it tests after a cut, the *cut* is wrong — escalate to the
  Architect for a seam revision. Silently promoting an item to `pub` to buy
  green inverts AC-7 into a rubber stamp.
- **Do not bundle the adversary docket items.** F4/F5/F6 are open against
  neighbouring surfaces and are being classified separately; none of them
  lands here.
- **Do not touch `crates/ken-interp/`, `crates/ken-host/`, or any `catalog/`,
  `spec/`, or `conformance/` path.** If a slice appears to need one, that is
  an escalation, not a scope stretch.
- **Every anchor in this frame is perishable.** The §6 figures and the §4
  `lib.rs:39` citation were measured at `origin/main @ c4f55c19`. Re-verify at
  pickup; **if a fixed input turns out false against the landed code, say so
  and escalate — do not quietly build around it.**

## 9. Sequencing and branches

- Frame branch: `wp/rt-split-frame` (Steward's; merges and dies).
- Build branches: `wp/rt-split-<n>-<slug>`, each cut **fresh from current
  `origin/main`** after the previous slice lands. A squash-merged branch
  cannot be continued.
- **Each slice is independently behavior-preserving, independently green, and
  independently mergeable.** This is not a land-together assembly — a slice
  that only makes sense alongside the next one is mis-cut.
- Rebase each slice onto current `origin/main` before its merge Decision;
  "rebased onto current main" is a perishable claim (§14(5)).

## 10. Phase 0 ruling — DELIVERED (Architect, `evt_1q0cdpv9qrjxe`)

Grounded at `origin/main @ 244cfe9c`; the §6 perishable inputs were
re-confirmed still true (22,081 lines, `lib.rs` glob export unchanged).

**This section is the binding decomposition.** It is transcribed here because
an in-thread ruling is not a durable deliverable — build from this file, never
from the convo thread.

### 10.0 Why the topical cut is forbidden

The cut is driven by the **call graph**, not by the apparent topic list. The
`Lowering` impl has **108 methods**. Its direct self/associated-call graph has
one **29-method strongly connected component occupying 5,864 method-body
lines**. The other **79 methods occupy 3,506 lines**; there are **145 calls
from the SCC into those helpers and zero calls from those helpers back into
the SCC**.

⇒ Splitting that SCC into "control", "source", "host", and "value" production
modules — i.e. the cluster grouping this frame listed in §6 as a *starting
signal only* — **would manufacture a module cycle and a broad visibility seam.
It is forbidden.**

### 10.1 Pinned production modules

**No physical Rust module file may exceed 6,500 lines after `rustfmt`.** Do
not satisfy the ceiling with a giant inline module; the ceiling applies to
each `.rs` module body too.

1. `cranelift_backend/mod.rs` — facade only: module declarations and explicit
   re-exports preserving the exact old `ken_runtime::<name>` surface.
2. `cranelift_backend/surface.rs` — reports, evidence, errors, outward data
   types, `NativeSeedEnvironment`, and their `Display`/`Error`/`From` impls.
3. `cranelift_backend/planning.rs` — native-join/oriented-plan extraction,
   checked-marker census, pre-emission transport validation; no CLIF emission.
4. `cranelift_backend/compiled.rs` — `CompiledModule`, `CompiledExpr`,
   `ResultDecoder`, result-table ownership, JIT result decoding/execution.
5. `cranelift_backend/lowering/mod.rs` — `Lowering` state, lowered-value and
   continuation/control data model, pure free helpers, and the 79 acyclic
   support methods outside the SCC.
6. `cranelift_backend/lowering/core.rs` — the **indivisible 29-method lowering
   SCC** plus `compile_expr_into_module`; the recursive lowering engine.
7. `cranelift_backend/artifact/mod.rs` — ISA/module setup and private
   JIT/object compilation and materialization machinery.
8. `cranelift_backend/artifact/api.rs` — the existing public and crate-facing
   run, validation, comparison, and object-emission entrypoints and their
   orchestration.

**Required test modules:** `planning/tests.rs` · `artifact/tests.rs` ·
`artifact/api/tests.rs` · `lowering/core/tests/mod.rs` (shared test-only
fixtures) · `lowering/core/tests/control.rs` ·
`lowering/core/tests/constructors.rs` · `lowering/core/tests/effects.rs` ·
`lowering/core/tests/values.rs`. The 6,500-line ceiling applies to these too.
**No residual omnibus `mod tests` remains in the facade.**

### 10.2 Assignment rule

- **`surface.rs`** — the current report/evidence/error declarations from the
  top of the file, `NativeSeedEnvironment`, the report/error impls, and
  `unsupported`/`backend`/`backend_module`.
- **`planning.rs`** — `native_join_plan_for_program`,
  `oriented_subcontinuation_plan_for_program`,
  `collect_checked_subcontinuation_frames`, the checked-marker collectors and
  exact-location checks, and `validate_oriented_subcontinuation_transport`.
- **`compiled.rs`** — exactly the compiled container, decoder, JIT `run`, and
  their directly-owned decoding state. **It does not own compilation policy.**
- **`artifact/mod.rs`** — `compile_expr`, `compile_program_expr`,
  `compile_expr_with_declarations{,_and_process_input}`, object/JIT module
  creation, verifier invocation, target naming, private object/JIT
  materializers.
- **`artifact/api.rs`** — all outward runners, preflight and
  differential/report orchestration, existing object-emission entrypoints.
- **`lowering/core.rs`** — `compile_expr_into_module` and exactly this SCC:
  `lower_recursor_residual_call`, `lower_computational_match_expr`,
  `lower_computational_producer_expr`, `resume_active_continuation`,
  `lower_computational_match_value_composed`, `lower_bounded_nat_computational`,
  `materialize_eliminator_frame_env`, `lower_source_machine`,
  `lower_source_machine_with_continuation`,
  `lower_source_machine_with_continuation_inner`,
  `lower_source_bounded_nat_match`, `lower_source_dynamic_bool_match`,
  `lower_source_dynamic_host_result_match`,
  `lower_source_dynamic_constructor_match`,
  `lower_source_nested_dynamic_constructor_match`,
  `lower_source_planned_dynamic_constructor_match`, `source_call_state`,
  `lower_source_declaration_call`, `lower_expr`, `lower_process_host_effect`,
  `lower_unary_recursive_nat_fold`, `lower_recursive_declaration_call`,
  `lower_declaration_ref`, `lower_borrowed_match`,
  `lower_borrowed_option_match`, `lower_dynamic_host_result_match`,
  `lower_bounded_nat_match`, `lower_dynamic_constructor_match`,
  `lower_primitive_call`.
- **`lowering/mod.rs`** — every other `Lowering` method; the private
  lowered-value, recursive-declaration, continuation, source-machine,
  oriented-control, bounded-Nat, dynamic-constructor and scalar-pair types
  plus their free helpers; the recursive-argument helpers after the impl.

**Ambiguous dispositions (ruled):**

- `with_px8ds_retired_flat_order` and the PX8 test/mutation ledgers stay with
  lowering; the facade explicitly re-exports their pre-existing visibility.
- `Px8trTrapProvenanceEvent`, `NativeIntLoweringMutation`, and
  `NATIVE_INT_LOWERING_MUTATION` remain **test-only lowering** ownership — not
  artifact, not surface.
- `ResultDecoder` belongs to `compiled`, **not** value lowering.
- `reject_program_blockers` belongs to `artifact/api`, **not** planning.
- Dynamic-constructor validation/selection and source-continuation free
  helpers belong to **lowering support**; their callers in the SCC do not make
  them part of the SCC.
- Test helpers go in the lowest `tests/mod.rs` ancestor shared by their actual
  users. **They never justify widening a production item.**

**Test assignment is by subject:** `oriented_*`, `px8j_*`, root-authority,
join-site, source-install and recursor tests → `control`; constructor-field,
dynamic-constructor, nested-computational and heterogeneous-eliminator tests →
`constructors`; host-reply, bounded-Nat, IO, borrowed-ingress and native-int
tests → `effects`; scalar/bytes/string/closure/primitive lowering tests →
`values`; certificate, preflight, differential and outward-runner tests →
`artifact/api/tests.rs`; exact JIT/object/ISA tests → `artifact/tests.rs`. **A
test spanning two topics is assigned by the private item whose behavior it
directly discriminates.**

### 10.3 Dependency DAG

Arrows mean "caller depends on callee":

```
facade          -> artifact::api, surface, existing lowering test hooks
artifact::api   -> artifact, planning, surface
artifact        -> lowering::core, compiled, planning, surface
lowering::core  -> lowering support, compiled, planning, surface
lowering support-> surface
planning        -> surface
compiled        -> surface
```

**There are no reverse edges.** In particular: `artifact` never imports
`artifact::api`; lowering support never calls `lowering::core`; and no
implementation module imports through the facade. Module declarations and
facade re-exports are **namespace wiring, not permission to introduce a
semantic back-edge**.

### 10.4 Visibility policy

- The facade uses **explicit re-export lists only**. No internal
  `pub use child::*`; the existing `lib.rs` glob remains unchanged.
- Existing bare-`pub` declarations stay bare `pub`; existing `pub(crate)`
  declarations retain that visibility. Explicit facade re-exports may expose
  **only** already-exported names.
- **No private declaration may gain bare `pub`.**
- A new production seam uses the narrowest `pub(super)` or
  `pub(in crate::cranelift_backend)`. **New `pub(crate)` is prohibited**
  unless an already-landed consumer outside `cranelift_backend` requires it.
- **Hierarchy is load-bearing:** `lowering::core` is a child of the module
  owning `Lowering` state/support, and `artifact::api` is a child of
  `artifact`. **Descendants consume ancestor-private items without widening
  them.**
- Tests move below their subject. **A production visibility change made only
  for a test is a seam failure.**
- **★ BUDGET — at most 24 newly visibility-widened declarations over the whole
  series, and at most 12 in one slice.** Existing visibility and explicit
  re-exporting of an already-exported name do not count. **Count fields
  individually.** If either budget would be exceeded, **stop and return the
  proposed extra seams to the Architect — do not spend through the cap.**
- Every slice reports a **before/after exported-name dump** and an **exact
  visibility ledger**: item, old visibility, new visibility, cross-module
  consumer.

**Expected widest single seam:** `compiled.rs` — its private container fields
are shared by artifact construction and lowering completion. That is a real
shared boundary and may consume most of one slice's allowance. The `Lowering`
fields and the 79 support methods should consume **zero** new visibility,
because `core` is their descendant.

### 10.5 Slice order

1. `surface`
2. `planning`
3. `compiled`
4. `lowering::core` plus its subject tests
5. `lowering` support/state plus its remaining subject tests
6. `artifact` plus artifact tests
7. `artifact::api`, API tests, and the final explicit facade

Slices 1–3 are true leaves. **The control SCC then LEADS rather than trails
the lowerer extraction — the one deliberate exception to leaf-first.** In
slice 4, create the final `lowering/mod.rs` scaffold with a private import of
the still-residual parent items, and make `core.rs` import only from its
parent. In slice 5, move those residual state/support items into
`lowering/mod.rs`; **`core.rs` is not touched again.** Moving support first
would force temporary widening of every field/method merely so the residual
parent could reach into its child.

Each slice is independently green and mergeable, starts fresh from the newly
landed `origin/main`, moves one production module plus its tests, and does not
re-touch a previously moved module. **If move-purity, the visibility budget,
or the DAG cannot be demonstrated for a slice, that slice stops for seam
revision — it does not improvise a topical split.**
