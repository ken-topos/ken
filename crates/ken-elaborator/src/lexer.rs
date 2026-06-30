//! V0/V1 lexer (`31 §8`, `21 §6.1`).
//!
//! Recognises the token subset for G1 (V0) plus V1 spec-annotation keywords:
//! `requires`, `ensures`, `prove`, `law`, `old`, `space`, and punctuation
//! `{ } |`. Whitespace and `-- …` line comments are skipped.

use crate::error::{ElabError, Span};

/// A V0/V1 token.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    // V0 keywords
    KwView,
    KwLet,
    KwIn,
    KwType,
    // V1 keywords
    KwRequires,
    KwEnsures,
    KwProve,
    KwLaw,
    KwOld,
    KwSpace,
    // V0 punctuation
    LParen,
    RParen,
    Colon,
    Eq,
    Dot,
    Arrow,
    Lambda,
    Semicolon,
    // V1 punctuation
    LBrace,
    RBrace,
    Pipe,
    Ident(String),   // lowercase-initial term variable
    ConId(String),   // uppercase-initial base type / constructor
    Nat(u32),        // level digit
    Eof,
}

pub struct Lexer<'s> {
    src: &'s str,
    pos: usize,
}

impl<'s> Lexer<'s> {
    pub fn new(src: &'s str) -> Self {
        Self { src, pos: 0 }
    }

    fn cur(&self) -> Option<char> {
        self.src[self.pos..].chars().next()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.cur()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    fn skip_ws_comments(&mut self) {
        loop {
            while self.cur().map(|c| c.is_whitespace()).unwrap_or(false) {
                self.advance();
            }
            if self.src[self.pos..].starts_with("--") {
                while self.cur().map(|c| c != '\n').unwrap_or(false) {
                    self.advance();
                }
            } else {
                break;
            }
        }
    }

    fn is_ident_continue(c: char) -> bool {
        c.is_alphanumeric() || c == '_' || c == '\''
    }

    pub fn next_token(&mut self) -> Result<(Token, Span), ElabError> {
        self.skip_ws_comments();
        let start = self.pos;

        let c = match self.cur() {
            None => return Ok((Token::Eof, Span::new(start, start))),
            Some(c) => c,
        };

        // Single-char punctuation
        match c {
            '(' => {
                self.advance();
                return Ok((Token::LParen, Span::new(start, self.pos)));
            }
            ')' => {
                self.advance();
                return Ok((Token::RParen, Span::new(start, self.pos)));
            }
            '{' => {
                self.advance();
                return Ok((Token::LBrace, Span::new(start, self.pos)));
            }
            '}' => {
                self.advance();
                return Ok((Token::RBrace, Span::new(start, self.pos)));
            }
            '|' => {
                self.advance();
                return Ok((Token::Pipe, Span::new(start, self.pos)));
            }
            ';' => {
                self.advance();
                return Ok((Token::Semicolon, Span::new(start, self.pos)));
            }
            ':' => {
                self.advance();
                return Ok((Token::Colon, Span::new(start, self.pos)));
            }
            '=' => {
                self.advance();
                return Ok((Token::Eq, Span::new(start, self.pos)));
            }
            '.' => {
                self.advance();
                return Ok((Token::Dot, Span::new(start, self.pos)));
            }
            '\\' | 'λ' => {
                self.advance();
                return Ok((Token::Lambda, Span::new(start, self.pos)));
            }
            '→' => {
                self.advance();
                return Ok((Token::Arrow, Span::new(start, self.pos)));
            }
            '-' => {
                self.advance();
                if self.cur() == Some('>') {
                    self.advance();
                    return Ok((Token::Arrow, Span::new(start, self.pos)));
                }
                return Err(ElabError::ParseError {
                    msg: "unexpected '-' (did you mean '->'?)".to_string(),
                    span: Span::new(start, self.pos),
                });
            }
            _ => {}
        }

        // Level digits (bare non-negative integers)
        if c.is_ascii_digit() {
            let mut s = String::new();
            while self.cur().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                s.push(self.advance().unwrap());
            }
            let n: u32 = s.parse().map_err(|_| ElabError::ParseError {
                msg: format!("level too large: {}", s),
                span: Span::new(start, self.pos),
            })?;
            return Ok((Token::Nat(n), Span::new(start, self.pos)));
        }

        // Identifiers and keywords
        if c.is_alphabetic() || c == '_' {
            let mut s = String::new();
            while self.cur().map(Self::is_ident_continue).unwrap_or(false) {
                s.push(self.advance().unwrap());
            }
            let tok = match s.as_str() {
                "view" => Token::KwView,
                "let" => Token::KwLet,
                "in" => Token::KwIn,
                "Type" => Token::KwType,
                "requires" => Token::KwRequires,
                "ensures" => Token::KwEnsures,
                "prove" => Token::KwProve,
                "law" => Token::KwLaw,
                "old" => Token::KwOld,
                "space" => Token::KwSpace,
                _ => {
                    let first = s.chars().next().unwrap();
                    if first.is_uppercase() {
                        Token::ConId(s)
                    } else {
                        Token::Ident(s)
                    }
                }
            };
            return Ok((tok, Span::new(start, self.pos)));
        }

        Err(ElabError::ParseError {
            msg: format!("unexpected character '{}'", c),
            span: Span::new(start, start + c.len_utf8()),
        })
    }

    /// Lex the entire source into a token+span list (including the `Eof` sentinel).
    pub fn lex(src: &'s str) -> Result<Vec<(Token, Span)>, ElabError> {
        let mut lx = Self::new(src);
        let mut out = Vec::new();
        loop {
            let (tok, span) = lx.next_token()?;
            let done = tok == Token::Eof;
            out.push((tok, span));
            if done {
                break;
            }
        }
        Ok(out)
    }
}
