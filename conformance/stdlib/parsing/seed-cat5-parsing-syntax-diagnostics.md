# CAT-5 parsing / syntax / diagnostics conformance -- seed cases

Format: `../../README.md`. CAT-5 is **Layer 3** of the catalog campaign:
ordinary package support for source artifacts, byte spans, total parsers,
package-owned syntax trees, parser/printer + formatter laws, and diagnostic
span validity. The spec chapter is
`spec/50-stdlib/59-parsing-syntax-diagnostics.md`.

**Grounding.** The seed is reconciled against `59`: `Source` is a byte artifact
(`59 §2`), not a normalized `String`; `Span` is a half-open byte interval
(`59 §3`); parsers are total and return structured failures (`59 §4`); the
chosen v1 grammar is a fully parenthesized Boolean expression grammar
(`59 §5`); diagnostics are single-source values whose primary and secondary
spans must all be valid (`59 §6`); `.ken.md` is consumed only as an
offset-preserving derived view (`59 §7`). The adjacent compiler surface already
uses spans internally (`39 §5.2`/`39 §5.6`), but CAT-5 does **not** expose
that compiler AST or claim full Ken syntax reflection.

**Status.** These cases are **red-until-built** for the CAT-5 Language build:
the package and its law proofs do not exist yet. They are the contract the
build implements. Cases tagged `(soundness)` guard the zero-trust-delta and
source-validity commitments. Cases tagged `(fidelity)` guard that diagnostics
and views keep the source identity they claim.

---

## Scope -- canonical shapes

```
Source
sourceId     : Source -> SourceId
sourceBytes  : Source -> Bytes
sourceLength : Source -> Nat

Span
ValidSpan s sp := 0 <= sp.start <= sp.end <= sourceLength s

Parser a := (s : Source) -> (start : Nat) ->
            start <= sourceLength s -> ParseResult a

ParseResult a = Parsed a Span Nat | Failed ParseError

BoolExpr = BTrue | BFalse | BNot BoolExpr | BAnd BoolExpr BoolExpr
expr ::= "true" | "false" | "(" "not" expr ")" | "(" "and" expr expr ")"

parseBoolExpr  : Parser (Syntax BoolExpr)
printBoolExpr  : BoolExpr -> Bytes
formatBoolExpr : Source -> Result ParseError Bytes

ValidDiagnostic s d :=
  d.source = sourceId s
  and ValidSpan s d.primary
  and every secondary span is ValidSpan s
```

The laws compare parsed trees through `eraseSpans`, because a printed tree
creates a new source artifact and therefore new spans.

---

## AC1 / AC2 -- source artifacts and byte spans

### stdlib/parsing/source-is-byte-artifact-not-normalized-string (soundness)
- spec: `59 §2`; `37 §2.1` / `37 §2.2`
- given: a source artifact whose bytes contain two source-visible spellings that
  can normalize to the same `String` value, and a token after that text.
- expect: spans are computed over `sourceBytes`, not over a constructed
  `String`. The token's byte offset is the offset in the original artifact
  bytes. An implementation that first converts the whole artifact to Ken
  `String` and then uses the normalized string's byte positions is rejected by
  the span law.
- why: `String` is NFC-normalized at construction, so it is not a safe offset
  basis for source artifacts. This is the load-bearing CAT-5 source-identity
  pin: byte spans index the artifact, not a normalized value.

### stdlib/parsing/valid-span-half-open-bounds
- spec: `59 §3`
- given: a source of byte length `n`, a span `[start,end)`, and the
  `ValidSpan` proof obligation.
- expect: spans with `0 <= start <= end <= n` are valid, including zero-width
  spans at valid offsets; spans with `end > n` or `start > end` are rejected.
- why: AC2/AC3's basic span law. The out-of-range arm is the flip for
  diagnostics and parse errors below.

### stdlib/parsing/text-slice-requires-utf8-boundary-proof
- spec: `59 §3`; `31 §1`
- given: a valid byte span whose endpoints fall in the middle of a multi-byte
  UTF-8 scalar value, and a request for a `String` slice.
- expect: `sliceBytes` may return the raw bytes because the byte bounds are
  valid, but producing a `String` slice requires an additional scalar-boundary
  proof and is rejected without it.
- why: byte-span validity and text-slice validity are different. This prevents
  an implementation from silently treating arbitrary byte intervals as text.

### stdlib/parsing/source-view-preserves-original-offsets (fidelity)
- spec: `59 §2` / `59 §3` / `59 §7`
- given: an offset-preserving view of a source artifact, with bytes before the
  parsed region blanked or otherwise retained at the same length; a token begins
  at original byte offset `k`.
- expect: the located token span starts at `k`. Reporting `0` because the
  parser copied the region into a fresh substring is rejected.
- why: AC4. This is the source-blind implementation flip: a parser can produce
  the right AST while losing the only offset a diagnostic can act on.

---

## AC4 / AC5 -- total parser result shape

### stdlib/parsing/parser-success-carries-valid-consumed-span
- spec: `59 §4`
- given: `parseBoolExpr` succeeds on a source at start offset `i`.
- expect: the `Parsed` result carries a consumed span valid in the same source,
  with `spanStart = i` and `spanEnd = next`; every node span inside the returned
  syntax tree is also valid in that source.
- why: success is not only an AST. CAT-5 requires source-located syntax, so a
  spanless success or success with copied-buffer offsets fails.

### stdlib/parsing/parser-failure-is-structured-and-located (fidelity)
- spec: `59 §4` / `59 §6`
- given: the malformed Boolean-expression source `(and true )`.
- expect: `parseBoolExpr` returns `Failed e`, not a panic, throw, loop, or
  unstructured string. `errorSource e = sourceId s` and `errorSpan e` is a valid
  span at the missing operand or closing-region location.
