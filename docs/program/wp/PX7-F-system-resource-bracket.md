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
- **Surface error sum + authorized native ABI projection (Architect-ruled,
  `evt_648wsvp2w33yg`).** Post-acquisition operations use ONE explicit Ken error
  sum, and the native reply gets ONE additive projection so the public
  checked-Ken interp↔native differential distinguishes the variants (**never**
  trace-only evidence). The reachable V1 roster is fixed; `ResourceKindMismatch`
  stays deferred (no V1 producer).
  - **Fixed Ken surface** — post-acquisition error sum. `required`/`held` are the
    exact widened `u8` right masks; `ReleaseFailed` exposes **no fd** — its
    surface fields are the ADR-0021 identity (kind + acquisition-order trace
    identity + host-neutral I/O identity); the internal wire schema version is
    validated **before** construction and is NOT a user field;
    `ResourceTraceIdentity` preserves ALL 64 bits via an opaque module-private
    two-`u32`-limb constructor — do **not** narrow to signed Ken `Int`;
    `MalformedResource` is a total/fail-closed case but is **not** claimed
    reachable from valid public Ken (`Resource k` has no public constructor):

    ```text
    ResourceError =
        HostIO(IOError)
      | Closed
      | MalformedResource
      | RightNotHeld(required: Int, held: Int)
      | ReleaseFailed(resource_kind: ResourceKind,
                      identity: ResourceTraceIdentity,
                      io: IOError)

    ResourceKind = FsHandle
    ```

    The separately-reachable metadata-backend I/O branch is `HostIO(IOError)` — it
    must **not** be confused with a resource-table error.
  - **Fixed native ABI projection** — keep existing `REPLY_ERROR/detail`
    UNCHANGED for generic I/O/file/capability errors (detail `6` remains
    `io.InvalidInput`, **never** reinterpreted as a resource error). Add a
    DISTINCT generated reply tag + a fixed, probed payload:

    ```text
    tag|reply.resource_error|6
    error|resource.Closed|0
    error|resource.MalformedResource|1
    error|resource.RightNotHeld|2
    error|resource.ReleaseFailed|3
    tag|resource_kind.FsHandle|0
    lifetime|resource_error_reply_schema|1

    ResourceErrorReplyV1 { schema_version: u64, resource_kind: u64,
                           identity: u64, io: u64, required: u64, held: u64 }
    HostReplyV1 { tag: u64, detail: u64, bytes: SliceV1,
                  resource_error: ResourceErrorReplyV1 }
    ```

    For `REPLY_RESOURCE_ERROR`, `detail` is EXACTLY the resource-error
    discriminator; the payload is always fully initialized —
    `Closed`/`MalformedResource` zero every field, `RightNotHeld` sets only schema
    + required/held, `ReleaseFailed` sets schema + kind + identity + the existing
    packed `IoErrorIdentityV1` and zeros required/held. FAIL CLOSED on an unknown
    discriminator, wrong schema, unknown kind, invalid I/O tag, out-of-range
    rights, or noncanonical nonzero unused fields. The C probe + Rust record agree
    on size/alignment/every offset; the catalog, generated binding registry,
    `HostEffectWireLayoutV1`, ABI hash, observer, Cranelift consumer, compiler
    symbol resolution, and mutation tests all move TOGETHER. Cranelift accepts:
    `FsOpen` → resource success or existing generic file error only;
    `FsHandleMetadata` → metadata success, existing generic I/O error → `HostIO`,
    or the new resource-error reply; `ResourceRelease` → unit success or the new
    resource-error reply only.
  - **Companion projection, NOT a substrate redesign.** `ResourceTableV1`,
    token/generation rules, invalidation-before-close, no-retry, and close
    ownership remain **byte-for-byte untouched**. The "consume without modifying
    PX7-R" guard is amended to authorize ONLY the ABI/projection files needed for
    this closed mapping.
  - **Evidence boundary.** The deterministic public checked-Ken
    interp↔linked-native controls MUST reach success, escaped/double-release
    `Closed`, and `RightNotHeld` with exact surface constructors + canonical
    observations. A valid public program CANNOT manufacture `MalformedResource` →
    do **not** demand a fake reaching case. Real OS-close failure is
    nondeterministic → keep the caller-control label: test `ReleaseFailed`
    construction, ABI encode/decode/field-preservation, closed-after-error,
    no-retry, and multi-fault ordering with the PRIVATE injected close result —
    but do **not** call that an observed linked-artifact OS-close failure, and do
    **not** add an env/TLS/script carrier across `exec` to force it.
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
terminal outcome); the surface error sum `HostIO` / `Closed` /
`MalformedResource` / `RightNotHeld` / `ReleaseFailed` lifted from PX7-R; **the
authorized additive native `REPLY_RESOURCE_ERROR` ABI projection** (a distinct
generated reply tag + `ResourceErrorReplyV1` payload, per the Architect ruling) so
the public checked-Ken interp↔native differential distinguishes the resource-error
variants, with its coordinated ABI-surface files (catalog / generated binding
registry / `HostEffectWireLayoutV1` / ABI hash / observer / Cranelift consumer /
compiler symbol resolution / C-probe + mutation tests) moving together; the
generated `ResourceLifetimeObligationV1` delegated T-obligation emitted into
`export.rs` (per the pinned Spec schema) whenever `Σ` reaches a resource
acquisition; the trap-primary + ordered-cleanup-failure secondary observation
(surface half of the versioned discriminator); the source-level honesty
statements; and end-to-end success/error/controlled-trap/escape controls.

