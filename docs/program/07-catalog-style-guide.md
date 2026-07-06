# Catalog style guide

**Owned by:** Spec enclave for the standard, Steward/Librarian for durable
navigation. This guide governs first-party packages under `packages/`; it does
not change language semantics, proof obligations, or the package trust model.

The first-party catalog is not only a place where useful code lives. It is the
reference corpus for how Ken code should state behavior, carry proofs, disclose
assumptions, and stay readable to a human reviewer. The guide is therefore a
review contract for catalog refinement WPs: a package may first land as a
functional, proved artifact, then a follow-on refinement WP raises it to this
standard without changing its behavior.

## 1. Scope

This guide applies to ordinary Ken packages, their `MANIFEST.md` files, and
package README/navigation docs. It is guidance for source organization, naming,
comments, proof presentation, and refinement review.

It is not:

- a new surface-language rule;
- a substitute for inhabited proof terms;
- permission to add `Axiom`, postulates, primitives, kernel features, or
  trusted-base entries;
- a reason to rewrite an active functional-build branch before it is gated.

Functional build WPs and refinement WPs have different bars:

- A **functional build** must implement the requested component, prove the
  required laws, declare its trust delta honestly, and pass its gates. The
  source may still carry discovery scaffolding, local helper names, and sparse
  explanation if the behavior and proofs are sound.
- A **catalog refinement** preserves behavior while improving organization,
  names, comments, package docs, and reader navigation. It is a real WP with its
  own gate, not optional cleanup hidden in a retro.

## 2. Exemplary Ken

First-party catalog code is exemplary when a reviewer can answer these questions
from the source and manifest without reverse-engineering the build history:

1. What abstraction does this package provide, and which spec chapter/WP owns
   the contract?
2. Which public names are stable, and where are their laws stated?
3. Which proof strategy discharges each law: definitional equality, finite
   case split, induction, transport, dictionary field reuse, or another named
   route?
4. What is the derivation path from built-ins to this package?
5. What is the `trusted_base()` delta, including every honest exception?
6. Which helpers are private scaffolding, and why do they exist?
7. Which tests or conformance seeds prove behavior preservation and trust
   posture?

The code should optimize for the human reader who is checking the proof, not for
the agent that found it. If a line or helper is hard to understand but
load-bearing, explain the invariant or proof move. If a helper is only an
artifact of discovery, rename or fold it during refinement.

## 3. Package Layout

Each package directory keeps the established catalog shape:

```text
packages/<package>/
  MANIFEST.md
  <module>.ken
  ...
```

`MANIFEST.md` is the package contract. The Ken files are the realized source.
The manifest must not be a prose duplicate of every line of source, but it must
give enough structure for a reviewer to navigate the source deliberately.

### 3.1 Source Order

Within a module, prefer this order unless Ken's current elaboration constraints
require a helper to be defined earlier:

1. Package header comment.
2. Imports and local aliases.
3. Public data/class declarations.
4. Public operations.
5. Public law statements and law-carrying classes.
6. Private helper operations grouped by the law or instance they support.
7. Proof terms, ordered by proof family.
8. Instance/dictionary assembly.
9. Narrow executable probes, only when the package uses source-local probes
   rather than external tests.

When dependencies force helper-before-public ordering, keep the reader-facing
order in the manifest's source map. The source may satisfy the elaborator; the
manifest must satisfy the reviewer.

Use section comments to make large files scanable. A section should name the
public abstraction or proof family it serves, not a WP milestone such as "D2" or
"blocker fix".

### 3.2 Public And Private Boundaries

Public names are the package API and proof surface. They should be stable across
refinement unless the WP explicitly includes a compatibility map approved by the
owner.

Private helpers should reveal their role in the name and sit close to the proof
family that uses them. If a private helper becomes useful to a sibling package,
do not copy it. Either move it to the right shared package in a separate scoped
WP or record why it remains local.

### 3.3 Splitting Files

A refinement WP may split files when the split improves review without changing
the public contract. Good split boundaries are:

- a public package family, such as algebra classes vs. functor classes;
- an implementation family, such as operations vs. law proofs;
- a carrier-specific instance family, such as `Bool` proofs vs. `List` proofs;
- a reusable helper package whose derivation path and trust delta are clear.

A split must preserve imports, manifests, public names, and tests. If a rename
is necessary, the WP must include a compatibility map listing old name, new
name, reason, and consumers checked.

