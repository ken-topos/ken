# PX7-R — Runtime resource-lifetime substrate: `ResourceTableV1` + `FsOpen`/use/`ResourceRelease` (Runtime lead)

- **ID:** PX7-R · **Owner:** Team Runtime · **Size:** L · **Risk:** High
  (introduces the **first held-across-steps resource handle** in the runtime — a
  real departure from the landed path-based/stateless FS model — plus a new
  generation-checked table, a new host-op family, ABI-hash movement, the
  interpreter **state-lifetime seam**, and the native + shared-dispatch
  differential for it (the first real **public** interpreter/native lifecycle
  equality is deferred to PX7-F — see AC6); must not perturb the landed
  `CapabilityTableV1`, PX16's `FsRootSpec` resolution, ADR-0018 §4 relative
  canonicalization, or PX6's twin-root differential).
- **Objective:** Land the **private runtime substrate** for opaque, dynamically
  acquired resource handles: a sibling `ResourceTableV1` with opaque
  `ResourceTokenV1 { slot, generation }`, generation-invalidate-before-close
  release, a lane-independent `ResourceTraceIdentityV1`, and three V1 host ops —
  `FsOpen` (capability-gated acquisition), one **real** non-release consumer
  (handle-metadata), and generic `ResourceRelease` — with structured
  `ReleaseFailed` errors, the invocation-scoped interpreter **state-lifetime
  seam**, and native + shared-dispatch differential closure (the first real
  **public** interpreter/native lifecycle equality is **PX7-F**, not this WP —
  see AC6). This is the **lead** WP of PX7; it lands a complete, stable, private
  boundary that
  PX7-F consumes.
- **Depends on:** the native cap-model campaign (PX13→PX16, all merged;
  `origin/main @ fa33fa55` — the `HostOpV1`/`CanonicalRequestV1`/ABI-hash
  inventory, `CapabilityTableV1` generation precedent, `IoErrorIdentityV1`,
  `FsScope`/`RootedHandle`, and the PX6 differential this extends) **and** the
  Architect PX7 ruling (`evt_1x3rcz9q7d8g7` → **ADR-0021**). **Gate:** G-Sec /
  native-effect lane, G-Ward-seam (produces the lifecycle-observation vocabulary
  Ward will monitor).
- **Feeds:** **PX7-F** (Foundation follow — `System.Resource`/`withResource`),
  which cannot start building until this substrate merges. Also feeds the Spec
  enclave's `ResourceLifetimeObligationV1` schema (the lifecycle-observation
  vocabulary this WP lands is the vocabulary the schema references).

## Fixed inputs — DO NOT REOPEN (cite ADR-0021 / `evt_1x3rcz9q7d8g7`; the
## operator strategy is settled — do not re-ask)

- **Operator strategy (Pat 2026-07-16), settled:** resources that need affine
  reasoning are served by doing the **affine enforcement in Rust** (Rust
  ownership = affine move), **lifting a reasonable interface to Ken**, and
  **discharging the exactly-once/no-leak obligation to Ward**. The guarantee is
  real but is **never a Ken affine claim and never a kernel `proved`**. Do not
  re-ask this.
- **Barred (Pat, verbatim):** *"Until CS research shows a proven path, Ken will
  not have affine types."* PX7-R introduces **no** Ken-level affine/linear type
  and no feature that quietly needs one. The affine discipline lives entirely in
  Rust ownership.
- **ADR home = new ADR-0021** (runtime-enforced resource lifetime + Ward
  delegation). It is authored and lands with this WP; implement against it, do
  not redesign it.
- **Representation is a SIBLING `ResourceTableV1`, not `CapabilityTableV1`.** Do
  **not** extend the capability table. Opaque `ResourceTokenV1 { slot,
  generation }`; neither field nor constructor Ken-visible. A live slot owns
  exactly one Rust resource and records ≥ generation, resource kind, the owned
  backend object, the attenuated rights/context inherited at acquisition, and a
  canonical trace identity distinct from the token.
- **Generation discipline:** on release, **move the owned object out and
  bump/invalidate the generation BEFORE invoking close**. The token is closed
  whether close succeeds or fails; **never retry a raw descriptor after a close
  error**. Slot reuse only at the bumped generation; on generation wrap, **retire
  the slot permanently** — never reissue an old identity. A stale token can never
  resolve a reused slot or a recycled fd.
