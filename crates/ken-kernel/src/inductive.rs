//! Inductive families — strict-positivity admission and the dependent
//! eliminator (`14-inductive.md`).
//!
//! Three loads:
//! - [`check_positivity`] — the strict-positivity check (`14 §8`), the
//!   **fixed** algorithm with `occurs`-guards on every position that could
//!   discard a subterm (application arguments `C u`, recursive-occurrence
//!   indices `D Δ_p t̄`, type parameters `X`). This is the soundness hole the
//!   Architect's review caught (`Bad3`/`Bad4`); the guards conservatively
//!   reject what K1 cannot prove strictly positive (`14 §8.4`).
//! - [`method_type`] — the dependent eliminator's per-constructor method type
//!   `Π Δₖ. Π (IHs). M t̄ₖ (cₖ p̄ Δₖ)`, computed from the family declaration and
//!   the concrete motive/params at a use site (`14 §3`, `14 §3.1`). W-style
//!   recursive args (`(b:B) → D Δ_p t̄[b]`) get a Π-abstracted IH
//!   `(b:B) → M t̄[b] (k b)` (K1.5).
//! - [`iota_reduct`] — the algorithmic ι-step `elim_D … (cₖ p̄ ā) ⇝ mₖ ā [IHs]`
//!   (`14 §7.3`, `14 §7.7`), capture-avoiding, with induction hypotheses on
//!   structurally smaller recursive arguments. W-style args produce a
//!   λ-abstracted IH `λb. elim_D … (k b)` (K1.5).
//!
//! **K1.5**: W-style (Π-bound) recursive arguments `(b:B) → D Δ_p t̄[b]` are
//! now **admitted** (`14 §2.1`, `14 §8.4`). The separate blanket gate
//! `check_no_pi_bound_recursive` is retired; strict positivity (`14 §8.2`) is
//! the sole structural admission test. The eliminator and ι handle the
//! Π-abstracted IH and the λ-threaded recursive call (`14 §3.1`, `14 §7.7`).

use crate::env::{ConstructorDecl, InductiveDecl};
use crate::error::{KernelError, KernelResult};
use crate::subst::{apply_args, subst_levels, subst_outer, subst_tel, weaken};
use crate::term::{GlobalId, Level, Term};

