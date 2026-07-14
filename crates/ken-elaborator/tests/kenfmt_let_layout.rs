//! LET-1 — exact readable layout for local binding chains.

use ken_elaborator::layout::{display_width, format_ken, CANONICAL_WIDTH};
use ken_elaborator::lossless::parse_lossless;
use ken_elaborator::resolve::resolve_decls;
use ken_elaborator::ElabEnv;

fn ast_shape(source: &str) -> String {
    let parsed = parse_lossless(source).expect("fixture must parse");
    erase_debug_spans(format!("{:?}", parsed.typed_decls()))
}

fn token_shape(source: &str) -> Vec<String> {
    parse_lossless(source)
        .expect("fixture must parse")
        .tokens()
        .iter()
        .map(|token| format!("{:?}", token.kind))
        .collect()
}

fn resolved_shape(source: &str) -> String {
    let parsed = parse_lossless(source).expect("fixture must parse");
    let resolved = resolve_decls(parsed.typed_decls()).expect("fixture must resolve");
    erase_debug_spans(format!("{resolved:?}"))
}

fn erase_debug_spans(mut debug: String) -> String {
    const PREFIX: &str = "Span { start: ";
    while let Some(start) = debug.find(PREFIX) {
        let Some(relative_end) = debug[start..].find(" }") else {
            break;
        };
        debug.replace_range(start..start + relative_end + 2, "Span");
    }
    debug
}

fn check(source: &str, expected: &str) {
    let formatted = format_ken(source).expect("fixture must format");
    assert_eq!(formatted, expected);
    if token_shape(source) == token_shape(&formatted) {
        assert_eq!(ast_shape(source), ast_shape(&formatted));
    } else {
        assert_eq!(resolved_shape(source), resolved_shape(&formatted));
    }
    assert_eq!(format_ken(&formatted).unwrap(), formatted);
    assert!(
        formatted
            .lines()
            .all(|line| display_width(line) <= CANONICAL_WIDTH),
        "fixture exceeded {CANONICAL_WIDTH} columns:\n{formatted}"
    );
}

fn check_trusted_base_preserved(source: &str, formatted: &str) {
    let mut original = ElabEnv::new().unwrap();
    let mut canonical = ElabEnv::new().unwrap();
    original
        .elaborate_file(source)
        .expect("original fixture must elaborate");
    canonical
        .elaborate_file(formatted)
        .expect("formatted fixture must elaborate");
    assert_eq!(original.env.trusted_base(), canonical.env.trusted_base());
}

#[test]
fn ac1_ac2_single_let_fit_and_structural_break_are_exact() {
    let tiny = "const tiny : Nat = let x : Nat = Zero in x";
    let tiny_expected = "const tiny : Nat = let x : Nat = Zero in x\n";
    check(tiny, tiny_expected);
    check_trusted_base_preserved(tiny, tiny_expected);

    check(
        "const selected : Nat = let value : Nat = match subject { Zero |-> first; Suc n |-> n } in value",
        "const selected : Nat =\n  let value : Nat =\n    match subject {\n      Zero ↦ first;\n      Suc n ↦ n\n    }\n  in\n    value\n",
    );
}

#[test]
fn ac3_ac4_nested_simple_bindings_fit_or_break_as_one_typed_chain() {
    check(
        "const tiny_chain : Nat = let first = Zero in let second = first in second",
        "const tiny_chain : Nat = let first = Zero; second = first in second\n",
    );
    check(
        "const grouped : Nat = let first = Zero in (let second = first in second)",
        "const grouped : Nat = let first = Zero; second = first in second\n",
    );

    let source = concat!(
        "const chars : List Char = ",
        "let left_chars : List Char = string_to_list_char left in ",
        "let right_chars : List Char = string_to_list_char right in ",
        "let joined_chars : List Char = append Char left_chars right_chars in ",
        "joined_chars"
    );
    let expected = concat!(
        "const chars : List Char =\n",
        "  let\n",
        "    left_chars : List Char = string_to_list_char left;\n",
        "    right_chars : List Char = string_to_list_char right;\n",
        "    joined_chars : List Char = append Char left_chars right_chars\n",
        "  in\n",
        "    joined_chars\n"
    );
    check(source, expected);
    assert!(!expected.contains("List\n"));
    assert!(!expected.contains("\nChar"));
}

