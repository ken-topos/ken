# WP frame — `DOC-ASBUILT-LEDGER` (as-built phase B, the terminal re-stamp)

Node: `docs/program/issues/DOC-ASBUILT-LEDGER.md`. Campaign:
`docs/program/issues/DOC-ASBUILT-AUDIT.md`. Owner: doc ring (doc-leader,
doc-author, Librarian as QA).

Phase A is complete. Six slices reconciled every claim in all 25 consuming
`library/` documents against the current blob of every source they cite, and
wrote no ledger row. This node is the single terminal act that installs the
reviewed ledger and turns the currency gate green.

**Size S. One candidate. Doc-only.**

## Fixed inputs

Measured on the tree of `steward/work = c45213ce` — `origin/main = 4c10ba4e`
plus the slice-6 D3 source repair, which is published together with this frame.

> **Every control below is stated over CONTENT — an output hash, a row count, a
> path set — never over a commit SHA.** The publisher squashes, so this tree
> lands on `main` under a SHA that does not exist yet. A control pinned to
> `c45213ce` would be unverifiable at the moment it matters. Re-derive each one
> at the candidate base; they are all reproducible read-only, no build turn.

| input | measured value |
|---|---|
| drift check | `scripts/gen-doc-status.sh --check` exits **1**, 32 lines, **28** drifted rows, SHA-256 `ae5ab2e9522d227ee59c9805853fc09240ebc28f6c0a46153df707e1aa9b72a6` |
| ledger | `library/SOURCE-ATTESTATIONS` — 1 header line + **52** attestation rows |
| drifted share | **28 of 52** rows |
| proposed ledger | `scripts/gen-source-attestations.sh` writes `library/SOURCE-ATTESTATIONS.proposed` (gitignored): **28 changed rows, 0 added, 0 removed**, 52 rows both sides |
| other ledger faults | **zero** "missing from ledger (cited, not attested)" and **zero** "stale in ledger (attested, no longer cited)" |
| `library/REVISION` | `4427147d5f24ca9a0820939bc6e831c986a17afa`, dated 2026-07-26, a valid ancestor of `main` but **320 commits behind it** |
| `library/STATUS.md` | 72 lines, generated, renders that stale revision as its provenance line |

Reproduce, read-only:

```sh
scripts/gen-doc-status.sh --check            # exits 1; the block is the population
scripts/gen-source-attestations.sh           # writes the .proposed sibling only
diff library/SOURCE-ATTESTATIONS library/SOURCE-ATTESTATIONS.proposed
```

## Three judgments, settled here so the ring does not have to stop for them

### 1. The ban was on the TIMING, not the tool — running the generator now is correct

`scripts/gen-source-attestations.sh` was banned for all of phase A. That ban
expires with this node. Running it during phase A would have laundered 28
unreviewed claims into a green gate; running it now, after every consuming page
has been reconciled, is the intended use and the only act that can make the gate
green.

The script cannot install its own output. It writes
`library/SOURCE-ATTESTATIONS.proposed` and stops; installing is a separate `mv`
by whoever reviewed the changed sources. Keep it that way — the two-file-paths
design is what makes "regenerate whenever HEAD differs" impossible by
construction rather than by convention (`SRC-ATTEST` Part 1,
Librarian-authoritative).

### 2. This node does NOT wait for a quiet tree, and a later red gate is not its bug

Four of the 28 drifted paths are live code or CI paths that in-flight build work
can touch: `.github/workflows/ci.yml`,
`crates/ken-cli/tests/px4b_native_production.rs`,
`crates/ken-interp/src/eval.rs`, `crates/ken-runtime/src/cranelift_backend.rs`.
The runtime capstone `RT-CONTSPEC-LOWER` is in flight against that crate right
now.

**Measured, not assumed:** its preservation commit
`12d724694e4affe56a11afa8dcb42f81402817ac` touches seven files —
`boundary_value.rs`, `boundary_value_clif.rs`,
`cranelift_backend/lowering/{core,mod,units}.rs`,
`cranelift_backend/planning.rs`, `native_process_entrypoint.rs` — and **none of
them is a cited source.** `cranelift_backend.rs` is the module file, distinct
from the `cranelift_backend/` directory the capstone is editing. So there is no
contention today. The capstone is unfinished and could still touch it.

**That does not gate this node, because the policy already answers it.**
`LIB-GATE-DECOUPLE` merged at `f84e4804` under an operator ruling that removed
live documentation and content CI coupling outright, and the resulting policy
**explicitly accepts that source attestations drift between release points.**
Re-stamping at a release point is what the ledger is for.

⇒ **A red `--check` some commits after this merges is the policy working, not a
regression in this candidate.** Reading a later red gate as "phase B failed"
applies the per-merge premise that [[DOC-ATTEST-LIVING]] was retired for
holding. Do not sequence this node behind any build WP, and do not add an AC
that asserts the gate stays green after merge — that AC would be false by design.

### 3. `library/REVISION` MUST be bumped — to the candidate's BASE, not its SHA

`library/REVISION` is an explicit committed input, deliberately set by whoever
validated the corpus against that revision. It is 320 commits stale. **Phase B
is the act of having validated the corpus**, so shipping a green gate while
`STATUS.md` still renders a 2026-07-26 provenance line would replace one
unbacked currency claim with another — a quieter version of the defect
`DOC-CURRENCY-ANCHOR` closed.