**Out of scope:** any change to PX7-R's `ResourceTableV1` / generation discipline
/ table RAII / native enforcement / token / close ownership (consume, do not
re-implement — these stay **byte-for-byte untouched**; the ONLY authorized
`crates/**` movement beyond the new surface + emitter is the closed additive
`REPLY_RESOURCE_ERROR` ABI/projection mapping above); the schema **definition**
(Spec-owned, pinned); read/write/seek ops (PX8); any Ken-level affine/linear type;
a new capability family or `RightSet` bit; any kernel change.

## Mandated deliverable outline — each section ends in a concrete choice

1. **`System.Resource` module surface + native ABI projection.** The first
   `System.*` module: opaque `Resource k` (no Ken constructor; minted only via
   `withResource`), the post-acquisition **error sum** `HostIO` / `Closed` /
   `MalformedResource` / `RightNotHeld(required, held)` / `ReleaseFailed(kind,
   identity, io)` exactly as fixed above (identity keeps all 64 bits; no fd on
   `ReleaseFailed`; `MalformedResource` total-but-not-publicly-reachable),
   `withResource`, the handle-metadata use combinator (over PX7-R's real
   consumer), and optional early `release`. **Plus** the additive native
   `REPLY_RESOURCE_ERROR` projection (distinct reply tag; `ResourceErrorReplyV1`
   fully-initialized payload; `detail` = the exact discriminator; fail-closed on
   every malformed field) with the C-probe/Rust-record offset agreement and the
   coordinated ABI-surface files moving together — the substrate untouched. No
   declaration-grammar change; existing `REPLY_ERROR/detail 6` semantics
   unchanged.
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
- **AC2 — copyable value, escape → `Closed`; public differential reaches the
  distinguishable variants.** A `Resource k` passed through ordinary `body`
  computation works while live; a handle copied/returned so it escapes the bracket
  returns `Closed` on every later use (assert the `Closed` variant specifically).
  The **deterministic public checked-Ken interp↔linked-native controls** reach
  **success, escaped/double-release `Closed`, and `RightNotHeld`** with the exact
  surface constructors and canonical observations **through the new
  `REPLY_RESOURCE_ERROR` reply tag** (not trace-only evidence, and not a
  `detail 6` reinterpretation). `MalformedResource` is a total surface case but is
  **not** demanded as a reaching case (no public constructor for `Resource k`).
  The `System/Resource` source text states escape is legal but stale — verified by
  reading the shipped source, not a comment.
- **AC3 — settlement on normal/error/controlled-trap.** Normal return, returned
  error, and a controlled Ken trap each run settlement exactly once. Early
  `release` in `body` + bracket exit does not double-close (no second OS close);
  public `release` twice → `Closed` on the second. The trap reaches the runtime
  as a controlled terminal outcome (an aborting trap is not exercised as success
  — it is a rejected shape).
