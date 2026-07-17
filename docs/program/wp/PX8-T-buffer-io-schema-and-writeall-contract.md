# PX8-T — buffer I/O schema and `writeAll` contract

**Owner:** Spec enclave (spec-leader coordinates; spec-author elaborates; **CV
casts the Spec/Fidelity vote**). **Size:** M. **Route:** Architect §14
(soundness) **+ CV** (spec/conformance). **Gates:** PX8-F (which consumes this and
**must not invent it locally**) and PX8-R (which emits against the pinned wire
schema). This is the Spec-owned prerequisite that **precedes the final Foundation
surface** — the Architect ruled it required (`evt_2brnz8wg3ecth`). PX8-F is
authored and blocked until this schema + contract is *pinned*.

## Objective

Pin, in `/spec` + `/conformance`, the four contracts the PX8 ruling makes
load-bearing, so PX8-R (runtime) and PX8-F (Foundation surface + proofs) implement
against a Spec oracle rather than inventing shape locally:

1. the **role-labelled multi-resource observation + Ward-correlation schema** (the
   V2 successor to PX7-T's single-resource `ResourceLifetimeObligationV1`);
2. the **exact progress partition** (the read/write closed sums and the
   error-vs-progress boundary);
3. the **primitive positivity/bounds contract** the Ken `writeAll` theorem
   consumes; and
4. the **formal `writeAll` statement + discriminating conformance seeds**.

Plus the two schema deltas the ruling forces: the **`ResourceKindMismatch`**
wire/surface identity (ADR-0021 multi-kind closure) and the **`BufferLimitsV1`**
admission policy shape.

## Fixed inputs — Architect-ruled, do not reopen

The Architect's PX8 component-design ruling (`evt_2brnz8wg3ecth`, thread
`thr_6z93rvehv0qjc`, grounded vs `origin/main @ 2e588367`) is the fixed design.
Do **not** relitigate any of: `Buffer` as a second `ResourceTable` kind; the
`withBuffer` bracket; fixed non-growing capacity; the runtime live-window (no
cursor); the closed progress sums; write-zero as the `NoProgress` error;
structural-`Nat`-fuel `writeAll`; runtime+Ward safety with **no** Ken affine/linear
type and **no** kernel `proved` for lifetime (R2 closed, operator 2026-07-15). This
WP pins the *contracts*, it does not re-decide them.

## What to author (mandated deliverables — each a concrete pin, not a survey)

### D1 — Role-labelled multi-resource Ward schema (V2)

PX7-T's `ResourceLifetimeObligationV1` correlates **one** resource identity; a
positioned `readAt`/`writeAt` binds **two** (a file identity *and* a buffer
identity), so the single-resource canonical event cannot state PX8 faithfully.
Pin the **schema-versioned, role-labelled** canonical binding:

- `resource_bindings : [(Role, ResourceTraceIdentityV1)]` where
  `Role = File | Buffer | Target`; **order is operation-defined and canonical**
  (do **not** encode a buffer token into request bytes; do **not** elect one
  identity as "the" trace identity).
- The Ward lifetime body becomes a **target-specialized V2 set of per-kind
  acquire/use/settle plans**. Include a kind's plan exactly when its acquisition
  is in reachable `Σ`; its uses are the canonical ordered subsequence of that
  kind's global inventory actually present in `Σ`. The full inventory remains:
  file acquire = `FsOpen`, uses = metadata/read-at/write-at; buffer acquire =
  `BufferAllocate`, uses = read-at/write-at/freeze; settlement = generic
  `ResourceRelease`.
- The Ward monitor template requires: **each identity correlated in every role**,
  **exactly one settlement per identity**, **no successful use after settlement**,
  and **no live bracket-owned identity at the three controlled exits** (normal /
  returned-error / controlled-trap).
- **Additive / superseding-for-multi-resource-only:** existing single-resource
  emissions and every pre-PX8 export hash are preserved byte-for-byte when no
  buffer participates (the PX7-T content-hash discipline is inherited — see
  [[static-export-governing-dynamic-correlation-must-serialize-the-binder-descriptor]]).
  Status stays **`delegated`** (Ward attests; Ken does not discharge).
- **Phase split:** malformed static V2 descriptors reject before `T`/hash
  publication. Malformed runtime `resource_bindings` reject only at observation
  and Ward validation, after the static export exists, and do not alter its `T`
  bytes or hash.

### D2 — The exact progress partition

Pin, as the normative contract PX8-R emits and PX8-F's surface consumes:

- `ReadProgress = ReadSome BufferSpan TransferCount | ReadEof`
- `WriteProgress = Wrote TransferCount`
- `TransferCount` = a **constructor-private strictly-positive bounded** count with
  an `Int` projection; the `BufferSpan` and its count are **minted together** so
  their lengths agree by construction. "Complete"/"partial" are **derived**
  (count vs. effective requested window), never distinct runtime states.
- For a **positive** effective request: read-zero ⇒ `ReadEof`; read `0<n≤req` ⇒
  `ReadSome` (even short); write `0<n≤req` ⇒ `Wrote` (even short); **write
  syscall-zero ⇒ the `NoProgress`/`WriteZero` error** (NOT success — this is
  load-bearing for total `writeAll`).
- **Errors, never progress:** `Closed`, `MalformedResource`,
  `ResourceKindMismatch`, `RightNotHeld`, invalid offset/window/bounds,
  buffer-limit/allocation failure, unsupported/nonblocking posture, host-I/O
  failure. `Interrupted` is a **named error** (no silent short-success
  reclassification). `WouldBlock` is PX12, not a PX8 status. Note the seam:
  **PX9 may refine error payloads but must not change this partition.**

### D3 — Primitive positivity/bounds contract (consumed by the Ken theorem)

