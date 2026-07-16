# Resource-lifetime obligation conformance — seed cases (PX7-T)

Format: `../../README.md`. These cases pin the additive
`ResourceLifetimeObligationV1` body carried by the behavioral export's `T`
channel. The body expresses one correlated resource lifetime for Ward to
monitor. It does not make resources affine in Ken, discharge the obligation,
or promote a Ward result to `proved`.

The contract is Spec-owned and is the oracle for PX7-F's emitter. Every case
that requires that emitter is **RED-UNTIL-PX7-F**. The landed PX7-R producer
inventory and trace carrier are already real; this seed derives its event and
identity fields from those producers rather than inventing a future source
surface.

## Producer grounding and locked schema

The landed producer chain is:

- `HostOpV1::{FsOpen, FsHandleMetadata, ResourceRelease}` in
  `crates/ken-host/src/effect_v1.rs` is the exact V1 operation set;
- `EffectEventV1.resource : Option<ResourceTraceIdentityV1>` is the common
  lifetime-identity carrier on all three events;
- a successful `FsOpen` returns `CanonicalReplyV1::ResourceAcquired` with the
  same `ResourceTraceIdentityV1`;
- `ResourceRelease` returns a `ResourceSettlementObservationV1`, or a
  `ReleaseFailed` error, retaining that same identity and the settlement
  outcome; and
- PX7-R's real resource lane observes the same acquisition-order identity on
  `FsOpen`, metadata use, and release. The identity is deliberately independent
  of fd, slot/generation, inode, pointer, and executor provenance.

The conformance oracle uses this locked conceptual shape. Field names below are
schema names, not guesses at a PX7-F source spelling:

```text
T entry:
  obligation_id : String
  status        : delegated
  body          : ResourceLifetimeObligationV1

ResourceLifetimeObligationV1:
  schema_version  : 1
  operations      : ResourceLifetimeOperationsV1
  correlation     : ResourceLifetimeCorrelationV1
  monitor_template: ResourceLifetimeMonitorV1

ResourceLifetimeOperationsV1:
  acquire : FsOpen
  use     : [FsHandleMetadata]
  settle  : ResourceRelease

ResourceLifetimeCorrelationV1:
  identity_type  : ResourceTraceIdentityV1
  event_field    : EffectEventV1.resource
  bind_at        : FsOpen
  require_same_at: [FsHandleMetadata, ResourceRelease]

ResourceLifetimeMonitorV1:
  exactly_one_terminal_settlement_per_successful_acquire
  no_successful_use_after_settlement
  settle_on_supported_exit(return | returned_error | controlled_trap)
  retain_settlement_outcome(including ReleaseFailed)
```

`correlation` is a binder template, not a concrete identity serialized before
execution. A successful `FsOpen` binds one runtime identity `r`; every matching
metadata-use and settlement event is selected by `event.resource = r`. This is
one correlated obligation. Two independent event predicates mentioning
`FsOpen` and `ResourceRelease` are not an alternative encoding.

The whole `T` entry, including the resource-lifetime body, participates in the
existing canonical export hash. Changing an operation, correlation rule, or
monitor property changes the hash. Literal outer wire-key spellings remain
governed by the existing behavioral-export contract; this seed locks the field
set and values above, not a second out-of-band serialization.

## RL-A. Conforming correlated obligation

### resource-lifetime/correlated-body-validates

- status: **RED-UNTIL-PX7-F**
- spec: PX7-T `ResourceLifetimeObligationV1`; `71 §2.1/§3.3/§5.1`;
  `73 §2.4/§2.6`; ADR 0021, "Expressibility prerequisite"
- given: a checked program whose reachable `Σ` contains `FsOpen`; run the real
  PX7-F export emitter and obtain its single resource-lifetime `T` entry
- expect: the entry validates against the schema above and has:
  `status = delegated`; operations exactly
  `[FsOpen, FsHandleMetadata, ResourceRelease]`; one correlation binder over
  `EffectEventV1.resource : ResourceTraceIdentityV1`; and the four-property Ward
  monitor template
- expect: for a runtime instance with acquisition identity `r`, the monitor
  matches metadata-use and release only when their `event.resource` is the same
  `r`; a release outcome, including `ReleaseFailed`, remains observable
- why: this is the positive arm of the correlation discriminator. It proves the
  emitter produced a single identity-bound obligation, not merely that all
  three operation names occur somewhere in `T`.

## RL-B. Uncorrelated lookalike is rejected

### resource-lifetime/independent-event-atoms-do-not-validate

- status: **RED-UNTIL-PX7-F**
- spec: PX7-T `ResourceLifetimeObligationV1`; ADR 0021, "Expressibility
  prerequisite"
- given: the same obligation id, `delegated` status, exact three operation
  names, and exact four monitor-property names as RL-A, but replace the single
  correlation binder with independent event atoms:

```text
event(FsOpen)
event(FsHandleMetadata)
event(ResourceRelease)
```

- expect: the Spec-oracle route rejects the body during schema validation; no
  `T` entry and no hash are emitted for the malformed body. PX7-T does not pin
  a diagnostic spelling for this pre-emission route
- why: this is the negative arm. A validator that checks only operation-set
  membership and monitor-property presence accepts both RL-A and RL-B. The
  correct validator accepts RL-A and rejects RL-B because the lookalike cannot
  state that the settled resource is the acquired resource. The verdict
  therefore flips on correlation alone.

