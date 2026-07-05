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

/// Surface effect-row syntax inside `visits [...]` (`36 §1.5`).
///
/// `heads` are concrete effect labels. `tail` is the optional row variable in
/// an open row: `[Console | e]`. A bare `[e]` is represented by no heads and
/// `tail = Some("e")`; a concrete row `[Console, FS]` has no tail.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EffectRowSyntax {
    pub heads: Vec<String>,
    pub tail: Option<String>,
    pub span: Span,
}

/// Definition keyword for SURF-1 purity checking (`36 §1.6`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DefKeyword {
    /// Legacy spelling kept until the D3/D4 corpus migration.
    View,
    Const,
    Fn,
    Proc,
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
        keyword: DefKeyword,
        name: String,
        params: Vec<Binder>,
        ret_ty: Option<Type>,
        /// `requires φ` clauses (V1; empty in V0 programs).
        requires: Vec<Expr>,
        /// `ensures ψ` clauses (V1; empty in V0 programs).
        ensures: Vec<Expr>,
        /// `where C₁ T₁ ; C₂ T₂` class constraints (`37 §6`, L3b).
        /// Each pair is `(class_name, head_type)`. Checked by the
        /// elaborator via `instance_search` (`classes.rs:91`).
        constraints: Vec<(String, Type)>,
        /// Optional `visits [...]` row annotation. D1 wires the row-variable
        /// surface (`[e]`, `[E | e]`) without changing the `view` keyword yet.
        visits: Option<EffectRowSyntax>,
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
    TypeAlias { name: String, ty: Type, span: Span },
    /// `foreign f : T = "symbol" "library" [pure] [E1, …]` — a C-ABI binding
    /// (`38 §2.1`, L7). Keyword spellings are `(oracle)`.
    ForeignDecl {
        name: String,
        ty: Type,
        symbol: String,
        library: String,
        is_pure: bool,
        /// Effect labels (e.g. `["FS", "Net"]`). Empty for `pure`.
        visits: Vec<String>,
        span: Span,
    },
    /// `temporal name { φ }` — a delegated temporal/behavioral obligation
    /// (`72 §4`). The body is a `TemporalExpr` (the surface notation) that
    /// elaborates to the §3 constructors and is tagged `delegated`. `source`
    /// is the verbatim formula text (human-visible, not erased). Keyword
    /// spellings are `(oracle)`/`OQ-syntax`; the elaboration target + the
    /// `delegated` status are pinned.
    TemporalDecl {
        name: String,
        formula: crate::temporal::TemporalExpr,
        source: String,
        span: Span,
    },
    /// `class C (A : Type) { field : Type ; … }` — typeclass declaration
    /// (`33 §5`). Elaborates to a record type (Σ-chain) whose kernel sort
    /// determines property vs structure via `sort_sigma` (`check.rs:192`).
    ClassDecl {
        name: String,
        /// Type parameter name (one param supported; `None` = no param).
        param: Option<String>,
        /// Optional kind annotation for the parameter. Absent means `Type0`.
        param_kind: Option<Type>,
        /// Field declarations: (name, type_expr).
        fields: Vec<(String, Type)>,
        span: Span,
    },
    /// `instance C HeadType [where C₁ T₁ ; …] { field = expr ; … }` —
    /// instance declaration (`33 §5`, `39 §6`).  Elaborates to a record value
    /// (Σ-chain of field terms) admitted through `declare_def` (kernel
    /// re-check, `check.rs:944`).
    InstanceDecl {
        class_name: String,
        /// Head type expression (may be a type constructor application).
        head_type: Type,
        /// Sub-constraints: [(class_name, head_type)] (`39 §6.4`).
        constraints: Vec<(String, Type)>,
        /// Field implementations: (name, expr).
        fields: Vec<(String, Expr)>,
        span: Span,
    },
    /// `derive ClassName for DataName` — auto-derive request (`33 §5.6`,
    /// `39 §6.6`). The elaborator generates a candidate instance and passes
    /// it through the real `declare_def` re-check (untrusted generation).
    DeriveDecl {
        class_name: String,
        data_name: String,
        span: Span,
    },
    /// `module M { … }` — a namespace/environment fragment (`33 §3.1`,
    /// ES3-build). Purely surface: elaborates its inner decls into the
    /// single flat `Σ` under qualified names; the kernel never sees a
    /// module. Nested `module N { … }` inside give `M.N`.
    ModuleDecl {
        name: String,
        decls: Vec<Decl>,
        span: Span,
    },
    /// `import M` / `import M as N` / `import M (foo, Bar)` / `use M`
    /// (`33 §3.2`) — brings another module's `pub` exports into scope.
    /// Surface/elaboration-time only; resolved away before the kernel.
    ImportDecl {
        module: String,
        kind: ImportKind,
        span: Span,
    },
    /// `pub <decl>` — marks the wrapped top-level decl's name as exported
    /// from its enclosing module (`33 §4.1`). Top-level (non-module) decls
    /// may also be wrapped; the marker is simply inert there (nothing to
    /// export from). Nested modules are unaffected by a `pub` on decls
    /// inside them other than the module's own name marker (modules
    /// themselves have no separate visibility — only their contents do).
    Pub(Box<Decl>),
}

