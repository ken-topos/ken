---
id: DOC-ASBUILT-SOLUTIONS
title: "As-built slice 3 — reconcile exercises/solutions.md against its 11 drifted cited sources; every claim here is a worked answer a reader will run"
status: ready
owner: doc
size: L
gate: none
depends_on: [DOC-ASBUILT-FRAGMENTS]
blocks: [DOC-ASBUILT-AUDIT]
github: null
origin: "Steward 2026-08-01, phase A slice 3 of DOC-ASBUILT-AUDIT, measured at origin/main 7a263d28 after slice 1 merged."
---

> # ▶ SLICE 3 OF PHASE A — THE ONE WHERE A STALE CLAIM IS A BROKEN ANSWER
>
> `library/learn/exercises/solutions.md` carries **11 distinct drifted sources
> across 12 citations** — second only to slice 2's 16.
>
> ⭐ **What makes it different from every other phase-A page is the genre.** The
> other pages *describe* Ken. This one hands the reader a **worked answer** and
> implies it works. ⇒ A drifted source here does not merely make a sentence
> inaccurate — it can make a **solution that no longer runs** look authoritative.

## ▶ THE FRAME IS WRITTEN

`docs/program/wp/DOC-ASBUILT-SOLUTIONS.md`. ⛔ **Read the frame, not this node**,
and read [[DOC-ASBUILT-AUDIT]] first — its two-phase law binds this slice.

## ⭐ The drift baseline is **28**, and it does not move

`fragments.md` was the **only** attested `library/` page, and slice 1 already
edited it. ⇒ ⛔ **No remaining phase-A slice can add a row.** Before and after,
`scripts/gen-doc-status.sh --check` exits 1 with the **same 28 paths**.

⚠ **`solutions.md` is NOT attested**, so ⛔ editing it adds nothing — unlike
slice 1. **29 means something was written that should not have been; 27 means a
pre-slice-1 base.**

## The 11 drifted sources, measured at `7a263d28`

⚠ **Re-derive at pickup.** ⭐ When you do, **strip the `#anchor`** from each
manifest `sources` entry before matching — the gate does, and an exact-string
match silently under-counts every anchored citation.

| source | consumers | cited here |
|---|---|---|
| `catalog/packages/Core/Logic/EmptyDec.ken.md` | **8** | 1 |
| `catalog/packages/Data/Sums/Combinators.ken.md` | 7 | 1 |
| `spec/30-surface/36-effects.md` | 7 | 1 |
| `catalog/packages/Core/Logic/Transport.ken.md` | 6 | 1 |
| `catalog/packages/Tooling/Testing/Property.ken.md` | 6 | 1 |
| `spec/30-surface/30-taxonomy.md` | 2 | **2** |
| `crates/ken-cli/tests/px4b_native_production.rs` | 2 | 1 |
| `crates/ken-runtime/src/cranelift_backend.rs` | 2 | 1 |
| `docs/program/issues/CAT-CAPEX.md` | 2 | 1 |
| `spec/40-runtime/45-native-backend.md` | 2 | 1 |
| `spec/90-open-decisions.md` | 2 | 1 |

⛔ **No row is re-stamped here** — phase A writes no ledger. The `consumers`
column is why: a shared row cannot be re-stamped until every consuming page has
been reconciled.

## ⭐ Why this page is high-yield

**Five of the eleven are `catalog/**.ken.md` — checked code**, and this page's
claims about them are *worked solutions*, so the oracle is unusually direct:
⭐ the exercise either still resolves against the current package or it does not.

⚠ **`spec/90-open-decisions.md` carries the same sharpest-risk shape as slice 2**
— a decision described as *open* may now be **settled**, and settled-vs-open
reads as ordinary current prose, so ⛔ nothing flags it.
