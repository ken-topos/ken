//! Name resolution: surface AST → resolved AST (`39 §5.3`, `21 §6.1`).
//!
//! V1 additions: resolves `requires`/`ensures` clause lists, scopes `result`
//! only into `ensures`, scopes `old` only into `space`-op `ensures`, resolves
//! `{x:A|φ}` refinement types, `prove`, and `law` declarations.
//! L2 additions: `data` declarations, `type` aliases, `match` expressions,
//! type application (`T a b`).

use crate::ast::{
    BinOp, ClassField, ConstructorSignatureArg, Decl, DefKeyword, EffectRowSyntax,
    ExplicitDataCtor, Expr, NumLit, PatKind, Type,
};
use crate::error::{ElabError, Span};

/// A resolved constructor declaration (from `data` decl resolution).
#[derive(Clone, Debug)]
pub struct RCtorDecl {
    pub name: String,
    pub args: Vec<RType>,
    pub field_labels: Option<Vec<String>>,
    pub span: Span,
}

/// A resolved data-head parameter or constructor telescope entry.
#[derive(Clone, Debug)]
pub struct RTelescopeEntry {
    pub name: Option<String>,
    pub ty: RType,
    pub span: Span,
}

/// A resolved constructor in an explicit `data ... where` family declaration.
#[derive(Clone, Debug)]
pub struct RExplicitCtorDecl {
    pub name: String,
    pub args: Vec<RTelescopeEntry>,
    /// `None` means simple default-result sugar; only valid for non-indexed
    /// explicit families.
    pub result: Option<RType>,
    pub span: Span,
}

/// A resolved pattern.
#[derive(Clone, Debug)]
pub struct RPattern {
    pub kind: RPatKind,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum RPatKind {
    Wild,
    Var(String),
    Ctor(String, Vec<RPattern>),
}

/// A resolved match arm.
#[derive(Clone, Debug)]
pub struct RMatchArm {
    pub pat: RPattern,
    pub body: RExpr,
    pub span: Span,
}

/// A resolved declaration (`21 §6.2`).
#[derive(Clone, Debug)]
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

/// A resolved class field declaration, with optional SURF-2 purity metadata.
#[derive(Clone, Debug)]
pub struct RClassField {
    pub purity: Option<DefKeyword>,
    pub name: String,
    pub ty: RType,
}

/// Discriminates the declaration kind for elaboration dispatch.
#[derive(Clone, Debug)]
pub enum RDeclKind {
    /// A definition using legacy `view` or SURF-1 `const`/`fn`/`proc`.
    /// `constraints` = `where C T` list resolved from the surface `where`
    /// clause; checked against `instance_search` in `elaborate_rdecl_v1`.
    View {
        keyword: DefKeyword,
        is_space_op: bool,
        constraints: Vec<(String, RType)>,
        visits: Option<EffectRowSyntax>,
    },
    /// A `let` binding.
    Let,
    /// A `prove name : φ` standalone obligation.
    Prove,
    /// `prop P ... : Omega where { ... }`.
    Prop { intros: Vec<RPropIntro> },
    /// `lemma name ... : theorem = proof`.
    Lemma,
    /// `proof p for subject ... : theorem = proof`.
    AttachedProof { subject: String, proof_name: String },
    /// A `law Name (param) { field : φ ; … }` bundle.
    Law {
        param: String,
        fields: Vec<(String, RExpr)>,
    },
    /// A `data D p₁…pₙ = C₁ τ… | …` inductive type (`34 §1`).
    DataDecl {
        type_params: Vec<String>,
        ctors: Vec<RCtorDecl>,
    },
    /// A `data D (Δp) : (Δi) -> Type where { ... }` inductive family (`34 §2`).
    ExplicitDataDecl {
        params: Vec<RTelescopeEntry>,
        indices: Vec<RTelescopeEntry>,
        level: Option<u32>,
        ctors: Vec<RExplicitCtorDecl>,
    },
    /// A `def T = A` surface definition (alias/refinement); was `type`.
    TypeAlias { ty: RType },
    /// `foreign f : T = "symbol" "library" [pure] [E1, …]` (`38 §2.1`).
    Foreign {
        symbol: String,
        library: String,
        is_pure: bool,
        visits: Vec<String>,
    },
    /// `temporal name { φ }` — a delegated temporal obligation (`72 §4`).
    /// The `TemporalExpr` is carried through: its atoms are event-predicate
    /// names over `Σ` (not term variables), so it needs no de Bruijn
    /// resolution. `source` is the verbatim formula text (human-visible).
    /// Elaboration expands the derived operators to the §3 core.
    Temporal {
        formula: crate::temporal::TemporalExpr,
        source: String,
    },
    /// `class C A { field : Type ; … }` (`33 §5`).
    ClassDecl {
        param: Option<String>,
        /// Optional resolved kind for the class parameter. Absent means `Type0`.
        param_kind: Option<RType>,
        /// Resolved field types (param and earlier fields in scope if present).
        fields: Vec<RClassField>,
    },
    /// `instance C HeadType [where …] { field = expr ; … }` (`39 §6`).
    InstanceDecl {
        /// Free lowercase type variables generalized from the instance head.
        /// Each is implicitly bound at `Type0`.
        head_params: Vec<String>,
        head_type: RType,
        /// Resolved constraint list: (class_name, head_type).
        constraints: Vec<(String, RType)>,
        /// Resolved field implementations: (name, expr).
        fields: Vec<(String, RExpr)>,
    },
    /// `derive ClassName for DataName` (`33 §5.6`, `39 §6.6`).
    DeriveDecl { data_name: String },
}

#[derive(Clone, Debug)]
pub struct RPropIntro {
    pub name: String,
    pub ty: RType,
    pub span: Span,
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
    /// Numeric literal (`35 §4.1`).
    RNumLit(NumLit, Span),
    /// String literal (`37 §2.1`, VAL1-surface).
    RStr(String, Span),
    /// Infix binary op (`35 §3`); names resolved, operands still unelab'd.
    RBinOp(BinOp, Box<RExpr>, Box<RExpr>, Span),
    /// `match scrut { P₁ => body₁ ; … }` — pattern match (`34 §3`).
    RMatch {
        scrut: Box<RExpr>,
        arms: Vec<RMatchArm>,
        span: Span,
    },
    /// `e.field` — Σ-record field projection (`33 §5.2` η).
    RProj(Box<RExpr>, String, Span),
    /// `(x : A) -> B` — dependent function type, expr position (VAL2 #4).
    /// Domain resolved as a `type` (mirrors type-position `Pi`); codomain
    /// resolved as an expr with `x` bound.
    RPi(String, Box<RType>, Box<RExpr>, Span),
    /// `A -> B` — non-dependent function type, expr position (VAL2 #4).
    /// Both sides resolved as exprs; right-associative.
    RArrow(Box<RExpr>, Box<RExpr>, Span),
    /// `subject::proof` / `(proof proof for subject)`.
    RAttachedProofRef {
        subject: String,
        proof_name: String,
        span: Span,
    },
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
            | RExpr::ROld(_, s)
            | RExpr::RNumLit(_, s)
            | RExpr::RStr(_, s)
            | RExpr::RProj(_, _, s)
            | RExpr::RPi(_, _, _, s)
            | RExpr::RArrow(_, _, s)
            | RExpr::RAttachedProofRef { span: s, .. }
            | RExpr::RBinOp(_, _, _, s) => s,
            RExpr::RMatch { span, .. } => span,
        }
    }
}

