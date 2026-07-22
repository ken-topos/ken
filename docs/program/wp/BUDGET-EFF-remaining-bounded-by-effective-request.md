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

⛔ **The paragraph that stood here was FALSE and is retracted.** It read:
*"the fixture genuinely exercises `:443-444` and only the spec bound fails —
not green-for-the-wrong-reason in either direction."* That is the adversary's
own characterization of its own artifact (commit message of `06bb9538`), which
the Steward repeated back as grounds for trusting it. **Neither of us had
checked the conclusion.**

What is true: the **three premise assertions** do pass and do drive real work
(a real `dispatch_host_op_v1`, a real file, a real buffer) — the tail cap
fires, the capped read fills the buffer exactly, `span.length == count` per
`:406`. **The conclusion observes none of it.** *Real premises wrapped around
a vacuous conclusion is far more convincing than an obviously thin test* —
that is the shape, and it is why the assurance read as credible.

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

## ✅ ARCHITECT DESIGN RULING — DELIVERED, BINDING (2026-07-22)

Decision `dec_1m6xdwjp2ttyn` — **resolved**. Ruling `evt_1g6j2p7jnwbfb`. The
two-option fork above is **closed**; the enclave ruling it waited on
(`SPEC-38-ERRATUM`) merged at `e5a400c7`.

### Option 1, refined as an INSEPARABLE VALIDATED CARRIER

**Preserve the caller-authored request unchanged.** Carry the effective request
as private reply data *inside* the transfer count:

```rust
pub struct TransferCountV1 {
    transferred: u64,
    effective_request: u64,
}

impl TransferCountV1 {
    fn new(transferred: u64, effective_request: u64) -> Option<Self> {
        (transferred > 0 && transferred <= effective_request).then_some(Self {
            transferred, effective_request,
        })
    }
    pub fn get(self) -> u64 { self.transferred }
    pub fn effective_request(self) -> u64 { self.effective_request }
}
```

Spelling may follow local convention; **the two private scalars and the
constructor invariant are the required shape.** `ReadSome { span, transferred }`
and `Wrote(transferred)` may keep their Rust enum arity — the budget lives
inside the constructor-private count, **not as a loose sibling field that can
drift from it.**

**Why.** The host is the only layer that knows the post-validation cap
(`effect_v1.rs:1621-1623`, `1679-1681`) and today destroys it in
`TransferCountV1::new` (`:2059-2068`). Interp receives the typed canonical
reply; native receives `HostReplyV1` and has **neither buffer capacity nor any
lawful way to reconstruct the cap.** So any solution preserving the raw request
must transport the fact across *both* reply boundaries. Inferring it from
`span.length` is **wrong on a short read**; inferring it from raw `length` **is
the defect**.

**Option 2 REJECTED.** `CanonicalRequestV1`/`FsPositionedRequestV1.length` is
the caller's request and the effect trace records it as such. Mutating it after
dispatch turns an input into an **undocumented in/out channel**, erases the
raw/effective distinction the landed spec now states explicitly, and leaves
audits unable to compare caller intent against host normalization.

### Boundary constraints — all six are binding

1. **The native reply gets a NAMED `effective_request: u64` success field.**
   Do **not** smuggle it through `bytes.data`, `resource_error.*`, or any
   tag-unrelated slot. Update Rust/C `HostReplyV1`, the C probe, the generated
   layout accessor, the **host-effect ABI hash**, and the Cranelift consumer
   **together**. ⚠ **This is a real semantic ABI change even if spare bytes
   exist — record it honestly.**
2. **The private effect-trace codec preserves BOTH raw and effective.**
   Encode/decode the new pair and **invoke the validating constructor on
   decode** — never deserialize an unchecked tuple, never reconstruct the budget
   from count. Follow the existing single-schema/fail-closed policy; do not
   invent a compatibility lane.
3. **Native validation is `0 < count <= effective_request <= raw_length`.**
   Range / no-wrap / span-containment use **effective_request**; only then may
   the private structural Nat be minted. Both reifiers compute exactly
   `remaining = effective_request - count`. **Raw remains an outer consistency
   ceiling and audit input — never a progress budget.**
4. **Applies to BOTH `ReadSome` and `Wrote`.** Write usually has effective equal
   to the valid remaining input span, but **the constructor and ABI must not
   encode a read-only invariant into the shared `TransferCount` type.**
5. **Preserve the one-mint property** — span length, transferred count,
   predecessor, and remaining all derive from one validated reply. No public
   `Int` budget, no caller-selected fuel, no surface constructor.
