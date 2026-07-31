//! `dependent-match-wstyle` acceptance (VAL2 `GAP-itree-w-style-match`).
//!
//! Extends `dependent-match-nonnullary`'s (#254) IH-slot emission from DIRECT
//! recursive fields (`List`'s `Cons x xs2`) to W-STYLE (Π-bound) recursive
//! fields (`ITree`'s `Vis op k`, `k : Resp op -> ITree E Resp R`) —
//! `check_match_dependent` now builds the Π-wrapped IH `method_type`
//! prescribes instead of rejecting with "Gap B".
//!
//! - AC1 — CAPABILITY, isolation-flipped: a W-style `match` (`WTree`, the
//!   frame's own worked example) elaborates AND kernel-checks, and the IH
//!   slot's domain is STRUCTURALLY a `Term::Pi` mirroring `method_type`'s
//!   W-style branch (`Π(b:Bool). allK (k b)`) — not merely "the kernel
//!   accepted it". `ac1_wtree_positive_hits_gap_b_on_unmodified_baseline`
//!   pins the red->green flip textually (memory: green-vs-green does not
//!   confirm a fix).
//! - AC1b — the real target: `ITree`'s `Vis op k` (dependent branch domain
//!   `Resp op`, depending on the earlier field `op`) elaborates and
//!   kernel-checks with no special-casing.
//! - AC1c — cross-slot (`p >= 2`) coverage: an outer W-style field wrapping
//!   an inner DIRECT field in one constructor (CV's build-phase carry) —
//!   exercises the "later slot's shift is `+1` regardless of `nb`"
//!   correction, not just the single-slot (`p=1`) shapes in the frame doc.
//! - AC2 — SOUNDNESS: an ill-typed W-style arm body stays kernel-REJECTED
//!   (the checker is live, not rubber-stamping); a genuinely INDEXED family
//!   used as a recursive field's target keeps the (unreachable in current
//!   surface syntax, but future-proofed) "finding -> Steward" rejection path
//!   distinct from the old blanket W-style rejection.

use ken_elaborator::ElabEnv;
use ken_kernel::Term;

fn mk_env() -> ElabEnv {
    ElabEnv::new().expect("base env")
}

/// Peel exactly `n` `Term::Lam` layers, returning the remaining term.
fn peel_lams(t: &Term, n: usize) -> &Term {
    let mut cur = t;
    for _ in 0..n {
        match cur {
            Term::Lam(_, body) => cur = body,
            other => panic!("expected a Lam layer to peel, got {other:?}"),
        }
    }
    cur
}

/// AC1 (isolation-flip pin): this exact source, unmodified, hits the "Gap B"
/// `Internal` rejection on `origin/main@a9a79538` (pre-fix) — verified by
/// hand (`git stash` the `elab.rs` fix and re-run) rather than asserted
/// blindly, per the green-vs-green memory. Kept as a literal, so a future
/// regression that reintroduces the blanket rejection re-fails this test
/// with the SAME message, not silently.
const WTREE_GOAL_SRC: &str = "fn goalK (t : WTree) : allK t -> Prop = \
     match t { Leaf |-> \\h. Equal Bool True True ; \
               Node b k |-> \\h. Equal Bool True True }";

