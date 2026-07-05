//! V0/V1/L2 parser: token stream → surface AST (`32 §8`, `39 §5.2`,
//! `21 §6.1`, `34`).
//!
//! Recursive descent, no backtracking beyond the fixed Pi-lookahead.
//! V1 additions: `space view`, `requires`/`ensures` contract clauses,
//! `{ x : A | φ }` refinement types, `prove` and `law` declarations, `old`.
//! L2 additions: `data D p₁…pₙ = C₁ τ… | C₂ τ…` sum types; `match e { … }`
//! pattern matching; `type T = A` surface type aliases; `T a b` type app.

use crate::ast::{
    Binder, CtorDecl, Decl, DefKeyword, EffectRowSyntax, Expr, MatchArm, PatKind, Pattern, Type,
};
use crate::error::{ElabError, Span};
use crate::lexer::Token;
use crate::temporal::TemporalExpr;

pub struct Parser {
    tokens: Vec<(Token, Span)>,
    pos: usize,
    /// The original source — retained so a `temporal{}` block can carry its
    /// verbatim formula text (human-visible, not erased, `72 §4`).
    src: String,
}

impl Parser {
    pub fn new(tokens: Vec<(Token, Span)>, src: String) -> Self {
        Self { tokens, pos: 0, src }
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

    fn expect_legacy_view_name(&mut self) -> Result<(String, Span), ElabError> {
        let (tok, span) = self.advance();
        match tok {
            Token::Ident(s) | Token::ConId(s) => Ok((s, span)),
            Token::KwConst => Ok(("const".to_string(), span)),
            Token::KwFn => Ok(("fn".to_string(), span)),
            Token::KwProc => Ok(("proc".to_string(), span)),
            other => Err(ElabError::ParseError {
                msg: format!("expected identifier, found {:?}", other),
                span,
            }),
        }
    }

    fn expect_con(&mut self) -> Result<(String, Span), ElabError> {
        let (tok, span) = self.advance();
        match tok {
            Token::ConId(s) => Ok((s, span)),
            other => Err(ElabError::ParseError {
                msg: format!("expected uppercase constructor name, found {:?}", other),
                span,
            }),
        }
    }

    fn at_eof(&self) -> bool {
        matches!(self.peek(), Token::Eof)
    }

    /// Extend `first` (a just-consumed `ConId`) with zero or more
    /// `. ident-or-conid` segments — `M.foo`, `M.N.Bar` (`33 §3.2`
    /// qualified reference syntax). Joins into a single dotted string;
    /// name resolution (`modules.rs`) splits it back apart at the last
    /// `.` to find the exporting module. Only triggered from a `ConId`
    /// start since qualifying modules are conventionally capitalized and
    /// a bare `.` is otherwise only a lambda-binder terminator (consumed
    /// directly by `parse_lambda`, never reaching here).
    fn parse_dotted(&mut self, first: String, first_span: Span) -> (String, Span) {
        let mut joined = first;
        let mut end = first_span.end;
        while matches!(self.peek(), Token::Dot)
            && matches!(self.lookahead(1), Token::Ident(_) | Token::ConId(_))
        {
            self.advance(); // consume '.'
            let (seg, seg_span) = match self.peek().clone() {
                Token::Ident(s) | Token::ConId(s) => {
                    self.advance();
                    (s, self.tokens[self.pos - 1].1.clone())
                }
                _ => unreachable!("guarded by lookahead above"),
            };
            joined.push('.');
            joined.push_str(&seg);
            end = seg_span.end;
        }
        (joined, Span::new(first_span.start, end))
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
            Token::KwView => self.parse_view_decl(start, false, DefKeyword::View),
            Token::KwConst => self.parse_view_decl(start, false, DefKeyword::Const),
            Token::KwFn => self.parse_view_decl(start, false, DefKeyword::Fn),
            Token::KwProc => self.parse_view_decl(start, false, DefKeyword::Proc),
            Token::KwLet => self.parse_let_decl(start),
            Token::KwProve => self.parse_prove_decl(start),
            Token::KwLaw => self.parse_law_decl(start),
            Token::KwData => self.parse_data_decl(start),
            Token::KwTypeAlias => self.parse_type_alias_decl(start),
            Token::KwForeign => self.parse_foreign_decl(start),
            Token::KwTemporal => self.parse_temporal_decl(start),
            Token::KwClass => self.parse_class_decl(start),
            Token::KwInstance => self.parse_instance_decl(start),
            Token::KwDerive => self.parse_derive_decl(start),
            Token::KwModule => self.parse_module_decl(start),
            Token::KwImport => self.parse_import_decl(start),
            Token::KwUse => self.parse_use_decl(start),
            Token::KwPub => self.parse_pub_decl(start),
            other => Err(ElabError::ParseError {
                msg: format!(
                    "expected 'view', 'const', 'fn', 'proc', 'let', 'prove', 'law', 'data', 'type', 'foreign', \
                     'temporal', 'class', 'instance', 'derive', 'module', 'import', 'use', \
                     'pub', or 'space view'/'space proc', found {:?}",
                    other
                ),
                span: self.peek_span().clone(),
            }),
        }
    }

