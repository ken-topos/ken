//! `obs-eq-termination` acceptance test — the `conv_struct` congruence-
//! first / lazy-δ fast path (`crates/ken-kernel/src/conv.rs`).
//!
//! Grounded against a real, live divergence: `map.ken`'s `ordBelowL`
//! (`allKeys k v (leBelow k v leq key) l` checked against `Ordered k v leq
//! (Node k v l key val r)`'s δ+ι-unfold, `And (allKeys (\k2. Equal Bool
//! (leq k2 key) True) l) ...`) hung the kernel's `declare_def` recheck —
//! confirmed via a depth-guarded trap (diverges at depth 2001 in ~1s,
//! `git-stash`-flip-tested against `conv.rs`) rather than a full OOM.
//!
//! **Root cause (traced live, not guessed):** `conv_struct` unconditionally
//! `whnf`s both operands *before* any congruence dispatch. `whnf` on a bare
//! transparent `Const` (`allKeys`, genuinely Tree-self-recursive) always
//! δ-unfolds it — so `allKeys P1 l` (`P1 = leBelow k v leq key`, a plain
//! non-recursive partial application) and `allKeys P2 l` (`P2 = \k2. Equal
//! Bool (leq k2 key) True`, `Ordered`'s own inline spelling of the *same*
//! predicate) both get fully δ-unfolded into a stuck `Elim` (scrutinee `l`
//! is free ⇒ ι never fires) *before* the dispatch can notice "same head
//! `Const`, same spine args, just compare the args." The `Elim~Elim`
//! congruence arm then has to compare the two stuck `Elim`s' **methods**,
//! whose Node-case body embeds a *further* `allKeys P (child)` occurrence
//! for the recursive-field IH slot — re-triggering the identical
//! δ-unfold-then-compare-methods cycle **one binder deeper**, forever (the
//! scrutinee's descendants never become a literal `Leaf`/`Node`, so nothing
//! ever bottoms out).
//!
//! **The goal is genuinely TRUE, not a false equality masquerading as one**
//! (Architect-verified, `evt_1nbtn609egpnn`/`evt_51wt8kwj5x8zm`): `leBelow
//! k v leq key` δ-unfolds in ONE non-recursive step to exactly `\k2. Equal
//! Bool (leq k2 key) True` — `leBelow`'s own defining equation. So a
//! fail-closed/stuck response would be sound but **wrong for this WP**: it
//! re-walls the very proof (`ordBelowL`, and transitively Map's law 4) the
//! `(Eq,Eq)` congruence arm was reinstated to unblock.
//!
//! **The fix:** before `whnf`, if both (pre-whnf) operands peel to the
//! *same* `Const` id (+ `level_args`) applied to the *same* number of
//! arguments, try converting the arguments pairwise directly — no δ-unfold
//! of the head at all. Application congruence for a deterministic function
//! is always sound regardless of whether the body would ever normalize, so
//! this can only recognise *more* true equalities, never a false one; it
//! **falls through unchanged** to the existing whnf-based path whenever it
//! doesn't apply or any argument fails to convert (preserves completeness —
//! a constant that ignores/absorbs an argument still gets the full
//! treatment via the fallback).
//!
//! This test hand-builds a faithful, minimal analogue of the real
//! `ordBelowL` shape directly via `Term`/`GlobalEnv` (mirroring
//! `k7_eq_at_inductive_whnf.rs`'s and `sct_completeness_nested_split.rs`'s
//! own hand-built-`Elim` conventions) — `Tree := Leaf | Node Tree Bool
//! Tree` (2 recursive fields, matching Map's `Node`'s recursive-field
//! shape), `allKeys(p)(m)` genuinely Tree-self-recursive with a
//! Node-method embedding a further `allKeys p (child)` IH-slot occurrence
//! (the exact shape that regenerates the stuck-`Elim` comparison one level
//! deeper) — registered directly via `add_decl` (bypassing `declare_def`'s
//! own `check`/SCT gate, which is irrelevant here: the REAL `allKeys` is
//! already elaborator-verified on `main`; this is a Term-shape-faithful
//! stand-in for exercising `conv_struct` specifically, not a re-proof of
//! `allKeys`'s own well-typedness).

