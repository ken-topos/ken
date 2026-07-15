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
    /// Optional field labels from record-style declaration sugar.
    pub field_labels: Option<Vec<String>>,
    pub span: Span,
}

/// A constructor argument in an explicit constructor signature (`34 §2.2`).
#[derive(Clone, Debug)]
pub enum ConstructorSignatureArg {
    /// `(x : A) -> ...` — an explicit dependent binder.
    Explicit(Binder),
    /// `{x : A} -> ...` — an implicit dependent binder.
    Implicit(Binder),
    /// `A -> ...` — an anonymous non-dependent argument.
    Anonymous(Expr),
}

impl ConstructorSignatureArg {
    pub fn span(&self) -> &Span {
        match self {
            ConstructorSignatureArg::Explicit(b) | ConstructorSignatureArg::Implicit(b) => &b.span,
            ConstructorSignatureArg::Anonymous(e) => e.span(),
        }
    }
}

/// The peeled telescope and final result target of `C : ...`.
#[derive(Clone, Debug)]
pub struct ConstructorSignature {
    pub args: Vec<ConstructorSignatureArg>,
    pub result: Expr,
    pub span: Span,
}

/// A constructor inside an explicit `where` data block.
#[derive(Clone, Debug)]
pub enum ExplicitDataCtor {
    /// Simple default-result sugar, same surface as legacy `C τ...`.
    Simple(CtorDecl),
    /// Full constructor signature `C : telescope -> D params indices`.
    Signature {
        name: String,
        signature: ConstructorSignature,
        span: Span,
    },
}

impl ExplicitDataCtor {
    pub fn name(&self) -> &str {
        match self {
            ExplicitDataCtor::Simple(c) => &c.name,
            ExplicitDataCtor::Signature { name, .. } => name,
        }
    }

