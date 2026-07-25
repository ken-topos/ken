//! KW-THEOREM AC-4: `theorem` is the sole standalone checked-theorem keyword.

use ken_elaborator::layout::format_ken;
use ken_elaborator::lexer::{Lexer, Token};
use ken_elaborator::{ElabEnv, ElabError, Span};

const THEOREM_SOURCE: &str = "theorem kw_theorem_refl (x : Bool) : Equal Bool x x = Refl";
// AC-2(d): this retired declaration head is an intentional control residual.
const RETIRED_SPELLING_SOURCE: &str = "lemma kw_theorem_refl (x : Bool) : Equal Bool x x = Refl";

// MEASURED: token identity, full elaboration, canonical formatting, and the
// retired declaration head's exact parse error.
// CLAIMED: `theorem` is the sole standalone checked-theorem keyword.
// THE GAP: token identity alone cannot prove parser, elaborator, or formatter
// wiring, so the same harness exercises all four layers.
#[test]
fn theorem_elaborates_and_formats_while_retired_spelling_is_an_identifier() {
    let tokens = Lexer::lex("theorem lemma")
        .expect("the new keyword and the retired spelling must lex")
        .into_iter()
        .map(|(token, _)| token)
        .collect::<Vec<_>>();
    assert_eq!(
        tokens,
        vec![
            Token::KwTheorem,
            Token::Ident("lemma".to_owned()),
            Token::Eof,
        ],
        "`theorem` must be reserved while `lemma` returns to ordinary identifier lexing"
    );

    let mut env = ElabEnv::new().expect("base environment must initialize");
    env.elaborate_file(THEOREM_SOURCE)
        .expect("theorem source must fully elaborate");
    assert!(
        env.globals.contains_key("kw_theorem_refl"),
        "full elaboration must register the checked theorem"
    );
    assert_eq!(
        format_ken(THEOREM_SOURCE).expect("theorem source must format"),
        "theorem kw_theorem_refl (x : Bool) : Equal Bool x x = Refl\n"
    );

    let error = env
        .elaborate_file(RETIRED_SPELLING_SOURCE)
        .expect_err("the retired declaration spelling must not remain an alias");
    match error {
        ElabError::ParseError { msg, span } => {
            assert_eq!(
                msg,
                "expected 'view', 'const', 'fn', 'proc', 'let', 'prove', 'prop', \
                 'theorem', 'proof', 'law', 'data', 'def', 'foreign', 'temporal', \
                 'class', 'instance', 'derive', 'module', 'import', 'export', \
                 'pub', 'program', 'package', or 'space proc', found Ident(\"lemma\")"
            );
            assert_eq!(span, Span::new(0, 5));
        }
        other => panic!("expected the exact retired-spelling ParseError, got {other:?}"),
    }
}