    fn parse_space_view_decl(&mut self, start: usize) -> Result<Decl, ElabError> {
        self.advance(); // consume 'space'
        match self.peek().clone() {
            Token::KwView => self.parse_view_decl(start, true, DefKeyword::View),
            Token::KwProc => self.parse_view_decl(start, true, DefKeyword::Proc),
            other => Err(ElabError::ParseError {
                msg: format!("expected 'view' or 'proc' after 'space', found {:?}", other),
                span: self.peek_span().clone(),
            }),
        }
    }

    fn parse_view_decl(
        &mut self,
        start: usize,
        is_space_op: bool,
        keyword: DefKeyword,
    ) -> Result<Decl, ElabError> {
        self.advance(); // consume definition keyword
        let (name, _) = if keyword == DefKeyword::View {
            self.expect_legacy_view_name()?
        } else {
            self.expect_ident()?
        };

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

        // L3b: optional `where C₁ T₁ ; C₂ T₂` class constraints (`37 §6`).
        // Parsed between the contract clauses and the `=` body.
        let mut constraints = Vec::new();
        if matches!(self.peek(), Token::KwWhere) {
            self.advance(); // consume 'where'
            loop {
                let (cname, _) = self.expect_ident()?;
                let cty = self.parse_type()?;
                constraints.push((cname, cty));
                if !matches!(self.peek(), Token::Semicolon) {
                    break;
                }
                self.advance(); // consume ';' (Semicolon)
            }
        }

        let visits = if self.is_contextual_ident("visits") {
            self.advance(); // consume contextual 'visits'
            Some(self.parse_effect_row_syntax()?)
        } else {
            None
        };

        self.expect(&Token::Eq)?;
        let body = self.parse_expr()?;
        let end = body.span().end;

        Ok(Decl::ViewDecl {
            keyword,
            name,
            params,
            ret_ty,
            requires,
            ensures,
            constraints,
            visits,
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

    /// `temporal name { φ }` — a delegated temporal obligation (`72 §4`).
    ///
    /// The body is a `temporal{}` formula (keywords `(oracle)`/`OQ-syntax`,
    /// contextual operator words) that elaborates to the §3 constructors and
    /// is tagged `delegated`.
    fn parse_temporal_decl(&mut self, start: usize) -> Result<Decl, ElabError> {
        self.advance(); // consume 'temporal'
        let (name, _) = self.expect_ident()?;
        let lb_span = self.expect(&Token::LBrace)?;
        let formula = self.parse_temporal_formula()?;
        let rb_span = self.expect(&Token::RBrace)?;
        // Verbatim formula text between `{` and `}` — human-visible in source
        // (the property appears verbatim, not erased, `72 §4`).
        let source = self.src[lb_span.end..rb_span.start].trim().to_string();
        Ok(Decl::TemporalDecl {
            name,
            formula,
            source,
            span: Span::new(start, rb_span.end),
        })
    }

    /// A `temporal{}` formula — recursive descent with precedence
    /// (loosest → tightest): `leadsto`, `until`, `or`, `and`, prefix
    /// (`not`/`eventually`/`always`/`next`), atom. Operator words are
    /// contextual: lowercase identifiers matched by name (only `temporal`
    /// itself is a lexer keyword), so the grammar adds no global keywords.
    fn parse_temporal_formula(&mut self) -> Result<TemporalExpr, ElabError> {
        self.parse_t_leadsto()
    }

    fn parse_t_leadsto(&mut self) -> Result<TemporalExpr, ElabError> {
        let mut lhs = self.parse_t_until()?;
        while self.is_t_op("leadsto") {
            self.advance();
            let rhs = self.parse_t_until()?;
            lhs = TemporalExpr::Leadsto(Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_t_until(&mut self) -> Result<TemporalExpr, ElabError> {
        let mut lhs = self.parse_t_or()?;
        while self.is_t_op("until") {
            self.advance();
            let rhs = self.parse_t_or()?;
            lhs = TemporalExpr::Until(Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_t_or(&mut self) -> Result<TemporalExpr, ElabError> {
        let mut lhs = self.parse_t_and()?;
        while self.is_t_op("or") {
            self.advance();
            let rhs = self.parse_t_and()?;
            lhs = TemporalExpr::Or(Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_t_and(&mut self) -> Result<TemporalExpr, ElabError> {
        let mut lhs = self.parse_t_prefix()?;
        while self.is_t_op("and") {
            self.advance();
            let rhs = self.parse_t_prefix()?;
            lhs = TemporalExpr::And(Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_t_prefix(&mut self) -> Result<TemporalExpr, ElabError> {
        // Prefix operators — right-associative (a prefix op wraps the next
        // prefix-or-atom). `top`/`true` are NOT operators (they are atoms).
        if self.is_t_op("not") {
            self.advance();
            return Ok(TemporalExpr::Not(Box::new(self.parse_t_prefix()?)));
        }
        if self.is_t_op("eventually") {
            self.advance();
            return Ok(TemporalExpr::Eventually(Box::new(self.parse_t_prefix()?)));
        }
        if self.is_t_op("always") {
            self.advance();
            return Ok(TemporalExpr::Always(Box::new(self.parse_t_prefix()?)));
        }
        if self.is_t_op("next") {
            self.advance();
            return Ok(TemporalExpr::Next(Box::new(self.parse_t_prefix()?)));
        }
        self.parse_t_atom()
    }

    fn parse_t_atom(&mut self) -> Result<TemporalExpr, ElabError> {
        match self.peek().clone() {
            Token::LParen => {
                self.advance();
                let e = self.parse_temporal_formula()?;
                self.expect(&Token::RParen)?;
                Ok(e)
            }
            Token::Ident(s) => {
                if is_temporal_operator(&s) {
                    return Err(ElabError::ParseError {
                        msg: format!(
                            "unexpected temporal operator '{}' in atom position",
                            s
                        ),
                        span: self.peek_span().clone(),
                    });
                }
                self.advance();
                Ok(TemporalExpr::Atom(s))
            }
            other => Err(ElabError::ParseError {
                msg: format!("expected a temporal formula atom, found {:?}", other),
                span: self.peek_span().clone(),
            }),
        }
    }

    /// Is the current token the contextual temporal-operator word `op`?
    fn is_t_op(&self, op: &str) -> bool {
        self.is_contextual_ident(op)
    }

    fn is_contextual_ident(&self, ident: &str) -> bool {
        matches!(self.peek(), Token::Ident(s) if s == ident)
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

    /// `class C (A : Type) { field : Type ; … }` — typeclass declaration
    /// (`33 §5`).  The single type param is optional; fields are `name : Type`.
    fn parse_class_decl(&mut self, start: usize) -> Result<Decl, ElabError> {
        self.advance(); // consume 'class'
        let (name, _) = self.expect_ident()?;
        // Optional single type parameter `(A : Type)` or bare ident `A`
        let param = if matches!(self.peek(), Token::Ident(_) | Token::ConId(_)) {
            let (p, _) = self.expect_ident()?;
            Some(p)
        } else {
            None
        };
        self.expect(&Token::LBrace)?;
        let mut fields = Vec::new();
        while !matches!(self.peek(), Token::RBrace | Token::Eof) {
            let (field_name, _) = self.expect_ident()?;
            self.expect(&Token::Colon)?;
            let ty = self.parse_type()?;
            fields.push((field_name, ty));
            if matches!(self.peek(), Token::Semicolon) {
                self.advance();
            }
        }
        let end = self.peek_span().end;
        self.expect(&Token::RBrace)?;
        Ok(Decl::ClassDecl {
            name,
            param,
            fields,
            span: Span::new(start, end),
        })
    }

    /// `instance C HeadType [where C1 T1 ; C2 T2] { field = expr ; … }`
    /// (`33 §5`, `39 §6`).
    fn parse_instance_decl(&mut self, start: usize) -> Result<Decl, ElabError> {
        self.advance(); // consume 'instance'
        let (class_name, _) = self.expect_ident()?;
        let head_type = self.parse_atom_type_app()?;
        // Optional `where C1 T1 ; C2 T2` constraint list
        let mut constraints = Vec::new();
        if matches!(self.peek(), Token::KwWhere) {
            self.advance(); // consume 'where'
            loop {
                let (cname, _) = self.expect_ident()?;
                let cty = self.parse_atom_type_app()?;
                constraints.push((cname, cty));
                if matches!(self.peek(), Token::Semicolon) {
                    self.advance();
                    // Continue if next is an ident (another constraint) not LBrace
                    if !matches!(self.peek(), Token::Ident(_) | Token::ConId(_)) {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        self.expect(&Token::LBrace)?;
        let mut fields = Vec::new();
        while !matches!(self.peek(), Token::RBrace | Token::Eof) {
            let (field_name, _) = self.expect_ident()?;
            self.expect(&Token::Eq)?;
            let expr = self.parse_expr()?;
            fields.push((field_name, expr));
            if matches!(self.peek(), Token::Semicolon) {
                self.advance();
            }
        }
        let end = self.peek_span().end;
        self.expect(&Token::RBrace)?;
        Ok(Decl::InstanceDecl {
            class_name,
            head_type,
            constraints,
            fields,
            span: Span::new(start, end),
        })
    }

    /// `derive ClassName for DataName` (`33 §5.6`, `39 §6.6`).
    fn parse_derive_decl(&mut self, start: usize) -> Result<Decl, ElabError> {
        self.advance(); // consume 'derive'
        let (class_name, _) = self.expect_ident()?;
        // consume 'for' as a contextual keyword (it's an Ident token)
        match self.peek().clone() {
            Token::Ident(s) if s == "for" => {
                self.advance();
            }
            other => {
                return Err(ElabError::ParseError {
                    msg: format!("expected 'for' in derive declaration, found {:?}", other),
                    span: self.peek_span().clone(),
                });
            }
        }
        let (data_name, _) = self.expect_con()?;
        let end = self.tokens[self.pos - 1].1.end;
        Ok(Decl::DeriveDecl {
            class_name,
            data_name,
            span: Span::new(start, end),
        })
    }

    /// `module M { decl₁ … declₙ }` (`33 §3.1`).
    fn parse_module_decl(&mut self, start: usize) -> Result<Decl, ElabError> {
        self.advance(); // consume 'module'
        let (name, _) = self.expect_ident()?;
        self.expect(&Token::LBrace)?;
        let mut decls = Vec::new();
        while !matches!(self.peek(), Token::RBrace | Token::Eof) {
            decls.push(self.parse_decl()?);
        }
        let end = self.peek_span().end;
        self.expect(&Token::RBrace)?;
        Ok(Decl::ModuleDecl { name, decls, span: Span::new(start, end) })
    }

    /// `import M` | `import M as N` | `import M (foo, Bar)` (`33 §3.2`).
    fn parse_import_decl(&mut self, start: usize) -> Result<Decl, ElabError> {
        self.advance(); // consume 'import'
        let (module, _) = self.expect_ident()?;
        let kind = match self.peek().clone() {
            Token::Ident(s) if s == "as" => {
                self.advance();
                let (alias, _) = self.expect_ident()?;
                crate::ast::ImportKind::Aliased(alias)
            }
            Token::LParen => {
                self.advance();
                let mut names = Vec::new();
                loop {
                    let (n, _) = self.expect_ident()?;
                    names.push(n);
                    if matches!(self.peek(), Token::Comma) {
                        self.advance();
                        continue;
                    }
                    break;
                }
                self.expect(&Token::RParen)?;
                crate::ast::ImportKind::Selective(names)
            }
            _ => crate::ast::ImportKind::Qualified,
        };
        let end = self.tokens[self.pos - 1].1.end;
        Ok(Decl::ImportDecl { module, kind, span: Span::new(start, end) })
    }

    /// `use M` — unqualified open import (`33 §3.2`).
    fn parse_use_decl(&mut self, start: usize) -> Result<Decl, ElabError> {
        self.advance(); // consume 'use'
        let (module, _) = self.expect_ident()?;
        let end = self.tokens[self.pos - 1].1.end;
        Ok(Decl::ImportDecl { module, kind: crate::ast::ImportKind::Open, span: Span::new(start, end) })
    }

    /// `pub <decl>` — export marker (`33 §4.1`).
    fn parse_pub_decl(&mut self, _start: usize) -> Result<Decl, ElabError> {
        self.advance(); // consume 'pub'
        let inner = self.parse_decl()?;
        Ok(Decl::Pub(Box::new(inner)))
    }

    /// `data D p₁…pₙ = C₁ τ₁₁… | C₂ τ₂₁… | …`
    ///
    /// Simple (non-indexed) inductive type declaration (`34 §1`). Type params
    /// are lowercase idents; constructors are `ConId type_atom*`.
    fn parse_data_decl(&mut self, start: usize) -> Result<Decl, ElabError> {
        self.advance(); // consume 'data'
        let (name, _) = self.expect_con()?;

        // Collect type-parameter names (lowercase identifiers before `=`).
        let mut type_params = Vec::new();
        while matches!(self.peek(), Token::Ident(_)) {
            let (p, _) = self.expect_ident()?;
            type_params.push(p);
        }

        self.expect(&Token::Eq)?;

        // Parse constructor list: `C₁ τ… | C₂ τ… | …`
        let mut ctors = Vec::new();
        loop {
            let ctor = self.parse_ctor_decl()?;
            ctors.push(ctor);
            if matches!(self.peek(), Token::Pipe) {
                self.advance(); // consume `|`
            } else {
                break;
            }
        }

        let end = ctors.last().map(|c| c.span.end).unwrap_or(start);
        Ok(Decl::DataDecl {
            name,
            type_params,
            ctors,
            span: Span::new(start, end),
        })
    }

    /// `C τ₁ τ₂ …` — one constructor in a `data` declaration.
    fn parse_ctor_decl(&mut self) -> Result<CtorDecl, ElabError> {
        let start = self.peek_span().start;
        let (name, _) = self.expect_con()?;
        let mut args = Vec::new();
        // Collect type atoms (stop at `|`, `=`, `\n`-equivalent token starts, EOF)
        while self.can_start_atom_type() {
            args.push(self.parse_atom_type_app()?);
        }
        let end = if args.is_empty() {
            self.tokens[self.pos - 1].1.end
        } else {
            args.last().unwrap().span().end
        };
        Ok(CtorDecl {
            name,
            args,
            span: Span::new(start, end),
        })
    }

    /// `type T = A` — surface type alias.
    fn parse_type_alias_decl(&mut self, start: usize) -> Result<Decl, ElabError> {
        self.advance(); // consume 'type'
        let (name, _) = self.expect_con()?;
        self.expect(&Token::Eq)?;
        let ty = self.parse_type()?;
        let end = ty.span().end;
        Ok(Decl::TypeAlias {
            name,
            ty,
            span: Span::new(start, end),
        })
    }

    /// `foreign f : T = "symbol" "library" [pure] [E1, E2, …]` (`38 §2.1`).
    ///
    /// Keyword spellings are `(oracle)` — the exact tokens are finalized by
    /// the build team. This implementation uses `foreign`, `pure` (as a
    /// contextual ident), and effect labels as ConIds.
    fn parse_foreign_decl(&mut self, start: usize) -> Result<Decl, ElabError> {
        self.advance(); // consume 'foreign'
        let (name, _) = self.expect_ident()?;
        self.expect(&Token::Colon)?;
        let ty = self.parse_type()?;
        self.expect(&Token::Eq)?;
        // symbol string literal
        let symbol = match self.advance() {
            (Token::Str(s), _) => s,
            (other, span) => {
                return Err(ElabError::ParseError {
                    msg: format!("expected string literal for symbol name, found {:?}", other),
                    span,
                })
            }
        };
        // library string literal
        let library = match self.advance() {
            (Token::Str(s), _) => s,
            (other, span) => {
                return Err(ElabError::ParseError {
                    msg: format!("expected string literal for library name, found {:?}", other),
                    span,
                })
            }
        };
        // optional `pure` contextual keyword
        let is_pure = if matches!(self.peek(), Token::Ident(s) if s == "pure") {
            self.advance();
            true
        } else {
            false
        };
        // optional `[E1, E2, …]` effect-row annotation
        let visits = if matches!(self.peek(), Token::LBracket) {
            self.advance(); // consume '['
            let mut labels = Vec::new();
            while !matches!(self.peek(), Token::RBracket | Token::Eof) {
                let (label, _) = self.expect_ident()?;
                labels.push(label);
                if matches!(self.peek(), Token::Comma) {
                    self.advance();
                }
            }
            let end = self.peek_span().end;
            self.expect(&Token::RBracket)?;
            let _ = end;
            labels
        } else {
            Vec::new()
        };
        let end = self.peek_span().start;
        Ok(Decl::ForeignDecl {
            name,
            ty,
            symbol,
            library,
            is_pure,
            visits,
            span: Span::new(start, end),
        })
    }

    /// Parse `[...]` effect-row syntax (`36 §1.5`).
    ///
    /// Accepted shapes:
    /// - `[Console, FS]` — concrete row
    /// - `[e]` — bare row variable
    /// - `[Console | e]` — open row with concrete heads and a variable tail
    pub fn parse_effect_row_syntax(&mut self) -> Result<EffectRowSyntax, ElabError> {
        let start = self.peek_span().start;
        self.expect(&Token::LBracket)?;

        let mut heads = Vec::new();
        let mut tail = None;
        while !matches!(self.peek(), Token::RBracket | Token::Eof) {
            let (name, span) = self.expect_ident()?;
            let is_row_var = name
                .chars()
                .next()
                .map(|c| c.is_lowercase())
                .unwrap_or(false);

            if is_row_var {
                if heads.is_empty() && tail.is_none() {
                    tail = Some(name);
                    break;
                }
                return Err(ElabError::ParseError {
                    msg: "row variable must appear as bare [e] or as the tail in [E | e]"
                        .to_string(),
                    span,
                });
            }

            heads.push(name);
            match self.peek() {
                Token::Comma => {
                    self.advance();
                }
                Token::Pipe => {
                    self.advance();
                    let (tail_name, tail_span) = self.expect_ident()?;
                    let tail_is_var = tail_name
                        .chars()
                        .next()
                        .map(|c| c.is_lowercase())
                        .unwrap_or(false);
                    if !tail_is_var {
                        return Err(ElabError::ParseError {
                            msg: "open row tail must be a lowercase row variable".to_string(),
                            span: tail_span,
                        });
                    }
                    tail = Some(tail_name);
                    break;
                }
                _ => {}
            }
        }

        let end = self.peek_span().end;
        self.expect(&Token::RBracket)?;
        Ok(EffectRowSyntax {
            heads,
            tail,
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
        // Parse the base type (possibly applied to type args)
        let lhs = self.parse_type_app()?;
        if matches!(self.peek(), Token::Arrow) {
            self.advance();
            let rhs = self.parse_type()?;
            let span = Span::merge(lhs.span(), rhs.span());
            return Ok(Type::TArr(Box::new(lhs), Box::new(rhs), span));
        }
        Ok(lhs)
    }

    /// Parse a (possibly applied) type: `T a b`.
    fn parse_type_app(&mut self) -> Result<Type, ElabError> {
        let mut ty = self.parse_atom_type()?;
        while self.can_start_atom_type() {
            let arg = self.parse_atom_type()?;
            let span = Span::merge(ty.span(), arg.span());
            ty = Type::TApp(Box::new(ty), Box::new(arg), span);
        }
        Ok(ty)
    }

    /// Parse a type atom followed by zero or more atom-type args (for ctor decl args).
    fn parse_atom_type_app(&mut self) -> Result<Type, ElabError> {
        // In ctor decl context, we parse ONE atom-level type (no arrow, no leading Pi).
        self.parse_atom_type()
    }

    fn can_start_atom_type(&self) -> bool {
        if matches!(self.peek(), Token::Ident(s) if s == "visits")
            && matches!(self.lookahead(1), Token::LBracket)
        {
            return false;
        }
        matches!(
            self.peek(),
            Token::ConId(_) | Token::Ident(_) | Token::KwType | Token::LParen
        )
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
            Token::ConId(s) => {
                let span = self.peek_span().clone();
                self.advance();
                let (name, span) = self.parse_dotted(s, span);
                Ok(Type::TVar(name, span))
            }
            Token::Ident(s) => {
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
        let lhs = self.parse_arrow_expr()?;
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

    /// `parse_arrow_expr` — expr-position `->` (VAL2 #4, `32 §3`): the
    /// dependent `(x:A) -> B` and non-dependent `A -> B` forms, both
    /// elaborating to the existing kernel `Pi`. Binds looser than `==`/all
    /// arithmetic, tighter than ascription (`32 §6`); right-associative.
    ///
    /// The dependent form needs a speculative parse: `(ident : type)` is
    /// ALSO an ordinary parenthesized ascription (no trailing `->`), so
    /// `is_dep_pi_ahead()`'s cheap token-shape check isn't sufficient by
    /// itself (unlike type position, where `(ident:A)` is unambiguously a
    /// Pi and never a bare ascription) — attempt it, and if the type
    /// domain isn't followed by `RParen` then `Arrow`, rewind and fall
    /// through to the ordinary ascription/grouping parse.
    fn parse_arrow_expr(&mut self) -> Result<Expr, ElabError> {
        if matches!(self.peek(), Token::LParen) && self.is_dep_pi_ahead() {
            let save = self.pos;
            let start = self.peek_span().start;
            self.advance(); // '('
            let (x, _) = self.expect_ident()?;
            self.expect(&Token::Colon)?;
            let a = self.parse_type()?;
            if matches!(self.peek(), Token::RParen) && matches!(self.lookahead(1), Token::Arrow) {
                self.advance(); // ')'
                self.advance(); // '->'
                let b = self.parse_arrow_expr()?; // right-assoc
                let end = b.span().end;
                return Ok(Expr::EPi(x, Box::new(a), Box::new(b), Span::new(start, end)));
            }
            // Not actually a dependent arrow (no trailing `->`) — this was
            // a plain parenthesized ascription/expr; rewind and re-parse
            // through the ordinary path (pure backtrack: only `self.pos`
            // changed above).
            self.pos = save;
        }
        let lhs = self.parse_infix_expr()?;
        if matches!(self.peek(), Token::Arrow) {
            self.advance();
            let rhs = self.parse_arrow_expr()?; // right-assoc
            let span = Span::merge(lhs.span(), rhs.span());
            return Ok(Expr::EArrow(Box::new(lhs), Box::new(rhs), span));
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

    /// `parse_additive_expr` — handles `+`, `+%`, `-` (left-associative,
    /// binds looser than `*`, VAL2 #11's conventional-precedence pin).
    fn parse_additive_expr(&mut self) -> Result<Expr, ElabError> {
        use crate::ast::BinOp;
        let mut lhs = self.parse_multiplicative_expr()?;
        loop {
            let op = match self.peek() {
                Token::Plus => BinOp::Add,
                Token::PlusPercent => BinOp::WrappingAdd,
                Token::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let rhs = self.parse_multiplicative_expr()?;
            let span = Span::merge(lhs.span(), rhs.span());
            lhs = Expr::EBinOp(op, Box::new(lhs), Box::new(rhs), span);
        }
        Ok(lhs)
    }

    /// `parse_multiplicative_expr` — handles `*` (binds tighter than `+`/`-`,
    /// left-associative; VAL2 #11's conventional-precedence pin — fixes the
    /// latent bug where `+`/`*` shared one flat precedence level).
    fn parse_multiplicative_expr(&mut self) -> Result<Expr, ElabError> {
        use crate::ast::BinOp;
        let mut lhs = self.parse_app_expr()?;
        loop {
            let op = match self.peek() {
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
            Token::KwMatch => self.parse_match_expr(),
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
                | Token::Str(_)
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
        // VAL2 #4: an arrow-type value must be reachable in `let`-bound
        // position too, not just annotations — `parse_arrow_expr`, not the
        // narrower `parse_infix_expr` this called before.
        let rhs = self.parse_arrow_expr()?;
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

    /// `match scrut { P₁ => body₁ ; P₂ => body₂ }` — pattern match (`34 §3`).
    fn parse_match_expr(&mut self) -> Result<Expr, ElabError> {
        let start = self.peek_span().start;
        self.advance(); // consume 'match'
        let scrut = self.parse_app_expr()?;
        self.expect(&Token::LBrace)?;
        let mut arms = Vec::new();
        while !matches!(self.peek(), Token::RBrace | Token::Eof) {
            let arm_start = self.peek_span().start;
            let pat = self.parse_pattern()?;
            self.expect(&Token::FatArrow)?;
            let body = self.parse_expr()?;
            let arm_end = body.span().end;
            arms.push(MatchArm { pat, body, span: Span::new(arm_start, arm_end) });
            if matches!(self.peek(), Token::Semicolon) {
                self.advance();
            }
        }
        let end = self.peek_span().end;
        self.expect(&Token::RBrace)?;
        Ok(Expr::EMatch {
            scrut: Box::new(scrut),
            arms,
            span: Span::new(start, end),
        })
    }

    /// Parse a pattern: `C p₁…pₙ` | `_` | `x`.
    fn parse_pattern(&mut self) -> Result<Pattern, ElabError> {
        let start = self.peek_span().start;
        match self.peek().clone() {
            Token::ConId(name) => {
                let con_span = self.peek_span().clone();
                self.advance();
                let (name, _) = self.parse_dotted(name, con_span);
                // Collect atom-level sub-patterns (stop at `=>`, `|`, `}`, `;`, EOF).
                let mut sub = Vec::new();
                while self.can_start_atom_pat() {
                    sub.push(self.parse_atom_pattern()?);
                }
                let end = if sub.is_empty() {
                    self.tokens[self.pos - 1].1.end
                } else {
                    sub.last().unwrap().span.end
                };
                Ok(Pattern { kind: PatKind::Ctor(name, sub), span: Span::new(start, end) })
            }
            Token::Ident(name) => {
                let span = self.peek_span().clone();
                self.advance();
                let kind = if name == "_" { PatKind::Wild } else { PatKind::Var(name) };
                Ok(Pattern { kind, span })
            }
            other => Err(ElabError::ParseError {
                msg: format!("expected a pattern, found {:?}", other),
                span: self.peek_span().clone(),
            }),
        }
    }

    fn can_start_atom_pat(&self) -> bool {
        matches!(
            self.peek(),
            Token::Ident(_) | Token::ConId(_) | Token::LParen
        ) && !matches!(self.peek(), Token::FatArrow)
    }

    fn parse_atom_pattern(&mut self) -> Result<Pattern, ElabError> {
        let start = self.peek_span().start;
        match self.peek().clone() {
            Token::Ident(name) => {
                let span = self.peek_span().clone();
                self.advance();
                let kind = if name == "_" { PatKind::Wild } else { PatKind::Var(name) };
                Ok(Pattern { kind, span })
            }
            Token::ConId(name) => {
                // Atom constructor (no sub-patterns at this level without parens)
                let span = self.peek_span().clone();
                self.advance();
                let (name, span) = self.parse_dotted(name, span);
                Ok(Pattern { kind: PatKind::Ctor(name, vec![]), span })
            }
            Token::LParen => {
                self.advance();
                let inner = self.parse_pattern()?;
                let end = self.peek_span().end;
                self.expect(&Token::RParen)?;
                Ok(Pattern { kind: inner.kind, span: Span::new(start, end) })
            }
            other => Err(ElabError::ParseError {
                msg: format!("expected an atom pattern, found {:?}", other),
                span: self.peek_span().clone(),
            }),
        }
    }

    /// Parse an atom, then zero or more postfix `.field` projections
    /// (`33 §5.2` η — Σ-record field access on a class dictionary value).
    /// A `ConId`-headed atom already greedily consumed any `.segment`
    /// chain as part of a qualified module reference (`parse_dotted`,
    /// inside the `ConId` arm below), so this loop finds nothing left to
    /// consume there — it only fires for atoms that didn't already eat
    /// their own dots (`d.leq`, `(sort xs).leq`, etc).
    fn parse_atom_expr(&mut self) -> Result<Expr, ElabError> {
        let mut e = self.parse_atom_expr_base()?;
        while matches!(self.peek(), Token::Dot) && matches!(self.lookahead(1), Token::Ident(_)) {
            self.advance(); // consume '.'
            let (field, field_span) = match self.peek().clone() {
                Token::Ident(s) => {
                    self.advance();
                    (s, self.tokens[self.pos - 1].1.clone())
                }
                _ => unreachable!("guarded by lookahead above"),
            };
            let span = Span::new(e.span().start, field_span.end);
            e = Expr::EProj(Box::new(e), field, span);
        }
        Ok(e)
    }

    fn parse_atom_expr_base(&mut self) -> Result<Expr, ElabError> {
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
            Token::Str(s) => {
                let span = self.peek_span().clone();
                self.advance();
                Ok(Expr::EStr(s, span))
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
                let (name, span) = self.parse_dotted(s, span);
                Ok(Expr::ECon(name, span))
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
                        Expr::EStr(s, _) => Expr::EStr(s, span),
                        Expr::EBinOp(op, l, r, _) => Expr::EBinOp(op, l, r, span),
                        Expr::EMatch { scrut, arms, span: _ } => {
                            Expr::EMatch { scrut, arms, span }
                        }
                        Expr::EProj(e, field, _) => Expr::EProj(e, field, span),
                        Expr::EPi(x, a, b, _) => Expr::EPi(x, a, b, span),
                        Expr::EArrow(a, b, _) => Expr::EArrow(a, b, span),
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

/// Is `s` a contextual `temporal{}` operator word? (Atoms are idents that are
/// NOT one of these; `top`/`true` are atoms, not operators.) Pinning the
/// operator set here keeps the temporal grammar lexeme-free — only `temporal`
/// itself is a lexer keyword, so the grammar adds no global identifiers.
fn is_temporal_operator(s: &str) -> bool {
    matches!(s, "not" | "eventually" | "always" | "next" | "and" | "or" | "until" | "leadsto")
}

pub fn parse_decls(src: &str) -> Result<Vec<Decl>, ElabError> {
    let tokens = crate::lexer::Lexer::lex(src)?;
    Parser::new(tokens, src.to_string()).parse_decls()
}

pub fn parse_expr(src: &str) -> Result<Expr, ElabError> {
    let tokens = crate::lexer::Lexer::lex(src)?;
    Parser::new(tokens, src.to_string()).parse_expr_only()
}