- **Resolution outcomes are distinct + fail-visible:** retired generation →
  `Closed`; zero/out-of-range/never-minted encoding → `MalformedResource`. V1 is
  single-kind (`ResourceKindV1::FsHandle`), so a live wrong-kind state is
  unreachable — PX7-R defines **no** `ResourceKindMismatch` (deferred; see the
  do-not-reopen guard). The resolver checks validity/liveness/rights; the kind is
  established by construction.
- **Canonical identity = lane-independent `ResourceTraceIdentityV1`** minted from
  deterministic acquisition order (successful acquire event identity suffices).
  **Never** an fd, slot/generation, pointer, inode, or executor provenance.
- **Rust enforces; Ward checks (this WP lands the Rust half + the observation
  vocabulary).** Rust guarantees: one live slot owns one Rust resource; every use
  checks slot/generation/kind/stored-rights; user release consumes the live owner
  at most once; explicit finalization runs on controlled return/error/trap;
  table/context RAII is the last-resort leak backstop; stale use/release is
  `Closed`. **Do NOT use `Drop` alone for the normal contract** (`Drop` cannot
  report a close failure): controlled exits call an **explicit finalizer**, record
  every result, then `Drop` covers only catastrophic unwinding.
- **Close-failure boundary = ONE confined `try_close` seam (Architect ruling
  `evt_7jk02bmwg7qj2` — supersedes the original AC9).** A real OS close failure is
  unobservable through safe `OwnedFd` `Drop` (it discards the result); the only
  errno-reporting close is rustix's feature-gated `unsafe fn try_close(RawFd)`.
  So: the resource slot owns a **distinct, unique, non-cloneable
  `ResourceHandleV1(OwnedFd)`** — **NO `Clone`, NO `Arc`**, NOT PX16's
  `Handle(Arc<OwnedFd>)`; correctness never depends on
  `Arc::strong_count`/`try_unwrap`; existing rooted/path handles unchanged. Add
  **exactly one** private Linux-only module-local `#![allow(unsafe_code)]` (e.g.
  `ken-host::resource_close_v1`) whose safe facade consumes the unique owner via
  `IntoRawFd`, performs the **sole** `rustix::io::try_close` call, and returns the
  close result; the crate stays `#![deny(unsafe_code)]` elsewhere. Enable rustix's
  **exact-pinned `try_close` feature** — an **audited TCB delta** (Cargo-manifest
  assertion, dependency identity/feature registry, target-ABI hash, mutation/
  foreign-target controls all move), NOT a routine toggle. Safety invariant local
  + checkable: raw fd from consuming the sole owner; no alias/borrower/in-flight op
  survives; call once; fd invalid on every return; no retry — no `ManuallyDrop`,
  fabricated raw fd, or second owner. Release order: take the unique owner from the
  live slot → bump/retire generation + commit tombstone → invoke the facade →
  map **once** to success or `ReleaseFailed{resource_kind,identity,io}`; slot
  closed on both outcomes. This stays within the **settled rustix/linux_raw
  boundary** (no
  new dependency) — the TCB growth is ratified at the PX7-R §14, NOT re-asked of
  the operator.
- **Authority = `FsOpen` on the existing FS grant.** `FsOpen` checks the landed
  FS capability plus the rights required by its requested mode; the resulting slot
  stores only the attenuated subset. **No** `program capabilities Resource …`
  family, **no** new `RightSet` bit (Resource is a lifetime class, not ambient
  authority; release needs a live handle, not a new right).
- **V1 op set (exactly these three):** domain-specific `FsOpen`
  (capability-gated acquisition); **≥1 real non-release consumer** (a
  handle-metadata op is the natural minimum) so "use-after-close" is not a
  renamed double-close; generic `ResourceRelease`. **Do NOT** add a generic
  authority-free `ResourceAcquire`.
- **Structured errors:** `ReleaseFailed { resource_kind, identity, io:
  IoErrorIdentityV1 }` for OS close failure (no fd exposed). Close error leaves
  the handle closed, never retried. Multi-fault ordering is versioned: body-error
  + release-failure → preserve both; a versioned observation/wire discriminator
  carries the pairing (trap-primary + ordered cleanup-failure list is **PX7-F's**
  concern — PX7-R lands the discriminator and the single-fault release-failure
  path).
- **Substrate is PRIVATE.** The acquire/release effect protocol is private
  substrate, **not** the public Ken safety API. No `System.Resource`, no
  `withResource`, no Ken-surface honesty prose in this WP — those are PX7-F.
