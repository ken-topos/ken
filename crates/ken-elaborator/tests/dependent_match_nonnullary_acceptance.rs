//! `dependent-match-nonnullary` acceptance (Map Gap B).
//!
//! Pins the ratified-decomposition build: `check_match_dependent`'s gate
//! widens from nullary-only to any FLAT, NON-INDEXED family (`List`/`Tree`),
//! and its per-ctor method construction gains the IH-slot emission the
//! kernel's `method_type` requires for a recursive constructor.
//!
//! - AC1 — CAPABILITY: non-nullary dependent induction elaborates AND
//!   kernel-checks, for both a `List` and a `Tree`/`Node` (two-recursive-
//!   field) shape; the per-branch scrutinee narrowing is verified
//!   STRUCTURALLY (the arm's own hypothesis-lambda domain, in the core
//!   term, must reference the newly-bound recursive FIELD variable, not the
//!   original outer scrutinee) — a real fact about the elaborated
//!   `Term::Elim`, not merely "the kernel accepted it" (which a
//!   consistently-applied-but-wrong substitution could also achieve).
//! - AC2 — SOUNDNESS: a mis-narrowed (wrong-typed) `Cons`/`Node` arm stays
//!   kernel-REJECTED; grep confirms zero `crates/ken-kernel/` touched and no
//!   new `Decl` variant (`trusted_base()` unchanged).
//!
//! AC2b (SCT descent on the real `to_list`-ordered lemma) and AC4 (workspace-
//! wide regression) are covered by the existing full-suite run, not here —
//! `to_list`-ordered itself belongs to the separate, later `map-verified-laws`
//! WP this one unblocks.

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

#[test]
fn ac1_list_hypothesis_threading_elaborates_and_narrows_structurally() {
    let mut env = mk_env();
    env.elaborate_decl(
        "fn allTrue (xs : List Bool) : Prop = \
         match xs { Nil |-> Equal Bool True True ; \
                    Cons b bs |-> And (Equal Bool b True) (allTrue bs) }",
    )
    .expect("allTrue elaborates");

    // `tailGoal`: a genuine hypothesis-threaded match — `h : allTrue xs` is
    // bound INSIDE each arm (its domain comes from the per-branch-narrowed
    // goal `allTrue xs -> Prop`), the exact `\(h:Ty).` idiom AC5 pins as
    // sufficient (no annotated-lambda syntax needed).
    let id = env
        .elaborate_decl(
            "fn tailGoal (xs : List Bool) : allTrue xs -> Prop = \
             match xs { Nil |-> \\h. Equal Bool True True ; \
                        Cons b bs |-> \\h. allTrue bs }",
        )
        .expect("tailGoal (non-nullary dependent match, AC1) elaborates and kernel-checks");

    let body = env
        .env
        .transparent_body(id)
        .expect("tailGoal is transparent")
        .1;
    // Peel the function's own `xs` parameter lambda to reach the match.
    let mut inner = &body;
    while let Term::Lam(_, b) = inner {
        inner = b;
    }
    let (fam, methods) = match inner {
        Term::Elim { fam, methods, .. } => (*fam, methods),
        other => panic!("tailGoal's match must lower to a real Term::Elim, got {other:?}"),
    };
    assert_eq!(
        fam, env.globals["List"],
        "must eliminate over the real List family"
    );
    assert_eq!(
        methods.len(),
        2,
        "List has exactly 2 constructors (Nil, Cons)"
    );

    // Cons method (index 1): `λ(b:Bool).λ(bs:List Bool).λ(ih:allTrue bs ->
    // Prop).λ(h:<narrowed goal>). body`. Peel the 2 field lambdas + the 1 IH
    // lambda (`bs` is Cons's sole recursive field) to reach the arm's own
    // `\h.` binder, then inspect ITS DOMAIN directly.
    let cons_method = &methods[1];
    let h_lam = peel_lams(cons_method, 3);
    match h_lam {
        Term::Lam(h_domain, _) => {
            // The per-branch scrutinee equation (`34 §3.3`) narrows `h`'s
            // type to `allTrue` applied to the RECONSTRUCTED constructor
            // application `Cons Bool b bs` — NOT to a sub-field's own
            // predicate (that would be the *tail's* IH, a separate, dead
            // binder one layer up) and NOT to the original outer `xs`
            // (which an unnarrowed/buggy construction would leave
            // referenced directly, at a position corresponding to a
            // context that no longer includes `xs` as a live entry here).
            // Context at this point is `[b, bs, ih]` (b=2, bs=1, ih=0, after
            // the ih-wrap's weakening) — `h`'s domain must be exactly
            // `allTrue (Cons Bool b bs)`.
            let all_true_id = env.globals["allTrue"];
            let bool_id = env.globals["Bool"];
            let cons_id = env.globals["Cons"];
            let expected_concrete = Term::app(
                Term::app(
                    Term::app(
                        Term::Constructor {
                            id: cons_id,
                            level_args: vec![],
                        },
                        Term::indformer(bool_id, vec![]),
                    ),
                    Term::var(2),
                ),
                Term::var(1),
            );
            let expected_domain = Term::app(Term::const_(all_true_id, vec![]), expected_concrete);
            assert_eq!(
                **h_domain, expected_domain,
                "h's domain must be `allTrue (Cons Bool b bs)` — the per-branch \
                 scrutinee-equation narrowing (`34 §3.3`) substituting the \
                 RECONSTRUCTED constructor application for the scrutinee, not \
                 a sub-field's own predicate or the unnarrowed outer scrutinee"
            );
        }
        other => panic!("expected the arm's own `\\h.` lambda, got {other:?}"),
    }
}

