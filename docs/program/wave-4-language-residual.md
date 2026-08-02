# Wave 4 language-reference residual

This report measures the nine language forms assigned to
`DOC-W4-LANGUAGE` against the existing human-audience
`library/guide/surface-reference.ken.md`. The measurement is at
`d26ccb004e61b761b142fd98fb25e2c805f51b37`.

The classification asks what each section actually does, independent of its
page title and manifest kind:

- **lookup-shaped** content gives a complete answer for the form without
  requiring the neighbouring sections;
- **explanatory** content motivates the form and teaches it through narrative,
  contrasts, or an ordered sequence of examples.

## Per-form measurement

| Form | Measured section and actual delivery | Shape | Residual |
|---|---|---|---|
| Purity keywords `const` / `fn` / `proc` | §1, lines 28–87: defines the three purity classes, demonstrates them together, teaches the current explicit-parameter convention, and preserves a rejected effect-in-`fn` example | Explanatory: the section motivates catalog practice and sends the reader forward to the effect-row and literate-format explanations | `none` |
| `def` | §2, lines 88–143: explains transparent refinement and alias definitions, demonstrates both, and contrasts them with opaque `data` through a rejection | Explanatory: the answer is developed through the transparency contrast rather than presented as a declaration lookup | `none` |
| `data` and `match` | §3, lines 144–187: teaches constructors, generated elimination, exhaustive matching, wildcard coverage, a missing-case rejection, and an `Option` example | Explanatory: the section builds the exhaustiveness rule through accepted and rejected examples | `none` |
| Refinement types | §4, lines 188–219: explains carrier-plus-obligation semantics, a refined result, conjunction orientation, and a refined parameter | Explanatory: it motivates postconditions and then develops the form through examples | `none` |
| `class` and `instance` | §5, lines 220–266: explains the dictionary model, shows declarations and instance lookup, rejects projecting from the class type, and points to law-bearing fields | Explanatory: the value-versus-type distinction is taught through a contrast and surrounding guidance | `none` |
| Effect rows (`visits`) | §6, lines 267–291: explains static and transitive checking, shows an accepted Console row, preserves an omitted-row rejection, and introduces row polymorphism | Explanatory: the section motivates the row rule and develops it through accepted and rejected cases | `none` |
| Named proof claims `prop` / `theorem` / `proof` | §7 and §7.1, lines 292–448: defines all three forms, records the current `prop where` limit, shows attachment and recursion, distinguishes `Omega` from `Type`, and gives the form-selection convention | Explanatory: the section is an ordered account of seed limits, ownership, recursion, classification, and authoring choice | `none` |
| Local `let` binding groups | §8, lines 449–541: gives group syntax, sequential non-recursive scope, formatter behaviour, naming guidance, proof and branch examples, a self-reference rejection, and the call-by-value effect rule | Explanatory: syntax and scope are interleaved with staged examples and style motivation | `none` |
| Literate `.ken.md` format | §9, lines 542–562: gives a four-row table for `ken`, `ken ignore`, `ken reject`, and `ken example`, including tangling, checking, and intended use | Lookup-shaped: the fence taxonomy is complete in one standalone table | `reclassify` |

The named-gap set is empty, so the permitted `D1` page set is also empty. The
existing guide already delivers every form; authoring a second page would
duplicate it. Eight sections remain honestly explanatory. Section 9 is already
lookup-shaped, so its only residual is the report-only `D2` classification
finding below.

## D2 reclassification finding

`surface-reference.ken.md` §9, “The `.ken.md` literate format,” is a complete
lookup table inside a page registered as `kind = "explanatory"`. Its row is a
corpus-classification question for the Librarian. This slice reports the
finding and does not change the merged page or its manifest record.

## Authorship and validation

The merged Wave 4 generation-capability report says exact syntax has no
emitter or generator. The syntax descriptions assessed here are therefore
**authored**, not generated.

The existing checked guide is 625 lines and contains every current form this
report assesses. The exact-checkout validation was:

```console
$ target/debug/ken check library/guide/surface-reference.ken.md
```

The command produced no output and exited 0. No new current-syntax display is
introduced by this report.

The section inventory used for the row boundaries was:

```console
$ grep -nE '^#{1,3} ' library/guide/surface-reference.ken.md
1:# Surface reference — the practical shape of Ken
16:## Contents
28:## 1. Purity keywords: `const`/`fn`/`proc`
88:## 2. `def`: transparent definitions
144:## 3. `data` and `match`
188:## 4. Refinement types
220:## 5. `class` and `instance`
267:## 6. Effect rows (`visits`)
292:## 7. Named proof claims: `prop`, `theorem`, `proof`
386:### 7.1 Choosing a form — the authoring convention
449:## 8. Local `let`: binding groups
542:## 9. The `.ken.md` literate format
563:## Design notes
606:## Findings
616:## References
```

Because there are no new `D1` pages, manifest registration, availability
labels, source-attestation additions, and generated STATUS closure are not
activated by this slice.