6. **⇒ BUDGET-EFF SEQUENCES BEFORE ABI-M1.** ABI-M1 must manifest the
   *post-fix* host-effect family projection/hash rather than generate a manifest
   over a knowingly incomplete reply and churn immediately.

> **On the obsolete guard.** PX8-N's *"no ABI expansion"* guard was conditional
> on the then-believed scalar reply being sufficient. `SPEC-38-ERRATUM` shows
> that premise is **false**. **Do not preserve that guard by corrupting request
> semantics** — the guard is what expires, not the spec.

### ★ Oracle constraints — the capped-SHORT case is load-bearing

AC-2 and AC-3 stand exactly as settled below (independent spec-absolute tests
at the reification seats). **Add two discriminator shapes on each side:**

| shape | raw | effective | count | ⇒ remaining | ⇒ total budget |
|---|---|---|---|---|---|
| capped-full | 8 | 4 | 4 | **0** | 4 |
| **capped-short** | 8 | 4 | **< 4** | **4 − count** | 4 |

**⛔ capped-short is not optional. Capped-full ALONE is green under the wrong
shortcut `effective := count`** — it cannot distinguish a correct
implementation from one that simply echoes the count. This is the
discriminating-axis discipline: one case that passes proves less than two cases
that separate the hypotheses.

Also add **fail-closed** native/wire mutations for `effective == 0`,
`effective < count`, and `effective > raw`. **A parity assertion may remain
supplementary — never the AC oracle** (parity is the instrument that missed
this defect).

### Sizing surface

`ken-host` typed carrier + dispatch + ABI/C probe + private trace codec ·
`ken-interp` reifier + test · `ken-runtime` layout consumption + validated mint
+ test · generated/hash fixtures.

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
reifier field** — not `remaining`, not `transfer_count_request_budget`.

**⛔ It is UNSATISFIABLE, which is strictly worse than non-discriminating**
(adversary, verified 2026-07-22). `count` appears on both sides and
**cancels**, so the assertion reduces to `RAW_LENGTH == effective` — i.e.
`8 == 4`. The adversary swept every value `count` can take (`0..=CAPACITY`)
and it is false for all of them. **No implementation, correct or defective,
could ever have discharged AC-3 as originally written.** A ring given it would
not have found it merely unhelpful — it would have worked until it either
edited an oracle pinned *"unchanged"* or escalated. It is a *proxy* (arithmetic over its own
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

> ### ⛔ AC-3 CANNOT BE SATISFIED IN PLACE — it must RELOCATE
>
> **Do not try to fix the oracle where it sits.** It lives in `ken-host`'s
> `effect_v1` dispatch test module (it needed the module-private
> `PositionedBackend`), and **`remaining` is not reachable from there** —
> verified: the string does not occur anywhere in
> `crates/ken-host/src/effect_v1.rs`. The field is constructed on the two
> reifier sides only:
>
> - `ken-interp/src/eval.rs:4934-4935` — `requested.checked_sub(count)`, into
>   the interpreter's value store via `make_ctor(fs.private_transfer_count_id, …)`
> - `ken-runtime/src/cranelift_backend.rs:13081-13082` —
>   `isub(request_length, count)`, as Cranelift IR
>
> **⇒ The rewrite is two new tests at seats that can observe a reified
> `TransferCount`:** an interp-level test, and a native one alongside
> `rt_parity_native.rs`. **Budget it as plumbing, not a formula swap** — the
> same shape as the underlying defect. (Adversary, `evt_521sw77qxkqxb`;
> independently re-verified by the Steward against the landed source before
> folding.)

> ⚠ **The general lesson, worth more than this WP.** An adversary's repro is
> built to *demonstrate a defect* (expected-fail); a completion oracle is built
> to *define correctness*. **They are different artifacts and one does not
> become the other by being pinned.** Before pinning any oracle as an AC,
> verify it observes the mechanism — a red result is not evidence that it
> can ever go green for the right reason.
>
> **And the sharper half, which is the oracle-builder's** (adversary's own
> carry, recorded here so it is not lost): before handing over any artifact as
> evidence, check whether the concluding assertion **can fail for a reason
> other than the one you want** — specifically, **whether the observed value
> appears on both sides of the comparison and cancels.** The instrument was
> the adversary's; the pin was the Steward's; **the assurance that stopped
> either of us looking was the commit message.** A seat's characterization of
> its own evidence is not evidence.
>
> **The R1 defect itself is UNAFFECTED and still stands**, on source
> inspection of the two reifier sites above. What was broken was the
> demonstration, not the finding — do not discount the defect along with the
> instrument. The corrected artifact is
> `adversary/R1-effective-request-repro @ bede2a37` (doc-comment retraction
> only; test body byte-identical so the defect stays inspectable).
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
