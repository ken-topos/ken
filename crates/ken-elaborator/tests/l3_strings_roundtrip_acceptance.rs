//! `L3-strings-roundtrip` acceptance tests (`37 §2.3`, Phase-3 slice 1).
//!
//! `string_to_list_char`/`list_char_to_string` are real now (native +
//! checked, `ken-interp/src/eval.rs`). ACs per `docs/program/wp/
//! L3-strings-roundtrip.md`:
//! - AC1: witness soundness — every `Char` `s2l` produces is a valid scalar.
//! - AC2: round-trip identity (non-circular defining oracle).
//! - AC3: independent UTF-8 boundary corpus + surrogate guard.
//! - AC4: `l2s` totality.
//! - AC5: whole-WP (kernel diff empty, workspace-green) — verified by CI/K7,
//!   not a unit test here.

use ken_elaborator::{ElabEnv, NumericLitVal};
use ken_interp::eval::{EvalStore, EvalVal, ListCharIds};
use ken_kernel::{Decl, GlobalId};

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
        Some(Decl::Transparent { body, .. }) => {
            ken_interp::eval::eval(&[], body, &env.env, store)
        }
        _ => EvalVal::Unknown,
    }
}

/// Elaborate `view <name> : <ty> = <expr>` and evaluate it.
///
/// Each new string literal in `expr` registers a fresh numeric-literal
/// postulate in `env.num_values` at elaboration time — `store.num_values`
/// must be resynced after every `elaborate_decl` call, not just once at
/// `make_store` time, or a literal declared after store creation evaluates
/// against a stale (missing-entry) snapshot.
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

/// Ken string-literal source for `s`: the lexer accumulates raw `char`s with
/// no escape processing (`lexer.rs:160-181`), so any Rust `&str` (including
/// exotic-Unicode content built via `\u{...}` Rust literals) embeds directly.
fn str_lit(s: &str) -> String {
    format!("\"{s}\"")
}

/// `string_to_list_char <s>` as a Ken source expression (type `List Char`).
fn s2l_expr(s: &str) -> String {
    format!("string_to_list_char {}", str_lit(s))
}

/// Decode a `List Char` `EvalVal` into a `Vec<u32>` of raw codepoints, by
/// walking the `Nil`/`Cons` chain directly (mirrors `eval.rs`'s
/// `list_char_to_evalval_string`, but surfacing the codepoints for AC1/AC3
/// range assertions rather than building a `String`).
fn list_char_codepoints(env: &ElabEnv, v: &EvalVal) -> Vec<u32> {
    let nil_id = env.prelude_env.nil_id;
    let cons_id = env.prelude_env.cons_id;
    let mut out = Vec::new();
    let mut cur = v.clone();
    loop {
        match &cur {
            EvalVal::Ctor { id, .. } if *id == nil_id => return out,
            EvalVal::Ctor { id, args, .. } if *id == cons_id => {
                match &args[1] {
                    EvalVal::Int(n) => out.push(*n as u32),
                    other => panic!("Cons head must be an Int-typed Char, got {other:?}"),
                }
                cur = args[2].clone();
            }
            other => panic!("not a well-formed List Char Ctor chain: {other:?}"),
        }
    }
}

/// `isScalar` per `decimal_char.rs:225`'s `inRangeBool`: `[0,55295] ∪
/// [57344,1114111]` — the surrogate-excluding Unicode scalar range.
fn is_scalar(cp: u32) -> bool {
    cp <= 55295 || (57344..=1114111).contains(&cp)
}

// ── AC1: witness soundness — every decoded Char is a valid scalar ──────────

/// `surface/strings/s2l-witness-soundness` — every codepoint `string_to_list_char`
/// produces satisfies `isScalar` (the surrogate-excluding scalar range), for
/// the full boundary corpus plus a mixed multi-scalar string.
#[test]
fn ac1_s2l_witness_is_always_a_valid_scalar() {
    let mut env = ElabEnv::new().expect("base env");
    let mut store = make_store(&env);

    let corpus = [
        "",
        "\u{0000}",
        "\u{007F}",
        "\u{0080}",
        "\u{07FF}",
        "\u{0800}",
        "\u{FFFF}",
        "\u{10000}",
        "\u{10FFFF}",
        "a\u{0080}\u{10FFFF}b\u{0800}",
    ];
    for s in corpus {
        let v = eval_view(&mut env, &mut store, "t_ac1", "List Char", &s2l_expr(s));
        for cp in list_char_codepoints(&env, &v) {
            assert!(is_scalar(cp), "codepoint {cp:#x} from {s:?} is not a valid scalar");
        }
    }
}