- **Honesty boundary (ADR-0017):** runtime-trusted + discriminator-tested, never
  kernel-proved, never Ken-affine. State the trust delta honestly in ADR-0021.

## Scope

**In scope:** ADR-0021 (authored below as fixed input — land as-is); the sibling
`ResourceTableV1` + opaque `ResourceTokenV1 { slot, generation }` in
`crates/ken-host/src/effect_v1.rs` (sibling to `CapabilityTableV1`); the
lane-independent `ResourceTraceIdentityV1`; the three V1 `HostOpV1` variants
(`FsOpen`, one handle-metadata consumer, `ResourceRelease`) with their
`CanonicalRequestV1` arms, wire request structs, dispatch arms, backend methods,
observations, and ABI size/offset hash entries following the PX13
`FsChangeModeRequestV1` template; capability attenuation at `FsOpen`;
generation-invalidate-before-close with explicit finalizer + RAII backstop;
structured `ReleaseFailed` + `Closed`/`MalformedResource` identities; the
versioned observation/wire discriminator; and the native + shared-dispatch
differential + the interpreter state-lifetime seam (the first real **public**
interpreter/native lifecycle equality is deferred to PX7-F — see AC6).

**Out of scope:** `System.Resource`, `withResource`, delayed-body/settlement
sequencing, optional early release, source-level honesty prose, the generated
`ResourceLifetimeObligationV1` T-obligation, and the end-to-end
success/error/trap/escape controls — **all PX7-F**. Read/write/seek ops (PX8).
Any change to `CapabilityTableV1`, PX16's `FsRootSpec` resolution, ADR-0018 §4
canonicalization, or PX15's `./`/`~/`/absolute behavior. Any kernel change. Any
Ken-level affine/linear type. `spec/`+`conformance/` are not touched by PX7-R
(the schema is Spec-owned and pinned separately) → Architect-only §14.

## Mandated deliverable outline — each section ends in a concrete choice

1. **ADR-0021.** Land `docs/adr/0021-resource-lifetime-and-ward-delegation.md`
   (authored with this frame) verbatim as the normative home. Do not redesign;
   refine only wording/xrefs if strictly needed.
2. **`ResourceTableV1` + `ResourceTokenV1`.** In `effect_v1.rs`, add a sibling
   table next to `CapabilityTableV1:348` (do **not** extend it). Opaque
   `ResourceTokenV1 { slot: u32, generation: u32 }` (private fields, no Ken
   constructor). A slot stores: generation, `ResourceKindV1` (V1 = a single
   `FsHandle` kind), the owned backend object as a **distinct, unique,
   non-cloneable `ResourceHandleV1(OwnedFd)`** (NO `Clone`, NO `Arc` — NOT PX16's
   `Handle(Arc<OwnedFd>)`), attenuated rights/context from acquisition, and the
   `ResourceTraceIdentityV1`. `resolve()` returns the live owner only on exact
   slot+generation match (plus liveness + attenuated-rights checks), else the
   exact distinct identity (`Closed` / `MalformedResource`). The single-variant
   `ResourceKindV1::FsHandle` is stored as a sealed expansion point, but the kind
   is established by construction — there is **no** wrong-kind resolver branch and
   **no** `ResourceKindMismatch` in PX7-R (deferred; see guard).
3. **`ResourceTraceIdentityV1`.** A lane-independent identity minted from the
   deterministic successful-acquire order (a monotone per-run acquire counter is
   the natural mint; the successful acquire event identity is sufficient).
   Explicitly derived from **no** fd/slot/generation/pointer/inode/executor
   provenance. This is the identity that appears in canonical observations and
   that Ward will pair; the opaque token is **not** it.
4. **`FsOpen` acquisition op (PX13 template).** New `HostOpV1::FsOpen` +
   `FsOpenRequestV1` wire struct + `CanonicalRequestV1::FsOpen` arm + dispatch
   arm + backend method + observation + ABI size/offset hash, mirroring
   `FsChangeModeRequestV1` (`abi_v1.rs:221`, dispatch `:936`). `FsOpen` checks
   the existing FS capability + the rights required by its requested mode, opens
   the backend handle, mints a slot at `generation` (start value per the
   `CapabilityTableV1` precedent) storing only the attenuated rights, and returns
   the opaque token. No new `RightSet` bit; no new capability family.
