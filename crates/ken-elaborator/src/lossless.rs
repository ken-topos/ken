//! Lossless source representation for formatter and source-tooling clients.
//!
//! The semantic view remains the existing surface AST.  This module adds the
//! co-authoritative token/trivia partition and deterministic comment homes
//! required by `spec/30-surface/31-lexical.md` §1d.  It deliberately performs
//! no canonicalization or layout changes.

use std::collections::HashSet;

use crate::ast::{
    Binder, ConstructorSignatureArg, Decl, ExplicitDataCtor, Expr, MatchArm, PatKind, Pattern, Type,
};
use crate::error::{ElabError, Span};
use crate::lexer::{Lexer, Token};
use crate::parser::Parser;

/// A source token together with its original byte span.
#[derive(Clone, Debug, PartialEq)]
pub struct SourceToken {
    pub kind: Token,
    pub span: Span,
}

/// Trivia is retained verbatim; its kind only describes how it attaches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriviaKind {
    Whitespace,
    LineComment,
}

/// One contiguous trivia fragment in the original source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Trivia {
    pub kind: TriviaKind,
    pub span: Span,
}

/// The ordered partition used to reconstruct the source byte-for-byte.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourcePieceKind {
    Token(usize),
    Trivia(usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourcePiece {
    pub kind: SourcePieceKind,
    pub span: Span,
}

/// The fixed three-way comment-home classification from the B1 contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommentPlacement {
    Leading,
    Trailing,
    Interstitial,
}

/// A comment has exactly one placement and one span-keyed AST/root home.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommentAttachment {
    pub comment_span: Span,
    pub home_span: Span,
    pub placement: CommentPlacement,
}

/// Failure of the gapless token/trivia partition invariant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartitionError {
    pub expected_offset: usize,
    pub found: Option<Span>,
}

/// Abstract formatter input.  B2/B3 consume this seam rather than the
/// concrete implementation, so a future recovering CST can implement the same
/// span-keyed contract without replacing formatter clients.
pub trait FormattableSource {
    /// Existing semantic AST, reused for precedence and grouping.
    fn typed_decls(&self) -> &[Decl];
    fn source(&self) -> &str;
    fn tokens(&self) -> &[SourceToken];
    fn trivia(&self) -> &[Trivia];
    fn pieces(&self) -> &[SourcePiece];
    fn comment_attachments(&self) -> &[CommentAttachment];

    /// Comments whose unique home is the requested AST/root span.
    fn comments_for_span(&self, span: &Span) -> Vec<&CommentAttachment> {
        self.comment_attachments()
            .iter()
            .filter(|attachment| attachment.home_span == *span)
            .collect()
    }

    /// Reconstruct solely by replaying the ordered token/trivia partition.
    fn reconstruct(&self) -> String {
        let mut rebuilt = String::with_capacity(self.source().len());
        for piece in self.pieces() {
            rebuilt.push_str(&self.source()[piece.span.start..piece.span.end]);
        }
        rebuilt
    }
}

struct AstTokenTriviaSource {
    source: String,
    decls: Vec<Decl>,
    tokens: Vec<SourceToken>,
    trivia: Vec<Trivia>,
    pieces: Vec<SourcePiece>,
    attachments: Vec<CommentAttachment>,
}

impl FormattableSource for AstTokenTriviaSource {
    fn typed_decls(&self) -> &[Decl] {
        &self.decls
    }

    fn source(&self) -> &str {
        &self.source
    }

    fn tokens(&self) -> &[SourceToken] {
        &self.tokens
    }

    fn trivia(&self) -> &[Trivia] {
        &self.trivia
    }

    fn pieces(&self) -> &[SourcePiece] {
        &self.pieces
    }

    fn comment_attachments(&self) -> &[CommentAttachment] {
        &self.attachments
    }
}