/// A resolved type expression.
#[derive(Clone, Debug)]
pub enum RType {
    RPi(String, Box<RType>, Box<RType>, Span),
    RArr(Box<RType>, Box<RType>, Span),
    /// `A ->[ρ] B` — latent-effect arrow, erased to the same kernel Π as
    /// `RArr` after class-field purity has inspected the row.
    REffectArr(Box<RType>, EffectRowSyntax, Box<RType>, Span),
    RUniv(Option<u32>, Span),
    RCon(String, Span),
    RVarTy(usize, String, Span),
    /// `{ x : A | φ }` — carrier `A` + tracked predicate `φ` (`21 §6.1`).
    RRefine(String, Box<RType>, Box<RExpr>, Span),
    /// `T a b` — type-level application (`34 §1`).
    RApp(Box<RType>, Box<RType>, Span),
}

impl RType {
    pub fn span(&self) -> &Span {
        match self {
            RType::RPi(_, _, _, s)
            | RType::RArr(_, _, s)
            | RType::REffectArr(_, _, _, s)
            | RType::RUniv(_, s)
            | RType::RCon(_, s)
            | RType::RVarTy(_, _, s)
            | RType::RRefine(_, _, _, s)
            | RType::RApp(_, _, s) => s,
        }
    }
}

// ----- scope -----

#[derive(Clone)]
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

fn is_instance_head_param(name: &str) -> bool {
    name.chars()
        .next()
        .map(|c| c == '_' || c.is_ascii_lowercase())
        .unwrap_or(false)
}

