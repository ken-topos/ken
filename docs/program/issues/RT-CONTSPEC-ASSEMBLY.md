---
id: RT-CONTSPEC-ASSEMBLY
title: "ContinuationSpecialization seam 1 — the lawful assembly: extract the accepted branch-scope helper and its feature-gated harness onto the landed slice 0-2 blobs, unactivated, and prove the prior-slice surfaces are untouched"
status: ready
owner: runtime
size: M
gate: none
depends_on: [RT-CONTSPEC-ABI]
blocks: [RT-CONTSPEC-ACTIVATE]
github: null
origin: "Architect ownership/sizing ruling evt_1yymw1gdszpbs (2026-08-02), outcome (c) on RT-CONTSPEC-LOWER. Seam 1 of the Steward's four-seam recut. Steward-filed (agents cannot create tracked work per COORDINATION section 2)."
---

# Seam 1 — establish that a lawful capstone assembly exists at all

RT-CONTSPEC-LOWER did not fail on 138 defects. It failed because **the held
lineage replaced the landed slice 0-2 blobs with older cumulative WIP instead
of consuming them.** Measured by the Architect against the rule-derived base
`b66dea6a`, and independently re-verified by the Steward at
`origin/main = 40f8757d`:

| surface | held lineage vs `b66dea6a` | `main` blob |
|---|---|---|
| `planning/static_transition.rs` | +20,199 / -6,120 | `6725971d`, identical to `b66dea6a` |
| `planning/static_transition/abi.rs` | +748 / -87 | `23b9f5d7`, identical to `b66dea6a` |
| `planning/static_transition/semantic_ir.rs` | +75 / -48 | — |
| `planning.rs` | +21 / -9 | — |
| `boundary_value.rs` | +32 / -28 | — |
| `boundary_value_clif.rs` | +57 / -2 | — |

**76 of the 138 failing rows are evidence against that assembly, not defects to
repair.** A planner- or ABI-worded refusal on the held tree says nothing about
whether the merged slices are correct.

⇒ This seam does one thing: **show that the accepted mechanism composes with the
landed slices at all.** Nothing is activated, so nothing can be measured about
behaviour yet — and that is the point. Until a lawful assembly exists, every
downstream measurement is uninterpretable.

## What is preserved and what it is for

- `46d29783b9d726e542bd9fed6833e2644a40b5fc` — the census object. **A
  preservation oracle, not a repair base and not a candidate.** Its value is the
  138-row run record; its lineage is void.
- `1aef3192` — its parent, the preservation-only D8 seam.
- `9d58df12` — the mechanism object the Architect accepted and froze.
- `refs/preserved/rt-contspec-lower-held-core-rs = 88972207`.

⛔ **None of these may be merged, rebased onto, or cherry-picked wholesale.** The
accepted helper is extracted by hand onto current `main`.

The frame is `docs/program/wp/RT-CONTSPEC-ASSEMBLY.md`.
