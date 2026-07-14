//! K5 follow-on — `Term::Absurd` in `trusted_base_delta`'s dependency walk.
//!
//! `foreign::collect_consts_in_tb` (`38 §3.1`'s dependency-cone walker) is a
//! purely structural traversal with one arm per `Term` variant, same shape as
//! `ken-kernel`'s `sct.rs::collect_calls`/`term.rs::children()`. K5 added
//! `Term::Absurd(motive, proof)` to the kernel's `Term` enum but this
//! downstream crate's own exhaustive match over `Term` wasn't in K5's own
//! (kernel-only) review scope — CI caught it as a build break, and Architect
//! separately named it the same class of hole as K5's own SCT-launder issue:
//! a definition whose only reference to a `trusted_base()` postulate sits
//! inside an `Absurd` subterm would be silently excluded from its
//! `trusted_base_delta`, undercounting the TCB dependency cone.
//!
//! Mirrors `l7_acceptance.rs`'s B1/B2 dependency-pair pattern, using the
//! kernel API directly (structural test — `trusted_base_delta` doesn't
//! type-check the body, so the injected `Absurd` terms need not themselves
//! be well-typed, same license L7 takes with its `Decl::Transparent` bodies).

use ken_elaborator::trusted_base_delta;
use ken_kernel::{declare_postulate, Decl, GlobalEnv, Level, Term};

/// A definition whose only reference to a trusted-base postulate sits in
/// `Absurd`'s **proof** position (the shape from K5's own SCT-launder test,
/// `loop : Bottom := absurd(Bottom, loop)`) must still count it.
#[test]
fn absurd_proof_position_counted_in_trusted_base_delta() {
    let mut env = GlobalEnv::new();
    let p = declare_postulate(&mut env, "test postulate".to_string(), vec![], Term::Omega(Level::zero())).expect("postulate p");

    let def_id = env.fresh_id();
    env.add_decl(Decl::Transparent {
        id: def_id,
        level_params: vec![],
        ty: Term::Omega(Level::zero()),
        body: Term::Absurd(
            Box::new(Term::Omega(Level::zero())), // motive — no reference to p
            Box::new(Term::const_(p, vec![])),    // proof — the only reference to p
        ),
    });

    let delta = trusted_base_delta(&env, def_id);
    assert!(
        delta.contains(&p),
        "a postulate referenced only in Absurd's proof position must appear in trusted_base_delta"
    );
}

/// Same, but the reference sits in the **motive** position instead — both
/// subterms must be walked, not just one.
#[test]
fn absurd_motive_position_counted_in_trusted_base_delta() {
    let mut env = GlobalEnv::new();
    let p = declare_postulate(&mut env, "test postulate".to_string(), vec![], Term::Omega(Level::zero())).expect("postulate p");

    let def_id = env.fresh_id();
    env.add_decl(Decl::Transparent {
        id: def_id,
        level_params: vec![],
        ty: Term::Omega(Level::zero()),
        body: Term::Absurd(
            Box::new(Term::const_(p, vec![])), // motive — the only reference to p
            Box::new(Term::Omega(Level::zero())), // proof — no reference to p
        ),
    });

    let delta = trusted_base_delta(&env, def_id);
    assert!(
        delta.contains(&p),
        "a postulate referenced only in Absurd's motive position must appear in trusted_base_delta"
    );
}

/// Discriminant: a definition whose `Absurd` subterms reference NO postulate
/// has an empty delta — the positive tests above aren't vacuously true
/// (`trusted_base_delta` isn't just returning every postulate in scope).
#[test]
fn absurd_with_no_postulate_reference_has_empty_delta() {
    let mut env = GlobalEnv::new();
    let p = declare_postulate(&mut env, "test postulate".to_string(), vec![], Term::Omega(Level::zero())).expect("postulate p");

    let def_id = env.fresh_id();
    env.add_decl(Decl::Transparent {
        id: def_id,
        level_params: vec![],
        ty: Term::Omega(Level::zero()),
        body: Term::Absurd(
            Box::new(Term::Omega(Level::zero())),
            Box::new(Term::Omega(Level::zero())),
        ),
    });

    let delta = trusted_base_delta(&env, def_id);
    assert!(
        !delta.contains(&p),
        "an Absurd term with no reference to p must not count it — being declared ≠ being reached"
    );
}
