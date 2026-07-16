# PX7-N — checked-host lowering: nested computational-eliminator composition/fusion (compiler prerequisite)

- **ID:** PX7-N · **Owner:** Team Runtime · **Size:** M · **Risk:** High
- **Objective:** Add an explicit **composition/fusion path** for a nested
  `RuntimeExpr::ComputationalMatch` used as the **producer** of another
  computational eliminator, so an **intermediate Ken aggregate `Result`**
  produced by an inner computational match **survives (is deforested)** into the
  outer eliminator instead of being **prematurely collapsed to
  `Lowered::ProcessExitStatus`**. This is a **general, resource-independent**
  compiler capability — the **same deforestation principle as PX7-L/PX7-M, one
  composition level deeper**. PX7-F is its (thrice-deferred) consumer.
- **Depends on:** landed **PX7-L** and **PX7-M** (both on `origin/main @
  0920adea`). Branches off `origin/main @ 0920adea`. **Does NOT depend on
  PX7-F** — it is PX7-F's third lowering prerequisite.
- **Feeds:** unblocks **PX7-F's (again-)deferred public linked-native AC2/AC3
  evidence** (Architect-ruled `evt_1nz919gmq6cwn`); and any nested higher-order
  host sequence whose inner computation returns an aggregate `Result` the outer
  continuation matches.
- **Gate:** no new gate. **Route = Architect §14 only** (compiler-lowering
  soundness / native-differential correctness). **No CV** — no `spec/`,
  `conformance/`, Ward schema, kernel, ABI-wire, or PX7 bracket-semantic change.

## The defect (Architect-grounded, `evt_1nz919gmq6cwn`)

The hard-stop is real, but the "terminal `ProcessExitStatus` propagation"
framing is **one layer too late**. `host_exit` is ordinary Ken `Ret … ExitCode
code`, **not** an OS-terminal effect; `Lowered::ProcessExitStatus`
(`cranelift_backend.rs:1480`) is the compiler's classification of a **final
process result**, and is **not** a Ken value that an ordinary `Result` match may
consume or transparently propagate. The premature collapse occurs **earlier**:

1. An **outer** computational eliminator encounters a nested
   `RuntimeExpr::ComputationalMatch` whose result is an **intermediate `Result
   FileError (ResourceBracketResult …)`**.
2. `lower_computational_match_expr` (`:1615`) has **no nested-computational-
   producer arm**, so its fallback calls ordinary `lower_expr` on the inner
   eliminator.
3. The inner dynamic branches reach `merge_branch_value` (`:1931`).
4. In a **process object**, that helper converts every non-`Int`,
   non-`ProcessExitStatus` branch result through `emit_process_exit_status`
   (`:3808`, called at `:1941`) and reports `is_exit = true`.
5. The intermediate Ken `Result` is therefore **destroyed** and returned as
   `Lowered::ProcessExitStatus`. The later source-level **ordinary** `Result`
   match correctly refuses it at `:2145` — `unsupported runtime-IR lowering:
   Match: scrutinee is not a constructor value`.

The byte-near `minimized_public_result_ignored_is_green` twin is green **only
because it never inspects the already-corrupted intermediate value** — a useful
opposite, **not** evidence that terminal propagation is correct.

## Fixed inputs — DO NOT REOPEN (Architect-ruled `evt_1nz919gmq6cwn`)

- **This is a separate Runtime-owned checked-host prerequisite.** Do **NOT** add
  a resource intrinsic, recognize a bracket by symbol, or add a resource-specific
  branch. The checked Ken definition is the single trust boundary.
- **Do NOT teach ordinary `Match` to accept or short-circuit
  `ProcessExitStatus`.** The fix is upstream (compose the eliminators), not at
  the ordinary-match consumer.
- **Do NOT** materialize a heap/dynamic Ken aggregate, a raw runtime tag, or a
  new runtime value representation; add **no** wire/ABI shape.
- **Do NOT change `ResourceOpenMode` to `Int`** and do not touch the PX7-F
  surface / bracket / full-observation contract.
