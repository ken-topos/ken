//! Bidirectional elaboration to kernel core terms (`39 §5.4`, `§5.7`).
//!
//! The elaborator converts resolved expressions (`resolve.rs`) into kernel
//! `Term`s. Level metas (`Level::Var(LevelVar(m))`) are introduced by bare
//! `Type` annotations and solved during mode-switch; unconstrained metas default
//! to `Zero` before any kernel call.

use std::collections::HashMap;

use ken_kernel::{
    check as kernel_check,
    declare_def,
    subst::{subst0, weaken},
    whnf, Context, GlobalEnv, GlobalId, Level, LevelVar, Term,
};

use crate::error::{ElabError, Span};
use crate::resolve::{RDecl, RExpr, RType};

// ----- level meta context -----

/// Level meta context — a flat list of optional solved levels.
///
/// Metas are introduced for bare `Type` annotations (`§5.7`). They are solved
/// during `unify_types` when one side is an unsolved `Level::Var(m)`. All
/// metas are zonked (substituted and defaulted to `Zero`) before any kernel
/// call.
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
                None => Level::Zero, // unconstrained → default 0 (`§5.7`)
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
            // K2 / other terms — pass through unchanged (not emitted by V0)
            other => other.clone(),
        }
    }
}

// ----- meta-level type unification -----

/// Try to solve level metas so that `l1 ≡ l2`. Non-meta mismatches are
/// silently ignored — the kernel is the backstop for semantic errors.
///
/// IMPORTANT: check for raw (not-yet-zonked) `Level::Var` metas BEFORE
/// calling `zonk_level`. `zonk_level` maps `None` metas to `Level::Zero`,
/// so zonking first would obscure unsolved metas as concrete zeros and
/// prevent them from being solved.
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
        // Both concrete (or already-solved vars) — kernel is backstop.
        _ => {}
    }
}

/// Try to solve level metas so that `t1 ≡ t2`. Structural mismatches (not
/// involving metas) are silently ignored; the kernel is the authority.
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
            Term::Const {
                id: id1,
                level_args: la1,
            },
            Term::Const {
                id: id2,
                level_args: la2,
            },
        ) if id1 == id2 => {
            for (l1, l2) in la1.iter().zip(la2.iter()) {
                unify_levels(metas, l1, l2);
            }
        }
        // Structural mismatch — kernel will judge
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

/// The shared elaboration state.
struct ElabCtx<'e> {
    env: &'e mut GlobalEnv,
    /// Local context Γ — kernel's `Context` (types of local de Bruijn vars).
    ctx: Context,
    metas: MetaCtx,
    /// Global name → `GlobalId` for declarations in Σ.
    globals: &'e HashMap<String, GlobalId>,
}

impl<'e> ElabCtx<'e> {
    fn new(env: &'e mut GlobalEnv, globals: &'e HashMap<String, GlobalId>) -> Self {
        Self {
            env,
            ctx: Context::new(),
            metas: MetaCtx::default(),
            globals,
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
            // Non-dependent arrow: elab both in the SAME context, weaken codomain
            // past the Pi's (unused) binder (`39 §5.4` arrow rule).
            let a_core = elab_type(cx, a)?;
            let b_core = elab_type(cx, b)?;
            Ok(Term::pi(a_core, weaken(&b_core, 1)))
        }

        RType::RPi(_, a, b, _) => {
            // Dependent Pi: extend context with the domain.
            let a_core = elab_type(cx, a)?;
            cx.ctx.push(a_core.clone());
            let b_core = elab_type(cx, b)?;
            cx.ctx.pop();
            Ok(Term::pi(a_core, b_core))
        }
    }
}

// ----- bidirectional elaboration -----

/// Check that `expr` has type `expected` (in context `cx.ctx`).
///
/// Returns the elaborated core term. Type mismatches that cannot be caught
/// structurally here are left for the kernel (the backstop).
fn check(cx: &mut ElabCtx, expr: &RExpr, expected: &Term, _span: &Span) -> Result<Term, ElabError> {
    match expr {
        RExpr::RLam(_, body, lam_span) => {
            // Lambda can only check against a Π type (`39 §5.4`).
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
            // Mode switch: infer then solve level metas (`39 §5.4` (Conv) rule).
            let (core, inferred_ty) = infer(cx, expr)?;
            unify_types(&mut cx.metas, expected, &inferred_ty);
            Ok(core)
        }
    }
}

