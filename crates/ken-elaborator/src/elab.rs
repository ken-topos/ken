//! Bidirectional elaboration to kernel core terms (`39 §5.4`, `§5.7`, `21 §6.3`).
//!
//! V1 additions: `requires`/`ensures` clause processing, obligation holes via
//! `declare_postulate`, honesty guard via `GlobalEnv::trusted_base()`, refinement
//! lowering to carrier, `prove`/`law` declaration elaboration, `old` elaboration.

use std::collections::HashMap;

use ken_kernel::{
    check as kernel_check,
    declare_def, declare_postulate,
    inductive::{peel_app, recursive_args},
    subst::{subst0, subst_outer, weaken},
    whnf, Context, GlobalEnv, GlobalId, Level, LevelVar, Term,
};

use crate::ast::{BinOp, NumLit};
use crate::data;
use crate::error::{ElabError, Span};
use crate::numbers::{AddEntry, NumericEnv, NumericLitVal};
use crate::resolve::{RDecl, RDeclKind, RExpr, RMatchArm, RPatKind, RType};

// ----- obligation model -----

/// Source clause kind for a V1 obligation hole (`22 §1`, §2).
#[derive(Debug, Clone)]
pub enum ObligationKind {
    /// From an `ensures ψ` clause or an implicit return-type refinement (`22 §2.2`/§2.1).
    Ensures,
    /// From a `prove name : φ` declaration (`22 §2.4`).
    Prove,
    /// From a `law Name { field : φ }` field (`22 §2.4`).
    LawField(String),
    /// From a bare fixed-width arithmetic op (`35 §3`, `43 §2`).
    PartialPrim,
    /// A `foreign` boundary contract that is statically unprovable → lowered
    /// to a runtime-checked assertion (`21 §5.2`, `38 §3.3`).
    FfiRuntimeCheck,
}

/// A single open obligation hole (`21 §6.5`).
///
/// The hole is admitted as a postulate in the kernel (`trusted_base()` membership
/// = `unknown` status). Discharging it via `ElabEnv::discharge_hole` retires the
/// postulate and moves it to `proved`.
#[derive(Debug, Clone)]
pub struct Obligation {
    /// Sequential id within this elaboration session.
    pub id: u32,
    /// The postulate `GlobalId` registered for this hole (opaque, in `trusted_base()`).
    pub hole_id: GlobalId,
    /// The goal in closed form (abstracted over the local context at the obligation
    /// site). For a goal `φ` in context `[x:A]`, closed = `Pi(A, φ)`.
    pub goal_closed: Term,
    /// The span of the originating clause.
    pub span: Span,
    /// The source clause kind (for V2 provenance and stable ids).
    pub kind: ObligationKind,
}

/// Result of a V1 declaration elaboration.
#[derive(Debug)]
pub struct ElabResult {
    /// Declaration name — used by V2 for stable obligation ids (`22 §1`).
    pub name: String,
    /// The definition's `GlobalId` (or, for `prove`, the hole's postulate id).
    pub def_id: GlobalId,
    /// Open obligation holes emitted during elaboration.
    pub obligations: Vec<Obligation>,
    /// For `foreign` declarations: the full binding record (AC1/AC5 tests).
    /// `None` for all other declaration kinds.
    pub foreign_binding: Option<crate::foreign::ForeignBinding>,
}

// ----- level meta context -----

#[derive(Default)]
struct MetaCtx {
    metas: Vec<Option<Level>>,
}

impl MetaCtx {
    fn fresh(&mut self) -> Level {
        let id = self.metas.len() as u32;
        self.metas.push(None);
        Level::Var(LevelVar(id))
    }

    fn zonk_level(&self, l: &Level) -> Level {
        match l {
            Level::Zero => Level::Zero,
            Level::Suc(inner) => Level::Suc(Box::new(self.zonk_level(inner))),
            Level::Max(a, b) => {
                Level::Max(Box::new(self.zonk_level(a)), Box::new(self.zonk_level(b)))
            }
            Level::Var(LevelVar(m)) => match &self.metas[*m as usize] {
                Some(sol) => self.zonk_level(sol),
                None => Level::Zero,
            },
        }
    }

    #[allow(dead_code)]
    fn solve(&mut self, m: u32, val: Level) {
        if self.metas[m as usize].is_none() {
            self.metas[m as usize] = Some(val);
        }
    }

    fn zonk_term(&self, t: &Term) -> Term {
        match t {
            Term::Type(l) => Term::ty(self.zonk_level(l)),
            Term::Omega(l) => Term::omega(self.zonk_level(l)),
            Term::Var(i) => Term::var(*i),
            Term::Pi(a, b) => Term::pi(self.zonk_term(a), self.zonk_term(b)),
            Term::Lam(a, body) => Term::lam(self.zonk_term(a), self.zonk_term(body)),
            Term::App(f, a) => Term::app(self.zonk_term(f), self.zonk_term(a)),
            Term::Let { ty, val, body } => Term::Let {
                ty: Box::new(self.zonk_term(ty)),
                val: Box::new(self.zonk_term(val)),
                body: Box::new(self.zonk_term(body)),
            },
            Term::Const { id, level_args } => Term::const_(
                *id,
                level_args.iter().map(|l| self.zonk_level(l)).collect(),
            ),
            other => other.clone(),
        }
    }
}

// ----- level unification -----

/// IMPORTANT: check raw `Level::Var` BEFORE `zonk_level` — zonking maps `None`
/// metas to `Level::Zero`, masking unsolved metas as concrete zeros.
fn unify_levels(metas: &mut MetaCtx, l1: &Level, l2: &Level) {
    match (l1, l2) {
        (Level::Var(LevelVar(m)), _) if metas.metas[*m as usize].is_none() => {
            let val = metas.zonk_level(l2);
            metas.metas[*m as usize] = Some(val);
        }
        (_, Level::Var(LevelVar(m))) if metas.metas[*m as usize].is_none() => {
            let val = metas.zonk_level(l1);
            metas.metas[*m as usize] = Some(val);
        }
        _ => {}
    }
}

fn unify_types(metas: &mut MetaCtx, t1: &Term, t2: &Term) {
    match (t1, t2) {
        (Term::Type(l1), Term::Type(l2)) => unify_levels(metas, l1, l2),
        (Term::Var(a), Term::Var(b)) if a == b => {}
        (Term::Pi(a1, b1), Term::Pi(a2, b2)) => {
            unify_types(metas, a1, a2);
            unify_types(metas, b1, b2);
        }
        (Term::App(f1, a1), Term::App(f2, a2)) => {
            unify_types(metas, f1, f2);
            unify_types(metas, a1, a2);
        }
        (Term::Lam(a1, b1), Term::Lam(a2, b2)) => {
            unify_types(metas, a1, a2);
            unify_types(metas, b1, b2);
        }
        (
            Term::Const { id: id1, level_args: la1 },
            Term::Const { id: id2, level_args: la2 },
        ) if id1 == id2 => {
            for (l1, l2) in la1.iter().zip(la2.iter()) {
                unify_levels(metas, l1, l2);
            }
        }
        _ => {}
    }
}

