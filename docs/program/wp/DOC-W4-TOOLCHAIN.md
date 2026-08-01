# WP frame — `DOC-W4-TOOLCHAIN` (Wave 4 slice 1)

Node: `docs/program/issues/DOC-W4-TOOLCHAIN.md`. Program:
`docs/program/12-documentation-program.md` §4b Wave 4. Owner: doc ring
(doc-leader, doc-author, Librarian as QA).

`depends_on: [DOC-W3-DEPDATA]` is **ring-capacity sequencing, not a content
dependency.** Nothing here needs the dependent-data page. The edge exists so
this node does not enter the frontier while the ring already has a live
candidate.

**Size M. One candidate. Doc-only.**

## Two deliverable classes, and the order matters

This slice does one authored thing and one measured thing, and the measured
thing outlives it:

1. **D0 — the generation-capability report.** Wave 4 commits that syntax, CLI,
   target, and public-declaration facts are *generated*. Nothing in the repo
   generates them. Every later Wave 4 slice rests on the answer, so it is
   produced here, once, as a durable artifact.
2. **The toolchain reference pages**, on the one Wave 4 surface that needs no
   generator.

⚠ **D0 is not a research detour that delays the pages.** It is one measurement
pass over a small surface and it decides the labelling of everything after it.
Do it first; do not let it grow.

## Fixed inputs

Measured at `origin/main = 7fa65b20`.

| input | measured value |
|---|---|
| `library/reference/` | **does not exist** |
| generators in `scripts/` | exactly 3 — `gen-doc-status.sh`, `gen-progress.sh`, `gen-source-attestations.sh`. **None** extracts a declaration, keyword, syntax production, or CLI surface |
| CLI machine-readable output | **none** — no `--format`, no JSON emission anywhere in `crates/ken-cli/src/main.rs` |
| the task surface | **7 subcommands** — `run`, `check`, `native-build`, `fmt`, `repl`, `version`, `help` (`print_help`) |
| the flag surface | **five spellings across three options** — `fmt --check`; `--version` / `-V`; `--help` / `-h`. `native-build`'s `<output-dir>` is a **positional argument, not a flag** |
| `print_help` completeness | **`print_help` omits every global flag.** `--version`, `-V`, `--help`, and `-h` are all accepted (`main.rs:30`, `main.rs:38`) and none appears in the help text. Help is *silent* about them, not contradictory |
| exit statuses | **three classes, not two** — 29 sites `exit(1)`, **2 sites `exit(2)`**, 1 site `exit(outcome.exit_status)`. The `exit(2)` arms are `RunError::EntrypointAbiUnavailable` (`main.rs:319`) and `RunError::ConsoleAbiUnavailable` (`main.rs:334`) |
| diagnostics | **no registry.** Order 300+ formatted-message sites, no index (measured for `DOC-W3-HOWTO`) |
| existing task pages | `library/how-to/` holds 5 recipes over this same CLI, merged at `c777d2d4` |

Reproduce, read-only:

```sh
ls scripts/ | grep '^gen-'
grep -n 'json\|--format\|Json' crates/ken-cli/src/main.rs
sed -n '/^fn print_help/,/^}/p' crates/ken-cli/src/main.rs
grep -oE 'process::exit\([0-9a-z_.]+\)' crates/ken-cli/src/main.rs | sort | uniq -c
grep -nE '"--version"|"-V"|"--help"|"-h"|"--check"' crates/ken-cli/src/main.rs
```

⚠ **The exit probe must enumerate, never count one value.** An earlier version of
this frame used `grep -c 'process::exit(1)'` and concluded the surface was
uniform. **A count of one value cannot establish the absence of another** — it
returns a number whatever else is there, so it would have reported success on
every possible source. The `sort | uniq -c` form is what found the two `exit(2)`
arms. This is the same defect class the frame's own judgment 2 warns about, so
treat it as a worked example rather than as a stale line.

## Three judgments, settled here so the ring does not stop for them

### 1. A reference page answers a LOOKUP. A how-to directs a TASK

`library/how-to/` already documents this CLI, and the two must not merge into
one shape. The discriminator is what the reader arrives with:

- **how-to** — *"I am trying to do X and the toolchain refused."* Ordered steps,
  one task, a real diagnostic.
- **reference** — *"what does `fmt --check` do, and what does it exit with?"*
  Complete over its surface, no narrative, answerable without reading
  neighbouring entries.

⇒ **A reference entry that walks a reader through a task is a duplicate of a
page that already exists**, and it will drift from it. Where a reader plainly
needs the procedure, **link the how-to**.

### 2. Every documented behaviour is observed, not read out of the source

Same rule that governed `DOC-W3-HOWTO`, and for the same reason: a paraphrase of
what `main.rs` appears to do is indistinguishable, to a reader, from a fact.
**Run the subcommand and record what it did.** Exit codes especially — an exit
code copied from a `process::exit` call site is a claim about the source, not
about the tool.

### 3. D0 reports capability, it does not build it

D0 says, per Wave 4 fact class (syntax, CLI, target, public declarations, plus
the symbol / keyword / diagnostic / glossary indexes), **whether the toolchain
can emit it today, and if not, what is missing.** That is the whole deliverable.

⛔ **Writing a generator is banned scope.** So is adding a `--format` flag, and
so is building a diagnostic registry. Those are `crates/` work, they belong to
whichever team owns the crate, and a doc candidate carrying one makes the
verification record unauditable. **A capability gap D0 finds is a finding to
report, and it is what lets a later slice label an authored fact as authored
instead of dressing it up as generated.**

