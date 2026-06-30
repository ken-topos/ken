//! Surface AST for the V0/V1/L2 elaborator (`39 §5.2`, `21 §6.1`/§6.2`,
//! `34` — data/match/refinements).

use crate::error::Span;

/// A binder group `(x y z : A)` from a `view` parameter list.
#[derive(Clone, Debug)]
pub struct Binder {
    pub names: Vec<String>,
    pub ty: Type,
    pub span: Span,
}

/// A constructor in a `data` declaration (`34 §1`).
#[derive(Clone, Debug)]
pub struct CtorDecl {
    /// Constructor name (uppercase-initial).
    pub name: String,
    /// Positional argument types (no binder names at the surface level).
    pub args: Vec<Type>,
    pub span: Span,
}

/// A single arm of a `match` expression.
#[derive(Clone, Debug)]
pub struct MatchArm {
    pub pat: Pattern,
    pub body: Expr,
    pub span: Span,
}

/// A surface pattern (`34 §3`, `32 §4`).
#[derive(Clone, Debug)]
pub struct Pattern {
    pub kind: PatKind,
    pub span: Span,
}

/// The discriminant of a surface pattern.
#[derive(Clone, Debug)]
pub enum PatKind {
    /// `_` — wildcard; binds nothing.
    Wild,
    /// `x` — variable binding.
    Var(String),
    /// `C p₁ … pₙ` — constructor pattern with sub-patterns.
    Ctor(String, Vec<Pattern>),
}

/// A top-level V0/V1/L2 declaration (`32 §8`, `21 §6.2`, `34`).
#[derive(Clone, Debug)]
pub enum Decl {
    ViewDecl {
        name: String,
        params: Vec<Binder>,
        ret_ty: Option<Type>,
        /// `requires φ` clauses (V1; empty in V0 programs).
        requires: Vec<Expr>,
        /// `ensures ψ` clauses (V1; empty in V0 programs).
        ensures: Vec<Expr>,
        body: Expr,
        /// Whether the `space` prefix was present (V1 §6.4).
        is_space_op: bool,
        span: Span,
    },
    LetDecl {
        name: String,
        ty: Option<Type>,
        val: Expr,
        span: Span,
    },
    /// `prove name : φ` — a standalone obligation (`21 §3`, §6.3).
    ProveDecl {
        name: String,
        prop: Expr,
        span: Span,
    },
    /// `law Name (param) { field : φ ; … }` — a named law (`21 §3`).
    LawDecl {
        name: String,
        param: String,
        fields: Vec<(String, Expr)>,
        span: Span,
    },
    /// `data D p₁…pₙ = C₁ τ… | C₂ τ…` — simple inductive sum type (`34 §1`).
    DataDecl {
        /// Type former name (uppercase-initial).
        name: String,
        /// Lowercase type-parameter names (each is implicitly `Type 0`).
        type_params: Vec<String>,
        /// Constructors in declaration order.
        ctors: Vec<CtorDecl>,
        span: Span,
    },
    /// `type T = A` — surface type alias.
    TypeAlias {
        name: String,
        ty: Type,
        span: Span,
    },
}

impl Decl {
    pub fn name(&self) -> &str {
        match self {
            Decl::ViewDecl { name, .. }
            | Decl::LetDecl { name, .. }
            | Decl::ProveDecl { name, .. }
            | Decl::LawDecl { name, .. }
            | Decl::DataDecl { name, .. }
            | Decl::TypeAlias { name, .. } => name,
        }
    }
    pub fn span(&self) -> &Span {
        match self {
            Decl::ViewDecl { span, .. }
            | Decl::LetDecl { span, .. }
            | Decl::ProveDecl { span, .. }
            | Decl::LawDecl { span, .. }
            | Decl::DataDecl { span, .. }
            | Decl::TypeAlias { span, .. } => span,
        }
    }
}

/// A numeric literal form (`35 §4.1`).
#[derive(Clone, Debug, PartialEq)]
pub enum NumLit {
    /// Integer literal — defaults to `Int` unless an expected type is given.
    Int(i128),
    /// Decimal-point float — defaults to `Float`.
    Float(f64),
    /// `d`-suffix exact decimal — defaults to `Decimal`.
    Decimal(i64, i32),
    /// `f32`-suffix IEEE single — defaults to `Float32`.
    Float32(f32),
}

/// Infix binary operator (`35 §3`, `35 §4.2`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    /// `+` — type-directed: total for `Int`, obligation-generating for
    /// fixed-width.
    Add,
    /// `+%` — always wrapping (explicit modular arithmetic).
    WrappingAdd,
    /// `*` — type-directed multiplication.
    Mul,
    /// `==` — structural equality.
    EqEq,
}

/// A surface expression (`39 §5.2`, `21 §6.1`).
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
    /// `old e` — pre-state reference in `space`-op `ensures` (`21 §6.4`).
    EOld(Box<Expr>, Span),
    /// Numeric literal (`35 §4.1`).
    ENumLit(NumLit, Span),
    /// Infix binary operation (`35 §3`).
    EBinOp(BinOp, Box<Expr>, Box<Expr>, Span),
    /// `match scrut { P₁ => body₁ ; … }` — pattern matching (`34 §3`).
    EMatch {
        scrut: Box<Expr>,
        arms: Vec<MatchArm>,
        span: Span,
    },
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
            | Expr::EAsc(_, _, s)
            | Expr::EOld(_, s)
            | Expr::ENumLit(_, s)
            | Expr::EBinOp(_, _, _, s) => s,
            Expr::EMatch { span, .. } => span,
        }
    }
}

/// A surface type expression (`39 §5.2`, `21 §6.1`).
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
    /// `{ x : A | φ }` — refinement type (`21 §6.1`).
    TRefine(String, Box<Type>, Box<Expr>, Span),
    /// `T a b` — type application (e.g. `Option Int`, `Vec a`).
    TApp(Box<Type>, Box<Type>, Span),
}

impl Type {
    pub fn span(&self) -> &Span {
        match self {
            Type::TPi(_, _, _, s)
            | Type::TArr(_, _, s)
            | Type::TUniv(_, s)
            | Type::TCon(_, s)
            | Type::TVar(_, s)
            | Type::TRefine(_, _, _, s)
            | Type::TApp(_, _, s) => s,
        }
    }
}
