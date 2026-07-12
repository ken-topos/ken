//! WP B2 — token-kind canonicalization over B1's lossless stream.

use ken_elaborator::format::{canonical_unicode, canonicalize_tokens};
use ken_elaborator::lexer::Token;
use ken_elaborator::lossless::{
    parse_lossless, CommentAttachment, FormattableSource, SourcePiece, SourcePieceKind,
    SourceToken, Trivia,
};
use ken_elaborator::{Decl, Span};

struct TokenStreamFixture {
    source: String,
    tokens: Vec<SourceToken>,
    pieces: Vec<SourcePiece>,
}

impl FormattableSource for TokenStreamFixture {
    fn typed_decls(&self) -> &[Decl] {
        &[]
    }

    fn source(&self) -> &str {
        &self.source
    }

    fn tokens(&self) -> &[SourceToken] {
        &self.tokens
    }

    fn trivia(&self) -> &[Trivia] {
        &[]
    }

    fn pieces(&self) -> &[SourcePiece] {
        &self.pieces
    }

    fn comment_attachments(&self) -> &[CommentAttachment] {
        &[]
    }
}

fn token_stream(tokens: Vec<(Token, &str)>) -> TokenStreamFixture {
    let mut source = String::new();
    let mut source_tokens = Vec::new();
    let mut pieces = Vec::new();
    for (index, (kind, lexeme)) in tokens.into_iter().enumerate() {
        let start = source.len();
        source.push_str(lexeme);
        let span = Span::new(start, source.len());
        source_tokens.push(SourceToken {
            kind,
            span: span.clone(),
        });
        pieces.push(SourcePiece {
            kind: SourcePieceKind::Token(index),
            span,
        });
    }
    TokenStreamFixture {
        source,
        tokens: source_tokens,
        pieces,
    }
}

#[test]
fn ac1_dispatches_on_kind_and_never_identifier_bytes() {
    let source = token_stream(vec![
        (Token::Ident("l".into()), "l"),
        (Token::Ident("level".into()), "level"),
        (Token::KwIn, "in"),
        (Token::Ident("not".into()), "not"),
        (Token::Arrow, "->"),
        (Token::MapsTo, "|->"),
        (Token::Lambda, "\\"),
        (Token::And, "/\\"),
        (Token::Or, "\\/"),
    ]);

    assert_eq!(
        canonicalize_tokens(&source),
        "llevelinnot→↦λ∧∨",
        "the same alias-shaped bytes must differ when their parsed kinds differ"
    );
}

#[test]
fn ac2_and_ac4_preserve_identifiers_and_layout_while_spelling_notation() {
    let input = "fn keep_l (l : Int) (level : Int) (not : Int) : Int -> Int = \\x . x  -- keep -> |-> l level in not\n\
fn keep_level_glyph (ℓ : Int) : Int = ℓ\n\
fn keep_words (forall : Sigma) : Sigma = forall\n";
    let expected = "fn keep_l (l : Int) (level : Int) (not : Int) : Int → Int = λx . x  -- keep -> |-> l level in not\n\
fn keep_level_glyph (ℓ : Int) : Int = ℓ\n\
fn keep_words (forall : Sigma) : Sigma = forall\n";

    assert_eq!(canonical_unicode(input), expected);
}

#[test]
fn ac3_protected_payloads_are_byte_identical() {
    let input = "foreign call : Int -> Int = \"symbol->not\" \"lib|->level\" [pure]\n\
temporal response { not eventually alias_arrow }\n\
-- doc-ish comment: -> |-> \\ in not l level\n";
    let expected = "foreign call : Int → Int = \"symbol->not\" \"lib|->level\" [pure]\n\
temporal response { not eventually alias_arrow }\n\
-- doc-ish comment: -> |-> \\ in not l level\n";

    let parsed = parse_lossless(input).expect("protected-payload fixture parses");
    assert_eq!(canonicalize_tokens(parsed.as_ref()), expected);
}

#[test]
fn ac6_function_and_match_arrows_are_distinct_wp_s_roles() {
    let input = "fn choose (x : Bool) : Bool -> Bool = \\y . match x {\n\
  True |-> y ; False |-> x\n\
}\n";
    let expected = "fn choose (x : Bool) : Bool → Bool = λy . match x {\n\
  True ↦ y ; False ↦ x\n\
}\n";

    assert_eq!(canonical_unicode(input), expected);
}

#[test]
fn ac5_invalid_fragments_have_no_raw_scanner_fallback() {
    let invalid = "not source -> |-> level";
    assert_eq!(canonical_unicode(invalid), invalid);
}
