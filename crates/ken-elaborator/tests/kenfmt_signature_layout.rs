//! WP #58 batch 1 — 96-column, horizontal-first declaration layout.

use ken_elaborator::layout::{display_width, format_ken, CANONICAL_WIDTH};
use ken_elaborator::lossless::parse_lossless;

fn check(source: &str, expected: &str) {
    let formatted = format_ken(source).expect("fixture must format");
    assert_eq!(formatted, expected);
    assert_eq!(format_ken(&formatted).unwrap(), formatted);
    assert!(
        formatted
            .lines()
            .all(|line| display_width(line) <= CANONICAL_WIDTH),
        "fixture exceeded {CANONICAL_WIDTH} columns:\n{formatted}"
    );
}

fn token_shape(source: &str) -> Vec<String> {
    parse_lossless(source)
        .expect("fixture must parse")
        .tokens()
        .iter()
        .map(|token| format!("{:?}", token.kind))
        .collect()
}

#[test]
fn r1_signature_fit_and_split_ladder_are_exact() {
    check(
        "fn tiny (x : Nat) : Nat = x",
        "fn tiny (x : Nat) : Nat = x\n",
    );

    check(
        "fn from_list_acc (k : Type) (v : Type) (leq : k -> k -> Bool) (xs : List (Pair k v)) (acc : Tree k v) : Tree k v = acc",
        "fn from_list_acc\n      (k : Type) (v : Type) (leq : k → k → Bool) (xs : List (Pair k v)) (acc : Tree k v)\n    : Tree k v =\n  acc\n",
    );

    check(
        "fn wide (x : Nat) : SomeLongReturnConstructorName AlphaArgumentName BetaArgumentName GammaArgumentName DeltaArgumentName = x",
        "fn wide\n      (x : Nat)\n    : SomeLongReturnConstructorName\n      AlphaArgumentName\n      BetaArgumentName\n      GammaArgumentName\n      DeltaArgumentName =\n  x\n",
    );
}

#[test]
fn r2_r3_r4_r5_prefer_horizontal_when_the_construct_fits() {
    check(
        "lemma total : Claim = \\x.\\y.proof eq_true_of_or for bool_or",
        "lemma total : Claim = λx. λy. proof eq_true_of_or for bool_or\n",
    );
    check(
        "fn wrapped (x : Nat) (y : Nat) : Bool = decide (leq_nat x y)",
        "fn wrapped (x : Nat) (y : Nat) : Bool = decide (leq_nat x y)\n",
    );
    check(
        "data OrdResult = Lt | Eq | Gt",
        "data OrdResult = Lt | Eq | Gt\n",
    );
    check(
        "fn sub_case (x : Nat) (n : Nat) : Nat = match x { Zero |-> n; Suc m |-> sub n m }",
        "fn sub_case (x : Nat) (n : Nat) : Nat =\n  match x {\n    Zero ↦ n;\n    Suc m ↦ sub n m\n  }\n",
    );
}

#[test]
fn layout_changes_only_whitespace_and_token_spelling_by_existing_token_kind() {
    let source = "module M { fn keep (x : Nat) : Nat = (step x); }\ndata Choice = Left | Right\nfn choose (x : Choice) : Nat = match x { Left |-> apply a b; Right |-> apply b a; }\n";
    let formatted = format_ken(source).unwrap();
    assert_eq!(token_shape(source), token_shape(&formatted));
    assert!(formatted.contains("apply a b"));
    assert!(formatted.contains("apply b a"));
    assert_eq!(format_ken(&formatted).unwrap(), formatted);
}
