---
id: DOC-PROGRAM-SELF-REFUTE
title: "three sites of current program law assert assurances the same corpus has already measured as absent, and 12-documentation-program.md now carries both a drift-gate claim and the measurement refuting it"
status: ready
owner: doc
size: M
gate: none
depends_on: [DOC-AGENT-CITE]
blocks: []
github: null
origin: "DOC-AGENT-CITE D5 (merged 428ee50f) recorded two current program-law assurances as corrected-only-by-record and stated 'This WP does not edit the program frame.' Task #186 independently found the third site. Steward-filed (agents cannot create tracked work per COORDINATION section 2)."
---

# The defect: a document that contains both a claim and its refutation

`docs/program/12-documentation-program.md` records, at lines 221-235 and dated
`Measured 2026-08-01 at f31e8d94`, that **the entire declared validation
vocabulary is unreachable code** and that "a document's `validation` list in
`manifest.toml` currently names checks that nothing runs."

The same document, at lines 70-72, still states that a **drift gate verifies
that section still exists**. `check_source_anchors` is one of the eleven inert
gates that measurement covers.

⇒ The contradiction is not across two documents that drifted apart. It is
inside one document, between a load-bearing decision and a measurement added
later by `f31e8d94` — which repaired two *other* self-contradictory sources and
did not reach this one.

## The three sites

| # | site | claims | refuted by |
|---|---|---|---|
| A | `12-documentation-program.md:70-72` | a drift gate verifies a cited spec section still exists | same file, 221-235 (registry unreachable) |
| B | `12-documentation-program.md:124-126` | currency is recorded by generated `STATUS.md` **and build output** | same file, 635-641; and `.github/` has **0** references to `gen-doc-status.sh` or `library/REVISION` |
| C | `06-catalog-campaign.md` (10 sites) | Findings is live purpose 4 of the catalog campaign | `07-catalog-style-guide.md` §5, which retired the Findings section 2026-07-11 |

Site B's `STATUS.md` half is **true** — it is generated. Only the build-output
channel is false.

## Why this is a node and not a stale-prose sweep

The constraint is `docs/PRINCIPLES.md`'s honesty about the boundary: current
program law is what the fleet reads to know what is enforced. A false
enforcement claim in program law is the reason a reader stops checking. Two
merged WPs measured these exact sites and each recorded them as deliberately
out of its own scope, so the finding is inherited from measurement, not from
prose.

This is **not** a proposal to re-arm anything. See judgment 2.