use ken_kernel::env::{Context, Decl};
use ken_kernel::term::{Level, Term};
use ken_kernel::{check, declare_def, declare_inductive, declare_postulate, CtorSpec, GlobalEnv, GlobalId, InductiveSpec};

struct B {
    bool_: GlobalId,
    true_: GlobalId,
    tree: GlobalId,
    all_keys: GlobalId,
    leq: GlobalId,
    le_below: GlobalId,
}

fn omega0() -> Term {
    Term::Omega(Level::zero())
}

/// `And : Ω -> Ω -> Ω := λA.λB. Σ(_:A).B` — same shape as the prelude's
/// `And` (`crates/ken-elaborator/src/prelude.rs`).
fn mk_and(env: &mut GlobalEnv) -> GlobalId {
    let ty = Term::pi(omega0(), Term::pi(omega0(), omega0()));
    let body = Term::lam(
        omega0(),
        Term::lam(
            omega0(),
            Term::sigma(Term::var(1), ken_kernel::subst::weaken(&Term::var(0), 1)),
        ),
    );
    declare_def(env, vec![], ty, body).expect("And")
}

fn mk_env() -> (GlobalEnv, B) {
    let mut env = GlobalEnv::new();

    let bool_ = declare_inductive(&mut env, |_| InductiveSpec {
        level_params: vec![],
        params: vec![],
        indices: vec![],
        level: Level::zero(),
        constructors: vec![
            CtorSpec { args: vec![], target_indices: vec![] }, // True
            CtorSpec { args: vec![], target_indices: vec![] }, // False
        ],
    })
    .expect("Bool");
    let true_ = env.inductive(bool_).unwrap().constructors[0].id;
    let bool_t = Term::indformer(bool_, vec![]);

    // Tree := Leaf | Node Tree Bool Tree  (2 recursive fields l,r + 1
    // non-recursive `key`, matching Map's real `Node`'s recursive-field
    // shape without needing k/v type-polymorphism).
    let tree = declare_inductive(&mut env, |d_id| InductiveSpec {
        level_params: vec![],
        params: vec![],
        indices: vec![],
        level: Level::zero(),
        constructors: vec![
            CtorSpec { args: vec![], target_indices: vec![] }, // Leaf
            CtorSpec {
                args: vec![Term::indformer(d_id, vec![]), bool_t.clone(), Term::indformer(d_id, vec![])],
                target_indices: vec![],
            }, // Node Tree Bool Tree
        ],
    })
    .expect("Tree = Leaf | Node Tree Bool Tree");
    let tree_t = Term::indformer(tree, vec![]);

    // leq : Bool -> Bool -> Bool  (abstract, opaque -- mirrors the real
    // `leq` being a universally-quantified parameter, never δ-unfoldable).
    let leq = declare_postulate(&mut env, vec![], Term::pi(bool_t.clone(), Term::pi(bool_t.clone(), bool_t.clone())))
        .expect("leq : Bool -> Bool -> Bool");

    // allKeys(p : Bool -> Prop)(m : Tree) : Prop :=
    //   match m { Leaf => Top ; Node l key r => And (p key) (And (allKeys p l) (allKeys p r)) }
    // Genuinely Tree-self-recursive -- registered via `add_decl` directly
    // (see module doc: a Term-shape-faithful stand-in, not a re-proof of
    // `allKeys`'s own well-typedness, which the elaborator already covers
    // on `main`).
    let and_ = mk_and(&mut env);
    let p_dom = Term::pi(bool_t.clone(), omega0());
    let all_keys_ty = Term::pi(p_dom.clone(), Term::pi(tree_t.clone(), omega0()));
    let all_keys = env.fresh_id();

    let top_c = Term::Const { id: env.top_id(), level_args: vec![] };
    let leaf_method = top_c; // 0 fields, 0 IH -> bare term.
    // Node method: field binders [l, key, r] then IH binders [IH_l, IH_r]
    // (fields-first-then-IHs, the K7/sct_completeness_nested_split
    // convention). Context inside the body: [p, m, l, key, r, IH_l, IH_r]
    // (7 entries) -- 0=IH_r, 1=IH_l, 2=r, 3=key, 4=l, 5=m(unused), 6=p.
    let p_var = Term::var(6);
    let key_var = Term::var(3);
    let l_var = Term::var(4);
    let r_var = Term::var(2);
    let and_app = |a: Term, b: Term| {
        Term::app(Term::app(Term::const_(and_, vec![]), a), b)
    };
    let all_keys_call = |arg: Term| {
        Term::app(Term::app(Term::const_(all_keys, vec![]), p_var.clone()), arg)
    };
    let node_body = and_app(
        Term::app(p_var.clone(), key_var),
        and_app(all_keys_call(l_var), all_keys_call(r_var)),
    );
    let node_method = Term::lam(
        tree_t.clone(),
        Term::lam(bool_t.clone(), Term::lam(tree_t.clone(), Term::lam(top_dom(), Term::lam(top_dom(), node_body)))),
    );
    let all_keys_body = Term::lam(
        p_dom,
        Term::lam(
            tree_t.clone(),
            Term::Elim {
                fam: tree,
                level_args: vec![],
                params: vec![],
                motive: Box::new(Term::lam(tree_t.clone(), omega0())),
                methods: vec![leaf_method, node_method],
                indices: vec![],
                scrut: Box::new(Term::var(0)),
            },
        ),
    );
    env.add_decl(Decl::Transparent {
        id: all_keys,
        level_params: vec![],
        ty: all_keys_ty,
        body: all_keys_body,
    });

    // leBelow(bound : Bool)(k2 : Bool) : Prop := Equal Bool (leq k2 bound) True
    // -- plain, non-recursive, exactly Map's `leBelow` (with k,v elided).
    let le_below_ty = Term::pi(bool_t.clone(), Term::pi(bool_t.clone(), omega0()));
    let le_below_body = Term::lam(
        bool_t.clone(),
        Term::lam(
            bool_t.clone(),
            Term::Eq(
                Box::new(bool_t.clone()),
                Box::new(Term::app(Term::app(Term::const_(leq, vec![]), Term::var(0)), Term::var(1))),
                Box::new(Term::constructor(true_, vec![])),
            ),
        ),
    );
    let le_below = declare_def(&mut env, vec![], le_below_ty, le_below_body).expect("leBelow");

    (env, B { bool_, true_, tree, all_keys, leq, le_below })
}