5. **Real non-release consumer (handle-metadata).** One genuine
   non-release op that resolves a live token and returns handle metadata (e.g. a
   `FsHandleStatV1`-style canonical metadata read). It exists so use-after-close
   is a real distinct control, not a renamed double-release. Full PX13-template
   wiring + observation + ABI hash.
6. **`ResourceRelease` op + generation discipline + confined close seam.** Generic
   `HostOpV1::ResourceRelease` resolves the live slot, **moves the unique owner
   out, bumps/invalidates the generation + commits the tombstone, THEN closes via
   the confined facade**. Add the one private Linux-only `resource_close_v1`
   module (`#![allow(unsafe_code)]`; crate stays `#![deny]` elsewhere): its safe
   fn consumes the `ResourceHandleV1(OwnedFd)` via `IntoRawFd` and performs the
   sole `rustix::io::try_close`, returning `Result<()>`. Enable rustix's
   exact-pinned `try_close` feature and move the Cargo-manifest assertion,
   dependency/feature registry, target-ABI hash, and mutation/foreign-target
   controls. Public release is non-idempotent (2nd call → `Closed`). Map the close
   result **once**: a close error → `ReleaseFailed { resource_kind, identity:
   ResourceTraceIdentityV1, io: IoErrorIdentityV1 }`, **never retried**; the handle
   is closed regardless. The **explicit finalizer** records the outcome; `Drop`
   covers only catastrophic unwinding (owners never extracted by explicit
   settlement). On generation wrap, retire the slot permanently.
7. **Versioned observation/wire discriminator.** Add the versioned
   observation/wire discriminator that lets release outcome (success vs.
   `ReleaseFailed`) and, in PX7-F, body/settlement pairing be represented without
   overwriting either fault. PX7-R lands the discriminator + the single-fault
   release-failure encoding and round-trips it exactly.
8. **Native + shared-dispatch differential + the interpreter state-lifetime seam
   (NOT a public interpreter lifecycle differential — that is PX7-F).** (a) A
   **real linked-native** acquire→use→release→stale-use path, plus shared
   semantic-dispatch + wire/ABI agreement across the reachable V1 set, produce
   **byte-identical** canonical observations; the `ResourceTraceIdentityV1`
   (acquisition-order) is what makes the lanes agree without leaking fd/slot
   provenance. (b) One `ResourceTableV1` is **invocation-scoped persistent state**
   for the real `run_io_effect_observation_v1` / `run_io_with_effect_recorder`
   producer, and the production dispatch helpers **borrow that same table** —
   remove the per-dispatch `ResourceTableV1::default()` at `eval.rs:4438` and
   `:4540` so PX7-F need not repair Runtime's state lifetime before it can express
   the bracket. (c) The test-local comparator is named `SharedSemanticBackend`
   (NOT `InterpreterSemanticBackend`): it is shared semantic-dispatch substrate
   evidence, **not** `ken-interp` evidence. The first real **public**
   interpreter/native lifecycle + negative-control equality is **PX7-F**, once the
   `System.Resource`/bracket constructors + IDs land.

## Acceptance criteria (testable)

- **AC1 — sibling table, opaque token.** `ResourceTableV1` is a distinct type
  from `CapabilityTableV1`; `git grep` shows `CapabilityTableV1` /
  `MalformedCapability` semantics unchanged. `ResourceTokenV1` fields and
  constructor are not Ken-visible (no Ken surface constructs one; PX7-F's bracket
  is the only mint path once it lands). No `RightSet` bit is spent (`RightSet(u8)`
  free-bit count unchanged); no `program capabilities Resource` grammar is added
  (`parser.rs` cap parse unchanged).
- **AC2 — generation-invalidate-before-close.** A structural test proves the
  release path bumps/invalidates the generation **before** the OS close call
  (order asserted, not merely that both happen). After release, the same token
  resolves `Closed`; a second release resolves `Closed` (non-idempotent); the
  raw descriptor is never retried on a close error. On a forced generation wrap
  the slot is retired and never reissued.
- **AC3 — distinct fail-visible identities (reachable V1 set only).** Each of
  these produces its exact distinct identity (assert the specific variant, never
  `is_err`): retired generation → `Closed`; zero/out-of-range/never-minted →
  `MalformedResource`; `FsOpen` under a grant lacking the requested rights → the
  existing capability/file denial identity (not a resource identity); OS close
  failure → `ReleaseFailed { resource_kind, identity, io: IoErrorIdentityV1 }`
  with **no** fd exposed. A stale token never resolves a reused slot or a recycled
  fd (stale-slot/recycled-fd separation asserted). `ResourceKindMismatch` is **out
  of scope for PX7-R** — single-kind V1 has no live wrong-kind producer; it is
  deferred to the first second-kind WP (see do-not-reopen guard).
