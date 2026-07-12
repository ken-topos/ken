//! Canonical token spelling (`31 §1b`, `31 §1d`).
//!
//! Canonicalization consumes B1's lossless token/trivia partition.  It never
//! scans source bytes for alias-shaped substrings: notation is selected from
//! the parsed token kind, while every other token and all trivia retain their
//! original source lexeme.

use crate::lexer::Token;
use crate::lossless::{parse_lossless, FormattableSource, SourcePieceKind};

/// The blessed spelling for an unambiguous notation token kind.
fn canonical_token_spelling(token: &Token) -> Option<&'static str> {
    match token {
        Token::Arrow => Some("→"),
        Token::MapsTo => Some("↦"),
        Token::Lambda => Some("λ"),
        Token::PropEq => Some("≡"),
        Token::Le => Some("≤"),
        Token::Ge => Some("≥"),
        Token::Ne => Some("≠"),
        Token::And => Some("∧"),
        Token::Or => Some("∨"),
        Token::FlowsTo => Some("⊑"),
        Token::Times => Some("×"),
        _ => None,
    }
}

/// Canonicalize notation spellings over an already-parsed B1 source stream.
///
/// Layout, identifiers, keywords, literals, comments, foreign string payloads,
/// and temporal formula bodies are replayed from their original source spans.
pub fn canonicalize_tokens(source: &dyn FormattableSource) -> String {
    let mut out = String::with_capacity(source.source().len());
    let mut temporal_brace_depth: Option<usize> = None;
    let mut temporal_pending_brace = false;

    for piece in source.pieces() {
        let lexeme = &source.source()[piece.span.start..piece.span.end];
        let SourcePieceKind::Token(token_index) = piece.kind else {
            out.push_str(lexeme);
            continue;
        };
        let token = &source.tokens()[token_index].kind;

        // Temporal formula text is a protected payload.  Its braces and every
        // token between them are replayed verbatim; the declaration keyword
        // and name remain ordinary stored spellings as well.
        if temporal_pending_brace {
            out.push_str(lexeme);
            if matches!(token, Token::LBrace) {
                temporal_pending_brace = false;
                temporal_brace_depth = Some(1);
            }
            continue;
        }
        if let Some(depth) = temporal_brace_depth.as_mut() {
            out.push_str(lexeme);
            match token {
                Token::LBrace => *depth += 1,
                Token::RBrace if *depth == 1 => temporal_brace_depth = None,
                Token::RBrace => *depth -= 1,
                _ => {}
            }
            continue;
        }
        if matches!(token, Token::KwTemporal) {
            temporal_pending_brace = true;
            out.push_str(lexeme);
            continue;
        }

        if let Some(canonical) = canonical_token_spelling(token) {
            out.push_str(canonical);
        } else {
            out.push_str(lexeme);
        }
    }

    out
}

/// Normalize notation in a syntactically valid Ken unit.
///
/// The stable legacy signature is retained for callers.  Invalid fragments
/// have no parsed token roles, so they are returned byte-for-byte rather than
/// being subjected to a raw-text fallback.
pub fn canonical_unicode(src: &str) -> String {
    match parse_lossless(src) {
        Ok(source) => canonicalize_tokens(source.as_ref()),
        Err(_) => src.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_kind_table_is_exhaustive_for_current_notation_variants() {
        let cases = [
            (Token::Arrow, "→"),
            (Token::MapsTo, "↦"),
            (Token::Lambda, "λ"),
            (Token::PropEq, "≡"),
            (Token::Le, "≤"),
            (Token::Ge, "≥"),
            (Token::Ne, "≠"),
            (Token::And, "∧"),
            (Token::Or, "∨"),
            (Token::FlowsTo, "⊑"),
            (Token::Times, "×"),
        ];
        for (token, spelling) in cases {
            assert_eq!(canonical_token_spelling(&token), Some(spelling));
        }
    }
}
