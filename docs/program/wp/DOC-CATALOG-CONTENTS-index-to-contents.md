# `DOC-CATALOG-CONTENTS` — rename `## Index` to `## Contents`, remove the reading-path sections, close the derived artifacts

> **Frame authored by the Steward, 2026-07-26.** Node:
> `docs/program/issues/DOC-CATALOG-CONTENTS.md`. Owning ring: **doc** —
> `doc-leader` + `doc-author` + **`librarian` as QA** (there is no `doc-qa` seat).
> Size **M**, doc-only. Base: `origin/main` at release (see the kickoff mention for
> the exact SHA — ⛔ do not take a SHA from this file).

## 0. What this is, and what it is NOT

✅ **Is:** a catalog *schema* change — one heading renamed, one section class
removed, four prose consumers reconciled, and the generated artifacts closed.

⛔ **Is NOT:**

- a prose-style change (`library-style` governs prose and does not move);
- a change to any Ken source, fence content, spec index, or historical record;
- an occasion to improve the entries while you are in them. **Scope discipline is
  an acceptance criterion here**, because a format sweep across 19 files is exactly
  where unrelated edits become invisible.

## 1. Requirements this WP serves

Per `docs/program/15-requirements-and-acceptance-criteria.md`, every AC names the
requirement it applies. The requirements live in
`docs/program/06-catalog-campaign.md` under `## Requirements`.

| RQ | text (abbreviated — the program doc is authoritative) |
|---|---|
| `DOC-CATALOG.RQ-1` | Every catalog entry presents one consistent, named navigation section. |
| `DOC-CATALOG.RQ-2` | Entry format facts have exactly one authoring home; consumers agree with the entries. |
| `DOC-CATALOG.RQ-3` | Ken fence content is byte-preserved across any documentation-format change. |
| `DOC-CATALOG.RQ-4` | Generated library artifacts are consistent with the entries they describe. |

## 2. The measured scope — verified at `origin/main = 870f5b65`

⭐ The librarian derived this independently; **the Steward re-measured every count
rather than accepting the report**, and all of it reproduces:

| edit class | count | how to find it |
|---|---|---|
| rename `## Index` → `## Contents` | **19** files | exact heading match under `catalog/` |
| remove `**Named reading paths**` blocks | **14** | whole section, not the heading alone |
| remove `## Reading paths` sections | **2** | `catalog/guide/README.md:49`, `catalog/packages/Tooling/Testing/Property.ken.md:16` |
| reconcile consumers | **4** | see §4 — ⚠ **they do not spell the heading** |

✅ **`## Index` headings outside `catalog/`: 0.** The rename cannot leak out of the
catalog tree, which is why the exact-heading anchor is sufficient *for the rename*.

⚠ **A count is a floor, not a population.** These reproduce against one notion of
the surface: an exact `^## Index$` heading. ⇒ **`AC-4` requires the ring to state
its own denominator and method**, so an entry with a variant heading shape surfaces
as a disagreement instead of as silent partial coverage.

## 3. ⛔ Trap (a) — a bare `s/Index/Contents/` CORRUPTS KEN SOURCE

There are exactly **3** non-heading `Index` tokens under `catalog/`, all in
`catalog/packages/Data/Collections/Derived.ken.md`:

| line | text | inside a Ken fence? |
|---|---|---|
| `:1155` | `class IndexedView A {` | ✅ **yes** |
| `:1234` | `instance IndexedView Unit {` | ✅ **yes** |
| `:1439` | prose backtick reference to `` `IndexedView` `` | no |

⇒ **A naive substitution yields `ContentsedView`, breaking a class declaration and
its instance.** The rename must be anchored to the exact heading line.

⭐ **This is the required positive control (`AC-3`):** after the sweep, show that
all three `IndexedView` occurrences are **byte-identical** to the base. ⛔ "The
diff looks fine" is not the control; name the three lines.

## 4. ⛔ Trap (b) — the CONSUMERS DO NOT SPELL THE HEADING

Only `docs/program/07-catalog-style-guide.md:41` contains the literal
`**Named reading paths**`. **The other three assert the format in lowercase
prose**, measured:

| consumer | sites | shape |
|---|---|---|
| `docs/program/06-catalog-campaign.md` | `:60`–`:61`, `:155`, `:160`, `:167`, `:322` | *"named reading paths"*, *"catalog index"*, *"package index / navigation"* |
| `docs/program/07-catalog-style-guide.md` | `:40`, `:41`, `:48`, `:389`, `:405` | *"An **index**: anchor links to the sections below"*, *"the index links…"* |
| `library/agents/tasks/author-package.md` | `:18`, `:28`–`:29`, `:39` | *"Current entries carry an index, named reading paths"* |
| `library/learn/reading-ken/01-anatomy.md` | `:16`–`:17`, `:42` | *"an index, and named reading paths"*, *"The index shows the whole entry at a glance"* |

