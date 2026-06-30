//! V0/V1 parser: token stream → surface AST (`32 §8`, `39 §5.2`, `21 §6.1`).
//!
//! Recursive descent, no backtracking beyond the fixed Pi-lookahead.
//! V1 additions: `space view`, `requires`/`ensures` contract clauses,
//! `{ x : A | φ }` refinement types, `prove` and `law` declarations, `old`.

use crate::ast::{Binder, Decl, Expr, Type};
use crate::error::{ElabError, Span};
use crate::lexer::Token;

pub struct Parser {
    tokens: Vec<(Token, Span)>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<(Token, Span)>) -> Self {
        Self { tokens, pos: 0 }
    }

    // ----- cursor helpers -----

    fn peek(&self) -> &Token {
        &self.tokens[self.pos].0
    }

    fn peek_span(&self) -> &Span {
        &self.tokens[self.pos].1
    }

    fn lookahead(&self, n: usize) -> &Token {
        let idx = (self.pos + n).min(self.tokens.len() - 1);
        &self.tokens[idx].0
    }

    fn advance(&mut self) -> (Token, Span) {
        let pair = self.tokens[self.pos].clone();
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        pair
    }

    fn expect(&mut self, expected: &Token) -> Result<Span, ElabError> {
        let (tok, span) = self.advance();
        if &tok == expected {
            Ok(span)
        } else {
            Err(ElabError::ParseError {
                msg: format!("expected {:?}, found {:?}", expected, tok),
                span,
            })
        }
    }

    fn expect_ident(&mut self) -> Result<(String, Span), ElabError> {
        let (tok, span) = self.advance();
        match tok {
            Token::Ident(s) | Token::ConId(s) => Ok((s, span)),
            other => Err(ElabError::ParseError {
                msg: format!("expected identifier, found {:?}", other),
                span,
            }),
        }
    }

    fn at_eof(&self) -> bool {
        matches!(self.peek(), Token::Eof)
    }

    // ----- declaration parsing -----

    pub fn parse_decls(&mut self) -> Result<Vec<Decl>, ElabError> {
        let mut decls = Vec::new();
        while !self.at_eof() {
            decls.push(self.parse_decl()?);
        }
        Ok(decls)
    }

    fn parse_decl(&mut self) -> Result<Decl, ElabError> {
        let start = self.peek_span().start;
        match self.peek().clone() {
            Token::KwSpace => self.parse_space_view_decl(start),
            Token::KwView => self.parse_view_decl(start, false),
            Token::KwLet => self.parse_let_decl(start),
            Token::KwProve => self.parse_prove_decl(start),
            Token::KwLaw => self.parse_law_decl(start),
            other => Err(ElabError::ParseError {
                msg: format!(
                    "expected 'view', 'let', 'prove', 'law', or 'space view', found {:?}",
                    other
                ),
                span: self.peek_span().clone(),
            }),
        }
    }

    fn parse_space_view_decl(&mut self, start: usize) -> Result<Decl, ElabError> {
        self.advance(); // consume 'space'
        match self.peek().clone() {
            Token::KwView => self.parse_view_decl(start, true),
            other => Err(ElabError::ParseError {
                msg: format!("expected 'view' after 'space', found {:?}", other),
                span: self.peek_span().clone(),
            }),
        }
    }

    fn parse_view_decl(&mut self, start: usize, is_space_op: bool) -> Result<Decl, ElabError> {
        self.advance(); // consume 'view'
        let (name, _) = self.expect_ident()?;

        let mut params = Vec::new();
        while matches!(self.peek(), Token::LParen)
            && matches!(self.lookahead(1), Token::Ident(_) | Token::ConId(_))
        {
            if self.is_binder_ahead() {
                params.push(self.parse_binder()?);
            } else {
                break;
            }
        }

        let ret_ty = if matches!(self.peek(), Token::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        // V1 contract clauses: zero or more `requires φ` then `ensures ψ`
        let mut requires = Vec::new();
        while matches!(self.peek(), Token::KwRequires) {
            self.advance(); // consume 'requires'
            requires.push(self.parse_prop_expr()?);
        }
        let mut ensures = Vec::new();
        while matches!(self.peek(), Token::KwEnsures) {
            self.advance(); // consume 'ensures'
            ensures.push(self.parse_prop_expr()?);
        }

        self.expect(&Token::Eq)?;
        let body = self.parse_expr()?;
        let end = body.span().end;

        Ok(Decl::ViewDecl {
            name,
            params,
            ret_ty,
            requires,
            ensures,
            body,
            is_space_op,
            span: Span::new(start, end),
        })
    }

    fn is_binder_ahead(&self) -> bool {
        if !matches!(self.peek(), Token::LParen) {
            return false;
        }
        let mut i = 1;
        while matches!(self.lookahead(i), Token::Ident(_) | Token::ConId(_)) {
            i += 1;
        }
        i > 1 && matches!(self.lookahead(i), Token::Colon)
    }

    fn parse_binder(&mut self) -> Result<Binder, ElabError> {
        let start = self.peek_span().start;
        self.expect(&Token::LParen)?;
        let mut names = Vec::new();
        while matches!(self.peek(), Token::Ident(_) | Token::ConId(_)) {
            let (n, _) = self.expect_ident()?;
            names.push(n);
        }
        if names.is_empty() {
            return Err(ElabError::ParseError {
                msg: "binder needs at least one name".to_string(),
                span: self.peek_span().clone(),
            });
        }
        self.expect(&Token::Colon)?;
        let ty = self.parse_type()?;
        let end = self.peek_span().end;
        self.expect(&Token::RParen)?;
        Ok(Binder {
            names,
            ty,
            span: Span::new(start, end),
        })
    }

    fn parse_let_decl(&mut self, start: usize) -> Result<Decl, ElabError> {
        self.advance(); // consume 'let'
        let (name, _) = self.expect_ident()?;
        let ty = if matches!(self.peek(), Token::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        self.expect(&Token::Eq)?;
        let val = self.parse_expr()?;
        let end = val.span().end;
        Ok(Decl::LetDecl {
            name,
            ty,
            val,
            span: Span::new(start, end),
        })
    }

    /// `prove name : φ`
    fn parse_prove_decl(&mut self, start: usize) -> Result<Decl, ElabError> {
        self.advance(); // consume 'prove'
        let (name, _) = self.expect_ident()?;
        self.expect(&Token::Colon)?;
        let prop = self.parse_prop_expr()?;
        let end = prop.span().end;
        Ok(Decl::ProveDecl {
            name,
            prop,
            span: Span::new(start, end),
        })
    }

    /// `law Name (param) { field : φ ; … }`
    fn parse_law_decl(&mut self, start: usize) -> Result<Decl, ElabError> {
        self.advance(); // consume 'law'
        let (name, _) = self.expect_ident()?;
        self.expect(&Token::LParen)?;
        let (param, _) = self.expect_ident()?;
        self.expect(&Token::RParen)?;
        self.expect(&Token::LBrace)?;
        let mut fields = Vec::new();
        while !matches!(self.peek(), Token::RBrace | Token::Eof) {
            let (field_name, _) = self.expect_ident()?;
            self.expect(&Token::Colon)?;
            let prop = self.parse_prop_expr()?;
            fields.push((field_name, prop));
            // optional semicolon separator
            if matches!(self.peek(), Token::Semicolon) {
                self.advance();
            }
        }
        let end = self.peek_span().end;
        self.expect(&Token::RBrace)?;
        Ok(Decl::LawDecl {
            name,
            param,
            fields,
            span: Span::new(start, end),
        })
    }

    // ----- type parsing -----

    pub fn parse_type(&mut self) -> Result<Type, ElabError> {
        if matches!(self.peek(), Token::LParen) && self.is_dep_pi_ahead() {
            return self.parse_dep_pi();
        }
        // Refinement: `{ x : A | φ }`
        if matches!(self.peek(), Token::LBrace) {
            return self.parse_refinement_type();
        }
        let lhs = self.parse_atom_type()?;
        if matches!(self.peek(), Token::Arrow) {
            self.advance();
            let rhs = self.parse_type()?;
            let span = Span::merge(lhs.span(), rhs.span());
            return Ok(Type::TArr(Box::new(lhs), Box::new(rhs), span));
        }
        Ok(lhs)
    }

    /// `{ x : A | φ }` — refinement type (`21 §6.1`).
    fn parse_refinement_type(&mut self) -> Result<Type, ElabError> {
        let start = self.peek_span().start;
        self.expect(&Token::LBrace)?;
        let (x, _) = self.expect_ident()?;
        self.expect(&Token::Colon)?;
        let a = self.parse_type()?;
        self.expect(&Token::Pipe)?;
        let phi = self.parse_prop_expr()?;
        let end = self.peek_span().end;
        self.expect(&Token::RBrace)?;
        Ok(Type::TRefine(x, Box::new(a), Box::new(phi), Span::new(start, end)))
    }

    fn is_dep_pi_ahead(&self) -> bool {
        if !matches!(self.peek(), Token::LParen) {
            return false;
        }
        if !matches!(self.lookahead(1), Token::Ident(_) | Token::ConId(_)) {
            return false;
        }
        matches!(self.lookahead(2), Token::Colon)
    }

    fn parse_dep_pi(&mut self) -> Result<Type, ElabError> {
        let start = self.peek_span().start;
        self.expect(&Token::LParen)?;
        let (x, _) = self.expect_ident()?;
        self.expect(&Token::Colon)?;
        let a = self.parse_type()?;
        self.expect(&Token::RParen)?;
        self.expect(&Token::Arrow)?;
        let b = self.parse_type()?;
        let end = b.span().end;
        Ok(Type::TPi(x, Box::new(a), Box::new(b), Span::new(start, end)))
    }

    fn parse_atom_type(&mut self) -> Result<Type, ElabError> {
        let start = self.peek_span().start;
        match self.peek().clone() {
            Token::KwType => {
                self.advance();
                let level = if let Token::Nat(n) = self.peek().clone() {
                    self.advance();
                    Some(n)
                } else {
                    None
                };
                Ok(Type::TUniv(level, Span::new(start, self.tokens[self.pos - 1].1.end)))
            }
            Token::ConId(s) | Token::Ident(s) => {
                let span = self.peek_span().clone();
                self.advance();
                Ok(Type::TVar(s, span))
            }
            Token::LParen => {
                self.advance();
                let ty = self.parse_type()?;
                self.expect(&Token::RParen)?;
                Ok(ty)
            }
            other => Err(ElabError::ParseError {
                msg: format!("expected a type, found {:?}", other),
                span: self.peek_span().clone(),
            }),
        }
    }

    // ----- expression parsing -----

    /// Parse a proposition expression (for `requires`, `ensures`, `prove` bodies,
    /// and law fields). Same grammar as `parse_expr` for V1 but allows `old`.
    fn parse_prop_expr(&mut self) -> Result<Expr, ElabError> {
        self.parse_expr()
    }

    pub fn parse_expr(&mut self) -> Result<Expr, ElabError> {
        let lhs = self.parse_infix_expr()?;
        if matches!(self.peek(), Token::Colon) {
            let colon_span = self.peek_span().clone();
            self.advance();
            let ty = self.parse_type()?;
            let span = Span::merge(lhs.span(), ty.span());
            let _ = colon_span;
            return Ok(Expr::EAsc(Box::new(lhs), Box::new(ty), span));
        }
        Ok(lhs)
    }

    /// `parse_infix_expr` — handles `==` (lowest precedence infix).
    fn parse_infix_expr(&mut self) -> Result<Expr, ElabError> {
        use crate::ast::BinOp;
        let mut lhs = self.parse_additive_expr()?;
        loop {
            if matches!(self.peek(), Token::EqEq) {
                self.advance();
                let rhs = self.parse_additive_expr()?;
                let span = Span::merge(lhs.span(), rhs.span());
                lhs = Expr::EBinOp(BinOp::EqEq, Box::new(lhs), Box::new(rhs), span);
            } else {
                break;
            }
        }
        Ok(lhs)
    }

    /// `parse_additive_expr` — handles `+` and `+%`.
    fn parse_additive_expr(&mut self) -> Result<Expr, ElabError> {
        use crate::ast::BinOp;
        let mut lhs = self.parse_app_expr()?;
        loop {
            let op = match self.peek() {
                Token::Plus => BinOp::Add,
                Token::PlusPercent => BinOp::WrappingAdd,
                Token::Star => BinOp::Mul,
                _ => break,
            };
            self.advance();
            let rhs = self.parse_app_expr()?;
            let span = Span::merge(lhs.span(), rhs.span());
            lhs = Expr::EBinOp(op, Box::new(lhs), Box::new(rhs), span);
        }
        Ok(lhs)
    }

    fn parse_app_expr(&mut self) -> Result<Expr, ElabError> {
        match self.peek().clone() {
            Token::Lambda => self.parse_lambda(),
            Token::KwLet => self.parse_let_expr(),
            _ => {
                let mut f = self.parse_atom_expr()?;
                loop {
                    if !self.can_start_atom_expr() {
                        break;
                    }
                    let arg = self.parse_atom_expr()?;
                    let span = Span::merge(f.span(), arg.span());
                    f = Expr::EApp(Box::new(f), Box::new(arg), span);
                }
                Ok(f)
            }
        }
    }

    fn can_start_atom_expr(&self) -> bool {
        matches!(
            self.peek(),
            Token::Ident(_)
                | Token::ConId(_)
                | Token::KwType
                | Token::LParen
                | Token::KwOld
                | Token::Nat(_)
                | Token::IntLit(_)
                | Token::FloatLit(_)
                | Token::DecimalLit(_, _)
                | Token::Float32Lit(_)
        )
    }

    fn parse_lambda(&mut self) -> Result<Expr, ElabError> {
        let start = self.peek_span().start;
        self.advance(); // consume `\` / `λ`
        let mut names = Vec::new();
        loop {
            match self.peek().clone() {
                Token::Ident(s) | Token::ConId(s) => {
                    self.advance();
                    names.push(s);
                }
                Token::Dot => break,
                other => {
                    return Err(ElabError::ParseError {
                        msg: format!("expected binder name or '.', found {:?}", other),
                        span: self.peek_span().clone(),
                    })
                }
            }
        }
        if names.is_empty() {
            return Err(ElabError::ParseError {
                msg: "lambda needs at least one binder name".to_string(),
                span: self.peek_span().clone(),
            });
        }
        self.expect(&Token::Dot)?;
        let body = self.parse_expr()?;
        let end = body.span().end;
        Ok(Expr::ELam(names, Box::new(body), Span::new(start, end)))
    }

    fn parse_let_expr(&mut self) -> Result<Expr, ElabError> {
        let start = self.peek_span().start;
        self.advance(); // consume 'let'
        let (x, _) = self.expect_ident()?;
        let ty = if matches!(self.peek(), Token::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        self.expect(&Token::Eq)?;
        let rhs = self.parse_infix_expr()?;
        self.expect(&Token::KwIn)?;
        let body = self.parse_expr()?;
        let end = body.span().end;
        Ok(Expr::ELet(
            x,
            ty,
            Box::new(rhs),
            Box::new(body),
            Span::new(start, end),
        ))
    }

    fn parse_atom_expr(&mut self) -> Result<Expr, ElabError> {
        use crate::ast::NumLit;
        let start = self.peek_span().start;
        match self.peek().clone() {
            Token::Nat(n) => {
                let span = self.peek_span().clone();
                self.advance();
                Ok(Expr::ENumLit(NumLit::Int(n as i128), span))
            }
            Token::IntLit(n) => {
                let span = self.peek_span().clone();
                self.advance();
                Ok(Expr::ENumLit(NumLit::Int(n), span))
            }
            Token::FloatLit(f) => {
                let span = self.peek_span().clone();
                self.advance();
                Ok(Expr::ENumLit(NumLit::Float(f), span))
            }
            Token::DecimalLit(c, e) => {
                let span = self.peek_span().clone();
                self.advance();
                Ok(Expr::ENumLit(NumLit::Decimal(c, e), span))
            }
            Token::Float32Lit(f) => {
                let span = self.peek_span().clone();
                self.advance();
                Ok(Expr::ENumLit(NumLit::Float32(f), span))
            }
            Token::Ident(s) => {
                let span = self.peek_span().clone();
                self.advance();
                Ok(Expr::EVar(s, span))
            }
            Token::ConId(s) => {
                let span = self.peek_span().clone();
                self.advance();
                Ok(Expr::ECon(s, span))
            }
            Token::KwType => {
                self.advance();
                let level = if let Token::Nat(n) = self.peek().clone() {
                    self.advance();
                    Some(n)
                } else {
                    None
                };
                let end = self.tokens[self.pos - 1].1.end;
                Ok(Expr::EUniv(level, Span::new(start, end)))
            }
            // `old e` — pre-state reference (`21 §6.4`)
            Token::KwOld => {
                self.advance(); // consume 'old'
                let arg = self.parse_atom_expr()?;
                let end = arg.span().end;
                Ok(Expr::EOld(Box::new(arg), Span::new(start, end)))
            }
            Token::LParen => {
                self.advance();
                let inner = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                let end = self.tokens[self.pos - 1].1.end;
                let span = Span::new(start, end);
                Ok(match inner {
                    Expr::EAsc(e, t, _) => Expr::EAsc(e, t, span),
                    e => match e {
                        Expr::EVar(s, _) => Expr::EVar(s, span),
                        Expr::ECon(s, _) => Expr::ECon(s, span),
                        Expr::EUniv(l, _) => Expr::EUniv(l, span),
                        Expr::EApp(f, a, _) => Expr::EApp(f, a, span),
                        Expr::ELam(ns, b, _) => Expr::ELam(ns, b, span),
                        Expr::ELet(x, ty, r, body, _) => Expr::ELet(x, ty, r, body, span),
                        Expr::EAsc(e, t, _) => Expr::EAsc(e, t, span),
                        Expr::EOld(e, _) => Expr::EOld(e, span),
                        Expr::ENumLit(lit, _) => Expr::ENumLit(lit, span),
                        Expr::EBinOp(op, l, r, _) => Expr::EBinOp(op, l, r, span),
                    },
                })
            }
            other => Err(ElabError::ParseError {
                msg: format!("expected an expression, found {:?}", other),
                span: self.peek_span().clone(),
            }),
        }
    }

    pub fn parse_expr_only(&mut self) -> Result<Expr, ElabError> {
        let e = self.parse_expr()?;
        if !self.at_eof() {
            return Err(ElabError::ParseError {
                msg: format!("unexpected token after expression: {:?}", self.peek()),
                span: self.peek_span().clone(),
            });
        }
        Ok(e)
    }
}

// ---- public parse functions ----

pub fn parse_decls(src: &str) -> Result<Vec<Decl>, ElabError> {
    let tokens = crate::lexer::Lexer::lex(src)?;
    Parser::new(tokens).parse_decls()
}

pub fn parse_expr(src: &str) -> Result<Expr, ElabError> {
    let tokens = crate::lexer::Lexer::lex(src)?;
    Parser::new(tokens).parse_expr_only()
}
