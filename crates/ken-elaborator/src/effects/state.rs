//! Direct `[State s]` surface build (VAL2 #10 / OQ-C·C2, `36 §4.5`).
//!
//! **Why hand-built, not surface `data`/`match`.** `Vis`'s continuation field
//! depends on the PRECEDING ctor argument's *value* (`Resp op`, `36 §4.5.2`)
//! — the surface `data` parser (`parser.rs::parse_ctor_decl`) only accepts a
//! flat list of type atoms, none referencing an earlier argument by name, and
//! `compile_match_matrix`'s `ColKind::Ih` computation (`elab.rs`, freshly
//! corrected by `wp/L-match-ih-fix`) always builds a FLAT induction-hypothesis
//! binder — it never Π-wraps for a W-style (Π-nested) recursive field the way
//! the kernel's own `method_type` (`ken-kernel/src/inductive.rs`) already does
//! generically. Reopening `ColKind`/`compile_match_matrix` for this would
//! touch the same soundness-adjacent machinery `L-match-ih-fix` just gated
//! through Architect review — avoided entirely here.
//!
//! Instead, every family/definition in this module is built directly via
//! `ken_kernel::declare_inductive`/`declare_def` — real kernel terms,
//! re-checked by `check`/`infer` exactly as any surface declaration would be
//! (AC1: `ken-kernel` itself is never touched). This is the same technique
//! `effects/lower.rs` already uses for the simplified `bind`/`handler_fold`,
//! extended to a genuinely dependent-response, coproduct-dispatching `ITree`.
//!
//! **No self-recursive `Const` reference anywhere in this file.** Every fold
//! (`bind`, `run_state`) is a SINGLE `Term::Elim` — the kernel's own
//! induction-hypothesis (`ih`) argument for a W-style recursive field already
//! computes the recursive result (`ih r` ≡ "the fold applied to `k r`"); there
//! is no need to build a self-referencing definition at all, so no SCT
//! interaction, no opaque pre-admit dance. Exactly the same shape
//! `effects/lower.rs::lower_bind` already uses for the simplified ITree.
//!
//! ## De Bruijn conventions (matches `data.rs`'s documented convention)
//! - A parameter/ctor-argument telescope is entered **left to right**; the
//!   LAST-declared entry sits at `Var(0)`, the FIRST at `Var(m-1)`.
//! - Each `params[j]`'s own stored type is relative to `[params[0..j]]` (j
//!   preceding entries — `Var(0)` = the immediately preceding entry).
//! - Each ctor `args[j]`'s stored type is relative to
//!   `[Δ_p, args[0..j]]` (all params, then the preceding ctor args).
//! - Helper [`v`] takes a logical (left-to-right, 0-indexed) binder position
//!   and the total bound-context length, returning the correct `Var`.
//!
//! ## The "generalize, then apply" trick (`run_state`'s Vis method)
//! A W-style method's higher-order IH (`ih : (r:Resp op) -> M(k r)`, i.e.
//! `ih : Resp op -> S -> ITree F RespF (Sigma A S)` here) mentions the OUTER
//! `op` free variable. Dispatching on `op`'s own substructure (peeling
//! `State`'s `Get`/`Put` out of the `Coproduct`) does NOT "refine" `ih`'s
//! already-fixed type — `op` never gets substituted merely by building
//! another elim that also scrutinizes it (no dependent-pattern-matching
//! machinery is invoked or needed). Instead, the inner elim's MOTIVE is
//! generalized to *produce a function expecting the exact ih-type for
//! whichever branch is taken*, and the real `ih` is applied to the elim's
//! result afterward: `(elim_Coproduct M inl_case inr_case op) ih`, where
//! `M := λo. (Resp(o) -> S -> ITree F RespF (Sigma A S)) -> (S -> ITree F RespF (Sigma A S))`.
//! `M(op)` is exactly the type `ih` already has (by the outer elim's own
//! `method_type` formula), so applying is always well-typed — ordinary motive
//! generalization, no new elaborator machinery.

use ken_kernel::subst::weaken;
use ken_kernel::{declare_def, declare_inductive, CtorSpec, GlobalEnv, GlobalId, InductiveSpec, Level, Term};

/// All `GlobalId`s the lifted `[State s]` surface registers.
#[derive(Clone, Copy)]
pub struct StateEffectIds {
    /// `ITree (E:Type) (Resp:E->Type) (R:Type)` — the lifted, dependent-
    /// response, effect-generic interaction tree (replaces the old
    /// Console-hardwired 1-param `ITree`).
    pub itree_id: GlobalId,
    pub ret_id: GlobalId,
    pub vis_id: GlobalId,
    /// `StateOp s = Get | Put s` (`36 §2.1`: `Op = Get | Put s`).
    pub state_op_id: GlobalId,
    pub get_id: GlobalId,
    pub put_id: GlobalId,
    /// `Coproduct a b = InL a | InR b` — the container coproduct (`36 §4.5.4` `⊕`).
    pub coproduct_id: GlobalId,
    pub inl_id: GlobalId,
    pub inr_id: GlobalId,
    /// `resp_state : (s:Type) -> StateOp s -> Type` (`Resp Get = s`,
    /// `Resp (Put _) = Unit`).
    pub resp_state_id: GlobalId,
    /// `resp_coproduct : (s f : Type) -> (RespF: f -> Type) -> Coproduct (StateOp s) f -> Type`
    /// — named-effect dispatch (c): peels `State`'s own response, passes any
    /// other effect's response through via the caller-supplied `RespF`.
    pub resp_coproduct_id: GlobalId,
    /// `bind : (e:Type)(resp:e->Type)(a b:Type) -> ITree e resp a -> (a -> ITree e resp b) -> ITree e resp b`.
    pub bind_id: GlobalId,
    /// `run_state : (s f : Type) -> (RespF: f -> Type) -> (a : Type) -> s ->
    ///   ITree (Coproduct (StateOp s) f) (resp_coproduct s f RespF) a ->
    ///   ITree f RespF (Sigma a s)` (`36 §4.5.3`, the `elim_ITree` fold at
    /// `F` — the return codomain is a genuine kernel `Sigma` pair, NOT the
    /// also-landed `Prod` inductive, `36 §4.5.3`).
    pub run_state_id: GlobalId,
    /// `get : (s f:Type) -> (RespF:f->Type) -> Unit -> ITree (Coproduct (StateOp s) f) (resp_coproduct s f RespF) s`.
    pub get_id_fn: GlobalId,
    /// `put : (s f:Type) -> (RespF:f->Type) -> s -> ITree (Coproduct (StateOp s) f) (resp_coproduct s f RespF) Unit`.
    pub put_id_fn: GlobalId,
}

fn lv0() -> Level {
    Level::zero()
}

fn ty0() -> Term {
    Term::ty(lv0())
}

/// `Type 1` — the classifier of `Type0`-VALUED terms (e.g. the literal term
/// `Type0` used as a large-elimination motive's body, or a type-former
/// application like `ITree e resp b`). Every motive in this file computes a
/// `Type0`-classified value per branch, so each motive's OWN ascribed type is
/// `Dom -> Type1` (one level up from the value it produces), never
/// `Dom -> Type0` — a "large elimination" motive throughout.
fn ty1() -> Term {
    Term::ty(lv0().suc())
}

/// `Var` for the logical (left-to-right, 0-indexed) binder position `i` in a
/// bound-context of total length `len` (last-bound = `Var(0)`).
fn v(len: usize, i: usize) -> Term {
    Term::var(len - 1 - i)
}

/// `Ck p̄ ā` / `f x1 x2 …` — left-fold application (declaration order).
fn apply_all(head: Term, args: &[Term]) -> Term {
    args.iter().fold(head, |f, a| Term::app(f, a.clone()))
}

