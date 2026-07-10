//! K2 observational reduction — `Eq`-by-type, `cast`-by-type, and the derived
//! `J` (`15-identity.md`, `16-observational.md` §2–§4). This is the ADR-0005
//! core: equality is a proposition computed by recursion on the type, and
//! `cast` transports along a type-equality computing from the **endpoints**
//! (never the proof `e`), so `J` reduces on non-`refl` equalities.
//!
//! `Eq` and `cast` form a **single mutual reduction system** (`16 §3.3`):
//! `Eq` at a compound type calls `cast` on component types (Σ, inductive), and
//! `cast` at a compound type recovers its sub-equality proofs by *projecting*
//! the proof `e` (`Eq Type (…) (…)` reduces to a Σ/Π of component equalities, so
//! `e.1`, `e.2` are the sub-proofs). Termination is structural on the type tree.
//!
//! Called from [`crate::conv::whnf`]. A neutral head type, mismatched heads, or
//! a sub-proof the kernel cannot build leaves the `Eq`/`cast` **neutral**
//! (stuck) — sound: a stuck `Eq`/`cast` simply does not reduce.

use crate::conv::{convert_type, whnf};
use crate::env::{Context, GlobalEnv};
use crate::inductive::peel_app;
use crate::subst::{apply_args, subst0, subst_levels, subst_outer, subst_tel, weaken};
use crate::term::Term;

// --- prelude proposition terms (`16 §1.3`) ---

/// `Top : Ω_0` — the truth proposition (the prelude constant).
pub fn top_term(env: &GlobalEnv) -> Term {
    Term::Const {
        id: env.top_id(),
        level_args: Vec::new(),
    }
}

/// `Bottom : Ω_0` — the falsity proposition (the prelude constant).
pub fn bottom_term(env: &GlobalEnv) -> Term {
    Term::Const {
        id: env.bottom_id(),
        level_args: Vec::new(),
    }
}

/// `tt : Top` — `Top`'s sole introduction (the prelude constant, `16 §1.3`,
/// K5).
pub fn tt_term(env: &GlobalEnv) -> Term {
    Term::Const {
        id: env.tt_id(),
        level_args: Vec::new(),
    }
}

/// Is `t` a `refl` proof?
fn is_refl(t: &Term) -> bool {
    matches!(t, Term::Refl(_))
}

/// `sym` of a type-equality proof (`16 §4`): `p : Eq Type A B ⇒ sym p : Eq Type
/// B A`. Since `Eq Type _ _ : Ω` (proof-irrelevant) and `cast` never inspects the
/// proof, this only needs to be well-typed; `sym (refl X) = refl X`, and a
/// pair-structured proof is reversed componentwise. Anything else is returned
/// unchanged (the dependent `cast` is then stuck — sound).
fn mk_sym(proof: &Term) -> Term {
    match proof {
        Term::Refl(x) => Term::Refl(x.clone()),
        Term::Pair(p, q) => Term::Pair(Box::new(mk_sym(p)), Box::new(mk_sym(q))),
        other => other.clone(),
    }
}

// ===========================================================================
// Eq-by-type (`16 §2.2`)
// ===========================================================================

/// Reduce `Eq ty a b` by recursion on the (already-whnf'd) type `ty`
/// (`16 §2.2`). Returns the reduct, or `None` if `ty` is neutral. `ty` is whnf'd
/// by the caller.
pub fn eq_reduce(env: &GlobalEnv, ctx: &Context, ty: &Term, a: &Term, b: &Term) -> Option<Term> {
    match ty {
        Term::Pi(a1, b1) => Some(eq_at_pi(a1, b1, a, b)),
        Term::Sigma(a1, b1) => Some(eq_at_sigma(env, ctx, a1, b1, a, b)),
        Term::Omega(_) => Some(eq_at_omega(a, b)),
        Term::Type(_) => eq_at_type(env, ctx, a, b),
        Term::Trunc(_) => Some(top_term(env)),
        Term::Quot(_, r) => eq_at_quot(r, a, b),
        Term::App(_, _) | Term::IndFormer { .. } => eq_at_inductive(env, ctx, ty, a, b),
        // A primitive type with a registered decidable-equality certificate
        // (ADR 0013 Layer 2) decides `Eq` between two checked literals by
        // value; general opt-in gate (`GlobalEnv::deceq_cert`), not
        // hardcoded to any specific primitive. An unregistered primitive
        // type falls through to the neutral default below, unchanged.
        Term::Const { id, .. } if env.deceq_cert(*id).is_some() => {
            eq_at_registered_literal(env, ctx, a, b)
        }
        // Primitive types enter as global declarations (`11 §1`); K2 defines no
        // `primEq` reduction yet — `Eq` at a primitive type stays neutral.
        _ => None,
    }
}

