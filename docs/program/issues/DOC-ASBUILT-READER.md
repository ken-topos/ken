---
id: DOC-ASBUILT-READER
title: "As-built slice 5 — reconcile the four reader-facing entry pages against their 6 shared drifted sources"
status: merged
owner: doc
size: M
gate: none
depends_on: [DOC-ASBUILT-FRAGMENTS]
blocks: [DOC-ASBUILT-AUDIT]
github: null
origin: "Steward 2026-08-01, phase A slice 5 of DOC-ASBUILT-AUDIT, measured at origin/main 4ab1c23e. Cut by audience, per the campaign's shared-source-set rule."
---

> # ✅ MERGED 2026-08-01 — PR #1297, `origin/main = ac187ada`
>
> Exact candidate `ba6edab3eac11ae2f9280722ff3b7c16d8155544`, tree
> `1a0e0071946ebbf17153366d39ab86f8c621d2d3`. Librarian QA approved
> (`evt_4mja28bqz4t95`) after an independent 6-source/11-citation re-derivation;
> Decision `dec_2xvqe6cxzrj85` resolved.
>
> **Repaired:** the stale-curriculum-progress class in `library/quickstart.md`
> and `library/introduction.md` (+3/−3). `exercises/README.md` and
> `04-effects-capabilities-and-authority.md` were reconciled and found still
> true.
>
> **All five post-conditions predicted before the merge came back exact**, drift
> check included: exit 1, 32 lines / 28 rows, SHA-256
> `349d545262be44c65a87b26a9aec730fdb9f23e7dbb9273fbf16c140ae8f75ce`.
>
> ⚠⚠ **The base-staleness read was misleading and is worth remembering.**
> `git diff origin/main <candidate>` reported **four** files and **−58** lines,
> because the candidate's merge-base was `2e94d907` while `main` had advanced to
> `92185f00` with the slice-6 route amendment (PR #1296). ⛔ That is the
> candidate *predating* those commits, **not reverting them** — the publisher
> merges, it does not reset. Resolved correctly: candidate-vs-base touched the
> two `library/` pages, main-vs-base touched the two `docs/program/` files, and
> the **intersection was empty** ⇒ immaterial. ⭐ I also asserted an explicit
> **anti-revert post-condition** — that both amendments still be present on
> `main` after the merge — and they were.
>
> ⭐⭐ **Hard stop 3 fired here and was upheld** (`evt_27hg7sk8q4pjd`). The
> repaired class survives in `library/README.md:18`, outside this slice. It was
> **routed, not followed**, and the route is durable in
> [[DOC-ASBUILT-AGENTS]]'s frame. ⛔ The candidate was **not** respun: its bytes
> were correct, and what was stale was the `D4` *report* — a channel deliverable
> — which `evt_3ncp204a779cf` corrected.

> # ▶ SLICE 5 OF PHASE A — THE READER-FACING ENTRY PAGES
>
> `library/quickstart.md` · `library/introduction.md` ·
> `library/learn/exercises/README.md` ·
> `library/learn/reading-ken/04-effects-capabilities-and-authority.md`
>
> ⭐ **4 pages · 6 distinct drifted sources · 11 citations.**
>
> ⚠⚠ **AMENDED 2026-08-01 — `library/learn/exercises/exercises.md` MOVED OUT**
> to [[DOC-ASBUILT-SOLUTIONS]], taking `EmptyDec`, `Combinators` and `Property`
> with it. ⭐ **An exercise and its solution are one artifact split across two
> files**, and those three sources were already in slice 3 — so the move was free
> and ⛔ **no source is read twice across the two slices** any more.
> ⚠ `library/learn/exercises/README.md` **stayed** — it is still yours.

## ▶ THE FRAME IS WRITTEN

`docs/program/wp/DOC-ASBUILT-READER.md`. ⛔ **Read the frame, not this node**,
and read [[DOC-ASBUILT-AUDIT]] first — its two-phase law binds this slice.

## ⭐ Why these four

**Cut by audience, per the campaign's shared-source-set rule** — these are the
pages a *person* meets first. ⚠ They share claim classes with each other (an
introduction and a quickstart make the same promises), so ⭐ **`D3`'s sweep is
cross-page**, exactly as in [[DOC-ASBUILT-CHAPTERS]].

⚠ **`04-effects-capabilities-and-authority.md` is here for a specific reason.**
`DOC-CAP-ASBUILT` already repaired its *capability-exemplar* claims. ⛔ That WP
did **not** discharge its other two drifted sources — `spec/30-surface/36-effects.md`
(cited **3×**) and the now-moved `fragments.md`. ⭐ **A merged neighbour WP is
not a clearance for the axes it did not name.**

## The 9 sources, measured at `4ab1c23e`

⚠ **Re-derive at pickup**, ⭐ **stripping the `#anchor`** before matching.

| source | cites here | consumers |
|---|---|---|
| `spec/30-surface/36-effects.md` | **4** | 7 |
| `spec/00-overview.md` | 2 | 2 |
| `library/learn/reading-ken/fragments.md` | 2 | 7 |
| `catalog/guide/decomposition-abstraction.ken.md` | 1 | 1 |
| `docs/program/07-catalog-style-guide.md` | 1 | 9 |
| `spec/60-security/64-trust-model.md` | 1 | 3 |
| `catalog/packages/Core/Logic/EmptyDec.ken.md` | 1 | 8 |
| `catalog/packages/Data/Sums/Combinators.ken.md` | 1 | 7 |
| `catalog/packages/Tooling/Testing/Property.ken.md` | 1 | 6 |

⛔ **No row is re-stamped here** — phase A writes no ledger.
⚠ `catalog/guide/decomposition-abstraction.ken.md` has **exactly one** consumer
(`quickstart.md`) and is the **only** sole-consumer source in this slice; ⛔ it
is still not re-stampable, because phase A writes none at all.

## ⭐ The drift baseline is **28** and does not move

⛔ None of these four is attested, so editing them adds no row. **29 means
something was written that should not have been; 27 means a pre-slice-1 base.**
