---
id: DOC-W3-GUIDE
title: "Wave 3 slice 1 — migrate catalog/guide/ into library/guide/ under migration-local fence verification, conserving all 40 checked fences through the move"
status: ready
owner: doc
size: M
gate: none
depends_on: [DOC-ASBUILT-LEDGER]
blocks: []
github: null
origin: "Steward 2026-08-01. Wave 3's §3 fence precondition was reconciled in docs/program/12-documentation-program.md §4 the same day; this is the first slice released under it. Measured at origin/main = f31e8d94."
---

# Wave 3, slice 1 — the conceptual guide over the existing guide material

Wave 3 produces `library/guide/` (contracts, dependent data, proofs, effects,
security, packages, execution), `library/how-to/` recipes, and the
`catalog/guide/` migration. **This slice is the migration and the pages that
sit directly on it.** The how-to recipes are a separate slice: they are driven
by actual diagnostics and recurring fleet failures, which is a different input
and a different research act.

## Why this slice is releasable now

Wave 3 carried the program's one hard ordering constraint: the `ken example` /
`ken reject` fences must be verified before any `catalog/guide/` content moves.
That precondition sat **underspecified** — it named a CI gate whose form no
longer existed after `LIB-GATE-DECOUPLE`, so nobody could discharge it, and
reinstating a global gate would have walked back an operator ruling.

It is now reconciled (`12-documentation-program.md` §4, Steward 2026-08-01).
The substantive requirement is unchanged; the form is **migration-local
verification at candidate time**, and the frame carries it as a deliverable
with a control.

## The shape, stated up front: the fences must survive the move

D2 of the program is ratified and settles the direction — migration is
**subsumptive**: `catalog/guide/` moves into `library/` and does not persist
alongside, leaving pointers rather than a second maintained guide. So the
Wave 1 pattern of citing a checked file that stays in `catalog/` is **not
available here**; the spine cites `catalog/packages/`, which persists, and the
guide does not.

The obligation is therefore the harder one §3 states: the **40 checked fences
must still be checked once they land in `library/`** — preserved *through* the
move, not preserved in place. The binding control is a conservation law: 40
before, 40 after, each still exercised by the real extractor.

The frame's one genuinely unverified premise is what a checked page in
`library/` looks like. `ken check` selects literate extraction by the `.ken.md`
suffix, and `library/` today holds **zero `.ken.md` files** and 7 ken fences
across 25 documents — no literate document has ever been registered. Whether one
can be is the Librarian's call, and it is a hard stop to raise early.

The frame is `docs/program/wp/DOC-W3-GUIDE.md`.
