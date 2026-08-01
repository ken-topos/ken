# WP frame — `DOC-W3-HOWTO` (Wave 3 slice 2, the how-to recipes)

Node: `docs/program/issues/DOC-W3-HOWTO.md`. Program:
`docs/program/12-documentation-program.md` §4b Wave 3. Owner: doc ring
(doc-leader, doc-author, Librarian as QA).

Depends on `DOC-W3-GUIDE`: recipes link the conceptual pages that slice
produces, so they cannot be authored against a `library/guide/` that does not
exist yet.

**Size M. One candidate. Doc-only.**

## Fixed inputs

Measured at `origin/main = 0cde815f`, which is **before** slice 1 lands. Slice 1
adds `library/guide/` pages and moves `catalog/guide/`, so re-derive the
`library/` figures at this WP's actual base — the two marked *pre-slice-1* will
have moved, and that is expected, not a corpus drift finding.

| input | measured value |
|---|---|
| `library/how-to/` | **does not exist** |
| how-to records | **zero** — the `kind` census across the 26 records is explanatory 7, portal 2, reference 12, status 1, tutorial 4 *(pre-slice-1)* |
| the vocabulary | `manifest.toml`'s header already admits `kind = "how-to"` and `authority = "how-to"` in its closed set, since Wave 0. Nothing needs widening |
| the task surface | **exactly 7 subcommands** — `run`, `check`, `native-build`, `fmt`, `repl`, `version`, `help` (`crates/ken-cli/src/main.rs`, `print_help`) |
| the refusal population | **order 300+**, concentrated in the elaborator: formatted-message sites matching refusal wording are 279 in `ken-elaborator/src`, 51 in `ken-runtime/src`, 2 in `ken-verify/src`, **0** in `ken-cli/src` |
| the spine | `library/learn/reading-ken/` holds **7 files** — six numbered chapters plus `fragments.md` — covering six of Wave 3's seven guide subjects *(pre-slice-1)* |
| checked-page mechanism | settled: the Librarian ruled a checked `library/**/*.ken.md` is a valid first-class manifest document and the status/attestation machinery works over it (`evt_453cj700kw59a`, 2026-08-01) |

Reproduce, read-only:

```sh
sed -n '/^fn print_help/,/^}/p' crates/ken-cli/src/main.rs
grep -n 'kind\s*=' library/manifest.toml | awk -F'"' '{print $2}' | sort | uniq -c
for c in ken-elaborator ken-runtime ken-verify ken-cli; do printf '%-16s ' "$c"; \
  grep -rn 'format!("' --include=*.rs crates/$c/src/ \
  | grep -ci 'error\|cannot\|expected\|unknown\|invalid\|refus\|not \|must '; done
```

**On that last figure — read it as an order of magnitude, not a catalogue.** It
counts message-construction sites whose text matches refusal wording; it is not
a diagnostic registry, and Ken has none. It is in the frame for exactly one
purpose: to establish that the refusal population is **two orders of magnitude
larger than the task surface**, which is what judgment 1 turns on. Do not try to
make this number exact, and do not derive a coverage target from it.

## Three judgments, settled here so the ring does not stop for them

### 1. A recipe is scoped by a TASK, and the task enumeration is the CLI

The program says the recipes are driven by *actual diagnostics and recurring
failures, not by an imagined task list.* Taken literally as "one recipe per
diagnostic" that is unachievable and would be the wrong shape anyway: the
refusal population is order 300+ against a task surface of 7.

⇒ **Enumerate by what the toolchain lets a person do — `run`, `check`,
`native-build`, `fmt`, `repl` — and let the diagnostics ground the recipes
rather than index them.** A recipe answers *"I am trying to do X and the
toolchain refused"*, and the diagnostic is its evidence that X is a real task
someone hits, not an imagined one.

**`version` and `help` do not get recipes.** They are self-describing; a page
explaining how to run `ken help` is the imagined-task-list failure in its purest
form.

### 2. Every recipe is grounded in something the toolchain actually does

**A quoted diagnostic must be one the toolchain really emits, produced and
observed — not paraphrased from reading the source, and never composed to fit
the prose.** The same for a claimed remedy: if a recipe says a change makes the
refusal go away, that has to have been run.

This is the whole content control for the slice, and it is what "actual
diagnostics" means operationally. A recipe whose failure or fix was never
observed is indistinguishable, to a reader, from one that was — which is why it
has to be a rule rather than a preference.

⇒ **If a task has no real refusal and no checked artifact behind it, that task
is a Wave 3 gap to report, not prose to invent** (hard stop 1).

### 3. Recipes DIRECT WORK. They do not explain, and they do not teach

Wave 3's exit property: *tutorials teach, how-tos direct work, and conceptual
pages explain; no single page is forced to do all three.*

⚠ **The pressure to explain will be strong here and it is a defect, not
helpfulness.** By the time this slice is authored, the same subjects are covered
twice already — the Wave 1 spine has six chapters, and slice 1 adds conceptual
guide pages. A recipe that re-derives that background is the drift-prone
duplicate D1 exists to prevent, and it will look authoritative while it rots.

⇒ **Where a reader needs background, link the spine or guide page and continue
with the steps.** If a recipe cannot be written without restating a concept,
that is a signal the concept page is missing or wrong — route it (hard stop 3),
do not absorb it into the recipe.

## Deliverables

