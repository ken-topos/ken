# PX8-X — single-schema unification train (Runtime + Verify + Spec enclave)

> **Architect rulings `evt_291b8gcwde32v` + naming-correction `evt_5v6jrc6rnva`
> (2026-07-17), grounded on operator ruling (no backwards-compat, PRINCIPLES
> transient T, landed `78ef39eb`).** The parallel V1/V2 effect-observation and
> obligation schemas are unwanted weight. **Collapse to ONE unversioned schema —
> delete BOTH the V1 and V2 type families**, not merely V1. PX8-X is no longer a
> thin export exposure; it is an **atomic multi-owner unification train** with
> three legs (Runtime producer, Verify export, Spec enclave conformance) that
> **assemble linearly and publish under ONE combined exact SHA + ONE Decision**.
> The Verify/export leg **must not merge independently** — either half alone
> leaves a false or uncompilable schema boundary. **Ward remains external to Ken
> throughout** (the old in-Ken PX8-W consumer is struck).

- **ID:** PX8-X · **Owners:** **Team Runtime** (leg 1 — leader
  `agt_37reqrd72cg00` / impl `agt_37reqg3nync00` / qa `agt_37reqvb6ce400`),
  **Team Verify** (leg 2 — leader `agt_37reqqf16g800` / impl `agt_37reqfz3jnw00`
  / qa `agt_37reqtacftr00`), **Spec enclave** (leg 3 — spec-leader
  `agt_37reqwresqc00` / spec-author `agt_37reqfp1tm400` / conformance-validator
  `agt_37reqfr97xm00`) · **Size:** L · **Risk:** High (a whole-schema collapse
  touching the linked-native wire codec, the export projection, and the normative
  spec — a mis-delete leaves a compiling-but-lying boundary; the wire codec must
  stay **`KETRACE2`** and total/fail-closed).
- **Branch:** `wp/px8x-single-schema-unification` off **`origin/main @ 78ef39eb`**
  (one shared branch; the three legs commit onto it **linearly** — see
  Choreography). **Salvage source:** `preserved/px8x-salvage-ef938b92` (the parked
  additive-V2 WIP — salvage material, NOT a candidate; see Salvage list).
- **Route:** **ONE Decision** on the combined tip — **Runtime QA + Verify QA**
  (each leg's gate), **Architect §14** (schema-collapse soundness: the delete is
  complete, the unversioned names are correct, the wire codec is total/fail-closed,
  the export projection is faithful), **+ CV** (leg 3 touches
  `spec/`/`conformance/`). Publisher merges **only** on the resolved Decision
  OBJECT with all approvals verified fresh (§14).
- **Parallel with PX8-N** (both partly Runtime): PX8-N is the bounded-`Nat`
  compiler carrier; PX8-X is the observation/obligation schema collapse.
  Independent failure modes; coordinate on the two branches, do not bundle.
  **PX8-F is held until BOTH the PX8-X train and PX8-N land.**

## Objective

Replace the duplicated `EffectEventV1`/`EffectEventV2`,
`EffectObservationV1`/(future V2), `LinkedEffectTraceV1`/(future V2), and
`ResourceLifetimeObligation::{V1,V2}` families with **ONE unversioned schema** at
the observation and export layers, carried by a single **`KETRACE2`** codec and a
single export projection, and made normatively current in `spec/`/`conformance/`.
The role-labelled `resource_bindings` shape strictly subsumes the old single-
resource field (file-only = the degenerate case). Delete every dual-trace
accumulator, alias, wrapper, and conversion view. This is a **schema collapse
under the no-backwards-compat ruling**, not an additive exposure.

## Fixed inputs — DO NOT REOPEN (settled; do not re-ask the operator/Architect)

### The sole (unversioned) names — `evt_5v6jrc6rnva` controls

At the **observation/export layer**, the sole surviving names are **unversioned**:

- `ResourceBindingRole = File | Buffer | Target` (delete `ResourceBindingRoleV2`);
- `EffectEvent { ..., resource_bindings: Vec<(ResourceBindingRole,
  ResourceTraceIdentityV1)>, ... }` (delete `EffectEventV1`, `EffectEventV2`, and
  `EffectEventV1.resource`);
