//! `Map-build` acceptance tests (`docs/program/wp/Map-build.md`,
//! `spec/50-stdlib/52-map.md`, `conformance/stdlib/map/seed-map.md`).
//!
//! **Partial-scope candidate.** `insert`/`lookup`/`member`/`from_list` and the
//! `52 §5` proof obligations need a key comparator threaded generically over
//! an abstract `Ord k` dictionary — no landed mechanism exists for that yet
//! (confirmed empirically against `elab.rs`'s `instance_search`, escalated to
//! Architect, `evt_1wd56hecqhm06`/`evt_64j01esqw86pf`/`evt_1wsk6dracp10r` in
//! the Map-build thread). This file covers only what
//! `catalog/packages/Data/Collections/Map.ken.md` ships today: the `Tree k v`
//! carrier, `empty`, `to_list`, `fold`,
//! and the `Pair`/`mk_pair`/`pair_fst`/`pair_snd` Σ-pair plumbing
//! (`ken-elaborator/src/prelude.rs`) those two ops route through. Extended
//! once the generic-dictionary gap resolves.

use ken_elaborator::{foreign::trusted_base_delta, ElabEnv, NumericLitVal};
use ken_interp::eval::{eval, EvalStore, EvalVal, ListCharIds};
use ken_kernel::{Decl, GlobalId};

const COLLECTIONS_KEN_MD: &str = include_str!("../../../catalog/packages/Data/Collections/Derived.ken.md");
const TRANSPORT_KEN_MD: &str = include_str!("../../../catalog/packages/Core/Logic/Transport.ken.md");
const MAP_KEN_MD: &str = include_str!("../../../catalog/packages/Data/Collections/Map.ken.md");

fn mk_env() -> ElabEnv {
    let mut env = ElabEnv::new().expect("base env");
    env.elaborate_ken_md_file(TRANSPORT_KEN_MD)
        .expect("catalog/packages/Core/Logic/Transport.ken must elaborate");
    env.elaborate_ken_md_file(COLLECTIONS_KEN_MD)
        .expect("catalog/packages/Data/Collections/Derived.ken.md must elaborate");
    env.elaborate_ken_md_file(MAP_KEN_MD)
        .expect("catalog/packages/Data/Collections/Map.ken.md must elaborate");
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
/// `to_list`'s ascending-order claim is actually exercised (an
/// insertion/construction-order-preserving bug would fail this even though
/// it might pass a pre-sorted tree). `insert` isn't landed yet, so this is a
/// hand-built `Node` tree, not a real `insert` sequence — honestly the
/// non-`insert` half of AC2 (`to_list`/`fold`'s own correctness), not a
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
    for name in ["empty", "to_list", "fold", "Pair", "mk_pair", "pair_fst", "pair_snd"] {
        let id = env.globals[name];
        assert!(
            matches!(env.env.lookup(id), Some(Decl::Transparent { .. })),
            "{name} must be Decl::Transparent (declare_def), not a primitive/postulate"
        );
    }
    // Zero-NEW-delta: none of these mint a fresh trusted_base entry.
    let delta_empty = trusted_base_delta(&env.env, env.globals["empty"]);
    assert!(delta_empty.is_empty(), "empty must add zero trusted_base delta, got {delta_empty:?}");
    let delta_tolist = trusted_base_delta(&env.env, env.globals["to_list"]);
    assert!(delta_tolist.is_empty(), "to_list must add zero trusted_base delta, got {delta_tolist:?}");
    let delta_fold = trusted_base_delta(&env.env, env.globals["fold"]);
    assert!(delta_fold.is_empty(), "fold must add zero trusted_base delta, got {delta_fold:?}");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC2 (partial) — to_list / fold correct end-to-end through the real interpreter
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn tolist_of_empty_is_nil() {
    let mut env = mk_env();
    let mut store = make_store(&env);
    let nil_id = env.globals["Nil"];
    let v = eval_view(&mut env, &mut store, "t_empty_tolist", "List (Pair Nat Nat)", "to_list Nat Nat (empty Nat Nat)");
    assert!(matches!(v, EvalVal::Ctor { id, .. } if id == nil_id), "to_list of empty must be Nil, got {v:?}");
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
            &format!("to_list Nat Nat ({})", tree_2_1_3()),
        );
        let out = list_pair_nat_nat(&env, &v);
        // The flip: a bug emitting construction/insertion order instead of
        // in-order-by-key traversal would yield [(2,2),(1,1),(3,3)] — this
        // asserts the ASCENDING list, not just the element set.
        assert_eq!(out, vec![(1, 1), (2, 2), (3, 3)], "to_list must be ascending by key, got {out:?}");
    });
}