**Set it to the candidate's base commit on `origin/main`.** Not to the candidate
branch's own SHA: the publisher squashes, so a branch SHA is unreachable on
`main` afterwards, and `revision_resolved()` requires a real commit object that
is an ancestor of `HEAD`. That is exactly fold 1 of `DOC-CURRENCY-ANCHOR`
(*"REVISION named a pre-squash branch commit, unreachable once main
squash-merged the branch"*) — it has been paid for once already.

## Deliverables

- **D1 — the installed ledger.** `library/SOURCE-ATTESTATIONS` with all 28
  drifted rows re-stamped, produced by running the generator and installing its
  `.proposed` output. No hand-edited row.
- **D2 — `library/REVISION` bumped** to the candidate's base commit on
  `origin/main`, as a full 40-hex id.
- **D3 — `library/STATUS.md` regenerated** by `scripts/gen-doc-status.sh` with
  no arguments, after D1 and D2. Generated, never hand-edited.
- **D4 — the review record**, in the candidate's PR body or the node: for each
  of the 28 paths, which phase-A slice reconciled the pages citing it (PR
  numbers #1282, #1287, #1292, #1294, #1297, #1304). This is the human
  authorization the generator deliberately refuses to fabricate. It is a review
  artifact, **not** a CI test — no gate may assert facts about source lines
  (operator test policy).

## Acceptance criteria

- **AC-1 — the gate goes green on the candidate.** `scripts/gen-doc-status.sh
  --check` exits **0** and prints `library/STATUS.md is current.`
  *Control:* run it on the candidate tree.
- **AC-2 — the ledger diff is exactly the drift population and nothing else.**
  Against the base ledger: **28 changed rows, 0 added, 0 removed**, 52
  attestation rows on both sides, and the 28 changed paths are **set-equal** to
  the 28 paths in the base `--check` output.
  *Control:* `diff` the two ledgers and compare the path set against the
  recorded drift block. This is the AC that catches an accidental population
  change — a row count that still reads 52 while the membership moved.
- **AC-3 — no re-stamped row is hand-written.** Re-running
  `scripts/gen-source-attestations.sh` on the candidate produces a `.proposed`
  file **byte-identical** to the installed `library/SOURCE-ATTESTATIONS`.
  *Control:* generate and `diff`; it must be empty.
- **AC-4 — `STATUS.md` is generated, not authored.** Re-running
  `scripts/gen-doc-status.sh` (no arguments) on the candidate leaves
  `library/STATUS.md` byte-identical.
  *Control:* run it, then `git diff --quiet -- library/STATUS.md`. Use
  `--quiet`; `--stat` always exits 0 and would pass vacuously.
- **AC-5 — `REVISION` resolves and survives the squash.** The recorded value is
  a real commit object, an ancestor of the candidate, and **reachable from
  `origin/main`** — so it is still valid after the merge.
  *Control:* `git cat-file -e <rev>^{commit}` and
  `git merge-base --is-ancestor <rev> origin/main`.
- **AC-6 — the candidate touches `library/` only.** `git diff --name-only`
  against the base is confined to `library/`, with no `spec/`, `catalog/`,
  `crates/`, or `docs/` path.
  *Control:* the path list.
- **AC-7 — CI green** on the merge. Workspace-green means green in CI, never a
  local `--workspace` run.

## Banned scope

- **No source repair.** If a page's claim is false because the source is wrong,
  that is a finding to route, not a repair to make here. Phase A is closed;
  reopening a claim in this candidate mixes reconciliation into a re-stamp and
  makes D4's review record unauditable.
- **No manifest edits.** No new `sources` entry, no new document record. The
  per-slice citation-registration grant of phase A does not extend to this node
  and was never standing.
- **No hand-edited ledger row and no partial install.** All 28 or none.
- **No new CI gate** asserting facts about source, catalog, or documentation
  lines (operator test policy).
- **No AC or mechanism that tries to keep the gate green after merge.** That is
  the retired per-merge premise; see judgment 2.

## Contention

None with the build rings. This candidate writes only `library/`
(`SOURCE-ATTESTATIONS`, `REVISION`, `STATUS.md`). The doc track runs
concurrently with build work by standing operator exception because it touches
`library/` and `agent/` rather than `crates/`, and that exception holds exactly
here.

The one thing to check at candidate time is whether another doc-track change is
in flight that would move a cited `library/` path — `fragments.md` is the only
attested `library/` page and nothing is queued against it.

## Hard stops

Stop and route to the Steward, do not improvise, if any of these hold:

1. **The proposed diff is not 28 rows**, or adds or removes a row. That means
   the cited-source population itself moved, and the review record in D4 no
   longer covers what is being stamped. Report the exact delta.
2. **`--check` still exits 1 after installing the ledger.** The residual is a
   fault class the drift block does not describe (a cited-but-unattested path,
   or an attested-but-uncited one). Report which of the two, with paths.
3. **A path in the drift population has no phase-A slice that covers it.** D4 is
   then unwritable honestly, and the gap is a phase-A coverage hole, not a
   phase-B problem.
4. **`REVISION` cannot be set to a base that is reachable from `origin/main`** —
   for example the branch was cut from an unpublished commit. Do not fall back
   to the branch SHA.