fn pi_chain(domains: &[Term], body: Term) -> Term {
    domains.iter().rev().fold(body, |acc, d| Term::pi(d.clone(), acc))
}

fn lam_chain(domains: &[Term], body: Term) -> Term {
    domains.iter().rev().fold(body, |acc, d| Term::lam(d.clone(), acc))
}

/// An `Elim` motive `λ(_:domain). Type0` (or any other Type0-classified
/// codomain) is a bare introduction form (`Term::Lam`) — `infer` can never
/// synthesize a type for a bare λ/pair/refl/… (`check.rs`'s "cannot infer an
/// introduction form … use ascription"), and `infer_elim`'s own
/// `infer_motive_level` unconditionally calls `infer` on the motive (even
/// when the whole `Elim` is reached via `check`, since `check`'s fallback for
/// non-Lam/Pair/Ascript/… terms is itself "infer, then compare"). Every
/// motive here must therefore be wrapped in `Term::Ascript(motive, motive_ty)`
/// — exactly `k1p5_wstyle.rs`'s own dependent-motive test does — so `infer`'s
/// `Ascript` arm can dispatch to `check(motive, motive_ty)` instead.
fn ascribed(term: Term, ty: Term) -> Term {
    Term::Ascript(Box::new(term), Box::new(ty))
}

// ---------------------------------------------------------------------------
// ITree (lifted, 3-param, dependent-response Vis)
// ---------------------------------------------------------------------------

pub fn declare_itree(env: &mut GlobalEnv) -> Result<(GlobalId, GlobalId, GlobalId), String> {
    let itree = declare_inductive(env, |itree| InductiveSpec {
        level_params: vec![],
        params: vec![
            ty0(),                          // E : Type 0
            Term::pi(Term::var(0), ty0()),  // Resp : E -> Type 0, ctx [E]
            ty0(),                          // R : Type 0, ctx [E,Resp]
        ],
        indices: vec![],
        level: lv0(),
        constructors: vec![
            // Ret : R -> ITree E Resp R. ctx [E,Resp,R]: R=v(3,2).
            CtorSpec { args: vec![v(3, 2)], target_indices: vec![] },
            // Vis : (op:E) -> (Resp op -> ITree E Resp R) -> ITree E Resp R.
            CtorSpec {
                args: vec![
                    // arg0 (op), ctx [E,Resp,R]: E=v(3,0).
                    v(3, 0),
                    // arg1, ctx [E,Resp,R,op] (len=4): op=v(4,3), R=v(4,2),
                    // Resp=v(4,1), E=v(4,0).
                    Term::pi(
                        Term::app(v(4, 1), v(4, 3)), // Resp op
                        // codomain ctx [E,Resp,R,op,b] (len=5): op=v(5,3),
                        // R=v(5,2), Resp=v(5,1), E=v(5,0).
                        apply_all(Term::indformer(itree, vec![]), &[v(5, 0), v(5, 1), v(5, 2)]),
                    ),
                ],
                target_indices: vec![],
            },
        ],
    })
    .map_err(|e| format!("lifted ITree declaration failed: {e:?}"))?;
    let decl = env.inductive(itree).ok_or("ITree not found after declare")?;
    Ok((itree, decl.constructors[0].id, decl.constructors[1].id))
}

/// `StateOp s = Get | Put s`.
pub fn declare_state_op(env: &mut GlobalEnv) -> Result<(GlobalId, GlobalId, GlobalId), String> {
    let state_op = declare_inductive(env, |_| InductiveSpec {
        level_params: vec![],
        params: vec![ty0()], // s : Type 0
        indices: vec![],
        level: lv0(),
        constructors: vec![
            CtorSpec { args: vec![], target_indices: vec![] },       // Get
            CtorSpec { args: vec![v(1, 0)], target_indices: vec![] }, // Put : s -> StateOp s
        ],
    })
    .map_err(|e| format!("StateOp declaration failed: {e:?}"))?;
    let decl = env.inductive(state_op).ok_or("StateOp not found after declare")?;
    Ok((state_op, decl.constructors[0].id, decl.constructors[1].id))
}

/// `Coproduct a b = InL a | InR b` — the internal effect-signature
/// coproduct (L5, renamed from `Sum`; `docs/program/wp/either-neutral-
/// coproduct.md`). Non-dependent/`Type0`-monomorphic like `Either` above,
/// but kept hand-built here rather than migrated to a surface `data`
/// declaration — the deliberate risk-reduction this WP's rescope chose
/// (comment-only touch on the `eval.rs` D3.2 peel, no id-consumer
/// re-plumb). `state_effect_build_acceptance.rs`'s isolated bare-
/// `GlobalEnv` harness also calls this directly. Open for Architect to
/// re-open at the gate if a future WP wants the reflect-don't-extend win.
pub fn declare_coproduct(env: &mut GlobalEnv) -> Result<(GlobalId, GlobalId, GlobalId), String> {
    let coproduct = declare_inductive(env, |_| InductiveSpec {
        level_params: vec![],
        params: vec![ty0(), ty0()], // a, b : Type 0
        indices: vec![],
        level: lv0(),
        constructors: vec![
            CtorSpec { args: vec![v(2, 0)], target_indices: vec![] }, // InL : a -> Coproduct a b
            CtorSpec { args: vec![v(2, 1)], target_indices: vec![] }, // InR : b -> Coproduct a b
        ],
    })
    .map_err(|e| format!("Coproduct declaration failed: {e:?}"))?;
    let decl = env.inductive(coproduct).ok_or("Coproduct not found after declare")?;
    Ok((coproduct, decl.constructors[0].id, decl.constructors[1].id))
}

/// `resp_state : (s:Type) -> StateOp s -> Type = \s op. match op { Get => s ; Put _ => Unit }`.
pub fn declare_resp_state(env: &mut GlobalEnv, state_op_id: GlobalId, unit_id: GlobalId) -> Result<GlobalId, String> {
    // Elim node ctx [s, op] (len=2): s=v(2,0), op=v(2,1).
    let method_get = v(2, 0); // s
    let method_put = Term::lam(v(2, 0), Term::indformer(unit_id, vec![])); // \x:s. Unit
    let motive_dom = Term::app(Term::indformer(state_op_id, vec![]), v(2, 0));
    let motive = ascribed(Term::lam(motive_dom.clone(), ty0()), Term::pi(motive_dom, ty1()));
    let elim = Term::Elim {
        fam: state_op_id,
        level_args: vec![],
        params: vec![v(2, 0)], // s
        motive: Box::new(motive),
        methods: vec![method_get, method_put],
        indices: vec![],
        scrut: Box::new(v(2, 1)), // op
    };
    let body = lam_chain(&[ty0(), Term::app(Term::indformer(state_op_id, vec![]), Term::var(0))], elim);
    let ty = pi_chain(&[ty0(), Term::app(Term::indformer(state_op_id, vec![]), Term::var(0))], ty0());
    declare_def(env, vec![], ty, body).map_err(|e| format!("resp_state declaration failed: {e:?}"))
}