#[test]
fn ac5_nested_bindings_in_a_match_arm_are_structurally_indented() {
    let source = concat!(
        "fn choose (choice : Choice) : Nat = match choice { ",
        "Left |-> let first_stage = transform_first initial_value in ",
        "let second_stage = transform_second first_stage in second_stage; ",
        "Right |-> fallback }"
    );
    let expected = concat!(
        "fn choose (choice : Choice) : Nat =\n",
        "  match choice {\n",
        "    Left ↦\n",
        "      let\n",
        "        first_stage = transform_first initial_value;\n",
        "        second_stage = transform_second first_stage\n",
        "      in\n",
        "        second_stage;\n",
        "    Right ↦ fallback\n",
        "  }\n"
    );
    check(source, expected);
}

#[test]
fn ac6_ac7_worked_six_binding_proof_has_an_exact_readable_fixed_point() {
    let source = r#"lemma string_to_list_char_injective_with_lets
      (left : String)
      (right : String)
      (same_chars : Equal (List Char) (string_to_list_char left) (string_to_list_char right))
    : Equal String left right =
  let left_chars : List Char = string_to_list_char left in
  let right_chars : List Char = string_to_list_char right in
  let left_round_trip : String = list_char_to_string left_chars in
  let right_round_trip : String = list_char_to_string right_chars in
  let left_retracts : Equal String left left_round_trip =
    sym String left_round_trip left (string_to_list_char_retraction left)
  in
  let mapped_chars : Equal String left_round_trip right_round_trip =
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
"#;
    let expected = concat!(
        "lemma string_to_list_char_injective_with_lets\n",
        "      (left : String)\n",
        "      (right : String)\n",
        "      (same_chars : Equal (List Char) (string_to_list_char left) (string_to_list_char right))\n",
        "    : Equal String left right =\n",
        "  let\n",
        "    left_chars : List Char = string_to_list_char left;\n",
        "    right_chars : List Char = string_to_list_char right;\n",
        "    left_round_trip : String = list_char_to_string left_chars;\n",
        "    right_round_trip : String = list_char_to_string right_chars;\n",
        "    left_retracts : Equal String left left_round_trip =\n",
        "      sym String left_round_trip left (string_to_list_char_retraction left);\n",
        "    mapped_chars : Equal String left_round_trip right_round_trip =\n",
        "      cong (List Char) String left_chars right_chars list_char_to_string same_chars\n",
        "  in\n",
        "    trans\n",
        "      String\n",
        "      left\n",
        "      left_round_trip\n",
        "      right\n",
        "      left_retracts\n",
        "      (trans\n",
        "        String\n",
        "        left_round_trip\n",
        "        right_round_trip\n",
        "        right\n",
        "        mapped_chars\n",
        "        (string_to_list_char_retraction right))\n"
    );
    check(source, expected);
}

#[test]
fn let4_comments_stay_owned_and_shadowing_starts_a_new_segment() {
    let single = r#"const single_comment : Nat =
  let
    -- bound value
    x = Zero
  in
    -- returned value
    x
"#;
    check(single, single);

    let commented = r#"const commented : Nat =
  let
    -- first binding
    x = Zero
  in
    let
      -- second binding
      y = x
    in
      -- final body
      y
"#;
    let expected = r#"const commented : Nat =
  let
    -- first binding
    x = Zero;
    -- second binding
    y = x
  in
    -- final body
    y
"#;
    check(commented, expected);

    check(
        "const shadowed : Nat = let x = Zero in let y = x in let x = Suc y in x",
        concat!(
            "const shadowed : Nat =\n",
            "  let\n",
            "    x = Zero;\n",
            "    y = x\n",
            "  in\n",
            "    let x = Suc y in x\n"
        ),
    );
}
