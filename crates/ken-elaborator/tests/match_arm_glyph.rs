//! Match-arm `↦` / `|->` surface acceptance.

use ken_elaborator::{
    format::canonical_unicode,
    lexer::{Lexer, Token},
    parser::{parse_decls, parse_expr},
};

fn token_kinds(src: &str) -> Vec<Token> {
    Lexer::lex(src)
        .expect("source must lex")
        .into_iter()
        .map(|(token, _)| token)
        .collect()
}

#[test]
fn match_arm_spellings_share_one_token() {
    for spelling in ["|->", "↦"] {
        assert_eq!(token_kinds(spelling), vec![Token::MapsTo, Token::Eof]);
    }
    assert!(Lexer::lex("=>").is_err());
    assert!(Lexer::lex("⇒").is_err());
    assert_eq!(token_kinds("|"), vec![Token::Pipe, Token::Eof]);
}

#[test]
fn match_arm_spellings_parse_and_retired_spellings_reject() {
    for separator in ["|->", "↦"] {
        let source = format!("match Zero {{ Zero {separator} v }}");
        parse_expr(&source).unwrap_or_else(|error| {
            panic!("match arm spelling {separator:?} did not parse: {error}")
        });
    }

    for separator in ["=>", "⇒"] {
        let source = format!("match Zero {{ Zero {separator} v }}");
        assert!(
            parse_expr(&source).is_err(),
            "retired match arm spelling {separator:?} still parsed"
        );
    }

    parse_expr("match Zero { Zero|->v }").expect("adjacent ASCII spelling parses");
    parse_expr("match Zero { Zero |-> v }").expect("spaced ASCII spelling parses");
    parse_expr("match Zero { Zero ↦ v }").expect("Unicode spelling parses");
}

#[test]
fn pipe_grammar_remains_distinct_from_ascii_maps_to() {
    parse_decls("data X = A | B").expect("data constructor pipe still parses");
    parse_decls("def R = { x : Int | Equal Int x x }").expect("refinement pipe still parses");
    parse_decls("proc f (x : Int) : Int visits [Console | e] = x")
        .expect("open effect-row pipe still parses");
}

#[test]
fn formatter_canonicalizes_ascii_maps_to_before_arrow() {
    let formatted = canonical_unicode("match x { A |-> B } ; A -> B");
    assert_eq!(formatted, "match x { A ↦ B } ; A → B");
    assert_eq!(canonical_unicode("=> ⇒"), "=> ⇒");

    assert_eq!(
        canonical_unicode("-- keep |->\n\"keep |->\""),
        "-- keep |->\n\"keep |->\""
    );
}