/// `resp_coproduct : (g h:Type) -> (rg:g->Type) -> (rh:h->Type) -> Coproduct g h -> Type`
/// `  = \g h rg rh op. match op { InL x => rg x ; InR y => rh y }`.
///
/// Generalizes the former State-first-pinned response combinator (`Coproduct
/// (StateOp s) f`) to an arbitrary coproduct `Coproduct g h`, given each summand's
/// own response family as an explicit parameter (`effect-composition` D1,
/// `evt_241dchcb5y6j8` / doc §D1.1). **Reducing `declare_def`, NEVER a
/// postulate — the hinge D2's `inject_l`/`inject_r` need**: `resp_coproduct g h rg rh
/// (InL x)` ι-reduces to `rg x`, `(InR y)` to `rh y`, definitionally, with no
/// wrap/reorder — the injected summand's OWN response, verbatim. One
/// **non-recursive** `Term::Elim` over `Coproduct` (`Coproduct` has no recursive field ⇒
/// trivially total, no SCT). State becomes the literal instance `resp_coproduct
/// (StateOp s) f (resp_state s) RespF` (§D1.3) — `get`/`put`/`run_state` below
/// apply the 4-arg form.
pub fn declare_resp_coproduct(env: &mut GlobalEnv, coproduct_id: GlobalId) -> Result<GlobalId, String> {
    // Elim node ctx [g,h,rg,rh,op] (len=5): g=v(5,0), h=v(5,1), rg=v(5,2), rh=v(5,3), op=v(5,4).
    let coproduct_ty = apply_all(Term::indformer(coproduct_id, vec![]), &[v(5, 0), v(5, 1)]);
    let motive = ascribed(Term::lam(coproduct_ty.clone(), ty0()), Term::pi(coproduct_ty, ty1()));
    // method_inl = \(x:g). rg x.
    //   Domain, ctx len5: g=v(5,0). Body, ctx len6 [g,h,rg,rh,op,x]: rg=v(6,2), x=v(6,5).
    let method_inl = Term::lam(v(5, 0), Term::app(v(6, 2), v(6, 5)));
    // method_inr = \(y:h). rh y.
    //   Domain, ctx len5: h=v(5,1). Body, ctx len6: rh=v(6,3), y=v(6,5).
    let method_inr = Term::lam(v(5, 1), Term::app(v(6, 3), v(6, 5)));
    let elim = Term::Elim {
        fam: coproduct_id,
        level_args: vec![],
        params: vec![v(5, 0), v(5, 1)], // [g, h]
        motive: Box::new(motive),
        methods: vec![method_inl, method_inr],
        indices: vec![],
        scrut: Box::new(v(5, 4)), // op
    };
    let domains = [
        ty0(),                          // g, ctx []
        ty0(),                          // h, ctx [g]
        Term::pi(Term::var(1), ty0()),  // rg : g -> Type, ctx [g,h] (len2): g=Var(1)
        Term::pi(Term::var(1), ty0()),  // rh : h -> Type, ctx [g,h,rg] (len3): h=Var(1)
        apply_all(Term::indformer(coproduct_id, vec![]), &[Term::var(3), Term::var(2)]), // op:Coproduct g h, ctx [g,h,rg,rh] (len4): g=Var(3),h=Var(2)
    ];
    let body = lam_chain(&domains, elim);
    let ty = pi_chain(&domains, ty0());
    declare_def(env, vec![], ty, body).map_err(|e| format!("resp_coproduct declaration failed: {e:?}"))
}

/// `inject_l : (g h:Type) -> (rg:g->Type) -> (rh:h->Type) -> (a:Type) ->
///   ITree g rg a -> ITree (Coproduct g h) (resp_coproduct g h rg rh) a`.
///
/// The left inclusion `g ↪ Coproduct g h` (`36 §2.4`'s `incl`, `effect-composition`
/// D2, doc §D2.1) — a structural `elim_ITree` re-tag: `Ret x ↦ Ret x`
/// (re-typed into the coproduct tree); `Vis op k ↦ Vis (InL op) ih`, where the
/// kernel-supplied IH `ih` already computes `inject_l … ∘ k` on the sub-tree
/// (no self-recursive `Const`, total by structural descent — the same
/// IH-reuse shape `bind` uses). **Coercion-free by D1's reduction (§D1.1,
/// §D2.2):** `ih`'s domain `rg op` is definitionally equal to `Vis (InL
/// op)`'s required continuation domain `resp_coproduct g h rg rh (InL op)`
/// (ι-reduces to `rg op`), so `ih` is used directly, no transport term.
///
/// Constant motive (the target type doesn't depend on the scrutinee) →
/// ordinary SMALL elimination, like `bind`'s — not the large-elim motives
/// `resp_state`/`resp_coproduct` use.
pub fn declare_inject_l(
    env: &mut GlobalEnv,
    itree_id: GlobalId,
    ret_id: GlobalId,
    vis_id: GlobalId,
    coproduct_id: GlobalId,
    resp_coproduct_id: GlobalId,
    inl_id: GlobalId,
) -> Result<GlobalId, String> {
    let itree_app = |len: usize, e_i: usize, resp_i: usize, r: Term| {
        apply_all(Term::indformer(itree_id, vec![]), &[v(len, e_i), v(len, resp_i), r])
    };
    let coproduct_gh = |len: usize| apply_all(Term::indformer(coproduct_id, vec![]), &[v(len, 0), v(len, 1)]);
    let resp_coproduct_ghrgrh = |len: usize| {
        apply_all(Term::const_(resp_coproduct_id, vec![]), &[v(len, 0), v(len, 1), v(len, 2), v(len, 3)])
    };

    // Outer ctx (declaration order): g, h, rg, rh, a, t (len=6). t : ITree g rg a.
    let motive_dom = itree_app(6, 0, 2, v(6, 4));
    let target_at6 = apply_all(Term::indformer(itree_id, vec![]), &[coproduct_gh(6), resp_coproduct_ghrgrh(6), v(6, 4)]);
    let motive = ascribed(
        Term::lam(motive_dom.clone(), weaken(&target_at6, 1)),
        Term::pi(motive_dom, ty0()),
    );

    // method_ret, ctx len6: \(x:a). Ret[Coproduct g h, resp_coproduct…, a] x.
    let method_ret = {
        let x_dom = v(6, 4);
        // Body, ctx len7 (x bound): local order g,h,rg,rh,a,t,x -> x=idx6=v(7,6).
        let ret_applied = apply_all(Term::constructor(ret_id, vec![]), &[coproduct_gh(7), resp_coproduct_ghrgrh(7), v(7, 4)]);
        Term::lam(x_dom, Term::app(ret_applied, v(7, 6)))
    };

    // method_vis, ctx len6: \(op:g). \(cont:rg op -> ITree g rg a).
    //   \(ih: rg op -> ITree (Coproduct g h)(resp_coproduct…) a). Vis … (InL g h op) ih.
    let method_vis = {
        let op_dom = v(6, 0); // g
        // cont domain, ctx len7 (op bound): rg=v(7,2), op=v(7,6).
        let cont_dom = Term::pi(Term::app(v(7, 2), v(7, 6)), itree_app(8, 0, 2, v(8, 4)));
        // ih domain, ctx len8 (op,cont bound): rg=v(8,2), op=v(8,6).
        let ih_dom = Term::pi(
            Term::app(v(8, 2), v(8, 6)),
            apply_all(Term::indformer(itree_id, vec![]), &[coproduct_gh(9), resp_coproduct_ghrgrh(9), v(9, 4)]),
        );
        // body, ctx len9 (op,cont,ih bound): op=v(9,6), ih=v(9,8).
        let inl_applied = apply_all(Term::constructor(inl_id, vec![]), &[v(9, 0), v(9, 1), v(9, 6)]);
        let vis_body = apply_all(
            Term::constructor(vis_id, vec![]),
            &[coproduct_gh(9), resp_coproduct_ghrgrh(9), v(9, 4), inl_applied, v(9, 8)],
        );
        Term::lam(op_dom, Term::lam(cont_dom, Term::lam(ih_dom, vis_body)))
    };

    let elim = Term::Elim {
        fam: itree_id,
        level_args: vec![],
        params: vec![v(6, 0), v(6, 2), v(6, 4)], // [g, rg, a]
        motive: Box::new(motive),
        methods: vec![method_ret, method_vis],
        indices: vec![],
        scrut: Box::new(v(6, 5)), // t
    };

    let d_g = ty0();
    let d_h = ty0();
    let d_rg = Term::pi(Term::var(1), ty0()); // ctx [g,h] (len2): g=Var(1)
    let d_rh = Term::pi(Term::var(1), ty0()); // ctx [g,h,rg] (len3): h=Var(1)
    let d_a = ty0();
    // d_t: ITree g rg a, ctx [g,h,rg,rh,a] (len5): g=Var(4), rg=Var(2), a=Var(0).
    let d_t = apply_all(Term::indformer(itree_id, vec![]), &[Term::var(4), Term::var(2), Term::var(0)]);
    let final_domains = [d_g, d_h, d_rg, d_rh, d_a, d_t];
    let body = lam_chain(&final_domains, elim);
    // ret_ty: ITree (Coproduct g h) (resp_coproduct g h rg rh) a, ctx len6: g=Var(5),h=Var(4),rg=Var(3),rh=Var(2),a=Var(1).
    let ret_coproduct = apply_all(Term::indformer(coproduct_id, vec![]), &[Term::var(5), Term::var(4)]);
    let ret_resp = apply_all(
        Term::const_(resp_coproduct_id, vec![]),
        &[Term::var(5), Term::var(4), Term::var(3), Term::var(2)],
    );
    let ret_ty = apply_all(Term::indformer(itree_id, vec![]), &[ret_coproduct, ret_resp, Term::var(1)]);
    let ty = pi_chain(&final_domains, ret_ty);
    declare_def(env, vec![], ty, body).map_err(|e| format!("inject_l declaration failed: {e:?}"))
}

