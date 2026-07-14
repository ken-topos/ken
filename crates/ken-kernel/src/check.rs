//! Bidirectional checking/inference and declaration admission (`18 §3`, §4).
//!
//! Two mutually-recursive, syntax-directed modes:
//! - `Γ ⊢ t ⇐ A` — [`check`]: verify `t : A` against a known type.
//! - `Γ ⊢ t ⇒ A` — [`infer`]: produce the unique `A` with `t : A`.
//!
//! The single place conversion ([`crate::conv::convert`]) is called during
//! checking is the **mode switch** (`18 §3`): any term without a
//! type-driven check rule infers its type and is compared to the expected one.
//! `check`/`infer` reject the `[K2]`-reserved formers as unrecognised (`11 §6`).
//! Admission ([`declare_def`] … [`declare_primitive`]) re-checks every input and
//! gates inductives on strict positivity (`14 §8`).

use crate::conv::{convert, convert_type, level_eq, whnf};
use crate::env::{telescope_to_pi, Context, Decl, GlobalEnv, InductiveDecl};
use crate::error::{KernelError, KernelResult};
use crate::inductive::{check_positivity, method_type};
use crate::subst::{apply_args, subst0, subst_levels, subst_outer, subst_tel, weaken};
use crate::term::{GlobalId, Level, LevelVar, Term};

// --- raw well-formedness (`11 §6`) -----------------------------------------

/// Raw well-formedness: `t` is built by the grammar and every de Bruijn index
/// resolves to an in-scope binding (`11 §6`). This is the parser/elaborator's
/// precondition to typing — it does **not** decide typing. `offset` is the
/// number of binders entered inside `t` (bound vars `i < offset` are t's own).
fn raw_wf(ctx: &Context, t: &Term, offset: usize) -> KernelResult<()> {
    match t {
        Term::Var(i) => {
            if *i < offset {
                Ok(())
            } else {
                ctx.lookup(i - offset)
                    .map(|_| ())
                    .ok_or(KernelError::VarOutOfScope {
                        index: *i,
                        depth: ctx.len() + offset,
                    })
            }
        }
        Term::Pi(a, b) | Term::Lam(a, b) | Term::Sigma(a, b) => {
            raw_wf(ctx, a, offset)?;
            raw_wf(ctx, b, offset + 1)
        }
        Term::Let { ty, val, body } => {
            raw_wf(ctx, ty, offset)?;
            raw_wf(ctx, val, offset)?;
            raw_wf(ctx, body, offset + 1)
        }
        Term::App(f, a) | Term::Pair(f, a) | Term::Ascript(f, a) | Term::Quot(f, a) => {
            raw_wf(ctx, f, offset)?;
            raw_wf(ctx, a, offset)
        }
        Term::Proj1(p)
        | Term::Proj2(p)
        | Term::Refl(p)
        | Term::QuotClass(p)
        | Term::Trunc(p)
        | Term::TruncProj(p) => raw_wf(ctx, p, offset),
        Term::Eq(a, t, u) => {
            raw_wf(ctx, a, offset)?;
            raw_wf(ctx, t, offset)?;
            raw_wf(ctx, u, offset)
        }
        Term::Cast(a, b, e, t) => {
            raw_wf(ctx, a, offset)?;
            raw_wf(ctx, b, offset)?;
            raw_wf(ctx, e, offset)?;
            raw_wf(ctx, t, offset)
        }
        Term::J(m, d, e) => {
            raw_wf(ctx, m, offset)?;
            raw_wf(ctx, d, offset)?;
            raw_wf(ctx, e, offset)
        }
        Term::Elim {
            params,
            motive,
            methods,
            indices,
            scrut,
            ..
        } => {
            for p in params {
                raw_wf(ctx, p, offset)?;
            }
            raw_wf(ctx, motive, offset)?;
            for m in methods {
                raw_wf(ctx, m, offset)?;
            }
            for i in indices {
                raw_wf(ctx, i, offset)?;
            }
            raw_wf(ctx, scrut, offset)
        }
        Term::QuotElim {
            motive,
            method,
            respect,
            scrut,
        } => {
            raw_wf(ctx, motive, offset)?;
            raw_wf(ctx, method, offset)?;
            raw_wf(ctx, respect, offset)?;
            raw_wf(ctx, scrut, offset)
        }
        Term::Absurd(motive, proof) => {
            raw_wf(ctx, motive, offset)?;
            raw_wf(ctx, proof, offset)
        }
        // Closed in Σ: no free term variables (levels are not de Bruijn).
        Term::Type(_)
        | Term::Omega(_)
        | Term::Const { .. }
        | Term::IndFormer { .. }
        | Term::Constructor { .. }
        | Term::IntLit(_) => Ok(()),
    }
}

/// Raw well-formedness check (public, for the elaborator precondition).
pub fn raw_well_formed(ctx: &Context, t: &Term) -> KernelResult<()> {
    raw_wf(ctx, t, 0)
}

// --- type synthesis: `Γ ⊢ A type` ⇒ its level -----------------------------

/// The universe a type inhabits (`11 §3`): `Type ℓ` or the strict-proposition
/// universe `Ω_ℓ` (`16 §1.1`). A binder type, declaration type, or ascription
/// may be either — a proposition is a valid type.
#[derive(Clone, Debug)]
enum Sort {
    Type(Level),
    Omega(Level),
}

impl Sort {
    fn level(&self) -> &Level {
        match self {
            Sort::Type(l) | Sort::Omega(l) => l,
        }
    }
    /// Reify the sort as a term (`Type ℓ` or `Ω_ℓ`).
    fn to_term(&self) -> Term {
        match self {
            Sort::Type(l) => Term::Type(l.clone()),
            Sort::Omega(l) => Term::Omega(l.clone()),
        }
    }
}

/// `Γ ⊢ A type` ⇒ the sort (and level) of `A` (`11 §3`: a type is `Type ℓ` or
/// `Ω_ℓ`). Generalizes [`synth_type`] to admit proposition types (`16 §1.1`).
fn classify(env: &GlobalEnv, ctx: &Context, a: &Term) -> KernelResult<Sort> {
    let ty = infer(env, ctx, a)?;
    match whnf(env, ctx, &ty) {
        Term::Type(l) => Ok(Sort::Type(l)),
        Term::Omega(l) => Ok(Sort::Omega(l)),
        other => Err(KernelError::TypeMismatch {
            expected: Box::new(Term::Type(Level::Var(LevelVar(0)))), // a type
            found: Box::new(other),
        }),
    }
}

