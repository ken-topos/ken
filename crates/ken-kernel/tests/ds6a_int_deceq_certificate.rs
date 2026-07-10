//! DS-6a conformance — the decidable-equality certificate mechanism (ADR
//! 0013 Layer 1): a general, opt-in per-primitive registration by which an
//! opaque primitive type may admit a kernel-audited `eq`-decides-equality
//! certificate, split sound + complete. This is scratch/minimal machinery
//! (a stand-in primitive `PrimT`, not `Int`) so the mechanism itself is
//! exercised independent of the full numeric tower; `Int`'s own registration
//! is exercised at the elaborator layer (`ds6a_int_deceq_acceptance.rs`).

use ken_kernel::env::{Context, Decl, PrimReduction};
use ken_kernel::term::{Level, Term};
use ken_kernel::{
    declare_deceq_certificate, declare_inductive, declare_primitive, infer, CtorSpec, GlobalEnv,
    GlobalId, InductiveSpec, KernelError,
};
use std::collections::BTreeSet;

/// Minimal env: `Bool` (`True | False`) + an opaque primitive `PrimT`.
struct Env0 {
    prim_t: GlobalId,
    bool_: GlobalId,
    true_: GlobalId,
}

fn mk_env() -> (GlobalEnv, Env0) {
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
    let true_ = env.inductive(bool_).unwrap().constructors[0].id;

    let prim_t = declare_primitive(
        &mut env,
        vec![],
        Term::Type(Level::zero()),
        PrimReduction::OpaqueType,
    )
    .expect("PrimT");

    (env, Env0 { prim_t, bool_, true_ })
}

fn eq_op_of_type(env: &mut GlobalEnv, ty: Term) -> GlobalId {
    declare_primitive(env, vec![], ty, PrimReduction::Op { symbol: "eq_test" })
        .expect("eq_op registration")
}

/// A well-shaped `eq_op : PrimT -> PrimT -> Bool` registers cleanly, and the
/// resulting `sound`/`complete` postulates have exactly the certificate's
/// Pi-chain type (round-tripped through `infer`, confirming the constructed
/// terms are themselves well-typed, not just accepted by construction).
#[test]
fn well_shaped_registration_succeeds_and_types_round_trip() {
    let (mut env, e) = mk_env();
    let prim_const = Term::const_(e.prim_t, vec![]);
    let bool_t = Term::indformer(e.bool_, vec![]);
    let eq_op = eq_op_of_type(
        &mut env,
        Term::pi(prim_const.clone(), Term::pi(prim_const, bool_t)),
    );

    let cert = declare_deceq_certificate(&mut env, e.prim_t, eq_op, e.bool_, e.true_)
        .expect("well-shaped eq_op registers");

    assert_eq!(cert.eq_op, eq_op);

    // The certificate is recorded under `prim_t`.
    let looked_up = env.deceq_cert(e.prim_t).expect("cert recorded");
    assert_eq!(looked_up.sound, cert.sound);
    assert_eq!(looked_up.complete, cert.complete);

    // `sound`/`complete` are themselves well-typed constants (infer succeeds
    // — trivially true since `declare_postulate` already checked them, but
    // this confirms the stored `Decl::Opaque` is reachable and consistent).
    let ctx = Context::new();
    infer(&env, &ctx, &Term::const_(cert.sound, vec![])).expect("sound infers");
    infer(&env, &ctx, &Term::const_(cert.complete, vec![])).expect("complete infers");
}

/// Zero-Axiom-delta discriminator (DS-2-style before/after set-diff):
/// registering the certificate adds **exactly** the two certificate
/// postulates (`sound`, `complete`) to `trusted_base()` — no more, no less.
#[test]
fn trusted_base_delta_is_exactly_the_two_certificate_postulates() {
    let (mut env, e) = mk_env();
    let prim_const = Term::const_(e.prim_t, vec![]);
    let bool_t = Term::indformer(e.bool_, vec![]);
    let eq_op = eq_op_of_type(
        &mut env,
        Term::pi(prim_const.clone(), Term::pi(prim_const, bool_t)),
    );

    let before: BTreeSet<GlobalId> = env.trusted_base().into_iter().collect();
    let cert = declare_deceq_certificate(&mut env, e.prim_t, eq_op, e.bool_, e.true_)
        .expect("well-shaped eq_op registers");
    let after: BTreeSet<GlobalId> = env.trusted_base().into_iter().collect();

    let added: BTreeSet<GlobalId> = after.difference(&before).copied().collect();
    let expected: BTreeSet<GlobalId> = [cert.sound, cert.complete].into_iter().collect();
    assert_eq!(added, expected);

    // Both new trusted entries are genuinely `Decl::Opaque` postulates (the
    // honest "trusted, no computation" shape), not primitives or data.
    for id in [cert.sound, cert.complete] {
        assert!(matches!(env.lookup(id), Some(Decl::Opaque { .. })));
    }
}

