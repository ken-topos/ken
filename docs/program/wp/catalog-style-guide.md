# Catalog style guide — first-party package quality

**Owner:** Spec enclave (style/design) -> Steward/Librarian corpus ownership.
**Branch:** `wp/catalog-style-guide` (off current `origin/main` at pickup).
**Status:** Steward frame. **Queued; not yet kicked off.**
**Size:** S/M. **Risk:** medium — this is not kernel-risk, but it sets the
standard every future catalog package will be judged against.

**Trigger.** The current catalog work shows the natural shape of hard proof
engineering: first get the component to exist and prove its laws, then refine
the result into a readable artifact. That process is close to mathematical
discovery. Ken should make the refinement phase explicit, because the first-party
catalog is not only infrastructure; it should be exemplary Ken literature.

## 1. Objective

Develop the **Ken catalog style guide** and the operational cadence that goes
with it:

1. A first catalog component WP may merge once it is functional, proved, and
   gated, even if its source is rough from the struggle of discovery.
2. A second, explicit refinement WP raises that component to catalog standards:
   organization, naming, comments, proof narrative, harmonization with sibling
   packages, and documentation.
3. The style guide is validated first on **small bodies of code**, then applied
   to larger components such as maps/sets/relations after the pattern is proven.

This should make the process honest rather than punitive: a discovered proof is
not automatically a teachable proof. The catalog should plan for both.

## 2. Deliverables

- **`docs/program/07-catalog-style-guide.md`** — the durable style guide.
- Updates to `docs/program/06-catalog-campaign.md` describing the two-phase
  catalog cadence: **functional build** -> **catalog refinement**.
- A small-package refinement pilot frame or validation checklist, suitable for
  `catalog-refinement-pilot`.
- If needed, small pointers from `catalog/packages/README.md` and
  `spec/50-stdlib/README.md` to the style guide. Do not turn the language spec
  into a prose-style manual.

## 3. Questions for the enclave

The enclave should answer these at the level of policy, not by rewriting any one
package:

- What makes first-party catalog code exemplary Ken?
- How should proof-carrying files be organized: sections, helper layers, public
  API before private lemmas, proof-family grouping, law tables, and local
  narrative?
- What comments are required? Prefer comments that explain proof strategy,
  invariants, law shape, and why a helper exists; avoid comments that restate
  syntax.
- What naming conventions distinguish public operations, public laws, private
  worker lemmas, dispatch lemmas, bridge lemmas, and proof probes?
- How should package docs connect source order to the spec chapter and WP
  acceptance criteria?
- When should a component split across files or packages, and what compatibility
  constraints apply to imports/manifests?
- What is the definition of done for a **refinement** WP when behavior should
  not change?
- Which small packages should be the first style-guide pilots, and what evidence
  proves the guide improved readability without semantic churn?

## 4. Acceptance Criteria

- **AC1 — guide exists and is actionable.** The style guide includes concrete
  rules, examples, and review checklists; it is not only aspirational prose.
- **AC2 — two-phase lifecycle is pinned.** The campaign docs explicitly state
  that catalog components may have a rough functional build followed by a
  planned refinement WP. The refinement WP is a real work item, not optional
  cleanup hidden in retros.
- **AC3 — small-pilot-first policy.** The guide says to validate style/refactor
  practice on smaller packages before applying it to large proof-heavy bodies.
- **AC4 — no semantic churn.** This WP authors guidance only. It does not
  rewrite catalog packages and does not change language semantics.
- **AC5 — review roles are clear.** Future refinement WPs say who reviews what:
  owning build team for behavior-preservation, QA for gates, Architect for any
  proof/abstraction-boundary risk, Librarian for durable docs.

## 5. Guardrails

- Do not block active functional catalog work on this style guide.
- Do not use style guidance to weaken proof requirements. A literate statement
  of a law is not a substitute for an inhabited proof term.
- Do not create a one-off rule for `map.ken`; generalize to catalog packages.
- Do not require maximal polish in the first functional build pass. Make the
  refinement pass explicit and scheduled.
- Do not start with the largest, hardest file. Prove the refinement workflow on
  smaller packages first.

## 6. Dependencies and Sequencing

- **Can start:** once the spec enclave is free of in-flight review obligations
  and has been compacted per the Steward handoff gate.
- **Feeds:** `catalog-refinement-pilot`; then later large-component refinement
  WPs, including maps/sets/relations after CAT-4-build lands.
- **Does not block:** current CAT-4 functional proof work, current SURF-2 build
  merge review.
