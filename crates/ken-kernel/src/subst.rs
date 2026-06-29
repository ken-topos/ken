//! Capture-avoiding substitution, weakening, and level instantiation (`11 §5`).
//!
//! de Bruijn machinery: shifting free indices (weakening), single substitution
//! (β/ι/δ are defined in terms of it), and substituting level parameters with
//! concrete level arguments (`12 §4` instantiation). Global constants are
//! invariant under these operations — they are closed in `Σ` (`11 §5`).

use crate::term::{Level, LevelVar, Term};

/// Shift free de Bruijn indices `>= cutoff` by `d` (Pierce TaPL §6). Used for
/// weakening (`cutoff = 0`) and internally by [`subst_var`].
pub fn shift(term: &Term, d: i64, cutoff: usize) -> Term {
    match term {
        Term::Var(i) => {
            if *i >= cutoff {
                Term::Var(((*i as i64) + d) as usize)
            } else {
                Term::Var(*i)
            }
        }
        // Binders: the body's cutoff increases by one per bound variable.
        Term::Pi(a, b) => Term::pi(shift(a, d, cutoff), shift(b, d, cutoff + 1)),
        Term::Lam(a, t) => Term::lam(shift(a, d, cutoff), shift(t, d, cutoff + 1)),
        Term::Sigma(a, b) => Term::sigma(shift(a, d, cutoff), shift(b, d, cutoff + 1)),
        Term::Let { ty, val, body } => Term::Let {
            ty: Box::new(shift(ty, d, cutoff)),
            val: Box::new(shift(val, d, cutoff)),
            body: Box::new(shift(body, d, cutoff + 1)),
        },
        // Non-binders: recurse into children at the same cutoff.
        Term::App(f, a) => Term::app(shift(f, d, cutoff), shift(a, d, cutoff)),
        Term::Pair(a, b) => Term::pair(shift(a, d, cutoff), shift(b, d, cutoff)),
        Term::Proj1(p) => Term::proj1(shift(p, d, cutoff)),
        Term::Proj2(p) => Term::proj2(shift(p, d, cutoff)),
        Term::Ascript(t, a) => {
            Term::Ascript(Box::new(shift(t, d, cutoff)), Box::new(shift(a, d, cutoff)))
        }
        Term::Eq(a, t, u) => Term::Eq(
            Box::new(shift(a, d, cutoff)),
            Box::new(shift(t, d, cutoff)),
            Box::new(shift(u, d, cutoff)),
        ),
        Term::Cast(a, b, e, t) => Term::Cast(
            Box::new(shift(a, d, cutoff)),
            Box::new(shift(b, d, cutoff)),
            Box::new(shift(e, d, cutoff)),
            Box::new(shift(t, d, cutoff)),
        ),
        Term::J(m, d2, e) => Term::J(
            Box::new(shift(m, d, cutoff)),
            Box::new(shift(d2, d, cutoff)),
            Box::new(shift(e, d, cutoff)),
        ),
        Term::Quot(a, r) => {
            Term::Quot(Box::new(shift(a, d, cutoff)), Box::new(shift(r, d, cutoff)))
        }
        Term::QuotClass(t) => Term::QuotClass(Box::new(shift(t, d, cutoff))),
        Term::Trunc(a) => Term::Trunc(Box::new(shift(a, d, cutoff))),
        Term::TruncProj(t) => Term::TruncProj(Box::new(shift(t, d, cutoff))),
        Term::Refl(t) => Term::Refl(Box::new(shift(t, d, cutoff))),
        Term::QuotElim {
            motive,
            method,
            scrut,
        } => Term::QuotElim {
            motive: Box::new(shift(motive, d, cutoff)),
            method: Box::new(shift(method, d, cutoff)),
            scrut: Box::new(shift(scrut, d, cutoff)),
        },
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
            params: params.iter().map(|p| shift(p, d, cutoff)).collect(),
            motive: Box::new(shift(motive, d, cutoff)),
            methods: methods.iter().map(|m| shift(m, d, cutoff)).collect(),
            indices: indices.iter().map(|i| shift(i, d, cutoff)).collect(),
            scrut: Box::new(shift(scrut, d, cutoff)),
        },
        // Closed-ish nodes: no free variables to shift (levels are not de Bruijn).
        Term::Type(_)
        | Term::Omega
        | Term::Const { .. }
        | Term::IndFormer { .. }
        | Term::Constructor { .. } => term.clone(),
    }
}