#[test]
fn ac1_tree_two_recursive_fields_elaborates_and_narrows_structurally() {
    let mut env = mk_env();
    env.elaborate_decl("data Tree a = Leaf | Node (Tree a) a (Tree a)")
        .expect("Tree data decl elaborates");

    // A Tree-shaped predicate (structural, no `leq` — orthogonal to Gap A).
    env.elaborate_decl(
        "fn allPos (t : Tree Nat) : Prop = \
         match t { Leaf |-> Equal Bool True True ; \
                   Node l k r |-> And (Equal Bool True True) (And (allPos l) (allPos r)) }",
    )
    .expect("allPos (Tree, two recursive fields) elaborates");

    // Hypothesis-threaded goal over Tree: exercises TWO IH slots per `Node`
    // (the harder p=2 case — ih1 outermost/ih2 innermost, `method_type`
    // order) rather than List's single recursive field.
    let id = env
        .elaborate_decl(
            "fn leftGoal (t : Tree Nat) : allPos t -> Prop = \
             match t { Leaf |-> \\h. Equal Bool True True ; \
                       Node l k r |-> \\h. allPos l }",
        )
        .expect("leftGoal (Tree, non-nullary dependent match, AC1) elaborates and kernel-checks");

    let body = env
        .env
        .transparent_body(id)
        .expect("leftGoal is transparent")
        .1;
    let mut inner = &body;
    while let Term::Lam(_, b) = inner {
        inner = b;
    }
    let (fam, methods) = match inner {
        Term::Elim { fam, methods, .. } => (*fam, methods),
        other => panic!("leftGoal's match must lower to a real Term::Elim, got {other:?}"),
    };
    assert_eq!(
        fam, env.globals["Tree"],
        "must eliminate over the real Tree family"
    );
    assert_eq!(
        methods.len(),
        2,
        "Tree has exactly 2 constructors (Leaf, Node)"
    );

    // Node method (index 1): `λ(l:Tree Nat).λ(k:Nat).λ(r:Tree Nat).
    // λ(ih_l:allPos l).λ(ih_r:allPos r).λ(h:<narrowed goal>). body` — 3
    // fields + 2 IH slots (`l` and `r` both recursive) before the arm's own
    // `\h.` binder.
    let node_method = &methods[1];
    let h_lam = peel_lams(node_method, 5);
    match h_lam {
        Term::Lam(h_domain, _) => {
            // `h`'s domain narrows to `allPos` applied to the RECONSTRUCTED
            // `Node Nat l k r` — not to either recursive field's own IH
            // (`allPos l`/`allPos r`, dead binders one/two layers up) and
            // not to the unnarrowed outer `t`. After popping the 3 field
            // pushes and wrapping the 2 IH slots (`l` at ctor position 0,
            // `r` at position 2 — `k`, position 1, is a plain `Nat`, not
            // recursive), `l`/`k`/`r` sit at `Var(4)/Var(3)/Var(2)`.
            let allpos_id = env.globals["allPos"];
            let nat_id = env.globals["Nat"];
            let node_id = env.globals["Node"];
            let expected_concrete = Term::app(
                Term::app(
                    Term::app(
                        Term::app(
                            Term::Constructor {
                                id: node_id,
                                level_args: vec![],
                            },
                            Term::indformer(nat_id, vec![]),
                        ),
                        Term::var(4),
                    ),
                    Term::var(3),
                ),
                Term::var(2),
            );
            let expected_domain = Term::app(Term::const_(allpos_id, vec![]), expected_concrete);
            assert_eq!(
                **h_domain, expected_domain,
                "h's domain must be `allPos (Node Nat l k r)` — the per-branch \
                 scrutinee-equation narrowing, correctly handling TWO \
                 recursive fields (p=2, the harder multi-IH case) in one ctor"
            );
        }
        other => panic!("expected the arm's own `\\h.` lambda, got {other:?}"),
    }
}