- `TerminalExitClass = NormalReturn | ReturnedError | ControlledTrap`;
- `EffectObservation` (delete `EffectObservationV1` + any V2);
- `LinkedEffectTrace` (delete `LinkedEffectTraceV1` + any V2);
- direct `ResourceLifetimeObligation`, `ResourceLifetimeCorrelation`,
  `ResourceLifetimePlan`, `ResourceLifetimeBindingPoint` (delete
  `ResourceLifetimeObligationV1` and the internal `::{V1,V2}` sum).

**Delete BOTH V1 and V2 families, not merely V1.** The direct obligation has **no
`schema_version` and no versioned `body_kind`**; its canonical body kind is
`ResourceLifetimeObligation` and its correlation names
`EffectEvent.resource_bindings`.

**The wire magic stays `KETRACE2`** — it is the operator-named codec discriminator,
**not** a licence to keep versioned Rust/schema families. **Independently
versioned host ABI/catalog vocabulary stays as-is** unless it has a *parallel
duplicate*: `HostOpV1`, `CanonicalRequestV1`, `ResourceTraceIdentityV1`,
`HostDispatchReplyV1`, `HostReplyV1`, `BufferSpanV1`, `TransferCountV1`,
`BufferRegionV1` **keep their V1 names** — this ruling does **not** rename the host
ABI. Only the *observation/obligation* families collapse.

### The unified runtime/observation shape

- **`EffectEvent` is the sole event type.** Its only resource-correlation field is
  the ordered `resource_bindings`. Binding conventions (order is **semantic**, not
  map order):
  - single-resource ops → exact degenerate `[(Target, id)]`;
  - positioned transfers → exact `[(File, file), (Buffer, buffer)]`;
  - non-resource **and** uncorrelated failing ops → `[]`;
  - `ResourceRelease` retains `[(Target, id)]` once release has begun, **including
    a fail-visible close result**.
- **`HostDispatchReplyV1` keeps its name** (host ABI catalog) but
  **`resource_identity` is deleted**. `resource_token` remains the opaque
  execution result; `resource_bindings` is the **sole** trace-correlation
  result. A successful acquisition's canonical reply identity is part of the
  **result payload**, not a second trace lane.
- **`EffectObservation` + `LinkedEffectTrace` are the sole public observation/
  linked types.** They carry the sole `EffectEvent` sequence and the closed
  `TerminalExitClass`, **computed once from the pre-exit-normalization runtime
  outcome**. The class is an **observation, never a Ward verdict** (Ken exports it;
  the external Ward project judges).
- **Exactly one direct `KETRACE2` codec.** It serializes the complete sole linked
  record; it does **not** embed an old V1 record. **Do NOT land `KETRACE3`, a
  starter byte selecting V1/V2, or dual decoders.** The decoder stays
  **total/fail-closed** over tags, lengths, roles, terminal class, and trailing
  bytes, with **canonical round-trip evidence**.

### The unified static shape

`project_resource_lifetime_obligation` returns
**`Option<ResourceLifetimeObligation>`** directly (delete the `V1|V2` sum):

- file-only exact Σ → **one `FsHandle` plan**;
- buffer-only exact Σ → **one `Buffer` plan**;
- full positioned I/O → **ordered `FsHandle`, then `Buffer` plans**;
- every `require_same_at` is the **reachable ordered subsequence**;
- **no acquisition in exact Σ → `None`.**

The correlation descriptor always names `EffectEvent.resource_bindings`. **Runtime
identities never enter the static export.** Every resource-producing fixture/hash
is **intentionally rebaselined once** (the old byte-preservation rule is
superseded). A **no-acquire export remains an unchanged negative control** — it
contains no resource obligation, so nothing changes; it is a useful control, **not
a compatibility promise**.

### Landed anchors on `f2f60083` (verify before editing; `git show origin/main:`)