    pub fn span(&self) -> &Span {
        match self {
            ExplicitDataCtor::Simple(c) => &c.span,
            ExplicitDataCtor::Signature { span, .. } => span,
        }
    }
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

/// A class field declaration, with optional SURF-2 purity metadata.
#[derive(Clone, Debug)]
pub struct ClassField {
    pub purity: Option<DefKeyword>,
    pub name: String,
    pub ty: Type,
}

/// A prerequisite dictionary on an instance declaration.
#[derive(Clone, Debug)]
pub struct InstanceConstraint {
    pub class_name: String,
    pub head_type: Type,
    pub binder: Option<String>,
}

/// A constructor-style intro helper inside a `prop ... where { ... }` block.
#[derive(Clone, Debug)]
pub struct PropIntro {
    pub name: String,
    pub ty: Type,
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
    /// Anonymous `program` / `package` instance-admission boundary
    /// (`33 §3.2.1`, §5.5.1). The enclosing dotted file path is its identity;
    /// the header deliberately carries no name or entry-point declaration.
    BoundaryDecl {
        kind: BoundaryKind,
        /// Present iff an `admits` clause was written. `None` is distinct
        /// from a clause value, so admission and capability readers remain
        /// structurally independent.
        admits: Option<Vec<String>>,
        /// Present iff a `capabilities` clause was written. Packages never
        /// carry this field; the parser rejects that surface before loading.
        capabilities: Option<Vec<CapabilityDecl>>,
        span: Span,
    },
    ViewDecl {
        keyword: DefKeyword,
        name: String,
        params: Vec<Binder>,
        ret_ty: Option<Type>,
        /// `requires φ` clauses (V1; empty in V0 programs).
        requires: Vec<Expr>,
        /// `ensures ψ` clauses (V1; empty in V0 programs).
        ensures: Vec<Expr>,
        /// `where C₁ T₁, (d₂ : C₂ T₂)` class constraints (`37 §6`, L3b).
        /// This deliberately shares the instance-path representation so
        /// naming, binding, and use-site resolution cannot diverge.
        constraints: Vec<InstanceConstraint>,
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
    /// `prop P ... : Omega where { intro : P ... }` — proposition-family
    /// claim shape (`33 §8.1`), elaborated to existing Ω-checked terms only.
    PropDecl {
        name: String,
        params: Vec<Binder>,
        ret_ty: Type,
        intros: Vec<PropIntro>,
        span: Span,
    },
    /// `lemma name ... : φ = proof` — standalone checked proof theorem
    /// (`33 §8.3`), ordinary module namespace.
    LemmaDecl {
        name: String,
        params: Vec<Binder>,
        theorem: Type,
        body: Expr,
        span: Span,
    },
    /// `axiom name : T` — mechanical sugar for
    /// `lemma name : T = Axiom`.
    AxiomDecl {
        name: String,
        theorem: Type,
        span: Span,
    },
    /// `proof p for subject ... : φ = proof` — attached checked proof theorem
    /// (`33 §8.2`), exported only through `subject::p`.
    AttachedProofDecl {
        proof_name: String,
        subject: String,
        params: Vec<Binder>,
        theorem: Type,
        body: Expr,
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
    /// `data D (Δp) : Δi -> Type where { ... }` — explicit inductive-family
    /// syntax (`34 §2`), lowered by the data elaborator to a kernel inductive
    /// family.
    ExplicitDataDecl {
        /// Type former name (uppercase-initial).
        name: String,
        /// Parameter binders before the colon.
        params: Vec<Binder>,
        /// Family result/index telescope after the colon.
        family: Type,
        /// Constructors in declaration order.
        ctors: Vec<ExplicitDataCtor>,
        span: Span,
    },
    /// `def T = A` — surface definition (alias/refinement); was `type`.
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
        /// Field declarations, in Sigma-telescope order.
        fields: Vec<ClassField>,
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
        /// Sub-constraints and their optional explicit dictionary binders.
        constraints: Vec<InstanceConstraint>,
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
    /// `import M` / `import M as N` / `import M (foo, Bar)` (`33 §3.2`)
    /// — brings another module's `pub` exports into scope.
    /// Surface/elaboration-time only; resolved away before the kernel.
    ImportDecl {
        module: String,
        kind: ImportKind,
        span: Span,
    },
    /// `export M (foo, Bar as baz)` / `export foo, Bar as baz`
    /// (`33 §3.2`) — republishes existing canonical identities through this
    /// module's interface without allocating a kernel declaration.
    ExportDecl { form: ExportForm, span: Span },
    /// `pub <decl>` — marks the wrapped top-level decl's name as exported
    /// from its enclosing module (`33 §4.1`). Top-level (non-module) decls
    /// may also be wrapped; the marker is simply inert there (nothing to
    /// export from). Nested modules are unaffected by a `pub` on decls
    /// inside them other than the module's own name marker (modules
    /// themselves have no separate visibility — only their contents do).
    Pub(Box<Decl>),
}

/// The three import forms (`33 §3.2`).
#[derive(Clone, Debug)]
pub enum ImportKind {
    /// `import M` — qualified: `M`'s exports accessible as `M.foo`.
    Qualified,
    /// `import M as N` — aliased: exports accessible as `N.foo`.
    Aliased(String),
    /// `import M (foo, Bar as Baz)` — selective: exactly these names,
    /// optionally renamed at the import site, unqualified.
    Selective(Vec<ImportItem>),
}

/// One selective-import item (`33 §3.2`). `rename` is the unqualified name
/// installed in the importing scope; the declaration identity remains
/// `module.name`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportItem {
    pub name: String,
    pub rename: Option<String>,
}

/// The two re-export forms (`33 §3.2`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExportForm {
    /// A direct loader edge whose selected names are published but are not
    /// installed in the facade module's body scope.
    Facade {
        module: String,
        items: Vec<ImportItem>,
    },
    /// Republishes names already available in the current body scope.
    InScope { items: Vec<ImportItem> },
}

/// The two hosts of the N4 instance-admission boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoundaryKind {
    Program,
    Package,
}

/// One `capabilities Family Authority` item from an anonymous program header.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapabilityDecl {
    pub family: String,
    pub authority: String,
}

/// The runner-readable projection of a parsed anonymous boundary header.
///
/// This is surface data only. It neither mints a capability nor introduces a
/// kernel declaration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundaryHeader {
    pub kind: BoundaryKind,
    pub admits: Option<Vec<String>>,
    pub capabilities: Option<Vec<CapabilityDecl>>,
}

