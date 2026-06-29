//! Surface AST for the V0 minimal elaborator (`39 §5.2`).

use crate::error::Span;

/// A binder group `(x y z : A)` from a `view` parameter list.
#[derive(Clone, Debug)]
pub struct Binder {
    pub names: Vec<String>,
    pub ty: Type,
    pub span: Span,
}

/// A top-level V0 declaration (`32 §8`).
#[derive(Clone, Debug)]
pub enum Decl {
    ViewDecl {
        name: String,
        params: Vec<Binder>,
        ret_ty: Option<Type>,
        body: Expr,
        span: Span,
    },
    LetDecl {
        name: String,
        ty: Option<Type>,
        val: Expr,
        span: Span,
    },
}

impl Decl {
    pub fn name(&self) -> &str {
        match self {
            Decl::ViewDecl { name, .. } | Decl::LetDecl { name, .. } => name,
        }
    }
    pub fn span(&self) -> &Span {
        match self {
            Decl::ViewDecl { span, .. } | Decl::LetDecl { span, .. } => span,
        }
    }
}

/// A surface expression (`39 §5.2`).
///
/// Names in `EVar`/`ECon` are still surface names; `§5.3` converts them to
/// de Bruijn indices. `ELam` carries multiple binder names (desugared to
/// single-binder form during resolution).
#[derive(Clone, Debug)]
pub enum Expr {
    /// `ident` — a term variable (lowercase).
    EVar(String, Span),
    /// `ConId` — a base type used as a term (uppercase).
    ECon(String, Span),
    /// `Type` or `Type n` — the universe as a term.
    EUniv(Option<u32>, Span),
    /// `f a` — application (left-associative).
    EApp(Box<Expr>, Box<Expr>, Span),
    /// `\ x y z . body` — lambda (multiple names desugared by resolver).
    ELam(Vec<String>, Box<Expr>, Span),
    /// `let x : A = e in body` — local binding.
    ELet(String, Option<Type>, Box<Expr>, Box<Expr>, Span),
    /// `e : A` — type ascription (checking hint).
    EAsc(Box<Expr>, Box<Type>, Span),
}

impl Expr {
    pub fn span(&self) -> &Span {
        match self {
            Expr::EVar(_, s)
            | Expr::ECon(_, s)
            | Expr::EUniv(_, s)
            | Expr::EApp(_, _, s)
            | Expr::ELam(_, _, s)
            | Expr::ELet(_, _, _, _, s)
            | Expr::EAsc(_, _, s) => s,
        }
    }
}

/// A surface type expression (`39 §5.2`).
///
/// Types and terms are unified at the kernel level, but V0's parser separates
/// them syntactically (types appear after `:` in binders and declarations).
#[derive(Clone, Debug)]
pub enum Type {
    /// `(x : A) -> B` — dependent function type (Π).
    TPi(String, Box<Type>, Box<Type>, Span),
    /// `A -> B` — non-dependent arrow.
    TArr(Box<Type>, Box<Type>, Span),
    /// `Type` or `Type n` — universe.
    TUniv(Option<u32>, Span),
    /// `ConId` — a base type by name (uppercase).
    TCon(String, Span),
    /// `ident` — a bound type variable (lowercase, e.g. `A` in `(A : Type) → A`).
    TVar(String, Span),
}

impl Type {
    pub fn span(&self) -> &Span {
        match self {
            Type::TPi(_, _, _, s)
            | Type::TArr(_, _, s)
            | Type::TUniv(_, s)
            | Type::TCon(_, s)
            | Type::TVar(_, s) => s,
        }
    }
}