- **AC4 — real consumer, real use-after-close.** The handle-metadata consumer
  resolves a live token and returns metadata on a live handle; the **same**
  consumer after release returns `Closed`. This control is distinct from the
  double-release control (i.e. use-after-close is not a renamed double-close).
- **AC5 — capability-gated acquisition + attenuation.** `FsOpen` under an FS
  grant that lacks the requested mode's rights fails with the existing
  capability/file identity (not a resource identity); a successful `FsOpen`
  stores only the attenuated subset, and a later handle op requiring a right
  outside that subset fails fail-visibly. Acquisition adds no new authority
  right.
- **AC6 — lane-independent identity + native/shared-dispatch agreement + the
  interpreter state-lifetime seam (NOT a public interpreter lifecycle
  differential — deferred to PX7-F; Architect ruling `evt_5f65tm0ymzwbh`).** This
  WP requires: (a) the `ResourceTraceIdentityV1` in canonical observations is the
  acquisition-order identity, `git grep`/structural-checked as derived from no
  fd/slot/generation/pointer/inode/executor value; (b) a **real linked-native**
  acquire→use→release→stale-use path; (c) shared semantic-dispatch and wire/ABI
  agreement across the reachable V1 set; and (d) **one invocation-scoped
  persistent `ResourceTableV1`** in the real interpreter producer
  (`run_io_effect_observation_v1` / `run_io_with_effect_recorder`), with the
  production dispatch helpers borrowing that same table and the per-dispatch
  `ResourceTableV1::default()` at `eval.rs:4438`/`:4540` removed. The test-local
  comparator is named `SharedSemanticBackend` (NOT `InterpreterSemanticBackend`) —
  shared-dispatch substrate evidence, not `ken-interp` evidence. PX7-R does
  **not** claim, and merging it must **not** be reported as delivering, a public
  interpreter↔native lifecycle differential; that first equality is **PX7-F**.
- **AC7 — explicit finalizer + confined close facade.** A structural check
  confirms the controlled-exit release path runs an explicit finalizer that
  consumes the unique owner via `IntoRawFd` and reaches the real
  `resource_close_v1` `try_close` facade **only after** generation invalidation
  (invalidate-before-close, asserted by order), records the outcome, and maps a
  close error to `ReleaseFailed`. `Drop` is only the catastrophic-unwinding
  backstop — the normal contract does not depend on `Drop`, and once the owner is
  consumed no `OwnedFd` remains to double-close.
- **AC8 — ABI/hash/inventory closed + wire round-trips.** The three new
  `HostOpV1` variants, their `CanonicalRequestV1` arms, wire request structs, and
  the versioned discriminator are in the generated ABI size/offset hash and the
  op inventory; every new wire form round-trips exactly (encode→decode identity),
  and the mutation gates cover them. **No-regression = green in CI**, never a
  local `--workspace` run.
- **AC9 — honesty + confined-unsafe delta (supersedes the original AC9;
  Architect ruling `evt_7jk02bmwg7qj2`).** ADR-0021 states the runtime-enforced /
  Ward-delegated / never-Ken-affine / never-kernel-proved boundary honestly and
  **names the bounded exception**: `git grep` shows the **only** new
  `#![allow(unsafe_code)]` introduced by PX7-R is inside `resource_close_v1`,
  while the crate stays `#![deny]` and the pre-existing audited `abi_v1` boundary
  remains separate and unchanged; all public dispatch/table APIs remain safe;
  rustix stays exact-pinned and target-identity-bound with only the `try_close`
  feature added (dependency/feature registry + target-ABI hash + mutation gates
  updated). PX7-R adds no affine/linear type and no kernel postulate.