// ----- level helpers -----

fn level_from_nat(n: u32) -> Level {
    let mut l = Level::Zero;
    for _ in 0..n {
        l = Level::Suc(Box::new(l));
    }
    l
}

// ----- elaboration context -----

struct ElabCtx<'e> {
    env: &'e mut GlobalEnv,
    ctx: Context,
    metas: MetaCtx,
    globals: &'e HashMap<String, GlobalId>,
    num_values: &'e mut HashMap<GlobalId, NumericLitVal>,
    numeric_env: &'e NumericEnv,
    obligations: Vec<Obligation>,
    obl_counter: u32,
}

impl<'e> ElabCtx<'e> {
    fn new(
        env: &'e mut GlobalEnv,
        globals: &'e HashMap<String, GlobalId>,
        num_values: &'e mut HashMap<GlobalId, NumericLitVal>,
        numeric_env: &'e NumericEnv,
    ) -> Self {
        Self {
            env,
            ctx: Context::new(),
            metas: MetaCtx::default(),
            globals,
            num_values,
            numeric_env,
            obligations: Vec::new(),
            obl_counter: 0,
        }
    }
}

// ----- type elaboration -----

fn elab_type(cx: &mut ElabCtx, ty: &RType) -> Result<Term, ElabError> {
    match ty {
        RType::RUniv(None, _) => {
            let l = cx.metas.fresh();
            Ok(Term::ty(l))
        }
        RType::RUniv(Some(n), _) => Ok(Term::ty(level_from_nat(*n))),

        RType::RCon(name, span) => {
            let id = cx
                .globals
                .get(name)
                .copied()
                .ok_or_else(|| ElabError::UnresolvedCon {
                    name: name.clone(),
                    span: span.clone(),
                })?;
            // Inductive type formers must be Term::IndFormer so the kernel's
            // eliminator / conversion rules treat them correctly.
            if cx.env.inductive(id).is_some() {
                Ok(Term::IndFormer { id, level_args: vec![] })
            } else {
                Ok(Term::const_(id, vec![]))
            }
        }

        RType::RApp(f, a, _) => {
            let f_k = elab_type(cx, f)?;
            let a_k = elab_type(cx, a)?;
            Ok(Term::app(f_k, a_k))
        }

        RType::RVarTy(i, _, _) => Ok(Term::var(*i)),

        RType::RArr(a, b, _) => {
            let a_core = elab_type(cx, a)?;
            let b_core = elab_type(cx, b)?;
            Ok(Term::pi(a_core, weaken(&b_core, 1)))
        }

        RType::RPi(_, a, b, _) => {
            let a_core = elab_type(cx, a)?;
            cx.ctx.push(a_core.clone());
            let b_core = elab_type(cx, b)?;
            cx.ctx.pop();
            Ok(Term::pi(a_core, b_core))
        }

        // Refinement lowers to the carrier type (`21 §6.3`): `{x:A|φ}` → `A`.
        // The predicate φ is tracked separately; obligation emitted at introduction.
        RType::RRefine(_, carrier, _phi, _) => {
            elab_type(cx, carrier)
        }
    }
}

// ----- bidirectional elaboration -----

fn check(cx: &mut ElabCtx, expr: &RExpr, expected: &Term, _span: &Span) -> Result<Term, ElabError> {
    match expr {
        RExpr::RNumLit(lit, num_span) => {
            elab_num_lit_checked(cx, lit, expected, num_span)
        }
        RExpr::RLam(_, body, lam_span) => {
            let exp_wh = whnf(cx.env, &cx.ctx, expected);
            match exp_wh {
                Term::Pi(dom, cod) => {
                    cx.ctx.push(*dom.clone());
                    let body_core = check(cx, body, &cod, lam_span)?;
                    cx.ctx.pop();
                    Ok(Term::lam(*dom, body_core))
                }
                _ => Err(ElabError::LambdaVsNonFunction {
                    span: lam_span.clone(),
                }),
            }
        }
        _ => {
            let (core, inferred_ty) = infer(cx, expr)?;
            unify_types(&mut cx.metas, expected, &inferred_ty);
            Ok(core)
        }
    }
}

fn infer(cx: &mut ElabCtx, expr: &RExpr) -> Result<(Term, Term), ElabError> {
    match expr {
        RExpr::RVar(i, _, _) => {
            let ty_stored = cx
                .ctx
                .lookup(*i)
                .ok_or_else(|| ElabError::Internal(format!("Var({}) out of range", i)))?;
            let ty = weaken(ty_stored, (*i as i64) + 1);
            Ok((Term::var(*i), ty))
        }

        RExpr::RCon(name, span) => {
            let id = cx
                .globals
                .get(name)
                .copied()
                .ok_or_else(|| ElabError::UnresolvedCon {
                    name: name.clone(),
                    span: span.clone(),
                })?;
            // Constructor: Term::Constructor with the ctor's declared type.
            let ctor_ty = cx.env.constructor(id).map(|(ind, k)| {
                ind.constructors[k].type_.clone()
            });
            if let Some(ty) = ctor_ty {
                return Ok((Term::Constructor { id, level_args: vec![] }, ty));
            }
            // Inductive type former: Term::IndFormer.
            let ind_ty = cx.env.inductive(id).map(|ind| ind.former_type.clone());
            if let Some(ty) = ind_ty {
                return Ok((Term::IndFormer { id, level_args: vec![] }, ty));
            }
            // Regular constant (postulate/def/primitive).
            let (_, decl_ty) = cx.env.const_type(id).ok_or_else(|| {
                ElabError::Internal(format!("no type for global '{}'", name))
            })?;
            Ok((Term::const_(id, vec![]), decl_ty.clone()))
        }

        RExpr::RUniv(None, _) => {
            let l = cx.metas.fresh();
            let ty = Term::ty(Level::Suc(Box::new(l.clone())));
            Ok((Term::ty(l), ty))
        }
        RExpr::RUniv(Some(n), _) => {
            let l = level_from_nat(*n);
            let ty = Term::ty(Level::Suc(Box::new(l.clone())));
            Ok((Term::ty(l), ty))
        }

        RExpr::RApp(f, a, span) => {
            let (f_core, f_ty) = infer(cx, f)?;
            let f_ty_wh = whnf(cx.env, &cx.ctx, &f_ty);
            match f_ty_wh {
                Term::Pi(dom, cod) => {
                    let a_core = check(cx, a, &dom, span)?;
                    let result_ty = subst0(&cod, &a_core);
                    Ok((Term::app(f_core, a_core), result_ty))
                }
                _ => Err(ElabError::NotAFunction { span: span.clone() }),
            }
        }

        RExpr::RAsc(e, ty, _) => {
            let ty_core = elab_type(cx, ty)?;
            let e_core = check(cx, e, &ty_core, e.span())?;
            Ok((e_core, ty_core))
        }

        RExpr::RLam(_, _, span) => Err(ElabError::TypeMismatch {
            span: span.clone(),
            reason: "cannot infer type of lambda without annotation".into(),
        }),

        RExpr::RLet(_x, ty_opt, rhs, body, span) => {
            let (rhs_core, rhs_ty) = match ty_opt {
                Some(ty) => {
                    let ty_core = elab_type(cx, ty)?;
                    let rhs_c = check(cx, rhs, &ty_core, span)?;
                    (rhs_c, ty_core)
                }
                None => infer(cx, rhs)?,
            };
            cx.ctx.push(rhs_ty.clone());
            let (body_core, body_ty) = infer(cx, body)?;
            cx.ctx.pop();
            let result_ty = subst0(&body_ty, &rhs_core);
            Ok((
                Term::Let {
                    ty: Box::new(rhs_ty),
                    val: Box::new(rhs_core),
                    body: Box::new(body_core),
                },
                result_ty,
            ))
        }

        // `old e` in a space-op ensures (`21 §6.4`).
        // Simplified V1 model: elaborates to the same term as `e` (the pre-state
        // value shares the type of `e`; full state-transformer semantics is V3+).
        RExpr::ROld(e, _) => {
            infer(cx, e)
        }

        RExpr::RNumLit(lit, span) => {
            elab_num_lit_infer(cx, lit, span)
        }

        RExpr::RBinOp(op, lhs, rhs, span) => {
            elab_binop(cx, op, lhs, rhs, span)
        }

        RExpr::RMatch { scrut, arms, span } => {
            infer_match(cx, scrut, arms, span)
        }
    }
}

