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
//!   the concrete motive/params at a use site (`14 §3`).
//! - [`iota_reduct`] — the algorithmic ι-step `elim_D … (cₖ p̄ ā) ⇝ mₖ ā [IHs]`
//!   (`14 §7.3`), capture-avoiding, with induction hypotheses on structurally
//!   smaller recursive arguments.
//!
//! K1 scope: **direct** recursive arguments (`D Δ_p t̄` as a constructor arg
//! type). Π-bound recursive arguments (W-style `(Nat → D) → D`) are
//! strictly-positive (`14 §2`) but their eliminator's ι needs a Π-abstracted
//! induction hypothesis; that is deferred to K1.5, so such declarations are
//! rejected at admission with a precise reason (`14 §8.4`). This is a
//! conservative, sound K1 boundary — it never admits an eliminator the kernel
//! cannot reduce.

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

/// Reject Π-bound recursive arguments (W-style) in K1 — their ι needs a
/// Π-abstracted induction hypothesis, deferred to K1.5 (`14 §8.4`). Direct
/// recursive arguments (`D Δ_p t̄`) are admitted.
pub fn check_no_pi_bound_recursive(ind: &InductiveDecl) -> KernelResult<()> {
    let d = ind.id;
    for c in &ind.constructors {
        for (j, a) in c.args.iter().enumerate() {
            let (pis, body) = peel_pi(a);
            let (head, _args) = peel_app(&body);
            if !pis.is_empty() && matches!(head, Term::IndFormer { id, .. } if id == d) {
                return Err(KernelError::PositivityViolation(format!(
                    "Π-bound recursive argument in constructor {:?} arg {j} (W-style) \
                     is strictly positive but its eliminator ι is deferred to K1.5",
                    c.id
                )));
            }
        }
    }
    Ok(())
}

/// The direct recursive arguments of a constructor: `(arg_position,
/// index_exprs)` for each arg whose type is `D Δ_p t̄` (no leading Π binders).
/// `index_exprs` are in context `[Δ_p, Δₖ]` (reference params and earlier
/// args), used to build induction hypotheses.
pub fn recursive_args(c: &ConstructorDecl, d: GlobalId, m: usize) -> Vec<(usize, Vec<Term>)> {
    let mut out = Vec::new();
    for (j, a) in c.args.iter().enumerate() {
        let (pis, body) = peel_pi(a);
        if !pis.is_empty() {
            continue; // Π-bound recursive handled by check_no_pi_bound_recursive.
        }
        let (head, args) = peel_app(&body);
        if let Term::IndFormer { id, .. } = head {
            if id == d && args.len() >= m {
                out.push((j, args[m..].to_vec()));
            }
        }
    }
    out
}

/// The dependent eliminator's method type for constructor `k`:
/// `Π Δₖ. Π (IH₁…IH_p). M t̄ₖ (cₖ p̄ ā)` (`14 §3`), in the caller's context Γ.
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
    let mut ty = conclusion;
    for j in (0..p).rev() {
        let (pos, idxs) = &rec[j];
        // IH_j type in context [Γ, args, ih₁..ih_{j-1}] (depth ctx_depth+n+j).
        let m_w_ih = weaken(motive, (n + j) as i64);
        // The recursive arg's index exprs live in `[Δ_p, args_before_pos]`
        // (depth 1+pos): substitute the params (weakening by `pos`) then lift
        // past the remaining args and the preceding IHs (n-pos+j binders).
        let idxs_w: Vec<Term> = idxs
            .iter()
            .map(|t| {
                weaken(
                    &subst_levels(
                        &subst_outer(t, m, params, *pos),
                        &ind.level_params,
                        level_args,
                    ),
                    (n - pos + j) as i64,
                )
            })
            .collect();
        // a_{pos}' at index (n - 1 - pos + j) in [Γ, args, ih₁..ih_{j-1}].
        let a_var = Term::var(n - 1 - pos + j);
        let mut ih_ty = m_w_ih;
        for ix in &idxs_w {
            ih_ty = Term::app(ih_ty, ix.clone());
        }
        ih_ty = Term::app(ih_ty, a_var);
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
    // Induction hypotheses: `elim_D p̄ M m̄ idx(a_j) a_j` for each recursive arg.
    let mut ihs: Vec<Term> = Vec::new();
    for (pos, idxs) in &rec {
        let a_j = &ctor_args[*pos];
        // idx(a_j): the index exprs live in `[Δ_p, args_before_pos]` (depth
        // 1+pos) — substitute the params (weakening by `pos`), then the actual
        // args before `pos`, landing in the caller's context Γ.
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
    use crate::term::Level;

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
        assert!(check_no_pi_bound_recursive(&ind).is_ok());
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
    fn w_style_pi_bound_recursive_rejected_in_k1() {
        // data W : Type 0 where mk : (Nat → W) → W   (strictly positive, but
        // Π-bound recursive — deferred to K1.5).
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
        assert!(check_positivity(&ind).is_ok()); // strictly positive
        assert!(
            check_no_pi_bound_recursive(&ind).is_err(),
            "W-style Π-bound recursive deferred to K1.5"
        );
    }
}
