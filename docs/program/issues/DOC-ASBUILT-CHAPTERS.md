---
id: DOC-ASBUILT-CHAPTERS
title: "As-built slice 4 — reconcile the four remaining reading-ken chapters together; they share 9 sources and their claim classes cross page boundaries"
status: merged
owner: doc
size: L
gate: none
depends_on: [DOC-ASBUILT-FRAGMENTS]
blocks: [DOC-ASBUILT-AUDIT]
github: null
origin: "Steward 2026-08-01, phase A slice 4 of DOC-ASBUILT-AUDIT, measured at origin/main 181f1e58. Cut as one WP on the measured fact that 01-anatomy's sources are a strict subset of 05-packages's and the four chapters share 9 distinct sources total."
---

> # ✅ MERGED 2026-08-01 — PR #1294, `origin/main = 62a8250d`
>
> Exact candidate `e0b8196de105172b3c5e22cfde577ae129d31e61`, tree
> `270e96e845c0d0777da7d588ed1be83bfe3fa905`. Librarian QA approved
> (`evt_5r0mwpm2rx31n`); Decision `dec_668pazj4248vv` resolved.
>
> **All four post-conditions predicted before the merge came back exact:**
> post-merge `origin/main^{tree}` = `270e96e8…`; the blob at
> `library/learn/reading-ken/03-assurance-and-trust.md` = `c8207af4…` (was
> `2be7aa3b…`); `git diff --name-only c8d978f5 62a8250d` = exactly that one
> path; and `gen-doc-status.sh --check` still exits 1 at **32 lines / 28 rows**
> with SHA-256 `349d545262be44c65a87b26a9aec730fdb9f23e7dbb9273fbf16c140ae8f75ce`
> — byte-identical to the baseline pinned before any work began.
>
> ⭐ **One page edited of four, and that is the correct outcome.** A drifted
> source does not entail a false claim in every citing page: the stale
> unconfined-authority claim in `03-assurance-and-trust.md` was false and is
> repaired; the other three chapters were reconciled against their current
> blobs and found still true. ⚠ The evidence that nothing was *missed* is not
> the edit count — it is the Librarian's independent rerun of the byte-identical
> drift control, which is what makes "reconciled, no repair needed" a
> measurement rather than a claim.
>
> ⚠ The first candidate `7e7cf099` was **rejected** by the Librarian for a stale
> claim in the replacement text; `e0b8196d` is the one-word corrective fold atop
> it. ⛔ No approval transferred — the Librarian re-reviewed the fresh SHA.

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