#[test]
fn fold_agrees_with_left_fold_over_tolist() {
    run_with_big_stack(|| {
        let mut env = mk_env();
        let mut store = make_store(&env);
        // Order-sensitive `f`: append the key onto an accumulator list, so a
        // fold visiting a different order than to_list's ascending order
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
        assert_eq!(out, vec![1, 2, 3], "fold must visit ascending key order, matching to_list; got {out:?}");
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// AC1/AC5 (partial) — insert/lookup/member/from_list admitted, non-primitive
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn map_ops_full_api_not_primitive() {
    let env = mk_env();
    for name in ["insert", "lookup", "member", "from_list", "from_list_acc", "set_insert", "set_member", "set_to_list", "Ordered", "all_keys", "lookup_empty_is_none"] {
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
// AC2 — insert/lookup/member/from_list correct end-to-end (Char keys, `leqChar`
// computes — `52 §5.4` — never hand-fed: constructed via real `insert`, read
// via real `lookup`/`to_list`)
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
        "to_list Char Char (insert Char Char leqChar ({k}) ({v2}) (insert Char Char leqChar ({k}) ({v1}) (empty Char Char)))",
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
        "to_list Char Char (insert Char Char leqChar ({a}) ({a}) (insert Char Char leqChar ({c}) ({c}) (insert Char Char leqChar ({b}) ({b}) (empty Char Char))))",
        a = char_lit('a'),
        b = char_lit('b'),
        c = char_lit('c')
    );
    let v = eval_view(&mut env, &mut store, "t_tolist_real_insert", "List (Pair Char Char)", &expr);
    let out = list_pair_char_char(&env, &v);
    assert_eq!(
        out,
        vec![('a' as i64, 'a' as i64), ('b' as i64, 'b' as i64), ('c' as i64, 'c' as i64)],
        "to_list over a real b,c,a-order insert sequence must be ascending by key, got {out:?}"
    );
}

#[test]
fn fromlist_last_writer_and_ordered() {
    let mut env = mk_env();
    let mut store = make_store(&env);
    // [(2,'b'),(1,'a'),(2,'c')] -> to_list must be [(1,'a'),(2,'c')]: ascending
    // AND the LAST list entry ('c') wins on the duplicate key 2.
    let list_expr = format!(
        "Cons (Pair Char Char) (mk_pair Char Char ({two}) ({b})) \
           (Cons (Pair Char Char) (mk_pair Char Char ({one}) ({a})) \
             (Cons (Pair Char Char) (mk_pair Char Char ({two}) ({c})) (Nil (Pair Char Char))))",
        two = char_lit('2'),
        one = char_lit('1'),
        a = char_lit('a'),
        b = char_lit('b'),
        c = char_lit('c')
    );
    let expr = format!("to_list Char Char (from_list Char Char leqChar ({list_expr}))");
    let v = eval_view(&mut env, &mut store, "t_fromlist", "List (Pair Char Char)", &expr);
    let out = list_pair_char_char(&env, &v);
    assert_eq!(
        out,
        vec![('1' as i64, 'a' as i64), ('2' as i64, 'c' as i64)],
        "from_list must be ascending AND last-writer-wins ('c' beats 'b' on key '2'), got {out:?}"
    );
}

#[test]
fn set_is_map_unit() {
    let mut env = mk_env();
    let mut store = make_store(&env);
    let true_id = env.globals["True"];
    let false_id = env.globals["False"];
    let expr = format!(
        "set_member Char leqChar ({a}) (set_insert Char leqChar ({a}) (set_insert Char leqChar ({b}) (Leaf Char Unit)))",
        a = char_lit('a'),
        b = char_lit('b')
    );
    let v = eval_view(&mut env, &mut store, "t_set_member_hit", "Bool", &expr);
    assert!(matches!(v, EvalVal::Ctor { id, .. } if id == true_id), "member of an inserted element must be True, got {v:?}");

    let expr_absent = format!(
        "set_member Char leqChar ({z}) (set_insert Char leqChar ({a}) (Leaf Char Unit))",
        z = char_lit('z'),
        a = char_lit('a')
    );
    let v = eval_view(&mut env, &mut store, "t_set_member_miss", "Bool", &expr_absent);
    assert!(matches!(v, EvalVal::Ctor { id, .. } if id == false_id), "member of an absent element must be False, got {v:?}");

    let tolist_expr = format!(
        "set_to_list Char (set_insert Char leqChar ({a}) (set_insert Char leqChar ({c}) (set_insert Char leqChar ({b}) (Leaf Char Unit))))",
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
                    other => panic!("set_to_list head must be Char-as-Int, got {other:?}"),
                }
                cur = args[2].clone();
            }
            other => panic!("not a well-formed List chain: {other:?}"),
        }
    }
    assert_eq!(out, vec!['a' as i64, 'b' as i64, 'c' as i64], "set_to_list must be ascending, got {out:?}");
}

