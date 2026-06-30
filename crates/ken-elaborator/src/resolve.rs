//! Name resolution: surface AST → resolved AST (`39 §5.3`, `21 §6.1`).
//!
//! V1 additions: resolves `requires`/`ensures` clause lists, scopes `result`
//! only into `ensures`, scopes `old` only into `space`-op `ensures`, resolves
//! `{x:A|φ}` refinement types, `prove`, and `law` declarations.

use crate::ast::{Decl, Expr, Type};
use crate::error::{ElabError, Span};

/// A resolved declaration (`21 §6.2`).
pub struct RDecl {
    pub name: String,
    pub ty: Option<RType>,
    pub body: RExpr,
    /// Resolved `requires` propositions (V1; empty for V0 programs).
    pub requires: Vec<RExpr>,
    /// Resolved `ensures` propositions (V1; result in scope).
    pub ensures: Vec<RExpr>,
    pub span: Span,
    pub kind: RDeclKind,
}

/// Discriminates the declaration kind for elaboration dispatch.
pub enum RDeclKind {
    /// A `view` (or `space view`) definition.
    View { is_space_op: bool },
    /// A `let` binding.
    Let,
    /// A `prove name : φ` standalone obligation.
    Prove,
    /// A `law Name (param) { field : φ ; … }` bundle.
    Law {
        param: String,
        fields: Vec<(String, RExpr)>,
    },
}

/// A resolved expression — names replaced by de Bruijn indices.
#[derive(Clone, Debug)]
pub enum RExpr {
    RVar(usize, String, Span),
    RCon(String, Span),
    RUniv(Option<u32>, Span),
    RApp(Box<RExpr>, Box<RExpr>, Span),
    RLam(String, Box<RExpr>, Span),
    RLet(String, Option<RType>, Box<RExpr>, Box<RExpr>, Span),
    RAsc(Box<RExpr>, Box<RType>, Span),
    /// `old e` resolved in a `space`-op `ensures` (`21 §6.4`).
    ROld(Box<RExpr>, Span),
}

impl RExpr {
    pub fn span(&self) -> &Span {
        match self {
            RExpr::RVar(_, _, s)
            | RExpr::RCon(_, s)
            | RExpr::RUniv(_, s)
            | RExpr::RApp(_, _, s)
            | RExpr::RLam(_, _, s)
            | RExpr::RLet(_, _, _, _, s)
            | RExpr::RAsc(_, _, s)
            | RExpr::ROld(_, s) => s,
        }
    }
}

/// A resolved type expression.
#[derive(Clone, Debug)]
pub enum RType {
    RPi(String, Box<RType>, Box<RType>, Span),
    RArr(Box<RType>, Box<RType>, Span),
    RUniv(Option<u32>, Span),
    RCon(String, Span),
    RVarTy(usize, String, Span),
    /// `{ x : A | φ }` — carrier `A` + tracked predicate `φ` (`21 §6.1`).
    RRefine(String, Box<RType>, Box<RExpr>, Span),
}

impl RType {
    pub fn span(&self) -> &Span {
        match self {
            RType::RPi(_, _, _, s)
            | RType::RArr(_, _, s)
            | RType::RUniv(_, s)
            | RType::RCon(_, s)
            | RType::RVarTy(_, _, s)
            | RType::RRefine(_, _, _, s) => s,
        }
    }
}

// ----- scope -----

struct Scope(Vec<String>);

impl Scope {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn push(&mut self, name: &str) {
        self.0.push(name.to_string());
    }

    fn pop(&mut self) {
        self.0.pop();
    }

    fn index_of(&self, name: &str) -> Option<usize> {
        self.0.iter().rev().position(|n| n == name)
    }

    fn depth(&self) -> usize {
        self.0.len()
    }
}

// ----- resolution context -----

/// Tracks what special names are in scope and what context we're resolving in.
#[derive(Clone, Copy, PartialEq, Eq)]
enum PropCtx {
    /// In a `requires` clause — `result` and `old` are out of scope.
    Requires,
    /// In an `ensures` clause of a pure `view` — `result` is in scope, `old` is not.
    PureViewEnsures,
    /// In an `ensures` clause of a `space view` — `result` and `old` are in scope.
    SpaceOpEnsures,
    /// Not in a spec clause.
    None,
}

// ----- public entry points -----

pub fn resolve_decls(decls: &[Decl]) -> Result<Vec<RDecl>, ElabError> {
    let mut out = Vec::new();
    for d in decls {
        out.push(resolve_decl(d)?);
    }
    Ok(out)
}