/// Parse a syntactically valid Ken unit into the accepted Option-2 lossless
/// layer.  The returned trait object makes the interface, rather than the
/// concrete AST+stream carrier, the public dependency.
pub fn parse_lossless(src: &str) -> Result<Box<dyn FormattableSource>, ElabError> {
    let lexed = Lexer::lex(src)?;
    let decls = Parser::new(lexed.clone(), src.to_owned()).parse_decls()?;
    let tokens: Vec<SourceToken> = lexed
        .into_iter()
        .map(|(kind, span)| SourceToken { kind, span })
        .collect();
    let (trivia, pieces) = materialize_partition(src, &tokens)?;

    let token_spans: Vec<Span> = tokens
        .iter()
        .filter(|token| token.span.start != token.span.end)
        .map(|token| token.span.clone())
        .collect();
    let trivia_spans: Vec<Span> = trivia.iter().map(|item| item.span.clone()).collect();
    validate_partition(src.len(), &token_spans, &trivia_spans).map_err(|error| {
        ElabError::Internal(format!(
            "lossless partition is not contiguous at byte {}: {:?}",
            error.expected_offset, error.found
        ))
    })?;

    let attachments = attach_comments(src, &tokens, &trivia, &decls);
    validate_attachment_totality(&trivia, &attachments)?;

    Ok(Box::new(AstTokenTriviaSource {
        source: src.to_owned(),
        decls,
        tokens,
        trivia,
        pieces,
        attachments,
    }))
}

/// Assert that token and trivia spans form exactly one ordered partition of
/// `0..source_len`. Empty sentinel spans (such as EOF) are intentionally
/// ignored because they account for no source bytes.
pub fn validate_partition(
    source_len: usize,
    token_spans: &[Span],
    trivia_spans: &[Span],
) -> Result<(), PartitionError> {
    let mut spans: Vec<Span> = token_spans
        .iter()
        .chain(trivia_spans)
        .filter(|span| span.start != span.end)
        .cloned()
        .collect();
    spans.sort_by_key(|span| (span.start, span.end));

    let mut cursor = 0;
    for span in spans {
        if span.start != cursor || span.end < span.start || span.end > source_len {
            return Err(PartitionError {
                expected_offset: cursor,
                found: Some(span),
            });
        }
        cursor = span.end;
    }
    if cursor != source_len {
        return Err(PartitionError {
            expected_offset: cursor,
            found: None,
        });
    }
    Ok(())
}

fn materialize_partition(
    src: &str,
    tokens: &[SourceToken],
) -> Result<(Vec<Trivia>, Vec<SourcePiece>), ElabError> {
    let mut trivia = Vec::new();
    let mut pieces = Vec::new();
    let mut cursor = 0;

    for (token_index, token) in tokens.iter().enumerate() {
        if token.span.start == token.span.end {
            continue;
        }
        if token.span.start < cursor || token.span.end > src.len() {
            return Err(ElabError::Internal(format!(
                "lexer produced overlapping/out-of-bounds span {:?}",
                token.span
            )));
        }
        append_trivia(src, cursor, token.span.start, &mut trivia, &mut pieces)?;
        pieces.push(SourcePiece {
            kind: SourcePieceKind::Token(token_index),
            span: token.span.clone(),
        });
        cursor = token.span.end;
    }
    append_trivia(src, cursor, src.len(), &mut trivia, &mut pieces)?;
    Ok((trivia, pieces))
}

