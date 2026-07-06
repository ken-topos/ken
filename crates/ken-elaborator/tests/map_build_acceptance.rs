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
const TRANSPORT_KEN: &str = include_str!("../../../packages/transport/transport.ken");
const MAP_KEN: &str = include_str!("../../../packages/collections/map.ken");

fn mk_env() -> ElabEnv {
    let mut env = ElabEnv::new().expect("base env");
    env.elaborate_file(COLLECTIONS_KEN)
        .expect("packages/collections/collections.ken must elaborate");
    env.elaborate_file(TRANSPORT_KEN)
        .expect("packages/transport/transport.ken must elaborate");
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
    let src = format!("const {name} : {ty} = {expr}");
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

fn list_nat(env: &ElabEnv, v: &EvalVal) -> Vec<u64> {
    let nil_id = env.prelude_env.nil_id;
    let cons_id = env.prelude_env.cons_id;
    let mut out = Vec::new();
    let mut cur = v.clone();
    loop {
        match &cur {
            EvalVal::Ctor { id, .. } if *id == nil_id => return out,
            EvalVal::Ctor { id, args, .. } if *id == cons_id => {
                out.push(nat_count(env, &args[1]));
                cur = args[2].clone();
            }
            other => panic!("not a well-formed List Nat chain: {other:?}"),
        }
    }
}

fn option_nat(env: &ElabEnv, v: &EvalVal) -> Option<u64> {
    let none_id = env.globals["None"];
    let some_id = env.globals["Some"];
    match v {
        EvalVal::Ctor { id, .. } if *id == none_id => None,
        EvalVal::Ctor { id, args, .. } if *id == some_id => Some(nat_count(env, &args[1])),
        other => panic!("not an Option Nat value: {other:?}"),
    }
}

fn bool_value(env: &ElabEnv, v: &EvalVal) -> bool {
    let true_id = env.globals["True"];
    let false_id = env.globals["False"];
    match v {
        EvalVal::Ctor { id, .. } if *id == true_id => true,
        EvalVal::Ctor { id, .. } if *id == false_id => false,
        other => panic!("not a Bool value: {other:?}"),
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
// AC1/AC5 (partial) — insert/lookup/member/fromList admitted, non-primitive
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn map_ops_full_api_not_primitive() {
    let env = mk_env();
    for name in ["insert", "lookup", "member", "fromList", "fromListAcc", "setInsert", "setMember", "setToList", "Ordered", "allKeys", "lookupEmptyIsNone"] {
        let id = env.globals[name];
        assert!(
            matches!(env.env.lookup(id), Some(Decl::Transparent { .. })),
            "{name} must be Decl::Transparent (declare_def), not a primitive/postulate"
        );
        let delta = trusted_base_delta(&env.env, id);
        assert!(delta.is_empty(), "{name} must add zero trusted_base delta, got {delta:?}");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC2 — insert/lookup/member/fromList correct end-to-end (Char keys, `leqChar`
// computes — `52 §5.4` — never hand-fed: constructed via real `insert`, read
// via real `lookup`/`toList`)
// ─────────────────────────────────────────────────────────────────────────────

fn char_lit(c: char) -> String {
    (c as u32).to_string()
}

#[test]
fn insert_lookup_roundtrip_some() {
    let mut env = mk_env();
    let mut store = make_store(&env);
    let some_id = env.globals["Some"];
    let expr = format!(
        "lookup Char Char leqChar ({k}) (insert Char Char leqChar ({k}) ({v}) (empty Char Char))",
        k = char_lit('k'),
        v = char_lit('v')
    );
    let v = eval_view(&mut env, &mut store, "t_roundtrip", "Option Char", &expr);
    match v {
        EvalVal::Ctor { id, ref args, .. } if id == some_id => {
            assert_eq!(args[1], EvalVal::Int('v' as i64), "lookup after insert must return Some 'v', got {args:?}");
        }
        other => panic!("insert-then-lookup must be Some 'v'; got {other:?}"),
    }
}

#[test]
fn lookup_order_distinct_key_is_none() {
    let mut env = mk_env();
    let mut store = make_store(&env);
    let none_id = env.globals["None"];
    // Insert 'k', query the order-distinct 'z' — a "return whatever was
    // last inserted regardless of key" bug would yield Some, not None.
    let expr = format!(
        "lookup Char Char leqChar ({z}) (insert Char Char leqChar ({k}) ({v}) (empty Char Char))",
        z = char_lit('z'),
        k = char_lit('k'),
        v = char_lit('v')
    );
    let v = eval_view(&mut env, &mut store, "t_distinct", "Option Char", &expr);
    assert!(matches!(v, EvalVal::Ctor { id, .. } if id == none_id), "distinct-key lookup must be None, got {v:?}");

    let expr_empty = format!("lookup Char Char leqChar ({z}) (empty Char Char)", z = char_lit('z'));
    let v = eval_view(&mut env, &mut store, "t_lookup_empty", "Option Char", &expr_empty);
    assert!(matches!(v, EvalVal::Ctor { id, .. } if id == none_id), "lookup on empty must be None, got {v:?}");
}

#[test]
fn overwrite_last_writer_wins() {
    let mut env = mk_env();
    let mut store = make_store(&env);
    let some_id = env.globals["Some"];
    let expr = format!(
        "lookup Char Char leqChar ({k}) (insert Char Char leqChar ({k}) ({v2}) (insert Char Char leqChar ({k}) ({v1}) (empty Char Char)))",
        k = char_lit('k'),
        v1 = char_lit('1'),
        v2 = char_lit('2')
    );
    let v = eval_view(&mut env, &mut store, "t_overwrite_lookup", "Option Char", &expr);
    match v {
        EvalVal::Ctor { id, ref args, .. } if id == some_id => {
            assert_eq!(args[1], EvalVal::Int('2' as i64), "re-insert must overwrite to the LAST writer's value, got {args:?}");
        }
        other => panic!("overwrite lookup must be Some '2'; got {other:?}"),
    }
    let tolist_expr = format!(
        "toList Char Char (insert Char Char leqChar ({k}) ({v2}) (insert Char Char leqChar ({k}) ({v1}) (empty Char Char)))",
        k = char_lit('k'),
        v1 = char_lit('1'),
        v2 = char_lit('2')
    );
    let v = eval_view(&mut env, &mut store, "t_overwrite_tolist", "List (Pair Char Char)", &tolist_expr);
    let out = list_pair_char_char(&env, &v);
    assert_eq!(out, vec![('k' as i64, '2' as i64)], "re-insert must NOT duplicate the node, got {out:?}");
}

#[test]
fn tolist_ascending_via_real_insert() {
    // Insert in deliberately non-ascending order — the real AC2 driver,
    // superseding the hand-built-tree probe now that `insert` is landed.
    let mut env = mk_env();
    let mut store = make_store(&env);
    let expr = format!(
        "toList Char Char (insert Char Char leqChar ({a}) ({a}) (insert Char Char leqChar ({c}) ({c}) (insert Char Char leqChar ({b}) ({b}) (empty Char Char))))",
        a = char_lit('a'),
        b = char_lit('b'),
        c = char_lit('c')
    );
    let v = eval_view(&mut env, &mut store, "t_tolist_real_insert", "List (Pair Char Char)", &expr);
    let out = list_pair_char_char(&env, &v);
    assert_eq!(
        out,
        vec![('a' as i64, 'a' as i64), ('b' as i64, 'b' as i64), ('c' as i64, 'c' as i64)],
        "toList over a real b,c,a-order insert sequence must be ascending by key, got {out:?}"
    );
}

#[test]
fn fromlist_last_writer_and_ordered() {
    let mut env = mk_env();
    let mut store = make_store(&env);
    // [(2,'b'),(1,'a'),(2,'c')] -> toList must be [(1,'a'),(2,'c')]: ascending
    // AND the LAST list entry ('c') wins on the duplicate key 2.
    let list_expr = format!(
        "Cons (Pair Char Char) (mkPair Char Char ({two}) ({b})) \
           (Cons (Pair Char Char) (mkPair Char Char ({one}) ({a})) \
             (Cons (Pair Char Char) (mkPair Char Char ({two}) ({c})) (Nil (Pair Char Char))))",
        two = char_lit('2'),
        one = char_lit('1'),
        a = char_lit('a'),
        b = char_lit('b'),
        c = char_lit('c')
    );
    let expr = format!("toList Char Char (fromList Char Char leqChar ({list_expr}))");
    let v = eval_view(&mut env, &mut store, "t_fromlist", "List (Pair Char Char)", &expr);
    let out = list_pair_char_char(&env, &v);
    assert_eq!(
        out,
        vec![('1' as i64, 'a' as i64), ('2' as i64, 'c' as i64)],
        "fromList must be ascending AND last-writer-wins ('c' beats 'b' on key '2'), got {out:?}"
    );
}

#[test]
fn set_is_map_unit() {
    let mut env = mk_env();
    let mut store = make_store(&env);
    let true_id = env.globals["True"];
    let false_id = env.globals["False"];
    let expr = format!(
        "setMember Char leqChar ({a}) (setInsert Char leqChar ({a}) (setInsert Char leqChar ({b}) (Leaf Char Unit)))",
        a = char_lit('a'),
        b = char_lit('b')
    );
    let v = eval_view(&mut env, &mut store, "t_set_member_hit", "Bool", &expr);
    assert!(matches!(v, EvalVal::Ctor { id, .. } if id == true_id), "member of an inserted element must be True, got {v:?}");

    let expr_absent = format!(
        "setMember Char leqChar ({z}) (setInsert Char leqChar ({a}) (Leaf Char Unit))",
        z = char_lit('z'),
        a = char_lit('a')
    );
    let v = eval_view(&mut env, &mut store, "t_set_member_miss", "Bool", &expr_absent);
    assert!(matches!(v, EvalVal::Ctor { id, .. } if id == false_id), "member of an absent element must be False, got {v:?}");

    let tolist_expr = format!(
        "setToList Char (setInsert Char leqChar ({a}) (setInsert Char leqChar ({c}) (setInsert Char leqChar ({b}) (Leaf Char Unit))))",
        a = char_lit('a'),
        b = char_lit('b'),
        c = char_lit('c')
    );
    let v = eval_view(&mut env, &mut store, "t_set_tolist", "List Char", &tolist_expr);
    let nil_id = env.globals["Nil"];
    let cons_id = env.globals["Cons"];
    let mut out = Vec::new();
    let mut cur = v;
    loop {
        match &cur {
            EvalVal::Ctor { id, .. } if *id == nil_id => break,
            EvalVal::Ctor { id, args, .. } if *id == cons_id => {
                match &args[1] {
                    EvalVal::Int(n) => out.push(*n),
                    other => panic!("setToList head must be Char-as-Int, got {other:?}"),
                }
                cur = args[2].clone();
            }
            other => panic!("not a well-formed List chain: {other:?}"),
        }
    }
    assert_eq!(out, vec!['a' as i64, 'b' as i64, 'c' as i64], "setToList must be ascending, got {out:?}");
}

#[test]
fn letter_frequency_shape() {
    let mut env = mk_env();
    let mut store = make_store(&env);
    // "banana": b,a,n,a,n,a -> {'a':3,'b':1,'n':2}, ascending by key.
    env.elaborate_decl(
        "fn bumpCount (leq : Char -> Char -> Bool) (key : Char) (m : Tree Char Nat) : Tree Char Nat = \
         match lookup Char Nat leq key m { \
           None => insert Char Nat leq key (Suc Zero) m ; \
           Some n => insert Char Nat leq key (Suc n) m \
         }",
    )
    .expect("bumpCount should elaborate");
    env.elaborate_decl(
        "fn countChars (leq : Char -> Char -> Bool) (cs : List Char) (m : Tree Char Nat) : Tree Char Nat = \
         match cs { \
           Nil => m ; \
           Cons c cs2 => countChars leq cs2 (bumpCount leq c m) \
         }",
    )
    .expect("countChars should elaborate");
    let banana = format!(
        "Cons Char ({b}) (Cons Char ({a}) (Cons Char ({n}) (Cons Char ({a}) (Cons Char ({n}) (Cons Char ({a}) (Nil Char))))))",
        b = char_lit('b'),
        a = char_lit('a'),
        n = char_lit('n')
    );
    let expr = format!("toList Char Nat (countChars leqChar ({banana}) (empty Char Nat))");
    let v = eval_view(&mut env, &mut store, "t_letter_freq", "List (Pair Char Nat)", &expr);
    let nil_id = env.globals["Nil"];
    let cons_id = env.globals["Cons"];
    let mut out = Vec::new();
    let mut cur = v;
    loop {
        match &cur {
            EvalVal::Ctor { id, .. } if *id == nil_id => break,
            EvalVal::Ctor { id, args, .. } if *id == cons_id => {
                match &args[1] {
                    EvalVal::Pair { fst, snd, .. } => {
                        let key = match fst.as_ref() {
                            EvalVal::Int(n) => *n,
                            other => panic!("pair fst must be Char-as-Int, got {other:?}"),
                        };
                        out.push((key, nat_count(&env, snd)));
                    }
                    other => panic!("Cons head must be an EvalVal::Pair, got {other:?}"),
                }
                cur = args[2].clone();
            }
            other => panic!("not a well-formed List chain: {other:?}"),
        }
    }
    assert_eq!(
        out,
        vec![('a' as i64, 3), ('b' as i64, 1), ('n' as i64, 2)],
        "letter-frequency('banana') must be ascending-by-key [('a',3),('b',1),('n',2)], got {out:?}"
    );
}

fn list_pair_char_char(env: &ElabEnv, v: &EvalVal) -> Vec<(i64, i64)> {
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
                        let k = match fst.as_ref() {
                            EvalVal::Int(n) => *n,
                            other => panic!("pair fst must be Char-as-Int, got {other:?}"),
                        };
                        let vv = match snd.as_ref() {
                            EvalVal::Int(n) => *n,
                            other => panic!("pair snd must be Char-as-Int, got {other:?}"),
                        };
                        out.push((k, vv));
                    }
                    other => panic!("Cons head must be an EvalVal::Pair, got {other:?}"),
                }
                cur = args[2].clone();
            }
            other => panic!("not a well-formed List Ctor chain: {other:?}"),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC3 (partial) — the ordered-invariant law's own base case (trivial),
// Ordered/allKeys admitted as declare_def (never a postulate)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn lookup_empty_law_is_a_real_reducing_proof() {
    let mut env = mk_env();
    // The stated law itself (`lookupEmptyIsNone`) must be admitted as a
    // real Decl::Transparent proof term (never Decl::Opaque/Axiom).
    let id = env.globals["lookupEmptyIsNone"];
    assert!(
        matches!(env.env.lookup(id), Some(Decl::Transparent { .. })),
        "lookupEmptyIsNone must be a real proof term, not a postulate"
    );
    // `Ordered` on an empty map is provable by `tt` — the invariant reduces
    // to a trivially-true Prop (Equal Bool True True) at Leaf, closable the
    // same way `lookupEmptyIsNone` closes (K5 same-nullary-ctor collapse).
    // This is a kernel CHECK (is the type inhabited), not an `eval` — the
    // Prop itself is a type, not a runtime data value.
    env.elaborate_decl(
        "fn orderedEmptyProof (k : Type) (v : Type) (leq : k -> k -> Bool) : \
         Ordered k v leq (empty k v) = tt",
    )
    .expect("Ordered on an empty map must be provable by tt");
}

// ─────────────────────────────────────────────────────────────────────────────
// Law 4 (`54 §3`, "toList ordered") — `toListOrdered`
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn tolistordered_law4_is_a_real_general_proof_term() {
    let env = mk_env();
    // `toListOrdered : (k v : Type) -> (leq : k -> k -> Bool) -> (m : Tree k v)
    //   -> Ordered k v leq m -> isSorted (Pair k v) (pairLeq k v leq) (toList k v m)`
    // must be admitted as a real Decl::Transparent proof term (never a
    // postulate/axiom) — this IS the whole-body `declare_def` kernel recheck
    // that used to OOM (~12 GB) before `wp/obs-eq-termination` (`9cf468a`)
    // fixed the underlying conv/obs termination bug; `mk_env()` above
    // elaborating `map.ken` at all is itself the completion proof, this just
    // pins the trust-level assertion on the specific declaration.
    for name in [
        "toListOrdered",
        "isSortedAppend",
        "consSortedHead",
        "allKeysToAllInList",
        "allInListAppendIntro",
    ] {
        let id = env.globals[name];
        assert!(
            matches!(env.env.lookup(id), Some(Decl::Transparent { .. })),
            "{name} must be a real proof term, not a postulate"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Law 1 (`54 §5.1`, Map capstone unit 2) — `preservesOrdered`
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn preservesordered_law1_is_a_real_general_proof_term() {
    let env = mk_env();
    // `preservesOrdered : ... -> Ordered m -> Ordered (insert key val m)`
    // must be a real Decl::Transparent proof term (never a postulate) —
    // the whole-body `declare_def` kernel recheck for the top-level
    // induction plus every supporting transport bridge / comparison-
    // independent lemma / totality-derived reflection it composes.
    for name in [
        "preservesOrdered",
        "insertCaseTransportDispatch",
        "dispatchOnQ1",
        "dispatchOnQ2",
        "insertCaseTransportOverwrite",
        "insertCaseTransportIntoL",
        "insertCaseTransportIntoR",
        "insertPreservesAllKeys",
        "allKeysTransBelow",
        "allKeysTransAbove",
        "deriveFromFalse",
    ] {
        let id = env.globals[name];
        assert!(
            matches!(env.env.lookup(id), Some(Decl::Transparent { .. })),
            "{name} must be a real proof term, not a postulate"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Law 2 (`54 §5.2`, Map capstone unit 2) — `lookupFoundAfterInsert`
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn lookupfoundafterinsert_law2_is_a_real_general_proof_term() {
    let env = mk_env();
    // `lookupFoundAfterInsert : ... -> lookup key (insert key val m) =
    // Some val` must be a real Decl::Transparent proof term (never a
    // postulate) — reuses Law 1's goal-generic transport bridges directly
    // (asserted there), plus its own lookup-side step mirrors/bridges.
    for name in [
        "lookupFoundAfterInsert",
        "lookupFoundDispatch",
        "lookupFoundDispatchQ1",
        "lookupFoundDispatchQ2",
        "lookupOverwriteResult",
        "lookupIntoLBridge",
        "lookupIntoRBridge",
    ] {
        let id = env.globals[name];
        assert!(
            matches!(env.env.lookup(id), Some(Decl::Transparent { .. })),
            "{name} must be a real proof term, not a postulate"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Law 3 (`54 §5.2`, Map capstone unit 2) — `lookupLocality`
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn lookuplocality_law3_is_a_real_general_proof_term() {
    let env = mk_env();
    // `lookupLocality : distinct key key' -> lookup key' (insert key val m)
    // = lookup key' m` must be a real Decl::Transparent proof term — reuses
    // Law 1's goal-generic transport bridges directly (asserted there) and
    // Law 2's lookupIntoLBridge/IntoRBridge, plus its own agreement lemmas.
    for name in [
        "lookupLocality",
        "lookupLocalityNodeDispatch",
        "lookupLocalityQ2Dispatch",
        "lookupLeafLocalityWitness",
        "lookupOverwriteLocalityWitness",
        "lookupIntoLLocalityWitness",
        "lookupIntoRLocalityWitness",
        "boolValueEqFromBiimpl",
        "lookupOverwriteAgreesOuter",
        "lookupOverwriteAgreesInner",
    ] {
        let id = env.globals[name];
        assert!(
            matches!(env.env.lookup(id), Some(Decl::Transparent { .. })),
            "{name} must be a real proof term, not a postulate"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Law 5 (`54 §5.3`, Map capstone unit 2 — the final law) — `lookupAssocAgree`
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn lookupassocagree_law5_is_a_real_general_proof_term() {
    let env = mk_env();
    // `lookupAssocAgree : Ordered m -> Distinct leq m -> lookup key m =
    // assoc key (toList m)` must be a real Decl::Transparent proof term —
    // the restated statement (Distinct precondition added after the
    // original false statement was caught and escalated), reusing Law 1's
    // goal-generic transport bridges and Law 2's lookup-side bridges.
    for name in [
        "lookupAssocAgree",
        "law5NodeDispatch",
        "law5NodeQ2Dispatch",
        "law5DistinctL",
        "law5DistinctR",
        "orderEquiv",
        "NoDup",
        "Distinct",
        "distinctEmpty",
        "assocSkipPrefix",
        "assocPrefixWins",
        "assocNoneImpliesNoMatchInner",
        "assocNoneImpliesNoMatchDispatch",
        "assocNoneImpliesNoMatch",
        "assocNoMatchIsNone",
        "noDupAppendHeadExcl",
        "noDupAppendLeft",
        "noDupAppendRight",
        "notMatchTransferViaEquiv",
        "lookupStopBridge",
        "lookupOrderEquivOuterAgree",
        "lookupOrderEquivInnerAgree",
        "lookupOrderEquivBothFalse",
        "lookupOrderEquivBothInnerFalse",
        "lookupOrderEquivBothStop",
        "lookupOrderEquivInnerDispatch",
        "lookupOrderEquivNodeDispatch",
        "lookupOrderEquivAgree",
        "memberOrderEquivAgree",
        "setMemberOrderEquivAgree",
    ] {
        let id = env.globals[name];
        assert!(
            matches!(env.env.lookup(id), Some(Decl::Transparent { .. })),
            "{name} must be a real proof term, not a postulate"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CAT-4 (`58`) — Layer-2 keyed collections / sets / relations
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn cat4_new_api_is_derived_and_axiom_free() {
    let env = mk_env();
    for name in [
        "bool_and",
        "bool_not",
        "cat4_bool_or",
        "cat4_bool_or_comm",
        "cat4_bool_or_assoc",
        "cat4_bool_or_idempotent",
        "cat4_bool_or_left_identity",
        "cat4_bool_or_right_identity",
        "bool_and_comm",
        "bool_and_assoc",
        "bool_and_idempotent",
        "bool_and_left_identity",
        "bool_and_right_identity",
        "leqNat",
        "reflLeqNat",
        "transLeqNat",
        "antisymLeqNat",
        "totalLeqNat",
        "orderEquivKey",
        "boolAndTrueIntro",
        "orderEquivKeyTrueFromOrderEquiv",
        "orderEquivFromOrderEquivKeyTrueInner",
        "orderEquivFromOrderEquivKeyTrueDispatch",
        "orderEquivFromOrderEquivKeyTrue",
        "orderEquivKeyFalseToNot",
        "notOrderEquivFromLeftFalse",
        "notOrderEquivFromRightFalse",
        "dropKey",
        "deleteFromListAcc",
        "deleteFromListAccStep",
        "deleteFromListAccFinalBridge",
        "deleteFromListAccStepTrueEq",
        "deleteFromListAccStepFalseEq",
        "deleteFromListAccStepTrueReduces",
        "deleteFromListAccStepFalseReduces",
        "deleteFromListAccTrueBridge",
        "deleteFromListAccFalseBridge",
        "deleteFromList",
        "delete",
        "deleteFromListAccLookupNoneDispatch",
        "deleteFromListAccLookupNone",
        "deleteLookupNoneLaw",
        "notOrderEquivFromDeletedMatch",
        "deleteFromListAccLookupLocalityDispatch",
        "deleteFromListAccLookupLocality",
        "deleteFromListAccLookupOtherAssocDeletedHitAbsurd",
        "deleteFromListAccLookupOtherAssocHitSurvivor",
        "deleteFromListAccLookupOtherAssocHit",
        "deleteFromListAccLookupOtherAssocMiss",
        "deleteFromListAccLookupOtherAssocInner",
        "deleteFromListAccLookupOtherAssocDispatch",
        "deleteFromListAccLookupOtherAssoc",
        "deleteLookupOtherKeyLaw",
        "fromListAccPreservesOrdered",
        "fromListPreservesOrdered",
        "deleteFromListAccPreservesOrderedDispatch",
        "deleteFromListAccPreservesOrdered",
        "deleteFromListPreservesOrdered",
        "deletePreservesOrdered",
        "insertWith",
        "insertWithFoldStep",
        "insertWithFoldStepReduces",
        "unionFromListAcc",
        "unionFromListAccConsBridge",
        "union",
        "unionLookupTable",
        "optionIsSome",
        "unitCombine",
        "unionLookupTableMember",
        "intersectionLookupTable",
        "differenceLookupTable",
        "differenceLookupTableFalseNoneNone",
        "differenceLookupTableFalseNoneSome",
        "differenceLookupTableFalseNone",
        "differenceLookupExpected",
        "differenceLookupExpectedTrue",
        "differenceLookupExpectedFalse",
        "differenceLookupExpectedMemberOption",
        "differenceLookupExpectedMemberTable",
        "differenceLookupExpectedMember",
        "insertWithLookupResult",
        "insertWithLookupResultFor",
        "insertWithLookupOverwriteWitness",
        "insertWithLookupIntoLWitness",
        "insertWithLookupIntoRWitness",
        "insertWithLookupDispatchQ2",
        "insertWithLookupDispatchQ1",
        "insertWithLookupCharacterization",
        "lookupReplaceLInnerDispatch",
        "lookupReplaceLDispatch",
        "lookupReplaceLWitness",
        "lookupReplaceRInnerDispatch",
        "lookupReplaceRDispatch",
        "lookupReplaceRWitness",
        "insertLookupHit",
        "insertWithLookupLocalityQ2Dispatch",
        "insertWithLookupLocalityNodeDispatch",
        "insertWithLookupLocality",
        "insertWithFoldStepLookupLocality",
        "insertWithFoldStepLookupHit",
        "unionFromListAccLookupAssocHit",
        "unionFromListAccLookupAssocMiss",
        "unionFromListAccLookupAssocInner",
        "unionFromListAccLookupAssocDispatch",
        "unionFromListAccLookupAssoc",
        "insertWithFoldStepPreservesOrdered",
        "unionFromListAccPreservesOrdered",
        "unionLookupCharacterization",
        "unionLookupBothNoneLaw",
        "unionLookupLeftOnlyLaw",
        "unionLookupRightOnlyLaw",
        "unionLookupBothSomeLaw",
        "memberFromLookupNone",
        "memberFromLookupSome",
        "lookupNoneFromMemberFalseHit",
        "lookupNoneFromMemberFalse",
        "lookupUnitSomeFromMemberTrueLeaf",
        "lookupUnitSomeFromMemberTrueHit",
        "lookupUnitSomeFromMemberTrue",
        "notOrderEquivFromMemberTrueFalse",
        "notOrderEquivFromMemberFalseTrue",
        "intersectionFromListAccLookupNoneDispatch",
        "intersectionFromListAccLookupNone",
        "intersectionFromListAccLookupLocalityDispatch",
        "intersectionFromListAccLookupLocality",
        "intersectionFromListAccLookupSomeHit",
        "intersectionFromListAccLookupSomeMissDispatch",
        "intersectionFromListAccLookupSomeInner",
        "intersectionFromListAccLookupSomeDispatch",
        "intersectionFromListAccLookupSome",
        "intersectionLookupLeftNoneLaw",
        "differenceFromListAccLookupLocalityDispatch",
        "differenceFromListAccLookupLocality",
        "differenceFromListAccLookupNoneDispatch",
        "differenceFromListAccLookupNone",
        "differenceFromListAccLookupKeepHit",
        "differenceFromListAccLookupKeepMissDispatch",
        "differenceFromListAccLookupKeepInner",
        "differenceFromListAccLookupKeepDispatch",
        "differenceFromListAccLookupKeep",
        "differenceLookupCharacterizationReject",
        "differenceLookupCharacterizationKeep",
        "differenceLookupCharacterizationDispatch",
        "allKeysMapNotMatchBelow",
        "allKeysMapNotMatchAbove",
        "intersectionFromListAcc",
        "intersectionFromListAccStep",
        "intersectionFromListAccFinalBridge",
        "intersectionFromListAccStepTrueEq",
        "intersectionFromListAccStepFalseEq",
        "intersectionFromListAccStepTrueReduces",
        "intersectionFromListAccStepFalseReduces",
        "intersectionFromListAccTrueBridge",
        "intersectionFromListAccFalseBridge",
        "intersection",
        "intersectionLookupCharacterization",
        "intersectionLookupSomeLaw",
        "differenceFromListAcc",
        "differenceFromListAccStep",
        "differenceFromListAccFinalBridge",
        "differenceFromListAccStepTrueEq",
        "differenceFromListAccStepFalseEq",
        "differenceFromListAccStepTrueReduces",
        "differenceFromListAccStepFalseReduces",
        "differenceFromListAccTrueBridge",
        "differenceFromListAccFalseBridge",
        "difference",
        "differenceLookupCharacterization",
        "insertFoldStep",
        "foldInsertPreservesOrdered",
        "insertWithPreservesOrdered",
        "foldInsertWithPreservesOrdered",
        "unionPreservesOrdered",
        "intersectionFromListAccPreservesOrderedDispatch",
        "intersectionFromListAccPreservesOrdered",
        "intersectionPreservesOrdered",
        "differenceFromListAccPreservesOrderedDispatch",
        "differenceFromListAccPreservesOrdered",
        "differencePreservesOrdered",
        "setUnion",
        "setIntersection",
        "setDifference",
        "setUnionMemberLaw",
        "setIntersectionMemberLeftFalseRhs",
        "setIntersectionMemberRightFalseRhs",
        "setIntersectionMemberBothTrueRhs",
        "setIntersectionMemberLeftFalseCase",
        "setIntersectionMemberRightFalseCase",
        "setIntersectionMemberBothTrueCase",
        "setIntersectionMemberRightDispatch",
        "setIntersectionMemberDispatch",
        "setIntersectionMemberLaw",
        "setDifferenceMemberLaw",
        "setMemberEmptyFalse",
        "setUnionCommLaw",
        "setUnionAssocLaw",
        "setUnionIdempotentLaw",
        "setUnionIdentityLaw",
        "setIntersectionCommLaw",
        "setIntersectionAssocLaw",
        "setIntersectionIdempotentLaw",
        "setIntersectionIdentityLaw",
        "pairVals",
        "pairKeysPreservesSortedCons",
        "pairKeysPreservesSorted",
        "keys",
        "values",
        "keysProjectToList",
        "valuesProjectToList",
        "keysValuesProjectionCoherence",
        "keysAscending",
        "succ",
        "relMember",
        "addEdge",
        "composeSuccStep",
        "composeSucc",
        "compose",
        "converseTargets",
        "converse",
        "isReflexive",
        "isSymmetric",
        "isTransitive",
        "isEquivalence",
    ] {
        let id = env.globals[name];
        assert!(
            matches!(env.env.lookup(id), Some(Decl::Transparent { .. })),
            "{name} must be transparent derived Ken, not a primitive/postulate"
        );
        let delta = trusted_base_delta(&env.env, id);
        assert!(delta.is_empty(), "{name} must add zero trusted_base delta, got {delta:?}");
    }
}

#[test]
fn cat4_delete_dropkey_filters_all_equivalent_keys() {
    let mut env = mk_env();
    let mut store = make_store(&env);
    let list_expr = format!(
        "Cons (Pair Nat Nat) (mkPair Nat Nat ({one}) ({ten})) \
           (Cons (Pair Nat Nat) (mkPair Nat Nat ({two}) ({twenty})) \
             (Cons (Pair Nat Nat) (mkPair Nat Nat ({one}) ({thirty})) (Nil (Pair Nat Nat))))",
        one = nat(1),
        two = nat(2),
        ten = nat(10),
        twenty = nat(20),
        thirty = nat(30)
    );
    let v = eval_view(
        &mut env,
        &mut store,
        "t_cat4_dropkey_filter",
        "List (Pair Nat Nat)",
        &format!("dropKey Nat Nat leqNat ({}) ({list_expr})", nat(1)),
    );
    assert_eq!(
        list_pair_nat_nat(&env, &v),
        vec![(2, 20)],
        "dropKey must filter every order-equivalent key, not just the first match"
    );

    let m = format!(
        "insert Nat Nat leqNat ({one}) ({ten}) \
           (insert Nat Nat leqNat ({two}) ({twenty}) (empty Nat Nat))",
        one = nat(1),
        two = nat(2),
        ten = nat(10),
        twenty = nat(20)
    );
    let v = eval_view(
        &mut env,
        &mut store,
        "t_cat4_delete_lookup",
        "Option Nat",
        &format!("lookup Nat Nat leqNat ({}) (delete Nat Nat leqNat ({}) ({m}))", nat(1), nat(1)),
    );
    assert_eq!(option_nat(&env, &v), None, "deleted key must look up as None");
}

#[test]
fn cat4_union_intersection_difference_execute_over_nat() {
    let mut env = mk_env();
    let mut store = make_store(&env);
    let a = format!(
        "insert Nat Nat leqNat ({one}) ({ten}) \
           (insert Nat Nat leqNat ({two}) ({twenty}) (empty Nat Nat))",
        one = nat(1),
        two = nat(2),
        ten = nat(10),
        twenty = nat(20)
    );
    let b = format!(
        "insert Nat Nat leqNat ({one}) ({thirty}) \
           (insert Nat Nat leqNat ({three}) ({forty}) (empty Nat Nat))",
        one = nat(1),
        three = nat(3),
        thirty = nat(30),
        forty = nat(40)
    );

    let v = eval_view(
        &mut env,
        &mut store,
        "t_cat4_union_orientation",
        "Option Nat",
        &format!("lookup Nat Nat leqNat ({}) (union Nat Nat leqNat (λx.λy. x) ({a}) ({b}))", nat(1)),
    );
    assert_eq!(
        option_nat(&env, &v),
        Some(10),
        "union collision must call f (from-a) (from-b); reversed orientation would return 30"
    );

    let v = eval_view(
        &mut env,
        &mut store,
        "t_cat4_intersection",
        "List (Pair Nat Nat)",
        &format!("toList Nat Nat (intersection Nat Nat leqNat ({a}) ({b}))"),
    );
    assert_eq!(list_pair_nat_nat(&env, &v), vec![(1, 10)], "intersection keeps only shared keys with values from the left map");

    let v = eval_view(
        &mut env,
        &mut store,
        "t_cat4_difference",
        "List (Pair Nat Nat)",
        &format!("toList Nat Nat (difference Nat Nat leqNat ({a}) ({b}))"),
    );
    assert_eq!(list_pair_nat_nat(&env, &v), vec![(2, 20)], "difference keeps left-only keys");
}

#[test]
fn cat4_keys_values_are_aligned_tolist_projections() {
    let mut env = mk_env();
    let mut store = make_store(&env);
    let m = format!(
        "insert Nat Nat leqNat ({two}) ({twenty}) \
           (insert Nat Nat leqNat ({one}) ({ten}) \
             (insert Nat Nat leqNat ({three}) ({thirty}) (empty Nat Nat)))",
        one = nat(1),
        two = nat(2),
        three = nat(3),
        ten = nat(10),
        twenty = nat(20),
        thirty = nat(30)
    );
    let ks = eval_view(&mut env, &mut store, "t_cat4_keys", "List Nat", &format!("keys Nat Nat ({m})"));
    let vs = eval_view(&mut env, &mut store, "t_cat4_values", "List Nat", &format!("values Nat Nat ({m})"));
    assert_eq!(list_nat(&env, &ks), vec![1, 2, 3], "keys must follow toList ascending key order");
    assert_eq!(list_nat(&env, &vs), vec![10, 20, 30], "values must stay positionally aligned with keys");
}

#[test]
fn cat4_relations_compose_and_converse_over_adjacency_maps() {
    let mut env = mk_env();
    let mut store = make_store(&env);
    let r = format!(
        "addEdge Nat leqNat ({one}) ({two}) (empty Nat (Tree Nat Unit))",
        one = nat(1),
        two = nat(2)
    );
    let s = format!(
        "addEdge Nat leqNat ({two}) ({three}) (empty Nat (Tree Nat Unit))",
        two = nat(2),
        three = nat(3)
    );
    let v = eval_view(
        &mut env,
        &mut store,
        "t_cat4_compose_member",
        "Bool",
        &format!(
            "setMember Nat leqNat ({three}) (succ Nat leqNat ({one}) (compose Nat leqNat ({r}) ({s})))",
            one = nat(1),
            three = nat(3)
        ),
    );
    assert!(bool_value(&env, &v), "compose must include 1 -> 3 when R has 1 -> 2 and S has 2 -> 3");

    let v = eval_view(
        &mut env,
        &mut store,
        "t_cat4_converse_member",
        "Bool",
        &format!(
            "setMember Nat leqNat ({one}) (succ Nat leqNat ({two}) (converse Nat leqNat ({r})))",
            one = nat(1),
            two = nat(2)
        ),
    );
    assert!(bool_value(&env, &v), "converse must reverse the adjacency edge 1 -> 2 into 2 -> 1");
}

// A hand-built concrete-instance application (`tree_2_1_3` under a trivial
// always-true comparator) was tried here as a second smoke test, but
// `Ordered`'s real Node case (`And (allKeys (\k2. ...) l) (And (allKeys
// (\k2. ...) r) (And (Ordered l) (Ordered r)))`) needs an exactly-nested
// `andIntro` witness matching that inline-lambda predicate spelling
// precisely, not a re-derivation via `leBelow`/`leAbove` or a bare `tt` —
// getting the by-hand nesting exactly right is its own small proof exercise
// and not necessary evidence: `tolistordered_law4_is_a_real_general_proof_
// term` above is the load-bearing check (the fully general, quantified
// theorem already IS the whole-body kernel recheck that used to OOM).
// Left as a natural follow-on if a concrete instance-level regression test
// is wanted later.

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
