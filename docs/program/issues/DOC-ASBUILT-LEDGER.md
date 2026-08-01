---
id: DOC-ASBUILT-LEDGER
title: "As-built phase B — the terminal re-stamp: install the reviewed attestation ledger for all 28 drifted rows at once and regenerate library/STATUS.md"
status: ready
owner: doc
size: S
gate: none
depends_on: [DOC-ASBUILT-AGENTS]
blocks: [DOC-ASBUILT-AUDIT]
github: null
origin: "Steward 2026-08-01, phase B of DOC-ASBUILT-AUDIT, measured at steward/work = c45213ce (origin/main 4c10ba4e plus the slice-6 D3 source repair). Releasable because all six phase-A slices merged: PR #1282, #1287, #1292, #1294, #1297, #1304."
---

# Phase B — the terminal re-stamp

Phase A reconciled every claim in all 25 consuming `library/` documents against
the current blob of every source they cite, across six merged slices. It wrote
**no ledger row**, by design, and left the currency gate red the whole way. That
red gate was the honest state: 28 cited sources had moved since anyone last read
them.

This node is the single act that closes it. It installs the reviewed
`library/SOURCE-ATTESTATIONS` ledger for **all 28 drifted rows at once** and
regenerates `library/STATUS.md`.

## Why it is one act and cannot be sliced

`library/SOURCE-ATTESTATIONS` holds **one row per path, corpus-wide** — not one
row per (document, source) pair. A row may be re-stamped only once *every*
document citing it has been reconciled. With 17 of the 28 drifted sources shared
by two or more documents and 22 of 25 documents in one connected component, no
per-page slicing clears the gate incrementally.

`DOC-ASBUILT-AUDIT` records the two-component structure of the citation graph
and states the conclusion this node inherits: splitting the ledger write to bank
one row early is exactly what makes a partial re-stamp look complete. The
measurement is kept there because it is true, not as licence to split.

## The standing ban was on the TIMING, not the tool

`scripts/gen-source-attestations.sh` was banned throughout phase A. Running it
then would have laundered 28 unreviewed claims into a green gate. Running it
**now**, after every consuming page has been reconciled, is the correct and
intended use, and is the only act that can make the gate green.

The script itself enforces the human boundary: it writes
`library/SOURCE-ATTESTATIONS.proposed` and **never** the real ledger path.
Installing it is a separate deliberate act by whoever reviewed the changed
sources. That two-file-paths-not-one-flag design is what makes "regenerate
whenever HEAD differs" impossible by construction rather than by convention
(`SRC-ATTEST` Part 1, Librarian-authoritative).

## Drift after this node merges is EXPECTED, not a regression

`LIB-GATE-DECOUPLE` merged at `f84e4804` under an operator ruling that removed
live documentation and content CI coupling outright. **The resulting policy
explicitly accepts that source attestations drift between release points.**

Two consequences, both load-bearing for how this node is sequenced and judged:

- **This node does not have to wait for a quiet tree.** Four of the 28 drifted
  paths are live code or CI paths that in-flight build work can touch. Under a
  release-point policy that is not contention — re-stamping at a release point
  is what the ledger is for.
- **A red `--check` some commits later is not a defect in this candidate.** It
  is the policy working as ruled. Anyone who reads a later red gate as
  "phase B failed" is applying the per-merge premise that
  [[DOC-ATTEST-LIVING]] was retired for holding.

## Relationship to the campaign

Merging this node satisfies the last obligation of [[DOC-ASBUILT-AUDIT]] and
closes the as-built campaign. The frame is
`docs/program/wp/DOC-ASBUILT-LEDGER.md`.
