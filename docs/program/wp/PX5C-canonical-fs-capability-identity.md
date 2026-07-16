# PX5C — canonical FS capability trace identity (Runtime producer erratum)

- **ID:** PX5C · **Owner:** Team Runtime · **Size:** S · **Risk:** Low
  (unifies one canonical trace-label seed across the two capability-table grant
  constructors; behaviorally inert beyond the label; no evaluation, authorization,
  or op-catalog change).
- **Objective:** Make the canonical FS **`CapabilityTraceIdentity`** identical
  across the interpreter and native lanes, sourced from a single canonical owner,
  so PX6's differential harness compares whole observations by exact equality with
  **no comparator normalization**. Today the two producers seed
  **structurally-different substrate labels** — interp `"interpreter:FS"`
  (`crates/ken-interp/src/eval.rs:4253`), native `"declared:FS"`
  (`crates/ken-host/src/abi_v1.rs:465`) — so every real FS op diverges on identity
  alone while matching on request bytes, outcome, deltas, terminal state,
  stdout/stderr, and exit. This unifies both to the exact family key **`FS`**.
- **Depends on:** PX5 (`origin/main @ 049628f8`) + PX5B (`c4200a84`), both merged —
  the canonical observation types and both grant constructors already exist.
- **Blocks:** **PX6** (Verify). PX6 holds uncommitted mechanical
  PX5B-consumption/assertion-only work on its rebased branch and resumes
  **unchanged** once this erratum merges.
- **Gate it feeds:** G-Ward-seam / native-effect lane (with PX6).

## Fixed inputs — DO NOT REOPEN (cite, do not relitigate)

The mechanism is **pinned by the Architect ruling `evt_5gpv47r3pdj2`** (thread
`thr_74tys5r5e39s4`, PX6). It is a clarification of the **accepted** ADR-0018 §4
contract, **not** an ADR reopen. Fixed inputs:

- **ADR-0018 §4 already defines** `CapabilityTraceIdentity` as the **stable
  declared `ProgramCaps` field identity — not a credential, not executor
  provenance.** Both landed values violate that meaning in opposite directions:
  `interpreter:FS` injects the executor substrate; `declared:FS` injects a
  producer-local provenance prefix absent from the declaration.
- **The canonical identity for v1's sole declared FS capability is the exact byte
  string `FS`** — the family key present in `capabilities FS <authority>`, shared
  by both executions. It is **NOT** the erased `Cap` type symbol, a token/table
  identity, a source binder (`fsCap`), or an executor label. **Do not choose any
  other spelling.**
- **`dispatch_host_op_v1` remains the sole owner** that copies the resolved grant
  identity into `HostDispatchReplyV1`; **both** observation producers continue to
  copy the reply-owned identity **verbatim**. Reply-sourcing discipline is
  unchanged — this WP corrects only the *seed* at grant construction, never the
  copy path.
- **Verify performs NO normalization** and keeps exact whole-observation equality.
  This WP does not touch the comparator, the twin-root harness, or `expected_fs`
  demotion (all PX6).
- **Option 2 (lane-local comparison) and Option 3 (new canonical field) are
  rejected/deferred** by the ruling — do not implement either. Substrate
  provenance, if ever needed, is out-of-band diagnostic metadata under a
  separately-named noncanonical field, not this canonical one.
- Trust: producer instrumentation for a tested/target-validated harness — **zero
  kernel rule, zero new Ken postulate.** It is a trusted-base (host/interpreter)
  change, so it takes the **§14 Architect** soundness gate. **CV is not in route**
  unless the repair touches `spec/`/`conformance/` (it should not; the ADR-0018 §4
  clarification is a `docs/adr` edit).

## Scope

**In scope (PX5C owns):**
1. A canonical helper in **`ken-host`** (the owner of `CapabilityTraceIdentity`):
   ```rust
   pub const PROGRAM_CAPS_FS_TRACE_IDENTITY_V1: &str = "FS";
   pub fn program_caps_fs_trace_identity_v1() -> CapabilityTraceIdentity {
       CapabilityTraceIdentity(PROGRAM_CAPS_FS_TRACE_IDENTITY_V1.to_string())
   }
   ```