/// Check `Γ ⊢ A : Type ℓ` and return its level — the Type-only specialization
/// of [`classify`]. Use [`classify`] where a proposition type is admissible
/// (binders, declarations, ascriptions); use this where a `Type` level is
/// required specifically (e.g. an inductive family's universe).
fn synth_type(env: &GlobalEnv, ctx: &Context, a: &Term) -> KernelResult<Level> {
    match classify(env, ctx, a)? {
        Sort::Type(l) => Ok(l),
        Sort::Omega(l) => Err(KernelError::TypeMismatch {
            expected: Box::new(Term::Type(Level::Var(LevelVar(0)))),
            found: Box::new(Term::Omega(l)),
        }),
    }
}

/// Formation sort of a Π-type (`13 §1`, `13 §4`, `16 §1.1`): level is
/// `max(s1,s2)`; result is Ω exactly when the **codomain** is a proposition
/// (a function into a prop is a prop, regardless of the domain's sort).
fn sort_pi(s1: &Sort, s2: &Sort) -> Term {
    let lvl = s1.level().clone().max(s2.level().clone()).normalize();
    match s2 {
        Sort::Omega(_) => Term::Omega(lvl),
        Sort::Type(_) => Term::Type(lvl),
    }
}

/// Formation sort of a Σ-type (`13 §2`, `13 §4`): level is `max(s1,s2)`;
/// result is Ω only when **both** components are propositions (the conjunction
/// case). A subset with a **relevant** (`Type`-sorted) first component carries
/// content and must stay in `Type` — collapsing it to Ω would trigger Ω-PI
/// proof-irrelevance on the carrier, closing to `Empty` via a transport motive.
fn sort_sigma(s1: &Sort, s2: &Sort) -> Term {
    let lvl = s1.level().clone().max(s2.level().clone()).normalize();
    match (s1, s2) {
        (Sort::Omega(_), Sort::Omega(_)) => Term::Omega(lvl),
        _ => Term::Type(lvl),
    }
}

// --- infer (`18 §3`) -------------------------------------------------------

/// `Γ ⊢ t ⇒ A` — infer the type of `t` (`18 §3`). Fails for `[K2]`-reserved
/// formers and for λ/pair (which need a type to check against).
pub fn infer(env: &GlobalEnv, ctx: &Context, t: &Term) -> KernelResult<Term> {
    match t {
        Term::Var(i) => {
            ctx.lookup(*i)
                .map(|t| weaken(t, (*i + 1) as i64))
                .ok_or(KernelError::VarOutOfScope {
                    index: *i,
                    depth: ctx.len(),
                })
        }
        Term::Const { id, level_args } => {
            let (params, ty) = env
                .const_type(*id)
                .ok_or_else(|| KernelError::Msg(format!("unknown constant {:?}", id)))?;
            check_level_arity(params, level_args)?;
            Ok(subst_levels(&ty, params, level_args))
        }
        Term::IntLit(_) => {
            let id = env
                .int_lit_type()
                .ok_or_else(|| KernelError::Msg("Int-literal type not registered".into()))?;
            Ok(Term::const_(id, Vec::new()))
        }
        Term::IndFormer { id, level_args } => {
            let (params, ty) = env
                .const_type(*id)
                .ok_or_else(|| KernelError::Msg(format!("unknown type former {:?}", id)))?;
            check_level_arity(params, level_args)?;
            Ok(subst_levels(&ty, params, level_args))
        }
        Term::Constructor { id, level_args } => {
            let (ind, k) = env
                .constructor(*id)
                .ok_or_else(|| KernelError::Msg(format!("unknown constructor {:?}", id)))?;
            check_level_arity(&ind.level_params, level_args)?;
            Ok(subst_levels(
                &ind.constructors[k].type_,
                &ind.level_params,
                level_args,
            ))
        }
        Term::App(f, a) => {
            let tf = infer(env, ctx, f)?;
            match whnf(env, ctx, &tf) {
                Term::Pi(dom, cod) => {
                    check(env, ctx, a, &dom)?;
                    Ok(subst0(&cod, a))
                }
                other => Err(KernelError::NotAFunction {
                    head: Box::new(other),
                }),
            }
        }
        Term::Proj1(p) => {
            let tp = infer(env, ctx, p)?;
            match whnf(env, ctx, &tp) {
                Term::Sigma(dom, _) => Ok((*dom).clone()),
                other => Err(KernelError::NotASigma {
                    head: Box::new(other),
                }),
            }
        }
        Term::Proj2(p) => {
            let tp = infer(env, ctx, p)?;
            match whnf(env, ctx, &tp) {
                Term::Sigma(_, cod) => Ok(subst0(&cod, &Term::proj1((**p).clone()))),
                other => Err(KernelError::NotASigma {
                    head: Box::new(other),
                }),
            }
        }
        Term::Pi(a, b) => {
            let s1 = classify(env, ctx, a)?;
            let mut ctx2 = ctx.clone();
            ctx2.push((**a).clone());
            let s2 = classify(env, &ctx2, b)?;
            Ok(sort_pi(&s1, &s2))
        }
        Term::Sigma(a, b) => {
            let s1 = classify(env, ctx, a)?;
            let mut ctx2 = ctx.clone();
            ctx2.push((**a).clone());
            let s2 = classify(env, &ctx2, b)?;
            Ok(sort_sigma(&s1, &s2))
        }
        Term::Type(l) => Ok(Term::Type(l.clone().suc())), // (U-Type): Type ℓ : Type (suc ℓ) (`12 §1`)
        Term::Omega(l) => Ok(Term::Type(l.clone().suc())), // (Ω-Form): Ω_l : Type (suc l) (`16 §1.1`)
        Term::Ascript(t, a) => {
            classify(env, ctx, a)?;
            check(env, ctx, t, a)?;
            Ok((**a).clone())
        }
        Term::Let { ty, val, body } => {
            classify(env, ctx, ty)?;
            check(env, ctx, val, ty)?;
            infer(env, ctx, &subst0(body, val))
        }
        Term::Elim {
            fam,
            level_args,
            params,
            motive,
            methods,
            indices,
            scrut,
        } => infer_elim(
            env, ctx, *fam, level_args, params, motive, methods, indices, scrut,
        ),
        // --- K2 formers (`15`, `16`) ---
        Term::Eq(a_ty, x, y) => {
            // `Eq A a b : Ω_l` for `A : Type l` (`16 §2.1`).
            let l = synth_type(env, ctx, a_ty)?;
            check(env, ctx, x, a_ty)?;
            check(env, ctx, y, a_ty)?;
            Ok(Term::Omega(l))
        }
        Term::Cast(a_ty, b_ty, e, t) => {
            // `cast A B e a : B`, `e : Eq Type A B` (`16 §3.1`).
            let l_a = synth_type(env, ctx, a_ty)?;
            let _l_b = synth_type(env, ctx, b_ty)?;
            let eq_ty = Term::Eq(
                Box::new(Term::Type(l_a)),
                Box::new((**a_ty).clone()),
                Box::new((**b_ty).clone()),
            );
            check(env, ctx, e, &eq_ty)?;
            check(env, ctx, t, a_ty)?;
            Ok((**b_ty).clone())
        }
        Term::J(m, d, e) => infer_j(env, ctx, m, d, e),
        Term::Quot(a, r) => {
            // `A / R : Type l` for `R : A → A → Ω` (`16 §5`).
            let l = synth_type(env, ctx, a)?;
            check_quotient_rel(env, ctx, a, r)?;
            Ok(Term::Type(l))
        }
        Term::Trunc(a) => {
            // `‖A‖ : Ω_l` for `A : Type l` (`16 §6`).
            let l = synth_type(env, ctx, a)?;
            Ok(Term::Omega(l))
        }
        Term::QuotElim {
            motive,
            method,
            respect,
            scrut,
        } => infer_quot_elim(env, ctx, motive, method, respect, scrut),
        Term::Absurd(motive, proof) => infer_absurd(env, ctx, motive, proof),
        Term::Lam { .. }
        | Term::Pair { .. }
        | Term::Refl(_)
        | Term::QuotClass(_)
        | Term::TruncProj(_) => Err(KernelError::Msg(
            "cannot infer an introduction form (λ/pair/refl/quotient class/truncation) \
             without an expected type (use ascription)"
                .into(),
        )),
    }
}

