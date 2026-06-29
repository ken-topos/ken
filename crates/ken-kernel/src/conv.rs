//! K1 conversion — weak-head reduction, normalization, and definitional
//! equality (`13-pi-sigma.md §6`).
//!
//! K1 builds only the conversion its own rules require: α (de Bruijn syntactic
//! identity), β/Σ-β/ι/δ reduction, and **type-directed η** for Π and Σ. The
//! full decidable conversion (lazy-WHNF NbE, `Eq`/`cast` equations, Ω proof
//! irrelevance, SCT-gated δ) is **K2c** (`17`). [`convert`] is the standalone
//! entry point the rest of K1 calls and that K2c replaces, body-only, without
//! changing the signature (`13 §6.3`).
//!
//! Termination on the K1 fragment (`14 §9.2`): β strictly decreases size; η
//! descends on the (finite) type; ι descends on structurally smaller
//! scrutinees; δ is acyclic (non-recursive transparent defs only, `11 §4`).

use crate::env::{Context, GlobalEnv};
use crate::inductive::{iota_reduct, peel_app};
use crate::subst::{subst0, subst_levels, weaken};
use crate::term::{Level, Term};

/// Decidable level equality (`12 §1`, §6.1) — the semilattice normal form.
pub fn level_eq(a: &Level, b: &Level) -> bool {
    a.equiv(b)
}

/// Equality of level-argument lists (polymorphic uses agree on instantiation).
fn level_args_eq(a: &[Level], b: &[Level]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| level_eq(x, y))
}

/// Unfold a transparent constant `c` to its body with `level_args`
/// instantiated (δ-reduction, `11 §4`). Returns `None` if `c` is not
/// transparent (opaque/primitive/inductive — no δ).
fn unfold_const(env: &GlobalEnv, id: crate::term::GlobalId, level_args: &[Level]) -> Option<Term> {
    let (params, body) = env.transparent_body(id)?;
    Some(subst_levels(&body, &params, level_args))
}

/// Weak-head normal form: reduce head redexes (β, δ, Σ-β, ι, let, ascription)
/// until the head is not a redex. Infallible — an ι arity mismatch leaves the
/// eliminator stuck (neutral), which is sound (`14 §7.6`).
///
/// `ctx` is threaded for the K2c NbE replacement (which evaluates against a
/// context); K1's head reduction does not consult it, hence the allow.
#[allow(clippy::only_used_in_recursion)]
pub fn whnf(env: &GlobalEnv, ctx: &Context, t: &Term) -> Term {
    let mut cur = t.clone();
    loop {
        match &cur {
            Term::App(f, a) => {
                let f_w = whnf(env, ctx, f);
                match &f_w {
                    Term::Lam(_, body) => {
                        cur = subst0(body, a);
                        continue;
                    }
                    Term::Const { id, level_args } if env.transparent_body(*id).is_some() => {
                        if let Some(body) = unfold_const(env, *id, level_args) {
                            cur = Term::app(body, (**a).clone());
                            continue;
                        }
                        return Term::app(f_w, (**a).clone());
                    }
                    _ => return Term::app(f_w, (**a).clone()), // stuck neutral application
                }
            }
            Term::Proj1(p) => {
                let p_w = whnf(env, ctx, p);
                match &p_w {
                    Term::Pair(a, _) => {
                        cur = (**a).clone();
                        continue;
                    }
                    Term::Const { id, level_args } if env.transparent_body(*id).is_some() => {
                        if let Some(body) = unfold_const(env, *id, level_args) {
                            cur = Term::proj1(body);
                            continue;
                        }
                        return Term::proj1(p_w);
                    }
                    _ => return Term::proj1(p_w),
                }
            }
            Term::Proj2(p) => {
                let p_w = whnf(env, ctx, p);
                match &p_w {
                    Term::Pair(_, b) => {
                        cur = (**b).clone();
                        continue;
                    }
                    Term::Const { id, level_args } if env.transparent_body(*id).is_some() => {
                        if let Some(body) = unfold_const(env, *id, level_args) {
                            cur = Term::proj2(body);
                            continue;
                        }
                        return Term::proj2(p_w);
                    }
                    _ => return Term::proj2(p_w),
                }
            }
            Term::Elim {
                fam,
                level_args,
                params,
                motive,
                methods,
                indices,
                scrut,
            } => {
                let s_w = whnf(env, ctx, scrut);
                let (head, all_args) = peel_app(&s_w);
                if let Term::Constructor { id, .. } = head {
                    if let Some((ind, k)) = env.constructor(id) {
                        if ind.id == *fam {
                            if let Ok(reduct) =
                                iota_reduct(ind, k, level_args, params, motive, methods, &all_args)
                            {
                                cur = reduct;
                                continue;
                            }
                        }
                    }
                }
                // Stuck eliminator (neutral): rebuild with the whnf'd scrutinee
                // (`14 §7.6`). Indices don't gate ι firing (`14 §7.2`).
                return Term::Elim {
                    fam: *fam,
                    level_args: level_args.clone(),
                    params: params.clone(),
                    motive: motive.clone(),
                    methods: methods.clone(),
                    indices: indices.clone(),
                    scrut: Box::new(s_w),
                };
            }
            Term::Const { id, level_args } if env.transparent_body(*id).is_some() => {
                if let Some(body) = unfold_const(env, *id, level_args) {
                    cur = body;
                    continue;
                }
                return cur;
            }
            Term::Let { body, val, .. } => {
                cur = subst0(body, val);
                continue;
            }
            Term::Ascript(t, _) => {
                cur = (**t).clone();
                continue;
            }
            _ => return cur, // already in weak-head normal form
        }
    }
}

