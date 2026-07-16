//! DS-6c — elaborator `IntLit` emission (the DS-6b fast-follow, ADR 0013
//! Layer 2: `docs/adr/0013-int-decidable-equality-kernel-posture.md`).
//!
//! DS-6b landed the kernel mechanism (`Term::IntLit` + `eq_reduce`) but left
//! it surface-unreachable — nothing in `.ken` source ever produced an
//! `IntLit`. This proves the REAL surface-syntax path: an ordinary `.ken`
//! integer literal now elaborates to `Term::IntLit`, so `Eq Int 5 5`
//! genuinely computes end-to-end from real source, not a hand-built kernel
//! term (that style of test lives in `ken-kernel/tests/`, one layer down).

use ken_elaborator::ElabEnv;
use ken_kernel::env::Context;
use ken_kernel::error::KernelError;
use ken_kernel::term::Term;
use ken_kernel::whnf;
use num_bigint::BigInt;

/// AC1 — surface reachability, positive arm: a real `.ken` integer literal
/// elaborates directly to `Term::IntLit` (structural check on the ACTUAL
/// declaration body `elab_num_lit_infer` produced, not a hand-built term),
/// and referencing it twice in an `Equal Int` goal reduces to `Top` and
/// accepts `Proved` end to end from real surface syntax.
///
/// (The literal itself is written in EXPRESSION position — `= 5` — and
/// referenced by name in the type position, `Equal Int five five`, rather
/// than written directly as `Equal Int 5 5`: the surface type-annotation
/// grammar doesn't accept a bare numeric-literal argument today — confirmed
/// pre-existing and orthogonal to this WP, `lemma b : Equal Bool True True`
/// parses fine, `lemma a : Equal Int 5 5` doesn't, `Equal Int x x` with a
/// variable does. DS-6c is emission/wiring only, not a parser change, so
/// this works within the existing grammar rather than extending it.)
#[test]
fn real_int_literal_elaborates_to_intlit_and_tt_checks_end_to_end() {
    let mut env = ElabEnv::new().expect("base env");

    let five_id = env
        .elaborate_decl("const five : Int = 5")
        .expect("a plain Int literal must elaborate");
    let (_, five_body) = env
        .env
        .transparent_body(five_id)
        .expect("const five must be transparent");
    assert_eq!(
        five_body,
        Term::IntLit(BigInt::from(5)),
        "a real .ken Int literal must elaborate directly to Term::IntLit, not an opaque postulate reference — got {:?}",
        five_body
    );

    let id = env
        .elaborate_decl("lemma int_5_eq_5 : Equal Int five five = Proved")
        .expect("Proved must check against the reduced Top goal");
    let (_, ty) = env
        .env
        .const_type(id)
        .expect("declared type must be recorded");
    let ctx = Context::new();
    let reduced = whnf(&env.env, &ctx, &ty);
    assert!(
        matches!(&reduced, Term::Const { id, .. } if *id == env.env.top_id()),
        "Equal Int five five must whnf to Top (DS-6b's arm firing on a real surface literal, via five's own transparent unfolding), got {:?}",
        reduced
    );
}

/// AC1 — surface reachability, negative arm: `Equal Int 5 6` reduces to
/// `Bottom`, and BOTH `Proved` and `Refl` are rejected against it — specific
/// `TypeMismatch`, not bare `is_err()` (per the Architect's correction:
/// `Bottom` has no valid introduction form at all, so `Refl` doesn't fail
/// via `BadEliminator` here — the whnf'd goal isn't even `Eq`-shaped by the
/// time `Refl`'s check rule inspects it).
#[test]
fn real_distinct_int_literals_reduce_to_bottom_and_reject_tt_and_refl() {
    let mut env = ElabEnv::new().expect("base env");
    env.elaborate_decl("const five : Int = 5").expect("five");
    env.elaborate_decl("const six : Int = 6").expect("six");

    let err_tt = env
        .elaborate_decl("lemma int_5_eq_6_tt : Equal Int five six = Proved")
        .expect_err("Proved must not prove Equal Int five six");
    assert!(
        matches!(
            &err_tt,
            ken_elaborator::ElabError::KernelRejected {
                error: KernelError::TypeMismatch { .. },
                ..
            }
        ),
        "expected KernelRejected(TypeMismatch), got {:?}",
        err_tt
    );

    // `Refl`'s rejection is a DIFFERENT (but equally specific) ElabError
    // variant than `Proved`'s: the elaborator's own surface handling of `Refl`
    // must inspect the expected type's shape to infer `Refl`'s witness
    // BEFORE ever dispatching into the kernel, so it short-circuits with
    // its own `ElabError::TypeMismatch{reason: "Refl expects an
    // `Eq`-shaped goal"}` diagnostic rather than reaching
    // `KernelRejected(KernelError::TypeMismatch)` the way `Proved` does.
    // Asserting the SPECIFIC variant each path actually produces, not
    // assuming both funnel through the same one.
    let err_refl = env
        .elaborate_decl("lemma int_5_eq_6_refl : Equal Int five six = Refl")
        .expect_err("Refl must not prove Equal Int five six");
    assert!(
        matches!(&err_refl, ken_elaborator::ElabError::TypeMismatch { .. }),
        "expected ElabError::TypeMismatch, got {:?}",
        err_refl
    );

    // Independently confirm the goal itself reduces to Bottom (not merely
    // "some rejection happened for some other reason") — re-derive the
    // type from `five`/`six`'s own real elaborated bodies, not hand-typed
    // literals, so this stays anchored to the actual surface declarations.
    let (_, five_body) = env
        .env
        .transparent_body(*env.globals.get("five").unwrap())
        .unwrap();
    let (_, six_body) = env
        .env
        .transparent_body(*env.globals.get("six").unwrap())
        .unwrap();
    let int_ty = ken_kernel::Term::const_(env.numeric_env.int_id, vec![]);
    let ctx = Context::new();
    let goal = ken_kernel::Term::Eq(Box::new(int_ty), Box::new(five_body), Box::new(six_body));
    let reduced = whnf(&env.env, &ctx, &goal);
    assert!(
        matches!(&reduced, Term::Const { id, .. } if *id == env.env.bottom_id()),
        "Eq Int five six must whnf to Bottom, got {:?}",
        reduced
    );
}

