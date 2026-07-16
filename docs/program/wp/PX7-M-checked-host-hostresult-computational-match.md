# PX7-M — checked-host lowering for a dynamic `HostResult` match in a computational tree producer (compiler prerequisite)

- **ID:** PX7-M · **Owner:** Team Runtime · **Size:** M · **Risk:** High
- **Objective:** Extend the checked-host native lowering so that a **dynamic
  host `Result` match sitting inside a computational *tree producer*** — i.e. the
  recursively produced `HostIO` tree is itself selected by matching the dynamic
  result of an already-lowered host effect — deforests correctly to a sound
  linked-native artifact. This is a **general, resource-independent** compiler
  capability, **downstream of and distinct from PX7-L**. PX7-F is its first
  consumer.
- **Depends on:** landed **PX7-L** (`origin/main @ 736a9ef5`:
  `lower_runtime_selected_host_operation`, computational-IH retention, reply
  binder at cutoff 1) and the current checked-host / cranelift lowering pipeline.
  Branches off `origin/main @ 736a9ef5`. **Does NOT depend on PX7-F** — it is
  PX7-F's second lowering prerequisite.
- **Feeds:** unblocks **PX7-F's (again-)deferred public linked-native AC2/AC3
  evidence** (Architect-ruled `evt_4nsnbj80e9d3w`); and any higher-order host
  sequence that branches on a dynamic host `Result` reply to select the next
  computation.
- **Gate:** no new gate. **Route = Architect §14 only** (compiler-lowering
  soundness / native-differential correctness). **No CV** — no `spec/`,
  `conformance/`, Ward schema, kernel, ABI-wire, or PX7 bracket-semantic change.

## The defect (Architect-grounded, `evt_4nsnbj80e9d3w`)

PX7-L correctly handles the runtime-selected **operation coproduct**, the
computational induction hypothesis, and the reply binder. The **new** failure
occurs *later*: when the recursively produced `HostIO` tree is selected by
matching the **dynamic result of an already-lowered host effect**.

- Checked `withResource` receives a dynamic `Result FileError (Resource
  FsHandle)` from `FsOpen`; the producer matches that reply to choose the next
  `HostIO` tree. Cranelift lowers the reply as `Lowered::HostResult`
  (`cranelift_backend.rs:1493` — carries `success`, `ok_constructor`,
  `err_constructor`, and both typed payload representations).
- The **value-producing** `RuntimeExpr::Match` path already handles this:
  `lower_dynamic_host_result_match` (`cranelift_backend.rs:2879`, dispatched at
  `:2004`).
- But the **tree-producing** producer arm of `lower_computational_match_expr`
  (`cranelift_backend.rs:1596`) only handles a dynamic **Bool** scrutinee, then
  demands a static `Lowered::Constructor` (`:1724`). It has **no dynamic
  `HostResult` arm** — so a branch-producing computational match over a dynamic
  host `Result` is unlowerable: `unsupported runtime-IR lowering:
  ComputationalMatch: tree-producing match scrutinee is not Bool or a
  constructor`.

## Fixed inputs — DO NOT REOPEN (Architect-ruled `evt_4nsnbj80e9d3w`)

- **This is a separate Runtime-owned checked-host prerequisite.** Do **NOT** add
  a resource intrinsic, recognize `withResource`/any bracket by symbol, or add a
  resource-specific special case. The checked Ken definition is the single trust
  boundary.
- **Do NOT change `ResourceOpenMode` to `Int`** and do not touch the PX7-F
  surface — that is a PX7-F companion concern (see below), not this WP.
- **Bounded mechanism only** (below). Do **NOT** introduce an untyped dynamic
  tree, a heap aggregate, a raw result tag, or a new runtime value
  representation. `Lowered::HostResult` already carries the branch condition, the
  exact `Result` identities, and both typed payloads — the prerequisite adds
  **no** wire/ABI shape and **no** new runtime value representation; it only lets
  the existing computational recursor continue inside either already-represented
  `Result` arm.
- **Preserve everything PX7-L landed** — the full PX7-L suite (including the
  consumed runtime-selected reply discriminator), the computational Bool /
  static-constructor computational paths, static direct-`Vis`, and identity /
  capability negatives all stay unchanged. This WP **adds** the dynamic
  `HostResult` producer arm; it does not alter the existing ones.

## Mandated deliverable outline — the Architect's sanctioned mechanism

Extend `lower_computational_match_expr`'s `RuntimeExpr::Match` **producer** arm
with an explicit `Lowered::HostResult` case. **Reuse the *branching discipline*
of `lower_dynamic_host_result_match`, but do NOT call that helper as-is** — it
lowers case bodies with `lower_expr`, which would stop *before* applying the
surrounding computational recursor. For each dynamic result direction:

1. **Branch on `HostResult.success`** into the exact `ok_constructor` and
   `err_constructor` carried by the checked lowering.
2. **Resolve the corresponding producer `RuntimeMatchCase` by exact constructor
   identity**, require **exactly one binder**, and **prepend the matching
   `ok`/`error` payload to `producer_env`**.
3. **Recursively call `lower_computational_match_expr`** on that producer case
   body with the **same eliminator cases/default and unchanged `eliminator_env`**,
   so the selected `HostIO` tree is **deforested rather than materialized**.
4. **Merge the recursively lowered arms** with the existing `merge_branch_value`
   (`:1864`) plus `record_merge_kind` (`:1883`) discipline used by the dynamic
   Bool / If computational producers.
