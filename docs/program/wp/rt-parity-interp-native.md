# RT-PARITY — interpreter/native parity erratum (adversary F5 + F6)

**Owner:** Team Runtime · **Size:** M · **Risk:** low-medium (small diff,
high coverage obligation) · **Gate:** none — correctness erratum on a shipped
surface · **Deps:** none (PX8 series landed)

**Origin:** adversary findings F5 (`evt_7njbntfhre2qx`) and F6 (same), both
**CONFIRMED** by the Architect's batch ruling `evt_4e5bqa5tes7nm`, which
directed they be folded into **one** Runtime parity erratum with a conformance
companion.

## 1. Objective

Ken maintains an "interpreter is the reference oracle, native is the trusted
lowering; they must agree" premise. Two seams break it. Both live in the
**per-implementation reify/lowering of the shared host reply** — the one place
interp and native are *not* unified (the core bounds/rights/`NoProgress` logic
is shared through `ken_host::dispatch_host_op_v1` and is consistent by
construction).

Neither is a safety hole; both fail closed. The damage is to the oracle
premise itself: on the affected inputs a differential test would flag *native*
as wrong when native is correct.

## 2. The two defects

**F5 — `ReadSome` reification hardcodes `remaining = 0`.** At
`crates/ken-interp/src/eval.rs:4823`:

```rust
let remaining = buffer_nat_value(0, fs, store)?;
```

The **write** arm in the same function computes it correctly
(`eval.rs:4842`), reading the requested length out of the request:

```rust
let remaining = buffer_nat_value(requested.checked_sub(count).ok_or(())?, fs, store)?;
```

Native does it correctly for both via `mint_validated_progress_nat`
(`crates/ken-runtime/src/cranelift_backend.rs:13078-13082`). **The contract
settles which is right:** `transfer_count_request_budget = transfer_count_nat
+ transfer_count_remaining` (`crates/ken-elaborator/src/prelude.rs:1458`) must
equal the requested window — native satisfies it, interp violates it on every
**positive short read** (`0 < transferred < requested`). Publicly reachable
via `transfer_count_remaining` on a `ReadSome` result.

That one arm computes it right and its sibling does not is the tell: this is
an omission, not a semantic choice.

**F6 — sentinel narrowing lets the wrong error win.** The interpreter converts
out-of-range `Int` arguments to a sentinel and **proceeds into dispatch**,
where resource liveness/kind/rights are checked *before* bounds. Native
narrows via `narrow_native_int_u64` and, on failure, synthesizes the consuming
op's `InvalidOffset`/`InvalidBounds` **in codegen, skipping dispatch
entirely**. So an input that is *both* malformed and hits a resource-state
problem returns a **different public error variant** from each. Both reject;
the observable closed-sum result differs. The Architect ruled this is **not**
accepted implementation latitude — PX8-I requires both that out-of-range
narrowing yields the consuming op's error and that native agrees with the
reference interpreter.

## 3. ★ The sentinel inventory is NOT uniform — and it corrects both sources

The Architect required enumerating **every** landed host-width consumer rather
than patching only the two reported. Grounded at `origin/main @ 244cfe9c`, the
complete inventory is **4 consumers / 9 sentinel conversions**, all in
`crates/ken-interp/src/eval.rs`:

| Consumer | Fields | Sentinel | Lines |
|---|---|---|---|
| `BufferAllocate` | `capacity` | **`0`** | 4588 |
| `FsReadAt` | `file_offset`, `buffer_start`, `length` | `u64::MAX` | 4597, 4600, 4603 |
| `FsWriteAt` | `file_offset`, `buffer_start`, `length` | `u64::MAX` | 4616, 4619, 4622 |
| `BufferFreeze` | `start`, `length` | `u64::MAX` | 4635, 4638 |

**`BufferAllocate` does not use `u64::MAX`. It uses `0`, and that is a
different — arguably worse — hazard class.** Both upstream sources describe
these as one pattern: the adversary wrote "the identical `.unwrap_or(u64::MAX)`
pattern is on `BufferFreeze`/`spanBytes`/`freeze`", and the ruling said
"current sentinels also occur at `BufferAllocate` and `BufferFreeze`/the
`spanBytes` route". Neither had traced it. **They are not the same shape:**

- `u64::MAX` is almost certainly invalid downstream, so it *tends* to fail
  closed — the F6 damage is which error variant surfaces.
