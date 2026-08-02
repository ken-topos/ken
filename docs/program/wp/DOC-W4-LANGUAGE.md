# WP frame — `DOC-W4-LANGUAGE` (Wave 4 slice 2)

Node: `docs/program/issues/DOC-W4-LANGUAGE.md`. Program:
`docs/program/12-documentation-program.md` §4b Wave 4. Owner: doc ring
(doc-leader, doc-author, Librarian as QA).

`depends_on: [DOC-W4-TOOLCHAIN]` is a **genuine content dependency.** Slice 1's
`D0` says, per Wave 4 fact class, whether the toolchain can emit it today. This
slice's syntax facts are one of those classes, and `D0`'s answer decides whether
they are labelled generated or authored. Do not start before `D0` is on `main`.

**Size S. One candidate. Doc-only.** The size is small **because `D0` may find
the residual is small or empty** — see *Sizing*.

## Fixed inputs

Measured at `origin/main = 09931340`.

| input | measured value |
|---|---|
| `library/reference/` | does not exist on `main`; `DOC-W4-TOOLCHAIN` creates it with 8 `toolchain/` pages |
| `library/guide/surface-reference.ken.md` | **625 lines**, `kind = "explanatory"`, `authority = "explanatory"`, human audience. Nine numbered sections, one per language form: purity keywords, `def`, `data`/`match`, refinement types, `class`/`instance`, effect rows, proof claims, local `let`, the literate format. Also carries `Design notes`, `Findings`, `References` |
| `library/agents/core/read-ken.md` | 89 lines, `kind = "reference"`, `authority = "derived-reference"`, **`audience = ["agent-reader"]`**. Ten fixed sections (use-when, prerequisites, current capability, canonical forms, invariants, decision procedure, failure signatures, validation, authority, known-unavailable) |
| syntax generation | **none.** Slice 1 measured three generators in `scripts/`, none extracting a declaration, keyword, or syntax production. Confirm against the merged `D0` rather than this row |
| normative source | `spec/30-surface/` |

Reproduce, read-only:

```sh
ls library/reference/ 2>/dev/null || echo "absent"
grep -nE '^#{1,3} ' library/guide/surface-reference.ken.md
grep -nE '^#{1,3} ' library/agents/core/read-ken.md
sed -n '/surface-reference/,/^owner/p' library/manifest.toml
sed -n '/agents\/core\/read-ken/,/^owner/p' library/manifest.toml
```

## Two judgments, settled here

### 1. Audience discriminates the agents surface. That axis is closed

`library/agents/core/read-ken.md` is already a language reference. It coexists
with a human-audience page for the same reason `library/reference/toolchain/`
coexists with `library/agents/core/toolchain.md`: `audience` differs, and the
agents corpus has its own fixed ten-section contract that a human reference does
not follow. **Do not spend the round re-deriving this**, and do not propose
folding one into the other.

### 2. `surface-reference.ken.md` is the open question, and `D0` answers it

Same audience, same subject, adjacent kind, and it is *called* a surface
reference. Slice 1's judgment 1 separates reference from how-to; it does not
separate a reference from an explanatory page organised per form.

⇒ **This slice may not author a page for a form until `D0` has said what the
residual is for that form.** Authoring first and measuring after is how a
program re-authors material it already has — the failure Wave 3 made twice, on
L5 and V3, and did not make a third time.

## Deliverables

- **D0 — the per-form residual measurement**, one row per language form in the
  fixed-inputs table, with columns: form; what `surface-reference` §n actually
  delivers for it; is that content **lookup-shaped** (complete over the form,
  answerable without reading neighbours) or **explanatory** (narrative, motivating,
  read in order); and therefore the residual — `none`, `reclassify`, or a named
  gap. Filed under `docs/program/`.
- **D1 — `library/reference/language/`**, one entry per form whose `D0` residual
  is a named gap. **No entry for a form whose residual is `none`.**
- **D2 — the `reclassify` rows, reported not executed.** If `D0` finds a form
  where `surface-reference`'s section is already lookup-shaped and only its
  `kind` is wrong, that is a manifest classification question affecting a merged
  page. Report it as a finding with the section named. **Changing a merged
  page's `kind` is banned scope** — it is a Librarian call about the corpus, not
  a side effect of authoring a new page.
- **D3 — manifest registration** for each new page: `kind = "reference"`,
  `authority = "derived-reference"`, audience, availability, `sources`,
  `validation`, owner — consistent with the existing `reference` records.
- **D4 — availability labels.** Any form whose behaviour is partial today gets
  `partial`, `planned`, or `unavailable` and a sentence saying why.
- **D5 — ledger consistency**: if a newly cited path lacks a row, install the
  generator's `.proposed` output; never hand-write a row.

