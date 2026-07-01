//! Name resolution: surface AST → resolved AST (`39 §5.3`, `21 §6.1`).
//!
//! V1 additions: resolves `requires`/`ensures` clause lists, scopes `result`
//! only into `ensures`, scopes `old` only into `space`-op `ensures`, resolves
//! `{x:A|φ}` refinement types, `prove`, and `law` declarations.
//! L2 additions: `data` declarations, `type` aliases, `match` expressions,
//! type application (`T a b`).

use crate::ast::{BinOp, Decl, Expr, NumLit, PatKind, Type};
use crate::error::{ElabError, Span};

/// A resolved constructor declaration (from `data` decl resolution).
#[derive(Clone, Debug)]
pub struct RCtorDecl {
    pub name: String,
    pub args: Vec<RType>,
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
    /// `constraints` = `where C T` list resolved from the surface `where`
    /// clause; checked against `instance_search` in `elaborate_rdecl_v1`.
    View { is_space_op: bool, constraints: Vec<(String, RType)> },
    /// A `let` binding.
    Let,
    /// A `prove name : φ` standalone obligation.
    Prove,
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
    /// A `type T = A` surface type alias.
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
        /// Resolved field types (param in scope if present).
        fields: Vec<(String, RType)>,
    },
    /// `instance C HeadType [where …] { field = expr ; … }` (`39 §6`).
    InstanceDecl {
        head_type: RType,
        /// Resolved constraint list: (class_name, head_type).
        constraints: Vec<(String, RType)>,
        /// Resolved field implementations: (name, expr).
        fields: Vec<(String, RExpr)>,
    },
    /// `derive ClassName for DataName` (`33 §5.6`, `39 §6.6`).
    DeriveDecl {
        data_name: String,
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
            | RType::RUniv(_, s)
            | RType::RCon(_, s)
            | RType::RVarTy(_, _, s)
            | RType::RRefine(_, _, _, s)
            | RType::RApp(_, _, s) => s,
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
            constraints,
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
                    is_space_op: *is_space_op,
                    constraints: resolved_constraints,
                },
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

        Decl::DataDecl { name, type_params, ctors, span } => {
            // Each type param is in scope (as a type variable) for the ctor args.
            let mut scope = Scope::new();
            for p in type_params {
                scope.push(p);
            }
            let mut rctors = Vec::new();
            for c in ctors {
                let rargs = c
                    .args
                    .iter()
                    .map(|t| resolve_type(&mut scope, t))
                    .collect::<Result<Vec<_>, _>>()?;
                rctors.push(RCtorDecl {
                    name: c.name.clone(),
                    args: rargs,
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

        Decl::ForeignDecl { name, ty, symbol, library, is_pure, visits, span } => {
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

        Decl::TemporalDecl { name, formula, source, span } => {
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

        Decl::ClassDecl { name, param, fields, span } => {
            let mut scope = Scope::new();
            if let Some(p) = param {
                scope.push(p);
            }
            let resolved_fields = fields
                .iter()
                .map(|(fname, ty)| {
                    let rty = resolve_type(&mut scope, ty)?;
                    Ok((fname.clone(), rty))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(RDecl {
                name: name.clone(),
                ty: None,
                body: RExpr::RUniv(None, span.clone()),
                requires: vec![],
                ensures: vec![],
                span: span.clone(),
                kind: RDeclKind::ClassDecl {
                    param: param.clone(),
                    fields: resolved_fields,
                },
            })
        }

        Decl::InstanceDecl { class_name, head_type, constraints, fields, span } => {
            let mut scope = Scope::new();
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
                    head_type: rhead,
                    constraints: rconstraints,
                    fields: rfields,
                },
            })
        }

        Decl::DeriveDecl { class_name, data_name, span } => Ok(RDecl {
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
        PatKind::Wild => Ok((RPattern { kind: RPatKind::Wild, span: pat.span.clone() }, vec!["_".to_string()])),
        PatKind::Var(name) => Ok((
            RPattern { kind: RPatKind::Var(name.clone()), span: pat.span.clone() },
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
                RPattern { kind: RPatKind::Ctor(name.clone(), rsubs), span: pat.span.clone() },
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

        Type::TPi(x, a, b, span) => {
            let ra = resolve_type(scope, a)?;
            scope.push(x);
            let rb = resolve_type(scope, b)?;
            scope.pop();
            Ok(RType::RPi(x.clone(), Box::new(ra), Box::new(rb), span.clone()))
        }

        Type::TRefine(x, a, phi, span) => {
            let ra = resolve_type(scope, a)?;
            scope.push(x);
            let rphi = resolve_expr_ctx(scope, phi, PropCtx::None)?;
            scope.pop();
            Ok(RType::RRefine(x.clone(), Box::new(ra), Box::new(rphi), span.clone()))
        }

        Type::TApp(f, a, span) => {
            let rf = resolve_type(scope, f)?;
            let ra = resolve_type(scope, a)?;
            Ok(RType::RApp(Box::new(rf), Box::new(ra), span.clone()))
        }
    }
}