Do not split only because a proof was hard to write. Split because the resulting
artifact is easier to review.

## 4. Comments

Comments are required where they carry proof-review information. Comments should
explain a contract, invariant, proof strategy, trust posture, or non-obvious
elaboration constraint. They should not restate the syntax directly below them.

Required comment classes:

- **Package header.** Names the spec chapter/WP, public abstraction, derivation
  path, and trust delta summary.
- **Section comments.** Divide operations, laws, helpers, proofs, and instances.
- **Law comments.** State the proposition in reader terms and the intended proof
  route.
- **Helper comments.** Say why the helper exists when that is not obvious from
  the name.
- **Trust comments.** Explain every `Axiom`, primitive wrapper, audited delta,
  or intentionally tested-not-trusted boundary.
- **Staging comments.** Name the gated future capability when the package cannot
  yet prove a natural law.

Example law comment:

```ken
-- Right unit for append. The base branch reduces both endpoints to `Nil`,
-- so the equality collapses to `Top` and closes with `tt`; the step branch
-- lifts the tail induction hypothesis under `Cons` with `cong`.
fn list_right_unit ...
```

Poor comments merely paraphrase syntax:

```ken
-- Defines list_right_unit.
fn list_right_unit ...
```

Keep comments short enough to scan. If a proof needs a long explanation, prefer
a compact source comment plus a manifest subsection that gives the proof map.

## 5. Naming

Names should tell a reviewer the abstraction, property, and role. Do not encode
the WP history into durable names.

General rules:

- Use lowercase identifiers for term/type parameters that are referenced in term
  bodies. Capitalized identifiers are constructor-shaped in the current surface.
- Public operations should have stable domain/action names:
  `list_append`, `bool_and`, `compareChar`.
- Class fields should stay short and conventional when the class gives the
  context: `map`, `foldr`, `assoc`, `left_unit`.
- Public law proofs should name subject plus law:
  `list_assoc`, `list_right_unit`, `band_left_unit`.
- Private helpers should add a role suffix:
  `_step`, `_case`, `_dispatch`, `_bridge`, `_locality`, `_expected`.
- Avoid opaque sequence names (`lemma1`, `helper2`, `d3_case`) in durable
  catalog source.
- Avoid overclaiming names. A tested function should not be named as if it were
  a lawful instance or proof-carrying theorem.

Role suffixes should mean what they say:

| Suffix | Use |
|---|---|
| `_step` | One inductive or recursive step. |
| `_case` | A finite case-split branch helper. |
| `_dispatch` | Chooses among already-named cases or branches. |
| `_bridge` | Converts between equivalent statement shapes. |
| `_locality` | Proves a result is unchanged outside a focused key/path. |
| `_expected` | Computes the expected value used by a statement. |

Refinement may rename helpers freely if they are private. Public renames require
the compatibility map described in section 3.3.

## 6. Proof Presentation

Proof-carrying code should show the shape of the proof, not hide it behind a
mass of incidental helpers.

Preferred proof-family structure:

1. State the public law or class field.
2. Define the operation being proved, if it is not already public.
3. Define the minimum local helpers needed to make the proof readable.
4. Prove carrier-specific lemmas in a stable order: base/finite cases,
   induction steps, bridge lemmas, final assembly.
5. Assemble the instance or public theorem.

Every law field that claims to be proved must be inhabited by a real proof term.
An `Axiom` is allowed only when the package's contract explicitly permits an
audited delta, such as a primitive carrier whose universally quantified laws are
not kernel-provable. In that case:

- the manifest lists the delta;
- the source comment explains why the proof is not available at this layer;
- conformance/tests distinguish the audited-delta case from a zero-delta lawful
  exemplar;
- the package does not present the instance as zero-delta.

Do not replace a hard proof with a more literate statement of the law. The
source must still carry the proof term, or the package must disclose the
trusted-base delta.

## 7. Manifest And Docs

Every package manifest should contain these sections, in this order unless a
package has a reason to differ:

1. Spec catalog entry and build/refinement WP.
2. Public API.
3. Source map.
4. Derivation path.
5. `trusted_base()` delta.
6. Proof families.
7. Consumers and compatibility notes.
8. Validation evidence or the conformance seed that owns validation.

A source map is a compact table from reader task to file section:

| Reader task | Where |
|---|---|
| Public operations | `<module>.ken` section "Operations" |
| Law statements | `<module>.ken` section "Laws" |
| `Bool` instance proofs | `<module>.ken` section "Bool instances" |
| Trust delta | `MANIFEST.md` "trusted_base() delta" |

Package docs should connect source order to the spec chapter and WP acceptance
criteria. They should not copy the whole spec chapter into the manifest. Link to
the spec for the contract; use the manifest to explain how this package realizes
it.

The standard-package index in `spec/50-stdlib/README.md` remains a spec index,
not a prose-style manual. It may point to this guide, but package-level style
and navigation live in `docs/program/07-catalog-style-guide.md` and
`packages/`.

## 8. Refinement WP Contract

A refinement WP is behavior-preserving unless its kickoff explicitly says
otherwise. Its definition of done:

- Public API/proof names are preserved, or the compatibility map is approved.
- Existing package tests and relevant acceptance/conformance tests pass.
- `crates/ken-kernel` and `Cargo.lock` diffs are empty.
- No new `Axiom`, postulate, primitive, opaque trusted entry, raw
  proof-relevant `data ... : Omega`, or proof-surface downgrade appears.
- `trusted_base()` delta is unchanged or narrowed.
- The manifest/source map reflects the new organization.
- The diff improves at least one concrete guide axis: organization, naming,
  comments, proof-family grouping, docs, or package split.
- The retro records which guide rules were useful and which were ambiguous.

Review roles:

- **Owning build team:** behavior preservation, imports, public names, package
  tests, and compatibility map.
- **QA:** gates, diff hygiene, trust-drift grep, and exact-head validation.
- **Librarian:** durable navigation, manifest/source map, README/spec index
  pointers, and whether the package is findable.
- **Architect:** only when the refinement crosses a proof boundary, changes a
  law shape, changes an abstraction boundary, alters a trust-delta claim, or
  introduces a split that changes module/package boundaries in a way clients may
  observe.

The Architect is not required for a pure naming/comment/source-map refinement
whose proof terms, public laws, and abstraction boundaries are unchanged.

## 9. Pilot Policy

Validate the guide on one or two small packages before applying it to large
proof-heavy bodies. The first pilot should prefer:

- `packages/transport/transport.ken` plus its manifest: small, zero-delta, and
  proof-strategy-rich without a large law surface;
- optionally `packages/lawful-classes/lawful_classes.ken` if the pilot needs to
  exercise audited-delta comments and zero-delta/primitive-carrier distinctions.

A narrow slice of `packages/lawful-functors/lawful_functors.ken` is acceptable
only after the pilot has a precise slice boundary, such as the `Bool` monoid
section. Do not begin with `packages/collections/map.ken` or any CAT-4
proof-heavy body.

Pilot evidence must show readability improved without behavior drift:

- before/after source map;
- list of public names checked;
- tests run on the exact head;
- trust-drift grep results;
- short reviewer note naming which guide checklist items were exercised.

## 10. Checklists

### Functional Build Gate

- [ ] Component implements the requested behavior.
- [ ] Required laws are real proof terms, or the trusted delta is explicit and
      permitted by the spec.
- [ ] Derivation path is stated in the manifest.
- [ ] Relevant conformance/acceptance tests pass.
- [ ] Kernel/Cargo lock invariants are checked when the WP claims them.
- [ ] Rough organization or naming is acceptable only if a refinement follow-on
      is recorded.

### Refinement Author Checklist

- [ ] No package semantics, law shapes, or proof requirements were weakened.
- [ ] Public API/proof names are preserved or mapped.
- [ ] Sections and helper names reveal the proof-family structure.
- [ ] Required comments explain strategy, invariants, and trust posture.
- [ ] Manifest source map and trust delta are current.
- [ ] Small-package pilot rule is respected unless a prior pilot has landed.
- [ ] Tests and trust-drift greps are recorded.

### Review Checklist

- [ ] Diff is docs/style/package-organization only for the stated scope.
- [ ] Existing tests cover behavior preservation.
- [ ] No new `Axiom`, postulate, primitive, or trusted entry appears.
- [ ] Any file split preserves imports and public consumers.
- [ ] Package docs connect source order to spec chapters and acceptance
      criteria.
- [ ] The reviewer can find operations, laws, helpers, proofs, and trust posture
      without reading the WP thread.