impl Decl {
    pub fn name(&self) -> &str {
        match self {
            Decl::Pub(inner) => inner.name(),
            // A boundary has no declared name. This sentinel is never entered
            // into a scope or the flat kernel environment.
            Decl::BoundaryDecl { .. } => "",
            Decl::ModuleDecl { name, .. } => name,
            // `ImportDecl` has no declared name of its own; callers that
            // need a per-decl name must special-case it (it's never
            // registered as a global).
            Decl::ImportDecl { module, .. } => module,
            // An export declaration has no declaration-level name.
            Decl::ExportDecl { .. } => "",
            Decl::ViewDecl { name, .. }
            | Decl::LetDecl { name, .. }
            | Decl::ProveDecl { name, .. }
            | Decl::PropDecl { name, .. }
            | Decl::LemmaDecl { name, .. }
            | Decl::AxiomDecl { name, .. }
            | Decl::LawDecl { name, .. }
            | Decl::DataDecl { name, .. }
            | Decl::ExplicitDataDecl { name, .. }
            | Decl::TypeAlias { name, .. }
            | Decl::ForeignDecl { name, .. }
            | Decl::TemporalDecl { name, .. }
            | Decl::ClassDecl { name, .. } => name,
            Decl::InstanceDecl { class_name, .. } => class_name,
            Decl::DeriveDecl { class_name, .. } => class_name,
            Decl::AttachedProofDecl { proof_name, .. } => proof_name,
        }
    }
    pub fn span(&self) -> &Span {
        match self {
            Decl::Pub(inner) => inner.span(),
            Decl::BoundaryDecl { span, .. }
            | Decl::ViewDecl { span, .. }
            | Decl::LetDecl { span, .. }
            | Decl::ProveDecl { span, .. }
            | Decl::PropDecl { span, .. }
            | Decl::LemmaDecl { span, .. }
            | Decl::AxiomDecl { span, .. }
            | Decl::AttachedProofDecl { span, .. }
            | Decl::LawDecl { span, .. }
            | Decl::DataDecl { span, .. }
            | Decl::ExplicitDataDecl { span, .. }
            | Decl::TypeAlias { span, .. }
            | Decl::ForeignDecl { span, .. }
            | Decl::TemporalDecl { span, .. }
            | Decl::ClassDecl { span, .. }
            | Decl::InstanceDecl { span, .. }
            | Decl::DeriveDecl { span, .. }
            | Decl::ModuleDecl { span, .. }
            | Decl::ImportDecl { span, .. }
            | Decl::ExportDecl { span, .. } => span,
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

/// One source binding in a sequential local `let` group.
#[derive(Clone, Debug)]
pub struct LetBinding {
    pub name: String,
    pub name_span: Span,
    pub annotation: Option<Type>,
    pub annotation_span: Option<Span>,
    pub value: Box<Expr>,
    pub span: Span,
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
    /// `let x : A = e; y = f x in body` — sequential local binding group.
    ELet(Vec<LetBinding>, Box<Expr>, Span),
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
        /// Optional `eqn: h` dependent-match modifier.
        equation: Option<String>,
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
    /// `subject::proof` or `(proof proof for subject)` — canonical attached
    /// proof reference. Resolver/module rewriting turns this into an ordinary
    /// global name after resolving the subject path first.
    EAttachedProofRef {
        subject: String,
        proof_name: String,
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
            | Expr::ELet(_, _, s)
            | Expr::EAsc(_, _, s)
            | Expr::EOld(_, s)
            | Expr::ENumLit(_, s)
            | Expr::EStr(_, s)
            | Expr::EPi(_, _, _, s)
            | Expr::EArrow(_, _, s)
            | Expr::EAttachedProofRef { span: s, .. }
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
    /// `A ->[ρ] B` — latent-effect arrow. The row is surface metadata for
    /// purity classification and erases to the same kernel Π as `TArr`.
    TEffectArr(Box<Type>, EffectRowSyntax, Box<Type>, Span),
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
            | Type::TEffectArr(_, _, _, s)
            | Type::TUniv(_, s)
            | Type::TCon(_, s)
            | Type::TVar(_, s)
            | Type::TRefine(_, _, _, s)
            | Type::TApp(_, _, s) => s,
        }
    }
}
