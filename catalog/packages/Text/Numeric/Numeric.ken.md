# `Text.Numeric` — located decimal parsing

`Text.Numeric` parses decimal characters into arbitrary-precision `Int` values
and reports the exact character index of the first failure. Its formatting
floor is structural: decimal digits convert to characters and back without
crossing the opaque `String`/`List Char` bijection.

## Index

1. [Motivation](#1-motivation)
2. [Located errors](#2-located-errors)
3. [Decimal parsing](#3-decimal-parsing)
4. [Structural formatting](#4-structural-formatting)
5. [Checked examples](#5-checked-examples)
6. [Trust and derivation](#6-trust-and-derivation)

**Named reading paths**

- *Newcomer* → [Motivation](#1-motivation) → [Decimal parsing](#3-decimal-parsing)
- *Practitioner* → [Decimal parsing](#3-decimal-parsing) →
  [Checked examples](#5-checked-examples)
- *Researcher* → [Structural formatting](#4-structural-formatting) →
  [Trust and derivation](#6-trust-and-derivation)

## 1. Motivation

Parsing is total: success is `Ok`, and failure is an ordinary `Err` value.
Locations count Unicode scalar values in the input `List Char`, never UTF-8
bytes. `Int` is arbitrary-precision, so there is deliberately no overflow
case.

## 2. Located errors

This is the minimal pre-diagnostic carrier. A later diagnostic package can
subsume it without changing the parsing semantics.

```ken
data NumericErrorKind = EmptyInput | InvalidDigit

data NumericError = MkNumericError NumericErrorKind Nat
```

## 3. Decimal parsing

`char_to_digit` uses the landed identity projection `charToInt` and the landed
integer order. The recursive worker carries both the character index and the
base-ten accumulator.

```ken
fn char_to_digit (c : Char) : Option Int =
  match and_bool (leq_int (48 : Int) (charToInt c)) (leq_int (charToInt c) (57 : Int)) {
    True ↦ Some Int (sub_int (charToInt c) (48 : Int));
    False ↦ None Int
  }

fn parse_digits_at
      (chars : List Char) (position : Nat) (accumulator : Int)
    : Result NumericError Int =
  match chars {
    Nil ↦ Ok NumericError Int accumulator;
    Cons c rest ↦
      match char_to_digit c {
        None ↦ Err NumericError Int (MkNumericError InvalidDigit position);
        Some digit ↦ parse_digits_at
          rest
          (Suc position)
          (add_int (mul_int accumulator (10 : Int)) digit)
      }
  }

fn parse_nat_chars (chars : List Char) : Result NumericError Int =
  match chars {
    Nil ↦ Err NumericError Int (MkNumericError EmptyInput Zero);
    Cons c rest ↦ parse_digits_at (Cons Char c rest) Zero (0 : Int)
  }

fn negate_parsed (x : Result NumericError Int) : Result NumericError Int =
  match x {
    Err problem ↦ Err NumericError Int problem;
    Ok value ↦ Ok NumericError Int (sub_int (0 : Int) value)
  }

fn parse_int_chars (chars : List Char) : Result NumericError Int =
  match chars {
    Nil ↦ Err NumericError Int (MkNumericError EmptyInput Zero);
    Cons c rest ↦
      match eq_int (charToInt c) (45 : Int) {
        True ↦
          match rest {
            Nil ↦ Err NumericError Int (MkNumericError EmptyInput (Suc Zero));
            Cons d more ↦ negate_parsed
              (parse_digits_at (Cons Char d more) (Suc Zero) (0 : Int))
          };
        False ↦ parse_digits_at (Cons Char c rest) Zero (0 : Int)
      }
  }

fn parse_nat (text : String) : Result NumericError Int =
  parse_nat_chars (string_to_list_char text)

fn parse_int (text : String) : Result NumericError Int =
  parse_int_chars (string_to_list_char text)
```

## 4. Structural formatting

The format direction uses an explicit digit carrier. This makes its verified
round trip a purely structural `List DecimalDigit`/`List Char` theorem. The
String-facing wrapper remains only a function; no universal String bijection
law is asserted. A total `show_int : Int → String` is a named fast-follow:
opaque `Int` has no division, remainder, destructor, or `Int → Nat` bridge, so
CC2 does not fake that missing operation with a bounded table or a
non-structural loop.

```ken
data DecimalDigit : Type where {
  MkDecimalDigit :
    (value : Int)
    → (glyph : Char)
    → Equal (Option Int) (char_to_digit glyph) (Some Int value)
    → DecimalDigit
}

fn decimal_digit_value (digit : DecimalDigit) : Int =
  match digit {
    MkDecimalDigit value glyph valid ↦ value
  }

fn decimal_digit_to_char (digit : DecimalDigit) : Char =
  match digit {
    MkDecimalDigit value glyph valid ↦ glyph
  }

proof valid for decimal_digit_to_char
      (digit : DecimalDigit)
    : Equal
        (Option Int)
        (char_to_digit (decimal_digit_to_char digit))
        (Some Int (decimal_digit_value digit)) =
  match digit {
    MkDecimalDigit value glyph valid ↦ valid
  }

fn decimal_digit_values (digits : List DecimalDigit) : List Int =
  match digits {
    Nil ↦ Nil Int;
    Cons digit rest ↦ Cons Int (decimal_digit_value digit) (decimal_digit_values rest)
  }

fn format_digits (digits : List DecimalDigit) : List Char =
  match digits {
    Nil ↦ Nil Char;
    Cons digit rest ↦ Cons Char (decimal_digit_to_char digit) (format_digits rest)
  }

fn parsed_int_prepend (digit : Int) (parsed : Option (List Int)) : Option (List Int) =
  match parsed {
    None ↦ None (List Int);
    Some rest ↦ Some (List Int) (Cons Int digit rest)
  }

fn parse_digit_result
      (parsed_rest : Option (List Int)) (parsed_digit : Option Int)
    : Option (List Int) =
  match parsed_digit {
    None ↦ None (List Int);
    Some digit ↦ parsed_int_prepend digit parsed_rest
  }

fn parse_formatted_digits (chars : List Char) : Option (List Int) =
  match chars {
    Nil ↦ Some (List Int) (Nil Int);
    Cons c rest ↦ parse_digit_result (parse_formatted_digits rest) (char_to_digit c)
  }

fn show_digits (digits : List DecimalDigit) : String =
  list_char_to_string (format_digits digits)

lemma format_digits_roundtrip
      (digits : List DecimalDigit)
    : Equal
        (Option (List Int))
        (parse_formatted_digits (format_digits digits))
        (Some (List Int) (decimal_digit_values digits)) =
  match digits {
    Nil ↦ Proved;
    Cons digit rest ↦ trans
      (Option (List Int))
      (parse_digit_result
        (parse_formatted_digits (format_digits rest))
        (char_to_digit (decimal_digit_to_char digit)))
      (parsed_int_prepend
        (decimal_digit_value digit)
        (parse_formatted_digits (format_digits rest)))
      (Some (List Int) (Cons Int (decimal_digit_value digit) (decimal_digit_values rest)))
      (cong
        (Option Int)
        (Option (List Int))
        (char_to_digit (decimal_digit_to_char digit))
        (Some Int (decimal_digit_value digit))
        (parse_digit_result (parse_formatted_digits (format_digits rest)))
        ((proof valid for decimal_digit_to_char) digit))
      (cong
        (Option (List Int))
        (Option (List Int))
        (parse_formatted_digits (format_digits rest))
        (Some (List Int) (decimal_digit_values rest))
        (parsed_int_prepend (decimal_digit_value digit))
        (format_digits_roundtrip rest))
  }
```

## 5. Checked examples

These cases pin the valid, empty, negative, and exact-index failure semantics.

```ken example
const digit_zero_result : Option Int = char_to_digit (48 : Int)

const digit_nine_result : Option Int = char_to_digit (57 : Int)

const letter_digit_result : Option Int = char_to_digit (120 : Int)

const parsed_decimal_result : Result NumericError Int = parse_nat "123"

const empty_input_result : Result NumericError Int = parse_nat ""

const bad_digit_result : Result NumericError Int = parse_nat "12x4"

const parsed_negative_result : Result NumericError Int = parse_int "-42"
```

## 6. Trust and derivation

**Public API:** `NumericErrorKind`, `NumericError`, `char_to_digit`,
`parse_digits_at`, `parse_nat_chars`, `parse_int_chars`, `parse_nat`,
`parse_int`, `DecimalDigit`, `format_digits`, `parse_formatted_digits`,
`format_digits_roundtrip`, and `show_digits`.

**Derivation.** Parsing uses structural recursion on `List Char`, positions use
structural `Nat`, and values use the landed `charToInt`, `leq_int`, `eq_int`,
`add_int`, `mul_int`, and `sub_int` operations. The verified format law never
mentions `String` or consumes an opaque `Int`; `show_digits` is only a function
across the String boundary. `show_int : Int → String` remains deferred until a
sound Int-destruction substrate exists.

**Trust delta:** zero. The package declares no primitive, postulate, opaque
constant, or `Axiom`.
