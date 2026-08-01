---
id: DOC-ASBUILT-SOLUTIONS
title: "As-built slice 3 — reconcile the exercise/solution PAIR against its 11 drifted cited sources; a stale claim here is a broken answer under a retired question"
status: merged
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
> # ✅ MERGED 2026-08-01 — `origin/main = 90ecf7f2` (PR #1292)
>
> Candidate `e163af4c328536ffbec69053cf87d43e61bd2cc3`, three paths, `+18/−15`.
> Verified by **content**, ⛔ not ancestry. Four post-conditions asserted
> **before** publishing, all four exact: merged tree `abf7f6a3`,
> `exercises.md` `2195b9f0`, `solutions.md` `3d4543fb`, `manifest.toml`
> `0d6fd7e1`. Base was current `main`, so no staleness question arose.
>
> Librarian QA approve; Decision `dec_40p3kcmmaday2` resolved.
>
> ⭐⭐ **THE `D5` GRANT HELD IN FACT, NOT ONLY IN ARGUMENT.** I re-ran all three
> `AC-6` controls myself in a detached worktree at the candidate rather than
> accepting the report: the manifest diff is **exactly** the two authorized
> entries on the one record; `SOURCE-ATTESTATIONS` and `STATUS.md` are untouched
> (zero files); and `gen-doc-status.sh --check` is **byte-identical** at 28 rows,
> SHA-256 `349d5452…` — ⭐ **the value pinned in the frame BEFORE the work
> began.**
>
> ⚠ **The first candidate `6a140197` was rejected on a real grounding defect**
> (`evt_3wej9paqbwf23`): 04.2's answer leaned on `Authority.ken.md` and
> `cat_capex_authority.rs` while the manifest record named neither. ⭐ The paired
> 04.1 repair in it was already correct — the rejection bought the **structural**
> half.
>
> ⭐⭐ **AMENDED 2026-08-01 — TWO pages, not one:**
> `library/learn/exercises/solutions.md` **and**
> `library/learn/exercises/exercises.md`, carrying **11 distinct drifted sources
> across 15 citations** — second only to slice 2's 16.
>
> ⚠ **`exercises.md` moved here from [[DOC-ASBUILT-READER]]** after doc-author
> fired hard stop 3 correctly, before any edit (`evt_41th5chexqwv`): 04.1 asks
> what `AFull` "does not yet confine", a premise the current
> `Capability/Filesystem/Errors.ken.md` has retired. ⇒ ⭐ **An exercise and its
> solution are one artifact split across two files** — repairing the solution
> alone writes a correct answer to a wrong question.
>
> ⭐ **The move cost nothing:** `exercises.md`'s three drifted sources were a
> **strict subset** of `solutions.md`'s 11, so the distinct population is
> unchanged. ⛔ It is **not** a new node — the only argument for one was this
> frame's own prose, which the node gate names as ungrounded.
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

⚠ **CORRECTED 2026-08-01 (Librarian, `evt_4gxaxq79kctr`) — it is FOUR, not
five.** The eleven split **4 `catalog/**.ken.md` + 2 checked Rust
(`cranelift_backend.rs`, `px4b_native_production.rs`) + 5 `spec`/`docs`.**
⭐ Six of the eleven are **checked code** either way, and this page's claims
about them are *worked solutions*, so the oracle is unusually direct: the
exercise either still resolves against the current artifact or it does not.
⛔ The miscount called two Rust files "catalog code" — different trees, different
oracles.

⚠ **`spec/90-open-decisions.md` carries the same sharpest-risk shape as slice 2**
— a decision described as *open* may now be **settled**, and settled-vs-open
reads as ordinary current prose, so ⛔ nothing flags it.
