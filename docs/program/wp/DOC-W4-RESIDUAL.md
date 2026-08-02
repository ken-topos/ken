# WP frame — `DOC-W4-RESIDUAL` (Wave 4 slice 3, terminal)

Node: `docs/program/issues/DOC-W4-RESIDUAL.md`. Program:
`docs/program/12-documentation-program.md` §4b Wave 4. Owner: doc ring
(doc-leader, doc-author, Librarian as QA).

`depends_on: [DOC-W4-LANGUAGE]` is a genuine content dependency: slice 2's
merged residual report establishes the measurement method and the precedent
that an empty gap set closes a surface. Do not restate that method; apply it.

**Size S. One candidate. Doc-only.** Small **because the measurement may
license very little authoring** — see *Sizing*.

## Fixed inputs

Measured at `origin/main = 3b873896`.

| input | measured value |
|---|---|
| `library/reference/` | exists with **8 `toolchain/` pages only** |
| diagnostics mechanism | **SUPERSEDED by the 2026-08-02 amendment — this row's census was scoped to two paths and is not a repo-wide absence claim.** `KernelError`, `RuntimeTrapCode` and `IoErrorIdentityV1` exist; what is absent is a unified public registry |
| index pages | **zero.** No file under `library/` matching index, glossary, symbol, or keyword |
| `library/guide/proof-techniques.ken.md` | 474 lines |
| `library/learn/reading-ken/` | `01-anatomy` 121, `02-types-contracts-and-proofs` 142, `03-assurance-and-trust` 221, `04-effects-capabilities-and-authority` 171, `05-packages-and-provenance` 199, `06-execution` 239, `fragments` 125 |
| manifest `kind` census | 20 `reference`, 11 `explanatory`, 5 `how-to`, 4 `tutorial`, 2 `portal`, 1 `status` |
| the 20 `reference` records | 11 are `audience = ["agent-reader"]`, 8 are the new `toolchain/` pages, 1 is `fragments.md` |
| generation capability | slice 1's merged `D0` — **but its census could not see build scripts, and `crates/ken-host/build.rs:479` emits a generated `target_abi.rs`.** See the 2026-08-02 amendment before relying on any absence in it |

Reproduce, read-only:

```sh
find library/reference -type f -name '*.md' | sort
find library -iname '*index*' -o -iname '*glossary*' -o -iname '*symbol*' -o -iname '*keyword*'
git grep -lni 'diagnostic' -- crates/   # repo-wide; the two-path form was the defect
grep -oE 'kind = "[a-z-]+"' library/manifest.toml | sort | uniq -c
wc -l library/guide/*.md library/learn/reading-ken/*.md
```

## Three judgments, settled here

### 1. Audience closes the agents-corpus axis. Settled twice; do not re-derive

Eleven `reference` records are `audience = ["agent-reader"]`, including
`proof-and-trust.md` (verification) and `diagnose.md` (diagnostics). They
coexist with human-audience pages on the same subject, exactly as
`library/reference/toolchain/` coexists with `library/agents/core/toolchain.md`.
Slices 1 and 2 both settled this. **Spend no part of this round on it, and do
not propose folding either corpus into the other.**

### 2. The open question per surface is the human-audience explanatory corpus

That is the axis slice 2 measured and it is the axis here. A surface's residual
is whatever survives against `library/guide/` and
`library/learn/reading-ken/` — not against the spec, and not against the agents
corpus.

### 3. A missing mechanism closes a row on different grounds than a full corpus

Slice 2's rows closed because the material was **already delivered**.
Diagnostics may instead close because there is **nothing to derive a page
from** — no registry, no stable identity set. **These are different verdicts
and `D0` must not conflate them.** A row that closes for absence of mechanism
is a finding about Ken, not about the library, and it is reported as such.


## Amendment 2026-08-02 — hard stop 4 upheld; four rows re-scoped

Librarian QA blocked exact `4a6423d4` and was right on every count. The Steward
verified each claim independently before ruling.

### The instrument defect, which is this frame's and slice 1's

Slice 1's generation-capability census searched `scripts/gen-*` and CLI
dispatch. **That instrument cannot see a build script**, and its narrow result
was written up as the broad claim *"no target-fact command or generator."* It is
false: `crates/ken-host/build.rs:68-93` derives target, target OS, backend and
probed ABI facts, and `build.rs:479` writes a generated `OUT_DIR/target_abi.rs`.