- **D0 — the grounding table**, recorded before authoring: for each candidate
  recipe, the subcommand it sits on, and the exact refusal text or checked
  artifact that grounds it, **as observed by running the toolchain**. This is
  what makes the page set defensible; it is also what shows which candidate
  recipes have to be dropped.
- **D1 — `library/how-to/` recipes**, one page per grounded task. Scope the set
  to what D0 supports. A short set of real recipes is the correct outcome; a
  complete-looking set with invented ones is not.
- **D2 — manifest registration** for each page: `kind = "how-to"`,
  `authority = "how-to"`, audience, availability label, `sources`, `validation`,
  owner — consistent with the existing records' shape.
- **D3 — the availability labels**, per recipe. A task that does not work today
  gets `partial`, `planned`, or `unavailable` **and a sentence saying why**.
  Present-tense prose about a lane that does not exist is aspirational syntax by
  another name.
- **D4 — ledger consistency**: if a newly cited path lacks a row, install the
  generator's `.proposed` output; never hand-write a row.

## Acceptance criteria

- **AC-1 — every recipe is grounded, and the grounding was observed.** Each page
  in D1 appears in D0's table with a refusal text or checked artifact that was
  produced by running the toolchain.
  *Control:* the D0 table, with the command run and its actual output per row.
  **A row citing a message located by reading the source, rather than by running
  the toolchain, fails this AC** — that is the difference between a recipe and a
  plausible-looking recipe, and nothing downstream can tell them apart.
- **AC-2 — every quoted diagnostic is reproducible from the page itself.** A
  reader following the page's own steps gets the message the page quotes.
  *Control:* run each page's steps as written; compare to the quoted text.
- **AC-3 — no recipe restates spine or guide material.** Background is linked,
  not re-derived.
  *Control:* each page's background references resolve to a `learn/` or `guide/`
  page, and no page carries a conceptual section of its own.
- **AC-4 — every page carries an authority class and an availability label**,
  and any label other than `current` carries its one-sentence reason.
  *Control:* the manifest records plus the pages' own scope notes.
- **AC-5 — every cited source is attested.** No page cites a path absent from
  `library/SOURCE-ATTESTATIONS`.
  *Control:* the `sources` path set minus the ledger's path set is empty.
- **AC-6 — no hand-written ledger row.** Re-running
  `scripts/gen-source-attestations.sh` produces a `.proposed` byte-identical to
  the installed ledger.
  *Control:* generate and `diff`; must be empty.
- **AC-7 — `library/STATUS.md` regenerated** by `scripts/gen-doc-status.sh` with
  no arguments, if the manifest changed. Generated, never hand-edited.
  *Control:* rerun it, then `git diff --quiet -- library/STATUS.md`. Use
  `--quiet`; `--stat` always exits 0 and would pass vacuously.
- **AC-8 — the candidate touches `library/` and `docs/program/` only.** No
  `crates/`, no `spec/`, no `catalog/`, no CI path.
  *Control:* the path list.
- **AC-9 — CI green** on the merge. Workspace-green means green in CI, never a
  local `--workspace` run.

## Banned scope

- **No fixing a diagnostic that reads badly.** A confusing message found while
  grounding a recipe is a real finding and it belongs to whichever team owns
  that crate. Report it; do not edit `crates/`. A documentation candidate that
  carries a source change makes the verification record unauditable.
- **No new CI gate, and no reviving `VALIDATION_GATES`.** The registry is
  unreachable code and that finding is **Steward-owned**. ⚠ `manifest.toml`'s
  own header still calls it "the single source of truth for vocabulary,
  applicability, and executable validation" — **that comment is false today**,
  and correcting it is not this WP's job either.
- **No re-explaining what the spine or the guide covers** (judgment 3).
- **No inventing a diagnostic, a failure mode, or a remedy** (judgment 2). This
  is the one that will not announce itself in review.
- **No recipes for `version` or `help`.**
- **No campaign or WP history in a page** — a reader does not care which WP
  landed a feature (Wave 3 exit property).
- **No new test asserting facts about source, catalog, or documentation lines**
  (operator test policy). D0 is a review artifact, not a gate.

## Contention

The doc track runs concurrently with build work by standing operator exception
because it touches `library/` and `agent/` rather than `crates/`. This candidate
writes `library/` and `docs/program/` only.

D0, AC-1, and AC-2 need the `ken` binary, so they need a build turn:
`scripts/ken-cargo build -p ken-cli`, targeted, **never `--workspace`**. Probe
for the lock without blocking before taking it; the runtime ring holds the build
turn for the capstone. If it is contended, draft page structure first — but
**do not hand off a candidate without D0**, because AC-1 has no meaning without
it and every other content control descends from it.

## Hard stops

Stop and route to the Steward, do not improvise, if any of these hold:

1. **A candidate recipe has no real grounding** — no refusal you can produce, no
   checked artifact behind it. Report the task and what you tried; do not write
   the page from what the source suggests the behaviour should be.
2. **The task surface is not the 7 subcommands** at your base. The CLI moved
   under the frame; the enumeration in judgment 1 needs re-deriving.
3. **A recipe cannot be written without restating a concept**, because the
   concept page is missing, wrong, or says something the toolchain does not do.
   That is a slice-1 or spine finding and it is worth more than the recipe.
4. **A newly cited path has no ledger row and the generator will not produce
   one.** Report the path.
5. **The honest answer to a whole recipe is "this does not work yet."** One step
   of a task being unavailable is a label and a sentence (D3). An entire task
   being unavailable is a Wave 3 scope question — report it rather than
   publishing a page that describes a lane that does not exist.
