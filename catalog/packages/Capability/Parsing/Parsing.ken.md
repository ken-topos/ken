# `parsing` — source artifacts, spans, parsers, and a Boolean grammar

The source/span core, a total parser-result surface, and a fully
parenthesized Boolean-expression grammar built end to end on top of it. This
package models source identity as an immutable byte artifact: spans are only
half-open byte endpoints, and source identity is supplied by values such as
`Located` and by validity predicates, never by a bare `Span`.

## Index

1. [Motivation](#1-motivation)
2. [Definition](#2-definition)
3. [Using it](#3-using-it)
4. [Laws  proofs](#4-laws--proofs)
5. [Design notes](#5-design-notes)
6. [References](#6-references)
7. [Trust  derivation](#7-trust--derivation)

**Named reading paths**

- *Newcomer* → [Motivation](#1-motivation) → [Using it](#3-using-it)
- *Practitioner* → [Using it](#3-using-it) →
  [Laws  proofs](#4-laws--proofs)
- *Researcher* →
  [Laws  proofs](#4-laws--proofs) → [Design notes](#5-design-notes)

## 1. Motivation

Every parser needs vocabulary to state where in the source a value came from
and
whether a parse succeeded: a `SourceId` to disambiguate which source, `Bytes`
(not codepoints) as the offset basis for a `Span`, and a `ParseResult` that
is total — `Parsed` or `Failed`, never a partial function — with public
validity predicates a caller can check rather than trust. This package
states that vocabulary once, then exercises the whole stack end to end with
one concrete parser: a fully parenthesized Boolean-expression grammar.

## 2. Definition

`Source` is a checked record carrying artifact identity, the original
`Bytes`, UTF-8 evidence, a `Nat` byte length, a non-empty byte-atomic
length-unit witness for converting that `Nat` count to an `Int` offset, and
a proof that the converted recorded length is the byte length of the
source's own bytes. `String` is deliberately not the offset basis anywhere
in this package — every length and position is a `Bytes`/`Int` quantity,
sidestepping UTF-8 boundary bugs entirely; `IsUtf8` is a proof the bytes
*happen* to decode losslessly, not a requirement they must.

```ken
data SourceId = MkSourceId Nat

fn IsUtf8 (bs : Bytes) : Prop =
  Equal Bytes (bytes_encode (bytes_decode bs)) bs

fn byte_unit_zero_int (unit : Bytes) : Int =
  (bytes_length unit) - (bytes_length unit)

fn byte_unit_nat_to_int (unit : Bytes) (n : Nat) : Int =
  match n {
    Zero => byte_unit_zero_int unit ;
    Suc n2 => (bytes_length unit) + (byte_unit_nat_to_int unit n2)
  }

fn EmptyBytes (bs : Bytes) : Prop =
  Equal Bytes bs (bytes_slice bs (bytes_length bs) (byte_unit_zero_int bs))

fn NonEmptyBytes (bs : Bytes) : Prop =
  EmptyBytes bs -> Bottom

fn UnitByteLength (unit : Bytes) : Prop =
  And
    (NonEmptyBytes unit)
    ((left : Bytes) -> (right : Bytes) ->
      Equal Bytes unit (bytes_concat left right) ->
      NonEmptyBytes left ->
      EmptyBytes right)

fn SourceLength (unit : Bytes) (bs : Bytes) (n : Nat) : Prop =
  Equal Int (bytes_length bs) (byte_unit_nat_to_int unit n)

class Source {
  source_id_field : SourceId ;
  source_bytes_field : Bytes ;
  source_length_field : Nat ;
  source_length_unit_field : Bytes ;
  source_length_unit_valid_field : UnitByteLength source_length_unit_field ;
  source_utf8_field : IsUtf8 source_bytes_field ;
  source_length_valid_field : SourceLength source_length_unit_field source_bytes_field source_length_field
}

fn source_id (s : Source) : SourceId =
  s.source_id_field

fn source_bytes (s : Source) : Bytes =
  s.source_bytes_field

fn source_length (s : Source) : Nat =
  s.source_length_field

lemma source_utf8 (s : Source) : IsUtf8 (source_bytes s) =
  s.source_utf8_field

fn source_length_unit (s : Source) : Bytes =
  s.source_length_unit_field

lemma source_length_unit_valid (s : Source) : UnitByteLength (source_length_unit s) =
  s.source_length_unit_valid_field

lemma source_length_valid (s : Source)
  : SourceLength (source_length_unit s) (source_bytes s) (source_length s) =
  s.source_length_valid_field
```

## 3. Using it

A caller builds a `Source` once per artifact (supplying its own
`source_length_unit`, `source_length_unit_valid`, `source_utf8`, and
`source_length_valid` evidence — this package states the record shape, not a
constructor helper, so every field is honest at the call site), then drives
`§4.3`'s `parse_bool_expr : Parser (Syntax BoolExpr)` over it. `format_bool_expr`
is the single-call entry point: it parses a `Source` end to end and, on
success, prints the erased (span-free) tree back out — the worked round
trip this package ships as its concrete example, in `§4.3` rather than a
separate illustrative fence, since it is real, checked package content, not
a demo.

## 4. Laws  proofs

### 4.1 Span validity and the zero-width-span proof

`Span` carries only half-open byte endpoints — a bare `Span` never
identifies a source artifact by itself. `ValidSpan s sp` requires
`span_start sp <= span_end sp <= source_length s`, stated through
`LessEqNat`, the same `Bool`-bridged pattern the lawful-classes packages use
(`Equal Bool (nat_leq_bool m n) True`), here without a named `IsTrue` alias
since this package has no `Eq`/`Ord`-style class to hang one on.
`less_eq_nat_refl` is a genuine proof by induction on `n`; `less_eq_nat_zero_left`
is definitional (`nat_leq_bool Zero n` reduces to `True` on its very first
match arm, for any `n`). `valid_zero_width_span` is the one composite proof
in this package: given a valid offset (`LessEqNat offset (source_length s)`),
a zero-width span at that offset is valid, by pairing `less_eq_nat_refl`
(the span's own `start <= end`, both `offset`) with the supplied hypothesis
(`end <= source_length s`) via `and_intro`.

```ken
data Span = MkSpan Nat Nat

fn span_start (sp : Span) : Nat =
  match sp { MkSpan start end => start }

fn span_end (sp : Span) : Nat =
  match sp { MkSpan start end => end }

fn nat_leq_bool (m : Nat) (n : Nat) : Bool =
  match m {
    Zero => True ;
    Suc m2 => match n { Zero => False ; Suc n2 => nat_leq_bool m2 n2 }
  }

fn LessEqNat (m : Nat) (n : Nat) : Prop =
  Equal Bool (nat_leq_bool m n) True

lemma less_eq_nat_refl (n : Nat) : LessEqNat n n =
  match n { Zero => Proved ; Suc n2 => less_eq_nat_refl n2 }

lemma less_eq_nat_zero_left (n : Nat) : LessEqNat Zero n =
  Proved

fn ValidSpan (s : Source) (sp : Span) : Prop =
  And
    (LessEqNat (span_start sp) (span_end sp))
    (LessEqNat (span_end sp) (source_length s))

lemma valid_zero_width_span (s : Source) (offset : Nat)
  : LessEqNat offset (source_length s) -> ValidSpan s (MkSpan offset offset) =
  \h.
    and_intro
      (LessEqNat offset offset)
      (LessEqNat offset (source_length s))
      (less_eq_nat_refl offset)
      h
```

### 4.2 Located values, parse errors, and the total `Parser` contract

`Located a` pairs a value with a `SourceId` and `Span`; `ValidLocated`
checks both the source id and span against a concrete `Source`. `ParseError`
carries just enough to be checkable the same way — a `SourceId` and `Span`
of its own. `Parser a` is total *by construction*: it always returns a
`ParseResult a` (`Parsed` or `Failed`), never diverges or partial-applies,
conditional on the caller supplying a proof the start position is in bounds
(`LessEqNat start (source_length s)`). `ParserValid`/`ParserTotal`/
`ParserSourceLocal` are the three public well-formedness *properties* a
`Parser` should satisfy — plain predicates over a `Parser a`, not enforced
by the `Parser` type itself, so a caller can state and check them per
concrete parser. `parser_pure` and `parser_fail` are the two base combinators:
the former always succeeds on a zero-width span at `start`, the latter
always fails at `start` with a zero-width error span.

```ken
data Located a = MkLocated SourceId Span a

fn located_source (a : Type) (x : Located a) : SourceId =
  match x { MkLocated sid sp value => sid }

fn located_span (a : Type) (x : Located a) : Span =
  match x { MkLocated sid sp value => sp }

fn located_value (a : Type) (x : Located a) : a =
  match x { MkLocated sid sp value => value }

fn ValidLocated (a : Type) (s : Source) (x : Located a) : Prop =
  And
    (Equal SourceId (located_source a x) (source_id s))
    (ValidSpan s (located_span a x))

data ParseError = MkParseError SourceId Span

fn error_source (err : ParseError) : SourceId =
  match err { MkParseError sid sp => sid }

fn error_span (err : ParseError) : Span =
  match err { MkParseError sid sp => sp }

data ParseResult a =
  Parsed a Span Nat |
  Failed ParseError

const Parser (a : Type) : Type =
  (s : Source) -> (start : Nat) -> LessEqNat start (source_length s) -> ParseResult a

fn ParsedValid (s : Source) (start : Nat) (consumed : Span) (next : Nat) : Prop =
  And
    (ValidSpan s consumed)
    (And
      (Equal Nat (span_start consumed) start)
      (Equal Nat (span_end consumed) next))

fn FailedValid (s : Source) (err : ParseError) : Prop =
  And
    (Equal SourceId (error_source err) (source_id s))
    (ValidSpan s (error_span err))

fn ParseResultValid (a : Type) (s : Source) (start : Nat) (r : ParseResult a) : Prop =
  match r {
    Parsed value consumed next => ParsedValid s start consumed next ;
    Failed err => FailedValid s err
  }

fn ParserValid (a : Type) (p : Parser a) : Prop =
  (s : Source) -> (start : Nat) -> (h : LessEqNat start (source_length s)) ->
    ParseResultValid a s start (p s start h)

fn ParseResultTotal (a : Type) (r : ParseResult a) : Prop =
  match r {
    Parsed value consumed next => Top ;
    Failed err => Top
  }

fn ParserTotal (a : Type) (p : Parser a) : Prop =
  (s : Source) -> (start : Nat) -> (h : LessEqNat start (source_length s)) ->
    ParseResultTotal a (p s start h)

fn ParseResultSourceLocal (a : Type) (s : Source) (r : ParseResult a) : Prop =
  match r {
    Parsed value consumed next => ValidSpan s consumed ;
    Failed err => Equal SourceId (error_source err) (source_id s)
  }

fn ParserSourceLocal (a : Type) (p : Parser a) : Prop =
  (s : Source) -> (start : Nat) -> (h : LessEqNat start (source_length s)) ->
    ParseResultSourceLocal a s (p s start h)

fn ParserLaws (a : Type) (p : Parser a) : Prop =
  And
    (ParserValid a p)
    (And (ParserTotal a p) (ParserSourceLocal a p))

fn parser_pure (a : Type) (value : a) : Parser a =
  \s. \start. \h. Parsed a value (MkSpan start start) start

const parser_fail (a : Type) : Parser a =
  \s. \start. \h. Failed a (MkParseError (source_id s) (MkSpan start start))
```

### 4.3 A worked grammar: parenthesized Boolean expressions

`BoolExpr` is fully parenthesized: `true`, `false`, `(not e)`, and
`(and e1 e2)`. There is no precedence table — `true and false` rejects,
deliberately; a real expression grammar with precedence is out of scope for
this worked example. `Syntax a` pairs a `Located a` root with a `List` of
`Located a` children, giving every parsed node its own span independent of
its value's own recursive structure; `erase_spans` recovers the bare
`BoolExpr` by walking back down to the root value.

Token recognition is byte-by-byte through `source_byte_eq_at`, matching
literal ASCII codepoints spelled as `Int` literals (`116` is `t`, `102` is
`f`, `40` is `(`, `32` is space, and so on) — this package has no `Char`
literal syntax of its own, so every fixed token is matched digit by digit
against `bytes_at`. `parse_bool_expr_at_fuel` and `skip_spaces_fuel` are both
fuel-bounded recursive descent: each takes an explicit `Nat` fuel parameter
decremented on every recursive step and seeded from `source_length s` (an
upper bound on how many bytes remain to consume), the standard
structural-recursion-via-fuel pattern for a function whose real termination
measure — unconsumed input — is not syntactically a subterm of the fuel
itself. `list_append` here is a second, verbatim copy local to this
package, not a re-export of `catalog/packages/Data/Collections/Collections.ken`'s
combinator of the same name — a self-containment choice
(`07-catalog-style-guide.md §13`) appropriate for a leaf capability package
that otherwise takes no catalog dependency.

```ken
data BoolExpr =
  BTrue |
  BFalse |
  BNot BoolExpr |
  BAnd BoolExpr BoolExpr

data Syntax a = MkSyntax (Located a) (List (Located a))

fn syntax_root (a : Type) (x : Syntax a) : Located a =
  match x { MkSyntax root children => root }

fn syntax_children (a : Type) (x : Syntax a) : List (Located a) =
  match x { MkSyntax root children => children }

fn erase_spans (x : Syntax BoolExpr) : BoolExpr =
  located_value BoolExpr (syntax_root BoolExpr x)

fn list_append (a : Type) (xs : List a) (ys : List a) : List a =
  match xs {
    Nil => ys ;
    Cons x rest => Cons a x (list_append a rest ys)
  }

fn ValidLocatedList (a : Type) (s : Source) (xs : List (Located a)) : Prop =
  match xs {
    Nil => Top ;
    Cons x rest =>
      And
        (ValidLocated a s x)
        (ValidLocatedList a s rest)
  }

fn ValidSyntax (a : Type) (s : Source) (x : Syntax a) : Prop =
  And
    (ValidLocated a s (syntax_root a x))
    (ValidLocatedList a s (syntax_children a x))

fn bool_expr_eq (x : BoolExpr) (y : BoolExpr) : Bool =
  match x {
    BTrue => match y { BTrue => True ; BFalse => False ; BNot y1 => False ; BAnd yl yr => False } ;
    BFalse => match y { BTrue => False ; BFalse => True ; BNot y1 => False ; BAnd yl yr => False } ;
    BNot x1 => match y {
      BTrue => False ;
      BFalse => False ;
      BNot y1 => bool_expr_eq x1 y1 ;
      BAnd yl yr => False
    } ;
    BAnd xl xr => match y {
      BTrue => False ;
      BFalse => False ;
      BNot y1 => False ;
      BAnd yl yr => match bool_expr_eq xl yl {
        True => bool_expr_eq xr yr ;
        False => False
      }
    }
  }

fn nat_eq_bool (m : Nat) (n : Nat) : Bool =
  match m {
    Zero => match n { Zero => True ; Suc n2 => False } ;
    Suc m2 => match n { Zero => False ; Suc n2 => nat_eq_bool m2 n2 }
  }

fn nat_add (m : Nat) (n : Nat) : Nat =
  match n { Zero => m ; Suc n2 => Suc (nat_add m n2) }

fn nat_lt_bool (m : Nat) (n : Nat) : Bool =
  match n { Zero => False ; Suc n2 => nat_leq_bool m n2 }

fn bool_and (p : Bool) (q : Bool) : Bool =
  match p { True => q ; False => False }

fn source_byte_eq (s : Source) (pos : Nat) (code : Int) : Bool =
  match nat_lt_bool pos (source_length s) {
    True => eq_int (bytes_at (source_bytes s) (byte_unit_nat_to_int (source_length_unit s) pos)) code ;
    False => False
  }

fn source_byte_eq_at (s : Source) (start : Nat) (offset : Nat) (code : Int) : Bool =
  source_byte_eq s (nat_add start offset) code

fn starts_true_token (s : Source) (start : Nat) : Bool =
  bool_and
    (source_byte_eq_at s start Zero (116 : Int))
    (bool_and
      (source_byte_eq_at s start (Suc Zero) (114 : Int))
      (bool_and
        (source_byte_eq_at s start (Suc (Suc Zero)) (117 : Int))
        (source_byte_eq_at s start (Suc (Suc (Suc Zero))) (101 : Int))))

fn starts_false_token (s : Source) (start : Nat) : Bool =
  bool_and
    (source_byte_eq_at s start Zero (102 : Int))
    (bool_and
      (source_byte_eq_at s start (Suc Zero) (97 : Int))
      (bool_and
        (source_byte_eq_at s start (Suc (Suc Zero)) (108 : Int))
        (bool_and
          (source_byte_eq_at s start (Suc (Suc (Suc Zero))) (115 : Int))
          (source_byte_eq_at s start (Suc (Suc (Suc (Suc Zero)))) (101 : Int)))))

fn starts_not_open_token (s : Source) (start : Nat) : Bool =
  bool_and
    (source_byte_eq_at s start Zero (40 : Int))
    (bool_and
      (source_byte_eq_at s start (Suc Zero) (110 : Int))
      (bool_and
        (source_byte_eq_at s start (Suc (Suc Zero)) (111 : Int))
        (bool_and
          (source_byte_eq_at s start (Suc (Suc (Suc Zero))) (116 : Int))
          (source_byte_eq_at s start (Suc (Suc (Suc (Suc Zero)))) (32 : Int)))))

fn starts_and_open_token (s : Source) (start : Nat) : Bool =
  bool_and
    (source_byte_eq_at s start Zero (40 : Int))
    (bool_and
      (source_byte_eq_at s start (Suc Zero) (97 : Int))
      (bool_and
        (source_byte_eq_at s start (Suc (Suc Zero)) (110 : Int))
        (bool_and
          (source_byte_eq_at s start (Suc (Suc (Suc Zero))) (100 : Int))
          (source_byte_eq_at s start (Suc (Suc (Suc (Suc Zero)))) (32 : Int)))))

fn skip_spaces_fuel (fuel : Nat) (s : Source) (pos : Nat) : Nat =
  match fuel {
    Zero => pos ;
    Suc fuel2 => match source_byte_eq s pos (32 : Int) {
      True => skip_spaces_fuel fuel2 s (Suc pos) ;
      False => pos
    }
  }

fn skip_spaces (s : Source) (pos : Nat) : Nat =
  skip_spaces_fuel (source_length s) s pos

fn syntax_leaf (s : Source) (start : Nat) (end : Nat) (value : BoolExpr) : Syntax BoolExpr =
  MkSyntax BoolExpr
    (MkLocated BoolExpr (source_id s) (MkSpan start end) value)
    (Nil (Located BoolExpr))

fn syntax_node_unary
  (s : Source) (start : Nat) (end : Nat) (value : BoolExpr) (child : Syntax BoolExpr)
  : Syntax BoolExpr =
  MkSyntax BoolExpr
    (MkLocated BoolExpr (source_id s) (MkSpan start end) value)
    (Cons (Located BoolExpr)
      (syntax_root BoolExpr child)
      (syntax_children BoolExpr child))

fn syntax_node_binary
  (s : Source) (start : Nat) (end : Nat) (value : BoolExpr)
  (left : Syntax BoolExpr) (right : Syntax BoolExpr)
  : Syntax BoolExpr =
  MkSyntax BoolExpr
    (MkLocated BoolExpr (source_id s) (MkSpan start end) value)
    (list_append
      (Located BoolExpr)
      (Cons (Located BoolExpr)
        (syntax_root BoolExpr left)
        (syntax_children BoolExpr left))
      (Cons (Located BoolExpr)
        (syntax_root BoolExpr right)
        (syntax_children BoolExpr right)))

fn parse_bool_expr_at_fuel (fuel : Nat) (s : Source) (start : Nat) : ParseResult (Syntax BoolExpr) =
  match fuel {
    Zero => Failed (Syntax BoolExpr) (MkParseError (source_id s) (MkSpan start start)) ;
    Suc fuel2 => match starts_true_token s start {
      True =>
        Parsed (Syntax BoolExpr)
          (syntax_leaf s start (nat_add start (Suc (Suc (Suc (Suc Zero))))) BTrue)
          (MkSpan start (nat_add start (Suc (Suc (Suc (Suc Zero))))))
          (nat_add start (Suc (Suc (Suc (Suc Zero))))) ;
      False => match starts_false_token s start {
        True =>
          Parsed (Syntax BoolExpr)
            (syntax_leaf s start (nat_add start (Suc (Suc (Suc (Suc (Suc Zero)))))) BFalse)
            (MkSpan start (nat_add start (Suc (Suc (Suc (Suc (Suc Zero)))))))
            (nat_add start (Suc (Suc (Suc (Suc (Suc Zero)))))) ;
        False => match starts_not_open_token s start {
          True =>
            match parse_bool_expr_at_fuel fuel2 s (skip_spaces s (nat_add start (Suc (Suc (Suc (Suc (Suc Zero))))))) {
              Parsed child childSpan childNext => match source_byte_eq s (skip_spaces s childNext) (41 : Int) {
                True =>
                  Parsed (Syntax BoolExpr)
                    (syntax_node_unary
                      s
                      start
                      (Suc (skip_spaces s childNext))
                      (BNot (erase_spans child))
                      child)
                    (MkSpan start (Suc (skip_spaces s childNext)))
                    (Suc (skip_spaces s childNext)) ;
                False => Failed (Syntax BoolExpr) (MkParseError (source_id s) (MkSpan (skip_spaces s childNext) (skip_spaces s childNext)))
              } ;
              Failed err => Failed (Syntax BoolExpr) err
            } ;
          False => match starts_and_open_token s start {
            True =>
              match parse_bool_expr_at_fuel fuel2 s (skip_spaces s (nat_add start (Suc (Suc (Suc (Suc (Suc Zero))))))) {
                Parsed left leftSpan leftNext => match source_byte_eq s leftNext (32 : Int) {
                  True =>
                    match parse_bool_expr_at_fuel fuel2 s (skip_spaces s (Suc leftNext)) {
                      Parsed right rightSpan rightNext => match source_byte_eq s (skip_spaces s rightNext) (41 : Int) {
                        True =>
                          Parsed (Syntax BoolExpr)
                            (syntax_node_binary
                              s
                              start
                              (Suc (skip_spaces s rightNext))
                              (BAnd (erase_spans left) (erase_spans right))
                              left
                              right)
                            (MkSpan start (Suc (skip_spaces s rightNext)))
                            (Suc (skip_spaces s rightNext)) ;
                        False => Failed (Syntax BoolExpr) (MkParseError (source_id s) (MkSpan (skip_spaces s rightNext) (skip_spaces s rightNext)))
                      } ;
                      Failed err => Failed (Syntax BoolExpr) err
                    } ;
                  False => Failed (Syntax BoolExpr) (MkParseError (source_id s) (MkSpan leftNext leftNext))
                } ;
                Failed err => Failed (Syntax BoolExpr) err
              } ;
            False => Failed (Syntax BoolExpr) (MkParseError (source_id s) (MkSpan start start))
          }
        }
      }
    }
  }

const parse_bool_expr : Parser (Syntax BoolExpr) =
  \s. \start. \h.
    match parse_bool_expr_at_fuel (source_length s) s (skip_spaces s start) {
      Parsed syntax consumed next => match nat_eq_bool (skip_spaces s next) (source_length s) {
        True => Parsed (Syntax BoolExpr) syntax (MkSpan start (skip_spaces s next)) (skip_spaces s next) ;
        False => Failed (Syntax BoolExpr) (MkParseError (source_id s) (MkSpan (skip_spaces s next) (skip_spaces s next)))
      } ;
      Failed err => Failed (Syntax BoolExpr) err
    }

fn print_bool_expr (e : BoolExpr) : Bytes =
  match e {
    BTrue => bytes_encode "true" ;
    BFalse => bytes_encode "false" ;
    BNot child =>
      bytes_concat
        (bytes_concat (bytes_encode "(not ") (print_bool_expr child))
        (bytes_encode ")") ;
    BAnd left right =>
      bytes_concat
        (bytes_concat
          (bytes_concat (bytes_encode "(and ") (print_bool_expr left))
          (bytes_encode " "))
        (bytes_concat (print_bool_expr right) (bytes_encode ")"))
  }

fn format_bool_expr (s : Source) : Result ParseError Bytes =
  match parse_bool_expr s Zero (less_eq_nat_zero_left (source_length s)) {
    Parsed syntax consumed next => Ok ParseError Bytes (print_bool_expr (erase_spans syntax)) ;
    Failed err => Err ParseError Bytes err
  }
```

## 5. Design notes

**Why `Bytes`, never `String`, as the offset basis.** Every position and
length in this package is a `Bytes`/`Int` quantity; `IsUtf8` is carried as
evidence the bytes happen to decode losslessly, never as a precondition
positions are computed against. A codepoint-indexed offset would need a
decode step (and a failure mode) just to compute `span_end - span_start`;
a byte-indexed offset never does.

**`ParseResultTotal`/`ParserTotal` are honestly weak.** Both match arms of
`ParseResultTotal` reduce to the same `Top` — the only
content this witnesses is that the `Parsed`/`Failed` case-split is
exhaustive, not any deeper property of a particular parser. A reader should
not over-read "Total" here as more than "this function always returns one
of its two constructors," which the type checker already guarantees; see
`§6`.

**Why unguarded repetition is not exported.** An unguarded `many`-style
combinator could loop forever on an inner parser that succeeds without
consuming input; a fuel-bounded one would need its own checked
progress/next-validity surface to be trustworthy. Neither is exported by
this package — `§4.3`'s `parse_bool_expr_at_fuel`/`skip_spaces_fuel` are
package-internal, fuel-bounded recursions over a syntactically-decreasing
`Nat`, not a general combinator a caller could misuse.

**Why the Boolean grammar has no precedence table.** `true`, `false`,
`(not e)`, and `(and e1 e2)` are fully parenthesized on purpose — `true and
false` rejects, deliberately, keeping this worked example small. A real
expression grammar with precedence climbing is future package work, not
attempted here.

## 6. References

None — this entry's design is Ken-native, not consulted from an external
reference implementation.

## 7. Trust  derivation

1. **Public API.** `SourceId`, `Source`, `IsUtf8`, `EmptyBytes`,
   `NonEmptyBytes`, `UnitByteLength`, `SourceLength`, `source_id`,
   `source_bytes`, `source_utf8`, `source_length`, `source_length_unit`,
   `source_length_unit_valid`, `source_length_valid`, `Span`, `span_start`,
   `span_end`, `LessEqNat`, `ValidSpan`, `Located`, `located_source`,
   `located_span`, `located_value`, `ValidLocated`,
   `valid_zero_width_span`, `ParseError`, `error_source`, `error_span`,
   `ParseResult`, `Parser`, `ParsedValid`, `FailedValid`,
   `ParseResultValid`, `ParserValid`, `ParserTotal`, `ParserSourceLocal`,
   `ParserLaws`, `parser_pure`, `parser_fail`, `BoolExpr`, `Syntax`,
   `syntax_root`, `syntax_children`, `erase_spans`, `ValidLocatedList`,
   `ValidSyntax`, `parse_bool_expr`, `print_bool_expr`, `format_bool_expr`.
2. **Source map.**

   | Task | Section |
   |---|---|
   | See the source/span/parser vocabulary | [Definition](#2-definition) |
   | Build a `Source`, drive the grammar | [Using it](#3-using-it) |
   | The zero-width-span proof, the total `Parser` contract, the worked grammar | [Laws  proofs](#4-laws--proofs) |
   | Why `Bytes` not `String`, why no unguarded repetition | [Design notes](#5-design-notes) |

3. **Derivation path.** The package surface is ordinary Ken data, a
   class-backed record, transparent functions, and proof-returning
   definitions over `Nat`, `Bool`, `Bytes`, `Equal`, `And`, `List`, and
   parser-result data. It adds no kernel primitive, no source-loader
   behavior, and no language-semantics change.
4. **`trusted_base()` delta.** **Zero.** Every proof in this package —
   `less_eq_nat_refl`, `less_eq_nat_zero_left`, `valid_zero_width_span` — is
   real and kernel-checked; no law or predicate is postulated.
5. **Proof families.** `less_eq_nat_refl` — induction on `n`.
   `less_eq_nat_zero_left` — definitional (first match arm). `valid_zero_width_span`
   — direct composition of the two via `and_intro`, no case-split of its
   own.
6. **Consumers.** Source-aware parser implementations can use this package's
   source, span, result, and validity vocabulary.
7. **Validation evidence.** The catalog checks the
   `Source`/`Span`/`Located`/`ParseResult`/`Parser` surface, its zero
   `trusted_base()` delta, the Boolean grammar's constructors and byte-token
   matching, and the absence of an exported unguarded repetition combinator.
