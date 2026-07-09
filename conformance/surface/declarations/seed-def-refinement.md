# SURF-def-refinement conformance — seed cases

Format: `../../README.md`. These pin the `SURF-def-refinement` slice: the
`type` → `def` declaration-keyword rename (`spec/30-surface/33-declarations.md
§1`, `32-grammar.md §1`, `31-lexical.md §4`). Semantics are unchanged — a
`def` elaborates to exactly what `type` did (zero kernel delta); only the
surface keyword and `type`'s reserved-word status are pinned here.

## surface/declarations/def-refinement-parses
- spec: `spec/30-surface/32-grammar.md §1`, `spec/30-surface/33-declarations.md
  §1`
- given:
  ```ken
  def Pos = { n : Int | n > 0 }
  ```
- expect: accepts — parses and elaborates to the carrier `Int` plus the
  tracked obligation `n > 0` at each introduction (`34 §5`), identically to
  the pre-rename `type Pos = { n : Int | n > 0 }`.
- why: the refinement-definition case of the renamed keyword terminal (frame
  §4.2/§4.3).

## surface/declarations/def-alias-parses
- spec: `spec/30-surface/32-grammar.md §1`, `spec/30-surface/33-declarations.md
  §1`
- given:
  ```ken
  data DecimalPair = MkDecimalPair Int Int
  def Decimal = DecimalPair
  ```
- expect: accepts — `Decimal` elaborates as a transparent alias for
  `DecimalPair` (unfolds by δ), the zero-condition case of a definition.
- why: the plain-alias case of the renamed keyword terminal; pins that `def`
  covers both the refinement and alias RHS shapes with one production.

## surface/declarations/type-keyword-rejected
- spec: `spec/30-surface/31-lexical.md §4`, `32-grammar.md §1`
- given:
  ```ken
  type Foo = Int
  ```
- expect: rejects(parse error) — `type` is **reserved**, not a declaration
  keyword; the parser reports that `type` is reserved and steers to `def`.
- why: the discriminating negative (frame §4.2 AC) — confirms the old
  spelling no longer parses as a declaration, distinguishing this from a
  same-behavior no-op rename.

## surface/declarations/type-not-a-free-identifier
- spec: `spec/30-surface/31-lexical.md §4`
- given:
  ```ken
  fn type (x : Int) : Int = x
  ```
- expect: rejects(parse error) — `type` still lexes as a reserved keyword
  token, so it cannot be used as a parameter/definition name either.
- why: pins that `type` is fully reserved (not merely removed from decl
  position) — it must not fall back to a free identifier once no longer a
  declaration keyword (frame §2 pinned input).

## surface/declarations/def-value-position-diagnostic
- spec: `spec/30-surface/33-declarations.md §1`, frame §4.6 (should-have)
- given:
  ```ken
  def double x = x * 2
  ```
- expect: rejects(error) — a lowercase head after `def` yields the steering
  diagnostic ("'def' defines a type … use 'fn' … or 'const' …"), not a bare
  parse error.
- why: pins the should-have diagnostic (frame AC §4.6) that helps a
  `const`/`fn` author who reaches for `def` by analogy with value definitions.