## Acceptance criteria

- **AC-1 — `D0` covers every form in the fixed-inputs table**, each with its
  residual verdict and the section it was measured against.
  *Control:* the nine-row table against `D0`'s rows.
- **AC-2 — every `D1` page traces to a `D0` gap row.** A page whose form's
  residual is `none` or `reclassify` fails this AC.
  *Control:* the `D1` path set against `D0`'s gap rows; the two must match
  exactly, in both directions.
- **AC-3 — no `D1` page restates `surface-reference` prose.** Where a reader
  needs the explanation, link it.
  *Control:* read each new entry against the section `D0` paired it with.
- **AC-4 — the syntax labelling matches the merged `D0` of slice 1.** A syntax
  fact this slice cannot generate is labelled authored, never dressed as
  generated.
  *Control:* slice 1's `D0` row for syntax, against this slice's labels.
- **AC-5 — every documented form was checked, not paraphrased.** A form's
  declaration syntax appears in a checked fence that the toolchain accepts, or
  it is not displayed as current syntax.
  *Control:* `ken check` over the candidate's fences, with output.
  ⚠ **This AC has already bitten this program once.** `DOC-W3-DEPDATA` hard stop
  1 fired because a *normative spec display block* does not parse — see task
  #192 and `spec/50-stdlib/60-length-indexed-vectors.md` §1. **A form's spelling
  in `spec/30-surface/` is not evidence that the parser accepts it.** Run it.
- **AC-6 — every cited source is attested.** No page cites a path absent from
  `library/SOURCE-ATTESTATIONS`.
  *Control:* the `sources` path set minus the ledger's path set is empty.
- **AC-7 — no hand-written ledger row.** Re-running
  `scripts/gen-source-attestations.sh` produces a `.proposed` byte-identical to
  the installed ledger.
  *Control:* generate and `diff`; must be empty.
- **AC-8 — `library/STATUS.md` regenerated** by `scripts/gen-doc-status.sh` with
  no arguments.
  *Control:* rerun it, then `git diff --quiet -- library/STATUS.md`. Use
  `--quiet`; `--stat` always exits 0 and would pass vacuously.
- **AC-9 — the candidate touches `library/` and `docs/program/` only.** No
  `crates/`, no `spec/`, no `catalog/`, no CI path.
  *Control:* the path list.
- **AC-10 — CI green** on the merge. Workspace-green means green in CI, never a
  local `--workspace` run.

## Banned scope

- **No change to `library/guide/surface-reference.ken.md`**, including its
  manifest `kind`. `D2` reports; it does not edit.
- **No change to the agents corpus** (judgment 1).
- **No other Wave 4 surface** — verification, runtime, platform, diagnostics and
  the four indexes are later slices.
- **No generator, no CLI output format, no diagnostic registry** — `crates/`
  work, and slice 1 already banned it for the same reason.
- **No reviving `VALIDATION_GATES`.** Steward-owned finding.
- **No campaign or WP history in a page.**
- **No new test asserting facts about source, catalog, or documentation lines**
  (operator test policy). `D0` is a review artifact, not a gate.

## Contention

The doc track runs concurrently with build work by standing operator exception
because it touches `library/` and `agent/` rather than `crates/`. This candidate
writes `library/` and `docs/program/` only.

`AC-5` needs the `ken` binary, so it needs a build turn:
`scripts/ken-cargo build -p ken-cli`, targeted, **never `--workspace`**. Probe
for the lock without blocking before taking it.

## Sizing

**Size `S`, and that is a prediction `D0` is allowed to falsify.** Nine forms
already have a 625-line page. If `D0` finds most residuals are `none`, this
slice ships two or three entries and a report — which is the correct outcome,
not an under-delivery. If `D0` finds most are real gaps, **stop and route**:
that is a size `M`/`L` node wearing an `S` label, and the recut is the Steward's
(hard stop 4).

## Hard stops

Stop and route to the Steward, do not improvise, if any of these hold:

1. **`D0` finds every form's residual is `none`.** Then this node has no
   subject, and the honest outcome is `closed` plus the `D2` reclassification
   findings — not a set of pages authored to justify the node.
2. **A form's spelling in `spec/30-surface/` does not parse** (`AC-5`). That is
   a spec defect of the same class as task #192, it belongs to the enclave, and
   the page must not display a form the toolchain rejects.
3. **`D0`'s answer contradicts slice 1's merged `D0`** on the syntax fact class.
   One of the two measurements is wrong and a page must not be labelled from a
   contradiction.
4. **`D0` finds gaps at a scale that makes this an `M` or `L`.** Report the row
   count; the recut is the Steward's, not a mid-turn scope expansion.
5. **A newly cited path has no ledger row and the generator will not produce
   one.** Report the path.