- **Duplication to collapse (grounded by the Architect):** `HostDispatchReplyV1`
  (`ken-host/src/effect_v1.rs:1323`) carries **both** `resource_identity` +
  `resource_bindings`; `EffectEventV1` (`:2112`) `resource: Option<
  ResourceTraceIdentityV1>` vs `EffectEventV2` (`:2129`) `resource_bindings`
  (`:2133`), `ResourceBindingRoleV2 = File|Buffer|Target` (`:2122`);
  `effect_event_v2_from_dispatch` (`:2140-2153`).
- **Interpreter dual accumulator:** `EffectTraceRecorderV1` (`ken-interp/src/
  eval.rs:4871-4877`) `events: Vec<EffectEventV1>` (`:4873`) **and** `events_v2:
  Vec<EffectEventV2>` (`:4874`), both in `record` (`:4884-4897`);
  `run_io_effect_observation_v1` (`:4910`) drops `events_v2`.
- **Native dual accumulator:** `ProcessContext` (`ken-host/src/abi_v1.rs:291`)
  `effect_trace: Vec<EffectEventV1>` (`:297`) **and** `effect_trace_v2` (`:298`);
  `write_observation_v1` (`:914-930`) serializes V1 only; linked runner
  `run_bound_process_effect_observation_v1` (`ken-runtime/src/
  object_linker_packaging.rs:240-298`) returns `EffectObservationV1`, decoding via
  `decode_linked_effect_trace_v1` (`:268`).
- **Wire codec:** `KETRACE2` in `ken-host/src/effect_wire_v1.rs`
  (`LinkedEffectTraceV1` `:15-22`).
- **Export sum:** `ResourceLifetimeObligation = V1 | V2` in
  `ken-elaborator/src/export.rs` (V1 file-only; V2 +buffer/role bindings);
  terminal-exit policy strings `export.rs:191/1114-1116`.
- **Normative fork:** `spec/.../§71` §2.3 monitor template + `§71:342-343`
  terminal-exit; `SPEC-PROGRESS.md`; `conformance/behavioral/buffer-io/` seed
  cases — all still describe a V1/V2 fork.

### Salvage from `preserved/px8x-salvage-ef938b92` (NOT a candidate)

**Retain:** the sealed three-arm terminal classifier and the **real role-binding
fixtures** (interpreter Buffer bracket + linked-native resource bracket role-binding
discriminators; all three terminal arms). **Discard:** the additive V1/V2
collectors, the `KETRACE3` wrapper, the embedded-V1 payload, the same-event
agreement checks, and the compatibility start-byte selection. The salvage is
**source material to lift the classifier + fixtures from**, re-expressed against
the sole unversioned schema — not a branch to build on.

## The three legs (component ownership; ONE combined SHA)

### Leg 1 — Runtime (the sole producer)

Owns the sole `EffectEvent`/`HostDispatchReplyV1`(field-pruned)/`EffectObservation`/
`LinkedEffectTrace`/`KETRACE2` producer and the **interpreter + native + CLI
transport migration**.

- **X1-D1** — collapse the event type: delete `EffectEventV1` +
  `EffectEventV1.resource` + `EffectEventV2`; introduce the sole
  `EffectEvent` with `resource_bindings` (unversioned
  `ResourceBindingRole`). Delete every dual-trace accumulator
  (`events`+`events_v2` → one `events`; `effect_trace`+`effect_trace_v2` → one
  `effect_trace`).
- **X1-D2** — prune `HostDispatchReplyV1.resource_identity`; `resource_bindings`
  is the sole trace-correlation result; `resource_token` unchanged.
- **X1-D3** — the sole `EffectObservation`/`LinkedEffectTrace` carrying the sole
  `EffectEvent` sequence + the sealed `TerminalExitClass` (**no `_ =>`**), computed
  once from the pre-exit-normalization outcome (lift the classifier from salvage).
- **X1-D4** — the sole `KETRACE2` codec: one direct linked record,
  total/fail-closed decoder, canonical round-trip evidence. No
  `KETRACE3`/starter-byte/dual decoder.