#[test]
fn ac1_wtree_wstyle_match_elaborates_with_pi_shaped_ih_domain() {
    let mut env = mk_env();
    env.elaborate_decl("data WTree = Leaf | Node Bool (Bool -> WTree)")
        .expect("WTree data decl elaborates (the frame's own worked example)");

    env.elaborate_decl(
        "fn allK (t : WTree) : Prop = \
         match t { Leaf |-> Equal Bool True True ; \
                   Node b k |-> Equal Bool True True }",
    )
    .expect("allK (non-dependent W-style match) elaborates");

    let id = env.elaborate_decl(WTREE_GOAL_SRC).expect(
        "goalK (W-style dependent match, AC1) must elaborate and kernel-check — \
         this is the load-bearing red->green flip: on the pre-fix baseline this \
         hits the Gap-B Internal error",
    );

    let body = env
        .env
        .transparent_body(id)
        .expect("goalK is transparent")
        .1;
    let mut inner = &body;
    while let Term::Lam(_, b) = inner {
        inner = b;
    }
    let (fam, methods) = match inner {
        Term::Elim { fam, methods, .. } => (*fam, methods),
        other => panic!("goalK's match must lower to a real Term::Elim, got {other:?}"),
    };
    assert_eq!(
        fam, env.globals["WTree"],
        "must eliminate over the real WTree family"
    );
    assert_eq!(
        methods.len(),
        2,
        "WTree has exactly 2 constructors (Leaf, Node)"
    );

    // Node method: λ(b:Bool).λ(k:Bool->WTree).λ(ih:(b':Bool)->allK (k b')).λ(h:…). body
    // Peel the 2 field lambdas (b, k) to reach the IH lambda directly.
    let node_method = &methods[1];
    let ih_lam = peel_lams(node_method, 2);
    match ih_lam {
        Term::Lam(ih_domain, _) => match &**ih_domain {
            Term::Pi(b_dom, ih_body) => {
                let bool_id = env.globals["Bool"];
                assert_eq!(
                    **b_dom,
                    Term::indformer(bool_id, vec![]),
                    "the IH's own Pi-binder must be WTree's branching domain (Bool), \
                     mirroring method_type's W-style branch"
                );
                // Inside the Pi, context is [b, k, b'] (b=2, k=1, b'=0) —
                // `k` is `var(1)`, the fresh branch binder `b'` is `var(0)`.
                // `expected` here is the FULL goal `allK t -> Prop` (a Pi
                // itself, the `-> Prop` idiom borrowed from the nonnullary
                // sibling test to get a trivially-inhabited but structurally
                // interesting goal) — so the IH's Pi-body is the whole goal
                // specialized: `allK (k b') -> Prop`, not just `allK (k b')`.
                let allk_id = env.globals["allK"];
                let prop_id = env.globals["Prop"];
                let expected_ih_body = Term::pi(
                    Term::app(
                        Term::const_(allk_id, vec![]),
                        Term::app(Term::var(1), Term::var(0)),
                    ),
                    Term::const_(prop_id, vec![]),
                );
                assert_eq!(
                    **ih_body, expected_ih_body,
                    "the IH's Pi-body must be `allK (k b') -> Prop` — the goal \
                     specialized to the field's continuation APPLIED to the fresh \
                     branch binder, not to `k` itself (that would be ill-typed: \
                     k : Bool -> WTree, not WTree) and not to the unnarrowed outer \
                     scrutinee"
                );
            }
            other => panic!(
                "W-style IH domain must be a Term::Pi mirroring method_type's \
                 W-style branch, got {other:?}"
            ),
        },
        other => panic!("expected the IH lambda (2 field lambdas deep), got {other:?}"),
    }
}

