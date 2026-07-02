//! Bidirectional elaboration to kernel core terms (`39 §5.4`, `§5.7`, `21 §6.3`).
//!
//! V1 additions: `requires`/`ensures` clause processing, obligation holes via
//! `declare_postulate`, honesty guard via `GlobalEnv::trusted_base()`, refinement
//! lowering to carrier, `prove`/`law` declaration elaboration, `old` elaboration.

use std::collections::HashMap;

use ken_kernel::{
    check as kernel_check,
    convert,
    declare_def, declare_postulate, declare_primitive, declare_recursive_group,
    env::PrimReduction,
    infer as kernel_infer,
    inductive::{peel_app, recursive_args},
    sct::sct_check,
    subst::{subst0, subst_outer, subst_var, weaken},
    whnf, Context, Decl, GlobalEnv, GlobalId, InductiveDecl, Level, LevelVar, Term,
};

use crate::ast::{BinOp, NumLit};
use crate::classes::{ClassEnv, ClassInfo, ClassKind, InstanceInfo};
use crate::data;
use crate::error::{ElabError, Span};
use crate::numbers::{AddEntry, NumericEnv, NumericLitVal};
use crate::resolve::{RDecl, RDeclKind, RExpr, RMatchArm, RPatKind, RPattern, RType};

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
    /// Delegated `Temporal` obligations from `temporal{}` blocks (`72 §4`).
    /// These are **not** kernel holes — a delegated property is exported, not
    /// assumed (`21 §5.2`); they never enter `trusted_base()`. Their sole
    /// projection is the B1 `T`/`delegated` channel (TE-E).
    pub temporal_obligations: Vec<crate::temporal::TemporalObligation>,
}

impl ElabResult {
    /// Build [`TEntry`]s from the delegated `Temporal` obligations — the B2
    /// body of the B1 `T` channel (`72 §5`). Each entry carries the elaborated
    /// `Temporal` value with status `delegated` (the constant, pinned at
    /// source).
    pub fn temporal_tentries(&self) -> Vec<crate::export::TEntry> {
        self.temporal_obligations
            .iter()
            .map(|o| crate::export::TEntry {
                obligation_id: o.id.clone(),
                formula: o.formula.clone(),
            })
            .collect()
    }
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
    /// The typeclass registry, when available — needed only for `.field`
    /// Σ-record projection (`RExpr::RProj`, `33 §5.2` η). `None` in every
    /// elaboration path that predates class support and never projects
    /// (prove/law/typealias/foreign/temporal/derive/data, recursive views,
    /// the match compiler); wired via `.with_classes` in the view/let path
    /// so a `where C a`-constrained body can project its resolved
    /// dictionary's fields.
    class_env: Option<&'e ClassEnv>,
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
            class_env: None,
        }
    }

    fn with_classes(mut self, class_env: &'e ClassEnv) -> Self {
        self.class_env = Some(class_env);
        self
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
            // Inductive type formers must be Term::IndFormer, and
            // CONSTRUCTORS must be Term::Constructor, so the kernel's
            // eliminator / conversion rules treat them correctly — a
            // constructor value (e.g. `True`) embedded in a TYPE position
            // (a law-field return-type annotation like
            // `Equal Bool (bool_or True False) True`, ES4-classes) that
            // silently became a bare `Term::Const` would never match
            // `whnf`'s ι-reduction head check (`if let
            // Term::Constructor{..} = head`), permanently stalling
            // reduction on an otherwise-concrete scrutinee.
            if let Some(_) = cx.env.constructor(id) {
                Ok(Term::Constructor { id, level_args: vec![] })
            } else if cx.env.inductive(id).is_some() {
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
        RExpr::RStr(s, span) => {
            elab_str_lit(cx, s, Some(expected), span).map(|(t, _)| t)
        }
        // `Refl` — reflexivity, checked (never inferred): the expected goal
        // must whnf to a kernel `Eq A t u` with `t`/`u` CONVERTIBLE (usually
        // because both sides have already been reduced to the same
        // constructor by an enclosing `match` — the standard way a
        // structure-class law proof discharges, `33 §5.3`/ES4-classes).
        // Surface sugar only: `Refl` is a bare `ConId` the resolver emits as
        // an `RCon` on scope miss (never registered as a real global), so
        // this must be checked BEFORE the generic `RCon` global lookup.
        RExpr::RCon(name, rspan) if name == "Refl" => {
            let exp_wh = whnf(cx.env, &cx.ctx, expected);
            match exp_wh {
                Term::Eq(a_ty, t, u) => {
                    if convert(cx.env, &cx.ctx, &a_ty, &t, &u) {
                        Ok(Term::Refl(t))
                    } else {
                        Err(ElabError::TypeMismatch {
                            span: rspan.clone(),
                            reason: "Refl: the two sides of the goal are not convertible".into(),
                        })
                    }
                }
                _ => Err(ElabError::TypeMismatch {
                    span: rspan.clone(),
                    reason: "Refl expects an `Eq`-shaped goal".into(),
                }),
            }
        }
        // `Axiom` — an EXPLICIT, visible postulate of the expected type
        // (`declare_postulate`, `Decl::Opaque`). The honest surface spelling
        // for an audited-delta law field (`51 §6` erratum's non-zero-delta
        // posture): the resulting `trusted_base()` entry is a real,
        // grep-able `Opaque` — never a silent/implicit assumption. Checked
        // (not inferred), same discipline as `Refl`.
        RExpr::RCon(name, rspan) if name == "Axiom" => {
            let id = declare_postulate(cx.env, vec![], expected.clone())
                .map_err(|e| ElabError::KernelRejected { error: e, span: rspan.clone() })?;
            Ok(Term::const_(id, vec![]))
        }
        // `absurd h` — Bottom-elimination (K5, `16 §1.4`): from `h : Bottom`
        // (a hypothesis that has observationally collapsed to `Bottom`, e.g.
        // `Equal D c₁ c₂` for a different-constructor pair), discharge ANY
        // Ω-classified goal — the ascribed `expected` type becomes the
        // eliminator's explicit motive. Surface sugar only: `absurd` is a
        // bare lowercase identifier the resolver emits as an `RCon` on scope
        // miss. Checked (not inferred) so the motive comes from the goal,
        // mirroring `Refl`/`Axiom`/`tt`.
        RExpr::RApp(f, arg, rspan) if matches!(f.as_ref(), RExpr::RCon(n, _) if n == "absurd") => {
            let bottom = Term::const_(cx.env.bottom_id(), vec![]);
            let proof_core = check(cx, arg, &bottom, rspan)?;
            Ok(Term::Absurd(Box::new(expected.clone()), Box::new(proof_core)))
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
        // `match` against a KNOWN expected type: build the motive from the
        // ascribed goal (`λd. expected[d/scrut]`), not inferred from the
        // first arm's body (ES4-lawproofs AC4). This is what lets a
        // per-branch-varying `Ω`-goal (a structure-class law, `refl :
        // (x:a)->IsTrue (leq x x)`) be proved by case-split at all — the
        // pre-existing `infer_match`/`compile_match_matrix` path (used by
        // `isSorted`/`Perm`, untouched by this) only ever built a CONSTANT
        // motive derived from arm0's inferred type, which cannot express a
        // goal that differs per constructor.
        RExpr::RMatch { scrut, arms, span } => {
            // Gate on PATTERN SHAPE, not goal-dependence: `check_match_
            // dependent` is correct whenever every arm's pattern is FLAT
            // (a constructor with only `Var`/`Wild` sub-patterns) —
            // whether or not `expected` actually mentions the scrutinee.
            // A goal that doesn't mention it (`isSorted`/`Perm`/`sort`'s
            // `Prop`/carrier-typed returns) just yields a genuinely
            // constant motive (still correctly built and checked — no
            // special-casing needed, verified against `isSorted`). A goal
            // that mentions a DIFFERENT bound variable than the immediate
            // scrutinee (a hypothesis-driven case-split, e.g. `trans`'s
            // `match y {...}` where the CONCLUSION mentions `x`/`z` but
            // not `y`) is exactly why goal-dependence was the wrong test
            // — the per-arm substitution still correctly threads `x`/`z`
            // through regardless of whether `y` itself appears. Nested
            // constructor sub-patterns (`Suc (Suc m)`) are NOT supported
            // by the flat-pattern builder, so those keep using the
            // existing general `infer_match`/`compile_match_matrix`
            // nested-pattern compiler unchanged.
            let flat = arms.iter().all(|a| match &a.pat.kind {
                RPatKind::Ctor(_, subs) => subs.iter().all(|s| matches!(s.kind, RPatKind::Var(_) | RPatKind::Wild)),
                _ => false,
            });
            // Further restrict to NULLARY-constructor families (`Bool`,
            // every carrier this WP's law proofs actually case-split on).
            // A parameterized/non-nullary family (`List a`'s `Cons`, what
            // `isSorted`/`Perm` match on) is left entirely to the existing
            // `infer_match` path — untouched, not just by preference but
            // because this narrower builder hasn't been validated against
            // ctor-argument telescopes yet (a real, contained follow-on,
            // not something to risk on a live prelude decl).
            let nullary = flat
                && {
                    let (probe_core, probe_ty) = infer(cx, scrut)?;
                    match probe_core {
                        Term::Var(_) => {
                            let probe_ty_wh = whnf(cx.env, &cx.ctx, &probe_ty);
                            let (head, _) = peel_app(&probe_ty_wh);
                            match head {
                                Term::IndFormer { id, .. } => cx
                                    .env
                                    .inductive(id)
                                    .map(|ind| ind.constructors.iter().all(|c| c.args.is_empty()))
                                    .unwrap_or(false),
                                _ => false,
                            }
                        }
                        _ => false,
                    }
                };
            if nullary {
                check_match_dependent(cx, scrut, arms, expected, span)
            } else {
                let (core, inferred_ty) = infer_match(cx, scrut, arms, span)?;
                unify_types(&mut cx.metas, expected, &inferred_ty);
                Ok(core)
            }
        }
        _ => {
            let (core, inferred_ty) = infer(cx, expr)?;
            unify_types(&mut cx.metas, expected, &inferred_ty);
            Ok(core)
        }
    }
}

/// Check `match scrut { C₁ p… => e₁ ; … }` against a KNOWN `expected` goal
/// that may reference the scrutinee (a per-branch-varying `Ω`- or `Type`-
/// motive) — the K4/AC4 dependent-elimination path. `scrut` must elaborate
/// to a bound variable (`Term::Var`); only FLAT constructor patterns are
/// supported (no nested constructor sub-patterns) — both are sufficient for
/// a structure-class law proof (`ES4-lawproofs`) and deliberately narrower
/// than `infer_match`'s general nested-pattern compiler, which this does
/// not touch or replace.
fn check_match_dependent(
    cx: &mut ElabCtx,
    scrut: &RExpr,
    arms: &[RMatchArm],
    expected: &Term,
    span: &Span,
) -> Result<Term, ElabError> {
    let (scrut_core, scrut_ty_raw) = infer(cx, scrut)?;
    let scrut_ty = whnf(cx.env, &cx.ctx, &scrut_ty_raw);
    let scrut_idx = match &scrut_core {
        Term::Var(k) => *k,
        _ => {
            return Err(ElabError::Internal(
                "dependent match (AC4): scrutinee must be a bound variable so the \
                 goal can be generalized over it"
                    .into(),
            ))
        }
    };

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
    let ind = cx
        .env
        .inductive(d_id)
        .ok_or_else(|| ElabError::Internal(format!("inductive {:?} not found", d_id)))?
        .clone();
    let m = ind.params.len();

    // The motive: `expected` with `Var(scrut_idx)` abstracted to a fresh
    // outer binder — `weaken` shifts every free var (scrut_idx included) up
    // by 1 first, then `subst_var` replaces the shifted scrut_idx with the
    // new Var(0); every OTHER free var is left as-is by `subst_var`'s
    // `i < j` branch, exactly matching its new (one-deeper) home.
    let motive_body = subst_var(&weaken(expected, 1), scrut_idx + 1, &Term::var(0));
    let motive_sort = kernel_infer(cx.env, &cx.ctx, expected)
        .map_err(|e| ElabError::KernelRejected { error: e, span: span.clone() })?;
    let motive_ty = Term::pi(scrut_ty.clone(), weaken(&motive_sort, 1));
    let motive = Term::Ascript(
        Box::new(Term::lam(scrut_ty.clone(), motive_body)),
        Box::new(motive_ty),
    );

    let mut methods: Vec<Option<Term>> = vec![None; ind.constructors.len()];
    let mut arm_used = vec![false; arms.len()];
    for (k, ctor) in ind.constructors.iter().enumerate() {
        let arm_idx = arms
            .iter()
            .position(|a| matches!(&a.pat.kind, RPatKind::Ctor(name, _) if cx.globals.get(name).copied() == Some(ctor.id)))
            .ok_or_else(|| ElabError::ExhaustivenessError {
                missing: cx.globals.iter().find(|(_, &id)| id == ctor.id).map(|(n, _)| n.clone()).unwrap_or_default(),
                span: span.clone(),
            })?;
        arm_used[arm_idx] = true;
        let arm = &arms[arm_idx];
        let sub_pats = match &arm.pat.kind {
            RPatKind::Ctor(_, subs) => subs.clone(),
            _ => unreachable!("guarded by the position() match above"),
        };
        if sub_pats.len() != ctor.args.len() {
            return Err(ElabError::Internal(
                "dependent match (AC4): constructor arity mismatch".into(),
            ));
        }
        let n = sub_pats.len();
        for (j, sp) in sub_pats.iter().enumerate() {
            if !matches!(sp.kind, RPatKind::Var(_) | RPatKind::Wild) {
                return Err(ElabError::Internal(
                    "dependent match (AC4): nested constructor sub-patterns are not \
                     yet supported here"
                        .into(),
                ));
            }
            let raw_ty = subst_outer(&ctor.args[j], m, &params_terms, j);
            cx.ctx.push(raw_ty);
        }
        // Reconstruct the concrete scrutinee `Cₖ p̄ (Var(n-1)) … (Var(0))`
        // in the (now n-deeper) context.
        let mut concrete = Term::Constructor { id: ctor.id, level_args: vec![] };
        for p in &params_terms {
            concrete = Term::app(concrete, weaken(p, n as i64));
        }
        for j in (0..n).rev() {
            concrete = Term::app(concrete, Term::var(j));
        }
        let expected_here = subst_var(&weaken(expected, n as i64), scrut_idx + n, &concrete);
        let body_core = check(cx, &arm.body, &expected_here, &arm.span)?;
        for _ in 0..n {
            cx.ctx.pop();
        }
        let mut method = body_core;
        for j in (0..n).rev() {
            method = Term::lam(subst_outer(&ctor.args[j], m, &params_terms, j), method);
        }
        methods[k] = Some(method);
    }
    for (i, used) in arm_used.iter().enumerate() {
        if !used {
            return Err(ElabError::ReachabilityError { span: arms[i].span.clone() });
        }
    }
    let methods: Vec<Term> = methods.into_iter().map(|m| m.expect("every ctor bucket filled above")).collect();

    Ok(Term::Elim {
        fam: d_id,
        level_args: vec![],
        params: params_terms,
        motive: Box::new(motive),
        methods,
        indices: vec![],
        scrut: Box::new(scrut_core),
    })
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

        RExpr::RStr(s, span) => {
            elab_str_lit(cx, s, None, span)
        }

        RExpr::RBinOp(op, lhs, rhs, span) => {
            elab_binop(cx, op, lhs, rhs, span)
        }

        RExpr::RMatch { scrut, arms, span } => {
            infer_match(cx, scrut, arms, span)
        }

        RExpr::RProj(base, field, span) => infer_proj(cx, base, field, span),
    }
}

