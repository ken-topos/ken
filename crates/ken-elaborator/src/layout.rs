//! Canonical structural layout (`31 §1d`, WP B3).
//!
//! The formatter consumes B1's typed, lossless source view.  The AST supplies
//! production boundaries and the token stream supplies protected lexemes; the
//! source is never re-lexed and notation spelling remains B2's responsibility.

use crate::ast::{BinOp, Decl, Expr, LetBinding, MatchArm, Type};
use crate::error::{ElabError, Span};
use crate::lexer::Token;
use crate::lossless::{parse_lossless, CommentPlacement, FormattableSource};

pub const CANONICAL_WIDTH: usize = 96;
pub const INDENT_WIDTH: usize = 2;

/// The deliberately small Wadler/Leijen document algebra used by kenfmt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Doc {
    Nil,
    Text(String),
    /// A space in flat mode and a newline in broken mode.
    Line,
    /// A newline in every mode.  Its presence makes a group non-flattenable.
    HardLine,
    Concat(Vec<Doc>),
    Nest(usize, Box<Doc>),
    Group(Box<Doc>),
    /// A group whose fit decision is limited to its own flattened contents.
    ///
    /// Ordinary groups include the pending line suffix in their decision.
    /// Locally fitted productions such as declaration signatures,
    /// applications, and parenthesized expressions instead own independent
    /// width decisions, so a fitting child stays horizontal when its parent
    /// must break.
    FitGroup(Box<Doc>),
}

