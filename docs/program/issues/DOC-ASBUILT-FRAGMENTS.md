---
id: DOC-ASBUILT-FRAGMENTS
title: "As-built slice 1 — reconcile fragments.md against its 9 drifted cited sources; it is the keystone because 7 other documents cite it"
status: merged
owner: doc
size: M
gate: none
depends_on: []
blocks: [DOC-ASBUILT-AUDIT]
github: null
origin: "Steward 2026-08-01, phase A slice 1 of DOC-ASBUILT-AUDIT, measured at origin/main 2e6c4f15. Ordering ruled by the Steward on the measured fact that fragments.md is cited as a source by 7 documents."
---

> # ✅ MERGED 2026-08-01 — `origin/main = d7435f50` (PR #1282)
>
> Candidate `2c7807960bfa791997e58caef540e86a48a9a237`, one file, `+6/−7`.
> Blob-verified: `fragments.md` is `e1392c8a` on both the candidate and `main`.
> ⭐ The publisher squashes — this is a **content** check, ⛔ not ancestry.
>
> Librarian QA approve `evt_4rm0rj7f6w6w0`; Decision `dec_21fefm9aqmv89`
> resolved. Base staleness (`5619748c` → `433ef1b2`) was answered by an **empty
> merge-base intersection** plus a positive post-condition on the computed merge
> tree — predicted `30a509cc` before publishing, and `30a509cc` is what landed.
>
> ⭐ **`AC-5` came back 27 → 28 with `fragments.md` as the sole added path — the
> expected result**, and the drift baseline is **28 for every remaining phase-A
> slice**. See [[DOC-ASBUILT-EXECUTION]].

> # ⭐ SLICE 1 OF PHASE A — THE KEYSTONE
>
> `library/learn/reading-ken/fragments.md` is **both** a consuming document
> (**9** drifted cited sources) **and a cited source of 7 other documents.**
>
> ⛔ **Editing it moves its own blob OID, which drifts all 7 citing pages.** ⇒ It
> goes **first**, so the pages that cite it absorb its final content inside this
> same campaign instead of discovering fresh drift after they are done.
>
> ⚠ **True even for a locator-only edit** — an anchor or heading change moves
> the blob exactly as prose does.

## ▶ THE FRAME IS WRITTEN

`docs/program/wp/DOC-ASBUILT-FRAGMENTS.md`. ⛔ **Read the frame, not this node.**
⭐ Read [[DOC-ASBUILT-AUDIT]] first — its two-phase law binds this slice.

## The 9 drifted sources, measured at `2e6c4f15`

⚠ **Re-derive at pickup**; `main` moves.

| source | attested | actual | rows re-stampable here? |
|---|---|---|---|
| `catalog/packages/Core/Classes/EffectfulClasses.ken.md` | `1d650cf0fbb9` | `abdbe0a8800f` | — |
| `catalog/packages/Core/Classes/LawfulClasses.ken.md` | `bb82debfefd9` | `c1adc74f78e2` | — |
| `catalog/packages/Core/Classes/LawfulFunctors.ken.md` | `8d884c540dab` | `8b0e9b8fb639` | — |
| `catalog/packages/Data/Numeric/Nat/Order.ken.md` | `e31985f35e82` | `aa1c82f94cfc` | — |
| `catalog/packages/Core/Logic/EmptyDec.ken.md` | `06ed431a77b1` | `f236fe851ce7` | ⛔ shared with 7 pages |
| `catalog/packages/Core/Logic/Transport.ken.md` | `fce718c3145d` | `de9da7228782` | ⛔ shared with 5 pages |
| `catalog/packages/Data/Sums/Combinators.ken.md` | `7620482049fb` | `99a1e0763b69` | ⛔ shared with 6 pages |
| `catalog/packages/Tooling/Testing/Property.ken.md` | `12e5a0989421` | `13f04cb2fbba` | ⛔ shared with 5 pages |
| `docs/program/07-catalog-style-guide.md` | `c45bde4f70cb` | `f3251b0832b6` | ⛔ shared with 8 pages |

⭐ **The right-hand column is informational only.** ⛔ **No row is re-stamped in
this slice** — phase A writes no ledger at all ([[DOC-ASBUILT-AUDIT]]). Four of
these are sole-consumer and *would* be re-stampable in principle; they are still
deferred, because splitting the ledger act across slices is what makes a partial
re-stamp look complete.

⚠ **`07-catalog-style-guide.md` is cited twice**, at two different anchors
(`#13-path--import--the-normative-rule` and
`#3-code-block-roles-the-fence-taxonomy`). Both claims are owed.

## Why this page is high-yield

⭐ **Eight of the nine drifted sources are `catalog/**.ken.md` — checked code.**
A claim this page makes about them is verifiable against the current blob rather
than a matter of taste, so the review has a real oracle.