/// `Eq ty (IntLit m) (IntLit n) ⇝ Top if m == n else Bottom` for a `ty` with
/// a registered decidable-equality certificate (ADR 0013 Layer 2, `docs/adr/
/// 0013-int-decidable-equality-kernel-posture.md`). Neutral if either
/// operand is not (already, or after `whnf`) a literal — covers abstract
/// variables at any binder depth, and any other non-canonical operand
/// shape, with the same `None` default every other reduction arm uses.
///
/// `m == n` decides `num_bigint::BigInt` value equality — the SAME
/// crate/type/operator the registered primitive's runtime decider computes
/// (e.g. `eq_int`'s interp-side implementation), not a second,
/// independently-written comparison; pinned by a cross-layer test, not left
/// as an inspection-only claim.
fn eq_at_registered_literal(env: &GlobalEnv, ctx: &Context, a: &Term, b: &Term) -> Option<Term> {
    let a_w = whnf(env, ctx, a);
    let b_w = whnf(env, ctx, b);
    match (&a_w, &b_w) {
        (Term::IntLit(m), Term::IntLit(n)) => {
            Some(if m == n { top_term(env) } else { bottom_term(env) })
        }
        _ => None,
    }
}

/// `Eq ((x:A1)→B1) f g ⇝ (x:A1) → Eq (B1 x) (f x) (g x)` — funext definitional
/// (`16 §2.2`). The body's `Eq` reduces lazily by later `whnf` calls.
fn eq_at_pi(a1: &Term, b1: &Term, f: &Term, g: &Term) -> Term {
    // Under the binder `x : A1` (de Bruijn 0): `B1 x`, `f x`, `g x`. `B1` already
    // has `x` at index 0 (it is the Π codomain), so `B1 x` is `B1` itself; `f`,`g`
    // live in the outer context and are weakened by 1 past the new binder.
    let b1_x = b1.clone();
    let f_x = Term::app(weaken(f, 1), Term::var(0));
    let g_x = Term::app(weaken(g, 1), Term::var(0));
    Term::pi(
        a1.clone(),
        Term::Eq(Box::new(b1_x), Box::new(f_x), Box::new(g_x)),
    )
}

/// `Eq ((x:A1)×B1) p q ⇝ Eq A1 p.1 q.1 and Eq (B1 q.1) (cast (B1 p.1) (B1 q.1)
/// (cong (x.B1 x) eq-fst) p.2) q.2` (`16 §2.2`). The "and" is a Σ. The transport
/// proof for `p.2` is `Eq Type (B1 p.1) (B1 q.1)`; when `B1` is non-dependent
/// this is `refl` (and the `cast` reduces to `p.2` by regularity).
fn eq_at_sigma(env: &GlobalEnv, ctx: &Context, a1: &Term, b1: &Term, p: &Term, q: &Term) -> Term {
    let _ = (env, ctx);
    let p1 = Term::proj1(p.clone());
    let q1 = Term::proj1(q.clone());
    let eq_fst = Term::Eq(
        Box::new(a1.clone()),
        Box::new(p1.clone()),
        Box::new(q1.clone()),
    );
    let b1_p1 = subst0(b1, &p1);
    let b1_q1 = subst0(b1, &q1);
    // `cong B1 eq-fst` placeholder: `refl` (non-dep ⇒ exact; dep ⇒ the cast is
    // stuck, sound — K2 conformance does not exercise dependent-Σ equality).
    let proof = Term::Refl(Box::new(b1_p1.clone()));
    let p2_cast = Term::Cast(
        Box::new(b1_p1),
        Box::new(b1_q1.clone()),
        Box::new(proof),
        Box::new(Term::proj2(p.clone())),
    );
    let second = Term::Eq(
        Box::new(b1_q1),
        Box::new(p2_cast),
        Box::new(Term::proj2(q.clone())),
    );
    // `second` is built in the outer context. As the codomain of the Sigma
    // proof, it lives under the newly-bound first-component equality proof.
    Term::sigma(eq_fst, weaken(&second, 1))
}