/// Weaken a term into a context with `by` more bindings (`11 §5`).
pub fn weaken(term: &Term, by: i64) -> Term {
    if by == 0 {
        return term.clone();
    }
    shift(term, by, 0)
}

/// Single substitution `t[j := u]`: replace de Bruijn index `j` with `u`,
/// decrementing indices `> j` by one (the binder `j` is removed), capture
/// avoiding (u is weakened under each inner binder). Pierce TaML §6.
pub fn subst_var(term: &Term, j: usize, u: &Term) -> Term {
    let u_under = |binder: usize| -> Term { shift(u, binder as i64, 0) };
    match term {
        Term::Var(i) => {
            if *i == j {
                u_under(0)
            } else if *i > j {
                Term::Var(*i - 1)
            } else {
                Term::Var(*i)
            }
        }
        Term::Pi(a, b) => Term::pi(subst_var(a, j, u), subst_var(b, j + 1, &shift(u, 1, 0))),
        Term::Lam(a, t) => Term::lam(subst_var(a, j, u), subst_var(t, j + 1, &shift(u, 1, 0))),
        Term::Sigma(a, b) => Term::sigma(subst_var(a, j, u), subst_var(b, j + 1, &shift(u, 1, 0))),
        Term::Let { ty, val, body } => Term::Let {
            ty: Box::new(subst_var(ty, j, u)),
            val: Box::new(subst_var(val, j, u)),
            body: Box::new(subst_var(body, j + 1, &shift(u, 1, 0))),
        },
        Term::App(f, a) => Term::app(subst_var(f, j, u), subst_var(a, j, u)),
        Term::Pair(a, b) => Term::pair(subst_var(a, j, u), subst_var(b, j, u)),
        Term::Proj1(p) => Term::proj1(subst_var(p, j, u)),
        Term::Proj2(p) => Term::proj2(subst_var(p, j, u)),
        Term::Ascript(t, a) => {
            Term::Ascript(Box::new(subst_var(t, j, u)), Box::new(subst_var(a, j, u)))
        }
        Term::Eq(a, t, u2) => Term::Eq(
            Box::new(subst_var(a, j, u)),
            Box::new(subst_var(t, j, u)),
            Box::new(subst_var(u2, j, u)),
        ),
        Term::Cast(a, b, e, t) => Term::Cast(
            Box::new(subst_var(a, j, u)),
            Box::new(subst_var(b, j, u)),
            Box::new(subst_var(e, j, u)),
            Box::new(subst_var(t, j, u)),
        ),
        Term::J(m, d2, e) => Term::J(
            Box::new(subst_var(m, j, u)),
            Box::new(subst_var(d2, j, u)),
            Box::new(subst_var(e, j, u)),
        ),
        Term::Quot(a, r) => Term::Quot(Box::new(subst_var(a, j, u)), Box::new(subst_var(r, j, u))),
        Term::QuotClass(t) => Term::QuotClass(Box::new(subst_var(t, j, u))),
        Term::Trunc(a) => Term::Trunc(Box::new(subst_var(a, j, u))),
        Term::TruncProj(t) => Term::TruncProj(Box::new(subst_var(t, j, u))),
        Term::Refl(t) => Term::Refl(Box::new(subst_var(t, j, u))),
        Term::QuotElim {
            motive,
            method,
            scrut,
        } => Term::QuotElim {
            motive: Box::new(subst_var(motive, j, u)),
            method: Box::new(subst_var(method, j, u)),
            scrut: Box::new(subst_var(scrut, j, u)),
        },
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
            params: params.iter().map(|p| subst_var(p, j, u)).collect(),
            motive: Box::new(subst_var(motive, j, u)),
            methods: methods.iter().map(|m| subst_var(m, j, u)).collect(),
            indices: indices.iter().map(|i| subst_var(i, j, u)).collect(),
            scrut: Box::new(subst_var(scrut, j, u)),
        },
        Term::Type(_)
        | Term::Omega
        | Term::Const { .. }
        | Term::IndFormer { .. }
        | Term::Constructor { .. } => term.clone(),
    }
}

/// `t[0 := u]` — substitute the nearest bound variable, the form used by β
/// (`(λx.t) a ⇝ t[a/x]`), Σ-β, and ι method instantiation (`13 §6.1`).
pub fn subst0(term: &Term, u: &Term) -> Term {
    subst_var(term, 0, u)
}