- **X1-D5** — migrate every interpreter/native/CLI transport caller to the sole
  routes (delete the V1-only `run_io_effect_observation_v1` /
  `run_bound_process_effect_observation_v1` shapes → sole routes returning
  `EffectObservation`). Bindings follow the semantic-order conventions above.
- **Leg-1 QA:** Runtime QA — the sole producer emits ordered role-labelled
  bindings on both lanes; all three `TerminalExitClass` arms reach; `KETRACE2`
  round-trips; no dangling V1 accumulator/type.

### Leg 2 — Verify (the export collapse + consumer migration)

Owns the `ken-elaborator/export.rs` collapse, the `ken-verify` observation-consumer
migration, and **all exact-Σ/hash tests**. **Does not merge without leg 1** (its
types come from leg 1) and **leg 1 does not merge without leg 2** (leg 1 alone
leaves `export.rs` referencing deleted types).

- **X2-D1** — collapse `ResourceLifetimeObligation::{V1,V2}` → direct
  `ResourceLifetimeObligation`; `project_resource_lifetime_obligation` returns
  `Option<ResourceLifetimeObligation>` with the file-only / buffer-only / full
  positioned / `None` cases above; correlation names `EffectEvent.resource_bindings`.
- **X2-D2** — migrate every `ken-verify` observation consumer
  (`canonical`/`catalog`/`host`/`lib`/`scenario`.rs + the differential harness) to
  the sole `EffectObservation`/`LinkedEffectTrace`.
- **X2-D3** — rebaseline every resource-producing exact-Σ/hash fixture **once**;
  retain the no-acquire export negative control unchanged
  (`ken-export-v0:6360c2cb74f78f7e` — confirm it is genuinely unchanged, as it has
  no resource obligation).
- **Leg-2 QA:** Verify QA — export projection faithful to the sole schema; all
  exact-Σ subsequence cases correct; the differential harness green; the no-acquire
  control byte-identical.

### Leg 3 — Spec enclave (normative currency)

Owns the `§71`/`SPEC-PROGRESS.md`/`conformance/` companion that removes the live
V1/V2 normative fork.

