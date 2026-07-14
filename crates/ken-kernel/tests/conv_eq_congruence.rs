//! `conv-eq-congruence` acceptance tests (Gap-conv, Map capstone law 4) — the
//! `(Term::Eq, Term::Eq)` congruence arm added to `conv_struct`
//! (`crates/ken-kernel/src/conv.rs`).
//!
//! **Re-landed here under `obs-eq-termination`** — the arm originally merged
//! as `conv-eq-congruence` (`90f39fe`) was **reverted-to-green** (`b79313f`)
//! after it exposed a pre-existing, latent non-termination in the
//! observational reducer (`conv_struct` unconditionally δ-unfolding a
//! transparent recursive `Const` before any congruence dispatch could run,
//! e.g. `allKeys P1 l` vs `allKeys P2 l` for two convertible-but-syntactically-
//! distinct predicate spellings — see `obs_eq_termination_congruence.rs`).
//! `obs-eq-termination`'s congruence-first/lazy-δ fast path in `conv_struct`
//! fixes that root cause, so the arm is safe to re-land unchanged.
//!
//! Fix (vector confirmed + soundness-gated by the Architect,
//! `evt_2s3brvnr2cta2`): two `Eq` *types* now convert iff their three
//! components (motive type / lhs / rhs) do, recursively via the same
//! `conv_struct` — the missing congruence closure for `Eq`, restoring the
//! invariant every other type-former (`Pi`/`Sigma`/`App`/`Elim`/...) already
//! carries. Fail-closed direction only: the arm can only recognise strictly
//! *more* true equalities (never accepts a false one), because each
//! component is still checked via the same sound `conv_struct`.
//!
//! AC3 (`cTest`, the discriminating isolation-flip): `h : Eq Bool (idB x)
//! True` (`idB b := b`) must check at the reconstructed-endpoint type `Eq
//! Bool x True` — the `idB x` redex is stuck inside the `Eq`'s `a`
//! component, so only a component-wise `conv_struct` recursion (this fix)
//! whnfs it down to `x`; pre-fix, `conv_struct` had no `(Eq, Eq)` arm at all
//! and fell straight to `_ => false` regardless of whether the components
//! were convertible. Flip verified empirically (git-stash the `conv.rs` arm,
//! rerun: `eq_congruence_reconstructed_lhs_converts` and
//! `eq_congruence_both_sides_reconstructed_converts` fail; restore: pass).
//!
//! AC4 (over-conversion gate, the real hazard of a conv-completeness fix):
//! `eq_congruence_distinct_endpoint_stays_rejected` pins that a genuinely
//! different endpoint (`True` vs `False`) is NOT admitted by the new
//! congruence arm — it must recurse to a real, load-bearing
//! `conv_struct(True, False)` check that fails, not short-circuit to true.

use ken_kernel::env::Context;
use ken_kernel::term::{Level, Term};
use ken_kernel::{
    check, declare_def, declare_inductive, declare_postulate, CtorSpec, GlobalEnv, GlobalId,
    InductiveSpec,
};

struct B {
    bool_: GlobalId,
    true_: GlobalId,
    false_: GlobalId,
    id_b: GlobalId,
}

fn mk_env() -> (GlobalEnv, B) {
    let mut env = GlobalEnv::new();
    let bool_ = declare_inductive(&mut env, |_| InductiveSpec {
        level_params: vec![],
        params: vec![],
        indices: vec![],
        level: Level::zero(),
        constructors: vec![
            CtorSpec {
                args: vec![],
                target_indices: vec![],
            },
            CtorSpec {
                args: vec![],
                target_indices: vec![],
            },
        ],
    })
    .expect("Bool");
    let (true_, false_) = {
        let cs = &env.inductive(bool_).unwrap().constructors;
        (cs[0].id, cs[1].id)
    };
    let bool_t = Term::indformer(bool_, vec![]);
    // idB : Bool -> Bool := λb. b
    let id_b = declare_def(
        &mut env,
        vec![],
        Term::pi(bool_t.clone(), bool_t.clone()),
        Term::lam(bool_t.clone(), Term::var(0)),
    )
    .expect("idB : Bool -> Bool");
    (
        env,
        B {
            bool_,
            true_,
            false_,
            id_b,
        },
    )
}

