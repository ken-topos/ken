---
id: DOC-W6-AGENT-EVAL
title: "Wave 6 residual — the cold-context agent evaluation certifies agent_core_ready against a corpus 3.4x smaller than today's, and three of the four pack-selected core modules have changed since"
status: ready
owner: doc
size: M
gate: none
depends_on: [DOC-W5D-INDEXES]
blocks: []
github: null
origin: "Steward, 2026-08-02. Wave 6's four Produces items measured at origin/main = 5a0fd8e6; three are blocked by absent users, absent releases, or the operator ruling in f52b0f61. Agent-pack evaluation is the one live component. Filed under section 2a-bis so the doc ring has framed work after Wave 5 closes."
---

# The suite's verdict is stale on both axes

`library/agents/evaluations/results-2026-07-24.toml` records
`final_suite_ready = true` — the corpus's only evidence that a cold agent given
a pack can work in Ken without inventing syntax or capabilities.

Measured at `main = 5a0fd8e6` against the run's commit `d3b9f36c`
(2026-07-25 01:14):

| axis | then | now |
|---|---|---|
| `library/` markdown files | 26 | 89 |
| `library/reference/` files | absent | 54 added |
| pack-selected `core/` modules changed since | — | 3 of 4 |

`core/proof-and-trust.md`, `core/toolchain.md`, and `core/write-ken.md` have
all changed since the run that certified them. **The verdict was established
against different bytes and a corpus a third the size.**

## The new failure mode that did not exist at the last run

The suite scores `cited_authority` per task, and the 2026-07-24 run's
`authority_paths` cite raw `catalog/packages/...` fence files. Waves 3-5 then
landed `library/reference/catalog/` — 39 derived cards and five indexes — as a
**reader-facing projection** of exactly that material.

§4c is explicit that **no `library/` page is normative**: where a reader needs
the rule, the page's job is to name the spec section. So a cold agent can now
reach a plausible, well-formed, *derived* page where a normative source exists,
and cite it. **That is a wrong-answer path the last run could not have
exercised, because the pages did not exist.**

The `find-package-by-task` task is the sharpest instance: Wave 5 shipped
`subjects.md`, an index built precisely to answer "select a package by task" —
the thing that task measures.

## Scope note

This node measures. It does not build a currency mechanism: `f52b0f61` and
`LIB-GATE-DECOUPLE` (`f84e4804`) removed that coupling by operator ruling, and
the Wave 6 note in `12-documentation-program.md` records why the wave's other
three Produces items are not framable today.

Frame: `docs/program/wp/DOC-W6-AGENT-EVAL.md`.