// ── AC2: round-trip identity (non-circular defining oracle) ────────────────

/// `surface/strings/roundtrip-l2s-s2l` — `list_char_to_string
/// (string_to_list_char s) ≡ s` for the boundary corpus.
#[test]
fn ac2_round_trip_l2s_s2l_identity() {
    let mut env = ElabEnv::new().expect("base env");
    let mut store = make_store(&env);

    let corpus = [
        "",
        "\u{0000}",
        "\u{007F}",
        "\u{0080}",
        "\u{07FF}",
        "\u{0800}",
        "\u{FFFF}",
        "\u{10000}",
        "\u{10FFFF}",
        "hello, \u{4e16}\u{754c}! \u{10000}\u{1F600}",
    ];
    for s in corpus {
        let v = eval_view(
            &mut env,
            &mut store,
            "t_ac2",
            "String",
            &format!("list_char_to_string (string_to_list_char {})", str_lit(s)),
        );
        assert_eq!(
            v,
            EvalVal::Str(s.to_string()),
            "round-trip failed for {s:?}"
        );
    }
}

/// `surface/strings/roundtrip-s2l-l2s` — `string_to_list_char
/// (list_char_to_string cs) ≡ cs` for a well-formed `cs`, built directly as a
/// Ken `List Char` literal expression (not via `s2l`, so this is a genuinely
/// independent direction from AC2's first test — a round-trip pinned from
/// both ends nets the inverse-error pair a one-directional check would miss).
#[test]
fn ac2_round_trip_s2l_l2s_identity() {
    let mut env = ElabEnv::new().expect("base env");
    let mut store = make_store(&env);

    // `Cons Char 65 (Cons Char 66 (Nil Char))` — "AB" as an explicit List Char.
    let cs_expr = "Cons Char 65 (Cons Char 66 (Nil Char))";
    let v = eval_view(
        &mut env,
        &mut store,
        "t_ac2b",
        "List Char",
        &format!("string_to_list_char (list_char_to_string ({cs_expr}))"),
    );
    assert_eq!(
        list_char_codepoints(&env, &v),
        vec![65, 66],
        "s2l(l2s cs) must reproduce cs's codepoints exactly"
    );
}

// ── AC3: independent boundary corpus + surrogate guard ──────────────────────

/// `surface/strings/utf8-boundary-corpus` — each UTF-8 length-boundary
/// codepoint decodes to its exact objective Unicode value (CV independently
/// re-derives these against the Unicode standard, not implementer-trusted).
#[test]
fn ac3_utf8_length_boundary_corpus() {
    let mut env = ElabEnv::new().expect("base env");
    let mut store = make_store(&env);

    let cases: &[(&str, u32)] = &[
        ("\u{0000}", 0x0000), // 1-byte min
        ("\u{007F}", 0x007F), // 1-byte max
        ("\u{0080}", 0x0080), // 2-byte min
        ("\u{07FF}", 0x07FF), // 2-byte max
        ("\u{0800}", 0x0800), // 3-byte min
        ("\u{FFFF}", 0xFFFF), // 3-byte max
        ("\u{10000}", 0x10000), // 4-byte min
        ("\u{10FFFF}", 0x10FFFF), // 4-byte max (Unicode ceiling)
    ];
    for (s, expected) in cases {
        let v = eval_view(&mut env, &mut store, "t_ac3", "List Char", &s2l_expr(s));
        assert_eq!(
            list_char_codepoints(&env, &v),
            vec![*expected],
            "boundary codepoint mismatch for {s:?}"
        );
    }
}

