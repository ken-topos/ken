# PX7-P — constructor-field → selected-case consumer composition (producer `Construct` deforestation arm)

- **ID:** PX7-P · **Owner:** Team Runtime · **Size:** M · **Risk:** High
  (extends the heterogeneous eliminator-frame deforestation spine across the
  producer-`Construct` seam in the cranelift backend — TCB-relevant lowering;
  must complete the producer-form coverage without weakening any landed
  fail-closed guard).
- **Objective:** Add the missing **producer-`Construct` composition arm** to the
  checked-host deforestation spine, so a **dynamic aggregate-producing field of a
  constructor that is consumed by an active eliminator frame** is composed
  **through that frame before the field is lowered** — instead of the field's
  dynamic ordinary `Match` merging to `Lowered::ProcessExitStatus` while
  `process_object` is active, which the later selected computational case's
  inlined ordinary `Result` match then correctly rejects as non-constructor. This
  is the **fifth** downstream Runtime compiler prerequisite in the PX7 chain
  (PX7-L → PX7-M → PX7-N → PX7-O → **PX7-P**), and it **completes the
  producer-form coverage**: `Construct` joins `Match`/`If`/`Call`/
  `ComputationalMatch` as a producer form the composed frame stack can execute.
- **Depends on:** **PX7-O merged** (`origin/main @ f2d3cf77`) — this WP extends
  the `ComputationalEliminatorFrame` / `OrdinaryEliminatorFrame` heterogeneous
  architecture and the `requires_heterogeneous_deforestation` result-shape
  judgment landed there. Resource-independent; carries **no** resource content.
  Base = `origin/main @ f2d3cf77`.
- **Feeds:** unblocks **PX7-F**'s deferred public linked-native AC2/AC3 (the
  `withResource` producer whose value is `Construct ITree::Ret [dynamic
  Result-producing Match]`, consumed by the final computational eliminator's
  `ITree::Ret` case whose body is the ordinary `Result::{Err,Ok}` match). PX7-F is
  the first consumer, **held at `wp/px7f-system-resource-bracket @ 7ccbc226`**
  and rebased onto the merged PX7-P head afterward. PX7-F's native AC2/AC3 stay
  **explicitly deferred until the public linked-artifact pair is green on the
  landed PX7-P** (do not mark them satisfied before).

## Same-WP amendment — closed dynamic-constructor producer seam

The Architect's follow-up ruling (`evt_6qk7606adaf8x`) and the Steward's scope
call (`evt_4c4chdwekpfwp`) retain the constructor-field bridge below and extend
this same WP with the resource-independent producer seam that bridge exposed.
This is not a new prerequisite and carries no PX7-F or Resource vocabulary.

Ordinary lowering already owns closed dynamic-sum matchers, but computational
producer lowering recognized only separately hard-coded dynamic carriers. PX7-P
therefore generalizes the landed nullary dynamic-constructor lane to this closed
carrier:

```rust
DynamicConstructorV1 {
    discriminator: Value,
    alternatives: Vec<DynamicConstructorAlternativeV1>,
}

DynamicConstructorAlternativeV1 {
    tag: i64,
    constructor: CanonicalIdentity,
    fields: Vec<Lowered>,
}
```

One shared selected-case dispatcher consumes that carrier in two modes:

- ordinary mode lowers the selected source case with `lower_expr`;
- producer mode lowers it with `lower_computational_producer_expr` under every
  active ordered eliminator frame.

Both modes share tag selection, exact constructor lookup, binder-arity
validation, declaration-order field/environment materialization, source-match
default ownership, and branch-value merging. `HostResult` may retain its
existing specialized lane.

The amended fail-closed bounds are binding:

1. Alternatives have unique tags and canonical constructor identities.
2. Fields are installed in declaration order and dominate the selected branch.
3. A known alternative omitted by the source match takes that match's own
   default.
4. A discriminator outside the closed alternative table is malformed
   runtime/ABI input, not a silently selected case.
5. Binder arity is exact.
6. Producer mode preserves every active eliminator frame and never lowers the
   intermediate aggregate to `ProcessExitStatus`.
7. Ordinary `Match` remains unable to consume `ProcessExitStatus`.
8. PX7-P production and committed proof remain free of Resource/PX7-F names.

The committed proof uses non-resource constructors of differing arities in both
continuation modes and covers middle-field/trailing-environment order,
recursive-IH offsets, missing/default, unknown tag, duplicate tag and identity,
arity/binder, unconsumed/scalar fields, and established `HostResult`
compatibility. PX7-L/M/N/O remain green.

