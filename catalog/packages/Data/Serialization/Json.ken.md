# `Json` — a lawful character codec

`Json` supplies a pure JSON value model, a `List Char` encoder and decoder,
and checked laws for the character cursor and the codec.

## Contents

1. [Motivation](#1-motivation)
2. [Definition](#2-definition)
3. [Using it](#3-using-it)
4. [Laws & proofs](#4-laws--proofs)
5. [Design notes](#5-design-notes)
6. [References](#6-references)
7. [Trust & derivation](#7-trust--derivation)

## 1. Motivation

JSON is the first catalog driver that composes the landed collection, lawful
equality, string-view, cursor, and decoder packages into one recursive codec.
The law-bearing carrier is `List Char`; byte and string convenience shells are
deliberately outside this package.

## 2. Definition

The numeric domain is canonical decimal integers: zero, or a non-zero leading
digit followed by decimal digits, with an optional minus sign. Fractions and
exponents are intentionally excluded and recorded in Design notes.

```ken
data JsonDigit =
  JsonDigit0
  | JsonDigit1
  | JsonDigit2
  | JsonDigit3
  | JsonDigit4
  | JsonDigit5
  | JsonDigit6
  | JsonDigit7
  | JsonDigit8
  | JsonDigit9

data JsonNonZeroDigit =
  JsonNonZero1
  | JsonNonZero2
  | JsonNonZero3
  | JsonNonZero4
  | JsonNonZero5
  | JsonNonZero6
  | JsonNonZero7
  | JsonNonZero8
  | JsonNonZero9

data JsonInteger =
  JsonIntegerZero
  | JsonIntegerPositive JsonNonZeroDigit (List JsonDigit)
  | JsonIntegerNegative JsonNonZeroDigit (List JsonDigit)

data Json =
  JsonNull
  | JsonBool Bool
  | JsonNumber JsonInteger
  | JsonString String
  | JsonArray (List Json)
  | JsonObject (List (Pair String Json))

fn char_cursor_remaining (chars : List Char) : Nat = length Char chars

fn char_cursor_peek (chars : List Char) : Option Char =
  match chars {
    Nil ↦ None Char;
    Cons head tail ↦ Some Char head
  }

fn char_cursor_advance (chars : List Char) : List Char =
  match chars {
    Nil ↦ Nil Char;
    Cons head tail ↦ tail
  }

fn char_cursor_locate (chars : List Char) : Nat = length Char chars

const char_cursor_ops : CursorOps (List Char) Char Nat =
  MkCursorOps
    (List Char)
    Char
    Nat
    char_cursor_remaining
    char_cursor_peek
    char_cursor_advance
    char_cursor_locate

fn json_char_equal (left : Char) (right : Char) : Bool =
  (DecEq_instance_Char).eq left right

fn json_char_plain (character : Char) : Bool =
  and_bool
    (leq_int (32 : Int) (charToInt character))
    (and_bool
      (not_bool (json_char_equal character (34 : Char)))
      (not_bool (json_char_equal character (92 : Char))))

fn json_chars_plain (characters : List Char) : Bool =
  match characters {
    Nil ↦ True;
    Cons character rest ↦
      and_bool (json_char_plain character) (json_chars_plain rest)
  }

fn json_token (expected : Char) : Decoder (List Char) Nat Char =
  decoder_token
    (List Char)
    Char
    Nat
    char_cursor_ops
    json_char_equal
    expected

fn json_plain_character_decoder : Decoder (List Char) Nat Char =
  decoder_satisfy
    (List Char)
    Char
    Nat
    char_cursor_ops
    json_char_plain

fn json_string_decoder : Decoder (List Char) Nat String =
  decoder_bind
    (List Char)
    Nat
    Char
    String
    (json_token (34 : Char))
    (λopening.
      decoder_bind
        (List Char)
        Nat
        (List Char)
        String
        (decoder_many
          (List Char)
          Char
          Nat
          Char
          char_cursor_ops
          json_plain_character_decoder)
        (λcharacters.
          decoder_map
            (List Char)
            Nat
            Char
            String
            (λclosing. list_char_to_string characters)
            (json_token (34 : Char))))

fn json_digit_to_char (digit : JsonDigit) : Char =
  match digit {
    JsonDigit0 ↦ (48 : Char);
    JsonDigit1 ↦ (49 : Char);
    JsonDigit2 ↦ (50 : Char);
    JsonDigit3 ↦ (51 : Char);
    JsonDigit4 ↦ (52 : Char);
    JsonDigit5 ↦ (53 : Char);
    JsonDigit6 ↦ (54 : Char);
    JsonDigit7 ↦ (55 : Char);
    JsonDigit8 ↦ (56 : Char);
    JsonDigit9 ↦ (57 : Char)
  }

fn json_nonzero_to_char (digit : JsonNonZeroDigit) : Char =
  match digit {
    JsonNonZero1 ↦ (49 : Char);
    JsonNonZero2 ↦ (50 : Char);
    JsonNonZero3 ↦ (51 : Char);
    JsonNonZero4 ↦ (52 : Char);
    JsonNonZero5 ↦ (53 : Char);
    JsonNonZero6 ↦ (54 : Char);
    JsonNonZero7 ↦ (55 : Char);
    JsonNonZero8 ↦ (56 : Char);
    JsonNonZero9 ↦ (57 : Char)
  }

fn json_digits_encode (digits : List JsonDigit) : List Char =
  match digits {
    Nil ↦ Nil Char;
    Cons digit rest ↦
      Cons Char (json_digit_to_char digit) (json_digits_encode rest)
  }

fn json_integer_encode (number : JsonInteger) : List Char =
  match number {
    JsonIntegerZero ↦ Cons Char (48 : Char) (Nil Char);
    JsonIntegerPositive leading rest ↦
      Cons Char (json_nonzero_to_char leading) (json_digits_encode rest);
    JsonIntegerNegative leading rest ↦
      Cons
        Char
        (45 : Char)
        (Cons Char (json_nonzero_to_char leading) (json_digits_encode rest))
  }

fn json_digit_decoder : Decoder (List Char) Nat JsonDigit =
  decoder_alt
    (List Char)
    Nat
    JsonDigit
    (decoder_map (List Char) Nat Char JsonDigit (λc. JsonDigit0) (json_token (48 : Char)))
    (decoder_alt
      (List Char)
      Nat
      JsonDigit
      (decoder_map (List Char) Nat Char JsonDigit (λc. JsonDigit1) (json_token (49 : Char)))
      (decoder_alt
        (List Char)
        Nat
        JsonDigit
        (decoder_map (List Char) Nat Char JsonDigit (λc. JsonDigit2) (json_token (50 : Char)))
        (decoder_alt
          (List Char)
          Nat
          JsonDigit
          (decoder_map (List Char) Nat Char JsonDigit (λc. JsonDigit3) (json_token (51 : Char)))
          (decoder_alt
            (List Char)
            Nat
            JsonDigit
            (decoder_map (List Char) Nat Char JsonDigit (λc. JsonDigit4) (json_token (52 : Char)))
            (decoder_alt
              (List Char)
              Nat
              JsonDigit
              (decoder_map (List Char) Nat Char JsonDigit (λc. JsonDigit5) (json_token (53 : Char)))
              (decoder_alt
                (List Char)
                Nat
                JsonDigit
                (decoder_map
                  (List Char)
                  Nat
                  Char
                  JsonDigit
                  (λc. JsonDigit6)
                  (json_token (54 : Char)))
                (decoder_alt
                  (List Char)
                  Nat
                  JsonDigit
                  (decoder_map
                    (List Char)
                    Nat
                    Char
                    JsonDigit
                    (λc. JsonDigit7)
                    (json_token (55 : Char)))
                  (decoder_alt
                    (List Char)
                    Nat
                    JsonDigit
                    (decoder_map
                      (List Char)
                      Nat
                      Char
                      JsonDigit
                      (λc. JsonDigit8)
                      (json_token (56 : Char)))
                    (decoder_map
                      (List Char)
                      Nat
                      Char
                      JsonDigit
                      (λc. JsonDigit9)
                      (json_token (57 : Char)))))))))))

fn json_nonzero_decoder : Decoder (List Char) Nat JsonNonZeroDigit =
  decoder_alt
    (List Char)
    Nat
    JsonNonZeroDigit
    (decoder_map
      (List Char)
      Nat
      Char
      JsonNonZeroDigit
      (λc. JsonNonZero1)
      (json_token (49 : Char)))
    (decoder_alt
      (List Char)
      Nat
      JsonNonZeroDigit
      (decoder_map
        (List Char)
        Nat
        Char
        JsonNonZeroDigit
        (λc. JsonNonZero2)
        (json_token (50 : Char)))
      (decoder_alt
        (List Char)
        Nat
        JsonNonZeroDigit
        (decoder_map
          (List Char)
          Nat
          Char
          JsonNonZeroDigit
          (λc. JsonNonZero3)
          (json_token (51 : Char)))
        (decoder_alt
          (List Char)
          Nat
          JsonNonZeroDigit
          (decoder_map
            (List Char)
            Nat
            Char
            JsonNonZeroDigit
            (λc. JsonNonZero4)
            (json_token (52 : Char)))
          (decoder_alt
            (List Char)
            Nat
            JsonNonZeroDigit
            (decoder_map
              (List Char)
              Nat
              Char
              JsonNonZeroDigit
              (λc. JsonNonZero5)
              (json_token (53 : Char)))
            (decoder_alt
              (List Char)
              Nat
              JsonNonZeroDigit
              (decoder_map
                (List Char)
                Nat
                Char
                JsonNonZeroDigit
                (λc. JsonNonZero6)
                (json_token (54 : Char)))
              (decoder_alt
                (List Char)
                Nat
                JsonNonZeroDigit
                (decoder_map
                  (List Char)
                  Nat
                  Char
                  JsonNonZeroDigit
                  (λc. JsonNonZero7)
                  (json_token (55 : Char)))
                (decoder_alt
                  (List Char)
                  Nat
                  JsonNonZeroDigit
                  (decoder_map
                    (List Char)
                    Nat
                    Char
                    JsonNonZeroDigit
                    (λc. JsonNonZero8)
                    (json_token (56 : Char)))
                  (decoder_map
                    (List Char)
                    Nat
                    Char
                    JsonNonZeroDigit
                    (λc. JsonNonZero9)
                    (json_token (57 : Char))))))))))

fn json_positive_integer_decoder : Decoder (List Char) Nat JsonInteger =
  decoder_bind
    (List Char)
    Nat
    JsonNonZeroDigit
    JsonInteger
    json_nonzero_decoder
    (λleading.
      decoder_map
        (List Char)
        Nat
        (List JsonDigit)
        JsonInteger
        (JsonIntegerPositive leading)
        (decoder_many
          (List Char)
          Char
          Nat
          JsonDigit
          char_cursor_ops
          json_digit_decoder))

fn json_negative_integer_decoder : Decoder (List Char) Nat JsonInteger =
  decoder_seq
    (List Char)
    Nat
    Char
    JsonInteger
    (json_token (45 : Char))
    (decoder_bind
      (List Char)
      Nat
      JsonNonZeroDigit
      JsonInteger
      json_nonzero_decoder
      (λleading.
        decoder_map
          (List Char)
          Nat
          (List JsonDigit)
          JsonInteger
          (JsonIntegerNegative leading)
          (decoder_many
            (List Char)
            Char
            Nat
            JsonDigit
            char_cursor_ops
            json_digit_decoder)))

fn json_integer_decoder : Decoder (List Char) Nat JsonInteger =
  decoder_alt
    (List Char)
    Nat
    JsonInteger
    (decoder_map
      (List Char)
      Nat
      Char
      JsonInteger
      (λc. JsonIntegerZero)
      (json_token (48 : Char)))
    (decoder_alt
      (List Char)
      Nat
      JsonInteger
      json_negative_integer_decoder
      json_positive_integer_decoder)

fn json_literal
      (a : Type) (value : a) (text : List Char)
    : Decoder (List Char) Nat a =
  match text {
    Nil ↦ decoder_pure (List Char) Nat a value;
    Cons character rest ↦
      decoder_seq
        (List Char)
        Nat
        Char
        a
        (json_token character)
        (json_literal a value rest)
  }

fn json_comma_value_decoder
      (recur : Decoder (List Char) Nat Json)
    : Decoder (List Char) Nat Json =
  decoder_seq
    (List Char)
    Nat
    Char
    Json
    (json_token (44 : Char))
    recur

fn json_array_decoder
      (recur : Decoder (List Char) Nat Json)
    : Decoder (List Char) Nat Json =
  decoder_seq
    (List Char)
    Nat
    Char
    Json
    (json_token (91 : Char))
    (decoder_alt
      (List Char)
      Nat
      Json
      (decoder_map
        (List Char)
        Nat
        Char
        Json
        (λclosing. JsonArray (Nil Json))
        (json_token (93 : Char)))
      (decoder_bind
        (List Char)
        Nat
        Json
        Json
        recur
        (λfirst.
          decoder_bind
            (List Char)
            Nat
            (List Json)
            Json
            (decoder_many
              (List Char)
              Char
              Nat
              Json
              char_cursor_ops
              (json_comma_value_decoder recur))
            (λrest.
              decoder_map
                (List Char)
                Nat
                Char
                Json
                (λclosing. JsonArray (Cons Json first rest))
                (json_token (93 : Char))))))

fn json_member_decoder
      (recur : Decoder (List Char) Nat Json)
    : Decoder (List Char) Nat (Pair String Json) =
  decoder_bind
    (List Char)
    Nat
    String
    (Pair String Json)
    json_string_decoder
    (λkey.
      decoder_seq
        (List Char)
        Nat
        Char
        (Pair String Json)
        (json_token (58 : Char))
        (decoder_map
          (List Char)
          Nat
          Json
          (Pair String Json)
          (mk_pair String Json key)
          recur))

fn json_comma_member_decoder
      (recur : Decoder (List Char) Nat Json)
    : Decoder (List Char) Nat (Pair String Json) =
  decoder_seq
    (List Char)
    Nat
    Char
    (Pair String Json)
    (json_token (44 : Char))
    (json_member_decoder recur)

fn json_key_present (key : String) (members : List (Pair String Json)) : Bool =
  match members {
    Nil ↦ False;
    Cons member rest ↦
      match (DecEq_instance_String).eq key (pair_fst String Json member) {
        True ↦ True;
        False ↦ json_key_present key rest
      }
  }

fn json_members_unique (members : List (Pair String Json)) : Bool =
  match members {
    Nil ↦ True;
    Cons member rest ↦
      match json_key_present (pair_fst String Json member) rest {
        True ↦ False;
        False ↦ json_members_unique rest
      }
  }

fn json_checked_object
      (members : List (Pair String Json))
    : Decoder (List Char) Nat Json =
  match json_members_unique members {
    True ↦ decoder_pure (List Char) Nat Json (JsonObject members);
    False ↦ decoder_fail (List Char) Char Nat Json char_cursor_ops
  }

fn json_object_decoder
      (recur : Decoder (List Char) Nat Json)
    : Decoder (List Char) Nat Json =
  decoder_seq
    (List Char)
    Nat
    Char
    Json
    (json_token (123 : Char))
    (decoder_alt
      (List Char)
      Nat
      Json
      (decoder_map
        (List Char)
        Nat
        Char
        Json
        (λclosing. JsonObject (Nil (Pair String Json)))
        (json_token (125 : Char)))
      (decoder_bind
        (List Char)
        Nat
        (Pair String Json)
        Json
        (json_member_decoder recur)
        (λfirst.
          decoder_bind
            (List Char)
            Nat
            (List (Pair String Json))
            Json
            (decoder_many
              (List Char)
              Char
              Nat
              (Pair String Json)
              char_cursor_ops
              (json_comma_member_decoder recur))
            (λrest.
              decoder_seq
                (List Char)
                Nat
                Char
                Json
                (json_token (125 : Char))
                (json_checked_object (Cons (Pair String Json) first rest))))))

fn json_decoder_layer
      (recur : Decoder (List Char) Nat Json)
    : Decoder (List Char) Nat Json =
  decoder_alt
    (List Char)
    Nat
    Json
    (json_literal Json JsonNull (string_to_list_char "null"))
    (decoder_alt
      (List Char)
      Nat
      Json
      (json_literal Json (JsonBool True) (string_to_list_char "true"))
      (decoder_alt
        (List Char)
        Nat
        Json
        (json_literal Json (JsonBool False) (string_to_list_char "false"))
        (decoder_alt
          (List Char)
          Nat
          Json
          (decoder_map
            (List Char)
            Nat
            String
            Json
            JsonString
            json_string_decoder)
          (decoder_alt
            (List Char)
            Nat
            Json
            (decoder_map
              (List Char)
              Nat
              JsonInteger
              Json
              JsonNumber
              json_integer_decoder)
            (decoder_alt
              (List Char)
              Nat
              Json
              (json_array_decoder recur)
              (json_object_decoder recur))))))

const json_decoder : Decoder (List Char) Nat Json =
  decoder_recursive
    (List Char)
    Char
    Nat
    Json
    char_cursor_ops
    json_decoder_layer

data JsonError = JsonDecoderError (DecoderError Nat)

fn decode (characters : List Char) : Result JsonError Json =
  match json_decoder characters {
    DecoderFailed problem ↦ Err JsonError Json (JsonDecoderError problem);
    Decoded value rest ↦
      match rest {
        Nil ↦ Ok JsonError Json value;
        Cons character tail ↦
          Err
            JsonError
            Json
            (JsonDecoderError
              (DecoderRejected Nat (char_cursor_locate rest)))
      }
  }

fn json_string_encode (text : String) : List Char =
  Cons
    Char
    (34 : Char)
    (list_append
      Char
      (string_to_list_char text)
      (Cons Char (34 : Char) (Nil Char)))

fn json_values_encode (values : List Json) : List Char =
  match values {
    Nil ↦ Nil Char;
    Cons value rest ↦
      match rest {
        Nil ↦ encode value;
        Cons next tail ↦
          list_append
            Char
            (encode value)
            (Cons Char (44 : Char) (json_values_encode (Cons Json next tail)))
      }
  }

fn json_member_encode (member : Pair String Json) : List Char =
  list_append
    Char
    (json_string_encode (pair_fst String Json member))
    (Cons Char (58 : Char) (encode (pair_snd String Json member)))

fn json_members_encode (members : List (Pair String Json)) : List Char =
  match members {
    Nil ↦ Nil Char;
    Cons member rest ↦
      match rest {
        Nil ↦ json_member_encode member;
        Cons next tail ↦
          list_append
            Char
            (json_member_encode member)
            (Cons
              Char
              (44 : Char)
              (json_members_encode (Cons (Pair String Json) next tail)))
      }
  }

fn encode (value : Json) : List Char =
  match value {
    JsonNull ↦ string_to_list_char "null";
    JsonBool truth ↦
      match truth {
        True ↦ string_to_list_char "true";
        False ↦ string_to_list_char "false"
      };
    JsonNumber number ↦ json_integer_encode number;
    JsonString text ↦ json_string_encode text;
    JsonArray values ↦
      Cons
        Char
        (91 : Char)
        (list_append
          Char
          (json_values_encode values)
          (Cons Char (93 : Char) (Nil Char)));
    JsonObject members ↦
      Cons
        Char
        (123 : Char)
        (list_append
          Char
          (json_members_encode members)
          (Cons Char (125 : Char) (Nil Char)))
  }

fn json_values_encodable (values : List Json) : Prop =
  match values {
    Nil ↦ Top;
    Cons value rest ↦ And (JsonEncodable value) (json_values_encodable rest)
  }

fn json_members_encodable (members : List (Pair String Json)) : Prop =
  match members {
    Nil ↦ Top;
    Cons member rest ↦
      And
        (IsTrue
          (json_chars_plain
            (string_to_list_char (pair_fst String Json member))))
        (And
          (JsonEncodable (pair_snd String Json member))
          (json_members_encodable rest))
  }

fn JsonEncodable (value : Json) : Prop =
  match value {
    JsonNull ↦ Top;
    JsonBool truth ↦ Top;
    JsonNumber number ↦ Top;
    JsonString text ↦
      IsTrue (json_chars_plain (string_to_list_char text));
    JsonArray values ↦ json_values_encodable values;
    JsonObject members ↦
      And
        (IsTrue (json_members_unique members))
        (json_members_encodable members)
  }
```

## 3. Using it

`encode` emits compact canonical text. `decode` consumes the complete input and
returns a `JsonDecoderError` containing the exact `DecoderError Nat`; its
location is the number of characters remaining at the failing cursor.

## 4. Laws & proofs

The character cursor laws are structural: a successful peek implies a
non-empty list, advancing removes exactly one constructor, and zero remaining
characters implies `Nil`.

```ken
theorem char_cursor_peek_has_remaining
      (characters : List Char)
    : (value : Char)
      → Equal
        (Option Char)
        (cursor_peek
          (List Char)
          Char
          Nat
          char_cursor_ops
          characters)
        (Some Char value)
      → Equal
        Bool
        (cursor_nat_lt
          Zero
          (cursor_remaining
            (List Char)
            Char
            Nat
            char_cursor_ops
            characters))
        True =
  match characters {
    Nil ↦ λvalue. λpeek. absurd peek;
    Cons head tail ↦ λvalue. λpeek. Proved
  }

theorem char_cursor_advance_progress
      (characters : List Char)
    : (value : Char)
      → Equal
        (Option Char)
        (cursor_peek
          (List Char)
          Char
          Nat
          char_cursor_ops
          characters)
        (Some Char value)
      → Equal
        Bool
        (cursor_nat_lt
          (cursor_remaining
            (List Char)
            Char
            Nat
            char_cursor_ops
            (cursor_advance
              (List Char)
              Char
              Nat
              char_cursor_ops
              characters))
          (cursor_remaining
            (List Char)
            Char
            Nat
            char_cursor_ops
            characters))
        True =
  match characters {
    Nil ↦ λvalue. λpeek. absurd peek;
    Cons head tail ↦ λvalue. λpeek. Proved
  }

theorem char_cursor_end_valid
      (characters : List Char)
    : Equal
        Nat
        (cursor_remaining
          (List Char)
          Char
          Nat
          char_cursor_ops
          characters)
        Zero
      → Equal
        (Option Char)
        (cursor_peek
          (List Char)
          Char
          Nat
          char_cursor_ops
          characters)
        (None Char) =
  match characters {
    Nil ↦ λremaining. Refl;
    Cons head tail ↦ λremaining. absurd remaining
  }

proof lawful for char_cursor_ops
    : CursorLaws (List Char) Char Nat char_cursor_ops =
  and_intro
    (CursorPeekHasRemaining
      (List Char)
      Char
      Nat
      char_cursor_ops)
    (And
      (CursorAdvanceProgress
        (List Char)
        Char
        Nat
        char_cursor_ops)
      (CursorEndValid
        (List Char)
        Char
        Nat
        char_cursor_ops))
    char_cursor_peek_has_remaining
    (and_intro
      (CursorAdvanceProgress
        (List Char)
        Char
        Nat
        char_cursor_ops)
      (CursorEndValid
        (List Char)
        Char
        Nat
        char_cursor_ops)
      char_cursor_advance_progress
      char_cursor_end_valid)
```

The codec prefix, fuel, and constructor round-trip proofs follow the definition
so their statements are checked against the exact public operations above.

## 5. Design notes

The accepted number domain is the full unbounded integer subset of JSON's
number grammar, in canonical decimal form. Fractional and exponential forms are
the scoped residual: the landed numeric package can parse integers but cannot
structurally print an opaque `Int`, and this package does not invent an
`Int` destructor or a second numeric authority.

Strings and object keys are accepted by the round-trip law when their scalar
lists contain no control character, quotation mark, or reverse solidus. This
first codec keeps escaping outside the proved core rather than silently
emitting invalid JSON. The residual is a reusable lawful JSON-string escape
codec over `List Char`.

Objects are represented as ordered association lists. Decoding rejects a
duplicate key through the canonical `DecEq String`; the round-trip domain
therefore requires unique keys.

## 6. References

- [RFC 8259](https://www.rfc-editor.org/rfc/rfc8259) — the JSON data model and
  grammar whose integer subset is implemented here.
- [ECMA-404][ecma-404] — the independent JSON syntax description.

[ecma-404]: https://ecma-international.org/publications-and-standards/standards/ecma-404/

## 7. Trust & derivation

This file introduces zero axioms, postulates, primitives, effects, or holes.
The string constructor's round trip is proved relative to the existing
`string_to_list_char_retraction` certificate from
`Data/Text/StringBijection.ken.md`; null, Boolean, integer, array, and object
cases add no inherited string assumption except object keys.

The implementation derives from transparent `List` operations, the canonical
`DecEq Char` and `DecEq String` dictionaries, and
`Capability.Parsing.Cursor`/`Decoder`. The acceptance suite elaborates this
whole entry, checks its public globals and trust delta, exercises typed error
and fuel boundaries, and evaluates all six constructor families.