/// Placeholder domain for the (dead, K7/dependent-match-convention)
/// IH-slot binders -- their content is never inspected by anything this
/// test exercises (mirrors `sct_completeness_nested_split.rs`'s
/// `wrap_lams` doc comment: only the body structure matters, not what
/// the IH binder "means").
fn top_dom() -> Term {
    omega0()
}

fn bool_t(b: &B) -> Term {
    Term::indformer(b.bool_, vec![])
}
fn tree_t(b: &B) -> Term {
    Term::indformer(b.tree, vec![])
}
fn true_c(b: &B) -> Term {
    Term::constructor(b.true_, vec![])
}
fn const_(id: GlobalId) -> Term {
    Term::Const { id, level_args: vec![] }
}
fn all_keys_app(b: &B, p: Term, m: Term) -> Term {
    Term::app(Term::app(const_(b.all_keys), p), m)
}
fn le_below_app(b: &B, bound: Term) -> Term {
    Term::app(const_(b.le_below), bound)
}
/// `\k2. Equal Bool (leq k2 bound) True` -- `Ordered`'s own inline
/// spelling of the SAME predicate `leBelow bound` unfolds to.
fn inline_le_below(b: &B, bound: Term) -> Term {
    Term::lam(
        bool_t(b),
        Term::Eq(
            Box::new(bool_t(b)),
            Box::new(Term::app(Term::app(const_(b.leq), Term::var(0)), bound)),
            Box::new(true_c(b)),
        ),
    )
}

