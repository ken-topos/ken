# PX7-O — heterogeneous eliminator-frame composition (producer→ordinary-consumer deforestation bridge)

- **ID:** PX7-O · **Owner:** Team Runtime · **Size:** M · **Risk:** High
  (extends the checked-host deforestation spine across the
  computational→ordinary boundary in the cranelift backend —
  TCB-relevant lowering; must generalize PX7-N's frame stack
  without weakening any landed fail-closed guard).
- **Objective:** Generalize PX7-N's ordered eliminator-frame stack — which
  today carries only **computational** frames — into a **heterogeneous**
  spine that also carries **ordinary value-eliminator frames**, so a
  **dynamic computational producer** whose value flows into a **known
  ordinary constructor-match continuation** is **deforested through that
  ordinary frame**. The intermediate constructor/`Result`/aggregate must
  **never** materialize as a `Lowered` value that collapses to
  `Lowered::ProcessExitStatus` before the ordinary consumer can
  destructure it. This is the fourth downstream Runtime compiler
  prerequisite (PX7-L → PX7-M → PX7-N → **PX7-O**), one boundary further:
  PX7-N fused computational-producer→**computational**-consumer; PX7-O
  bridges computational-producer→**ordinary**-consumer.
- **Depends on:** **PX7-N merged** (`origin/main @ 5b123af3`) — this WP
  generalizes the `ComputationalEliminatorFrame` stack landed there.
  Resource-independent; carries **no** resource content. Base =
  `origin/main @ 5b123af3`.
- **Feeds:** unblocks **PX7-F**'s deferred public linked-native AC2/AC3
  controls (the `withResource` success producer feeding an ordinary
  `match Result FileError (ResourceBracketResult Unit Unit)`); PX7-F is
  the first consumer, held at recoverable
  `wp/px7f-system-resource-bracket @ a8fd0129` and rebased onto the merged
  PX7-O head afterward. PX7-F's public bracket contract is **deferred
  again, not weakened**.

## Why this exists (the boundary, Architect-ruled `evt_t3ywzdkqm6gw`)

Rebasing PX7-F onto merged PX7-N (`origin/main @ 5b123af3`) cleared the
nested-computational-composition boundary and exposed a **fourth, distinct**
gap. The exact minimized public discriminator still reds at object emission:
`unsupported runtime-IR lowering: Match: scrutinee is not a constructor value`.

**Grounded on `5b123af3`:** the failing site is
`crates/ken-runtime/src/cranelift_backend.rs:2212` —
`return Err(unsupported("Match", "scrutinee is not a constructor value"))` —
inside **`lower_expr` (fn @2035), the ordinary value-producing `Match`
arm** (after the Bool-scrutinee `brif` block 2185–2210, the fall-through
requires `let Lowered::Constructor { .. } = lowered_scrutinee` and
refuses everything else). The Architect's focused diagnostic proved the
scrutinee is source `Var(0)` already bound to
`Lowered::ProcessExitStatus { value }`. PX7-N extended **only** the
computational/tree-producing path (`lower_computational_match_expr` @1646,
`lower_computational_producer_expr` @1667,
`lower_computational_match_value_composed` @1930, over
`struct ComputationalEliminatorFrame` @1615); its ordered stack **ends
before** the downstream ordinary value eliminators, so a dynamic branch
can be merged as a final process result **while its constructor payload
is still an intermediate value** — the later ordinary consumer then
receives `ProcessExitStatus`.

**The Architect's mechanism ruling — sanction (b), and what it is NOT:**

1. **NOT terminal propagation.** By the time ordinary `Match` sees
   `ProcessExitStatus`, the intermediate constructor/payload identity is
   already destroyed; accepting/decoding/forwarding it cannot recover which
   `Result` / `ResourceBracketResult` constructor or payload existed. **Keep
   the ordinary refusal fail-closed** (do NOT teach ordinary `Match` to
   consume `ProcessExitStatus`).