/// `Eq Ω P Q ⇝ (P → Q) and (Q → P)` — propext definitional (`16 §2.2`).
fn eq_at_omega(p: &Term, q: &Term) -> Term {
    let p_to_q = Term::pi(p.clone(), q.clone()); // (x:P) → Q
    let q_to_p = Term::pi(q.clone(), p.clone()); // (x:Q) → P
    Term::sigma(p_to_q, q_to_p) // (P→Q) and (Q→P)
}

/// `Eq (A/R) [a] [b] ⇝ R a b` — quotient equality is the relation (`16 §2.2`,
/// §5). `a`,`b` are class representatives `[a']`,`[b']`; a neutral representative
/// leaves `Eq` neutral.
fn eq_at_quot(r: &Term, a: &Term, b: &Term) -> Option<Term> {
    match (a, b) {
        (Term::QuotClass(a0), Term::QuotClass(b0)) => {
            Some(apply_args(r.clone(), &[(**a0).clone(), (**b0).clone()]))
        }
        _ => None,
    }
}

/// Structural type equality `Eq Type A B` (`16 §2.2`, §3). Same universe head
/// with equal level ⇒ `Top`; different heads (Π/Σ/Ω/Type vs a different head) ⇒
/// `Bottom`; compound heads (Π/Π, Σ/Σ, inductive, quotient) and neutral heads ⇒
/// neutral — a full structural type-equality needs congruence over families,
/// deferred (sound: a stuck `Eq Type` is fine; K2 conformance does not exercise
/// it directly).
fn eq_at_type(env: &GlobalEnv, ctx: &Context, a: &Term, b: &Term) -> Option<Term> {
    let a_w = whnf(env, ctx, a);
    let b_w = whnf(env, ctx, b);
    match (&a_w, &b_w) {
        (Term::Type(l1), Term::Type(l2)) => {
            if l1.equiv(l2) {
                Some(top_term(env))
            } else {
                Some(bottom_term(env))
            }
        }
        (Term::Omega(l1), Term::Omega(l2)) => {
            if l1.equiv(l2) {
                Some(top_term(env))
            } else {
                Some(bottom_term(env))
            }
        }
        // Different universe/compound heads ⇒ the empty proposition.
        (Term::Pi(_, _) | Term::Sigma(_, _) | Term::Omega(_) | Term::Type(_), _)
        | (_, Term::Pi(_, _) | Term::Sigma(_, _) | Term::Omega(_) | Term::Type(_)) => {
            Some(bottom_term(env))
        }
        // Π/Π, Σ/Σ, inductive, quotient, neutral: leave neutral for K2.
        _ => None,
    }
}