/// Confirms the `Refl`-vs-`Proved` asymmetry above is genuinely a NARROWING to
/// the correct rejection shape, not an over-broad pre-check that would also
/// wrongly reject `Refl` on a legitimately `Eq`-shaped goal. `Refl`'s
/// surface-level shape check only fires when the whnf'd expected type has
/// ALREADY collapsed past `Eq` (to `Top`/`Bottom`); for an ABSTRACT (still
/// genuinely `Eq`-shaped, never reduced) goal, `Refl` must still be
/// accepted exactly as before this WP.
#[test]
fn refl_still_accepted_on_a_genuinely_abstract_eq_shaped_goal() {
    let mut env = ElabEnv::new().expect("base env");
    env.elaborate_decl("lemma int_refl_abstract (x : Int) : Equal Int x x = Refl")
        .expect("Refl must still check against a genuinely abstract (unreduced) Eq Int x x goal");
}

/// AC2 — the hack is retired on the Int path: elaborating an Int literal
/// mints NO opaque `Decl::Primitive{reduction: Literal}` and inserts
/// NOTHING into `num_values`. Grep-confirmable evidence, not just absence
/// of a symptom.
#[test]
fn int_literal_mints_no_postulate_and_no_num_values_entry() {
    let mut env = ElabEnv::new().expect("base env");
    let before_decls = env.env.decls().count();
    let before_num_values = env.num_values.len();

    env.elaborate_decl("const some_int : Int = 42")
        .expect("plain Int literal must elaborate");

    let after_decls = env.env.decls().count();
    let after_num_values = env.num_values.len();

    // Exactly one NEW declaration — the `const some_int` binding itself
    // (a `Decl::Transparent`) — and nothing else. No fresh
    // `Decl::Primitive{Literal}` postulate accompanying it.
    assert_eq!(
        after_decls,
        before_decls + 1,
        "elaborating an Int literal must add exactly one Decl (the binding itself), no side postulate"
    );
    assert_eq!(
        after_num_values, before_num_values,
        "an Int literal must insert nothing into num_values (the value lives in the IntLit term itself)"
    );
}

/// AC3 (Architect-corrected wording) — `trusted_base()` delta is exactly
/// EMPTY, not a shrink: `PrimReduction::Literal` primitives were already
/// excluded from `trusted_base()` before this WP (an accounting-neutral
/// value, per the pre-existing doc comment at the old call site), so
/// retiring them doesn't move the trusted_base() count — it reduces the
/// overall `Decl` surface (fewer declarations minted per literal
/// occurrence), a real simplification, just not a trusted_base()-visible
/// one. Assert the honest claim, not an over-claimed shrink.
#[test]
fn trusted_base_delta_is_exactly_empty_not_a_shrink() {
    let mut env = ElabEnv::new().expect("base env");
    let before: std::collections::BTreeSet<_> = env.env.trusted_base().into_iter().collect();

    env.elaborate_decl("const int_tb_probe : Int = 7")
        .expect("must elaborate");

    let after: std::collections::BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    assert_eq!(
        before, after,
        "Int-literal elaboration must leave trusted_base() completely unchanged (empty delta, not a shrink — Literal primitives were never counted)"
    );
}

/// AC4 — runtime intact: the interpreter evaluates a real elaborated
/// `IntLit` term directly (no `num_values` lookup involved), confirming
/// `eval()`'s new `Term::IntLit` arm is wired, not just present in isolation.
#[test]
fn interpreter_evaluates_real_elaborated_intlit() {
    let mut env = ElabEnv::new().expect("base env");
    let id = env
        .elaborate_decl("const int_runtime_probe : Int = 12345")
        .expect("must elaborate");
    let (_, body) = env
        .env
        .transparent_body(id)
        .expect("const binding must be transparent");

    let mut store = ken_interp::EvalStore::new();
    // Deliberately do NOT populate store.num_values — if eval() still
    // depended on the side table for this term, it would degrade to
    // Neutral instead of the real value.
    let result = ken_interp::eval(&[], &body, &env.env, &mut store);
    assert_eq!(
        result,
        ken_interp::EvalVal::Int(12345),
        "IntLit must evaluate to its real value with no num_values entry, got {:?}",
        result
    );
}