/// `e.field` — Σ-record field projection (`33 §5.2` η). Infers `e`'s type,
/// identifies which registered class it's a dictionary of (matching the
/// type's head `Const` against `ClassInfo::type_id`), finds `field`'s
/// declared position, and builds `proj1(proj2^k(e))` — the field's
/// expected type is `field_types[k]` with the class param (this
/// dictionary's concrete head type) and every EARLIER field substituted by
/// its own self-projection off the SAME base (works whether `base` is a
/// concrete instance value or an opaque bound variable like a `where`-
/// supplied dictionary).
fn infer_proj(cx: &mut ElabCtx, base: &RExpr, field: &str, span: &Span) -> Result<(Term, Term), ElabError> {
    let (base_core, base_ty) = infer(cx, base)?;
    // Deliberately inspect `base_ty` AS ELABORATED (never `whnf`'d): a class
    // type is itself `Decl::Transparent` (`elab_class_decl` admits it via
    // `declare_def`, `33 §5.2`), so `whnf` would eagerly unfold `App(Const
    // (class_id), head)` straight through into the raw Σ-chain — losing
    // exactly the "which class is this" information this lookup needs.
    // The surface-elaborated shape (`App(Const(class_id), head)` or bare
    // `Const(class_id)` for an unparameterized class) is always already in
    // this un-unfolded form immediately after `infer`/`env.const_type`.
    let (class_type_id, head_arg) = match &base_ty {
        Term::App(f, a) => match f.as_ref() {
            Term::Const { id, .. } => (*id, Some((**a).clone())),
            _ => return Err(ElabError::TypeMismatch {
                span: span.clone(),
                reason: "projection base's type is not a class dictionary".into(),
            }),
        },
        Term::Const { id, .. } => (*id, None),
        _ => return Err(ElabError::TypeMismatch {
            span: span.clone(),
            reason: "projection base's type is not a class dictionary".into(),
        }),
    };
    let class_env = cx.class_env.ok_or_else(|| ElabError::TypeMismatch {
        span: span.clone(),
        reason: "`.field` projection is unavailable in this elaboration context".into(),
    })?;
    let (field_names, field_types) = class_env
        .classes
        .values()
        .find(|ci| ci.type_id == class_type_id)
        .map(|ci| (ci.field_names.clone(), ci.field_types.clone()))
        .ok_or_else(|| ElabError::TypeMismatch {
            span: span.clone(),
            reason: "projection base's type is not a known class dictionary".into(),
        })?;
    let idx = field_names.iter().position(|n| n == field).ok_or_else(|| ElabError::UnresolvedCon {
        name: field.to_string(),
        span: span.clone(),
    })?;

    // Build proj1(proj2^idx(base_core)) — field `idx`'s value. Each
    // earlier field's self-projection (proj1(proj2^j(base_core)), j<idx)
    // is built off the SAME base, cloned before consuming it below.
    let mut args: Vec<Term> = Vec::new();
    if let Some(h) = head_arg {
        args.push(h);
    }
    args.extend((0..idx).map(|j| {
        let mut v = base_core.clone();
        for _ in 0..j {
            v = Term::proj2(v);
        }
        Term::proj1(v)
    }));

    let mut val = base_core;
    for _ in 0..idx {
        val = Term::proj2(val);
    }
    let val = Term::proj1(val);

    let expected_ty = ken_kernel::subst::subst_tel(&field_types[idx], &args);
    Ok((val, expected_ty))
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
    // ES2: RE-CLASS declare_postulate -> declare_primitive. A literal's value
    // is an audited primitive constant (its content is the number itself),
    // not an assumed axiom — `PrimReduction::Op` is reused as the id-kind
    // marker only; `eval`'s `num_values` side-table lookup (`ken-interp`)
    // intercepts before any symbol dispatch, so the symbol is never actually
    // invoked.
    let postulate_id = declare_primitive(
        cx.env,
        vec![],
        ty_term.clone(),
        PrimReduction::Op { symbol: "literal" },
    )
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

    // Try type-directed dispatch: if expected type is a numeric Const (or,
    // for `Decimal := DecimalPair`, the `IndFormer` `whnf` unfolds the
    // transparent alias to — `18a §5.6.1`), use it.
    let const_or_indformer_id = match &exp_wh {
        Term::Const { id, .. } => Some(*id),
        Term::IndFormer { id, .. } => Some(*id),
        _ => None,
    };
    if let Some(id) = const_or_indformer_id {
        let ty_id = id;
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
            NumLit::Decimal(c, e) if ty_id == nenv.decimal_id || ty_id == nenv.decimalpair_id => {
                Some(NumericLitVal::Decimal { coeff: *c, exp: *e })
            }
            NumLit::Float32(f) if ty_id == nenv.float32_id => Some(NumericLitVal::Float32(*f)),
            _ => None,
        };
        if let Some(val) = val_opt {
            // ES2: RE-CLASS declare_postulate -> declare_primitive (see
            // `elab_num_lit_infer`'s comment — same rationale).
            let postulate_id = declare_primitive(
                cx.env,
                vec![],
                exp_wh.clone(),
                PrimReduction::Op { symbol: "literal" },
            )
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

// ----- string literal helper -----

/// Elaborate a string literal (`37 §2.1`, VAL1-surface).
///
/// `expected` is `Some(ty)` in the check path, `None` in the infer path.
/// Always resolves to `String` type; if an expected type is provided the
/// caller is responsible for unifying (or delegating to `check`).
fn elab_str_lit(
    cx: &mut ElabCtx,
    s: &str,
    expected: Option<&Term>,
    span: &Span,
) -> Result<(Term, Term), ElabError> {
    let str_id = cx
        .globals
        .get("String")
        .copied()
        .ok_or_else(|| ElabError::UnresolvedCon {
            name: "String".to_owned(),
            span: span.clone(),
        })?;
    let str_ty = Term::const_(str_id, vec![]);
    if let Some(exp) = expected {
        unify_types(&mut cx.metas, exp, &str_ty);
    }
    // ES2: RE-CLASS declare_postulate -> declare_primitive (same rationale as
    // `elab_num_lit_infer`).
    let lit_id = declare_primitive(
        cx.env,
        vec![],
        str_ty.clone(),
        PrimReduction::Op { symbol: "literal" },
    )
    .map_err(|e| ElabError::KernelRejected { error: e, span: span.clone() })?;
    cx.num_values.insert(lit_id, NumericLitVal::Str(s.to_owned()));
    Ok((Term::const_(lit_id, vec![]), str_ty))
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
            let bool_ty = Term::indformer(cx.numeric_env.bool_id, vec![]);
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
    let mut sentinel = ClassEnv::sentinel();
    let result = elaborate_rdecl_v1(env, globals, num_values, numeric_env, &mut sentinel, rdecl)?;
    Ok(result.def_id)
}

/// Extract the outermost constructor name from a resolved type for
/// `instance_search` key lookup (`37 §6`, L3b).
fn rtype_head_name(ty: &RType) -> String {
    match ty {
        RType::RCon(name, _) => name.clone(),
        RType::RApp(f, _, _) => rtype_head_name(f),
        RType::RVarTy(_, name, _) => name.clone(),
        _ => String::new(),
    }
}

/// V1 elaboration: returns the definition id plus any emitted obligation holes.
pub fn elaborate_rdecl_v1(
    env: &mut GlobalEnv,
    globals: &mut HashMap<String, GlobalId>,
    num_values: &mut HashMap<GlobalId, NumericLitVal>,
    numeric_env: &NumericEnv,
    class_env: &mut ClassEnv,
    rdecl: &RDecl,
) -> Result<ElabResult, ElabError> {
    match &rdecl.kind {
        RDeclKind::View { constraints, .. } => {
            // Check `where C T` constraints via `instance_search` (`37 §6`,
            // L3b). This is the producer the QA grep gate checks.
            let mut dict_id: Option<GlobalId> = None;
            for (class_name, head_ty) in constraints {
                let head_name = rtype_head_name(head_ty);
                match class_env.instance_search(class_name, &head_name) {
                    Some(id) => dict_id = Some(id),
                    None => {
                        return Err(ElabError::NoInstance {
                            class: class_name.clone(),
                            ty: head_name,
                            span: rdecl.span.clone(),
                        });
                    }
                }
            }
            // `where C a` supplies the resolved dictionary under the fixed
            // surface name `d` (`51 §4` — the illustrative name the spec
            // itself uses), so the body can project its fields (`d.leq`,
            // `RExpr::RProj`) exactly as if it had been passed explicitly —
            // ordinary implicit-dictionary insertion, no second mechanism
            // (AC2, reflect-don't-extend). Scoped to THIS decl only:
            // save/restore any prior `d` binding around the call so it
            // never leaks to sibling decls.
            let saved_d = dict_id.map(|id| globals.insert("d".to_string(), id));
            let result = elaborate_view_or_let(env, globals, num_values, numeric_env, class_env, rdecl);
            if dict_id.is_some() {
                match saved_d.unwrap() {
                    Some(prev) => { globals.insert("d".to_string(), prev); }
                    None => { globals.remove("d"); }
                }
            }
            result
        }
        RDeclKind::Let => {
            elaborate_view_or_let(env, globals, num_values, numeric_env, class_env, rdecl)
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
            // Register data type in the module map for orphan check (`33 §5.3`).
            class_env.global_modules.insert(d_id, class_env.current_module);
            Ok(ElabResult { name: rdecl.name.clone(), def_id: d_id, obligations: vec![], foreign_binding: None, temporal_obligations: vec![] })
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
            Ok(ElabResult { name: rdecl.name.clone(), def_id: id, obligations: vec![], foreign_binding: None, temporal_obligations: vec![] })
        }
        RDeclKind::Foreign { symbol, library, is_pure, visits } => {
            elaborate_foreign_decl(
                env, globals, num_values, numeric_env, rdecl,
                symbol, library, *is_pure, visits,
            )
        }
        RDeclKind::Temporal { formula, source } => {
            elaborate_temporal(env, globals, rdecl, formula, source)
        }
        RDeclKind::ClassDecl { param, fields } => {
            elab_class_decl(env, globals, num_values, numeric_env, class_env, rdecl, param, fields)
        }
        RDeclKind::InstanceDecl { head_type, constraints, fields } => {
            elab_instance_decl(
                env, globals, num_values, numeric_env, class_env, rdecl,
                &rdecl.name.clone(), head_type, constraints, fields,
            )
        }
        RDeclKind::DeriveDecl { data_name } => {
            elab_derive(env, globals, num_values, numeric_env, class_env, rdecl, &rdecl.name.clone(), data_name)
        }
    }
}

/// Initialize the typeclass environment, pre-declaring `RecordNil` and
/// `record_nil_val` as structural postulates (`33 §5`).
pub fn init_class_env(
    env: &mut GlobalEnv,
    globals: &mut HashMap<String, GlobalId>,
) -> Result<ClassEnv, ElabError> {
    // RecordNil : Omega 0 — the Σ-chain prop terminator.
    let record_nil_id = declare_postulate(env, vec![], Term::omega(Level::Zero))
        .map_err(|e| ElabError::Internal(format!("RecordNil postulate: {}", e)))?;
    globals.insert("RecordNil".to_string(), record_nil_id);
    // record_nil_val : RecordNil — the unique inhabitant.
    let record_nil_val_id =
        declare_postulate(env, vec![], Term::const_(record_nil_id, vec![]))
            .map_err(|e| ElabError::Internal(format!("record_nil_val postulate: {}", e)))?;
    globals.insert("record_nil_val".to_string(), record_nil_val_id);
    Ok(ClassEnv {
        classes: std::collections::HashMap::new(),
        instances: std::collections::HashMap::new(),
        record_nil_id,
        record_nil_val_id,
        current_module: 0,
        global_modules: std::collections::HashMap::new(),
    })
}

// ---- typeclass elaboration (`33 §5`, `39 §6`) --------------------------------

/// Sigma chain type for field types `[T1, T2, …, Tn]`.
///
/// Chain: `Sigma(T1, Sigma(T2, …Sigma(Tn, RecordNil)…))`. Each `Ti` MUST
/// already be elaborated in the correct nested context — `T0` in `[a?]`,
/// `T1` in `[a?, T0]`, …, `Ti` in `[a?, T0, …, T_{i-1}]` (a real Σ-telescope,
/// `33 §5.2`: a later field's type may reference an earlier field's VALUE
/// as `Var(0)`, e.g. `refl : (x:a) -> IsTrue (eq x x)`). No `weaken` is
/// needed here — placing `Ti` as the head of `Sigma(Ti, rest)` is *exactly*
/// what "one more binder than `rest`'s context" requires, and that's
/// precisely the context `Ti` was elaborated in.
fn build_sigma_chain(field_types: &[Term], record_nil_id: GlobalId) -> Term {
    let mut acc = Term::const_(record_nil_id, vec![]);
    for t in field_types.iter().rev() {
        acc = Term::sigma(t.clone(), acc);
    }
    acc
}

/// Pair chain value for field values `[v1, v2, …, vn]`.
/// Chain: `Pair(v1, Pair(v2, …Pair(vn, record_nil_val)…))`.
fn build_pair_chain(field_vals: &[Term], record_nil_val_id: GlobalId) -> Term {
    let mut acc = Term::const_(record_nil_val_id, vec![]);
    for v in field_vals.iter().rev() {
        acc = Term::pair(v.clone(), acc);
    }
    acc
}

/// Extract the outermost type constructor name from a resolved type.
fn head_type_name(ty: &RType) -> String {
    match ty {
        RType::RCon(s, _) | RType::RVarTy(_, s, _) => s.clone(),
        RType::RApp(f, _, _) => head_type_name(f),
        RType::RUniv(_, _) => "Type".to_string(),
        RType::RArr(_, _, _) | RType::RPi(_, _, _, _) => "->".to_string(),
        RType::RRefine(_, inner, _, _) => head_type_name(inner),
    }
}

/// Elaborate `class C A { f1 : T1 ; … }` → Σ-record type (`33 §5`).
///
/// The Σ-chain sort (via `sort_sigma`, `check.rs:192`) determines whether the
/// class is a property class (Ω, coherence-free) or structure class (Type,
/// canonical-instance policy). The class type is admitted via `declare_def`
/// (kernel re-check at `check.rs:944`).
fn elab_class_decl(
    env: &mut GlobalEnv,
    globals: &mut HashMap<String, GlobalId>,
    num_values: &mut HashMap<GlobalId, NumericLitVal>,
    numeric_env: &NumericEnv,
    class_env: &mut ClassEnv,
    rdecl: &RDecl,
    param: &Option<String>,
    fields: &[(String, RType)],
) -> Result<ElabResult, ElabError> {
    let span = &rdecl.span;
    let has_param = param.is_some();

    // Elaborate each field type incrementally: a real Σ-telescope (`33
    // §5.2`) where a later field's type may reference an EARLIER field's
    // value (a law like `refl : (x:a) -> IsTrue (eq x x)` refers to the
    // `eq` op field). Push each field's OWN elaborated type onto `cx.ctx`
    // before elaborating the next, so `resolve.rs`'s bound `RVarTy`
    // reference for that field name lines up with the real kernel depth.
    let field_types: Vec<Term> = {
        let mut cx = ElabCtx::new(env, globals, num_values, numeric_env);
        if has_param {
            cx.ctx.push(Term::ty(Level::Zero));
        }
        let mut tys = Vec::new();
        for (_, fty) in fields {
            let t = elab_type(&mut cx, fty)?;
            let t = cx.metas.zonk_term(&t);
            cx.ctx.push(t.clone());
            tys.push(t);
        }
        tys
    };

    // Build Σ-chain (under the param binder if present).
    let sigma_chain = build_sigma_chain(&field_types, class_env.record_nil_id);

    // Determine the sort of the Σ-chain by calling kernel infer on it.
    // Sigma inference is supported (`check.rs:276`). We need a context for A.
    let chain_sort = {
        let mut ctx_a = Context::new();
        if has_param {
            ctx_a.push(Term::ty(Level::Zero));
        }
        kernel_infer(env, &ctx_a, &sigma_chain)
            .map_err(|e| ElabError::KernelRejected { error: e, span: span.clone() })?
    };

    // Classify: Ω = property class, Type = structure class.
    let kind = match &chain_sort {
        Term::Omega(_) => ClassKind::Property,
        _ => ClassKind::Structure,
    };

    // Build class type and body.
    let (class_ty, class_body) = if has_param {
        let pi_ty = Term::pi(Term::ty(Level::Zero), weaken(&chain_sort, 1));
        let lam_body = Term::lam(Term::ty(Level::Zero), sigma_chain);
        (pi_ty, lam_body)
    } else {
        (chain_sort, sigma_chain)
    };

    let id = declare_def(env, vec![], class_ty, class_body)
        .map_err(|e| ElabError::KernelRejected { error: e, span: span.clone() })?;
    globals.insert(rdecl.name.clone(), id);
    class_env.global_modules.insert(id, class_env.current_module);
    class_env.classes.insert(
        rdecl.name.clone(),
        ClassInfo {
            param: param.clone(),
            field_names: fields.iter().map(|(n, _)| n.clone()).collect(),
            field_types: field_types.clone(),
            type_id: id,
            kind,
            module_id: class_env.current_module,
        },
    );

    Ok(ElabResult {
        name: rdecl.name.clone(),
        def_id: id,
        obligations: vec![],
        foreign_binding: None,
        temporal_obligations: vec![],
    })
}

/// Compute an instance's field VALUES, in class-declaration order, each
/// **checked** (not blindly inferred) against its properly-substituted
/// expected type (`33 §5.3` Σ-Intro re-check) — the load-bearing mechanism
/// for AC3 (`ES4-classes`): a law field's declared type (e.g.
/// `refl : (x:a) -> IsTrue (eq x x)`) is a Σ-telescope term referencing the
/// class param and every EARLIER field by position (`ClassInfo::field_types`,
/// `elab_class_decl`). For THIS instance, substitute the concrete head type
/// for the param and every ALREADY-COMPUTED field value for its slot
/// (`ken_kernel::subst::subst_tel`, outermost-first) to get field `i`'s
/// concrete expected type, then `check` the provided expression against it.
/// A postulated/holed/wrong-shaped proof fails right here (kernel re-check),
/// never silently accepted — the whole "laws PROVED, not postulated" gate.
fn compute_ordered_field_values(
    cx: &mut ElabCtx,
    class_env: &ClassEnv,
    class_name: &str,
    head_core: &Term,
    fields: &[(String, RExpr)],
    span: &Span,
) -> Result<Vec<Term>, ElabError> {
    let (field_names, field_types, has_param) = {
        let ci = class_env.classes.get(class_name).ok_or_else(|| {
            ElabError::UnresolvedCon { name: class_name.to_string(), span: span.clone() }
        })?;
        (ci.field_names.clone(), ci.field_types.clone(), ci.param.is_some())
    };
    let mut values: Vec<Term> = Vec::new();
    for (i, fname) in field_names.iter().enumerate() {
        let pos = fields.iter().position(|(n, _)| n == fname).ok_or_else(|| {
            ElabError::Internal(format!("instance missing field '{}'", fname))
        })?;
        let mut args: Vec<Term> = Vec::new();
        if has_param {
            args.push(head_core.clone());
        }
        args.extend(values.iter().cloned());
        let expected = ken_kernel::subst::subst_tel(&field_types[i], &args);
        let v = check(cx, &fields[pos].1, &expected, span)?;
        values.push(cx.metas.zonk_term(&v));
    }
    Ok(values)
}

/// Elaborate `instance C HeadType [where C1 T1 ; …] { f1 = e1 ; … }`.
///
/// Enforces the orphan check (`33 §5.3`) and overlap check (`39 §6.1`),
/// builds the Σ-chain value, and admits it through `declare_def` (kernel
/// re-check).  For constraint-carrying instances, uses
/// `declare_recursive_group` so that `sct_check` can reject non-terminating
/// resolution chains at admission time (`39 §6.4`).
fn elab_instance_decl(
    env: &mut GlobalEnv,
    globals: &mut HashMap<String, GlobalId>,
    num_values: &mut HashMap<GlobalId, NumericLitVal>,
    numeric_env: &NumericEnv,
    class_env: &mut ClassEnv,
    rdecl: &RDecl,
    class_name: &str,
    head_type: &RType,
    constraints: &[(String, RType)],
    fields: &[(String, RExpr)],
) -> Result<ElabResult, ElabError> {
    let span = &rdecl.span;

    // ---- look up class ---------------------------------------------------
    let (class_module, class_type_id, class_kind) = {
        let ci = class_env.classes.get(class_name).ok_or_else(|| {
            ElabError::UnresolvedCon { name: class_name.to_string(), span: span.clone() }
        })?;
        (ci.module_id, ci.type_id, ci.kind.clone())
    };

    let head_name = head_type_name(head_type);
    let instance_key = (class_name.to_string(), head_name.clone());

    // ---- orphan check (`33 §5.3`) ----------------------------------------
    let in_class_module = class_module == class_env.current_module;
    let in_head_module = globals
        .get(&head_name)
        .and_then(|id| class_env.global_modules.get(id))
        .map(|m| *m == class_env.current_module)
        .unwrap_or(false);
    if !in_class_module && !in_head_module {
        return Err(ElabError::OrphanInstance {
            class: class_name.to_string(),
            head_type: head_name.clone(),
            span: span.clone(),
        });
    }

    // ---- overlap check (`39 §6.1`) — skip for property classes (Ω-PI) ---
    if class_kind == ClassKind::Structure && class_env.instances.contains_key(&instance_key) {
        return Err(ElabError::OverlappingInstances {
            class: class_name.to_string(),
            head_type: head_name.clone(),
            span: span.clone(),
        });
    }

    // ---- elaborate head type --------------------------------------------
    let head_core = {
        let mut cx = ElabCtx::new(env, globals, num_values, numeric_env);
        let h = elab_type(&mut cx, head_type)?;
        cx.metas.zonk_term(&h)
    };

    // ---- build instance type --------------------------------------------
    // App(class_type, head) if parameterized, else class_type directly.
    let instance_ty = if class_env.classes.get(class_name).map(|ci| ci.param.is_some()).unwrap_or(false) {
        Term::app(Term::const_(class_type_id, vec![]), head_core.clone())
    } else {
        Term::const_(class_type_id, vec![])
    };

    // ---- direct-self-reference detection (`39 §6.4`, scope-limited) -------
    //
    // This check detects DIRECT self-reference: a constraint whose (class, head)
    // is identical to the instance being declared. It does NOT detect mutual or
    // indirect cycles (e.g. `instance C (F a) where C (G a)` +
    // `instance C (G a) where C (F a)` — each admits as zero-edge, but resolution
    // loops at runtime).
    //
    // [tracked follow-on: Lc-mutual-cycle-termination]
    // Faithful reification (§6.4: one group node per sub-goal, one edge per
    // dischargeSubConstraints call, head-type metric for descent) would require
    // gathering ALL transitively-constrained instances into one
    // declare_recursive_group and threading the head-type metric through the edges.
    // This is deferred — the current slice covers direct-self-ref rejection only.
    // There is NO search-side backstop (no resolution-depth bound or occurs-check);
    // faithful reification is the sole net for mutual-cycle termination.
    let has_self_ref = constraints.iter().any(|(cn, ct)| {
        let chead = head_type_name(ct);
        (cn.as_str(), chead.as_str()) == (class_name, head_name.as_str())
    });

    // ---- admit the instance ----------------------------------------------
    let instance_id = if has_self_ref {
        // Direct self-referential constraint: encode as a fixpoint-arrow so
        // sct_check sees the self-loop in App position and rejects (`39 §6.4`).
        //
        // Type  = Pi(T, T)   where T = instance_ty.
        // Body  = Lam(T, App(Const(own_id), Var(0)))
        //
        // collect_calls sees App(Const(own_id), Var(0)) → edge with M=[[?]]
        // (Var(0) = the parameter, not strictly decreasing) → SCT rejects.
        let t = instance_ty.clone();
        let fixpoint_ty = Term::pi(t.clone(), t.clone());
        let ids = declare_recursive_group(
            env,
            vec![(vec![], fixpoint_ty)],
            |ids| {
                let own_id = ids[0];
                let body = Term::lam(
                    t.clone(),
                    Term::app(Term::const_(own_id, vec![]), Term::var(0)),
                );
                vec![body]
            },
        )
        .map_err(|_| ElabError::NonTerminatingInstances { span: span.clone() })?;
        ids[0]
    } else if !constraints.is_empty() {
        // Non-self-ref constrained instance: elaborate fields, then route through
        // declare_recursive_group so sct_check runs on the group (`39 §6.4`).
        // Body has no App(Const(own_id), ...) → edges.is_empty() → sct_check
        // accepts. Mutual/indirect cycles are not detected here (see above).
        let ordered_vals: Vec<Term> = {
            let mut cx = ElabCtx::new(env, globals, num_values, numeric_env).with_classes(&*class_env);
            compute_ordered_field_values(&mut cx, class_env, class_name, &head_core, fields, span)?
        };
        let pair_chain = build_pair_chain(&ordered_vals, class_env.record_nil_val_id);
        let inst_ty = instance_ty.clone();
        let ids = declare_recursive_group(
            env,
            vec![(vec![], inst_ty)],
            |_ids| vec![pair_chain],
        )
        .map_err(|e| ElabError::KernelRejected { error: e, span: span.clone() })?;
        ids[0]
    } else {
        // No constraints: declare_def path (no recursion possible, SCT not needed).
        let ordered_vals: Vec<Term> = {
            let mut cx = ElabCtx::new(env, globals, num_values, numeric_env).with_classes(&*class_env);
            compute_ordered_field_values(&mut cx, class_env, class_name, &head_core, fields, span)?
        };
        let pair_chain = build_pair_chain(&ordered_vals, class_env.record_nil_val_id);
        declare_def(env, vec![], instance_ty, pair_chain)
            .map_err(|e| ElabError::KernelRejected { error: e, span: span.clone() })?
    };

    // ---- register instance ----------------------------------------------
    let inst_name = format!("{}_instance_{}", class_name, head_name);
    globals.insert(inst_name, instance_id);
    class_env.global_modules.insert(instance_id, class_env.current_module);
    // For property classes, allow multiple registrations (Ω-PI means they're
    // all definitionally equal; the key is occupied but we don't error).
    class_env.instances.insert(
        instance_key,
        InstanceInfo { instance_id, module_id: class_env.current_module },
    );

    Ok(ElabResult {
        name: rdecl.name.clone(),
        def_id: instance_id,
        obligations: vec![],
        foreign_binding: None,
        temporal_obligations: vec![],
    })
}

/// Elaborate `derive ClassName for DataName` (`33 §5.6`, `39 §6.6`).
///
/// Generates a candidate instance through the real `declare_def` re-check
/// (untrusted generation — the kernel re-verifies). For the current build:
/// the candidate for nullary/prop-only classes is `record_nil_val` directly;
/// the kernel rejects malformed candidates.
fn elab_derive(
    env: &mut GlobalEnv,
    globals: &mut HashMap<String, GlobalId>,
    _num_values: &mut HashMap<GlobalId, NumericLitVal>,
    _numeric_env: &NumericEnv,
    class_env: &mut ClassEnv,
    rdecl: &RDecl,
    class_name: &str,
    data_name: &str,
) -> Result<ElabResult, ElabError> {
    let span = &rdecl.span;

    let (class_type_id, has_param) = {
        let ci = class_env.classes.get(class_name).ok_or_else(|| {
            ElabError::UnresolvedCon { name: class_name.to_string(), span: span.clone() }
        })?;
        (ci.type_id, ci.param.is_some())
    };

    let data_id = globals.get(data_name).copied().ok_or_else(|| {
        ElabError::UnresolvedCon { name: data_name.to_string(), span: span.clone() }
    })?;

    let data_term = if env.inductive(data_id).is_some() {
        Term::indformer(data_id, vec![])
    } else {
        Term::const_(data_id, vec![])
    };

    let instance_ty = if has_param {
        Term::app(Term::const_(class_type_id, vec![]), data_term)
    } else {
        Term::const_(class_type_id, vec![])
    };

    // Generate candidate: record_nil_val (minimal inhabitant of a prop-only
    // class Σ-chain). The kernel's declare_def re-checks: a malformed candidate
    // (wrong type) is rejected here.
    let candidate = Term::const_(class_env.record_nil_val_id, vec![]);
    let instance_id = declare_def(env, vec![], instance_ty, candidate)
        .map_err(|e| ElabError::KernelRejected { error: e, span: span.clone() })?;

    let head_name = data_name.to_string();
    let inst_name = format!("{}_instance_{}", class_name, head_name);
    globals.insert(inst_name, instance_id);
    class_env.global_modules.insert(instance_id, class_env.current_module);
    class_env.instances.insert(
        (class_name.to_string(), head_name),
        InstanceInfo { instance_id, module_id: class_env.current_module },
    );

    Ok(ElabResult {
        name: rdecl.name.clone(),
        def_id: instance_id,
        obligations: vec![],
        foreign_binding: None,
        temporal_obligations: vec![],
    })
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
        temporal_obligations: vec![],
    })
}