2. Both capability-table grant constructors call that helper instead of a literal:
   - `crates/ken-interp/src/eval.rs` at the FS grant construction (was
     `"interpreter:FS"`, line ~4253);
   - `crates/ken-host/src/abi_v1.rs` at native `ProcessContext` initialization
     (was `"declared:FS"`, line ~465).
3. The four discriminator tests below.
4. A one-sentence **ADR-0018 §4** clarification (see AC5).

**Out of scope:** the PX6 comparator / twin-root harness / `expected_fs` demotion
(Verify, held, resumes unchanged); evaluation semantics; path authorization /
denial handling; the sealed op catalog; ambient Console/Clock identity (stays
`None` — do not add a synthetic identity); native production logic beyond the
identity seed.

## Mandated deliverable outline — each section ends in a concrete choice

1. **Canonical owner.** Add the const + helper to `ken-host` beside
   `CapabilityTraceIdentity`. **Concrete choice:** exactly the two items in scope
   §1, byte string `"FS"`, no feature flag, no per-op variance (v1 has one FS
   family).
2. **Interp seed.** Replace the `"interpreter:FS"` literal at the interp FS grant
   construction with `ken_host::program_caps_fs_trace_identity_v1()`. **Concrete
   choice:** call the helper; do not inline the string; touch nothing else in the
   dispatch/reify path.
3. **Native seed.** Replace the `"declared:FS"` literal at native `ProcessContext`
   init with the same helper. **Concrete choice:** call the helper; leave the
   `CapabilityGrantV1`/table shape and dispatch untouched.
4. **ADR clarification.** Add one sentence to ADR-0018 §4 (see AC5).

## Acceptance criteria (testable)

- **AC1 — cross-lane equality.** The same successful FS operation run through both
  real lanes yields reply/event `capability_identity == Some("FS")` and the two
  assembled observations compare **exactly equal** on identity.
- **AC2 — denial identity preserved.** Malformed/missing/wrong token in **each**
  lane still yields reply/event `capability_identity == None` **plus** exact
  `MalformedCapability`, recorded **before** any host leaf. (The unification must
  not weaken the intra-lane reply-sourced denial discriminator.)
- **AC3 — drift control (proves producer-sourced, not comparator-normalized).**
  A test that replaces **either** producer seed with `"interpreter:FS"`,
  `"declared:FS"`, or any other string **fails** the cross-lane comparison. This
  demonstrates equality comes from **canonical producer construction**, not from
  any comparator normalization.
- **AC4 — ambient unchanged.** Ambient Console/Clock dispatch identity remains
  `None`; no synthetic identity is introduced.
- **AC5 — ADR clarified.** ADR-0018 §4 carries the sentence: *For v1's sole FS
  `ProgramCaps` field, the canonical trace identity is the exact byte string
  `FS`; executor/substrate prefixes are forbidden.*
- **AC6 — inert + honest + green.** No evaluation-semantics, authorization,
  op-catalog, `/spec`, or `/conformance` change; targeted gates green locally
  (`scripts/ken-cargo test -p ken-host -p ken-interp` and any directly-touched
  crate; `-p ken-cli` if the checked-source producer path is exercised); full
  no-regression is **CI-owned** (never a local `--workspace` run).

## Do-not-reopen guard

The identity spelling (`FS`), the single-owner helper location (`ken-host`), the
rejection of Options 2/3, and the no-normalization-in-Verify posture are **settled
by the Architect ruling** `evt_5gpv47r3pdj2`. Do not relitigate them, do not add a
comparator exception, do not introduce a second canonical field. If building
surfaces a genuine *new* fork (e.g. a second capability family needs a distinct
key), stop and route it to the Steward — do not decide it in-line.

## Flow

Targeted gates only (⛔ never `--workspace`; no-regression = green in CI). Hand
back the **exact SHA** with per-AC evidence + targeted gate output. Route:
runtime-qa gate → §14 (Architect soundness — trusted-base; **CV not in route**) →
publisher path (CI) → Steward content-verify + close. On merge, the Steward
routes the unpark to Verify; PX6 resumes its held work **unchanged**.