pub fn resolve_decl(decl: &Decl) -> Result<RDecl, ElabError> {
    match decl {
        Decl::ViewDecl {
            name,
            params,
            ret_ty,
            requires,
            ensures,
            body,
            is_space_op,
            span,
        } => {
            let mut scope = Scope::new();

            let mut single_binders: Vec<(String, RType)> = Vec::new();
            for b in params {
                let a = resolve_type(&mut scope, &b.ty)?;
                for name_str in &b.names {
                    single_binders.push((name_str.clone(), a.clone()));
                    scope.push(name_str);
                }
            }

            // Resolve `requires` clauses — `result` and `old` are NOT in scope
            let resolved_requires = requires
                .iter()
                .map(|phi| resolve_prop(&mut scope, phi, PropCtx::Requires))
                .collect::<Result<Vec<_>, _>>()?;

            // Resolve return type
            let resolved_ret = match ret_ty {
                Some(t) => Some(resolve_type(&mut scope, t)?),
                None => None,
            };

            // Resolve body (same scope as params)
            let resolved_body = resolve_expr(&mut scope, body)?;

            // Resolve `ensures` clauses — `result` IS in scope; `old` iff space-op
            let prop_ctx = if *is_space_op {
                PropCtx::SpaceOpEnsures
            } else {
                PropCtx::PureViewEnsures
            };
            // Bind `result` in scope for ensures resolution
            scope.push("result");
            let resolved_ensures = ensures
                .iter()
                .map(|psi| resolve_prop(&mut scope, psi, prop_ctx))
                .collect::<Result<Vec<_>, _>>()?;
            scope.pop(); // unbind `result`

            // Wrap body in lambdas (right-to-left)
            let full_body = single_binders
                .iter()
                .rev()
                .fold(resolved_body, |acc, (bname, _)| {
                    let s = acc.span().clone();
                    RExpr::RLam(bname.clone(), Box::new(acc), s)
                });

            // Build Pi-type from binders + requires (as Π proof-args) + return type
            // Requires propositions become additional Π-args after params.
            // The body includes corresponding λ-binders for proof-args.
            // NOTE: full_requires_body is built below during elaboration, not here.
            // The resolver just passes the resolved clauses; elab does the Pi/Lam wrapping.

            let full_ty = match resolved_ret {
                Some(ret) => Some(
                    single_binders
                        .into_iter()
                        .rev()
                        .fold(ret, |acc, (bname, bty)| {
                            let s = acc.span().clone();
                            RType::RPi(bname, Box::new(bty), Box::new(acc), s)
                        }),
                ),
                None => None,
            };

            Ok(RDecl {
                name: name.clone(),
                ty: full_ty,
                body: full_body,
                requires: resolved_requires,
                ensures: resolved_ensures,
                span: span.clone(),
                kind: RDeclKind::View { is_space_op: *is_space_op },
            })
        }

        Decl::LetDecl { name, ty, val, span } => {
            let mut scope = Scope::new();
            let resolved_ty = match ty {
                Some(t) => Some(resolve_type(&mut scope, t)?),
                None => None,
            };
            let resolved_val = resolve_expr(&mut scope, val)?;
            Ok(RDecl {
                name: name.clone(),
                ty: resolved_ty,
                body: resolved_val,
                requires: vec![],
                ensures: vec![],
                span: span.clone(),
                kind: RDeclKind::Let,
            })
        }

        Decl::ProveDecl { name, prop, span } => {
            let mut scope = Scope::new();
            let resolved_prop = resolve_expr(&mut scope, prop)?;
            Ok(RDecl {
                name: name.clone(),
                ty: None,
                body: resolved_prop,
                requires: vec![],
                ensures: vec![],
                span: span.clone(),
                kind: RDeclKind::Prove,
            })
        }

        Decl::LawDecl { name, param, fields, span } => {
            let mut scope = Scope::new();
            // `param` is in scope for each field proposition
            scope.push(param);
            let resolved_fields = fields
                .iter()
                .map(|(fname, phi)| {
                    let rphi = resolve_expr(&mut scope, phi)?;
                    Ok((fname.clone(), rphi))
                })
                .collect::<Result<Vec<_>, _>>()?;
            scope.pop();
            Ok(RDecl {
                name: name.clone(),
                ty: None,
                body: RExpr::RUniv(None, span.clone()), // placeholder
                requires: vec![],
                ensures: vec![],
                span: span.clone(),
                kind: RDeclKind::Law {
                    param: param.clone(),
                    fields: resolved_fields,
                },
            })
        }
    }
}

pub fn resolve_expr_standalone(expr: &Expr) -> Result<RExpr, ElabError> {
    let mut scope = Scope::new();
    resolve_expr(&mut scope, expr)
}

// ----- internal resolution -----

fn resolve_expr(scope: &mut Scope, expr: &Expr) -> Result<RExpr, ElabError> {
    resolve_expr_ctx(scope, expr, PropCtx::None)
}

