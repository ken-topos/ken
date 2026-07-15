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

`SourceId` comes from the lower `Capability.Diagnostics.Core` package. `Source` is a
checked record carrying artifact identity, the original `Bytes`, and UTF-8
evidence. Lengths and positions are structural `Nat` quantities computed from
the total `List UInt8` view. `String` is deliberately not the offset basis;
`IsUtf8` is a proof the bytes *happen* to decode losslessly, not a requirement
they must.

```ken
fn IsUtf8 (bs : Bytes) : Prop =
  match bytes_decode bs {
    Err _ ↦ Bottom;
    Ok text ↦ Equal Bytes (bytes_encode text) bs
  }

class Source {
  source_id_field : SourceId;
  source_bytes_field : Bytes;
  source_utf8_field : IsUtf8 source_bytes_field
}

fn source_id (s : Source) : SourceId = s.source_id_field

fn source_bytes (s : Source) : Bytes = s.source_bytes_field

fn source_length (s : Source) : Nat = bytes_nat_length s.source_bytes_field

proof utf8 for source_bytes (s : Source) : IsUtf8 (source_bytes s) = s.source_utf8_field
```

## 3. Using it

A caller builds a `Source` once per artifact (supplying its artifact identity,
bytes, and `source_bytes::utf8` evidence; length is computed), then drives
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
`LessEqNat::refl` is a genuine proof by induction on `n`; `LessEqNat::zero_left`
is definitional (`nat_leq_bool Zero n` reduces to `True` on its very first
match arm, for any `n`). `valid_zero_width_span` is the one composite proof
in this package: given a valid offset (`LessEqNat offset (source_length s)`),
a zero-width span at that offset is valid, by pairing `LessEqNat::refl`
(the span's own `start <= end`, both `offset`) with the supplied hypothesis
(`end <= source_length s`) via `and_intro`.

```ken
data Span = MkSpan Nat Nat

fn span_start (sp : Span) : Nat =
  match sp {
    MkSpan start end ↦ start
  }

fn span_end (sp : Span) : Nat =
  match sp {
    MkSpan start end ↦ end
  }

fn span_to_byte_range (sp : Span) : ByteRange = MkByteRange (span_start sp) (span_end sp)

fn span_origin (source : SourceId) (sp : Span) : Origin =
  SourceOrigin source (span_to_byte_range sp)

lemma span_to_byte_range_faithful
      (sp : Span)
    : And
        (Equal Nat (byte_range_start (span_to_byte_range sp)) (span_start sp))
        (Equal Nat (byte_range_end (span_to_byte_range sp)) (span_end sp)) =
  match sp {
    MkSpan start end ↦ and_intro (Equal Nat start start) (Equal Nat end end) Refl Refl
  }

lemma span_origin_source_faithful
      (source : SourceId) (sp : Span)
    : Equal
        (Option SourceId)
        (origin_source_id (span_origin source sp))
        (Some SourceId source) =
  match sp {
    MkSpan start end ↦ Refl
  }

data ByteCursor = MkByteCursor Source Nat

fn byte_cursor_source (cur : ByteCursor) : Source =
  match cur {
    MkByteCursor source position ↦ source
  }

fn byte_cursor_position (cur : ByteCursor) : Nat =
  match cur {
    MkByteCursor source position ↦ position
  }

fn byte_cursor_remaining (cur : ByteCursor) : Nat =
  cursor_nat_sub (source_length (byte_cursor_source cur)) (byte_cursor_position cur)

fn byte_cursor_peek (cur : ByteCursor) : Option UInt8 =
  nth UInt8 (byte_cursor_position cur) (bytes_to_list (source_bytes (byte_cursor_source cur)))

fn byte_cursor_advance (cur : ByteCursor) : ByteCursor =
  MkByteCursor (byte_cursor_source cur) (Suc (byte_cursor_position cur))

fn byte_cursor_locate (cur : ByteCursor) : Span =
  MkSpan (byte_cursor_position cur) (byte_cursor_position cur)

const byte_cursor_ops : CursorOps ByteCursor UInt8 Span =
  MkCursorOps
    ByteCursor
    UInt8
    Span
    byte_cursor_remaining
    byte_cursor_peek
    byte_cursor_advance
    byte_cursor_locate

fn nat_leq_bool (m : Nat) (n : Nat) : Bool =
  match m {
    Zero ↦ True;
    Suc m2 ↦
      match n {
        Zero ↦ False;
        Suc n2 ↦ nat_leq_bool m2 n2
      }
  }

fn LessEqNat (m : Nat) (n : Nat) : Prop = Equal Bool (nat_leq_bool m n) True

proof refl for LessEqNat (n : Nat) : LessEqNat n n =
  match n {
    Zero ↦ Proved;
    Suc n2 ↦ proof refl for LessEqNat n2
  }

proof zero_left for LessEqNat (n : Nat) : LessEqNat Zero n = Proved

fn ValidSpan (s : Source) (sp : Span) : Prop =
  And (LessEqNat (span_start sp) (span_end sp)) (LessEqNat (span_end sp) (source_length s))

lemma valid_zero_width_span
      (s : Source) (offset : Nat)
    : LessEqNat offset (source_length s) → ValidSpan s (MkSpan offset offset) =
  λh.
    and_intro
      (LessEqNat offset offset)
      (LessEqNat offset (source_length s))
      ((proof refl for LessEqNat) offset)
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
  match x {
    MkLocated sid sp value ↦ sid
  }

fn located_span (a : Type) (x : Located a) : Span =
  match x {
    MkLocated sid sp value ↦ sp
  }

fn located_value (a : Type) (x : Located a) : a =
  match x {
    MkLocated sid sp value ↦ value
  }

fn ValidLocated (a : Type) (s : Source) (x : Located a) : Prop =
  And (Equal SourceId (located_source a x) (source_id s)) (ValidSpan s (located_span a x))

data ParseError = MkParseError SourceId Span

fn error_source (err : ParseError) : SourceId =
  match err {
    MkParseError sid sp ↦ sid
  }

fn error_span (err : ParseError) : Span =
  match err {
    MkParseError sid sp ↦ sp
  }

data ParseResult a = Parsed a Span Nat | Failed ParseError

const Parser (a : Type) : Type =
  (s : Source) → (start : Nat) → LessEqNat start (source_length s) → ParseResult a

fn decoder_parse_error (s : Source) (err : DecoderError Span) : ParseError =
  MkParseError (source_id s) (decoder_error_location Span err)

fn parser_from_decoder (a : Type) (decoder : Decoder ByteCursor Span a) : Parser a =
  λs.
    λstart.
      λh.
        match decoder (MkByteCursor s start) {
          Decoded value next ↦
            Parsed
              a
              value
              (MkSpan start (byte_cursor_position next))
              (byte_cursor_position next);
          DecoderFailed err ↦ Failed a (decoder_parse_error s err)
        }

fn ParsedValid (s : Source) (start : Nat) (consumed : Span) (next : Nat) : Prop =
  And
    (ValidSpan s consumed)
    (And (Equal Nat (span_start consumed) start) (Equal Nat (span_end consumed) next))

fn FailedValid (s : Source) (err : ParseError) : Prop =
  And (Equal SourceId (error_source err) (source_id s)) (ValidSpan s (error_span err))

fn ParseResultValid (a : Type) (s : Source) (start : Nat) (r : ParseResult a) : Prop =
  match r {
    Parsed value consumed next ↦ ParsedValid s start consumed next;
    Failed err ↦ FailedValid s err
  }

fn ParserValid (a : Type) (p : Parser a) : Prop =
  (s : Source)
    → (start : Nat)
    → (h : LessEqNat start (source_length s))
    → ParseResultValid a s start
    (p s start h)

fn ParseResultTotal (a : Type) (r : ParseResult a) : Prop =
  match r {
    Parsed value consumed next ↦ Top;
    Failed err ↦ Top
  }

fn ParserTotal (a : Type) (p : Parser a) : Prop =
  (s : Source)
    → (start : Nat)
    → (h : LessEqNat start (source_length s))
    → ParseResultTotal a
    (p s start h)

fn ParseResultSourceLocal (a : Type) (s : Source) (r : ParseResult a) : Prop =
  match r {
    Parsed value consumed next ↦ ValidSpan s consumed;
    Failed err ↦ Equal SourceId (error_source err) (source_id s)
  }

fn ParserSourceLocal (a : Type) (p : Parser a) : Prop =
  (s : Source)
    → (start : Nat)
    → (h : LessEqNat start (source_length s))
    → ParseResultSourceLocal a s
    (p s start h)

fn ParserLaws (a : Type) (p : Parser a) : Prop =
  And (ParserValid a p) (And (ParserTotal a p) (ParserSourceLocal a p))

fn parser_pure (a : Type) (value : a) : Parser a =
  parser_from_decoder a (decoder_pure ByteCursor Span a value)

const parser_fail (a : Type) : Parser a =
  parser_from_decoder a (decoder_fail ByteCursor UInt8 Span a byte_cursor_ops)
```

### 4.3 A worked grammar: parenthesized Boolean expressions

`BoolExpr` is fully parenthesized: `true`, `false`, `(not e)`, and
`(and e1 e2)`. There is no precedence table — `true and false` rejects,
deliberately; a real expression grammar with precedence is out of scope for
this worked example. `Syntax a` pairs a `Located a` root with a `List` of
`Located a` children, giving every parsed node its own span independent of
its value's own recursive structure; `erase_spans` recovers the bare
`BoolExpr` by walking back down to the root value.

Token recognition is byte-by-byte through CAT-5's explicit
`byte_cursor_ops`, matching literal ASCII codepoints spelled as `Int` literals
(`116` is `t`, `102` is `f`, `40` is `(`, `32` is space, and so on). The
worked grammar is a genuine `Capability.Parsing.Decoder` client: fixed tokens use
`decoder_satisfy`/`decoder_seq`, whitespace uses progress-checked
`decoder_many`, and recursive expressions use `decoder_recursive`. Both
repetition and recursive descent seed their private structural fuel from the
cursor's `remaining`; the old CAT-5-local fuel recursions are retired.
`list_append` remains a second, verbatim package-local copy rather than a
re-export. Keeping that landed helper avoids widening this focused refactor;
the shared environment's later CAT-5 declaration intentionally shadows the
earlier catalog helper for this compilation unit.

```ken
data BoolExpr = BTrue | BFalse | BNot BoolExpr | BAnd BoolExpr BoolExpr

data Syntax a = MkSyntax (Located a) (List (Located a))

fn syntax_root (a : Type) (x : Syntax a) : Located a =
  match x {
    MkSyntax root children ↦ root
  }

fn syntax_children (a : Type) (x : Syntax a) : List (Located a) =
  match x {
    MkSyntax root children ↦ children
  }

fn erase_spans (x : Syntax BoolExpr) : BoolExpr =
  located_value BoolExpr (syntax_root BoolExpr x)

fn list_append (a : Type) (xs : List a) (ys : List a) : List a =
  match xs {
    Nil ↦ ys;
    Cons x rest ↦ Cons a x (list_append a rest ys)
  }

fn ValidLocatedList (a : Type) (s : Source) (xs : List (Located a)) : Prop =
  match xs {
    Nil ↦ Top;
    Cons x rest ↦ And (ValidLocated a s x) (ValidLocatedList a s rest)
  }

fn ValidSyntax (a : Type) (s : Source) (x : Syntax a) : Prop =
  And (ValidLocated a s (syntax_root a x)) (ValidLocatedList a s (syntax_children a x))

fn bool_expr_eq (x : BoolExpr) (y : BoolExpr) : Bool =
  match x {
    BTrue ↦
      match y {
        BTrue ↦ True;
        BFalse ↦ False;
        BNot y1 ↦ False;
        BAnd yl yr ↦ False
      };
    BFalse ↦
      match y {
        BTrue ↦ False;
        BFalse ↦ True;
        BNot y1 ↦ False;
        BAnd yl yr ↦ False
      };
    BNot x1 ↦
      match y {
        BTrue ↦ False;
        BFalse ↦ False;
        BNot y1 ↦ bool_expr_eq x1 y1;
        BAnd yl yr ↦ False
      };
    BAnd xl xr ↦
      match y {
        BTrue ↦ False;
        BFalse ↦ False;
        BNot y1 ↦ False;
        BAnd yl yr ↦
          match bool_expr_eq xl yl {
            True ↦ bool_expr_eq xr yr;
            False ↦ False
          }
      }
  }

fn syntax_leaf (s : Source) (start : Nat) (end : Nat) (value : BoolExpr) : Syntax BoolExpr =
  MkSyntax
    BoolExpr
    (MkLocated BoolExpr (source_id s) (MkSpan start end) value)
    (Nil (Located BoolExpr))

fn syntax_node_unary
      (s : Source) (start : Nat) (end : Nat) (value : BoolExpr) (child : Syntax BoolExpr)
    : Syntax BoolExpr =
  MkSyntax
    BoolExpr
    (MkLocated BoolExpr (source_id s) (MkSpan start end) value)
    (Cons (Located BoolExpr) (syntax_root BoolExpr child) (syntax_children BoolExpr child))

fn syntax_node_binary
      (s : Source)
      (start : Nat)
      (end : Nat)
      (value : BoolExpr)
      (left : Syntax BoolExpr)
      (right : Syntax BoolExpr)
    : Syntax BoolExpr =
  MkSyntax
    BoolExpr
    (MkLocated BoolExpr (source_id s) (MkSpan start end) value)
    (list_append
      (Located BoolExpr)
      (Cons (Located BoolExpr) (syntax_root BoolExpr left) (syntax_children BoolExpr left))
      (Cons (Located BoolExpr) (syntax_root BoolExpr right) (syntax_children BoolExpr right)))

fn byte_code_decoder (code : Int) : Decoder ByteCursor Span UInt8 =
  decoder_satisfy ByteCursor UInt8 Span byte_cursor_ops (λbyte. eq_int (uint8_to_int byte) code)

const true_token_decoder : Decoder ByteCursor Span UInt8 =
  decoder_seq
    ByteCursor
    Span
    UInt8
    UInt8
    (byte_code_decoder (116 : Int))
    (decoder_seq
      ByteCursor
      Span
      UInt8
      UInt8
      (byte_code_decoder (114 : Int))
      (decoder_seq
        ByteCursor
        Span
        UInt8
        UInt8
        (byte_code_decoder (117 : Int))
        (byte_code_decoder (101 : Int))))

const false_token_decoder : Decoder ByteCursor Span UInt8 =
  decoder_seq
    ByteCursor
    Span
    UInt8
    UInt8
    (byte_code_decoder (102 : Int))
    (decoder_seq
      ByteCursor
      Span
      UInt8
      UInt8
      (byte_code_decoder (97 : Int))
      (decoder_seq
        ByteCursor
        Span
        UInt8
        UInt8
        (byte_code_decoder (108 : Int))
        (decoder_seq
          ByteCursor
          Span
          UInt8
          UInt8
          (byte_code_decoder (115 : Int))
          (byte_code_decoder (101 : Int)))))

const not_open_token_decoder : Decoder ByteCursor Span UInt8 =
  decoder_seq
    ByteCursor
    Span
    UInt8
    UInt8
    (byte_code_decoder (40 : Int))
    (decoder_seq
      ByteCursor
      Span
      UInt8
      UInt8
      (byte_code_decoder (110 : Int))
      (decoder_seq
        ByteCursor
        Span
        UInt8
        UInt8
        (byte_code_decoder (111 : Int))
        (decoder_seq
          ByteCursor
          Span
          UInt8
          UInt8
          (byte_code_decoder (116 : Int))
          (byte_code_decoder (32 : Int)))))

const and_open_token_decoder : Decoder ByteCursor Span UInt8 =
  decoder_seq
    ByteCursor
    Span
    UInt8
    UInt8
    (byte_code_decoder (40 : Int))
    (decoder_seq
      ByteCursor
      Span
      UInt8
      UInt8
      (byte_code_decoder (97 : Int))
      (decoder_seq
        ByteCursor
        Span
        UInt8
        UInt8
        (byte_code_decoder (110 : Int))
        (decoder_seq
          ByteCursor
          Span
          UInt8
          UInt8
          (byte_code_decoder (100 : Int))
          (byte_code_decoder (32 : Int)))))

const spaces_decoder : Decoder ByteCursor Span (List UInt8) =
  decoder_many ByteCursor UInt8 Span UInt8 byte_cursor_ops (byte_code_decoder (32 : Int))

fn bool_true_decoder (cur : ByteCursor) : DecoderResult ByteCursor Span (Syntax BoolExpr) =
  match true_token_decoder cur {
    DecoderFailed err ↦ DecoderFailed ByteCursor Span (Syntax BoolExpr) err;
    Decoded ignored next ↦
      Decoded
        ByteCursor
        Span
        (Syntax BoolExpr)
        (syntax_leaf
          (byte_cursor_source cur)
          (byte_cursor_position cur)
          (byte_cursor_position next)
          BTrue)
        next
  }

fn bool_false_decoder (cur : ByteCursor) : DecoderResult ByteCursor Span (Syntax BoolExpr) =
  match false_token_decoder cur {
    DecoderFailed err ↦ DecoderFailed ByteCursor Span (Syntax BoolExpr) err;
    Decoded ignored next ↦
      Decoded
        ByteCursor
        Span
        (Syntax BoolExpr)
        (syntax_leaf
          (byte_cursor_source cur)
          (byte_cursor_position cur)
          (byte_cursor_position next)
          BFalse)
        next
  }

fn bool_not_decoder
      (recur : Decoder ByteCursor Span (Syntax BoolExpr))
    : Decoder ByteCursor Span (Syntax BoolExpr) =
  λcur.
    match not_open_token_decoder cur {
      DecoderFailed err ↦ DecoderFailed ByteCursor Span (Syntax BoolExpr) err;
      Decoded ignored after_open ↦
        match spaces_decoder after_open {
          DecoderFailed err ↦ DecoderFailed ByteCursor Span (Syntax BoolExpr) err;
          Decoded spaces child_start ↦
            match recur child_start {
              DecoderFailed err ↦ DecoderFailed ByteCursor Span (Syntax BoolExpr) err;
              Decoded child child_end ↦
                match spaces_decoder child_end {
                  DecoderFailed err ↦ DecoderFailed ByteCursor Span (Syntax BoolExpr) err;
                  Decoded trailing close_start ↦
                    match byte_code_decoder (41 : Int) close_start {
                      DecoderFailed err ↦ DecoderFailed ByteCursor Span (Syntax BoolExpr) err;
                      Decoded close after_close ↦
                        Decoded
                          ByteCursor
                          Span
                          (Syntax BoolExpr)
                          (syntax_node_unary
                            (byte_cursor_source cur)
                            (byte_cursor_position cur)
                            (byte_cursor_position after_close)
                            (BNot (erase_spans child))
                            child)
                          after_close
                    }
                }
            }
        }
    }

fn bool_and_decoder
      (recur : Decoder ByteCursor Span (Syntax BoolExpr))
    : Decoder ByteCursor Span (Syntax BoolExpr) =
  λcur.
    match and_open_token_decoder cur {
      DecoderFailed err ↦ DecoderFailed ByteCursor Span (Syntax BoolExpr) err;
      Decoded ignored after_open ↦
        match spaces_decoder after_open {
          DecoderFailed err ↦ DecoderFailed ByteCursor Span (Syntax BoolExpr) err;
          Decoded leading left_start ↦
            match recur left_start {
              DecoderFailed err ↦ DecoderFailed ByteCursor Span (Syntax BoolExpr) err;
              Decoded left left_end ↦
                match byte_code_decoder (32 : Int) left_end {
                  DecoderFailed err ↦ DecoderFailed ByteCursor Span (Syntax BoolExpr) err;
                  Decoded separator after_separator ↦
                    match spaces_decoder after_separator {
                      DecoderFailed err ↦ DecoderFailed ByteCursor Span (Syntax BoolExpr) err;
                      Decoded middle right_start ↦
                        match recur right_start {
                          DecoderFailed err ↦
                            DecoderFailed ByteCursor Span (Syntax BoolExpr) err;
                          Decoded right right_end ↦
                            match spaces_decoder right_end {
                              DecoderFailed err ↦
                                DecoderFailed ByteCursor Span (Syntax BoolExpr) err;
                              Decoded trailing close_start ↦
                                match byte_code_decoder (41 : Int) close_start {
                                  DecoderFailed err ↦
                                    DecoderFailed ByteCursor Span (Syntax BoolExpr) err;
                                  Decoded close after_close ↦
                                    Decoded
                                      ByteCursor
                                      Span
                                      (Syntax BoolExpr)
                                      (syntax_node_binary
                                        (byte_cursor_source cur)
                                        (byte_cursor_position cur)
                                        (byte_cursor_position after_close)
                                        (BAnd (erase_spans left) (erase_spans right))
                                        left
                                        right)
                                      after_close
                                }
                            }
                        }
                    }
                }
            }
        }
    }

fn bool_decoder_layer
      (recur : Decoder ByteCursor Span (Syntax BoolExpr))
    : Decoder ByteCursor Span (Syntax BoolExpr) =
  decoder_alt
    ByteCursor
    Span
    (Syntax BoolExpr)
    bool_true_decoder
    (decoder_alt
      ByteCursor
      Span
      (Syntax BoolExpr)
      bool_false_decoder
      (decoder_alt
        ByteCursor
        Span
        (Syntax BoolExpr)
        (bool_not_decoder recur)
        (bool_and_decoder recur)))

const bool_expression_decoder : Decoder ByteCursor Span (Syntax BoolExpr) =
  decoder_recursive ByteCursor UInt8 Span (Syntax BoolExpr) byte_cursor_ops bool_decoder_layer

fn complete_bool_decoder (cur : ByteCursor) : DecoderResult ByteCursor Span (Syntax BoolExpr) =
  match spaces_decoder cur {
    DecoderFailed err ↦ DecoderFailed ByteCursor Span (Syntax BoolExpr) err;
    Decoded leading start ↦
      match bool_expression_decoder start {
        DecoderFailed err ↦ DecoderFailed ByteCursor Span (Syntax BoolExpr) err;
        Decoded syntax next ↦
          match spaces_decoder next {
            DecoderFailed err ↦ DecoderFailed ByteCursor Span (Syntax BoolExpr) err;
            Decoded trailing end ↦
              match byte_cursor_remaining end {
                Zero ↦ Decoded ByteCursor Span (Syntax BoolExpr) syntax end;
                Suc rest ↦
                  DecoderFailed
                    ByteCursor
                    Span
                    (Syntax BoolExpr)
                    (DecoderRejected Span (byte_cursor_locate end))
              }
          }
      }
  }

const parse_bool_expr : Parser (Syntax BoolExpr) =
  parser_from_decoder (Syntax BoolExpr) complete_bool_decoder

fn print_bool_expr (e : BoolExpr) : Bytes =
  match e {
    BTrue ↦ bytes_encode "true";
    BFalse ↦ bytes_encode "false";
    BNot child ↦
      bytes_concat
        (bytes_concat (bytes_encode "(not ") (print_bool_expr child))
        (bytes_encode ")");
    BAnd left right ↦
      bytes_concat
        (bytes_concat
          (bytes_concat (bytes_encode "(and ") (print_bool_expr left))
          (bytes_encode " "))
        (bytes_concat (print_bool_expr right) (bytes_encode ")"))
  }

fn format_bool_expr (s : Source) : Result ParseError Bytes =
  match parse_bool_expr s Zero ((proof zero_left for LessEqNat) (source_length s)) {
    Parsed syntax consumed next ↦ Ok ParseError Bytes (print_bool_expr (erase_spans syntax));
    Failed err ↦ Err ParseError Bytes err
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

**Why repetition is progress checked.** `Capability.Parsing.Decoder` exports repetition,
but a successful step must strictly decrease the cursor's remaining count.
`decoder_many` reports the named `DecoderZeroProgress` failure when that
check fails and derives its private fuel from `remaining`, so it neither loops
nor truncates at an arbitrary caller budget. CAT-5's whitespace decoder is a
direct client of that shared mechanism.

**Why the Boolean grammar has no precedence table.** `true`, `false`,
`(not e)`, and `(and e1 e2)` are fully parenthesized on purpose — `true and
false` rejects, deliberately, keeping this worked example small. A real
expression grammar with precedence climbing is future package work, not
attempted here.

## 6. References

None — this entry's design is Ken-native, not consulted from an external
reference implementation.

## 7. Trust  derivation

1. **Public API.** `Source`, `IsUtf8`, `source_id`, `source_bytes`,
   `source_bytes::utf8`, `source_length`, `Span`, `span_start`,
   `span_end`, `span_to_byte_range`, `span_origin`, `ByteCursor`,
   `byte_cursor_ops`, `LessEqNat`, `ValidSpan`,
   `Located`, `located_source`,
   `located_span`, `located_value`, `ValidLocated`,
   `valid_zero_width_span`, `ParseError`, `error_source`, `error_span`,
   `ParseResult`, `Parser`, `ParsedValid`, `FailedValid`,
   `ParseResultValid`, `ParserValid`, `ParserTotal`, `ParserSourceLocal`,
   `ParserLaws`, `parser_from_decoder`, `parser_pure`, `parser_fail`,
   `BoolExpr`, `Syntax`,
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
   `LessEqNat::refl`, `LessEqNat::zero_left`, `valid_zero_width_span` — is
   real and kernel-checked; no law or predicate is postulated.
5. **Proof families.** `LessEqNat::refl` — induction on `n`.
   `LessEqNat::zero_left` — definitional (first match arm). `valid_zero_width_span`
   — direct composition of the two via `and_intro`, no case-split of its
   own.
6. **Consumers.** Source-aware parser implementations can use this package's
   source, span, result, and validity vocabulary.
7. **Validation evidence.** The catalog checks the
   `Source`/`Span`/`Located`/`ParseResult`/`Parser` surface, its zero
   `trusted_base()` delta, the Boolean grammar's constructors and byte-token
   matching, and the absence of an exported unguarded repetition combinator.