impl Doc {
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text(text.into())
    }

    pub fn line() -> Self {
        Self::Line
    }

    pub fn hard_line() -> Self {
        Self::HardLine
    }

    pub fn concat(parts: impl IntoIterator<Item = Doc>) -> Self {
        let mut flat = Vec::new();
        for part in parts {
            match part {
                Doc::Nil => {}
                Doc::Concat(children) => flat.extend(children),
                other => flat.push(other),
            }
        }
        match flat.len() {
            0 => Doc::Nil,
            1 => flat.pop().unwrap(),
            _ => Doc::Concat(flat),
        }
    }

    pub fn nest(self, columns: usize) -> Self {
        Self::Nest(columns, Box::new(self))
    }

    pub fn group(self) -> Self {
        Self::Group(Box::new(self))
    }

    pub fn fit_group(self) -> Self {
        Self::FitGroup(Box::new(self))
    }

    pub fn append(self, other: Doc) -> Self {
        Self::concat([self, other])
    }

    /// Flatten soft lines. A hard line has no flat form.
    pub fn flatten(&self) -> Option<Doc> {
        match self {
            Doc::Nil => Some(Doc::Nil),
            Doc::Text(text) => Some(Doc::Text(text.clone())),
            Doc::Line => Some(Doc::text(" ")),
            Doc::HardLine => None,
            Doc::Concat(parts) => parts
                .iter()
                .map(Doc::flatten)
                .collect::<Option<Vec<_>>>()
                .map(Doc::concat),
            Doc::Nest(_, child) | Doc::Group(child) | Doc::FitGroup(child) => child.flatten(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Flat,
    Broken,
}

/// Render with the binary group rule: flatten iff the complete flattened
/// group fits the remaining display columns. There is no fill/packing pass.
pub fn render(doc: &Doc, width: usize) -> String {
    let mut output = String::new();
    let mut column = 0usize;
    let mut commands = vec![Command {
        indent: 0,
        mode: Mode::Broken,
        doc,
    }];
    while let Some(command) = commands.pop() {
        match command.doc {
            Doc::Nil => {}
            Doc::Text(text) => {
                output.push_str(text);
                column += display_width(text);
            }
            Doc::Line if command.mode == Mode::Flat => {
                output.push(' ');
                column += 1;
            }
            Doc::Line | Doc::HardLine => {
                while output.ends_with(' ') || output.ends_with('\t') {
                    output.pop();
                }
                output.push('\n');
                output.extend(std::iter::repeat(' ').take(command.indent));
                column = command.indent;
            }
            Doc::Concat(parts) => {
                for part in parts.iter().rev() {
                    commands.push(Command {
                        indent: command.indent,
                        mode: command.mode,
                        doc: part,
                    });
                }
            }
            Doc::Nest(extra, child) => commands.push(Command {
                indent: command.indent + extra,
                mode: command.mode,
                doc: child,
            }),
            Doc::Group(child) => {
                let can_flatten = child.flatten().is_some();
                let mut probe = commands.clone();
                probe.push(Command {
                    indent: command.indent,
                    mode: Mode::Flat,
                    doc: child,
                });
                commands.push(Command {
                    indent: command.indent,
                    mode: if can_flatten && fits(width.saturating_sub(column), probe) {
                        Mode::Flat
                    } else {
                        Mode::Broken
                    },
                    doc: child,
                });
            }
            Doc::FitGroup(child) if command.mode == Mode::Flat => commands.push(Command {
                indent: command.indent,
                mode: Mode::Flat,
                doc: child,
            }),
            Doc::FitGroup(child) => {
                let punctuation_suffix = pending_punctuation_width(&commands);
                let fits_locally =
                    child
                        .flatten()
                        .as_ref()
                        .and_then(flat_width)
                        .is_some_and(|flat| {
                            flat <= width.saturating_sub(column.saturating_add(punctuation_suffix))
                        });
                commands.push(Command {
                    indent: command.indent,
                    mode: if fits_locally {
                        Mode::Flat
                    } else {
                        Mode::Broken
                    },
                    doc: child,
                });
            }
        }
    }
    output
}

fn pending_punctuation_width(commands: &[Command<'_>]) -> usize {
    commands
        .iter()
        .rev()
        .map_while(|command| flat_punctuation_width(command.doc))
        .sum()
}

fn flat_punctuation_width(doc: &Doc) -> Option<usize> {
    match doc {
        Doc::Nil => Some(0),
        Doc::Text(text)
            if text
                .chars()
                .all(|ch| matches!(ch, ')' | ']' | '}' | ',' | ';' | '=' | ' ' | '\t')) =>
        {
            Some(display_width(text))
        }
        Doc::Concat(parts) => parts.iter().try_fold(0usize, |sum, part| {
            Some(sum + flat_punctuation_width(part)?)
        }),
        Doc::Nest(_, child) | Doc::Group(child) | Doc::FitGroup(child) => {
            flat_punctuation_width(child)
        }
        _ => None,
    }
}

#[derive(Clone)]
struct Command<'a> {
    indent: usize,
    mode: Mode,
    doc: &'a Doc,
}

fn fits(mut remaining: usize, mut commands: Vec<Command<'_>>) -> bool {
    while let Some(command) = commands.pop() {
        match command.doc {
            Doc::Nil => {}
            Doc::Text(text) => {
                let needed = display_width(text);
                if needed > remaining {
                    return false;
                }
                remaining -= needed;
            }
            Doc::Line if command.mode == Mode::Flat => {
                if remaining == 0 {
                    return false;
                }
                remaining -= 1;
            }
            Doc::Line | Doc::HardLine => return true,
            Doc::Concat(parts) => {
                for part in parts.iter().rev() {
                    commands.push(Command {
                        indent: command.indent,
                        mode: command.mode,
                        doc: part,
                    });
                }
            }
            Doc::Nest(extra, child) => commands.push(Command {
                indent: command.indent + extra,
                mode: command.mode,
                doc: child,
            }),
            Doc::Group(child) | Doc::FitGroup(child) => commands.push(Command {
                indent: command.indent,
                mode: command.mode,
                doc: child,
            }),
        }
    }
    true
}

fn flat_width(doc: &Doc) -> Option<usize> {
    match doc {
        Doc::Nil => Some(0),
        Doc::Text(text) => Some(display_width(text)),
        Doc::Line => Some(1),
        Doc::HardLine => None,
        Doc::Concat(parts) => parts
            .iter()
            .try_fold(0usize, |sum, part| Some(sum + flat_width(part)?)),
        Doc::Nest(_, child) | Doc::Group(child) | Doc::FitGroup(child) => flat_width(child),
    }
}

/// Unicode terminal-column measurement. Ken's blessed notation is narrow;
/// combining marks are zero-width and the standard East-Asian ranges are two.
pub fn display_width(text: &str) -> usize {
    text.chars().map(char_display_width).sum()
}

fn char_display_width(ch: char) -> usize {
    let cp = ch as u32;
    if ch == '\n' || ch == '\r' || ch == '\t' || ch.is_control() {
        return 0;
    }
    if matches!(cp,
        0x0300..=0x036f | 0x0483..=0x0489 | 0x0591..=0x05bd |
        0x05bf | 0x05c1..=0x05c2 | 0x05c4..=0x05c5 | 0x0610..=0x061a |
        0x064b..=0x065f | 0x0670 | 0x06d6..=0x06ed | 0x1ab0..=0x1aff |
        0x1dc0..=0x1dff | 0x20d0..=0x20ff | 0xfe20..=0xfe2f)
    {
        return 0;
    }
    if matches!(cp,
        0x1100..=0x115f | 0x2329..=0x232a | 0x2e80..=0xa4cf |
        0xac00..=0xd7a3 | 0xf900..=0xfaff | 0xfe10..=0xfe19 |
        0xfe30..=0xfe6f | 0xff00..=0xff60 | 0xffe0..=0xffe6 |
        0x1f300..=0x1faff | 0x20000..=0x3fffd)
    {
        2
    } else {
        1
    }
}

/// Format a parsed B1 source into the unique `31 §1d` layout.
pub fn format_source(source: &dyn FormattableSource) -> String {
    LayoutPrinter::new(source).format()
}

/// Parse and format one Ken compilation unit.
pub fn format_ken(source: &str) -> Result<String, ElabError> {
    let parsed = parse_lossless(source)?;
    Ok(format_source(parsed.as_ref()))
}

struct LayoutPrinter<'a> {
    source: &'a dyn FormattableSource,
}

impl<'a> LayoutPrinter<'a> {
    fn new(source: &'a dyn FormattableSource) -> Self {
        Self { source }
    }

    fn format(&self) -> String {
        let docs = self
            .source
            .typed_decls()
            .iter()
            .map(|decl| self.print_decl(decl))
            .collect::<Vec<_>>();
        let mut out = render(
            &join(docs, Doc::concat([Doc::hard_line(), Doc::hard_line()])),
            CANONICAL_WIDTH,
        );
        while out.ends_with([' ', '\t', '\n']) {
            out.pop();
        }
        out.push('\n');
        out
    }

    /// Declaration production. Structure-sensitive blocks are emitted here;
    /// expression/type sub-productions retain their own span boundaries.
    fn print_decl(&self, decl: &Decl) -> Doc {
        if let Decl::Pub(inner) = decl {
            let core = Doc::concat([Doc::text("pub "), self.print_decl(inner)]);
            return self.with_comments(decl.span(), core);
        }
        if let Some(span) = self.nonempty_outer_decl_block_span(decl) {
            let recursive_children = match decl {
                Decl::ModuleDecl { decls, .. } => {
                    Some(decls.iter().map(|child| self.print_decl(child)).collect())
                }
                _ => None,
            };
            let core = self.print_hard_line_decl_block(decl, span, recursive_children);
            return self.with_comments(decl.span(), core);
        }
        let core = match decl {
            Decl::DataDecl { ctors, span, .. }
                if ctors.len() > 1 || ctors.iter().any(|ctor| !ctor.args.is_empty()) =>
            {
                self.print_sum(
                    decl,
                    span,
                    ctors.iter().map(|ctor| ctor.span.clone()).collect(),
                )
            }
            Decl::ViewDecl { body, span, .. }
            | Decl::LetDecl {
                val: body, span, ..
            }
            | Decl::LemmaDecl { body, span, .. }
            | Decl::AttachedProofDecl { body, span, .. } => {
                self.print_decl_with_body(decl, span, body)
            }
            Decl::DataDecl { span, .. } | Decl::AxiomDecl { span, .. } => {
                self.print_decl_signature(decl, span)
            }
            _ => self.print_span(decl.span()),
        };
        self.with_comments(decl.span(), core)
    }

    /// Expression production. It is intentionally separate from token block
    /// layout so precedence and mandatory match breaks have one owner.
    fn print_expr(&self, expr: &Expr) -> Doc {
        let reconstructed = matches!(
            expr,
            Expr::EMatch { .. }
                | Expr::ELam(_, _, _)
                | Expr::ELet(_, _, _)
                | Expr::EApp(_, _, _)
                | Expr::EArrow(_, _, _)
        );
        let doc = match expr {
            Expr::EMatch { span, arms, .. } => self.print_match(span, arms),
            Expr::ELam(_, body, span) => self.print_lambda(span, body),
            Expr::ELet(bindings, body, span) => self.print_let(span, bindings, body),
            Expr::EApp(_, _, _) => self.print_application(expr, false),
            Expr::EArrow(left, right, _) => self.print_arrow(left, right),
            Expr::EAsc(_, _, span)
            | Expr::EOld(_, span)
            | Expr::EBinOp(_, _, _, span)
            | Expr::EProj(_, _, span)
            | Expr::EPi(_, _, _, span) => self.print_span(span),
            _ => self.print_span(expr.span()),
        };
        if reconstructed {
            self.with_source_parens(expr.span(), doc)
        } else if self.span_has_outer_parens(expr.span()) {
            doc.fit_group()
        } else {
            doc
        }
    }

    /// Type production. Arrow/application break opportunities are derived
    /// from parsed tokens, while the AST retains precedence ownership.
    #[allow(dead_code)]
    fn print_type(&self, ty: &Type) -> Doc {
        match ty {
            Type::TPi(_, _, _, span)
            | Type::TArr(_, _, span)
            | Type::TEffectArr(_, _, _, span)
            | Type::TApp(_, _, span)
            | Type::TRefine(_, _, _, span) => self.print_span(span),
            _ => self.print_span(ty.span()),
        }
    }

    /// The single entry path for §1d nonempty declaration blocks. Most block
    /// siblings are token-delimited; modules supply recursively printed child
    /// declarations so compound bodies retain their production-specific docs.
    fn print_hard_line_decl_block(
        &self,
        decl: &Decl,
        span: &Span,
        children: Option<Vec<Doc>>,
    ) -> Doc {
        let Some(children) = children else {
            return self.print_token_decl_block(decl, span);
        };
        let indices = self.token_indices(span);
        let Some(open_pos) = indices
            .iter()
            .position(|index| matches!(self.source.tokens()[*index].kind, Token::LBrace))
        else {
            return self.doc_from_tokens(span, TokenLayout::Block);
        };
        let Some(close_pos) = matching_rbrace(self.source, &indices, open_pos) else {
            return self.doc_from_tokens(span, TokenLayout::Block);
        };
        let head_span = Span::new(span.start, self.source.tokens()[indices[open_pos]].span.end);
        let close = &self.source.tokens()[indices[close_pos]].span;
        let children = if let Decl::ModuleDecl { decls, .. } = decl {
            children
                .into_iter()
                .zip(decls)
                .enumerate()
                .map(|(position, (child, child_decl))| {
                    let limit = decls
                        .get(position + 1)
                        .map_or(close.start, |next| next.span().start);
                    if self.has_token_between(Token::Semicolon, child_decl.span().end, limit) {
                        Doc::concat([child, Doc::text(";")])
                    } else {
                        child
                    }
                })
                .collect()
        } else {
            children
        };
        Doc::concat([
            self.print_decl_signature(decl, &head_span),
            Doc::concat([Doc::hard_line(), join(children, Doc::hard_line())]).nest(INDENT_WIDTH),
            Doc::hard_line(),
            Doc::text(&self.source.source()[close.start..close.end]),
        ])
    }

    /// Recognize a declaration block from its typed production boundary and
    /// outer brace pair, rather than from a list of block keywords. The
    /// negative cases own expression/type/verbatim braces, so a future parsed
    /// declaration-block variant automatically takes this path.
    fn nonempty_outer_decl_block_span<'d>(&self, decl: &'d Decl) -> Option<&'d Span> {
        if decl_owns_non_block_braces(decl) {
            return None;
        }
        let span = decl.span();
        let indices = self.token_indices(span);
        indices.iter().enumerate().find_map(|(position, index)| {
            if !matches!(self.source.tokens()[*index].kind, Token::LBrace) {
                return None;
            }
            let close = matching_rbrace(self.source, &indices, position)?;
            (close + 1 == indices.len() && close > position + 1).then_some(span)
        })
    }

    fn print_sum(&self, decl: &Decl, span: &Span, ctors: Vec<Span>) -> Doc {
        let mut complete = span.clone();
        loop {
            let next = self.source.tokens().iter().find(|token| {
                token.span.start == complete.end && token.span.start != token.span.end
            });
            match next {
                Some(token) if matches!(token.kind, Token::RParen) => {
                    complete.end = token.span.end;
                }
                _ => break,
            }
        }
        let indices = self.token_indices(&complete);
        let Some(eq_pos) = indices
            .iter()
            .rposition(|index| matches!(self.source.tokens()[*index].kind, Token::Eq))
        else {
            return self.doc_from_tokens(&complete, TokenLayout::Sum);
        };
        let head = Span::new(
            complete.start,
            self.source.tokens()[indices[eq_pos]].span.end,
        );
        let ctor_docs = ctors
            .iter()
            .enumerate()
            .map(|(position, ctor)| {
                let effective = if position + 1 == ctors.len() {
                    Span::new(ctor.start, complete.end)
                } else {
                    ctor.clone()
                };
                let ctor = self.doc_from_tokens(&effective, TokenLayout::Soft);
                if position == 0 {
                    ctor
                } else {
                    Doc::concat([Doc::text("| "), ctor])
                }
            })
            .collect();
        Doc::concat([
            self.print_decl_signature(decl, &head),
            Doc::concat([Doc::line(), join(ctor_docs, Doc::line())]).nest(INDENT_WIDTH),
        ])
        .group()
    }

    fn print_decl_with_body(&self, decl: &Decl, span: &Span, body: &Expr) -> Doc {
        let Some(eq) = self.last_token_index_before(Token::Eq, body.span().start, span) else {
            return self.print_span(span);
        };
        let head_span = Span::new(span.start, self.source.tokens()[eq].span.end);
        let separator = if is_compound_expr(body) || self.has_comment_within(body.span()) {
            Doc::hard_line()
        } else {
            Doc::line()
        };
        Doc::concat([
            self.print_decl_signature(decl, &head_span),
            Doc::concat([separator, self.print_expr(body)]).nest(INDENT_WIDTH),
        ])
        .group()
    }

    /// Render a declaration signature as an independent horizontal unit when
    /// it fits. Its broken form follows the operator's readability ladder:
    /// parameters at +6, clause markers at +4, and clause continuations at +6.
    /// The declaration body is owned by `print_decl_with_body` and remains at
    /// +2, so no signature continuation can be mistaken for body content.
    fn print_decl_signature(&self, decl: &Decl, span: &Span) -> Doc {
        let indices = self.token_indices(span);
        if indices.is_empty() {
            return Doc::Nil;
        }
        let ranges = self.decl_binder_ranges(decl, &indices);
        let clauses = signature_clause_ranges(self.source, &indices);
        let prefix_end = ranges
            .first()
            .map(|(start, _)| *start)
            .into_iter()
            .chain(clauses.first().map(|(start, _)| *start))
            .min();
        let Some(prefix_end) = prefix_end else {
            return self.doc_from_tokens(span, TokenLayout::Soft);
        };
        if prefix_end == 0 {
            return self.doc_from_tokens(span, TokenLayout::Soft);
        }

        let prefix = self.grouped_token_slice(&indices[..prefix_end]);
        let mut signature = vec![prefix];

        if !ranges.is_empty() {
            let mut parameter_docs = Vec::new();
            let mut previous_end = None;
            for (start, end) in &ranges {
                if let Some(previous_end) = previous_end {
                    parameter_docs.push(self.token_boundary(
                        indices[previous_end - 1],
                        indices[*start],
                        Doc::line(),
                    ));
                }
                parameter_docs.push(self.grouped_token_slice(&indices[*start..*end]));
                previous_end = Some(*end);
            }
            let parameter_end = ranges.last().unwrap().1;
            let next_clause = clauses.first().map_or(indices.len(), |(start, _)| *start);
            if parameter_end < next_clause {
                let tail = self.grouped_token_slice(&indices[parameter_end..next_clause]);
                parameter_docs.push(self.token_boundary(
                    indices[parameter_end - 1],
                    indices[parameter_end],
                    Doc::text(" "),
                ));
                parameter_docs.push(tail);
            }
            signature.push(
                Doc::concat([Doc::line(), Doc::concat(parameter_docs).fit_group()])
                    .nest(INDENT_WIDTH * 3),
            );
        }

        for (start, end) in clauses {
            signature.push(
                Doc::concat([
                    Doc::line(),
                    self.print_signature_clause(&indices[start..end]),
                ])
                .nest(INDENT_WIDTH * 2),
            );
        }

        Doc::concat(signature).fit_group()
    }

    fn print_signature_clause(&self, indices: &[usize]) -> Doc {
        if indices.len() < 2 {
            return self.grouped_token_slice(indices);
        }
        let boundary = self.token_boundary(indices[0], indices[1], Doc::text(" "));
        let continuation = if matches!(self.source.tokens()[indices[0]].kind, Token::Colon) {
            self.print_return_type(&indices[1..])
        } else {
            self.grouped_token_slice(&indices[1..]).fit_group()
        };
        Doc::concat([
            Doc::text(self.token_text(indices[0])),
            boundary,
            continuation.nest(INDENT_WIDTH),
        ])
        .fit_group()
    }

    /// Preserve arrow chains as the return type's outer break structure. When
    /// the chain breaks, each arrow and its operand remain a locally fitted
    /// unit; only an operand that is itself too wide may break internally.
    fn print_return_type(&self, indices: &[usize]) -> Doc {
        let mut starts = vec![0usize];
        let mut paren_depth = 0usize;
        let mut bracket_depth = 0usize;
        let mut brace_depth = 0usize;
        for (position, index) in indices.iter().copied().enumerate() {
            let token = &self.source.tokens()[index].kind;
            if matches!(token, Token::Arrow)
                && paren_depth == 0
                && bracket_depth == 0
                && brace_depth == 0
            {
                starts.push(position);
            }
            match token {
                Token::LParen => paren_depth += 1,
                Token::RParen => paren_depth = paren_depth.saturating_sub(1),
                Token::LBracket => bracket_depth += 1,
                Token::RBracket => bracket_depth = bracket_depth.saturating_sub(1),
                Token::LBrace => brace_depth += 1,
                Token::RBrace => brace_depth = brace_depth.saturating_sub(1),
                _ => {}
            }
        }
        if starts.len() == 1 {
            return self.grouped_token_slice(indices).fit_group();
        }

        let mut operands = Vec::new();
        for (position, start) in starts.iter().copied().enumerate() {
            let end = starts.get(position + 1).copied().unwrap_or(indices.len());
            if position > 0 {
                operands.push(self.token_boundary(indices[start - 1], indices[start], Doc::line()));
            }
            operands.push(self.grouped_token_slice(&indices[start..end]).fit_group());
        }
        Doc::concat(operands).fit_group()
    }

    fn print_token_decl_block(&self, decl: &Decl, span: &Span) -> Doc {
        let indices = self.token_indices(span);
        let Some(open_pos) = indices
            .iter()
            .position(|index| matches!(self.source.tokens()[*index].kind, Token::LBrace))
        else {
            return self.doc_from_tokens(span, TokenLayout::Block);
        };
        let Some(close_pos) = matching_rbrace(self.source, &indices, open_pos) else {
            return self.doc_from_tokens(span, TokenLayout::Block);
        };
        if close_pos == open_pos + 1 {
            return self.doc_from_tokens(span, TokenLayout::Block);
        }

        let head_span = Span::new(span.start, self.source.tokens()[indices[open_pos]].span.end);
        let inner = &indices[open_pos + 1..close_pos];
        let inner_span = Span::new(
            self.source.tokens()[inner[0]].span.start,
            self.source.tokens()[*inner.last().unwrap()].span.end,
        );
        Doc::concat([
            self.print_decl_signature(decl, &head_span),
            Doc::concat([
                Doc::hard_line(),
                self.print_block_inner(inner, &inner_span, TokenLayout::Block),
            ])
            .nest(INDENT_WIDTH),
            Doc::hard_line(),
            Doc::text("}"),
        ])
    }

    fn grouped_token_slice(&self, indices: &[usize]) -> Doc {
        if matches!(self.source.tokens()[indices[0]].kind, Token::LParen)
            && matching_rparen(self.source, indices, 0) == Some(indices.len() - 1)
        {
            if indices.len() == 2 {
                return Doc::text("()");
            }
            let before = self.token_boundary(indices[0], indices[1], Doc::Nil);
            let after = self.token_boundary(
                indices[indices.len() - 2],
                indices[indices.len() - 1],
                Doc::Nil,
            );
            return Doc::concat([
                Doc::text(self.token_text(indices[0])),
                before,
                self.consistent_token_slice(&indices[1..indices.len() - 1]),
                after,
                Doc::text(self.token_text(indices[indices.len() - 1])),
            ])
            .fit_group();
        }

        let mut segments = Vec::new();
        let mut start = 0usize;
        let mut position = 0usize;
        while position < indices.len() {
            if matches!(self.source.tokens()[indices[position]].kind, Token::LParen) {
                let Some(close) = matching_rparen(self.source, indices, position) else {
                    break;
                };
                if start < position {
                    segments.push((start, position));
                }
                segments.push((position, close + 1));
                start = close + 1;
                position = close + 1;
            } else {
                position += 1;
            }
        }
        if start < indices.len() {
            segments.push((start, indices.len()));
        }
        if segments.len() <= 1 {
            return self.raw_grouped_token_slice(indices);
        }

        let (head_start, head_end) = segments[0];
        let head = self.raw_grouped_token_slice(&indices[head_start..head_end]);
        let mut continuation = Vec::new();
        let mut previous_end = head_end;
        for (start, end) in segments.into_iter().skip(1) {
            let left = &self.source.tokens()[indices[previous_end - 1]].kind;
            let right = &self.source.tokens()[indices[start]].kind;
            let separator = if soft_break_between(left, right, TokenLayout::Soft) {
                Doc::line()
            } else if needs_space(left, right) {
                Doc::text(" ")
            } else {
                Doc::Nil
            };
            continuation.push(self.token_boundary(
                indices[previous_end - 1],
                indices[start],
                separator,
            ));
            continuation.push(self.raw_grouped_token_slice(&indices[start..end]));
            previous_end = end;
        }
        Doc::concat([head, Doc::concat(continuation).nest(INDENT_WIDTH)]).fit_group()
    }

    /// Build a recursive application/arrow group inside a delimited child.
    /// Unlike the declaration-level R1 return ladder, every top-level atom in
    /// this local group is an application child and therefore owns a +2 break.
    fn consistent_token_slice(&self, indices: &[usize]) -> Doc {
        let mut starts = vec![0usize];
        let mut paren_depth = 0usize;
        let mut bracket_depth = 0usize;
        let mut brace_depth = 0usize;
        for position in 0..indices.len() {
            if position > 0 && paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 {
                let left = &self.source.tokens()[indices[position - 1]].kind;
                let right = &self.source.tokens()[indices[position]].kind;
                if needs_space(left, right) && soft_break_between(left, right, TokenLayout::Soft) {
                    starts.push(position);
                }
            }
            match &self.source.tokens()[indices[position]].kind {
                Token::LParen => paren_depth += 1,
                Token::RParen => paren_depth = paren_depth.saturating_sub(1),
                Token::LBracket => bracket_depth += 1,
                Token::RBracket => bracket_depth = bracket_depth.saturating_sub(1),
                Token::LBrace => brace_depth += 1,
                Token::RBrace => brace_depth = brace_depth.saturating_sub(1),
                _ => {}
            }
        }
        if starts.len() == 1 {
            return self.raw_grouped_token_slice(indices);
        }

        let head_end = starts[1];
        let head = self.raw_grouped_token_slice(&indices[..head_end]);
        let mut continuation = Vec::new();
        let mut previous_end = head_end;
        for (position, start) in starts.iter().copied().enumerate().skip(1) {
            let end = starts.get(position + 1).copied().unwrap_or(indices.len());
            continuation.push(self.token_boundary(
                indices[previous_end - 1],
                indices[start],
                Doc::line(),
            ));
            continuation.push(self.raw_grouped_token_slice(&indices[start..end]));
            previous_end = end;
        }
        Doc::concat([head, Doc::concat(continuation).nest(INDENT_WIDTH)]).fit_group()
    }

    fn raw_grouped_token_slice(&self, indices: &[usize]) -> Doc {
        let mut ranges = Vec::new();
        let mut position = 0usize;
        while position < indices.len() {
            if matches!(self.source.tokens()[indices[position]].kind, Token::LParen) {
                let Some(close) = matching_rparen(self.source, indices, position) else {
                    break;
                };
                ranges.push((position, close + 1));
                position = close + 1;
            } else {
                position += 1;
            }
        }
        if !ranges.is_empty() {
            let mut parts = Vec::new();
            let mut start = 0usize;
            let mut previous_end: Option<usize> = None;
            for (open, close) in ranges {
                if start < open {
                    if let Some(previous) = previous_end {
                        parts.push(self.token_boundary(
                            indices[previous - 1],
                            indices[start],
                            token_spacing(
                                &self.source.tokens()[indices[previous - 1]].kind,
                                &self.source.tokens()[indices[start]].kind,
                            ),
                        ));
                    }
                    parts.push(self.raw_grouped_token_slice(&indices[start..open]));
                    previous_end = Some(open);
                }
                if let Some(previous) = previous_end {
                    parts.push(self.token_boundary(
                        indices[previous - 1],
                        indices[open],
                        token_spacing(
                            &self.source.tokens()[indices[previous - 1]].kind,
                            &self.source.tokens()[indices[open]].kind,
                        ),
                    ));
                }
                parts.push(self.grouped_token_slice(&indices[open..close]));
                previous_end = Some(close);
                start = close;
            }
            if start < indices.len() {
                if let Some(previous) = previous_end {
                    parts.push(self.token_boundary(
                        indices[previous - 1],
                        indices[start],
                        token_spacing(
                            &self.source.tokens()[indices[previous - 1]].kind,
                            &self.source.tokens()[indices[start]].kind,
                        ),
                    ));
                }
                parts.push(self.raw_grouped_token_slice(&indices[start..]));
            }
            return Doc::concat(parts).fit_group();
        }
        let span = Span::new(
            self.source.tokens()[indices[0]].span.start,
            self.source.tokens()[*indices.last().unwrap()].span.end,
        );
        self.doc_token_slice(indices, &span, TokenLayout::Soft)
            .group()
    }

    fn token_boundary(&self, left: usize, right: usize, default: Doc) -> Doc {
        let start = self.source.tokens()[left].span.end;
        let end = self.source.tokens()[right].span.start;
        let owner = Span::new(start, end);
        let mut comments = Vec::new();
        self.push_comments_between(&mut comments, start, end, &owner);
        if comments.is_empty() {
            default
        } else {
            Doc::concat(comments)
        }
    }

    fn decl_binder_ranges(&self, decl: &Decl, indices: &[usize]) -> Vec<(usize, usize)> {
        let spans: Vec<&Span> = match decl {
            Decl::ViewDecl { params, .. }
            | Decl::PropDecl { params, .. }
            | Decl::LemmaDecl { params, .. }
            | Decl::AttachedProofDecl { params, .. }
            | Decl::ExplicitDataDecl { params, .. } => {
                params.iter().map(|binder| &binder.span).collect()
            }
            _ => Vec::new(),
        };
        let mut ranges = spans
            .into_iter()
            .filter_map(|span| token_range_for_span(self.source, indices, span))
            .collect::<Vec<_>>();

        if ranges.is_empty() {
            match decl {
                Decl::DataDecl { type_params, .. } if !type_params.is_empty() => {
                    let eq = indices
                        .iter()
                        .position(|index| matches!(self.source.tokens()[*index].kind, Token::Eq));
                    let limit = eq.unwrap_or(indices.len());
                    ranges.extend(
                        (2..limit)
                            .filter(|position| {
                                matches!(
                                    self.source.tokens()[indices[*position]].kind,
                                    Token::Ident(_)
                                )
                            })
                            .take(type_params.len())
                            .map(|position| (position, position + 1)),
                    );
                }
                Decl::ClassDecl { param: Some(_), .. } => {
                    if let Some(range) = first_top_level_paren_range(self.source, indices, 2) {
                        ranges.push(range);
                    } else if indices.len() > 2 {
                        ranges.push((2, 3));
                    }
                }
                _ => {}
            }
        }
        ranges.sort_unstable();
        ranges
    }

    fn print_match(&self, span: &Span, arms: &[MatchArm]) -> Doc {
        if arms.is_empty() {
            return self.doc_from_tokens(span, TokenLayout::Soft);
        }
        // Reaching this production means either the match is compound itself
        // or it is nested in a compound parent. Both are mandatory-break cases.
        let indices = self.token_indices(span);
        let Some(open_pos) = indices
            .iter()
            .position(|index| matches!(self.source.tokens()[*index].kind, Token::LBrace))
        else {
            return self.doc_from_tokens(span, TokenLayout::Block);
        };
        let match_start = indices
            .iter()
            .find(|index| matches!(self.source.tokens()[**index].kind, Token::KwMatch))
            .map_or(span.start, |index| self.source.tokens()[*index].span.start);
        let head = Span::new(
            match_start,
            self.source.tokens()[indices[open_pos]].span.end,
        );
        let arm_docs: Vec<_> = arms
            .iter()
            .enumerate()
            .map(|(index, arm)| {
                let doc = self.print_match_arm(arm);
                let limit = arms.get(index + 1).map_or(span.end, |next| next.span.start);
                if self.has_token_between(Token::Semicolon, arm.span.end, limit) {
                    Doc::concat([doc, Doc::text(";")]).group()
                } else {
                    doc
                }
            })
            .collect();
        Doc::concat([
            self.doc_from_tokens(&head, TokenLayout::Soft),
            Doc::concat([Doc::hard_line(), join(arm_docs, Doc::hard_line())]).nest(INDENT_WIDTH),
            Doc::hard_line(),
            Doc::text("}"),
        ])
    }

    fn print_match_arm(&self, arm: &MatchArm) -> Doc {
        let indices = self.token_indices(&arm.span);
        let Some(arrow_pos) = indices.iter().rposition(|index| {
            matches!(self.source.tokens()[*index].kind, Token::MapsTo)
                && self.source.tokens()[*index].span.end <= arm.body.span().start
        }) else {
            return self.doc_from_tokens(&arm.span, TokenLayout::Soft);
        };
        let head = Span::new(
            arm.span.start,
            self.source.tokens()[indices[arrow_pos]].span.end,
        );
        let compound = matches!(arm.body, Expr::EMatch { .. })
            || is_let_chain(&arm.body)
            || is_compound_expr(&arm.body);
        if compound {
            Doc::concat([
                self.doc_from_tokens(&head, TokenLayout::Soft),
                Doc::concat([Doc::hard_line(), self.print_expr(&arm.body)]).nest(INDENT_WIDTH),
            ])
        } else {
            Doc::concat([
                self.doc_from_tokens(&head, TokenLayout::Soft),
                Doc::text(" "),
                self.print_expr(&arm.body),
            ])
            .group()
        }
    }

    fn print_let(&self, _span: &Span, bindings: &[LetBinding], body: &Expr) -> Doc {
        self.print_let_bindings(bindings, body)
    }

    fn print_let_bindings(&self, bindings: &[LetBinding], body: &Expr) -> Doc {
        self.print_let_bindings_from(bindings, body, None)
    }

    fn print_let_bindings_from(
        &self,
        bindings: &[LetBinding],
        body: &Expr,
        synthetic_start: Option<usize>,
    ) -> Doc {
        let (segment, tail) = collect_let_segment(bindings, body);
        let source_let_span = if synthetic_start.is_none() {
            segment.first().and_then(|first| {
                self.source.tokens().iter().rev().find_map(|token| {
                    (token.span.end <= first.name_span.start && matches!(token.kind, Token::KwLet))
                        .then_some(token.span.clone())
                })
            })
        } else {
            None
        };
        let let_start = synthetic_start.or_else(|| source_let_span.as_ref().map(|span| span.start));
        let has_comments = let_start.is_some_and(|start| {
            self.source.comment_attachments().iter().any(|attachment| {
                start <= attachment.comment_span.start
                    && attachment.comment_span.end <= body.span().end
            })
        });
        if segment.len() == 1 && matches!(tail, LetTail::Expr(_)) && !has_comments {
            let binding = segment[0];
            let body_doc = match tail {
                LetTail::Expr(expr) => self.print_expr(expr),
                LetTail::Group(..) => unreachable!(),
            };
            let head = Span::new(binding.span.start, binding.value.span().start);
            return Doc::concat([
                Doc::text("let "),
                self.doc_from_tokens(&head, TokenLayout::Soft),
                Doc::concat([Doc::line(), self.print_expr(&binding.value)]).nest(INDENT_WIDTH),
                Doc::line(),
                Doc::text("in"),
                Doc::concat([Doc::line(), body_doc]).nest(INDENT_WIDTH),
            ])
            .group();
        }
        let mut binding_docs = Vec::with_capacity(segment.len());
        if let Some(first) = segment.first() {
            let let_end = synthetic_start
                .or_else(|| source_let_span.as_ref().map(|span| span.end))
                .unwrap_or(first.name_span.start);
            binding_docs.extend(self.let_comment_docs(
                let_end,
                first.name_span.start,
                &[CommentPlacement::Leading, CommentPlacement::Interstitial],
            ));
        }
        for (index, binding) in segment.iter().enumerate() {
            binding_docs.push(self.print_let_binding(binding));
            if index + 1 < segment.len() {
                binding_docs.push(Doc::text(";"));
                let next = segment[index + 1];
                let trailing = self.let_comment_docs(
                    binding.value.span().end,
                    next.name_span.start,
                    &[CommentPlacement::Trailing],
                );
                if trailing.is_empty() {
                    binding_docs.push(Doc::line());
                } else {
                    binding_docs.push(Doc::text("  "));
                    binding_docs.extend(trailing);
                }
                binding_docs.extend(self.let_comment_docs(
                    binding.value.span().end,
                    next.name_span.start,
                    &[CommentPlacement::Leading, CommentPlacement::Interstitial],
                ));
            }
        }
        let tail_start = match tail {
            LetTail::Expr(expr) => expr.span().start,
            LetTail::Group(rest, _) => rest[0].name_span.start,
        };
        let last_value_end = segment.last().unwrap().value.span().end;
        let trailing_binding_comments =
            self.let_comment_docs(last_value_end, tail_start, &[CommentPlacement::Trailing]);
        let leading_body_comments = self.let_comment_docs(
            last_value_end,
            tail_start,
            &[CommentPlacement::Leading, CommentPlacement::Interstitial],
        );
        let body_doc = match tail {
            LetTail::Expr(expr) => self.print_expr(expr),
            LetTail::Group(rest, tail_body) => {
                self.print_let_bindings_from(rest, tail_body, Some(rest[0].name_span.start))
            }
        };
        let body_boundary = match tail {
            LetTail::Expr(expr) if !is_compound_expr(expr) => Doc::line(),
            LetTail::Expr(_) | LetTail::Group(..) => Doc::hard_line(),
        };
        Doc::concat([
            Doc::text("let"),
            Doc::concat([Doc::line(), Doc::concat(binding_docs)]).nest(INDENT_WIDTH),
            if trailing_binding_comments.is_empty() {
                Doc::Nil
            } else {
                Doc::concat([Doc::text("  "), Doc::concat(trailing_binding_comments)])
            },
            Doc::line(),
            Doc::text("in"),
            Doc::concat([body_boundary, Doc::concat(leading_body_comments), body_doc])
                .nest(INDENT_WIDTH),
        ])
        .group()
    }

    fn print_let_binding(&self, binding: &LetBinding) -> Doc {
        let head = Span::new(binding.span.start, binding.value.span().start);
        let head_doc = self.doc_from_tokens(&head, TokenLayout::Soft);
        if is_compound_expr(&binding.value) || self.has_comment_within(&binding.span) {
            Doc::concat([
                head_doc,
                Doc::concat([Doc::hard_line(), self.print_expr(&binding.value)]).nest(INDENT_WIDTH),
            ])
        } else {
            Doc::concat([
                head_doc,
                Doc::concat([Doc::line(), self.print_expr(&binding.value)]).nest(INDENT_WIDTH),
            ])
            .fit_group()
        }
    }

    fn let_comment_docs(
        &self,
        start: usize,
        end: usize,
        placements: &[CommentPlacement],
    ) -> Vec<Doc> {
        self.source
            .comment_attachments()
            .iter()
            .filter(|attachment| {
                start <= attachment.comment_span.start
                    && attachment.comment_span.end <= end
                    && placements.contains(&attachment.placement)
            })
            .flat_map(|attachment| {
                let text = self.source.source()
                    [attachment.comment_span.start..attachment.comment_span.end]
                    .trim_end_matches([' ', '\t'])
                    .to_owned();
                [Doc::text(text), Doc::hard_line()]
            })
            .collect()
    }

    fn print_lambda(&self, span: &Span, body: &Expr) -> Doc {
        let start = self
            .source
            .tokens()
            .iter()
            .find(|token| {
                span.start <= token.span.start
                    && token.span.end <= span.end
                    && matches!(token.kind, Token::Lambda)
            })
            .map_or(span.start, |token| token.span.start);
        let prefix = Span::new(start, body.span().start);
        Doc::concat([
            self.doc_from_tokens(&prefix, TokenLayout::Soft),
            Doc::concat([Doc::line(), self.print_expr_locally(body)]).nest(INDENT_WIDTH),
        ])
        .group()
    }

    fn print_expr_locally(&self, expr: &Expr) -> Doc {
        if matches!(expr, Expr::EApp(_, _, _)) {
            let doc = self.print_application(expr, true);
            self.with_source_parens(expr.span(), doc)
        } else {
            self.print_expr(expr)
        }
    }

    fn print_application(&self, expr: &Expr, local_fit: bool) -> Doc {
        let mut arguments = Vec::new();
        let head = flatten_application(expr, &mut arguments);
        let mut continuation = Vec::new();
        let mut previous_end = head.span().end;
        for argument in arguments {
            let comments = self.comments_between(previous_end, argument.span().start);
            if comments.is_empty() {
                continuation.push(Doc::line());
            } else {
                continuation.push(Doc::hard_line());
                for comment in comments {
                    continuation.push(Doc::text(comment));
                    continuation.push(Doc::hard_line());
                }
            }
            let argument_doc = self.print_expr_locally(argument);
            continuation.push(
                if expr_needs_parens(argument, ExprContext::ApplicationArgument)
                    && !self.rendered_expr_has_outer_parens(argument)
                {
                    Doc::concat([Doc::text("("), argument_doc, Doc::text(")")]).fit_group()
                } else {
                    argument_doc
                },
            );
            previous_end = argument.span().end;
        }
        let head_doc = self.print_expr(head);
        let head_doc = if expr_needs_parens(head, ExprContext::ApplicationHead)
            && !self.rendered_expr_has_outer_parens(head)
        {
            Doc::concat([Doc::text("("), head_doc, Doc::text(")")]).fit_group()
        } else {
            head_doc
        };
        let application = Doc::concat([head_doc, Doc::concat(continuation).nest(INDENT_WIDTH)]);
        if local_fit {
            application.fit_group()
        } else {
            application.group()
        }
    }

    fn print_arrow(&self, left: &Expr, right: &Expr) -> Doc {
        let boundary = Span::new(left.span().end, right.span().start);
        Doc::concat([
            self.print_expr_locally(left),
            Doc::line(),
            self.doc_from_tokens(&boundary, TokenLayout::Soft),
            Doc::text(" "),
            self.print_expr_locally(right),
        ])
        .fit_group()
    }

    fn comments_between(&self, start: usize, end: usize) -> Vec<String> {
        self.source
            .comment_attachments()
            .iter()
            .filter(|attachment| {
                start <= attachment.comment_span.start && attachment.comment_span.end <= end
            })
            .map(|attachment| {
                self.source.source()[attachment.comment_span.start..attachment.comment_span.end]
                    .trim_end_matches([' ', '\t'])
                    .to_owned()
            })
            .collect()
    }

    fn has_comment_within(&self, span: &Span) -> bool {
        self.source.comment_attachments().iter().any(|attachment| {
            span.start <= attachment.comment_span.start && attachment.comment_span.end <= span.end
        })
    }

    fn span_has_outer_parens(&self, span: &Span) -> bool {
        let indices = self.token_indices(span);
        let Some(first) = indices.first() else {
            return false;
        };
        matches!(self.source.tokens()[*first].kind, Token::LParen)
            && matching_rparen(self.source, &indices, 0) == Some(indices.len() - 1)
    }

    fn rendered_expr_has_outer_parens(&self, expr: &Expr) -> bool {
        self.span_has_outer_parens(expr.span())
    }

    fn with_source_parens(&self, span: &Span, mut doc: Doc) -> Doc {
        let mut indices = self.token_indices(span);
        let mut wrappers = 0usize;
        while indices.len() >= 2
            && matches!(self.source.tokens()[indices[0]].kind, Token::LParen)
            && matching_rparen(self.source, &indices, 0) == Some(indices.len() - 1)
        {
            wrappers += 1;
            indices = indices[1..indices.len() - 1].to_vec();
        }
        for _ in 0..wrappers {
            doc = Doc::concat([Doc::text("("), doc, Doc::text(")")]).fit_group();
        }
        doc
    }

    fn print_span(&self, span: &Span) -> Doc {
        self.doc_from_tokens(span, TokenLayout::Soft)
    }

    fn doc_from_tokens(&self, span: &Span, layout: TokenLayout) -> Doc {
        let indices = self.token_indices(span);
        if layout == TokenLayout::Soft {
            self.grouped_token_slice(&indices)
        } else {
            self.doc_token_slice(&indices, span, layout)
        }
    }

    fn doc_token_slice(&self, indices: &[usize], span: &Span, layout: TokenLayout) -> Doc {
        let mut docs = Vec::new();
        let mut previous: Option<&Token> = None;
        let mut previous_end = span.start;
        let mut position = 0;
        while position < indices.len() {
            let index = indices[position];
            let token = &self.source.tokens()[index].kind;
            if matches!(token, Token::Eof) {
                position += 1;
                continue;
            }
            let token_span = &self.source.tokens()[index].span;
            self.push_comments_between(&mut docs, previous_end, token_span.start, span);
            if layout == TokenLayout::Sum && matches!(token, Token::Pipe) {
                docs.push(Doc::hard_line());
                previous = None;
            }

            if let Some(prev) = previous {
                if needs_space(prev, token) {
                    docs.push(if soft_break_between(prev, token, layout) {
                        Doc::line()
                    } else {
                        Doc::text(" ")
                    });
                }
            }

            docs.push(Doc::text(self.token_text(index)));

            match token {
                Token::LBrace
                    if layout.breaks_braces() || is_match_brace(self.source, indices, position) =>
                {
                    if let Some(close) = matching_rbrace(self.source, indices, position) {
                        if close == position + 1 {
                            docs.push(Doc::text("}"));
                        } else {
                            let inner = &indices[position + 1..close];
                            let inner_span = Span::new(
                                self.source.tokens()[inner[0]].span.start,
                                self.source.tokens()[*inner.last().unwrap()].span.end,
                            );
                            docs.push(
                                Doc::concat([
                                    Doc::hard_line(),
                                    self.print_block_inner(inner, &inner_span, TokenLayout::Block),
                                ])
                                .nest(INDENT_WIDTH),
                            );
                            docs.push(Doc::hard_line());
                            docs.push(Doc::text("}"));
                        }
                        previous = Some(&self.source.tokens()[indices[close]].kind);
                        previous_end = self.source.tokens()[indices[close]].span.end;
                        position = close + 1;
                        continue;
                    }
                }
                Token::Semicolon if layout.breaks_siblings() => {
                    docs.push(Doc::hard_line());
                    previous = None;
                    previous_end = token_span.end;
                    position += 1;
                    continue;
                }
                Token::KwIn if layout == TokenLayout::Let => docs.push(Doc::hard_line()),
                _ => {}
            }
            previous = Some(token);
            previous_end = token_span.end;
            position += 1;
        }
        self.push_comments_between(&mut docs, previous_end, span.end, span);
        Doc::concat(docs)
    }

    fn print_block_inner(&self, indices: &[usize], span: &Span, layout: TokenLayout) -> Doc {
        if !layout.breaks_siblings() {
            return self.doc_token_slice(indices, span, layout);
        }
        let mut segments = Vec::new();
        let mut start = 0usize;
        let mut brace_depth = 0usize;
        for (position, index) in indices.iter().copied().enumerate() {
            match &self.source.tokens()[index].kind {
                Token::LBrace => brace_depth += 1,
                Token::RBrace => brace_depth = brace_depth.saturating_sub(1),
                Token::Semicolon if brace_depth == 0 => {
                    if start < position {
                        let slice = &indices[start..position];
                        segments.push((self.print_block_segment(slice), true));
                    }
                    start = position + 1;
                }
                _ => {}
            }
        }
        if start < indices.len() {
            let slice = &indices[start..];
            segments.push((self.print_block_segment(slice), false));
        }
        if segments.is_empty() {
            self.doc_token_slice(indices, span, layout)
        } else {
            let len = segments.len();
            let segments = segments
                .into_iter()
                .enumerate()
                .map(|(index, (segment, terminated))| {
                    if terminated || index + 1 < len {
                        Doc::concat([segment, Doc::text(";")]).group()
                    } else {
                        segment
                    }
                })
                .collect();
            join(segments, Doc::hard_line())
        }
    }

    /// Give a declaration-block field the same locally fitted return-type
    /// hierarchy as a top-level declaration signature.  This keeps the field
    /// head with `:`, then lets an over-width arrow chain break only at its
    /// top-level arrows; parenthesized inner arrows retain their own groups.
    fn print_block_segment(&self, indices: &[usize]) -> Doc {
        let mut paren_depth = 0usize;
        let mut bracket_depth = 0usize;
        let mut brace_depth = 0usize;
        let colon = indices
            .iter()
            .copied()
            .enumerate()
            .find_map(|(position, index)| {
                let token = &self.source.tokens()[index].kind;
                let top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;
                let found = top_level && matches!(token, Token::Colon);
                match token {
                    Token::LParen => paren_depth += 1,
                    Token::RParen => paren_depth = paren_depth.saturating_sub(1),
                    Token::LBracket => bracket_depth += 1,
                    Token::RBracket => bracket_depth = bracket_depth.saturating_sub(1),
                    Token::LBrace => brace_depth += 1,
                    Token::RBrace => brace_depth = brace_depth.saturating_sub(1),
                    _ => {}
                }
                found.then_some(position)
            });

        let Some(colon) = colon.filter(|colon| colon + 1 < indices.len()) else {
            return self.grouped_token_slice(indices);
        };
        let boundary = self.token_boundary(indices[colon], indices[colon + 1], Doc::line());
        Doc::concat([
            self.grouped_token_slice(&indices[..=colon]),
            Doc::concat([boundary, self.print_return_type(&indices[colon + 1..])])
                .nest(INDENT_WIDTH),
        ])
        .fit_group()
    }

    fn push_comments_between(&self, docs: &mut Vec<Doc>, start: usize, end: usize, owner: &Span) {
        for attachment in self
            .source
            .comment_attachments()
            .iter()
            .filter(|attachment| {
                owner.start <= attachment.comment_span.start
                    && attachment.comment_span.end <= owner.end
                    && start <= attachment.comment_span.start
                    && attachment.comment_span.end <= end
            })
        {
            let text = self.source.source()
                [attachment.comment_span.start..attachment.comment_span.end]
                .trim_end_matches([' ', '\t'])
                .to_owned();
            match attachment.placement {
                CommentPlacement::Trailing => {
                    docs.push(Doc::text("  "));
                    docs.push(Doc::text(text));
                    docs.push(Doc::hard_line());
                }
                CommentPlacement::Leading | CommentPlacement::Interstitial => {
                    if !docs.last().is_some_and(|doc| matches!(doc, Doc::HardLine)) {
                        docs.push(Doc::hard_line());
                    }
                    docs.push(Doc::text(text));
                    docs.push(Doc::hard_line());
                }
            }
        }
    }

    fn token_indices(&self, span: &Span) -> Vec<usize> {
        self.source
            .tokens()
            .iter()
            .enumerate()
            .filter(|(_, token)| {
                token.span.start != token.span.end
                    && span.start <= token.span.start
                    && token.span.end <= span.end
            })
            .map(|(index, _)| index)
            .collect()
    }

    fn token_text(&self, index: usize) -> &str {
        let span = &self.source.tokens()[index].span;
        // `canonical` has different byte offsets, so obtain B2's spelling by
        // kind and otherwise replay the B1 token lexeme.
        crate::format::canonical_token_spelling(&self.source.tokens()[index].kind)
            .unwrap_or(&self.source.source()[span.start..span.end])
    }

    fn last_token_index_before(&self, wanted: Token, before: usize, span: &Span) -> Option<usize> {
        self.source
            .tokens()
            .iter()
            .enumerate()
            .rev()
            .find_map(|(index, token)| {
                (token.span.start >= span.start
                    && token.span.end <= before
                    && same_variant(&token.kind, &wanted))
                .then_some(index)
            })
    }

    fn has_token_between(&self, wanted: Token, start: usize, end: usize) -> bool {
        self.source.tokens().iter().any(|token| {
            start <= token.span.start && token.span.end <= end && same_variant(&token.kind, &wanted)
        })
    }

    fn with_comments(&self, span: &Span, doc: Doc) -> Doc {
        let comments: Vec<_> = self
            .source
            .comment_attachments()
            .iter()
            .filter(|attachment| {
                span.start <= attachment.home_span.start
                    && attachment.home_span.end <= span.end
                    && !(span.start <= attachment.comment_span.start
                        && attachment.comment_span.end <= span.end)
            })
            .collect();
        if comments.is_empty() {
            return doc;
        }
        let mut leading = Vec::new();
        let mut trailing = Vec::new();
        let mut interstitial = Vec::new();
        for attachment in comments {
            if span.start <= attachment.comment_span.start
                && attachment.comment_span.end <= span.end
            {
                continue;
            }
            let text = self.source.source()
                [attachment.comment_span.start..attachment.comment_span.end]
                .trim_end_matches([' ', '\t'])
                .to_owned();
            match attachment.placement {
                CommentPlacement::Leading => leading.push(Doc::text(text)),
                CommentPlacement::Trailing => trailing.push(Doc::text(text)),
                CommentPlacement::Interstitial => interstitial.push(Doc::text(text)),
            }
        }
        let mut parts = Vec::new();
        for comment in leading.into_iter().chain(interstitial) {
            parts.push(comment);
            parts.push(Doc::hard_line());
        }
        let mut inline = Vec::new();
        for comment in trailing {
            let rendered = render(&doc, CANONICAL_WIDTH);
            let code = rendered
                .rsplit_once('\n')
                .map_or(rendered.as_str(), |(_, line)| line);
            let fits = flat_width(&comment)
                .is_some_and(|text| display_width(code) + 2 + text <= CANONICAL_WIDTH);
            if fits {
                inline.push(comment);
            } else {
                parts.push(comment);
                parts.push(Doc::hard_line());
            }
        }
        parts.push(doc);
        for comment in inline {
            parts.push(Doc::text("  "));
            parts.push(comment);
        }
        Doc::concat(parts)
    }
}