## RL-C. Wrong identity does not satisfy the template

### resource-lifetime/different-settlement-identity-does-not-match

- status: **RED-UNTIL-PX7-F + Ward monitor consumer**
- spec: PX7-T `ResourceLifetimeObligationV1`; `73 §2.2/§2.4`
- given: one conforming RL-A template and two runtime traces that differ only
  in the release identity:
  - positive: `FsOpen(r1); FsHandleMetadata(r1); ResourceRelease(r1)`;
  - negative: `FsOpen(r1); FsHandleMetadata(r1); ResourceRelease(r2)` where
    `r1 != r2`
- expect: the positive trace matches one lifetime and reaches one terminal
  settlement; the negative trace does not settle `r1` and does not satisfy the
  obligation
- why: a constant/global key or an operation-name-only monitor accepts both.
  Holding operation order and every non-identity field fixed makes identity
  correlation the sole discriminating variable. This consumer check is staged
  to Ward; RL-A/RL-B are the build-now PX7-F emitter/schema gate.

## RL-D. Exact landed operation inventory

### resource-lifetime/v1-operation-set-is-closed

- status: **RED-UNTIL-PX7-F**
- spec: PX7-T `ResourceLifetimeObligationV1`; ADR 0021, "Host catalog and
  fail-visible errors"
- given: the resource-lifetime body emitted for the PX7-F bracket
- expect: `acquire = FsOpen`, `use = [FsHandleMetadata]`, and
  `settle = ResourceRelease`, exactly; no generic `ResourceAcquire`, `FsClose`,
  read/write/seek, or second resource-kind operation appears
- why: the oracle follows PX7-R's landed closed inventory. An imagined future
  operation makes the schema unproducible by the prerequisite it claims to
  describe.

## RL-E. Delegated, content-hashed, and one-way

### resource-lifetime/body-participates-in-T-hash

- status: **RED-UNTIL-PX7-F**
- spec: `71 §3.3/§5.1`; `73 §2.5/§2.6`; ADR 0021
- given: emit RL-A twice, then change only one locked body field (for example,
  remove `FsHandleMetadata` from `use`) and emit again
- expect: the identical body yields the identical export hash; the changed body
  yields a different hash. The body is hashed inside `T`, with no out-of-band
  resource-lifetime field
- expect: status remains `delegated`; a Ward accept/reject result cannot write
  `proved`, cannot move the entry to `Q`, and cannot enter kernel or runtime
  control flow
- why: the first pair rejects a non-canonical hash; the changed-body arm rejects
  a constant hash or a hash that omits the new body. The one-way assertion
  preserves the existing G-Ward seam.

## RL-F. Existing temporal route remains byte-for-byte semantic

### resource-lifetime/ordinary-temporal-entry-is-unchanged

- status: **RED-UNTIL-PX7-F**
- spec: `71 §2.1/§3.3`; `72 §5`; PX7-T additive boundary
- given: an ordinary existing `temporal { ... }` declaration that does not
  express a resource lifetime, emitted before and after enabling the PX7-T
  route
- expect: it remains the existing `TEntry { formula: Temporal }`, with the same
  `delegated` status, formula projection, canonical representation, and hash;
  it is not rewritten as `ResourceLifetimeObligationV1`
- why: PX7-T supersedes `TemporalObligation` only where acquire/use/settle
  identity correlation is required. A broad replacement weakens or re-spells
  the landed temporal route and fails this control.

## Coverage map

| PX7-T acceptance criterion | Cases |
|---|---|
| AC1 locked fields, delegated status, Ward template, T hash | RL-A, RL-E |
| AC2 one correlated obligation, not independent atoms | RL-A, RL-B, RL-C |
| AC3 positive validates / malformed or uncorrelated rejects | RL-A, RL-B |
| AC4 additive; existing temporal machinery unchanged | RL-F |
| AC5 exact landed V1 operation set | RL-D |

## Cross-case and reachability sweep

- **The build-now verdict flip is non-vacuous.** RL-A and RL-B hold the id,
  status, operations, and monitor-property set fixed. Only the structural
  correlation binder changes. Correct route: accept/reject. Operation-name-only
  route: accept/accept.
- **The runtime identity flip is isolated.** RL-C holds operations, order, and
  template fixed and varies only `r1` versus `r2`. It is explicitly staged to
  the Ward monitor consumer rather than presented as PX7-F emitter evidence.
- **Producer reachability is explicit.** PX7-R already produces all three
  `EffectEventV1` operations with one `ResourceTraceIdentityV1`. PX7-F is the
  named gate for creating the `T` body and schema validator. Ward is the named
  gate for executing the monitor template. No case claims an unbuilt producer
  is live.
- **No synthetic PX7-F emitter is accepted as evidence.** The build gate must
  drive a checked Ken program through the real PX7-F export emitter. A unit test
  that constructs `ResourceLifetimeObligationV1` directly is only a schema-unit
  control and cannot discharge RL-A, RL-D, RL-E, or RL-F.
- **Existing routes are preserved.** Ordinary `Temporal` export coverage stays
  in `../temporal/seed-temporal.md`; trace projection and the G-Ward one-way gate
  stay in `../trace/seed-trace.md`. This seed adds only the correlated lifetime
  body and references, rather than duplicating those corpora.