/// `inject_r : (g h:Type) -> (rg:g->Type) -> (rh:h->Type) -> (a:Type) ->
///   ITree h rh a -> ITree (Coproduct g h) (resp_coproduct g h rg rh) a`.
///
/// The right inclusion `h ↪ Coproduct g h` — the exact mirror of [`declare_inject_l`]
/// (scrutinee `ITree h rh a`, tag `InR`); see that function's doc for the
/// full rationale (coercion-free by D1's reduction, small/constant-motive
/// elim, total by structural descent).
pub fn declare_inject_r(
    env: &mut GlobalEnv,
    itree_id: GlobalId,
    ret_id: GlobalId,
    vis_id: GlobalId,
    coproduct_id: GlobalId,
    resp_coproduct_id: GlobalId,
    inr_id: GlobalId,
) -> Result<GlobalId, String> {
    let itree_app = |len: usize, e_i: usize, resp_i: usize, r: Term| {
        apply_all(Term::indformer(itree_id, vec![]), &[v(len, e_i), v(len, resp_i), r])
    };
    let coproduct_gh = |len: usize| apply_all(Term::indformer(coproduct_id, vec![]), &[v(len, 0), v(len, 1)]);
    let resp_coproduct_ghrgrh = |len: usize| {
        apply_all(Term::const_(resp_coproduct_id, vec![]), &[v(len, 0), v(len, 1), v(len, 2), v(len, 3)])
    };

    // Outer ctx (declaration order): g, h, rg, rh, a, t (len=6). t : ITree h rh a.
    let motive_dom = itree_app(6, 1, 3, v(6, 4));
    let target_at6 = apply_all(Term::indformer(itree_id, vec![]), &[coproduct_gh(6), resp_coproduct_ghrgrh(6), v(6, 4)]);
    let motive = ascribed(
        Term::lam(motive_dom.clone(), weaken(&target_at6, 1)),
        Term::pi(motive_dom, ty0()),
    );

    let method_ret = {
        let x_dom = v(6, 4);
        let ret_applied = apply_all(Term::constructor(ret_id, vec![]), &[coproduct_gh(7), resp_coproduct_ghrgrh(7), v(7, 4)]);
        Term::lam(x_dom, Term::app(ret_applied, v(7, 6)))
    };

    let method_vis = {
        let op_dom = v(6, 1); // h
        // cont domain, ctx len7 (op bound): rh=v(7,3), op=v(7,6).
        let cont_dom = Term::pi(Term::app(v(7, 3), v(7, 6)), itree_app(8, 1, 3, v(8, 4)));
        // ih domain, ctx len8 (op,cont bound): rh=v(8,3), op=v(8,6).
        let ih_dom = Term::pi(
            Term::app(v(8, 3), v(8, 6)),
            apply_all(Term::indformer(itree_id, vec![]), &[coproduct_gh(9), resp_coproduct_ghrgrh(9), v(9, 4)]),
        );
        // body, ctx len9 (op,cont,ih bound): op=v(9,6), ih=v(9,8).
        let inr_applied = apply_all(Term::constructor(inr_id, vec![]), &[v(9, 0), v(9, 1), v(9, 6)]);
        let vis_body = apply_all(
            Term::constructor(vis_id, vec![]),
            &[coproduct_gh(9), resp_coproduct_ghrgrh(9), v(9, 4), inr_applied, v(9, 8)],
        );
        Term::lam(op_dom, Term::lam(cont_dom, Term::lam(ih_dom, vis_body)))
    };

    let elim = Term::Elim {
        fam: itree_id,
        level_args: vec![],
        params: vec![v(6, 1), v(6, 3), v(6, 4)], // [h, rh, a]
        motive: Box::new(motive),
        methods: vec![method_ret, method_vis],
        indices: vec![],
        scrut: Box::new(v(6, 5)), // t
    };

    let d_g = ty0();
    let d_h = ty0();
    let d_rg = Term::pi(Term::var(1), ty0());
    let d_rh = Term::pi(Term::var(1), ty0());
    let d_a = ty0();
    // d_t: ITree h rh a, ctx [g,h,rg,rh,a] (len5): h=Var(3), rh=Var(1), a=Var(0).
    let d_t = apply_all(Term::indformer(itree_id, vec![]), &[Term::var(3), Term::var(1), Term::var(0)]);
    let final_domains = [d_g, d_h, d_rg, d_rh, d_a, d_t];
    let body = lam_chain(&final_domains, elim);
    let ret_coproduct = apply_all(Term::indformer(coproduct_id, vec![]), &[Term::var(5), Term::var(4)]);
    let ret_resp = apply_all(
        Term::const_(resp_coproduct_id, vec![]),
        &[Term::var(5), Term::var(4), Term::var(3), Term::var(2)],
    );
    let ret_ty = apply_all(Term::indformer(itree_id, vec![]), &[ret_coproduct, ret_resp, Term::var(1)]);
    let ty = pi_chain(&final_domains, ret_ty);
    declare_def(env, vec![], ty, body).map_err(|e| format!("inject_r declaration failed: {e:?}"))
}

