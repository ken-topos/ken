---
id: SRC-ATTEST
title: "squash-stable whole-source attestation + fresh merge-result authorization"
status: active
owner: doc
size: M
gate: none
depends_on: []
blocks: [ORACLE-VIS-PACKAGING]
github: null
origin: >
  Librarian impossibility proof (evt_6t6wz1aw18291) during ORACLE-VIS-PACKAGING
  re-validation; Architect ruling dec_7q3kes0jcx1kn (evt_5vp06mb9v26mh).
---

**The frame is the authoritative artifact:
`docs/program/wp/SRC-ATTEST-currency-substrate.md`** (on `main` at `b860be22`).
This file is the tracker entry only; do not re-derive scope from it.

## One line

No valid `library/REVISION` can ride a PR that edits a manifest-cited source —
`REVISION = origin/main` fails source-currency, `REVISION = branch tip` fails
squash-stability, and **no third commit is both an ancestor of `origin/main` and
a holder of unmerged bytes.** SRC-ATTEST dissolves that by keying currency on
**blob OIDs** (which survive a squash) instead of commit ancestry, and by making
the publisher authorize the **merge result it actually lands** rather than a
prior one.

## Shape

| part | owner | scope |
|---|---|---|
| **Part 1** | doc ring (owns branch `wp/SRC-ATTEST`) | `library/SOURCE-ATTESTATIONS`, `library/STATUS.md`, `scripts/gen-doc-status.sh`, `crates/ken-cli/tests/library_documentation_gates.rs` |
| **Part 2** | Steward spillover, same branch | `scripts/scripted-pr-automerge.sh` — generalize the existing `--doc-only` guard (`a9554a07`) to every merge, add the re-read-before-merge step |

⛔ **One branch, one Decision. Neither part merges alone.** Gate order:
Librarian QA binds the assembled exact SHA → Architect terminal review. **Full
CI** — the diff reaches `scripts/` and `crates/`, so this is **not** a §14a
library-only merge. No Spec vote unless scope reaches `spec/`/`conformance/`.

## Status

Released to the doc ring 2026-07-22. `ORACLE-VIS-PACKAGING` is held on this
landing — that hold is the Steward's sequencing call, **not** a defect in the
runtime ring's candidate (`wp/ORACLE-VIS-PACKAGING @ 8dad30de`, content verdict
APPROVED/benign).