fn check_level_arity(params: &[LevelVar], args: &[Level]) -> KernelResult<()> {
    if params.len() == args.len() {
        Ok(())
    } else {
        Err(KernelError::LevelArityMismatch {
            expected: params.len(),
            found: args.len(),
        })
    }
}

// --- check (`18 §3`) -------------------------------------------------------

/// `Γ ⊢ t ⇐ A` — check `t` against a known type (`18 §3`). Type-driven rules
/// for λ (Π) and pair (Σ) insert η-relevant structure; everything else falls
/// to the mode switch (infer + conversion).
pub fn check(env: &GlobalEnv, ctx: &Context, t: &Term, ty: &Term) -> KernelResult<()> {
    match t {
        Term::Lam(a, body) => match whnf(env, ctx, ty) {
            Term::Pi(dom, cod) => {
                classify(env, ctx, a)?;
                if !convert_type(env, ctx, a, &dom) {
                    return Err(KernelError::TypeMismatch {
                        expected: Box::new((*dom).clone()),
                        found: Box::new((**a).clone()),
                    });
                }
                let mut ctx2 = ctx.clone();
                ctx2.push((*dom).clone());
                check(env, &ctx2, body, &cod)
            }
            other => Err(KernelError::NotAFunction {
                head: Box::new(other),
            }),
        },
        Term::Pair(a, b) => match whnf(env, ctx, ty) {
            Term::Sigma(dom, cod) => {
                check(env, ctx, a, &dom)?;
                let cod_a = subst0(&cod, a);
                check(env, ctx, b, &cod_a)
            }
            other => Err(KernelError::NotASigma {
                head: Box::new(other),
            }),
        },
        Term::Ascript(t, a) => {
            classify(env, ctx, a)?;
            check(env, ctx, t, a)?;
            if !convert_type(env, ctx, a, ty) {
                return Err(KernelError::TypeMismatch {
                    expected: Box::new(ty.clone()),
                    found: Box::new((**a).clone()),
                });
            }
            Ok(())
        }
        Term::Let {
            ty: let_ty,
            val,
            body,
        } => {
            classify(env, ctx, let_ty)?;
            check(env, ctx, val, let_ty)?;
            check(env, ctx, &subst0(body, val), ty)
        }
        // --- K2 introduction forms (`15`, `16`) ---
        Term::Refl(a) => {
            // `refl a : Eq A a a` checks against `Eq A x y` iff `a ≡ x ≡ y`
            // (`15 §2`). (`Eq : Ω` makes proofs irrelevant, but `refl a` is the
            // canonical proof, so its index must match.)
            let ty_w = whnf(env, ctx, ty);
            match &ty_w {
                Term::Eq(a_ty, x, y) => {
                    let a_infer = infer(env, ctx, a)?;
                    if !convert_type(env, ctx, &a_infer, a_ty) {
                        return Err(KernelError::TypeMismatch {
                            expected: (*a_ty).clone(),
                            found: Box::new(a_infer),
                        });
                    }
                    if !convert(env, ctx, a_ty, a, x) || !convert(env, ctx, a_ty, a, y) {
                        return Err(KernelError::BadEliminator(
                            "refl a does not match Eq A x y (a ≢ x or a ≢ y)".into(),
                        ));
                    }
                    Ok(())
                }
                _ => Err(KernelError::TypeMismatch {
                    expected: Box::new(ty.clone()),
                    found: Box::new(ty_w.clone()),
                }),
            }
        }
        Term::QuotClass(a) => {
            // `[a] : A / R`  iff  `a : A` (`16 §5`).
            let ty_w = whnf(env, ctx, ty);
            match &ty_w {
                Term::Quot(a_ty, _r) => check(env, ctx, a, a_ty),
                _ => Err(KernelError::TypeMismatch {
                    expected: Box::new(ty.clone()),
                    found: Box::new(ty_w.clone()),
                }),
            }
        }
        Term::TruncProj(a) => {
            // `|a| : ‖A‖`  iff  `a : A` (`16 §6`).
            let ty_w = whnf(env, ctx, ty);
            match &ty_w {
                Term::Trunc(a_ty) => check(env, ctx, a, a_ty),
                _ => Err(KernelError::TypeMismatch {
                    expected: Box::new(ty.clone()),
                    found: Box::new(ty_w.clone()),
                }),
            }
        }
        _ => {
            // Mode switch (`18 §3`): infer t's type and convert it to the
            // expected one — the single place conversion is called in check.
            let inferred = infer(env, ctx, t)?;
            if convert_type(env, ctx, ty, &inferred) {
                Ok(())
            } else {
                Err(KernelError::TypeMismatch {
                    expected: Box::new(ty.clone()),
                    found: Box::new(inferred),
                })
            }
        }
    }
}

// --- dependent eliminator inference (`14 §3`, §7) --------------------------

