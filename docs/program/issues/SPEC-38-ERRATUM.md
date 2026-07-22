---
id: SPEC-38-ERRATUM
title: "spec 38-ffi-io self-contradicts on the transfer bound — rule and reconcile"
status: closed
owner: spec
size: S
gate: none
depends_on: []
blocks: [BUDGET-EFF]
github: 827
origin: adversary R1 via docs/program/wp/BUDGET-EFF-remaining-bounded-by-effective-request.md
---

> ## ✅ MERGED 2026-07-22 — `origin/main @ e5a400c7` (PR #827)
>
> Verified **by content**, not by the publisher's exit code: the merged file
> is byte-identical to the reviewed blob `fc1d3425`; `effective request`
> governs both partition rows, the read contract, and the tail-cap clause;
> the 8→4 target is recorded at `:410`; **no `≤ requested` remains.**
> Approvals on the exact SHA from conformance-validator (Spec) and Architect;
> Decision `dec_4mkztzh3f9rxy` resolved.
>
> **CLOSED 2026-07-22 — retros in.** Author `evt_6wmrgtrrm5h1t`, validator
> `evt_7a59bn6r6yrhp`, coordination `evt_4qevmd7mhdnnm`. Carry: define shared
> normative quantities per operation; keep semantic target / conformance
> oracle / implementation mechanism as **separate scopes**; re-anchor with
> both current-base *and* reviewed-subtree byte-identity checks.
>
> **Unblocks `BUDGET-EFF`**, which is parked pending operator go. The
> closure-mechanism call (reply-carries-effective vs. host-caps-the-request-
> record) is an **Architect** decision and routes with that release, not now.

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

⛔ **CORRECTED 2026-07-21 — the earlier "confirmed by execution" claim was
FALSE, and it was the Steward's.** The defect is identified by **source
inspection** of the two reifiers (`ken-interp/src/eval.rs:4934-4935` and
`ken-runtime/src/cranelift_backend.rs:13081-13082`, both subtracting from the
RAW length), **not** by execution. `adversary/R1-effective-request-repro @
06bb9538` does fail at `e892777c`, but that failure is worthless as evidence:
**the oracle's own conclusion is broken** — it compares two values computed
from its own constants and never reads a reifier field, so it fails on any
implementation, so it would have failed identically against a *correct* one.
Its failure message quotes "`remaining = 8`" — but that number is the test's
own literal, never a value read back from a reifier. **BUDGET-EFF's AC-3 has
been rewritten accordingly** (the "pass unchanged" pinning is withdrawn).

**The fixture is sound and the source reading is probably right — but nothing
has executed that demonstrates the defect.** Establishing it by execution is
part of BUDGET-EFF's work, not an input to it. Do not repeat "confirmed by
execution" anywhere downstream.

Full analysis:
[`docs/program/wp/BUDGET-EFF-remaining-bounded-by-effective-request.md`](../wp/BUDGET-EFF-remaining-bounded-by-effective-request.md).
