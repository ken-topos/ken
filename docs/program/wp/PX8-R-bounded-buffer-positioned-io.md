# PX8-R — Runtime bounded-buffer + positioned-syscall substrate (consumes pinned PX8-T)

- **State:** READY — released on the PX8-T pin. The **runtime substrate** for the
  PX8 arc; analog of PX7-R. Consumes the **pinned** PX8-T Spec+conformance oracle;
  does **not** invent shape.
- **Owner:** **Team Runtime** (leader `agt_37reqrd72cg00` / implementer
  `agt_37reqg3nync00` / qa `agt_37reqvb6ce400`) — Runtime owns the native
  substrate (positioned syscalls, the runtime buffer region, progress lowering,
  runtime effect-event emission), per the PX8 opening brief §Ownership.
- **Size:** L. **Risk:** High (first bounded mutable byte region in the runtime;
  new resource kind in the live `ResourceTable`; positioned syscalls; V2
  Ward-event emission that must match the pinned schema byte-for-byte).
- **Branch:** `wp/px8r-bounded-buffer-positioned-io` off `origin/main @ d69819ca`
  (`git fetch` first — the ref, never stale local `main`). **ONE branch, ONE
  Decision** (a short series is fine for an L).
- **Route:** **Architect §14** (soundness — new resource kind + positioned
  syscalls + the fail-closed generation discipline) **+ G-Ward-seam** (the V2
  `resource_bindings` emission feeds the Ward monitor). The **R-D7 two-site
  `export.rs` closure rides this same Architect §14 Decision** — it is forced
  §9a spillover on the branch owner, **not** a separate cross-team Verify review
  cycle (Architect `evt_4ewpg88ndpxy6`). **CV only if** you touch `spec/` or
  `conformance/` — you should **not**; those are pinned by PX8-T. If a
  demonstrated behavior needs a conformance seed the pinned PX8-T seed does not
  already cover, **hard-stop to me** (do not add it yourself).

## Objective

Build the runtime substrate that makes the pinned PX8-T progress-IO + bounded
buffer contract real: a second `ResourceKind::Buffer` in the live resource table,
a bounded non-growing runtime buffer region acquired via `withBuffer`, positioned
`readAt`/`writeAt` host methods returning **progress** (short transfer = success),
and the **V2 role-labelled `resource_bindings` runtime event emission** that the
assumption-boundary export projects and the Ward monitor consumes. PX8-F (the
`System.*` Ken surface + the derived `writeAll` Omega proofs) builds **on top of
this** and must not be started until PX8-R lands.

## Fixed inputs — DO NOT REOPEN (settled; do not re-ask the operator)

Cite the Architect PX8 ruling `evt_2brnz8wg3ecth` (thread `thr_6z93rvehv0qjc`),
ADR-0021, and the **pinned PX8-T artifacts on `d69819ca`** — read these as the
authoritative contract before writing code:
- `spec/30-surface/38-ffi-io.md` — the buffer model + positioned progress surface;
- `spec/70-behavioral/71-assumption-boundary.md` — the **V2 emission schema** you
  must produce (`resource_bindings`, `ResourceBindingRoleV2`,
  `ResourceLifetimeObligationV2`/`PlanV2`, the Σ-gated plan-inclusion rule);
- `conformance/behavioral/buffer-io/seed-buffer-io.md` — the discriminating seeds
  your emission must satisfy (RB-* cases; the `RED-UNTIL-PX8-R + PX8-F` ones turn
  green as their producers land).

Settled, not forks:
- **No affine/linear types; no in-language mutable Ken value (R2 closed
  2026-07-15).** `System.Buffer` is an **opaque runtime-backed handle**; all
  mutation is a runtime effect; liveness is **runtime-enforced** (generation
  check), never type-enforced. Structurally determined — not a fork.
- **Short transfer = SUCCESS carrying progress**, never an all-or-nothing error;
  EOF-on-read is an ordinary progress outcome; **write-syscall-zero is the
  load-bearing `NoProgress`/`WriteZero` error.**
