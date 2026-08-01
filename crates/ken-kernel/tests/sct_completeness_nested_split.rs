//! `sct-completeness` shape (a) — direct `sct_check` tests for the
//! properties Architect's approach-review checklist (`evt_51fjq30yftax4`)
//! required but couldn't be exercised through the elaborator: a genuine
//! **≥2-level** nested sub-pattern split. A real surface `match` with a
//! 3-deep nested pattern trips a SEPARATE, pre-existing match-compiler
//! `TypeMismatch` during elaboration (before SCT ever runs) — an unrelated
//! gap, out of this WP's scope (flagged to kernel-leader). `sct_check`
//! itself is a pure structural analysis over `Term` — it never
//! type-checks its input (`declare_def`, `check.rs`, calls `check` THEN
//! `sct_check` as two separate steps) — so these tests hand-build the
//! `Term::Elim` skeleton directly, bypassing the elaborator's motive
//! machinery entirely, exactly as `k7_eq_at_inductive_whnf.rs` and the K1.5
//! W-style tests do for their own kernel-level properties.
//!
//! `T = Leaf | Node T T` (2 recursive fields — both `l`/`r` get IH slots,
//! `is_recursive_field`, `sct.rs`, exercising checklist item 2 "IH in
//! scope" at the same time). `countR` recurses only on the flat sibling
//! `r`, dispatched from underneath a genuine 2-level nested split on `l`
//! (`Node (Node ll lr) => ...`, further splitting `ll` a second time
//! before finally reaching `r`) — checklist item 4.

use ken_kernel::sct::sct_check;
use ken_kernel::term::{Level, Term};
use ken_kernel::{declare_inductive, CtorSpec, GlobalEnv, GlobalId, InductiveSpec};

fn mk_env() -> (GlobalEnv, GlobalId, GlobalId, GlobalId) {
    let mut env = GlobalEnv::new();
    let t_id = declare_inductive(&mut env, |d_id| InductiveSpec {
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
                args: vec![Term::indformer(d_id, vec![]), Term::indformer(d_id, vec![])],
                target_indices: vec![],
            },
        ],
    })
    .expect("T = Leaf | Node T T");
    let (leaf_id, node_id) = {
        let cs = &env.inductive(t_id).unwrap().constructors;
        (cs[0].id, cs[1].id)
    };
    (env, t_id, leaf_id, node_id)
}

/// Wrap `body` in `n` leading `Lam`s over a dummy domain (`sct.rs` never
/// inspects `Term::Lam`'s domain — only the body structure matters to its
/// analysis, so a single placeholder type is reused for every binder,
/// whether it "means" a field, an IH, or a deferred continuation slot).
fn wrap_lams(n: usize, dom: &Term, body: Term) -> Term {
    let mut b = body;
    for _ in 0..n {
        b = Term::lam(dom.clone(), b);
    }
    b
}

/// Build the 2-level-nested `countR : T -> T` body:
/// ```text
/// countR t = match t {
///   Leaf => Leaf ;
///   Node l r => match l {           -- level 1 split (on l)
///     Leaf => countR r ;
///     Node ll lr => match ll {      -- level 2 split (on ll, INSIDE the
///       Leaf => countR r ;             Node-of-l branch, BEFORE lr/IH_l/
///       Node lll llr => countR r       IH_r are bound -- the genuine
///     }                                2-level-defer shape)
///   }
/// }
/// ```
/// Every leaf recurses only on `r`, the OUTERMOST `Node`'s flat sibling
/// field — `r`'s true `Down` provenance must survive threading through
/// TWO nested-Elim boundaries for `sct_check` to accept this.
fn build_2level_body(t_id: GlobalId, leaf_id: GlobalId, _node_id: GlobalId, countr_id: GlobalId) -> Term {
    let dom = Term::indformer(t_id, vec![]);
    let motive = dom.clone(); // sct.rs only scans the motive for calls; content is irrelevant
    let leaf_c = Term::constructor(leaf_id, vec![]);
    let call_r = Term::app(Term::const_(countr_id, vec![]), Term::var(2)); // countR r, r always ends up at index 2 (see module doc)

    // Level 2: elim on `ll` (Var(0) at this point — just bound as the
    // single split-field of level 1's Node-of-l branch).
    // - Leaf-of-ll: own arity 0 + continuation [lr, IH_ll, IH_lr, r, IH_l, IH_r] (6)
    // - Node-of-ll: own arity 4 (lll, llr, IH_lll, IH_llr) + same continuation (6) = 10
    let level2_elim = Term::Elim {
        fam: t_id,
        level_args: vec![],
        params: vec![],
        motive: Box::new(motive.clone()),
        methods: vec![
            wrap_lams(6, &dom, call_r.clone()),
            wrap_lams(10, &dom, call_r.clone()),
        ],
        indices: vec![],
        scrut: Box::new(Term::var(0)),
    };

    // Level 1's Node-of-l branch: bind `ll` only (1 lambda), THEN split
    // again immediately — deferring `lr, IH_ll, IH_lr` past the level-2
    // split, exactly the #12 shape one level deeper.
    let node_of_l_method = Term::lam(dom.clone(), level2_elim);

    // Level 1's Leaf-of-l branch: own arity 0 + continuation [r, IH_l, IH_r] (3).
    let leaf_of_l_method = wrap_lams(3, &dom, call_r.clone());

    // Level 1: elim on `l` (Var(0) — just bound as the outer Node method's
    // single split-field).
    let level1_elim = Term::Elim {
        fam: t_id,
        level_args: vec![],
        params: vec![],
        motive: Box::new(motive.clone()),
        methods: vec![leaf_of_l_method, node_of_l_method],
        indices: vec![],
        scrut: Box::new(Term::var(0)),
    };

    // Level 0's Node method: bind `l` only (1 lambda), split immediately
    // (the original #12 shape), deferring `r, IH_l, IH_r`.
    let node_method = Term::lam(dom.clone(), level1_elim);
    let leaf_method = leaf_c;

    let level0_elim = Term::Elim {
        fam: t_id,
        level_args: vec![],
        params: vec![],
        motive: Box::new(motive),
        methods: vec![leaf_method, node_method],
        indices: vec![],
        scrut: Box::new(Term::var(0)),
    };

    Term::lam(dom, level0_elim) // countR := \t. level0_elim
}