Pin the `readAt`/`writeAt` primitive contract precisely enough that PX8-F's
`writeAll` proof can consume it as a lemma:

- `readAt file fileOffset buffer window` / `writeAt file fileOffset buffer span`;
  `fileOffset` an explicit **nonnegative** value, target offset+length
  **overflow-checked**; neither mutates a file cursor.
- **Positivity lemma:** on a positive effective request, a successful `writeAt`
  returns `Wrote n` with `0 < n ≤ remaining`; a successful `readAt` returns
  `ReadSome span n` with `0 < n ≤ requested` **or** `ReadEof`. This strict
  positivity is the fact the fuel proof needs — pin it as a contract, so PX8-F
  proves against a Spec statement, not a runtime hope.
- A request past the available tail is **capped** (ordinary short progress); a
  start outside the buffer is an **invalid-bounds error**.

### D4 — Formal `writeAll` statement + discriminating seeds

- Pin the **theorem** `writeAll` must satisfy: it terminates; it returns the first
  transfer error **after an exact written prefix**, or returns success **only
  after the entire input span is written**; if every primitive write succeeds,
  the whole span is written.
- Pin the **discriminating conformance seeds** (§7 discipline — one per
  load-bearing branch, plus one behind indirection): (a) an all-full-writes run →
  whole span written; (b) a **short-write** sink → still completes, exact prefix
  accounting; (c) a **write-syscall-zero** → surfaces `NoProgress` (proves that
  branch is reachable and load-bearing, not vacuous); (d) a mid-stream
  **transfer error** → exact written-prefix preservation. The seeds must
  **reach** the claimed observation (not suite-green by proxy — the PX7-P/PX7-F
  QA carry).

### D5 — `ResourceKindMismatch` + `BufferLimitsV1` schema deltas

- **`ResourceKindMismatch { expected, actual }`** — its own wire/surface identity
  (not collapsed into `MalformedResource`), plus the conformance route for the
  **real reversed cross-kind pair**: (1) a buffer token to an Fs-handle-only
  consumer; (2) an Fs-handle token to a buffer-only consumer — with valid
  same-kind controls that **succeed** (a non-degenerate discriminating pair,
  §7).
- **`BufferLimitsV1`** — the deterministic admission policy shape: a per-buffer
  maximum and an invocation-wide maximum live capacity, **bound into the
  checked/native plan** (not read from an environment variable). Pin the shape;
  PX8-R enforces it.

## Do-not-reopen guard

- Do **not** redesign the Ward-delegation boundary or the hybrid-lifetime model
  (ADR-0021, landed) — this WP is the **additive V2 schema + the PX8 contracts**,
  nothing more.
- Do **not** add a Ken-level discharge path or any affine/linear machinery —
  status `delegated`, attestation only (standing operator ruling).
- Do **not** perturb any existing single-resource obligation emission or pre-PX8
  export hash (byte-preservation when no buffer participates).
- Do **not** author the runtime substrate or the Foundation surface here — this
  is the **contract**; PX8-R/PX8-F implement it.

## Acceptance criteria

- **AC1 (D1)** — the role-labelled `resource_bindings` V2 schema,
  target-specialized per-kind plans, and monitor template
  (correlate-every-role / exactly-one-settle / no-use-after-settle /
  no-live-at-three-exits) are defined in `/spec` with a locked field set, status
  `delegated`, and content-hashed with `T`. Full two-resource, buffer-only, and
  read-only-positioned targets prove plan closure over exact `Σ`; one extra
  unreachable static operation rejects before export.
- **AC2 (D1)** — no-buffer emissions and every pre-PX8 export are preserved
  byte-for-byte. The regression must include the existing checked
  resource-producing `px7f_export_resource` V1 fixture: its denotation-derived
  `Σ` is exactly `{FsOpen, FsHandleMetadata, ResourceRelease}`, excludes `FS`,
  and its complete V1 body remains coherent with I3. This with-resource control
  is structural and does not freeze an export hash. The independent no-acquire
  export remains byte-identical at its current frozen hash
  `ken-export-v0:6360c2cb74f78f7e`; a no-acquire fixture alone is insufficient.
- **AC3 (D2/D3)** — the progress partition and the primitive positivity/bounds
  contract are pinned normatively, including write-zero ⇒ `NoProgress` and the
  strict-positivity lemma the `writeAll` proof consumes.
- **AC4 (D4)** — the formal `writeAll` statement is pinned, and the four
  discriminating seeds each **reach** their branch (short-write, write-zero,
  transfer-error, all-full) — the write-zero/`NoProgress` seed is a real reaching
  negative control, not vacuous.
- **AC5 (D5)** — `ResourceKindMismatch` has its own conformance identity and the
  reversed cross-kind pair rejects while same-kind succeeds (non-degenerate
  pair); `BufferLimitsV1` shape is pinned.
- **AC6** — `spec/SPEC-PROGRESS.md` updated per enclave discipline. Static
  descriptor/version/plan/order errors reject before export; runtime missing,
  swapped, reordered, duplicate/extra, or split bindings fail observation/Ward
  validation while retaining identical static `T` bytes/hash. Kind-mismatched
  inputs remain real negative controls, not vacuous.

## Normative pin locations

- `spec/70-behavioral/71-assumption-boundary.md §2.3` owns D1 and its V1
  byte-preserving fallback.
- `spec/30-surface/38-ffi-io.md §1.7` owns D2–D5 and the formal `writeAll`
  contract.
- `spec/SPEC-PROGRESS.md` records the incremental PX8-T pin.

On pin, spec-leader routes the candidate to the Steward for publish; PX8-R may
then build the substrate against the fixed identities in parallel, and PX8-F is
unblocked to author its surface + `writeAll` Omega proofs against this oracle.
