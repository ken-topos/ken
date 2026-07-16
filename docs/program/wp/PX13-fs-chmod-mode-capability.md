# PX13 — FS `chmod` mode capability (Runtime)

- **ID:** PX13 · **Owner:** Team Runtime · **Size:** M · **Risk:** Medium
  (capability-model growth: a new right-bit + a new host op + a canonical
  observation-field extension that the PX6 differential closure must move with).
- **Objective:** Add one FS **mode-mutation** operation — `chmod` — as a
  versioned `HostOpV1` catalog extension gated behind a **new, distinct**
  capability right, executed by `fchmod` on the already-authorized handle, and
  make the resulting mode change **observable** in the canonical filesystem
  delta (mode only — never owner). Ship the durable design record as the **new
  capability-evolution / process-admission ADR** (this WP's lead deliverable).
- **Depends on:** PX5 (merged — the sealed `HostOpV1` catalog, shared dispatch
  core, live-capability route, `HostEffectAbiV1` manifest) **and** PX6 (merged —
  the interp/native differential closure whose twin-root delta this WP extends)
  **and** the shared Architect capability-model ruling (**delivered
  `evt_7k8n8rwj1xbh1`, thread `thr_szhcns1f2mpe`** — the controlling design). All
  three met → **ready**. **Gate:** G-Sec / native-effect lane.
- **Feeds:** PX14 + PX15(`./`) — both cite the ADR this WP lands. Runtime is a
  single ring; PX13 is released **first** so the shared ADR is on `main` before
  the siblings build.

## Fixed inputs — DO NOT REOPEN (cite, do not relitigate)

The component + security design is **ruled** by the Architect
(`evt_7k8n8rwj1xbh1`) on top of the settled ADR-0017 (authority/confinement
honesty boundary) and ADR-0018 (native-effect execution / ABI / differential
contract). This frame transcribes that ruling into deliverables + ACs. Binding:

- **`chmod` ONLY.** `chown`/`chgrp` are **REJECTED** from PX13 — they need a
  principal-ID model, user-namespace semantics, and a privilege story `chmod`
  does not; shipping them now would smuggle host numeric owner identities into a
  purportedly portable capability surface. If ever admitted they need a
  **separate `CHANGE_OWNER` right** (not `CHANGE_MODE`, not `WRITE`) and a
  **separately-versioned** observation contract — a future WP, not this one.
- **A distinct right.** Mode mutation is gated by a **new**
  `RightSet::CHANGE_MODE` bit + a new `FsCapabilityOperation::ChangeMode`.
  Content `WRITE` does **not** imply it; read-only `METADATA` does **not** imply
  it. `AUTH_FULL` (which is `RightSet::ALL`) includes it; `AUTH_PARTIAL`
  (`READ∪ENUMERATE∪METADATA`) does **not**. Attenuation intersects it exactly
  like every other bit. *Rationale (do not relitigate): changing bytes and
  changing who may later access those bytes are different authority.*
- **`fchmod` on the authorized handle — no second lookup.** The host op uses the
  existing component-by-component **no-follow** resolution to obtain the
  already-authorized handle, then calls `fchmod` **on that handle**. A path-based
  second lookup after authorization is **forbidden** (TOCTOU / symlink-swap
  surface). Accept only `mode & !0o7777 == 0`; **file-type bits are not input**.
- **Observe mode, NOT owner.** Extend the canonical node observation with
  `mode: Option<u16>`, defined as `st_mode & 0o7777`: `Some` for regular files
  and directories, `None` for symlink / other unsupported node kinds. This
  covers permission + setuid/setgid/sticky bits and **excludes** the file-type
  bits already represented by `kind`. Do **NOT** add uid/gid, timestamps, ACLs,
  xattrs, inode/device, or absolute-root identity — numeric owner IDs would be
  host-namespace leakage with no principal model behind them.
- **The whole observation closure moves together.** The generated
  catalog/registry/observer/consumer inventory, the `HostEffectAbiV1` manifest
  hash, the wire layout, and **PX6's twin-root differential delta** must all move
  in lockstep. Append **one** explicit FS operation ID (`FsChangeMode`); fail
  closed on old inventory/hash. Existing op IDs + meanings stay fixed.
- **Honesty boundary (ADR-0017).** Runtime-trusted + discriminator-tested,
  **never kernel-proved**. Zero kernel rule, zero new Ken postulate, no
  confinement proof. **NO affine/linear types.**

## Lead deliverable — the new ADR (author FIRST, in this branch)

Author **`docs/adr/0019-capability-evolution-and-process-admission.md`**
(confirm `0019` is unclaimed against `docs/adr/` at branch time; take the next
free number if not). It is the durable home for the shared PX13/PX14/PX15 ruling
(an in-thread ruling is not a durable deliverable). Content, transcribed from
`evt_7k8n8rwj1xbh1` — **do not editorialize or re-derive**:

- **Purpose + non-goals.** A capability-evolution / process-admission contract.
  ADR-0017 remains the authority/confinement honesty boundary; ADR-0018 remains
  the native-effect execution / ABI / differential-observation contract. This
  ADR **names the extension points** in both (operation rights + root resolution
  from ADR-0017; catalog, observation, and startup posture from ADR-0018) rather
  than growing either into a grab-bag. All claims runtime-trusted +
  discriminator-tested, never kernel-proved.
- **§ Operation-right evolution** (PX13's normative basis): a new operation that
  changes access-control metadata takes a **distinct right**, never a reuse of a
  content or metadata-read right; observation fields are added only where the Ken
  surface already exposes the corresponding mutation, and never leak a host
  namespace (numeric owner IDs) absent a principal model.
- **§ Process-admission posture** (PX14's basis — state normatively, even though
  PX14 implements it): effective-root admission is a **startup declaration
  check**, not a capability token; predicate `geteuid()==0`; header-declaration
  surface; one shared admission function before `ProcessContext`; the
  `RootExecutionDenied` startup-terminal outcome.
- **§ Path-root resolution** (PX15's basis): `./` binds cwd at execution start,
  snapshotted once at capability-table init; `~/` requires an account-database
  (NSS) boundary and is **out of scope** until that lands; `$HOME` is rejected as
  a forgeable authority source.

The ADR is reviewed on the PX13 §14 Decision (Architect soundness; and because
this branch touches **no** `spec/`+`conformance/` paths, **CV is not in route** —
confirm the diff scope). Because the ADR is a `docs/` change riding with crate
changes, this is **one branch, one Decision** (COORDINATION §14(4)).

## Scope

**In scope (PX13 owns):** the new ADR; `RightSet::CHANGE_MODE`;
`FsCapabilityOperation::ChangeMode` + its required-right mapping; the
`HostOpV1::FsChangeMode` catalog entry (new stable ID) + its typed
request/reply + per-op availability; the shared-dispatch-core arm that
authorizes + `fchmod`s on the resolved handle; the `mode: Option<u16>` extension
to `FsNodeObservationV1` and its population in both the interp and native
observation producers; the `HostEffectAbiV1` manifest-closure move; and the PX6
`ken-verify` twin-root delta extension for mode.

**Out of scope:** `chown`/`chgrp` and any owner/uid/gid observation (future
`CHANGE_OWNER` WP); any new kernel rule / Ken postulate; any spec/conformance
change; widening the native-tested op set beyond adding `FsChangeMode`; PX14 /
PX15 mechanisms (separate WPs, though they cite this ADR).

## Mandated deliverable outline — each section ends in a concrete choice

1. **New ADR** (lead deliverable, above) — authored first, from
   `evt_7k8n8rwj1xbh1`, no re-derivation.
2. **`RightSet::CHANGE_MODE` (`crates/ken-host/src/capability.rs:27`).** Add
   `pub const CHANGE_MODE: Self = Self(1 << 6);` and widen
   `ALL` from `(1<<6)-1` to `(1<<7)-1` so `AUTH_FULL` (`rights_for_authority`,
   `:139`) carries it; leave `AUTH_PARTIAL` (`READ∪ENUMERATE∪METADATA`) untouched
   so it does **not**. `union`/`intersect`/`contains` are bit-generic — no change.
3. **`FsCapabilityOperation::ChangeMode` (`capability.rs:190`).** Add the variant;
   in `required_right()` (`:203`) map `Self::ChangeMode => RightSet::CHANGE_MODE`
   (do **not** fold it into the `Write | Append` arm). Set `resolves_parent`
   (`:215`) to the same discipline as the in-place file ops (operates on the
   target node, not its parent).
4. **`HostOpV1::FsChangeMode` catalog entry (`effect_v1.rs:16`).** Add
   `FsChangeMode = 0x030A` (next free FS ID after `FsRename = 0x0309`); extend the
   required-right/authority map (`effect_v1.rs:457`) with
   `HostOpV1::FsChangeMode => Some((FsCapabilityOperation::ChangeMode, AUTH_FULL))`.
   Add the typed request `{capability, path, mode:u16}` → `Result FileError Unit`
   in the ADR-0018 §3 fixed-layout style (explicit width; `mode` validated
   `& !0o7777 == 0` before dispatch). Availability = `NativeTested` (this is the
   op PX13 lights up); the 9 prior deferred lanes are untouched.
5. **Shared-dispatch host leaf.** In the one shared dispatch core
   (`dispatch_host_op_v1`, `effect_v1.rs:449`), the `FsChangeMode` arm: resolve
   the capability (generational token → host table), run the SAME
   rights/authority/scope/symlink checks as every FS op, obtain the authorized
   handle via the existing component-by-component **no-follow** resolution, then
   `fchmod(handle, mode)` — **no path re-open**. Wrong slot/generation/kind/shape
   → `MalformedCapability` before any host action; right-not-held (no
   `CHANGE_MODE`) → the exact `RightNotHeld` denial before the leaf. One leaf,
   both executors — no second op-switch, no second cap-check.
6. **`mode: Option<u16>` observation (`FsNodeObservationV1`, `effect_v1.rs:858`).**
   Add the field; populate it as `st_mode & 0o7777` for regular-file/dir nodes,
   `None` for symlink/other, in **both** producers (native FS-delta snapshotter
   and interp `run_io_effect_observation_v1`, `eval.rs:4467`). A `chmod` on an
   existing file surfaces as an `FsDeltaV1::Modified{before, after}` whose
   `after.mode != before.mode`. Do not touch `kind`/`file_bytes`/`symlink_target`
   semantics; do not add any other field.
7. **Manifest + differential closure.** Regenerate `HostEffectAbiV1` from the
   single catalog descriptor so the new op ID + request/reply layout + the
   widened node-observation schema are covered by its canonical hash; the
   producer↔registry↔observer↔per-op-consumer inventories close **bidirectionally**
   with the new op present. Extend PX6 `ken-verify`'s twin-root differential so a
   `chmod` scenario runs both lanes and exact-compares the mode-bearing delta;
   old inventory/hash fails closed.
8. **Explicit availability + honesty.** Source discloses tested/target-validated,
   never proved; enumerate the trusted-surface delta (the `fchmod` call + its
   handle-lifetime obligation). No new unsafe beyond the existing audited
   `abi_v1` boundary.

## Acceptance criteria (testable — one per ruling control)

- **AC1 — ADR landed + faithful.** `docs/adr/0019-*.md` exists on the branch,
  transcribes `evt_7k8n8rwj1xbh1` (operation-right evolution, process-admission,
  path-root sections), and the diff scope pulls **no** `spec/`+`conformance/`
  (Architect-only review; CV not in route). One branch, one Decision.
- **AC2 — `chmod` executes natively, matches interp observably.** A Ken entry
  that `chmod`s an authorized file yields the SAME `EffectObservationV1` (mode in
  the FS delta, ordered trace, exit) through the native artifact and the
  interpreter on identical ProcessInput/ProgramCaps. (Interp is the oracle.)
- **AC3 — distinct-right discrimination (non-degenerate pair).**
  `WRITE`-without-`CHANGE_MODE` **denies** `chmod` before the host leaf
  (`RightNotHeld`), **while** `CHANGE_MODE`-without-`WRITE` **succeeds** at
  `chmod` on the **same** file — so a right-map that conflated the two inverts
  both and fails. Plus: attenuation can **drop** `CHANGE_MODE` from a child cap.
- **AC4 — handle-bound, no re-lookup.** A symlink / path-swap mutant interposed
  between authorization and the call **cannot** redirect the `fchmod` — it hits
  the already-authorized handle (structural: assert the leaf takes the handle,
  never a path). Wrong slot/generation/kind → `MalformedCapability` before any
  action; `mode & !0o7777 != 0` rejected before dispatch.
- **AC5 — mode observed, owner NOT.** A mode-only mutation changes the canonical
  delta (`after.mode != before.mode`); a uid/gid drift with identical mode bits
  produces **no** delta difference (owner is not observed). `FsNodeObservationV1`
  gains exactly `mode`, nothing else.
- **AC6 — closure moves together / fails closed.** `HostEffectAbiV1` covers the
  new op + widened observation; producer-only / registry-only / observer-only /
  one-field-layout mutation each fail the build closed; the PX6 twin-root
  differential exact-compares the mode delta across lanes. `chown`/`chgrp` are
  **absent** from every crate (`git grep` clean).
- **AC7 — confined + honest, CI-green.** No new unsafe beyond `abi_v1`; interp
  keeps `forbid(unsafe_code)`; disclosure tested/validated, not proved; no
  kernel/spec/conformance movement. **No-regression = green in CI** (full locked
  workspace), **never** a local `--workspace` run (COORDINATION §12).

## Do-not-reopen guards

- Do NOT reuse `WRITE`/`METADATA`/`CREATE` for mode mutation — the ruling
  mandates a **distinct** `CHANGE_MODE`. A "simpler" reuse is REJECTED.
- Do NOT add `chown`/`chgrp`, uid/gid observation, or any owner field. That is a
  future `CHANGE_OWNER` WP with its own right + versioned observation.
- Do NOT do a path-based second lookup for `chmod`; `fchmod` the authorized
  handle only.
- Do NOT add any node-observation field beyond `mode: Option<u16>`; do NOT
  include file-type bits (that is `kind`).
- A genuinely new fixed boundary (only) hard-stops and routes to the
  Steward/Architect — do not resolve it in-frame.

## Grounding anchors (landed on `origin/main`; re-ground before building)

- Rights: `crates/ken-host/src/capability.rs:25` (`RightSet(u8)`; `WRITE=1<<1`
  `:30`, `METADATA=1<<5` `:34`, `ALL=(1<<6)-1` `:35`); `rights_for_authority`
  `:139` (`AUTH_FULL=ALL`, `AUTH_PARTIAL=READ∪ENUMERATE∪METADATA`).
- Operation: `FsCapabilityOperation` enum `capability.rs:190`; `required_right()`
  `:203`; `resolves_parent` `:215`.
- Catalog: `HostOpV1` enum `effect_v1.rs:16` (FS ops `0x0301`–`0x0309`; add
  `FsChangeMode=0x030A`); required-right map `effect_v1.rs:457`.
- Dispatch: `dispatch_host_op_v1` `effect_v1.rs:449`; interp producer
  `run_io_effect_observation_v1` `eval.rs:4467`; canonical identity
  `program_caps_fs_trace_identity_v1` `effect_v1.rs:676`.
- Observation: `FsNodeObservationV1` `effect_v1.rs:858` (add `mode`); `FsDeltaV1`
  `:866`; `EffectObservationV1.filesystem_delta` `:905`.
- Differential: PX6 `crates/ken-verify/src/scenario.rs` (twin-root run; extend
  for a `chmod` scenario). `chmod`/`fchmod`/`ChangeMode` currently **absent**
  from all crates (confirmed `git grep` clean) — this is new surface.
- ADR homes: `docs/adr/0017-scoped-capability-tcb-posture.md`,
  `docs/adr/0018-native-effect-execution-contract.md`; next free ADR `0019`.
