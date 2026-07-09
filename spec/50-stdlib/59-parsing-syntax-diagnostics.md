# Parsing, syntax, and diagnostics (CAT-5, Layer 3)

> Status: **DRAFT v0** (CAT-5). This chapter is the **contract** for the
> Layer-3 catalog package: source artifacts, byte spans, total parsers, small
> package-owned grammars, parser/printer and formatter laws, and source-located
> diagnostics. It is an ordinary standard package, expected under
> `catalog/packages/parsing/`, with **zero kernel/trusted-base/language-semantics
> change**. It does **not** replace Ken's compiler lexer/parser, expose the
> compiler's internal AST, claim full Ken syntax reflection, or implement
> `.ken.md`; it only consumes the source-identity seam pinned there.

## 1. Package Shape And Boundary

CAT-5 ships as one package, `catalog/packages/parsing/`, rather than a split
`source`/`parsing` pair. The source/span types are inseparable from parser and
diagnostic validity in this first slice: a parser that can return an unlocated
failure, or a diagnostic whose span is not checked against its source, has not
implemented the abstraction. A later refinement WP may split files or package
names if that improves review, but the v1 public contract is one catalog entry.

The package namespace is the ordinary catalog namespace `ken.parse` from the
campaign report. It derives from:

- built-in `Bytes`, `String`, `Bool`, and primitive byte/string operations
  (`37`/`38`);
- prelude/package `Nat`, `List`, `Option`, `Result`, `Pair`, and `Equal`;
- CAT-1/CAT-2/CAT-3 list and lawful-class facilities where useful;
- no kernel primitive, no compiler parser hook, and no hidden built-in syntax
  object.

`trusted_base()` delta is expected to be **empty**. Any package implementation
that postulates parser totality, span validity, or round-trip laws has failed
the v1 contract unless a future WP explicitly scopes an audited delta.

## 2. Source Artifacts

`Source` is a package value that represents one immutable source artifact:

```
SourceId : Type
IsUtf8  : Bytes -> Prop

Source : Type
sourceId     : Source -> SourceId
sourceBytes  : Source -> Bytes
sourceUtf8   : (s : Source) -> IsUtf8 (sourceBytes s)
sourceLength : Source -> Nat       -- byte length of sourceBytes s
```

The canonical identity is the artifact identity and the bytes it names. `String`
is **not** the offset basis for `Source`: Ken `String` values are stored as
NFC-normalized UTF-8 (`37 §2.1`), so constructing a `String` can change byte
layout. CAT-5 spans index the original `Bytes` of the source artifact. A decoded
or normalized text value is a derived view, useful for display and token
classification, but it is not the identity used by spans.

`SourceId` is an opaque package value: a path, content hash, in-memory label, or
loader handle may produce it, but the package laws only require stable equality
within one run and no accidental conflation of different artifacts. The package
does not decide supply-chain hashing or export provenance (`63`); those remain
their own layers.

Derived source views are allowed only when their offset basis is explicit:

- an **offset-preserving view** has the same byte length and byte indices as the
  parent artifact. The active `.ken.md` blanking design is in this class: the
  original `.ken.md` artifact owns identity, and the blanked buffer is only a
  lexer input view.
- a **non-offset-preserving view** such as concatenation, extraction without
  padding, pretty-printed output, or normalized text is a distinct source
  artifact for CAT-5 v1. Reporting its offsets as if they were the parent
  artifact is invalid unless a future source-map WP explicitly adds that map.

Line and column positions are derived by scanning the byte artifact for LF
boundaries. They are display views, never span identity. CRLF input is tolerated
as surface source (`31 §1`), but CAT-5 v1's laws are stated over byte offsets
and LF-derived line views after the source artifact has been admitted.

## 3. Spans And Located Values

A `Span` is a half-open byte interval:

```
Span : Type
spanStart : Span -> Nat
spanEnd   : Span -> Nat

LessEqNat : Nat -> Nat -> Prop

ValidSpan (s : Source) (sp : Span) : Prop =
  And (LessEqNat (spanStart sp) (spanEnd sp))
      (LessEqNat (spanEnd sp) (sourceLength s))
```

`LessEqNat` is the package's ordinary decidable `Nat` order predicate. The
representation may use the CAT-4 `leqNat` carrier once it lands, but the CAT-5
contract is the proposition above: `0 <= start <= end <= sourceLength`.

`Span` equality is equality of byte endpoints; source identity is supplied by
the value that carries the span. A bare `Span` does not identify a source.

```
Located (a : Type) : Type
locatedSource : Located a -> SourceId
locatedSpan   : Located a -> Span
locatedValue  : Located a -> a

ValidLocated (s : Source) (x : Located a) : Prop =
  And (Equal SourceId (locatedSource x) (sourceId s))
      (ValidSpan s (locatedSpan x))
```

Slicing is valid only when it preserves the span basis:

```
sliceBytes : (s : Source) -> (sp : Span) -> ValidSpan s sp -> Bytes
```

`sliceBytes s sp h` returns bytes `sp.start .. sp.end` from `sourceBytes s`.
If a caller wants a `String` view of the slice, it must also prove the interval
starts and ends on UTF-8 scalar boundaries; CAT-5 does not pretend every valid
byte span is a valid standalone string.

The first source-span discriminator is offset preservation: if a parser is run
on a view of `s` starting at byte offset `k`, a token that begins at byte
`k + n` in `s` reports span start `k + n`, not `n` in the copied slice and not
an offset in a concatenated buffer.

## 4. Tokens, Grammars, And Total Parse Results

CAT-5 describes **package-owned grammars**, not Ken's compiler grammar. Public
syntax trees are ordinary data declared by the package. A conforming package
must not expose compiler-internal AST constructors from `39 §5.2` as its public
API and must not claim to parse all Ken source.

A parser is total over well-formed inputs:

```
ParseError : Type
errorSource : ParseError -> SourceId
errorSpan   : ParseError -> Span

ParseResult (a : Type) : Type
  = Parsed (value : a) (consumed : Span) (next : Nat)
  | Failed (err : ParseError)

Parser (a : Type) : Type =
  (s : Source) -> (start : Nat) ->
  LessEqNat start (sourceLength s) ->
  ParseResult a
```

The laws are:

1. **Success validity.** If `p s i h = Parsed v consumed next`, then
   `ValidSpan s consumed`, `spanStart consumed = i`, and
   `spanEnd consumed = next`.
2. **Failure validity.** If `p s i h = Failed e`, then
   `errorSource e = sourceId s` and `ValidSpan s (errorSpan e)`.
3. **Totality.** Every well-typed parser call returns `Parsed` or `Failed`.
   It does not throw, panic, loop, or return an unlocated failure.
4. **Source locality.** A parser reports spans in the source artifact it was
   given. It must not silently switch to a copied substring, a concatenation,
   or a normalized text artifact.

Parser combinators preserve these laws. Sequencing carries the original
`SourceId` and advances by byte offset. Choice returns either the successful
branch or a structured failure whose span is valid in the same source.
Repetition is admitted only with a termination discipline: either the repeated
parser is proved to consume at least one byte on success, or the combinator
takes explicit fuel. An unguarded `many` over a zero-width parser is not a CAT-5
parser.

## 5. The V1 Grammar: Boolean Expressions

The first grammar is deliberately small, package-owned, and fully
parenthesized:

```
BoolExpr : Type
  = BTrue
  | BFalse
  | BNot  BoolExpr
  | BAnd  BoolExpr BoolExpr

expr ::= "true"
       | "false"
       | "(" "not" expr ")"
       | "(" "and" expr expr ")"
```

Whitespace may appear between tokens but not inside the keyword tokens. There
is no precedence table in v1 because the grammar is prefix and fully
parenthesized. The string `true and false` is therefore rejected by this
grammar, not parsed by an implicit precedence rule.

Any future CAT-5 grammar that is not syntactically unambiguous must make its
precedence/associativity policy explicit or return a structured ambiguity
failure with valid source spans. Silent first-parse-wins is not a law.

The package exposes a located tree:

```
Syntax (a : Type) : Type
eraseSpans : Syntax BoolExpr -> BoolExpr

parseBoolExpr  : Parser (Syntax BoolExpr)
printBoolExpr  : BoolExpr -> Bytes
formatBoolExpr : Source -> Result ParseError Bytes
```

`printBoolExpr` prints canonical ASCII bytes:

```
BTrue      -> "true"
BFalse     -> "false"
BNot e     -> "(not " ++ print e ++ ")"
BAnd x y   -> "(and " ++ print x ++ " " ++ print y ++ ")"
```

The v1 laws are:

1. **Printer/parser round trip.**
   `parseBoolExpr (sourceOf (printBoolExpr e)) 0 _` succeeds and the erased
   syntax tree is `e`.
2. **Parse/format preservation.** If `parseBoolExpr s 0 _` succeeds with
   erased tree `e` and `formatBoolExpr s = Ok bs`, then parsing
   `sourceOf bs` succeeds with erased tree `e`.
3. **Formatter idempotence.** If `formatBoolExpr s = Ok bs`, then
   `formatBoolExpr (sourceOf bs) = Ok bs`. If formatting fails, the
   `ParseError` has a valid span in `s`.
4. **Span validity.** Every node span in a parsed `Syntax BoolExpr` is valid in
   the source used for that parse.

The round-trip law is stated modulo span erasure because printing creates a new
source artifact. A tree parsed from `original.ken` and a tree parsed from
`printBoolExpr e` cannot have equal spans; their semantic equality is equality
of `eraseSpans`.

## 6. Diagnostics