/// Substitute the **outermost** `m` de Bruijn binders of a term with concrete
/// values `params` (taken from an outer context Γ), keeping the inner
/// `inner_depth` binders in place.
///
/// `term` is in context `[p₁..pₘ, inner₁..inner_{inner_depth}]` (params
/// outermost, at indices `inner_depth .. inner_depth+m-1`; inner binders at
/// `0..inner_depth-1`). The result is in context `[Γ, inner₁..]` — the params
/// are replaced by `params` (values in Γ, weakened below the inner part), and
/// the inner binders keep their indices. Under binders in `term`, `inner_depth`
/// grows (those binders join the inner part).
///
/// Used to instantiate a declaration's stored telescope (which is parametric in
/// `Δ_p`) against concrete params at a use site — e.g. computing an
/// eliminator's method type from the family's constructor signature.
pub fn subst_outer(term: &Term, m: usize, params: &[Term], inner_depth: usize) -> Term {
    match term {
        Term::Var(i) => {
            if *i < inner_depth {
                Term::Var(*i)
            } else {
                // p_{j'} (1-indexed) is at index inner_depth + m - j'; its value
                // is params[j'-1] = params[inner_depth + m - 1 - i], weakened
                // by inner_depth to sit below the inner telescope.
                let p_idx = inner_depth + m - 1 - *i;
                weaken(&params[p_idx], inner_depth as i64)
            }
        }
        Term::Pi(a, b) => Term::pi(
            subst_outer(a, m, params, inner_depth),
            subst_outer(b, m, params, inner_depth + 1),
        ),
        Term::Lam(a, t) => Term::lam(
            subst_outer(a, m, params, inner_depth),
            subst_outer(t, m, params, inner_depth + 1),
        ),
        Term::Sigma(a, b) => Term::sigma(
            subst_outer(a, m, params, inner_depth),
            subst_outer(b, m, params, inner_depth + 1),
        ),
        Term::Let { ty, val, body } => Term::Let {
            ty: Box::new(subst_outer(ty, m, params, inner_depth)),
            val: Box::new(subst_outer(val, m, params, inner_depth)),
            body: Box::new(subst_outer(body, m, params, inner_depth + 1)),
        },
        Term::App(f, a) => Term::app(
            subst_outer(f, m, params, inner_depth),
            subst_outer(a, m, params, inner_depth),
        ),
        Term::Pair(a, b) => Term::pair(
            subst_outer(a, m, params, inner_depth),
            subst_outer(b, m, params, inner_depth),
        ),
        Term::Proj1(p) => Term::proj1(subst_outer(p, m, params, inner_depth)),
        Term::Proj2(p) => Term::proj2(subst_outer(p, m, params, inner_depth)),
        Term::Ascript(t, a) => Term::Ascript(
            Box::new(subst_outer(t, m, params, inner_depth)),
            Box::new(subst_outer(a, m, params, inner_depth)),
        ),
        Term::Eq(a, t, u) => Term::Eq(
            Box::new(subst_outer(a, m, params, inner_depth)),
            Box::new(subst_outer(t, m, params, inner_depth)),
            Box::new(subst_outer(u, m, params, inner_depth)),
        ),
        Term::Cast(a, b, e, t) => Term::Cast(
            Box::new(subst_outer(a, m, params, inner_depth)),
            Box::new(subst_outer(b, m, params, inner_depth)),
            Box::new(subst_outer(e, m, params, inner_depth)),
            Box::new(subst_outer(t, m, params, inner_depth)),
        ),
        Term::J(ml, d2, e) => Term::J(
            Box::new(subst_outer(ml, m, params, inner_depth)),
            Box::new(subst_outer(d2, m, params, inner_depth)),
            Box::new(subst_outer(e, m, params, inner_depth)),
        ),
        Term::Quot(a, r) => Term::Quot(
            Box::new(subst_outer(a, m, params, inner_depth)),
            Box::new(subst_outer(r, m, params, inner_depth)),
        ),
        Term::QuotClass(t) => Term::QuotClass(Box::new(subst_outer(t, m, params, inner_depth))),
        Term::Trunc(a) => Term::Trunc(Box::new(subst_outer(a, m, params, inner_depth))),
        Term::TruncProj(t) => Term::TruncProj(Box::new(subst_outer(t, m, params, inner_depth))),
        Term::Refl(t) => Term::Refl(Box::new(subst_outer(t, m, params, inner_depth))),
        Term::QuotElim {
            motive,
            method,
            scrut,
        } => Term::QuotElim {
            motive: Box::new(subst_outer(motive, m, params, inner_depth)),
            method: Box::new(subst_outer(method, m, params, inner_depth)),
            scrut: Box::new(subst_outer(scrut, m, params, inner_depth)),
        },
        Term::Elim {
            fam,
            level_args,
            params: eps,
            motive,
            methods,
            indices,
            scrut,
        } => Term::Elim {
            fam: *fam,
            level_args: level_args.clone(),
            params: eps
                .iter()
                .map(|p| subst_outer(p, m, params, inner_depth))
                .collect(),
            motive: Box::new(subst_outer(motive, m, params, inner_depth)),
            methods: methods
                .iter()
                .map(|mth| subst_outer(mth, m, params, inner_depth))
                .collect(),
            indices: indices
                .iter()
                .map(|i| subst_outer(i, m, params, inner_depth))
                .collect(),
            scrut: Box::new(subst_outer(scrut, m, params, inner_depth)),
        },
        // No free term variables; levels are untouched (term-var subst only).
        Term::Type(_)
        | Term::Omega
        | Term::Const { .. }
        | Term::IndFormer { .. }
        | Term::Constructor { .. } => term.clone(),
    }
}

