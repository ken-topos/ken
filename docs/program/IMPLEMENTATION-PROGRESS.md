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

2026-07-24 20:33:53Z — from 45 issue file(s) in `docs/program/issues/`.

## Work-item status

| ID | Title | Status | Owner | Size | Gate | GitHub |
|---|---|---|---|---|---|---|
| `A3` | catalog-coverage walker | draft | TBD | TBD | none | — |
| `ABI-REVOKE` | runtime revocation membrane — the deferred runtime face of 62 §4 | draft | runtime | TBD | none | — |
| `BUDGET-EFF` | TransferCount.remaining must be bounded by the effective request | merged | verify | M | none | — |
| `BUDGET-EXHAUST` | transfer-budget bound checks are fail-open on variant extension | merged | verify | S | none | — |
| `CAT-CAPEX` | catalog exhibits no checked capability/authority exemplar | draft | steward | TBD | none | — |
| `CB-HYGIENE` | cranelift_backend facade: strip WP-token narration, separate test material from implementation | merged | runtime | S | none | — |
| `CI-SKIPPED-NATIVE-TESTS` | Restore rt_parity_native — dedicated CI job, outlier not fixed | merged | verify | S | none | — |
| `CI-TRACKER-GATE` | Wire the issue-tracker schema + regeneration gate into CI | closed | operator | S | none | 804 |
| `DOC-CURRENCY-ANCHOR` | library/REVISION certifies nothing about the corpus — currency is unchecked | closed | doc | S | none | — |
| `DOC-GATE-CONTROL-BINDING` | validation-gate registry: make the two DOC-GATE-RECORD-AXIS checks orphan-proof by lifting them to pure detectors with committed controls | ready | verify | S | none | — |
| `DOC-GATE-RECORD-AXIS` | validation-gate registry: bind token→runner COVERAGE on the record axis, and close the `kind` vocabulary | merged | verify | S | none | https://github.com/ken-topos/ken/pull/922 |
| `DOC-VALIDATION-BINDING` | validation vocabulary claims a 1:1 binding to the gates; nothing binds it | merged | verify | S | none | — |
| `DOC-W0` | documentation Wave 0 — library/ charter and currency substrate | closed | doc | M | none | 830 |
| `DOC-W1` | documentation Wave 1 — the read-Ken spine, taught from checked fragments | closed | doc | L | none | — |
| `DOC-W2` | documentation Wave 2 — agent core modules, task packs, and cold-context evals | draft | doc | L | none | — |
| `F1-37` | F1 [task-list #37] — bignum Int soundness review for K3 trusted-base promotion | ready | runtime | TBD | none | — |
| `F3-39` | F3 [task-list #39] — reducer: degrade-not-wrap + retire legacy arms | draft | runtime | TBD | none | — |
| `F4` | content-addressing + value-model design (aka PX8-F-PROOF) | draft | foundation+spec-enclave | M | none | — |
| `KW-THEOREM` | rename the surface keyword `lemma` to `theorem` | ready | spec | M | none | — |
| `LOADER-CITE-ANCHOR` | LOADER-STALE-PREMISE cites the spec by line number (:147-158) — rots silently in the one catalog file outside the currency gate | merged | doc | XS | none | — |
| `LOADER-STALE-PREMISE` | \"no disk loader yet\" is stale in 9 places — including already-landed library/ content | merged | doc | S | none | — |
| `MODELS-TIER` | agent/MODELS.md — the Runtime seating is the fleet-wide norm, not an exception | ready | steward | S | none | — |
| `NATIVE-HANDLE-CARRIER` | Native build-pipeline completeness — a constructor-private resource-carrying handle fails checked-core body-view lowering (MissingClosureMetadata) when it crosses the higher-order withBuffer normalization boundary | draft | runtime | M | none | — |
| `ORACLE-VIS-CHECK` | replace the text-pin oracle in px4b_native_production.rs with a real visibility check | merged | runtime | S | none | — |
| `ORACLE-VIS-PACKAGING` | replace the text-pin visibility oracle on build_process_starter_executable_artifact | merged | runtime | XS | none | — |
| `PUB-VERIFY` | scripted-pr-automerge.sh exits 0 on a failed push | closed | steward | S | none | — |
| `PX8-F-CAP-41` | PX8 clause-(a) behavior blocker — closed buffer endpoint (start==capacity) must derive zero-effective ReadEof, not host-reject | active | foundation | M | none | 41 |
| `PX8-SPAN-PROV` | PX8 clause-(b) gap — BufferSpan carries no originating-buffer identity; freeze accepts a same-shape span from a different buffer | merged | spec-enclave | M | none | 914 |
| `PX8-WROTE-ABS` | PX8 clause-(a) evidence gap — interpreter capped-short Wrote lacks an absolute oracle; PR-C error identities unreached | draft | TBD | TBD | none | — |
| `PX8` | partial/positioned IO — the completion program's root; closure condition | active | runtime | L | none | — |
| `Q-CLAIM-CLOSURE` | Q-RESIDUE adversary findings — claim-loss in multi-claim test blocks, plus R1/R2/R3 | merged | runtime | S | none | — |
| `Q-CLAIM-COMPARE-ORD` | claim-loss in list_instance_routes... (compare_ord) — both routing claims dropped, replacement only instantiates Bool | merged | runtime | XS | none | — |
| `Q-RESIDUE` | the Track Q rework residue — 10 tests, folded from Q3-Q7 | closed | runtime | S | none | 818 |
| `RT-AGG-COMPOSE` | escaping two Resources into one aggregate (Prod (Resource _) (Resource _)) fails at erasure — checked endpoints do not compose | draft | runtime | TBD | none | — |
| `RT-ESCAPE` | escaping a second Resource through a bracket fails native lowering | merged | runtime | M | none | PR #911 @ 238a5c5d (origin/main 4ac9141e, CI green) |
| `RT-NATIVE-FNSPLIT` | Native backend: bound per-function lowering growth to O(n) — helper identity is a variable-width whole-configuration key (orig. single-Function VReg::MAX, since fixed) | active | runtime | TBD | none | — |
| `RT-PARITY` | interpreter/native parity erratum (adversary F5 + F6) | closed | runtime | M | none | — |
| `RT-SPLIT` | decompose cranelift_backend.rs | merged | runtime | L | none | — |
| `RT-SRC-DISPATCH-COVER` | close the source-machine scrutinee-dispatch coverage tier surfaced by RT-SPLIT slice 4 | draft | runtime | TBD | none | — |
| `SEAL-2` | carrier producer closure, over a derived enumeration | merged | foundation | M | none | PR #912 @ 4ac9141e (origin/main, CI green) |
| `SPAN-SEAL` | seal the BufferSpan producer surface | merged | foundation | M | none | — |
| `SPEC-38-ERRATUM` | spec 38-ffi-io self-contradicts on the transfer bound — rule and reconcile | closed | spec | S | none | 827 |
| `SRC-ATTEST` | squash-stable whole-source attestation + fresh merge-result authorization | merged | doc | M | none | — |
| `STR-BIJ` | the String/List Char 'bijection' over-claim (adversary A1 + A2) | ready | spec-enclave | S | none | — |
| `VIS-BR-LITERAL` | visibility walk: raw-string prefixes br and cr are unrecognized by the literal scanner | merged | runtime | XS | none | — |

## Releasable frontier

Items whose status is `ready` and whose every `depends_on` entry is
itself `merged` or `closed` (i.e. nothing left blocking a kickoff):

- `DOC-GATE-CONTROL-BINDING` — validation-gate registry: make the two DOC-GATE-RECORD-AXIS checks orphan-proof by lifting them to pure detectors with committed controls
- `F1-37` — F1 [task-list #37] — bignum Int soundness review for K3 trusted-base promotion
- `KW-THEOREM` — rename the surface keyword `lemma` to `theorem`
- `MODELS-TIER` — agent/MODELS.md — the Runtime seating is the fleet-wide norm, not an exception
- `STR-BIJ` — the String/List Char 'bijection' over-claim (adversary A1 + A2)

## Blockers

Items not yet `merged`/`closed` whose `depends_on` names an id that
is itself not yet `merged`/`closed`:

- `F4` blocked by `A3` (status: draft)
- `NATIVE-HANDLE-CARRIER` blocked by `RT-NATIVE-FNSPLIT` (status: active)
- `PX8-F-CAP-41` blocked by `NATIVE-HANDLE-CARRIER` (status: draft)

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
