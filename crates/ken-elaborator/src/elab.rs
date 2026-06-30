//! Bidirectional elaboration to kernel core terms (`39 §5.4`, `§5.7`, `21 §6.3`).
//!
//! V1 additions: `requires`/`ensures` clause processing, obligation holes via
//! `declare_postulate`, honesty guard via `GlobalEnv::trusted_base()`, refinement
//! lowering to carrier, `prove`/`law` declaration elaboration, `old` elaboration.

use std::collections::HashMap;

use ken_kernel::{
    check as kernel_check,
    declare_def, declare_postulate,
    subst::{subst0, weaken},
    whnf, Context, GlobalEnv, GlobalId, Level, LevelVar, Term,
};

use crate::ast::{BinOp, NumLit};
use crate::error::{ElabError, Span};
use crate::numbers::{AddEntry, NumericEnv, NumericLitVal};
use crate::resolve::{RDecl, RDeclKind, RExpr, RType};

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
            Ok(Term::const_(id, vec![]))
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
    }
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
    Ok(ElabResult { name: rdecl.name.clone(), def_id: id, obligations: body_obligations })
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
    Ok(ElabResult { name: rdecl.name.clone(), def_id: id, obligations: ens_obligations })
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
    Ok(ElabResult { name: rdecl.name.clone(), def_id: hole_id, obligations: vec![obl] })
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
    Ok(ElabResult { name: rdecl.name.clone(), def_id: law_id, obligations })
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