A CAT-5 diagnostic is package data for source-located explanations. It is
separate from verification-layer proof-failure diagnostics (`24`/`25`), but it
uses the same honesty rule: a diagnostic explains a failure; it does not create
a proof or change the kernel verdict.

V1 diagnostics are single-artifact:

```
Severity : Type = Error | Warning | Note

RelatedSpan : Type
relatedSpan    : RelatedSpan -> Span
relatedMessage : RelatedSpan -> String

Diagnostic : Type
diagSource      : Diagnostic -> SourceId
diagPrimary     : Diagnostic -> Span
diagSecondaries : Diagnostic -> List RelatedSpan
diagSeverity    : Diagnostic -> Severity
diagCode        : Diagnostic -> String
diagMessage     : Diagnostic -> String
```

Validity is structural:

```
ValidDiagnostic (s : Source) (d : Diagnostic) : Prop =
  And (Equal SourceId (diagSource d) (sourceId s))
      (And (ValidSpan s (diagPrimary d))
           (All (ValidRelatedSpan s) (diagSecondaries d)))

ValidRelatedSpan (s : Source) (r : RelatedSpan) : Prop =
  ValidSpan s (relatedSpan r)
```

All primary and secondary spans index the named source artifact. A diagnostic
with an out-of-bounds primary span is invalid. A diagnostic whose primary span
is valid but whose secondary span is out of bounds is also invalid. Dropping
secondary spans to make validity easier is not preserving the diagnostic.

Parse errors are diagnostics with a required primary span:

```
parseErrorDiagnostic :
  (s : Source) -> (e : ParseError) ->
  Equal SourceId (errorSource e) (sourceId s) ->
  ValidSpan s (errorSpan e) ->
  Diagnostic
```

Rendering a diagnostic to human text is outside the proof obligation except
that it may not fabricate source identity or report a span that fails
`ValidDiagnostic`.

## 7. Relationship To `.ken.md`

CAT-5 consumes the `.ken.md` decision only at the source-identity boundary.
The original `.ken.md` path/content is the source artifact for diagnostics. A
blanked compiled view is a derived lexer input, not the sole identity and not a
new proof of source provenance.

Because the active `.ken.md` design preserves byte length and newlines, its
compiled view is offset-preserving: spans inside compiled fences can be reported
against the original artifact without a general source-map mechanism. CAT-5 v1
does not define CommonMark, nested fences, extraction policy, or `.ken.md`
execution. A package parser may consume an offset-preserving view, but it must
state whether spans are in the parent source or in a new generated artifact.

## 8. Build Slices

- **D1 -- source and span core.** `Source`, `Span`, validity predicates,
  byte slicing, located values, newline/line-column derived views, and the
  offset-preserving view boundary.
- **D2 -- parser result and combinators.** `ParseResult`, `Parser`,
  structured `ParseError`, sequencing/choice, repetition with consumption proof
  or fuel, and span propagation.
- **D3 -- Boolean expression grammar.** `BoolExpr`, located syntax,
  `parseBoolExpr`, `printBoolExpr`, `formatBoolExpr`, and the round-trip /
  idempotence laws.
- **D4 -- diagnostics.** `Diagnostic`, `RelatedSpan`, `ValidDiagnostic`,
  parse-error diagnostics, and rejection of out-of-bounds primary or secondary
  spans.
- **D5 -- examples.** Small examples for success, failure, source offset
  preservation, formatter idempotence, and diagnostic rendering inputs.

## 9. Acceptance Criteria

1. **Zero trust delta.** CAT-5 is ordinary Ken: no kernel diff, no primitive, no
   compiler parser hook, no `Axiom` for public laws, no export-trust behavior
   change.
2. **Byte artifact identity.** `Source` spans index the original source bytes,
   not a normalized `String` or a copied substring. Line/column is derived.
3. **Valid spans.** Every located token, syntax node, parse error, and
   diagnostic span satisfies `0 <= start <= end <= sourceLength`.
4. **Offset preservation.** Parsers over source views report original byte
   offsets when the view is offset-preserving; non-offset-preserving views are
   distinct source artifacts unless a future source map is explicit.
5. **Parser totality.** Every parser returns `Parsed` or `Failed`; failure is a
   structured value with a valid span in the same source.
6. **Small grammar boundary.** The v1 grammar is the package's Boolean
   expression grammar. It does not expose compiler-internal AST or parse all
   Ken syntax.
7. **Parser/printer law.** Parsing `printBoolExpr e` succeeds and returns `e`
   after erasing spans.
8. **Formatter law.** Formatting preserves the parsed tree modulo span erasure
   and is idempotent on successful output.
9. **Diagnostic validity.** Primary and secondary spans are all checked against
   the named source artifact; out-of-range spans reject.
10. **`.ken.md` boundary.** CAT-5 treats extracted buffers as derived
    offset-preserving views only; it does not implement or change
    `ken-md-literate`.

Conformance:
`../../conformance/stdlib/parsing/seed-cat5-parsing-syntax-diagnostics.md`.
