# PX4B — Base native-CLI production spine (Runtime, predecessor to PX5)

- **ID:** PX4B · **Owner:** Team Runtime · **Size:** L · **Risk:** High (new
  production source-to-native route; identity/provenance is trust-adjacent).
- **Objective:** Build the **real, effect-free** production route that turns a
  checked Ken `main` into one identity-bound linked native artifact through the
  public CLI/library — the base producer that PX5 then extends with effect
  lowering. Today no such route exists (the CLI is interpreter-only; the
  entrypoint packager rejects effects and runtime args; nothing constructs the
  effect IR). PX5 cannot be built on a fixture-fed `RuntimeExpr` — it must
  consume a **public production result**, and this WP produces it.
- **Depends on:** PX0–PX4 (merged) — PX4's native-entrypoint artifact/packaging
  substrate is the low-level target this route drives. **Feeds:** PX5 (native
  effect lowering) consumes this WP's public production result; PX6 (differential
  harness) then compares native vs. interpreter. **Gate:** G5-perf / native lane.
- **Design source of truth:** Architect ruling **`evt_3bsx082257v92`** (PX5
  producer-boundary hard-stop resolution). This frame transcribes its §1–§4 into
  deliverables + ACs. ADR-0018 remains the effect contract; the §0.5 ingress
  (`evt_3nkr1vx55ca1n`) and identity option-b (`evt_7hj00prqfjqs2`) rulings bind.

## Fixed inputs — DO NOT REOPEN (cite, do not relitigate)

- **This is a PREDECESSOR, not an expansion of PX5** (Architect ruling §4). The
  split is architectural and independently discriminable: **PX4B owns the base
  production spine** (checked source/main admission, plan insertion + parent-owned
  identity, exact closure erasure, bound `RuntimeProgram`→PX4 object/link
  packaging, one real CLI artifact path); **PX5 owns the semantic effect
  extension only** and MUST consume PX4B's public result. Do not build effect
  lowering here; do not fold PX4B into PX5.
- **Identity rulings stand** — the plan is **hash-free**; the final
  `core_semantic_hash` covers it by containment; a child never contains a parent
  digest; the adjacent plan hash is transport identity only
  (`evt_7hj00prqfjqs2`). The §0.5 ingress carrier stands (`evt_3nkr1vx55ca1n`).
- **The interpreter `ken run` stays the PX6 oracle.** PX4B may add an explicit
  native package/run path, but **must not silently change the oracle**;
  differential authority remains the interpreter until PX6 has evidence.
- **NO hard-coded `prelude::…` identity strings; NO public/default
  `NativeEntrypointPlanV1` constructor.** Admission resolves identities from the
  live `ElabEnv`; construction is internal to the production transaction.
- **A test helper is not a producer.** Zero hand-built `RuntimeExpr`; the object
  builder must not accept a naked `RuntimeExpr`. Fixture-fed IR earns **no** AC
  evidence (the exact proxy path PX6 rejected).
- **⛔ Targeted local builds only** (`scripts/ken-cargo`, scoped); the full
  workspace/locked/conformance gates run in CI. No-regression = **green in CI**.

## Mandated deliverable outline — each section ends in a concrete choice

1. **Checked-main admission (Elaborator-owned).** EXTRACT — do not copy — the
   semantics currently private in `ken-cli/src/lib.rs:146-218` (`resolve_main`,
   `declared_fs_authority`, `entrypoint_has_abi`) into **one new Elaborator-owned
   admission operation** that consumes the **same live `ElabEnv`** that elaborated
   the source and: (1) requires an anonymous program boundary with exactly one FS
   authority declaration; (2) resolves `main`, `ProcessInput`, `ProgramCaps`,
   `MkProgramCaps`, `Cap`, the selected `Auth` constructor, `HostIO`, `ExitCode`
   from that env; (3) requires the closed supported effect-row policy; (4)
   constructs `ProcessInput → ProgramCaps a → HostIO a ExitCode`; (5) calls
   `ken_kernel::convert_type(&env.env, Context::new(), actual, expected)`. Result
   = a **checked-main descriptor** (GlobalIds + declared authority); during
   checked-core emission those IDs map to the package's canonical `StableSymbol`
   identities. **Both** the interpreter `run_program` **and** the native compiler
   path call this ONE operation (API choice cannot change term legality).
