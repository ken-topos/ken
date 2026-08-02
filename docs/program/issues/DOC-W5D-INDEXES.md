---
id: DOC-W5D-INDEXES
title: "Wave 5 closeout — build the four cross-package indexes the cards can support (declaration/type, law, effect/capability, assurance) and record why the four held-class indexes cannot be built"
status: merged
owner: doc
size: M
gate: none
depends_on: [DOC-W5C-CARDS-CAPABILITY]
blocks: []
github: null
origin: "Steward, 2026-08-02. Wave 5's stated Produces names nine indexes; slices 1-3 deliver the 39 cards and the subject index only. Filed under section 2a-bis so the doc ring has framed work when slice 3 merges."
---

# Wave 5 closeout — the wave promised nine indexes and one exists

`docs/program/12-documentation-program.md` §Wave 5 Produces names subject,
declaration/type, law, effect/capability, assurance, platform, maturity,
dependency, and reverse-dependency indexes. Measured at `main = 8c12f5d4`,
`library/reference/catalog/` holds `subjects.md` and `card-format.md` and
nothing else.

The wave's exit property is discoverability *"by what a reader wants to
accomplish and by the exact checked abstractions available."* Per-package cards
answer the first half. **Discovery by abstraction is cross-package and no
artifact answers it today.**

## The split

- **Four are buildable** — declaration/type, law, effect/capability, assurance.
  The cards carry a labelled row for each across all 39 packages.
- **Four are not** — platform, maturity, dependency, reverse-dependency. The
  precondition report measured the first two as reserved-but-uninstantiated and
  the second two as lacking any complete package-level projection. They get a
  recorded disposition, and no file.

## The measurement worth carrying into the frame

Over the 14 landed Application and Data cards, **effect/capability is
`none-declared` on all 14**. If Capability's 19 match, the effect/capability
index is empty across the whole catalog.

That is a finding about the catalog, not a failure of the WP. The frame states
plainly that an empty index is a valid result to be reported with its
population — not padded, not widened, and not quietly dropped. An empty effect
index sitting beside 19 packages under `Capability/` will look like a mistake,
so the index says why it is not.

## Sequencing

`depends_on` gates release on slice 3 merging. Filed `ready` under section
2a-bis: Wave 5 completes when `DOC-W5C-CARDS-CAPABILITY` lands, and without this
node the doc ring would have zero available work at that moment.

Frame: `docs/program/wp/DOC-W5D-INDEXES.md`.