2. **NOT surface reclassification.** `RuntimeExpr::ComputationalMatch` records a
   checked-core match whose recursive hypotheses are computational; an ordinary
   `Result` match has **no** such IH. Marking it computational conflates two
   independent properties, and cannot solve this site locally — the consumer's
   scrutinee is only `Var(0)`; the producer relation lives across the
   continuation/application boundary. **Do NOT relabel ordinary surface matches
   as computational recursion.**
3. **The missing mechanism is heterogeneous composition** — the deforestation
   spine must be extended so an ordinary constructor match contributes a
   **value-eliminator frame after the computational frames**.

## Mechanism (Architect-sanctioned — FIXED semantics)

Generalize the deforestation spine so a frame is **either** a computational
frame (PX7-N) **or** an ordinary value-eliminator frame. The representation may
be an enum (computational vs ordinary) **or** two typed frame stacks; either is
acceptable, but these semantics are **FIXED**:

1. **A computational frame** retains cases, default, recursive positions, and
   its own environment/IH discipline **exactly as PX7-N** — unchanged.
2. **An ordinary frame** retains ordinary cases, binder arity, default, and
   its own environment; it has **no** recursive-IH fields.
3. **Install-before-lower.** When a computational producer's value flows into
   a **known ordinary-match continuation**, install the ordinary frame **before
   lowering/merging the producer**. Selected producer branches **recurse
   through that frame**; they do **not** materialize an intermediate
   `Lowered` aggregate first.
4. **Ordinary-frame selection** uses **exact constructor identity**, checks
   **binder arity**, **prepends constructor arguments** to that frame's
   environment, applies the selected case body, and continues through any
   remaining frames. **A miss returns that frame's exact default
   trap.**
5. **Merge only at the end.** Dynamic Bool / `HostResult` / conditional
   branches merge **only after ALL** computational **and** ordinary frames
   have produced the genuine final scalar / `ExitCode`. **No** intermediate
   constructor, nested payload, `Result`, or record may pass through
   `merge_branch_value` (@1998) or `emit_process_exit_status` (@3875).
6. **General + resource-independent.** The bridge may be recovered at the
   backend's **call/continuation seam** if the relation is structurally
   unambiguous; **if the current `RuntimeExpr` loses that relation, add an
   explicit checked-host IR composition/continuation form.** Do **NOT**
   pattern-match `withResource`, a symbol name, or a resource constructor.
   Do **NOT** add a dynamic aggregate / value representation or any
   ABI/wire shape. Same deforestation principle as PX7-L/M/N, one boundary
   further (across the computational→ordinary seam).

## Acceptance criteria (the Architect's required proof net)

- **AC1 — positive bridge fixture (resource-independent).** A public
  checked-Ken fixture whose **dynamic computational producer yields an outer
  constructor containing an intermediate aggregate payload**; ordinary
  continuations match the **outer** constructor and **then** the **inner**
  payload, **consume both payload directions**, and choose final host
  trees/exits. Compare **complete** interpreter and **same-linked-artifact**
  observations — they agree exactly in both directions.
- **AC2 — bridge-removal mutation recovers the exact failure.** Removing the
  composition bridge makes the ordinary `Match` over `Var(0)` receive
  `ProcessExitStatus` — i.e. it recovers the **exact** reported
  object-emission boundary (`scrutinee is not a constructor value` at the
  ordinary `lower_expr` Match). This is the load-bearing negative.
- **AC3 — ignored-payload twin is an OPPOSITE only.** Keep a byte-near
  ignored-payload variant **only** as the opposite control; it is **not**
  the positive proof (the positive proof consumes both payload
  directions).
- **AC4 — distinct reaching controls, distinguishable default identities.**
  Distinct reaching controls for: **computational-frame miss**, **first
  ordinary-frame miss**, **later ordinary-frame miss**, **arity mismatch**,
  **frame environment / binder order**, **nested payload kind**, and
  **final scalar-versus-`ExitCode` merge**. Each frame's default must have
  an **observably distinguishable identity** (distinct trap code/message or
  inspected compiled trap identity), so a wrong-frame collapse fails a
  control. *(This is the PX7-N AC5-vacuity carry-lesson generalized across
  the heterogeneous stack — a code-present discriminator is vacuous unless
  each frame-relative edge is reached with distinct identities.)*
