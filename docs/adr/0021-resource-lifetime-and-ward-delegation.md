# ADR 0021 — Runtime-enforced resource lifetime and Ward delegation

- **Status:** Accepted.
- **Date:** 2026-07-16.
- **Deciders:** Architect, with the operator's settled strategy for resources
  that need affine reasoning (do the affine enforcement in Rust, lift a
  reasonable interface to Ken, discharge the exactly-once-release obligation to
  Ward).
- **Relates to:** ADR 0017 (scoped-capability trust boundary), ADR 0006
  (behavioral-assurance complement / Ward attestation), ADR 0018 (native-effect
  execution contract), ADR 0019 (capability evolution and process admission),
  PX7-R (runtime substrate) and PX7-F (Ken surface, implementation).

## Context

Tool authors need first-class **resources** — objects that are dynamically
acquired, held across effect steps, used, and then released, with a destructor
that can fail. A file opened for a sequence of reads is the motivating case. The
landed capability model does not express this: capabilities are durable program
grants, and the filesystem model is path-based and stateless (every op
re-resolves from the root handle and drops the transient fd; there is no
`FsOpen`/`FsClose`, no held-across-steps handle, no use-after-close path).

The obvious way to make "release exactly once, never use after release" safe is
an affine/linear type discipline in Ken. **That path is barred.** The operator's
standing ruling is verbatim: *"Until CS research shows a proven path, Ken will
not have affine types."* Ken introduces no affine or linear types, and no
feature that quietly depends on them.

The operator's settled strategy for exactly this situation is: resources that
would need affine reasoning are served by doing the **affine enforcement in
Rust** — Rust's ownership already gives affine move semantics — **lifting a
reasonable interface to Ken**, and **discharging the exactly-once / no-leak
obligation to Ward**, the sibling behavioral verifier, to be checked
post-build. The guarantee is real (Rust affinity plus a Ward-checked
obligation) but it is **never a Ken-level affine claim and never a kernel
`proved`**.

Ward is reachable only as a one-way, data-only export: obligations are emitted
as inert `Temporal` values into the assumption-boundary `T` channel
(`crates/ken-elaborator/src/export.rs`; `crates/ken-elaborator/src/temporal.rs`
`TemporalObligation` "delegates discharge to Ward"), the status is the pinned
constant `delegated`, and the verdict returns only as a signed, export-hash-
bound epistemic-status attestation. There is no runtime obligation-discharge
hook, and no path by which a Ward verdict enters `Q`, the trusted base, or
runtime control flow.

## Decision

Ken gains a first-class opaque **resource handle** whose lifetime is
**dynamically bracket-bounded**, enforced by the runtime in Rust, and whose
exactly-once/no-leak property is delegated to Ward. It is neither Ken-affine nor
kernel-proved.

### Lifetime — hybrid, dynamically bracket-bounded

`System.Resource.withResource acquire body` is the **sole public acquisition
route** in V1. The acquired `Resource k` is an ordinary, copyable Ken value that
may pass through arbitrary computations inside `body`; there is no Ken affine
fiction. Its *liveness* ends when the bracket settles. A handle copied or
returned so that it escapes the bracket is legal Ken syntactically, but stale
operationally: every later use returns `Closed`. The surface states this
directly; it does not claim the type prevents escape.

An explicit early `release` inside the body is permitted; it invalidates all
copies. The bracket owns a private `release_if_live` finalizer, so an early
release followed by bracket exit performs no second OS close. Public `release`
is non-idempotent: a second call returns `Closed`. The raw acquire/release
effect protocol is private substrate and is **not** exported as a general Ken
operation.

Normal return, a returned error, and a **controlled Ken trap** all run
settlement. External kill, abort, fatal signal, and machine failure are outside
the guarantee and are named as such. A Ken trap must reach the runtime as a
controlled terminal outcome; an aborting trap is not an acceptable behavior.

### Representation — sibling `ResourceTableV1`

Resources use a separate `ResourceTableV1`, not the landed `CapabilityTableV1`.
Capabilities are durable grants; resources are dynamically created, revocable
objects with destructors, kind, attenuated rights, and close outcomes. Mixing
them would change `MalformedCapability` semantics and make grant identity double
as object lifetime.

