---
id: SPEC-38-ERRATUM
title: "spec 38-ffi-io self-contradicts on the transfer bound — rule and reconcile"
status: active
owner: spec
size: S
gate: none
depends_on: []
blocks: [BUDGET-EFF]
github: null
origin: adversary R1 via docs/program/wp/BUDGET-EFF-remaining-bounded-by-effective-request.md
---

**Locked** `spec/30-surface/38-ffi-io.md` contradicts itself at exactly the
point where a read is clamped. Four sites, two incompatible readings:

| site | says the bound is |
|---|---|
| `:404-405` | the **effective** request |
| `:443-444` | the capped **effective** range |
| `:419-420` (partition table) | `0 < n ≤` **requested** |
| `:438-440` (read contract) | `0 < n ≤` **requested** |

## Why this must land BEFORE the code fix

**An implementer reading only the partition table writes exactly what is
landed today.** The host currently clamps
(`let effective = requested.min(capacity - start);`), validates against
`effective`, and then both reifiers derive `remaining` from the **raw**
request. That is a faithful implementation of `:419-420`.

So fixing the implementation first means **the next implementation re-derives
the same defect from the same sentence.** The defect is not primarily in the
code; it is in the normative text the code was written against.

## The deliverable

Rule which reading is normative, and make `38` **self-consistent across all
four sites** — no site may still say `≤ requested` if the ruling is
`effective`, and vice versa.

> **The expected outcome is `effective`** — a budget describing a buffer you
> cannot write to is not a usable value. **But that is the enclave's ruling to
> make, not the Steward's, and the reasoning matters more than the verdict.**
> If the enclave rules `requested`, the code is correct and BUDGET-EFF
> collapses to a documentation fix. Rule it on the merits.

## Explicitly NOT in scope

- **Do not touch the implementation.** `crates/ken-host`, `ken-interp`,
  `ken-runtime`, and the catalog's `transfer_is_bounded` lemma /
  `transfer_count_request_budget` contract are **BUDGET-EFF's** half, which is
  blocked on this ruling.
- **Do not choose the closure mechanism.** Two candidates exist with
  materially different blast radii (reply carries the effective request —
  wire/ABI-shaped; vs. host caps the request record before reification — no
  wire change). **That is an Architect call**, and it routes *after* this
  ruling, carrying it.

## Grounding

Confirmed **by execution**, not inference: `adversary/R1-effective-request-repro
@ 06bb9538` fails at `e892777c` — a read of 8 into a 4-byte capacity reports
`remaining = 8`, the raw request, not the effective 4. The oracle is pinned as
BUDGET-EFF's AC-3 and must pass **unchanged**.

Full analysis:
[`docs/program/wp/BUDGET-EFF-remaining-bounded-by-effective-request.md`](../wp/BUDGET-EFF-remaining-bounded-by-effective-request.md).