#[test]
fn letter_frequency_shape() {
    let mut env = mk_env();
    let mut store = make_store(&env);
    // "banana": b,a,n,a,n,a -> {'a':3,'b':1,'n':2}, ascending by key.
    env.elaborate_decl(
        "fn bumpCount (leq : Char -> Char -> Bool) (key : Char) (m : Tree Char Nat) : Tree Char Nat = \
         match lookup Char Nat leq key m { \
           None |-> insert Char Nat leq key (Suc Zero) m ; \
           Some n |-> insert Char Nat leq key (Suc n) m \
         }",
    )
    .expect("bumpCount should elaborate");
    env.elaborate_decl(
        "fn countChars (leq : Char -> Char -> Bool) (cs : List Char) (m : Tree Char Nat) : Tree Char Nat = \
         match cs { \
           Nil |-> m ; \
           Cons c cs2 |-> countChars leq cs2 (bumpCount leq c m) \
         }",
    )
    .expect("countChars should elaborate");
    let banana = format!(
        "Cons Char ({b}) (Cons Char ({a}) (Cons Char ({n}) (Cons Char ({a}) (Cons Char ({n}) (Cons Char ({a}) (Nil Char))))))",
        b = char_lit('b'),
        a = char_lit('a'),
        n = char_lit('n')
    );
    let expr = format!("to_list Char Nat (countChars leqChar ({banana}) (empty Char Nat))");
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
// Ordered/all_keys admitted as declare_def (never a postulate)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn lookup_empty_law_is_a_real_reducing_proof() {
    let mut env = mk_env();
    // The stated law itself (`lookup_empty_is_none`) must be admitted as a
    // real Decl::Transparent proof term (never Decl::Opaque/Axiom).
    let id = env.globals["lookup_empty_is_none"];
    assert!(
        matches!(env.env.lookup(id), Some(Decl::Transparent { .. })),
        "lookup_empty_is_none must be a real proof term, not a postulate"
    );
    // `Ordered` on an empty map is provable by `Proved` — the invariant reduces
    // to a trivially-true Prop (Equal Bool True True) at Leaf, closable the
    // same way `lookup_empty_is_none` closes (K5 same-nullary-ctor collapse).
    // This is a kernel CHECK (is the type inhabited), not an `eval` — the
    // Prop itself is a type, not a runtime data value.
    env.elaborate_decl(
        "lemma orderedEmptyProof (k : Type) (v : Type) (leq : k -> k -> Bool) : \
         Ordered k v leq (empty k v) = Proved",
    )
    .expect("Ordered on an empty map must be provable by Proved");
}

