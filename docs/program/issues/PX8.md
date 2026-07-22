---
id: PX8
title: "partial/positioned IO — the completion program's root; closure condition"
status: active
owner: runtime
size: L
gate: none
depends_on: []
blocks: []
github: null
origin: docs/program/09-posix-linux-abi-campaign.md (charter, PX-C phase); closure condition added 2026-07-22 (operator-approved)
---

**This issue exists because PX8 had none.** `docs/program/10-linux-abi-completion.md`
makes **15 of its 18 items** unblock on `PX8 --> ABI-R3` and `PX8 --> PX9`, and
**nothing in the repository defined when that happens.** The sub-issues existed;
the parent did not. So "is PX8 done?" was a judgement call with no artifact
behind it — which is the same *restated-instead-of-derived* defect that
**ABI-R3 exists to fix**, applied to the program's own root.

> **Why `blocks:` is empty.** `ABI-R3` and `PX9` are the two items this gates,
> but **neither has an issue file yet** (they are unframed items in
> `10-linux-abi-completion.md`, not tracked issues), so there is no id to point
> at. Minting stubs purely to satisfy a schema field would be inventing scope.
> **The gate is bound in prose below and in that document's §5 graph.** When
> `ABI-R3` and `PX9` are framed, each takes `depends_on: [PX8]` and this note
> comes out.

## ⛔ Closure condition — a PROPERTY, not a checklist

Per `10-…:239-241` (*"Frames must state the mechanism, not name one"*), the
gate is the property. The sub-issue list below is the **currently known
sufficient set**, not the definition — if these all merge and the property does
not hold, **PX8 is not done.**

> **PX8 is closed when: every value the positioned/partial IO path reifies into
> checked Ken code is (a) correct against the LOCKED text of
> `spec/30-surface/38-ffi-io.md`, asserted ABSOLUTELY rather than differentially,
> and (b) indexed to the same request/span/buffer as every other value in the
> same reply — on BOTH the interpreter and the native backend.**

Two clauses, both load-bearing, both learned the hard way:

- **(a) absolutely, not differentially.** `RT-PARITY` merged green with six
  differentials over a **shared wrong basis**. A differential is a *relative*
  oracle: it establishes that two implementations agree, never that either is
  right. Where the spec is the authority, the oracle asserts against the
  **normative text**. This is exactly how `BUDGET-EFF`'s defect survived.
- **(b) co-indexed.** `SPAN-SEAL`'s whole subject. `BUDGET-EFF` is the same
  shape arriving through **reification** rather than through a producer: the
  span is correct and the count's `remaining` is indexed to the *raw* request
  while the span is indexed to the *effective* one — two halves of one
  `ReadSome` describing different requests.

## Known sufficient set

| sub-issue | status | note |
|---|---|---|
| `SPAN-SEAL` | ✅ merged | co-indexing for carrier producers |
| `RT-PARITY` | ✅ closed | interp/native parity — **necessary, not sufficient**, see (a) |
| `BUDGET-EFF` | ▶ **active** | `remaining` bounded by the effective request |
| `SEAL-2` | queued | derived carrier-producer enumeration; `depends_on: [RT-PARITY, BUDGET-EFF]` |
| `RT-ESCAPE` | queued | ⚠ `size: TBD` — size it before releasing |
| `RT-SPLIT` | ⚠ **see below** | probably NOT a PX8 semantic dependency |

### ⚠ RT-SPLIT is bundled into PX8 by the docs, and probably should not be

`RT-SPLIT` is a **pure maintainability decomposition** of
`cranelift_backend.rs` (22,081 lines) and its own text says it *"feeds no
G-gate."* `10-…:272-275` lists it among PX8's in-flight set, but on its own
terms it carries **no semantic obligation** in the closure property above.

**Ruling (Steward, 2026-07-22): `RT-SPLIT` does NOT gate PX8.** It is real work
and stays queued for the Runtime ring, but `ABI-R3` and `PX9` **do not wait for
it**. Holding 15 items behind a file-splitting refactor would be a pure
sequencing loss. If someone believes it *is* a semantic dependency, the burden
is to name which clause of the closure property it discharges.

## Why this matters downstream

`ABI-R3` (the derived operation inventory) and `PX9` (cross-domain
`System.Error`) are the two gates. **Nothing else in the completion program
moves until they open**, and between them they transitively gate every item
except `ABI-R1` and `ABI-S3`. `PX9` alone gates six items including the whole
of Track T.

## Notes

- **Do not close this issue on the sub-issue list going green.** Re-check the
  property. The list is evidence, not the definition — that distinction is the
  entire reason this file exists.
- The completion program's coverage record is `10-linux-abi-completion.md` §9.
  Update it when this closes.