/// Substitute a telescope of arguments `[u₁,…,uₙ]` for the bound variables of
/// a telescope, leftmost variable first. `body` is in the context of the
/// telescope (depth `n`); the result is in the context outside it. Used for
/// `B[a/x]` and applying a type former to arguments.
///
/// Substitutes `u₁` for the outermost (highest-index) variable and works
/// inward, so each `uᵢ` is substituted at the depth matching its binder.
pub fn subst_tel(body: &Term, args: &[Term]) -> Term {
    // The telescope binds n variables; body's var n-1 is the outermost (first)
    // param, var 0 the innermost (last). Substitute from the outermost in:
    // replace var (n-1) with args[0], then var (n-2) with args[1], etc. Each
    // subst decrements the remaining indices, so substitute in descending order
    // of the original index, which after prior substitutions still lines up.
    let mut t = body.clone();
    let n = args.len();
    for (k, u) in args.iter().enumerate() {
        // At this point `t` is in a context of depth (n - k); the variable to
        // replace is the outermost, index (n - k - 1). Substitute it with u
        // (weakened to the current outer context if needed — u is already in
        // the outer context, depth 0 here, so no weakening).
        let j = n - k - 1;
        t = subst_var(&t, j, u);
    }
    t
}

/// Apply a term `head` (a type former / constructor / function) to a vector of
/// arguments left-to-right: `head u₁ u₂ … uₙ`.
pub fn apply_args(head: Term, args: &[Term]) -> Term {
    args.iter().fold(head, |acc, a| Term::app(acc, a.clone()))
}

/// Instantiate a Π-telescope's codomain: given `f : Π (x₁:A₁)…(xₙ:Aₙ). B` and
/// `n` arguments, produce `B[u₁/x₁,…,uₙ/xₙ]` by peeling `n` Π binders and
/// substituting. If `f` is not a Π-chain of length `>= n`, returns `None`.
pub fn instantiate_codomain(f: &Term, args: &[Term]) -> Option<Term> {
    let mut t = f.clone();
    for u in args {
        match t {
            Term::Pi(_, b) => t = subst0(&b, u),
            _ => return None,
        }
    }
    Some(t)
}

// --- Level instantiation (`12 §4`) -----------------------------------------

/// Substitute level parameters `params` with `args` in a level. A variable not
/// in `params` is left intact (it belongs to a different level abstraction,
/// e.g. the motive's `ℓ'`).
pub fn subst_level(level: &Level, params: &[LevelVar], args: &[Level]) -> Level {
    match level {
        Level::Zero => Level::Zero,
        Level::Suc(a) => Level::Suc(Box::new(subst_level(a, params, args))),
        Level::Max(a, b) => Level::Max(
            Box::new(subst_level(a, params, args)),
            Box::new(subst_level(b, params, args)),
        ),
        Level::Var(v) => params
            .iter()
            .position(|p| p == v)
            .map(|i| args[i].clone())
            .unwrap_or_else(|| Level::Var(*v)),
    }
}

