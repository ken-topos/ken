# PX16 — Account-database (NSS) resolver boundary + `~/` home-root tail (Runtime)

- **ID:** PX16 · **Owner:** Team Runtime · **Size:** M · **Risk:** Medium
  (introduces the **first libc/NSS trusted-surface** in the runtime — a confined
  `unsafe` boundary crossing the operator-settled `rustix`/`linux_raw`-only seam;
  must not perturb PX15's `FsRootSpec` resolution, ADR-0018 §4 relative
  canonicalization, or PX6's twin-root differential).
- **Objective:** Admit the **`~/…`** FS-authority-root spelling: it resolves the
  **executing euid's** home directory via `getpwuid_r(geteuid())` — an
  **account-database (NSS) policy lookup, not a Linux syscall** — then resolves
  the suffix beneath that home exactly like any other root. `~/…` enters the
  **same** `FsRootSpec` enum PX15 landed, as a new `EffectiveUserHome(suffix)`
  variant, resolved **once** at cap-table init. This is the **final** WP of the
  native capability-model campaign (PX13→PX14→PX15→PX16).
- **Depends on:** PX15 (merged `e4475143` — the `FsRootSpec` enum +
  `resolve_fs_root_spec_v1` + one-resolver-below-both-executors machinery this
  extends) **and** PX14 (merged — the immutable `EffectiveUidSnapshotV1` this
  **consumes**, no second `geteuid`) **and** the Architect ruling
  (`evt_1hxnmejwcvz1d` → **ADR-0020**). **Gate:** G-Sec / native-effect lane.
- **Feeds:** completes ergonomic capability declaration (`./`, `~/`, absolute all
  land). **Gate closer:** with PX16 merged the native cap-model campaign is done.

## Fixed inputs — DO NOT REOPEN (cite ADR-0020 / `evt_1hxnmejwcvz1d`; the
## operator sanction is settled — do not re-ask)

- **Operator sanction (Pat 2026-07-16):** the libc/NSS dependency IS approved for
  this bounded `~/`-resolution purpose ("`~/` is functionality tool writers will
  want; libc/NSS is the right way"). `rustix` stays the boundary for
  euid/process; **do not re-ask** this — it is a settled input.
- **ADR home = new ADR-0020** (account-database home-root resolution). ADR-0019
  stays the semantic owner of `EffectiveUserHome`; PX16 adds a **narrow xref**,
  and does **not** amend ADR-0019.
- **Dependency:** a **direct, Linux-target, exact-pin** `libc = "=0.2.186"`,
  `default-features = false`. Do **NOT** enable `rustix`'s libc backend, add a
  higher-level NSS crate, or hand-write C. Record the delta in
  `docs/ops/dependency-deltas.md` (or the established dependency-delta doc).
- **Confined boundary:** one **private** `ken-host::account_db_v1` module with an
  **inner** `#![allow(unsafe_code)]` (the crate stays `#![deny(unsafe_code)]`).
  A **safe, owned facade**: no `libc` type crosses the module boundary; callers
  see only owned Rust (`Vec<u8>`/`PathBuf`-shaped) values. Consumes **PX14's**
  `EffectiveUidSnapshotV1` — **no second `geteuid`**, no CLI/env UID.
- **Algorithm (`getpwuid_r` only, for `EffectiveUserHome`):** buffer starts
  **1 KiB**, **doubles on `ERANGE`**, hard **cap 1 MiB**; validate the result
  pointer is non-NULL, `pw_uid` matches the requested euid, `pw_dir` lies inside
  the buffer, is NUL-terminated, non-empty, and **absolute**; bounded scan (no
  unbounded `CStr::from_ptr`); **copy the bytes out before** the buffer drops;
  open the home handle **once**, then resolve the suffix component-by-component
  no-follow under the **existing** `ScopeEscape`/`SymlinkDenied` policy.
- **Failure semantics:** a new `HomeRootResolutionFailureV1` enum (buffer-cap
  exceeded / no-entry / `pw_dir` invalid / NSS error) → a new
  `TerminalErrorV1::HomeRootResolutionFailed`, **exact mapping**, delivered as a
  **startup-terminal** observation via **PX14's** no-context startup writer
  (`write_startup_terminal_observation_v1`) — **NO wall-clock bound** (NSS may
  block; do not add a timeout).
- **Verify seam (Architect carrier ruling `evt_302e43e52x7j9`):** a private
  `AccountHomeLookupV1` trait/seam with a **production libc impl** and a
  **scripted in-process test impl**. The script drives the real private
  init/resolution helper; it never crosses `exec`. PX6's twin-root differential
  stays **lane-local after resolution**, and a separate linked-artifact
  integration exercises the real child libc/NSS lookup. **No
  env/CLI/ProcessInput/artifact/observation home injection** exists.
- **`$HOME` is REJECTED outright** (forgeable). Do not read it, anywhere.
- **Honesty boundary (ADR-0017):** home is **NSS policy, not a kernel fact** —
  runtime-trusted + discriminator-tested, never kernel-proved. State the new
  libc/NSS trusted-surface delta honestly. No affine/linear types.

## Scope

**In scope:** the `EffectiveUserHome(suffix)` variant on the landed `FsRootSpec`
enum (`crates/ken-host/src/capability.rs`); its parse from the `~/…`
FS-authority-root declaration; the confined `ken-host::account_db_v1` boundary
(`getpwuid_r`, safe owned facade, `AccountHomeLookupV1` seam); wiring the
`EffectiveUserHome` arm into `resolve_fs_root_spec_v1` (consuming PX14's euid
snapshot, binding the home handle once at cap-table init); the
`HomeRootResolutionFailureV1` + `TerminalErrorV1::HomeRootResolutionFailed`
failure path via PX14's startup writer; the `libc` dependency delta; and the
discriminators below.

**Out of scope:** any change to `ScopeEscape`/`SymlinkDenied`, the ADR-0018 §4
relative canonicalization, or PX15's `./`/absolute behavior; `$HOME` (rejected);
a second `geteuid` (reuse PX14's snapshot); real/saved/fs-uid or
caps(7)/userns/securebits (PX14 deferred those and PX16 does not revisit them);
any kernel/spec/conformance change.

## Mandated deliverable outline — each section ends in a concrete choice

1. **Enum variant.** Add `FsRootSpec::EffectiveUserHome(Vec<u8>)` next to
   `Absolute`/`ExecutionStartCwd` (`capability.rs:14`), and parse `~/suffix` to
   it at the same declaration ingress that produces `ExecutionStartCwd`
   (`abi_v1.rs:497` wire tag + the elaborator/CLI header path). Do **not**
   represent the home path as a post-resolution string.
2. **Confined `account_db_v1` boundary.** New private module, inner
   `#![allow(unsafe_code)]`, exact-pinned `libc`. One safe fn
   `resolve_effective_user_home(uid: EffectiveUidSnapshotV1) -> Result<Vec<u8>,
   HomeRootResolutionFailureV1>` implementing the `getpwuid_r`
   grow-on-ERANGE/cap/validate/copy-out algorithm above. No `libc` type in the
   signature. Route it behind the `AccountHomeLookupV1` seam so tests inject a
   scripted impl.
3. **Wire into the shared resolver.** In `resolve_fs_root_spec_v1`
   (`lib.rs:480`), add the `EffectiveUserHome(suffix)` arm: pull PX14's euid
   snapshot (already available at init — **no** new `geteuid`), call the seam,
   open the home handle once, resolve `suffix` component-by-component no-follow,
   store the resolved `FsScope.root`. Both executor lanes go through this one
   arm; dispatch keeps taking only `scope.root`.
4. **Failure path.** Define `HomeRootResolutionFailureV1` + map each variant to
   `TerminalErrorV1::HomeRootResolutionFailed`; emit it as a startup-terminal via
   PX14's `write_startup_terminal_observation_v1` (no live `ProcessContext`, no
   wall-clock bound).
5. **Dependency + honesty.** Add the exact-pinned `libc` dep (Linux target,
   default-features off); update the dependency-delta doc; write the ADR-0017
   honesty note (home = NSS policy, first libc/NSS trusted surface, confined
   `unsafe`, tested-not-proved).

## Acceptance criteria (testable)

- **AC1 — three-face home-root evidence (carrier-ruling amendment).** (1) A
  scripted in-process `AccountHomeLookupV1` drives the real private
  init/resolution helper with two UID/home roots and proves isolated,
  exactly-once lookup/open, suffix resolution, mint-after-resolution, and
  relative spelling. (2) PX6 retains its real post-resolution
  interpreter/native twin-root equality over lane-local handles; it makes no
  claim about equal NSS records. (3) a real checked `~/` linked artifact uses
  the child process's production euid + `getpwuid_r`, accepting only successful
  startup or exact `HomeRootResolutionFailed(NoAccountRecord)` with empty
  stdout/stderr/trace/delta. No scripted value crosses `exec`.
- **AC2 — snapshot-once + euid-bound.** The home resolves from **PX14's** euid
  snapshot (structural: no second `geteuid` on the path — `git grep` shows the
  sole `geteuid` remains PX14's); a cwd/uid change after init does not re-resolve.
- **AC3 — scope/symlink unchanged.** A `~/…` suffix that escapes (`../…`) →
  `ScopeEscape`; a symlink out → `SymlinkDenied`, exactly as absolute/`./`.
- **AC4 — spelling-free observation.** The canonical delta/trace for a
  `~/`-rooted op is byte-identical to the same op under an absolute root at the
  same directory; the home spelling never enters an observation. PX6 twin-root
  passes.
- **AC5 — failure is a clean startup-terminal.** The same private injectable
  initialization helper drives each `HomeRootResolutionFailureV1` cause
  (buffer-cap exceeded, no account record, invalid `pw_dir`, NSS error) through
  the real pre-context writer and shared exit mapper to
  `TerminalErrorV1::HomeRootResolutionFailed`, with empty
  trace/delta/leaves. The separate linked integration proves the produced child
  reaches the production boundary. No wall-clock bound exists on the path.
- **AC6 — `$HOME` absent; confined unsafe.** `git grep` clean of `$HOME`/`env`
  home reads; the **only new** `#![allow(unsafe_code)]` introduced by PX16 is
  inside `account_db_v1`, while the crate stays `#![deny]` and the pre-existing
  audited `abi_v1` unsafe boundary remains separate and unchanged. No `libc`
  type crosses the facade. `libc` is an exact-pinned, default-features-off,
  Linux-target dep; dependency-delta doc updated. **No-regression = green in
  CI**, never a local `--workspace` run.

## Do-not-reopen guards

- Do NOT add a second `geteuid` — consume PX14's `EffectiveUidSnapshotV1`.
- Do NOT read `$HOME`/env, add a wall-clock timeout, enable rustix's libc
  backend, add an NSS crate, or hand-write C.
- Do NOT let a `libc` type escape `account_db_v1`, add any other PX16 unsafe
  allowance, or relax the crate's `#![deny(unsafe_code)]`. The inherited
  audited `abi_v1` unsafe boundary remains separate and unchanged.
- Do NOT change `ScopeEscape`/`SymlinkDenied`, ADR-0018 §4 canonicalization, or
  PX15's `./`/absolute behavior; do NOT let the home spelling leak into an
  observation.

## Grounding anchors (landed on `origin/main @ e4475143`; re-ground before building)

- `FsRootSpec` enum (extend): `crates/ken-host/src/capability.rs:14–30`
  (`Absolute`/`ExecutionStartCwd`; wire tag parse `abi_v1.rs:497`).
- Shared resolver (add the arm): `resolve_fs_root_spec_v1`
  `crates/ken-host/src/lib.rs:480`.
- PX14 euid snapshot (consume): `admit_root_execution` / `EffectiveUidSnapshotV1`
  `crates/ken-host/src/abi_v1.rs:50–70`.
- No-context startup terminal writer (ride): `write_startup_terminal_observation_v1`
  `crates/ken-host/src/abi_v1.rs:634`; `PostureErrorV1` `abi_v1.rs:125`;
  `TerminalErrorV1` in `ken-runtime`.
- Canonical observation (relative, spelling-free): `FsDeltaV1`/`FsNodeObservationV1`
  `crates/ken-host/src/effect_v1.rs`; PX6 twin-root `crates/ken-verify/src/scenario.rs`.
- Declaration surface: `program capabilities FS <authority>` `crates/ken-cli/src/lib.rs`.
- ADRs: **ADR-0020** (this WP's normative home), ADR-0019 (`EffectiveUserHome`
  semantic owner, narrow xref only), ADR-0018 §2/§4, ADR-0017 (honesty).

## Diff scope / review route

Touches `crates/**` + `Cargo.toml`/lock (the `libc` dep) + `docs/` — **no**
`spec/`+`conformance/` → **Architect-only §14** (CV not in route). One branch,
one Decision. The `Cargo.lock` movement is expected (the new `libc` dep) — flag
it in the handoff so the diff-scope note is accurate.
