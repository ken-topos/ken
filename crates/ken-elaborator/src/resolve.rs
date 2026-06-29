//! Name resolution: surface AST → resolved AST (`39 §5.3`).
//!
//! This is the **correctness-critical** pass (`39 §5.3`): a capture or
//! mis-scoping bug yields a well-typed-looking but wrong core term that the
//! kernel cannot backstop. The algorithm is a standard scope-stack walk,
//! innermost first, with single-binder desugaring applied before the walk.

use crate::ast::{Decl, Expr, Type};
use crate::error::{ElabError, Span};

/// A resolved declaration: the name + a resolved type + a resolved body.
///
/// `view f (A:Type)(x:A):A = x` is fully desugared into a Π-type and a
/// sequence of nested λs; the resolver handles this desugaring so the
/// elaborator only sees single-binder forms.
pub struct RDecl {
    pub name: String,
    pub ty: Option<RType>,   // declared type (None → infer)
    pub body: RExpr,
    pub span: Span,
}

/// A resolved expression — names replaced by de Bruijn indices.
#[derive(Clone, Debug)]
pub enum RExpr {
    /// A de Bruijn variable (`11 §2`): index + source name for diagnostics.
    RVar(usize, String, Span),
    /// A global type/constant referenced by name (resolved by elaborator in Σ).
    RCon(String, Span),
    /// `Type` or `Type n` — the universe.
    RUniv(Option<u32>, Span),
    /// Application.
    RApp(Box<RExpr>, Box<RExpr>, Span),
    /// Single-binder lambda.
    RLam(String, Box<RExpr>, Span),
    /// Local let-binding.
    RLet(String, Option<RType>, Box<RExpr>, Box<RExpr>, Span),
    /// Type ascription.
    RAsc(Box<RExpr>, Box<RType>, Span),
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
            | RExpr::RAsc(_, _, s) => s,
        }
    }
}

/// A resolved type expression.
#[derive(Clone, Debug)]
pub enum RType {
    /// Dependent Π.
    RPi(String, Box<RType>, Box<RType>, Span),
    /// Non-dependent arrow.
    RArr(Box<RType>, Box<RType>, Span),
    /// Universe.
    RUniv(Option<u32>, Span),
    /// A global base type by name.
    RCon(String, Span),
    /// A bound type variable (de Bruijn index + source name).
    RVarTy(usize, String, Span),
}

impl RType {
    pub fn span(&self) -> &Span {
        match self {
            RType::RPi(_, _, _, s)
            | RType::RArr(_, _, s)
            | RType::RUniv(_, s)
            | RType::RCon(_, s)
            | RType::RVarTy(_, _, s) => s,
        }
    }
}

// ----- scope -----

/// The scope stack: a list of names, innermost first. An index `i` into the
/// stack is the de Bruijn index of that binder (`indexOf` in the spec).
struct Scope(Vec<String>);

impl Scope {
    fn new() -> Self {
        Self(Vec::new())
    }

    /// Push a binder name; returns the index it occupies (always 0 — callers
    /// must increment their own counter if they push multiple binders).
    fn push(&mut self, name: &str) {
        self.0.push(name.to_string());
    }

    fn pop(&mut self) {
        self.0.pop();
    }

    /// `indexOf(scope, name)` — first (innermost) match, 0-based.
    fn index_of(&self, name: &str) -> Option<usize> {
        self.0.iter().rev().position(|n| n == name)
    }

    fn depth(&self) -> usize {
        self.0.len()
    }
}

// ----- public entry point -----