// ----- numeric literal helpers -----

/// Elaborate a numeric literal with its default type (no expected type).
fn elab_num_lit_infer(
    cx: &mut ElabCtx,
    lit: &NumLit,
    span: &Span,
) -> Result<(Term, Term), ElabError> {
    let (val, type_id) = num_lit_default_type(lit, cx.numeric_env);
    let ty_term = Term::const_(type_id, vec![]);
    let postulate_id = declare_postulate(cx.env, vec![], ty_term.clone())
        .map_err(|e| ElabError::KernelRejected { error: e, span: span.clone() })?;
    cx.num_values.insert(postulate_id, val);
    Ok((Term::const_(postulate_id, vec![]), ty_term))
}

/// Elaborate a numeric literal with a known expected type.
///
/// If the expected type is a numeric type that accepts this literal form, use it.
/// Otherwise infer the default type and unify (may yield a type error).
fn elab_num_lit_checked(
    cx: &mut ElabCtx,
    lit: &NumLit,
    expected: &Term,
    span: &Span,
) -> Result<Term, ElabError> {
    let nenv = cx.numeric_env;
    let exp_wh = whnf(cx.env, &cx.ctx, expected);

    // Try type-directed dispatch: if expected type is a numeric Const, use it.
    if let Term::Const { id, .. } = &exp_wh {
        let ty_id = *id;
        let val_opt: Option<NumericLitVal> = match lit {
            NumLit::Int(n) => {
                // Accept Int literals at any integer numeric type.
                let is_int_type = [
                    nenv.int_id, nenv.int8_id, nenv.int16_id, nenv.int32_id, nenv.int64_id,
                    nenv.uint8_id, nenv.uint16_id, nenv.uint32_id, nenv.uint64_id,
                ].contains(&ty_id);
                if is_int_type {
                    Some(crate::numbers::int_lit_val(*n, &exp_wh, nenv))
                } else {
                    None
                }
            }
            NumLit::Float(f) if ty_id == nenv.float_id => Some(NumericLitVal::Float(*f)),
            NumLit::Decimal(c, e) if ty_id == nenv.decimal_id => {
                Some(NumericLitVal::Decimal { coeff: *c, exp: *e })
            }
            NumLit::Float32(f) if ty_id == nenv.float32_id => Some(NumericLitVal::Float32(*f)),
            _ => None,
        };
        if let Some(val) = val_opt {
            let postulate_id = declare_postulate(cx.env, vec![], exp_wh.clone())
                .map_err(|e| ElabError::KernelRejected { error: e, span: span.clone() })?;
            cx.num_values.insert(postulate_id, val);
            return Ok(Term::const_(postulate_id, vec![]));
        }
    }

    // Fall through: infer default type, then unify with expected.
    let (core, inferred_ty) = elab_num_lit_infer(cx, lit, span)?;
    unify_types(&mut cx.metas, expected, &inferred_ty);
    Ok(core)
}

/// Returns the default (Val, TypeId) for a literal without an expected type.
fn num_lit_default_type(lit: &NumLit, nenv: &NumericEnv) -> (NumericLitVal, GlobalId) {
    match lit {
        NumLit::Int(n)       => (NumericLitVal::Int(*n), nenv.int_id),
        NumLit::Float(f)     => (NumericLitVal::Float(*f), nenv.float_id),
        NumLit::Decimal(c,e) => (NumericLitVal::Decimal { coeff: *c, exp: *e }, nenv.decimal_id),
        NumLit::Float32(f)   => (NumericLitVal::Float32(*f), nenv.float32_id),
    }
}