/// `bind : (e:Type)(resp:e->Type)(a b:Type) -> ITree e resp a -> (a -> ITree e resp b) -> ITree e resp b`.
///
/// `bind (Ret x) f = f x` ; `bind (Vis op k) f = Vis op ih` (`ih` — the
/// kernel-supplied W-style IH — already computes `\r. bind (k r) f`; no
/// self-reference is built or needed).
pub fn declare_bind(env: &mut GlobalEnv, itree_id: GlobalId, vis_id: GlobalId) -> Result<GlobalId, String> {
    // Outer domains, declaration order: e, resp, a, b, t, f.
    // Full outer ctx (len=6): e=v(6,0), resp=v(6,1), a=v(6,2), b=v(6,3), t=v(6,4), f=v(6,5).
    let itree_app = |len: usize, e_i: usize, resp_i: usize, r: Term| {
        apply_all(Term::indformer(itree_id, vec![]), &[v(len, e_i), v(len, resp_i), r])
    };

    // motive, ctx [e,resp,a,b,t,f] (len=6): \_:ITree e resp a. ITree e resp b.
    let motive_dom = itree_app(6, 0, 1, v(6, 2));
    // NOTE: unlike `resp_state`/`resp_coproduct` (whose motive body is the LITERAL
    // universe term `Type0`, itself classified by `Type1` — large
    // elimination), `bind`'s motive body is `itree_app(...)`, a type-FORMER
    // APPLICATION — its own classifier is whatever level `ITree` was declared
    // at (`Level::Zero`), i.e. `Type0` directly. Ordinary (small) elimination.
    let motive = ascribed(
        Term::lam(motive_dom.clone(), weaken(&itree_app(6, 0, 1, v(6, 3)), 1)),
        Term::pi(motive_dom, ty0()),
    );

    // method_ret, ctx [e,resp,a,b,t,f] (len=6): \x:a. f x.
    // Body ctx (len=7, x appended): f=v(7,5), x=v(7,6).
    let method_ret = Term::lam(v(6, 2), Term::app(v(7, 5), v(7, 6)));

    // method_vis, ctx [e,resp,a,b,t,f] (len=6):
    //   \(op:e). \(cont: Resp op -> ITree e resp a). \(ih: Resp op -> ITree e resp b).
    //     Vis e resp b op ih.
    // `cont` is the eliminator's mandatory Δ_k binder for Vis's OWN
    // (unused-in-body) continuation field — method arity is `Π Δk. Π IHs. M`
    // (`ken-kernel/inductive.rs::method_type`), so it must still be bound even
    // though the body only needs `ih`. Omitting it (as an earlier draft did)
    // shifts every subsequent de Bruijn index by one and was caught by the
    // kernel's own `check` (TypeMismatch on the IH's codomain) — a completeness
    // bug in THIS file, never a kernel concern.
    //
    // op domain, ctx len=6: e = v(6,0).
    let op_dom = v(6, 0);
    // cont domain, ctx [..,op] (len=7): Resp op -> ITree e resp a.
    //   resp=v(7,1), op=v(7,6) => "Resp op" = App(v(7,1),v(7,6)).
    //   codomain (ctx len=8, inside cont's own Pi binder): e=v(8,0),
    //   resp=v(8,1), a=v(8,2).
    let cont_dom = Term::pi(Term::app(v(7, 1), v(7, 6)), itree_app(8, 0, 1, v(8, 2)));
    // ih domain, ctx [..,op,cont] (len=8): Resp op -> ITree e resp b.
    //   resp=v(8,1), op=v(8,6) => "Resp op" = App(v(8,1),v(8,6)).
    //   codomain (ctx len=9, inside ih's own Pi binder): e=v(9,0),
    //   resp=v(9,1), b=v(9,3).
    let ih_dom = Term::pi(Term::app(v(8, 1), v(8, 6)), itree_app(9, 0, 1, v(9, 3)));
    // body, ctx [..,op,cont,ih] (len=9): Vis e resp b op ih.
    //   e=v(9,0), resp=v(9,1), b=v(9,3), op=v(9,6), ih=v(9,8).
    let vis_body = apply_all(
        Term::constructor(vis_id, vec![]),
        &[v(9, 0), v(9, 1), v(9, 3), v(9, 6), v(9, 8)],
    );
    let method_vis = Term::lam(op_dom, Term::lam(cont_dom, Term::lam(ih_dom, vis_body)));

    let elim = Term::Elim {
        fam: itree_id,
        level_args: vec![],
        params: vec![v(6, 0), v(6, 1), v(6, 2)], // e, resp, a
        motive: Box::new(motive),
        methods: vec![method_ret, method_vis],
        indices: vec![],
        scrut: Box::new(v(6, 4)), // t
    };

    // Domains, one at a time, each verified against its OWN preceding context
    // (`data.rs`'s "arg j is in context [Δ_p, arg0..argj-1]" discipline).
    let d_e = ty0(); // ctx []
    let d_resp = Term::pi(Term::var(0), ty0()); // ctx [e]: e=Var(0)
    let d_a = ty0(); // ctx [e,resp]
    let d_b = ty0(); // ctx [e,resp,a]
    // d_t: ITree e resp a, ctx [e,resp,a,b] (len=4): e=Var(3),resp=Var(2),a=Var(1).
    let d_t = apply_all(Term::indformer(itree_id, vec![]), &[Term::var(3), Term::var(2), Term::var(1)]);
    // d_f: a -> ITree e resp b, ctx [e,resp,a,b,t] (len=5): a=Var(2),
    //   domain=a=Var(2); codomain ctx [e,resp,a,b,t,_] (len=6): e=Var(5),resp=Var(4),b=Var(2).
    let d_f = Term::pi(
        Term::var(2),
        apply_all(Term::indformer(itree_id, vec![]), &[Term::var(5), Term::var(4), Term::var(2)]),
    );
    let final_domains = [d_e, d_resp, d_a, d_b, d_t, d_f];
    let body = lam_chain(&final_domains, elim);
    // Return type: ITree e resp b, ctx [e,resp,a,b,t,f] (len=6): e=Var(5),resp=Var(4),b=Var(2).
    let ret_ty = apply_all(Term::indformer(itree_id, vec![]), &[Term::var(5), Term::var(4), Term::var(2)]);
    let ty = pi_chain(&final_domains, ret_ty);
    declare_def(env, vec![], ty, body).map_err(|e| format!("bind declaration failed: {e:?}"))
}

