# PX7-R — Runtime resource-lifetime substrate: `ResourceTableV1` + `FsOpen`/use/`ResourceRelease` (Runtime lead)

- **ID:** PX7-R · **Owner:** Team Runtime · **Size:** L · **Risk:** High
  (introduces the **first held-across-steps resource handle** in the runtime — a
  real departure from the landed path-based/stateless FS model — plus a new
  generation-checked table, a new host-op family, ABI-hash movement, and the
  interpreter/native differential for all of it; must not perturb the landed
  `CapabilityTableV1`, PX16's `FsRootSpec` resolution, ADR-0018 §4 relative
  canonicalization, or PX6's twin-root differential).
- **Objective:** Land the **private runtime substrate** for opaque, dynamically
  acquired resource handles: a sibling `ResourceTableV1` with opaque
  `ResourceTokenV1 { slot, generation }`, generation-invalidate-before-close
  release, a lane-independent `ResourceTraceIdentityV1`, and three V1 host ops —
  `FsOpen` (capability-gated acquisition), one **real** non-release consumer
  (handle-metadata), and generic `ResourceRelease` — with structured
  `ReleaseFailed` errors and full interpreter/native differential closure. This
  is the **lead** WP of PX7; it lands a complete, stable, private boundary that
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
  `Closed`; zero/out-of-range/never-minted encoding → `MalformedResource`; live
  token, wrong op → `ResourceKindMismatch`.
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
structured `ReleaseFailed` + `Closed`/`MalformedResource`/`ResourceKindMismatch`
identities; the versioned observation/wire discriminator; and the
interpreter/native differential for every op and every negative control.

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
   `FsHandle` kind), the owned backend object (an `OwnedFd`-backed handle),
   attenuated rights/context from acquisition, and the `ResourceTraceIdentityV1`.
   `resolve()` returns the live owner only on exact slot+generation+kind match,
   else the exact distinct identity (`Closed` / `MalformedResource` /
   `ResourceKindMismatch`).
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
6. **`ResourceRelease` op + generation discipline.** Generic
   `HostOpV1::ResourceRelease` resolves the live slot, **moves the owned object
   out, bumps/invalidates the generation, THEN closes**. Public release is
   non-idempotent (2nd call → `Closed`). A close error becomes `ReleaseFailed {
   resource_kind, identity: ResourceTraceIdentityV1, io: IoErrorIdentityV1 }` and
   is **never retried**; the handle is closed regardless. Use an **explicit
   finalizer** that records the outcome; `Drop` covers only catastrophic
   unwinding. On generation wrap, retire the slot permanently.
7. **Versioned observation/wire discriminator.** Add the versioned
   observation/wire discriminator that lets release outcome (success vs.
   `ReleaseFailed`) and, in PX7-F, body/settlement pairing be represented without
   overwriting either fault. PX7-R lands the discriminator + the single-fault
   release-failure encoding and round-trips it exactly.
8. **Interpreter/native differential.** Extend the PX6 differential and the
   native lane so every V1 op and every negative control produces
   **byte-identical** canonical observations across interpreter and native
   lanes; the `ResourceTraceIdentityV1` (acquisition-order) is what makes the two
   lanes agree without leaking fd/slot provenance.

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
- **AC3 — distinct fail-visible identities.** Each of these produces its exact
  distinct identity (assert the specific variant, never `is_err`): retired
  generation → `Closed`; zero/out-of-range/never-minted → `MalformedResource`;
  live token used by the wrong op → `ResourceKindMismatch`; OS close failure →
  `ReleaseFailed { resource_kind, identity, io: IoErrorIdentityV1 }` with **no**
  fd exposed. A stale token never resolves a reused slot or a recycled fd.
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
- **AC6 — lane-independent identity + interp/native equality.** For every V1 op
  and every negative control, the interpreter and native lanes produce
  **byte-identical** canonical observations; the `ResourceTraceIdentityV1` in
  those observations is the acquisition-order identity and `git grep` /
  structural check confirms it is derived from no
  fd/slot/generation/pointer/inode/executor value.
- **AC7 — explicit finalizer, not `Drop` alone.** A structural check confirms
  the controlled-exit close path runs an explicit finalizer that records the
  outcome (so a close failure is observable as `ReleaseFailed`), and that `Drop`
  is only the catastrophic-unwinding backstop — the normal contract does not
  depend on `Drop`.
- **AC8 — ABI/hash/inventory closed + wire round-trips.** The three new
  `HostOpV1` variants, their `CanonicalRequestV1` arms, wire request structs, and
  the versioned discriminator are in the generated ABI size/offset hash and the
  op inventory; every new wire form round-trips exactly (encode→decode identity),
  and the mutation gates cover them. **No-regression = green in CI**, never a
  local `--workspace` run.
- **AC9 — honesty + no-affine.** ADR-0021 states the runtime-enforced /
  Ward-delegated / never-Ken-affine / never-kernel-proved boundary honestly.
  `git grep` shows PX7-R adds no affine/linear type, no kernel postulate, and no
  new `unsafe` allowance beyond the audited boundaries (the `OwnedFd` handle uses
  the landed safe boundary).

## Do-not-reopen guards

- Do NOT extend `CapabilityTableV1` or perturb `MalformedCapability`; the
  resource table is a **sibling**.
- Do NOT add a Ken-level affine/linear type, a kernel postulate/primitive, or a
  feature that quietly needs affinity. The affine discipline is Rust-only.
- Do NOT add a `program capabilities Resource` family or spend a `RightSet` bit;
  `FsOpen` carries the domain grant.
- Do NOT use `Drop` alone for the normal close contract; use an explicit
  finalizer that records the outcome.
- Do NOT key `ResourceTraceIdentityV1` on fd/slot/generation/pointer/inode/
  executor provenance; it is acquisition-order.
- Do NOT retry a raw descriptor after a close error; do NOT reissue a retired
  generation/slot; do NOT let a stale token resolve a reused slot or recycled fd.
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
- Cap declaration surface (unchanged): `parser.rs:269`
  (`crates/ken-elaborator/src/parser.rs`); `program capabilities FS <authority>`
  `crates/ken-cli/src/lib.rs`.
- Differential harness (extend): PX6 twin-root
  `crates/ken-verify/src/scenario.rs`; canonical FS observations `FsDeltaV1`/
  `FsNodeObservationV1` `crates/ken-host/src/effect_v1.rs`.
- ADRs: **ADR-0021** (this WP's normative home), ADR-0019 (capability
  evolution), ADR-0018 (native-effect contract), ADR-0017 (honesty), ADR-0006
  (Ward attestation complement).

## Diff scope / review route

Touches `crates/**` + `docs/adr/0021-*` + `docs/program/wp/PX7-R-*` — **no**
`spec/`+`conformance/` (the `ResourceLifetimeObligationV1` schema is Spec-owned
and pinned separately for PX7-F) → **Architect-only §14** (CV not in route). One
branch (`wp/px7r-resource-lifetime-substrate`), one Decision. An `L` may land as
a short commit series on the one branch. Full locked CI + conformance run on
GitHub, not locally.