/// Resolve a sequence of top-level declarations.
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
            body,
            span,
        } => {
            // Desugar params: build Π-chain for the type, λ-chain for the body.
            // Each binder `(x1 x2 … xn : A)` desugars to n single-binder Pi/Lam.
            //
            // Scope grows left-to-right across binders (outermost param first):
            // `view f (A : Type) (x : A) : A = x`
            //   scope for (A : Type) domain = []
            //   scope for (x : A) domain   = ["A"]
            //   scope for ret_ty            = ["x", "A"]   (innermost first)
            //   scope for body              = ["x", "A"]

            let mut scope = Scope::new();

            // Expand binders into (name, resolved_type) pairs in order
            let mut single_binders: Vec<(String, RType)> = Vec::new();
            for b in params {
                let a_span = b.ty.span().clone();
                let a = resolve_type(&mut scope, &b.ty)?;
                for name_str in &b.names {
                    single_binders.push((name_str.clone(), a.clone()));
                    scope.push(name_str);
                }
                let _ = a_span;
            }

            // Resolved return type (in the scope after all params)
            let resolved_ret = match ret_ty {
                Some(t) => Some(resolve_type(&mut scope, t)?),
                None => None,
            };

            // Resolved body (same scope as ret_ty, i.e. all params in scope)
            let resolved_body = resolve_expr(&mut scope, body)?;

            // Wrap body in lambdas (right-to-left, innermost first)
            let full_body = single_binders
                .iter()
                .rev()
                .fold(resolved_body, |acc, (bname, _)| {
                    let s = acc.span().clone();
                    RExpr::RLam(bname.clone(), Box::new(acc), s)
                });

            // Build the Π-type from binders + return type
            let full_ty = match resolved_ret {
                Some(ret) => {
                    // Wrap ret in Pi-chain: innermost binder wraps first
                    Some(
                        single_binders
                            .into_iter()
                            .rev()
                            .fold(ret, |acc, (bname, bty)| {
                                let s = acc.span().clone();
                                RType::RPi(bname, Box::new(bty), Box::new(acc), s)
                            }),
                    )
                }
                None => None,
            };

            Ok(RDecl {
                name: name.clone(),
                ty: full_ty,
                body: full_body,
                span: span.clone(),
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
                span: span.clone(),
            })
        }
    }
}

/// Resolve a standalone expression (no outer binders).
pub fn resolve_expr_standalone(expr: &Expr) -> Result<RExpr, ElabError> {
    let mut scope = Scope::new();
    resolve_expr(&mut scope, expr)
}

// ----- internal resolution -----

fn resolve_expr(scope: &mut Scope, expr: &Expr) -> Result<RExpr, ElabError> {
    match expr {
        Expr::EVar(name, span) => {
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
            let rf = resolve_expr(scope, f)?;
            let ra = resolve_expr(scope, a)?;
            Ok(RExpr::RApp(Box::new(rf), Box::new(ra), span.clone()))
        }

        Expr::ELam(names, body, span) => {
            // Multi-name lambda: desugar to nested single-binder lambdas.
            // Resolution of each binder happens with the enclosing scope.
            let mut rexpr = {
                // Push all names, resolve body, pop in reverse
                let depth_before = scope.depth();
                for n in names {
                    scope.push(n);
                }
                let rb = resolve_expr(scope, body)?;
                // Pop all
                for _ in names {
                    scope.pop();
                }
                assert_eq!(scope.depth(), depth_before);
                rb
            };
            // Wrap in nested RLam, innermost first (reverse the names list)
            for n in names.iter().rev() {
                let s = rexpr.span().clone();
                rexpr = RExpr::RLam(n.clone(), Box::new(rexpr), s);
            }
            let _ = span;
            Ok(rexpr)
        }

        Expr::ELet(x, ty, rhs, body, span) => {
            // `x` is NOT in scope of its own rhs (no V0 recursion in let)
            let resolved_rhs = resolve_expr(scope, rhs)?;
            let resolved_ty = match ty {
                Some(t) => Some(resolve_type(scope, t)?),
                None => None,
            };
            // `x` IS in scope of body
            scope.push(x);
            let resolved_body = resolve_expr(scope, body)?;
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
            let re = resolve_expr(scope, e)?;
            let rt = resolve_type(scope, ty)?;
            Ok(RExpr::RAsc(Box::new(re), Box::new(rt), span.clone()))
        }
    }
}

fn resolve_type(scope: &mut Scope, ty: &Type) -> Result<RType, ElabError> {
    match ty {
        Type::TUniv(l, span) => Ok(RType::RUniv(*l, span.clone())),

        Type::TCon(name, span) => Ok(RType::RCon(name.clone(), span.clone())),

        Type::TVar(name, span) => {
            // Scope-first resolution: if the name is in scope it's a bound
            // variable; if not, treat as a global `ConId` (e.g. `Nat`, `Bool`).
            // The elaborator's `elab_type` will look it up in Σ.
            if let Some(i) = scope.index_of(name) {
                Ok(RType::RVarTy(i, name.clone(), span.clone()))
            } else {
                // Global reference — elaborator must resolve in Σ.
                Ok(RType::RCon(name.clone(), span.clone()))
            }
        }

        Type::TArr(a, b, span) => {
            let ra = resolve_type(scope, a)?;
            let rb = resolve_type(scope, b)?;
            Ok(RType::RArr(Box::new(ra), Box::new(rb), span.clone()))
        }

        Type::TPi(x, a, b, span) => {
            // `x` is NOT in scope of `a`; it IS in scope of `b`.
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
    }
}