/// Productions whose final braces belong to an expression, type, constructor,
/// or verbatim payload rather than a declaration-block sibling list.
fn decl_owns_non_block_braces(decl: &Decl) -> bool {
    match decl {
        Decl::ViewDecl { .. }
        | Decl::LetDecl { .. }
        | Decl::ProveDecl { .. }
        | Decl::LemmaDecl { .. }
        | Decl::AxiomDecl { .. }
        | Decl::AttachedProofDecl { .. }
        | Decl::DataDecl { .. }
        | Decl::TypeAlias { .. }
        | Decl::TemporalDecl { .. } => true,
        Decl::Pub(inner) => decl_owns_non_block_braces(inner),
        _ => false,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TokenLayout {
    Soft,
    Block,
    Sum,
    Let,
}

#[derive(Clone, Copy)]
enum ExprContext {
    ApplicationHead,
    ApplicationArgument,
}

fn expr_precedence(expr: &Expr) -> u8 {
    match expr {
        Expr::ELam(_, _, _) | Expr::ELet(_, _, _) | Expr::EMatch { .. } => 0,
        Expr::EPi(_, _, _, _) | Expr::EArrow(_, _, _) => 1,
        Expr::EAsc(_, _, _) => 2,
        Expr::EBinOp(op, _, _, _) => binop_precedence(*op),
        Expr::EApp(_, _, _) => 7,
        Expr::EOld(_, _) | Expr::EProj(_, _, _) => 8,
        _ => 9,
    }
}

fn binop_precedence(op: BinOp) -> u8 {
    match op {
        BinOp::EqEq => 3,
        BinOp::Add | BinOp::WrappingAdd | BinOp::Sub => 4,
        BinOp::Mul => 5,
    }
}

fn expr_needs_parens(expr: &Expr, context: ExprContext) -> bool {
    let precedence = expr_precedence(expr);
    match context {
        ExprContext::ApplicationHead => precedence < 7,
        // An arrow argument is a mandatory-clarity case; a nested application
        // argument also needs grouping to preserve the left-associated tree.
        ExprContext::ApplicationArgument => precedence <= 7,
    }
}

impl TokenLayout {
    fn breaks_braces(self) -> bool {
        matches!(self, TokenLayout::Block | TokenLayout::Let)
    }

    fn breaks_siblings(self) -> bool {
        matches!(self, TokenLayout::Block | TokenLayout::Let)
    }
}

fn same_variant(left: &Token, right: &Token) -> bool {
    std::mem::discriminant(left) == std::mem::discriminant(right)
}

fn matching_rbrace(
    source: &dyn FormattableSource,
    indices: &[usize],
    open_position: usize,
) -> Option<usize> {
    let mut depth = 0usize;
    for (position, index) in indices.iter().copied().enumerate().skip(open_position) {
        match &source.tokens()[index].kind {
            Token::LBrace => depth += 1,
            Token::RBrace if depth == 1 => return Some(position),
            Token::RBrace => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    None
}

fn is_match_brace(
    source: &dyn FormattableSource,
    indices: &[usize],
    brace_position: usize,
) -> bool {
    let mut depth = 0usize;
    for index in indices[..brace_position].iter().copied().rev() {
        match &source.tokens()[index].kind {
            Token::RBrace => depth += 1,
            Token::LBrace if depth > 0 => depth -= 1,
            Token::KwMatch if depth == 0 => return true,
            Token::Semicolon if depth == 0 => return false,
            _ => {}
        }
    }
    false
}

fn matching_rparen(
    source: &dyn FormattableSource,
    indices: &[usize],
    open_position: usize,
) -> Option<usize> {
    let mut depth = 0usize;
    for (position, index) in indices.iter().copied().enumerate().skip(open_position) {
        match &source.tokens()[index].kind {
            Token::LParen => depth += 1,
            Token::RParen if depth == 1 => return Some(position),
            Token::RParen => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    None
}

fn token_range_for_span(
    source: &dyn FormattableSource,
    indices: &[usize],
    span: &Span,
) -> Option<(usize, usize)> {
    let start = indices.iter().position(|index| {
        let token = &source.tokens()[*index];
        span.start <= token.span.start && token.span.end <= span.end
    })?;
    let end = indices.iter().rposition(|index| {
        let token = &source.tokens()[*index];
        span.start <= token.span.start && token.span.end <= span.end
    })? + 1;
    Some((start, end))
}

fn first_top_level_paren_range(
    source: &dyn FormattableSource,
    indices: &[usize],
    from: usize,
) -> Option<(usize, usize)> {
    let mut depth = 0usize;
    for position in from..indices.len() {
        match &source.tokens()[indices[position]].kind {
            Token::LParen if depth == 0 => {
                let close = matching_rparen(source, indices, position)?;
                return Some((position, close + 1));
            }
            Token::LParen => depth += 1,
            Token::RParen => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    None
}

fn signature_clause_ranges(
    source: &dyn FormattableSource,
    indices: &[usize],
) -> Vec<(usize, usize)> {
    let mut starts = Vec::new();
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    for (position, index) in indices.iter().copied().enumerate() {
        let token = &source.tokens()[index].kind;
        if paren_depth == 0
            && bracket_depth == 0
            && brace_depth == 0
            && (matches!(
                token,
                Token::Colon | Token::KwRequires | Token::KwEnsures | Token::KwWhere
            ) || (matches!(token, Token::Ident(name) if name == "visits")
                && indices
                    .get(position + 1)
                    .is_some_and(|next| matches!(source.tokens()[*next].kind, Token::LBracket))))
        {
            starts.push(position);
        }
        match token {
            Token::LParen => paren_depth += 1,
            Token::RParen => paren_depth = paren_depth.saturating_sub(1),
            Token::LBracket => bracket_depth += 1,
            Token::RBracket => bracket_depth = bracket_depth.saturating_sub(1),
            Token::LBrace => brace_depth += 1,
            Token::RBrace => brace_depth = brace_depth.saturating_sub(1),
            _ => {}
        }
    }
    starts
        .iter()
        .enumerate()
        .map(|(position, start)| {
            let end = starts.get(position + 1).copied().unwrap_or(indices.len());
            (*start, end)
        })
        .collect()
}

fn needs_space(left: &Token, right: &Token) -> bool {
    if matches!(
        left,
        Token::LParen | Token::LBracket | Token::LBrace | Token::Dot | Token::DoubleColon
    ) || matches!(
        right,
        Token::RParen
            | Token::RBracket
            | Token::RBrace
            | Token::Comma
            | Token::Semicolon
            | Token::Dot
            | Token::DoubleColon
    ) {
        return false;
    }
    if matches!(left, Token::Lambda) || matches!(right, Token::Dot) {
        return false;
    }
    true
}

fn token_spacing(left: &Token, right: &Token) -> Doc {
    if needs_space(left, right) {
        Doc::text(" ")
    } else {
        Doc::Nil
    }
}

fn soft_break_between(left: &Token, right: &Token, layout: TokenLayout) -> bool {
    if layout != TokenLayout::Soft {
        return false;
    }
    matches!(
        right,
        Token::Arrow | Token::KwRequires | Token::KwEnsures | Token::KwWhere
    ) || (atom_can_end(left) && atom_can_start(right))
}

fn atom_can_start(token: &Token) -> bool {
    matches!(
        token,
        Token::Ident(_)
            | Token::ConId(_)
            | Token::Nat(_)
            | Token::IntLit(_)
            | Token::FloatLit(_)
            | Token::DecimalLit(_, _)
            | Token::Float32Lit(_)
            | Token::Str(_)
            | Token::LParen
            | Token::KwType
            | Token::KwMatch
            | Token::KwLet
            | Token::Lambda
            | Token::KwOld
    )
}

fn atom_can_end(token: &Token) -> bool {
    matches!(
        token,
        Token::Ident(_)
            | Token::ConId(_)
            | Token::Nat(_)
            | Token::IntLit(_)
            | Token::FloatLit(_)
            | Token::DecimalLit(_, _)
            | Token::Float32Lit(_)
            | Token::Str(_)
            | Token::RParen
            | Token::RBracket
            | Token::RBrace
    )
}

fn is_compound_expr(expr: &Expr) -> bool {
    match expr {
        Expr::EMatch { arms, .. } => {
            arms.len() >= 2 || arms.iter().any(|arm| is_compound_expr(&arm.body))
        }
        Expr::ELam(_, body, _) => is_compound_expr(body),
        Expr::ELet(bindings, body, _) => {
            bindings
                .iter()
                .any(|binding| is_compound_expr(&binding.value))
                || is_compound_expr(body)
        }
        _ => false,
    }
}

fn is_let_chain(expr: &Expr) -> bool {
    matches!(expr, Expr::ELet(bindings, body, _)
        if bindings.len() >= 2 || matches!(body.as_ref(), Expr::ELet(..)))
}

#[derive(Clone, Copy)]
enum LetTail<'a> {
    Expr(&'a Expr),
    Group(&'a [LetBinding], &'a Expr),
}

fn collect_let_segment<'a>(
    bindings: &'a [LetBinding],
    body: &'a Expr,
) -> (Vec<&'a LetBinding>, LetTail<'a>) {
    let mut segment = Vec::new();
    let mut names = std::collections::HashSet::new();
    for binding in bindings {
        debug_assert!(names.insert(binding.name.as_str()));
        segment.push(binding);
    }
    let mut tail = body;
    loop {
        let Expr::ELet(next, next_body, _) = tail else {
            return (segment, LetTail::Expr(tail));
        };
        for (index, binding) in next.iter().enumerate() {
            if !names.insert(binding.name.as_str()) {
                return (segment, LetTail::Group(&next[index..], next_body));
            }
            segment.push(binding);
        }
        tail = next_body;
    }
}

fn flatten_application<'a>(expr: &'a Expr, arguments: &mut Vec<&'a Expr>) -> &'a Expr {
    if let Expr::EApp(function, argument, _) = expr {
        let head = flatten_application(function, arguments);
        arguments.push(argument);
        head
    } else {
        expr
    }
}

fn join(mut docs: Vec<Doc>, separator: Doc) -> Doc {
    if docs.is_empty() {
        return Doc::Nil;
    }
    let mut out = Vec::with_capacity(docs.len() * 2 - 1);
    out.push(docs.remove(0));
    for doc in docs {
        out.push(separator.clone());
        out.push(doc);
    }
    Doc::concat(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_is_binary_and_hard_lines_never_flatten() {
        let soft = Doc::concat([Doc::text("abc"), Doc::line(), Doc::text("def")]).group();
        assert_eq!(render(&soft, 7), "abc def");
        assert_eq!(render(&soft, 6), "abc\ndef");

        let hard = Doc::concat([Doc::text("a"), Doc::hard_line(), Doc::text("b")]).group();
        assert_eq!(hard.flatten(), None);
        assert_eq!(render(&hard, 80), "a\nb");

        let nested = Doc::concat([
            Doc::concat([Doc::text("abc"), Doc::line(), Doc::text("def")]).group(),
            Doc::text(";"),
        ]);
        assert_eq!(
            render(&nested, 7),
            "abc\ndef;",
            "a nested group fit includes the remaining line suffix"
        );
    }

    #[test]
    fn display_columns_are_not_utf8_bytes() {
        assert_eq!(display_width("λ → Ω"), 5);
        assert_eq!(display_width("e\u{301}"), 1);
        assert_eq!(display_width("界"), 2);
    }

    #[test]
    fn formats_blocks_with_two_space_relative_nesting() {
        let source = "module M { const a : Nat = Zero; const b : Nat = Zero; }";
        assert_eq!(
            format_ken(source).unwrap(),
            "module M {\n  const a : Nat = Zero;\n  const b : Nat = Zero;\n}\n"
        );
    }

    #[test]
    fn two_arm_match_is_mandatorily_broken() {
        let source = "fn choose (x : Bool) : Bool = match x { True |-> True; False |-> False }";
        assert_eq!(
            format_ken(source).unwrap(),
            "fn choose (x : Bool) : Bool =\n  match x {\n    True ↦ True;\n    False ↦ False\n  }\n"
        );
    }
}