- **AC10 — close-boundary discriminator net.** (1) A structural test proves the
  production release path reaches the exact real `try_close` facade only after
  invalidation. (2) A **real linked-artifact** success test exercises acquire →
  genuine handle use → release → stale use. (3) A private injected close-result
  seam MAY deterministically prove invalidation-before-call, error mapping,
  no-retry, and persistent `Closed` — but it is **labelled a caller-control
  test**, NOT evidence the OS produced a close error. The test suite does **not**
  manufacture `EBADF` by double-closing or otherwise violate the facade
  precondition; the production failure branch is real because the deployed facade
  returns the OS result. **Every** close-error-injecting test carries this
  caller-control / non-OS-error label explicitly — including
  `resource_release_invalidates_before_close_and_never_retries` (Architect §14
  `evt_5f65tm0ymzwbh`).
- **AC11 — unique non-cloneable owner (compile/structural guard).** A
  compile/structural guard proves the resource owner
  (`ResourceHandleV1(OwnedFd)`) is **non-cloneable** and **never crosses into the
  existing `Arc<OwnedFd>` representation**; correctness does not depend on
  `Arc::strong_count`/`try_unwrap`. The existing rooted/path `Handle(Arc<OwnedFd>)`
  is unchanged.

## Do-not-reopen guards

- Do NOT extend `CapabilityTableV1` or perturb `MalformedCapability`; the
  resource table is a **sibling**.
- Do NOT add a Ken-level affine/linear type, a kernel postulate/primitive, or a
  feature that quietly needs affinity. The affine discipline is Rust-only.
- Do NOT add a `program capabilities Resource` family or spend a `RightSet` bit;
  `FsOpen` carries the domain grant.
- Do NOT use `Drop` alone for the normal close contract; use an explicit
  finalizer that records the outcome.
- Do NOT add more than **one** new `#![allow(unsafe_code)]` (the confined
  `resource_close_v1`); do NOT relax the crate `#![deny]` elsewhere or touch the
  audited `abi_v1` boundary. Do NOT store the resource owner as `Arc<OwnedFd>` or
  make correctness depend on `Arc::strong_count`/`try_unwrap`.
- Do NOT manufacture `EBADF` (double-close) or use a mock/`sync_all` as the
  production close-error mechanism; injection tests are labelled caller-control.
- Do NOT re-ask the operator for the `try_close`/confined-unsafe delta — it is
  within the settled rustix/linux_raw boundary and ratified at the PX7-R §14.
- Do NOT key `ResourceTraceIdentityV1` on fd/slot/generation/pointer/inode/
  executor provenance; it is acquisition-order.
- Do NOT retry a raw descriptor after a close error; do NOT reissue a retired
  generation/slot; do NOT let a stale token resolve a reused slot or recycled fd.
- Do NOT define `ResourceErrorV1::ResourceKindMismatch`, its canonical wire form,
  an error-tag inventory entry, or a wrong-kind branch in `resolve_fs_handle` —
  single-kind V1 makes it vacuous/unreachable. Do NOT add a `#[cfg(test)]`
  reserved kind, test-only slot seeder, or fabricated malformed token to force a
  wrong-kind test (a test-only universe cannot evidence the production V1
  contract). Do NOT relabel "resource supplied to a non-resource op" as kind
  mismatch — that is malformed request routing. Keep `ResourceKindV1::FsHandle`
  and the observation/slot `resource_kind` field (sealed expansion point).
  **Deferred trigger:** the first WP that adds a genuinely different production
  resource kind atomically adds `ResourceKindMismatch { expected, actual }` + its
  versioned wire discriminator / schema / hash / inventory movement + a real mint
  for both kinds + a non-degenerate production-reaching pair (mint A apply B-only,
  mint B apply A-only, both returning the exact mismatch with expected/actual
  reversed, valid same-kind controls succeeding). PX8 read/write/seek over
  `FsHandle` does NOT trigger it; the trigger is a second real resource kind.
- Do NOT claim or report PX7-R as delivering a **public** interpreter↔native
  lifecycle differential (Architect ruling `evt_5f65tm0ymzwbh`): the real Ken
  `System.Resource`/bracket constructors are PX7-F, so the first public
  lifecycle + reachable-negative-control interp↔native equality is **PX7-F**.
  PX7-R delivers the runtime substrate + the invocation-scoped interpreter
  **state-lifetime seam** only. Do NOT relabel a direct `ken-host` dispatcher /
  `SharedSemanticBackend` fixture as `ken-interp` evidence, and do NOT add a Rust
  script API, fabricated `GlobalId` set, test-only constructors, or a private Ken
  ingress to force the public lifecycle (all rejected proxies). Do NOT recreate
  impossible negatives: `MalformedResource` is unreachable from the opaque public
  token surface (it stays an internal table/wire discriminator) and injected
  `ReleaseFailed` stays labelled caller-control — neither is a mandatory public
  interp↔native fixture.
