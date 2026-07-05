//! Smoke test for the recursive-view-through-SCT wiring (Approach A).
//!
//! Validates the contained elaborator extension in isolation, before the full
//! L3a prelude: a test-declared Peano nat + a self-recursive view. Pins:
//!  - the body's self-call resolves (name pre-admitted as Opaque → globals);
//!  - `sct_check` accepts the structural descent (`Suc m` → `m`);
//!  - the def upgrades to transparent (δ-unfoldable, out of `trusted_base`).
//!
//! Not a conformance case (no AC pin); deleted or folded into l3a_acceptance
//! once the prelude lands.

use ken_elaborator::{error::ElabError, ElabEnv};
use ken_kernel::{whnf, Context, GlobalId, Term};

fn mk_env() -> ElabEnv {
    ElabEnv::new().expect("base env")
}

fn elab_ok(env: &mut ElabEnv, src: &str) -> GlobalId {
    env.elaborate_decl(src)
        .unwrap_or_else(|e| panic!("elab_ok failed: {e}"))
}

#[test]
fn recursive_view_self_ref_resolves_and_sct_accepts() {
    let mut env = mk_env();
    // A Peano nat (test-local; the prelude Nat decision is separate).
    elab_ok(&mut env, "data Peano = Z | S Peano");

    // Self-recursive view: `double n = match n { Z => Z ; S m => S (S (double m)) }`.
    // Structural descent on `m` (sub-term of `S m`). SCT must accept.
    let id = elab_ok(
        &mut env,
        "fn double (n : Peano) : Peano = match n { Z => Z ; S m => S (S (double m)) }",
    );

    // The def must be transparent (upgraded after SCT), not an open hole.
    assert!(
        !env.env.trusted_base().contains(&id),
        "recursive const must upgrade to transparent after SCT (not in trusted_base)"
    );
    assert!(
        env.env.transparent_body(id).is_some(),
        "recursive const must have a δ-unfoldable body after SCT upgrade"
    );
}

#[test]
fn recursive_view_reduces_on_constructor() {
    let mut env = mk_env();
    elab_ok(&mut env, "data Peano = Z | S Peano");
    // `pred n = match n { Z => Z ; S m => m }` — non-recursive baseline (sanity).
    elab_ok(
        &mut env,
        "fn pred (n : Peano) : Peano = match n { Z => Z ; S m => m }",
    );
    // `count n = match n { Z => Z ; S m => S (count m) }` — recursive.
    let id = elab_ok(
        &mut env,
        "fn count (n : Peano) : Peano = match n { Z => Z ; S m => S (count m) }",
    );
    // Apply count to (S Z) and whnf: count (S Z) ⇝ S (count Z) ⇝ S Z.
    let body = env.env.transparent_body(id).expect("count body").1;
    // body = Lam(Peano, match n {...}). Apply to the constructor `S Z`.
    let s_id = *env.globals.get("S").expect("S ctor");
    let z_id = *env.globals.get("Z").expect("Z ctor");
    let s_z = Term::app(
        Term::Constructor {
            id: s_id,
            level_args: vec![],
        },
        Term::Constructor {
            id: z_id,
            level_args: vec![],
        },
    );
    let applied = whnf(&env.env, &Context::new(), &Term::app(body, s_z));
    // Should reduce to `S Z` (a constructor application), not stay stuck.
    assert!(
        !matches!(applied, Term::Elim { .. }),
        "count (S Z) should ι-reduce, got {:?}",
        applied
    );
}

#[test]
fn non_terminating_recursive_view_rejected_by_sct() {
    // A recursive const with NO structural descent must be SCT-rejected.
    // `loop (n : Peano) : Peano = loop n` — self-call on the SAME arg (no ↓).
    let mut env = mk_env();
    elab_ok(&mut env, "data Peano = Z | S Peano");
    let result = env.elaborate_decl("fn loopn (n : Peano) : Peano = loopn n");
    assert!(
        matches!(result, Err(ElabError::KernelRejected { .. })),
        "non-terminating recursive const must be SCT-rejected, got {:?}",
        result.err()
    );
    // The failed admission must roll back: name not in globals, no orphan opaque.
    assert!(
        !env.globals.contains_key("loopn"),
        "failed recursive const must unbind its name from globals"
    );
}

// ── polymorphic-type-param regression guards ─────────────────────────────────
// `infer_match` once mishandled a parameterized return type (the IH lambda
// domain was not weakened by the ctor-arg count); these guard the fix.

#[test]
fn poly_id_elaborates() {
    let mut env = mk_env();
    let r = env.elaborate_decl("fn id (a : Type) (x : a) : a = x");
    assert!(
        r.is_ok(),
        "polymorphic id (a : Type) (x : a) : a should elaborate; got {:?}",
        r.err()
    );
}

#[test]
fn poly_id_list_elaborates() {
    let mut env = mk_env();
    let r = env.elaborate_decl("fn idl (a : Type) (xs : List a) : List a = xs");
    assert!(
        r.is_ok(),
        "polymorphic idl (a : Type) (xs : List a) : List a should elaborate; got {:?}",
        r.err()
    );
}

#[test]
fn poly_match_return_elaborates() {
    // A match returning a parameterized type (`Option a`) — the case that
    // exposed the IH-domain weakening bug.
    let mut env = mk_env();
    let r = env.elaborate_decl(
        "fn head (a : Type) (xs : List a) : Option a = match xs { Nil => None a ; Cons h t => Some a h }",
    );
    assert!(
        r.is_ok(),
        "polymorphic match returning Option a should elaborate; got {:?}",
        r.err()
    );
}
