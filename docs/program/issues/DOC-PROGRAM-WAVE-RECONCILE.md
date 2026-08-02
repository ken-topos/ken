---
id: DOC-PROGRAM-WAVE-RECONCILE
title: "Reconcile the documentation program's wave status against the landed corpus — the status line, the wave table, and the section 4b headers all say map only over bodies that measured otherwise, and produce the residual register that says what the doc ring owes next"
status: merged
owner: doc
size: M
gate: none
depends_on: [DOC-PROGRAM-SELF-REFUTE]
blocks: []
github: null
origin: "Steward measurement 2026-08-02 at origin/main = 70441007, taken while looking for the doc ring's next node after DOC-PROGRAM-SELF-REFUTE merged and every doc-owned node reached merged or closed."
---

# The program describes itself as four waves behind where it is

`docs/program/12-documentation-program.md` opens by saying `library/` carries
**26 documents** and that **"Wave 3 and beyond remain map only."** Measured at
`origin/main = 70441007`: `library/` carries **89** markdown documents and
**89** manifest entries, and waves 3, 4, 5 and 6 have **twelve merged nodes
between them**.

The wave sections themselves are not wrong. Each carries a dated Steward
measurement block — Wave 3 at `c777d2d4`, Wave 4 at `7fa65b20`, Wave 6 at
`5a0fd8e6` — and those blocks are current. **What is stale is everything a
reader meets first:** the status line, the section 4 wave table, and the `(MAP)`
labels in the section 4b headings.

⇒ **This is an appended-correction defect, not a measurement defect.** The
program has been corrected four times by adding a dated block underneath a
heading that still asserts the superseded state. A later note saying a
deliverable is false does not replace the deliverable, and a reader who stops
at the table never reaches the block.

## Why this is the doc ring's node and not a Steward tidy-up

Every doc-owned node is now `merged` or `closed`. The residual register this WP
produces (`D4`) is the input that decides what the doc ring is framed next —
without it the Steward is choosing the next doc node from a document that
understates the corpus by 63 files.

The Steward hit this directly: the search for the doc ring's next node began
against **"Wave 3 and beyond remain map only"** and would have re-released work
that landed days ago. That is the mistake this program has already made on L5
and V3, and the Wave 3 block warns about it in its own words.

## What it may not touch

- **No new gate, no re-armed registry, no CI coupling.** `LIB-GATE-DECOUPLE`
  retired that coupling by operator ruling. This inherits
  `DOC-PROGRAM-SELF-REFUTE`'s J2 verbatim: zero changes under `crates/`,
  `.github/`, or `scripts/`.
- **No `library/` edits.** `SOURCE-ATTESTATIONS`, `STATUS.md` and `REVISION`
  are release-point artifacts that lag by design, and the COORDINATION §14a
  ledger rider that appears to require a fold is retired.
- **No new documentation.** This reconciles the program's account of the
  corpus. Writing a missing page is the *next* node, which `D4` names.

The frame is `docs/program/wp/DOC-PROGRAM-WAVE-RECONCILE.md`.