The former requirement to commit both real PX7-F fixtures as PX7-P proof is
replaced by an evidence-only pre-review overlay. That overlay may add the
minimal five-constructor PX7-F adapter, must make both exact held
`px7f_resource_native` fixtures green, and must reproduce the current failure
when producer continuation is mutated back to ordinary-only. The adapter is not
PX7-P content; PX7-F owns its reply/tag/schema validation and the translation
from decoded resource reply into this generic carrier.

## Why this exists (the boundary, Architect-ruled `evt_5v45spvc0ec3w`)

Rebasing PX7-F onto merged PX7-O (`origin/main @ f2d3cf77`) cleared the
producer→ordinary-consumer and recursive-IH seams but exposed a **fifth,
distinct** gap. The exact minimized public reaching case still reds at object
emission: `unsupported runtime-IR lowering: Match: scrutinee is not a
constructor value`.

**The Architect's focused diagnostic (corrects the initial `finish` attribution
— `finish` is already erased/inlined; there is no declaration-call boundary):**

- The final computational eliminator has an `ITree::Ret` case whose body is the
  ordinary `Result::Err | Result::Ok` match with scrutinee **exactly `Var(0)`**.
- The corresponding producer is syntactically **`Construct ITree::Ret [dynamic
  Result-producing Match]`**.
- `lower_computational_producer_expr` (**@1790**) has **no explicit `Construct`
  producer arm.** Its fallback calls ordinary `lower_expr` on the whole
  constructor (**`RuntimeExpr::Construct` arm @2326**), which **eagerly lowers the
  field.**
- That field's dynamic ordinary `Match` merges while `process_object` is active,
  so the still-intermediate `Result` becomes `Lowered::ProcessExitStatus`.
- Only afterward is the `ITree::Ret` computational case selected. Its binder
  environment therefore receives `ProcessExitStatus` (observed: `scrutinee =
  Var(0)`, cases `Result::Err/Result::Ok`, **binder 0 = `Lowered::
  ProcessExitStatus`**), and the inlined ordinary `Result` match correctly
  rejects it as non-constructor (**`lower_expr::Match` refusal @2450**).
- A probe of both ordinary and computational `Let` paths **did not fire** — this
  is **not** an eager-`Let` defect. PX7-O's direct continuation bridge
  (`ordinary_match_continuation` @1635) and recursive producer path are present;
  **neither owns this constructor-field transition.**

Note: `requires_heterogeneous_deforestation` **already returns `true` for
`Construct`** (**@1672**) — the classifier recognizes the field as a deforestable
result shape, but the **producer path cannot execute that route** because it has
no `Construct` arm. This is the exact PX7-O block-5 pattern (a classifier route
the producer implementation cannot run), now at the `Construct` seam.

## Mechanism (Architect-sanctioned — FIXED semantics)

Extend the existing heterogeneous eliminator-frame architecture **at the producer
`Construct` seam**. When a producer constructor is being consumed by the active
frame:

1. **Select the frame's exact constructor case BEFORE lowering constructor
   fields.** (Not after — lowering the field first is exactly what collapses the
   intermediate aggregate to `ProcessExitStatus`.)
2. If the selected case **immediately eliminates a bound field** and that field
   expression has the **proven deforestable aggregate result shape**
   (`requires_heterogeneous_deforestation`), **compose that ordinary/computational
   consumer ahead of the remaining frames and recurse into the field producer.**
3. The **intermediate aggregate must remain structural**; merge to scalar /
   `ProcessExitStatus` **only after the entire composed frame stack is
   discharged.**

This is a **general constructor-field → selected-case consumer bridge** — NOT a
`Resource` intrinsic and NOT a spelling rule for `finish` or any symbol. Same
deforestation principle as PX7-L/M/N/O, one producer form further (the
`Construct` seam).

## Acceptance criteria (the Architect's required fail-closed bounds + proof net)

- **AC1 — exact constructor selection / default ownership.** The active frame
  selects the producer constructor by **exact canonical identity**; a **missing
  constructor returns that frame's own default** (observably distinguishable
  identity).
- **AC2 — exact structural checks.** Constructor arity, selected-case binder
  arity, recursive positions, binder shifts, and producer/case environment order
  are **checked exactly**.
- **AC3 — narrow admission (no over-admission).** Admission requires **a
  selected-case consumer of the corresponding binder** AND the existing recursive
  result-shape proof for that field expression. **Do NOT admit every `Construct`,
  every field, or every case merely because a `Match` appears somewhere
  downstream.** (The PX7-O block-1→4 over-admission lesson, at the `Construct`
  seam.)
- **AC4 — non-consumed fields untouched.** Fields **not** consumed through the
  proven bridge **retain ordinary lowering and evaluation order.** The **ignored
  twin must remain green without entering the bridge.**