- **Preserve everything PX7-L and PX7-M landed** — both focused suites and all
  existing computational Bool / static-constructor / dynamic-`HostResult`
  producer paths, static direct-`Vis`, and identity / capability negatives stay
  unchanged. This WP **adds** the nested-composition path; it does not alter the
  existing arms.

## Mandated deliverable — the Architect's sanctioned mechanism (composition/fusion)

Add an explicit composition path for a nested `RuntimeExpr::ComputationalMatch`
used as the **producer** of another computational eliminator. It **may** factor
through a helper, but these semantics are **fixed**:

1. **Do NOT lower the inner computational match to a `Lowered` value first.**
2. For each **selected inner producer case**, **recursively lower that case
   body as the producer** for the **outer** eliminator's cases/default.
3. **Keep the two environments distinct.** The **inner** case environment holds
   its induction hypotheses and constructor arguments **ahead of** its current
   producer environment; the **outer** eliminator environment remains
   **unchanged**.
4. **Preserve the inner and outer defaults separately:** a missing **inner** case
   traps with the **inner** default; a missing **outer** case traps with the
   **outer** default.
5. **Dynamic control-flow branches merge only after the recursive outer lowering
   has produced the genuine final scalar/`ExitCode` result.** **No** intermediate
   constructor, `Result`, or record may pass through `merge_branch_value`
   (`:1931`) or `emit_process_exit_status` (`:3808`).
6. **Preserve** exact constructor identity, binder arity/order, recursive-
   position validation, and the existing **final scalar-versus-`ExitCode` kind
   guard**.

## Companion spillover — the PX7-L capability-policy consistency repair (FOLD IN HERE)

The `erasure.rs` repair carried in the PX7-F durable red object (`e96902bc`,
`lower_runtime_selected_host_operation` capability gate) is **correct and must
land in THIS prerequisite** — **not** ride silently in PX7-F, and **not** as a
second PX7-L erratum branch (Architect-ruled).

- The already-landed **static** path classifies `FsHandleMetadata` and
  `ResourceRelease` as **capability-less** because the **resource token is their
  authority-bearing operand**; the **runtime-selected** path
  (`lower_runtime_selected_host_operation`, `erasure.rs:624`, `is_ambient()`
  capability gate at `:682` on `0920adea`) must use the **same classification**.
- **Ambient** operations remain capability-less; **every other FS operation
  still consumes exactly one capability operand.**
- Ship an **exhaustive static-versus-runtime-selected capability-policy
  discriminator** (see AC7) so the two paths' classifications are proven
  identical, not just patched for two ops.
- **Re-author** this repair on the PX7-N branch (do not depend on PX7-F
  sources). After PX7-N lands, **PX7-F rebases and drops its duplicate diagnostic
  copy**; the durable red object may remain as evidence until then.

## Acceptance criteria (Architect's required proof net)

- **AC1 — resource-independent nested-`ITree.bind` fixture, interp↔native
  agree.** A checked Ken program (no resource content) with a **nested generic
  `ITree.bind`**: the **inner** computation dynamically returns an **aggregate
  `Result`**; the **outer** continuation **matches** that result, **consumes each
  arm's payload**, and selects final host trees/exits. Real interpreter and
  linked-native observations **agree** as **complete canonical observations**
  (not trace proxies).
- **AC2 — both `Result` directions, payload-consuming.** Both `Ok`/`Err`
  directions deterministically reach the real interpreter and the same linked
  artifact and compare complete observations.
- **AC3 — fallback-restoration recovers the exact premature boundary.** A
  mutation restoring the current fallback/ordinary-lower path **must recover the
  exact premature `ProcessExitStatus` / non-constructor boundary** (`Match:
  scrutinee is not a constructor value`) on the AC1 fixture (proves the
  composition path is load-bearing).
- **AC4 — env/binder opposite.** A mutation that **swaps inner-versus-outer
  environments** or **changes binder order** must **fail closed or change the
  complete observation**.
- **AC5 — fail-closed controls.** **Missing/default, wrong-arity,
  malformed-recursive-position, payload-kind, and final merge-kind** controls
  remain **fail-closed** (assert the specific rejection, not merely `is_err`).