The Ken-facing token is opaque `ResourceTokenV1 { slot, generation }`; neither
field nor constructor is Ken-visible. A live slot owns exactly one Rust resource
and records at least its generation, resource kind, the owned backend object,
the attenuated rights/context inherited at acquisition, and a canonical trace
identity distinct from the token. The owned backend object is a **distinct,
unique, non-cloneable** owner — e.g. `ResourceHandleV1(OwnedFd)` with no `Clone`
and no `Arc` — **not** PX16's cloneable `Handle(Arc<OwnedFd>)`. Correctness never
depends on `Arc::strong_count` or `try_unwrap`; the existing rooted/path handles
are unchanged.

On release, the owned object is moved out and the generation is bumped /
invalidated **before** close is invoked. The token is closed whether close
reports success or failure; a raw descriptor is never retried after a close
error. Slot reuse is allowed only at the bumped generation. On generation wrap
the slot is retired permanently rather than reissued at an old identity. A token
for a retired generation yields `Closed`; a zero / out-of-range / never-minted
encoding yields `MalformedResource`. A stale token can never resolve a reused
slot or a recycled fd. `ResourceKindV1` has a single V1 variant (`FsHandle`), so
a live wrong-kind state is not reachable and PX7-R defines **no**
`ResourceKindMismatch`: the resolver verifies validity, liveness, and attenuated
rights, and the Fs-handle owner is established by construction. The mismatch
identity is deferred to the first WP that adds a genuinely different production
resource kind (see the fail-visible-identity roster below).

Canonical observations and Ward pairing use a lane-independent
`ResourceTraceIdentityV1` minted from deterministic acquisition order (the
successful acquire event identity is sufficient) — **never** an fd,
slot/generation, pointer, inode, or executor provenance.

### Rust enforces; Ward checks the stated behavior

Rust is the live enforcement net: one live slot owns one Rust resource; every
use checks slot, generation, kind, and stored rights; a user release consumes
that live owner at most once; explicit finalization runs on controlled return,
error, and trap paths; table/context RAII is the last-resort leak backstop; and
stale use or release is fail-visible `Closed`. `Drop` alone is **not** used for
the normal contract, because `Drop` cannot report a close failure: controlled
exits call an explicit finalizer, record every result, and let `Drop` cover only
catastrophic unwinding.

**Close-failure boundary — one confined `try_close` seam.** A real production OS
close failure cannot be observed through the safe `OwnedFd` boundary: `OwnedFd`'s
`Drop` discards the close result, and rustix's only error-reporting close is the
feature-gated `unsafe fn try_close(RawFd) -> Result<()>` (which specifies the raw
fd is invalid after the call, even on error). A mock failure or a `sync_all`
flush cannot substitute for that production mechanism. PX7 therefore adds exactly
**one** private, Linux-only, module-local `#![allow(unsafe_code)]` allowance (e.g.
`ken-host::resource_close_v1`) — the crate stays `#![deny(unsafe_code)]`
everywhere else. Its safe facade consumes the unique owner via `IntoRawFd` and
performs the sole `rustix::io::try_close` call, returning the close result. Its
safety invariant is local and checkable: the raw fd came from consuming the sole
owner; no alias, borrower, or in-flight operation survives; the call occurs
exactly once; the fd is treated invalid on every return; no retry is possible —
no `ManuallyDrop`, fabricated raw fd, or second owner. Enabling rustix's
exact-pinned `try_close` feature is an **audited TCB delta** (Cargo-manifest
assertion, dependency identity/feature registry, target-ABI hash, and
mutation/foreign-target controls all move), not a routine feature toggle. The
explicit release path takes the unique owner from the live slot, bumps/retires
the generation and commits the tombstone, then invokes this facade and maps the
result **once** to success or `ReleaseFailed { resource_kind, identity, io }`; the
slot stays closed on both outcomes. Once explicit release consumes the owner into
the raw-fd seam, no `OwnedFd` remains to double-close, and RAII covers only owners
never extracted by explicit settlement (catastrophic unwind / leak backstop).

Ward is not the sole net for host safety or leak prevention. It checks the
exported property over canonical lifecycle observations: (1) every successful
acquire identity has exactly one terminal settlement identity; (2) no successful
use occurs after settlement; (3) a bracket return, returned error, or controlled
trap leaves no live resource acquired by that bracket; and (4) the settlement
outcome, including release failure, remains in the trace. The runtime's
generation discipline supplies the at-most-once half; explicit cleanup supplies
eventual settlement on supported executions; Ward checks that the produced
behavior conforms. Ward's result is a signed, export-hash-bound attestation
only.