/// Elaborate a type-directed binary operator.
///
/// Infers the LHS type, dispatches to the right op, and emits an obligation for
/// fixed-width addition (`35 §3`, `43 §2`).
fn elab_binop(
    cx: &mut ElabCtx,
    op: &BinOp,
    lhs: &RExpr,
    rhs: &RExpr,
    span: &Span,
) -> Result<(Term, Term), ElabError> {
    let (lhs_core, lhs_ty) = infer(cx, lhs)?;
    let lhs_ty_wh = whnf(cx.env, &cx.ctx, &lhs_ty);

    match op {
        BinOp::Add | BinOp::WrappingAdd => {
            let entry: &AddEntry = cx.numeric_env.classify_add(&lhs_ty_wh).ok_or_else(|| {
                ElabError::TypeMismatch {
                    span: span.clone(),
                    reason: format!("'+' / '+%' not supported on this type"),
                }
            })?;
            let result_ty = Term::const_(entry.result_id, vec![]);
            let rhs_core = check(cx, rhs, &result_ty, span)?;
            let op_id = if matches!(op, BinOp::WrappingAdd) {
                entry.wrapping_id.ok_or_else(|| ElabError::TypeMismatch {
                    span: span.clone(),
                    reason: format!("'+%' wrapping not available on this type"),
                })?
            } else {
                entry.op_id
            };
            let op_term = Term::const_(op_id, vec![]);
            let applied = Term::app(Term::app(op_term, lhs_core.clone()), rhs_core.clone());

            // Emit no-overflow obligation for bare '+' on fixed-width types.
            if matches!(op, BinOp::Add) {
                if let Some(novf_id) = entry.no_ovf_id {
                    // phi = NoOvf a b : Ω₀
                    let phi = Term::app(
                        Term::app(Term::const_(novf_id, vec![]), lhs_core.clone()),
                        rhs_core.clone(),
                    );
                    let closed = close_goal(&cx.ctx, phi);
                    let hole_id = declare_postulate(cx.env, vec![], closed.clone())
                        .map_err(|e| ElabError::KernelRejected { error: e, span: span.clone() })?;
                    let obl_id = cx.obl_counter;
                    cx.obl_counter += 1;
                    cx.obligations.push(Obligation {
                        id: obl_id,
                        hole_id,
                        goal_closed: closed,
                        span: span.clone(),
                        kind: ObligationKind::PartialPrim,
                    });
                }
            }

            Ok((applied, result_ty))
        }

        BinOp::Mul => {
            return Err(ElabError::Internal("'*' not yet supported".to_string()));
        }

        BinOp::EqEq => {
            let eq_entry = cx.numeric_env.classify_eq(&lhs_ty_wh).ok_or_else(|| {
                ElabError::TypeMismatch {
                    span: span.clone(),
                    reason: format!("'==' not supported on this type"),
                }
            })?;
            let rhs_core = check(cx, rhs, &lhs_ty_wh, span)?;
            let bool_ty = Term::const_(cx.numeric_env.bool_id, vec![]);
            let op_term = Term::const_(eq_entry.op_id, vec![]);
            let applied = Term::app(Term::app(op_term, lhs_core), rhs_core);
            Ok((applied, bool_ty))
        }
    }
}

// ----- goal closing -----

/// Close an open goal over the local context.
///
/// Given `goal` valid in `ctx` (depth = n), builds `Pi(T_{n-1}, ..., Pi(T_0, goal))`
/// — the universally quantified form suitable for `declare_postulate`.
///
/// Limitation (V1): works correctly for independent parameter types (no mutual
/// de Bruijn references between stored types). Sufficient for all V1 conformance
/// cases.
fn close_goal(ctx: &Context, goal: Term) -> Term {
    let n = ctx.types.len();
    let mut result = goal;
    // Wrap from innermost (Var(0)) to outermost (Var(n-1))
    for i in 0..n {
        // types[n-1-i] = stored type of Var(i) (innermost-first indexing)
        let stored_ty = ctx.types[n - 1 - i].clone();
        result = Term::pi(stored_ty, result);
    }
    result
}

// ----- declaration elaboration -----

/// V0-compatible elaboration (no spec clauses).
pub fn elaborate_rdecl(
    env: &mut GlobalEnv,
    globals: &mut HashMap<String, GlobalId>,
    num_values: &mut HashMap<GlobalId, NumericLitVal>,
    numeric_env: &NumericEnv,
    rdecl: &RDecl,
) -> Result<GlobalId, ElabError> {
    let result = elaborate_rdecl_v1(env, globals, num_values, numeric_env, rdecl)?;
    Ok(result.def_id)
}

/// V1 elaboration: returns the definition id plus any emitted obligation holes.
pub fn elaborate_rdecl_v1(
    env: &mut GlobalEnv,
    globals: &mut HashMap<String, GlobalId>,
    num_values: &mut HashMap<GlobalId, NumericLitVal>,
    numeric_env: &NumericEnv,
    rdecl: &RDecl,
) -> Result<ElabResult, ElabError> {
    match &rdecl.kind {
        RDeclKind::View { .. } | RDeclKind::Let => {
            elaborate_view_or_let(env, globals, num_values, numeric_env, rdecl)
        }
        RDeclKind::Prove => elaborate_prove(env, globals, num_values, numeric_env, rdecl),
        RDeclKind::Law { param, fields } => {
            elaborate_law(env, globals, num_values, numeric_env, rdecl, param.clone(), fields.clone())
        }
        RDeclKind::DataDecl { type_params, ctors } => {
            let d_id = data::elab_data_decl(
                env,
                globals,
                &rdecl.name,
                type_params,
                ctors,
                &rdecl.span,
            )?;
            Ok(ElabResult { name: rdecl.name.clone(), def_id: d_id, obligations: vec![], foreign_binding: None })
        }
        RDeclKind::TypeAlias { ty } => {
            // A type alias `type T = A` declares T as a transparent definition
            // of type `Type 0` whose body is A (`34 §2`).
            let (alias_body, alias_id) = {
                let mut cx = ElabCtx::new(env, globals, num_values, numeric_env);
                let body = elab_type(&mut cx, ty)?;
                let body_z = cx.metas.zonk_term(&body);
                (body_z, ())
            };
            let _ = alias_id;
            let alias_ty = Term::ty(Level::Zero);
            let id = declare_def(env, vec![], alias_ty, alias_body)
                .map_err(|e| ElabError::KernelRejected { error: e, span: rdecl.span.clone() })?;
            globals.insert(rdecl.name.clone(), id);
            Ok(ElabResult { name: rdecl.name.clone(), def_id: id, obligations: vec![], foreign_binding: None })
        }
        RDeclKind::Foreign { symbol, library, is_pure, visits } => {
            elaborate_foreign_decl(
                env, globals, num_values, numeric_env, rdecl,
                symbol, library, *is_pure, visits,
            )
        }
    }
}