/// `run_state : (s f:Type) -> (RespF:f->Type) -> (a:Type) -> s ->
///   ITree (Coproduct (StateOp s) f) (resp_coproduct s f RespF) a -> ITree f RespF (Sigma a s)`.
///
/// Outer binder order (declaration order, base context): `s, f, RespF, a, s0,
/// t` — logical positions 0..5, "base-6" context. Every helper below takes an
/// `extra` count: how many MORE binders have been pushed on top of that
/// base-6 context at the point the fragment is placed; `v`/`state_result_ty`
/// then compute the correct `Var` from `6 + extra`.
///
/// `state_result_ty(extra)` = `s -> ITree f RespF (Sigma a s)`, the
/// state-passing motive's value — reused at THREE nesting depths (the outer
/// `elim_ITree`'s motive, the `elim_Coproduct`'s generalized motive, the
/// `elim_StateOp`'s generalized motive) via the "generalize, then apply"
/// trick documented at the top of this file.
pub fn declare_run_state(
    env: &mut GlobalEnv,
    itree_id: GlobalId,
    ret_id: GlobalId,
    vis_id: GlobalId,
    state_op_id: GlobalId,
    get_id: GlobalId,
    put_id: GlobalId,
    coproduct_id: GlobalId,
    inl_id: GlobalId,
    inr_id: GlobalId,
    resp_state_id: GlobalId,
    resp_coproduct_id: GlobalId,
    unit_id: GlobalId,
    mkunit_id: GlobalId,
) -> Result<GlobalId, String> {
    let itree3 = |e: Term, resp: Term, r: Term| apply_all(Term::indformer(itree_id, vec![]), &[e, resp, r]);

    // `ITree f RespF (Sigma a s)`, evaluated so that `f`,`RespF`,`a` sit at
    // logical positions 1,2,3 of a `(6+extra)`-length context and `s` (the
    // Sigma's 2nd-slot type) sits at `(6+extra)+1`.
    let final_result_ty = |extra: usize| -> Term {
        let len = 6 + extra;
        let sigma_len = len + 1;
        itree3(v(len, 1), v(len, 2), Term::sigma(v(len, 3), v(sigma_len, 0)))
    };
    // `s -> ITree f RespF (Sigma a s)`, the whole state-passing function type,
    // placed so its OWN Pi sits at a `(6+extra)`-length context (i.e. `s`
    // itself is `v(6+extra, 0)`); the codomain is `final_result_ty(extra+1)`.
    let state_result_ty = |extra: usize| -> Term {
        let len = 6 + extra;
        Term::pi(v(len, 0), final_result_ty(extra + 1))
    };
    // `Coproduct (StateOp s) f`, at a `(6+extra)`-length context.
    let coproduct_ty = |extra: usize| -> Term {
        let len = 6 + extra;
        apply_all(Term::indformer(coproduct_id, vec![]), &[Term::app(Term::indformer(state_op_id, vec![]), v(len, 0)), v(len, 1)])
    };
    // `resp_coproduct (StateOp s) f (resp_state s) RespF`, at a `(6+extra)`-length
    // context — the general `resp_coproduct`'s 4-arg form (`effect-composition`
    // D1, §D1.3: State is the literal instance `resp_coproduct (StateOp s) f
    // (resp_state s) RespF`), still curried awaiting `op` like the old 3-arg
    // form was.
    let resp_coproduct_app = |extra: usize| -> Term {
        let len = 6 + extra;
        let state_op_s = Term::app(Term::indformer(state_op_id, vec![]), v(len, 0));
        let resp_state_s = Term::app(Term::const_(resp_state_id, vec![]), v(len, 0));
        apply_all(Term::const_(resp_coproduct_id, vec![]), &[state_op_s, v(len, 1), resp_state_s, v(len, 2)])
    };

    // ---- elim_ITree (outermost fold) --------------------------------------
    // ctx [s,f,RespF,a,s0,t] (len=6, extra=0).
    let motive_dom = itree3(coproduct_ty(0), resp_coproduct_app(0), v(6, 3)); // ITree Op Resp a
    let motive = ascribed(Term::lam(motive_dom.clone(), state_result_ty(1)), Term::pi(motive_dom, ty0()));

    // method_ret = \(x:a). \(sv:s). Ret[f,RespF,Sigma(a,s)] (Pair x sv).
    //   x domain, ctx len=6: a=v(6,3). Body ctx len=7 (x bound, extra=1).
    let method_ret = {
        let x_dom = v(6, 3);
        // sv domain, ctx len=7 (extra=1): s = v(7,0).
        let sv_dom = v(7, 0);
        // Body, ctx len=8 (x,sv bound): f=v(8,1),RespF=v(8,2),a=v(8,3),s=v(8,0).
        // Sigma(a,s) 2nd-slot ctx len=9: s=v(9,0).
        let sigma_ty = Term::sigma(v(8, 3), v(9, 0));
        let ret_applied = apply_all(Term::constructor(ret_id, vec![]), &[v(8, 1), v(8, 2), sigma_ty]);
        // x=v(8,6)? -- local order to ctx8: s,f,RespF,a,s0,t,x,sv (8 total): x=idx6,sv=idx7.
        let x_var = v(8, 6);
        let sv_var = v(8, 7);
        let body = Term::app(ret_applied, Term::pair(x_var, sv_var));
        Term::lam(x_dom, Term::lam(sv_dom, body))
    };

    // method_vis = \(op:Op). \(cont:Resp op->ITree Op Resp a). \(ih:Resp op->state_result_ty).
    //   (elim_Coproduct M_coproduct method_inl method_inr op) ih.
    let method_vis = {
        // op domain, ctx len=6 (extra=0): Op = Coproduct (StateOp s) f.
        let op_dom = coproduct_ty(0);
        // cont domain, ctx len=7 (op bound, extra=1): Resp op -> ITree Op Resp a.
        //   Resp = resp_coproduct_app(1); op = v(7,6) [local order s,f,RespF,a,s0,t,op].
        let resp_op_1 = Term::app(resp_coproduct_app(1), v(7, 6));
        let cont_dom = Term::pi(resp_op_1, itree3(coproduct_ty(2), resp_coproduct_app(2), v(8, 3)));
        // ih domain, ctx len=8 (op,cont bound, extra=2): Resp op -> state_result_ty(3).
        //   Resp = resp_coproduct_app(2); op = v(8,6) [local order +cont].
        let resp_op_2 = Term::app(resp_coproduct_app(2), v(8, 6));
        let ih_dom = Term::pi(resp_op_2, state_result_ty(3));

        // BODY, ctx len=9 (op,cont,ih bound, extra=3).
        // local order to ctx9: s,f,RespF,a,s0,t,op,cont,ih -> op=idx6,cont=idx7,ih=idx8.
        let op_var_9 = v(9, 6);
        let ih_var_9 = v(9, 8);

        // ---- elim_Coproduct (peel State's own ops out of the coproduct) --------
        // Placed at ctx9 (extra=3). Its own motive's binder `o` pushes to ctx10 (extra=4).
        let m_coproduct = {
            let dom = coproduct_ty(3); // Coproduct (StateOp s) f, ctx9
            // body, ctx10 (extra=4): (resp_coproduct(o) -> state_result_ty(5)) -> state_result_ty(5).
            // local order to ctx10: ...,op,cont,ih,o -> o is newest = v(10,9).
            let resp_coproduct_o = Term::app(resp_coproduct_app(4), v(10, 9));
            let domain_y = Term::pi(resp_coproduct_o, state_result_ty(5));
            Term::lam(dom, Term::pi(domain_y, state_result_ty(5)))
        };
        let m_coproduct_ty = Term::pi(coproduct_ty(3), ty0());

        // method_inl = \(a':StateOp s). \(ih2:resp_state(s,a')->state_result_ty(5)).
        //     (elim_StateOp M_state method_get method_put a') ih2.
        let method_inl = {
            let aprime_dom = Term::app(Term::indformer(state_op_id, vec![]), v(9, 0)); // StateOp s, ctx9
            // ih2 domain, ctx10 (a' bound, extra=4): resp_state(s,a') -> state_result_ty(5).
            //   local order to ctx10: ...,op,cont,ih,a' -> a' newest = v(10,9).
            let resp_state_s_aprime_10 = apply_all(Term::const_(resp_state_id, vec![]), &[v(10, 0), v(10, 9)]);
            let ih2_dom = Term::pi(resp_state_s_aprime_10, state_result_ty(5));

            // BODY2, ctx11 (a',ih2 bound, extra=5).
            // local order to ctx11: ...,op,cont,ih,a',ih2 -> a'=idx9,ih2=idx10.
            let aprime_var_11 = v(11, 9);
            let ih2_var_11 = v(11, 10);

            // ---- elim_StateOp (peel Get/Put out of StateOp) --------------
            let inner_motive = {
                let dom2 = Term::app(Term::indformer(state_op_id, vec![]), v(11, 0)); // StateOp s, ctx11
                // body, ctx12 (extra=6): (resp_state(s,a'')->state_result_ty(7)) -> state_result_ty(7).
                // local order to ctx12: ...,a',ih2,a'' -> a'' newest = v(12,11).
                let resp_state_s_aprimeprime = apply_all(Term::const_(resp_state_id, vec![]), &[v(12, 0), v(12, 11)]);
                let domain_w = Term::pi(resp_state_s_aprimeprime, state_result_ty(7));
                Term::lam(dom2, Term::pi(domain_w, state_result_ty(7)))
            };
            let inner_motive_ty = Term::pi(Term::app(Term::indformer(state_op_id, vec![]), v(11, 0)), ty0());

            // method_get = \(ih3: s -> state_result_ty(6)). \(sv:s). ih3 sv sv.
            //   -- Resp_state(s,Get) reduces to `s` itself (the response IS
            //   the current state, unchanged: `get` never mutates).
            let method_get = {
                let ih3_dom = Term::pi(v(11, 0), state_result_ty(6)); // ctx11: s=v(11,0); codomain ctx12=extra6.
                let sv_dom = v(12, 0); // s, ctx12 (ih3 bound, extra6).
                // body, ctx13 (ih3,sv bound, extra7). local order to ctx13:
                // ...,a',ih2,ih3,sv -> ih3=idx11,sv=idx12.
                let ih3_var = v(13, 11);
                let sv_var = v(13, 12);
                let body = apply_all(ih3_var, &[sv_var.clone(), sv_var]);
                Term::lam(ih3_dom, Term::lam(sv_dom, body))
            };
            // method_put = \(s'':s). \(ih3': Unit -> state_result_ty(7)). \(sv:s). ih3' MkUnit s''.
            //   -- Put ignores the OLD state `sv`; the NEW state is `s''`, and
            //   the response is Unit (`Resp_state(s,Put s'') reduces to Unit`).
            let method_put = {
                let sprime_dom = v(11, 0); // s, ctx11.
                let ih3p_dom = Term::pi(Term::indformer(unit_id, vec![]), state_result_ty(7)); // ctx12 (s'' bound, extra6); codomain ctx13=extra7.
                let sv_dom = v(13, 0); // s, ctx13 (s'',ih3' bound, extra7).
                // body, ctx14 (s'',ih3',sv bound, extra8). local order to
                // ctx14: ...,a',ih2,s'',ih3',sv -> s''=idx11,ih3'=idx12,sv=idx13.
                let sprime_var = v(14, 11);
                let ih3p_var = v(14, 12);
                let body = apply_all(ih3p_var, &[Term::constructor(mkunit_id, vec![]), sprime_var]);
                Term::lam(sprime_dom, Term::lam(ih3p_dom, Term::lam(sv_dom, body)))
            };

            let elim_state_op = Term::Elim {
                fam: state_op_id,
                level_args: vec![],
                params: vec![v(11, 0)], // s
                motive: Box::new(ascribed(inner_motive, inner_motive_ty)),
                methods: vec![method_get, method_put],
                indices: vec![],
                scrut: Box::new(aprime_var_11),
            };
            Term::lam(aprime_dom, Term::lam(ih2_dom, Term::app(elim_state_op, ih2_var_11)))
        };

        // method_inr = \(o':f). \(ih2'':RespF(o')->state_result_ty(5)). \(sv:s).
        //     Vis[f,RespF,Sigma(a,s)] o' (\r. ih2'' r sv)  -- pass F's op
        //     through untouched, threading the SAME state `sv` across it.
        let method_inr = {
            let oprime_dom = v(9, 1); // f, ctx9.
            // ih2'' domain, ctx10 (o' bound, extra=4): RespF o' -> state_result_ty(5).
            // local order to ctx10: ...,op,cont,ih,o' -> RespF=v(10,2), o'=v(10,9).
            let respf_oprime_10 = Term::app(v(10, 2), v(10, 9));
            let ih2pp_dom = Term::pi(respf_oprime_10, state_result_ty(5));

            // sv domain, ctx11 (o',ih2'' bound, extra=5): s = v(11,0).
            let sv_dom = v(11, 0);

            // Vis application, ctx12 (o',ih2'',sv bound, extra=6).
            // local order to ctx12: ...,op,cont,ih,o',ih2'',sv -> o'=idx9,ih2''=idx10,sv=idx11.
            let f_12 = v(12, 1);
            let respf_12 = v(12, 2);
            let sigma_a_s_12 = Term::sigma(v(12, 3), v(13, 0));
            let vis_params = apply_all(Term::constructor(vis_id, vec![]), &[f_12, respf_12, sigma_a_s_12]);
            let oprime_var_12 = v(12, 9);
            // cont_lambda = \(r: RespF o'). ih2'' r sv.
            let respf_oprime_12 = Term::app(v(12, 2), oprime_var_12.clone());
            // r domain ctx12: RespF o' (above). Body ctx13 (extra=7).
            // local order to ctx13: ...,o',ih2'',sv,r -> ih2''=idx10,sv=idx11,r=idx12.
            let ih2pp_var_13 = v(13, 10);
            let sv_var_13 = v(13, 11);
            let r_var_13 = v(13, 12);
            let cont_body = apply_all(ih2pp_var_13, &[r_var_13, sv_var_13]);
            let cont_lambda = Term::lam(respf_oprime_12, cont_body);
            let vis_applied = apply_all(vis_params, &[oprime_var_12, cont_lambda]);

            Term::lam(oprime_dom, Term::lam(ih2pp_dom, Term::lam(sv_dom, vis_applied)))
        };

        let elim_coproduct = Term::Elim {
            fam: coproduct_id,
            level_args: vec![],
            params: vec![Term::app(Term::indformer(state_op_id, vec![]), v(9, 0)), v(9, 1)], // [StateOp s, f]
            motive: Box::new(ascribed(m_coproduct, m_coproduct_ty)),
            methods: vec![method_inl, method_inr],
            indices: vec![],
            scrut: Box::new(op_var_9),
        };
        let body = Term::app(elim_coproduct, ih_var_9);
        Term::lam(op_dom, Term::lam(cont_dom, Term::lam(ih_dom, body)))
    };

    let elim_itree = Term::Elim {
        fam: itree_id,
        level_args: vec![],
        params: vec![coproduct_ty(0), resp_coproduct_app(0), v(6, 3)], // [Op, Resp, a]
        motive: Box::new(motive),
        methods: vec![method_ret, method_vis],
        indices: vec![],
        scrut: Box::new(v(6, 5)), // t
    };
    // run_state s f RespF a s0 t = (elim_ITree ...) s0.
    let elim_result = Term::app(elim_itree, v(6, 4) /* s0 */);

    // ---- outer declaration (s,f,RespF,a,s0,t) ------------------------------
    let d_s = ty0(); // ctx []
    let d_f = ty0(); // ctx [s]
    let d_respf = Term::pi(Term::var(0), ty0()); // ctx [s,f]: f=Var(0)
    let d_a = ty0(); // ctx [s,f,RespF]
    let d_s0 = Term::var(3); // s, ctx [s,f,RespF,a] (len=4): s=Var(3)
    // d_t: ITree Op Resp a, ctx [s,f,RespF,a,s0] (len=5): s=Var(4),f=Var(3),RespF=Var(2),a=Var(1).
    // `Resp = resp_coproduct (StateOp s) f (resp_state s) RespF` — the general
    // `resp_coproduct`'s 4-arg form (State as the literal instance, §D1.3).
    let d_t = itree3(
        apply_all(
            Term::indformer(coproduct_id, vec![]),
            &[Term::app(Term::indformer(state_op_id, vec![]), Term::var(4)), Term::var(3)],
        ),
        apply_all(
            Term::const_(resp_coproduct_id, vec![]),
            &[
                Term::app(Term::indformer(state_op_id, vec![]), Term::var(4)),
                Term::var(3),
                Term::app(Term::const_(resp_state_id, vec![]), Term::var(4)),
                Term::var(2),
            ],
        ),
        Term::var(1),
    );
    let final_domains = [d_s, d_f, d_respf, d_a, d_s0, d_t];
    let body = lam_chain(&final_domains, elim_result);
    // Return type: ITree f RespF (Sigma a s), ctx [s,f,RespF,a,s0,t] (len=6):
    // f=Var(4),RespF=Var(3),a=Var(2),s=Var(5). Sigma 2nd-slot ctx (len=7): s=Var(6).
    let ret_ty = itree3(Term::var(4), Term::var(3), Term::sigma(Term::var(2), Term::var(6)));
    let ty = pi_chain(&final_domains, ret_ty);

    let _ = (get_id, put_id, inl_id, inr_id);
    declare_def(env, vec![], ty, body).map_err(|e| format!("run_state declaration failed: {e:?}"))
}

