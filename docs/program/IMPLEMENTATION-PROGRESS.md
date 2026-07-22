# Implementation progress — the build backbone

**Owned by the Steward** (`agent/playbooks/federation/steward.md §2a`). This
file tracks execution **against the implementation DAG**
(`05-implementation-dag.md`), the build's analog of `spec/SPEC-PROGRESS.md`.
It **survives compaction**: on a cold start or after a compact, read this
first, then continue from the frontier (below). Update it **every synthesis
pass and on every WP state change**. The plan lives in `05`; this file
tracks *progress against it*. Run until complete, blocked, or instructed
(§2b).

**This file holds CURRENT STATE ONLY, and it is GENERATED** — edit
`docs/program/issues/*.md` and re-run `scripts/gen-progress.sh`; hand edits
here are overwritten. The full chronicle — every prior "live state"
snapshot, the detailed evidence trail for every merged WP, and the
day-by-day session logs back to project start — lives in
[`diary/`](diary/INDEX.md). If you need *why* a past call was made, or the
mechanism detail behind a closed WP, start there;
[`diary/CURRENT-BRIEFING.md`](diary/CURRENT-BRIEFING.md) carries the live
operator briefing and the Steward's resume state.

**Status legend:** `draft` (not framed / deps unmet) · `ready` (deps met,
unassigned) · `active` (a team is building) · `in-review` (PR open / QA / CI)
· `merged` (landed + retro in) · `closed` (resolved without landing, e.g. a
superseded or withdrawn item). Gates: see `05-implementation-dag.md`.

**★ GENERATED FILE — do not hand-edit.** This file is regenerated from the
frontmatter of every `docs/program/issues/*.md` work-item file by
`scripts/gen-progress.sh`. To change tracked status, edit the relevant
`docs/program/issues/<ID>.md` file and re-run the generator. CI checks that
the committed file matches the generator's output.

## Last generated

2026-07-22 13:45:52Z — from 27 issue file(s) in `docs/program/issues/`.

## Work-item status

