//! `surface-arrow-and-infix` acceptance — the arrow-in-expr piece (VAL2 #4),
//! folded in once `spec/30-surface/32-grammar.md` landed the expr-position
//! `->` productions (`origin/main@23a6cba`). The `-`/`*` infix piece has its
//! own acceptance file (`surface_arrow_and_infix_acceptance.rs`).
//!
//! `->` in expr position was previously reachable ONLY inside a type
//! annotation. Per Steward's flagged hazard (gate-widening can expose a
//! latent bug on a newly-reachable path,
//! `[[gate-widening-exposes-latent-bugs-in-newly-reachable-code]]`), this
//! file specifically exercises positions the OLD surface could not reach:
//! a function argument, a `let`-bound value, and a `view`'s returned body —
//! not just the annotation position, which already worked.
//!
//! - AC2 — an arrow-type expression elaborates to the correct `Pi` (checked
//!   structurally, not just "it ran"), in each of the newly-reachable
//!   positions, plus the dependent form and right-associativity.
//! - AC1 (kernel untouched) is verified out-of-band (`git diff -- crates/
//!   ken-kernel/` empty) — both new `RExpr` arms (`RPi`/`RArrow`) elaborate
//!   to the existing `Term::Pi`, no new kernel variant.

use ken_elaborator::ElabEnv;
use ken_kernel::Term;

fn fresh_env() -> ElabEnv {
    ElabEnv::new().expect("prelude should elaborate")
}

fn body_of(env: &ElabEnv, id: ken_kernel::GlobalId) -> Term {
    env.env.transparent_body(id).expect("must be transparent").1
}

/// Does `t` contain a `Term::Pi` anywhere in its structure? (`let`/`app`
/// wrappers around the arrow value aren't reduced away in the stored body,
/// so a plain top-level `matches!` would miss a `let`-bound or argument-
/// position arrow even though it elaborated correctly.)
fn contains_pi(t: &Term) -> bool {
    match t {
        Term::Pi(_, _) => true,
        Term::App(f, a) => contains_pi(f) || contains_pi(a),
        Term::Lam(a, b) | Term::Sigma(a, b) => contains_pi(a) || contains_pi(b),
        Term::Ascript(a, b) => contains_pi(a) || contains_pi(b),
        Term::Let { ty, val, body } => contains_pi(ty) || contains_pi(val) || contains_pi(body),
        _ => false,
    }
}

#[test]
fn arrow_returned_from_view_body_is_a_real_pi() {
    let mut env = fresh_env();
    // `Int -> Int` written as the BODY, not an annotation — unreachable by
    // the old surface (`->` was type-position only).
    let id = env.elaborate_decl("const t : Type = Int -> Int").expect("arrow as body value");
    assert!(matches!(body_of(&env, id), Term::Pi(_, _)), "must lower to a real Term::Pi");
}

#[test]
fn arrow_let_bound_is_a_real_pi() {
    let mut env = fresh_env();
    let id = env
        .elaborate_decl("const t : Type = let ty = Int -> Bool in ty")
        .expect("arrow let-bound");
    assert!(contains_pi(&body_of(&env, id)), "let-bound arrow must lower to Term::Pi");
}

#[test]
fn arrow_passed_as_an_ordinary_function_argument() {
    let mut env = fresh_env();
    env.elaborate_decl("const id_ty (t : Type) : Type = t").expect("id_ty declares");
    let id = env
        .elaborate_decl("const t : Type = id_ty (Int -> Bool)")
        .expect("arrow passed as a function argument");
    // `id_ty` is the identity on Type, so the body contains the argument's
    // core term (wrapped in the `App(Const(id_ty), _)` call).
    assert!(contains_pi(&body_of(&env, id)), "argument-position arrow must lower to Term::Pi");
}

#[test]
fn dependent_arrow_in_expr_position_is_a_real_pi() {
    let mut env = fresh_env();
    let id = env
        .elaborate_decl("const t : Type = (x : Int) -> Bool")
        .expect("dependent arrow as body value");
    assert!(matches!(body_of(&env, id), Term::Pi(_, _)), "dependent arrow must lower to Term::Pi");
}

#[test]
fn arrow_is_right_associative() {
    let mut env = fresh_env();
    let id = env
        .elaborate_decl("const t : Type = Int -> Int -> Bool")
        .expect("right-assoc arrow chain");
    // `Int -> Int -> Bool` = `Int -> (Int -> Bool)`: outer Pi's codomain is
    // itself a Pi (not the other possible grouping).
    match body_of(&env, id) {
        Term::Pi(_, cod) => assert!(matches!(*cod, Term::Pi(_, _)), "must group as Int -> (Int -> Bool)"),
        other => panic!("expected a Term::Pi, got {other:?}"),
    }
}

#[test]
fn plain_parenthesized_ascription_is_unaffected() {
    // The exact shape the dependent-arrow speculative parse must NOT
    // misfire on: `(x : Int)` with no trailing `->` is an ordinary
    // parenthesized ascription, unrelated to VAL2 #4.
    let mut env = fresh_env();
    env.elaborate_decl("fn t (x : Int) : Int = (x : Int)")
        .expect("plain parenthesized ascription must still parse and elaborate");
}

#[test]
fn ill_typed_arrow_domain_is_still_kernel_rejected() {
    // Soundness spot-check (AC4-equivalent for this piece): an arrow whose
    // domain is NOT actually a type (a plain Int value) must still be
    // rejected — the new expr-position path doesn't bypass kernel Pi
    // well-formedness checking.
    let mut env = fresh_env();
    let res = env.elaborate_decl("const t : Type = 5 -> Bool");
    assert!(res.is_err(), "an arrow with a non-type domain must be rejected, got {res:?}");
}

#[test]
fn annotation_position_arrow_unaffected_by_the_new_expr_path() {
    // The pre-existing (already-working) type-annotation use must stay
    // byte-behaviorally unaffected by this widening.
    let mut env = fresh_env();
    env.elaborate_decl("fn f (g : Int -> Int) (x : Int) : Int = g x")
        .expect("annotation-position arrow must still work exactly as before");
}