- Do NOT build `System.Resource`, `withResource`, the T-obligation, or the
  end-to-end trap/escape controls here — those are **PX7-F**.
- Do NOT change PX16's `FsRootSpec` resolution, ADR-0018 §4 canonicalization, or
  PX15 `./`/`~/`/absolute behavior.

## Grounding anchors (landed on `origin/main @ fa33fa55`; re-ground before building)

- Generation-table precedent (sibling, do NOT extend): `CapabilityTokenV1`
  `crates/ken-host/src/effect_v1.rs:317`; `CapabilityTableV1`
  `crates/ken-host/src/effect_v1.rs:348` (mints `generation=1`, never bumps,
  mismatch → `MalformedCapability`).
- Host-op inventory (extend): `HostOpV1` enum
  `crates/ken-host/src/effect_v1.rs:16`; `CanonicalRequestV1`
  `crates/ken-host/src/effect_v1.rs:722`.
- PX13 op-add template (mirror for each new op): `FsChangeModeRequestV1`
  `crates/ken-host/src/abi_v1.rs:221`; dispatch arm `abi_v1.rs:936`;
  `CanonicalRequestV1::FsChangeMode` `abi_v1.rs:961`; ABI size/offset hash
  `abi_v1.rs:1043,1092`.
- I/O identity (reuse for `ReleaseFailed`): `IoErrorIdentityV1`
  `crates/ken-host/src/effect_v1.rs:793`.
- Rights + capability posture: `RightSet(u8)`
  `crates/ken-host/src/capability.rs:94` (do NOT spend bit 7); `FsScope`
  `crates/ken-host/src/capability.rs:167`; `RootedHandle`
  `crates/ken-host/src/lib.rs:297` (the RAII-closed handle model this WP holds
  across steps).
- Close boundary (the confined seam): existing cloneable
  `Handle(Arc<OwnedFd>)` `crates/ken-host/src/lib.rs:112` (do NOT reuse for the
  resource owner — the resource owner is a unique `ResourceHandleV1(OwnedFd)`);
  crate `#![deny(unsafe_code)]` `crates/ken-host/src/lib.rs:23`; rustix pin
  `crates/ken-host/Cargo.toml:15` (`features=["std","fs","process"]` — add
  `try_close`); rustix `try_close` = `rustix::io::try_close` (unsafe, feature-gated,
  raw fd invalid after call even on error).
- Cap declaration surface (unchanged): `parser.rs:269`
  (`crates/ken-elaborator/src/parser.rs`); `program capabilities FS <authority>`
  `crates/ken-cli/src/lib.rs`.
- Differential harness (extend for native + shared-dispatch only): PX6 twin-root
  `crates/ken-verify/src/scenario.rs`; canonical FS observations `FsDeltaV1`/
  `FsNodeObservationV1` `crates/ken-host/src/effect_v1.rs`. Interpreter
  state-lifetime seam: `run_io_effect_observation_v1` /
  `run_io_with_effect_recorder` + the per-dispatch `ResourceTableV1::default()` at
  `crates/ken-interp/src/eval.rs:4438`/`:4540` (make invocation-scoped). The
  **public** interpreter/native lifecycle differential is **PX7-F**, not here.
- ADRs: **ADR-0021** (this WP's normative home), ADR-0019 (capability
  evolution), ADR-0018 (native-effect contract), ADR-0017 (honesty), ADR-0006
  (Ward attestation complement).

## Diff scope / review route

Touches `crates/**` + `Cargo.toml`/lock (rustix `try_close` feature +
dependency/feature registry) + `docs/adr/0021-*` + `docs/program/wp/PX7-R-*` —
**no** `spec/`+`conformance/` (the `ResourceLifetimeObligationV1` schema is
Spec-owned and pinned separately for PX7-F) → **Architect-only §14** (CV not in
route). Because the confined `try_close` seam is a **trusted-base growth**, the
§14 is a soundness/TCB-delta review — flag the `Cargo.toml`/lock movement and the
new `resource_close_v1` allowance in the handoff so the diff-scope note is
accurate. One branch (`wp/px7r-resource-lifetime-substrate`), one Decision. An
`L` may land as a short commit series on the one branch. Full locked CI +
conformance run on GitHub, not locally.
