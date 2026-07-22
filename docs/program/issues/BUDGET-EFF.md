---
id: BUDGET-EFF
title: TransferCount.remaining must be bounded by the effective request
status: active
owner: verify
size: M
gate: none
depends_on: [SPEC-38-ERRATUM]
blocks: [SEAL-2]
github: null
origin: evt_1s9rt48z7bpsn
---

Adversary-confirmed (finding R1) violation of **locked**
`spec/30-surface/38-ffi-io.md`: `TransferCount.remaining` must be bounded by
the *effective* request, but the host clamps instead of rejecting, and
validates against the wrong bound. Fail-closed — not memory-unsafe, not a
forgery, not a parity bug: wrong value, right memory. Identified by **source inspection** of the two reifiers — ⛔ **NOT** confirmed
by execution; the earlier claim to that effect was false
(`adversary/R1-effective-request-repro @ 06bb9538` fails at `e892777c`, but its
conclusion never reads a reifier field).
⚠ **That oracle's final assertion is itself broken** — it compares constants
and never reads a reifier field, so it fails regardless of implementation.
**AC-3 is rewritten**: the oracle must be re-derived to observe the mechanism,
NOT passed unchanged. See the brief's AC-3.

This is a **plumbing gap, not a formula fix**: `effective` is discarded at
validation and reaches neither reifier, so two closures see different blast
radii. Prioritized ahead of `SEAL-2` — SEAL-2 closes a gate with no live
defect, this is a live contradiction of locked normative text.

## ✅ BOTH BLOCKERS CLEARED — ready to release

- **`SPEC-38-ERRATUM` MERGED** (`origin/main @ e5a400c7`, PR #827, retros in).
  `spec/30-surface/38-ffi-io.md` is now self-consistent: `TransferCount` is
  bounded by the **effective** request, and *"the raw caller-requested length is
  not a progress bound after capping."*
- **Architect design ruling DELIVERED and BINDING** — Decision
  `dec_1m6xdwjp2ttyn` resolved, `evt_1g6j2p7jnwbfb`. **Option 1**, refined as an
  inseparable validated carrier. Six boundary constraints + the load-bearing
  **capped-short** oracle case. **Transcribed in the brief — read it there, it
  is the authority.**

**Owner: Runtime.** ⛔ **Sequenced AFTER `RT-SPLIT`** (operator, dev
efficiency: this WP edits `cranelift_backend.rs:13081-13082`, so the
decomposition lands first). ⛔ **Sequenced BEFORE `ABI-M1`** (Architect
constraint 6).

Full brief: [`docs/program/wp/BUDGET-EFF-remaining-bounded-by-effective-request.md`](../wp/BUDGET-EFF-remaining-bounded-by-effective-request.md).

## ⚠ HALF LANDED — host/interp MERGED, native half STILL OPEN

**`origin/main @ 214bf4de` (PR #863, CI-gated).** Decision `dec_mpr5xcbrrnkf`
resolved; Architect APPROVE on exact `99076768`, QA APPROVE from verify-qa on
the whole assembled and rebased diff. Verified by content on `main`, not by the
publisher's exit code.

Landed in `crates/ken-host` + `crates/ken-interp`: the two-field inseparable
`TransferCountV1 { transferred, effective_request }` carrier with constructor
invariant `0 < transferred <= effective_request`; the named `effective_request`
reply field appended last (no offset shift); and post-decode correlation
`validate_transfer_request_bound` proving the full chain
`0 < count <= effective_request <= raw request.length`.

⛔ **The native half is NOT in this merge and the issue stays `active`.** The
native reifier is in `crates/ken-runtime/src/cranelift_backend/lowering/`,
inside the in-flight RT-SPLIT slice-7 tree; a concurrent logic edit there would
break that slice's move-pure ordered-identity criterion.

**⇒ `main` currently carries an intentional asymmetry: host/interp derive
`remaining` from the effective request, the native path still derives it from
the raw length.** That divergence closes when slice 7 lands and the deferred
half is released to Verify. It is sequenced, not forgotten — but it is live on
`main` and anyone reading only the host side will draw the wrong conclusion
about what the runtime as a whole guarantees today.
