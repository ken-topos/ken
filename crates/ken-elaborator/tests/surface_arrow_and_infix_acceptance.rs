//! `surface-arrow-and-infix` acceptance — the `-`/`*` infix piece (VAL2
//! #11). Arrow-in-expr (VAL2 #4) is a separate, later piece of this same WP
//! (gated on a `/spec` grammar merge) — not covered by this file.
//!
//! - AC2 — infix `-`/`*` parse + elaborate + evaluate to the same values as
//!   the prefix `sub_int`/`mul_int` forms (discriminating parse+eval test),
//!   plus the conventional-precedence pin (`*` binds tighter than `+`/`-`).
//! - AC1 (kernel untouched) / AC3 (no regression) are verified out-of-band
//!   (`git diff -- crates/ken-kernel/` empty; `cargo test --workspace`).

use ken_elaborator::ElabEnv;
use ken_interp::eval::{eval, EvalStore, EvalVal};

fn fresh_env() -> ElabEnv {
    ElabEnv::new().expect("prelude should elaborate")
}

fn eval_int_decl(env: &mut ElabEnv, src: &str) -> i128 {
    let id = env.elaborate_decl(src).expect("decl should elaborate");
    let mut store = EvalStore::new();
    for (nid, v) in &env.num_values {
        store.num_values.entry(*nid).or_insert_with(|| match v {
            ken_elaborator::NumericLitVal::Int(n) => EvalVal::from(*n),
            other => panic!("unexpected literal kind in this test: {other:?}"),
        });
    }
    let body = match env.env.lookup(id) {
        Some(ken_kernel::Decl::Transparent { body, .. }) => body.clone(),
        _ => panic!("decl should be Transparent"),
    };
    match eval(&[], &body, &env.env, &mut store) {
        EvalVal::Int(n) => n as i128,
        other => panic!("expected an Int value, got {other:?}"),
    }
}

#[test]
fn infix_minus_matches_prefix_sub_int() {
    let mut env = fresh_env();
    let infix = eval_int_decl(&mut env, "const r : Int = 10 - 3");
    let mut env2 = fresh_env();
    let prefix = eval_int_decl(&mut env2, "const r : Int = sub_int 10 3");
    assert_eq!(infix, prefix, "infix '-' must match prefix 'sub_int'");
    assert_eq!(infix, 7);
}

#[test]
fn infix_star_matches_prefix_mul_int() {
    let mut env = fresh_env();
    let infix = eval_int_decl(&mut env, "const r : Int = 6 * 7");
    let mut env2 = fresh_env();
    let prefix = eval_int_decl(&mut env2, "const r : Int = mul_int 6 7");
    assert_eq!(infix, prefix, "infix '*' must match prefix 'mul_int'");
    assert_eq!(infix, 42);
}

#[test]
fn star_binds_tighter_than_plus_and_minus() {
    let mut env = fresh_env();
    // 2 + 3 * 4 = 2 + 12 = 14, NOT (2 + 3) * 4 = 20.
    let v = eval_int_decl(&mut env, "const r : Int = 2 + 3 * 4");
    assert_eq!(v, 14, "'*' must bind tighter than '+' (conventional precedence)");

    let mut env2 = fresh_env();
    // 20 - 3 * 4 = 20 - 12 = 8, NOT (20 - 3) * 4 = 68.
    let v2 = eval_int_decl(&mut env2, "const r : Int = 20 - 3 * 4");
    assert_eq!(v2, 8, "'*' must bind tighter than '-' (conventional precedence)");
}

#[test]
fn minus_and_plus_are_left_associative() {
    let mut env = fresh_env();
    // 10 - 3 - 2 = (10 - 3) - 2 = 5, NOT 10 - (3 - 2) = 9.
    let v = eval_int_decl(&mut env, "const r : Int = 10 - 3 - 2");
    assert_eq!(v, 5, "'-' must be left-associative");
}
