# PX7-T — `ResourceLifetimeObligationV1` delegated T-schema (Spec enclave)

**Owner:** Spec enclave (spec-leader coordinates; spec-author elaborates; CV
validates). **Size:** S. **Route:** Architect §14 **+ CV** (touches
spec/conformance). **Gates:** PX7-F. This is the Spec-owned prerequisite PX7-F
consumes but must not invent — PX7-F is authored and blocked until this schema is
*pinned* (PX7-F frame §"Do NOT author the schema").

## Objective

Author a narrow, additive, Spec-owned T-body schema `ResourceLifetimeObligationV1`
plus its conformance route, so a resource lifetime's `acquire ↔ settle`
correlation can be expressed as **one delegated obligation** and Ward can attest
it. PX7-F's target emitter then generates exactly one such obligation per program
whose `Σ` contains a resource acquisition, validated against this route.

## Why (the expressibility gap — a fixed input, Architect-ruled)

The landed Ward assumption boundary is `TEntry { formula: Temporal }` with
`Pred::Event` (`crates/ken-elaborator/src/export.rs:440`; `TemporalObligation`
at `crates/ken-elaborator/src/temporal.rs:232`). That shape records events but
**cannot correlate an `acquire` identity with its matching `settle`** across a
bracket — it yields two *uncorrelated* events, not one *correlated* lifetime
obligation. PX7's guarantee (exactly-once settle of the *same* acquired handle)
needs that correlation. Hence a narrow additive schema that *supersedes*
`TemporalObligation` **only for the correlation case**.

## What to author (Architect-ruled fixed inputs — do NOT relitigate)

A new delegated T-body schema **`ResourceLifetimeObligationV1`**:

1. **Carries the acquire / use / settle op set** for a resource lifetime, plus an
   **identity-correlation policy** tying the acquisition identity to its
   settlement (structurally *one correlated obligation*, **not** two independent
   `Pred::Event`s). Correlation keys on the lifetime identity — **not**
   fd/slot/inode — mirroring Runtime's lane-independent `ResourceTraceIdentityV1`
   (acquisition-order identity, landed in PX7-R `crates/ken-host`).
   The locked, canonical descriptor is:

   ```text
   ResourceLifetimeCorrelationV1 {
     identity_type: ResourceTraceIdentityV1,
     event_field: EffectEventV1.resource,
     bind_at: Successful(FsOpen),
     require_same_at: [FsHandleMetadata, ResourceRelease],
   }
   ```

   The target-level `T` entry hashes this descriptor, not a runtime identity
   value. A successful `FsOpen` binds `r` only when Ward evaluates the monitor;
   the two named later operations must carry that same `r` in
   `EffectEventV1.resource`.
2. **Status = `delegated`** — Ken does not discharge it; **Ward attests** the four
   lifecycle properties. The schema carries a **Ward monitor template** (the
   attestation shape Ward consumes). Attestation-only: Ken emits, Ward checks.
   (Cf. ADR-0021 on `origin/main`.)
3. **Content-hashed with `T`** — the obligation participates in the same
   content-hash discipline as the rest of the `T` channel; no out-of-band field.
4. **Additive / superseding-for-correlation-only** — must **not** disturb, weaken,
   or re-spell the landed temporal machinery (`export.rs` / `temporal.rs`) for any
   existing obligation. Existing emissions keep emitting `TEntry{formula:Temporal}`
   unchanged.
5. **Op set must match PX7-R's landed V1 inventory** — `FsOpen`, handle-metadata
   use, `ResourceRelease` (one non-release consumer exists). Do not assume ops
   PX7-R did not ship.

## Mandated deliverable

- The **pinned schema** in `spec/` (enclave chooses exact section/spelling and
  locked granularity — that is the enclave's call), defining the fields above.
- A **conformance route** for `ResourceLifetimeObligationV1` so PX7-F's emitter
  can validate its generated obligation against a Spec oracle — no
  Foundation-invented field passes. PX7-F AC5 / its conformance AC bind here.
- Update `spec/SPEC-PROGRESS.md` per enclave discipline.

## Do-not-reopen guard

- Do **not** redesign the Ward-delegation boundary or the hybrid-lifetime model —
  Architect-ruled and landed in ADR-0021. This WP is *only* the additive T-body
  schema + its conformance route.
- Do **not** add a Ken-level discharge path — status is `delegated`, attestation
  only (no Ken affine/linear machinery; standing operator ruling).
- Do **not** extend or re-spell existing `TemporalObligation` semantics.

## Acceptance criteria

- **AC1** — `ResourceLifetimeObligationV1` is defined in `spec/` with a locked
  field set covering the acquire/use/settle op set + the explicit canonical
  `ResourceLifetimeCorrelationV1` binder descriptor + the Ward monitor template,
  status `delegated`, content-hashed with `T`. Every descriptor field is in the
  hash input; a runtime identity value is not.
- **AC2** — the schema structurally encodes **one correlated** obligation
  (`EffectEventV1.resource` bound at successful `FsOpen`, then required equal at
  `FsHandleMetadata` and `ResourceRelease`), demonstrably not two independent
  `Pred::Event`s.
- **AC3** — a conformance route validates a conforming obligation and rejects a
  malformed / uncorrelated one (a real negative control, not vacuous). The
  discriminating pair holds every other field fixed and varies only the
  canonical descriptor versus independent atoms.
- **AC4** — no change to existing `TemporalObligation` emission/semantics; the new
  schema is purely additive and supersedes only for the correlation case.
- **AC5** — the op set matches PX7-R's landed V1 inventory exactly (no ops PX7-R
  did not ship).

On pin, spec-leader routes the candidate to the Steward for publish; PX7-F's other
gate (PX7-R merge) is already met at `origin/main @ 88c8db02`.