/// Resolve an expression in a proposition context (for spec clauses).
fn resolve_prop(scope: &mut Scope, expr: &Expr, ctx: PropCtx) -> Result<RExpr, ElabError> {
    resolve_expr_ctx(scope, expr, ctx)
}

fn resolve_expr_ctx(scope: &mut Scope, expr: &Expr, ctx: PropCtx) -> Result<RExpr, ElabError> {
    match expr {
        Expr::EVar(name, span) => {
            // `result` is only valid in ensures clauses
            if name == "result" && !matches!(ctx, PropCtx::PureViewEnsures | PropCtx::SpaceOpEnsures) {
                return Err(ElabError::UnboundName {
                    name: name.clone(),
                    span: span.clone(),
                });
            }
            let i = scope
                .index_of(name)
                .ok_or_else(|| ElabError::UnboundName {
                    name: name.clone(),
                    span: span.clone(),
                })?;
            Ok(RExpr::RVar(i, name.clone(), span.clone()))
        }

        Expr::ECon(name, span) => Ok(RExpr::RCon(name.clone(), span.clone())),

        Expr::EUniv(l, span) => Ok(RExpr::RUniv(*l, span.clone())),

        Expr::EApp(f, a, span) => {
            let rf = resolve_expr_ctx(scope, f, ctx)?;
            let ra = resolve_expr_ctx(scope, a, ctx)?;
            Ok(RExpr::RApp(Box::new(rf), Box::new(ra), span.clone()))
        }

        Expr::ELam(names, body, span) => {
            let depth_before = scope.depth();
            for n in names {
                scope.push(n);
            }
            let rb = resolve_expr_ctx(scope, body, ctx)?;
            for _ in names {
                scope.pop();
            }
            assert_eq!(scope.depth(), depth_before);
            let mut rexpr = rb;
            for n in names.iter().rev() {
                let s = rexpr.span().clone();
                rexpr = RExpr::RLam(n.clone(), Box::new(rexpr), s);
            }
            let _ = span;
            Ok(rexpr)
        }

        Expr::ELet(x, ty, rhs, body, span) => {
            let resolved_rhs = resolve_expr_ctx(scope, rhs, ctx)?;
            let resolved_ty = match ty {
                Some(t) => Some(resolve_type(scope, t)?),
                None => None,
            };
            scope.push(x);
            let resolved_body = resolve_expr_ctx(scope, body, ctx)?;
            scope.pop();
            Ok(RExpr::RLet(
                x.clone(),
                resolved_ty,
                Box::new(resolved_rhs),
                Box::new(resolved_body),
                span.clone(),
            ))
        }

        Expr::EAsc(e, ty, span) => {
            let re = resolve_expr_ctx(scope, e, ctx)?;
            let rt = resolve_type(scope, ty)?;
            Ok(RExpr::RAsc(Box::new(re), Box::new(rt), span.clone()))
        }

        Expr::EOld(e, span) => {
            // `old` is only valid in `space`-op `ensures` (`21 §6.4`)
            if ctx != PropCtx::SpaceOpEnsures {
                return Err(ElabError::UnboundName {
                    name: "old".to_string(),
                    span: span.clone(),
                });
            }
            let re = resolve_expr_ctx(scope, e, ctx)?;
            Ok(RExpr::ROld(Box::new(re), span.clone()))
        }
    }
}

fn resolve_type(scope: &mut Scope, ty: &Type) -> Result<RType, ElabError> {
    match ty {
        Type::TUniv(l, span) => Ok(RType::RUniv(*l, span.clone())),

        Type::TCon(name, span) => Ok(RType::RCon(name.clone(), span.clone())),

        Type::TVar(name, span) => {
            if let Some(i) = scope.index_of(name) {
                Ok(RType::RVarTy(i, name.clone(), span.clone()))
            } else {
                Ok(RType::RCon(name.clone(), span.clone()))
            }
        }

        Type::TArr(a, b, span) => {
            let ra = resolve_type(scope, a)?;
            let rb = resolve_type(scope, b)?;
            Ok(RType::RArr(Box::new(ra), Box::new(rb), span.clone()))
        }

        Type::TPi(x, a, b, span) => {
            let ra = resolve_type(scope, a)?;
            scope.push(x);
            let rb = resolve_type(scope, b)?;
            scope.pop();
            Ok(RType::RPi(
                x.clone(),
                Box::new(ra),
                Box::new(rb),
                span.clone(),
            ))
        }

        Type::TRefine(x, a, phi, span) => {
            let ra = resolve_type(scope, a)?;
            // `x` is in scope for the predicate `φ`
            scope.push(x);
            let rphi = resolve_expr_ctx(scope, phi, PropCtx::None)?;
            scope.pop();
            Ok(RType::RRefine(
                x.clone(),
                Box::new(ra),
                Box::new(rphi),
                span.clone(),
            ))
        }
    }
}