| ID | Title | Status | Owner | Size | Gate | GitHub |
|---|---|---|---|---|---|---|
| `A3` | catalog-coverage walker | draft | TBD | TBD | none | — |
| `ABI-REVOKE` | runtime revocation membrane — the deferred runtime face of 62 §4 | draft | runtime | TBD | none | — |
| `BUDGET-EFF` | TransferCount.remaining must be bounded by the effective request | active | verify | M | none | — |
| `CI-SKIPPED-NATIVE-TESTS` | Restore rt_parity_native — one test at 221s is the blocker | ready | steward | S | none | — |
| `CI-TRACKER-GATE` | Wire the issue-tracker schema + regeneration gate into CI | closed | operator | S | none | 804 |
| `DOC-CURRENCY-ANCHOR` | library/REVISION certifies nothing about the corpus — currency is unchecked | closed | doc | S | none | — |
| `DOC-VALIDATION-BINDING` | validation vocabulary claims a 1:1 binding to the gates; nothing binds it | ready | doc | S | none | — |
| `DOC-W0` | documentation Wave 0 — library/ charter and currency substrate | closed | doc | M | none | 830 |
| `DOC-W1` | documentation Wave 1 — the read-Ken spine, taught from checked fragments | ready | doc | L | none | — |
| `DOC-W2` | documentation Wave 2 — agent core modules, task packs, and cold-context evals | draft | doc | L | none | — |
| `F1-37` | F1 [task-list #37] — bignum Int soundness review for K3 trusted-base promotion | ready | runtime | TBD | none | — |
| `F3-39` | F3 [task-list #39] — reducer: degrade-not-wrap + retire legacy arms | draft | runtime | TBD | none | — |
| `F4` | content-addressing + value-model design (aka PX8-F-PROOF) | draft | foundation+spec-enclave | M | none | — |
| `MODELS-TIER` | agent/MODELS.md — the Runtime seating is the fleet-wide norm, not an exception | ready | steward | S | none | — |
| `PUB-VERIFY` | scripted-pr-automerge.sh exits 0 on a failed push | ready | steward | S | none | — |
| `PX8-F-CAP-41` | PX8-F-CAP (#41) — backlog, deferred to spec-first | draft | TBD | TBD | none | 41 |
| `PX8` | partial/positioned IO — the completion program's root; closure condition | active | runtime | L | none | — |
| `Q-CLAIM-CLOSURE` | Q-RESIDUE adversary findings — claim-loss in multi-claim test blocks, plus R1/R2/R3 | ready | runtime | S | none | — |
| `Q-RESIDUE` | the Track Q rework residue — 10 tests, folded from Q3-Q7 | closed | runtime | S | none | 818 |
| `RT-ESCAPE` | escaping a second Resource through a bracket fails native lowering | draft | runtime | TBD | none | — |
| `RT-PARITY` | interpreter/native parity erratum (adversary F5 + F6) | closed | runtime | M | none | — |
| `RT-SPLIT` | decompose cranelift_backend.rs | active | runtime | L | none | — |
| `RT-SRC-DISPATCH-COVER` | close the source-machine scrutinee-dispatch coverage tier surfaced by RT-SPLIT slice 4 | draft | runtime | TBD | none | — |
| `SEAL-2` | carrier producer closure, over a derived enumeration | draft | foundation | M | none | — |
| `SPAN-SEAL` | seal the BufferSpan producer surface | merged | foundation | M | none | — |
| `SPEC-38-ERRATUM` | spec 38-ffi-io self-contradicts on the transfer bound — rule and reconcile | closed | spec | S | none | 827 |
| `STR-BIJ` | the String/List Char 'bijection' over-claim (adversary A1 + A2) | ready | spec-enclave | S | none | — |

## Releasable frontier

Items whose status is `ready` and whose every `depends_on` entry is
itself `merged` or `closed` (i.e. nothing left blocking a kickoff):

- `CI-SKIPPED-NATIVE-TESTS` — Restore rt_parity_native — one test at 221s is the blocker
- `DOC-VALIDATION-BINDING` — validation vocabulary claims a 1:1 binding to the gates; nothing binds it
- `DOC-W1` — documentation Wave 1 — the read-Ken spine, taught from checked fragments
- `F1-37` — F1 [task-list #37] — bignum Int soundness review for K3 trusted-base promotion
- `MODELS-TIER` — agent/MODELS.md — the Runtime seating is the fleet-wide norm, not an exception
- `PUB-VERIFY` — scripted-pr-automerge.sh exits 0 on a failed push
- `Q-CLAIM-CLOSURE` — Q-RESIDUE adversary findings — claim-loss in multi-claim test blocks, plus R1/R2/R3
- `STR-BIJ` — the String/List Char 'bijection' over-claim (adversary A1 + A2)

## Blockers

Items not yet `merged`/`closed` whose `depends_on` names an id that
is itself not yet `merged`/`closed`:

- `DOC-W2` blocked by `DOC-W1` (status: ready)
- `F4` blocked by `A3` (status: draft)
- `RT-SRC-DISPATCH-COVER` blocked by `RT-SPLIT` (status: active)
- `SEAL-2` blocked by `BUDGET-EFF` (status: active)

## Gate progress

Work items grouped by the gate (`05-implementation-dag.md`) they
feed; `none`/`TBD` gates are omitted here (see the status table above
for every item, gated or not):

- No item in the current queue cites a named gate.

## Archive & diary

- The complete build chronicle — every prior live-state snapshot, the full
  evidence trail behind every merged WP back to project start — and the
  day-to-day session narrative both live in [`diary/`](diary/INDEX.md), one
  file per day under `diary/YYYY/Mon/DD.md`. See
  [`diary/CURRENT-BRIEFING.md`](diary/CURRENT-BRIEFING.md) for the live
  operator briefing and Steward resume state.
- Per-item briefs, where they exist, live under
  [`wp/`](wp/) and are linked from the corresponding
  `docs/program/issues/<ID>.md` file.