/// `Eq (D Δp ī) (c_k ā) (c_l b̄)` — equality at an inductive family (`16 §2.2`).
/// Same constructor ⇒ the conjunction of argument-equalities, with later
/// arguments transported along earlier-argument equalities (the dependent
/// telescope `cast`s); different constructors ⇒ `Bottom`; a neutral scrutinee
/// ⇒ neutral.
fn eq_at_inductive(env: &GlobalEnv, ctx: &Context, ty: &Term, a: &Term, b: &Term) -> Option<Term> {
    let (head, _ty_args) = peel_app(ty);
    let d_id = match head {
        Term::IndFormer { id, .. } => id,
        _ => return None,
    };
    let ind = env.inductive(d_id)?;
    let m = ind.params.len();
    let a_w = whnf(env, ctx, a);
    let b_w = whnf(env, ctx, b);
    let (a_head, a_args) = peel_app(&a_w);
    let (b_head, b_args) = peel_app(&b_w);
    let (a_ctor, a_level_args, a_ctor_args) = match a_head {
        Term::Constructor { id, level_args, .. } => (id, level_args, a_args),
        _ => return None, // neutral scrutinee ⇒ neutral Eq
    };
    let (b_ctor, _b_level_args, b_ctor_args) = match b_head {
        Term::Constructor { id, level_args, .. } => (id, level_args, b_args),
        _ => return None,
    };
    if a_ctor_args.len() < m || b_ctor_args.len() < m {
        return None;
    }
    let a_bar = &a_ctor_args[m..];
    let b_bar = &b_ctor_args[m..];
    if a_ctor != b_ctor {
        return Some(bottom_term(env));
    }
    let (ind2, k) = env.constructor(a_ctor)?;
    if ind2.id != d_id {
        return None;
    }
    let c = &ind2.constructors[k];
    if a_bar.len() != c.args.len() || b_bar.len() != c.args.len() {
        return None; // arity guard — yes/no, never crash
    }
    let n = c.args.len();
    let a_param_args = &a_ctor_args[..m];
    let b_param_args = &b_ctor_args[..m];
    // Right-nested Σ (conjunction), `Top` the unit. A nullary ctor ⇒ `Top`.
    let mut acc = top_term(env);
    for j in (0..n).rev() {
        // `A_j` with the `m` params substituted; the `j` earlier-arg binders
        // (de Bruijn 0..j-1) remain. Instantiate them with the actual earlier
        // args — `a_bar[..j]` for the source, `b_bar[..j]` for the target — via
        // `subst_tel` (the K1-fixed telescope subst). This is what makes the
        // dependent-telescope detection sound: a non-dependent position has
        // `a_ty_j ≡ b_ty_j` (the earlier-arg subst is irrelevant), while a
        // dependent one (whose type mentions a differing earlier arg) does not.
        let a_ty_tpl = subst_levels(
            &subst_outer(&c.args[j], m, a_param_args, j),
            &ind2.level_params,
            &a_level_args,
        );
        let b_ty_tpl = subst_levels(
            &subst_outer(&c.args[j], m, b_param_args, j),
            &ind2.level_params,
            &a_level_args,
        );
        let a_ty_j = subst_tel(&a_ty_tpl, &a_bar[..j]);
        let b_ty_j = subst_tel(&b_ty_tpl, &b_bar[..j]);
        // Dependent telescope: when a_ty_j ≡ b_ty_j, compare directly
        // (non-dep position). When they differ, transport a_j to b_ty_j via
        // cast — cast ignores its proof (§3.4), so refl(b_ty_j) is a valid
        // Ω witness even though it has type Eq Type b_ty_j b_ty_j.
        let lhs = if convert_type(env, ctx, &a_ty_j, &b_ty_j) {
            a_bar[j].clone()
        } else {
            Term::Cast(
                Box::new(a_ty_j),
                Box::new(b_ty_j.clone()),
                Box::new(Term::Refl(Box::new(b_ty_j.clone()))),
                Box::new(a_bar[j].clone()),
            )
        };
        let conjunct = Term::Eq(
            Box::new(b_ty_j.clone()),
            Box::new(lhs),
            Box::new(b_bar[j].clone()),
        );
        acc = Term::sigma(conjunct, acc);
    }
    Some(strip_trailing_top(acc))
}

/// Peel the trailing `Top` (conjunction unit): a single argument yields its
/// lone `Eq` conjunct; multiple yield the right-nested `Σ` ending in the last;
/// a nullary constructor yields `Top`.
fn strip_trailing_top(t: Term) -> Term {
    match t {
        Term::Sigma(first, rest) => match *rest {
            Term::Const { .. } => *first,
            other => Term::sigma(*first, strip_trailing_top(other)),
        },
        other => other,
    }
}

// ===========================================================================
// cast-by-type (`16 §3.2`)
// ===========================================================================

