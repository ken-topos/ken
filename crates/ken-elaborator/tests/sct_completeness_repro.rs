//! `sct-completeness` acceptance tests (VAL2 #12, shape (a) — nested
//! sub-pattern split + flat-sibling-field recursion).
//!
//! Scope per `docs/program/wp/sct-completeness.md` and Steward's
//! decomposition (`evt_2m39w8j7xd296`): this WP ships shape (a) only.
//! Shape (b) (Ackermann/lexicographic reconstruction-descent) landed
//! separately as `sct-reconstruction-descent` — its repro and near-misses
//! now live in `sct_reconstruction_descent.rs` (`shape_b_ackermann_accepts`,
//! `shape_b_bad_ack_still_rejected`, etc.), not here.
//!
//! Fix (grounded, Architect-approved `evt_51fjq30yftax4`): `enter_method`
//! (`crates/ken-kernel/src/sct.rs`) now threads a remaining-arity
//! provenance queue through a nested `Term::Elim` split (via the shared
//! `dispatch_elim_methods` helper) instead of assuming a flat run of
//! leading `Lam`s, so a deferred sibling field's true `Down` survives
//! regardless of nesting depth. Building this also surfaced and fixed a
//! separate, pre-existing latent bug: `ConstructorDecl.recursive_positions`
//! is never populated by `declare_inductive` (always `Vec::new()`,
//! `check.rs:925`) — `sct.rs` now re-derives each field's recursiveness
//! directly from its declared type (`is_recursive_field`), mirroring the
//! same pre-existing workaround `ken-interp::eval::is_recursive_arg` uses
//! for the identical gap.

use ken_elaborator::ElabEnv;

fn fresh_env() -> ElabEnv {
    ElabEnv::new().expect("prelude should elaborate")
}

fn nat_count(env: &ken_elaborator::ElabEnv, v: &ken_interp::eval::EvalVal) -> u64 {
    use ken_interp::eval::EvalVal;
    match v {
        EvalVal::Ctor { id, args, .. } if *id == env.prelude_env.zero_id && args.is_empty() => 0,
        EvalVal::Ctor { id, args, .. } if *id == env.prelude_env.suc_id && args.len() == 1 => {
            1 + nat_count(env, &args[0])
        }
        other => panic!("expected a Nat Ctor chain, got {other:?}"),
    }
}

/// Control — flat (non-nested) match, ONE sibling field recurses. Baseline
/// that must remain accepted (no regression from the nested-split fix).
#[test]
fn control_flat_single_sibling_recurses() {
    let mut env = fresh_env();
    env.elaborate_decl("data Tree = Leaf | Node Tree Int Tree")
        .expect("Tree should declare");
    env.elaborate_decl(
        "view countR (t : Tree) : Nat = \
         match t { Leaf => Zero ; Node l c r => countR r }",
    )
    .expect("flat single-sibling recursion must be accepted (control)");
}

/// Control — flat (non-nested) match, BOTH sibling fields recurse (the
/// literal shape `tree-traversal`'s intended `inorder` uses, modulo `List`
/// `Char` machinery).
#[test]
fn control_flat_both_siblings_recurse() {
    let mut env = fresh_env();
    env.elaborate_decl("data Tree = Leaf | Node Tree Int Tree")
        .expect("Tree should declare");
    env.elaborate_decl(
        "view natAdd (a : Nat) (b : Nat) : Nat = \
         match a { Zero => b ; Suc m => Suc (natAdd m b) }",
    )
    .expect("natAdd should declare");
    env.elaborate_decl(
        "view countBoth (t : Tree) : Nat = \
         match t { \
           Leaf => Zero ; \
           Node l c r => Suc (natAdd (countBoth l) (countBoth r)) \
         }",
    )
    .expect("flat both-siblings recursion must be accepted (control)");
}

/// **(a) VAL2 #12 — FIXED, AC2 accepts.** A nested sub-pattern split on the
/// FIRST field of `Node`, with a recursive call on the FLAT SIBLING field
/// `r` (never descending into the nested part) — a genuine structural
/// recursion that must terminate.
#[test]
fn shape_a_val2_12_nested_split_flat_sibling_recursion_accepts() {
    let mut env = fresh_env();
    env.elaborate_decl("data Tree = Leaf | Node Tree Int Tree")
        .expect("Tree should declare");
    env.elaborate_decl(
        "view countR (t : Tree) : Nat = \
         match t { \
           Leaf => Zero ; \
           Node (Node ll lc lr) c r => countR r ; \
           Node Leaf c r => countR r \
         }",
    )
    .expect("nested-split + flat-sibling recursion must now be accepted (AC2)");
}

