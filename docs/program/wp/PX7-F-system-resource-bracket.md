# PX7-F — `System.Resource` bracket surface + Ward lifetime-obligation integration (Foundation follow)

- **ID:** PX7-F · **Owner:** Team Foundation · **Size:** L · **Risk:** High
  (introduces the **first `System.*` module surface** in Ken and the first
  bracket that sequences a delayed body around a runtime-enforced settlement;
  must consume PX7-R's private substrate through its stable public boundary
  without re-implementing any of it, and must integrate the Spec-owned
  `ResourceLifetimeObligationV1` T-obligation without inventing the schema).
- **Objective:** Lift PX7-R's private resource substrate to Ken as
  **`System.Resource`**: the opaque `Resource k`, the sole public acquisition
  route `withResource acquire body`, use combinators, an optional early
  `release`, and the source-level honesty statements — with the generated
  **`ResourceLifetimeObligationV1`** delegated T-obligation wired into the
  `export.rs` assumption boundary, and end-to-end success/error/**controlled
  trap**/escape controls. This is the **follow** WP of PX7; it consumes a stable
  substrate and adds no new runtime enforcement.
- **Depends on:** **PX7-R merged** (the `ResourceTableV1` / `ResourceTokenV1` /
  `ResourceTraceIdentityV1` / `FsOpen` / handle-metadata / `ResourceRelease` /
  versioned discriminator substrate this consumes) **AND** the **Spec enclave's
  pinned `ResourceLifetimeObligationV1` schema + conformance route** (the
  additive `T`-body extension; Foundation implements it, does not invent it).
  **PX7-F CANNOT merge without the pinned schema.** Also on ADR-0021 (landed with
  PX7-R). **Gate:** G-Ward-seam, G-Sec.
- **Feeds:** completes PX7 (Ken-visible resource handles + `System.Resource`
  bracket, generation-checked, fail-visible, Ward-delegated). Opens the
  `System.*` namespace for later modules; PX8 may add read/write/seek ops on the
  substrate.

## Fixed inputs — DO NOT REOPEN (cite ADR-0021 / `evt_1x3rcz9q7d8g7`; the
## operator strategy is settled — do not re-ask)

- **Operator strategy (Pat 2026-07-16), settled:** affine enforcement in Rust
  (landed by PX7-R), a lifted Ken interface (this WP), exactly-once/no-leak
  **discharged to Ward**. The guarantee is real but **never a Ken affine claim
  and never a kernel `proved`**. Do not re-ask.
- **Barred (Pat, verbatim):** *"Until CS research shows a proven path, Ken will
  not have affine types."* `Resource k` is an **ordinary copyable Ken value** —
  **no** affine/linear typing, **no** feature that quietly needs it. Liveness is
  runtime-enforced by PX7-R, not type-enforced.
- **ADR home = ADR-0021** (landed with PX7-R). Implement against it.
- **Lifetime is hybrid, dynamically bracket-bounded.** `withResource acquire
  body` is the **sole public acquisition route** in V1. The acquired `Resource k`
  is copyable and may pass through arbitrary `body` computation; its **liveness
  ends when the bracket settles**. A copied/returned handle may escape
  syntactically (legal Ken) but is stale operationally: **every later use returns
  `Closed`**. **State exactly that in `System/Resource`; do NOT say the type
  prevents escape.**
- **`body` is a DELAYED function** so acquisition precedes it and settlement
  follows its returned value/error. Normal return, returned error, and a
  **controlled Ken trap** all run settlement. **External kill/abort/fatal
  signal/machine failure are OUTSIDE the guarantee and must be named as such.** A
  Ken trap must reach the runtime as a **controlled terminal outcome**; an
  aborting trap is **not** an acceptable AC.
- **Optional early `release` inside `body`** invalidates all copies. The bracket
  owns a private `release_if_live` finalizer, so early release + bracket exit does
  **not** double-close. Public `release` is **non-idempotent** (2nd call →
  `Closed`). **Do NOT export raw acquire** as a general Ken op — the raw
  acquire/release protocol is PX7-R's private substrate.
