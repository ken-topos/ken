# 13 — Documentation migration ledger

**Status:** Wave 0 deliverable of `docs/program/12-documentation-program.md`
(`docs/program/issues/DOC-W0.md`, deliverable 6). A **record**, not the
migration itself — nothing listed below has moved as part of this wave
except where marked "performed."

This ledger tracks the disposition of the existing documentation corpus
against `library/`, per
`research/librarian-documentation-program-proposal.md` §"Migration from the
current corpus." It is process/program material, not a reader-facing
product page, so it lives under `docs/program/` rather than `library/`
(`docs/program/12-documentation-program.md` D3: federation/program history
stays outside the public product-doc navigation).

## Disposition table

| Current material | Disposition | Wave | Status |
|---|---|---|---|
| `README.md` (repo root) | Remains the public front door; shortens over time to thesis, status, and links into `library/`. | — | unchanged this wave |
| `spec/` | Remains normative and structurally unchanged. `library/` reference pages cite or derive from it, never restate it. | — | unchanged (cited from `library/introduction.md`) |
| `catalog/packages/` | Remains the canonical package source and per-package literate rationale. `library/catalog/` will generate discovery/structural reference around it. | 5 | not started |
| `catalog/guide/` (`README.md`, `decomposition-abstraction.ken.md`, `proof-techniques.ken.md`, `surface-reference.ken.md`) | **Migrates into `library/learn/`, `library/guide/`, `library/how-to/`.** ⛔ Gated: these four files are literate `.ken.md` with **checked** `` ```ken ``/`` ```ken example ``/`` ```ken reject `` fences (`crates/ken-elaborator/src/literate.rs`). They must not move until an equivalent fence-checking gate exists and passes for `library/` content — moving them first would silently stop checking them while they kept *looking* checked. | 3 | **not started — the fence gate is the precondition, see below** |
| `agent/playbooks/tools/write-ken.md` | Keeps its workflow trigger. Reusable Ken product facts move into `library/agents/`; the skill selects the appropriate pack. | 2 | not started (`library/agents/` is explicitly out of scope for Wave 0 — `docs/program/issues/DOC-W0.md`) |
| `docs/adr/` | Remains decision records. Conceptual `library/` pages cite the accepted ADR rather than teach from decision history. | — | unchanged |
| `docs/program/` | Remains internal program history and WP material, excluded from the public product-doc navigation (this ledger included). | — | unchanged |
| `conformance/` | Remains executable contract evidence. `library/` pages link cases and reuse checked fixtures rather than duplicate them. | — | unchanged |
| `research/` | Remains advisory background, not part of the ordinary reader path. | — | unchanged |

## The one ordering constraint, restated

`catalog/guide/`'s checked-fence machinery already exists and is exercised
today (`crates/ken-elaborator/tests/ken_md_literate.rs`,
`crates/ken-cli/tests/ken_check_mode.rs`, `ken check`/`ken run` on
`.ken.md` via `ElabEnv::elaborate_ken_md_file`). What Wave 0 did **not**
build is a *library-wide* gate that runs that same machinery over every
checked fence under `library/` as part of CI (Wave 0's gates 1/2/3/6 cover
manifest coverage, links, source anchors, and availability labels — not
fence elaboration, which is the proposal's documentation-gates item 4).
Wave 3 must add that gate **before** moving any `catalog/guide/` content,
per the frame's non-negotiable ordering constraint.

## D4 note — what Wave 0 learned about generation capability

The checked-fence extractor (`ken_elaborator::literate::extract_ken_md`)
classifies exactly four fence roles (source, ignore, reject, example) by an
**exact** info-string match and hard-errors on an unrecognized `ken`-tagged
opener — a typo'd role cannot silently downgrade to unchecked prose. This
is good news for D4 in one respect and a real gap in another:

- **What it can express today:** whether a fenced block elaborates, and
  whether a declared-reject block fails to elaborate for the stated
  reason. That is sufficient to keep a checked example or counter-example
  honest.
- **What it cannot express today:** none of the structural facts D4 asks
  for — signatures, dependency lists, laws, effect rows, capabilities,
  platform availability, or `trusted_base_delta` — are exposed as
  machine-readable output anywhere in the checked path. `elaborate_ken_md_file`
  returns `Vec<GlobalId>`, not a structural record. Generating `library/catalog/`
  content (Wave 5) needs a new extraction surface over the elaborator's
  environment; it does not exist yet. Record this now so Wave 5 is framed
  against reality rather than against the commitment.

Until that surface exists, any structural catalog fact Wave 5 needs gets
**authored and labelled as authored**, never generated-looking prose
(`docs/program/12-documentation-program.md` D4 note).