- **Fixed capacity at acquisition, non-growing; no mutable cursor** — the runtime
  tracks exactly one live window `[buffer_offset, +length)` per buffer.
- **`writeAll` is NOT yours** — it is a derived Ken loop proved in PX8-F. You
  provide the primitive single `writeAt`. Do **not** add an all-or-nothing `write`
  primitive to dodge the proof.
- **Out of scope (named so nobody builds them):** `io_uring`/async/event-loop
  (PX12), vectored `readv`/`writev`, `mmap` buffers, any seek primitive or file
  cursor (sequential IO + seek are **derived Ken** in PX8-F), any in-language
  mutable reference.

## Mandated deliverables (each a concrete implementable choice, not a survey)

- **R-D1 — `ResourceKind::Buffer` in the live `ResourceTable`.** Add
  `ResourceKind::Buffer` beside `FsHandle`; generalize the live owner to a
  **closed `FsHandle | BufferRegion` enum** (no second table, **not** in
  `CapabilityTableV1`). Generation-checked handle → `Closed` on escape after
  settlement (normal/error/trap settle; catastrophic destruction excluded exactly
  as PX7). Copyable handle; identity is the generation-checked slot.
- **R-D2 — `withBuffer capacity body` bracket** (raw allocate/release stay
  **private** substrate). Capacity **fixed, strictly positive, non-growing**,
  admitted by a deterministic **`BufferLimitsV1`** policy (per-buffer max +
  invocation-wide max) bound into the plan; over-limit acquisition rejects
  deterministically. The runtime buffer region is a bounded mutable byte area the
  handle owns for its bracket scope.
- **R-D3 — positioned `readAt file fileOffset buffer window` /
  `writeAt file fileOffset buffer span` host methods** (`pread`/`pwrite`
  semantics): explicit **non-negative** offset, **overflow-checked**, **no file
  cursor**. Return progress:
  `ReadProgress = ReadSome BufferSpan TransferCount | ReadEof`;
  `WriteProgress = Wrote TransferCount`. `TransferCount` is strictly-positive,
  bounded by the effective request, minted **with** the span so lengths agree. A
  positive short transfer is **success**; read-zero → `ReadEof`; **write-zero →
  `NoProgress`/`WriteZero` error** (never short success). Error set:
  `Closed`/`MalformedResource`/`ResourceKindMismatch`/`RightNotHeld`/invalid
  offset|window|bounds/limit/posture/host-IO/`Interrupted` (named, **no** silent
  retry); `WouldBlock` is PX12.
- **R-D4 — `ResourceKindMismatch { expected: ResourceKindV1, actual:
  ResourceKindV1 }` runtime** (ADR-0021 multi-kind closure) with its **own
  wire/surface identity** — **not** collapsed into `MalformedResource` — plus the
  **real reversed cross-kind pair** (a buffer op on a file handle and a file op on
  a buffer handle both reject with the correctly-oriented mismatch).
- **R-D5 — the canonical Runtime-owned V2 observation vocabulary + real ordered
  event emission** (Architect ruling `evt_3xs94r12c8fvg`). **Define** (Runtime owns
  these; PX8-V/Verify and PX8-F/Foundation **consume** them, never redefine
  privately): the new host-operation identities — the **four additive `HostOpV1`
  ops** `FsReadAt = 0x030D`, `FsWriteAt = 0x030E`, `BufferAllocate = 0x0402`,
  `BufferFreeze = 0x0403` (all 18 existing discriminants/spellings **unchanged**;
  extend the one first-party catalog, **not** a new `HostOpV2` — Architect ruling
  `evt_4ewpg88ndpxy6`) — `ResourceKindV1::Buffer`,
  `ResourceBindingRoleV2 = File | Buffer | Target`, and the runtime
  `EffectEventV2.resource_bindings : [(ResourceBindingRoleV2,
  ResourceTraceIdentityV1)]` carrier. **Emit** the real **ordered**
  `resource_bindings` at the successful operations, matching the **pinned §71
  canonical table**: `BufferAllocate` → `[(Target, buffer)]`;
  `FsReadAt`/`FsWriteAt` → `[(File, file), (Buffer, buffer)]`; `BufferFreeze` →
  `[(Target, buffer)]`; `ResourceRelease` → `[(Target, released)]`; file acquire
  `FsOpen`. **This deliverable ENDS at the real observation producer.** The static
  V2 export projection into `ResourceLifetimeObligationV2` (exact-Σ plan
  selection, validation, canonicalization, hash participation) is **PX8-V
  (Verify), a downstream WP that consumes this vocabulary — you neither build it
  nor block on it.** Do **not** define a second/private observation schema.