**Expressibility prerequisite (Spec-owned).** The landed `TEntry { formula:
Temporal }` plus `Pred::Event(String)` cannot correlate a dynamically minted
acquisition identity with its later settlement; two named atoms `"acquire"` and
`"release"` prove only uncorrelated traffic and are forbidden as a proxy. PX7
therefore requires a narrow, additive, **Spec-owned** `T`-channel body/schema
extension — `ResourceLifetimeObligationV1` — carrying the acquire/use/settle
operation set and an identity-correlation policy. The target emitter generates
exactly one such delegated obligation whenever its reachable `Σ` contains a
resource acquisition; it remains status `delegated`, is content-hashed with `T`,
and is compiled by Ward as a monitor template over runtime resource identities.
This is a schema prerequisite pinned by the Spec enclave — not a kernel rule and
not a Foundation-authored guess.

### Ken surface and authority

`System.Resource` — the first `System.*` namespace — owns the opaque
`Resource k`, the bracket result/error shape, `withResource`, use combinators,
and the optional early `release`. The `body` is a delayed function so that
acquisition precedes it and settlement follows its returned value or error.
`System.Resource` does **not** broaden the FS-only capability declaration grammar
in PX7.

Authority to acquire is the existing domain capability: `FsOpen` checks the
landed FS grant plus the rights required by its requested mode, and the resulting
slot stores only the attenuated subset needed by later handle operations.
Release needs possession of a live handle, not a new authority right.
Consequently PX7 adds **no** `program capabilities Resource …` family (Resource
is a lifetime class, not ambient authority) and spends **no** `RightSet` bit.

The source itself — not merely a Rust comment or the frame — must say that
handles are runtime-enforced and Ward-checked, that Ken does not make them
affine, that escaped copies become `Closed`, and that the guarantee excludes
external process destruction.

### Host catalog and fail-visible errors

PX7 extends the landed closed inventory (generated ABI, wire, observer,
dispatcher, backend, interpreter/native lanes, mutation gates) with, for V1: a
domain-specific `FsOpen` (capability-gated acquisition); a generic
`ResourceRelease`; and at least one **real** non-release consumer (a handle
metadata operation is the natural minimum) so that "use-after-close" is not a
renamed double-close test. A generic authority-free `ResourceAcquire` is **not**
added; acquisition semantics belong to the resource domain.

The fail-visible identities are distinct: `Closed` (known stale / double-release
/ use-after-release); `MalformedResource` (token never minted by this table); the
existing capability/file identities (acquisition denial or failure); and
`ReleaseFailed { resource_kind, identity, io: IoErrorIdentityV1 }` (OS close
failure, without exposing an fd). `ResourceKindMismatch` is **not** a V1 identity:
with a single-variant `ResourceKindV1` it has no production producer at all
(unlike close failure, whose facade returns a real OS error that is merely
nondeterministic). It, its versioned wire discriminator, and a wrong-kind
resolver branch are **deferred** — the first WP that adds a genuinely different
production resource kind atomically adds `ResourceKindMismatch { expected, actual
}`, its canonical wire discriminator plus schema/hash/inventory movement, a real
mint path for both kinds, and a non-degenerate production-reaching pair (mint kind
A then apply a kind-B-only op, and mint kind B then apply a kind-A-only op — both
returning the exact mismatch identities with expected/actual reversed, while valid
same-kind controls succeed). PX8 read/write/seek over `FsHandle` does **not**
trigger this; the trigger is a second real resource kind.

A close error leaves the handle closed and is never retried. If body success is
followed by release failure, the bracket returns the release
failure; if body error and release failure coexist, both are preserved; if a
trap and a cleanup failure coexist, the trap is preserved as primary and the
cleanup failures as an ordered secondary canonical observation, neither
overwritten nor dropped. This requires a versioned observation/wire
discriminator.

## Trust statement

Resource lifetime is **runtime-enforced and discriminator-tested**, and the
exactly-once / no-leak property is **Ward-delegated** and returned only as a
signed attestation. It is **never** a Ken affine/linear guarantee and **never** a
kernel `proved` fact. This ADR adds no postulate, primitive, or affine/linear
claim to the kernel or the language. The runtime's generation discipline is the
at-most-once enforcement; Ward checks the stated exactly-once/settlement
property over canonical observations; neither substitutes for the other.