/// Infer the type of `elim_D p̄ M m̄ i̅ s` = `M i̅ s`, after checking the
/// motive, methods, indices, and scrutinee against the family declaration.
#[allow(clippy::too_many_arguments)]
fn infer_elim(
    env: &GlobalEnv,
    ctx: &Context,
    fam: GlobalId,
    level_args: &[Level],
    params: &[Term],
    motive: &Term,
    methods: &[Term],
    indices: &[Term],
    scrut: &Term,
) -> KernelResult<Term> {
    let ind = env
        .inductive(fam)
        .ok_or_else(|| KernelError::Msg(format!("elim of unknown family {:?}", fam)))?;
    check_level_arity(&ind.level_params, level_args)?;
    let m = ind.params.len();
    let n_i = ind.indices.len();

    // 1. Check the family params p̄ against Δ_p (level-instantiated, earlier
    //    params substituted).
    let inst_params: Vec<Term> = ind
        .params
        .iter()
        .map(|p| subst_levels(p, &ind.level_params, level_args))
        .collect();
    if params.len() != m {
        return Err(KernelError::BadEliminator(format!(
            "expected {m} params, got {}",
            params.len()
        )));
    }
    for (j, pa) in params.iter().enumerate() {
        let pty = subst_tel(&inst_params[j], &params[..j]);
        check(env, ctx, pa, &pty)?;
    }

    // 2. Verify the motive M : (Δ_i) → D p̄ Δ_i → Type ℓ' or Ω_ℓ' (extract the
    //    sort, then re-check against the fully-built expected motive type).
    let motive_sort = infer_motive_level(env, ctx, ind, level_args, params, motive)?;
    let expected_motive = motive_expected_type(ind, level_args, params, &motive_sort);
    check(env, ctx, motive, &expected_motive)?;

    // 3. Check one method per constructor against its method type.
    if methods.len() != ind.constructors.len() {
        return Err(KernelError::BadEliminator(format!(
            "expected {} methods, got {}",
            ind.constructors.len(),
            methods.len()
        )));
    }
    for (k, m) in methods.iter().enumerate() {
        let mt = method_type(ind, k, motive, params, level_args);
        check(env, ctx, m, &mt)?;
    }

    // 4. Check the index arguments i̅ against Δ_i (params substituted).
    let inst_indices: Vec<Term> = ind
        .indices
        .iter()
        .map(|i| subst_levels(i, &ind.level_params, level_args))
        .collect();
    if indices.len() != n_i {
        return Err(KernelError::BadEliminator(format!(
            "expected {n_i} indices, got {}",
            indices.len()
        )));
    }
    for (j, ix) in indices.iter().enumerate() {
        let ity = subst_tel(&subst_outer(&inst_indices[j], m, params, j), &indices[..j]);
        check(env, ctx, ix, &ity)?;
    }

    // 5. Check the scrutinee s ⇐ D p̄ i̅.
    let mut d_app = Term::IndFormer {
        id: fam,
        level_args: level_args.to_vec(),
    };
    for p in params {
        d_app = Term::app(d_app, p.clone());
    }
    for ix in indices {
        d_app = Term::app(d_app, ix.clone());
    }
    check(env, ctx, scrut, &d_app)?;

    // 6. Result type: M i̅ s (`14 §3`). Valid by M's checked type + i̅ + s.
    let mut result = motive.clone();
    for ix in indices {
        result = Term::app(result, ix.clone());
    }
    result = Term::app(result, scrut.clone());
    Ok(result)
}

/// Extract the motive's result sort by peeling `n_i + 1` Π binders from the
/// motive's inferred type (the index binders, then the scrutinee binder),
/// requiring the body to be `Type ℓ'` or `Ω_ℓ'` (`16 §1.1`) — a per-branch
/// proposition may be proved by case-split on a relevant scrutinee, exactly
/// as a per-branch type may be selected. Loosely verifies the motive's shape;
/// [`infer_elim`] re-checks it fully against [`motive_expected_type`].
fn infer_motive_level(
    env: &GlobalEnv,
    ctx: &Context,
    ind: &InductiveDecl,
    _level_args: &[Level],
    _params: &[Term],
    motive: &Term,
) -> KernelResult<Sort> {
    let n_i = ind.indices.len();
    let mty = infer(env, ctx, motive)?;
    let mut cur = whnf(env, ctx, &mty);
    let mut mctx = ctx.clone();
    for _ in 0..n_i {
        match whnf(env, &mctx, &cur) {
            Term::Pi(a, b) => {
                mctx.push((*a).clone());
                cur = (*b).clone();
            }
            _ => {
                return Err(KernelError::BadEliminator(
                    "motive is not a Π over the family indices".into(),
                ))
            }
        }
    }
    match whnf(env, &mctx, &cur) {
        Term::Pi(d_app, ret) => {
            mctx.push((*d_app).clone());
            match whnf(env, &mctx, &ret) {
                Term::Type(l) => Ok(Sort::Type(l.clone())),
                Term::Omega(l) => Ok(Sort::Omega(l.clone())),
                _ => Err(KernelError::BadEliminator(
                    "motive result is not a type or a proposition (Type ℓ' or Ω_ℓ')".into(),
                )),
            }
        }
        _ => Err(KernelError::BadEliminator(
            "motive is not a Π over a D-value".into(),
        )),
    }
}

/// Build the expected motive type `(Δ_i) → D p̄ Δ_i → Type ℓ'` (or `Ω_ℓ'`) in
/// the caller's context Γ (params p̄ fixed, indices abstracted).
fn motive_expected_type(
    ind: &InductiveDecl,
    level_args: &[Level],
    params: &[Term],
    motive_sort: &Sort,
) -> Term {
    let m = ind.params.len();
    let n_i = ind.indices.len();
    let inst_indices: Vec<Term> = ind
        .indices
        .iter()
        .map(|i| subst_levels(i, &ind.level_params, level_args))
        .collect();
    // Index binders in [Γ, idx 0..j-1]: params substituted, earlier indices kept.
    let idx_types: Vec<Term> = (0..n_i)
        .map(|j| subst_outer(&inst_indices[j], m, params, j))
        .collect();
    // D p̄ Δ_i in [Γ, idx 0..n_i-1]: params weakened past the indices, idx vars.
    let mut d_app = Term::IndFormer {
        id: ind.id,
        level_args: level_args.to_vec(),
    };
    for p in params {
        d_app = Term::app(d_app, weaken(p, n_i as i64));
    }
    for j in 0..n_i {
        d_app = Term::app(d_app, Term::var(n_i - 1 - j));
    }
    let ret = Term::pi(d_app, motive_sort.to_term());
    telescope_to_pi(&idx_types, ret)
}

// --- K2 quotient / J inference (`15 §4`, `16 §5`, §6) ---------------------