This frame then carried the same narrow instrument into its own fixed inputs —
the diagnostics census names exactly two paths, `crates/ken-cli/src` and a
`crates/ken-diagnostics` that does not exist — and `D0` used the command the
frame supplied.

> ### BINDING — an absence claim needs a repo-wide census
>
> **`AC-4` is amended: a `not-producible` verdict must rest on a search that
> could find the mechanism wherever one could live, build scripts included.**
> Not `scripts/`, not one crate's `src`, not a path list authored from
> expectation. A census scoped to two paths cannot support a repo-wide claim,
> and this frame asked for one anyway.

### The four re-scoped rows

| row | what is now established | what the verdict must become |
|---|---|---|
| platform | a target-fact emitter exists (`build.rs:479`), host/Linux-only, failing closed off that lane | re-derive. Name what it emits and what it does not — there is still no ratified target-support contract or cross-target inventory. Probably `authored` with explicit `unavailable` labels, not `not-producible` |
| glossary | `spec/00-overview.md` is normative for terminology; §8 (line 216) is an explicit glossary | narrow to the missing **extraction path** into `library/`, not a missing source |
| keyword | `spec/30-surface/31-lexical.md §4` and `crates/ken-elaborator/src/lexer.rs`'s token enum are inventories | split the claim. "No generator" may survive; "no inventory" does not |
| diagnostics | `KernelError`, `RuntimeTrapCode`, `IoErrorIdentityV1` exist | narrow to **no unified public registry or derivation interface**, proved by a repo-wide census |

### What stands

**The `verification` and `runtime` rows stand at `none`.** They were measured
against the human-audience corpus, which is the correct instrument for them, and
nothing here touches them. Hard stop 1 remains available on the re-measured set.

### Carry-over

`DOC-W5-CAPABILITY`'s fixed inputs carry the same narrow generator census
(*"generators in `scripts/`: three"*). **Its `AC-2`/`AC-3` are read as
incorporating the repo-wide requirement above**, and its `D0` must not repeat
this error.

## Deliverables

- **D0 — the residual measurement**, one row per surface (verification,
  runtime, platform, diagnostics) and one per index (symbol, keyword,
  diagnostic, glossary). Columns: subject; the human-audience material measured
  against, cited by path and section; whether that material is **lookup-shaped**
  (complete over the subject, answerable without reading neighbours) or
  **explanatory** (narrative, motivating, read in order); and the verdict —
  `none`, `reclassify`, `not-producible`, or a named gap. Filed under
  `docs/program/`.
- **D1 — pages for named-gap rows only**, under
  `library/reference/<surface>/`. **No page for any row whose verdict is
  `none`, `reclassify`, or `not-producible`.**
- **D2 — the `reclassify` rows, reported not executed.** As in slice 2:
  changing a merged page's manifest `kind` is a Librarian call about the
  corpus. **Banned scope here.**
- **D3 — the `not-producible` rows, reported as findings about Ken**, each
  naming the missing mechanism and what would have to exist first. These are
  candidate inputs to a later program, not work this node performs.
- **D4 — manifest registration** for each new page: `kind = "reference"`,
  `authority = "derived-reference"`, audience, availability, `sources`,
  `validation`, owner.
- **D5 — availability labels.** Any documented behaviour that is partial or
  deferred today gets `partial`, `planned`, or `unavailable` and a sentence
  saying why.
- **D6 — ledger consistency**: if a newly cited path lacks a row, install the
  generator's `.proposed` output; never hand-write a row.

## Acceptance criteria

- **AC-1 — `D0` covers all eight rows** (four surfaces, four indexes), each
  with its verdict and the material it was measured against.
  *Control:* the eight-row list against `D0`'s rows.
- **AC-2 — every `D1` page traces to a `D0` named-gap row**, and every named-gap
  row has a page. The two sets match exactly, in both directions.
  *Control:* the `D1` path set against `D0`'s gap rows.
- **AC-3 — no `D1` page restates explanatory prose.** Where the reader needs
  the explanation, link it.
  *Control:* read each new page against the material `D0` paired it with.