/// `get : (s f:Type) -> (RespF:f->Type) -> Unit -> ITree (Coproduct (StateOp s) f) (resp_coproduct s f RespF) s`.
/// `  = \s f RespF _. Vis (InL Get) (\r. Ret r)` (`36 §4.5.2`: `Resp Get = s`
/// — the response the continuation receives IS the current state).
///
/// A plain constructor application (`Vis`/`InL`/`Get`/`Ret`) — no `Elim`, no
/// recursion, total by construction.
pub fn declare_get(
    env: &mut GlobalEnv,
    itree_id: GlobalId,
    ret_id: GlobalId,
    vis_id: GlobalId,
    state_op_id: GlobalId,
    get_id: GlobalId,
    coproduct_id: GlobalId,
    inl_id: GlobalId,
    resp_coproduct_id: GlobalId,
    resp_state_id: GlobalId,
    unit_id: GlobalId,
) -> Result<GlobalId, String> {
    // Base ctx (declaration order): s, f, RespF, _:Unit (len=4).
    let op_ty = |len: usize| apply_all(Term::indformer(coproduct_id, vec![]), &[Term::app(Term::indformer(state_op_id, vec![]), v(len, 0)), v(len, 1)]);
    // `resp_coproduct (StateOp s) f (resp_state s) RespF` — the general `resp_coproduct`'s
    // 4-arg form (State as the literal instance, §D1.3), still curried
    // awaiting `op`.
    let resp_ty = |len: usize| {
        let state_op_s = Term::app(Term::indformer(state_op_id, vec![]), v(len, 0));
        let resp_state_s = Term::app(Term::const_(resp_state_id, vec![]), v(len, 0));
        apply_all(Term::const_(resp_coproduct_id, vec![]), &[state_op_s, v(len, 1), resp_state_s, v(len, 2)])
    };

    // ctx4 (s,f,RespF,_ all bound): Op=op_ty(4), Resp=resp_ty(4), R=s=v(4,0).
    let get_at_s = apply_all(Term::constructor(get_id, vec![]), &[v(4, 0)]);
    let inl_applied = apply_all(
        Term::constructor(inl_id, vec![]),
        &[Term::app(Term::indformer(state_op_id, vec![]), v(4, 0)), v(4, 1), get_at_s],
    );
    // cont = \(r:s). Ret[Op,Resp,s] r. Domain ctx4: s=v(4,0). Body ctx5: r=v(5,4)...
    // local order to ctx5: s,f,RespF,_,r -> r newest = v(5,4).
    let cont = Term::lam(v(4, 0), Term::app(apply_all(Term::constructor(ret_id, vec![]), &[op_ty(5), resp_ty(5), v(5, 0)]), v(5, 4)));
    let vis_applied = apply_all(Term::constructor(vis_id, vec![]), &[op_ty(4), resp_ty(4), v(4, 0), inl_applied, cont]);

    let d_s = ty0();
    let d_f = ty0();
    let d_respf = Term::pi(Term::var(0), ty0()); // ctx [s,f]: f=Var(0)
    let d_unit = Term::indformer(unit_id, vec![]);
    let domains = [d_s, d_f, d_respf, d_unit];
    let body = lam_chain(&domains, vis_applied);
    // Return type ctx4: ITree Op Resp s.
    let ret_ty = itree3_standalone(itree_id, op_ty(4), resp_ty(4), v(4, 0));
    let ty = pi_chain(&domains, ret_ty);
    declare_def(env, vec![], ty, body).map_err(|e| format!("get declaration failed: {e:?}"))
}

