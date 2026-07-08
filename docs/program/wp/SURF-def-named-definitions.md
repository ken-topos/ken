# SURF-def-named-definitions — math-facing named definitions

**Steward frame -> Spec enclave.** Owner after design: undecided. Gate:
Spec/conformance design approval for syntax and meaning before any Language
implementation routing.

**Status:** framed for Spec D0. **Size:** S/M. **Risk:** medium — this sits
near existing declaration keywords and must not become an unprincipled catch-all
for terms, types, propositions, and proofs.

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

## Objective

Run a Spec-enclave design pass that decides whether `def` belongs in the
surface language, and if so pins its exact meaning tightly enough for a later
implementation WP.

The desired output is one of:

1. **No new keyword.** Document that `type`, `const` / `fn`, `record`,
   `prop`, `lemma`, and attached `proof` already cover the math-facing
   definition use cases, and add style guidance for which form to use.
2. **A narrow `def` spelling.** Specify grammar, name resolution,
   visibility/import behavior, elaboration target, conformance seeds, and
   catalog style. The spelling must elaborate to existing checked declarations
   and must not add kernel authority.

## D0 Questions

Spec D0 should answer these before authoring normative text:

- What exact thing would `def` denote?
  - A transparent term abbreviation?
  - A transparent type abbreviation?
  - A proposition/property bundle?
  - A documentation-facing synonym for one existing declaration class?
- If `def` can denote more than one of those, what prevents it from weakening
  Ken's settled readability split among `const`, `fn`, `proc`, `type`, `prop`,
  `lemma`, and `proof`?
- If the use case is "combine a set of properties into a single name," is that
  already a `type`, `record`, `prop`, or `lemma` pattern?
- If `def` exists, is it pure-only by construction? If so, how does the
  elaborator reject accidental effectful bodies without duplicating `fn` /
  `proc` purity rules?
- If `def` introduces a name usable by later proofs, is its transparency and
  delta-unfolding behavior identical to the existing target it elaborates to?
- What source examples should catalog authors learn from, especially around
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
- If accepted: normative edits to the surface grammar/declaration docs and
  conformance seed text for positive and negative cases.
- If rejected: a short normative/style note explaining the preferred existing
  declaration forms for named concepts and property bundles.
- A clear follow-on routing recommendation: Language implementation, catalog
  style cleanup, or no implementation work.
