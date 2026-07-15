# Catalog refinement pilot — small-package style application

**Owner:** Language or Librarian-assisted build team, depending on selected
package; Librarian reviews docs/navigation.
**Branch:** `wp/catalog-refinement-pilot` (after `catalog-style-guide` lands).
**Status:** Steward frame. **Queued; blocked on `catalog-style-guide`.**
**Size:** S/M. **Risk:** low/medium — behavior-preserving refactor; the risk is
accidental API/proof drift.

## 1. Objective

Apply `docs/program/07-catalog-style-guide.md` to one or two **small,
already-functional** packages before applying it to large proof-heavy bodies.
The goal is to test the refinement workflow itself: can a team improve
organization, naming, comments, and package navigation while preserving behavior
and proofs?

Candidate packages, to be confirmed at kickoff:

- **First choice:** `catalog/packages/Core/Logic/Transport.ken.md` plus
  `catalog/packages/Core/Logic/Transport.MANIFEST.md`. This is small, zero-delta, and
  proof-strategy-rich without a large law surface.
- **Second candidate:** `catalog/packages/Core/Classes/LawfulClasses.ken.md` plus its
  manifest, if the pilot needs to exercise audited-delta comments and
  zero-delta vs. primitive-carrier distinctions.
- **Narrow fallback:** a named slice of
  `catalog/packages/Core/Classes/LawfulFunctors.ken.md`, such as the `Bool` monoid
  section only. Do not take the whole file as the first pilot.

Do **not** start with `catalog/packages/Data/Collections/Map.ken.md`; use this pilot to learn
what the guide means on a tractable body first.

## 2. Scope

For the selected package(s):

- organize sections according to the style guide;
- add comments that explain proof strategy, invariants, and public law shape;
- align public/private naming with the guide;
- update `MANIFEST.md` with a source map and package navigation as needed;
- add or preserve tests that show public API/proof names still elaborate.

Out of scope:

- new semantics or new laws;
- large component split;
- CAT-4/map.ken cleanup;
- kernel, Cargo, or trusted-base changes.
- broad package renames unless the kickoff includes a compatibility map.

## 3. Acceptance Criteria

- **AC1 — behavior preserved.** Existing package tests and relevant acceptance
  tests pass on the exact head.
- **AC2 — API/proof surface preserved.** Public names remain available, or any
  rename has an explicit compatibility map approved by the owner.
- **AC3 — style guide exercised.** QA and Librarian can point to specific
  guide checklist items applied in the diff.
- **AC4 — docs improved.** The package manifest/source map tells readers where
  to find operations, laws, proof families, and trust posture.
- **AC5 — no trust drift.** `crates/ken-kernel` and `Cargo.lock` diffs are
  empty; no `Axiom`, postulate, primitive, raw `data ... : Omega`, or
  proof-surface downgrade appears.
- **AC6 — lessons captured.** The retro identifies what the guide got right,
  what was ambiguous, and what must change before large-package refinement.
- **AC7 — evidence of readability improvement.** The handoff includes a
  before/after source map, the public names checked, tests run on the exact
  head, and the trust-drift grep result.

## 4. Review Path

Owning team leader -> implementer -> QA, with **Librarian** review for durable
docs/navigation. Architect is required only if a refactor touches proof
boundaries, law shapes, abstraction boundaries, trust-delta claims, or an
observable package split.

## 5. Follow-on

After this pilot lands and retros are in, the Steward may frame large-component
refinement WPs. The maps/sets/relations refinement should wait until
`CAT-4-build` is merged and this pilot has validated the style workflow.
