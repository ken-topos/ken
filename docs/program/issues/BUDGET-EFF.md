---
id: BUDGET-EFF
title: TransferCount.remaining must be bounded by the effective request
status: draft
owner: TBD
size: M
gate: none
depends_on: []
blocks: [SEAL-2]
github: null
origin: evt_1s9rt48z7bpsn
---

Adversary-confirmed (finding R1) violation of **locked**
`spec/30-surface/38-ffi-io.md`: `TransferCount.remaining` must be bounded by
the *effective* request, but the host clamps instead of rejecting, and
validates against the wrong bound. Fail-closed — not memory-unsafe, not a
forgery, not a parity bug: wrong value, right memory. Confirmed by execution
(`adversary/R1-effective-request-repro @ 06bb9538`, fails at `e892777c`).

This is a **plumbing gap, not a formula fix**: `effective` is discarded at
validation and reaches neither reifier, so two closures see different blast
radii. Prioritized ahead of `SEAL-2` — SEAL-2 closes a gate with no live
defect, this is a live contradiction of locked normative text.

**Blocked on a spec erratum landing first** (not itself tracked as an issue
here): `spec/30-surface/38-ffi-io.md` currently contradicts itself, so a
code-first fix would re-derive from broken citation text. The Architect call
on the erratum routes together with this WP. Owning team not yet assigned
(tracker: `*TBD — not yet assigned*`) — do not guess.

Full brief: [`docs/program/wp/BUDGET-EFF-remaining-bounded-by-effective-request.md`](../wp/BUDGET-EFF-remaining-bounded-by-effective-request.md).