/// The four import forms (`33 §3.2`).
#[derive(Clone, Debug)]
pub enum ImportKind {
    /// `import M` — qualified: `M`'s exports accessible as `M.foo`.
    Qualified,
    /// `import M as N` — aliased: exports accessible as `N.foo`.
    Aliased(String),
    /// `import M (foo, Bar)` — selective: exactly these names, unqualified.
    Selective(Vec<String>),
    /// `use M` — open: all of `M`'s exports, unqualified.
    Open,
}

impl Decl {
    pub fn name(&self) -> &str {
        match self {
            Decl::Pub(inner) => inner.name(),
            Decl::ModuleDecl { name, .. } => name,
            // `ImportDecl` has no declared name of its own; callers that
            // need a per-decl name must special-case it (it's never
            // registered as a global).
            Decl::ImportDecl { module, .. } => module,
            Decl::ViewDecl { name, .. }
            | Decl::LetDecl { name, .. }
            | Decl::ProveDecl { name, .. }
            | Decl::LawDecl { name, .. }
            | Decl::DataDecl { name, .. }
            | Decl::TypeAlias { name, .. }
            | Decl::ForeignDecl { name, .. }
            | Decl::TemporalDecl { name, .. }
            | Decl::ClassDecl { name, .. } => name,
            Decl::InstanceDecl { class_name, .. } => class_name,
            Decl::DeriveDecl { class_name, .. } => class_name,
        }
    }
    pub fn span(&self) -> &Span {
        match self {
            Decl::Pub(inner) => inner.span(),
            Decl::ViewDecl { span, .. }
            | Decl::LetDecl { span, .. }
            | Decl::ProveDecl { span, .. }
            | Decl::LawDecl { span, .. }
            | Decl::DataDecl { span, .. }
            | Decl::TypeAlias { span, .. }
            | Decl::ForeignDecl { span, .. }
            | Decl::TemporalDecl { span, .. }
            | Decl::ClassDecl { span, .. }
            | Decl::InstanceDecl { span, .. }
            | Decl::DeriveDecl { span, .. }
            | Decl::ModuleDecl { span, .. }
            | Decl::ImportDecl { span, .. } => span,
        }
    }

    /// Is this decl (after unwrapping any `pub`) a `pub`-marked export?
    pub fn is_pub(&self) -> bool {
        matches!(self, Decl::Pub(_))
    }

    /// Unwrap a `Pub` wrapper, if present, to the inner decl.
    pub fn unwrap_pub(&self) -> &Decl {
        match self {
            Decl::Pub(inner) => inner,
            other => other,
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
    /// `-` — type-directed subtraction (VAL2 #11).
    Sub,
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
    /// String literal (`37 §2.1`, VAL1-surface).
    EStr(String, Span),
    /// Infix binary operation (`35 §3`).
    EBinOp(BinOp, Box<Expr>, Box<Expr>, Span),
    /// `match scrut { P₁ => body₁ ; … }` — pattern matching (`34 §3`).
    EMatch {
        scrut: Box<Expr>,
        arms: Vec<MatchArm>,
        span: Span,
    },
    /// `e.field` — Σ-record field projection (`33 §5.2` η) on a class
    /// instance/dictionary value. Postfix, lowest-binding-atom precedence;
    /// only fires on a non-`ConId`-headed base (a `ConId`-headed dotted
    /// chain is a module-qualified reference, `33 §3.2`, joined at parse
    /// time instead).
    EProj(Box<Expr>, String, Span),
    /// `(x : A) -> B` — dependent function type, expr position (VAL2 #4,
    /// `32 §3`). Domain is a `type` (mirrors the type-position `Pi`);
    /// codomain is an expr binding `x`. Elaborates to the existing kernel
    /// `Pi` — no new kernel variant (types are terms, `11 §1`).
    EPi(String, Box<Type>, Box<Expr>, Span),
    /// `A -> B` — non-dependent function type, expr position (VAL2 #4,
    /// `32 §3`). BOTH sides are exprs (elaborate to `Type`-classified
    /// terms); right-associative. Elaborates to the existing kernel `Pi`.
    EArrow(Box<Expr>, Box<Expr>, Span),
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
            | Expr::EStr(_, s)
            | Expr::EPi(_, _, _, s)
            | Expr::EArrow(_, _, s)
            | Expr::EBinOp(_, _, _, s)
            | Expr::EProj(_, _, s) => s,
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