2. **Base producer — a dedicated program/native compilation mode.** Add the mode
   at the existing compiler-driver spine; **do not weaken generic package/target
   compilation.** One ordered transaction: (1) elaborate sources once into
   `ElabEnv`; (2) run the checked-main admission (#1); (3) pass the checked
   descriptor into `emit_package_from_env`; (4) `emit_package_from_env`
   constructs the **hash-free semantic `NativeEntrypointPlanV1`** and inserts its
   canonical bytes into `CheckedCoreSemanticInputs` **before**
   `emit_checked_core_package` finalizes `core_semantic_hash`; (5) select
   canonical `main`, build its exact closure, erase that closure with
   `erase_checked_core_package_for_target` into **one** identity-bound
   `RuntimeProgram`; (6) package the exact main adapter + artifact from
   **`RuntimeProgram` + plan + closure/entrypoint metadata** (the production
   object builder must NOT accept a naked `RuntimeExpr`); (7) return the checked
   package, plan, target/closure, `RuntimeProgram`, executable-entrypoint
   metadata, and object/linker package as **one identity-consistent result**; (8)
   expose that result through a **real CLI native package/run path**.
3. **Exact lane replacement (do not delete/relax the negatives).** Make report
   state **derived from the artifacts actually returned, never toggled from
   intent**: `runtime_lowering_unavailable` becomes Emitted only when the exact
   selected closure has produced + validated the returned `RuntimeProgram`;
   `native_artifact_unavailable` becomes Emitted only after the identity-bound
   linked artifact exists + validates; `entrypoint_runtime_arguments_unavailable`
   remains for arbitrary parameterized targets, with the **only** base exception
   being the checked, plan-bound **two-binder program ABI** (arguments supplied by
   the generated adapter in source order). **Split
   `host_effect_or_foreign_entrypoint`:** foreign calls stay rejected; any
   executable `Vis`/host operation becomes the **named
   `host_effect_lowering_unavailable` lane** (which PX5 later replaces for its 5
   pinned ops only — the other 9 + all foreign stay unavailable).
4. **CLI production path + oracle preservation.** Wire the real CLI native
   package/run path (`ken-cli`) onto the mode (#2); keep today's interpreter
   `ken run` as the untouched PX6 oracle. State the exact new CLI surface (a
   native build/package subcommand or flag) and confirm it does not alter the
   interpreter path's term legality or the oracle's observations.

## Acceptance criteria (testable) — from the ruling's 7 discriminators

- **AC1 — real source through the public producer.** A real `.ken` source goes
  through the public CLI/library producer with **zero hand-built `RuntimeExpr`**;
  the object builder rejects a naked `RuntimeExpr` structurally.
- **AC2 — admission fails closed on authority/type mismatch.** A header-authority
  / type-authority mismatch fails **before** plan/package emission (admission
  rejects; no plan, no core hash, no artifact produced).
- **AC3 — named rejections retained.** Wrong main ABI and arbitrary runtime
  arguments retain their **named** rejections; only the checked plan-bound
  two-binder program ABI is admitted.
- **AC4 — effect-free main reaches one linked artifact.** An effect-free
  exact-ABI `HostIO`/`Ret` `main` that consumes real `ProcessInput` and returns a
  real `ExitCode` reaches **one linked artifact**, preserving byte-accurate
  argv/env/cwd and the shared `ProcessExitCode → i32` mapping (PX4's mapper).
- **AC5 — one Vis discriminates to the effect lane.** The **same** source changed
  to contain one `Vis` differs **only** by reaching the named
  `host_effect_lowering_unavailable` lane (the base negative control PX5 later
  lights up).
- **AC6 — identity mutation chain fails closed.** The plan/core/artifact mutation
  chain fails at the already-ruled identity layers (mutate a plan field w/o hash
  refresh → `core_semantic_hash` mismatch; refresh only core → `artifact_hash`
  mismatch; the plan carries no parent/self hash).
- **AC7 — report cannot claim absent artifacts.** The compiler report cannot
  claim runtime/native emission when the corresponding returned artifact is
  absent (report state derived from returned artifacts, structurally).
- **AC8 — no regression, oracle intact (CI green).** Full workspace/locked gates
  green in CI; the interpreter `ken run` oracle is byte-unchanged; generic
  package/target compilation unweakened. Targeted local via `scripts/ken-cargo`.

## Do-not-reopen guards

- The Architect ruling `evt_3bsx082257v92` is the design source; do NOT re-derive
  the split or invent effect lowering here. A genuinely new fixed boundary
  hard-stops to the Steward/Architect (do not synthesize fixture IR to proceed).
- Do NOT change the interpreter oracle, weaken generic compilation, delete the
  NC10/NC20 negatives, or expose a public/default plan constructor.
- Do NOT build PX5's effect extension in this WP; PX5 consumes this WP's public
  result.

## Grounding anchors

Architect ruling `evt_3bsx082257v92` (thr_rh6fmbpsn7ab). Extract source:
`ken-cli/src/lib.rs:146-218` (`resolve_main`/`declared_fs_authority`/
`entrypoint_has_abi`), `ken-cli/src/main.rs:255-264` (interpreter-only today).
Compiler driver: `compiler_driver.rs:443-446` (`emit_package_from_env` →
`CheckedCorePackage` only), `:540-560` (entrypoint packager rejects
effects/runtime args), `:1244-1253` (`runtime_lowering_unavailable` / NC10).
Erasure: `erasure.rs:77-125` (`RuntimeProgram` consumer), `:454+` (no
`RuntimeExpr::Effect` producer). Kernel: `ken_kernel::convert_type`. ADR-0018
(effect contract); PX5 frame `PX5-native-effect-lowering.md` (the consumer).
