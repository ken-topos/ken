---
id: DOC-ASBUILT-AUDIT
title: "As-built reconciliation — 27 cited sources have drifted from their attestations, so the library's currency claim is unbacked corpus-wide"
status: ready
owner: doc
size: L
gate: none
depends_on: [DOC-ASBUILT-FRAGMENTS]
blocks: []
github: null
origin: "Measured by the Steward 2026-08-01 at origin/main 2e6c4f15 with scripts/gen-doc-status.sh --check. Requested by the Librarian's restart brief (2026-08-01) and framed under the operator's doc-track restart. Steward-filed; agents cannot create tracked work per COORDINATION §2."
---

> # ⛔⛔ THE GATE IS ALREADY RED ON `main`, AND THAT IS THE SUBJECT
>
> `scripts/gen-doc-status.sh --check` **exits 1** at `2e6c4f15` with **27 cited
> sources whose content no longer matches their attested blob.** ⭐ This is not
> a regression introduced by any WP — it is accumulated drift the doc track's
> five idle days made visible.
>
> ⚠ **Do not "fix" it by regenerating the ledger.** `library/SOURCE-ATTESTATIONS`
> is the *evidence*; re-stamping it without reading the claims converts an
> honest red gate into a false green one.

## The measurement

| quantity | value |
|---|---|
| drifted cited sources | **27** |
| consuming `library/` documents | **24** |
| drifted sources with **exactly one** consuming document | **11** |
| drifted sources shared by **two or more** documents | ⭐ **16** (one is cited by **9**) |
| documents in the largest connected component | ⚠ **21 of 24** |

Reproduce it — read-only, no build turn:

```sh
scripts/gen-doc-status.sh --check   # exits 1; the drift block is the population
```

## ⭐⭐ Why this is TWO phases, not one — the ledger act is terminal

⚠ **`library/SOURCE-ATTESTATIONS` has one row per PATH, corpus-wide** — ⛔ not
one row per (document, source) pair. ⇒ **A row may be re-stamped only once
*every* document that cites it has been reconciled.**

With 16 of 27 sources shared, and 21 of 24 documents in a single connected
component, ⛔ **no per-page slicing can clear the gate incrementally.** The
arithmetic does not permit it.

| phase | what it does | ledger | gate |
|---|---|---|---|
| **A — per-page claim reconciliation** | one slice per consuming document: read each drifted source's **current blob**, check every claim the page derives from it, repair what is false | ⛔ **no ledger writes at all** | stays **red** — ⭐ expected, not a failure |
| **B — terminal re-stamp** | one slice: run `scripts/gen-source-attestations.sh`, review, commit, regenerate `library/STATUS.md` | ✅ **all 27 rows at once** | goes **green** |

⭐ **The standing ban is on the TIMING, not the tool.** Running
`gen-source-attestations.sh` today launders 27 unreviewed claims. Running it as
phase B — after every consuming page has been reconciled — is the *correct and
intended* use, and is the only act that can make the gate green.

⚠ **A phase-A slice that leaves the gate red has not failed.** Do not let a red
`--check` be read as a rejection of a phase-A candidate; its acceptance is about
claims, not about gate colour.

## ⭐ Ordering — `fragments.md` is the keystone and goes FIRST

`library/learn/reading-ken/fragments.md` is **both** a consuming document (9
drifted sources) **and a cited source of 7 other documents.**

⇒ ⛔ **Editing it moves its own blob OID, which drifts all 7 citing pages.** If
it is reconciled late, it manufactures fresh drift in pages already declared
done. ⭐ It is therefore [[DOC-ASBUILT-FRAGMENTS]], slice 1, and it lands before
any page that cites it.

⚠ **This is a real constraint even for a locator-only edit** — an anchor or
heading change moves the blob just as a prose change does.

## The slice roster

**Slice 1 — [[DOC-ASBUILT-FRAGMENTS]]** ✅ framed and `ready`.

**Phase A remainder — frames owed by the Steward, cut by consuming document**,
heaviest first (drifted-source count in parentheses):

| # | document | (n) |
|---|---|---|
| 2 | `library/learn/reading-ken/06-execution.md` | 15 |
| 3 | `library/learn/exercises/solutions.md` | 11 |
| 4 | `library/learn/reading-ken/05-packages-and-provenance.md` | 6 |
| 5 | `library/learn/reading-ken/02-types-contracts-and-proofs.md` | 5 |
| 6 | `library/learn/reading-ken/03-assurance-and-trust.md` | 4 |
| 7 | `library/learn/reading-ken/01-anatomy.md` · `library/quickstart.md` · `library/agents/core/write-ken.md` · `library/learn/exercises/exercises.md` | 3 each |
| 8 | `library/introduction.md` (2) + the twelve 1-source pages | 1–2 |

⚠ **The counts are per-document citations of drifted sources, not effort.** A
page with one drifted source can still owe a whole-page claim sweep — the
Librarian's rule is that the obligation terminates at *the claim in its
consuming page*, so the unit of review is the page.

**Phase B — one terminal slice**, gated on every phase-A slice merging.
⛔ Do not frame or start it early.

## ⛔ What this node does NOT authorize

- ⛔ **No ledger write in phase A**, for any reason, including "the row is
  obviously fine."
- ⛔ **No `library/STATUS.md` regeneration** before phase B — `gen-doc-status.sh`
  cannot even complete while the 27 stand.
- ⛔ **No new CI gate asserting facts about source lines** (operator test
  policy).
- ⛔ **No spec/catalog/crates edits.** ⚠ If a page's claim is false because the
  **source** is wrong, that is a finding to route, ⛔ not a repair to make here.

## Relationship to [[DOC-CAP-ASBUILT]]

⭐ **`DOC-CAP-ASBUILT` is NOT part of this campaign and must not be coupled to
it.** It declares two **never-attested** paths for the first time — an addition
of two rows, ⛔ not a re-stamp of a drifted one. That is a different operation
and it is explicitly authorized in that WP's own `D5`/`AC-6`.
