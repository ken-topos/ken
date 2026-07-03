//! `Map-build` acceptance tests (`docs/program/wp/Map-build.md`,
//! `spec/50-stdlib/52-map.md`, `conformance/stdlib/map/seed-map.md`).
//!
//! **Partial-scope candidate.** `insert`/`lookup`/`member`/`fromList` and the
//! `52 §5` proof obligations need a key comparator threaded generically over
//! an abstract `Ord k` dictionary — no landed mechanism exists for that yet
//! (confirmed empirically against `elab.rs`'s `instance_search`, escalated to
//! Architect, `evt_1wd56hecqhm06`/`evt_64j01esqw86pf`/`evt_1wsk6dracp10r` in
//! the Map-build thread). This file covers only what `packages/collections/
//! map.ken` ships today: the `Tree k v` carrier, `empty`, `toList`, `fold`,
//! and the `Pair`/`mkPair`/`pairFst`/`pairSnd` Σ-pair plumbing
//! (`ken-elaborator/src/prelude.rs`) those two ops route through. Extended
//! once the generic-dictionary gap resolves.

use ken_elaborator::{foreign::trusted_base_delta, ElabEnv, NumericLitVal};
use ken_interp::eval::{eval, EvalStore, EvalVal, ListCharIds};
use ken_kernel::{Decl, GlobalId};

const COLLECTIONS_KEN: &str = include_str!("../../../packages/collections/collections.ken");
const MAP_KEN: &str = include_str!("../../../packages/collections/map.ken");

fn mk_env() -> ElabEnv {
    let mut env = ElabEnv::new().expect("base env");
    env.elaborate_file(COLLECTIONS_KEN)
        .expect("packages/collections/collections.ken must elaborate");
    env.elaborate_file(MAP_KEN)
        .expect("packages/collections/map.ken must elaborate");
    env
}

fn make_store(env: &ElabEnv) -> EvalStore {
    let mut store = EvalStore::new();
    let mkdecimalpair_id = env.prelude_env.mkdecimalpair_id;
    for (id, v) in &env.num_values {
        store.num_values.insert(*id, lit_to_eval(v, mkdecimalpair_id));
    }
    store.list_char_ids = Some(ListCharIds {
        nil_id: env.prelude_env.nil_id,
        cons_id: env.prelude_env.cons_id,
    });
    store
}

fn lit_to_eval(v: &NumericLitVal, mkdecimalpair_id: GlobalId) -> EvalVal {
    match v {
        NumericLitVal::Int(n) => EvalVal::from(*n),
        NumericLitVal::Float(f) => EvalVal::Float(*f),
        NumericLitVal::Float32(f) => EvalVal::Float32(*f),
        NumericLitVal::Decimal { coeff, exp } => {
            ken_interp::decimal_value(mkdecimalpair_id, *coeff, *exp)
        }
        NumericLitVal::Str(s) => EvalVal::Str(s.clone()),
    }
}

fn eval_def(env: &ElabEnv, store: &mut EvalStore, id: GlobalId) -> EvalVal {
    match env.env.lookup(id) {
        Some(Decl::Transparent { body, .. }) => eval(&[], body, &env.env, store),
        _ => EvalVal::Unknown,
    }
}

fn eval_view(env: &mut ElabEnv, store: &mut EvalStore, name: &str, ty: &str, expr: &str) -> EvalVal {
    let src = format!("view {name} : {ty} = {expr}");
    let id = env
        .elaborate_decl(&src)
        .unwrap_or_else(|e| panic!("{name} failed to elaborate: {e}"));
    let mkdecimalpair_id = env.prelude_env.mkdecimalpair_id;
    for (nid, v) in &env.num_values {
        store.num_values.entry(*nid).or_insert_with(|| lit_to_eval(v, mkdecimalpair_id));
    }
    eval_def(env, store, id)
}

fn nat(n: u32) -> String {
    let mut s = "Zero".to_string();
    for _ in 0..n {
        s = format!("Suc ({s})");
    }
    s
}

