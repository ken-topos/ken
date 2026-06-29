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

use crate::conv::{convert_type, whnf};
use crate::env::{telescope_to_pi, Context, Decl, GlobalEnv, InductiveDecl};
use crate::error::{KernelError, KernelResult};
use crate::inductive::{check_no_pi_bound_recursive, check_positivity, method_type};
use crate::subst::{subst0, subst_levels, subst_outer, subst_tel, weaken};
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
            scrut,
        } => {
            raw_wf(ctx, motive, offset)?;
            raw_wf(ctx, method, offset)?;
            raw_wf(ctx, scrut, offset)
        }
        // Closed in Σ: no free term variables (levels are not de Bruijn).
        Term::Type(_)
        | Term::Omega
        | Term::Const { .. }
        | Term::IndFormer { .. }
        | Term::Constructor { .. } => Ok(()),
    }
}

/// Raw well-formedness check (public, for the elaborator precondition).
pub fn raw_well_formed(ctx: &Context, t: &Term) -> KernelResult<()> {
    raw_wf(ctx, t, 0)
}

// --- type synthesis: `Γ ⊢ A type` ⇒ its level -----------------------------

/// Check `Γ ⊢ A type` and return its universe level (`Γ ⊢ A : Type ℓ`).
fn synth_type(env: &GlobalEnv, ctx: &Context, a: &Term) -> KernelResult<Level> {
    let ty = infer(env, ctx, a)?;
    match whnf(env, ctx, &ty) {
        Term::Type(l) => Ok(l),
        other => Err(KernelError::TypeMismatch {
            expected: Box::new(Term::Type(Level::Var(LevelVar(0)))), // a type
            found: Box::new(other),
        }),
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
            let l1 = synth_type(env, ctx, a)?;
            let mut ctx2 = ctx.clone();
            ctx2.push((**a).clone());
            let l2 = synth_type(env, &ctx2, b)?;
            Ok(Term::Type(l1.max(l2).normalize()))
        }
        Term::Sigma(a, b) => {
            let l1 = synth_type(env, ctx, a)?;
            let mut ctx2 = ctx.clone();
            ctx2.push((**a).clone());
            let l2 = synth_type(env, &ctx2, b)?;
            Ok(Term::Type(l1.max(l2).normalize()))
        }
        Term::Type(l) => Ok(Term::Type(l.clone().suc())), // (U-Type): Type ℓ : Type (suc ℓ) (`12 §1`)
        Term::Ascript(t, a) => {
            synth_type(env, ctx, a)?;
            check(env, ctx, t, a)?;
            Ok((**a).clone())
        }
        Term::Let { ty, val, body } => {
            synth_type(env, ctx, ty)?;
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
        Term::Lam { .. } | Term::Pair { .. } => Err(KernelError::Msg(
            "cannot infer a λ or pair without an expected type (use ascription)".into(),
        )),
        t if t.is_k2_reserved() => Err(KernelError::K2ReservedFormer),
        _ => Err(KernelError::Msg(format!("cannot infer {:?}", t))),
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
                synth_type(env, ctx, a)?;
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
            synth_type(env, ctx, a)?;
            check(env, ctx, t, a)?;
            if !convert_type(env, ctx, a, ty) {
                return Err(KernelError::TypeMismatch {
                    expected: Box::new(ty.clone()),
                    found: Box::new((**a).clone()),
                });
            }
            Ok(())
        }
        Term::Let { ty, val, body } => {
            synth_type(env, ctx, ty)?;
            check(env, ctx, val, ty)?;
            check(env, ctx, &subst0(body, val), ty)
        }
        t if t.is_k2_reserved() => Err(KernelError::K2ReservedFormer),
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

    // 2. Verify the motive M : (Δ_i) → D p̄ Δ_i → Type ℓ' (extract ℓ', then
    //    re-check against the fully-built expected motive type).
    let motive_level = infer_motive_level(env, ctx, ind, level_args, params, motive)?;
    let expected_motive = motive_expected_type(ind, level_args, params, &motive_level);
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

/// Extract the motive's result level ℓ' by peeling `n_i + 1` Π binders from
/// the motive's inferred type (the index binders, then the scrutinee binder),
/// requiring the body to be `Type ℓ'`. Loosely verifies the motive's shape;
/// [`infer_elim`] re-checks it fully against [`motive_expected_type`].
fn infer_motive_level(
    env: &GlobalEnv,
    ctx: &Context,
    ind: &InductiveDecl,
    _level_args: &[Level],
    _params: &[Term],
    motive: &Term,
) -> KernelResult<Level> {
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
                Term::Type(l) => Ok(l.clone()),
                _ => Err(KernelError::BadEliminator(
                    "motive result is not a type (Type ℓ')".into(),
                )),
            }
        }
        _ => Err(KernelError::BadEliminator(
            "motive is not a Π over a D-value".into(),
        )),
    }
}

