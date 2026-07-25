---
id: RT-FNSPLIT-B2B
title: "RT-NATIVE-FNSPLIT Boundary B2b — full emission census, finite differences, and the explicit growth verdict"
status: draft
owner: runtime
size: M
gate: none
depends_on: [RT-FNSPLIT-B2A]
blocks: []
github: null
origin: recut frame docs/program/wp/RT-NATIVE-FNSPLIT-recut.md (Boundary B's metric list, unchanged); operator scaling gate evt_4btfhwqhah1ye. Split from B2 by the Steward 2026-07-25.
---

> ## ⛔ THIS IS THE SLICE THAT CLOSES `RT-NATIVE-FNSPLIT`
>
> It answers the operator's scaling gate. **`draft` until B2a lands** — the
> census cannot measure an emitter that is still being ported.

## Objective

Take the **full emission census** over the ported emitter, report growth as
**finite differences**, and state an **explicit verdict** on whether native
per-function lowering is bounded **O(n)** in nested resource-bracket depth.

## The metric list — Boundary B's, unchanged from the recut frame

- emitted helpers
- CLIF instructions / bytes
- descriptor construction / comparison work
- compile wall time + peak RSS
- the same structural counts as Boundary A's census
- **plus** the exact normal / abrupt / trap / join / affine differential suite

⛔ **Do not borrow Boundary A's numbers as a baseline.** A's landed census
(`647a2e5b`: `87/115/143/171/199`, `K=8`, widths `12/32/16`) is true **only for
the outer planner** and is **PROVISIONAL** for the completed representation.

## Acceptance — carried verbatim from the recut frame

- **AC1.1′ — fail-closed is retained.** A run that cannot complete reports
  `could_not_determine` as a **third outcome that FAILS**, never a silent pass.
  *(Recut Phase 1 returned exactly this; it is a real answer, not an error.)*
- **AC1.2′ — every metric in the list, reported.** Missing one is a **failed
  AC**, not a footnote.
- **AC1.3′ — first AND second finite differences.** ⛔ A single ratio, or a
  fitted curve alone, discharges nothing.
- **AC1.5′ — ⛔ do NOT claim an exponent from few points.** `370n`, `93n²`, and
  a product switching on at n=5 **all pass through the historic n=4 datum.**
  ★ **The four structural invariants are what discriminate; the table
  corroborates.** State the verdict on the invariants, and let the numbers
  support it rather than carry it.

### The four structural invariants (the discriminator)

1. No flattened env / pending / path member in helper identity.
2. Constant ID / node payload width.
3. Affine total persistent nodes.
4. At most affine logical chain depth.

⛔ **Do NOT require the logical chain length itself to be constant** — logical
persistent-chain depth may grow Θ(n) and that is **sound**, because the
helper/frame carries one constant-width ID into the persistent store rather
than the chain itself. *The original frame's metric demanded constant maxima
here and would have rejected a correct design.*

## ⭐ Symptom inventory

Armed at release (architect §1b / steward §5a-ii), seeded with the held chain's
four entries and their shared predicate — *a dynamic property must not name
static code*. A new entry reducing to that predicate means the **port** is
incomplete, and routes back to B2a rather than being ruled here.

## On closing

When this lands, the Steward flips `RT-NATIVE-FNSPLIT` from `active` to
`merged`, which unblocks `NATIVE-HANDLE-CARRIER` → `PX8-F-CAP-41`.