/// Elaborate a `foreign` declaration (`38 §2`, L7).
fn elaborate_foreign_decl(
    env: &mut GlobalEnv,
    globals: &mut HashMap<String, GlobalId>,
    num_values: &mut HashMap<GlobalId, NumericLitVal>,
    numeric_env: &NumericEnv,
    rdecl: &RDecl,
    symbol: &str,
    library: &str,
    is_pure: bool,
    visits: &[String],
) -> Result<ElabResult, ElabError> {
    use crate::foreign::elaborate_foreign;

    let ty_core = {
        let mut cx = ElabCtx::new(env, globals, num_values, numeric_env);
        let ty = rdecl.ty.as_ref().ok_or_else(|| ElabError::Internal(
            "foreign decl must have a type annotation".into()
        ))?;
        let ty_c = elab_type(&mut cx, ty)?;
        cx.metas.zonk_term(&ty_c)
    };

    let bytes_id = globals.get("Bytes").copied().ok_or_else(|| {
        ElabError::Internal("Bytes not registered before foreign layer".into())
    })?;

    // Foreign ensures → runtime check obligations (AC4).
    let ensures_strs: Vec<String> = rdecl.ensures
        .iter()
        .map(|e| format!("{:?}", e))
        .collect();

    let binding = elaborate_foreign(
        env, globals, bytes_id,
        &rdecl.name, ty_core,
        symbol, library, is_pure, visits,
        &ensures_strs,
        &rdecl.span,
    )?;

    let def_id = binding.postulate_id;

    let obligations: Vec<Obligation> = binding.runtime_checks
        .iter()
        .enumerate()
        .map(|(i, rc)| Obligation {
            id: i as u32,
            hole_id: rc.hole_id,
            goal_closed: Term::omega(Level::Zero),
            span: rdecl.span.clone(),
            kind: ObligationKind::FfiRuntimeCheck,
        })
        .collect();

    Ok(ElabResult {
        name: rdecl.name.clone(),
        def_id,
        obligations,
        foreign_binding: Some(binding),
    })
}

fn elaborate_view_or_let(
    env: &mut GlobalEnv,
    globals: &mut HashMap<String, GlobalId>,
    num_values: &mut HashMap<GlobalId, NumericLitVal>,
    numeric_env: &NumericEnv,
    rdecl: &RDecl,
) -> Result<ElabResult, ElabError> {
    // Check for implicit ensures from a return-type refinement (`22 §2.1`).
    let has_refine_return = rdecl.ty.as_ref()
        .and_then(|ty| innermost_refine_pred(ty))
        .is_some();
    if rdecl.requires.is_empty() && rdecl.ensures.is_empty() && !has_refine_return {
        // V0 path: no spec clauses and no return-type refinement
        return elaborate_v0(env, globals, num_values, numeric_env, rdecl);
    }
    // V1 path: has requires/ensures or implicit return-type refinement obligation
    elaborate_view_with_spec(env, globals, num_values, numeric_env, rdecl)
}

/// Extract the predicate from the innermost refinement in a resolved type.
///
/// `{ k : A | φ }` at the end of a Pi-chain → `Some(φ)`. Used by V2 to
/// emit a refinement-introduction obligation for the return type (`22 §2.1`).
fn innermost_refine_pred(ty: &RType) -> Option<&RExpr> {
    match ty {
        RType::RPi(_, _, cod, _) | RType::RArr(_, cod, _) => innermost_refine_pred(cod),
        RType::RRefine(_, _, phi, _) => Some(phi),
        _ => None,
    }
}

fn elaborate_v0(
    env: &mut GlobalEnv,
    globals: &mut HashMap<String, GlobalId>,
    num_values: &mut HashMap<GlobalId, NumericLitVal>,
    numeric_env: &NumericEnv,
    rdecl: &RDecl,
) -> Result<ElabResult, ElabError> {
    let (ty_core, body_core, body_obligations) = {
        let mut cx = ElabCtx::new(env, globals, num_values, numeric_env);
        let (body_raw, ty_raw) = if let Some(ty) = &rdecl.ty {
            let ty_c = elab_type(&mut cx, ty)?;
            let body_c = check(&mut cx, &rdecl.body, &ty_c, &rdecl.span)?;
            (body_c, ty_c)
        } else {
            let (body_c, ty_c) = infer(&mut cx, &rdecl.body)?;
            (body_c, ty_c)
        };
        let obligations = std::mem::take(&mut cx.obligations);
        (cx.metas.zonk_term(&ty_raw), cx.metas.zonk_term(&body_raw), obligations)
    };
    let id = declare_def(env, vec![], ty_core, body_core).map_err(|e| {
        ElabError::KernelRejected { error: e, span: rdecl.span.clone() }
    })?;
    globals.insert(rdecl.name.clone(), id);
    Ok(ElabResult { name: rdecl.name.clone(), def_id: id, obligations: body_obligations, foreign_binding: None })
}

/// Elaborate a `view` with `requires`/`ensures` clauses (`21 §6.3`).
fn elaborate_view_with_spec(
    env: &mut GlobalEnv,
    globals: &mut HashMap<String, GlobalId>,
    num_values: &mut HashMap<GlobalId, NumericLitVal>,
    numeric_env: &NumericEnv,
    rdecl: &RDecl,
) -> Result<ElabResult, ElabError> {
    let omega = Term::omega(Level::Zero);

    // Phase 1: elaborate the declared type (carrier) and body — drop borrow first.
    let (body_raw, carrier_ty_raw) = {
        let mut cx = ElabCtx::new(env, globals, num_values, numeric_env);
        let result = if let Some(ty) = &rdecl.ty {
            let ty_c = elab_type(&mut cx, ty)?;
            let body_c = check(&mut cx, &rdecl.body, &ty_c, &rdecl.span)?;
            (cx.metas.zonk_term(&body_c), cx.metas.zonk_term(&ty_c))
        } else {
            let (body_c, ty_c) = infer(&mut cx, &rdecl.body)?;
            (cx.metas.zonk_term(&body_c), cx.metas.zonk_term(&ty_c))
        };
        result
    };

    // Build the param context from the Pi-chain of the carrier type.
    let param_types = unwrap_pi_chain(&carrier_ty_raw);
    let carrier_b = innermost_codomain(&carrier_ty_raw);
    let mut param_ctx = Context::new();
    for pt in &param_types {
        param_ctx.push(pt.clone());
    }

    // Phase 2: process `requires` clauses.
    let mut req_cores: Vec<Term> = Vec::new();
    for req in &rdecl.requires {
        let phi_core = elab_in_ctx_at_omega(
            env, globals, num_values, numeric_env, &param_ctx, req, &omega, &rdecl.span,
        )?;
        req_cores.push(phi_core);
    }

    // Phase 3: process `ensures` clauses.
    // ensures context = param_ctx + [result : carrier_b]
    let mut ens_ctx = param_ctx.clone();
    ens_ctx.push(carrier_b.clone());

    // body_inner = the inner body term (past all param lambdas)
    let body_inner = unwrap_lam(&body_raw, param_types.len());

    // Collect ensures: explicit clauses + implicit from return-type refinement (`22 §2.1`).
    // A `{ x : A | φ }` return type is a refinement introduction at the body site;
    // its predicate φ is an implicit ensures with the same ψ[body/result] structure.
    let mut all_ensures: Vec<&RExpr> = rdecl.ensures.iter().collect();
    if let Some(phi) = rdecl.ty.as_ref().and_then(|ty| innermost_refine_pred(ty)) {
        all_ensures.push(phi);
    }

    let mut ens_obligations: Vec<Obligation> = Vec::new();
    let mut obl_counter = 0u32;
    for ens in &all_ensures {
        let psi_core = elab_in_ctx_at_omega(
            env, globals, num_values, numeric_env, &ens_ctx, ens, &omega, &rdecl.span,
        )?;
        // goal = ψ[body_inner/result]: result = Var(0) in ens_ctx, substitute body
        let goal_open = subst0(&psi_core, &body_inner);
        let closed = close_goal(&param_ctx, goal_open);
        let hole_id = declare_postulate(env, vec![], closed.clone())
            .map_err(|e| ElabError::KernelRejected { error: e, span: rdecl.span.clone() })?;
        ens_obligations.push(Obligation {
            id: obl_counter,
            hole_id,
            goal_closed: closed,
            span: rdecl.span.clone(),
            kind: ObligationKind::Ensures,
        });
        obl_counter += 1;
    }

    // Phase 4: build the full type and body.
    // full_ty = Pi(params..., Pi(req..., carrier_b))
    let mut full_ty = carrier_b.clone();
    for req in req_cores.iter().rev() {
        full_ty = Term::pi(req.clone(), weaken(&full_ty, 1));
    }
    for pt in param_types.iter().rev() {
        full_ty = Term::pi(pt.clone(), full_ty);
    }
    // full_body = Lam(params..., Lam(req..., body_inner))
    // body_inner has free variables indexed relative to param_ctx (depth n_params).
    // The req lambdas are inserted BETWEEN the param lambdas and the body, so each
    // param variable in body_inner shifts up by req_cores.len() to skip the req binders.
    let mut full_body = weaken(&body_inner, req_cores.len() as i64);
    for req in req_cores.iter().rev() {
        full_body = Term::lam(req.clone(), full_body);
    }
    for pt in param_types.iter().rev() {
        full_body = Term::lam(pt.clone(), full_body);
    }

    let id = declare_def(env, vec![], full_ty, full_body).map_err(|e| {
        ElabError::KernelRejected { error: e, span: rdecl.span.clone() }
    })?;
    globals.insert(rdecl.name.clone(), id);
    Ok(ElabResult { name: rdecl.name.clone(), def_id: id, obligations: ens_obligations, foreign_binding: None })
}