fn append_trivia(
    src: &str,
    start: usize,
    end: usize,
    trivia: &mut Vec<Trivia>,
    pieces: &mut Vec<SourcePiece>,
) -> Result<(), ElabError> {
    let mut cursor = start;
    while cursor < end {
        let (kind, next) = if src[cursor..end].starts_with("--") {
            let next = src[cursor..end]
                .find('\n')
                .map_or(end, |offset| cursor + offset);
            (TriviaKind::LineComment, next)
        } else {
            let mut next = cursor;
            while next < end && !src[next..end].starts_with("--") {
                let ch = src[next..end].chars().next().ok_or_else(|| {
                    ElabError::Internal("trivia cursor was not on a UTF-8 boundary".into())
                })?;
                if !ch.is_whitespace() {
                    return Err(ElabError::Internal(format!(
                        "non-trivia bytes between lexer tokens at {}",
                        next
                    )));
                }
                next += ch.len_utf8();
            }
            (TriviaKind::Whitespace, next)
        };
        if next == cursor {
            return Err(ElabError::Internal(format!(
                "zero-width trivia fragment at {}",
                cursor
            )));
        }
        let trivia_index = trivia.len();
        let span = Span::new(cursor, next);
        trivia.push(Trivia {
            kind,
            span: span.clone(),
        });
        pieces.push(SourcePiece {
            kind: SourcePieceKind::Trivia(trivia_index),
            span,
        });
        cursor = next;
    }
    Ok(())
}

fn attach_comments(
    src: &str,
    tokens: &[SourceToken],
    trivia: &[Trivia],
    decls: &[Decl],
) -> Vec<CommentAttachment> {
    let root = Span::new(0, src.len());
    let mut node_spans = Vec::new();
    for decl in decls {
        collect_decl_spans(decl, &mut node_spans);
    }
    let real_tokens: Vec<&SourceToken> = tokens
        .iter()
        .filter(|token| token.span.start != token.span.end)
        .collect();

    trivia
        .iter()
        .filter(|item| item.kind == TriviaKind::LineComment)
        .map(|comment| {
            let previous = real_tokens
                .iter()
                .rev()
                .find(|token| token.span.end <= comment.span.start)
                .copied();
            let next = real_tokens
                .iter()
                .find(|token| token.span.start >= comment.span.end)
                .copied();
            let previous_home = previous
                .and_then(|token| smallest_enclosing(&node_spans, &token.span))
                .unwrap_or_else(|| root.clone());
            let next_home = next
                .and_then(|token| smallest_enclosing(&node_spans, &token.span))
                .unwrap_or_else(|| root.clone());
            let common_home = previous.zip(next).and_then(|(before, after)| {
                smallest_common_enclosing(&node_spans, &before.span, &after.span)
            });

            let same_line_after = previous
                .is_some_and(|token| !src[token.span.end..comment.span.start].contains('\n'));
            let (placement, home_span) = if same_line_after {
                (CommentPlacement::Trailing, previous_home)
            } else if let Some(common) = common_home {
                (CommentPlacement::Interstitial, common)
            } else if next.is_some() {
                (CommentPlacement::Leading, next_home)
            } else {
                (CommentPlacement::Interstitial, root.clone())
            };

            CommentAttachment {
                comment_span: comment.span.clone(),
                home_span,
                placement,
            }
        })
        .collect()
}