- **AC4 — multi-fault ordering + `ReleaseFailed` via private injection.**
  body-success + release-failure → bracket result is the `ReleaseFailed`;
  body-error + release-failure → both preserved; controlled-trap + cleanup-failure
  → trap primary + ordered cleanup-failure list secondary, neither dropped nor
  overwritten (over PX7-R's versioned discriminator). Assert the exact canonical
  observation. `ReleaseFailed` construction, ABI encode/decode + field
  preservation, closed-after-error, and no-retry are exercised with the **private
  injected close result** (the caller-control label) — **not** claimed as an
  observed linked-artifact OS-close failure, and **no** env/TLS/script carrier is
  added across `exec` to force one (real OS-close failure is nondeterministic).
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
- **AC8 — substrate byte-for-byte untouched, CI-green.** PX7-R's `ResourceTableV1`
  / generation discipline / token / invalidation-before-close / no-retry / close
  ownership / native enforcement are **byte-for-byte unchanged** (`git grep` shows
  no re-implementation); the ONLY `crates/**` movement beyond the new surface +
  emitter is the authorized additive `REPLY_RESOURCE_ERROR` projection (AC9);
  interpreter/native differential stays green. **No-regression = green in CI**,
  never a local `--workspace` run.
- **AC9 — additive `REPLY_RESOURCE_ERROR` ABI projection, closed + fail-closed.**
  A DISTINCT generated reply tag carries `ResourceErrorReplyV1` with `detail` =
  exactly the resource-error discriminator; the existing `REPLY_ERROR/detail 6`
  (`io.InvalidInput`) semantics are **unchanged** and never reinterpreted as a
  resource error. The payload is always fully initialized per variant
  (`Closed`/`MalformedResource` all-zero; `RightNotHeld` schema + required/held;
  `ReleaseFailed` schema + kind + identity + packed `IoErrorIdentityV1`, zero
  required/held) and **fails closed** on unknown discriminator / wrong schema /
  unknown kind / invalid I/O tag / out-of-range rights / noncanonical nonzero
  unused fields. The C probe and Rust record agree on size/alignment/every offset,
  and the catalog, generated binding registry, `HostEffectWireLayoutV1`, ABI hash,
  observer, Cranelift consumer, compiler symbol resolution, and mutation tests all
  move together (assert the Cranelift accept-set per op). `ResourceTraceIdentity`
  preserves all 64 bits end-to-end.

## Do-not-reopen guards

- Do NOT re-implement or modify PX7-R's runtime substrate
  (`ResourceTableV1`/generation/token/invalidation-before-close/no-retry/close
  ownership) — consume it through its public boundary; it stays byte-for-byte
  untouched. The ONLY authorized exception is the closed additive
  `REPLY_RESOURCE_ERROR` ABI/projection mapping (Architect-ruled
  `evt_648wsvp2w33yg`): the reply tag + `ResourceErrorReplyV1` payload + its
  coordinated ABI-surface files (catalog / binding registry /
  `HostEffectWireLayoutV1` / ABI hash / observer / Cranelift consumer / symbol
  resolution / C-probe + mutation tests). Nothing else in `crates/ken-host` moves.
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
  ResourceRelease}` / `ResourceErrorV1::{Closed, MalformedResource,
  RightNotHeld{required,held}, ReleaseFailed{schema_version, resource_kind,
  identity, io}}` (`crates/ken-host/src/effect_v1.rs:465`) / the versioned
  observation discriminator — all in `crates/ken-host/src/effect_v1.rs` +
  `abi_v1.rs` as landed by PX7-R (re-ground exact lines on the merged PX7-R head).
- ABI projection site (extend — the ONLY authorized substrate-adjacent edit):
  `crates/ken-host/src/abi_v1.rs:869–879` currently collapses every
  `SemanticErrorV1::Resource(_)` to `REPLY_ERROR/detail 6`; add the distinct
  `REPLY_RESOURCE_ERROR` tag + `ResourceErrorReplyV1` payload here + the
  coordinated ABI-surface files (catalog / generated binding registry /
  `HostEffectWireLayoutV1` / ABI hash / observer / Cranelift consumer / compiler
  symbol resolution / C-probe + mutation tests). Leave `detail 6` =
  `io.InvalidInput`.
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

Touches `crates/**` (Foundation surface + emitter + the authorized additive
`REPLY_RESOURCE_ERROR` ABI/projection files, substrate untouched) + the pinned
`spec/` + `conformance/` route for `ResourceLifetimeObligationV1` → **Architect §14
+ CV** (the spec/conformance touch puts CV in the route). One branch
(`wp/px7f-system-resource-bracket`), one Decision. **Gated on PX7-R merge + the
Spec schema pin** — do not kick until both hold. Full locked CI + conformance run
on GitHub, not locally.
