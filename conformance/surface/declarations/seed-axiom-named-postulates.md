# Axiom declaration and named-postulate conformance

Format: `../../README.md`. These cases pin the additive declaration sugar and
the owner-label audit surface. Labels describe provenance; `GlobalId` remains
the identity.

## surface/declarations/axiom-mechanical-sugar

- spec: `spec/30-surface/32-grammar.md §1`,
  `spec/30-surface/39-elaboration.md §5.4`
- given:

  ```ken
  axiom assumed_top : Top
  ```

- expect: accepts and elaborates exactly as
  `lemma assumed_top : Top = Axiom`; the trusted-base entry is labeled
  `assumed_top`.
- why: `axiom` adds only declaration syntax. It introduces no new kernel term
  or elaboration rule.

## surface/declarations/repeated-axiom-shares-owner-not-identity

- spec: `spec/30-surface/39-elaboration.md §5.4`,
  `spec/60-security/64-trust-model.md §1.1`
- given:

  ```ken
  lemma choose (x : Top) (y : Top) : Top = x
  lemma shared : Top = choose Axiom Axiom
  ```

- expect: accepts and adds two trusted-base entries labeled `shared`, with two
  distinct `GlobalId`s.
- why: the semantic owner supplies provenance. Occurrence order is not part of
  the label and must not be encoded with an index, counter, or generated name.
