---
id: DOC-W0
title: "documentation Wave 0 — library/ charter and currency substrate"
status: ready
owner: doc
size: M
gate: none
depends_on: []
blocks: []
github: null
origin: docs/program/12-documentation-program.md (Steward frame, 2026-07-21) from research/librarian-documentation-program-proposal.md
---

Land `library/` as Ken's product-documentation portal, with the **substrate
that makes every later page's currency checkable** — before Wave 1 produces
content at volume.

> **Re-framed 2026-07-21 for the doc team.** This item was first released to
> the Librarian as a solo seat. The operator then established a three-seat doc
> unit — **`doc-leader`** (scoping, workflow), **`doc-author`** (authoring),
> and the **Librarian** as its QA (editor, fact-checker, reviewer, plus a
> standing as-built mandate). Held at `ready` until both new seats are wired
> and confirmed. The deliverables and acceptance criteria below are unchanged;
> only the ownership is.

**Read the design canonically, do not work from this summary:**
`research/librarian-documentation-program-proposal.md` (the shape) and
`docs/program/12-documentation-program.md` (the frame, and the four settled
decisions that bind you).

## Deliverables

1. **`library/README.md`** — the one-screen portal. Five reader routes: read
   Ken, write Ken, look something up, find a package, load agent context.
2. **`library/manifest.toml`** — every document's kind, audience, **authority
   class**, sources, validation gates, availability, owner. Record shape is in
   the proposal.
3. **`library/STATUS.md`** — **generated**, anchored to a repository revision.
   Not hand-edited.
4. **A small set of real documents** — enough to prove the substrate works.
   `introduction.md` is the natural candidate. **Real pages, not placeholders**
   — empty directories are not a deliverable.
5. **The first gates** (proposal §"Documentation gates" 1, 2, 3, 6):
   manifest covers every document and every manifest path exists; links valid;
   every source path and section anchor exists; every page labels
   current/partial/planned/unavailable.
6. **The migration ledger** — which existing pages move, stay canonical, or
   become pointers. A record, not the migration itself.

## Acceptance criteria

1. **A new page cannot land without declaring what it is, what grounds it, and
   how its currency is checked.** This is the wave's whole purpose — if a page
   can be added that the gates do not notice, Wave 0 is not done.
2. Every gate **actually fails on a planted violation.** Demonstrate it: break
   a link, point a source anchor at a deleted section, omit a manifest entry —
   show each gate goes red, then revert. **A gate that has only ever been run
   against a clean tree is unverified.** (Same discipline as Q-RESIDUE's
   mutation proof, which caught a real bad test on first application.)
3. `STATUS.md` is generated from a recorded revision, and re-running the
   generator on an unchanged tree is a no-op.
4. No page states a language rule on its own authority (**D1**).
5. Green in **CI** — never a local `--workspace` run (`COORDINATION §12`).

## ⛔ The one ordering constraint — do not migrate `catalog/guide/` in this wave

`catalog/guide/`'s four files are literate `.ken.md` whose fences are
**checked**. Moved into `library/` as prose they silently stop being checked
and become exactly the drift-prone duplicate the frame's D1 exists to prevent
— while still *looking* authoritative.

**The `ken example` / `ken reject` fence gate (gate 4) must exist and pass
before any of that content moves.** Wave 0 records the migration ledger;
Wave 3 performs the migration. If you find yourself moving guide content in
this wave, stop.

## Notes

- **D4 is a commitment, not yet a verified capability.** If you learn during
  Wave 0 which structural facts the checked artifact format can and cannot
  express today, record it — it directly shapes Wave 5's frame. A fact that
  cannot be generated gets **authored and labelled as authored.**
- **`library/agents/` is out of scope for Wave 0** (that is Wave 2). Do not
  pre-build its tree.
- Product context only under `library/`; federation workflow, roles, merge
  flow and fleet memory stay under `agent/` and remain Steward-owned (**D3**).
- If this starts feeling larger than an M, **stop and tell me** rather than
  growing the scope.