/// Build the expected motive type `(Δ_i) → D p̄ Δ_i → Type ℓ'` in the caller's
/// context Γ (params p̄ fixed, indices abstracted).
fn motive_expected_type(
    ind: &InductiveDecl,
    level_args: &[Level],
    params: &[Term],
    motive_level: &Level,
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
    let ret = Term::pi(d_app, Term::Type(motive_level.clone()));
    telescope_to_pi(&idx_types, ret)
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
/// re-checking signatures, strict positivity (`14 §8`), and the K1
/// Π-bound-recursive boundary. Generates the type former, constructors, and
/// (on use) the dependent eliminator. Returns the family's [`GlobalId`].
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

    // Positivity (and the K1 Π-bound-recursive boundary) before any types.
    check_positivity(&ind)?;
    check_no_pi_bound_recursive(&ind)?;

    // Generate former + constructor types (`Π Δ_p. Π Δ_i. Type ℓ`, etc.).
    ind.build_types();

    // Provisionally admit so signature checking can reference D / its
    // constructors; withdraw on any failure.
    env.add_decl(Decl::Inductive(ind.clone()));
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
/// `· ⊢ A type` and `· ⊢ t ⇐ A` (`11 §4`). Non-recursive in K1 (acyclic env);
/// general recursive δ is K2c. Returns `c`'s [`GlobalId`].
pub fn declare_def(
    env: &mut GlobalEnv,
    level_params: Vec<LevelVar>,
    ty: Term,
    body: Term,
) -> KernelResult<GlobalId> {
    let empty = Context::new();
    synth_type(env, &empty, &ty)?;
    check(env, &empty, &body, &ty)?;
    let id = env.fresh_id();
    env.add_decl(Decl::Transparent {
        id,
        level_params,
        ty,
        body,
    });
    Ok(id)
}

/// `declare_postulate` — admit an opaque constant `c : A` after checking
/// `· ⊢ A type` (`11 §4`). Recorded in the trusted base (`18 §5`).
pub fn declare_postulate(
    env: &mut GlobalEnv,
    level_params: Vec<LevelVar>,
    ty: Term,
) -> KernelResult<GlobalId> {
    let empty = Context::new();
    synth_type(env, &empty, &ty)?;
    let id = env.fresh_id();
    env.add_decl(Decl::Opaque {
        id,
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
    synth_type(env, &empty, &ty)?;
    let id = env.fresh_id();
    env.add_decl(Decl::Primitive {
        id,
        level_params,
        ty,
        reduction,
    });
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::GlobalEnv;
    use crate::term::Level;

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
    fn k2_former_rejected() {
        let env = GlobalEnv::new();
        let ctx = Context::new();
        // Ω is reserved but check/infer reject it in K1.
        assert!(matches!(
            infer(&env, &ctx, &Term::Omega),
            Err(KernelError::K2ReservedFormer)
        ));
        assert!(matches!(
            check(&env, &ctx, &Term::Omega, &Term::Type(Level::zero())),
            Err(KernelError::K2ReservedFormer)
        ));
    }
}