- **R-D6 — `withResource` capability-gated write/create mode** (truncate **once**
  at acquisition, **never** on a positioned write). Positioned writes never
  truncate.
- **R-D7 — forced B1 catalog-consumer closure (COORDINATION §9a same-PR
  spillover; Architect ruling `evt_4ewpg88ndpxy6`).** Extending Runtime's
  canonical `HostOpV1` catalog necessarily breaks its exhaustive first-party
  consumers, so PX8-R closes them **in this same WP** — this is spillover, **not**
  a transfer of the B1 export transaction's ownership to Runtime. Under a
  **narrow, exact** exception to the export-emitter guard, amend **only** the two
  HostOp catalog-consumer sites in `crates/ken-elaborator/src/export.rs` (+
  focused tests of those mappings): `host_operation_family` and
  `canonical_host_perform_signature_v1`. The new arms are **exact semantic
  mappings, never compile-satisfying stubs** — a wildcard, an "unavailable" alias,
  a family-label-as-identity, a duplicate spelling, or a sentinel is
  **forbidden**. All four ops stay in the existing checked host-I/O family
  `("FSOp", "FS")` (`System.Buffer` is a surface resource abstraction — **no**
  `BufferOp`/new effect label; none is authorized by PX8-T). Canonical B1 exact-Σ
  spellings are exactly `"FsReadAt"`, `"FsWriteAt"`, `"BufferAllocate"`,
  `"BufferFreeze"`. The prior 18 mappings/bytes are **unchanged**. This exception
  is **scoped to these two sites only** — it does **not** license adding or
  altering any V2 obligation type, the `V1 | V2` export body, exact-Σ V1/V2
  selection, V2 validation/canonicalization/serialization/hash, or any Ward
  consumer (all remain **PX8-V/PX8-F**, per R-D5).

## Required discriminators (all through the real runtime path)

1. **Non-degenerate progress pair** per direction: a **full** transfer and a
   **short** transfer on the same shape both return success with the exact count.
2. **EOF vs short-read:** read at/after end → `ReadEof`; read before end returning
   fewer bytes than requested → `ReadSome` success.
3. **Write-zero is load-bearing:** a sink accepting zero bytes yields
   `NoProgress`/`WriteZero` **error**, distinct from a positive short `Wrote`.
4. **Positioned isolation:** a positioned write at non-zero offset changes only
   that range; a positioned read at non-zero offset returns the correct bytes; the
   rest of the file is undisturbed.
5. **Bounded buffer:** a transfer request larger than capacity is bounded to
   capacity (partial progress), never over-runs; over-limit `withBuffer` rejects.
6. **Cross-kind mismatch:** the reversed pair (buffer-op-on-file,
   file-op-on-buffer) each reject with `ResourceKindMismatch` of the correct
   orientation, not `MalformedResource`.
7. **PX7 fail-closed intact:** read/write on a settled (`Closed`) handle returns
   `Closed`; insufficient authority returns `RightNotHeld`.
8. **V2 emission conformance:** the emitted `resource_bindings` for a
   buffer-acquiring target match the pinned §71 canonical bindings and the pinned
   buffer-io seed's structural expectations (grep the emission, not the name).

## Acceptance criteria

- **AC1** — R-D1..R-D7 landed; the buffer is a runtime-backed opaque handle, no
  path exposes the mutable region as an ordinary Ken value.
- **AC2** — positioned progress is honest: short transfer = success + exact count;
  EOF = progress; write-zero = `NoProgress` error (discriminators 1–3, reaching).
