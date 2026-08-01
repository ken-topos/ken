---
id: DOC-ASBUILT-AUDIT
title: "As-built reconciliation — 27 cited sources have drifted from their attestations, so the library's currency claim is unbacked corpus-wide"
status: ready
owner: doc
size: L
gate: none
depends_on: [DOC-ASBUILT-FRAGMENTS, DOC-ASBUILT-EXECUTION, DOC-ASBUILT-SOLUTIONS, DOC-ASBUILT-CHAPTERS, DOC-ASBUILT-READER, DOC-ASBUILT-AGENTS]
blocks: []
github: null
origin: "Measured by the Steward 2026-08-01 at origin/main 2e6c4f15 with scripts/gen-doc-status.sh --check. Requested by the Librarian's restart brief (2026-08-01) and framed under the operator's doc-track restart. Steward-filed; agents cannot create tracked work per COORDINATION §2."
---

> # ⛔⛔ THIS NODE IS **NOT** PICKABLE WORK — IT IS THE CAMPAIGN LAW PLUS PHASE B
>
> ⚠ **Its `depends_on` must name every phase-A slice.** It listed only slice 1;
> when slice 1 merged, this node surfaced on the tracker's **releasable
> frontier** as if a team could start it. ⛔ It cannot — phase B is gated on
> **every** phase-A slice merging.
>
> ⭐ **Every new phase-A slice frame MUST add itself to the `depends_on` above**,
> in the same commit that files it. That is the only thing keeping this node off
> the frontier. (Steward error, corrected 2026-08-01.)

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

⚠ **Re-measured at `7a263d28`, after slice 1.** The original row was measured at
`2e6c4f15` and read 27 / 24 / 11 / 16 / 21-of-24.

| quantity | value |
|---|---|
| drifted cited sources | **28** (27 + `fragments.md`, which slice 1 edited) |
| consuming `library/` documents | **25** |
| drifted sources with **exactly one** consuming document | **11** |
| drifted sources shared by **two or more** documents | ⭐ **17** (one is cited by **9**) |
| documents in the largest connected component | ⚠ **22 of 25** (+ 27 of the 28 sources) |

⭐ **The graph has exactly two components, and the small one is genuinely
separable**: `library/README.md`, `library/agents/README.md`, and
`library/agents/evaluations/README.md` share
`docs/program/12-documentation-program.md` and nothing else.
⛔ **Phase B is still ONE act anyway** — splitting the ledger
write to bank 1 row of 28 early is exactly what makes a partial re-stamp look
complete. Recorded because it is measured, ⛔ not as licence to split.

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