- **AC5 — landed suites preserved.** PX7-L, PX7-M, and PX7-N focused suites
  stay green — **including PX7-N's active-versus-outer default-collapse
  mutation** (the heterogeneous generalization must not regress the
  homogeneous computational fusion).
- **AC6 — structural guards / scope held.** Ordinary `Match` **still rejects
  `ProcessExitStatus`** (the fail-closed refusal is retained, not removed);
  **no** resource / PX7-F, dynamic-value-representation, ABI/wire, kernel,
  spec, or conformance movement. No-regression = **green in CI** (targeted
  `-p ken-runtime` / `-p ken-cli --test <name>` locally only; **NEVER
  `--workspace`**).

## Do-not-reopen guards

- Do **NOT** teach ordinary `Match` / the `:2212` consumer to accept, decode,
  or forward `ProcessExitStatus` — the payload identity is already destroyed
  there; keep the refusal fail-closed. The repair is the composition bridge,
  **upstream**.
- Do **NOT** relabel an ordinary `Result`/constructor match as
  `RuntimeExpr::ComputationalMatch` — an ordinary match has no computational
  IH; the two properties are independent.
- Do **NOT** pattern-match `withResource`, a symbol name, or a resource
  constructor; the bridge is **general and resource-independent**.
- Do **NOT** add a dynamic aggregate / new value representation, raw tag, or
  any wire/ABI shape.
- Do **NOT** touch `ResourceOpenMode` / the PX7-F surface / PX7-R substrate /
  kernel / spec / conformance / Ward.
- **Preserve everything landed** — the full PX7-L + PX7-M + PX7-N suites and
  all existing computational fusion, static-ctor, dynamic-`HostResult`,
  Bool, and identity/capability negatives. You **ADD** the heterogeneous
  ordinary-frame path.

## Scope boundary (hard)

Touches `crates/ken-runtime/**` (cranelift lowering + IR/differential; and,
**only if** the current `RuntimeExpr` loses the producer→consumer relation, an
explicit checked-host IR composition/continuation form in
`crates/ken-elaborator/**`) + new resource-independent tests. **NO** `spec/` /
`conformance/` / Ward / kernel / ABI-wire / `ResourceOpenMode` /
PX7-R-substrate / PX7-F-surface change.

## Route

**Architect §14 ONLY — NO CV** (no `spec/`/`conformance/`/Ward/kernel/ABI
change; crates + tests only). **ONE branch**
(`wp/px7-o-heterogeneous-eliminator-frame-composition`), **ONE Decision.**
No-regression = **green in CI**. Re-ground the exact anchor lines on
`origin/main @ 5b123af3` before building.

## Grounding anchors (verified on `origin/main @ 5b123af3`)

`crates/ken-runtime/src/cranelift_backend.rs`:
- Ordinary value-producing `Match` (the refusing consumer): `fn lower_expr`
  **@2035**; Bool-scrutinee `brif` block **2185–2210** (can itself yield
  `ProcessExitStatus`); the fall-through refusal
  `unsupported("Match", "scrutinee is not a constructor value")` **@2212**
  (requires `Lowered::Constructor`).
- PX7-N computational frame stack (generalize this): `struct
  ComputationalEliminatorFrame` **@1615**; `fn
  lower_computational_match_expr` **@1646**; `fn
  lower_computational_producer_expr` **@1667**; the composition push
  `composed.push(ComputationalEliminatorFrame { … })` **@1863**; `fn
  lower_computational_match_value_composed` **@1930**.
- Final-merge sites (nothing intermediate may pass here): `fn
  merge_branch_value` **@1998** (`Lowered::ProcessExitStatus { value } =>
  Ok((value, true))` @2006); `fn record_merge_kind` **@2017**; `fn
  emit_process_exit_status` **@3875**; `Lowered::ProcessExitStatus` variant
  **@1480**.
- Existing PX7-N frame-construction fixtures (extend the pattern for the
  heterogeneous controls): the `ComputationalEliminatorFrame { … }` test
  builders around **@4414–@4464**.