/// `surface/strings/utf8-mixed-and-empty` — a mixed multi-scalar string and
/// the empty string, per AC3's explicit extra cases beyond the boundary set.
#[test]
fn ac3_mixed_multiscalar_and_empty_string() {
    let mut env = ElabEnv::new().expect("base env");
    let mut store = make_store(&env);

    let v_empty = eval_view(&mut env, &mut store, "t_ac3_empty", "List Char", &s2l_expr(""));
    assert_eq!(list_char_codepoints(&env, &v_empty), Vec::<u32>::new());

    let mixed = "A\u{00e9}\u{4e2d}\u{1F600}z";
    let v_mixed = eval_view(&mut env, &mut store, "t_ac3_mixed", "List Char", &s2l_expr(mixed));
    assert_eq!(
        list_char_codepoints(&env, &v_mixed),
        vec![0x41, 0xe9, 0x4e2d, 0x1F600, 0x7A]
    );
}

/// `surface/strings/surrogate-guard` — `string_to_list_char` never emits a
/// codepoint in the surrogate range `[0xD800,0xDFFF]`.
///
/// **Transported, not flipped:** under a validated `String` input, Rust's own
/// `char`/`str` invariants structurally prevent a non-scalar from ever
/// reaching the `Char` constructor — there is no fabrication path this
/// assertion could catch. It proves the invariant *transports* through decode
/// (the corpus never violates it); it cannot demonstrate what a violation
/// would look like, because none is constructible (a lone surrogate is not
/// valid UTF-8, so no Rust `&str` can carry one). The genuinely discriminating
/// witness test — one that can actually reject an out-of-range `Int` — lives
/// at `intToChar` in `decimal_char.rs`, not here.
#[test]
fn ac3_surrogate_guard_across_corpus() {
    let mut env = ElabEnv::new().expect("base env");
    let mut store = make_store(&env);

    let corpus = [
        "\u{0000}", "\u{007F}", "\u{0080}", "\u{07FF}", "\u{0800}",
        "\u{D7FF}", "\u{E000}", "\u{FFFF}", "\u{10000}", "\u{10FFFF}",
    ];
    for s in corpus {
        let v = eval_view(&mut env, &mut store, "t_ac3_guard", "List Char", &s2l_expr(s));
        for cp in list_char_codepoints(&env, &v) {
            assert!(
                !(0xD800..=0xDFFF).contains(&cp),
                "surrogate {cp:#x} leaked from {s:?}"
            );
        }
    }
}

// ── AC4: l2s totality ────────────────────────────────────────────────────

/// `surface/strings/l2s-totality` — `list_char_to_string` returns a `String`
/// for a non-trivial `List Char`, never a stuck `Neutral`.
#[test]
fn ac4_l2s_totality_on_nontrivial_list() {
    let mut env = ElabEnv::new().expect("base env");
    let mut store = make_store(&env);

    // `['K', 'e', 'n']` as an explicit List Char literal — not built via s2l,
    // so this exercises l2s independently of the s2l path.
    let cs_expr = "Cons Char 75 (Cons Char 101 (Cons Char 110 (Nil Char)))";
    let v = eval_view(
        &mut env,
        &mut store,
        "t_ac4",
        "String",
        &format!("list_char_to_string ({cs_expr})"),
    );
    assert!(
        !matches!(v, EvalVal::Neutral),
        "l2s must not get stuck on a non-trivial List Char"
    );
    assert_eq!(v, EvalVal::Str("Ken".to_string()));
}

// ── Out-of-scope guard (verify by absence) ──────────────────────────────────

/// The derived surface (`concat`/`slice`/`charAt`/`eq`/`compare` over `List
/// Char`) is slice 2 (Team Language), gated on this WP — out of scope here.
/// Pin that it is NOT yet callable, so a future slice-2 landing is a visible
/// diff rather than a silent no-op.
#[test]
fn derived_list_char_surface_out_of_scope() {
    let mut env = ElabEnv::new().expect("base env");
    let result = env.elaborate_decl(
        "const t : String = concat (string_to_list_char \"a\") (string_to_list_char \"b\")",
    );
    assert!(result.is_err(), "concat over List Char must not exist yet (slice 2, Team Language)");
}