**Slice 1 — [[DOC-ASBUILT-FRAGMENTS]]** ✅ **merged** (`d7435f50`, PR #1282).
**Slice 2 — [[DOC-ASBUILT-EXECUTION]]** ✅ **merged** (`f8ede8a3`, PR #1287).
**Slice 3 — [[DOC-ASBUILT-SOLUTIONS]]** ✅ framed, `ready`, **in flight** —
⭐ **amended 2026-08-01 to TWO pages** (`solutions.md` + `exercises.md`).
**Slice 4 — [[DOC-ASBUILT-CHAPTERS]]** ✅ framed and `ready`.
**Slice 5 — [[DOC-ASBUILT-READER]]** ✅ framed and `ready`.
**Slice 6 — [[DOC-ASBUILT-AGENTS]]** ✅ framed and `ready`.

⭐⭐ **PHASE A IS FULLY FRAMED. All 25 consuming documents are covered by exactly
six slices**, and ⛔ no further phase-A frame is owed. When slice 6 merges,
phase B is releasable.

## ⭐⭐ THE ROSTER BELOW IS RE-MEASURED AT `7a263d28` AND IS NOW STABLE

⚠ **Every count moved when slice 1 landed**, because seven documents declare
`fragments.md` as a source and its blob changed. The original roster was
measured before that.

⭐ **They will not move again for the rest of phase A.** `fragments.md` was the
**only** attested `library/` page, so ⛔ **no remaining phase-A slice can add a
row to the drift population** — editing any other consuming page adds nothing.
⇒ **These numbers are fixed until phase B**, and a slice measuring something
different at its base is measuring wrong.

| # | slice | distinct drifted sources | citations |
|---|---|---|---|
| — | `library/learn/reading-ken/fragments.md` ✅ **merged** | 9 | 10 |
| 2 | [[DOC-ASBUILT-EXECUTION]] — `06-execution.md` | **16** | 25 |
| 3 | [[DOC-ASBUILT-SOLUTIONS]] — ⭐ `exercises/solutions.md` **+ `exercises/exercises.md`** (amended) | **11** | 15 |
| 4 | [[DOC-ASBUILT-CHAPTERS]] — `01-anatomy.md` + `02-types-contracts-and-proofs.md` + `03-assurance-and-trust.md` + `05-packages-and-provenance.md` | ⭐ **9** (union) | 30 |
| 5 | [[DOC-ASBUILT-READER]] — the ⭐ **four** reader-facing entry pages (amended): `quickstart.md` · `introduction.md` · `learn/exercises/README.md` · `reading-ken/04-effects-capabilities-and-authority.md` | ⭐ **6** (union) | 11 |
| 6 | [[DOC-ASBUILT-AGENTS]] — the **thirteen**-page agents corpus: `library/README.md` + all twelve `library/agents/**` | ⭐ **7** (union) | 18 |

⭐ **Six slices cover all 25 consuming documents.** Cut by audience, then by
shared source set: 3 duplicate source reads across the 5/6 boundary
(`07-catalog-style-guide`, `36-effects`, `64-trust-model`) buy two
genre-coherent, separately-reviewable slices instead of one 18-page candidate
whose sweep nobody could audit.

> ### ⭐⭐ SLICES ARE CUT BY SHARED SOURCE SET, NOT ONE-PER-PAGE
>
> ⚠ **The original roster was one WP per document.** Slice 4 measured that out
> of it: `01-anatomy`'s sources are a **strict subset** of `05-packages`'s, and
> all four remaining `reading-ken` chapters share **9** distinct sources — fewer
> than slice 2 alone.
>
> ⭐ **The expensive act is reading a source at its current blob**, not editing a
> page. Four separate WPs would have read the same six sources four times.
>
> ⭐⭐ **And it is a correctness argument.** Pages of one genre **share claim
> classes**, so a class repaired in one page may survive in a sibling. Grouping
> them makes `D3`'s sweep cross-page; splitting them reproduces — *across* WPs,
> where no single QA pass can see it — the defect that sank two candidates
> *within* a page.
>
> ⇒ ⛔ **Do not cut a remaining slice one-page-per-WP by default.** Measure the
> union first.
>
> ### ⭐⭐ AND THE CUT CAN BE WRONG IN THE OTHER DIRECTION — BY **AUDIENCE**
>
> ⚠ **Slices 3 and 5 were cut by audience, and that split a PAIR.** `exercises.md`
> reads as reader-facing; `solutions.md` reads as answer-key. But **an exercise
> and its solution are one artifact split across two files** — slice 3 hard-stopped
> before any edit (`evt_41th5chexqwv`) on an exercise whose *premise* a catalog
> source had retired. ⛔ There is no solution-only repair of that: it writes a
> correct answer to a wrong question, which reads as reconciled and is worse than
> the stale pair.
>
> ⇒ ⭐ **`exercises.md` moved from slice 5 into slice 3**, and the move was free —
> its three drifted sources were a **strict subset** of `solutions.md`'s 11.
> ⭐⭐ **The general rule: before cutting by genre or audience, ask whether the
> two pages are ONE ARTIFACT.** Shared *sources* argue for grouping; shared
> *identity* requires it.
>
> ⛔ **The alternative — a new node for the exercise repair — was rejected on the
> node gate.** The only constraint arguing for one was **this campaign's own frame
> prose**, which the gate names explicitly as ungrounded, *including prose the
> Steward wrote*.

**25 consuming documents**, and ⭐ **all 28 drifted sources are cited by at least
one of them** — none is reachable only through the generated `library/STATUS.md`,
whose own declared sources the gate exempts.

⚠ **The counts are citations of drifted sources, not effort.** A page with one
drifted source still owes a whole-page claim sweep — the obligation terminates
at *the claim in its consuming page*, so the unit of review is the page.

⚠⚠ **When measuring a page's population yourself, STRIP THE `#anchor` before
matching a manifest `sources` entry against the drift list.** ⭐ The gate does
(`REQUIRED_PATHS` is "every unique manifest-cited path, anchor stripped"), and a
naive exact-string match silently under-counts every anchored citation — it
reported 8 for `solutions.md` against the true 11. ⛔ An under-count reads as a
smaller slice, not as an error.

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
