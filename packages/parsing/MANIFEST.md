# `parsing` -- source artifacts, spans, parsers, syntax, and diagnostics

**Spec catalog entry:** `spec/50-stdlib/59-parsing-syntax-diagnostics.md`.
CAT-5 D1 lands the source/span core only.

## Public API in D1

- `SourceId`, `Source`, `IsUtf8`, `SourceLength`, `sourceId`,
  `sourceBytes`, `sourceUtf8`, `sourceLength`, and `sourceLengthValid`.
- `Span`, `spanStart`, `spanEnd`, `LessEqNat`, and `ValidSpan`.
- `Located`, `locatedSource`, `locatedSpan`, `locatedValue`, and
  `ValidLocated`.
- `validZeroWidthSpan` as a checked helper for the common valid
  zero-width-span case.

## Contract

- `Source` is a checked record carrying artifact identity, original `Bytes`,
  UTF-8 evidence, a `Nat` byte length, and a proof that the recorded length is
  the byte length of `sourceBytes`. `String` is not the offset basis.
- `Span` carries only half-open byte endpoints. A bare `Span` does not identify
  a source artifact.
- `ValidSpan s sp` requires `spanStart sp <= spanEnd sp <= sourceLength s`.
- `Located a` pairs a value with a `SourceId` and `Span`; `ValidLocated`
  checks both the source id and span against a concrete `Source`.

## Derivation path and trusted-base delta

The D1 surface is ordinary Ken data, a class-backed record, transparent
functions, and proof-returning definitions over `Nat`, `Bool`, `Bytes`,
`Equal`, and `And`. It adds no kernel primitive, compiler parser hook,
CLI/source-loader behavior, export/provenance policy, or language-semantics
change. Expected `trusted_base()` delta is zero.

## Deferred CAT-5 slices

Parser result and combinator floor, the fully parenthesized Boolean grammar,
formatter laws, diagnostics, and `.ken.md` derived-view examples are deferred
to later CAT-5 build slices.
