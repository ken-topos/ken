# WP frame — `DOC-W5-CAPABILITY` (Wave 5 precondition)

Node: `docs/program/issues/DOC-W5-CAPABILITY.md`. Program:
`docs/program/12-documentation-program.md` §4b Wave 5. Owner: doc ring
(doc-leader, doc-author, Librarian as QA).

**This node produces a report and a fork. It authors no reference page and
builds no generator.** Size S. One candidate. Doc-only.

`depends_on: [DOC-W4-RESIDUAL]` is a genuine content dependency: that node's
`not-producible` verdicts and their supporting censuses are the prior art this
report extends, and reusing them is cheaper and more consistent than
re-deriving them.

## Fixed inputs

Measured at `origin/main = 40f8757d`.

| input | measured value |
|---|---|
| `catalog/packages/` leaf packages | **39**, all literate `.ken.md`, under Application (3), Capability (19), Core (5), Data (11), Tooling (1) |
| other `catalog/` content | `catalog/examples/`, `catalog/guide/`; 47 `.md` and 1 bare `.ken` (`Tooling/Verification/ProofErasureBoundaryChecker.ken`) |
| CLI machine-readable output | **none**, per slice 1's merged `D0` |
| generators in `scripts/` | three — `gen-doc-status.sh`, `gen-progress.sh`, `gen-source-attestations.sh`. None extracts a declaration, type, law, effect, or dependency |
| Wave 4 terminal verdicts | six of eight rows `not-producible`, each for a missing inventory/registry/generator |
| the style/refinement contract | `docs/program/07-catalog-style-guide.md`; the campaign is `06-catalog-campaign.md` |

Reproduce, read-only:

```sh
find catalog/packages -name '*.md' ! -name 'README.md' | sort | wc -l
find catalog -type f | sed 's/.*\.//' | sort | uniq -c
ls scripts/gen-*.sh
```

## The nine fact classes, and the question asked of each

Wave 5's commitment names these. `D0` answers the same three questions for
every one of them, and **the unit of the answer is the fact class, never the
wave.**

1. subject · 2. declaration/type · 3. law · 4. effect/capability ·
5. assurance · 6. platform · 7. maturity · 8. dependency ·
9. reverse-dependency

For each: **(a)** is the fact *present* in the checked source at all, and where
— a declaration form, a manifest field, a convention in prose? **(b)** can it be
*extracted mechanically* today, and by exactly what command? **(c)** if not, what
is the smallest thing that would have to exist — and does that thing belong to
`crates/`, to the catalog's own conventions, or to the library?

## Two judgments, settled here

### 1. "Authored" is a legitimate answer, and it is not a failure

The program says so directly: *a fact we cannot generate gets authored and
labelled as authored.* **The report must not treat `not-extractable` as
equivalent to `not-documentable`.** A fact that is present in the source and
readable by a person, but not machine-extractable, is authorable today at the
cost of rot risk. **Say that cost out loud per class** — it is the input the
fork turns on, and it is different for 39 packages than for 8 CLI commands.

### 2. Do not design the generator

If a class needs extraction that does not exist, `D0` names **the smallest
missing capability** and stops. Sketching a schema, proposing an output format,
or scoping a `crates/` node is **banned scope** — that is Architect and
operator territory, and this report exists to inform it, not to pre-empt it.

## Deliverables

- **D0 — the nine-row capability report.** One row per fact class with columns:
  present in source (and where); mechanically extractable today (and by exactly
  what command, run and shown); if not, the smallest missing capability and its
  owner (`crates/` / catalog convention / library). Filed under `docs/program/`.
- **D1 — the per-class disposition**, each exactly one of:
  **`generated`** (extractable today, command shown), **`authored`** (present and
  readable, not extractable — with the rot cost stated), or **`blocked`** (not
  present in the source at all, so no page can carry it honestly).
- **D2 — the fork, stated for the operator.** Given `D1`, is Wave 5
  (a) authorable now at a stated cost, (b) blocked pending named `crates/` work,
  or (c) mixed — with the exact subset that is authorable now. **Recommend one,
  in one paragraph, with the reason.** Do not hedge across all three.