fn collect_instance_head_params(ty: &Type, out: &mut Vec<String>) {
    match ty {
        Type::TVar(name, _) if is_instance_head_param(name) => {
            if !out.iter().any(|p| p == name) {
                out.push(name.clone());
            }
        }
        Type::TPi(_, a, b, _)
        | Type::TArr(a, b, _)
        | Type::TEffectArr(a, _, b, _)
        | Type::TApp(a, b, _) => {
            collect_instance_head_params(a, out);
            collect_instance_head_params(b, out);
        }
        Type::TRefine(_, carrier, _, _) => collect_instance_head_params(carrier, out),
        Type::TUniv(_, _) | Type::TCon(_, _) | Type::TVar(_, _) => {}
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

fn unsupported_constructor_type_expr(expr: &Expr) -> ElabError {
    ElabError::ParseError {
        msg: "constructor signatures support type-forming expressions here; strings, matches, projections, lets, lambdas, and unsupported infix expressions are staged out".into(),
        span: expr.span().clone(),
    }
}

fn expr_as_type(expr: &Expr) -> Result<Type, ElabError> {
    match expr {
        Expr::EVar(name, span) | Expr::ECon(name, span) => {
            Ok(Type::TVar(name.clone(), span.clone()))
        }
        Expr::EUniv(level, span) => Ok(Type::TUniv(*level, span.clone())),
        Expr::EApp(f, a, span) => Ok(Type::TApp(
            Box::new(expr_as_type(f)?),
            Box::new(expr_as_type(a)?),
            span.clone(),
        )),
        Expr::EPi(name, domain, codomain, span) => Ok(Type::TPi(
            name.clone(),
            domain.clone(),
            Box::new(expr_as_type(codomain)?),
            span.clone(),
        )),
        Expr::EArrow(domain, codomain, span) => Ok(Type::TArr(
            Box::new(expr_as_type(domain)?),
            Box::new(expr_as_type(codomain)?),
            span.clone(),
        )),
        Expr::ENumLit(NumLit::Int(0), span) => Ok(Type::TVar("Zero".to_string(), span.clone())),
        Expr::EBinOp(BinOp::Add, lhs, rhs, span) => {
            if matches!(rhs.as_ref(), Expr::ENumLit(NumLit::Int(1), _)) {
                Ok(Type::TApp(
                    Box::new(Type::TVar("Suc".to_string(), span.clone())),
                    Box::new(expr_as_type(lhs)?),
                    span.clone(),
                ))
            } else {
                Err(unsupported_constructor_type_expr(expr))
            }
        }
        Expr::ELam(..)
        | Expr::ELet(..)
        | Expr::EAsc(..)
        | Expr::EOld(..)
        | Expr::ENumLit(..)
        | Expr::EStr(..)
        | Expr::EBinOp(..)
        | Expr::EMatch { .. }
        | Expr::EProj(..)
        | Expr::EAttachedProofRef { .. } => Err(unsupported_constructor_type_expr(expr)),
    }
}

fn require_single_binder(names: &[String], span: &Span, what: &str) -> Result<(), ElabError> {
    if names.len() == 1 {
        Ok(())
    } else {
        Err(ElabError::ParseError {
            msg: format!(
                "{} binder groups are staged out; write one binder at a time",
                what
            ),
            span: span.clone(),
        })
    }
}

fn resolve_explicit_family_indices(
    scope: &mut Scope,
    family: &Type,
) -> Result<(Vec<RTelescopeEntry>, Option<u32>), ElabError> {
    match family {
        Type::TUniv(level, _) => Ok((vec![], *level)),
        Type::TArr(domain, codomain, span) | Type::TEffectArr(domain, _, codomain, span) => {
            let domain_r = resolve_type(scope, domain)?;
            let name = format!("_index{}", scope.depth());
            scope.push(&name);
            let (mut rest, level) = resolve_explicit_family_indices(scope, codomain)?;
            scope.pop();
            let entry = RTelescopeEntry {
                name: None,
                ty: domain_r,
                span: span.clone(),
            };
            rest.insert(0, entry);
            Ok((rest, level))
        }
        Type::TPi(name, domain, codomain, span) => {
            let domain_r = resolve_type(scope, domain)?;
            scope.push(name);
            let (mut rest, level) = resolve_explicit_family_indices(scope, codomain)?;
            scope.pop();
            let entry = RTelescopeEntry {
                name: Some(name.clone()),
                ty: domain_r,
                span: span.clone(),
            };
            rest.insert(0, entry);
            Ok((rest, level))
        }
        _ => Err(ElabError::ParseError {
            msg: "explicit data family head must be an index telescope ending in Type".into(),
            span: family.span().clone(),
        }),
    }
}

fn resolve_explicit_ctor(
    params_scope: &Scope,
    ctor: &ExplicitDataCtor,
) -> Result<RExplicitCtorDecl, ElabError> {
    let mut scope = params_scope.clone();
    match ctor {
        ExplicitDataCtor::Simple(c) => {
            let mut args = Vec::new();
            for (idx, ty) in c.args.iter().enumerate() {
                let rty = resolve_type(&mut scope, ty)?;
                args.push(RTelescopeEntry {
                    name: c
                        .field_labels
                        .as_ref()
                        .and_then(|labels| labels.get(idx).cloned()),
                    ty: rty,
                    span: ty.span().clone(),
                });
                let name = format!("_arg{}", scope.depth());
                scope.push(&name);
            }
            Ok(RExplicitCtorDecl {
                name: c.name.clone(),
                args,
                result: None,
                span: c.span.clone(),
            })
        }
        ExplicitDataCtor::Signature {
            name,
            signature,
            span,
        } => {
            let mut args = Vec::new();
            for arg in &signature.args {
                match arg {
                    ConstructorSignatureArg::Explicit(binder)
                    | ConstructorSignatureArg::Implicit(binder) => {
                        require_single_binder(
                            &binder.names,
                            &binder.span,
                            "explicit data constructor",
                        )?;
                        let rty = resolve_type(&mut scope, &binder.ty)?;
                        let name = binder.names[0].clone();
                        scope.push(&name);
                        args.push(RTelescopeEntry {
                            name: Some(name),
                            ty: rty,
                            span: binder.span.clone(),
                        });
                    }
                    ConstructorSignatureArg::Anonymous(expr) => {
                        let ty = expr_as_type(expr)?;
                        let rty = resolve_type(&mut scope, &ty)?;
                        args.push(RTelescopeEntry {
                            name: None,
                            ty: rty,
                            span: expr.span().clone(),
                        });
                        let name = format!("_arg{}", scope.depth());
                        scope.push(&name);
                    }
                }
            }
            let result_ty = expr_as_type(&signature.result)?;
            let result = resolve_type(&mut scope, &result_ty)?;
            Ok(RExplicitCtorDecl {
                name: name.clone(),
                args,
                result: Some(result),
                span: span.clone(),
            })
        }
    }
}

// ----- reserved surface sugar (FR-2, `docs/program/wp/
// ds-1-findings-remediation.md`) -----

/// `elab.rs`'s five checked-mode surface sugar identifiers, named here so
/// both `elab.rs`'s interception arms and this module's declaration guard
/// below read the SAME constants and can't drift apart. They do NOT all
/// intercept the same way — this is exactly the distinction the Architect's
/// original FR-2 pin missed and then corrected (`docs/program/wp/
/// ds-1-findings-remediation.md` FR-2):
///
/// - `Refl`/`Axiom` — a bare `RCon`, TOTAL intercept at any arity (zero
///   arguments needed): a declared global of either name is wholly
///   unreachable, full stop.
/// - `absurd` — `RApp(RCon("absurd"), arg)`, arity-**1** only.
/// - `J`/`Eq` — `peel_named_app(_, name, 3)`, arity-**3** only, BY DESIGN so
///   a lower-arity type-former/class of the same name coexists (the landed
///   `class Eq a` — spec-grounded, `51-lawful-classes.md §2.1` — is arity-1
///   and never collides with the arity-3 kernel-equality sugar `Eq A a b`;
///   the elaborator's own arity gate already disambiguates the two).
pub(crate) const SUGAR_REFL: &str = "Refl";
pub(crate) const SUGAR_AXIOM: &str = "Axiom";
pub(crate) const SUGAR_ABSURD: &str = "absurd";
pub(crate) const SUGAR_J: &str = "J";
pub(crate) const SUGAR_EQ: &str = "Eq";

/// The declaration-time hard-error set — ONLY the names above whose
/// interception is total/arity-independent (`Refl`, `Axiom`, `absurd`).
/// `J`/`Eq` are deliberately excluded: their arity-3-gated coexistence with
/// lower-arity type-formers/classes is the intended, working design (a
/// declaration-time name reject would be the wrong tool — it would break
/// every legitimate `Eq A a b`/`J _ _ _` use in any file that also declares
/// a lower-arity `Eq`/`J`, i.e. most of the catalog: `DecEq`, `map`,
/// `EmptyDec.ken.md` all pull in `class Eq`). A user-declared arity-3
/// type-former literally named `Eq`/`J` remains a real, but deliberately
/// out-of-scope, reservation — not a bug this guard closes (see the
/// Architect's FR-2 ruling, `docs/program/wp/ds-1-findings-remediation.md`).
///
/// `Cast`/`Ascript` are excluded for a different reason: they're
/// `ken-kernel::term::Term` variants reached only via ordinary elaboration
/// of real surface syntax (a type ascription, the `cast` derived combinator
/// in `Core/Transport.ken`) — grep-confirmed no `RCon("Cast")`/
/// `RCon("Ascript")` match arm exists anywhere in
/// `elab.rs`/`resolve.rs`/`parser.rs`, so neither shadows a user global the
/// way `Refl`/`Axiom`/`absurd` do.
pub(crate) const RESERVED_SUGAR: &[&str] = &[SUGAR_REFL, SUGAR_AXIOM, SUGAR_ABSURD];

/// Resolve-time hard error (fail-closed) when a declared name collides with
/// a totally-reserved sugar identifier — the collision diagnostic FR-2 adds
/// in place of DS-1's silent-shadow footgun. Grep-confirmed none of
/// `RESERVED_SUGAR` is a prelude global, so this can never reject the
/// bootstrap.
fn check_no_reserved_sugar_collision(name: &str, span: &Span) -> Result<(), ElabError> {
    if RESERVED_SUGAR.contains(&name) {
        return Err(ElabError::ParseError {
            msg: format!(
                "'{name}' collides with a reserved surface sugar identifier — \
                 checked-mode sugar intercepts every syntactic '{name}' \
                 regardless of this declaration, leaving it permanently \
                 unreachable; choose a different name"
            ),
            span: span.clone(),
        });
    }
    Ok(())
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
    // The single decl-kind-uniform funnel every fn/const/proc/def/data/class/
    // instance declaration passes through — guard the produced decl's own
    // name here, not at the ~12 downstream `globals.insert` call sites or a
    // per-use path. `ModuleDecl`/`ImportDecl`/`Pub` are expanded away by
    // `modules.rs` before reaching this function (see the arm below) and
    // have no declared VALUE name of their own to guard here.
    if !matches!(
        decl,
        Decl::ModuleDecl { .. } | Decl::ImportDecl { .. } | Decl::Pub(_)
    ) {
        check_no_reserved_sugar_collision(decl.name(), decl.span())?;
    }
    match decl {
        // `module`/`import`/`use`/`pub` are resolved away entirely by
        // `modules.rs` before any decl reaches this function — it always
        // hands `resolve_decl` an already-unwrapped, already-qualified
        // ordinary decl. Unreachable from that pipeline; kept exhaustive
        // for `Decl`'s other (non-`ken-elaborator`-internal) callers.
        Decl::ModuleDecl { .. } | Decl::ImportDecl { .. } | Decl::Pub(_) => {
            Err(ElabError::Internal(
                "resolve_decl: module/import/pub decls must be expanded by modules.rs first".into(),
            ))
        }
        Decl::ViewDecl {
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

            let full_ty = match resolved_ret {
                Some(ret) => Some(single_binders.into_iter().rev().fold(
                    ret,
                    |acc, (bname, bty)| {
                        let s = acc.span().clone();
                        RType::RPi(bname, Box::new(bty), Box::new(acc), s)
                    },
                )),
                None => None,
            };

            // Resolve `where C T` constraints — types resolved in param scope
            // (so type vars from params are in scope, `39 §6`).
            let resolved_constraints = constraints
                .iter()
                .map(|(cname, cty)| {
                    // Use a fresh scope per constraint (each C T is standalone).
                    let mut cscope = Scope::new();
                    let rty = resolve_type(&mut cscope, cty)?;
                    Ok((cname.clone(), rty))
                })
                .collect::<Result<Vec<_>, ElabError>>()?;

            Ok(RDecl {
                name: name.clone(),
                ty: full_ty,
                body: full_body,
                requires: resolved_requires,
                ensures: resolved_ensures,
                span: span.clone(),
                kind: RDeclKind::View {
                    keyword: *keyword,
                    is_space_op: *is_space_op,
                    constraints: resolved_constraints,
                    visits: visits.clone(),
                },
            })
        }

        Decl::LetDecl {
            name,
            ty,
            val,
            span,
        } => {
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

        Decl::PropDecl {
            name,
            params,
            ret_ty,
            intros,
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
            let resolved_ret = resolve_type(&mut scope, ret_ty)?;
            let full_ty =
                single_binders
                    .into_iter()
                    .rev()
                    .fold(resolved_ret, |acc, (bname, bty)| {
                        let s = acc.span().clone();
                        RType::RPi(bname, Box::new(bty), Box::new(acc), s)
                    });
            let resolved_intros = intros
                .iter()
                .map(|intro| {
                    Ok(RPropIntro {
                        name: intro.name.clone(),
                        ty: resolve_type(&mut scope, &intro.ty)?,
                        span: intro.span.clone(),
                    })
                })
                .collect::<Result<Vec<_>, ElabError>>()?;
            Ok(RDecl {
                name: name.clone(),
                ty: Some(full_ty),
                body: RExpr::RUniv(None, span.clone()),
                requires: vec![],
                ensures: vec![],
                span: span.clone(),
                kind: RDeclKind::Prop {
                    intros: resolved_intros,
                },
            })
        }

        Decl::LemmaDecl {
            name,
            params,
            theorem,
            body,
            span,
        } => {
            let mut scope = Scope::new();
            let (full_ty, full_body) =
                resolve_checked_proof_decl(params, theorem, body, &mut scope)?;
            Ok(RDecl {
                name: name.clone(),
                ty: Some(full_ty),
                body: full_body,
                requires: vec![],
                ensures: vec![],
                span: span.clone(),
                kind: RDeclKind::Lemma,
            })
        }

        Decl::AttachedProofDecl {
            proof_name,
            subject,
            params,
            theorem,
            body,
            span,
        } => {
            let mut scope = Scope::new();
            let (full_ty, full_body) =
                resolve_checked_proof_decl(params, theorem, body, &mut scope)?;
            Ok(RDecl {
                name: format!("{subject}::{proof_name}"),
                ty: Some(full_ty),
                body: full_body,
                requires: vec![],
                ensures: vec![],
                span: span.clone(),
                kind: RDeclKind::AttachedProof {
                    subject: subject.clone(),
                    proof_name: proof_name.clone(),
                },
            })
        }

        Decl::LawDecl {
            name,
            param,
            fields,
            span,
        } => {
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
                body: RExpr::RUniv(None, span.clone()),
                requires: vec![],
                ensures: vec![],
                span: span.clone(),
                kind: RDeclKind::Law {
                    param: param.clone(),
                    fields: resolved_fields,
                },
            })
        }

        Decl::DataDecl {
            name,
            type_params,
            ctors,
            span,
        } => {
            // Each type param is in scope (as a type variable) for the ctor args.
            let mut scope = Scope::new();
            for p in type_params {
                scope.push(p);
            }
            let mut rctors = Vec::new();
            for c in ctors {
                // A constructor resolves as an `RCon` too (`33 §4.2`) — a
                // `data … = Eq | …` ctor collides with the reserved sugar
                // identically to a decl head bearing the same name.
                check_no_reserved_sugar_collision(&c.name, &c.span)?;
                let rargs = c
                    .args
                    .iter()
                    .map(|t| resolve_type(&mut scope, t))
                    .collect::<Result<Vec<_>, _>>()?;
                rctors.push(RCtorDecl {
                    name: c.name.clone(),
                    args: rargs,
                    field_labels: c.field_labels.clone(),
                    span: c.span.clone(),
                });
            }
            Ok(RDecl {
                name: name.clone(),
                ty: None,
                body: RExpr::RUniv(None, span.clone()),
                requires: vec![],
                ensures: vec![],
                span: span.clone(),
                kind: RDeclKind::DataDecl {
                    type_params: type_params.clone(),
                    ctors: rctors,
                },
            })
        }

        Decl::ExplicitDataDecl {
            name,
            params,
            family,
            ctors,
            span,
        } => {
            let mut scope = Scope::new();
            let mut rparams = Vec::new();
            for binder in params {
                require_single_binder(&binder.names, &binder.span, "explicit data parameter")?;
                let rty = resolve_type(&mut scope, &binder.ty)?;
                let pname = binder.names[0].clone();
                scope.push(&pname);
                rparams.push(RTelescopeEntry {
                    name: Some(pname),
                    ty: rty,
                    span: binder.span.clone(),
                });
            }
            let (indices, level) = resolve_explicit_family_indices(&mut scope, family)?;
            for ctor in ctors {
                // Same collision as the legacy-form sweep above — an
                // explicit-family ctor resolves as an `RCon` too.
                check_no_reserved_sugar_collision(ctor.name(), ctor.span())?;
            }
            let rctors = ctors
                .iter()
                .map(|ctor| resolve_explicit_ctor(&scope, ctor))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(RDecl {
                name: name.clone(),
                ty: None,
                body: RExpr::RUniv(level, span.clone()),
                requires: vec![],
                ensures: vec![],
                span: span.clone(),
                kind: RDeclKind::ExplicitDataDecl {
                    params: rparams,
                    indices,
                    level,
                    ctors: rctors,
                },
            })
        }

        Decl::TypeAlias { name, ty, span } => {
            let mut scope = Scope::new();
            let rty = resolve_type(&mut scope, ty)?;
            Ok(RDecl {
                name: name.clone(),
                ty: None,
                body: RExpr::RUniv(None, span.clone()),
                requires: vec![],
                ensures: vec![],
                span: span.clone(),
                kind: RDeclKind::TypeAlias { ty: rty },
            })
        }

        Decl::ForeignDecl {
            name,
            ty,
            symbol,
            library,
            is_pure,
            visits,
            span,
        } => {
            let mut scope = Scope::new();
            let rty = resolve_type(&mut scope, ty)?;
            Ok(RDecl {
                name: name.clone(),
                ty: Some(rty),
                body: RExpr::RUniv(None, span.clone()), // placeholder — unused for Foreign
                requires: vec![],
                ensures: vec![],
                span: span.clone(),
                kind: RDeclKind::Foreign {
                    symbol: symbol.clone(),
                    library: library.clone(),
                    is_pure: *is_pure,
                    visits: visits.clone(),
                },
            })
        }

        Decl::TemporalDecl {
            name,
            formula,
            source,
            span,
        } => {
            // The temporal formula is carried as-is — its atoms are event
            // names over `Σ`, not term variables, so no de Bruijn resolution.
            Ok(RDecl {
                name: name.clone(),
                ty: None,
                body: RExpr::RUniv(None, span.clone()), // placeholder — unused for Temporal
                requires: vec![],
                ensures: vec![],
                span: span.clone(),
                kind: RDeclKind::Temporal {
                    formula: formula.clone(),
                    source: source.clone(),
                },
            })
        }

        Decl::ClassDecl {
            name,
            param,
            param_kind,
            fields,
            span,
        } => {
            // Fields form a real Σ-telescope (`33 §5.2`): a later field's
            // type may reference an EARLIER field by name (a law like
            // `refl : (x:a) -> IsTrue (eq x x)` refers to the `eq` op
            // field declared above it). Push each field name into scope
            // right after resolving it, so it's a bound `RVarTy`/`RVar`
            // reference (matching the Sigma-chain's own binder depth) for
            // every subsequent field — never a stray global lookup.
            let mut scope = Scope::new();
            let resolved_param_kind = param_kind
                .as_ref()
                .map(|k| resolve_type(&mut scope, k))
                .transpose()?;
            if let Some(p) = param {
                scope.push(p);
            }
            let mut resolved_fields = Vec::new();
            for ClassField {
                purity,
                name: fname,
                ty,
            } in fields
            {
                let rty = resolve_type(&mut scope, ty)?;
                resolved_fields.push(RClassField {
                    purity: *purity,
                    name: fname.clone(),
                    ty: rty,
                });
                scope.push(fname);
            }
            Ok(RDecl {
                name: name.clone(),
                ty: None,
                body: RExpr::RUniv(None, span.clone()),
                requires: vec![],
                ensures: vec![],
                span: span.clone(),
                kind: RDeclKind::ClassDecl {
                    param: param.clone(),
                    param_kind: resolved_param_kind,
                    fields: resolved_fields,
                },
            })
        }

        Decl::InstanceDecl {
            class_name,
            head_type,
            constraints,
            fields,
            span,
        } => {
            let mut scope = Scope::new();
            let mut head_params = Vec::new();
            collect_instance_head_params(head_type, &mut head_params);
            for param in &head_params {
                scope.push(param);
            }
            let rhead = resolve_type(&mut scope, head_type)?;
            let rconstraints = constraints
                .iter()
                .map(|(cname, cty)| {
                    let rty = resolve_type(&mut scope, cty)?;
                    Ok((cname.clone(), rty))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let rfields = fields
                .iter()
                .map(|(fname, expr)| {
                    let rexpr = resolve_expr(&mut scope, expr)?;
                    Ok((fname.clone(), rexpr))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(RDecl {
                name: class_name.clone(),
                ty: None,
                body: RExpr::RUniv(None, span.clone()),
                requires: vec![],
                ensures: vec![],
                span: span.clone(),
                kind: RDeclKind::InstanceDecl {
                    head_params,
                    head_type: rhead,
                    constraints: rconstraints,
                    fields: rfields,
                },
            })
        }

        Decl::DeriveDecl {
            class_name,
            data_name,
            span,
        } => Ok(RDecl {
            name: class_name.clone(),
            ty: None,
            body: RExpr::RUniv(None, span.clone()),
            requires: vec![],
            ensures: vec![],
            span: span.clone(),
            kind: RDeclKind::DeriveDecl {
                data_name: data_name.clone(),
            },
        }),
    }
}

pub fn resolve_expr_standalone(expr: &Expr) -> Result<RExpr, ElabError> {
    let mut scope = Scope::new();
    resolve_expr(&mut scope, expr)
}

// ----- internal resolution -----

fn resolve_checked_proof_decl(
    params: &[crate::ast::Binder],
    theorem: &Type,
    body: &Expr,
    scope: &mut Scope,
) -> Result<(RType, RExpr), ElabError> {
    let mut single_binders: Vec<(String, RType)> = Vec::new();
    for b in params {
        let a = resolve_type(scope, &b.ty)?;
        for name_str in &b.names {
            single_binders.push((name_str.clone(), a.clone()));
            scope.push(name_str);
        }
    }
    let resolved_theorem = resolve_type(scope, theorem)?;
    let resolved_body = resolve_expr(scope, body)?;
    let full_body = single_binders
        .iter()
        .rev()
        .fold(resolved_body, |acc, (bname, _)| {
            let s = acc.span().clone();
            RExpr::RLam(bname.clone(), Box::new(acc), s)
        });
    let full_ty = single_binders
        .into_iter()
        .rev()
        .fold(resolved_theorem, |acc, (bname, bty)| {
            let s = acc.span().clone();
            RType::RPi(bname, Box::new(bty), Box::new(acc), s)
        });
    Ok((full_ty, full_body))
}

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
            if name == "result"
                && !matches!(ctx, PropCtx::PureViewEnsures | PropCtx::SpaceOpEnsures)
            {
                return Err(ElabError::UnboundName {
                    name: name.clone(),
                    span: span.clone(),
                });
            }
            if let Some(i) = scope.index_of(name) {
                Ok(RExpr::RVar(i, name.clone(), span.clone()))
            } else {
                // Local scope miss — fall through to global lookup (same as TVar).
                // The elaborator's RCon handler resolves the name against globals.
                Ok(RExpr::RCon(name.clone(), span.clone()))
            }
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
            if ctx != PropCtx::SpaceOpEnsures {
                return Err(ElabError::UnboundName {
                    name: "old".to_string(),
                    span: span.clone(),
                });
            }
            let re = resolve_expr_ctx(scope, e, ctx)?;
            Ok(RExpr::ROld(Box::new(re), span.clone()))
        }

        Expr::ENumLit(lit, span) => Ok(RExpr::RNumLit(lit.clone(), span.clone())),
        Expr::EStr(s, span) => Ok(RExpr::RStr(s.clone(), span.clone())),

        Expr::EBinOp(op, l, r, span) => {
            let rl = resolve_expr_ctx(scope, l, ctx)?;
            let rr = resolve_expr_ctx(scope, r, ctx)?;
            Ok(RExpr::RBinOp(*op, Box::new(rl), Box::new(rr), span.clone()))
        }

        Expr::EProj(e, field, span) => {
            let re = resolve_expr_ctx(scope, e, ctx)?;
            Ok(RExpr::RProj(Box::new(re), field.clone(), span.clone()))
        }

        Expr::EPi(x, a, b, span) => {
            let ra = resolve_type(scope, a)?;
            scope.push(x);
            let rb = resolve_expr_ctx(scope, b, ctx)?;
            scope.pop();
            Ok(RExpr::RPi(
                x.clone(),
                Box::new(ra),
                Box::new(rb),
                span.clone(),
            ))
        }

        Expr::EArrow(a, b, span) => {
            let ra = resolve_expr_ctx(scope, a, ctx)?;
            let rb = resolve_expr_ctx(scope, b, ctx)?;
            Ok(RExpr::RArrow(Box::new(ra), Box::new(rb), span.clone()))
        }

        Expr::EAttachedProofRef {
            subject,
            proof_name,
            span,
        } => Ok(RExpr::RAttachedProofRef {
            subject: subject.clone(),
            proof_name: proof_name.clone(),
            span: span.clone(),
        }),

        Expr::EMatch { scrut, arms, span } => {
            let rscrut = resolve_expr_ctx(scope, scrut, ctx)?;
            let mut rarms = Vec::new();
            for arm in arms {
                let (rpat, bound_names) = resolve_pattern(&arm.pat)?;
                let depth_before = scope.depth();
                for n in &bound_names {
                    scope.push(n);
                }
                let rbody = resolve_expr_ctx(scope, &arm.body, ctx)?;
                for _ in &bound_names {
                    scope.pop();
                }
                assert_eq!(scope.depth(), depth_before);
                rarms.push(RMatchArm {
                    pat: rpat,
                    body: rbody,
                    span: arm.span.clone(),
                });
            }
            Ok(RExpr::RMatch {
                scrut: Box::new(rscrut),
                arms: rarms,
                span: span.clone(),
            })
        }
    }
}

/// Resolve a pattern, returning the resolved pattern and the list of names
/// bound by it in left-to-right order (for scope introduction).
fn resolve_pattern(pat: &crate::ast::Pattern) -> Result<(RPattern, Vec<String>), ElabError> {
    match &pat.kind {
        // Wild patterns consume one de Bruijn slot so outer vars remain consistent
        // with the method lambda structure (which binds ALL ctor args, not just named ones).
        PatKind::Wild => Ok((
            RPattern {
                kind: RPatKind::Wild,
                span: pat.span.clone(),
            },
            vec!["_".to_string()],
        )),
        PatKind::Var(name) => Ok((
            RPattern {
                kind: RPatKind::Var(name.clone()),
                span: pat.span.clone(),
            },
            vec![name.clone()],
        )),
        PatKind::Ctor(name, subs) => {
            let mut rsubs = Vec::new();
            let mut all_names = Vec::new();
            for sub in subs {
                let (rpat, names) = resolve_pattern(sub)?;
                rsubs.push(rpat);
                all_names.extend(names);
            }
            Ok((
                RPattern {
                    kind: RPatKind::Ctor(name.clone(), rsubs),
                    span: pat.span.clone(),
                },
                all_names,
            ))
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
        Type::TEffectArr(a, row, b, span) => {
            let ra = resolve_type(scope, a)?;
            let rb = resolve_type(scope, b)?;
            Ok(RType::REffectArr(
                Box::new(ra),
                row.clone(),
                Box::new(rb),
                span.clone(),
            ))
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

        Type::TApp(f, a, span) => {
            let rf = resolve_type(scope, f)?;
            let ra = resolve_type(scope, a)?;
            Ok(RType::RApp(Box::new(rf), Box::new(ra), span.clone()))
        }
    }
}