PX7 introduces **one** new confined `unsafe` delta to observe a real OS close
result: the private `resource_close_v1` module-local `#![allow(unsafe_code)]`
plus rustix's exact-pinned `try_close` feature. This stays within the settled
`rustix`/`linux_raw` boundary (no new dependency; rustix is already vendored and
sanctioned), and it is the smallest mechanism that can state the `ReleaseFailed`
contract honestly. All public dispatch/table APIs remain safe; the crate stays
`#![deny(unsafe_code)]` outside that one module; the pre-existing audited `abi_v1`
unsafe boundary is separate and unchanged. This TCB growth is ratified at the
PX7-R §14 trusted-base-growth Decision; it is runtime-trusted and
discriminator-tested, never kernel-proved.

## Decomposition

The work lands as a lead WP plus a follow WP, never one WP split across two
branches or Decisions:

- **PX7-R (Runtime lead)** owns ADR-0021, `ResourceTableV1`, the opaque wire
  token, the canonical resource identity, `FsOpen` / real use / `ResourceRelease`,
  capability attenuation, generation invalidation, explicit controlled-exit
  cleanup plus the RAII backstop, structured release errors, ABI/hash/inventory,
  native + shared-dispatch differential closure, and the invocation-scoped
  interpreter **state-lifetime seam**. Its low-level protocol is private
  substrate, not the public Ken safety API. PX7-R does **not** deliver a public
  interpreter/native lifecycle differential — the real `System.Resource`/bracket
  constructors are PX7-F, so the first real **public** interpreter/native
  lifecycle + reachable-negative-control equality is **PX7-F** (Architect ruling
  `evt_5f65tm0ymzwbh`).
- **PX7-F (Foundation follow)** owns `System.Resource`, the sole public bracket
  acquisition, delayed-body and settlement sequencing, optional early release,
  the source-level honesty statements, the generated resource-lifetime `T`
  obligation integration, the end-to-end success/error/trap/escape controls, and
  the **first real public interpreter/native lifecycle + negative-control
  equality** — driving checked public Ken source through the real
  `run_io_effect_observation_v1` producer + the linked native artifact for every
  **publicly reachable** V1 operation and outcome (acquire, genuine use, release,
  stale-use/double-release as exposed by the bracket, capability denial,
  attenuation failure, and success/error/trap cleanup).

Before PX7-F builds, the Spec enclave pins the additive structured `T`-body
schema (`ResourceLifetimeObligationV1`) and its conformance route; Foundation
implements that contract rather than inventing it. PX7-R may proceed against the
lifecycle-observation vocabulary while the amendment is being pinned, but PX7-F
cannot merge without it. This preserves the one-ring rule: Runtime lands the
complete substrate, then Foundation consumes a stable public boundary.

## Rejected alternatives

- **Ken-level affine/linear types** for the handle — barred by the operator
  until CS research shows a proven path.
- **Extending `CapabilityTableV1`** to carry resources — corrupts
  `MalformedCapability` and conflates durable grant identity with object
  lifetime.
- **A persistent held-open handle with no bracket** (cross-step liveness with no
  guaranteed settlement) — has no exactly-once/no-leak enforcement point.
- **A scoped-bracket-only handle** that is not a first-class value — needlessly
  forbids passing the handle through ordinary `body` computation.
- **`Drop` alone as the normal close contract** — cannot report a close failure;
  release outcome would be unobservable.
- **Claiming a real `ReleaseFailed` while forbidding any new `unsafe`** (the
  original AC9) — internally contradictory: the safe `OwnedFd` boundary
  structurally discards the close result, so a real OS close errno is
  unobservable without the confined `try_close` seam.
- **A mock/backend-injected failure or a `sync_all` flush as the production
  close-error mechanism** — tests the enum/consumer logic but does not make a real
  OS close failure observable; permitted only as a labelled caller-control test.
- **Manufacturing `EBADF` by double-closing** or otherwise violating the facade
  precondition — the production failure branch must be real (the deployed facade
  returns the OS result).
- **Storing the resource owner as `Arc<OwnedFd>`** (PX16's cloneable `Handle`) or
  deciding correctness on `Arc::strong_count`/`try_unwrap` — the resource owner
  is a distinct, unique, non-cloneable `ResourceHandleV1(OwnedFd)`.
- **Ward as the sole net** for host safety or leak prevention, or a Ward verdict
  entering `Q` / the trusted base / runtime control flow.
- **Two named atoms `"acquire"`/`"release"`** in the landed `Pred::Event` schema
  as the exactly-once proxy — proves only uncorrelated traffic.
- **A new `program capabilities Resource` family or a new `RightSet` bit** for
  acquisition authority — `FsOpen` already carries the domain grant.
- **A token keyed on fd, slot/generation, pointer, inode, or executor
  provenance** for canonical observations — must be a lane-independent
  acquisition-order identity.