fn elaborate_view_or_let(
    env: &mut GlobalEnv,
    globals: &mut HashMap<String, GlobalId>,
    num_values: &mut HashMap<GlobalId, NumericLitVal>,
    numeric_env: &NumericEnv,
    class_env: &ClassEnv,
    rdecl: &RDecl,
) -> Result<ElabResult, ElabError> {
    // Check for implicit ensures from a return-type refinement (`22 §2.1`).
    let has_refine_return = rdecl.ty.as_ref()
        .and_then(|ty| innermost_refine_pred(ty))
        .is_some();
    if rdecl.requires.is_empty() && rdecl.ensures.is_empty() && !has_refine_return {
        // V0 path: no spec clauses and no return-type refinement
        return elaborate_v0(env, globals, num_values, numeric_env, class_env, rdecl);
    }
    // V1 path: has requires/ensures or implicit return-type refinement obligation
    elaborate_view_with_spec(env, globals, num_values, numeric_env, class_env, rdecl)
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
    class_env: &ClassEnv,
    rdecl: &RDecl,
) -> Result<ElabResult, ElabError> {
    // A self-recursive view/let (body mentions its own name) must be admitted
    // through the SCT gate with the name pre-bound, so the body's self-call
    // resolves — `declare_def` allocates the id only after the body is built,
    // which is too late for a self-reference. Route to the recursive path.
    if rexpr_mentions_name(&rdecl.body, &rdecl.name) {
        return elaborate_recursive_view(env, globals, num_values, numeric_env, class_env, rdecl);
    }
    let (ty_core, body_core, body_obligations) = {
        let mut cx = ElabCtx::new(env, globals, num_values, numeric_env).with_classes(class_env);
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
    Ok(ElabResult { name: rdecl.name.clone(), def_id: id, obligations: body_obligations, foreign_binding: None, temporal_obligations: vec![] })
}

/// Elaborate a self-recursive `view`/`let` through the SCT gate (Approach A).
///
/// The kernel's `declare_def` already pre-admits an opaque, kernel-checks the
/// body, runs `sct_check`, and upgrades to transparent — but it allocates the
/// id *after* the body is built. A recursive def's body references its own id
/// during elaboration (the resolver emits `RCon(name)` on a scope miss,
/// `c3a3f1d`; the elaborator resolves it against `globals`), so the id must be
/// visible *before* the body is elaborated. This function splits the sequence
/// the kernel performs atomically in `declare_def`:
///
///   1. Elaborate the declared type → `ty_core`.
///   2. Pre-admit the name as `Opaque` with that type and insert it into
///      `globals`, so the body's self-reference resolves to this id.
///   3. Elaborate the body checked against `ty_core` (self-calls see the
///      opaque's type; the kernel `check` sees the opaque too).
///   4. Kernel-check the closed body against `ty_core`, then `sct_check` the
///      singleton recursive group.
///   5. On SCT acceptance, `upgrade_to_transparent` (δ-unfoldable, leaves
///      `trusted_base`); on rejection, roll back the pre-admission — the opaque
///      plus any literal postulates body elaboration added after it — and
///      unbind the name from `globals`.
///
/// **Contained vs deferred (K2c).** This is a contained elaborator-side wiring
/// of an *existing* kernel capability (`sct_check` + `upgrade_to_transparent`);
/// the soundness-critical part — verifying structural descent — already lives
/// in the kernel. The deferred sibling is **K2c general recursive δ** (`11
/// §4`): arbitrary recursive δ-unfolding in conversion. Here the recursive call
/// is to an *opaque* (δ blocks during checking); only after SCT acceptance does
/// it become transparent, and termination is by structural descent on an
/// inductive sub-term (SCT's `↓`) — not general δ. A recursive view carrying
/// `requires` clauses (so the full type ≠ the carrier Pi-chain) is a tracked
/// follow-on; L3a's recursive views (`map`/`filter`/`fold`/`zip`/`unfoldUpTo`/
/// `sort`/`insert`) carry none.
fn elaborate_recursive_view(
    env: &mut GlobalEnv,
    globals: &mut HashMap<String, GlobalId>,
    num_values: &mut HashMap<GlobalId, NumericLitVal>,
    numeric_env: &NumericEnv,
    class_env: &ClassEnv,
    rdecl: &RDecl,
) -> Result<ElabResult, ElabError> {
    // 1. Elaborate the declared type (recursive views are annotated).
    let ty_core = {
        let mut cx = ElabCtx::new(env, globals, num_values, numeric_env);
        let ty = rdecl.ty.as_ref().ok_or_else(|| ElabError::Internal(
            "recursive view/let requires a type annotation".into(),
        ))?;
        let ty_c = elab_type(&mut cx, ty)?;
        cx.metas.zonk_term(&ty_c)
    };

    // 2. Pre-admit as Opaque so the body can self-reference.
    let id = env.fresh_id();
    env.add_decl(Decl::Opaque {
        id,
        level_params: vec![],
        ty: ty_core.clone(),
    });
    globals.insert(rdecl.name.clone(), id);

    // 3. Elaborate the body (self-ref resolves to `id` via globals).
    let (body_core, body_obligations) = {
        let mut cx = ElabCtx::new(env, globals, num_values, numeric_env).with_classes(class_env);
        let body_c = check(&mut cx, &rdecl.body, &ty_core, &rdecl.span)?;
        let obligations = std::mem::take(&mut cx.obligations);
        (cx.metas.zonk_term(&body_c), obligations)
    };

    // 4. Kernel type-check + SCT gate (singleton recursive group).
    let admit_result = kernel_check(env, &Context::new(), &body_core, &ty_core)
        .and_then(|_| sct_check(env, &[(id, body_core.clone())]));

    match admit_result {
        Ok(()) => {
            // 5. SCT accepted → upgrade opaque to transparent (δ-unfoldable).
            env.upgrade_to_transparent(id, body_core);
            Ok(ElabResult {
                name: rdecl.name.clone(),
                def_id: id,
                obligations: body_obligations,
                foreign_binding: None,
                temporal_obligations: vec![],
            })
        }
        Err(e) => {
            // Roll back: remove the pre-admitted opaque and any literal
            // postulates body elaboration added after it (remove_last until we
            // hit our opaque), then unbind the name.
            while let Some(d) = env.remove_last() {
                if d.id() == id {
                    break;
                }
            }
            globals.remove(&rdecl.name);
            Err(ElabError::KernelRejected { error: e, span: rdecl.span.clone() })
        }
    }
}

/// Does `expr` mention the global name `name` (as an `RCon`)? Used to detect
/// whether a view/let definition is self-recursive — the body references its
/// own name, which the resolver emits as `RCon(name)` on a scope miss. Pattern
/// positions are not scanned: a def name is a view/function, never a
/// constructor, so it cannot appear in a pattern.
fn rexpr_mentions_name(expr: &RExpr, name: &str) -> bool {
    match expr {
        RExpr::RCon(n, _) => n == name,
        RExpr::RVar(_, _, _) | RExpr::RUniv(_, _) | RExpr::RNumLit(_, _) | RExpr::RStr(_, _) => false,
        RExpr::RApp(f, a, _) => {
            rexpr_mentions_name(f, name) || rexpr_mentions_name(a, name)
        }
        RExpr::RLam(_, b, _) => rexpr_mentions_name(b, name),
        RExpr::RLet(_, _, rhs, body, _) => {
            rexpr_mentions_name(rhs, name) || rexpr_mentions_name(body, name)
        }
        RExpr::RAsc(e, _, _) => rexpr_mentions_name(e, name),
        RExpr::ROld(e, _) => rexpr_mentions_name(e, name),
        RExpr::RBinOp(_, l, r, _) => {
            rexpr_mentions_name(l, name) || rexpr_mentions_name(r, name)
        }
        RExpr::RMatch { scrut, arms, .. } => {
            rexpr_mentions_name(scrut, name)
                || arms.iter().any(|a| rexpr_mentions_name(&a.body, name))
        }
        RExpr::RProj(e, _, _) => rexpr_mentions_name(e, name),
    }
}

/// Elaborate a `view` with `requires`/`ensures` clauses (`21 §6.3`).
fn elaborate_view_with_spec(
    env: &mut GlobalEnv,
    globals: &mut HashMap<String, GlobalId>,
    num_values: &mut HashMap<GlobalId, NumericLitVal>,
    numeric_env: &NumericEnv,
    class_env: &ClassEnv,
    rdecl: &RDecl,
) -> Result<ElabResult, ElabError> {
    let omega = Term::omega(Level::Zero);

    // Phase 1: elaborate the declared type (carrier) and body.
    //
    // A self-recursive spec'd view (e.g. `sort`) must have its name pre-admitted
    // as Opaque before the body is elaborated, so the body's self-call resolves
    // (Approach A; see `elaborate_recursive_view`). The non-recursive path keeps
    // type+body in one context so their level metas unify.
    let is_recursive = rexpr_mentions_name(&rdecl.body, &rdecl.name);

    let (body_raw, carrier_ty_raw, pre_admit_id): (Term, Term, Option<GlobalId>) =
        if is_recursive {
            // Recursive: elab the carrier type, pre-admit, then elab the body.
            let carrier_ty = {
                let mut cx = ElabCtx::new(env, globals, num_values, numeric_env).with_classes(class_env);
                let ty = rdecl.ty.as_ref().ok_or_else(|| ElabError::Internal(
                    "recursive view with spec clauses requires a type annotation".into(),
                ))?;
                let ty_c = elab_type(&mut cx, ty)?;
                cx.metas.zonk_term(&ty_c)
            };
            let id = env.fresh_id();
            env.add_decl(Decl::Opaque {
                id,
                level_params: vec![],
                ty: carrier_ty.clone(),
            });
            globals.insert(rdecl.name.clone(), id);
            let body = {
                let mut cx = ElabCtx::new(env, globals, num_values, numeric_env).with_classes(class_env);
                let body_c = check(&mut cx, &rdecl.body, &carrier_ty, &rdecl.span)?;
                cx.metas.zonk_term(&body_c)
            };
            (body, carrier_ty, Some(id))
        } else {
            // Non-recursive: original one-context flow.
            let mut cx = ElabCtx::new(env, globals, num_values, numeric_env).with_classes(class_env);
            if let Some(ty) = &rdecl.ty {
                let ty_c = elab_type(&mut cx, ty)?;
                let body_c = check(&mut cx, &rdecl.body, &ty_c, &rdecl.span)?;
                (cx.metas.zonk_term(&body_c), cx.metas.zonk_term(&ty_c), None)
            } else {
                let (body_c, ty_c) = infer(&mut cx, &rdecl.body)?;
                (cx.metas.zonk_term(&body_c), cx.metas.zonk_term(&ty_c), None)
            }
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
            env, globals, num_values, numeric_env, class_env, &param_ctx, req, &omega, &rdecl.span,
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
            env, globals, num_values, numeric_env, class_env, &ens_ctx, ens, &omega, &rdecl.span,
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

    let id = if let Some(pre_id) = pre_admit_id {
        // Recursive: the opaque was pre-admitted with the carrier Pi-chain. For
        // L3a's recursive views (no `requires`), `full_ty` == the carrier
        // Pi-chain, so the opaque's type is already `full_ty`. Kernel-check +
        // SCT-gate the singleton group, then upgrade. (A recursive view WITH
        // `requires` — `full_ty` ≠ carrier — is a tracked follow-on; see
        // `elaborate_recursive_view`'s K2c note.)
        let result = kernel_check(env, &Context::new(), &full_body, &full_ty)
            .and_then(|_| sct_check(env, &[(pre_id, full_body.clone())]));
        match result {
            Ok(()) => {
                env.upgrade_to_transparent(pre_id, full_body);
                pre_id
            }
            Err(e) => {
                // Roll back the pre-admission + any obligation holes / literal
                // postulates added after it (ensures holes from Phase 3, etc.).
                while let Some(d) = env.remove_last() {
                    if d.id() == pre_id {
                        break;
                    }
                }
                globals.remove(&rdecl.name);
                return Err(ElabError::KernelRejected {
                    error: e,
                    span: rdecl.span.clone(),
                });
            }
        }
    } else {
        let id = declare_def(env, vec![], full_ty, full_body).map_err(|e| {
            ElabError::KernelRejected { error: e, span: rdecl.span.clone() }
        })?;
        globals.insert(rdecl.name.clone(), id);
        id
    };
    Ok(ElabResult { name: rdecl.name.clone(), def_id: id, obligations: ens_obligations, foreign_binding: None, temporal_obligations: vec![] })
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
    Ok(ElabResult { name: rdecl.name.clone(), def_id: hole_id, obligations: vec![obl], foreign_binding: None, temporal_obligations: vec![] })
}

/// Elaborate `temporal name { φ }` — a delegated temporal/behavioral
/// obligation (`72 §4`).
///
/// The surface formula elaborates to a [`Temporal`] value (the §3
/// constructors, derived ops expanded) and is recorded as a **delegated**
/// obligation — **not** a kernel hole. A delegated property is exported, not
/// assumed (`21 §5.2`): it never enters `trusted_base()` (it is not
/// `unknown`) and is never kernel-proved (not `proved`/`Q`). Its sole
/// projection is the B1 `T`/`delegated` channel (TE-E). The verbatim `source`
/// is carried for human-visibility (`72 §4`).
fn elaborate_temporal(
    env: &mut GlobalEnv,
    globals: &mut HashMap<String, GlobalId>,
    rdecl: &RDecl,
    formula: &crate::temporal::TemporalExpr,
    source: &str,
) -> Result<ElabResult, ElabError> {
    use crate::temporal::{elaborate_temporal_expr, TemporalObligation};

    let temporal_value = elaborate_temporal_expr(formula);
    // Stable obligation id (`22 §1`): one obligation per `temporal{}` block.
    let id = format!("{}.temporal.0", rdecl.name);
    let obl = TemporalObligation {
        id,
        formula: temporal_value,
        source: source.to_string(),
    };

    // Delegated ≠ unknown: allocate a placeholder `def_id` that is NOT
    // committed to the kernel env, so the obligation never enters
    // `trusted_base()`. Reserve the name in `globals`.
    let placeholder = env.fresh_id();
    globals.insert(rdecl.name.clone(), placeholder);

    Ok(ElabResult {
        name: rdecl.name.clone(),
        def_id: placeholder,
        obligations: vec![],
        foreign_binding: None,
        temporal_obligations: vec![obl],
    })
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
    Ok(ElabResult { name: rdecl.name.clone(), def_id: law_id, obligations, foreign_binding: None, temporal_obligations: vec![] })
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
    class_env: &ClassEnv,
    ctx: &Context,
    expr: &RExpr,
    omega: &Term,
    span: &Span,
) -> Result<Term, ElabError> {
    let mut cx = ElabCtx::new(env, globals, num_values, numeric_env).with_classes(class_env);
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
/// A pending column in the pattern-matrix compiler (`34-data-match.md §3.1`):
/// either a genuine surface column (tracked per-row in `RowState::real_pats`)
/// or a synthetic induction-hypothesis slot the eliminator's method type
/// requires but no surface pattern ever names.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ColKind {
    Real,
    Ih,
}

/// One row of the pattern matrix: the still-unconsumed `Real` column
/// patterns for one arm, plus which top-level arm it came from (for
/// reachability bookkeeping across wildcard-row expansion, `§4.2`).
struct RowState {
    real_pats: Vec<RPattern>,
    arm_idx: usize,
}

/// The type every raw method built from `col_types`/`col_kinds` (a suffix of
/// still-pending columns) ultimately has, as a Pi-chain ending in `ret_ty`:
/// each `Real` column contributes one arrow (regardless of whether it is
/// later bound flatly or split further — that happens *inside* the arrow's
/// codomain, never changing the arrow's own presence), each `Ih` column
/// contributes an arrow of type `ret_ty` weakened to its own position. This
/// is exactly what a split's motive must compute once applied to a
/// scrutinee value — a nested `elim_D` still owes whatever the tail owes.
fn tail_codomain(
    tail_col_types: &[Term],
    tail_col_kinds: &[ColKind],
    ret_ty_base: &Term,
    depth_before_tail: usize,
) -> Term {
    if tail_col_types.is_empty() {
        return weaken(ret_ty_base, depth_before_tail as i64);
    }
    match tail_col_kinds[0] {
        ColKind::Ih => {
            let ih_ty = weaken(ret_ty_base, depth_before_tail as i64);
            let rest =
                tail_codomain(&tail_col_types[1..], &tail_col_kinds[1..], ret_ty_base, depth_before_tail);
            Term::pi(ih_ty, weaken(&rest, 1))
        }
        ColKind::Real => {
            let rest = tail_codomain(
                &tail_col_types[1..],
                &tail_col_kinds[1..],
                ret_ty_base,
                depth_before_tail + 1,
            );
            Term::pi(tail_col_types[0].clone(), rest)
        }
    }
}

/// Compile the pattern matrix `col_types`/`col_kinds` (aligned; `Real`
/// columns are matched against `rows[_].real_pats`, `Ih` columns are
/// synthetic and never touch row patterns) down to a nested-`elim_D` method
/// term, per the standard column-by-column algorithm.
///
/// `real_depth_so_far` counts only genuine (`Real`, non-split) `cx.ctx`
/// pushes made along the current path — it lines up with what `resolve.rs`
/// counted when flattening pattern-bound names, so `infer`'s raw
/// `Term::var(i)` passthrough resolves correctly. Columns that need
/// splitting (a `Ctor` sub-pattern present) or `Ih` slots are *never* pushed
/// onto `cx.ctx` — they are woven in afterward via `weaken`, exactly as the
/// pre-existing single-level code already did for induction hypotheses.
fn compile_match_matrix(
    cx: &mut ElabCtx,
    arms: &[RMatchArm],
    col_types: &[Term],
    col_kinds: &[ColKind],
    rows: Vec<RowState>,
    real_depth_so_far: usize,
    top_span: &Span,
    ret_ty_slot: &mut Option<Term>,
    arm_used: &mut [bool],
) -> Result<Term, ElabError> {
    if col_types.is_empty() {
        // Leaf: the first row in preserved (first-match-wins) order claims
        // this path; any others are shadowed here (possibly still reachable
        // via a different expansion elsewhere — checked globally by the
        // caller via `arm_used`).
        let winner = rows[0].arm_idx;
        arm_used[winner] = true;
        let arm = &arms[winner];
        let (body_core, body_ty_ctx) = infer(cx, &arm.body)?;
        if ret_ty_slot.is_none() {
            let zonked = cx.metas.zonk_term(&body_ty_ctx);
            let lowered = lower_by(&zonked, real_depth_so_far).unwrap_or(zonked);
            *ret_ty_slot = Some(lowered);
        }
        return Ok(body_core);
    }

    match col_kinds[0] {
        ColKind::Ih => {
            // A synthetic induction-hypothesis slot: never resolver-counted,
            // so it is woven in via weaken-then-wrap rather than a real push.
            //
            // The IH's type is `M(k)` for the enclosing elim's motive `M` —
            // NOT necessarily the bare global return type: when a nested
            // split still owes a pending tail (this Ih slot's own
            // continuation, `col_types[1..]`/`col_kinds[1..]`), `M` is a
            // constant motive equal to that tail's own codomain (the very
            // value `tail_codomain` computes when building that split's
            // motive) — so re-derive it identically from this slot's own
            // position.
            let ret_ty = ret_ty_slot
                .as_ref()
                .expect("IH column reached before return type known")
                .clone();
            let ih_ty =
                tail_codomain(&col_types[1..], &col_kinds[1..], &ret_ty, real_depth_so_far);
            let inner = compile_match_matrix(
                cx,
                arms,
                &col_types[1..],
                &col_kinds[1..],
                rows,
                real_depth_so_far,
                top_span,
                ret_ty_slot,
                arm_used,
            )?;
            Ok(Term::lam(ih_ty, weaken(&inner, 1)))
        }
        ColKind::Real => {
            let all_flat = rows
                .iter()
                .all(|r| matches!(r.real_pats[0].kind, RPatKind::Wild | RPatKind::Var(_)));
            if all_flat {
                // No constructor pattern in this column across any row: bind
                // it flatly (a real `cx.ctx` push), matching the resolver's
                // count exactly, and move on.
                cx.ctx.push(col_types[0].clone());
                let new_rows: Vec<RowState> = rows
                    .into_iter()
                    .map(|r| RowState {
                        real_pats: r.real_pats[1..].to_vec(),
                        arm_idx: r.arm_idx,
                    })
                    .collect();
                let inner = compile_match_matrix(
                    cx,
                    arms,
                    &col_types[1..],
                    &col_kinds[1..],
                    new_rows,
                    real_depth_so_far + 1,
                    top_span,
                    ret_ty_slot,
                    arm_used,
                );
                cx.ctx.pop();
                return Ok(Term::lam(col_types[0].clone(), inner?));
            }

            // At least one row has a constructor pattern here: split.
            let ty0 = whnf(cx.env, &cx.ctx, &col_types[0]);
            let (head, params0) = peel_app(&ty0);
            let d_id0 = match head {
                Term::IndFormer { id, .. } => id,
                _ => {
                    return Err(ElabError::TypeMismatch {
                        span: top_span.clone(),
                        reason: "match scrutinee must have an inductive type".into(),
                    })
                }
            };
            let ind0 = cx
                .env
                .inductive(d_id0)
                .ok_or_else(|| ElabError::Internal(format!("inductive {:?} not found", d_id0)))?
                .clone();
            let m0 = ind0.params.len();

            let raw_methods = build_ctor_buckets(
                cx,
                arms,
                &ind0,
                d_id0,
                m0,
                &params0,
                rows,
                &col_types[1..],
                &col_kinds[1..],
                real_depth_so_far,
                top_span,
                ret_ty_slot,
                arm_used,
            )?;

            // The split column itself is a fresh binder no surface pattern
            // named — resolver never counted it, so (like the IH slots
            // above) it is woven in via weaken-then-wrap, never a real push.
            //
            // The motive's codomain is NOT bare `ret_ty`: any columns still
            // pending after this split (a sibling field, or an enclosing
            // constructor's own IH slot carried in via `tail_col_kinds`)
            // still owe a value, so each raw method's real type is
            // `(tail columns) -> ret_ty`, and the motive must match.
            let ret_ty_base = ret_ty_slot
                .as_ref()
                .expect("split column reached before return type known")
                .clone();
            let codomain = tail_codomain(
                &col_types[1..],
                &col_kinds[1..],
                &ret_ty_base,
                real_depth_so_far + 1,
            );
            let ret_level = match kernel_infer(cx.env, &cx.ctx, &codomain) {
                Ok(Term::Type(l)) => l,
                _ => Level::Zero,
            };
            let motive_ty = Term::pi(col_types[0].clone(), Term::ty(ret_level));
            let motive = Term::Ascript(
                Box::new(Term::lam(col_types[0].clone(), codomain)),
                Box::new(motive_ty),
            );
            let methods: Vec<Term> = raw_methods.iter().map(|m| weaken(m, 1)).collect();
            let elim = Term::Elim {
                fam: d_id0,
                level_args: vec![],
                params: params0.iter().map(|p| weaken(p, 1)).collect(),
                motive: Box::new(motive),
                methods,
                indices: vec![],
                scrut: Box::new(Term::var(0)),
            };
            Ok(Term::lam(col_types[0].clone(), elim))
        }
    }
}

/// Group `rows` (whose `real_pats[0]` matches the inductive `ind0`) into one
/// bucket per constructor — expanding a `Wild`/`Var` row into every
/// constructor (it matches all of them) — and recurse to build each
/// constructor's raw method term: `λ(fields). λ(IHs). <continuation>`,
/// where `<continuation>` threads through `tail_col_types`/`tail_col_kinds`
/// (the columns after this one). Each returned method is valid at
/// `real_depth_so_far` — i.e. as if the split column's own binder does not
/// yet exist; the caller (top-level `infer_match`, or a nested nested split
/// in `compile_match_matrix`) wraps accordingly.
#[allow(clippy::too_many_arguments)]
fn build_ctor_buckets(
    cx: &mut ElabCtx,
    arms: &[RMatchArm],
    ind0: &InductiveDecl,
    d_id0: GlobalId,
    m0: usize,
    params0: &[Term],
    rows: Vec<RowState>,
    tail_col_types: &[Term],
    tail_col_kinds: &[ColKind],
    real_depth_so_far: usize,
    top_span: &Span,
    ret_ty_slot: &mut Option<Term>,
    arm_used: &mut [bool],
) -> Result<Vec<Term>, ElabError> {
    let mut methods: Vec<Option<Term>> = vec![None; ind0.constructors.len()];

    for (k0, c0) in ind0.constructors.iter().enumerate() {
        let mut bucket: Vec<RowState> = Vec::new();
        for r in &rows {
            match &r.real_pats[0].kind {
                RPatKind::Ctor(name, subs) => {
                    if cx.globals.get(name).copied() == Some(c0.id) {
                        let mut new_pats = subs.clone();
                        new_pats.extend_from_slice(&r.real_pats[1..]);
                        bucket.push(RowState { real_pats: new_pats, arm_idx: r.arm_idx });
                    }
                }
                RPatKind::Wild | RPatKind::Var(_) => {
                    let span = r.real_pats[0].span.clone();
                    let mut new_pats: Vec<RPattern> = (0..c0.args.len())
                        .map(|_| RPattern { kind: RPatKind::Wild, span: span.clone() })
                        .collect();
                    new_pats.extend_from_slice(&r.real_pats[1..]);
                    bucket.push(RowState { real_pats: new_pats, arm_idx: r.arm_idx });
                }
            }
        }

        if bucket.is_empty() {
            let name0 = cx
                .globals
                .iter()
                .find(|(_, &id)| id == c0.id)
                .map(|(n, _)| n.clone())
                .unwrap_or_else(|| format!("<ctor_{:?}>", c0.id));
            return Err(ElabError::ExhaustivenessError {
                missing: name0,
                span: top_span.clone(),
            });
        }

        let n_args0 = c0.args.len();
        let field_types0: Vec<Term> =
            (0..n_args0).map(|j| subst_outer(&c0.args[j], m0, params0, j)).collect();
        let p_ihs0 = recursive_args(c0, d_id0, m0).len();

        // `col_types`/`col_kinds` stay index-aligned; an `Ih` slot's own type
        // entry is never read (its lambda domain is computed from `ret_ty`
        // instead) but must still occupy a position.
        let mut new_col_types = field_types0;
        new_col_types.extend(std::iter::repeat(Term::ty(Level::Zero)).take(p_ihs0));
        new_col_types.extend_from_slice(tail_col_types);
        let mut new_col_kinds: Vec<ColKind> = vec![ColKind::Real; n_args0];
        new_col_kinds.extend(std::iter::repeat(ColKind::Ih).take(p_ihs0));
        new_col_kinds.extend_from_slice(tail_col_kinds);

        let inner = compile_match_matrix(
            cx,
            arms,
            &new_col_types,
            &new_col_kinds,
            bucket,
            real_depth_so_far,
            top_span,
            ret_ty_slot,
            arm_used,
        )?;
        methods[k0] = Some(inner);
    }

    Ok(methods.into_iter().map(|m| m.unwrap()).collect())
}

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
    //    mutating cx.ctx inside the recursive matrix compiler.
    let ind = cx
        .env
        .inductive(d_id)
        .ok_or_else(|| ElabError::Internal(format!("inductive {:?} not found", d_id)))?
        .clone();
    let m = ind.params.len();

    // 4. Every arm must open with a constructor pattern (no top-level
    //    wildcard/var scrutinee-binding yet); nested sub-patterns may be
    //    arbitrary (`Ctor`, `Var`, `Wild`, recursively).
    for arm in arms {
        if let RPatKind::Wild | RPatKind::Var(_) = arm.pat.kind {
            return Err(ElabError::Internal(
                "non-constructor pattern in match (wildcard/var not yet supported \
                 at top level; use constructor patterns)"
                    .into(),
            ));
        }
    }

    // 5. Build the initial one-column matrix (the scrutinee itself) and
    //    compile it via the pattern-matrix algorithm (`34-data-match.md
    //    §3.1`): column-by-column, splitting on constructors, recursing on
    //    the residual matrix under each constructor's freshly-bound fields.
    let rows: Vec<RowState> = arms
        .iter()
        .enumerate()
        .map(|(i, arm)| RowState { real_pats: vec![arm.pat.clone()], arm_idx: i })
        .collect();

    let mut ret_ty_slot: Option<Term> = None;
    let mut arm_used = vec![false; arms.len()];

    let raw_methods = build_ctor_buckets(
        cx,
        arms,
        &ind,
        d_id,
        m,
        &params_terms,
        rows,
        &[],
        &[],
        0,
        span,
        &mut ret_ty_slot,
        &mut arm_used,
    )?;

    // 6. AC4: reachability — an arm that never won at any leaf (including any
    //    it was expanded into via a wildcard row) is dead code.
    for (i, used) in arm_used.iter().enumerate() {
        if !used {
            return Err(ElabError::ReachabilityError { span: arms[i].span.clone() });
        }
    }

    let ret_ty = ret_ty_slot.unwrap_or_else(|| Term::ty(Level::Zero));

    // 7. Build the constant motive: Ascript(λ(x: D). R, D → Type ℓ)
    //    The kernel can't infer the type of a bare lambda, so we annotate.
    //    Determine ℓ from the return type's own type.
    let ret_level = {
        match kernel_infer(cx.env, &cx.ctx, &ret_ty) {
            Ok(Term::Type(l)) => l,
            _ => Level::Zero, // fallback: level 0
        }
    };
    let motive_ty = Term::pi(scrut_ty.clone(), Term::ty(ret_level));
    let motive = Term::Ascript(
        Box::new(Term::lam(scrut_ty.clone(), weaken(&ret_ty, 1))),
        Box::new(motive_ty),
    );

    // 8. Build Term::Elim (non-indexed: indices = []). The top-level
    //    scrutinee is already a concrete elaborated value (`scrut_core`), so
    //    — unlike a nested split — no extra binder/weaken is needed here.
    let elim = Term::Elim {
        fam: d_id,
        level_args: vec![],
        params: params_terms,
        motive: Box::new(motive),
        methods: raw_methods,
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
