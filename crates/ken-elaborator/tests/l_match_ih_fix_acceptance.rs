//! `L-match-ih-fix` acceptance tests (VAL2 finding #5).
//!
//! `compile_match_matrix`'s `ColKind::Ih` branch built each induction-
//! hypothesis binder's type by folding `tail_codomain` over the *pending
//! tail* (the next sibling `Ih` column, for a ctor with >=2 recursive
//! fields) instead of using the flat `ret_ty`, over-building the first Ih
//! slot with an extra Pi layer and tripping a kernel `TypeMismatch` on any
//! `match` over a >=2-same-typed-recursive-field `data` type. ACs per
//! `docs/program/wp/L-match-ih-fix.md`:
//! - AC1: kernel untouched — verified out-of-band (`git diff` empty), not
//!   a unit test here.
//! - AC2: correct-for-the-right-reason — a real tree `size` using both IHs
//!   and computing the right value.
//! - AC3: discriminating pair — the valid `match` accepts, an ill-typed
//!   sibling still rejects.
//! - AC4: no regression — 0/1/2-recursive-field types (`List`, single-rec,
//!   1-rec+1-other) still elaborate.

use ken_elaborator::ElabEnv;

fn fresh_env() -> ElabEnv {
    ElabEnv::new().expect("prelude should elaborate")
}

/// `data Tree = Leaf | Node Tree Char Tree` — the canonical >=2-recursive-
/// field shape from the frame (`N10 x y`-style).
const TREE_DECL: &str = "data Tree = Leaf | Node Tree Char Tree";

#[test]
fn ac2_tree_size_uses_both_ihs_and_computes_right_value() {
    let mut env = fresh_env();
    env.elaborate_decl(TREE_DECL).expect("Tree should declare");

    // size (Node l _ r) = 1 + size l + size r -- exercises BOTH IH slots.
    // No `-`/`*`/`div_int` exist; build via nested Suc-nat nat-add instead
    // of Int arithmetic to keep this test self-contained and avoid the
    // native-Int-add ordering subtlety.
    env.elaborate_decl(
        "view natAdd (a : Nat) (b : Nat) : Nat = \
         match a { Zero => b ; Suc m => Suc (natAdd m b) }",
    )
    .expect("natAdd should declare");

    env.elaborate_decl(
        "view size (t : Tree) : Nat = \
         match t { \
           Leaf => Zero ; \
           Node l c r => Suc (natAdd (size l) (size r)) \
         }",
    )
    .expect("size should elaborate over a >=2-recursive-field ctor (AC2)");

    // A 3-node tree: Node (Node Leaf 'a' Leaf) 'b' (Node Leaf 'c' Leaf).
    // size = 1 + (1 + 0 + 0) + (1 + 0 + 0) = 3.
    let id = env
        .elaborate_decl(
            "view t3 : Tree = \
             Node (Node Leaf 97 Leaf) 98 (Node Leaf 99 Leaf)",
        )
        .expect("t3 should declare");
    let _ = id;

    let result_id = env
        .elaborate_decl("view result : Nat = size t3")
        .expect("size t3 should elaborate");

    let mut store = ken_interp::eval::EvalStore::new();
    for (nid, v) in &env.num_values {
        store.num_values.entry(*nid).or_insert_with(|| match v {
            ken_elaborator::NumericLitVal::Int(n) => ken_interp::eval::EvalVal::from(*n),
            _ => panic!("unexpected literal kind in this test"),
        });
    }

    let body = match env.env.lookup(result_id) {
        Some(ken_kernel::Decl::Transparent { body, .. }) => body.clone(),
        _ => panic!("result should be Transparent"),
    };
    let v = ken_interp::eval::eval(&[], &body, &env.env, &mut store);

    // Walk the Suc-chain to a plain count.
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
    assert_eq!(nat_count(&env, &v), 3, "size of the 3-node tree must be 3, not a wrong-but-flat value");
}

#[test]
fn ac3_discriminating_pair_accepts_valid_rejects_ill_typed_sibling() {
    // Valid: both arms consistently typed Nat.
    let mut env_ok = fresh_env();
    env_ok.elaborate_decl(TREE_DECL).expect("Tree should declare");
    env_ok
        .elaborate_decl(
            "view depth (t : Tree) : Nat = \
             match t { Leaf => Zero ; Node l c r => Suc Zero }",
        )
        .expect("valid >=2-rec-field match must accept (AC3, positive half)");

    // Ill-typed sibling: one arm returns Nat, the other returns Char --
    // inconsistent motive across arms must still be rejected.
    let mut env_bad = fresh_env();
    env_bad.elaborate_decl(TREE_DECL).expect("Tree should declare");
    let res = env_bad.elaborate_decl(
        "view badDepth (t : Tree) : Nat = \
         match t { Leaf => Zero ; Node l c r => 99 }",
    );
    assert!(
        res.is_err(),
        "an arm at the wrong type must still be rejected -- the fix must not make the motive machinery permissive (AC3, negative half)"
    );
}

#[test]
fn ac4_no_regression_0_and_1_recursive_field_types() {
    // 0-recursive-field: List (Cons carries a non-recursive head + one
    // recursive tail -- actually List IS 1-recursive-field; use a
    // genuinely 0-rec-field enum for the true 0 case).
    let mut env = fresh_env();
    env.elaborate_decl("data Color = Red | Green | Blue")
        .expect("Color should declare");
    env.elaborate_decl(
        "view isRed (c : Color) : Nat = \
         match c { Red => Suc Zero ; Green => Zero ; Blue => Zero }",
    )
    .expect("0-recursive-field match must still elaborate (AC4)");

    // 1-recursive-field: `data NatList = NNil | NCons Nat NatList`.
    let mut env2 = fresh_env();
    env2.elaborate_decl("data NatList = NNil | NCons Nat NatList")
        .expect("NatList should declare");
    env2.elaborate_decl(
        "view natListLen (xs : NatList) : Nat = \
         match xs { NNil => Zero ; NCons h t => Suc (natListLen t) }",
    )
    .expect("single-recursive-field match must still elaborate (AC4)");

    // 1-rec + 1-other (mirrors VAL2's T8 bisection case): a tree node with
    // one recursive child and one non-recursive payload only.
    let mut env3 = fresh_env();
    env3.elaborate_decl("data Snoc = SLeaf Char | SNode Snoc Char")
        .expect("Snoc should declare");
    env3.elaborate_decl(
        "view snocLen (s : Snoc) : Nat = \
         match s { SLeaf c => Suc Zero ; SNode t c => Suc (snocLen t) }",
    )
    .expect("1-rec+1-other match must still elaborate (AC4)");
}

#[test]
fn ac4_three_recursive_fields_also_elaborates() {
    // The frame's own bisection went up to 3 (`T9`) -- confirm the fix
    // generalizes past exactly 2.
    let mut env = fresh_env();
    env.elaborate_decl("data Tri = TLeaf | TNode Tri Tri Tri")
        .expect("Tri should declare");
    env.elaborate_decl(
        "view natAdd (a : Nat) (b : Nat) : Nat = \
         match a { Zero => b ; Suc m => Suc (natAdd m b) }",
    )
    .expect("natAdd should declare");
    env.elaborate_decl(
        "view triSize (t : Tri) : Nat = \
         match t { \
           TLeaf => Zero ; \
           TNode a b c => Suc (natAdd (natAdd (triSize a) (triSize b)) (triSize c)) \
         }",
    )
    .expect("3-recursive-field match must elaborate using all 3 IHs");
}