- **AC3** — bounded + positioned-isolation + cross-kind-mismatch + PX7-fail-closed
  all hold (discriminators 4–7).
- **AC4** — the real canonical `EffectEventV2.resource_bindings` producer emits the
  pinned §71 ordered vocabulary with correct **byte/order** through the real
  runtime op path (discriminator 8); **no second schema** invented. AC4 **ends at
  the observation producer** — it does **not** require the static V2 `T` projection
  or Ward consumption (those are PX8-V / PX8-F).
- **AC5** — **no-regression = GREEN IN CI** (never a local `--workspace` run;
  COORDINATION §12). Build/test **targeted only** (`scripts/ken-cargo -p
  ken-interp …` / `--test <name>`); the frozen-corpus/`--locked` gates run in CI.
- **AC6** — **HostOp catalog closure (Architect `evt_4ewpg88ndpxy6`):**
  `HostOpV1::ALL`, numeric decoding, the generated catalog, the ABI manifest,
  request/reply layout, dispatch, availability, recorder, and **every** existing
  inventory consumer close over exactly **22** operations; all prior 18
  discriminants + canonical spellings are byte-unchanged; the four new spellings
  are pairwise distinct and alias **no** existing Host or L5 identity; each new
  typed op accepts only `FSOp` and rejects a wrong family **before** export;
  unknown IDs still **fail closed**; and the retained B1/B2/B3 Host-sibling
  controls + the PX7 no-buffer V1 export controls stay green (in CI).
- **AC7** — **no PX8-V bleed:** PX8-R adds **no** static V2 obligation, **no**
  `V1 | V2` export body, **no** exact-Σ V1/V2 selection, **no** V2
  validation/canonicalization/hash, **no** Ward consumer, and **claims no real
  checked-export positive** for the four new ops. If a Runtime-only change
  unexpectedly makes a checked target containing one of these ops reachable before
  PX8-V (the current V1-only obligation selector may refuse to publish it),
  **hard-stop to me** — do not weaken the evidence.

## Do-not-reopen guard

- Do **not** add an all-or-nothing `write` primitive, a seek primitive, a file
  cursor, or any in-language mutable reference (R2 + fixed inputs).
- Do **not** touch the B1 static export emitter (`crates/ken-elaborator/src/export.rs`)
  **except** the one exact §9a-spillover exception in **R-D7** — the two HostOp
  catalog-consumer sites `host_operation_family` +
  `canonical_host_perform_signature_v1` (+ their focused tests), amended as **exact
  semantic mappings**. Everything else in that file — the static V2 projection
  (`ResourceLifetimeObligationV2`, the `V1 | V2` body, exact-Σ plan selection,
  validation/canonicalization/hash) — is **PX8-V (Verify)**, an explicit
  **downstream** dependency; PX8-R produces the observation vocabulary PX8-V
  consumes and does **not** block on it. (The original probe-and-stop guard served
  its diagnostic purpose — Architect `evt_3xs94r12c8fvg`; the HostOp-catalog
  exception is Architect `evt_4ewpg88ndpxy6`.)
- Do **not** make the buffer growable, expose the mutable region, or truncate on a
  positioned write.
- Do **not** start PX8-F work here (it consumes this; separate owner + WP).

## Sequencing

PX8-T (pinned `d69819ca`) → **PX8-R** (this — runtime observation producer) →
**PX8-V** (Verify — static V2 export projection in `export.rs`, one versioned
`V1 | V2` sum, exact-Σ selection, V1-preserving so `px7f`/no-acquire
`6360c2cb74f78f7e` stay byte-identical; framed now, implemented against the
**landed** PX8-R vocabulary) → **PX8-F** (Foundation `System.*` surface + derived
`writeAll` Omega proofs + `freeze/spanBytes` + Ward V2 consumer; **blocked until
BOTH PX8-R and PX8-V land**) → **Phase-C exit** (`cat`/`cp`/`wc` native over a
larger-than-memory file, interpreter ↔ native external-delta equality via the PX6
harness).