/// Walk a `Nat` `Zero`/`Suc` `EvalVal::Ctor` chain to a plain count (`Nat` is
/// a real inductive, not an `Int`-backed immediate — mirrors
/// `l_match_ih_fix_acceptance.rs`'s `nat_count`).
fn nat_count(env: &ElabEnv, v: &EvalVal) -> u64 {
    match v {
        EvalVal::Ctor { id, args, .. } if *id == env.prelude_env.zero_id && args.is_empty() => 0,
        EvalVal::Ctor { id, args, .. } if *id == env.prelude_env.suc_id && args.len() == 1 => {
            1 + nat_count(env, &args[0])
        }
        other => panic!("expected a Nat Ctor chain, got {other:?}"),
    }
}

/// Decode a `List (Pair Nat Nat)` value into `Vec<(u64,u64)>` by walking the
/// `Nil`/`Cons` chain and each entry's `Pair` — mirrors
/// `l3_strings_roundtrip_acceptance.rs`'s `list_char_codepoints` walk.
fn list_pair_nat_nat(env: &ElabEnv, v: &EvalVal) -> Vec<(u64, u64)> {
    let nil_id = env.prelude_env.nil_id;
    let cons_id = env.prelude_env.cons_id;
    let mut out = Vec::new();
    let mut cur = v.clone();
    loop {
        match &cur {
            EvalVal::Ctor { id, .. } if *id == nil_id => return out,
            EvalVal::Ctor { id, args, .. } if *id == cons_id => {
                match &args[1] {
                    EvalVal::Pair { fst, snd, .. } => {
                        out.push((nat_count(env, fst), nat_count(env, snd)));
                    }
                    other => panic!("Cons head of List (Pair k v) must be an EvalVal::Pair, got {other:?}"),
                }
                cur = args[2].clone();
            }
            other => panic!("not a well-formed List Ctor chain: {other:?}"),
        }
    }
}

/// A hand-built 3-node `Tree Nat Nat`, deliberately inserted (constructed) in
/// NON-ascending key order — `2`, then `1` under it, then `3` under it — so
/// `toList`'s ascending-order claim is actually exercised (an
/// insertion/construction-order-preserving bug would fail this even though
/// it might pass a pre-sorted tree). `insert` isn't landed yet, so this is a
/// hand-built `Node` tree, not a real `insert` sequence — honestly the
/// non-`insert` half of AC2 (`toList`/`fold`'s own correctness), not a
/// stand-in for the deferred `insert`-driven round-trip.
fn tree_2_1_3() -> String {
    "Node Nat Nat \
       (Node Nat Nat (Leaf Nat Nat) (Suc Zero) (Suc Zero) (Leaf Nat Nat)) \
       (Suc (Suc Zero)) (Suc (Suc Zero)) \
       (Node Nat Nat (Leaf Nat Nat) (Suc (Suc (Suc Zero))) (Suc (Suc (Suc Zero))) (Leaf Nat Nat))"
        .to_string()
}

