# String/List-Char bijection certificate

`String` is opaque, so its total conversion to `List Char` does not by itself
prove extensionality. This prerequisite names the single irreducible
certificate at the conversion layer and derives the injectivity form that
lawful String keys consume.

## 1. Retraction certificate

The assumption is explicit and unique. It is not minted in a Text package.

```ken
lemma string_to_list_char_retraction
    : (text : String) → Equal String (list_char_to_string (string_to_list_char text)) text =
  Axiom

lemma string_to_list_char_injective
      (left : String)
      (right : String)
      (same_chars : Equal (List Char) (string_to_list_char left) (string_to_list_char right))
    : Equal String left right =
  let
    left_chars : List Char = string_to_list_char left;
    right_chars : List Char = string_to_list_char right;
    left_round_trip : String = list_char_to_string left_chars;
    right_round_trip : String = list_char_to_string right_chars;
    left_retracts : Equal String left left_round_trip =
      sym String left_round_trip left (string_to_list_char_retraction left);
    mapped_chars : Equal String left_round_trip right_round_trip =
      cong (List Char) String left_chars right_chars list_char_to_string same_chars
  in
    trans
      String
      left
      left_round_trip
      right
      left_retracts
      (trans
        String
        left_round_trip
        right_round_trip
        right
        mapped_chars
        (string_to_list_char_retraction right))
```

## 2. Trust and derivation

`string_to_list_char_retraction` is the one named postulate selected by the
operator. `string_to_list_char_injective` is a transparent consequence using
only symmetry, transitivity, and congruence. No comparator primitive or second
certificate is introduced.