/// Reduce `cast a b e t` by recursion on the (whnf'd) types `a`,`b` (`16 §3.2`).
/// Returns the reduct, or `None` if the cast is stuck. `a`,`b` are whnf'd by the
/// caller. The proof `e` is **never inspected** for content — `cast` computes
/// from the endpoints; sub-equality proofs are *projected* from `e` (`16 §3.4`).
pub fn cast_reduce(
    env: &GlobalEnv,
    ctx: &Context,
    a: &Term,
    b: &Term,
    e: &Term,
    t: &Term,
) -> Option<Term> {
    // Regularity (`16 §3.2`): `cast A A refl a ⇝ a`. More generally, if `A ≡ B`
    // the transport is the identity (`e` is proof-irrelevant — `Eq Type A B : Ω`
    // when `A ≡ B`).
    if convert_type(env, ctx, a, b) {
        return Some(t.clone());
    }
    match (a, b) {
        (Term::Pi(a1, b1), Term::Pi(a2, b2)) => Some(cast_at_pi(a1, b1, a2, b2, e, t)),
        (Term::Sigma(a1, b1), Term::Sigma(a2, b2)) => {
            Some(cast_at_sigma(env, ctx, a1, b1, a2, b2, e, t))
        }
        (Term::Omega(_), Term::Omega(_)) => Some(t.clone()), // cast Ω Ω e P ⇝ P
        (Term::App(_, _) | Term::IndFormer { .. }, Term::App(_, _) | Term::IndFormer { .. }) => {
            cast_at_inductive(env, ctx, a, b, e, t)
        }
        (Term::Quot(_, _), Term::Quot(_, _)) => cast_at_quot(a, b, e, t),
        // `cast Type Type (refl _) A ⇝ A`; non-refl type-equality at a universe
        // is (oracle) neutral (`16 §3.2`).
        (Term::Type(_), Term::Type(_)) if is_refl(e) => Some(t.clone()),
        // Mismatched heads, or a neutral type on either side ⇒ neutral cast.
        _ => None,
    }
}

/// `cast ((x:A1)→B1) ((x:A2)→B2) e f ⇝ λ(x:A2). cast (B1 (back x)) (B2 x)
/// (cod-eq x) (f (back x))` where `back x = cast A2 A1 (sym dom-eq) x`,
/// `dom-eq = e.1`, `cod-eq x = (e.2)(back x)` (`16 §3.2`). Sub-equality proofs
/// are projected from `e`.
fn cast_at_pi(a1: &Term, b1: &Term, a2: &Term, b2: &Term, e: &Term, f: &Term) -> Term {
    let dom_eq = Term::proj1(e.clone()); // e.1 : Eq Type A1 A2
    let sym_dom = mk_sym(&dom_eq);
    // back x = cast A2 A1 (sym dom-eq) x,  x:A2 at index 0 under the λ.
    let back_x = Term::Cast(
        Box::new(weaken(a2, 1)),
        Box::new(weaken(a1, 1)),
        Box::new(weaken(&sym_dom, 1)),
        Box::new(Term::var(0)),
    );
    let b1_back = subst0(b1, &back_x); // B1[back x / x]  (B1's var 0 is the Π's x)
    let b2_x = b2.clone(); // B2[x / x] = B2  (B2's var 0 is already the λ's x)
    let cod_eq_x = Term::app(weaken(&Term::proj2(e.clone()), 1), back_x.clone()); // (e.2)(back x)
    let f_back = Term::app(weaken(f, 1), back_x); // f (back x)
    Term::lam(
        a2.clone(),
        Term::Cast(
            Box::new(b1_back),
            Box::new(b2_x),
            Box::new(cod_eq_x),
            Box::new(f_back),
        ),
    )
}