/// Elaborate `prove name : φ` (`21 §6.3`, §3).
///
/// Declares `name` as a postulate of `φ`, emitting one obligation hole.
fn elaborate_prove(
    env: &mut GlobalEnv,
    globals: &mut HashMap<String, GlobalId>,
    num_values: &mut HashMap<GlobalId, NumericLitVal>,
    numeric_env: &NumericEnv,
    rdecl: &RDecl,
) -> Result<ElabResult, ElabError> {
    let phi_core = {
        let mut cx = ElabCtx::new(env, globals, num_values, numeric_env);
        let omega = Term::omega(Level::Zero);
        let (phi_raw, phi_ty_raw) = infer(&mut cx, &rdecl.body)?;
        // Check φ is Ω-typed
        unify_types(&mut cx.metas, &omega, &phi_ty_raw);
        cx.metas.zonk_term(&phi_raw)
    };
    // Declare as postulate (the hole)
    let hole_id = declare_postulate(env, vec![], phi_core.clone())
        .map_err(|e| ElabError::KernelRejected { error: e, span: rdecl.span.clone() })?;
    globals.insert(rdecl.name.clone(), hole_id);
    let obl = Obligation {
        id: 0,
        hole_id,
        goal_closed: phi_core,
        span: rdecl.span.clone(),
        kind: ObligationKind::Prove,
    };
    Ok(ElabResult { name: rdecl.name.clone(), def_id: hole_id, obligations: vec![obl], foreign_binding: None })
}

/// Elaborate `law Name (param) { f : φ ; … }` (`21 §3`).
///
/// Each field φ is checked at Ω; one obligation hole per field.
fn elaborate_law(
    env: &mut GlobalEnv,
    globals: &mut HashMap<String, GlobalId>,
    num_values: &mut HashMap<GlobalId, NumericLitVal>,
    numeric_env: &NumericEnv,
    rdecl: &RDecl,
    _param: String,
    fields: Vec<(String, RExpr)>,
) -> Result<ElabResult, ElabError> {
    let omega = Term::omega(Level::Zero);
    let mut obligations: Vec<Obligation> = Vec::new();

    // The param is pre-declared by the resolver; for each field φ, check at Ω
    // and emit an obligation hole.
    for (i, (field_name, field_phi)) in fields.iter().enumerate() {
        let phi_core = {
            let mut cx = ElabCtx::new(env, globals, num_values, numeric_env);
            // param is the law's `param` argument — it's in scope (resolver pushed it)
            // For elaboration, we need the param in scope. Since the resolver resolved
            // field_phi with param in scope at Var(0), we replicate that:
            // Note: we DON'T have a declared type for the param here. For V1, the param
            // is just a term variable whose type must be inferrable from the field props.
            // For test cases, params will always be globally declared.
            let (phi_raw, phi_ty_raw) = infer(&mut cx, field_phi)?;
            unify_types(&mut cx.metas, &omega, &phi_ty_raw);
            cx.metas.zonk_term(&phi_raw)
        };
        let hole_id = declare_postulate(env, vec![], phi_core.clone())
            .map_err(|e| ElabError::KernelRejected { error: e, span: rdecl.span.clone() })?;
        let law_field_name = format!("{}_{}", rdecl.name, field_name);
        globals.insert(law_field_name, hole_id);
        obligations.push(Obligation {
            id: i as u32,
            hole_id,
            goal_closed: phi_core,
            span: rdecl.span.clone(),
            kind: ObligationKind::LawField(field_name.clone()),
        });
    }

    // The law itself: declare a postulate of the conjunction type.
    // For V1, law_id is a fresh postulate (placeholder — full Σ-of-Ω is V3+).
    let law_ty = Term::omega(Level::Zero);
    let law_id = declare_postulate(env, vec![], law_ty)
        .map_err(|e| ElabError::KernelRejected { error: e, span: rdecl.span.clone() })?;
    globals.insert(rdecl.name.clone(), law_id);

    // Return: def_id = law_id (the law postulate), obligations = per-field holes
    Ok(ElabResult { name: rdecl.name.clone(), def_id: law_id, obligations, foreign_binding: None })
}

// ----- helpers -----