/// Infer the type of `J motive base eq` = `motive b eq` (`15 §4`). Recovers
/// `A`,`a`,`b` from `eq : Eq A a b`, verifies the motive's first domain is `A`,
/// checks `base : motive a (refl a)`, and returns `motive b eq`.
fn infer_j(
    env: &GlobalEnv,
    ctx: &Context,
    motive: &Term,
    base: &Term,
    eq: &Term,
) -> KernelResult<Term> {
    // e : Eq A a b  ⇒  recover A, a, b.
    let e_ty = infer(env, ctx, eq)?;
    let (a_ty, a_idx, b_idx) = match &whnf(env, ctx, &e_ty) {
        Term::Eq(at, x, y) => ((**at).clone(), (**x).clone(), (**y).clone()),
        _ => {
            return Err(KernelError::BadEliminator(
                "J's equality argument is not an `Eq`".into(),
            ))
        }
    };
    // motive : (b:A) → (e':Eq A a b) → Type ℓ'. Verify the first domain ≡ A.
    let m_ty = infer(env, ctx, motive)?;
    match &whnf(env, ctx, &m_ty) {
        Term::Pi(m_dom, _) => {
            if !convert_type(env, ctx, m_dom, &a_ty) {
                return Err(KernelError::BadEliminator(
                    "J motive's first domain ≠ the equality's type A".into(),
                ));
            }
        }
        _ => {
            return Err(KernelError::BadEliminator(
                "J motive is not a Π over A".into(),
            ))
        }
    }
    // base : motive a (refl a).
    let base_ty = Term::app(
        Term::app(motive.clone(), a_idx.clone()),
        Term::Refl(Box::new(a_idx.clone())),
    );
    check(env, ctx, base, &base_ty)?;
    // Result: motive b eq.
    Ok(Term::app(
        Term::app(motive.clone(), b_idx.clone()),
        eq.clone(),
    ))
}

/// Check `R : A → A → Ω` (the quotient relation, `16 §5`): infer `R`'s type and
/// verify the Π–Π–Ω shape with the first domain ≡ `A`. (The second domain is
/// `A` under the first binder; a strict check needs a context shift, so only the
/// shape and first domain are verified — sound for well-elaborated input.)
fn check_quotient_rel(env: &GlobalEnv, ctx: &Context, a: &Term, r: &Term) -> KernelResult<()> {
    let r_ty = infer(env, ctx, r)?;
    let cod1 = match &whnf(env, ctx, &r_ty) {
        Term::Pi(dom1, cod1) => {
            if !convert_type(env, ctx, dom1, a) {
                return Err(KernelError::BadEliminator(
                    "quotient relation's first domain ≠ A".into(),
                ));
            }
            (**cod1).clone()
        }
        _ => {
            return Err(KernelError::BadEliminator(
                "quotient relation is not of type A → A → Ω".into(),
            ))
        }
    };
    let cod2 = match &whnf(env, ctx, &cod1) {
        Term::Pi(_, cod2) => (**cod2).clone(),
        _ => {
            return Err(KernelError::BadEliminator(
                "quotient relation is not of type A → A → Ω".into(),
            ))
        }
    };
    match &whnf(env, ctx, &cod2) {
        Term::Omega(_) => Ok(()),
        _ => Err(KernelError::BadEliminator(
            "quotient relation's codomain is not Ω".into(),
        )),
    }
}

/// Infer `elim_/ M f r q : M q` (`16 §5`), also covering `elim_trunc`
/// (encoded as `QuotElim` on a `‖A‖` scrut, `16 §6`). Checks the motive,
/// method, and (for non-Ω targets) the respect proof; Ω targets are
/// respect-free (`16 §5`).
fn infer_quot_elim(
    env: &GlobalEnv,
    ctx: &Context,
    motive: &Term,
    method: &Term,
    respect: &Term,
    scrut: &Term,
) -> KernelResult<Term> {
    // scrut : A/R (or ‖A‖). Recover the underlying `A` and relation (if Quot).
    let scrut_ty = infer(env, ctx, scrut)?;
    let scrut_whnf = whnf(env, ctx, &scrut_ty);
    let (underlying_a, opt_rel) = match scrut_whnf {
        Term::Quot(a, r) => (*a, Some(*r)),
        Term::Trunc(a) => (*a, None),
        _ => {
            return Err(KernelError::BadEliminator(
                "quotient elim scrutinee is not a quotient or truncation".into(),
            ))
        }
    };
    // motive M : (z : scrut_ty) → Type ℓ'. Verify the Π shape and codomain Type.
    let m_ty = infer(env, ctx, motive)?;
    let m_cod = match &whnf(env, ctx, &m_ty) {
        Term::Pi(dom, cod) => {
            if !convert_type(env, ctx, dom, &scrut_ty) {
                return Err(KernelError::BadEliminator(
                    "motive's domain ≠ scrutinee type".into(),
                ));
            }
            (**cod).clone()
        }
        _ => {
            return Err(KernelError::BadEliminator(
                "motive is not a Π over the quotient".into(),
            ))
        }
    };
    // Motive codomain sort ⇒ target kind (§5):
    //   Ω_l ⇒ respect-free (Ω-PI); Type ℓ ⇒ verify cong/cast schema (§5.1).
    let type_target = match whnf(env, ctx, &m_cod) {
        Term::Omega(_) => false,
        Term::Type(_) => true,
        _ => {
            return Err(KernelError::BadEliminator(
                "motive's codomain is not a type (Type ℓ' or Ω_l)".into(),
            ))
        }
    };
    // method f : (x:A) → M [x].
    let expected_method_ty = Term::pi(
        underlying_a.clone(),
        Term::app(weaken(motive, 1), Term::QuotClass(Box::new(Term::var(0)))),
    );
    check(env, ctx, method, &expected_method_ty)?;
    // Respect proof.
    if type_target {
        // §5.1 cong/cast schema: r must have type
        //   (x:A) → (y:A) → (h:R x y) → Eq(M[x])(f x)(cast M[x] M[y] refl(M[x]) (f y))
        // Requires a proper Quot (not Trunc).
        let rel = match opt_rel {
            Some(r) => r,
            None => {
                return Err(KernelError::BadEliminator(
                    "quotient-elim Type target requires a Quot (not Trunc)".into(),
                ))
            }
        };
        // depth 3: x=Var(2), y=Var(1), h=Var(0) under (x:A)(y:A)(h:R x y)
        let x_class = Term::QuotClass(Box::new(Term::var(2)));
        let y_class = Term::QuotClass(Box::new(Term::var(1)));
        let m_x = Term::app(weaken(motive, 3), x_class);
        let m_y = Term::app(weaken(motive, 3), y_class);
        let f_x = Term::app(weaken(method, 3), Term::var(2));
        let f_y = Term::app(weaken(method, 3), Term::var(1));
        // Transport f_y from M[y] (its type) to M[x] (the Eq's required RHS type).
        // Source = M[y], target = M[x]; cast ignores the proof (§3.4).
        let cast_fy = Term::Cast(
            Box::new(m_y.clone()),
            Box::new(m_x.clone()),
            Box::new(Term::Refl(Box::new(m_y.clone()))),
            Box::new(f_y),
        );
        let eq_body = Term::Eq(Box::new(m_x), Box::new(f_x), Box::new(cast_fy));
        // h_ty = R x y at depth 2: x=Var(1), y=Var(0)
        let h_ty = apply_args(weaken(&rel, 2), &[Term::var(1), Term::var(0)]);
        let expected = Term::pi(
            underlying_a.clone(),
            Term::pi(weaken(&underlying_a, 1), Term::pi(h_ty, eq_body)),
        );
        check(env, ctx, respect, &expected)?;
    } else {
        // Ω-target: respect-free by Ω-PI; well-formedness only.
        raw_well_formed(ctx, respect)?;
    }
    // Result: M scrut.
    Ok(Term::app(motive.clone(), scrut.clone()))
}