- **Authority = `FsOpen` (PX7-R), not a new family/bit.** `System.Resource` does
  **not** add `program capabilities Resource …` and does **not** broaden the
  FS-only capability declaration grammar. `System.Resource` is the **first
  `System.*` namespace** but changes no declaration grammar.
- **Ward integration = the Spec-owned `ResourceLifetimeObligationV1` schema.**
  The target emitter generates **exactly one** delegated obligation whenever its
  reachable `Σ` contains a resource acquisition; status stays `delegated`,
  content-hashed with `T`, compiled by Ward as a monitor template over runtime
  resource identities. **Foundation implements the pinned schema; it does NOT
  author or guess it.** The landed `Pred::Event(String)` two-atom form is
  **forbidden as a proxy** (it proves only uncorrelated traffic).
  - **LANDED SCHEMA (fixed input — PX7-T merged `origin/main @ 30bc5dfd`, PR
    #749):** the pinned body lives at
    `spec/70-behavioral/71-assumption-boundary.md §2.2`. The static, T-hashed
    correlation form the emitter MUST produce is a **descriptor, never a
    pre-minted runtime identity**:
    `correlation: ResourceLifetimeCorrelationV1 { identity_type:
    ResourceTraceIdentityV1, event_field: EffectEventV1.resource, bind_at:
    Successful(FsOpen), require_same_at: [FsHandleMetadata, ResourceRelease] }`,
    plus scalar `acquire_op=FsOpen` / `use_op=FsHandleMetadata` /
    `settle_op=ResourceRelease`, `status=delegated`, and
    `monitor_template: WardResourceLifetimeMonitorV1` (four checks). Runtime `r`
    is **selected by** the descriptor and is **excluded from export/hash**. The
    conformance oracle is `conformance/behavioral/resource-lifetime/
    seed-resource-lifetime.md` (RL-A validates / RL-B independent-atoms rejects);
    PX7-F's emitter output must validate against it byte-for-byte — no
    Foundation-invented field.
- **Multi-fault ordering (surface half):** body-success + release-failure →
  bracket returns the release failure; body-error + release-failure → preserve
  both; **controlled trap + cleanup-failure → trap primary + ordered
  cleanup-failure list secondary**, neither overwritten nor dropped (over PX7-R's
  versioned discriminator).
- **Source-level honesty (ADR-0017), in the Ken source itself — not just a Rust
  comment or this frame:** handles are runtime-enforced and Ward-checked; Ken
  does **not** make them affine; escaped copies become `Closed`; the guarantee
  **excludes external process destruction**.

## Scope

**In scope:** the `System.Resource` module (first `System.*` surface) exposing
the opaque `Resource k`, the bracket result/error shape, `withResource`, use
combinators, and optional early `release`; the delayed-`body` acquisition →
settlement sequencing over PX7-R's `FsOpen`/handle-metadata/`ResourceRelease`;
the controlled-trap settlement path (trap reaches the runtime as a controlled
terminal outcome); the surface `Closed` / `MalformedResource` /
`RightNotHeld` / `ReleaseFailed` result shapes lifted from PX7-R; the
generated `ResourceLifetimeObligationV1` delegated T-obligation emitted into
`export.rs` (per the pinned Spec schema) whenever `Σ` reaches a resource
acquisition; the trap-primary + ordered-cleanup-failure secondary observation
(surface half of the versioned discriminator); the source-level honesty
statements; and end-to-end success/error/controlled-trap/escape controls.

**Out of scope:** any change to PX7-R's `ResourceTableV1` / generation discipline
/ table RAII / native enforcement (consume, do not re-implement); the schema
**definition** (Spec-owned, pinned); read/write/seek ops (PX8); any Ken-level
affine/linear type; a new capability family or `RightSet` bit; any kernel change.

## Mandated deliverable outline — each section ends in a concrete choice

1. **`System.Resource` module surface.** The first `System.*` module: opaque
   `Resource k` (no Ken constructor; minted only via `withResource`), the bracket
   result/error type carrying the lifted `Closed` / `MalformedResource` /
   `RightNotHeld` / `ReleaseFailed` identities, `withResource`, the
   handle-metadata use combinator (over PX7-R's real consumer), and optional
   early `release`. No declaration-grammar change.
2. **`withResource acquire body` sequencing.** `body` is a delayed function.
   Acquire (via `FsOpen`) runs first; `body` runs with the copyable `Resource k`;
   settlement runs after `body`'s returned value/error. The private
   `release_if_live` finalizer settles once — early `release` in `body` +
   bracket exit does not double-close; public `release` is non-idempotent.
3. **Controlled-trap settlement.** A Ken trap inside `body` reaches the runtime
   as a **controlled terminal outcome** that still runs settlement (trap primary,
   ordered cleanup-failure list secondary). An aborting trap is not acceptable —
   the trap path is the controlled one. External kill/abort/signal/machine
   failure are documented as outside the guarantee.
4. **Escape semantics + source honesty.** A `Resource k` copied/returned out of
   the bracket is legal Ken but every later use returns `Closed`. The
   `System/Resource` **source** states this and the full honesty set
   (runtime-enforced + Ward-checked, not Ken-affine, escaped copies `Closed`,
   external destruction excluded) — verbatim in the Ken source, not only in Rust
   or this frame.
5. **`ResourceLifetimeObligationV1` emission.** Per the pinned Spec schema, the
   emitter produces exactly one delegated obligation into the `export.rs` `T`
   channel (status `delegated`, content-hashed with `T`) whenever the reachable
   `Σ` contains a resource acquisition, carrying the acquire/use/settle op set +
   identity-correlation policy. It correlates acquisition identity with
   settlement — **not** two uncorrelated `Pred::Event` atoms.
6. **End-to-end controls.** Success, returned-error, controlled-trap, and escape
   scenarios each drive the full acquire→body→settlement path and assert the
   exact surface result + the exact emitted obligation + the exact canonical
   observation (including multi-fault ordering).

## Acceptance criteria (testable)

- **AC1 — sole bracket acquisition, opaque handle.** `withResource` is the only
  Ken path that mints a `Resource k`; `git grep` shows no exported raw acquire and
  no Ken constructor for `Resource k`. `System.Resource` adds no
  declaration-grammar change (`parser.rs` cap parse unchanged) and no
  `program capabilities Resource` family.
- **AC2 — copyable value, escape → `Closed`.** A `Resource k` passed through
  ordinary `body` computation works while live; a handle copied/returned so it
  escapes the bracket returns `Closed` on every later use (assert the `Closed`
  variant specifically). The `System/Resource` source text states escape is legal
  but stale — verified by reading the shipped source, not a comment.
- **AC3 — settlement on normal/error/controlled-trap.** Normal return, returned
  error, and a controlled Ken trap each run settlement exactly once. Early
  `release` in `body` + bracket exit does not double-close (no second OS close);
  public `release` twice → `Closed` on the second. The trap reaches the runtime
  as a controlled terminal outcome (an aborting trap is not exercised as success
  — it is a rejected shape).
- **AC4 — multi-fault ordering.** body-success + release-failure → bracket
  result is the `ReleaseFailed`; body-error + release-failure → both preserved;
  controlled-trap + cleanup-failure → trap primary + ordered cleanup-failure list
  secondary, neither dropped nor overwritten (over PX7-R's versioned
  discriminator). Assert the exact canonical observation.
- **AC5 — exactly-one delegated obligation, correlated.** A program whose
  reachable `Σ` contains a resource acquisition emits **exactly one**
  `ResourceLifetimeObligationV1` into `export.rs`'s `T` channel, status
  `delegated`, content-hashed with `T`, matching the pinned Spec schema
  byte-for-byte; a program with no acquisition emits none. The obligation
  correlates acquire↔settle identity (structural check: it is **not** two
  `Pred::Event` atoms).
- **AC6 — honesty in source.** The shipped `System/Resource` source states:
  runtime-enforced + Ward-checked, not Ken-affine, escaped copies `Closed`,
  external process destruction excluded. `git grep` shows no affine/linear type
  introduced and no kernel `proved`/postulate.
- **AC7 — conforms to the pinned schema + Ward monitor template.** The emitted
  obligation validates against the Spec-owned conformance route for
  `ResourceLifetimeObligationV1`; no schema field is Foundation-invented. **No
  merge before the schema is pinned.**
- **AC8 — no substrate regression, CI-green.** PX7-R's `ResourceTableV1` /
  generation discipline / native enforcement are consumed unchanged (`git grep`
  shows no re-implementation); interpreter/native differential stays green.
  **No-regression = green in CI**, never a local `--workspace` run.

## Do-not-reopen guards

- Do NOT re-implement or modify PX7-R's runtime substrate — consume it through
  its public boundary.
- Do NOT author, guess, or extend the `ResourceLifetimeObligationV1` schema — it
  is Spec-owned and pinned; implement it.
- Do NOT add a Ken-level affine/linear type; `Resource k` is copyable and
  runtime-enforced.
- Do NOT export raw acquire; `withResource` is the sole public route.
- Do NOT add a `program capabilities Resource` family, spend a `RightSet` bit,
  or broaden the FS-only declaration grammar.
- Do NOT surface a `ResourceKindMismatch` identity — PX7-R fixed V1 as
  **single-kind** (`ResourceKindV1::FsHandle`) and ADR-0021 / the PX7-R frame
  **defer** the mismatch identity to the first WP that adds a second production
  resource kind. Lift only the reachable V1 roster from `ResourceErrorV1`:
  `{ Closed, MalformedResource, RightNotHeld, ReleaseFailed }` (grounded at
  `crates/ken-host/src/effect_v1.rs:465`).
- Do NOT accept an aborting trap as a settlement path; the controlled trap is the
  guaranteed one. Do NOT claim external process destruction is covered.
- Do NOT emit the two-atom `Pred::Event("acquire")`/`"release"` form as the
  exactly-once proxy.
- Do NOT merge before PX7-R is merged and the Spec schema is pinned.

## Grounding anchors (PX7-R substrate lands first; re-ground on the merged
## PX7-R head before building)

- PX7-R substrate (consume): `ResourceTableV1` / `ResourceTokenV1` /
  `ResourceTraceIdentityV1` / `HostOpV1::{FsOpen, <handle-metadata>,
  ResourceRelease}` / `ReleaseFailed` / the versioned observation discriminator —
  all in `crates/ken-host/src/effect_v1.rs` + `abi_v1.rs` as landed by PX7-R
  (re-ground exact lines on the merged PX7-R head).
- Ward assumption boundary (emit into): `TEntry { formula: Temporal }`
  `crates/ken-elaborator/src/export.rs:119`; delegated status constant
  `export.rs:440`; `TemporalObligation` `crates/ken-elaborator/src/temporal.rs:232`;
  `Pred` enum `temporal.rs:53`, `Pred::Event` `temporal.rs:193` (the form the new
  schema supersedes for correlation). The pinned `ResourceLifetimeObligationV1`
  schema is the Spec deliverable that governs this emission.
- Ken surface / module machinery: the elaborator/parser module + capability
  header path (`crates/ken-elaborator/src/parser.rs`, `crates/ken-cli/src/lib.rs`)
  — extended for the first `System.*` module **without** a declaration-grammar
  change.
- ADRs: **ADR-0021** (normative home), ADR-0006 (Ward attestation complement),
  ADR-0017 (honesty).

## Diff scope / review route

Touches `crates/**` (Foundation surface + emitter) + the pinned `spec/` +
`conformance/` route for `ResourceLifetimeObligationV1` → **Architect §14 + CV**
(the spec/conformance touch puts CV in the route). One branch
(`wp/px7f-system-resource-bracket`), one Decision. **Gated on PX7-R merge + the
Spec schema pin** — do not kick until both hold. Full locked CI + conformance run
on GitHub, not locally.