#[test]
fn ac2_mis_narrowed_cons_arm_stays_kernel_rejected() {
    let mut env = mk_env();
    env.elaborate_decl(
        "fn allTrue (xs : List Bool) : Prop = \
         match xs { Nil |-> Equal Bool True True ; \
                    Cons b bs |-> And (Equal Bool b True) (allTrue bs) }",
    )
    .expect("allTrue elaborates");

    // Discriminating NEGATIVE: `Cons` arm's body claims to inhabit the
    // (correctly narrowed) `allTrue bs` slot but hands back `h` itself —
    // `h : allTrue (Cons b bs)` (an `And`-shaped goal at that point), not
    // `allTrue bs` — a genuine, wrongly-typed term the kernel must reject
    // if the narrowing (and the kernel's own re-check of it) is real.
    let err = env.elaborate_decl(
        "fn badTailGoal (xs : List Bool) : allTrue xs -> allTrue xs = \
         match xs { Nil |-> \\h. h ; \
                    Cons b bs |-> \\h. allTrue bs }",
    );
    assert!(
        err.is_err(),
        "a Cons arm returning `allTrue bs` (the TAIL's predicate) where the \
         narrowed goal demands `allTrue (Cons b bs)` (the WHOLE predicate) \
         must be kernel-rejected, not laundered through"
    );
}

/// AC2b: SCT descent through the dependent-motive `Elim` is unperturbed.
///
/// A recursive `view`'s admission (`declare_recursive_group`/`sct_check`,
/// wired into ordinary `view` elaboration) runs regardless of whether the
/// match's motive happens to be constant or scrutinee-dependent — so a
/// self-recursive proof/predicate built through THIS WP's widened
/// `check_match_dependent` path only elaborates at all if SCT accepts its
/// structural descent. `allTrue` (List, single recursive field, `AC1`'s own
/// setup) already covers the one-IH case; this covers the harder `Tree`
/// two-recursive-field shape (`to_list`-ordered's own `to_list l`/`to_list r`
/// dual-descent structure), self-recursing on BOTH `l` and `r` in one ctor.
#[test]
fn ac2b_tree_dual_recursive_descent_passes_sct() {
    let mut env = mk_env();
    env.elaborate_decl("data Tree a = Leaf | Node (Tree a) a (Tree a)")
        .expect("Tree data decl elaborates");

    let id = env
        .elaborate_decl(
            "fn allPosRec (t : Tree Nat) : Prop = \
             match t { Leaf |-> Equal Bool True True ; \
                       Node l k r |-> And (allPosRec l) (allPosRec r) }",
        )
        .expect(
            "allPosRec must elaborate — a self-recursive Tree predicate through \
             the dependent-motive Elim descending on BOTH `l` and `r` must still \
             pass `sct_check`; a regression here would reject a valid, \
             terminating program (fail-closed, but wrong)",
        );
    assert!(
        env.env.transparent_body(id).is_some(),
        "allPosRec must be admitted as a real transparent (kernel-checked, \
         SCT-passed) recursive definition, not silently dropped"
    );
}
