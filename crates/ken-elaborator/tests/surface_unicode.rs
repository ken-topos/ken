//! SURF-1 D3 Unicode surface acceptance.

use ken_elaborator::{format::canonical_unicode, lexer::Lexer, ElabEnv};
use ken_kernel::Decl;

fn token_kinds(src: &str) -> Vec<ken_elaborator::lexer::Token> {
    Lexer::lex(src)
        .expect("source must lex")
        .into_iter()
        .map(|(tok, _)| tok)
        .collect()
}

fn transparent_debug(src: &str) -> (String, String) {
    let mut env = ElabEnv::new().expect("base env");
    let result = env.elaborate_decl_v1(src).expect("decl elaborates");
    match env.env.lookup(result.def_id) {
        Some(Decl::Transparent { ty, body, .. }) => (format!("{ty:?}"), format!("{body:?}")),
        other => panic!("expected transparent decl, got {other:?}"),
    }
}

#[test]
fn surf1_d3_unicode_and_ascii_lex_to_same_tokens() {
    let ascii = "fn surf1_u (A : Type) (x : A) : A -> A = \\y . y\n\
                 fn surf1_m (b : Bool) : Bool = match b { True |-> False ; False |-> True }";
    let unicode = "fn surf1_u (A : Type) (x : A) : A → A = λy . y\n\
                   fn surf1_m (b : Bool) : Bool = match b { True ↦ False ; False ↦ True }";

    assert_eq!(token_kinds(ascii), token_kinds(unicode));
    assert_eq!(
        token_kinds("Omega Sigma Pi forall exists not level l === <= >= /= /\\ \\/ <: ><"),
        token_kinds("Ω Σ Π ∀ ∃ ¬ ℓ ℓ ≡ ≤ ≥ ≠ ∧ ∨ ⊑ ×")
    );
    assert_ne!(token_kinds("in"), token_kinds("∈"));
}

#[test]
fn surf1_d3_formatter_emits_canonical_unicode() {
    let src = "fn f (l : Int) (level : Int) (not : Int) : Int -> Int = \\x . x\n\
fn invert (x : Bool) : Bool = match x { True |-> False ; False |-> True }\n\
foreign call : Int -> Int = \"keep -> and not in level\" \"lib|->l\" [pure]\n\
-- keep -> and => not in level in comments\n";
    let formatted = canonical_unicode(src);
    assert!(formatted.contains("fn f (l : Int) (level : Int) (not : Int) : Int → Int = λx . x"));
    assert!(formatted.contains("match x { True ↦ False ; False ↦ True }"));
    assert!(formatted.contains("-- keep -> and => not in level in comments"));
    assert!(formatted.contains("\"keep -> and not in level\" \"lib|->l\""));
}

#[test]
fn surf1_d3_rejects_unbounded_unicode_identifiers() {
    for src in [
        "fn surf1_bad (а : Type) : Type = Type",  // Cyrillic small a
        "fn surf1_bad (xа : Type) : Type = Type", // ASCII start, Cyrillic continuation
        "fn Ｔ : Type = Type",                    // fullwidth capital T
    ] {
        assert!(
            Lexer::lex(src).is_err(),
            "unbounded Unicode identifier accepted: {src}"
        );
    }
}

#[test]
fn surf1_d3_membership_glyph_is_not_let_delimiter() {
    let mut env = ElabEnv::new().expect("base env");
    env.elaborate_expr(
        "surf1_d3_membership_glyph_is_not_let_delimiter",
        "let x = 1 in x",
    )
    .expect("ASCII keyword in remains the let delimiter");
    assert!(
        env.elaborate_expr(
            "surf1_d3_membership_glyph_is_not_let_delimiter",
            "let x = 1 ∈ x",
        )
        .is_err(),
        "membership glyph must not parse as keyword `in`"
    );
}

#[test]
fn surf1_d3_unicode_and_ascii_elaborate_identically() {
    let ascii = "fn surf1_id (A : Type) (x : A) : A -> A = \\y . y";
    let unicode = "fn surf1_id (A : Type) (x : A) : A → A = λy . y";
    assert_eq!(transparent_debug(ascii), transparent_debug(unicode));
}
