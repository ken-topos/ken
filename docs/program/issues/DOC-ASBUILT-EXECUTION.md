---
id: DOC-ASBUILT-EXECUTION
title: "As-built slice 2 — reconcile 06-execution.md against its 16 drifted cited sources, the largest phase-A population"
status: merged
owner: doc
size: L
gate: none
depends_on: [DOC-ASBUILT-FRAGMENTS]
blocks: [DOC-ASBUILT-AUDIT]
github: null
origin: "Steward 2026-08-01, phase A slice 2 of DOC-ASBUILT-AUDIT, measured at origin/main 5619748c."
---

> # ✅ MERGED 2026-08-01 — `origin/main = f8ede8a3` (PR #1287)
>
> Candidate `a1f564219e6485d7a1f3ba7893a8219934a63dbb`, one file, `+17/−18`.
> Verified by **content**, ⛔ not ancestry — the publisher squashes. Both
> post-conditions were asserted **before** publishing and both came back exact:
> merged tree `6f413d8e`, `06-execution.md` blob `172ba03b`.
>
> Librarian QA approve `evt_66395f8wda9tk`; Decision `dec_49g4krcmgk7pv`
> resolved. Base staleness (`7a263d28` → `bb648bd2`) was answered by an **empty
> merge-base path intersection**.
>
> ⭐ **The 16th source was real and the ring absorbed it mid-turn.** The
> correction (`evt_3j54h1jaa6zh5`) reached the Librarian between its loading a
> 15-source QA basis and its superseding that basis — *"a 15-source report is
> incomplete."*
>
> ⚠ **The first candidate `2250fb22` was REJECTED on a real defect**, and the
> rejection is the more valuable artifact: the page asserted that an admitted,
> hole-free definition does not diverge, while `43-termination.md` §1 narrows SCT
> to **transparent** admission and §2 case 4 permits an opaque definition to
> diverge. ⭐ **The page falsified itself 20 lines later** — it documented the
> opaque SCT-rejected escape and still made the unqualified claim.
>
> # ▶ SLICE 2 OF PHASE A — THE HEAVIEST PAGE
>
> `library/learn/reading-ken/06-execution.md` carries ⭐ **16 distinct drifted
> sources across 25 citations** — the largest phase-A population, against slice
> 1's 9.
>
> ⚠ **AMENDED 2026-08-01 — 16, not 15.** This page **declares `fragments.md` as
> a source**, so slice 1's edit moved its blob and made it the **16th**. ⭐ That
> is the keystone effect arriving on schedule, ⛔ not new drift and ⛔ not a
> hard stop. Correction issued to the ring in-flight as `evt_3j54h1jaa6zh5`.
>
> ⚠ **`depends_on` names [[DOC-ASBUILT-FRAGMENTS]]** because slice 1 changes the
> expected drift count (below), ⛔ not because the pages overlap.

## ▶ THE FRAME IS WRITTEN

`docs/program/wp/DOC-ASBUILT-EXECUTION.md`. ⛔ **Read the frame, not this node**,
and read [[DOC-ASBUILT-AUDIT]] first — its two-phase law binds this slice.

## ⭐⭐ The expected drift count becomes 28 from slice 1 onward

`fragments.md` is the **only** `library/` page in `library/SOURCE-ATTESTATIONS`.
Slice 1 edits it, so its blob moves and it joins the drift population.

⇒ **The baseline is 28 from slice 1 onward and stays 28** for the rest of phase
A, because ⛔ **no other consuming page is attested** — editing them adds no row.
⚠ A candidate reporting **29** wrote something it should not have; one reporting
**27** is measured against a pre-slice-1 base.

## The 15 drifted sources, measured at `5619748c`

⚠ **Re-derive at pickup.**

| source | consumers | anchors cited here |
|---|---|---|
| `spec/40-runtime/42-evaluation.md` | 3 | **5** |
| `spec/40-runtime/45-native-backend.md` | 2 | **4** |
| `spec/40-runtime/43-termination.md` | 1 | 2 |
| `spec/40-runtime/44-capacity.md` | 1 | 2 |
| `spec/90-open-decisions.md` | 2 | — |
| `docs/program/07-catalog-style-guide.md` | 9 | 1 |
| `docs/program/issues/CAT-CAPEX.md` | 2 | — |
| `catalog/packages/Core/Logic/EmptyDec.ken.md` | 8 | — |
| `catalog/packages/Data/Sums/Combinators.ken.md` | 7 | — |
| `catalog/packages/Core/Logic/Transport.ken.md` | 6 | — |
| `catalog/packages/Tooling/Testing/Property.ken.md` | 6 | — |
| `crates/ken-interp/src/eval.rs` | 1 | — |
| `crates/ken-runtime/src/cranelift_backend.rs` | 2 | — |
| `crates/ken-cli/tests/px4b_native_production.rs` | 2 | — |
| `.github/workflows/ci.yml` | 1 | — |
| ⭐ `library/learn/reading-ken/fragments.md` | 7 | — | ← **added by slice 1** |

⛔ **No row is re-stamped here** — phase A writes no ledger. The `consumers`
column is why: most of these are shared, and a shared row cannot be re-stamped
until every consuming page is reconciled.

## ⭐ Why this page is high-yield

**Four sources are checked code or CI config**, so claims about what the
interpreter and backend *do* have a real oracle.

⚠ **`spec/90-open-decisions.md` is the sharpest risk.** Open decisions get
settled — a passage describing one as *open* may now describe a **settled** one,
and it reads as current prose, so nothing flags it.