- **D3 — the per-package sample.** Apply the nine questions concretely to
  **three** named packages spanning different sections — one `Core`, one
  `Capability`, one `Data` — and show the result. A capability claim over 39
  packages that was never tried on one is an impression.
- **D4 — reuse ledger.** Where a `DOC-W4-RESIDUAL` census already answers a
  question, cite it rather than re-running it, and say which rows were reused.

## Acceptance criteria

- **AC-1 — all nine fact classes appear in `D0`** with all four columns filled.
  *Control:* the nine-class list against `D0`'s rows.
- **AC-2 — every `generated` disposition shows its command and that command's
  actual output.** A claim that a fact is extractable is not accepted on
  description.
  *Control:* run each cited command; the output must match what `D0` shows.
- **AC-3 — every `blocked` disposition names what is absent and cites the
  census establishing the absence.**
  *Control:* run each cited census command.
- **AC-4 — `D3`'s three packages span three different sections** and each is
  carried through all nine classes.
  *Control:* the three paths and the 27 resulting cells.
- **AC-5 — `D2` recommends exactly one of the three options.**
  *Control:* read it. A report that lists options without a recommendation
  fails this AC.
- **AC-6 — no generator, schema, or output format is designed** (judgment 2).
  *Control:* the candidate contains no proposed field list, schema, or format
  specification.
- **AC-7 — the candidate touches `docs/program/` only.** No `library/`, no
  `crates/`, no `catalog/`, no `spec/`, no CI path. **This node writes no
  library page**, so `library/` is out of scope entirely and `STATUS.md` and the
  attestation ledger are untouched.
  *Control:* the path list.
- **AC-8 — no new test asserting facts about source, catalog, or documentation
  lines** (operator test policy). `D0` is a review artifact, not a gate.
- **AC-9 — CI green** on the merge. Workspace-green means green in CI, never a
  local `--workspace` run.

## Banned scope

- **No `library/` page of any kind.** This node is a report to the program.
- **No generator, schema, output format, or `crates/` design** (judgment 2).
- **No change to `catalog/`**, including its conventions. If a catalog
  convention would have to change for a fact to be extractable, that is a `D0`
  finding, not an edit.
- **No Wave 5 slice framing.** The fork's answer is the input to that, and the
  framing is the Steward's.
- **No reviving `VALIDATION_GATES`.** Steward-owned finding (task #188).

## Contention

The doc track runs concurrently with build work by standing operator exception.
This candidate writes `docs/program/` only.

`AC-2` may need the `ken` binary to demonstrate that a command does or does not
emit a fact. If so: `scripts/ken-cargo build -p ken-cli`, targeted, **never
`--workspace`**. Probe for the lock without blocking before taking it. If every
class resolves from source inspection, no build turn is needed.

## Sizing

**Size `S`.** Nine rows, three sampled packages, one fork paragraph. The work is
bounded by the fact-class count, not by the 39 packages — `D3` samples three
deliberately.

If the measurement turns out to need a per-package pass over all 39 to answer
even one class, **stop and route** (hard stop 3): that is a different and larger
node, and the recut is the Steward's.

## Hard stops

Stop and route to the Steward, do not improvise, if any of these hold:

1. **Every class comes back `blocked`.** Then Wave 5 has no authorable subject
   at all, which is a program-level finding and an operator decision — not a
   reason to author something weaker.
2. **A class cannot be answered without designing the thing that would extract
   it.** That is judgment 2's boundary; report the boundary rather than crossing
   it.
3. **Answering a class requires a per-package pass over all 39** (see *Sizing*).
4. **`D0` contradicts slice 1's or `DOC-W4-RESIDUAL`'s merged capability
   findings.** One measurement is wrong and the fork must not rest on a
   contradiction.
5. **A catalog convention would have to change** for a fact to be extractable.
   That crosses into `06`/`07` territory and is not this node's to decide.