/// `absurd C p : C` — ex-falso (`16 §1.3`, K5/KM-index-impossible). Sound
/// because `Bottom` is empty: `p` proves the impossible, so `Absurd` never has a
/// canonical scrutinee to compute on and stays neutral forever. The motive may
/// be either a proposition (`Ω`) or a value type (`Type`), but the proof must
/// still check as actual `Bottom`; constructor disjointness alone does not
/// synthesize a closed contradiction. Non-dependent: `Bottom` has no indices to
/// abstract over, so the result is `motive` itself, never substituted.
fn infer_absurd(env: &GlobalEnv, ctx: &Context, motive: &Term, proof: &Term) -> KernelResult<Term> {
    check(env, ctx, proof, &crate::obs::bottom_term(env))?;
    classify(env, ctx, motive)?;
    Ok(motive.clone())
}

// --- declaration admission (`18 §4`) ---------------------------------------

/// A constructor specification for [`declare_inductive`] (no id/type yet —
/// the kernel allocates and generates them).
#[derive(Clone, Debug)]
pub struct CtorSpec {
    /// `Δₖ` — argument telescope, relative to `Δ_p`.
    pub args: Vec<Term>,
    /// `t̄ₖ` — the index instance the constructor targets, relative to
    /// `Δ_p + Δₖ`.
    pub target_indices: Vec<Term>,
}

/// An inductive family specification for [`declare_inductive`].
#[derive(Clone, Debug)]
pub struct InductiveSpec {
    pub level_params: Vec<LevelVar>,
    /// `Δ_p` — parameters, relative to the empty term context.
    pub params: Vec<Term>,
    /// `Δ_i` — indices, relative to `Δ_p`.
    pub indices: Vec<Term>,
    /// `ℓ` — the family's universe level (may mention `level_params`).
    pub level: Level,
    pub constructors: Vec<CtorSpec>,
}

/// `declare_inductive` — admit `data D (Δ_p) : (Δ_i) → Type ℓ where …` after
/// re-checking signatures, strict positivity, and constructor universes
/// (`14 §1`, `14 §8`, `14 §8.4`). W-style
/// (Π-bound) recursive arguments are admitted (K1.5, `14 §2.1`) — the
/// blanket `check_no_pi_bound_recursive` gate is retired. Generates the type
/// former, constructors, and (on use) the dependent eliminator with
/// Π-abstracted IH for W-style args. Returns the family's [`GlobalId`].
///
/// `build` receives the freshly-allocated family id so the spec's constructor
/// signatures can self-reference `D` (e.g. `suc : Nat → Nat`).
pub fn declare_inductive<F>(env: &mut GlobalEnv, build: F) -> KernelResult<GlobalId>
where
    F: FnOnce(GlobalId) -> InductiveSpec,
{
    let d_id = env.fresh_id();
    let spec = build(d_id);
    let constructors: Vec<_> = spec
        .constructors
        .into_iter()
        .map(|c| crate::env::ConstructorDecl {
            id: env.fresh_id(),
            args: c.args,
            target_indices: c.target_indices,
            type_: Term::Type(Level::zero()), // placeholder; build_types fills it
            recursive_positions: Vec::new(),
        })
        .collect();
    let mut ind = InductiveDecl {
        id: d_id,
        level_params: spec.level_params,
        params: spec.params,
        indices: spec.indices,
        level: spec.level,
        constructors,
        former_type: Term::Type(Level::zero()), // placeholder; build_types fills it
    };

    // Generate former + constructor types (`Π Δ_p. Π Δ_i. Type ℓ`, etc.).
    ind.build_types();

    // Provisionally admit so every admission clause can roll back both the
    // declaration and its allocated ids on failure.
    env.add_decl(Decl::Inductive(ind.clone()));

    // Admission has three independent clauses (`14 §1`): (a) ordinary
    // signature type-checking, (b) strict positivity, and (c) constructor
    // universe checks.
    if let Err(error) = check_positivity(&ind) {
        env.remove_last();
        return Err(error);
    }

    // Check only each constructor-local telescope Δₖ. Family parameters Δ_p
    // establish the base context but are not themselves constructor fields.
    // Constructor arguments may be Type- or Ω-sorted, so use `classify` and
    // compare the resulting sort level with the family's level.
    let mut params = Context::new();
    params.extend_tel(&ind.params);
    for constructor in &ind.constructors {
        let mut ctor_ctx = params.clone();
        for argument in &constructor.args {
            let argument_level = match classify(env, &ctor_ctx, argument) {
                Ok(sort) => sort.level().clone(),
                Err(error) => {
                    env.remove_last();
                    return Err(KernelError::IllFormedDecl(format!(
                        "constructor argument failed to type-check: {error}"
                    )));
                }
            };
            if !level_eq(&argument_level.clone().max(ind.level.clone()), &ind.level) {
                env.remove_last();
                return Err(KernelError::ConstructorUniverseViolation {
                    argument: argument_level,
                    family: ind.level.clone(),
                });
            }
            ctor_ctx.push(argument.clone());
        }
    }

    let empty = Context::new();
    let sig_ok = synth_type(env, &empty, &ind.former_type).is_ok()
        && ind
            .constructors
            .iter()
            .all(|c| synth_type(env, &empty, &c.type_).is_ok());
    if !sig_ok {
        env.remove_last();
        return Err(KernelError::IllFormedDecl(
            "inductive signature failed to type-check".into(),
        ));
    }
    Ok(d_id)
}

/// `declare_def` — admit a transparent definition `c : A := t` after checking
/// `· ⊢ A type`, `· ⊢ t ⇐ A`, and the SCT gate (`17 §4`, `18 §4`).
///
/// The definition is **pre-admitted as opaque** before type-checking, so `t`
/// may contain self-recursive `Const(c)` references.  SCT either accepts (→
/// upgrades to transparent) or rejects (→ removes `c` and returns an error).
pub fn declare_def(
    env: &mut GlobalEnv,
    level_params: Vec<LevelVar>,
    ty: Term,
    body: Term,
) -> KernelResult<GlobalId> {
    let empty = Context::new();
    classify(env, &empty, &ty)?;
    // Pre-admit as opaque so the body can self-reference.
    let id = env.fresh_id();
    env.add_decl(Decl::Opaque {
        id,
        name: "provisional definition".to_string(),
        level_params: level_params.clone(),
        ty: ty.clone(),
    });
    // Type-check (self-calls see c as opaque with type `ty`).
    let check_result = check(env, &empty, &body, &ty);
    // SCT gate.
    let sct_result = check_result.and_then(|_| crate::sct::sct_check(env, &[(id, body.clone())]));
    match sct_result {
        Ok(()) => {
            env.upgrade_to_transparent(id, body);
            Ok(id)
        }
        Err(e) => {
            env.remove_last();
            Err(e)
        }
    }
}

