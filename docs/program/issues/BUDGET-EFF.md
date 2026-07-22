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

## ⚠ NATIVE HALF — BINDING ACCEPTANCE, folded from the adversary (`evt_68dzrj94erjz`)

**Released by RT-SPLIT slice 7 landing.** Owner: **Verify**. The three items
below are **inputs to the WP, not defect reports** — filed before the ring
started. Each independently re-derived on `origin/main` by the Steward rather
than accepted on report.

### The divergence, exactly

```
host    effect_v1.rs:1746    effective = requested.min(capacity - start)
host    effect_v1.rs:1654    TransferCountV1::new(read, effective)   <- mints the clamped value
interp  eval.rs:4943/:4965   remaining = effective - count            ✅
native  lowering/mod.rs:4700 remaining = isub(request_length, count)  ❌ raw length
```

**Repro, no build needed:** capacity 4, start 0, `FsReadAt` length 8, transfer 2.
`effective = min(8,4) = 4`. Interp → `remaining = 4-2 = 2` (asserted verbatim at
`eval.rs:6243`). Native → `remaining = 8-2 = 6`. **Same Ken program, two
engines, two observable `remaining` values — and `remaining` is a field the
program pattern-matches.**

### ★ 1 — the word "narrowed" is a trap sitting on the fix site

`core.rs:5081`/`:5119` hand the value over as
`.expect("positioned request bounds were narrowed before dispatch")`.
**"Narrowed" here means `narrow_native_int_u64` — an Int→u64 *width* cast
(`mod.rs:4471`). It is not the budget clamp.** `positioned_bounds` comes from
`core.rs:4605`, operand 4 — the **raw** length, never `min`'d against capacity.

⇒ An implementer landing on `mod.rs:4700` reads that message and reasonably
concludes the bounds are already effective. **The one comment adjacent to the
defect argues the defect isn't there.**

### ★ 2 — the normative comment already claims this is done, in the present tense

`effect_v1.rs:2081-2083`, on `effective_request()`:

> `remaining` (**both reifiers**) is `effective_request - get()` — never derived
> from the raw pre-clamp request length.

**"Both reifiers" is false on `main` today.** This is the *claim-in-two-places*
shape: the doc took the post-fix wording while one of its two subjects stayed
unfixed, so **the artifact asserts its own completion.**

⛔ **REQUIRED in the same candidate:** either that sentence becomes true, or it
names the pending reifier. It must not stay as-is.

### ★ 3 — nothing would catch it, and that is structural

```
interp   4 × budget_eff_capped_* tests (eval.rs:6091,6175,6250,…)
native   grep capped|effective_request|budget_eff in ken-runtime/src  ->  ZERO
```

The native counterpart suite is `ken-cli/tests/rt_parity_native.rs`, and per
`CI-SKIPPED-NATIVE-TESTS` it is **still skipped in CI**.

⇒ **A green CI on this WP carries no information about whether `remaining` was
fixed** — no test asserts it, and the binary that would host one does not run.

### ⛔ The binding acceptance criterion

**Assert the numbers, not the diff.** A native test at capacity 4 / length 8 /
transfer 2 expecting `remaining == 2`, and **demonstrate it fails before the
fix.**

★ The fail-first is **load-bearing, not stylistic**, precisely because the suite
hosting it may not run in CI. **If `rt_parity_native.rs` cannot be restored in
this window, the fail-first must be demonstrated locally and reported as such**
— otherwise the criterion is unfalsifiable and the WP closes green on an axis
that was never observed.

**Severity: correctness / engine-divergence. Deliberate and disclosed — not an
incident, not urgent.** The delta is that the guard rails around it currently
all point the wrong way.