fn bool_t(b: &B) -> Term {
    Term::indformer(b.bool_, vec![])
}
fn true_c(b: &B) -> Term {
    Term::constructor(b.true_, vec![])
}
fn false_c(b: &B) -> Term {
    Term::constructor(b.false_, vec![])
}
fn id_b_app(b: &B, x: Term) -> Term {
    Term::app(
        Term::Const {
            id: b.id_b,
            level_args: vec![],
        },
        x,
    )
}
fn const_(id: GlobalId) -> Term {
    Term::Const {
        id,
        level_args: vec![],
    }
}

/// **AC3 — the `cTest` isolation-flip repro.** `h : Eq Bool (idB x) True`
/// checks at `Eq Bool x True` only via the new congruence arm — the `Eq`
/// TYPES differ syntactically (`idB x` vs `x`), so the pre-fix
/// syntactic-identity-only path (`a == b` early return, then `_ => false`)
/// rejects this; only recursing `conv_struct` into the `a` component (which
/// then whnfs `idB x` to `x`) accepts it.
#[test]
fn eq_congruence_reconstructed_lhs_converts() {
    let (mut env, b) = mk_env();
    let x = declare_postulate(&mut env, "test postulate".to_string(), vec![], bool_t(&b)).expect("x : Bool");
    let h_ty = Term::Eq(
        Box::new(bool_t(&b)),
        Box::new(id_b_app(&b, const_(x))),
        Box::new(true_c(&b)),
    );
    let h = declare_postulate(&mut env, "test postulate".to_string(), vec![], h_ty).expect("h : Eq Bool (idB x) True");
    let expected_ty = Term::Eq(Box::new(bool_t(&b)), Box::new(const_(x)), Box::new(true_c(&b)));
    assert!(
        check(&env, &Context::new(), &const_(h), &expected_ty).is_ok(),
        "h : Eq Bool (idB x) True must check at Eq Bool x True (AC3, post-fix)"
    );
}

/// **AC1/AC4 stress — both non-endpoint components need reconstruction, not
/// just one side.** `h2 : Eq Bool (idB x) (idB y)` must check at `Eq Bool x
/// y` — proves the arm is a genuine three-way componentwise congruence
/// (`ty`, `a`, `b` each independently recursing via `conv_struct`), not a
/// single-hardcoded-slot special case.
#[test]
fn eq_congruence_both_sides_reconstructed_converts() {
    let (mut env, b) = mk_env();
    let x = declare_postulate(&mut env, "test postulate".to_string(), vec![], bool_t(&b)).expect("x : Bool");
    let y = declare_postulate(&mut env, "test postulate".to_string(), vec![], bool_t(&b)).expect("y : Bool");
    let h_ty = Term::Eq(
        Box::new(bool_t(&b)),
        Box::new(id_b_app(&b, const_(x))),
        Box::new(id_b_app(&b, const_(y))),
    );
    let h = declare_postulate(&mut env, "test postulate".to_string(), vec![], h_ty).expect("h : Eq Bool (idB x) (idB y)");
    let expected_ty = Term::Eq(Box::new(bool_t(&b)), Box::new(const_(x)), Box::new(const_(y)));
    assert!(
        check(&env, &Context::new(), &const_(h), &expected_ty).is_ok(),
        "h : Eq Bool (idB x) (idB y) must check at Eq Bool x y (both components reconstructed)"
    );
}

/// **AC4 — the over-conversion gate (the real hazard).** A genuinely
/// different endpoint (`True` vs `False`) must stay REJECTED — the
/// congruence arm recurses to a real `conv_struct(True, False)` check that
/// correctly fails; it must not short-circuit to true just because the `a`
/// components happen to be reconstructible.
#[test]
fn eq_congruence_distinct_endpoint_stays_rejected() {
    let (mut env, b) = mk_env();
    let x = declare_postulate(&mut env, "test postulate".to_string(), vec![], bool_t(&b)).expect("x : Bool");
    let h_ty = Term::Eq(
        Box::new(bool_t(&b)),
        Box::new(id_b_app(&b, const_(x))),
        Box::new(true_c(&b)),
    );
    let h = declare_postulate(&mut env, "test postulate".to_string(), vec![], h_ty).expect("h : Eq Bool (idB x) True");
    let bad_ty = Term::Eq(Box::new(bool_t(&b)), Box::new(const_(x)), Box::new(false_c(&b)));
    assert!(
        check(&env, &Context::new(), &const_(h), &bad_ty).is_err(),
        "h : Eq Bool (idB x) True must NOT check at Eq Bool x False (distinct endpoint)"
    );
}