5. **Preserve fail-closed semantics:** a **missing constructor** takes the
   producer match's **default trap**; **wrong arity** rejects specifically; and
   **scalar-versus-`ExitCode` arm disagreement** rejects. No silent or untyped
   fallthrough.

## Acceptance criteria (testable) — the Architect's required precursor discriminators

- **AC1 — resource-independent dynamic-`HostResult`-in-producer fixture,
  interp↔native agree.** A checked Ken program (no resource content) places a
  **dynamic host `Result` match *inside the tree producer traversed by generic
  `ITree.bind`***, with the selected arm producing the **next multi-step
  `HostIO` tree**. Real interpreter and linked-native observations **agree**.
- **AC2 — both directions, payload-consuming.** Exercise **both `Ok` and `Err`**
  directions deterministically. **Each arm must consume its payload** (or
  otherwise make binder order externally discriminating) — an **ignored-payload
  trace is insufficient** (sibling of PX7-L's consumed-reply lesson).
- **AC3 — constructor-only-path mutation fails at the exact boundary.**
  Restoring the current constructor-only producer path **must fail** the AC1
  public fixture at the exact `ComputationalMatch: tree-producing match
  scrutinee is not Bool or a constructor` boundary (proves the new arm is
  load-bearing). A **payload-binder / order mutation** must yield a **distinct
  wrong observation or fail closed**.
- **AC4 — fail-closed controls.** **Missing-case/default, wrong-arity, and
  mismatched branch-result-kind** controls remain **fail-closed** (assert the
  specific rejection, not merely `is_err`).
- **AC5 — no-regression on everything PX7-L/existing landed.** The full PX7-L
  suite (incl. consumed runtime-selected reply), computational Bool /
  static-constructor paths, static direct-`Vis`, and identity/capability
  negatives are **unchanged**. No-regression = **green in CI** (targeted
  `-p ken-runtime` / `-p ken-elaborator` / `-p ken-cli --test <name>` locally
  only; NEVER `--workspace`).
- **AC6 — scope boundary held.** **No** PX7 resource-table, bracket, Ward
  schema, kernel, host ABI, spec, or conformance content is in this WP's diff
  (`git grep` / diff review).

## Do-not-reopen guards

- Do NOT call `lower_dynamic_host_result_match` as-is for the producer arm (it
  `lower_expr`s the bodies and drops the computational recursor) — reuse its
  **branching discipline** with a **recursive** `lower_computational_match_expr`
  on each selected producer case body.
- Do NOT add a resource intrinsic, recognize a bracket by symbol, or add a
  resource-specific special case; the checked Ken definition is the trust
  boundary.
- Do NOT introduce an untyped dynamic tree / heap aggregate / raw result tag /
  new runtime value representation, and do NOT add wire/ABI shape.
- Do NOT weaken any fail-closed control, and do NOT change the existing dynamic
  Bool / static-constructor computational producer paths or the PX7-L suite.
- Do NOT touch `ResourceOpenMode`, the PX7-F surface, `ResourceTableV1`, Ward
  schema, kernel, or ABI wire — this is a **lowering** precursor only.

## Grounding anchors (re-ground on `origin/main @ 736a9ef5` before building)

- `crates/ken-runtime/src/cranelift_backend.rs`:
  - `lower_computational_match_expr` (`:1596`; signature carries `scrutinee,
    cases, default, producer_env, eliminator_env`) — the producer arm to extend;
    demands `Lowered::Constructor` at `:1724`.
  - `lower_dynamic_host_result_match` (`:2879`; dispatched at `:2004`) — the
    **branching-discipline reference**, not to be called as-is.
  - `Lowered::HostResult { success, err_constructor, ok_constructor, … }`
    (`:1493`).
  - `merge_branch_value` (`:1864`) / `record_merge_kind` (`:1883`) — arm merge.
  - `create_policy_tag` (`:1555`) — the analog for PX7-F's companion mode
    projection (NOT part of this WP).
- `crates/ken-runtime/src/artifact_validation.rs:742` — the `ComputationalMatch`
  unsupported-lowering emission (fail-closed reference).
- The exact PX7-F reproducer (parked on `wp/px7f-system-resource-bracket`, after
  the narrow mode projection): `source scripts/ken-env.sh && scripts/ken-cargo
  test -p ken-cli --test px7f_resource_native -- --nocapture` → both
  `linked_public_escape_is_exact_closed` and
  `linked_public_right_denial_preserves_exact_masks` fail at the boundary. Do NOT
  depend on PX7-F sources; author a **resource-independent** fixture for AC1.

## Diff scope / review route

- **Touches:** `crates/ken-runtime/**` (cranelift lowering + any IR/differential
  support) and the new/added lowering-precursor tests. Possibly
  `crates/ken-elaborator/**` only if the checked-core representation must carry
  the `ok`/`err` constructor identities to codegen (re-ground; keep minimal).
  **No** `spec/`, `conformance/`, Ward, kernel, ABI-wire, `ResourceOpenMode`, or
  PX7 substrate change.
- **Route:** **Architect §14 only** (compiler-lowering soundness; TCB-relevant
  because it changes native lowering). **No CV.** Gates: standard no-regression
  (**CI-green**, per `COORDINATION §12`; never local `--workspace`).
- **One branch, one Decision:** `wp/px7-m-checked-host-hostresult-computational-match`
  off `origin/main @ 736a9ef5`.