/// `cast ((x:A1)×B1) ((x:A2)×B2) e p ⇝ (cast A1 A2 dom-eq p.1, cast (B1 p.1)
/// (B2 (cast A1 A2 dom-eq p.1)) cod-eq' p.2)` (`16 §3.2`). `dom-eq = e.1`,
/// `cod-eq' = (e.2)(p.1)`. No back-cast (Σ cast goes forward).
#[allow(clippy::too_many_arguments)]
fn cast_at_sigma(
    env: &GlobalEnv,
    ctx: &Context,
    a1: &Term,
    b1: &Term,
    a2: &Term,
    b2: &Term,
    e: &Term,
    p: &Term,
) -> Term {
    let _ = (env, ctx);
    let dom_eq = Term::proj1(e.clone()); // e.1 : Eq Type A1 A2
    let p1 = Term::proj1(p.clone());
    let p1_cast = Term::Cast(
        Box::new(a1.clone()),
        Box::new(a2.clone()),
        Box::new(dom_eq.clone()),
        Box::new(p1.clone()),
    );
    let b1_p1 = subst0(b1, &p1);
    let b2_p1c = subst0(b2, &p1_cast);
    let cod_eq_prime = Term::app(Term::proj2(e.clone()), p1.clone()); // (e.2)(p.1)
    let p2_cast = Term::Cast(
        Box::new(b1_p1),
        Box::new(b2_p1c),
        Box::new(cod_eq_prime),
        Box::new(Term::proj2(p.clone())),
    );
    Term::pair(p1_cast, p2_cast)
}