- **AC5 — final-kind agreement / no intermediate conversion.** Final branches
  must agree on **scalar vs ExitCode** kind. **No intermediate constructor field
  may be converted with `emit_process_exit_status`.**
- **AC6 — fail-closed guards retained.** Ordinary `lower_expr::Match` (**@2450**)
  **must continue to reject `Lowered::ProcessExitStatus`** — no terminal-status
  propagation, no surface computational relabeling, no resource intrinsic, **no
  PX7-F program rewrite.**
- **AC7 — the amended proof net.** Committed proof covers the generic carrier
  with non-resource constructors of differing arities in both continuation
  modes and all eight amended bounds above. It retains the immutable reaching /
  ignored constructor-field pair, aggregate-unconsumed and scalar/`HostResult`
  controls, exact default and arity/binder failures, bridge-removal regression,
  and PX7-L/M/N/O compatibility. Before terminal review, an evidence-only PX7-F
  overlay supplies the minimal adapter and makes both exact held
  `px7f_resource_native` fixtures green; an ordinary-only producer-continuation
  mutation must recreate the current failure. The adapter is never committed to
  PX7-P. No-regression = **green in CI** (targeted `-p ken-runtime` / `-p ken-cli
  --test <name>` locally only; **NEVER `--workspace`**).

## Do-not-reopen guards

- Do **NOT** teach ordinary `Match` / the `@2450` consumer to accept, decode, or
  forward `ProcessExitStatus` — keep the refusal **fail-closed**; the repair is
  the composition bridge, **upstream at the `Construct` seam**.
- Do **NOT** relabel an ordinary `Result`/constructor match as
  `RuntimeExpr::ComputationalMatch`.
- Do **NOT** pattern-match `finish`, `withResource`, a symbol name, a resource
  constructor, or `ITree::Ret` specifically — the bridge is **general and
  resource-independent** (keyed on the frame's selected-case consumer + the
  field's proven result shape, not on any spelling).
- Do **NOT** admit every `Construct`/field/case (AC3). Do **NOT** convert an
  intermediate field with `emit_process_exit_status` (AC5).
- Do **NOT** touch `ResourceOpenMode` / the PX7-F surface program / PX7-R
  substrate / kernel / spec / conformance / Ward.
- **Preserve everything landed** — full PX7-L/M/N/O suites + all existing
  computational fusion, static-ctor, dynamic-`HostResult`, Bool, recursive-IH,
  and identity/capability negatives. You **ADD** the producer-`Construct` arm.

## Scope boundary (hard)

Touches `crates/ken-runtime/**` (cranelift lowering + differential) + new
resource-independent tests. **NO** `spec/` / `conformance/` / Ward / kernel /
ABI-wire / `ResourceOpenMode` / PX7-R-substrate / PX7-F-surface change. If the
current `RuntimeExpr` genuinely loses the producer→selected-case relation (it
should **not** — the relation is structural at the `Construct` producer under an
active frame), that is a **scope fork → escalate to the Architect**, not an
implementer judgment call.

## Route

**Architect §14 ONLY — NO CV** (no `spec/`/`conformance/`/Ward/kernel/ABI change;
crates + tests only). **ONE branch**
(`wp/px7-p-constructor-field-selected-case-composition`), **ONE Decision.**
No-regression = **green in CI**. Re-ground the exact anchor lines on
`origin/main @ f2d3cf77` before building.

## Grounding anchors (verified on `origin/main @ f2d3cf77`)

`crates/ken-runtime/src/cranelift_backend.rs`:
- `struct ComputationalEliminatorFrame` **@1615**; `struct
  OrdinaryEliminatorFrame` **@1622**; `fn ordinary_match_continuation` **@1635**
  (PX7-O's direct-closure-body recognizer — does NOT own the `Construct` seam).
- `fn requires_heterogeneous_deforestation` **@1653**; `RuntimeExpr::Construct {
  .. } => true` **@1672** (classifier already admits `Construct` — producer arm
  missing).
- `fn lower_computational_producer_expr` **@1790** (**ADD the `Construct`
  producer arm here**); the direct-closure-body admission @1821–1823.
- `fn lower_computational_match_value_composed` **@2127** (composed frame-stack
  discharge; `Lowered::Constructor` scrutinee guard @2139).
- `fn merge_branch_value` **@2224** (`Lowered::ProcessExitStatus` @2232 — nothing
  intermediate may pass here); `emit_process_exit_status` (final-merge only).
- `fn lower_expr` **@2261**; the eager ordinary `RuntimeExpr::Construct` arm
  **@2326** (`Ok(Lowered::Constructor { .. })` @2331 — the fallback that collapses
  the field); the ordinary `Match` fail-closed refusal (`let Lowered::Constructor
  { .. } = lowered_scrutinee else { … }`) **@2450**.
