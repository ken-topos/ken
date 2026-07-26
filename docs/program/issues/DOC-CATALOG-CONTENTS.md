---
id: DOC-CATALOG-CONTENTS
title: "Catalog entry format: rename the `## Index` heading to `## Contents` in 19 entries and remove the 16 reading-path sections, then close the derived artifacts"
status: ready
owner: doc
size: M
gate: none
depends_on: []
blocks: []
github: null
origin: Operator-directed catalog-format change, reported by the librarian (evt_66fx861j5xatx, thread thr_5wmt68pxnen2n) and confirmed mine by the operator 2026-07-26. Filed as a FRESH node rather than reusing the closed style WP, at the librarian's request — a closed WP has no live spine to hang a review or a publisher gate on. Steward-filed per COORDINATION §2.
---

> ## ▶ A DOC-ONLY FORMAT CHANGE WITH A DERIVED-ARTIFACT TAIL
>
> The visible work is a heading rename and a section removal. **The part that
> makes it an `M` rather than an `S` is everything downstream that is
> *generated*:** the source-attestation ledger, `REVISION`/`STATUS`, and the
> agent-pack token measures in `library/agents/manifest.toml`. ⛔ A candidate
> that changes the 19 entries and stops is incomplete.

## Bound base and independently verified scope

**Base: `origin/main = 870f5b65`.** The librarian derived the scope independently
and I re-measured every count against that tree rather than accepting the report.
⭐ **All of it reproduces exactly** — the report is confirmed, not merely trusted:

| claim | librarian | my measurement | verdict |
|---|---|---|---|
| exact `## Index` headings in `catalog/` | 19 | **19** (in 19 distinct files) | ✅ |
| `**Named reading paths**` occurrences | 14 | **14** | ✅ |
| `## Reading paths` headings | 2 | **2** — `catalog/guide/README.md:49`, `catalog/packages/Tooling/Testing/Property.ken.md:16` | ✅ |
| the four named consumers exist | 4 | **4**, all present | ✅ |
| `agent/playbooks/tools/write-ken.md` carries no format facts | unchanged | **0 hits** for `Index` / `reading path` | ✅ |
| `## Index` headings **outside** `catalog/` | (not claimed) | **0** — the rename cannot leak out of `catalog/` | ✅ |

⚠ **A count is a floor, not a population.** These reproduce against *my* notion of
the surface, which is the same notion the librarian used (exact heading match under
`catalog/`). ⇒ **The frame requires the ring to state its own denominator**, so a
missed entry shape surfaces as a disagreement rather than as silent partial
coverage.

## The three edit classes

1. **Rename** — `## Index` → `## Contents`, 19 files, exact-heading anchored.
2. **Remove** — 16 complete sections: the 14 `**Named reading paths**` blocks plus
   the 2 `## Reading paths` sections. ⛔ Whole sections, not just their headings.
3. **Reconcile the 4 consumers** — `docs/program/06-catalog-campaign.md`,
   `docs/program/07-catalog-style-guide.md`,
   `library/agents/tasks/author-package.md`,
   `library/learn/reading-ken/01-anatomy.md`.

## ⛔ Two traps that a competent sweep walks into

**(a) A bare `s/Index/Contents/` corrupts Ken source.** There are exactly **3**
non-heading `Index` tokens in `catalog/`, all in
`catalog/packages/Data/Collections/Derived.ken.md` — `class IndexedView A {`
(`:1155`), `instance IndexedView Unit {` (`:1234`), and a prose backtick reference
(`:1439`). **The first two are inside a Ken fence.** ⇒ A naive substitution
produces `ContentsedView`, breaking a declaration and its instance. **The rename
must be anchored to the exact heading line.**

**(b) The consumers do NOT spell the heading.** Only
`07-catalog-style-guide.md:41` contains `**Named reading paths**`. The other three
assert the format in **lowercase prose** — *"an index"*, *"named reading paths"*,
*"The index shows the whole entry at a glance"*. ⇒ ⛔ **A doc-author who greps for
`## Index` will conclude three of the four consumers need no change and be wrong.**
The consumer edits are prose edits.

## Exclusions — leave these alone

- ⛔ **Ken fences byte-for-byte.** No fence content changes for any reason.
- ⛔ **`spec/50-stdlib/README.md` is a *spec* index**, and
  `07-catalog-style-guide.md:242`–`:243` says it *"stays a spec index"*. That is
  not the catalog schema and does not move.
- ⛔ **Semantic/code uses of `Index`** (trap (a)) and **historical records**.
- ⛔ **`agent/playbooks/tools/write-ken.md`** — verified to carry no format facts;
  program `D3` keeps those product facts in the skill-consumed `author-package`
  module. ⚠ If the ring finds a format fact there, that is a **finding**, not a
  licence to edit it silently.
- ⛔ **`library-style` governs prose, not the catalog schema.**

## Derived closure — the part that is easy to skip

- **Source-attestation ledger:** regenerate **only after truth review**. 12
  attested source rows move.
- **`library/REVISION` and `library/STATUS.md`:** refresh.
- **`library/agents/manifest.toml`:** recompute token measures for the changed
  module **and its transitive packs**.

## Review and landing

The **librarian is the doc team's QA** (there is no `doc-qa` seat) and will
independently review the exact candidate before requesting the doc-only publisher
path. ⚠ Catalog-lane authority applies at that final gate, not during authoring.

⛔ **Not a merge request until the librarian's review returns.** Landing is the
normal path: Decision verified fresh from the object → doc-only publisher →
blob identity → retros.

## Concurrency

✅ **Contention-free with the live Runtime lane.** This touches `catalog/`,
`library/`, and `docs/program/`; `RT-FNSPLIT-B2E` touches `crates/`. The doc track
is the fleet's standing exception to single-threaded posture (operator,
2026-07-21).