/// Does the inductive former `d` occur anywhere in `t` (syntactic sub-term)?
/// Used by the positivity guards (`14 §8`). de Bruijn indices make this
/// unambiguous: a former is a `Term::IndFormer { id, .. }` node.
pub fn occurs(d: GlobalId, t: &Term) -> bool {
    match t {
        Term::IndFormer { id, .. } => *id == d,
        _ => t.children().iter().any(|c| occurs(d, c)),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Pol {
    Plus,
    Minus,
}

impl Pol {
    fn flip(self) -> Pol {
        match self {
            Pol::Plus => Pol::Minus,
            Pol::Minus => Pol::Plus,
        }
    }
}

/// Peel a left-nested `App` spine into `(head, args)` left-to-right.
pub fn peel_app(t: &Term) -> (Term, Vec<Term>) {
    let mut args = Vec::new();
    let mut cur = t.clone();
    while let Term::App(f, a) = cur {
        args.push((*a).clone());
        cur = (*f).clone();
    }
    args.reverse();
    (cur, args)
}

/// Peel leading `Pi` binders into `(binder_domain_types, body)`.
pub fn peel_pi(t: &Term) -> (Vec<Term>, Term) {
    let mut doms = Vec::new();
    let mut cur = t.clone();
    while let Term::Pi(a, b) = cur {
        doms.push((*a).clone());
        cur = (*b).clone();
    }
    (doms, cur)
}

/// `check-pos-arg(D, pol, A)` — the strict-positivity judgment (`14 §8.2`).
///
/// Returns `true` if `A` is strictly positive in `D` at polarisation `pol`.
/// Every position that would discard a subterm without inspection is guarded
/// by an `occurs` check (the fixed algorithm): application arguments, the
/// indices of a recursive occurrence, and bare type parameters.
fn check_pos_arg(d: GlobalId, pol: Pol, a: &Term) -> bool {
    match a {
        Term::Pi(dom, cod) => check_pos_arg(d, pol.flip(), dom) && check_pos_arg(d, pol, cod),
        Term::Sigma(dom, cod) => check_pos_arg(d, pol, dom) && check_pos_arg(d, pol, cod),
        Term::App(_, _) => {
            // `C u` (or `D Δ_p t̄` if the head is `D`).
            let (head, args) = peel_app(a);
            match head {
                Term::IndFormer { id, .. } if id == d => {
                    // Recursive occurrence `D Δ_p t̄`: positive polarity, and
                    // `D` must not occur in the (index) arguments.
                    pol == Pol::Plus && args.iter().all(|x| !occurs(d, x))
                }
                Term::IndFormer { .. }
                | Term::Const { .. }
                | Term::Constructor { .. }
                | Term::Var(_) => {
                    // `C u` with a non-`D` head: recurse into the (atomic) head
                    // and `occurs`-guard every argument.
                    check_pos_arg(d, pol, &head) && args.iter().all(|x| !occurs(d, x))
                }
                Term::Type(_) => {
                    // `Type ℓ` applied is ill-formed as a type; conservatively
                    // reject if `D` lurks anywhere.
                    args.is_empty() || !occurs(d, a)
                }
                _ => {
                    // Pi/Sigma/Lam/... applied: ill-formed; conservative reject.
                    !occurs(d, a)
                }
            }
        }
        Term::Type(_) => true, // `Type ℓ`; `D` is a type, not a level.
        Term::IndFormer { id, .. } if *id == d => {
            // Bare `D` (no arguments) — recursive occurrence with empty indices.
            pol == Pol::Plus
        }
        Term::IndFormer { .. } | Term::Const { .. } | Term::Constructor { .. } | Term::Var(_) => {
            // Bare `X` — a parameter or other type; reject if `D` occurs within.
            !occurs(d, a)
        }
        // Anything else as a type is ill-formed; conservatively reject if D hides.
        _ => !occurs(d, a),
    }
}

/// Run the strict-positivity check on a family declaration (`14 §8`): every
/// constructor argument type must be strictly positive in `D`. The family's
/// own parameters and indices are also `occurs`-checked (K1 rejects `D`
/// appearing in its own indices, `Bad4`, and nested parameter occurrences).
pub fn check_positivity(ind: &InductiveDecl) -> KernelResult<()> {
    let d = ind.id;
    for p in &ind.params {
        if occurs(d, p) {
            return Err(KernelError::PositivityViolation(
                "D occurs in its own parameter telescope".into(),
            ));
        }
    }
    for ix in &ind.indices {
        if occurs(d, ix) {
            return Err(KernelError::PositivityViolation(
                "D occurs in its own index telescope".into(),
            ));
        }
    }
    for c in &ind.constructors {
        for (j, a) in c.args.iter().enumerate() {
            if !check_pos_arg(d, Pol::Plus, a) {
                return Err(KernelError::PositivityViolation(format!(
                    "non-strictly-positive occurrence of D in constructor {:?} arg {j}",
                    c.id
                )));
            }
        }
    }
    Ok(())
}

/// The recursive arguments of a constructor: `(arg_position, branching_tel,
/// index_exprs)` for each arg whose type peels to `(b₁:B₁)...(b_{nb}:B_{nb})
/// → D Δ_p t̄` (K1.5, `14 §2.1`).
///
/// - `branching_tel` — the leading Π-binder domains `[B₁, B₂[b₁], ...]`
///   (empty for a direct `D Δ_p t̄`); each `B_k` is in context
///   `[Δ_p, args_before_pos, b₁..b_{k-1}]`.
/// - `index_exprs` — the index expressions after the family's `m` params, in
///   context `[Δ_p, args_before_pos, b₁..b_{nb}]` (under the branching binders).
pub fn recursive_args(
    c: &ConstructorDecl,
    d: GlobalId,
    m: usize,
) -> Vec<(usize, Vec<Term>, Vec<Term>)> {
    let mut out = Vec::new();
    for (j, a) in c.args.iter().enumerate() {
        let (pis, body) = peel_pi(a);
        let (head, args) = peel_app(&body);
        if let Term::IndFormer { id, .. } = head {
            if id == d && args.len() >= m {
                out.push((j, pis, args[m..].to_vec()));
            }
        }
    }
    out
}

/// The dependent eliminator's method type for constructor `k`:
/// `Π Δₖ. Π (IH₁…IH_p). M t̄ₖ (cₖ p̄ ā)` (`14 §3`, `14 §3.1`), in the
/// caller's context Γ.
///
/// W-style recursive args `(b:B) → D Δ_p t̄[b]` get a Π-abstracted IH
/// `(b:B) → M t̄[b] (k b)` (K1.5, `14 §3.1`).
///
/// `motive` (`M`) and `params` (`p̄`) are the concrete motive and param
/// instance at the use site (terms in Γ); `level_args` instantiate the
/// family's level parameters (used in the constructor reference).
pub fn method_type(
    ind: &InductiveDecl,
    k: usize,
    motive: &Term,
    params: &[Term],
    level_args: &[Level],
) -> Term {
    let c = &ind.constructors[k];
    let m = ind.params.len();
    let n = c.args.len();
    let rec = recursive_args(c, ind.id, m);
    let p = rec.len();

    // Conclusion `M t̄ₖ (cₖ p̄ ā')` in context [Γ, a₁'..aₙ', ih₁..ih_p]
    // (depth ctx_depth + n + p, but ctx_depth is implicit — we build relative
    // to Γ by weakening Γ-terms past the n+p new binders).
    let np = (n + p) as i64;
    let m_w = weaken(motive, np);
    let tgt: Vec<Term> = c
        .target_indices
        .iter()
        .map(|t| {
            weaken(
                &subst_levels(&subst_outer(t, m, params, n), &ind.level_params, level_args),
                p as i64,
            )
        })
        .collect();
    let mut capp = Term::Constructor {
        id: c.id,
        level_args: level_args.to_vec(),
    };
    for p in params {
        capp = Term::app(capp, weaken(p, np)); // p̄ weakened past args+IHs
    }
    for j in 0..n {
        // a_{j+1}' is at index (p + n - 1 - j) in [Γ, args, ihs].
        capp = Term::app(capp, Term::var(p + n - 1 - j));
    }
    let mut conclusion = m_w;
    for t in &tgt {
        conclusion = Term::app(conclusion, t.clone());
    }
    conclusion = Term::app(conclusion, capp);

    // Wrap IH binders innermost-first (ih_p … ih_1).
    // Each IH may be:
    //   - Direct (nb=0): `M idxs a_pos` — a plain type.
    //   - W-style (nb≥1): `Π(b₁:B₁)...(b_{nb}:B_{nb}). M idxs (a_pos b₁..b_{nb})`
    //     — a Π-type over the branching telescope (`14 §3.1`).
    let mut ty = conclusion;
    for j in (0..p).rev() {
        let (pos, branching_tel, idxs) = &rec[j];
        let nb = branching_tel.len();
        // Context when building IH_j: [Γ, args, ih₁..ih_{j-1}] (depth n+j from Γ).
        // Inside the nb Π-binders of the IH: [Γ, args, ih₁..ih_{j-1}, b₁..b_{nb}].
        let m_w_body = weaken(motive, (n + j + nb) as i64);
        // Index exprs are in [Δ_p, args_before_pos, b₁..b_{nb}].
        // subst_outer replaces m params (inner_depth = pos+nb); weaken lifts
        // past remaining args and IHs but NOT b vars (we're building inside Pis).
        let idxs_in_body: Vec<Term> = idxs
            .iter()
            .map(|t| {
                weaken(
                    &subst_levels(
                        &subst_outer(t, m, params, *pos + nb),
                        &ind.level_params,
                        level_args,
                    ),
                    (n - pos + j) as i64,
                )
            })
            .collect();
        // a_pos under nb extra binders: index n - 1 - pos + j + nb.
        // Apply to b₁..b_{nb}: Var(nb-1)=b₁, ..., Var(0)=b_{nb}.
        let mut scrut_body = Term::var(n - 1 - pos + j + nb);
        for bk in 0..nb {
            scrut_body = Term::app(scrut_body, Term::var(nb - 1 - bk));
        }
        // Assemble IH body: M idxs (a_pos b₁..b_{nb}).
        let mut ih_inner = m_w_body;
        for ix in &idxs_in_body {
            ih_inner = Term::app(ih_inner, ix.clone());
        }
        ih_inner = Term::app(ih_inner, scrut_body);
        // Wrap in Π-binders from innermost (B_{nb}) to outermost (B₁).
        // B_k is in [Δ_p, args_before_pos, b₁..b_{k-1}] (under k-1 extra binders,
        // 0-indexed: branching_tel[k] has k Pi-binders above it from peel_pi).
        let mut ih_ty = ih_inner;
        for bk in (0..nb).rev() {
            // branching_tel[bk] is in context [Δ_p, args_before_pos, b₁..b_{bk}].
            // Need it in [Γ, args, ihs_so_far, b₁..b_{bk}].
            let b_dom = weaken(
                &subst_levels(
                    &subst_outer(&branching_tel[bk], m, params, *pos + bk),
                    &ind.level_params,
                    level_args,
                ),
                (n - pos + j) as i64,
            );
            ih_ty = Term::pi(b_dom, ih_ty);
        }
        ty = Term::pi(ih_ty, ty);
    }

    // Wrap arg binders innermost-first (aₙ' … a₁').
    for j in (0..n).rev() {
        let a_ty = subst_levels(
            &subst_outer(&c.args[j], m, params, j),
            &ind.level_params,
            level_args,
        ); // in [Γ, a₁'..a_j']
        ty = Term::pi(a_ty, ty);
    }
    ty
}

/// The ι-reduct of an eliminator applied to a constructor-headed scrutinee
/// (`14 §7.3`): `elim_D p̄ M m̄ i̅ (cₖ p̄ ā) ⇝ mₖ ā [IHs]`.
///
/// `ctor_all_args` is the constructor's full argument spine `p̄ ++ ā` (params
/// then args), already peeled from the scrutinee. Returns the reduct, or an
/// error if the spine does not match the constructor's arity.
pub fn iota_reduct(
    ind: &InductiveDecl,
    k: usize,
    level_args: &[Level],
    params: &[Term],
    motive: &Term,
    methods: &[Term],
    ctor_all_args: &[Term],
) -> KernelResult<Term> {
    let c = &ind.constructors[k];
    let m = ind.params.len();
    let n = c.args.len();
    // Arity guards: `raw_wf` checks only scoping for an `Elim`, but `whnf` calls
    // `iota_reduct` on any constructor-headed scrutinee. A raw-well-formed
    // `Elim` with too few params/methods/level-args would index out of bounds
    // here — the kernel contract is yes/no, never a crash (`18 §4`).
    if params.len() != m {
        return Err(KernelError::BadEliminator(format!(
            "expected {m} params, got {}",
            params.len()
        )));
    }
    if methods.len() != ind.constructors.len() {
        return Err(KernelError::BadEliminator(format!(
            "expected {} methods, got {}",
            ind.constructors.len(),
            methods.len()
        )));
    }
    if level_args.len() != ind.level_params.len() {
        return Err(KernelError::BadEliminator(format!(
            "expected {} level args, got {}",
            ind.level_params.len(),
            level_args.len()
        )));
    }
    if ctor_all_args.len() != m + n {
        return Err(KernelError::BadEliminator(format!(
            "constructor {:?} arity mismatch: expected {} args, got {}",
            c.id,
            m + n,
            ctor_all_args.len()
        )));
    }
    let ctor_args = &ctor_all_args[m..]; // ā (the actual constructor args)
    let method = &methods[k];

    let rec = recursive_args(c, ind.id, m);
    // Induction hypotheses for each recursive arg (`14 §7.3`, `14 §7.7`):
    //   - Direct (nb=0):    `elim_D p̄ M m̄ idx(a_j) a_j`
    //   - W-style (nb≥1):  `λ(b₁:B₁)...(b_{nb}:B_{nb}). elim_D p̄ M m̄ idx(a_j b₁..b_{nb}) (a_j b₁..b_{nb})`
    let mut ihs: Vec<Term> = Vec::new();
    for (pos, branching_tel, idxs) in &rec {
        let a_j = &ctor_args[*pos];
        let nb = branching_tel.len();
        if nb == 0 {
            // Direct case: elim applied to a_j itself.
            let idx_vals: Vec<Term> = idxs
                .iter()
                .map(|t| {
                    subst_levels(
                        &subst_tel(&subst_outer(t, m, params, *pos), &ctor_args[..*pos]),
                        &ind.level_params,
                        level_args,
                    )
                })
                .collect();
            ihs.push(Term::Elim {
                fam: ind.id,
                level_args: level_args.to_vec(),
                params: params.to_vec(),
                motive: Box::new(motive.clone()),
                methods: methods.to_vec(),
                indices: idx_vals,
                scrut: Box::new(a_j.clone()),
            });
        } else {
            // W-style case: build λ(b₁:B₁)...(b_{nb}:B_{nb}). elim_D … (a_j b₁..b_{nb}).
            // Inside nb lambda binders, context extends by b₁..b_{nb}.
            // a_j weakened by nb to sit inside the binders.
            let a_j_inner = weaken(a_j, nb as i64);
            // a_j b₁ b₂ ... b_{nb}: b_k = Var(nb-1-k) under the lambdas.
            let mut scrut_inner = a_j_inner;
            for bk in 0..nb {
                scrut_inner = Term::app(scrut_inner, Term::var(nb - 1 - bk));
            }
            // Index vals in [Γ, b₁..b_{nb}]:
            // idxs[i] in [Δ_p, args_before_pos, b₁..b_{nb}]; subst_outer replaces
            // m params (inner_depth=pos+nb), then subst_tel substitutes pos args
            // (weakened by nb to sit inside the binders).
            let ctor_args_inner: Vec<Term> =
                ctor_args[..*pos].iter().map(|t| weaken(t, nb as i64)).collect();
            let idx_vals_inner: Vec<Term> = idxs
                .iter()
                .map(|t| {
                    subst_levels(
                        &subst_tel(
                            &subst_outer(t, m, params, *pos + nb),
                            &ctor_args_inner,
                        ),
                        &ind.level_params,
                        level_args,
                    )
                })
                .collect();
            // Build the elim call inside the lambdas (all Γ-terms weakened by nb).
            let elim_inner = Term::Elim {
                fam: ind.id,
                level_args: level_args.to_vec(),
                params: params.iter().map(|p| weaken(p, nb as i64)).collect(),
                motive: Box::new(weaken(motive, nb as i64)),
                methods: methods.iter().map(|mth| weaken(mth, nb as i64)).collect(),
                indices: idx_vals_inner,
                scrut: Box::new(scrut_inner),
            };
            // Wrap in λ-binders from innermost (B_{nb}) to outermost (B₁).
            // B_k (branching_tel[bk]) in [Δ_p, args_before_pos, b₁..b_{bk}].
            // subst_outer with inner_depth=pos+bk, then subst_tel with ctor_args
            // weakened by bk → result in [Γ, b₁..b_{bk}].
            let mut ih_term = elim_inner;
            for bk in (0..nb).rev() {
                let ctor_args_k: Vec<Term> =
                    ctor_args[..*pos].iter().map(|t| weaken(t, bk as i64)).collect();
                let b_dom = subst_levels(
                    &subst_tel(
                        &subst_outer(&branching_tel[bk], m, params, *pos + bk),
                        &ctor_args_k,
                    ),
                    &ind.level_params,
                    level_args,
                );
                ih_term = Term::lam(b_dom, ih_term);
            }
            ihs.push(ih_term);
        }
    }

    // `mₖ ā [IHs]` — method applied to the constructor args then the IHs.
    let mut full_args = ctor_args.to_vec();
    full_args.extend(ihs);
    Ok(apply_args(method.clone(), &full_args))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::telescope_to_pi;
    use crate::term::{Level, LevelVar};

    fn d(id: u32) -> GlobalId {
        GlobalId(id)
    }

    #[test]
    fn occurs_finds_former() {
        // D applied to something containing D.
        let t = Term::app(Term::indformer(d(0), vec![]), Term::indformer(d(0), vec![]));
        assert!(occurs(d(0), &t));
        assert!(!occurs(d(1), &t));
    }

    #[test]
    fn positivity_nat_accepted() {
        // data Nat : Type 0 where zero : Nat ; suc : Nat → Nat
        let mut ind = InductiveDecl {
            id: d(0),
            level_params: vec![],
            params: vec![],
            indices: vec![],
            level: Level::zero(),
            constructors: vec![
                ConstructorDecl {
                    id: d(1),
                    args: vec![],
                    target_indices: vec![],
                    type_: Term::Type(Level::zero()),
                    recursive_positions: vec![],
                },
                ConstructorDecl {
                    id: d(2),
                    args: vec![Term::indformer(d(0), vec![])],
                    target_indices: vec![],
                    type_: Term::Type(Level::zero()),
                    recursive_positions: vec![],
                },
            ],
            former_type: Term::Type(Level::zero()),
        };
        ind.build_types();
        assert!(check_positivity(&ind).is_ok());
    }

    #[test]
    fn positivity_bad_rejected() {
        // data Bad : Type 0 where mk : (Bad → Bool) → Bad
        let bool_ = Term::indformer(d(9), vec![]); // some other type `Bool`
        let arg = Term::pi(Term::indformer(d(0), vec![]), bool_); // Bad → Bool
        let mut ind = InductiveDecl {
            id: d(0),
            level_params: vec![],
            params: vec![],
            indices: vec![],
            level: Level::zero(),
            constructors: vec![ConstructorDecl {
                id: d(1),
                args: vec![arg],
                target_indices: vec![],
                type_: Term::Type(Level::zero()),
                recursive_positions: vec![],
            }],
            former_type: Term::Type(Level::zero()),
        };
        ind.build_types();
        assert!(check_positivity(&ind).is_err());
    }

    #[test]
    fn positivity_bad3_in_application_rejected() {
        // data Bad3 : Type 0 where mk : Pair (Bad3 → Empty) Unit → Bad3
        // `Pair` is an inductive former (id 7); arg = Pair (Bad3→Empty) Unit.
        let empty = Term::indformer(d(8), vec![]);
        let bad3 = Term::indformer(d(0), vec![]);
        let unit = Term::indformer(d(6), vec![]);
        let pair_ty = Term::app(
            Term::app(Term::indformer(d(7), vec![]), Term::pi(bad3.clone(), empty)),
            unit,
        );
        let mut ind = InductiveDecl {
            id: d(0),
            level_params: vec![],
            params: vec![],
            indices: vec![],
            level: Level::zero(),
            constructors: vec![ConstructorDecl {
                id: d(1),
                args: vec![pair_ty],
                target_indices: vec![],
                type_: Term::Type(Level::zero()),
                recursive_positions: vec![],
            }],
            former_type: Term::Type(Level::zero()),
        };
        ind.build_types();
        assert!(
            check_positivity(&ind).is_err(),
            "Bad3 nested-negative-in-application must be rejected"
        );
    }

    #[test]
    fn positivity_bad4_in_own_indices_rejected() {
        // data Bad4 : (Bad4 → Empty) → Type 0 where mk : Bad4 Empty
        let empty = Term::indformer(d(8), vec![]);
        let bad4 = Term::indformer(d(0), vec![]);
        let idx = Term::pi(bad4, empty); // Bad4 → Empty as an index
        let mut ind = InductiveDecl {
            id: d(0),
            level_params: vec![],
            params: vec![],
            indices: vec![idx], // D in its own index telescope
            level: Level::zero(),
            constructors: vec![],
            former_type: Term::Type(Level::zero()),
        };
        let _ = telescope_to_pi; // keep import
        ind.build_types();
        assert!(
            check_positivity(&ind).is_err(),
            "Bad4 D-in-own-indices must be rejected"
        );
    }

    #[test]
    fn w_style_pi_bound_admitted_in_k1p5() {
        // data W : Type 0 where mk : (Nat → W) → W   (strictly positive W-style;
        // K1.5 admits it, `14 §2.1`, `14 §8.4`).
        let nat = Term::indformer(d(5), vec![]);
        let w = Term::indformer(d(0), vec![]);
        let arg = Term::pi(nat, w); // Nat → W
        let mut ind = InductiveDecl {
            id: d(0),
            level_params: vec![],
            params: vec![],
            indices: vec![],
            level: Level::zero(),
            constructors: vec![ConstructorDecl {
                id: d(1),
                args: vec![arg],
                target_indices: vec![],
                type_: Term::Type(Level::zero()),
                recursive_positions: vec![],
            }],
            former_type: Term::Type(Level::zero()),
        };
        ind.build_types();
        assert!(check_positivity(&ind).is_ok(), "W-style is strictly positive");
        // K1.5: recursive_args now includes the W-style arg.
        let rec = recursive_args(&ind.constructors[0], d(0), 0);
        assert_eq!(rec.len(), 1);
        let (pos, branching_tel, _idxs) = &rec[0];
        assert_eq!(*pos, 0);
        assert_eq!(branching_tel.len(), 1, "one Π-binder (Nat)");
    }

    #[test]
    fn w_style_branching_domain_not_d_free_rejected() {
        // data Bad5 : Type 0 where mk : (Bad5 → Bad5) → Bad5
        // The branching domain `Bad5` is not D-free: §8.2 checks the domain at
        // flipped (−) polarity and finds D there, so it rejects.
        // `14 §2.1` "B contains no occurrence of D"; conformance `wstyle-branching-
        // domain-not-d-free-rejected`. Soundness guard: gate-removal must not
        // relax the polarity check on the branching domain.
        let bad5 = Term::indformer(d(0), vec![]);
        // (Bad5 → Bad5) → Bad5: Pi(Pi(Bad5, Bad5), Bad5)
        let neg_arg = Term::pi(Term::pi(bad5.clone(), bad5.clone()), bad5);
        let mut ind = InductiveDecl {
            id: d(0),
            level_params: vec![],
            params: vec![],
            indices: vec![],
            level: Level::zero(),
            constructors: vec![ConstructorDecl {
                id: d(1),
                args: vec![neg_arg],
                target_indices: vec![],
                type_: Term::Type(Level::zero()),
                recursive_positions: vec![],
            }],
            former_type: Term::Type(Level::zero()),
        };
        ind.build_types();
        assert!(
            check_positivity(&ind).is_err(),
            "branching domain not D-free must be rejected by §8.2 polarity check"
        );
    }

    // --- B3a regression: iota_reduct must not panic on arity mismatch ---
    // (Architect review on dec_2hnhhdb7mrxze.) `raw_wf` checks only scoping for
    // an `Elim`; `whnf` calls `iota_reduct` on any constructor-headed scrutinee.
    // A raw-well-formed `Elim` with too few params/methods/level-args must
    // return `KernelError::BadEliminator`, never panic.

    fn nat_decl() -> InductiveDecl {
        // data Nat : Type 0 where zero : Nat ; suc : Nat → Nat
        let mut ind = InductiveDecl {
            id: d(0),
            level_params: vec![],
            params: vec![],
            indices: vec![],
            level: Level::zero(),
            constructors: vec![
                ConstructorDecl {
                    id: d(1),
                    args: vec![],
                    target_indices: vec![],
                    type_: Term::Type(Level::zero()),
                    recursive_positions: vec![],
                },
                ConstructorDecl {
                    id: d(2),
                    args: vec![Term::indformer(d(0), vec![])],
                    target_indices: vec![],
                    type_: Term::Type(Level::zero()),
                    recursive_positions: vec![],
                },
            ],
            former_type: Term::Type(Level::zero()),
        };
        ind.build_types();
        ind
    }

    #[test]
    fn iota_reduct_wrong_methods_arity_errors_not_panics() {
        let ind = nat_decl();
        // `zero` (k=0) has no args; ctor_all_args = [] (m=0, n=0). But supply
        // only ONE method (Nat has two constructors) → must error, not panic.
        let motive = Term::lam(Term::indformer(d(0), vec![]), Term::indformer(d(0), vec![]));
        let res = iota_reduct(
            &ind,
            0,
            &[],
            &[],
            &motive,
            std::slice::from_ref(&motive), // 1 method, expected 2
            &[],
        );
        assert!(matches!(res, Err(KernelError::BadEliminator(_))));
    }

    #[test]
    fn iota_reduct_wrong_ctor_arity_errors_not_panics() {
        let ind = nat_decl();
        // `suc` (k=1) expects 1 ctor arg; supply 0 → must error, not panic.
        let motive = Term::lam(Term::indformer(d(0), vec![]), Term::indformer(d(0), vec![]));
        let m1 = Term::lam(
            Term::indformer(d(0), vec![]),
            Term::lam(Term::indformer(d(0), vec![]), Term::indformer(d(0), vec![])),
        );
        let res = iota_reduct(
            &ind,
            1, // suc
            &[],
            &[],
            &motive,
            &[motive.clone(), m1],
            &[], // 0 ctor args, expected 1
        );
        assert!(matches!(res, Err(KernelError::BadEliminator(_))));
    }

    #[test]
    fn iota_reduct_wrong_level_arity_errors_not_panics() {
        // A level-polymorphic family: supply the wrong number of level args.
        let mut ind = nat_decl();
        ind.level_params = vec![LevelVar(0)]; // one level param
        let motive = Term::lam(Term::indformer(d(0), vec![]), Term::indformer(d(0), vec![]));
        let res = iota_reduct(
            &ind,
            0,
            &[Level::zero(), Level::zero()], // 2 level args, expected 1
            &[],
            &motive,
            &[motive.clone(), motive.clone()],
            &[],
        );
        assert!(matches!(res, Err(KernelError::BadEliminator(_))));
    }
}