/// **The `ordBelowL` shape, faithfully minimized.** `h : allKeys (leBelow
/// key) l` (the free-var scrutinee `l`, matching Map's `ordBelowL` where
/// `l` is a bound parameter, not a concrete `Leaf`/`Node`). Check `h`
/// against `allKeys (\k2. Equal Bool (leq k2 key) True) l` -- the SAME
/// predicate, spelled the way `Ordered`'s own δ+ι-unfold produces it.
/// Genuinely convertible (one δ-unfold of `leBelow`), so this must
/// **succeed**, not just "not crash."
#[test]
fn allkeys_two_predicate_spellings_converts() {
    let (env, b) = mk_env();
    let mut ctx = Context::new();
    ctx.push(tree_t(&b)); // l : Tree, free
    let l = Term::var(0);
    let key = Term::constructor(b.true_, vec![]); // any Bool constant
    let h_ty = all_keys_app(&b, le_below_app(&b, key.clone()), l);

    // `h` is a free hypothesis (a bound `Context` variable, not a global
    // postulate) -- keeps `l` genuinely free/neutral as in the real
    // `ordBelowL`, and only the TYPE-level conversion is under test.
    let mut ctx2 = ctx.clone();
    ctx2.push(h_ty);
    let h_var = Term::var(0);
    // `expected_ty` is checked AT `ctx2` (post-push, one binder deeper than
    // `ctx`) -- `l` must be referenced at THAT frame (`Var(1)`), not the
    // pre-push `Var(0)` `h_ty` used.
    let l_at_ctx2 = Term::var(1);
    let expected_ty = all_keys_app(&b, inline_le_below(&b, key), l_at_ctx2);
    assert!(
        check(&env, &ctx2, &h_var, &expected_ty).is_ok(),
        "allKeys at two convertible predicate spellings (leBelow-named vs \
         Ordered's own inline lambda) must be accepted -- this is a \
         GENUINELY TRUE conversion (AC2/AC4: reach true, not just \
         terminate/stick)"
    );
}

/// **AC5-adjacent control — a genuinely non-convertible predicate must
/// still be rejected**, proving the fast path's fall-through preserves
/// soundness: `allKeys (leBelow key) l` must NOT convert against `allKeys
/// (\k2. Equal Bool (leq k2 OTHER) True) l` for a syntactically-distinct,
/// non-equal bound (`key` vs a different free `Bool` var).
#[test]
fn allkeys_distinct_predicate_stays_rejected() {
    let (env, b) = mk_env();
    let mut ctx = Context::new();
    ctx.push(tree_t(&b)); // l : Tree, free
    ctx.push(bool_t(&b)); // key : Bool, free (Var(0) now; l shifts to Var(1))
    ctx.push(bool_t(&b)); // other : Bool, free (Var(0); key -> Var(1); l -> Var(2))
    let l = Term::var(2);
    let key = Term::var(1);
    let h_ty = all_keys_app(&b, le_below_app(&b, key), l);

    let mut ctx2 = ctx.clone();
    ctx2.push(h_ty);
    let h_var = Term::var(0);
    // See the sibling test: `expected_ty` is checked at `ctx2`, one binder
    // deeper than `ctx` -- `l`/`other` must be referenced at that frame.
    let l_at_ctx2 = Term::var(3);
    let other_at_ctx2 = Term::var(1);
    let expected_ty = all_keys_app(&b, inline_le_below(&b, other_at_ctx2), l_at_ctx2);
    assert!(
        check(&env, &ctx2, &h_var, &expected_ty).is_err(),
        "allKeys at two genuinely DIFFERENT predicates (different bound \
         vars) must stay rejected -- the fast path must fall through, not \
         over-accept"
    );
}