/// Elaborate `expr` checked at Ω in `ctx`, returning the core term.
///
/// Used for requires/ensures proposition bodies.
fn elab_in_ctx_at_omega(
    env: &mut GlobalEnv,
    globals: &HashMap<String, GlobalId>,
    num_values: &mut HashMap<GlobalId, NumericLitVal>,
    numeric_env: &NumericEnv,
    ctx: &Context,
    expr: &RExpr,
    omega: &Term,
    span: &Span,
) -> Result<Term, ElabError> {
    let mut cx = ElabCtx::new(env, globals, num_values, numeric_env);
    // Populate cx.ctx from the snapshot
    for ty in &ctx.types {
        cx.ctx.push(ty.clone());
    }
    let (core_raw, ty_raw) = infer(&mut cx, expr)?;
    // Unify inferred type with Ω — if the proposition is non-Ω, this will
    // be caught by the kernel on the next kernel_check call.
    // For the surface error, check that ty is Ω-shaped.
    let ty_zonked = cx.metas.zonk_term(&ty_raw);
    let core_zonked = cx.metas.zonk_term(&core_raw);
    // Surface-level Ω check: if the type is not Omega(_), error
    match &ty_zonked {
        Term::Omega(_) => {}
        _ => {
            // Check if the kernel will accept it as Ω — check core at omega
            // If not, surface error
            kernel_check(env, ctx, &core_zonked, omega).map_err(|_| {
                ElabError::TypeMismatch {
                    span: span.clone(),
                    reason: format!(
                        "spec proposition must have type Ω, found non-proposition"
                    ),
                }
            })?;
        }
    }
    Ok(core_zonked)
}

/// Unwrap the outermost `n` Pi binders, collecting domain types.
///
/// `Pi(A, Pi(B, C))` with n=2 → `[A, B]` (A = outermost, B = innermost param).
fn unwrap_pi_chain(ty: &Term) -> Vec<Term> {
    let mut result = Vec::new();
    let mut cur = ty;
    loop {
        match cur {
            Term::Pi(dom, cod) => {
                result.push(*dom.clone());
                cur = cod;
            }
            _ => break,
        }
    }
    result
}

/// Return the innermost codomain of a Pi-chain.
fn innermost_codomain(ty: &Term) -> Term {
    let mut cur = ty;
    loop {
        match cur {
            Term::Pi(_, cod) => cur = cod,
            other => return other.clone(),
        }
    }
}

/// Unwrap the outermost `n` Lam binders, returning the inner body.
fn unwrap_lam(term: &Term, n: usize) -> Term {
    let mut cur = term;
    for _ in 0..n {
        match cur {
            Term::Lam(_, body) => cur = body,
            _ => break,
        }
    }
    cur.clone()
}

// ----- match elaboration -----

/// Elaborate `match scrut { C₁ x₁… => body₁ ; … }` (`34 §3`).
///
/// Compiles to `Term::Elim` with one method per constructor in declaration order.
/// Constant-motive variant: return type inferred from the first arm, checked
/// consistent across all arms by kernel type-checking the Elim.
fn infer_match(
    cx: &mut ElabCtx,
    scrut: &RExpr,
    arms: &[RMatchArm],
    span: &Span,
) -> Result<(Term, Term), ElabError> {
    // 1. Infer scrutinee.
    let (scrut_core, scrut_ty_raw) = infer(cx, scrut)?;
    let scrut_ty = whnf(cx.env, &cx.ctx, &scrut_ty_raw);

    // 2. Peel the type-former application: D p₀ … pₘ₋₁.
    let (head, params_terms) = peel_app(&scrut_ty);
    let d_id = match &head {
        Term::IndFormer { id, .. } => *id,
        _ => {
            return Err(ElabError::TypeMismatch {
                span: span.clone(),
                reason: "match scrutinee must have an inductive type".into(),
            })
        }
    };

    // 3. Clone the InductiveDecl so we can release the &env borrow before
    //    mutating cx.ctx inside the arm loop.
    let ind = cx
        .env
        .inductive(d_id)
        .ok_or_else(|| ElabError::Internal(format!("inductive {:?} not found", d_id)))?
        .clone();
    let m = ind.params.len();
    let n_ctors = ind.constructors.len();

    // 4. Build reverse-lookup tables (pre-snapshot to avoid borrow conflicts).
    //    ctor_id → ctor_index (position in ind.constructors)
    let ctor_id_to_idx: HashMap<GlobalId, usize> = ind
        .constructors
        .iter()
        .enumerate()
        .map(|(k, c)| (c.id, k))
        .collect();
    //    ctor_idx → name (for exhaustiveness error messages)
    let ctor_idx_to_name: Vec<String> = ind
        .constructors
        .iter()
        .map(|c| {
            cx.globals
                .iter()
                .find(|(_, &id)| id == c.id)
                .map(|(name, _)| name.clone())
                .unwrap_or_else(|| format!("<ctor_{:?}>", c.id))
        })
        .collect();
    //    ctor name → ctor_id (snapshot from globals for use inside arm loop)
    let ctor_name_to_id: HashMap<String, GlobalId> = cx
        .globals
        .iter()
        .filter(|(_, &id)| ctor_id_to_idx.contains_key(&id))
        .map(|(name, &id)| (name.clone(), id))
        .collect();

    // 5. Process arms: reachability check + build one method per ctor.
    let mut covered: Vec<bool> = vec![false; n_ctors];
    let mut methods_by_idx: Vec<Option<Term>> = vec![None; n_ctors];
    let mut return_ty: Option<Term> = None;

    for arm in arms {
        let ctor_name = match &arm.pat.kind {
            RPatKind::Ctor(name, _) => name.clone(),
            RPatKind::Wild | RPatKind::Var(_) => {
                return Err(ElabError::Internal(
                    "non-constructor pattern in match (wildcard/var not yet supported \
                     at top level; use constructor patterns)"
                        .into(),
                ))
            }
        };

        let ctor_id = ctor_name_to_id.get(&ctor_name).copied().ok_or_else(|| {
            ElabError::TypeMismatch {
                span: arm.span.clone(),
                reason: format!("'{}' is not a constructor of this type", ctor_name),
            }
        })?;
        let k = ctor_id_to_idx[&ctor_id];

        // AC4: redundant arm detection.
        if covered[k] {
            return Err(ElabError::ReachabilityError { span: arm.span.clone() });
        }
        covered[k] = true;

        // Ctor info.
        let c = &ind.constructors[k];
        let n_args = c.args.len();
        let rec = recursive_args(c, d_id, m);
        let p_ihs = rec.len();

        // Compute pushed_types[j] = type of arg j with params substituted.
        // `subst_outer(c.args[j], m, &params_terms, j)`:
        //   - c.args[j] is in context [Δ_p, arg₀…argⱼ₋₁] with Δ_p at outermost m positions.
        //   - inner_depth = j (j preceding arg binders inside the stored type).
        //   - Result is in context [outer_Γ, arg₀…argⱼ₋₁] — correct for cx.ctx push.
        let mut pushed_types: Vec<Term> = Vec::with_capacity(n_args);
        for j in 0..n_args {
            pushed_types.push(subst_outer(&c.args[j], m, &params_terms, j));
        }

        // Push ctor arg types into cx.ctx for arm body elaboration.
        // After n_args pushes: Var(0)=last_arg_ty, …, Var(n-1)=first_arg_ty.
        // The resolver bound names left-to-right, so first_name → Var(n-1), …, last_name → Var(0).
        for pt in &pushed_types {
            cx.ctx.push(pt.clone());
        }

        let (body_core, body_ty_ctx) = infer(cx, &arm.body)?;

        for _ in 0..n_args {
            cx.ctx.pop();
        }

        // Lower the body type from the arm context to the outer context.
        // For closed return types (Int, Color, …) this is a no-op.
        let body_ty_outer = lower_by(&cx.metas.zonk_term(&body_ty_ctx), n_args)
            .unwrap_or_else(|| cx.metas.zonk_term(&body_ty_ctx));

        if return_ty.is_none() {
            return_ty = Some(body_ty_outer);
        }
        let ret_ty = return_ty.as_ref().unwrap();

        // Build the method for constructor k:
        //   λ(arg₀:T₀). … λ(argₙ₋₁:Tₙ₋₁). λ(IH₀:R). … λ(IHₚ₋₁:R). body_for_method
        //
        // Inside the n+p-lambda body: Var(0)=IHₚ₋₁, …, Var(p-1)=IH₀,
        //                              Var(p)=argₙ₋₁, …, Var(n+p-1)=arg₀.
        // Resolver gave body_core with: Var(0)=argₙ₋₁, …, Var(n-1)=arg₀.
        // Weaken by p to shift arg vars past the IH slots.
        let body_for_method = weaken(&body_core, p_ihs as i64);

        // Wrap p IH lambdas (innermost first, type = return type R).
        let mut method = body_for_method;
        for _ in 0..p_ihs {
            method = Term::lam(ret_ty.clone(), method);
        }
        // Wrap n arg lambdas (outermost arg last — iterate j=n-1 down to 0).
        for j in (0..n_args).rev() {
            method = Term::lam(pushed_types[j].clone(), method);
        }

        methods_by_idx[k] = Some(method);
    }

    // 6. AC3: exhaustiveness — name the first uncovered constructor.
    for (idx, covered_flag) in covered.iter().enumerate() {
        if !covered_flag {
            return Err(ElabError::ExhaustivenessError {
                missing: ctor_idx_to_name[idx].clone(),
                span: span.clone(),
            });
        }
    }

    let methods: Vec<Term> = methods_by_idx
        .into_iter()
        .map(|m| m.unwrap())
        .collect();

    let ret_ty = return_ty.unwrap_or_else(|| Term::ty(Level::Zero));

    // 7. Build the constant motive: Ascript(λ(x: D). R, D → Type ℓ)
    //    The kernel can't infer the type of a bare lambda, so we annotate.
    //    Determine ℓ from the return type's own type.
    let ret_level = {
        match ken_kernel::infer(cx.env, &cx.ctx, &ret_ty) {
            Ok(Term::Type(l)) => l,
            _ => Level::Zero, // fallback: level 0
        }
    };
    let motive_ty = Term::pi(scrut_ty.clone(), Term::ty(ret_level));
    let motive = Term::Ascript(
        Box::new(Term::lam(scrut_ty.clone(), weaken(&ret_ty, 1))),
        Box::new(motive_ty),
    );

    // 8. Build Term::Elim (non-indexed: indices = []).
    let elim = Term::Elim {
        fam: d_id,
        level_args: vec![],
        params: params_terms,
        motive: Box::new(motive),
        methods,
        indices: vec![],
        scrut: Box::new(scrut_core),
    };

    Ok((elim, ret_ty))
}