/// Full normal form: whnf, then normalize the sub-terms (recursing under
/// binders). Used by the API surface and by tests; K1 conversion uses
/// [`convert`] (whnf + type-directed η), but `normalize` realises the
/// "reduce to normal form" half of `13 §6.2` for inspection.
pub fn normalize(env: &GlobalEnv, ctx: &Context, t: &Term) -> Term {
    let h = whnf(env, ctx, t);
    match &h {
        Term::Pi(a, b) => {
            let a_n = normalize(env, ctx, a);
            let mut ctx2 = ctx.clone();
            ctx2.push((**a).clone());
            Term::pi(a_n, normalize(env, &ctx2, b))
        }
        Term::Lam(a, body) => {
            let a_n = normalize(env, ctx, a);
            let mut ctx2 = ctx.clone();
            ctx2.push((**a).clone());
            Term::lam(a_n, normalize(env, &ctx2, body))
        }
        Term::Sigma(a, b) => {
            let a_n = normalize(env, ctx, a);
            let mut ctx2 = ctx.clone();
            ctx2.push((**a).clone());
            Term::sigma(a_n, normalize(env, &ctx2, b))
        }
        Term::Pair(a, b) => Term::pair(normalize(env, ctx, a), normalize(env, ctx, b)),
        Term::App(f, a) => Term::app(normalize(env, ctx, f), normalize(env, ctx, a)),
        Term::Proj1(p) => Term::proj1(normalize(env, ctx, p)),
        Term::Proj2(p) => Term::proj2(normalize(env, ctx, p)),
        Term::Elim {
            fam,
            level_args,
            params,
            motive,
            methods,
            indices,
            scrut,
        } => Term::Elim {
            fam: *fam,
            level_args: level_args.clone(),
            params: params.iter().map(|p| normalize(env, ctx, p)).collect(),
            motive: Box::new(normalize(env, ctx, motive)),
            methods: methods.iter().map(|m| normalize(env, ctx, m)).collect(),
            indices: indices.iter().map(|i| normalize(env, ctx, i)).collect(),
            scrut: Box::new(normalize(env, ctx, scrut)),
        },
        Term::Eq(a, t, u) => Term::Eq(
            Box::new(normalize(env, ctx, a)),
            Box::new(normalize(env, ctx, t)),
            Box::new(normalize(env, ctx, u)),
        ),
        Term::Cast(a, b, e, t) => Term::Cast(
            Box::new(normalize(env, ctx, a)),
            Box::new(normalize(env, ctx, b)),
            Box::new(normalize(env, ctx, e)),
            Box::new(normalize(env, ctx, t)),
        ),
        Term::J(m, d, e) => Term::J(
            Box::new(normalize(env, ctx, m)),
            Box::new(normalize(env, ctx, d)),
            Box::new(normalize(env, ctx, e)),
        ),
        Term::Quot(a, r) => Term::Quot(
            Box::new(normalize(env, ctx, a)),
            Box::new(normalize(env, ctx, r)),
        ),
        Term::QuotClass(t) => Term::QuotClass(Box::new(normalize(env, ctx, t))),
        Term::Trunc(a) => Term::Trunc(Box::new(normalize(env, ctx, a))),
        Term::TruncProj(t) => Term::TruncProj(Box::new(normalize(env, ctx, t))),
        Term::Refl(t) => Term::Refl(Box::new(normalize(env, ctx, t))),
        Term::QuotElim {
            motive,
            method,
            scrut,
        } => Term::QuotElim {
            motive: Box::new(normalize(env, ctx, motive)),
            method: Box::new(normalize(env, ctx, method)),
            scrut: Box::new(normalize(env, ctx, scrut)),
        },
        Term::Let { ty: _, val, body } => {
            // let reduces to body[val/x] before normalizing (it is a redex).
            normalize(env, ctx, &subst0(body, val))
        }
        Term::Ascript(t, _) => normalize(env, ctx, t),
        // Leaves and closed-ish nodes: no sub-terms to normalize (levels aside).
        Term::Type(_)
        | Term::Omega
        | Term::Var(_)
        | Term::Const { .. }
        | Term::IndFormer { .. }
        | Term::Constructor { .. } => h,
    }
}