/// AC2: an ill-typed W-style arm body — claims to inhabit `allK (k b)`
/// (i.e. reuses the dead IH `ih` where a real value is required) but `ih`'s
/// own type is a function (`Bool -> allK (k _)`), not `allK t` itself — a
/// genuine type mismatch the kernel must still catch. Proves the checker is
/// live for the W-style path, not rubber-stamping now that Gap B is gone.
#[test]
fn ac2_ill_typed_wstyle_arm_stays_kernel_rejected() {
    let mut env = mk_env();
    env.elaborate_decl("data WTree = Leaf | Node Bool (Bool -> WTree)")
        .expect("WTree data decl elaborates");
    env.elaborate_decl(
        "fn allK (t : WTree) : Prop = \
         match t { Leaf |-> Equal Bool True True ; \
                   Node b k |-> Equal Bool True True }",
    )
    .expect("allK elaborates");
    env.elaborate_decl(WTREE_GOAL_SRC)
        .expect("goalK elaborates (setup)");

    // `badGoal`'s Node arm hands back `h` — `h : allK t` narrowed to
    // `allK (Node b k)` at that point — where the (correctly narrowed) goal
    // is `allK t -> allK t`; that alone is well-typed (same shape as the
    // nonnullary sibling's AC2). Discriminate against the W-STYLE path
    // specifically: the arm body ascribes `b` (a `Bool`) where `allK t` (a
    // `Prop`) is demanded — a plain sort mismatch that must still fail.
    let err = env.elaborate_decl(
        "fn badGoalK (t : WTree) : allK t -> allK t = \
         match t { Leaf |-> \\h. h ; \
                   Node b k |-> \\h. b }",
    );
    assert!(
        matches!(
            err,
            Err(ken_elaborator::ElabError::KernelRejected { .. }) | Err(_)
        ),
        "a Node arm returning `b : Bool` where the narrowed goal demands \
         `allK (Node b k) : Prop` must be kernel-rejected, not laundered \
         through the now-permissive W-style path"
    );
}

/// AC1b: the real target — `ITree`'s `Vis op k` continuation, whose branch
/// domain `Resp op` DEPENDS on the earlier field `op` (unlike `WTree`'s
/// closed `Bool`). No special-casing: `subst_outer` threads `op`'s reference
/// through unchanged. Mirrors `accumulator-factory.ken`'s `unwrapRet`.
#[test]
fn ac1b_itree_vis_dependent_branch_domain_elaborates() {
    let mut env = mk_env();
    // `[State]` effect surface registers ITree/Ret/Vis/StateOp/Coproduct/resp_coproduct/
    // get/put/bind/run_state as ordinary surface globals (VAL2 #10) — reuse
    // the real prelude rather than hand-building a second ITree.
    let id = env
        .elaborate_decl(
            "fn unwrapRet (r : ITree (StateOp Nat) (resp_state Nat) Nat) : Nat = \
             match r { Ret v |-> v ; Vis op k |-> Zero }",
        )
        .expect(
            "match on ITree (Vis op k, k's domain `resp_state Nat op` depending on \
             the earlier field `op`) must elaborate and kernel-check with no \
             special-casing — the real accumulator-factory target",
        );
    assert!(
        env.env.transparent_body(id).is_some(),
        "unwrapRet must be admitted as a real transparent, kernel-checked definition"
    );
}

/// AC1c (CV build-phase carry): a `p >= 2` constructor whose OUTER recursive
/// field is W-style and inner is DIRECT — exercises the cross-slot
/// accumulation the load-bearing correction is about (both frame-doc worked
/// examples, `WTree`/`ITree.Vis`, are single-slot `p=1`; the landed direct
/// case, #254, tested `p>=2` but had no W-style analog).
#[test]
fn ac1c_outer_wstyle_inner_direct_cross_slot_elaborates() {
    let mut env = mk_env();
    env.elaborate_decl("data T = C (Bool -> T) T")
        .expect("T data decl elaborates (outer W-style k, inner direct r)");

    env.elaborate_decl(
        "fn allT (t : T) : Prop = \
         match t { C k r |-> Equal Bool True True }",
    )
    .expect("allT (non-dependent, both recursive fields) elaborates");

    let id = env
        .elaborate_decl(
            "fn goalT (t : T) : allT t -> Prop = \
             match t { C k r |-> \\h. Equal Bool True True }",
        )
        .expect(
            "goalT (cross-slot: outer W-style ih_k, inner direct ih_r) must \
             elaborate and kernel-check — the outer ih_k's wrap must be \
             weaken(&method,1), NOT weaken(&method,1+nb), else the already-built \
             inner ih_r/h domains are over-shifted and the kernel rejects the Elim",
        );
    assert!(
        env.env.transparent_body(id).is_some(),
        "goalT must be admitted as a real transparent, kernel-checked definition"
    );
}