/// **(a) AC2 — accepts AND evaluates to the correct value**, not just
/// type-checks (Architect's gate item 5). Also exercises the IH-in-scope
/// case (checklist item 2): both `Node` arms have 2 recursive fields, so
/// each compiled method genuinely carries IH `Lam` binders interleaved
/// between the split and the deferred `c, r` continuation — this is
/// exactly the shape that exposed the `recursive_positions`-empty bug.
#[test]
fn shape_a_accepts_and_evaluates_correctly() {
    let mut env = fresh_env();
    env.elaborate_decl("data Tree = Leaf | Node Tree Int Tree")
        .expect("Tree should declare");
    // depth counts steps down the RIGHT spine only, dispatched through a
    // nested split on the LEFT field — if `r`'s provenance were lost
    // (the bug), this wouldn't even accept; if it accepted but the
    // recursion were mis-wired, the count would be wrong.
    env.elaborate_decl(
        "view rightDepth (t : Tree) : Nat = \
         match t { \
           Leaf => Zero ; \
           Node (Node ll lc lr) c r => Suc (rightDepth r) ; \
           Node Leaf c r => Suc (rightDepth r) \
         }",
    )
    .expect("must accept (AC2)");
    env.elaborate_decl(
        "view t3 : Tree = \
         Node (Node Leaf 1 Leaf) 2 (Node (Node Leaf 3 Leaf) 4 Leaf)",
    )
    .expect("t3 should declare");
    env.elaborate_decl("view result : Nat = rightDepth t3")
        .expect("rightDepth t3 should elaborate");

    let mut store = ken_interp::eval::EvalStore::new();
    for (nid, v) in &env.num_values {
        store.num_values.entry(*nid).or_insert_with(|| match v {
            ken_elaborator::NumericLitVal::Int(n) => ken_interp::eval::EvalVal::from(*n),
            _ => panic!("unexpected literal kind in this test"),
        });
    }
    let result_id = *env.globals.get("result").expect("result should be a global");
    let body = match env.env.lookup(result_id) {
        Some(ken_kernel::Decl::Transparent { body, .. }) => body.clone(),
        other => panic!("result should be Transparent, got {other:?}"),
    };
    let v = ken_interp::eval::eval(&[], &body, &env.env, &mut store);
    // t3 = Node(Node(Leaf,1,Leaf), 2, Node(Node(Leaf,3,Leaf),4,Leaf)) — the
    // right spine from the root is: t3 -> right child (Node(Node..,4,Leaf))
    // -> its right child (Leaf). rightDepth = 2 steps.
    assert_eq!(
        nat_count(&env, &v),
        2,
        "rightDepth must walk the TRUE right spine (2 steps), not a wrong-\
         but-accepted value"
    );
}

// NOTE: Architect's checklist item 4 (≥2-level nesting) is covered at the
// `ken-kernel` level instead of here — a genuine surface `match` with a
// 3-deep nested pattern (`Node (Node (Node ..) ..) ..`) trips a SEPARATE,
// pre-existing match-compiler `TypeMismatch` during elaboration itself
// (before SCT ever runs), independent of this fix. See
// `crates/ken-kernel/tests/sct_completeness_nested_split.rs`, which
// hand-builds a well-typed 2-level-nested `Term::Elim` directly and drives
// `sct_check` on it, sidestepping that unrelated elaborator limitation.
// Flagged to kernel-leader as an out-of-scope finding, not fixed here.

/// **(a) discriminating near-miss (AC1).** Shares the EXACT syntactic shape
/// as the accepted repro (nested split, recursion dispatched from inside
/// it) but is genuinely non-terminating: one arm recurses on the UNCHANGED
/// original scrutinee `t` instead of a real sub-part. Must stay rejected.
#[test]
fn shape_a_near_miss_recurses_on_unchanged_scrutinee_stays_rejected() {
    let mut env = fresh_env();
    env.elaborate_decl("data Tree = Leaf | Node Tree Int Tree")
        .expect("Tree should declare");
    let res = env.elaborate_decl(
        "view bad (t : Tree) : Nat = \
         match t { \
           Leaf => Zero ; \
           Node (Node ll lc lr) c r => bad t ; \
           Node Leaf c r => bad r \
         }",
    );
    assert!(
        res.is_err(),
        "genuinely non-terminating (recurses on `t` itself, no descent) — \
         must stay rejected"
    );
}

// Shape (b) (Ackermann/lexicographic reconstruction-descent) and its
// discriminating near-miss (`badAck`) moved to `sct_reconstruction_descent.rs`
// (`shape_b_ackermann_accepts`, `shape_b_bad_ack_still_rejected`) now that
// `sct-reconstruction-descent` has landed its own fix — Ackermann now
// correctly ACCEPTS, so pinning it here as still-rejected would be stale.