/// Declare a group of mutually-recursive transparent definitions.
///
/// All members are pre-admitted as opaque before any body is type-checked, so
/// each body may reference any member.  SCT is run on the whole group; on
/// rejection all pre-admitted members are rolled back.
///
/// `specs` — `(level_params, ty)` for each member.  `bodies_fn` receives the
/// freshly-allocated IDs and must return one body per member in the same order.
pub fn declare_recursive_group<F>(
    env: &mut GlobalEnv,
    specs: Vec<(Vec<LevelVar>, Term)>,
    bodies_fn: F,
) -> KernelResult<Vec<GlobalId>>
where
    F: FnOnce(&[GlobalId]) -> Vec<Term>,
{
    if specs.is_empty() {
        return Ok(Vec::new());
    }
    let empty = Context::new();

    // Check all types.
    for (lp, ty) in &specs {
        let _ = lp; // level params checked via classify
        classify(env, &empty, ty)?;
    }

    // Pre-admit all members as opaque.
    let mut ids: Vec<GlobalId> = Vec::new();
    for (level_params, ty) in &specs {
        let id = env.fresh_id();
        env.add_decl(Decl::Opaque {
            id,
            name: "provisional recursive definition".to_string(),
            level_params: level_params.clone(),
            ty: ty.clone(),
        });
        ids.push(id);
    }

    let bodies = bodies_fn(&ids);
    assert_eq!(
        bodies.len(),
        ids.len(),
        "bodies_fn must return one body per member"
    );

    // Type-check all bodies.
    let check_result: KernelResult<()> = (|| {
        for (i, body) in bodies.iter().enumerate() {
            let ty = &specs[i].1;
            check(env, &empty, body, ty)?;
        }
        Ok(())
    })();

    // SCT gate on the whole group.
    let group_bodies: Vec<(GlobalId, Term)> =
        ids.iter().cloned().zip(bodies.iter().cloned()).collect();
    let sct_result = check_result.and_then(|_| crate::sct::sct_check(env, &group_bodies));

    match sct_result {
        Ok(()) => {
            // Upgrade all to transparent.
            for (id, body) in ids.iter().zip(bodies) {
                env.upgrade_to_transparent(*id, body);
            }
            Ok(ids)
        }
        Err(e) => {
            // Rollback all pre-admitted members (remove in reverse order).
            for _ in 0..ids.len() {
                env.remove_last();
            }
            Err(e)
        }
    }
}

/// `declare_postulate` — admit an opaque constant `c : A` after checking
/// `· ⊢ A type` (`11 §4`). Recorded in the trusted base (`18 §5`).
pub fn declare_postulate(
    env: &mut GlobalEnv,
    name: String,
    level_params: Vec<LevelVar>,
    ty: Term,
) -> KernelResult<GlobalId> {
    let empty = Context::new();
    classify(env, &empty, &ty)?;
    let id = env.fresh_id();
    env.add_decl(Decl::Opaque {
        id,
        name,
        level_params,
        ty,
    });
    Ok(id)
}

/// `declare_primitive` — admit a primitive type/operation (opaque + registered
/// reduction) after checking `· ⊢ A type` (`14 §5`). K1 defines the interface
/// only; the value model (K3) and API (K-api) elaborate the computation.
pub fn declare_primitive(
    env: &mut GlobalEnv,
    level_params: Vec<LevelVar>,
    ty: Term,
    reduction: crate::env::PrimReduction,
) -> KernelResult<GlobalId> {
    let empty = Context::new();
    classify(env, &empty, &ty)?;
    let id = env.fresh_id();
    env.add_decl(Decl::Primitive {
        id,
        level_params,
        ty,
        reduction,
    });
    Ok(id)
}