## Deliverables

- **D0 — the generation-capability report**, one table: fact class, can the
  toolchain emit it today (yes / no / partial), what is missing if not, and the
  command that establishes the answer. Filed under `docs/program/`.
- **D1 — `library/reference/toolchain/`**, one entry per real subcommand.
  `version` and `help` **do** get reference entries here (unlike the how-tos —
  a reader looking up what `version` prints is asking a lookup question).
- **D2 — the option and exit-status facts**, observed per D0's rule. The option
  surface is `fmt --check`, `--version` / `-V`, and `--help` / `-h`;
  `native-build`'s `<output-dir>` is a positional argument and must not be
  documented as a flag. **The exit surface has three classes and they are not
  equally knowable:**
  1. **non-`run` failures at 1** — observable, so document them from runs.
  2. **`run` propagating its program's status** — observable, so document it
     from a run.
  3. **the two source-declared `exit(2)` ABI-unavailable arms** — document
     **only if you can reach one with a real CLI command.** If you cannot, record
     them in D0 as *source-declared, unobserved*, and say nothing about them in
     the reference. They may be unreachable from the installed prelude.
  ⇒ **Do not restate a uniform exit rule.** There isn't one.
- **D3 — manifest registration** for each page: `kind = "reference"`,
  `authority = "derived-reference"`, audience, availability, `sources`,
  `validation`, owner — consistent with the existing `reference` records.
- **D4 — availability labels.** Any behaviour that does not work today gets
  `partial`, `planned`, or `unavailable` **and a sentence saying why**.
- **D5 — ledger consistency**: if a newly cited path lacks a row, install the
  generator's `.proposed` output; never hand-write a row.

## Acceptance criteria

- **AC-1 — every documented behaviour was observed.** Each flag, output, and
  exit code in D1/D2 traces to a recorded run.
  *Control:* the command log, with actual output. **A fact whose only support is
  a source line fails this AC.**
- **AC-2 — the reference is complete over its surface.** All 7 subcommands; the
  three options in their five accepted spellings; `<output-dir>` classified as
  positional; and the exit surface as D2's three classes, with class 3 present
  only if observed.
  *Control:* `print_help`'s subcommand list against the page set, **plus** the
  enumerating exit probe and the flag-spelling grep. ⚠ **`print_help` is not a
  sufficient control on its own** — it omits every global flag, which is how the
  first version of this frame undercounted the option surface.
- **AC-2a — `print_help`'s omissions are reported, not silently repaired.** The
  help text documents no global flag. The reference documents them because they
  are real; the gap between the two is a finding for whoever owns `ken-cli`.
  *Control:* the finding exists and names the four omitted spellings.
- **AC-3 — no reference entry duplicates a how-to** (judgment 1). Procedures are
  linked, not restated.
  *Control:* read each entry against the five `library/how-to/` pages.
- **AC-4 — D0 covers every Wave 4 fact class** named in the program, each with
  its establishing command.
  *Control:* the program's Wave 4 paragraph against D0's rows.
- **AC-5 — D0 claims no capability it did not run.** A "no" is as good as a
  "yes"; an unrun "yes" is the defect.
  *Control:* every D0 row's command and output.
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

- **No writing a generator, no adding CLI output formats, no diagnostic
  registry** (judgment 3). All three are `crates/` work.
- **No other Wave 4 surface.** language, verification, runtime, and platform
  reference, and the four indexes, are later slices and several of them depend
  on D0's answer. Authoring them now would commit to a generation story D0 has
  not yet established.
- **No reviving `VALIDATION_GATES`.** The registry is unreachable code and that
  finding is **Steward-owned**.
- **No fixing a diagnostic or help text that reads badly.** Report it.
- **No campaign or WP history in a page** — a reader does not care which WP
  landed a feature.
- **No new test asserting facts about source, catalog, or documentation lines**
  (operator test policy). D0 is a review artifact, not a gate.

## Contention

The doc track runs concurrently with build work by standing operator exception
because it touches `library/` and `agent/` rather than `crates/`. This candidate
writes `library/` and `docs/program/` only.

D0, AC-1, and AC-2 need the `ken` binary, so they need a build turn:
`scripts/ken-cargo build -p ken-cli`, targeted, **never `--workspace`**. Probe
for the lock without blocking before taking it. If it is contended, draft page
structure first — but **do not hand off a candidate without D0 and its command
log**, because AC-1 has no meaning without them.

## Hard stops

Stop and route to the Steward, do not improvise, if any of these hold:

1. **The subcommand or option surface is not what the fixed inputs record** at
   your base. The CLI moved under the frame. ⚠ **This stop already fired once,
   on 2026-08-01, and the fixed inputs above are its repair** — the original
   table said "two flags" by reading `print_help` instead of running the tool,
   which is the frame's own judgment 2 violated in its own fixed inputs. Firing
   it again means something moved; it is not a re-report of that.
2. **A subcommand's observed behaviour contradicts `print_help`.** That is a
   real defect in the tool, it is worth more than the page, and the page must
   not paper over it by documenting the help text instead.
3. **D0 finds that a fact class the program calls generated is not merely
   ungenerated but ungeneratable** without a design decision. Report it —
   that is a program-level scope question, not a page-level labelling choice.
4. **A newly cited path has no ledger row and the generator will not produce
   one.** Report the path.
5. **A reference entry cannot be written without restating a how-to.** That
   means the surface is task-shaped, not lookup-shaped, and the entry may not
   belong in Wave 4 at all. Report it rather than shipping the duplicate.