- **`0` is a perfectly legitimate capacity.** A malformed capacity is
  therefore **indistinguishable from a lawful zero-capacity request**, and the
  op may **succeed** rather than reject. That is not a variant-divergence
  defect; it is a potential silent-acceptance defect, and it would not be
  caught by any net written for the F6 shape.

⇒ **Do not apply a uniform fix or a uniform test template.** `BufferAllocate`
is classified on its own merits (AC-4) before any shared remedy is assumed.

This is exactly the "verify the property, not the representative case"
discipline (COORDINATION §7): the two reported consumers are the
representative case, and the pattern generalization was wrong at the first
consumer nobody checked.

## 4. Fixed inputs — settled, do not reopen

1. **Fix the interpreter; do NOT weaken native.** The contract
   (`prelude.rs:1458`) and PX8-I both make native correct on F5 and F6. Any
   remedy that changes native checked narrowing, or that "aligns" by relaxing
   native to match interp, is out of scope and wrong. (Architect, explicit.)
2. **F6 direction:** the interpreter's narrowing yields the **consuming
   operation's** error, rather than substituting a sentinel and entering
   dispatch.
3. **Scope:** `crates/ken-interp/src/eval.rs` and the conformance seed. **Not**
   native, **not** the shared `ken-host` dispatch — the shared path is
   consistent by construction and is not the defect.
4. **One branch, one Decision** (§14(4)). The conformance companion means the
   combined diff-scope touches `conformance/`, which correctly **pulls a CV
   vote** alongside the Architect's §14. Do not publish the `ken-interp` fix on
   its own crates-only branch while the seed waits — that is precisely the
   split that has dropped pieces before.
5. **CI is the venue for workspace-green** (COORDINATION §12). Local work is
   `scripts/ken-cargo -p ken-interp` (and `--test` for one suite). **Never**
   a local `--workspace`.

## 5. Acceptance criteria

1. **F5 fixed.** The `ReadSome` arm computes `remaining = requested − count`,
   reading the request length from the request exactly as the write sibling
   does. No other behavior in the arm changes.
2. **F5 net DISCRIMINATES.** A conformance case with a genuine short read
   (`0 < transferred < requested`) observing `transfer_count_remaining =
   requested − transferred` **and** the request budget. It must **fail against
   the pre-fix hardcoded zero and pass after** — state that flip explicitly.
   The existing seed observes count and span length only
   (`seed-buffer-io.md:420` exercises the full-read case, where both
   implementations agree at remaining 0), so it **cannot** distinguish full
   from short; a case that passes both before and after is not a net.
3. **F6 fixed, with per-consumer coverage.** For **each** of the 4 consumers
   in §3: one **single-fault** out-of-range case and one **overlapping**
   resource-error case (malformed input *and* a coincident liveness/rights
   fault), asserting the **exact** interp and native error variant — never
   `is_err`.
4. **`BufferAllocate` classified on its own merits.** Report a verdict:
   defect / accepted latitude / needs a different remedy — including whether
   a malformed capacity currently **fails closed at all**, given `0` is a
   lawful value. Do **not** fold it into the `u64::MAX` remedy by assumption.
5. **Non-reaching obligations are reported, not dropped.** If a consumer has
   no constructible overlapping-fault discriminator, say so with the reason
   and mark it honestly non-covered. A coverage obligation silently omitted
   reads as covered; an obligation reported unreachable is information.
6. **No native or shared-dispatch change.** `crates/ken-runtime/` and
   `crates/ken-host/` are byte-unchanged. Confirm in the PR body.
7. **Green:** `scripts/ken-cargo test -p ken-interp` on the candidate;
   full `--workspace --locked` green **in CI**, polled by the publisher path.

## 6. Guardrails — do not reopen

- **Do not buy parity by changing native.** If native looks wrong somewhere,
  that is a separate finding — file it, do not fix it here.
- **Do not weaken or delete an existing test** to accommodate the new
  behavior. A conformance seed that changes verdict is a signal to stop and
  escalate, not to re-baseline.
- **A test that passes before and after the fix is not a net.** Every net
  added here must have a stated pre-fix failure.
- **Do not bundle RT-SPLIT.** The `cranelift_backend.rs` decomposition is a
  separate WP on a separate branch and must not ride this one.
- **Every anchor here is perishable.** All line numbers and the §3 inventory
  were measured at `origin/main @ 244cfe9c`. Re-verify at pickup; **if a
  fixed input turns out false against the landed code, say so and escalate —
  do not quietly build around it.**