/// `cast (D Δp ī) (D Δp j̄) e (c_k ā) ⇝ c_k (cast A_1 A_1' eq_1 a_1, …)` — each
/// constructor argument is transported from its `i`-bar type to its `j`-bar type
/// (`16 §3.2`). The sub-equalities come from the `Eq Type (D ī) (D j̄)`
/// decomposition of `e`.
fn cast_at_inductive(
    env: &GlobalEnv,
    ctx: &Context,
    a: &Term,
    b: &Term,
    e: &Term,
    t: &Term,
) -> Option<Term> {
    let _ = e;
    let (a_head, a_args) = peel_app(a);
    let (b_head, b_args) = peel_app(b);
    let d_id = match a_head {
        Term::IndFormer { id, .. } => id,
        _ => return None,
    };
    let b_id = match b_head {
        Term::IndFormer { id, .. } => id,
        _ => return None,
    };
    if d_id != b_id {
        return None;
    }
    let ind = env.inductive(d_id)?;
    let m = ind.params.len();
    if a_args.len() < m || b_args.len() < m {
        return None;
    }
    let (t_head, t_args) = peel_app(t);
    let (ctor, level_args, ctor_args) = match t_head {
        Term::Constructor { id, level_args, .. } => (id, level_args, t_args),
        _ => return None, // neutral scrutinee ⇒ neutral cast
    };
    let (ind2, k) = env.constructor(ctor)?;
    if ind2.id != d_id {
        return None;
    }
    let c = &ind2.constructors[k];
    let ctor_arg_vals = &ctor_args[m..];
    if ctor_arg_vals.len() != c.args.len() {
        return None; // arity guard
    }
    let a_param_args = &a_args[..m]; // source family params (from `a = D p̄ ī`)
    let b_param_args = &b_args[..m]; // target family params (from `b = D p̄' j̄`)
    let i_bar = &a_args[m..]; // source result indices
    let j_bar = &b_args[m..]; // target result indices
    let n_ctor = c.args.len();

    // Phase 1: detect whether any index pair differs.
    let mut index_changed = false;
    for (ix, jx) in i_bar.iter().zip(j_bar.iter()) {
        if !convert_type(env, ctx, ix, jx) {
            index_changed = true;
            break;
        }
    }

    if !index_changed {
        // Indices agree — only a param change could remain. Transport each arg
        // from its source-param type to its target-param type; stuck if the
        // type depends on a differing param.
        let mut new_args: Vec<Term> = b_param_args.to_vec();
        for (j, val) in ctor_arg_vals.iter().enumerate() {
            let a_ty_tpl = subst_levels(
                &subst_outer(&c.args[j], m, a_param_args, j),
                &ind2.level_params,
                &level_args,
            );
            let b_ty_tpl = subst_levels(
                &subst_outer(&c.args[j], m, b_param_args, j),
                &ind2.level_params,
                &level_args,
            );
            let a_ty_j = subst_tel(&a_ty_tpl, &ctor_arg_vals[..j]);
            let b_ty_j = subst_tel(&b_ty_tpl, &ctor_arg_vals[..j]);
            if !convert_type(env, ctx, &a_ty_j, &b_ty_j) {
                return None;
            }
            new_args.push(val.clone());
        }
        return Some(apply_args(Term::Constructor { id: ctor, level_args }, &new_args));
    }

    // Index change present. Require params to agree; a mixed param+index
    // change is out of scope — conservative stuck (sound).
    for (ap, bp) in a_param_args.iter().zip(b_param_args.iter()) {
        if !convert_type(env, ctx, ap, bp) {
            return None;
        }
    }

    // Phase 2: for each differing index slot, peel the ctor heads and read
    // off which ctor arg positions are "forced" to target-index values. The
    // guard (§3.2): both sides must be headed by the SAME ctor; a neutral or
    // mismatched-ctor index ⇒ stuck.
    //
    // `c.target_indices[p]` (after param-subst, inner_depth=n_ctor) is a
    // template whose `Var(k)` slots identify ctor arg positions: nat_pos =
    // (n_ctor - 1) - k. The target inner value at that position gives the
    // forced value.
    let mut forced_values: Vec<Option<Term>> = vec![None; n_ctor];
    for p in 0..i_bar.len() {
        if convert_type(env, ctx, &i_bar[p], &j_bar[p]) {
            continue; // this index slot agrees — skip
        }
        let (i_head, i_inner) = peel_app(&i_bar[p]);
        let (j_head, j_inner) = peel_app(&j_bar[p]);
        let i_ctor = match i_head {
            Term::Constructor { id, .. } => id,
            _ => return None, // neutral index — stuck (§3.2 guard)
        };
        let j_ctor = match j_head {
            Term::Constructor { id, .. } => id,
            _ => return None,
        };
        if i_ctor != j_ctor || i_inner.len() != j_inner.len() {
            return None; // different ctors or arity mismatch — stuck
        }
        let ti = subst_levels(
            &subst_outer(&c.target_indices[p], m, a_param_args, n_ctor),
            &ind2.level_params,
            &level_args,
        );
        let (ti_head, ti_inner) = peel_app(&ti);
        let ti_ctor = match ti_head {
            Term::Constructor { id, .. } => id,
            _ => return None,
        };
        if ti_ctor != i_ctor || ti_inner.len() != i_inner.len() {
            return None;
        }
        for (ti_arg, j_val) in ti_inner.iter().zip(j_inner.iter()) {
            if let Term::Var(vi) = ti_arg {
                let vi = *vi as usize;
                if vi < n_ctor {
                    forced_values[(n_ctor - 1) - vi] = Some(j_val.clone());
                }
            }
        }
    }

    // Phase 3: rebuild the constructor's arg list.
    //   forced position  → target index value
    //   non-dep position → source arg (same type on both sides)
    //   dep position     → sub-cast from a_ty_j to b_ty_j
    //
    // `target_earlier` tracks target-side earlier arg values so that
    // `subst_tel(&b_ty_tpl, &target_earlier)` gives the correct b_ty_j for
    // dependent arg types (those whose type mentions an earlier forced arg).
    // cast ignores its proof (§3.4), so refl(a_ty_j) is a valid Ω witness.
    let mut new_args: Vec<Term> = b_param_args.to_vec();
    let mut target_earlier: Vec<Term> = vec![];
    for j in 0..n_ctor {
        let a_ty_tpl = subst_levels(
            &subst_outer(&c.args[j], m, a_param_args, j),
            &ind2.level_params,
            &level_args,
        );
        let b_ty_tpl = subst_levels(
            &subst_outer(&c.args[j], m, b_param_args, j),
            &ind2.level_params,
            &level_args,
        );
        let a_ty_j = subst_tel(&a_ty_tpl, &ctor_arg_vals[..j]);
        let b_ty_j = subst_tel(&b_ty_tpl, &target_earlier);
        let (new_val, target_val) = if let Some(fv) = &forced_values[j] {
            (fv.clone(), fv.clone())
        } else if convert_type(env, ctx, &a_ty_j, &b_ty_j) {
            (ctor_arg_vals[j].clone(), ctor_arg_vals[j].clone())
        } else {
            let cast_val = Term::Cast(
                Box::new(a_ty_j.clone()),
                Box::new(b_ty_j),
                Box::new(Term::Refl(Box::new(a_ty_j))),
                Box::new(ctor_arg_vals[j].clone()),
            );
            (cast_val.clone(), cast_val)
        };
        new_args.push(new_val);
        target_earlier.push(target_val);
    }
    Some(apply_args(Term::Constructor { id: ctor, level_args }, &new_args))
}

