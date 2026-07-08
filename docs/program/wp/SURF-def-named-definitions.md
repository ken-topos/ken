# SURF-def-named-definitions — math-facing named definitions

**Steward frame -> Spec enclave.** Owner after design: undecided. Gate:
Spec/conformance design approval for syntax and meaning before any Language
implementation routing.

**Status:** framed for Spec D0. **Size:** S/M. **Risk:** medium — this sits
near existing declaration keywords and must not become an unprincipled catch-
all for terms, types, propositions, and proofs.

## Trigger

The `SURF-named-proof-claims` discussion settled a math-facing vocabulary for
semantic claims:

- `prop` names a proposition family / claim shape.
- `lemma` names a reusable standalone proof theorem.
- `proof <name> for <subject>` names a proof attached to a subject, referenced
  canonically as `subject::name`.

That leaves one remaining math-presentation analogue: a named definition,
usually written `def` in proof-assistant literature. The immediate catalog use
case is not executable codegen. It is source readability for proof-heavy
catalog entries where a human wants to name a concept or bundle of already
proved properties before using that name in later propositions and proofs.

The open question is whether Ken needs a distinct `def` surface form, or
whether the existing vocabulary already covers the space:

- `const` / `fn` / `proc` name checked computational definitions with explicit
  purity.
- `type` names transparent type aliases and refinement / Sigma / Pi type
  abbreviations.
- `record` names bundled dependent data.
- `prop`, `lemma`, and attached `proof` name checked propositions and proofs.

## D0 outcome

`def` is **out of scope** for this WP. The existing surface already covers the
named-concept and property-bundle use cases, so this WP routes to **style
guidance only**.

## Objective

Write the style guidance that tells catalog authors which existing form to use
when they want a readable named concept, bundle of properties, or reusable
proof-facing name. The guidance must stay within the locked boundary and must
not introduce a new surface keyword.

The preferred forms are:

1. **`type`** for transparent abbreviations whose role is to name a type-level
   shape or abbreviation that should unfold transparently.
2. **`record`** for bundled data or witnesses, especially when the named
   concept needs projections.
3. **`prop`** for proposition families / claim shapes that are checked as proof
   surfaces.
4. **`lemma`** for reusable standalone theorems.
5. **Attached `proof`** for proof facts that belong to a specific subject and
   should remain under `subject::name`.
6. **`const` / `fn` / `proc`** for computational definitions; do not use
   `def` as a synonym for any of them.

## D0 Questions

Spec D0 already answered the keyword question. The remaining D1 task is to
codify the existing-form guidance for catalog authors:

- When a concept is merely a transparent abbreviation, should the reader spell
  it as `type` instead of inventing a new name class?
- When a concept bundles witnesses or fields, should the reader spell it as a
  `record` instead of a bare definition?
- When the name is a proposition family or claim shape, should the reader
  spell it as `prop` and keep the checked proof surface explicit?
- When the name is a reusable theorem, should the reader spell it as a
  `lemma` rather than attaching an unrelated computational definition?
- Which catalog examples should the style guide point at, especially around
  `AppendsTo`, `list_append`, and attached proof claims?

## Guardrails

- Do not add a kernel declaration class, trusted definition table, or new
  trusted-base authority.
- Do not weaken the settled purity surface: `const`, `fn`, and `proc` remain
  the readable computational definition split.
- Do not let `def` become an ambiguous replacement for every declaration form.
  If the chosen design cannot state a crisp elaboration target, prefer the
  "no new keyword" outcome.
- Do not reopen `prop`, `lemma`, `proof`, `::`, or explicit attached-proof
  telescope decisions from `SURF-named-proof-claims`.
- Keep any accepted syntax orthogonal to modules, imports, and visibility:
  exported names must behave like the declaration class they elaborate to.
- Keep the first pass spec-only. Implementation is a follow-on only if the
  Spec enclave accepts a new surface spelling.

## Deliverables

- A D0 boundary decision in Convo, grounded on the exact branch/base.
- This WP's routed follow-on is **catalog style cleanup** only.
