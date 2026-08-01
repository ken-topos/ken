# WP frame — `DOC-W3-GUIDE` (Wave 3 slice 1, the conceptual guide)

Node: `docs/program/issues/DOC-W3-GUIDE.md`. Program:
`docs/program/12-documentation-program.md` §4b Wave 3. Owner: doc ring
(doc-leader, doc-author, Librarian as QA).

Wave 3's §3 fence precondition was **reconciled** on 2026-08-01
(`12-documentation-program.md` §4). This is the first slice released under it.

**Size M. One candidate. Doc-only.**

## Fixed inputs

Measured at `origin/main = f31e8d94`. Every control below is stated over
**content** — a fence count, a path set, a row count — never over a commit SHA.
The publisher squashes; re-derive each at the candidate base, all read-only.

| input | measured value |
|---|---|
| `catalog/guide/` | 4 files: `surface-reference.ken.md` (628 lines), `proof-techniques.ken.md` (479), `decomposition-abstraction.ken.md` (170), `README.md` (120, plain `.md`) |
| checked fences | **40 total** — `surface-reference` 17 example + 7 reject; `proof-techniques` 8 + 5; `decomposition-abstraction` 3 + 0; `README.md` **0** |
| `library/` fences | **7 across 25 documents** — six plain ```` ```ken ````, one `ken example`; **zero `.ken.md` files** |
| already-attested | the three literate guide files are **already ledger rows** (`library/SOURCE-ATTESTATIONS` rows 5-7); `README.md` is not |
| already-cited | **4** `catalog/guide` citations in `library/manifest.toml` — two anchored into `proof-techniques`, one into `surface-reference`, one whole-file to `decomposition-abstraction` |
| `library/guide/` | **does not exist** |
| detector | `checked_examples_detector_rejects_invalid_example_and_stale_reject` is live, one of the 26 `#[test]`s in `crates/ken-cli/tests/library_documentation_gates.rs` |
| gate registry | `VALIDATION_GATES` is **unreachable** — it and all 11 gate fns appear exactly twice each (definition + registry row); no test iterates it |

Reproduce, read-only:

```sh
grep -c '^```ken example$' catalog/guide/*.ken.md
grep -c '^```ken reject$'  catalog/guide/*.ken.md
grep -n 'catalog/guide' library/manifest.toml library/SOURCE-ATTESTATIONS
```

## Four judgments, settled here so the ring does not stop for them

### 1. The material MOVES, and the fences must survive the move

**D2 is ratified and settles the direction** (`12-documentation-program.md`
§2): migration is *subsumptive* — `catalog/guide/` moves into `library/` and
does **not** persist alongside, leaving pointers rather than a second
maintained guide. *Subsume-don't-proliferate.*

⚠ So the shape the Wave 1 spine uses — prose in `library/` citing a checked
file that stays in `catalog/` — is **not available for this material.** The
spine cites `catalog/packages/`, which persists by design. `catalog/guide/`
does not. Do not reach for the citation pattern here; it looks like the
established convention and it would quietly defeat D2.

⇒ The obligation is the harder one §3 states: **the 40 fences must still be
checked once they land in `library/`.** Not preserved in place — preserved
*through* the move. That conservation is what this WP has to demonstrate.

**The one thing genuinely unknown, to settle EARLY.** `ken check` selects
literate extraction **by the `.ken.md` suffix**, so a checked page in
`library/guide/` would carry that suffix. But `library/` holds **zero `.ken.md`
files** today and **7 ken fences across 25 documents**: the corpus has never
registered a literate document. Whether a `.ken.md` can be a `manifest.toml`
document record, with the generated-status and attestation machinery working
over it, is **unverified**. Answer it before authoring at volume — it decides
the shape of every page in this slice. It is the Librarian's call on corpus
convention (hard stop 3).

### 2. The verification is migration-local, and its control is the fence COUNT

The precondition's mutation-proof half is **already discharged and live** — the
detector test plants an invalid `ken example` and a stale `ken reject` and
requires the specific diagnostic from each, on every PR. **Do not rebuild it.**

What this WP owes is the other half: exercising the fences over the files it
touches, at candidate time. **Not a standing CI gate** — the registry that once
did that is unreachable, so "restoring the gate" would mean building new
coupling and walking back the operator's no-live-doc-CI-coupling ruling.

**The binding control is the count, not the exit status.** A checker run that
returns success is satisfied vacuously by a file whose fences became plain code
blocks — which is exactly the failure §3 is about. So the control is a
**conservation law across the move**: **40 fences before, 40 after**, per
originating file 17+7 / 8+5 / 3+0, each still exercised by the real extractor
wherever it now lives.

### 3. Phase B's manifest ban was phase-B-local and does not bind here

`DOC-ASBUILT-LEDGER` banned new `sources` entries and new document records.
**That ban was scoped to that node** — a terminal re-stamp must not move the
population it is stamping. This WP registers new documents and may cite new
anchors; that is its job. If a new citation names a path with no ledger row,
the row is added by running `scripts/gen-source-attestations.sh` and installing
its `.proposed` output — never by hand-writing a row.

For the three literate guide files specifically, **no new row is needed**: rows
already exist, and the ledger is keyed per path, not per citation.

### 4. A red fence baseline is a finding to route, not a repair to make

Nothing has exercised those 40 fences since the registry went inert, so some may
already fail. **Establish the baseline before authoring** (D0). If a fence is
red at the base, that is a pre-existing defect in `catalog/`: report it, do not
fix it inside this candidate. Repairing catalog code inside a documentation WP
mixes an unreviewed source change into a doc merge and makes the verification
record unauditable.

## Deliverables

- **D0 — the fence baseline**, recorded before any authoring: per-file
  `ken example` / `ken reject` counts and the pass/fail result of running the
  real extractor over each of the three literate files at the candidate base.
  Use the technique the live test uses — copy the source to a `.ken.md` path and
  run `ken check` on it (`run_checked_markdown` is the reference); `ken check`
  selects literate extraction by suffix.
- **D1 — `library/guide/` conceptual pages** carrying the migrated material, in
  a form that keeps its checked fences checked. Scope the page set to what the
  existing guide material actually supports; a page with no grounding is a
  Wave 3 gap to report, not prose to invent.
- **D2 — `catalog/guide/` reduced to pointers**, per D2 of the program: the
  migrated material does not persist alongside as a second maintained guide.
- **D3 — manifest registration** for each new page: `kind`, `sources`,
  `validation`, authority class and availability label, consistent with the
  Wave 1 records.
- **D4 — the migration-local verification record**: the per-file fence counts
  and extractor results at the candidate, set beside D0's baseline values, and
  the mapping from each originating file to where its fences now live.
- **D5 — ledger consistency**: if any newly cited path lacks a row, the
  generator-produced rows installed from `.proposed`. If a path stops being
  cited, its row leaves the ledger the same way.

## Acceptance criteria

- **AC-1 — fences are CONSERVED across the move: 40 before, 40 after.**
  Per originating file, 17+7 / 8+5 / 3+0 arrive intact at their new home.
  *Control:* the two `grep -c` counts over the migrated destinations, summed
  and per file, compared to D0.
  **This is the AC that catches a migration which quietly demoted fences to
  plain code blocks — the exact §3 failure, which a green checker run does not
  catch because it passes vacuously when there is nothing left to check.**
- **AC-2 — every conserved fence is actually exercised.** The real extractor
  runs over each destination page and the count it extracts is non-zero and
  equals AC-1's count for that page.
  *Control:* the extractor invocation plus its reported fence count. A run that
  extracts zero fences from a page that should carry seven is a failure here
  even though the command succeeded.
- **AC-3 — the extractor result is no worse than baseline.** Every fence green
  at D0 is green at the candidate.
  *Control:* rerun D0's command; compare per fence. A fence red at D0 and still
  red is a routed finding, not a failure of this AC.
- **AC-4 — every page's cited sources are attested.** No page cites a
  path absent from `library/SOURCE-ATTESTATIONS`.
  *Control:* the path set of `sources` minus the ledger's path set is empty.
- **AC-5 — no hand-written ledger row.** If D5 changed rows, re-running
  `scripts/gen-source-attestations.sh` produces a `.proposed` byte-identical to
  the installed ledger.
  *Control:* generate and `diff`; must be empty.
- **AC-6 — `library/STATUS.md` regenerated** by `scripts/gen-doc-status.sh` with
  no arguments, if the manifest changed. Generated, never hand-edited.
  *Control:* rerun it, then `git diff --quiet -- library/STATUS.md`. Use
  `--quiet`; `--stat` always exits 0 and would pass vacuously.
- **AC-7 — the candidate touches `library/`, `docs/program/`, and
  `catalog/guide/` only.** No `crates/`, no `spec/`, and nothing under
  `catalog/` outside `guide/`.
  *Control:* the path list. `catalog/guide/` is in scope **only** for D2's
  reduction to pointers — not for editing the Ken code in a fence.
- **AC-8 — CI green** on the merge. Workspace-green means green in CI, never a
  local `--workspace` run.

## Banned scope

- **No new CI gate, and no reviving `VALIDATION_GATES`.** The registry being
  unreachable is a real finding and it is **Steward-owned**, not this WP's to
  fix. Adding a standing corpus gate here walks back an operator ruling.
- **No editing the Ken code inside a fence.** The fences move; their contents do
  not change. A fence whose code is altered in transit is no longer the checked
  material that was reviewed, and AC-1's count would not notice.
- **No repairing a red fence.** Route it (judgment 4).
- **No how-to recipes.** They are the next slice: their input is actual
  diagnostics and recurring fleet failures, which is a separate research act.
  A recipe invented from an imagined task list is exactly what the program
  forbids.
- **No campaign or WP history in an explanatory page** — a reader does not care
  which WP landed a feature (Wave 3 exit property).
- **No new test asserting facts about source, catalog, or documentation lines**
  (operator test policy). D4 is a review artifact, not a gate.

## Contention

The doc track runs concurrently with build work by standing operator exception
because it touches `library/` and `agent/` rather than `crates/`. This candidate
writes `library/`, `docs/program/`, and `catalog/guide/` — the last is a
documentation directory, not Ken source the build rings compile.

D0 and AC-2/AC-3 need the `ken` binary, so they need a build turn:
`scripts/ken-cargo build -p ken-cli`, targeted, never `--workspace`. **The
runtime ring holds the build turn for the capstone.** Take the turn when it is
free; if it is contended, do D1 authoring first and run D0 before the candidate
— but **do not hand off a candidate without D0**, because AC-3 has no meaning
without a baseline.

## Hard stops

Stop and route to the Steward, do not improvise, if any of these hold:

1. **A fence is red at the base** (judgment 4). Report the file, the fence, and
   the diagnostic.
2. **The fence count at the base is not 40**, or not 17+7 / 8+5 / 3+0 per file.
   The corpus moved under the frame; the controls need re-deriving.
3. **A checked page cannot be registered in `library/`** — a `.ken.md` (or
   whatever form keeps the fences live) will not go into `manifest.toml` as a
   document record, or the status/attestation machinery does not work over it.
   This is the frame's one genuinely unverified premise (judgment 1). It is the
   Librarian's call on corpus convention, and it decides the shape of every page
   in the slice, so **raise it before authoring at volume, not at candidate
   time.**
4. **A newly cited path has no ledger row and the generator will not produce
   one.** Report the path.
5. **Conserving the fences and satisfying D2 turn out to conflict** — the only
   way to keep them checked is to leave the material in `catalog/guide/`, which
   D2 forbids. That is a collision between two ratified decisions and it is the
   Steward's to resolve, not something to split the difference on.