/// Definitional equality `Γ ⊢ a ≡ b : A` for the K1 fragment (`13 §6.2`):
/// α (de Bruijn syntactic identity), then type-directed η (Π-η, Σ-η) when the
/// type is a Π/Σ, else structural congruence with whnf. This is the **K2c
/// extension seam** — K2c replaces this body with lazy-WHNF NbE without
/// changing the signature (`13 §6.3`).
pub fn convert(env: &GlobalEnv, ctx: &Context, ty: &Term, a: &Term, b: &Term) -> bool {
    if a == b {
        return true; // α: syntactic identity under de Bruijn (`13 §6.2` step 1)
    }
    let ty_w = whnf(env, ctx, ty);
    match &ty_w {
        Term::Pi(dom, cod) => {
            // Π-η (`13 §6.2` step 3): compare `f x` and `g x` at the codomain,
            // for a fresh `x : dom` (`f ≡ λx. f x`).
            let a_w = whnf(env, ctx, a);
            let b_w = whnf(env, ctx, b);
            let a_ext = weaken(&a_w, 1);
            let b_ext = weaken(&b_w, 1);
            let lhs = Term::app(a_ext, Term::var(0));
            let rhs = Term::app(b_ext, Term::var(0));
            let mut ctx2 = ctx.clone();
            ctx2.push((**dom).clone());
            convert(env, &ctx2, cod, &lhs, &rhs)
        }
        Term::Sigma(dom, cod) => {
            // Σ-η (`13 §6.2` step 3): compare both projections.
            let a_w = whnf(env, ctx, a);
            let b_w = whnf(env, ctx, b);
            let a1 = whnf(env, ctx, &Term::proj1(a_w.clone()));
            let b1 = whnf(env, ctx, &Term::proj1(b_w.clone()));
            if !convert(env, ctx, dom, &a1, &b1) {
                return false;
            }
            let cod_a1 = subst0(cod, &a1); // B[a1/x]
            let a2 = whnf(env, ctx, &Term::proj2(a_w.clone()));
            let b2 = whnf(env, ctx, &Term::proj2(b_w.clone()));
            convert(env, ctx, &cod_a1, &a2, &b2)
        }
        _ => conv_struct(env, ctx, a, b),
    }
}

