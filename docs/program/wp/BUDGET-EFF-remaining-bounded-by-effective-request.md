# BUDGET-EFF — `TransferCount.remaining` must be bounded by the EFFECTIVE request

**Status:** DRAFT (not framed, not kicked) · **Size:** M
**Origin:** adversary hunt `evt_1s9rt48z7bpsn` on RT-PARITY `e892777c`, finding R1
**Severity:** CONFIRMED violation of a **locked** spec line. Fail-closed — not
memory-unsafe, not a forgery, not a parity bug. **Wrong value, right memory.**
**Priority: ahead of SEAL-2.** SEAL-2 closes a gate with no live defect; this is
a live contradiction of locked normative text.

---

## The defect

`spec/30-surface/38-ffi-io.md:404-405` — *"`TransferCount` is constructor-private,
strictly positive, and **bounded by the effective request**."* Plus `:443-444`
tail capping — *"a request that starts in range but extends beyond the buffer
tail uses the **capped effective range**."*

The host **clamps rather than rejects** (`crates/ken-host/src/effect_v1.rs:1743-1744`,
`let effective = requested.min(capacity - start);`) and validates against
`effective` (`:1652`). **Both implementations then derive `remaining` from the
RAW request length:**

- interp — `crates/ken-interp/src/eval.rs:4922-4935`: `requested` comes from
  `CanonicalRequestV1::FsReadAt { length }`, passed by `&`, never re-capped;
  `remaining = requested.checked_sub(count)`.
- native — `crates/ken-runtime/src/cranelift_backend.rs:13081-13082` via
  `mint_validated_progress_nat`: `remaining = isub(request_length, count)`,
  `request_length` from `positioned_bounds`, narrowed **before** dispatch and
  therefore blind to buffer capacity.

**Worked example, already pinned green in the landed tree**
(`effect_v1.rs:3407-3429`: capacity-4 buffer, `FsReadAt { file_offset: 3,
buffer_start: 0, length: 8 }` → `span.start == 0, span.length == 4,
transferred == 4`):

| | value |
|---|---|
| raw `length` | 8 |
| `effective` = min(8, 4−0) | 4 |
| `count` | 4 |
| landed `remaining` = 8 − 4 | **4** |
| `transfer_count_request_budget` = 4 + 4 | **8** |
| **spec-required** (bounded by *effective*) | remaining **0**, budget **4** |

**The buffer is completely full (`span.length == 4 == capacity`) while the
reified count tells checked source 4 bytes of budget remain.** `:406` (span
length equals count) *is* satisfied — the span is correct. It is the count's
`remaining` that drifts, so **the two halves of one `ReadSome` are indexed to
different requests** — the raw one and the capped one. That is the same
"values not indexed to the same request/span/buffer" shape SPAN-SEAL exists to
prevent, arriving through **reification** instead of through a producer.

## ★ Why RT-PARITY could not have caught this — the transferable lesson

Interp and native **agree**. Six differentials comparing two implementations
that share a wrong basis all pass green. **A differential is a RELATIVE oracle:
it can only establish that two things agree, never that either is right.** This
defect was structurally invisible to RT-PARITY's entire evidence structure and
would have stayed invisible under any amount of additional parity testing.
**Where the spec is the authority, the fix's oracle must assert against the
NORMATIVE TEXT, not against the other implementation.**

## Sequencing — the spec erratum comes FIRST, and this is not negotiable

`38` contradicts itself exactly where the clamp occurs:

- `:404-405` — bounded by the **effective** request; `:443-444` — capped
  effective range.
- `:419-420` (partition table) and `:438-440` (read contract) — `0 < n ≤
  **requested**`.

**An implementer reading only the table writes exactly what is landed.** Fixing
the implementation first means the next implementation re-derives the same
defect from the same sentence. So:

1. **Spec enclave rules** which reading is normative and makes `38`
   self-consistent across all four sites. (Expected outcome is *effective* — a
   budget describing a buffer you cannot write to is not a usable value — **but
   that is the enclave's ruling to make, not mine.**)
2. **Then the implementation**, against the settled wording: interp + native +
   the reified count, plus the catalog's `transfer_is_bounded` lemma and the
   `transfer_count_request_budget` contract, which today describe a bound taken
   from the wrong request.

## ★★ THIS IS A PLUMBING GAP, NOT A WRONG SUBTRACTION — size it accordingly

⛔ **CORRECTED — this section previously read "Confirmed by EXECUTION, not
inference." That was FALSE** (Steward, 2026-07-21). The R1 oracle never reads
a reifier field, so its failure is not evidence of anything; see AC-3. **The
defect rests on SOURCE INSPECTION** of the two reifiers, which is strong but is
not execution. Producing a real executable demonstration is part of this WP.

Reference: `adversary/R1-effective-request-repro
@ 06bb9538` (local, unpushed; lives in the `effect_v1` dispatch test module
because it needs module-private `PositionedBackend`). Fails at `e892777c`:

```
left: 4, right: 0
the buffer is FULL (span.length 4 == capacity 4), yet the reified count reports
4 bytes of request budget remaining. transfer_count_request_budget = count +
remaining = 8, which is the RAW request 8, not the effective request 4.
```

Its **three premise assertions all pass** — the tail cap fires
(`effective == capacity`), the capped read fills the buffer exactly, and
`span.length == count` per `:406`. So the fixture genuinely exercises
`:443-444` and **only the spec bound fails**. Not green-for-the-wrong-reason in
either direction, which matters for a test whose whole job is to flip.

**⛔ The effective value is NOT AVAILABLE at the reification site.**
`TransferCountV1` is a single-field newtype
(`pub struct TransferCountV1(pub(crate) u64);`). `TransferCountV1::new(read,
effective)` uses `effective` to **validate**, then **discards** it. Nothing else
carries it: the reply is `ReadSome { span, transferred }` with
`span.length == count`, and the request still holds the **raw** length. So
**neither reifier can compute the spec-required bound from the information it is
given.**

**Two candidate closures, materially different blast radii — ARCHITECT CALL,
not the Steward's and not the ring's:**

1. **The reply carries the effective request** — wire/ABI-shaped, so
   `crates/ken-host/` and `crates/ken-runtime/` **stop being byte-unchanged**.
2. **The host caps the request record before it is reified**, so "the request"
   *means* the effective request downstream. This is also the option that would
   make `transfer_count_request_budget` mean what its name says **without
   touching the wire**.

The adversary explicitly declined to choose and was right to. **Route this to
the Architect with the enclave ruling**, before sizing the implementation half.

## Acceptance criteria (draft — finalize after the enclave ruling)

**AC-1** — `remaining` and `transfer_count_request_budget` derive from the
**effective** (post-clamp) request on **both** interp and native.
**AC-2 — ★ the oracle asserts against the SPEC, not against the other
implementation.** A differential between interp and native is **not** acceptable
evidence for this AC; it is the instrument that missed the defect. Assert the
absolute property (`request_budget == effective`) on each side independently.
**AC-3 — ⛔ REWRITTEN 2026-07-21. The previous wording was VOID and the
error was the Steward's.** It said the R1 oracle at
`adversary/R1-effective-request-repro @ 06bb9538` must *"pass, unchanged"*.
**That is impossible, and not because of anything an implementer could do.**
The oracle's conclusion is:

```rust
let derived_remaining = RAW_LENGTH - count;   // 8 - 4 = 4
let spec_remaining    = effective - count;    // 4 - 4 = 0
assert_eq!(derived_remaining, spec_remaining) // 4 == 0 — always fails
```

**Both sides are computed from the test's own constants. It never observes a
reifier field** — not `remaining`, not `transfer_count_request_budget`. It
fails identically against a perfect implementation, so its observed failure at
`e892777c` confirmed nothing. It is a *proxy* (arithmetic over its own
literals) standing in for the *mechanism*, which is precisely the defect class
`Q-RESIDUE` existed to remove. Its commit message compounds this: the
"absolute oracle" and "not green-for-the-wrong-reason" claims describe the
three **premise** assertions, which do pass and do exercise the tail cap — the
**conclusion** reaches nothing. The failure message quotes "the reified count
reports N bytes remaining" where every number is the test's own literal.

**AC-3, corrected:** the R1 oracle is **rewritten to read the reifier's actual
output** and assert it equals `effective - count`, on **both** interp and
native. It then passes. **"Unchanged" is withdrawn** — it pinned a broken
instrument as the definition of done.

> ⚠ **The general lesson, worth more than this WP.** An adversary's repro is
> built to *demonstrate a defect* (expected-fail); a completion oracle is built
> to *define correctness*. **They are different artifacts and one does not
> become the other by being pinned.** Before pinning any oracle as an AC,
> verify it observes the mechanism — a red result is not evidence that it
> can ever go green for the right reason.
**AC-4** — `38` is self-consistent at `:404-405`, `:419-420`, `:438-440`,
`:443-444`; no site still says `≤ requested` if the ruling is `effective`.
**AC-5** — the catalog lemma/contract are re-grounded on the settled bound.

## Coupling flagged but NOT claimed (adversary traced it, does not reach today)

`write_all_advance_span span count` sets the next span's budget to
`transfer_count_remaining count`, so an inflated `remaining` reaching that
transition would inflate `writeAll`'s fuel and span budget. **It does not reach
there today:** `writeAll`'s counts come from `writeAt`, whose request length is
`buffer_span_length span` — a span within its buffer, so no clamp, no inflation.
**Confined to the read path's reified count. One cross-buffer span away from
mattering.**

## Related, filed separately

**R2 (untested hypothesis, adversary)** — `freeze (a) (buffer) (span)` takes
buffer and span as **independent** parameters and `PrivateBufferSpan Int Nat`
carries **no buffer identity**, so a span minted against buffer A is well-typed
against buffer B. That is a **call-site indexing** question, orthogonal to the
BufferFreeze exemption's **constructibility** claim. Static reading says
fail-closed (`buffer.initialized_slice` bounds-errors rather than clamping).
Reaching it needs two nested `withBuffer` brackets — plausibly the same shape as
**`RT-ESCAPE`**, where it is attached. **If it IS source-reachable, BufferFreeze
owes executable single-fault and overlap coverage by the exemption's own terms.**

## Verification

Targeted only per `agent/COORDINATION.md §12` — `scripts/ken-cargo -p <crate>`.
**Never `--workspace`.**