/// `put : (s f:Type) -> (RespF:f->Type) -> s -> ITree (Coproduct (StateOp s) f) (resp_coproduct s f RespF) Unit`.
/// `  = \s f RespF s'. Vis (InL (Put s')) (\_. Ret MkUnit)` (`36 §4.5.2`:
/// `Resp (Put _) = Unit` — the old state is discarded, the new state `s'` is
/// threaded by `run_state`, not mutated here).
pub fn declare_put(
    env: &mut GlobalEnv,
    itree_id: GlobalId,
    ret_id: GlobalId,
    vis_id: GlobalId,
    state_op_id: GlobalId,
    put_id: GlobalId,
    coproduct_id: GlobalId,
    inl_id: GlobalId,
    resp_coproduct_id: GlobalId,
    resp_state_id: GlobalId,
    unit_id: GlobalId,
    mkunit_id: GlobalId,
) -> Result<GlobalId, String> {
    // Base ctx (declaration order): s, f, RespF, s':s (len=4).
    let op_ty = |len: usize| apply_all(Term::indformer(coproduct_id, vec![]), &[Term::app(Term::indformer(state_op_id, vec![]), v(len, 0)), v(len, 1)]);
    // `resp_coproduct (StateOp s) f (resp_state s) RespF` — see `declare_get`'s
    // identical closure for the rationale.
    let resp_ty = |len: usize| {
        let state_op_s = Term::app(Term::indformer(state_op_id, vec![]), v(len, 0));
        let resp_state_s = Term::app(Term::const_(resp_state_id, vec![]), v(len, 0));
        apply_all(Term::const_(resp_coproduct_id, vec![]), &[state_op_s, v(len, 1), resp_state_s, v(len, 2)])
    };

    // ctx4 (s,f,RespF,s' all bound): s'=newest=v(4,3).
    let put_applied = apply_all(Term::constructor(put_id, vec![]), &[v(4, 0), v(4, 3)]);
    let inl_applied = apply_all(
        Term::constructor(inl_id, vec![]),
        &[Term::app(Term::indformer(state_op_id, vec![]), v(4, 0)), v(4, 1), put_applied],
    );
    // cont = \(_:Unit). Ret[Op,Resp,Unit] MkUnit. Domain ctx4: Unit (closed).
    let cont = Term::lam(
        Term::indformer(unit_id, vec![]),
        Term::app(
            apply_all(Term::constructor(ret_id, vec![]), &[op_ty(5), resp_ty(5), Term::indformer(unit_id, vec![])]),
            Term::constructor(mkunit_id, vec![]),
        ),
    );
    let vis_applied = apply_all(
        Term::constructor(vis_id, vec![]),
        &[op_ty(4), resp_ty(4), Term::indformer(unit_id, vec![]), inl_applied, cont],
    );

    let d_s = ty0();
    let d_f = ty0();
    let d_respf = Term::pi(Term::var(0), ty0());
    let d_sprime = Term::var(2); // s, ctx [s,f,RespF] (len=3): s=Var(2)
    let domains = [d_s, d_f, d_respf, d_sprime];
    let body = lam_chain(&domains, vis_applied);
    let ret_ty = itree3_standalone(itree_id, op_ty(4), resp_ty(4), Term::indformer(unit_id, vec![]));
    let ty = pi_chain(&domains, ret_ty);
    declare_def(env, vec![], ty, body).map_err(|e| format!("put declaration failed: {e:?}"))
}

pub fn itree3_standalone(itree_id: GlobalId, e: Term, resp: Term, r: Term) -> Term {
    apply_all(Term::indformer(itree_id, vec![]), &[e, resp, r])
}