/// Substitute level parameters `params` with `args` throughout a term
/// (in `Type ℓ` and in `level_args` of const/former/constructor/elim uses).
pub fn subst_levels(term: &Term, params: &[LevelVar], args: &[Level]) -> Term {
    match term {
        Term::Type(l) => Term::Type(subst_level(l, params, args)),
        Term::Const { id, level_args } => Term::Const {
            id: *id,
            level_args: level_args
                .iter()
                .map(|l| subst_level(l, params, args))
                .collect(),
        },
        Term::IndFormer { id, level_args } => Term::IndFormer {
            id: *id,
            level_args: level_args
                .iter()
                .map(|l| subst_level(l, params, args))
                .collect(),
        },
        Term::Constructor { id, level_args } => Term::Constructor {
            id: *id,
            level_args: level_args
                .iter()
                .map(|l| subst_level(l, params, args))
                .collect(),
        },
        Term::Elim {
            fam,
            level_args,
            params: eps,
            motive,
            methods,
            indices,
            scrut,
        } => Term::Elim {
            fam: *fam,
            level_args: level_args
                .iter()
                .map(|l| subst_level(l, params, args))
                .collect(),
            params: eps.iter().map(|p| subst_levels(p, params, args)).collect(),
            motive: Box::new(subst_levels(motive, params, args)),
            methods: methods
                .iter()
                .map(|m| subst_levels(m, params, args))
                .collect(),
            indices: indices
                .iter()
                .map(|i| subst_levels(i, params, args))
                .collect(),
            scrut: Box::new(subst_levels(scrut, params, args)),
        },
        // Recurse into children (no level fields elsewhere).
        Term::Pi(a, b) => Term::pi(subst_levels(a, params, args), subst_levels(b, params, args)),
        Term::Lam(a, t) => Term::lam(subst_levels(a, params, args), subst_levels(t, params, args)),
        Term::Sigma(a, b) => {
            Term::sigma(subst_levels(a, params, args), subst_levels(b, params, args))
        }
        Term::App(f, a) => Term::app(subst_levels(f, params, args), subst_levels(a, params, args)),
        Term::Pair(a, b) => {
            Term::pair(subst_levels(a, params, args), subst_levels(b, params, args))
        }
        Term::Proj1(p) => Term::proj1(subst_levels(p, params, args)),
        Term::Proj2(p) => Term::proj2(subst_levels(p, params, args)),
        Term::Let { ty, val, body } => Term::Let {
            ty: Box::new(subst_levels(ty, params, args)),
            val: Box::new(subst_levels(val, params, args)),
            body: Box::new(subst_levels(body, params, args)),
        },
        Term::Ascript(t, a) => Term::Ascript(
            Box::new(subst_levels(t, params, args)),
            Box::new(subst_levels(a, params, args)),
        ),
        Term::Eq(a, t, u) => Term::Eq(
            Box::new(subst_levels(a, params, args)),
            Box::new(subst_levels(t, params, args)),
            Box::new(subst_levels(u, params, args)),
        ),
        Term::Cast(a, b, e, t) => Term::Cast(
            Box::new(subst_levels(a, params, args)),
            Box::new(subst_levels(b, params, args)),
            Box::new(subst_levels(e, params, args)),
            Box::new(subst_levels(t, params, args)),
        ),
        Term::J(m, d2, e) => Term::J(
            Box::new(subst_levels(m, params, args)),
            Box::new(subst_levels(d2, params, args)),
            Box::new(subst_levels(e, params, args)),
        ),
        Term::Quot(a, r) => Term::Quot(
            Box::new(subst_levels(a, params, args)),
            Box::new(subst_levels(r, params, args)),
        ),
        Term::QuotClass(t) => Term::QuotClass(Box::new(subst_levels(t, params, args))),
        Term::Trunc(a) => Term::Trunc(Box::new(subst_levels(a, params, args))),
        Term::TruncProj(t) => Term::TruncProj(Box::new(subst_levels(t, params, args))),
        Term::Refl(t) => Term::Refl(Box::new(subst_levels(t, params, args))),
        Term::QuotElim {
            motive,
            method,
            scrut,
        } => Term::QuotElim {
            motive: Box::new(subst_levels(motive, params, args)),
            method: Box::new(subst_levels(method, params, args)),
            scrut: Box::new(subst_levels(scrut, params, args)),
        },
        Term::Omega | Term::Var(_) => term.clone(),
    }
}