- why: AC5. Parse failure is an ordinary value a program can inspect and turn
  into a valid diagnostic.

### stdlib/parsing/repetition-requires-consumption-or-fuel (soundness)
- spec: `59 §4`
- given: a repeated parser whose body can succeed while consuming zero bytes.
- expect: an unguarded `many`-style combinator is rejected by the CAT-5 parser
  laws. The implementation must either prove successful iterations consume at
  least one byte or require explicit fuel.
- why: parser totality includes termination. A parser that can loop on a
  well-typed call is not a total package parser.

---

## AC6 / AC7 / AC8 -- small grammar and printer/formatter laws

### stdlib/parsing/bool-expr-printer-parser-round-trip
- spec: `59 §5`
- given: any `BoolExpr` value `e`.
- expect: parsing `printBoolExpr e` succeeds, and the parsed syntax tree erases
  to `e`.
- why: AC7. The equality is modulo span erasure because the printed bytes are a
  new source artifact.

### stdlib/parsing/bool-expr-formatter-idempotent
- spec: `59 §5`
- given: a parseable source for the Boolean-expression grammar with arbitrary
  allowed whitespace.
- expect: `formatBoolExpr` succeeds with canonical bytes; parsing those bytes
  as a generated source erases to the same `BoolExpr`; formatting that
  generated source returns the same bytes again.
- why: AC8. Formatting is not evidence of parser correctness by itself; the
  case asserts both parse preservation and idempotence.

### stdlib/parsing/no-implicit-precedence-for-v1-bool-expr
- spec: `59 §5`
- given: the source `true and false`.
- expect: rejected by `parseBoolExpr`. The v1 grammar accepts `true`, `false`,
  `(not e)`, and `(and e1 e2)` only; it has no infix precedence table.
- why: AC6/AC7. This pins the ambiguity policy at the source: CAT-5's first
  grammar is prefix/parenthesized, not a hidden Ken-like expression parser.

### stdlib/parsing/not-compiler-ast-public-api (soundness)
- spec: `59 §1` / `59 §4` / `59 §5`; `39 §5.2`
- given: the CAT-5 package public API.
- expect: public syntax values are package-owned `BoolExpr` / `Syntax BoolExpr`
  values. No public constructor or law exposes the compiler surface AST
  constructors from `39 §5.2`, and no package function claims to parse all Ken
  source.
- why: CAT-5 is a catalog package, not a compiler parser rewrite. Exposing the
  compiler AST would make an untrusted implementation detail part of the
  public catalog API.

---

## AC9 -- diagnostic span validity

### stdlib/parsing/diagnostic-primary-and-secondary-spans-valid (fidelity)
- spec: `59 §6`
- given: a diagnostic with one valid primary span and two valid secondary spans
  in the same source artifact.
- expect: `ValidDiagnostic s d` holds. The source id on the diagnostic matches
  `sourceId s`, and every related span satisfies `ValidSpan s`.
- why: accept arm for CAT-5 diagnostics. A diagnostic is not merely a message;
  it is a source-located value whose locations are all checked.

### stdlib/parsing/diagnostic-out-of-range-primary-rejected (fidelity)
- spec: `59 §6`
- given: a diagnostic whose primary span has `end > sourceLength s`.
- expect: `ValidDiagnostic s d` is rejected.
- why: AC9 primary-span flip. A renderer might still print a message, but it is
  not a valid CAT-5 diagnostic.

### stdlib/parsing/diagnostic-out-of-range-secondary-rejected (fidelity)
- spec: `59 §6`
- given: a diagnostic whose primary span is valid but one secondary span has
  `end > sourceLength s`.
- expect: `ValidDiagnostic s d` is rejected. Dropping the secondary span to make
  validation pass does not preserve the diagnostic value.
- why: AC9 secondary-span flip. This catches implementations that check only
  the primary location or silently discard related locations.

---

## AC10 -- `.ken.md` source identity boundary

### stdlib/parsing/ken-md-derived-view-keeps-original-source-id (fidelity)
- spec: `59 §7`; `31 §1`
- given: an offset-preserving `.ken.md` compiled view produced by blanking prose
  while preserving byte length and newlines, and a parse error inside a compiled
  fence.
- expect: the parse error's source id is the original `.ken.md` artifact, and
  its byte span is the original artifact byte span. The extracted buffer is not
  the sole source identity.
- why: CAT-5 consumes the `.ken.md` seam without implementing it. A diagnostic
  pointing at the derived buffer alone is source-blind to the user's file.

### stdlib/parsing/non-offset-preserving-view-is-new-source
- spec: `59 §2` / `59 §7`
- given: a parser input formed by concatenating two snippets or extracting a
  fence without byte-preserving blanks.
- expect: the result is a new source artifact for CAT-5 v1. It may not report
  spans against the original parent artifact unless a future source-map
  mechanism explicitly supplies that map.
- why: v1 has byte spans over a single source artifact, not general source maps.
  This prevents string concatenation from becoming a hidden source identity.

---

## AC1 -- zero trust delta

### stdlib/parsing/kernel-untouched-zero-trust-delta (soundness)
- spec: `59 §1` / `59 §9`
- given: the CAT-5 build diff for `catalog/packages/parsing/`.
- expect: no `crates/ken-kernel`, compiler parser, primitive registry, or
  `Cargo.lock` trust-surface change is needed for the package contract; public
  laws are real proof terms with empty `trusted_base()` delta.
- why: CAT-5 is ordinary catalog Ken. A kernel touch or postulated parser law
  would turn a package abstraction into a hidden built-in.
