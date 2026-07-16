# PX15 — FS capability path-root grammar: `./` (cwd) only (Runtime)

- **ID:** PX15 · **Owner:** Team Runtime · **Size:** S · **Risk:** Low-Medium
  (capability-root grammar addition on the settled root-handle machinery; must
  not perturb the ADR-0018 §4 relative canonicalization or PX6's twin-root
  differential).
- **Objective:** Let a `ProgramCaps` FS authority root be written **`./…`**
  (relative to **cwd at execution start**), not only an absolute path. The
  declared `./…` root resolves to a concrete ADR-0018 §2 **root handle**
  **once** at capability-table initialization, identically in both executors,
  and all downstream scope/symlink checks + canonical observations are unchanged
  relative to that resolved root. **`~/` (home) is NOT in this WP** — it is split
  to PX16 (account-database/NSS boundary, operator-gated).
- **Depends on:** PX5 (merged — the FS capability table + root-handle machinery;
  its `ken-host` init **already opens the startup cwd `.`** and builds the
  host-owned cap table from the declared `ProgramCaps`) **and** the shared
  Architect capability-model ruling (`evt_7k8n8rwj1xbh1`) **and** PX13 merged
  (ADR-0019 §path-root-resolution is the normative basis). **Gate:** G-Sec /
  native-effect lane.
- **Feeds:** ergonomic capability declaration; `~/` (PX16) enters the same
  root-spec enum later. Released **after PX13** (Runtime is one ring).

## Fixed inputs — DO NOT REOPEN (cite, do not relitigate)

Ruled by the Architect (`evt_7k8n8rwj1xbh1`); recorded in ADR-0019 (landed by
PX13). Binding:

- **`./` binds cwd at execution start, snapshotted ONCE at cap-table init.**
  Define a **typed root-spec variant** for execution-start cwd. Capture cwd
  **once** at capability-table initialization, open the root handle **then**, and
  retain **no** ambient cwd dependency afterward (cwd may move during the run —
  per-op re-resolution is **forbidden**: it would break ADR-0018 §4 relative
  canonicalization and PX6's twin-root differential).
- **Resolution is single-sourced below both executors.** ADR-0018 requires
  **one** capability check/representation beneath the interpreter and the native
  artifact — so the root-spec resolver lives there too, resolved **identically**
  in both.
- **Suffix resolution unchanged.** Any suffix under the `./` root resolves
  component-by-component using the **existing** `ScopeEscape` / `SymlinkDenied`
  policy — **no new escape surface**; the checks apply relative to the resolved
  root regardless of spelling.
- **Observations stay relative + spelling-free.** Canonical observations remain
  ADR-0018 §4 relative raw-byte paths that ignore absolute root names; a `./`
  root **never** exposes the cwd spelling into the observation.
- **`~/` and `$HOME` are OUT OF SCOPE.** `~/` waits behind PX16; **`$HOME` is
  rejected outright** as a forgeable authority source. Do not add either here.
- **Honesty boundary (ADR-0017).** Runtime-trusted + discriminator-tested, never
  kernel-proved. No new kernel rule / Ken postulate; no affine/linear types.

## Scope

**In scope:** the typed root-spec enum variant for `./` (execution-start cwd);
its parse from the `ProgramCaps` FS-authority-root declaration; binding it to the
cwd root handle **once** at cap-table init (leveraging PX5's existing
`open startup cwd .` seam); the single below-both-executors resolver; and the
discriminators proving snapshot-once + relative-canonical + no-new-escape.

**Out of scope:** `~/` / `EffectiveUserHome` and any NSS/`getpwuid`/`$HOME`
resolution (PX16); any change to `ScopeEscape`/`SymlinkDenied` policy; absolute-
root behavior (unchanged); PX13/PX14 mechanisms.

## Mandated deliverable outline — each section ends in a concrete choice

1. **Typed root-spec variant.** Introduce a root-spec type (e.g.
   `FsRootSpec::{ Absolute(path), ExecutionStartCwd(suffix) }`) below both
   executors; the declared `./suffix` parses to `ExecutionStartCwd(suffix)`. Do
   **not** represent it as a post-resolution string — it is resolved to a handle
   at init.
2. **Bind once at cap-table init.** At the single cap-table initialization
   (PX5's `ken-host` init that already opens startup cwd `.`), resolve
   `ExecutionStartCwd(suffix)` to a concrete root handle: open the cwd handle
   once, then resolve `suffix` component-by-component beneath it under the
   existing `ScopeEscape`/`SymlinkDenied` policy. Store the resolved handle in
   the cap table; keep **no** ambient cwd reference. The interpreter path binds
   identically at its equivalent init.
3. **No per-op re-resolution.** Every FS op uses the pre-resolved handle from the
   cap table; assert (structurally) there is no cwd lookup on the op path.
4. **Observation invariance.** Confirm canonical deltas/traces remain ADR-0018 §4
   relative and never carry the cwd spelling; PX6's twin-root differential still
   holds for a `./`-rooted scenario.
5. **Honesty.** Disclosure tested/target-validated; no new unsafe; trusted-
   surface delta note (the one-time cwd open already exists from PX5 — confirm no
   new obligation).

## Acceptance criteria (testable)

- **AC1 — `./` resolves + runs, interp == native.** A `ProgramCaps` with a
  `./data` FS root runs an FS op through both executors with identical
  `EffectObservationV1` on twin roots; the op reaches the intended node beneath
  cwd-at-start.
- **AC2 — snapshot-once (non-degenerate).** If cwd is changed **after** cap-table
  init, the `./` root still resolves to the **original** cwd handle (the op hits
  the start-cwd node, not the moved one) — a per-op re-resolver would hit the
  moved cwd and fail this. Structural: the op path takes the stored handle, never
  a fresh cwd lookup.
- **AC3 — scope/symlink unchanged.** A suffix that escapes the resolved `./` root
  (`../…`) → `ScopeEscape`; a symlink out → `SymlinkDenied` — exactly as for an
  absolute root. No new escape surface.
- **AC4 — spelling-free observation.** The canonical delta/trace for a `./`-rooted
  op is byte-identical to the same op under an absolute root pointing at the same
  directory (relative canonicalization ignores the root name); PX6 twin-root
  differential passes.
- **AC5 — `~/`/`$HOME` absent.** No `~`, `$HOME`, `getpwuid`, or NSS resolution
  anywhere in this WP (`git grep` clean) — that is PX16.
- **AC6 — confined + honest, CI-green.** No new unsafe; disclosure
  tested/validated; no kernel/spec/conformance movement. **No-regression = green
  in CI**, never a local `--workspace` run.

## Do-not-reopen guards

- Do NOT re-resolve `./` per op; bind once at cap-table init to a handle.
- Do NOT add `~/`, `$HOME`, `getpwuid`, or any NSS path — PX16 owns that, and
  `$HOME` is rejected outright.
- Do NOT change `ScopeEscape`/`SymlinkDenied` or the ADR-0018 §4 relative
  canonicalization.
- Do NOT let the cwd spelling leak into any canonical observation.

## Grounding anchors (landed on `origin/main`; re-ground before building)

- Cap-table init that already opens startup cwd `.` and builds the table from
  declared `ProgramCaps`: PX5's `ken-host` `abi_v1` init (see
  `PX5-native-effect-lowering.md` §0.5; `crates/ken-host/src/abi_v1.rs` init path
  ~`:441`–`:488`).
- Capability/scope types: `crates/ken-host/src/capability.rs` (`Cap`, `FsScope`,
  `SymlinkPolicy`); elaborator-facing `crates/ken-elaborator/src/capabilities.rs`
  (`FsScope`, `RightSet`, `AUTH_*`).
- Declaration surface: `program capabilities FS <authority>` —
  `crates/ken-cli/src/lib.rs:428`.
- Canonical observation (relative, spelling-free): `FsDeltaV1` /
  `FsNodeObservationV1` `crates/ken-host/src/effect_v1.rs:858`–`905`; PX6
  twin-root differential `crates/ken-verify/src/scenario.rs`.
- ADR homes: ADR-0018 §2 (root handle) + §4 (relative canonicalization);
  ADR-0019 §path-root-resolution (landed by PX13) is the normative basis.
