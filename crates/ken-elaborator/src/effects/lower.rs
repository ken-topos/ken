//! Kernel-term lowering: emit `Term::Elim` over ITree (`36 §2.2`, `14 §3.1`).
//!
//! Bridges the elaborator's ITree analysis model (`itree.rs`) and the kernel's
//! Term language. Every function here emits a `Term::Elim { fam: itree_id, … }`
//! that can be passed to `ken_kernel::infer` or `ken_kernel::check` for
//! independent verification — mis-lowering produces a type error, not
//! unsoundness.
//!
//! ## De Bruijn discipline
//!
//! Every builder takes caller-context terms and correctly applies `weaken` when
//! crossing binder boundaries:
//! - Terms from context C under one lambda → `weaken(t, 1)`.
//! - Terms from context C inside a `Pi(A, B)` body → `weaken(t, 1)`.
//! - Under two lambdas → `weaken(t, 2)`. Etc.
//! Closed terms (`IndFormer`, `Constructor`, …) weaken trivially.
//!
//! ## Scope
//!
//! Lowering the simplified kernel ITree (fixed `Nat`-response, no named-effect
//! dispatch). The full named-effect dispatch is a downstream WP (requires
//! kernel-level effect-tag equality, `36 §5.2`).

use ken_kernel::subst::weaken;
use ken_kernel::{GlobalId, Term};

// ---------------------------------------------------------------------------
// Base builder
// ---------------------------------------------------------------------------

/// Build `elim_ITree M mr mv s` — the kernel form of ITree elimination.
///
/// `Term::Elim` layout:
/// - `fam  = itree_id`
/// - `params = [r_param]`     (the result type R)
/// - `motive  = M`
/// - `methods = [mr, mv]`     (one per constructor: Ret then Vis)
/// - `indices = []`           (ITree has no indices)
/// - `scrut   = s`
///
/// All terms must be in the caller's de Bruijn context; this function does no
/// shifting. Callers are responsible for structuring methods correctly (see
/// `lower_bind`, `lower_handler_fold_uniform` for examples).
pub fn lower_elim_itree(
    itree_id: GlobalId,
    r_param: Term,
    motive: Term,
    method_ret: Term,
    method_vis: Term,
    scrut: Term,
) -> Term {
    Term::Elim {
        fam: itree_id,
        level_args: vec![],
        params: vec![r_param],
        motive: Box::new(motive),
        methods: vec![method_ret, method_vis],
        indices: vec![],
        scrut: Box::new(scrut),
    }
}

// ---------------------------------------------------------------------------
// bind
// ---------------------------------------------------------------------------

/// Lower `bind t f` to a kernel `Term::Elim` (§2.2).
///
/// `bind : ITree R → (R → ITree S) → ITree S`
/// ```text
/// bind (Ret r)   f = f r
/// bind (Vis k)   f = Vis_S (λx. bind (k x) f)
/// ```
///
/// Kernel encoding via `elim_ITree`:
/// ```text
/// elim_ITree
///   (λ_:ITree R. ITree S)                      -- constant motive
///   (λ(r:R). f r)                               -- Ret method
///   (λ(k:Nat→ITree R). λ(ih:Nat→ITree S). Vis_S ih)  -- Vis method: re-wrap IH
///   t
/// ```
///
/// ## Arguments (all in the caller's de Bruijn context C)
/// - `itree_id` — `GlobalId` of the registered ITree inductive
/// - `nat_id`   — `GlobalId` of the registered Nat inductive
/// - `vis_id`   — `GlobalId` of the `Vis` constructor
/// - `r_type`   — the input result type `R` (e.g. `IndFormer(nat_id, [])`)
/// - `s_type`   — the output result type `S`
/// - `t`        — scrutinee of type `ITree R`
/// - `f_term`   — the continuation `f : R → ITree S`
pub fn lower_bind(
    itree_id: GlobalId,
    nat_id: GlobalId,
    vis_id: GlobalId,
    r_type: Term,
    s_type: Term,
    t: Term,
    f_term: Term,
) -> Term {
    // motive: λ(_:ITree R). ITree S
    // Domain (in C): App(ITree, r_type).
    // Body   (in C+[_]): App(ITree, weaken(s_type, 1)).
    let motive = Term::lam(
        Term::app(Term::indformer(itree_id, vec![]), r_type.clone()),
        Term::app(
            Term::indformer(itree_id, vec![]),
            weaken(&s_type, 1),
        ),
    );

    // method_ret: λ(r:R). f r
    // Domain (in C): r_type.
    // Body   (in C+[r]): App(weaken(f_term, 1), Var(0)).
    let method_ret = Term::lam(
        r_type.clone(),
        Term::app(weaken(&f_term, 1), Term::var(0)),
    );

    // method_vis: λ(k:Nat→ITree R). λ(ih:Nat→ITree S). Vis_S ih
    //
    // Outer Lam domain (in C):
    //   Pi(Nat, App(ITree, weaken(r_type, 1)))
    //   — inside the Pi body, r_type shifts by +1.
    //
    // Inner Lam domain (in C+[k]):
    //   Pi(Nat, App(ITree, weaken(s_type, 2)))
    //   — C+[k] shifts by 1; Pi body shifts by 1 more = 2 total.
    //
    // Body (in C+[k,ih]):
    //   App(App(Constructor(vis), weaken(s_type, 2)), Var(0))
    //   — C+[k,ih] shifts s_type by 2; Var(0)=ih.
    //
    // Note: s_type shifts by 2 in BOTH the inner Pi body and the outer body
    // — one coincidence, two distinct contexts (C+[k,_:Nat] vs C+[k,ih]) each
    // 2 binders deep from C.
    let outer_k_dom = Term::pi(
        Term::indformer(nat_id, vec![]),
        Term::app(Term::indformer(itree_id, vec![]), weaken(&r_type, 1)),
    );
    let inner_ih_dom = Term::pi(
        Term::indformer(nat_id, vec![]),
        Term::app(Term::indformer(itree_id, vec![]), weaken(&s_type, 2)),
    );
    let vis_s_ih = Term::app(
        Term::app(Term::constructor(vis_id, vec![]), weaken(&s_type, 2)),
        Term::var(0), // ih
    );
    let method_vis = Term::lam(outer_k_dom, Term::lam(inner_ih_dom, vis_s_ih));

    lower_elim_itree(itree_id, r_type, motive, method_ret, method_vis, t)
}

