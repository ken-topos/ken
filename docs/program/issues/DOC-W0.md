---
id: DOC-W0
title: "documentation Wave 0 — library/ charter and currency substrate"
status: closed
owner: doc
size: M
gate: none
depends_on: []
blocks: []
closed: 2026-07-22
merged: 6be9754b5fc3c6c7f9d026ef3f113b754c658c8b
github: 830
origin: docs/program/12-documentation-program.md (Steward frame, 2026-07-21) from research/librarian-documentation-program-proposal.md
---

Land `library/` as Ken's product-documentation portal, with the **substrate
that makes every later page's currency checkable** — before Wave 1 produces
content at volume.

> **Re-framed 2026-07-21 for the doc team.** This item was first released to
> the Librarian as a solo seat. The operator then established a three-seat doc
> unit — **`doc-leader`** (scoping, workflow), **`doc-author`** (authoring),
> and the **Librarian** as its QA (editor, fact-checker, reviewer, plus a
> standing as-built mandate). Both new seats are wired and confirmed
> (`doc-leader` `agt_37w6sznc4nw00`, `doc-author` `agt_37w6t02849400`, both
> T2 Sonnet 5); the Librarian is **T1** (`gpt-5.6-sol`). The deliverables and
> acceptance criteria below are unchanged; only the ownership is.
>
> **Released 2026-07-21.**

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

## ✅ CLOSED 2026-07-22 — merged `origin/main @ 6be9754b` (PR #830)

Verified on `main` **by content**, not by the publisher's exit code: all eight
delivered blobs byte-identical to reviewed `d56abbb1`; `revision_resolved()`
present in `scripts/gen-doc-status.sh`; both shallow-history regressions
present by name; `walk_library()` reports symlinks and gate 1 rejects them.

**Retros in (all four):** doc-author `evt_5xd2xss4byfv4`, librarian
`evt_48xce0bsy51kq`, Architect `evt_7702wanstax1h`, doc-leader coordination
`evt_4dd99cbsx6e8h`.

### ⚠ AC-1 was met STRUCTURALLY, not SUBSTANTIVELY — the record must not overclaim

AC-1 required *"a new page cannot land without declaring what it is, what
grounds it, and **how its currency is checked**."* A revision **is** recorded
and validated as a real ancestor — but the recorded revision **certifies
nothing about the corpus**: no code path reads a cited source's bytes at
`REVISION`. The unmet half is carried by
[`DOC-CURRENCY-ANCHOR`](DOC-CURRENCY-ANCHOR.md), which **blocks Wave 1**.
A second, lower-severity gap is carried by
[`DOC-VALIDATION-BINDING`](DOC-VALIDATION-BINDING.md).

Both were found by the **adversary, post-merge** — after nine review rounds by
a T1 QA seat and the Architect. That is not a criticism of the ring; it is the
strongest available evidence for why the post-merge adversary pass exists.

### ★ The durable output — nine rounds, EIGHT findings, ONE defect class

Every finding was **a proxy standing in for the property**:

| # | proxy checked | property that mattered | found by |
|---|---|---|---|
| 1 | rejects a *fake* revision | accepts a real one, **in CI's env** | CI (red) |
| 2 | test clones `file://{repo_root}` | an **independent** history source | librarian |
| 3 | `cat-file` says object present | present **AND** ancestry provable | architect |
| 4 | symlink not *discovered* | symlink **rejected and reported** | architect |
| 5 | SHA reviewed + approved | SHA **on `origin`** | steward |
| 6 | process fix *agreed to* | seat **can perform it** | doc-author |
| 7 | `REVISION` is a real ancestor | **corpus validated against it** | adversary |
| 8 | vocabulary is a closed set | **bound to the gates it names** | adversary |

**What stopped the in-review recursion** was naming the predicate once
(`revision_resolved()` = object present AND ancestry provable) and deriving
self-heal, every deepen checkpoint, the unshallow fallback, and all diagnostics
from it — not any individual fix.

**What the last two show is that it did not go far enough.** In the adversary's
framing, which is the sharpest statement of the whole wave:

> Object-present was true. Ancestry-provable is true. `REVISION`-is-an-ancestor
> is true. **A false proxy gets caught in review; a _true_ one is what ships.**
> Nine rounds of careful review kept converging on better and better *true
> statements about the anchor* without anyone asking what the anchor was *for*.

**⇒ Standing carry for every gate-authoring WP** (adopted from the Architect's
and Librarian's retros): before implementing *or reviewing* an
environment-dependent gate, **state its complete success predicate once, from
every downstream consumer**, then build one probe per independence boundary
between its clauses. Tests, self-healing, and diagnostics all key on that
predicate — never on a convenient observable that merely implies it in the
authoring environment. And ask at **frame** time (doc-leader's carry): *what
does this check assume about where it runs, and is that assumption itself
checked?*
