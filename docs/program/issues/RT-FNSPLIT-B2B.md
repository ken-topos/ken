---
id: RT-FNSPLIT-B2B
title: "RT-NATIVE-FNSPLIT Boundary B2b — full emission census, finite differences, and the explicit growth verdict"
status: closed
owner: runtime
size: M
gate: none
depends_on: [RT-FNSPLIT-B2F]
blocks: []
github: null
origin: recut frame docs/program/wp/RT-NATIVE-FNSPLIT-recut.md (Boundary B's metric list, unchanged); operator scaling gate evt_4btfhwqhah1ye. Split from B2 by the Steward 2026-07-25.
---

> ## ⛔ CLOSED 2026-07-28 — SUBSUMED INTO [[RT-SCALE-B]]. Do not build from this.
>
> **This node and `RT-SCALE-B` were the same deliverable, filed twice by the
> Steward** — this one 2026-07-25, `RT-SCALE-B` 2026-07-26 on the stated premise
> that the operator's scaling gate *"had acceptance criteria and no tracked
> node."* ⚠ **That premise was false: this file was the tracked node.**
>
> | this node | `RT-SCALE-B` |
> |---|---|
> | `AC1.1′` fail-closed `could_not_determine` | `AC-B1`, same |
> | `AC1.2′` every metric reported | `AC-B2`, same |
> | `AC1.3′` first **and** second finite differences | `AC-B3`, same |
> | `AC1.5′` no exponent from few points | `AC-B4`, same sentence |
> | Boundary B's metric list | `D2` |
> | the normal/abrupt/trap/join/affine differential suite | `D3` |
> | "on closing, flip `RT-NATIVE-FNSPLIT` → `merged`" | the `blocks` edge |
> | — | `D4` **the analytical model** (Architect, research-grounded) |
>
> ⇒ `RT-SCALE-B` is strictly the larger node: it carries everything here **plus**
> the analytical half and a **written frame**. This one never had a frame — its
> own release note said *"⚠ Re-frame owed before release."*
>
> ⭐ **Four things this node carried that the `RT-SCALE-B` frame lacked have been
> folded in**, and they are why this is a fold and not a deletion: the **four
> structural invariants** (`AC-B4` named a discriminator it never defined), the
> **⛔ do-NOT-require-constant-chain-depth** guard (anti-false-negative — it would
> otherwise **reject a correct design**), the **Boundary-A absolute-numbers**
> trap, and the **symptom-inventory arming**.
>
> ⛔ **Nothing below is live.** It is kept for lineage and for the two
> re-derivations recorded in it, which the fold does not disturb.

> ## ⚠ RE-DERIVED 2026-07-25 — NOT the slice that closes `RT-NATIVE-FNSPLIT`
>
> **Steward re-derivation, resolving the "re-derive or subsume" question the
> `B2F` rulings opened. This node SURVIVES, with a narrowed purpose and a
> corrected predecessor.**
>
> ⛔ **`RT-FNSPLIT-B2F` is the node that closes `RT-NATIVE-FNSPLIT`** — it closes
> symptom-inventory entry 2, the last open entry. This node no longer does.
>
> ⛔ **`depends_on` corrected `RT-FNSPLIT-B2A` → `RT-FNSPLIT-B2F`.** `B2A` was
> **retired** and re-sliced into `B2A-C` / `B2A-S` / `B2F`; a dependency on a
> retired node would never become satisfiable.
>
> ### Why re-derived rather than subsumed
>
> The Architect ruled the **scaling verdict** onto `B2F`'s atomic boundary (old
> `AC-8` superseded there), which appears to empty this node. It does not — the
> two carry **different claims**, and collapsing them would weaken both:
>
> | node | claim | kind |
> |---|---|---|
> | `B2F` | *total units may be Θ(n) while each function is bounded by its own static body/transition contract* | **structural invariant**, mechanically pinned |
> | `B2B` | the measured emission census and finite differences on the landed tree | **empirical quantity**, answering the operator's gate |
>
> ⇒ `B2F` proves the invariant holds; `B2B` reports what the numbers actually
> are. **A structural assertion is not a measurement, and the operator's scaling
> gate asked for a measurement.**
>
> ⚠ **This is NOT the live `ii`/`iii` split Q3 rejected.** That rejection was
> about leaving two live production **authorities**. This node changes **no
> production code** — it measures a tree whose authority is already single and
> already landed. If a reviewer reads it as a re-litigation of Q3, that reading
> is wrong and this paragraph is the answer.
>
> **Its original premise still holds and is why it is sequenced after, not
> merged in:** *a census taken while the emitter is still moving measures a
> moving target.* What changed is only which node the verdict attaches to.
>
> ⚠ **Re-frame owed before release** — the current frame below is written against
> the retired `B2A` and its anchors are stale. It must also inherit `B2F`'s new
> **AC-G0**: name the denominator, and justify excluding any production Cranelift
> emitter (`native_int_clif.rs` is production and emits 5 functions outside the
> backend census). **`draft` until `B2F` lands.**

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
