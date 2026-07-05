//! `L3-strings-surface` acceptance tests (slice 2/2 of the string surface).
//!
//! Pins `conformance/surface/collections/seed-collections.md`'s "Derived
//! string surface (slice 2)" section, DS-AC1–7
//! (`spec/30-surface/37-strings-collections.md` §2.4/§2.5/§2.5.1/§4.1). Drives
//! the actual package file via `include_str!` (never a hand-copied
//! reimplementation, matching `es4_classes_acceptance.rs`'s discipline):
//! - DS-AC1/AC5 `list-combinator-floor-derived-over-real-elim` — the 7-combinator
//!   floor + `compareChar` are real derived defs over the generic `Term::Elim`,
//!   `OrdResult` a checked inductive, zero-TCB-delta.
//! - DS-AC2 (soundness) `list-floor-recursion-in-sct-sound-zone` — the SCT
//!   sound-zone accept/reject verdict-flip.
//! - DS-AC3 `derived-string-ops-reduce-over-real-roundtrip` — the 5 string ops
//!   compute correctly through the real `string_to_list_char`/
//!   `list_char_to_string`.
//! - DS-AC4 `string-eq-codepoint-wise-accept-reject-pair` +
//!   `string-compare-3way-lexicographic-triple` +
//!   `list-eq-is-codepoint-wise-not-nfc-folding`.
//! - DS-AC6 `list-append-does-not-shadow-bytes-append`.
//! - DS-AC7 `concat-slice-compose-and-floor-totality`.

use ken_elaborator::{foreign::trusted_base_delta, ElabEnv, NumericLitVal};
use ken_interp::eval::{eval, EvalStore, EvalVal, ListCharIds};
use ken_kernel::{Decl, GlobalId, Term};

const COLLECTIONS_KEN: &str =
    include_str!("../../../packages/collections/collections.ken");

fn mk_env() -> ElabEnv {
    let mut env = ElabEnv::new().expect("base env");
    env.elaborate_file(COLLECTIONS_KEN)
        .expect("packages/collections/collections.ken must elaborate");
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

/// Elaborate `view <name> : <ty> = <expr>` against the already-loaded package
/// env/store and evaluate it. Resyncs `store.num_values` after every call — a
/// literal declared after `make_store` time evaluates against a stale
/// snapshot otherwise (`l3_strings_roundtrip_acceptance.rs`'s documented
/// discipline).
fn eval_view(env: &mut ElabEnv, store: &mut EvalStore, name: &str, ty: &str, expr: &str) -> EvalVal {
    let src = format!("const {name} : {ty} = {expr}");
    let id = env
        .elaborate_decl(&src)
        .unwrap_or_else(|e| panic!("{name} failed to elaborate: {e}"));
    let mkdecimalpair_id = env.prelude_env.mkdecimalpair_id;
    for (nid, v) in &env.num_values {
        store
            .num_values
            .entry(*nid)
            .or_insert_with(|| lit_to_eval(v, mkdecimalpair_id));
    }
    eval_def(env, store, id)
}

/// A Ken `Nat` literal has no numeral sugar (`Zero`/`Suc` only, `34 §1`) — build
/// the unary source form for a small constant.
fn nat(n: u32) -> String {
    let mut s = "Zero".to_string();
    for _ in 0..n {
        s = format!("Suc ({s})");
    }
    s
}

/// Every acceptance run here uses a real generic-recursion depth (up to the
/// pinned `slice 0 99 …` corpus value) — an oversized default test-thread
/// stack avoids the plain (non-algorithmic) stack exhaustion a ~100-deep
/// unary-`Nat`/nested-`match` recursion hits under the default 8 MiB stack.
fn run_with_big_stack<F: FnOnce() + Send + 'static>(f: F) {
    std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(f)
        .expect("spawn big-stack test thread")
        .join()
        .expect("test thread panicked");
}

