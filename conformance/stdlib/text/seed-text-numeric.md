# Text.Numeric conformance seed

Format: `../../README.md`. These cases pin the black-box semantics of the CC2
decimal parser. Locations are Unicode-scalar (`List Char`) indexes, not byte
offsets. `Int` is arbitrary-precision, so none of these cases admits an
overflow failure.

## AC7 — decimal parsing and located failures

### text/numeric/valid-decimal-parse

- given: the text `"123"`.
- expect: `parse_nat "123" = Ok 123`.
- why: pins left-to-right base-ten accumulation and rejects a parser that
  merely validates the characters without accumulating their value.

### text/numeric/empty-input-located-at-zero

- given: the empty text `""`.
- expect: `parse_nat "" = Err (MkNumericError EmptyInput 0)`.
- why: distinguishes empty input from an invalid digit and requires the
  minimal located carrier even when no character was consumed.

### text/numeric/invalid-digit-exact-char-index

- given: the text `"12x4"`.
- expect: `parse_nat "12x4" = Err (MkNumericError InvalidDigit 2)`.
- why: the exact index defeats a location-free implementation and one-based
  variants. Parsing stops at the first invalid character.

### text/numeric/signed-decimal-parse

- given: the text `"-42"`.
- expect: `parse_int "-42" = Ok (-42)`.
- why: pins the optional leading minus and confirms that its character counts
  toward later error locations.

## Trust and scope

The implementation is ordinary catalog Ken. It adds no primitive, postulate,
kernel rule, prelude entry, or Cargo dependency. The seed does not authorize a
universal `String` round-trip law or lawful String/Bytes key instances.