/// Infer the type of `expr` in context `cx.ctx`.
///
/// Returns `(core_term, type)`.
fn infer(cx: &mut ElabCtx, expr: &RExpr) -> Result<(Term, Term), ElabError> {
    match expr {
        RExpr::RVar(i, _, _) => {
            // Type stored in context without weakening; kernel applies weaken(ty, i+1).
            // We replicate that here (`11 §2`, `check.rs` line ~197).
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
            // The type of a global postulate is stored in the environment.
            let (_, decl_ty) = cx.env.const_type(id).ok_or_else(|| {
                ElabError::Internal(format!("no type for global '{}'", name))
            })?;
            Ok((Term::const_(id, vec![]), decl_ty.clone()))
        }

        RExpr::RUniv(None, _) => {
            let l = cx.metas.fresh();
            // Type : Type (suc Type) — universe-in-universe (`12 §1`)
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

        RExpr::RLam(_, _, span) => {
            // Cannot infer type of a bare lambda — ascription required.
            Err(ElabError::TypeMismatch {
                span: span.clone(),
                reason: "cannot infer type of lambda without annotation".into(),
            })
        }

        RExpr::RLet(_x, ty_opt, rhs, body, span) => {
            let (rhs_core, rhs_ty) = match ty_opt {
                Some(ty) => {
                    let ty_core = elab_type(cx, ty)?;
                    let rhs_c = check(cx, rhs, &ty_core, span)?;
                    (rhs_c, ty_core)
                }
                None => infer(cx, rhs)?,
            };
            // Body is checked in context extended with x : rhs_ty.
            cx.ctx.push(rhs_ty.clone());
            let (body_core, body_ty) = infer(cx, body)?;
            cx.ctx.pop();
            // The let's type is the body's type with x substituted out.
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
    }
}

// ----- declaration elaboration -----

/// Elaborate one resolved declaration into the global environment.
///
/// Returns `Ok(id)` where `id` is the newly registered kernel `GlobalId`.
pub fn elaborate_rdecl(
    env: &mut GlobalEnv,
    globals: &mut HashMap<String, GlobalId>,
    rdecl: &RDecl,
) -> Result<GlobalId, ElabError> {
    // Elaboration borrows env and globals; drop cx before calling declare_def.
    let (ty_core, body_core) = {
        let mut cx = ElabCtx::new(env, globals);

        let (body_raw, ty_raw) = if let Some(ty) = &rdecl.ty {
            let ty_c = elab_type(&mut cx, ty)?;
            let body_c = check(&mut cx, &rdecl.body, &ty_c, &rdecl.span)?;
            (body_c, ty_c)
        } else {
            let (body_c, ty_c) = infer(&mut cx, &rdecl.body)?;
            (body_c, ty_c)
        };

        (cx.metas.zonk_term(&ty_raw), cx.metas.zonk_term(&body_raw))
    };

    // Register in the kernel (which performs the authoritative type check).
    let id = declare_def(env, vec![], ty_core, body_core).map_err(|e| {
        ElabError::KernelRejected {
            error: e,
            span: rdecl.span.clone(),
        }
    })?;

    globals.insert(rdecl.name.clone(), id);
    Ok(id)
}

/// Elaborate a resolved expression (standalone — no global registration).
///
/// Returns `(core_term_zonked, type_zonked)` after kernel validation.
pub fn elaborate_rexpr(
    env: &mut GlobalEnv,
    globals: &HashMap<String, GlobalId>,
    rexpr: &RExpr,
) -> Result<(Term, Term), ElabError> {
    let (core, ty, expr_span) = {
        let mut cx = ElabCtx::new(env, globals);
        let (core_raw, ty_raw) = infer(&mut cx, rexpr)?;
        let c = cx.metas.zonk_term(&core_raw);
        let t = cx.metas.zonk_term(&ty_raw);
        (c, t, rexpr.span().clone())
    };
    // Check with the kernel (authoritative: core : ty in the empty context).
    kernel_check(env, &Context::new(), &core, &ty).map_err(|e| ElabError::KernelRejected {
        error: e,
        span: expr_span,
    })?;
    Ok((core, ty))
}