/// `cast (A1/R) (A2/S) e [a] ⇝ [cast A1 A2 e0 a]` where `e0 = e.1` is the
/// underlying type equality from the decomposition of `e` (`16 §3.2`). The class
/// structure is preserved; a non-class representative leaves the cast neutral.
fn cast_at_quot(a: &Term, b: &Term, e: &Term, t: &Term) -> Option<Term> {
    let e0 = Term::proj1(e.clone());
    let a_inner = match a {
        Term::Quot(x, _) => (**x).clone(),
        _ => return None,
    };
    let b_inner = match b {
        Term::Quot(y, _) => (**y).clone(),
        _ => return None,
    };
    match t {
        Term::QuotClass(a0) => Some(Term::QuotClass(Box::new(Term::Cast(
            Box::new(a_inner),
            Box::new(b_inner),
            Box::new(e0),
            Box::new((**a0).clone()),
        )))),
        _ => None,
    }
}

// ===========================================================================
// derived J (`15 §4`)
// ===========================================================================

/// Reduce `J motive base eq` (`15 §4`). `J-β`: when `eq` whnf's to `refl a`,
/// reduce to `base`. On a non-`refl` canonical proof, `J` reduces via the
/// `cast` construction (`15 §4.3`). When `eq` is neutral or the motive is not
/// constant, `J` stays neutral (stuck) — sound.
pub fn j_reduce(
    env: &GlobalEnv,
    ctx: &Context,
    motive: &Term,
    base: &Term,
    eq: &Term,
) -> Option<Term> {
    let eq_w = whnf(env, ctx, eq);
    if let Term::Refl(_a) = &eq_w {
        // J-β (`15 §4.2`): J A a P d a (refl a) ≡ d.
        return Some(base.clone());
    }
    j_nonrefl(env, ctx, motive, base, &eq_w)
}

/// `J` on a non-`refl` equality (`15 §4.3`): `J ≡ cast (P a (refl a)) (P b e)
/// pair-eq d`. Fires for every non-`refl` `e` — motive constancy is not a gate
/// (`§4.1`). pair-eq is a typing witness only, never inspected by cast (`§3.4`).
/// For a constant motive cast reduces by regularity; for a dependent motive cast
/// descends by type structure (`§3.2`).
fn j_nonrefl(
    env: &GlobalEnv,
    ctx: &Context,
    motive: &Term,
    base: &Term,
    eq: &Term,
) -> Option<Term> {
    let eq_ty = crate::check::infer(env, ctx, eq).ok()?;
    let (_a_type, a_idx, b_idx) = match whnf(env, ctx, &eq_ty) {
        Term::Eq(a_t, x, y) => ((*a_t).clone(), (*x).clone(), (*y).clone()),
        _ => return None,
    };
    let p_a_refl = apply_args(
        motive.clone(),
        &[a_idx.clone(), Term::Refl(Box::new(a_idx.clone()))],
    );
    let p_b_e = apply_args(motive.clone(), &[b_idx.clone(), eq.clone()]);
    // J-cast fires for every non-refl e (§4.1). pair-eq is a typing witness
    // only, never inspected by cast (§3.4).
    let pair_eq = Term::Refl(Box::new(p_a_refl.clone()));
    Some(Term::Cast(
        Box::new(p_a_refl),
        Box::new(p_b_e),
        Box::new(pair_eq),
        Box::new(base.clone()),
    ))
}