// ─────────────────────────────────────────────────────────────────────────────
// Law 4 (`54 §3`, "to_list ordered") — `to_list_ordered`
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn tolistordered_law4_is_a_real_general_proof_term() {
    let env = mk_env();
    // `to_list_ordered : (k v : Type) -> (leq : k -> k -> Bool) -> (m : Tree k v)
    //   -> Ordered k v leq m -> is_sorted (Pair k v) (pair_leq k v leq) (to_list k v m)`
    // must be admitted as a real Decl::Transparent proof term (never a
    // postulate/axiom) — this IS the whole-body `declare_def` kernel recheck
    // that used to OOM (~12 GB) before `wp/obs-eq-termination` (`9cf468a`)
    // fixed the underlying conv/obs termination bug; `mk_env()` above
    // elaborating `map.ken` at all is itself the completion proof, this just
    // pins the trust-level assertion on the specific declaration.
    for name in [
        "to_list_ordered",
        "is_sorted_append",
        "cons_sorted_head",
        "all_keys_to_all_in_list",
        "all_in_list_append_intro",
    ] {
        let id = env.globals[name];
        assert!(
            matches!(env.env.lookup(id), Some(Decl::Transparent { .. })),
            "{name} must be a real proof term, not a postulate"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Law 1 (`54 §5.1`, Map capstone unit 2) — `preserves_ordered`
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn preservesordered_law1_is_a_real_general_proof_term() {
    let env = mk_env();
    // `preserves_ordered : ... -> Ordered m -> Ordered (insert key val m)`
    // must be a real Decl::Transparent proof term (never a postulate) —
    // the whole-body `declare_def` kernel recheck for the top-level
    // induction plus every supporting transport bridge / comparison-
    // independent lemma / totality-derived reflection it composes.
    for name in [
        "preserves_ordered",
        "insert_case_transport_dispatch",
        "dispatch_on_q1",
        "dispatch_on_q2",
        "insert_case_transport_overwrite",
        "insert_case_transport_into_l",
        "insert_case_transport_into_r",
        "insert_preserves_all_keys",
        "all_keys_trans_below",
        "all_keys_trans_above",
        "derive_from_false",
    ] {
        let id = env.globals[name];
        assert!(
            matches!(env.env.lookup(id), Some(Decl::Transparent { .. })),
            "{name} must be a real proof term, not a postulate"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Law 2 (`54 §5.2`, Map capstone unit 2) — `lookup_found_after_insert`
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn lookupfoundafterinsert_law2_is_a_real_general_proof_term() {
    let env = mk_env();
    // `lookup_found_after_insert : ... -> lookup key (insert key val m) =
    // Some val` must be a real Decl::Transparent proof term (never a
    // postulate) — reuses Law 1's goal-generic transport bridges directly
    // (asserted there), plus its own lookup-side step mirrors/bridges.
    for name in [
        "lookup_found_after_insert",
        "lookup_found_dispatch",
        "lookup_found_dispatch_q1",
        "lookup_found_dispatch_q2",
        "lookup_overwrite_result",
        "lookup_into_l_bridge",
        "lookup_into_r_bridge",
    ] {
        let id = env.globals[name];
        assert!(
            matches!(env.env.lookup(id), Some(Decl::Transparent { .. })),
            "{name} must be a real proof term, not a postulate"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Law 3 (`54 §5.2`, Map capstone unit 2) — `lookup_locality`
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn lookuplocality_law3_is_a_real_general_proof_term() {
    let env = mk_env();
    // `lookup_locality : distinct key key' -> lookup key' (insert key val m)
    // = lookup key' m` must be a real Decl::Transparent proof term — reuses
    // Law 1's goal-generic transport bridges directly (asserted there) and
    // Law 2's lookup_into_l_bridge/IntoRBridge, plus its own agreement lemmas.
    for name in [
        "lookup_locality",
        "lookup_locality_node_dispatch",
        "lookup_locality_q2_dispatch",
        "lookup_leaf_locality_witness",
        "lookup_overwrite_locality_witness",
        "lookup_into_l_locality_witness",
        "lookup_into_r_locality_witness",
        "bool_value_eq_from_biimpl",
        "lookup_overwrite_agrees_outer",
        "lookup_overwrite_agrees_inner",
    ] {
        let id = env.globals[name];
        assert!(
            matches!(env.env.lookup(id), Some(Decl::Transparent { .. })),
            "{name} must be a real proof term, not a postulate"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Law 5 (`54 §5.3`, Map capstone unit 2 — the final law) — `lookup_assoc_agree`
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn lookupassocagree_law5_is_a_real_general_proof_term() {
    let env = mk_env();
    // `lookup_assoc_agree : Ordered m -> Distinct leq m -> lookup key m =
    // assoc key (to_list m)` must be a real Decl::Transparent proof term —
    // the restated statement (Distinct precondition added after the
    // original false statement was caught and escalated), reusing Law 1's
    // goal-generic transport bridges and Law 2's lookup-side bridges.
    for name in [
        "lookup_assoc_agree",
        "law5_node_dispatch",
        "law5_node_q2_dispatch",
        "law5_distinct_l",
        "law5_distinct_r",
        "order_equiv",
        "NoDup",
        "Distinct",
        "distinct_empty",
        "assoc_skip_prefix",
        "assoc_prefix_wins",
        "assoc_none_implies_no_match_inner",
        "assoc_none_implies_no_match_dispatch",
        "assoc_none_implies_no_match",
        "assoc_no_match_is_none",
        "no_dup_append_head_excl",
        "no_dup_append_left",
        "no_dup_append_right",
        "not_match_transfer_via_equiv",
        "lookup_stop_bridge",
        "lookup_order_equiv_outer_agree",
        "lookup_order_equiv_inner_agree",
        "lookup_order_equiv_both_false",
        "lookup_order_equiv_both_inner_false",
        "lookup_order_equiv_both_stop",
        "lookup_order_equiv_inner_dispatch",
        "lookup_order_equiv_node_dispatch",
        "lookup_order_equiv_agree",
        "member_order_equiv_agree",
        "set_member_order_equiv_agree",
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
        "cat4_bool_or::comm",
        "cat4_bool_or::assoc",
        "cat4_bool_or::idempotent",
        "cat4_bool_or::left_identity",
        "cat4_bool_or::right_identity",
        "bool_and::comm",
        "bool_and::assoc",
        "bool_and::idempotent",
        "bool_and::left_identity",
        "bool_and::right_identity",
        "leq_nat",
        "leq_nat::refl",
        "leq_nat::trans",
        "leq_nat::antisym",
        "total_leq_nat",
        "order_equiv_key",
        "bool_and::true_intro",
        "order_equiv_key_true_from_order_equiv",
        "order_equiv_from_order_equiv_key_true_inner",
        "order_equiv_from_order_equiv_key_true_dispatch",
        "order_equiv_from_order_equiv_key_true",
        "order_equiv_key_false_to_not",
        "not_order_equiv_from_left_false",
        "not_order_equiv_from_right_false",
        "drop_key",
        "delete_from_list_acc",
        "delete_from_list_acc_step",
        "delete_from_list_acc_final_bridge",
        "delete_from_list_acc_step_true_eq",
        "delete_from_list_acc_step_false_eq",
        "delete_from_list_acc_step_true_reduces",
        "delete_from_list_acc_step_false_reduces",
        "delete_from_list_acc_true_bridge",
        "delete_from_list_acc_false_bridge",
        "delete_from_list",
        "delete",
        "delete_from_list_acc_lookup_none_dispatch",
        "delete_from_list_acc_lookup_none",
        "delete_lookup_none_law",
        "not_order_equiv_from_deleted_match",
        "delete_from_list_acc_lookup_locality_dispatch",
        "delete_from_list_acc_lookup_locality",
        "delete_from_list_acc_lookup_other_assoc_deleted_hit_absurd",
        "delete_from_list_acc_lookup_other_assoc_hit_survivor",
        "delete_from_list_acc_lookup_other_assoc_hit",
        "delete_from_list_acc_lookup_other_assoc_miss",
        "delete_from_list_acc_lookup_other_assoc_inner",
        "delete_from_list_acc_lookup_other_assoc_dispatch",
        "delete_from_list_acc_lookup_other_assoc",
        "delete_lookup_other_key_law",
        "from_list_acc_preserves_ordered",
        "from_list_preserves_ordered",
        "delete_from_list_acc_preserves_ordered_dispatch",
        "delete_from_list_acc_preserves_ordered",
        "delete_from_list_preserves_ordered",
        "delete_preserves_ordered",
        "insert_with",
        "insert_with_fold_step",
        "insert_with_fold_step_reduces",
        "union_from_list_acc",
        "union_from_list_acc_cons_bridge",
        "union",
        "union_lookup_table",
        "option_is_some",
        "unit_combine",
        "union_lookup_table_member",
        "intersection_lookup_table",
        "difference_lookup_table",
        "difference_lookup_table_false_none_none",
        "difference_lookup_table_false_none_some",
        "difference_lookup_table_false_none",
        "difference_lookup_expected",
        "difference_lookup_expected_true",
        "difference_lookup_expected_false",
        "difference_lookup_expected_member_option",
        "difference_lookup_expected_member_table",
        "difference_lookup_expected_member",
        "insert_with_lookup_result",
        "insert_with_lookup_result_for",
        "insert_with_lookup_overwrite_witness",
        "insert_with_lookup_into_l_witness",
        "insert_with_lookup_into_r_witness",
        "insert_with_lookup_dispatch_q2",
        "insert_with_lookup_dispatch_q1",
        "insert_with_lookup_characterization",
        "lookup_replace_l_inner_dispatch",
        "lookup_replace_l_dispatch",
        "lookup_replace_l_witness",
        "lookup_replace_r_inner_dispatch",
        "lookup_replace_r_dispatch",
        "lookup_replace_r_witness",
        "insert_lookup_hit",
        "insert_with_lookup_locality_q2_dispatch",
        "insert_with_lookup_locality_node_dispatch",
        "insert_with_lookup_locality",
        "insert_with_fold_step_lookup_locality",
        "insert_with_fold_step_lookup_hit",
        "union_from_list_acc_lookup_assoc_hit",
        "union_from_list_acc_lookup_assoc_miss",
        "union_from_list_acc_lookup_assoc_inner",
        "union_from_list_acc_lookup_assoc_dispatch",
        "union_from_list_acc_lookup_assoc",
        "insert_with_fold_step_preserves_ordered",
        "union_from_list_acc_preserves_ordered",
        "union_lookup_characterization",
        "union_lookup_both_none_law",
        "union_lookup_left_only_law",
        "union_lookup_right_only_law",
        "union_lookup_both_some_law",
        "member_from_lookup_none",
        "member_from_lookup_some",
        "lookup_none_from_member_false_hit",
        "lookup_none_from_member_false",
        "lookup_unit_some_from_member_true_leaf",
        "lookup_unit_some_from_member_true_hit",
        "lookup_unit_some_from_member_true",
        "not_order_equiv_from_member_true_false",
        "not_order_equiv_from_member_false_true",
        "intersection_from_list_acc_lookup_none_dispatch",
        "intersection_from_list_acc_lookup_none",
        "intersection_from_list_acc_lookup_locality_dispatch",
        "intersection_from_list_acc_lookup_locality",
        "intersection_from_list_acc_lookup_some_hit",
        "intersection_from_list_acc_lookup_some_miss_dispatch",
        "intersection_from_list_acc_lookup_some_inner",
        "intersection_from_list_acc_lookup_some_dispatch",
        "intersection_from_list_acc_lookup_some",
        "intersection_lookup_left_none_law",
        "difference_from_list_acc_lookup_locality_dispatch",
        "difference_from_list_acc_lookup_locality",
        "difference_from_list_acc_lookup_none_dispatch",
        "difference_from_list_acc_lookup_none",
        "difference_from_list_acc_lookup_keep_hit",
        "difference_from_list_acc_lookup_keep_miss_dispatch",
        "difference_from_list_acc_lookup_keep_inner",
        "difference_from_list_acc_lookup_keep_dispatch",
        "difference_from_list_acc_lookup_keep",
        "difference_lookup_characterization_reject",
        "difference_lookup_characterization_keep",
        "difference_lookup_characterization_dispatch",
        "all_keys_map_not_match_below",
        "all_keys_map_not_match_above",
        "intersection_from_list_acc",
        "intersection_from_list_acc_step",
        "intersection_from_list_acc_final_bridge",
        "intersection_from_list_acc_step_true_eq",
        "intersection_from_list_acc_step_false_eq",
        "intersection_from_list_acc_step_true_reduces",
        "intersection_from_list_acc_step_false_reduces",
        "intersection_from_list_acc_true_bridge",
        "intersection_from_list_acc_false_bridge",
        "intersection",
        "intersection_lookup_characterization",
        "intersection_lookup_some_law",
        "difference_from_list_acc",
        "difference_from_list_acc_step",
        "difference_from_list_acc_final_bridge",
        "difference_from_list_acc_step_true_eq",
        "difference_from_list_acc_step_false_eq",
        "difference_from_list_acc_step_true_reduces",
        "difference_from_list_acc_step_false_reduces",
        "difference_from_list_acc_true_bridge",
        "difference_from_list_acc_false_bridge",
        "difference",
        "difference_lookup_characterization",
        "insert_fold_step",
        "fold_insert_preserves_ordered",
        "insert_with_preserves_ordered",
        "fold_insert_with_preserves_ordered",
        "union_preserves_ordered",
        "intersection_from_list_acc_preserves_ordered_dispatch",
        "intersection_from_list_acc_preserves_ordered",
        "intersection_preserves_ordered",
        "difference_from_list_acc_preserves_ordered_dispatch",
        "difference_from_list_acc_preserves_ordered",
        "difference_preserves_ordered",
        "set_union",
        "set_intersection",
        "set_difference",
        "set_union_member_law",
        "set_intersection_member_left_false_rhs",
        "set_intersection_member_right_false_rhs",
        "set_intersection_member_both_true_rhs",
        "set_intersection_member_left_false_case",
        "set_intersection_member_right_false_case",
        "set_intersection_member_both_true_case",
        "set_intersection_member_right_dispatch",
        "set_intersection_member_dispatch",
        "set_intersection_member_law",
        "set_difference_member_law",
        "set_member_empty_false",
        "set_union_comm_law",
        "set_union_assoc_law",
        "set_union_idempotent_law",
        "set_union_identity_law",
        "set_intersection_comm_law",
        "set_intersection_assoc_law",
        "set_intersection_idempotent_law",
        "set_intersection_identity_law",
        "pair_vals",
        "pair_keys_preserves_sorted_cons",
        "pair_keys_preserves_sorted",
        "keys",
        "values",
        "keys_project_to_list",
        "values_project_to_list",
        "keys_values_projection_coherence",
        "keys_ascending",
        "succ",
        "rel_member",
        "add_edge",
        "compose_succ_step",
        "compose_succ",
        "compose",
        "converse_targets",
        "converse",
        "is_reflexive",
        "is_symmetric",
        "is_transitive",
        "is_equivalence",
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
        "Cons (Pair Nat Nat) (mk_pair Nat Nat ({one}) ({ten})) \
           (Cons (Pair Nat Nat) (mk_pair Nat Nat ({two}) ({twenty})) \
             (Cons (Pair Nat Nat) (mk_pair Nat Nat ({one}) ({thirty})) (Nil (Pair Nat Nat))))",
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
        &format!("drop_key Nat Nat leq_nat ({}) ({list_expr})", nat(1)),
    );
    assert_eq!(
        list_pair_nat_nat(&env, &v),
        vec![(2, 20)],
        "drop_key must filter every order-equivalent key, not just the first match"
    );

    let m = format!(
        "insert Nat Nat leq_nat ({one}) ({ten}) \
           (insert Nat Nat leq_nat ({two}) ({twenty}) (empty Nat Nat))",
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
        &format!("lookup Nat Nat leq_nat ({}) (delete Nat Nat leq_nat ({}) ({m}))", nat(1), nat(1)),
    );
    assert_eq!(option_nat(&env, &v), None, "deleted key must look up as None");
}

#[test]
fn cat4_union_intersection_difference_execute_over_nat() {
    let mut env = mk_env();
    let mut store = make_store(&env);
    let a = format!(
        "insert Nat Nat leq_nat ({one}) ({ten}) \
           (insert Nat Nat leq_nat ({two}) ({twenty}) (empty Nat Nat))",
        one = nat(1),
        two = nat(2),
        ten = nat(10),
        twenty = nat(20)
    );
    let b = format!(
        "insert Nat Nat leq_nat ({one}) ({thirty}) \
           (insert Nat Nat leq_nat ({three}) ({forty}) (empty Nat Nat))",
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
        &format!("lookup Nat Nat leq_nat ({}) (union Nat Nat leq_nat (λx.λy. x) ({a}) ({b}))", nat(1)),
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
        &format!("to_list Nat Nat (intersection Nat Nat leq_nat ({a}) ({b}))"),
    );
    assert_eq!(list_pair_nat_nat(&env, &v), vec![(1, 10)], "intersection keeps only shared keys with values from the left map");

    let v = eval_view(
        &mut env,
        &mut store,
        "t_cat4_difference",
        "List (Pair Nat Nat)",
        &format!("to_list Nat Nat (difference Nat Nat leq_nat ({a}) ({b}))"),
    );
    assert_eq!(list_pair_nat_nat(&env, &v), vec![(2, 20)], "difference keeps left-only keys");
}

#[test]
fn cat4_keys_values_are_aligned_tolist_projections() {
    let mut env = mk_env();
    let mut store = make_store(&env);
    let m = format!(
        "insert Nat Nat leq_nat ({two}) ({twenty}) \
           (insert Nat Nat leq_nat ({one}) ({ten}) \
             (insert Nat Nat leq_nat ({three}) ({thirty}) (empty Nat Nat)))",
        one = nat(1),
        two = nat(2),
        three = nat(3),
        ten = nat(10),
        twenty = nat(20),
        thirty = nat(30)
    );
    let ks = eval_view(&mut env, &mut store, "t_cat4_keys", "List Nat", &format!("keys Nat Nat ({m})"));
    let vs = eval_view(&mut env, &mut store, "t_cat4_values", "List Nat", &format!("values Nat Nat ({m})"));
    assert_eq!(list_nat(&env, &ks), vec![1, 2, 3], "keys must follow to_list ascending key order");
    assert_eq!(list_nat(&env, &vs), vec![10, 20, 30], "values must stay positionally aligned with keys");
}

#[test]
fn cat4_relations_compose_and_converse_over_adjacency_maps() {
    let mut env = mk_env();
    let mut store = make_store(&env);
    let r = format!(
        "add_edge Nat leq_nat ({one}) ({two}) (empty Nat (Tree Nat Unit))",
        one = nat(1),
        two = nat(2)
    );
    let s = format!(
        "add_edge Nat leq_nat ({two}) ({three}) (empty Nat (Tree Nat Unit))",
        two = nat(2),
        three = nat(3)
    );
    let v = eval_view(
        &mut env,
        &mut store,
        "t_cat4_compose_member",
        "Bool",
        &format!(
            "set_member Nat leq_nat ({three}) (succ Nat leq_nat ({one}) (compose Nat leq_nat ({r}) ({s})))",
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
            "set_member Nat leq_nat ({one}) (succ Nat leq_nat ({two}) (converse Nat leq_nat ({r})))",
            one = nat(1),
            two = nat(2)
        ),
    );
    assert!(bool_value(&env, &v), "converse must reverse the adjacency edge 1 -> 2 into 2 -> 1");
}

// A hand-built concrete-instance application (`tree_2_1_3` under a trivial
// always-true comparator) was tried here as a second smoke test, but
// `Ordered`'s real Node case (`And (all_keys (\k2. ...) l) (And (all_keys
// (\k2. ...) r) (And (Ordered l) (Ordered r)))`) needs an exactly-nested
// `and_intro` witness matching that inline-lambda predicate spelling
// precisely, not a re-derivation via `le_below`/`le_above` or a bare `Proved` —
// getting the by-hand nesting exactly right is its own small proof exercise
// and not necessary evidence: `tolistordered_law4_is_a_real_general_proof_
// term` above is the load-bearing check (the fully general, quantified
// theorem already IS the whole-body kernel recheck that used to OOM).
// Left as a natural follow-on if a concrete instance-level regression test
// is wanted later.

// ─────────────────────────────────────────────────────────────────────────────
// Pair (Σ-pair, `52 §4`) plumbing sanity — `mk_pair`/`pair_fst`/`pair_snd`
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn pair_roundtrip() {
    let mut env = mk_env();
    let mut store = make_store(&env);
    let v = eval_view(&mut env, &mut store, "t_fst", "Nat", &format!("pair_fst Nat Nat (mk_pair Nat Nat ({}) ({}))", nat(3), nat(4)));
    assert_eq!(nat_count(&env, &v), 3, "pair_fst (mk_pair 3 4) must be 3, got {v:?}");
    let v = eval_view(&mut env, &mut store, "t_snd", "Nat", &format!("pair_snd Nat Nat (mk_pair Nat Nat ({}) ({}))", nat(3), nat(4)));
    assert_eq!(nat_count(&env, &v), 4, "pair_snd (mk_pair 3 4) must be 4, got {v:?}");
}

fn run_with_big_stack<F: FnOnce() + Send + 'static>(f: F) {
    std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(f)
        .expect("spawn big-stack test thread")
        .join()
        .expect("test thread panicked");
}
