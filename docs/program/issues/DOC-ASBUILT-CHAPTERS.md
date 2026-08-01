---
id: DOC-ASBUILT-CHAPTERS
title: "As-built slice 4 — reconcile the four remaining reading-ken chapters together; they share 9 sources and their claim classes cross page boundaries"
status: ready
owner: doc
size: L
gate: none
depends_on: [DOC-ASBUILT-FRAGMENTS]
blocks: [DOC-ASBUILT-AUDIT]
github: null
origin: "Steward 2026-08-01, phase A slice 4 of DOC-ASBUILT-AUDIT, measured at origin/main 181f1e58. Cut as one WP on the measured fact that 01-anatomy's sources are a strict subset of 05-packages's and the four chapters share 9 distinct sources total."
---

> # ▶ SLICE 4 OF PHASE A — FOUR CHAPTERS, ONE WP, AND THE GROUPING IS THE POINT
>
> `01-anatomy.md` · `02-types-contracts-and-proofs.md` ·
> `03-assurance-and-trust.md` · `05-packages-and-provenance.md`
>
> Together: ⭐ **9 distinct drifted sources across 30 citations** — **fewer
> distinct sources than slice 2 alone** (16).

## ▶ THE FRAME IS WRITTEN

`docs/program/wp/DOC-ASBUILT-CHAPTERS.md`. ⛔ **Read the frame, not this node**,
and read [[DOC-ASBUILT-AUDIT]] first — its two-phase law binds this slice.

## ⭐⭐ Why these four are ONE work package

**Measured at `181f1e58`:**

| relation | measurement |
|---|---|
| `01-anatomy`'s sources ⊆ `05-packages`'s sources | ⭐ **true — it adds nothing** |
| `02-types` adds over `05-packages` | **1** (`spec/30-surface/36-effects.md`) |
| `03-assurance` adds over `05-packages` | **1** (`spec/60-security/64-trust-model.md`) |
| union of all four | ⭐ **9** |

⇒ **The expensive act in this campaign is reading a source at its current blob
and deciding what it falsifies.** Nine reads serve all four chapters. Cutting
them apart would mean reading `EmptyDec`, `Combinators`, `Transport`,
`Property`, `07-catalog-style-guide` and `fragments.md` **four times over**.

⭐⭐ **And the grouping is a CORRECTNESS argument, not only a cost one.** These
four chapters are the same genre and **share claim classes**. If a drifted
source falsifies a claim, that claim is likely to appear in more than one of
them. ⇒ Reviewing them together makes the whole-page sweep naturally
cross-page. ⛔ Reviewing them apart reproduces — *across* WPs — the exact defect
that sank two earlier candidates *within* a page: the named site is repaired
while the same claim survives elsewhere.

## The 9 sources, measured at `181f1e58`

⚠ **Re-derive at pickup**, and ⭐ **strip the `#anchor`** from each manifest
`sources` entry before matching — the gate does, and an exact-string match
silently under-counts every anchored citation.

| source | consumers | 01 | 02 | 03 | 05 |
|---|---|---|---|---|---|
| `docs/program/07-catalog-style-guide.md` | **9** | 3 | 1 | — | 1 |
| `catalog/packages/Core/Logic/EmptyDec.ken.md` | 8 | 1 | 1 | 1 | 1 |
| `catalog/packages/Data/Sums/Combinators.ken.md` | 7 | 1 | 1 | — | 1 |
| `library/learn/reading-ken/fragments.md` | 7 | 1 | 1 | 1 | 1 |
| `spec/30-surface/36-effects.md` | 7 | — | 1 | — | — |
| `catalog/packages/Core/Logic/Transport.ken.md` | 6 | — | 1 | 1 | 1 |
| `catalog/packages/Tooling/Testing/Property.ken.md` | 6 | — | — | 1 | 1 |
| `spec/60-security/64-trust-model.md` | 3 | — | — | **4** | — |
| `spec/30-surface/30-taxonomy.md` | 2 | — | — | — | **4** |

⛔ **No row is re-stamped here** — phase A writes no ledger. Every one of these
is shared with a page outside this slice, so ⛔ **not one of them could be
re-stamped even if phase A permitted it.**

## ⭐ The drift baseline is **28**, and it does not move

`fragments.md` was the only attested `library/` page and slice 1 already edited
it. ⛔ **None of these four is attested**, so editing them adds no row.
**29 means something was written that should not have been; 27 means a
pre-slice-1 base.**
