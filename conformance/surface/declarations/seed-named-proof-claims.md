# Named proof claims conformance - seed cases

Format: `../../README.md`. These pin the `SURF-named-proof-claims` slice:
`prop` families, standalone `theorem`s, attached `proof` theorems, and explicit
attached-proof references.

## surface/declarations/prop-family-checked
- spec: `spec/30-surface/32-grammar.md §1`,
  `spec/30-surface/33-declarations.md §8.1`
- given:
  ```ken
  prop HasProof (A : Type) : Omega where {
    intro : HasProof A
  }
  ```
- expect: accepts
- why: `prop` names an Ω-checked proposition family; intro helpers live under
  the family namespace and do not introduce a new kernel declaration class.

## surface/declarations/theorem-checked-theorem
- spec: `spec/30-surface/32-grammar.md §1`,
  `spec/30-surface/33-declarations.md §8.3`
- given:
  ```ken
  theorem self_eq (x : Int) : x == x = refl
  ```
- expect: accepts
- why: a `theorem` is a standalone checked proof theorem in the ordinary module
  namespace.

## surface/declarations/retired-lemma-declaration-rejected
- spec: `spec/30-surface/32-grammar.md §1`,
  `spec/30-surface/33-declarations.md §8.3`
- positive:
  ```ken
  theorem kw_theorem_refl (x : Bool) : Equal Bool x x = Refl
  ```
- expect-positive: full elaboration accepts and canonical formatting produces
  exactly
  `theorem kw_theorem_refl (x : Bool) : Equal Bool x x = Refl\n`.
- negative:
  ```ken reject
  lemma kw_theorem_refl (x : Bool) : Equal Bool x x = Refl
  ```
- expect-negative: rejects with span `0..5` and exactly
  `expected 'view', 'const', 'fn', 'proc', 'let', 'prove', 'prop', 'theorem',
  'proof', 'law', 'data', 'def', 'foreign', 'temporal', 'class', 'instance',
  'derive', 'module', 'import', 'export', 'pub', 'program', 'package', or
  'space proc', found Ident("lemma")`.
- why: this is an AC-2(d) intentional residual and AC-4 negative control.
  `lemma` lexes as an ordinary identifier after the hard rename; it is neither
  an alias nor a migration diagnostic. The paired positive and negative run
  through the same elaboration-and-format harness.

## surface/declarations/attached-proof-canonical-path
- spec: `spec/30-surface/32-grammar.md §1`,
  `spec/30-surface/33-declarations.md §8.2`
- given:
  ```ken
  fn id (x : Int) : Int = x
  proof id_self for id (x : Int) : id x == x = refl
  ```
- expect: accepts
- why: an attached proof is still an ordinary checked proof term, exported
  under the canonical attached name `id::id_self`.

## surface/declarations/attached-proof-bare-name-rejected
- spec: `spec/30-surface/32-grammar.md §1`,
  `spec/30-surface/33-declarations.md §8.2`
- given:
  ```ken
  fn id (x : Int) : Int = x
  proof id_self for id (x : Int) : id x == x = refl
  theorem probe (x : Int) : id x == x = id_self x
  ```
- expect: rejects(unresolved name)
- why: attached proof names do not enter the ordinary namespace; only the
  canonical `subject::proof_name` path or an explicit selector resolves. This
  is the negative arm paired with `attached-proof-bare-selector-resolves`:
  changing only the reference from `id_self` to `proof id_self for id` flips
  the verdict.

## surface/declarations/attached-proof-bare-selector-resolves
- spec: `spec/30-surface/32-grammar.md §1`,
  `spec/30-surface/33-declarations.md §8.2`
- given:
  ```ken
  fn id (x : Int) : Int = x
  proof id_self for id (x : Int) : id x == x = refl
  theorem probe (x : Int) : id x == x = proof id_self for id x
  ```
- expect: accepts
- why: the bare `proof name for subject` expression is a primary selector atom;
  it resolves the attached proof and leaves the following `x` as an argument.
  Paired with `attached-proof-bare-name-rejected`, the explicit selector is the
  only changed variable and therefore the resolution verdict must flip.

## surface/declarations/attached-proof-selector-spellings-identical
- spec: `spec/30-surface/32-grammar.md §1`,
  `spec/30-surface/33-declarations.md §8.2`
- given:
  ```ken
  fn id (x : Int) : Int = x
  proof id_self for id (x : Int) : id x == x = refl
  theorem via_bare (x : Int) : id x == x = proof id_self for id x
  theorem via_grouped (x : Int) : id x == x = (proof id_self for id) x
  theorem via_canonical (x : Int) : id x == x = id::id_self x
  ```
- expect: accepts — all three theorem bodies elaborate to the identical
  transparent proof term
- why: bare, grouped, and canonical attached-proof references produce the same
  `EAttachedProofRef` payload and resolve to the same `id::id_self` global;
  parentheses are optional grouping, not a distinct reference form.

## Clean-room provenance

The KW-THEOREM rows and structural oracle were independently derived from the
candidate specification, the resolved Architect decision, and first
principles. No implementation under `local/refs/`, permissive reference, or
copyleft reference was consulted. This no-reference-contact statement is the
required originality record; an originality scan is not applicable.