/// Shift a term's free variables DOWN by `k`, stopping with `None` if any
/// variable at index `i` (outer context) satisfies `0 ≤ i < k` (it references
/// a ctor-arg binder that doesn't exist in the outer scope).
///
/// Used to extract the return type from a match arm body type (which was
/// inferred in a context extended by k ctor-arg binders) back into the outer
/// context.  Closed types (Int, Bool, Color, …) pass through unchanged.
fn lower_by(term: &Term, k: usize) -> Option<Term> {
    if k == 0 {
        return Some(term.clone());
    }
    lower_by_inner(term, k, 0)
}

fn lower_by_inner(term: &Term, k: usize, cutoff: usize) -> Option<Term> {
    match term {
        Term::Var(i) => {
            if *i < cutoff {
                Some(Term::var(*i)) // bound under a local binder — keep as is
            } else if *i < cutoff + k {
                None // refers to a ctor-arg var — can't project to outer scope
            } else {
                Some(Term::var(*i - k)) // outer context var — shift down
            }
        }
        Term::Type(l) => Some(Term::ty(l.clone())),
        Term::Omega(l) => Some(Term::omega(l.clone())),
        Term::Pi(a, b) => Some(Term::pi(
            lower_by_inner(a, k, cutoff)?,
            lower_by_inner(b, k, cutoff + 1)?,
        )),
        Term::Lam(a, body) => Some(Term::lam(
            lower_by_inner(a, k, cutoff)?,
            lower_by_inner(body, k, cutoff + 1)?,
        )),
        Term::App(f, a) => Some(Term::app(
            lower_by_inner(f, k, cutoff)?,
            lower_by_inner(a, k, cutoff)?,
        )),
        Term::Const { id, level_args } => Some(Term::const_(*id, level_args.clone())),
        Term::IndFormer { id, level_args } => {
            Some(Term::IndFormer { id: *id, level_args: level_args.clone() })
        }
        Term::Constructor { id, level_args } => {
            Some(Term::Constructor { id: *id, level_args: level_args.clone() })
        }
        other => Some(other.clone()),
    }
}

// ----- standalone expression elaboration -----

pub fn elaborate_rexpr(
    env: &mut GlobalEnv,
    globals: &HashMap<String, GlobalId>,
    num_values: &mut HashMap<GlobalId, NumericLitVal>,
    numeric_env: &NumericEnv,
    rexpr: &RExpr,
) -> Result<(Term, Term), ElabError> {
    let (core, ty, expr_span) = {
        let mut cx = ElabCtx::new(env, globals, num_values, numeric_env);
        let (core_raw, ty_raw) = infer(&mut cx, rexpr)?;
        let c = cx.metas.zonk_term(&core_raw);
        let t = cx.metas.zonk_term(&ty_raw);
        (c, t, rexpr.span().clone())
    };
    kernel_check(env, &Context::new(), &core, &ty).map_err(|e| ElabError::KernelRejected {
        error: e,
        span: expr_span,
    })?;
    Ok((core, ty))
}