- **AC-4 — `not-producible` is asserted against a measurement, not an
  impression.** Each such row cites the command or path census establishing the
  absence.
  *Control:* run each cited command.
- **AC-5 — every documented form was checked, not paraphrased.** Any current
  syntax displayed appears in a checked fence the toolchain accepts, or it is
  not displayed as current.
  *Control:* `ken check` over the candidate's fences, with output.
  **This AC has bitten this program once.** `DOC-W3-DEPDATA` hard stop 1 fired
  because a normative spec display block does not parse — task #192,
  `spec/50-stdlib/60-length-indexed-vectors.md` §1. **A form's spelling in the
  spec is not evidence the parser accepts it.** Run it.
- **AC-6 — no page describes a deferred lane in the present tense.** Any
  platform content states its availability explicitly.
  *Control:* read every availability claim in `D1` against `D5`'s labels.
- **AC-7 — every cited source is attested.** No page cites a path absent from
  `library/SOURCE-ATTESTATIONS`.
  *Control:* the `sources` path set minus the ledger's path set is empty.
- **AC-8 — no hand-written ledger row.** Re-running
  `scripts/gen-source-attestations.sh` produces a `.proposed` byte-identical to
  the installed ledger.
  *Control:* generate and `diff`; must be empty.
- **AC-9 — `library/STATUS.md` regenerated** by `scripts/gen-doc-status.sh` with
  no arguments.
  *Control:* rerun it, then `git diff --quiet -- library/STATUS.md`. Use
  `--quiet`; `--stat` always exits 0 and would pass vacuously.
- **AC-10 — the candidate touches `library/` and `docs/program/` only.** No
  `crates/`, no `spec/`, no `catalog/`, no CI path.
  *Control:* the path list.
- **AC-11 — CI green** on the merge. Workspace-green means green in CI, never a
  local `--workspace` run.

## Banned scope

- **No change to any merged `library/guide/` or `library/learn/` page**,
  including its manifest `kind`. `D2` reports; it does not edit.
- **No change to the agents corpus** (judgment 1).
- **No building of a diagnostic registry, generator, or CLI output format** —
  that is `crates/` work and slices 1 and 2 both banned it. `D3` reports the
  absence; it does not repair it.
- **No `library/releases/`** — absent until Ken has versioned public releases.
- **No reviving `VALIDATION_GATES`.** Steward-owned finding (task #188).
- **No campaign or WP history in a page.**
- **No new test asserting facts about source, catalog, or documentation lines**
  (operator test policy). `D0` is a review artifact, not a gate.

## Contention

The doc track runs concurrently with build work by standing operator exception
because it touches `library/` and `agent/` rather than `crates/`. This candidate
writes `library/` and `docs/program/` only.

`AC-5` needs the `ken` binary only **if** `D1` displays syntax. If every row
closes without a page, no build turn is needed at all. If one is:
`scripts/ken-cargo build -p ken-cli`, targeted, **never `--workspace`**. Probe
for the lock without blocking before taking it.

## Sizing

**Size `S`, and that is a prediction `D0` is allowed to falsify.** Slices 1 and
2 both found their subject already covered, and two of this slice's four
surfaces are suspect on mechanism before measurement begins. The likely shape is
a report plus zero to two pages.

If `D0` instead finds real gaps across several surfaces, **stop and route**:
that is an `M`/`L` wearing an `S` label, and the recut is the Steward's (hard
stop 3).

## Hard stops

Stop and route to the Steward, do not improvise, if any of these hold:

1. **Every row closes `none` / `reclassify` / `not-producible`.** Then Wave 4's
   remaining reference scope has no subject, this node closes on the report
   alone, and **that is a complete outcome, not an under-delivery.** It is also
   a finding about the wave worth stating plainly in `D0`.
2. **A form's spelling in `spec/` does not parse** (`AC-5`). Same class as task
   #192; it belongs to the enclave, and no page may display a form the
   toolchain rejects.
3. **`D0` finds gaps at a scale that makes this an `M` or `L`.** Report the row
   count; the recut is the Steward's, not a mid-turn scope expansion.
4. **`D0` contradicts slice 1's merged generation-capability report.** One of
   the two measurements is wrong, and a row must not be labelled from a
   contradiction.
5. **A newly cited path has no ledger row and the generator will not produce
   one.** Report the path.