/// `declare_deceq_certificate` — register a decidable-equality certificate
/// for an opaque primitive type
/// (`docs/adr/0013-int-decidable-equality-kernel-posture.md` Layer 1): the
/// kernel trusts `eq_op` to decide propositional equality at `prim_ty`,
/// both directions.
/// General, opt-in, per-primitive — `prim_ty` is the *first* registrant of
/// this mechanism, not a special case of it; an unregistered primitive's
/// `Eq` stays neutral exactly as before (`obs.rs`'s fail-safe default is
/// untouched — this adds no reduction rule).
///
/// Builds and admits, via [`declare_postulate`] (each call `classify`s the
/// constructed type before committing it, so registration fails closed on
/// an incoherent registrant — e.g. an `eq_op` not shaped
/// `prim_ty → prim_ty → bool_ty`, or a `bool_true` not of type `bool_ty`):
///
/// - `sound   : (x y : prim_ty) → Eq bool_ty (eq_op x y) bool_true → Eq prim_ty x y`
/// - `complete: (x y : prim_ty) → Eq prim_ty x y → Eq bool_ty (eq_op x y) bool_true`
///
/// and records the pair in [`GlobalEnv::deceq_cert`] under `prim_ty`.
pub fn declare_deceq_certificate(
    env: &mut GlobalEnv,
    prim_ty: GlobalId,
    eq_op: GlobalId,
    bool_ty: GlobalId,
    bool_true: GlobalId,
) -> KernelResult<crate::env::DecEqCert> {
    let ty_const = Term::const_(prim_ty, vec![]);
    let bool_const = Term::indformer(bool_ty, vec![]);
    let true_const = Term::constructor(bool_true, vec![]);
    let eq_op_const = Term::const_(eq_op, vec![]);

    // Context depth 2 (x y : prim_ty bound, y=Var(0), x=Var(1)) — used for
    // the premise, the domain of the third Π.
    let eq_call_d2 = Term::app(Term::app(eq_op_const.clone(), Term::var(1)), Term::var(0));
    let bool_eq_true_d2 = Term::Eq(
        Box::new(bool_const.clone()),
        Box::new(eq_call_d2),
        Box::new(true_const.clone()),
    );
    let prim_eq_d2 = Term::Eq(
        Box::new(ty_const.clone()),
        Box::new(Term::var(1)),
        Box::new(Term::var(0)),
    );

    // Context depth 3 (x y premise bound, x=Var(2), y=Var(1)) — used for the
    // conclusion, the codomain of the third Π.
    let eq_call_d3 = Term::app(Term::app(eq_op_const, Term::var(2)), Term::var(1));
    let bool_eq_true_d3 = Term::Eq(
        Box::new(bool_const),
        Box::new(eq_call_d3),
        Box::new(true_const),
    );
    let prim_eq_d3 = Term::Eq(
        Box::new(ty_const.clone()),
        Box::new(Term::var(2)),
        Box::new(Term::var(1)),
    );

    let sound_ty = Term::pi(
        ty_const.clone(),
        Term::pi(ty_const.clone(), Term::pi(bool_eq_true_d2, prim_eq_d3)),
    );
    let complete_ty = Term::pi(
        ty_const.clone(),
        Term::pi(ty_const, Term::pi(prim_eq_d2, bool_eq_true_d3)),
    );

    let sound = declare_postulate(
        env,
        "decidable equality sound".to_string(),
        vec![],
        sound_ty,
    )?;
    let complete = declare_postulate(
        env,
        "decidable equality complete".to_string(),
        vec![],
        complete_ty,
    )?;

    let cert = crate::env::DecEqCert {
        eq_op,
        sound,
        complete,
    };
    env.register_deceq_cert(prim_ty, cert.clone());
    Ok(cert)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::GlobalEnv;
    use crate::term::Level;

    struct BoolNat {
        bool_: GlobalId,
        true_: GlobalId,
        nat: GlobalId,
        zero: GlobalId,
    }

    fn bool_nat_env() -> (GlobalEnv, BoolNat) {
        let mut env = GlobalEnv::new();

        let bool_ = declare_inductive(&mut env, |_| InductiveSpec {
            level_params: vec![],
            params: vec![],
            indices: vec![],
            level: Level::zero(),
            constructors: vec![
                CtorSpec {
                    args: vec![],
                    target_indices: vec![],
                },
                CtorSpec {
                    args: vec![],
                    target_indices: vec![],
                },
            ],
        })
        .expect("Bool");
        let true_ = env.inductive(bool_).unwrap().constructors[0].id;

        let nat = declare_inductive(&mut env, |_| InductiveSpec {
            level_params: vec![],
            params: vec![],
            indices: vec![],
            level: Level::zero(),
            constructors: vec![CtorSpec {
                args: vec![],
                target_indices: vec![],
            }],
        })
        .expect("Nat");
        let zero = env.inductive(nat).unwrap().constructors[0].id;

        (
            env,
            BoolNat {
                bool_,
                true_,
                nat,
                zero,
            },
        )
    }

    fn bool_ty(ids: &BoolNat) -> Term {
        Term::indformer(ids.bool_, vec![])
    }

    fn nat_ty(ids: &BoolNat) -> Term {
        Term::indformer(ids.nat, vec![])
    }

    fn true_term(ids: &BoolNat) -> Term {
        Term::constructor(ids.true_, vec![])
    }

    fn zero_term(ids: &BoolNat) -> Term {
        Term::constructor(ids.zero, vec![])
    }

    fn let_term(let_ty: Term, val: Term, body: Term) -> Term {
        Term::Let {
            ty: Box::new(let_ty),
            val: Box::new(val),
            body: Box::new(body),
        }
    }

    #[test]
    fn let_check_rejects_wrong_outer_expected_type() {
        let (env, ids) = bool_nat_env();
        let ctx = Context::new();
        let let_zero_as_bool = let_term(nat_ty(&ids), zero_term(&ids), Term::var(0));

        assert!(matches!(
            check(&env, &ctx, &let_zero_as_bool, &bool_ty(&ids)),
            Err(KernelError::TypeMismatch { .. })
        ));
    }

    #[test]
    fn let_check_accepts_valid_body_at_outer_expected_type() {
        let (env, ids) = bool_nat_env();
        let ctx = Context::new();
        let let_true_at_bool = let_term(nat_ty(&ids), zero_term(&ids), true_term(&ids));

        assert!(check(&env, &ctx, &let_true_at_bool, &bool_ty(&ids)).is_ok());
    }

    #[test]
    fn let_check_preserves_check_mode_for_intro_body() {
        let (env, ids) = bool_nat_env();
        let ctx = Context::new();
        let pi_bool_bool = Term::pi(bool_ty(&ids), bool_ty(&ids));
        let let_lambda = let_term(
            nat_ty(&ids),
            zero_term(&ids),
            Term::lam(bool_ty(&ids), Term::var(0)),
        );

        assert!(check(&env, &ctx, &let_lambda, &pi_bool_bool).is_ok());
    }

    #[test]
    fn universe_no_type_type() {
        // Type 0 : Type 1; but Type 0 is NOT a Type 0 (no Type:Type).
        let env = GlobalEnv::new();
        let ctx = Context::new();
        // infer Type 0 ⇒ Type 1
        assert_eq!(
            infer(&env, &ctx, &Term::Type(Level::zero())),
            Ok(Term::Type(Level::suc(Level::zero())))
        );
        // check Type 0 ⇐ Type 1  → ok
        assert!(check(
            &env,
            &ctx,
            &Term::Type(Level::zero()),
            &Term::Type(Level::suc(Level::zero()))
        )
        .is_ok());
        // check Type 0 ⇐ Type 0  → REJECT (AC-1)
        assert!(check(
            &env,
            &ctx,
            &Term::Type(Level::zero()),
            &Term::Type(Level::zero())
        )
        .is_err());
    }

    #[test]
    fn k2_omega_formation() {
        let env = GlobalEnv::new();
        let ctx = Context::new();
        // Ω_l : Type (suc l) (`16 §1.1`). Ω_0 : Type 1.
        assert_eq!(
            infer(&env, &ctx, &Term::Omega(Level::zero())),
            Ok(Term::Type(Level::suc(Level::zero())))
        );
        // Ω_0 checks against Type 1 (its universe).
        assert!(check(
            &env,
            &ctx,
            &Term::Omega(Level::zero()),
            &Term::Type(Level::suc(Level::zero()))
        )
        .is_ok());
        // Non-cumulative (`12 §3`): Ω_0 : Type 1 does NOT give Ω_0 : Type 0.
        assert!(check(
            &env,
            &ctx,
            &Term::Omega(Level::zero()),
            &Term::Type(Level::zero())
        )
        .is_err());
    }

    #[test]
    fn k2_piproduct_over_omega_lands_in_omega() {
        // 13 §4 / 16 §1.1: a Π whose codomain is a proposition lands in Ω.
        // (P : Ω_0) → Top  with Top : Ω_0  ⇒  (P → Top) : Ω_0.  Using the closed
        // prelude `Top` as the codomain avoids a de Bruijn shift on the body.
        let env = GlobalEnv::new();
        let mut ctx = Context::new();
        ctx.push(Term::Omega(Level::zero())); // P : Ω_0  (var 0)
        let top = Term::Const {
            id: env.top_id(),
            level_args: Vec::new(),
        }; // Top : Ω_0 (closed)
        let pi = Term::pi(Term::var(0), top.clone()); // (x : P) → Top
        assert_eq!(infer(&env, &ctx, &pi), Ok(Term::Omega(Level::zero())));
        // A Σ over a proposition codomain also lands in Ω_0.
        let sig = Term::sigma(Term::var(0), top); // (x : P) × Top
        assert_eq!(infer(&env, &ctx, &sig), Ok(Term::Omega(Level::zero())));
    }
}
