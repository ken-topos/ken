# `parsing` -- source artifacts, spans, parsers, syntax, and diagnostics

**Spec catalog entry:** `spec/50-stdlib/59-parsing-syntax-diagnostics.md`.
This package provides the source/span core, parser result surface, and a
fully parenthesized Boolean expression grammar.

## Public API

- `SourceId`, `Source`, `IsUtf8`, `EmptyBytes`, `NonEmptyBytes`,
  `UnitByteLength`, `SourceLength`, `sourceId`, `sourceBytes`, `sourceUtf8`,
  `sourceLength`, `sourceLengthUnit`, `sourceLengthUnitValid`, and
  `sourceLengthValid`.
- `Span`, `spanStart`, `spanEnd`, `LessEqNat`, and `ValidSpan`.
- `Located`, `locatedSource`, `locatedSpan`, `locatedValue`, and
  `ValidLocated`.
- `validZeroWidthSpan` as a checked helper for the common valid
  zero-width-span case.
- `ParseError`, `errorSource`, and `errorSpan`.
- `ParseResult a = Parsed ... | Failed ...`.
- `Parser a`, `ParsedValid`, `FailedValid`, `ParseResultValid`,
  `ParserValid`, `ParserTotal`, `ParserSourceLocal`, and `ParserLaws`.
- `parserPure` and `parserFail` as the basic parser producers.
- `BoolExpr = BTrue | BFalse | BNot BoolExpr | BAnd BoolExpr BoolExpr`.
- `Syntax a`, `syntaxRoot`, `syntaxChildren`, `eraseSpans`,
  `ValidLocatedList`, and `ValidSyntax`.
- `parseBoolExpr`, `printBoolExpr`, and `formatBoolExpr`.

## Contract

- `Source` is a checked record carrying artifact identity, original `Bytes`,
  UTF-8 evidence, a `Nat` byte length, a non-empty byte-atomic unit witness for
  converting that `Nat` count to `Int`, and a proof that the converted recorded
  length is the byte length of `sourceBytes`. `String` is not the offset basis.
- `Span` carries only half-open byte endpoints. A bare `Span` does not identify
  a source artifact.
- `ValidSpan s sp` requires `spanStart sp <= spanEnd sp <= sourceLength s`.
- `Located a` pairs a value with a `SourceId` and `Span`; `ValidLocated`
  checks both the source id and span against a concrete `Source`.
- `Parser a` is total over well-formed calls by returning `ParseResult a`.
  Success and failure validity are public predicates: `ParsedValid` requires a
  valid consumed span with `spanStart = start` and `spanEnd = next`;
  `FailedValid` requires the error to point at the input source and carry a
  valid span.
- Repetition is not exported. There is no exported unguarded `many` or
  fuel-bounded repetition helper until the package can also expose a checked
  progress/next-validity surface.
- `parseBoolExpr` recognizes the Boolean grammar over source bytes: `true`, `false`,
  `(not e)`, and `(and e1 e2)`. There is no precedence table; `true and false`
  rejects. `printBoolExpr` emits canonical ASCII bytes, and `formatBoolExpr`
  parses a source and prints the erased tree.

## Derivation path and trusted-base delta

The package surface is ordinary Ken data, a class-backed record, transparent
functions, and proof-returning definitions over `Nat`, `Bool`, `Bytes`,
`Equal`, `And`, `List`, and parser-result data. It adds no kernel primitive,
compiler parser hook, CLI/source-loader behavior, export/provenance policy, or
language-semantics change. Expected `trusted_base()` delta is zero.

## Deferred Work

Diagnostics and `.ken.md` derived-view examples are deferred to later package
work.