fn collect_decl_spans(decl: &Decl, out: &mut Vec<Span>) {
    out.push(decl.span().clone());
    match decl {
        Decl::ViewDecl {
            params,
            ret_ty,
            requires,
            ensures,
            constraints,
            visits,
            body,
            ..
        } => {
            params
                .iter()
                .for_each(|binder| collect_binder_spans(binder, out));
            if let Some(ty) = ret_ty {
                collect_type_spans(ty, out);
            }
            requires
                .iter()
                .for_each(|expr| collect_expr_spans(expr, out));
            ensures
                .iter()
                .for_each(|expr| collect_expr_spans(expr, out));
            constraints
                .iter()
                .for_each(|constraint| collect_type_spans(&constraint.head_type, out));
            if let Some(row) = visits {
                out.push(row.span.clone());
            }
            collect_expr_spans(body, out);
        }
        Decl::LetDecl { ty, val, .. } => {
            if let Some(ty) = ty {
                collect_type_spans(ty, out);
            }
            collect_expr_spans(val, out);
        }
        Decl::ProveDecl { prop, .. } => collect_expr_spans(prop, out),
        Decl::PropDecl {
            params,
            ret_ty,
            intros,
            ..
        } => {
            params
                .iter()
                .for_each(|binder| collect_binder_spans(binder, out));
            collect_type_spans(ret_ty, out);
            for intro in intros {
                out.push(intro.span.clone());
                collect_type_spans(&intro.ty, out);
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
            params
                .iter()
                .for_each(|binder| collect_binder_spans(binder, out));
            collect_type_spans(theorem, out);
            collect_expr_spans(body, out);
        }
        Decl::AxiomDecl { theorem, .. } => collect_type_spans(theorem, out),
        Decl::LawDecl { fields, .. } | Decl::InstanceDecl { fields, .. } => {
            if let Decl::InstanceDecl {
                head_type,
                constraints,
                ..
            } = decl
            {
                collect_type_spans(head_type, out);
                constraints
                    .iter()
                    .for_each(|constraint| collect_type_spans(&constraint.head_type, out));
            }
            fields
                .iter()
                .for_each(|(_, expr)| collect_expr_spans(expr, out));
        }
        Decl::DataDecl { ctors, .. } => {
            for ctor in ctors {
                out.push(ctor.span.clone());
                ctor.args.iter().for_each(|ty| collect_type_spans(ty, out));
            }
        }
        Decl::ExplicitDataDecl {
            params,
            family,
            ctors,
            ..
        } => {
            params
                .iter()
                .for_each(|binder| collect_binder_spans(binder, out));
            collect_type_spans(family, out);
            for ctor in ctors {
                out.push(ctor.span().clone());
                match ctor {
                    ExplicitDataCtor::Simple(ctor) => {
                        ctor.args.iter().for_each(|ty| collect_type_spans(ty, out));
                    }
                    ExplicitDataCtor::Signature { signature, .. } => {
                        out.push(signature.span.clone());
                        for arg in &signature.args {
                            match arg {
                                ConstructorSignatureArg::Explicit(binder)
                                | ConstructorSignatureArg::Implicit(binder) => {
                                    collect_binder_spans(binder, out);
                                }
                                ConstructorSignatureArg::Anonymous(expr) => {
                                    collect_expr_spans(expr, out);
                                }
                            }
                        }
                        collect_expr_spans(&signature.result, out);
                    }
                }
            }
        }
        Decl::TypeAlias { ty, .. } | Decl::ForeignDecl { ty, .. } => collect_type_spans(ty, out),
        Decl::ClassDecl {
            param_kind, fields, ..
        } => {
            if let Some(kind) = param_kind {
                collect_type_spans(kind, out);
            }
            fields
                .iter()
                .for_each(|field| collect_type_spans(&field.ty, out));
        }
        Decl::ModuleDecl { decls, .. } => {
            decls.iter().for_each(|decl| collect_decl_spans(decl, out));
        }
        Decl::Pub(inner) => collect_decl_spans(inner, out),
        Decl::BoundaryDecl { .. }
        | Decl::TemporalDecl { .. }
        | Decl::DeriveDecl { .. }
        | Decl::ImportDecl { .. }
        | Decl::ExportDecl { .. } => {}
    }
}

fn collect_binder_spans(binder: &Binder, out: &mut Vec<Span>) {
    out.push(binder.span.clone());
    collect_type_spans(&binder.ty, out);
}

fn collect_match_arm_spans(arm: &MatchArm, out: &mut Vec<Span>) {
    out.push(arm.span.clone());
    collect_pattern_spans(&arm.pat, out);
    collect_expr_spans(&arm.body, out);
}

fn collect_pattern_spans(pattern: &Pattern, out: &mut Vec<Span>) {
    out.push(pattern.span.clone());
    if let PatKind::Ctor(_, fields) = &pattern.kind {
        fields
            .iter()
            .for_each(|field| collect_pattern_spans(field, out));
    }
}

fn collect_expr_spans(expr: &Expr, out: &mut Vec<Span>) {
    out.push(expr.span().clone());
    match expr {
        Expr::EApp(function, argument, _)
        | Expr::EBinOp(_, function, argument, _)
        | Expr::EArrow(function, argument, _) => {
            collect_expr_spans(function, out);
            collect_expr_spans(argument, out);
        }
        Expr::ELam(_, body, _) | Expr::EOld(body, _) | Expr::EProj(body, _, _) => {
            collect_expr_spans(body, out)
        }
        Expr::ELet(bindings, body, _) => {
            for binding in bindings {
                out.push(binding.span.clone());
                out.push(binding.name_span.clone());
                if let Some(annotation_span) = &binding.annotation_span {
                    out.push(annotation_span.clone());
                }
                if let Some(ty) = &binding.annotation {
                    collect_type_spans(ty, out);
                }
                collect_expr_spans(&binding.value, out);
            }
            collect_expr_spans(body, out);
        }
        Expr::EAsc(value, ty, _) => {
            collect_expr_spans(value, out);
            collect_type_spans(ty, out);
        }
        Expr::EMatch { scrut, arms, .. } => {
            collect_expr_spans(scrut, out);
            arms.iter()
                .for_each(|arm| collect_match_arm_spans(arm, out));
        }
        Expr::EPi(_, domain, codomain, _) => {
            collect_type_spans(domain, out);
            collect_expr_spans(codomain, out);
        }
        Expr::EVar(_, _)
        | Expr::ECon(_, _)
        | Expr::EUniv(_, _)
        | Expr::ENumLit(_, _)
        | Expr::EStr(_, _)
        | Expr::EAttachedProofRef { .. } => {}
    }
}

fn collect_type_spans(ty: &Type, out: &mut Vec<Span>) {
    out.push(ty.span().clone());
    match ty {
        Type::TPi(_, domain, codomain, _)
        | Type::TArr(domain, codomain, _)
        | Type::TApp(domain, codomain, _) => {
            collect_type_spans(domain, out);
            collect_type_spans(codomain, out);
        }
        Type::TEffectArr(domain, row, codomain, _) => {
            collect_type_spans(domain, out);
            out.push(row.span.clone());
            collect_type_spans(codomain, out);
        }
        Type::TRefine(_, domain, predicate, _) => {
            collect_type_spans(domain, out);
            collect_expr_spans(predicate, out);
        }
        Type::TUniv(_, _) | Type::TCon(_, _) | Type::TVar(_, _) => {}
    }
}

fn smallest_enclosing(spans: &[Span], target: &Span) -> Option<Span> {
    spans
        .iter()
        .filter(|span| span.start <= target.start && target.end <= span.end)
        .min_by_key(|span| span.end - span.start)
        .cloned()
}

fn smallest_common_enclosing(spans: &[Span], left: &Span, right: &Span) -> Option<Span> {
    spans
        .iter()
        .filter(|span| span.start <= left.start && right.end <= span.end)
        .min_by_key(|span| span.end - span.start)
        .cloned()
}

fn validate_attachment_totality(
    trivia: &[Trivia],
    attachments: &[CommentAttachment],
) -> Result<(), ElabError> {
    let comments: HashSet<(usize, usize)> = trivia
        .iter()
        .filter(|item| item.kind == TriviaKind::LineComment)
        .map(|item| (item.span.start, item.span.end))
        .collect();
    let homes: HashSet<(usize, usize)> = attachments
        .iter()
        .map(|item| (item.comment_span.start, item.comment_span.end))
        .collect();
    if comments.len() != attachments.len() || comments != homes {
        return Err(ElabError::Internal(format!(
            "comment attachment is not total: {} comments, {} unique homes, {} attachments",
            comments.len(),
            homes.len(),
            attachments.len()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partition_rejects_gap_and_overlap() {
        assert!(validate_partition(3, &[Span::new(0, 1), Span::new(2, 3)], &[]).is_err());
        assert!(validate_partition(3, &[Span::new(0, 2)], &[Span::new(1, 3)]).is_err());
    }
}