⇒ ⛔ **A doc-author who greps for `## Index` will conclude three of four consumers
need no change, and will be wrong.** The consumer edits are **prose** edits.

⚠ **And one site inside a consumer is an EXCLUSION:**
`07-catalog-style-guide.md:242`–`:243` says `spec/50-stdlib/README.md`
*"stays a spec index"*. **That is a spec index, not the catalog schema, and it does
not move.** A file-wide prose sweep of "index" in that file would wrongly rename
it.

## 5. Exclusions — do not touch

- ⛔ **Ken fences byte-for-byte** (`RQ-3`). No fence content changes, ever.
- ⛔ **`spec/50-stdlib/README.md`** and every spec-index reference to it.
- ⛔ **Semantic/code uses of `Index`** — the three in §3.
- ⛔ **Historical records** — retros, diary, archived program text.
- ⛔ **`agent/playbooks/tools/write-ken.md`** — verified to carry **no** format
  facts (0 hits for `Index` / `reading path`); program `D3` keeps those product
  facts in the skill-consumed `author-package` module. ⚠ If you find a format fact
  there, that is a **finding to report**, not a licence to edit it.
- ⛔ **`library-style`** governs prose, not the catalog schema.

## 6. Derived closure — the half that is easy to skip

⛔ **A candidate that edits the 19 entries and stops is incomplete.**

1. **Source-attestation ledger** — regenerate **only after truth review**. 12
   attested source rows move. ⚠ Regenerating before the review launders an
   unreviewed change into an attested artifact.
2. **`library/REVISION` and `library/STATUS.md`** — refresh.
3. **`library/agents/manifest.toml`** — recompute token measures for the changed
   module **and its transitive packs**. ⚠ Transitive: a pack that merely includes a
   changed module has a new measure.

## 7. Acceptance criteria

Short forms are unambiguous inside this frame; cross-WP references use
`DOC-CATALOG-CONTENTS.AC-<n>`.

| AC | criterion | RQ |
|---|---|---|
| `AC-1` | All **19** `## Index` headings under `catalog/` read `## Contents`; zero `^## Index$` headings remain anywhere in the repo. | `RQ-1` |
| `AC-2` | All **16** reading-path sections are removed **whole** — 14 `**Named reading paths**` blocks and the 2 `## Reading paths` sections. Zero residual references to a removed section remain in `catalog/`. | `RQ-1` |
| `AC-3` | ⭐ **Control:** the three `IndexedView` occurrences in `Derived.ken.md` (`:1155`, `:1234`, `:1439`) are **byte-identical** to the base, and **every Ken fence in every touched file is byte-identical**. Report the three lines explicitly. | `RQ-3` |
| `AC-4` | The four consumers no longer describe an index or named reading paths, **including their lowercase-prose assertions** (§4 site table). State the **denominator and method** used to find consumer sites. | `RQ-2` |
| `AC-5` | ⭐ **Control:** `spec/50-stdlib/README.md` and the `07-catalog-style-guide.md:242`–`:243` spec-index reference are **unchanged**. A sweep that renamed them fails this AC. | `RQ-2` |
| `AC-6` | `agent/playbooks/tools/write-ken.md` is **unchanged**; if a format fact is found there it is reported, not edited. | `RQ-2` |
| `AC-7` | Derived closure complete: ledger regenerated **after** truth review (12 rows), `REVISION`/`STATUS.md` refreshed, `manifest.toml` token measures recomputed for the changed module and its transitive packs. | `RQ-4` |
| `AC-8` | Scope discipline: the diff contains **no** unrelated edits. `git diff --stat` path list matches the §2/§4 inventory plus §6 derived artifacts, and nothing else. | `none` — a process control; it applies no definitional requirement |

⛔ **`AC-8`'s `none` is deliberate, not an omission.** Per `15-*.md` §4 an AC may
legitimately name no requirement when it is a process or inertness control; the
`none` cell exists so a mandatory-citation rule does not manufacture a plausible
wrong link.

## 8. Review and landing

The **librarian is this ring's QA** and holds a standing as-built mandate no build
QA has. It will independently review the exact candidate and then request the
**doc-only publisher path** from the Steward. ⚠ Catalog-lane authority applies at
that final gate, not during authoring.

⛔ **Do not treat a green local check as the gate.** The full-workspace build, the
`--locked` gate, and the conformance suite run **in CI**, never on this box.

## 9. This WP's own residual

⚠ Stated so it is visible rather than discovered later: **this WP does not verify
that the removed reading-path sections were unused by any reader-facing
navigation.** It verifies they are gone and that no reference to them survives in
`catalog/`. Whether any *external* artifact (a published page, an agent pack
prompt) depended on them is outside the measured surface. ⇒ If the ring finds such
a dependant, that is a finding to route, not a scope expansion to absorb.