// ─────────────────────────────────────────────────────────────────────────────
// AC1 (partial) — carrier + ops admitted via declare_inductive/declare_def
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn tree_carrier_and_ops_are_not_primitive() {
    let env = mk_env();
    // `Tree` must be a real inductive (declare_inductive), never a primitive.
    let tree_id = env.globals["Tree"];
    assert!(
        matches!(env.env.lookup(tree_id), Some(Decl::Inductive { .. })),
        "Tree k v must be Decl::Inductive"
    );
    for name in ["empty", "toList", "fold", "Pair", "mkPair", "pairFst", "pairSnd"] {
        let id = env.globals[name];
        assert!(
            matches!(env.env.lookup(id), Some(Decl::Transparent { .. })),
            "{name} must be Decl::Transparent (declare_def), not a primitive/postulate"
        );
    }
    // Zero-NEW-delta: none of these mint a fresh trusted_base entry.
    let delta_empty = trusted_base_delta(&env.env, env.globals["empty"]);
    assert!(delta_empty.is_empty(), "empty must add zero trusted_base delta, got {delta_empty:?}");
    let delta_tolist = trusted_base_delta(&env.env, env.globals["toList"]);
    assert!(delta_tolist.is_empty(), "toList must add zero trusted_base delta, got {delta_tolist:?}");
    let delta_fold = trusted_base_delta(&env.env, env.globals["fold"]);
    assert!(delta_fold.is_empty(), "fold must add zero trusted_base delta, got {delta_fold:?}");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC2 (partial) — toList / fold correct end-to-end through the real interpreter
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn tolist_of_empty_is_nil() {
    let mut env = mk_env();
    let mut store = make_store(&env);
    let nil_id = env.globals["Nil"];
    let v = eval_view(&mut env, &mut store, "t_empty_tolist", "List (Pair Nat Nat)", "toList Nat Nat (empty Nat Nat)");
    assert!(matches!(v, EvalVal::Ctor { id, .. } if id == nil_id), "toList of empty must be Nil, got {v:?}");
}

#[test]
fn tolist_ascending_by_key_on_hand_built_tree() {
    run_with_big_stack(|| {
        let mut env = mk_env();
        let mut store = make_store(&env);
        let v = eval_view(
            &mut env,
            &mut store,
            "t_tolist",
            "List (Pair Nat Nat)",
            &format!("toList Nat Nat ({})", tree_2_1_3()),
        );
        let out = list_pair_nat_nat(&env, &v);
        // The flip: a bug emitting construction/insertion order instead of
        // in-order-by-key traversal would yield [(2,2),(1,1),(3,3)] — this
        // asserts the ASCENDING list, not just the element set.
        assert_eq!(out, vec![(1, 1), (2, 2), (3, 3)], "toList must be ascending by key, got {out:?}");
    });
}

#[test]
fn fold_agrees_with_left_fold_over_tolist() {
    run_with_big_stack(|| {
        let mut env = mk_env();
        let mut store = make_store(&env);
        // Order-sensitive `f`: append the key onto an accumulator list, so a
        // fold visiting a different order than toList's ascending order
        // yields a different (non-commutative) result list.
        let fold_src = format!(
            "fold Nat Nat (List Nat) (\\k.\\v.\\acc. list_append Nat acc (Cons Nat k (Nil Nat))) (Nil Nat) ({})",
            tree_2_1_3()
        );
        let v = eval_view(&mut env, &mut store, "t_fold", "List Nat", &fold_src);
        let nil_id = env.globals["Nil"];
        let cons_id = env.globals["Cons"];
        let mut out = Vec::new();
        let mut cur = v;
        loop {
            match &cur {
                EvalVal::Ctor { id, .. } if *id == nil_id => break,
                EvalVal::Ctor { id, args, .. } if *id == cons_id => {
                    out.push(nat_count(&env, &args[1]));
                    cur = args[2].clone();
                }
                other => panic!("not a well-formed List chain: {other:?}"),
            }
        }
        assert_eq!(out, vec![1, 2, 3], "fold must visit ascending key order, matching toList; got {out:?}");
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Pair (Σ-pair, `52 §4`) plumbing sanity — `mkPair`/`pairFst`/`pairSnd`
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn pair_roundtrip() {
    let mut env = mk_env();
    let mut store = make_store(&env);
    let v = eval_view(&mut env, &mut store, "t_fst", "Nat", &format!("pairFst Nat Nat (mkPair Nat Nat ({}) ({}))", nat(3), nat(4)));
    assert_eq!(nat_count(&env, &v), 3, "pairFst (mkPair 3 4) must be 3, got {v:?}");
    let v = eval_view(&mut env, &mut store, "t_snd", "Nat", &format!("pairSnd Nat Nat (mkPair Nat Nat ({}) ({}))", nat(3), nat(4)));
    assert_eq!(nat_count(&env, &v), 4, "pairSnd (mkPair 3 4) must be 4, got {v:?}");
}

fn run_with_big_stack<F: FnOnce() + Send + 'static>(f: F) {
    std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(f)
        .expect("spawn big-stack test thread")
        .join()
        .expect("test thread panicked");
}