- **AC6 — no-regression on PX7-L/PX7-M/existing.** The **full PX7-L and PX7-M
  focused suites** and all existing computational producer paths remain
  **green**. No-regression = **green in CI** (targeted `-p ken-runtime` /
  `-p ken-elaborator` / `-p ken-cli --test <name>` locally only; **NEVER
  `--workspace`**).
- **AC7 — capability-policy discriminator (companion).** An **exhaustive
  static-versus-runtime-selected capability-policy discriminator** proves the two
  paths classify **every** `HostOpV1` identically (ambient → capability-less;
  `FsHandleMetadata`/`ResourceRelease` → capability-less; every other FS op →
  exactly one capability operand). The pre-repair runtime-selected classification
  **fails** it.
- **AC8 — scope boundary held.** **No** PX7 resource-table, bracket, Ward
  schema, kernel, host ABI, spec, or conformance content is in this WP's diff
  (`git grep` / diff review).

## Do-not-reopen guards

- Do NOT lower the inner computational match to a `Lowered` value before
  composing; recurse on each selected inner producer case as the outer
  producer.
- Do NOT teach ordinary `Match` / the `:2145` consumer to accept or short-circuit
  `ProcessExitStatus`; the repair is the composition path, upstream.
- Do NOT let any intermediate constructor/`Result`/record pass through
  `merge_branch_value` / `emit_process_exit_status`.
- Do NOT add a resource intrinsic, bracket-by-symbol, untyped dynamic tree, heap
  aggregate, raw tag, new value representation, or wire/ABI shape.
- Do NOT change `ResourceOpenMode`, the PX7-F surface, Ward, kernel, or ABI wire.
- Do NOT open a second PX7-L erratum branch for the capability repair — it folds
  here.

## Grounding anchors (re-ground on `origin/main @ 0920adea` before building)

- `crates/ken-runtime/src/cranelift_backend.rs`:
  - `lower_computational_match_expr` (`:1615`) — the fallback that ordinary-lowers
    the inner eliminator; the arm to extend with nested composition.
  - `merge_branch_value` (`:1931`) / `record_merge_kind` (`:1950`) — arm merge;
    `emit_process_exit_status` (`:3808`, invoked at `:1941` with `is_exit=true`)
    is the **premature-collapse site** for non-`Int`/non-`ProcessExitStatus`
    branch results.
  - `Lowered::ProcessExitStatus` (`:1480`) — the final-process-result
    classification (NOT a Ken value; do not teach ordinary `Match` to consume it).
  - The refusal boundary: ordinary `Match` at `:2145` (`scrutinee is not a
    constructor value`); also `:1889`.
- `crates/ken-elaborator/src/erasure.rs`:
  - `lower_runtime_selected_host_operation` (`:624`), `is_ambient()` capability
    gate (`:682` on `0920adea`) — the companion capability-policy repair site.
- Evidence reference (do NOT depend on PX7-F sources; author a **resource-
  independent** fixture): PX7-F durable red object `wp/px7f-system-resource-
  bracket @ e96902bc` — `minimized_public_result_match_reaches_process_exit_status_boundary`
  (red) vs `minimized_public_result_ignored_is_green` (green).

## Diff scope / review route

- **Touches:** `crates/ken-runtime/**` (cranelift lowering + any IR/differential
  support) and `crates/ken-elaborator/**` (the `lower_runtime_selected_host_
  operation` capability-policy repair + composition support if the checked-core
  must carry identities), plus the new/added lowering-precursor tests. **No**
  `spec/`, `conformance/`, Ward, kernel, ABI-wire, `ResourceOpenMode`, or PX7
  substrate change.
- **Route:** **Architect §14 only** (compiler-lowering soundness; TCB-relevant).
  **No CV.** Gates: standard no-regression (**CI-green**, per `COORDINATION §12`;
  never local `--workspace`).
- **One branch, one Decision:**
  `wp/px7-n-nested-computational-eliminator-composition` off `origin/main @
  0920adea`.
