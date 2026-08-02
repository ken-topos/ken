# Read Ken

## 1. Use when

Use this module to orient in an unfamiliar `.ken` or `.ken.md` file and report
what its declarations claim. Do not use it to author new code, repair a proof,
or infer behavior that is not visible in the source and cited artifacts.

## 2. Prerequisites

No Ken knowledge is assumed. For a human-facing review, also load
`../tasks/read-review.md`. Load `proof-and-trust.md` when the file contains
claims, proofs, `Axiom`, or trust accounting.

## 3. Current capability

The landed surface distinguishes pure `const` and `fn` declarations from
effectful `proc` declarations. Types, parameters, result types, proof result
types, effect rows, capability requirements, modules, imports, and exports are
all visible in source. A literate `.ken.md` file combines explanatory prose
with fences that the Ken tooling can extract and check.

## 4. Canonical forms

Read a declaration from the outside inward:

```text
keyword name parameters : result-type [visits effect-row] = body
```

For a catalog entry, first read its purpose, public API, laws, trust and
derivation, examples, validation, and references. Then inspect checked fences.
The real fragment set in
`../../learn/reading-ken/fragments.md` provides current examples for each
shape.

## 5. Invariants and prohibitions

- A signature states the contract; prose does not replace it.
- `proc` signals an effectful declaration. Do not report a `fn` as effectful
  merely because its name suggests I/O.
- A proof term establishes only its stated result type.
- A package's trust statement is local: “no new trust” does not erase trust
  inherited from an argument, instance, primitive, or imported dependency.
- Do not treat comments, filenames, or status labels as stronger evidence than
  checked declarations and generated artifacts.

## 6. Decision procedure

1. Identify the file role: plain source, literate package, example, or test.
2. Inventory declarations and their keywords.
3. Record each public input, result, effect row, capability, and proof claim.
4. Trace cited trust and validation artifacts.
5. Stop when a required dependency or generated artifact is missing; report
   the missing evidence instead of filling it in.

## 7. Failure signatures

| Signature | Likely layer | Inspect next |
|---|---|---|
| token or fence-role error | parser/literate extractor | exact source span and fence tag |
| unknown or ambiguous name | elaboration/name resolution | imports, qualifiers, declaration order |
| type mismatch | elaboration/kernel boundary | expected and inferred types |
| `unknown` result | open or partial boundary | claim status and trusted-base delta |

## 8. Validation

For a pure file, run `ken check <file>`. For a runnable file whose final
declaration is `proc main`, run `ken run <file>`. For a review, cite the exact
file and declaration names used and distinguish checked facts from explanatory
interpretation.

## 9. Authority and sources

If reporting a checked package's public declarations, class meaning, or effect
signature, load that current `catalog/packages/**/*.ken.md` source and the
applicable `spec/30-surface/33-declarations.md` or
`spec/30-surface/36-effects.md` section first. If classifying a parser or
elaboration/name-resolution failure, load the applicable
`spec/30-surface/31-lexical.md` or `spec/30-surface/39-elaboration.md` section
first. If asserting runtime behavior, a runtime boundary, or a limit on what
checking establishes about execution, load
`spec/40-runtime/42-evaluation.md` first. Current reading guidance comes from
`docs/program/07-catalog-style-guide.md` and `library/learn/reading-ken/`.
The verified revision is in `library/agents/manifest.toml`.

## 10. Known unavailable or partial behavior

`ken check` checks one file and does not exercise runtime behavior. The
roots-based loader supports cross-file imports, but the ordinary catalog
fragment check path does not demonstrate them. If a review depends on
cross-file loading, runtime effects, native execution, or an unstated package,
stop and request the relevant test or artifact; do not infer success from a
single-file check.
