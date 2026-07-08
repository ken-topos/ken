# Named proof claims conformance - seed cases

Format: `../../README.md`. These pin the `SURF-named-proof-claims` slice:
`prop` families, standalone `lemma`s, and attached `proof` theorems.

## surface/declarations/prop-family-checked
- spec: `spec/30-surface/32-grammar.md §1`, `spec/30-surface/33-declarations.md §8.1`
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
- spec: `spec/30-surface/32-grammar.md §1`, `spec/30-surface/33-declarations.md §8.3`
- given:
  ```ken
  lemma self_eq (x : Int) : x == x = refl
  ```
- expect: accepts
- why: a `lemma` is a standalone checked proof theorem in the ordinary module
  namespace.

## surface/declarations/attached-proof-canonical-path
- spec: `spec/30-surface/32-grammar.md §1`, `spec/30-surface/33-declarations.md §8.2`
- given:
  ```ken
  fn id (x : Int) : Int = x
  proof id_self for id (x : Int) : id x == x = refl
  ```
- expect: accepts
- why: an attached proof is still an ordinary checked proof term, exported
  under the canonical attached name `id::id_self`.

## surface/declarations/attached-proof-bare-name-rejected
- spec: `spec/30-surface/32-grammar.md §1`, `spec/30-surface/33-declarations.md §8.2`
- given:
  ```ken
  fn id (x : Int) : Int = x
  proof id_self for id (x : Int) : id x == x = refl
  fn probe () : Int = id_self
  ```
- expect: rejects(unresolved name)
- why: attached proof names do not enter the ordinary namespace; only the
  canonical `subject::proof_name` path resolves.
