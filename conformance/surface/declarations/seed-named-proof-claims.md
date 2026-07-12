# Named proof claims conformance - seed cases

Format: `../../README.md`. These pin the `SURF-named-proof-claims` slice:
`prop` families, standalone `lemma`s, attached `proof` theorems, and explicit
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

## surface/declarations/lemma-checked-theorem
- spec: `spec/30-surface/32-grammar.md §1`,
  `spec/30-surface/33-declarations.md §8.3`
- given:
  ```ken
  lemma self_eq (x : Int) : x == x = refl
  ```
- expect: accepts
- why: a `lemma` is a standalone checked proof theorem in the ordinary module
  namespace.

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
  lemma probe (x : Int) : id x == x = id_self x
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
  lemma probe (x : Int) : id x == x = proof id_self for id x
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
  lemma via_bare (x : Int) : id x == x = proof id_self for id x
  lemma via_grouped (x : Int) : id x == x = (proof id_self for id) x
  lemma via_canonical (x : Int) : id x == x = id::id_self x
  ```
- expect: accepts — all three lemma bodies elaborate to the identical
  transparent proof term
- why: bare, grouped, and canonical attached-proof references produce the same
  `EAttachedProofRef` payload and resolve to the same `id::id_self` global;
  parentheses are optional grouping, not a distinct reference form.