/// **Checklist item 4 — ≥2-level nesting must accept.** `r`'s `Down`
/// provenance must survive two nested-Elim boundaries.
#[test]
fn two_level_nested_split_flat_sibling_recursion_accepts() {
    let (mut env, t_id, leaf_id, node_id) = mk_env();
    let countr_id = env.fresh_id();
    let body = build_2level_body(t_id, leaf_id, node_id, countr_id);
    let result = sct_check(&env, &[(countr_id, body)]);
    assert!(
        result.is_ok(),
        "2-level nested split + flat-sibling recursion must be accepted: {:?}",
        result
    );
}

/// **Checklist items 1+2 — slot disjointness / IH-stays-`None` sanity
/// check.** Same 2-level shape, but the Leaf-of-ll branch (own arity 0,
/// pure continuation) additionally recurses on `IH_l` instead of `r` —
/// `IH_l` is ALWAYS `None` (unknown) provenance, never a field, so this
/// must stay REJECTED. If the continuation queue ever mis-tagged a
/// deferred IH slot as a field (a slot-collision bug), this would
/// wrongly accept — the sharpest possible tripwire for that failure
/// mode, since it fails IMMEDIATELY (single self-loop `[[Unknown]]`)
/// rather than needing composition to expose it.
#[test]
fn recursing_on_a_deferred_ih_slot_stays_rejected() {
    let (mut env, t_id, leaf_id, _node_id) = mk_env();
    let countr_id = env.fresh_id();
    let dom = Term::indformer(t_id, vec![]);
    let motive = dom.clone();
    let leaf_c = Term::constructor(leaf_id, vec![]);
    // Same continuation layout as build_2level_body's Leaf-of-ll branch:
    // [lr, IH_ll, IH_lr, r, IH_l, IH_r] (6) — IH_l sits at index 1.
    let call_ih_l = Term::app(Term::const_(countr_id, vec![]), Term::var(1));

    let level2_elim = Term::Elim {
        fam: t_id,
        level_args: vec![],
        params: vec![],
        motive: Box::new(motive.clone()),
        methods: vec![
            wrap_lams(6, &dom, call_ih_l.clone()),
            wrap_lams(10, &dom, call_ih_l),
        ],
        indices: vec![],
        scrut: Box::new(Term::var(0)),
    };
    let node_of_l_method = Term::lam(dom.clone(), level2_elim);
    let leaf_of_l_method = wrap_lams(
        3,
        &dom,
        Term::app(Term::const_(countr_id, vec![]), Term::var(1)),
    );
    let level1_elim = Term::Elim {
        fam: t_id,
        level_args: vec![],
        params: vec![],
        motive: Box::new(motive.clone()),
        methods: vec![leaf_of_l_method, node_of_l_method],
        indices: vec![],
        scrut: Box::new(Term::var(0)),
    };
    let node_method = Term::lam(dom.clone(), level1_elim);
    let level0_elim = Term::Elim {
        fam: t_id,
        level_args: vec![],
        params: vec![],
        motive: Box::new(motive),
        methods: vec![leaf_c, node_method],
        indices: vec![],
        scrut: Box::new(Term::var(0)),
    };
    let body = Term::lam(dom, level0_elim);

    let result = sct_check(&env, &[(countr_id, body)]);
    assert!(
        result.is_err(),
        "recursing on a deferred IH slot (always-Unknown) must stay \
         rejected — a fix that mis-tags an IH slot as a field would \
         wrongly accept this"
    );
}