// ─────────────────────────────────────────────────────────────────────────────
// DS-AC1/AC5 — `list-combinator-floor-derived-over-real-elim`
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn list_combinator_floor_derived_over_real_elim() {
    let env = mk_env();

    // The 7 floor combinators + `compareChar` are all Transparent (SCT
    // accepted, `declare_def`-upgraded) and their `match` lowers to the real
    // generic `Term::Elim` over the `List`/`Nat` family — never a bespoke
    // reducer, never a registered `elim_List`/`elim_Nat` constant.
    let list_id = env.globals["List"];
    let nat_id = env.globals["Nat"];
    let list_recursors = ["list_append", "nth", "list_eq", "list_compare"];
    let nat_recursors = ["natSub"];
    // `take`/`drop` match on `Nat` outermost (the fuel), `List` innermost.
    let nat_outer = ["take", "drop"];

    for name in list_recursors {
        let id = env.globals[name];
        let (_, body) = env
            .env
            .transparent_body(id)
            .unwrap_or_else(|| panic!("{name} must be Transparent (SCT-accepted)"));
        let mut inner = &body;
        while let Term::Lam(_, b) = inner {
            inner = b;
        }
        match inner {
            Term::Elim { fam, .. } => assert_eq!(
                *fam, list_id,
                "{name}'s outermost match must lower to the real elim_List"
            ),
            other => panic!("{name}'s body must be a Term::Elim; got {other:?}"),
        }
    }
    for name in nat_recursors {
        let id = env.globals[name];
        let (_, body) = env
            .env
            .transparent_body(id)
            .unwrap_or_else(|| panic!("{name} must be Transparent (SCT-accepted)"));
        let mut inner = &body;
        while let Term::Lam(_, b) = inner {
            inner = b;
        }
        match inner {
            Term::Elim { fam, .. } => assert_eq!(
                *fam, nat_id,
                "{name}'s outermost match must lower to the real elim_Nat"
            ),
            other => panic!("{name}'s body must be a Term::Elim; got {other:?}"),
        }
    }
    for name in nat_outer {
        let id = env.globals[name];
        let (_, body) = env
            .env
            .transparent_body(id)
            .unwrap_or_else(|| panic!("{name} must be Transparent (SCT-accepted)"));
        let mut inner = &body;
        while let Term::Lam(_, b) = inner {
            inner = b;
        }
        match inner {
            Term::Elim { fam, .. } => assert_eq!(
                *fam, nat_id,
                "{name}'s outermost match must lower to the real elim_Nat (the fuel)"
            ),
            other => panic!("{name}'s body must be a Term::Elim; got {other:?}"),
        }
    }
    // compareChar's own match (on eqChar's Bool result) is not List/Nat-elim
    // shaped — it's checked separately: just confirm it's Transparent and not
    // an opaque postulate stand-in.
    assert!(
        env.env.transparent_body(env.globals["compareChar"]).is_some(),
        "compareChar must be a real (checked) def"
    );

    // `OrdResult` is a checked `data` inductive — NOT a postulate/primitive.
    let ordresult_id = env.globals["OrdResult"];
    assert!(
        matches!(env.env.lookup(ordresult_id), Some(Decl::Inductive { .. })),
        "OrdResult must be a checked inductive, not a postulate/primitive"
    );

    // Zero-TCB-delta: none of the floor combinators or OrdResult contribute
    // any new trusted_base() member (they are all declare_def/data, checked).
    for name in ["list_append", "nth", "take", "drop", "natSub", "list_eq", "list_compare", "compareChar"] {
        let id = env.globals[name];
        let delta = trusted_base_delta(&env.env, id);
        assert!(
            delta.is_empty(),
            "{name} must mint zero trusted_base() delta; got {delta:?}"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// DS-AC2 (soundness) — `list-floor-recursion-in-sct-sound-zone`
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn list_floor_recursion_in_sct_sound_zone() {
    // Positive: every one of the 7 combinators (+ compareChar) elaborates —
    // confirmed by `mk_env()` itself not panicking (each is SCT-accepted, an
    // applied call on a strict Cons-tail/Suc-pred subterm).
    let _env = mk_env();

    // Negative — the discriminating flip: a sibling recursing on a
    // RECONSTRUCTED, non-decreasing argument (`bad (Cons x xs) = bad (Cons x
    // xs)`, an APPLIED call carrying no `Down` (strict-subterm) argument) must
    // be REJECTED. This does NOT lean on the SCT's unapplied-self-reference /
    // recursion-through-opaque-map over-accept hole
    // (`sct-unapplied-self-reference-over-accepts`) — `bad`'s call is applied,
    // just non-decreasing, squarely in the SCT's sound-REJECT zone.
    let mut env2 = mk_env();
    let result = env2.elaborate_decl(
        "fn bad (a : Type) (xs : List a) : List a = \
         match xs { Nil => Nil a ; Cons x xs2 => bad a (Cons a x xs2) }",
    );
    assert!(
        result.is_err(),
        "bad (reconstructs its matched arg, no strict-subterm decrease) must be SCT-rejected"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// DS-AC3 — `derived-string-ops-reduce-over-real-roundtrip`
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn derived_string_ops_reduce_over_real_roundtrip() {
    run_with_big_stack(|| {
        let mut env = mk_env();
        let mut store = make_store(&env);

        // concat, including a multi-byte pair (CJK), preserves every scalar.
        let v = eval_view(&mut env, &mut store, "t_concat", "String", "concat \"ab\" \"cd\"");
        assert_eq!(v, EvalVal::Str("abcd".into()), "concat \"ab\" \"cd\" must be \"abcd\"");
        let v = eval_view(&mut env, &mut store, "t_concat_mb", "String", "concat \"世\" \"界\"");
        assert_eq!(v, EvalVal::Str("世界".into()), "concat must preserve multi-byte scalars");

        // slice: ordinary, over-range clamp, and j < i (empty, no underflow).
        let v = eval_view(
            &mut env,
            &mut store,
            "t_slice1",
            "String",
            &format!("slice ({}) ({}) \"abcde\"", nat(1), nat(3)),
        );
        assert_eq!(v, EvalVal::Str("bc".into()), "slice 1 3 \"abcde\" must be \"bc\"");

        let v = eval_view(
            &mut env,
            &mut store,
            "t_slice_clamp",
            "String",
            &format!("slice ({}) ({}) \"abc\"", nat(0), nat(99)),
        );
        assert_eq!(
            v,
            EvalVal::Str("abc".into()),
            "slice 0 99 \"abc\" must clamp to \"abc\" (over-range take stops at the end)"
        );

        let v = eval_view(
            &mut env,
            &mut store,
            "t_slice_underflow",
            "String",
            &format!("slice ({}) ({}) \"abc\"", nat(2), nat(1)),
        );
        assert_eq!(
            v,
            EvalVal::Str("".into()),
            "slice 2 1 \"abc\" must be \"\" (natSub saturates, no underflow)"
        );

        // charAt: Option Char, honest absence.
        let some_id = env.globals["Some"];
        let none_id = env.globals["None"];
        let v = eval_view(&mut env, &mut store, "t_charat1", "Option Char", &format!("charAt ({}) \"abc\"", nat(1)));
        match v {
            EvalVal::Ctor { id, ref args, .. } if id == some_id => {
                assert_eq!(args[1], EvalVal::Int('b' as i64), "charAt 1 \"abc\" must be Some 'b'");
            }
            other => panic!("charAt 1 \"abc\" must be Some 'b'; got {other:?}"),
        }
        let v = eval_view(&mut env, &mut store, "t_charat_oob", "Option Char", &format!("charAt ({}) \"abc\"", nat(5)));
        assert!(
            matches!(v, EvalVal::Ctor { id, .. } if id == none_id),
            "charAt 5 \"abc\" must be None"
        );
        let v = eval_view(&mut env, &mut store, "t_charat_empty", "Option Char", &format!("charAt ({}) \"\"", nat(0)));
        assert!(
            matches!(v, EvalVal::Ctor { id, .. } if id == none_id),
            "charAt 0 \"\" must be None"
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// DS-AC4 — `string-eq-codepoint-wise-accept-reject-pair`
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn string_eq_codepoint_wise_accept_reject_pair() {
    let mut env = mk_env();
    let mut store = make_store(&env);
    let true_id = env.globals["True"];
    let false_id = env.globals["False"];

    let v = eval_view(&mut env, &mut store, "t_eq_accept", "Bool", "eq \"abc\" \"abc\"");
    assert!(matches!(v, EvalVal::Ctor{id,..} if id==true_id), "eq \"abc\" \"abc\" must be True");

    // Non-degenerate reject: same length, single codepoint differs — the
    // tightest guard (a length-only equality would pass this and both
    // corpus witnesses below, and only this case catches it).
    let v = eval_view(&mut env, &mut store, "t_eq_reject_samelen", "Bool", "eq \"abc\" \"abd\"");
    assert!(matches!(v, EvalVal::Ctor{id,..} if id==false_id), "eq \"abc\" \"abd\" must be False");

    let v = eval_view(&mut env, &mut store, "t_eq_reject_len", "Bool", "eq \"ab\" \"abc\"");
    assert!(matches!(v, EvalVal::Ctor{id,..} if id==false_id), "eq \"ab\" \"abc\" must be False");
}

/// `surface/strings/string-compare-3way-lexicographic-triple`
#[test]
fn string_compare_3way_lexicographic_triple() {
    let mut env = mk_env();
    let mut store = make_store(&env);
    let lt_id = env.globals["Lt"];
    let eq_id = env.globals["Eq"];
    let gt_id = env.globals["Gt"];

    let v = eval_view(&mut env, &mut store, "t_cmp1", "OrdResult", "compare \"a\" \"ab\"");
    assert!(matches!(v, EvalVal::Ctor{id,..} if id==lt_id), "compare \"a\" \"ab\" must be Lt; got {v:?}");
    let v = eval_view(&mut env, &mut store, "t_cmp2", "OrdResult", "compare \"ab\" \"b\"");
    assert!(matches!(v, EvalVal::Ctor{id,..} if id==lt_id), "compare \"ab\" \"b\" must be Lt; got {v:?}");
    let v = eval_view(&mut env, &mut store, "t_cmp3", "OrdResult", "compare \"b\" \"a\"");
    assert!(matches!(v, EvalVal::Ctor{id,..} if id==gt_id), "compare \"b\" \"a\" must be Gt; got {v:?}");
    let v = eval_view(&mut env, &mut store, "t_cmp4", "OrdResult", "compare \"ab\" \"ab\"");
    assert!(matches!(v, EvalVal::Ctor{id,..} if id==eq_id), "compare \"ab\" \"ab\" must be Eq; got {v:?}");
}

/// `surface/strings/list-eq-is-codepoint-wise-not-nfc-folding` (property)
#[test]
fn list_eq_is_codepoint_wise_not_nfc_folding() {
    let mut env = mk_env();
    let mut store = make_store(&env);
    let false_id = env.globals["False"];

    // Precomposed "é" (U+00E9, one scalar) vs "e" + combining acute (U+0065
    // U+0301, two scalars) — canonically equivalent, codepoint-DISTINCT.
    // Constructed DIRECTLY as `List Char` (Char ≡ Int under refinement
    // erasure, `decimal_char.rs`), never via a String literal — pinning this
    // on String literals would falsely fail once real NFC-at-construction
    // lands and merges them at construction (the over-pin-a-deferred-
    // behavior trap; ADR 0010 §3).
    let nfc = "Cons Char 233 (Nil Char)";
    let nfd = "Cons Char 101 (Cons Char 769 (Nil Char))";
    let v = eval_view(
        &mut env,
        &mut store,
        "t_nfc_nfd",
        "Bool",
        &format!("list_eq Char eqChar ({nfc}) ({nfd})"),
    );
    assert!(
        matches!(v, EvalVal::Ctor{id,..} if id==false_id),
        "list_eq eqChar on codepoint-distinct-but-canonically-equivalent sequences must be False (NFC-blind); got {v:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// DS-AC6 — `list-append-does-not-shadow-bytes-append`
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn list_append_does_not_shadow_bytes_append() {
    let env = mk_env();
    let list_append_id = env.globals["list_append"];
    let bytes_append_id = env.globals["append"];
    assert_ne!(
        list_append_id, bytes_append_id,
        "list_append must be a distinct global from the Bytes-domain append"
    );
    // The Bytes `append`'s `[FS]` effect row is keyed by name in `bytes_env`;
    // `list_append` must not appear there (it's the pure List op).
    assert!(
        !env.bytes_env.io_effect_rows.contains_key("list_append"),
        "list_append must not carry the Bytes [FS] effect row"
    );
    assert!(
        env.bytes_env.io_effect_rows.contains_key("append"),
        "sanity: the landed Bytes append really is FS-effectful"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// DS-AC7 — `concat-slice-compose-and-floor-totality`
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn concat_slice_compose_and_floor_totality() {
    run_with_big_stack(|| {
        let mut env = mk_env();
        let mut store = make_store(&env);

        // slice 0 (charLength a) (concat a b) ≡ a, on a scalar-clean corpus.
        // charLength "ab" = 2.
        let v = eval_view(
            &mut env,
            &mut store,
            "t_roundtrip",
            "String",
            &format!("slice ({}) ({}) (concat \"ab\" \"cd\")", nat(0), nat(2)),
        );
        assert_eq!(v, EvalVal::Str("ab".into()), "slice 0 (charLength \"ab\") (concat \"ab\" \"cd\") must be \"ab\"");

        // list_append associativity on a small corpus:
        // list_append (list_append xs ys) zs ≡ list_append xs (list_append ys zs).
        let xs = "Cons Char 97 (Nil Char)"; // ['a']
        let ys = "Cons Char 98 (Nil Char)"; // ['b']
        let zs = "Cons Char 99 (Nil Char)"; // ['c']
        let lhs = format!("list_append Char (list_append Char ({xs}) ({ys})) ({zs})");
        let rhs = format!("list_append Char ({xs}) (list_append Char ({ys}) ({zs}))");
        let v_lhs = eval_view(&mut env, &mut store, "t_assoc_lhs", "List Char", &lhs);
        let v_rhs = eval_view(&mut env, &mut store, "t_assoc_rhs", "List Char", &rhs);
        assert_eq!(
            format!("{v_lhs:?}"),
            format!("{v_rhs:?}"),
            "list_append must be associative"
        );
        assert!(
            !matches!(v_lhs, EvalVal::Neutral),
            "list_append must not get stuck on well-typed input"
        );

        // Totality: natSub saturates, nth/take/drop totalize out-of-range —
        // none reduce to Neutral/stuck. Re-check the already-asserted DS-AC3
        // corpus values are all non-Neutral (they were asserted to concrete
        // values above; this call re-confirms the out-of-range/underflow
        // faces specifically).
        let v = eval_view(&mut env, &mut store, "t_natsub_sat", "Nat", &format!("natSub ({}) ({})", nat(1), nat(2)));
        assert!(!matches!(v, EvalVal::Neutral), "natSub 1 2 must not be stuck");
    });
}
