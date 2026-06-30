//! V0/V1/L1 lexer (`31 §8`, `21 §6.1`, `35 §4.1`).
//!
//! Recognises the token subset for G1 (V0), V1 spec-annotation keywords, and
//! L1 numeric literals (integer, float, decimal with `d`-suffix, float32 with
//! `f32`-suffix) plus infix arithmetic operators `+`, `+%`, `*`, `==`.
//! Whitespace and `-- …` line comments are skipped.

use crate::error::{ElabError, Span};

/// A V0/V1/L1 token.
#[derive(Clone, Debug, PartialEq)]
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
    // L1 arithmetic operators
    Plus,         // `+`  — type-directed infix addition
    PlusPercent,  // `+%` — explicit wrapping add
    Star,         // `*`  — type-directed infix multiply
    EqEq,         // `==` — structural equality
    // L1 numeric literal tokens
    IntLit(i128),           // integer literal too large for u32
    FloatLit(f64),          // decimal-point float: `3.14`, `1e-9`
    DecimalLit(i64, i32),   // `d`-suffix: coeff × 10^exp; e.g. `0.1d` → (1,-1)
    Float32Lit(f32),        // `f32`-suffix: `1.5f32`
    // Atoms
    Ident(String),   // lowercase-initial term variable
    ConId(String),   // uppercase-initial base type / constructor
    Nat(u32),        // small non-negative integer (≤ u32::MAX); also a level digit
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

        // Single-char and multi-char punctuation
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
                if self.cur() == Some('=') {
                    self.advance();
                    return Ok((Token::EqEq, Span::new(start, self.pos)));
                }
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
            '+' => {
                self.advance();
                if self.cur() == Some('%') {
                    self.advance();
                    return Ok((Token::PlusPercent, Span::new(start, self.pos)));
                }
                return Ok((Token::Plus, Span::new(start, self.pos)));
            }
            '*' => {
                self.advance();
                return Ok((Token::Star, Span::new(start, self.pos)));
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

        // Numeric literals: starts with a digit
        if c.is_ascii_digit() {
            return self.lex_numeric(start);
        }

        // Identifiers and keywords
        if c.is_alphabetic() || c == '_' {
            let mut s = String::new();
            while self.cur().map(Self::is_ident_continue).unwrap_or(false) {
                s.push(self.advance().unwrap());
            }
            let tok = match s.as_str() {
                "view"     => Token::KwView,
                "let"      => Token::KwLet,
                "in"       => Token::KwIn,
                "Type"     => Token::KwType,
                "requires" => Token::KwRequires,
                "ensures"  => Token::KwEnsures,
                "prove"    => Token::KwProve,
                "law"      => Token::KwLaw,
                "old"      => Token::KwOld,
                "space"    => Token::KwSpace,
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

    /// Lex a numeric literal starting at `start`.
    /// Handles: integer, large-integer, float, decimal (`d`-suffix),
    /// float32 (`f32`-suffix).
    fn lex_numeric(&mut self, start: usize) -> Result<(Token, Span), ElabError> {
        // Read integer part
        let mut int_str = String::new();
        while self.cur().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            int_str.push(self.advance().unwrap());
        }

        // Optional fractional part
        let mut has_dot = false;
        let mut frac_str = String::new();
        let mut frac_places: i32 = 0;
        if self.cur() == Some('.')
            && self.src[self.pos + 1..]
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
        {
            self.advance(); // consume '.'
            has_dot = true;
            while self.cur().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                frac_str.push(self.advance().unwrap());
                frac_places += 1;
            }
        }

        // Optional exponent (for FloatLit only)
        let mut exp_str = String::new();
        if has_dot && (self.cur() == Some('e') || self.cur() == Some('E')) {
            exp_str.push(self.advance().unwrap());
            if self.cur() == Some('+') || self.cur() == Some('-') {
                exp_str.push(self.advance().unwrap());
            }
            while self.cur().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                exp_str.push(self.advance().unwrap());
            }
        }

        // Check for `d` suffix → Decimal
        if self.cur() == Some('d')
            && !self.src[self.pos + 1..]
                .chars()
                .next()
                .map(Self::is_ident_continue)
                .unwrap_or(false)
        {
            self.advance(); // consume 'd'
            let coeff_str = format!("{}{}", int_str, frac_str);
            let coeff: i64 = coeff_str.parse().map_err(|_| ElabError::ParseError {
                msg: format!("decimal literal coefficient too large: {}", coeff_str),
                span: Span::new(start, self.pos),
            })?;
            let exp: i32 = -frac_places;
            return Ok((Token::DecimalLit(coeff, exp), Span::new(start, self.pos)));
        }

        // Check for `f32` suffix → Float32Lit
        if self.src[self.pos..].starts_with("f32")
            && !self.src[self.pos + 3..]
                .chars()
                .next()
                .map(Self::is_ident_continue)
                .unwrap_or(false)
        {
            self.advance();
            self.advance();
            self.advance(); // consume "f32"
            let s = if has_dot {
                format!("{}.{}", int_str, frac_str)
            } else {
                int_str.clone()
            };
            let f: f32 = s.parse().unwrap_or(0.0_f32);
            return Ok((Token::Float32Lit(f), Span::new(start, self.pos)));
        }

        // Float if has dot or exponent
        if has_dot || !exp_str.is_empty() {
            let s = if exp_str.is_empty() {
                format!("{}.{}", int_str, frac_str)
            } else {
                format!("{}.{}e{}", int_str, frac_str, exp_str)
            };
            let f: f64 = s.parse().unwrap_or(0.0_f64);
            return Ok((Token::FloatLit(f), Span::new(start, self.pos)));
        }

        // Plain integer
        let n: i128 = int_str.parse().map_err(|_| ElabError::ParseError {
            msg: format!("integer literal too large: {}", int_str),
            span: Span::new(start, self.pos),
        })?;
        if n >= 0 && n <= u32::MAX as i128 {
            Ok((Token::Nat(n as u32), Span::new(start, self.pos)))
        } else {
            Ok((Token::IntLit(n), Span::new(start, self.pos)))
        }
    }

    /// Lex the entire source into a token+span list (including the `Eof`
    /// sentinel).
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