- **X3-D1** — `§71` §2.3 + `§71:342-343`: name the sole unversioned
  `EffectObservation`/`TerminalExitClass`/`ResourceLifetimeObligation`; the monitor
  template consumes `EffectEvent.resource_bindings` (the monitor itself stays the
  **external Ward project's** — spec only fixes the exported shape).
- **X3-D2** — `SPEC-PROGRESS.md` currency; sweep
  `conformance/behavioral/buffer-io/` seed labels so no live normative route
  advertises V1/V2 (dated historical prose
  may remain only when **explicitly marked superseded**).
- **X3-D3** — the RB-A…RB-O behavioral seed RED-UNTIL labels read
  **Ward-delegated / out-of-Ken** (the deferred hygiene sweep folds in here).
- **Leg-3 gate:** CV — normative currency; the conformance corpus matches the sole
  schema; no live V1/V2 fork remains.

## Choreography (atomic train — like PX8-P+PX8-V)

1. **Steward** frames + pushes `wp/px8x-single-schema-unification @ <frame>` off
   `origin/main`, carrying this brief.
2. **Leg 1 (Runtime)** builds on the frame → releases the producer tip; hands the
   branch forward with the sole types landed.
3. **Leg 2 (Verify)** continues on the **same branch** atop leg 1 → collapses
   `export.rs` + migrates consumers + rebaselines hashes.
4. **Leg 3 (Spec enclave)** continues on the **same branch** atop leg 2 → normative
   currency + conformance sweep.
5. **Steward** assembles the combined tip, opens **ONE Decision**; gates =
   **Runtime QA + Verify QA + Architect §14 + CV**. Publisher merges the combined
   SHA only on the resolved Decision object (§14), non-doc-only (crates touched →
   wait CI). Verify byte-identical crates on main after.

**The export/Verify leg must not merge independently** — leg 1 alone leaves
`export.rs` uncompilable; leg 2 alone references undeleted types; leg 3 alone
documents a schema that isn't landed. One SHA, one Decision, or nothing.

## Acceptance criteria (testable)

- **AC1** — the sole unversioned schema exists and **both** V1 and V2 families are
  **deleted** (grep: no `EffectEventV1`/`EffectEventV2`/`EffectObservationV1`/
  `LinkedEffectTraceV1`/`ResourceLifetimeObligationV1`/`::{V1,V2}` in live code;
  `resource_identity` removed from `HostDispatchReplyV1`).
- **AC2** — the sole `EffectEvent` carries ordered role-labelled
  `resource_bindings` per the semantic-order conventions on both lanes;
  single-resource → `[(Target, id)]`, positioned →
  `[(File,file),(Buffer,buffer)]`, uncorrelated failing → `[]`,
  release → `[(Target, id)]` (grep the emitted order/roles).
- **AC3** — `TerminalExitClass` is sealed (no `_ =>`); all three arms reach.
- **AC4** — exactly one `KETRACE2` codec; total/fail-closed decoder over
  tags/lengths/roles/terminal-class/trailing bytes; canonical round-trip evidence;
  **no `KETRACE3`/starter-byte/dual decoder**.
- **AC5** —
  `project_resource_lifetime_obligation : Option<ResourceLifetimeObligation>`
  with the four exact cases; correlation names `EffectEvent.resource_bindings`; no
  runtime identity in the static export; resource-producing fixtures rebaselined
  once; the no-acquire control unchanged.
- **AC6** — normative currency: `§71`/`SPEC-PROGRESS`/`conformance` name the sole
  schema; no live route advertises V1/V2; RB-A…RB-O read Ward-delegated/out-of-Ken.
- **AC7** — **ONE combined SHA + ONE Decision**; the export leg did **not** merge
  alone; gates Runtime QA + Verify QA + Architect §14 + CV all fresh-verified.
- **AC8** — **no-regression = GREEN IN CI** (never a local `--workspace` run;
  COORDINATION §12). Build/test **targeted only** per leg (`-p ken-host` / `-p
  ken-interp` / `-p ken-runtime` / `-p ken-elaborator` / `-p ken-verify` / `-p
  ken-cli --test <name>`), **plus** the ken-cli integration + `ken-verify`
  differential suites the wire/export change implicates before release.

## Do-not-reopen guard

- Do **not** keep any V1 **or** V2 observation/obligation type, alias, wrapper, or
  conversion view — delete both families; the sole schema is unversioned.
- Do **not** rename the host ABI vocabulary (`HostOpV1`/`CanonicalRequestV1`/
  `ResourceTraceIdentityV1`/`HostDispatchReplyV1`/`HostReplyV1`/
  `BufferSpanV1`/`TransferCountV1`) — only the observation/obligation
  families collapse.
- Do **not** add `schema_version`, a versioned `body_kind`, `KETRACE3`, a starter
  byte, or a dual decoder — one `KETRACE2` codec, one direct record.
- Do **not** add a second event producer or a second trace lane —
  `resource_bindings` is the sole correlation result; `resource_token` stays
  the opaque exec result.
- Do **not** let the class become a verdict, populate `Q`, or touch
  `status: delegated` — `TerminalExitClass` is an observation; Ward is external.
- Do **not** merge the export/Verify leg independently — one combined SHA + one
  Decision.
- Do **not** carry a runtime identity into the static export; do **not** treat the
  no-acquire control as a compatibility promise.
- If the collapse cannot be done without keeping a V1 alias somewhere (e.g. an
  external consumer the ruling didn't anticipate), **HARD-STOP to the Steward** —
  do not smuggle a compatibility shim.

## Sequencing

`{ PX8-X train ∥ PX8-N } → PX8-F rebased terminal gate → Phase-C exit`. The
unified train **lands before PX8-F's terminal release** (PX8-F must not merge on
today's sum and then be converted). PX8-X and PX8-N are independent and parallel;
both must land before Foundation rebases the preserved PX8-F candidate onto the
combined main. Ward remains external to Ken throughout — Ken exports the sole
`ResourceLifetimeObligation` + the sole `EffectObservation`; the external Ward
project consumes and judges them.
