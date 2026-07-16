# PX14 — Root-execution posture admission (Runtime)

- **ID:** PX14 · **Owner:** Team Runtime · **Size:** M · **Risk:** Medium
  (startup-admission gate on the trusted-base boundary; a new terminal outcome
  the native init path must emit without a live `ProcessContext`).
- **Objective:** Refuse, by default, to run a program that finds itself started
  as **effective root** (`euid == 0`), with a stable terminal error **before any
  effect** — unless the program **declares** a root-execution allowance. The
  allowance **permits continuing** when already privileged; it **never**
  escalates privilege (caps attenuate). Both executors apply the identical check
  at the same fail-closed startup seam PX5 established.
- **Depends on:** PX5 (merged — the fail-closed startup-posture seam:
  `establish_process_posture_v1` + the `ProcessPostureV1` witness required to
  construct `ProcessContext`, `dec_1gk5vbw2bbg05`) **and** the shared Architect
  capability-model ruling (`evt_7k8n8rwj1xbh1`) **and** PX13 merged (it lands
  ADR-0019, whose process-admission section is PX14's normative basis). **Gate:**
  G-Sec / native-effect lane.
- **Feeds:** the honesty story for privileged deployment; no downstream WP blocks
  on it. Released **after PX13** (Runtime is one ring; PX14 cites ADR-0019).

## Fixed inputs — DO NOT REOPEN (cite, do not relitigate)

Ruled by the Architect (`evt_7k8n8rwj1xbh1`, thread `thr_szhcns1f2mpe`) on the
ADR-0017 honesty boundary + ADR-0018 startup seam; recorded normatively in
ADR-0019 (landed by PX13). Binding:

- **NOT a capability token.** The allowance is **not** a field in the Ken-visible
  `ProgramCaps a` value, **not** an FS right, **not** a forgeable scalar passed
  to generated code, and **not** authority escalation. It is
  **declaration / checked-plan metadata** the compiler binds into the native
  entrypoint plan/hash. A **CLI flag or environment variable may NOT add it.**
- **v1 predicate is EXACTLY `geteuid() == 0`.** Query it through the sole audited
  `ken-host` Linux-direct boundary (the pinned **rustix** process API). Real UID,
  saved-set UID, filesystem UID, ambient/permitted/effective Linux
  `capabilities(7)` sets, user namespaces, and securebits are **different**
  privilege models and are **deferred** (not this WP).
- **Declaration surface = a header marker.** Spelled
  `program capabilities FS <authority>, RootExecution Allow`. **Omission means
  deny.** It extends the existing `program capabilities …` header
  (`ken-cli/src/lib.rs:428`) as checked-plan metadata; it does not alter the
  `ProgramCaps` inductive or `MkProgramCaps`.
- **One shared startup-admission function.** It consumes an **immutable
  effective-UID snapshot** + the declaration, and is called by **both** the
  interpreter and native init **before** constructing `ProcessContext`,
  **before** minting/inserting any cap-table grant, and **before** the first
  effect. The `ProcessContext` posture witness records that **both** this
  admission **and** the SIGPIPE posture completed.
- **Refuse-root outcome = `TerminalErrorV1::RootExecutionDenied`** (a new **unit**
  variant). It is a **startup terminal outcome, not a fake HostOp event**: empty
  effect trace, empty FS delta, empty stdout/stderr, exit status through the one
  shared `ProcessExitCode -> i32` mapper (`process_exit_status`). The native init
  path **must be able to write that canonical terminal observation even though it
  deliberately returns no live `ProcessContext`.**
- **Honesty boundary (ADR-0017).** Runtime-trusted + discriminator-tested, never
  kernel-proved. Zero kernel rule, zero Ken postulate, no confinement proof. NO
  affine/linear types. Confinement is honest-boundary, never kernel-proved.

## Scope

**In scope:** the euid snapshot at the audited rustix boundary; the
`RootExecution Allow` header-marker parse into checked-plan metadata (bound into
`NativeEntrypointPlanV1` + its hash); the one shared admission function; its
call at both executors' startup seam before `ProcessContext`; the
`TerminalErrorV1::RootExecutionDenied` variant + the native init path that emits
its canonical terminal observation with no live context; the exit mapping.

**Out of scope:** any richer privilege model (real/saved/fs uid, caps(7),
user-ns, securebits) — deferred; any privilege *escalation* (setuid/seteuid) —
forbidden by construction (the marker only permits continuing); FS mode/owner
ops (PX13); path-root grammar (PX15). No test-time environment override in
production paths — **test injection at the host-observer seam only.**

## Mandated deliverable outline — each section ends in a concrete choice

1. **Effective-UID snapshot (`ken-host` rustix boundary).** In the audited
   `abi_v1` module, read `geteuid()` via the pinned rustix process API exactly
   once at startup and carry it as an immutable snapshot. No `libc`, no `/proc`
   parse, no env. This is a new call on the existing audited unsafe/direct
   boundary; enumerate it in the trusted-surface delta.
2. **Header-marker parse → checked-plan metadata.** Extend the `program
   capabilities …` header grammar with the optional `, RootExecution Allow`
   clause; carry it as a boolean in the **checked plan** (`NativeEntrypointPlanV1`
   / the checked-source plan the interpreter also sees), **not** in the
   `ProgramCaps` Ken value. Omission ⇒ `false` (deny). Bind it into the plan hash
   so a plan-field mutation without a hash refresh fails closed (ADR-0018
   identity discipline). A CLI/env path that tries to set it is rejected
   structurally (there is no such input).
3. **One shared admission function.** `fn admit_root_execution(euid_snapshot,
   allow_root: bool) -> Result<(), RootExecutionDenied>`: `Err` iff `euid == 0 &&
   !allow_root`; `Ok` otherwise. Pure, total, no I/O. Call it from **both** the
   interpreter startup and the native `abi_v1` init, **before** `ProcessContext`
   construction (`abi_v1.rs:~441`), **before** any cap-table grant, **before**
   the first effect. Extend the `ProcessPostureV1` witness so its construction
   requires this admission to have run (same shape as the SIGPIPE witness).
4. **`RootExecutionDenied` terminal outcome.** Add the unit variant to
   `TerminalErrorV1` (`effect_v1.rs`). On denial, both lanes produce the
   canonical `EffectObservationV1{ stdout:[], stderr:[], filesystem_delta:[],
   terminal_error:Some(RootExecutionDenied), effect_trace:[], exit_status }` with
   `exit_status` from `process_exit_status(ProcessExitCode::Failure(_))`
   (`native_process_entrypoint.rs:108`). The **native init path must emit this
   without a live `ProcessContext`** — the denial precedes context construction,
   so the terminal-observation writer cannot depend on the context.
5. **Honesty + trusted-surface delta.** Source discloses tested/target-validated,
   never proved; enumerate the new euid read + its obligation. No new unsafe
   outside the audited `abi_v1` boundary.

## Acceptance criteria (testable — the ruling's controls)

- **AC1 — non-root proceeds.** The same checked program/plan runs to completion
  at non-root (euid ≠ 0) with or without the marker; observably identical interp
  vs native.
- **AC2 — root-without-marker denies, zero effect (non-degenerate w/ AC3).**
  euid 0 **without** `RootExecution Allow` yields exactly
  `TerminalErrorV1::RootExecutionDenied`, **zero host-leaf calls**, empty
  trace/delta/stdout/stderr, and the mapped failure exit — **before any effect**;
  interp and native identical.
- **AC3 — root-with-marker proceeds.** euid 0 **with** the marker runs to
  completion (same program as AC2) — so a flipped predicate inverts both AC2/AC3
  and fails. The allowance only *permits continuing*; it performs **no**
  setuid/seteuid (structural: grep clean).
- **AC4 — marker is unforgeable via CLI/env/plan mutation.** No CLI flag or env
  var can manufacture the allowance; a plan-field mutation of the marker without
  a hash refresh fails closed (plan-hash mismatch). The marker is not in the
  `ProgramCaps` Ken value (structural).
- **AC5 — one shared pure checker.** Interp and native both route through the
  single `admit_root_execution`; assert there is no second euid check or second
  admission path. The `ProcessContext` witness requires the admission ran.
- **AC6 — confined + honest, CI-green.** New euid read confined to the audited
  `abi_v1`; interp keeps `forbid(unsafe_code)`; disclosure tested/validated, not
  proved; no kernel/spec/conformance movement. Test injection at the
  host-observer seam only (no production env override). **No-regression = green
  in CI**, never a local `--workspace` run.

## Do-not-reopen guards

- Do NOT model the allowance as a capability/right/`ProgramCaps` field or a
  runtime scalar — it is checked-plan metadata bound into the plan hash.
- Do NOT widen the predicate beyond `geteuid()==0` (no real/saved/fs uid, no
  caps(7)/user-ns/securebits) — those are deferred, explicitly.
- Do NOT let a CLI flag / env var set the marker; do NOT perform any privilege
  escalation (setuid/seteuid) — the marker only permits *continuing*.
- Do NOT emit `RootExecutionDenied` as a HostOp effect event — it is a startup
  terminal outcome with empty trace.
- A genuinely new fixed boundary (only) hard-stops to the Steward/Architect.

## Grounding anchors (landed on `origin/main`; re-ground before building)

- Startup seam: `crates/ken-host/src/abi_v1.rs` — `ProcessPostureV1(())` `:27`;
  `establish_process_posture_v1` `:29`; SIGPIPE extern `:23`, check `:35`;
  `struct ProcessContext{_posture,…}` `:148`; context constructor consuming
  `posture: Result<ProcessPostureV1,()>` `:441`, `posture.ok()?` `:443`,
  `_posture: posture` `:488`. Add the admission call + euid snapshot here.
- Terminal error: `TerminalErrorV1` enum in `crates/ken-host/src/effect_v1.rs`
  (variants `UnknownFamily … DriverFailure`); add unit `RootExecutionDenied`.
  `EffectObservationV1` `effect_v1.rs:905`.
- Exit mapper: `ProcessExitCode` `crates/ken-runtime/src/native_process_entrypoint.rs:91`;
  `process_exit_status` `:108`.
- Declaration surface: `program capabilities FS <authority>` header —
  `crates/ken-cli/src/lib.rs:428`; `ProgramCaps`/`MkProgramCaps` prelude
  `crates/ken-elaborator/src/prelude.rs:1466`. euid/`geteuid` currently
  **absent** from `ken-host` (confirmed `git grep` clean) — new rustix call.
- ADR homes: ADR-0017, ADR-0018; ADR-0019 (landed by PX13) §process-admission is
  the normative basis.
