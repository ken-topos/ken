//! WP #58 batch 1 — 96-column, horizontal-first declaration layout.

use ken_elaborator::layout::{display_width, format_ken, CANONICAL_WIDTH};
use ken_elaborator::literate::format_ken_md;
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

fn ast_shape(source: &str) -> String {
    let parsed = parse_lossless(source).expect("source must parse");
    erase_debug_spans(format!("{:?}", parsed.typed_decls()))
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

    check(
        "proof transitive_witness for leq_nat (x : Nat) : (y : Nat) -> Equal Bool (leq_nat x y) True -> (z : Nat) -> Equal Bool (leq_nat y z) True -> Equal Bool (leq_nat x z) True = x",
        "proof transitive_witness for leq_nat\n      (x : Nat)\n    : (y : Nat)\n      → Equal Bool (leq_nat x y) True\n      → (z : Nat)\n      → Equal Bool (leq_nat y z) True\n      → Equal Bool (leq_nat x z) True =\n  x\n",
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

#[test]
fn r3_r5_nested_applications_fit_recursively_inside_a_broken_parent() {
    let source = "lemma nested : Claim = outer (\\r. Equal OrdResult (compare a da (pair_fst a b x) (pair_fst a b y)) r -> Equal Bool (ord_leq_at a da (pair_fst a b x) (pair_fst a b y)) True) tail";
    let expected = "lemma nested : Claim =\n  outer\n    (λr.\n      Equal OrdResult (compare a da (pair_fst a b x) (pair_fst a b y)) r\n      → Equal Bool (ord_leq_at a da (pair_fst a b x) (pair_fst a b y)) True)\n    tail\n";

    check(source, expected);
    assert_eq!(token_shape(source), token_shape(expected));
    assert_eq!(ast_shape(source), ast_shape(expected));
}

#[test]
fn batch3_stage1_breaks_high_and_keeps_each_fitting_child_horizontal() {
    let fusion = "class Functor (f : Type -> Type) { fusion_law : (a : Type) -> (b : Type) -> (c : Type) -> (g : b -> c) -> (h : a -> b) -> (x : f a) -> Equal (f c) (map a c (comp a b c g h) x) (map b c g (map a b h x)) }";
    let fusion_expected = "class Functor (f : Type → Type) {\n  fusion_law :\n    (a : Type)\n    → (b : Type)\n    → (c : Type)\n    → (g : b → c)\n    → (h : a → b)\n    → (x : f a)\n    → Equal (f c) (map a c (comp a b c g h) x) (map b c g (map a b h x))\n}\n";
    check(fusion, fusion_expected);
    assert_eq!(token_shape(fusion), token_shape(fusion_expected));
    assert_eq!(ast_shape(fusion), ast_shape(fusion_expected));
    assert!(fusion_expected.contains("→ (g : b → c)"));
    assert!(!fusion_expected.contains("(g : b\n"));

    let assoc = "proof assoc for list_append (a : Type) (xs : List a) (ys : List a) (zs : List a) : Equal (List a) (list_append a (list_append a xs ys) zs) (list_append a xs (list_append a ys zs)) = Refl";
    let assoc_expected = "proof assoc for list_append\n      (a : Type) (xs : List a) (ys : List a) (zs : List a)\n    : Equal\n        (List a)\n        (list_append a (list_append a xs ys) zs)\n        (list_append a xs (list_append a ys zs)) =\n  Refl\n";
    check(assoc, assoc_expected);
    assert_eq!(token_shape(assoc), token_shape(assoc_expected));
    assert_eq!(ast_shape(assoc), ast_shape(assoc_expected));

    let lawful_functors = include_str!("../../../catalog/packages/Core/LawfulFunctors.ken.md");
    assert_eq!(format_ken_md(lawful_functors).unwrap(), lawful_functors);
    assert!(lawful_functors.contains(
        "  fusion_law :\n    (a : Type)\n    → (b : Type)\n    → (c : Type)\n    → (g : b → c)"
    ));

    let ord_nat = include_str!("../../../catalog/packages/Core/OrdNat.ken.md");
    assert_eq!(
        format_ken_md(ord_nat).unwrap(),
        ord_nat,
        "batch-1 OrdNat rendering must remain byte-identical"
    );
}