/// **Required hardening (Architect gate):** registering with an `eq_op`
/// whose domain does not match `prim_ty` is rejected **at registration**
/// (`declare_deceq_certificate` returns `Err` before `add_decl` ever runs),
/// not silently minted as an incoherent trusted postulate. Assert the
/// specific `KernelError` variant, not bare `is_err()`.
#[test]
fn mistyped_eq_op_domain_is_rejected_at_registration() {
    let (mut env, e) = mk_env();
    let bool_t = Term::indformer(e.bool_, vec![]);
    // `eq_op : Bool -> Bool -> Bool` — registered against `PrimT`, so the
    // certificate's `x : PrimT` argument does not check against `eq_op`'s
    // actual `Bool` domain.
    let eq_op = eq_op_of_type(
        &mut env,
        Term::pi(bool_t.clone(), Term::pi(bool_t.clone(), bool_t)),
    );

    let before: BTreeSet<GlobalId> = env.trusted_base().into_iter().collect();
    let err = declare_deceq_certificate(&mut env, e.prim_t, eq_op, e.bool_, e.true_)
        .expect_err("mistyped eq_op must be rejected, not admitted");
    assert!(
        matches!(err, KernelError::TypeMismatch { .. }),
        "expected TypeMismatch, got {:?}",
        err
    );

    // Fail-closed: no partial postulate was minted (the rejected `sound_ty`
    // or a stray entry would show up as a trusted_base growth).
    let after: BTreeSet<GlobalId> = env.trusted_base().into_iter().collect();
    assert_eq!(before, after);
    assert!(env.deceq_cert(e.prim_t).is_none());
}

/// **Required hardening, arity variant:** an `eq_op` that isn't even
/// binary (wrong shape, not just wrong domain) is likewise rejected at
/// registration with a specific variant (`NotAFunction`, since the
/// certificate's second application has nothing left to apply to).
#[test]
fn wrong_arity_eq_op_is_rejected_at_registration() {
    let (mut env, e) = mk_env();
    let prim_const = Term::const_(e.prim_t, vec![]);
    let bool_t = Term::indformer(e.bool_, vec![]);
    // `eq_op : PrimT -> Bool` — unary, missing the second `PrimT` argument.
    let eq_op = eq_op_of_type(&mut env, Term::pi(prim_const, bool_t));

    let before: BTreeSet<GlobalId> = env.trusted_base().into_iter().collect();
    let err = declare_deceq_certificate(&mut env, e.prim_t, eq_op, e.bool_, e.true_)
        .expect_err("wrong-arity eq_op must be rejected, not admitted");
    assert!(
        matches!(err, KernelError::NotAFunction { .. }),
        "expected NotAFunction, got {:?}",
        err
    );

    let after: BTreeSet<GlobalId> = env.trusted_base().into_iter().collect();
    assert_eq!(before, after);
    assert!(env.deceq_cert(e.prim_t).is_none());
}

/// **Neutral preserved:** registering a certificate touches no reduction
/// machinery — an *unregistered* primitive's `Eq` stays exactly as
/// unreachable as before (there is simply no certificate to look up), and
/// this mechanism never mutates `obs.rs`. Regression pin: an unregistered
/// second primitive has no certificate even after `PrimT` is registered.
#[test]
fn unregistered_primitive_has_no_certificate() {
    let (mut env, e) = mk_env();
    let prim_const = Term::const_(e.prim_t, vec![]);
    let bool_t = Term::indformer(e.bool_, vec![]);
    let eq_op = eq_op_of_type(
        &mut env,
        Term::pi(prim_const.clone(), Term::pi(prim_const, bool_t.clone())),
    );
    declare_deceq_certificate(&mut env, e.prim_t, eq_op, e.bool_, e.true_)
        .expect("well-shaped eq_op registers");

    let other_prim = declare_primitive(
        &mut env,
        vec![],
        Term::Type(Level::zero()),
        PrimReduction::OpaqueType,
    )
    .expect("second PrimT-like primitive");
    assert!(env.deceq_cert(other_prim).is_none());
    // `Bool` itself (never registered) also has none.
    assert!(env.deceq_cert(e.bool_).is_none());
}