/// Definitional equality of two **types** `Γ ⊢ A ≡ B type` (`13 §6.2` for
/// type expressions). Types do not take η (η is for values at Π/Σ types), so
/// this is whnf + structural congruence. Used for domain matching, ascription,
/// and the mode-switch `A ≡ A'` between the expected and inferred types.
pub fn convert_type(env: &GlobalEnv, ctx: &Context, a: &Term, b: &Term) -> bool {
    conv_struct(env, ctx, a, b)
}

/// Structural congruence (no type-directed η): whnf both sides, then compare
/// structurally, recursing. Used when the type is not Π/Σ (`13 §6.2` step 4
/// and the congruence closure).
fn conv_struct(env: &GlobalEnv, ctx: &Context, a: &Term, b: &Term) -> bool {
    let a = whnf(env, ctx, a);
    let b = whnf(env, ctx, b);
    if a == b {
        return true;
    }
    match (&a, &b) {
        (Term::Type(l1), Term::Type(l2)) => level_eq(l1, l2),
        (Term::Var(i), Term::Var(j)) => i == j,
        (
            Term::Const {
                id: id1,
                level_args: la1,
            },
            Term::Const {
                id: id2,
                level_args: la2,
            },
        ) => id1 == id2 && level_args_eq(la1, la2),
        (
            Term::IndFormer {
                id: id1,
                level_args: la1,
            },
            Term::IndFormer {
                id: id2,
                level_args: la2,
            },
        ) => id1 == id2 && level_args_eq(la1, la2),
        (
            Term::Constructor {
                id: id1,
                level_args: la1,
            },
            Term::Constructor {
                id: id2,
                level_args: la2,
            },
        ) => id1 == id2 && level_args_eq(la1, la2),
        (Term::Pi(a1, b1), Term::Pi(a2, b2)) => {
            conv_struct(env, ctx, a1, a2) && {
                let mut c = ctx.clone();
                c.push((**a1).clone());
                conv_struct(env, &c, b1, b2)
            }
        }
        (Term::Lam(a1, t1), Term::Lam(a2, t2)) => {
            conv_struct(env, ctx, a1, a2) && {
                let mut c = ctx.clone();
                c.push((**a1).clone());
                conv_struct(env, &c, t1, t2)
            }
        }
        (Term::Sigma(a1, b1), Term::Sigma(a2, b2)) => {
            conv_struct(env, ctx, a1, a2) && {
                let mut c = ctx.clone();
                c.push((**a1).clone());
                conv_struct(env, &c, b1, b2)
            }
        }
        (Term::Pair(a1, b1), Term::Pair(a2, b2)) => {
            conv_struct(env, ctx, a1, a2) && conv_struct(env, ctx, b1, b2)
        }
        (Term::App(f1, a1), Term::App(f2, a2)) => {
            conv_struct(env, ctx, f1, f2) && conv_struct(env, ctx, a1, a2)
        }
        (Term::Proj1(p1), Term::Proj1(p2)) => conv_struct(env, ctx, p1, p2),
        (Term::Proj2(p1), Term::Proj2(p2)) => conv_struct(env, ctx, p1, p2),
        (
            Term::Elim {
                fam: f1,
                level_args: la1,
                params: p1,
                motive: m1,
                methods: ms1,
                indices: ix1,
                scrut: s1,
            },
            Term::Elim {
                fam: f2,
                level_args: la2,
                params: p2,
                motive: m2,
                methods: ms2,
                indices: ix2,
                scrut: s2,
            },
        ) => {
            f1 == f2
                && level_args_eq(la1, la2)
                && p1.len() == p2.len()
                && p1.iter().zip(p2).all(|(x, y)| conv_struct(env, ctx, x, y))
                && conv_struct(env, ctx, m1, m2)
                && ms1.len() == ms2.len()
                && ms1
                    .iter()
                    .zip(ms2)
                    .all(|(x, y)| conv_struct(env, ctx, x, y))
                && ix1.len() == ix2.len()
                && ix1
                    .iter()
                    .zip(ix2)
                    .all(|(x, y)| conv_struct(env, ctx, x, y))
                && conv_struct(env, ctx, s1, s2)
        }
        (Term::Ascript(t1, _), x) => conv_struct(env, ctx, t1, x),
        (x, Term::Ascript(t2, _)) => conv_struct(env, ctx, x, t2),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::term::Level;

    #[test]
    fn level_semilattice_eq() {
        assert!(level_eq(&Level::zero(), &Level::zero()));
        assert!(level_eq(
            &Level::zero().max(Level::suc(Level::zero())),
            &Level::suc(Level::zero())
        ));
        assert!(level_eq(
            &Level::suc(Level::zero()).max(Level::zero()),
            &Level::suc(Level::zero())
        ));
        assert!(level_eq(&Level::zero().max(Level::zero()), &Level::zero())); // idempotent
        assert!(!level_eq(&Level::zero(), &Level::suc(Level::zero())));
    }

    #[test]
    fn beta_whnf() {
        let env = GlobalEnv::new();
        let ctx = Context::new();
        // (λ x. x) y  ⇝  y   (x at index 0, y a free var 0 in empty ctx)
        let redex = Term::app(
            Term::lam(Term::Type(Level::zero()), Term::var(0)),
            Term::Type(Level::zero()),
        );
        assert_eq!(whnf(&env, &ctx, &redex), Term::Type(Level::zero()));
    }

    #[test]
    fn sigma_beta_whnf() {
        let env = GlobalEnv::new();
        let ctx = Context::new();
        let pair = Term::pair(Term::Type(Level::zero()), Term::Omega);
        assert_eq!(
            whnf(&env, &ctx, &Term::proj1(pair.clone())),
            Term::Type(Level::zero())
        );
        assert_eq!(whnf(&env, &ctx, &Term::proj2(pair)), Term::Omega);
    }

    #[test]
    fn pi_eta_convert() {
        let env = GlobalEnv::new();
        let ctx = Context::new();
        // f : (x:A)→B  in context; f ≡ λx. f x  at the Π-type.
        let a = Term::Type(Level::zero());
        let b = Term::Type(Level::suc(Level::zero()));
        let pi_ty = Term::pi(a.clone(), b.clone());
        // context: f at index 0 with type (x:A)→B
        let mut c = ctx.clone();
        c.push(pi_ty.clone());
        let f = Term::var(0);
        let eta = Term::lam(a.clone(), Term::app(Term::var(1), Term::var(0))); // λx. f x (f at 1, x at 0)
        assert!(convert(&env, &c, &pi_ty, &f, &eta));
        assert!(convert(&env, &c, &pi_ty, &eta, &f));
    }

    #[test]
    fn sigma_eta_convert() {
        let env = GlobalEnv::new();
        let ctx = Context::new();
        let a = Term::Type(Level::zero());
        let b = Term::Type(Level::suc(Level::zero()));
        let sig_ty = Term::sigma(a.clone(), b.clone());
        let mut c = ctx.clone();
        c.push(sig_ty.clone());
        let p = Term::var(0);
        let eta = Term::pair(Term::proj1(p.clone()), Term::proj2(p.clone()));
        assert!(convert(&env, &c, &sig_ty, &p, &eta));
    }
}