// ---------------------------------------------------------------------------
// handler_fold (uniform)
// ---------------------------------------------------------------------------

/// Lower a uniform `handler_fold t fixed_response` to a kernel `Term::Elim`.
///
/// "Uniform" = all `Vis` nodes are handled with the same Nat response
/// `fixed_response`. This corresponds to the simplified kernel ITree without
/// named-effect dispatch. Full dispatch (§5.2) requires a kernel-level
/// effect-tag equality test — a downstream WP.
///
/// ```text
/// handler_fold_uniform (Ret r) n = Ret_R r       -- identity on Ret
/// handler_fold_uniform (Vis k) n = ih n           -- apply IH at fixed response
/// ```
///
/// Kernel encoding:
/// ```text
/// elim_ITree
///   (λ_:ITree R. ITree R)                         -- constant motive
///   (λ(r:R). Ret_R r)                              -- Ret method: identity
///   (λ(k:Nat→ITree R). λ(ih:Nat→ITree R). ih fixed_response)  -- Vis: apply IH
///   t
/// ```
///
/// ## Arguments (all in the caller's de Bruijn context C)
/// - `itree_id`       — `GlobalId` of the registered ITree inductive
/// - `nat_id`         — `GlobalId` of the registered Nat inductive
/// - `ret_id`         — `GlobalId` of the `Ret` constructor
/// - `r_type`         — the result type `R`
/// - `fixed_response` — a `Term` of type `Nat`; the response provided to every Vis
/// - `t`              — scrutinee of type `ITree R`
pub fn lower_handler_fold_uniform(
    itree_id: GlobalId,
    nat_id: GlobalId,
    ret_id: GlobalId,
    r_type: Term,
    fixed_response: Term,
    t: Term,
) -> Term {
    // motive: λ(_:ITree R). ITree R
    let motive = Term::lam(
        Term::app(Term::indformer(itree_id, vec![]), r_type.clone()),
        Term::app(Term::indformer(itree_id, vec![]), weaken(&r_type, 1)),
    );

    // method_ret: λ(r:R). Ret_R r
    // Body (in C+[r]): App(App(Ret, weaken(r_type, 1)), Var(0)).
    let method_ret = Term::lam(
        r_type.clone(),
        Term::app(
            Term::app(Term::constructor(ret_id, vec![]), weaken(&r_type, 1)),
            Term::var(0),
        ),
    );

    // method_vis: λ(k:Nat→ITree R). λ(ih:Nat→ITree R). ih fixed_response
    //
    // Outer Lam domain (in C): Pi(Nat, App(ITree, weaken(r_type, 1)))
    // Inner Lam domain (in C+[k]): Pi(Nat, App(ITree, weaken(r_type, 2)))
    // Body (in C+[k,ih]): App(Var(0), weaken(fixed_response, 2))
    //   Var(0)=ih; fixed_response from C shifts by 2.
    let outer_k_dom = Term::pi(
        Term::indformer(nat_id, vec![]),
        Term::app(Term::indformer(itree_id, vec![]), weaken(&r_type, 1)),
    );
    let inner_ih_dom = Term::pi(
        Term::indformer(nat_id, vec![]),
        Term::app(Term::indformer(itree_id, vec![]), weaken(&r_type, 2)),
    );
    let body = Term::app(Term::var(0), weaken(&fixed_response, 2)); // ih fixed_response
    let method_vis = Term::lam(outer_k_dom, Term::lam(inner_ih_dom, body));

    lower_elim_itree(itree_id, r_type, motive, method_ret, method_vis, t)
}
