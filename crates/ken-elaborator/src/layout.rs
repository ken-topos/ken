//! Canonical structural layout (`31 §1d`, WP B3).
//!
//! The formatter consumes B1's typed, lossless source view.  The AST supplies
//! production boundaries and the token stream supplies protected lexemes; the
//! source is never re-lexed and notation spelling remains B2's responsibility.

use std::collections::HashSet;

use crate::ast::{BinOp, Decl, ExplicitDataCtor, Expr, MatchArm, Type};
use crate::error::{ElabError, Span};
use crate::lexer::Token;
use crate::lossless::{parse_lossless, CommentPlacement, FormattableSource};

pub const CANONICAL_WIDTH: usize = 88;
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
            Doc::Nest(_, child) | Doc::Group(child) => child.flatten(),
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
        }
    }
    output
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
            Doc::Group(child) => commands.push(Command {
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
        Doc::Nest(_, child) | Doc::Group(child) => flat_width(child),
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
    skipped_parens: HashSet<usize>,
}

impl<'a> LayoutPrinter<'a> {
    fn new(source: &'a dyn FormattableSource) -> Self {
        let mut printer = Self {
            source,
            skipped_parens: HashSet::new(),
        };
        for decl in source.typed_decls() {
            printer.collect_decl_parens(decl);
        }
        printer
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
            Decl::DataDecl { span, .. } => self.print_decl_signature(decl, span),
            _ => self.print_span(decl.span()),
        };
        self.with_comments(decl.span(), core)
    }

    /// Expression production. It is intentionally separate from token block
    /// layout so precedence and mandatory match breaks have one owner.
    fn print_expr(&self, expr: &Expr) -> Doc {
        match expr {
            Expr::EMatch { span, arms, .. } => self.print_match(span, arms),
            Expr::ELam(_, body, span) => self.print_lambda(span, body),
            Expr::ELet(_, _, value, body, span) => self.print_let(span, value, body),
            Expr::EApp(_, _, _) => self.print_application(expr, false),
            Expr::EAsc(_, _, span)
            | Expr::EOld(_, span)
            | Expr::EBinOp(_, _, _, span)
            | Expr::EProj(_, _, span)
            | Expr::EPi(_, _, _, span)
            | Expr::EArrow(_, _, span) => self.print_span(span),
            _ => self.print_span(expr.span()),
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
        let separator = Doc::concat([Doc::text(";"), Doc::hard_line()]);
        let close = &self.source.tokens()[indices[close_pos]].span;
        Doc::concat([
            self.print_decl_signature(decl, &head_span),
            Doc::concat([Doc::hard_line(), join(children, separator)]).nest(INDENT_WIDTH),
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
            Doc::concat([Doc::hard_line(), join(ctor_docs, Doc::hard_line())]).nest(INDENT_WIDTH),
        ])
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

    /// Declaration telescopes have an enclosing-relative continuation level,
    /// while each binder remains an independent group. A broken signature
    /// therefore chooses line boundaries between binders without forcing a
    /// fitting binder type to break token-by-token.
    fn print_decl_signature(&self, decl: &Decl, span: &Span) -> Doc {
        let indices = self.token_indices(span);
        let ranges = self.decl_binder_ranges(decl, &indices);
        let Some(&(first_start, _)) = ranges.first() else {
            let clauses = signature_clause_ranges(self.source, &indices);
            let Some(&(prefix_end, _)) = clauses.first() else {
                return self.doc_from_tokens(span, TokenLayout::Soft);
            };
            if prefix_end == 0 {
                return self.doc_from_tokens(span, TokenLayout::Soft);
            }
            let prefix = self.grouped_token_slice(&indices[..prefix_end]);
            let mut continuation = Vec::new();
            for (start, end) in clauses {
                continuation.push(self.token_boundary(
                    indices[start - 1],
                    indices[start],
                    Doc::line(),
                ));
                continuation.push(self.grouped_token_slice(&indices[start..end]));
            }
            return Doc::concat([prefix, Doc::concat(continuation).nest(INDENT_WIDTH)]).group();
        };
        if first_start == 0 {
            return self.doc_from_tokens(span, TokenLayout::Soft);
        }

        let prefix = self.grouped_token_slice(&indices[..first_start]);
        let mut continuation = Vec::new();
        let mut cursor = first_start;
        for (start, end) in ranges {
            if cursor < start {
                continuation.push(self.grouped_token_slice(&indices[cursor..start]));
            }
            continuation.push(self.token_boundary(indices[start - 1], indices[start], Doc::line()));
            continuation.push(self.grouped_token_slice(&indices[start..end]));
            cursor = end;
        }
        if cursor < indices.len() {
            let tail = &indices[cursor..];
            let clauses = signature_clause_ranges(self.source, tail);
            if clauses.first().is_some_and(|(start, _)| *start == 0) {
                for (start, end) in clauses {
                    let left = if start == 0 {
                        indices[cursor - 1]
                    } else {
                        tail[start - 1]
                    };
                    continuation.push(self.token_boundary(left, tail[start], Doc::line()));
                    continuation.push(self.grouped_token_slice(&tail[start..end]));
                }
            } else {
                let left = &self.source.tokens()[indices[cursor - 1]].kind;
                let right = &self.source.tokens()[indices[cursor]].kind;
                let separator = if needs_space(left, right) {
                    Doc::text(" ")
                } else {
                    Doc::Nil
                };
                continuation.push(self.token_boundary(
                    indices[cursor - 1],
                    indices[cursor],
                    separator,
                ));
                continuation.push(self.grouped_token_slice(tail));
            }
        }

        Doc::concat([prefix, Doc::concat(continuation).nest(INDENT_WIDTH)]).group()
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
            return self.raw_grouped_token_slice(indices);
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

        let mut docs = Vec::new();
        let mut previous_end: Option<usize> = None;
        for (start, end) in segments {
            if let Some(previous) = previous_end {
                let left = &self.source.tokens()[indices[previous - 1]].kind;
                let right = &self.source.tokens()[indices[start]].kind;
                let separator = if needs_space(left, right) {
                    Doc::text(" ")
                } else {
                    Doc::Nil
                };
                docs.push(self.token_boundary(indices[previous - 1], indices[start], separator));
            }
            docs.push(self.raw_grouped_token_slice(&indices[start..end]));
            previous_end = Some(end);
        }
        Doc::concat(docs).group()
    }

    fn raw_grouped_token_slice(&self, indices: &[usize]) -> Doc {
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
                if index + 1 < arms.len() {
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
        let broken_application = application_argument_count(&arm.body) >= 2;
        let compound = matches!(arm.body, Expr::EMatch { .. })
            || is_compound_expr(&arm.body)
            || broken_application;
        if compound {
            let body = if broken_application {
                self.print_application(&arm.body, true)
            } else {
                self.print_expr(&arm.body)
            };
            Doc::concat([
                self.doc_from_tokens(&head, TokenLayout::Soft),
                Doc::concat([Doc::hard_line(), body]).nest(INDENT_WIDTH),
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

    fn print_let(&self, span: &Span, value: &Expr, body: &Expr) -> Doc {
        if is_compound_expr(value) || is_compound_expr(body) {
            let indices = self.token_indices(span);
            let Some(eq_pos) = indices.iter().position(|index| {
                matches!(self.source.tokens()[*index].kind, Token::Eq)
                    && self.source.tokens()[*index].span.end <= value.span().start
            }) else {
                return self.doc_from_tokens(span, TokenLayout::Let);
            };
            let head = Span::new(span.start, self.source.tokens()[indices[eq_pos]].span.end);
            Doc::concat([
                self.doc_from_tokens(&head, TokenLayout::Soft),
                Doc::concat([Doc::hard_line(), self.print_expr(value)]).nest(INDENT_WIDTH),
                Doc::hard_line(),
                Doc::text("in"),
                Doc::concat([Doc::hard_line(), self.print_expr(body)]).nest(INDENT_WIDTH),
            ])
        } else {
            self.doc_from_tokens(span, TokenLayout::Soft)
        }
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
            Doc::concat([Doc::line(), self.print_expr(body)]).nest(INDENT_WIDTH),
        ])
        .group()
    }

    fn print_application(&self, expr: &Expr, force_break: bool) -> Doc {
        let mut arguments = Vec::new();
        let head = flatten_application(expr, &mut arguments);
        let mut continuation = Vec::new();
        let mut previous_end = head.span().end;
        for argument in arguments {
            let comments = self.comments_between(previous_end, argument.span().start);
            if comments.is_empty() {
                continuation.push(if force_break {
                    Doc::hard_line()
                } else {
                    Doc::line()
                });
            } else {
                continuation.push(Doc::hard_line());
                for comment in comments {
                    continuation.push(Doc::text(comment));
                    continuation.push(Doc::hard_line());
                }
            }
            let argument_doc = self.print_expr(argument);
            continuation.push(
                if expr_needs_parens(argument, ExprContext::ApplicationArgument)
                    && !self.rendered_expr_has_outer_parens(argument)
                {
                    Doc::concat([Doc::text("("), argument_doc, Doc::text(")")])
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
            Doc::concat([Doc::text("("), head_doc, Doc::text(")")])
        } else {
            head_doc
        };
        Doc::concat([head_doc, Doc::concat(continuation).nest(INDENT_WIDTH)]).group()
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
        matches!(
            expr,
            Expr::EAsc(_, _, _)
                | Expr::EOld(_, _)
                | Expr::EBinOp(_, _, _, _)
                | Expr::EProj(_, _, _)
                | Expr::EPi(_, _, _, _)
                | Expr::EArrow(_, _, _)
        ) && self.span_has_outer_parens(expr.span())
    }

    fn print_span(&self, span: &Span) -> Doc {
        self.doc_from_tokens(span, TokenLayout::Soft)
    }

    fn doc_from_tokens(&self, span: &Span, layout: TokenLayout) -> Doc {
        let indices = self.token_indices(span);
        let doc = self.doc_token_slice(&indices, span, layout);
        if layout == TokenLayout::Soft {
            doc.group()
        } else {
            doc
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
            let next = indices
                .get(position + 1)
                .map(|next| &self.source.tokens()[*next].kind);

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

            // Semicolon before a closing brace is non-canonical and omitted.
            if matches!(token, Token::Semicolon)
                && (matches!(next, Some(Token::RBrace))
                    || (layout.breaks_siblings() && position + 1 == indices.len()))
            {
                previous = Some(token);
                previous_end = token_span.end;
                position += 1;
                continue;
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
                        let segment_span = Span::new(
                            self.source.tokens()[slice[0]].span.start,
                            self.source.tokens()[*slice.last().unwrap()].span.end,
                        );
                        segments.push(
                            self.doc_token_slice(slice, &segment_span, TokenLayout::Soft)
                                .group(),
                        );
                    }
                    start = position + 1;
                }
                _ => {}
            }
        }
        if start < indices.len() {
            let slice = &indices[start..];
            let segment_span = Span::new(
                self.source.tokens()[slice[0]].span.start,
                self.source.tokens()[*slice.last().unwrap()].span.end,
            );
            segments.push(
                self.doc_token_slice(slice, &segment_span, TokenLayout::Soft)
                    .group(),
            );
        }
        if segments.is_empty() {
            self.doc_token_slice(indices, span, layout)
        } else {
            let len = segments.len();
            let segments = segments
                .into_iter()
                .enumerate()
                .map(|(index, segment)| {
                    if index + 1 < len {
                        Doc::concat([segment, Doc::text(";")]).group()
                    } else {
                        segment
                    }
                })
                .collect();
            join(segments, Doc::hard_line())
        }
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
            .filter(|(index, token)| {
                token.span.start != token.span.end
                    && !self.skipped_parens.contains(index)
                    && span.start <= token.span.start
                    && token.span.end <= span.end
            })
            .map(|(index, _)| index)
            .collect()
    }

    fn collect_decl_parens(&mut self, decl: &Decl) {
        match decl {
            Decl::Pub(inner) => self.collect_decl_parens(inner),
            Decl::ViewDecl {
                params,
                ret_ty,
                requires,
                ensures,
                constraints,
                body,
                ..
            } => {
                for binder in params {
                    self.collect_type_parens(&binder.ty, TypeContext::Top);
                }
                if let Some(ty) = ret_ty {
                    self.collect_type_parens(ty, TypeContext::Top);
                }
                for expr in requires.iter().chain(ensures) {
                    self.collect_expr_parens(expr, ExprContext::Top);
                }
                for constraint in constraints {
                    self.collect_type_parens(&constraint.head_type, TypeContext::Top);
                }
                self.collect_expr_parens(body, ExprContext::Top);
            }
            Decl::LetDecl { ty, val, .. } => {
                if let Some(ty) = ty {
                    self.collect_type_parens(ty, TypeContext::Top);
                }
                self.collect_expr_parens(val, ExprContext::Top);
            }
            Decl::ProveDecl { prop, .. } => self.collect_expr_parens(prop, ExprContext::Top),
            Decl::PropDecl {
                params,
                ret_ty,
                intros,
                ..
            } => {
                for binder in params {
                    self.collect_type_parens(&binder.ty, TypeContext::Top);
                }
                self.collect_type_parens(ret_ty, TypeContext::Top);
                for intro in intros {
                    self.collect_type_parens(&intro.ty, TypeContext::Top);
                }
            }
            Decl::LemmaDecl {
                params,
                theorem,
                body,
                ..
            }
            | Decl::AttachedProofDecl {
                params,
                theorem,
                body,
                ..
            } => {
                for binder in params {
                    self.collect_type_parens(&binder.ty, TypeContext::Top);
                }
                self.collect_type_parens(theorem, TypeContext::Top);
                self.collect_expr_parens(body, ExprContext::Top);
            }
            Decl::LawDecl { fields, .. } => {
                for (_, expr) in fields {
                    self.collect_expr_parens(expr, ExprContext::Top);
                }
            }
            // Constructor telescopes are token-delimited by the next
            // constructor rather than a dedicated AST wrapper. Preserve their
            // grouping until the type printer owns the whole production.
            Decl::DataDecl { .. } => {}
            Decl::ExplicitDataDecl {
                params,
                family,
                ctors,
                ..
            } => {
                for binder in params {
                    self.collect_type_parens(&binder.ty, TypeContext::Top);
                }
                self.collect_type_parens(family, TypeContext::Top);
                for ctor in ctors {
                    match ctor {
                        ExplicitDataCtor::Simple(_) => {}
                        ExplicitDataCtor::Signature { signature, .. } => {
                            for arg in &signature.args {
                                match arg {
                                    crate::ast::ConstructorSignatureArg::Explicit(binder)
                                    | crate::ast::ConstructorSignatureArg::Implicit(binder) => {
                                        self.collect_type_parens(&binder.ty, TypeContext::Top)
                                    }
                                    crate::ast::ConstructorSignatureArg::Anonymous(expr) => {
                                        self.collect_expr_parens(expr, ExprContext::ArrowLeft)
                                    }
                                }
                            }
                            self.collect_expr_parens(&signature.result, ExprContext::ArrowRight);
                        }
                    }
                }
            }
            Decl::TypeAlias { ty, .. } | Decl::ForeignDecl { ty, .. } => {
                self.collect_type_parens(ty, TypeContext::Top)
            }
            Decl::ClassDecl {
                param_kind, fields, ..
            } => {
                if let Some(kind) = param_kind {
                    self.collect_type_parens(kind, TypeContext::Top);
                }
                for field in fields {
                    self.collect_type_parens(&field.ty, TypeContext::Top);
                }
            }
            Decl::InstanceDecl {
                head_type,
                constraints,
                fields,
                ..
            } => {
                self.collect_type_parens(head_type, TypeContext::Top);
                for constraint in constraints {
                    self.collect_type_parens(&constraint.head_type, TypeContext::Top);
                }
                for (_, expr) in fields {
                    self.collect_expr_parens(expr, ExprContext::Top);
                }
            }
            Decl::ModuleDecl { decls, .. } => {
                for decl in decls {
                    self.collect_decl_parens(decl);
                }
            }
            Decl::BoundaryDecl { .. }
            | Decl::TemporalDecl { .. }
            | Decl::DeriveDecl { .. }
            | Decl::ImportDecl { .. } => {}
        }
    }

    fn collect_expr_parens(&mut self, expr: &Expr, context: ExprContext) {
        let needed = expr_needs_parens(expr, context);
        if !matches!(expr, Expr::EAttachedProofRef { .. }) {
            self.mark_redundant_wrappers(expr.span(), needed);
        }
        match expr {
            Expr::EApp(function, argument, _) => {
                self.collect_expr_parens(function, ExprContext::ApplicationHead);
                self.collect_expr_parens(argument, ExprContext::ApplicationArgument);
            }
            Expr::ELam(_, body, _) => self.collect_expr_parens(body, ExprContext::Top),
            Expr::ELet(_, ty, value, body, _) => {
                if let Some(ty) = ty {
                    self.collect_type_parens(ty, TypeContext::Top);
                }
                self.collect_expr_parens(value, ExprContext::Top);
                self.collect_expr_parens(body, ExprContext::Top);
            }
            Expr::EAsc(value, ty, _) => {
                self.collect_expr_parens(value, ExprContext::AscriptionValue);
                self.collect_type_parens(ty, TypeContext::Top);
            }
            Expr::EOld(value, _) => self.collect_expr_parens(value, ExprContext::OldOperand),
            Expr::EProj(value, _, _) => {
                self.collect_expr_parens(value, ExprContext::ProjectionBase)
            }
            Expr::EBinOp(op, left, right, _) => {
                self.collect_expr_parens(left, ExprContext::InfixLeft(*op));
                self.collect_expr_parens(right, ExprContext::InfixRight(*op));
            }
            Expr::EMatch { scrut, arms, .. } => {
                self.collect_expr_parens(scrut, ExprContext::Top);
                for arm in arms {
                    self.collect_expr_parens(&arm.body, ExprContext::Top);
                }
            }
            Expr::EPi(_, domain, codomain, _) => {
                self.collect_type_parens(domain, TypeContext::Top);
                self.collect_expr_parens(codomain, ExprContext::ArrowRight);
            }
            Expr::EArrow(left, right, _) => {
                self.collect_expr_parens(left, ExprContext::ArrowLeft);
                self.collect_expr_parens(right, ExprContext::ArrowRight);
            }
            Expr::EVar(_, _)
            | Expr::ECon(_, _)
            | Expr::EUniv(_, _)
            | Expr::ENumLit(_, _)
            | Expr::EStr(_, _)
            | Expr::EAttachedProofRef { .. } => {}
        }
    }

    fn collect_type_parens(&mut self, ty: &Type, context: TypeContext) {
        self.mark_redundant_wrappers(ty.span(), type_needs_parens(ty, context));
        match ty {
            Type::TPi(_, domain, codomain, _) => {
                self.collect_type_parens(domain, TypeContext::Top);
                self.collect_type_parens(codomain, TypeContext::ArrowRight);
            }
            Type::TArr(left, right, _) | Type::TEffectArr(left, _, right, _) => {
                self.collect_type_parens(left, TypeContext::ArrowLeft);
                self.collect_type_parens(right, TypeContext::ArrowRight);
            }
            Type::TRefine(_, base, predicate, _) => {
                self.collect_type_parens(base, TypeContext::Top);
                self.collect_expr_parens(predicate, ExprContext::Top);
            }
            Type::TApp(head, argument, _) => {
                self.collect_type_parens(head, TypeContext::ApplicationHead);
                self.collect_type_parens(argument, TypeContext::ApplicationArgument);
            }
            Type::TUniv(_, _) | Type::TCon(_, _) | Type::TVar(_, _) => {}
        }
    }

    fn mark_redundant_wrappers(&mut self, span: &Span, needed: bool) {
        let mut indices: Vec<_> = self
            .source
            .tokens()
            .iter()
            .enumerate()
            .filter(|(_, token)| {
                token.span.start != token.span.end
                    && span.start <= token.span.start
                    && token.span.end <= span.end
            })
            .map(|(index, _)| index)
            .collect();
        let mut pairs = Vec::new();
        while indices.len() >= 2
            && matches!(self.source.tokens()[indices[0]].kind, Token::LParen)
            && matches!(
                self.source.tokens()[*indices.last().unwrap()].kind,
                Token::RParen
            )
            && matching_rparen(self.source, &indices, 0) == Some(indices.len() - 1)
        {
            pairs.push((indices[0], *indices.last().unwrap()));
            indices = indices[1..indices.len() - 1].to_vec();
        }
        let keep = usize::from(needed && !pairs.is_empty());
        for (open, close) in pairs.into_iter().skip(keep) {
            self.skipped_parens.insert(open);
            self.skipped_parens.insert(close);
        }
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
    Top,
    ApplicationHead,
    ApplicationArgument,
    OldOperand,
    ProjectionBase,
    AscriptionValue,
    ArrowLeft,
    ArrowRight,
    InfixLeft(BinOp),
    InfixRight(BinOp),
}

#[derive(Clone, Copy)]
enum TypeContext {
    Top,
    ApplicationHead,
    ApplicationArgument,
    ArrowLeft,
    ArrowRight,
}

fn expr_precedence(expr: &Expr) -> u8 {
    match expr {
        Expr::ELam(_, _, _) | Expr::ELet(_, _, _, _, _) | Expr::EMatch { .. } => 0,
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
        ExprContext::Top | ExprContext::ArrowRight => false,
        ExprContext::ApplicationHead => precedence < 7,
        // An arrow argument is a mandatory-clarity case; a nested application
        // argument also needs grouping to preserve the left-associated tree.
        ExprContext::ApplicationArgument => precedence <= 7,
        ExprContext::OldOperand => precedence < 8,
        ExprContext::ProjectionBase => {
            matches!(
                expr,
                Expr::ECon(_, _) | Expr::EApp(_, _, _) | Expr::EOld(_, _)
            ) || precedence < 7
        }
        ExprContext::AscriptionValue => precedence <= 2,
        ExprContext::ArrowLeft => precedence <= 1,
        ExprContext::InfixLeft(parent) => precedence < binop_precedence(parent),
        ExprContext::InfixRight(parent) => precedence <= binop_precedence(parent),
    }
}

fn type_precedence(ty: &Type) -> u8 {
    match ty {
        Type::TPi(_, _, _, _) | Type::TArr(_, _, _) | Type::TEffectArr(_, _, _, _) => 1,
        Type::TApp(_, _, _) => 7,
        _ => 9,
    }
}

fn type_needs_parens(ty: &Type, context: TypeContext) -> bool {
    let precedence = type_precedence(ty);
    match context {
        TypeContext::Top | TypeContext::ArrowRight => false,
        TypeContext::ApplicationHead => precedence < 7,
        TypeContext::ApplicationArgument => precedence <= 7,
        TypeContext::ArrowLeft => precedence <= 1,
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
        Expr::ELet(_, _, value, body, _) => is_compound_expr(value) || is_compound_expr(body),
        _ => false,
    }
}

fn application_argument_count(expr: &Expr) -> usize {
    let mut count = 0usize;
    let mut cursor = expr;
    while let Expr::EApp(function, _, _) = cursor {
        count += 1;
        cursor = function;
    }
    count
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
            "module M {\n  const a : Nat = Zero;\n  const b : Nat = Zero\n}\n"
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
