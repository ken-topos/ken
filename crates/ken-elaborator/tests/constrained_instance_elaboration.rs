//! Constrained instance dictionaries bind in instance fields and are applied
//! recursively when a concrete use-site resolves the instance.

use ken_elaborator::{error::ElabError, ElabEnv};

fn elab(env: &mut ElabEnv, source: &str) -> Result<ken_kernel::GlobalId, ElabError> {
    env.elaborate_decl(source)
}

#[test]
fn constrained_pair_instance_binds_two_dicts_and_resolves_at_use_site() {
    let mut env = ElabEnv::new().unwrap();
    elab(&mut env, "class Pick A { select : A }").unwrap();
    elab(&mut env, "instance Pick Int { select = 0 }").unwrap();
    elab(&mut env, "instance Pick Bool { select = True }").unwrap();
    let source = "instance Pick (Pair a b) where Pick a, Pick b { \
         select = mk_pair a b da.select db.select \
         }";
    elab(&mut env, source).unwrap();

    let use_site = elab(
        &mut env,
        "const pickedPair : Pair Int Bool where Pick (Pair Int Bool) = d.select",
    );
    assert!(
        use_site.is_ok(),
        "two-constraint instance must recursively resolve and kernel-check at its use site: {use_site:?}"
    );
}

#[test]
fn constraint_names_are_deterministic_and_surface_misuse_rejects() {
    let mut env = ElabEnv::new().unwrap();
    elab(&mut env, "class Single A { selected : A }").unwrap();
    elab(&mut env, "instance Single Bool { selected = True }").unwrap();
    elab(
        &mut env,
        "instance Single (Pair a Bool) where Single a { \
         selected = mk_pair a Bool d.selected True \
         }",
    )
    .unwrap();
    let single_use = elab(
        &mut env,
        "const selectedPair : Pair Bool Bool where Single (Pair Bool Bool) = d.selected",
    );
    assert!(
        single_use.is_ok(),
        "the sole-constraint d alias must resolve: {single_use:?}"
    );

    elab(&mut env, "class Q A { }").unwrap();
    elab(&mut env, "class R A { }").unwrap();
    elab(&mut env, "class Goal A { }").unwrap();

    let collision = elab(&mut env, "instance Goal (List a) where Q a, R a { }");
    assert!(
        matches!(collision, Err(ElabError::ParseError { .. })),
        "same-variable automatic names must reject as ambiguous: {collision:?}"
    );
    let named_collision = elab(
        &mut env,
        "instance Goal (List a) where (qa : Q a), (ra : R a) { }",
    );
    assert!(
        named_collision.is_ok(),
        "distinct explicit dictionary binders must accept: {named_collision:?}"
    );

    let duplicate_name = elab(
        &mut env,
        "instance Goal (Pair a b) where (d : Q a), (d : R b) { }",
    );
    assert!(
        matches!(duplicate_name, Err(ElabError::ParseError { .. })),
        "duplicate explicit dictionary names must reject: {duplicate_name:?}"
    );

    let compound_unnamed = elab(&mut env, "instance Goal (List a) where Q (List a) { }");
    assert!(
        matches!(compound_unnamed, Err(ElabError::ParseError { .. })),
        "compound constraint arguments require an explicit binder: {compound_unnamed:?}"
    );
}

#[test]
fn swapped_constraint_names_fail_closed() {
    let mut env = ElabEnv::new().unwrap();
    elab(&mut env, "class Pick A { select : A }").unwrap();
    let swapped = elab(
        &mut env,
        "instance Pick (Pair a b) where Pick a, Pick b { \
         select = mk_pair a b db.select da.select \
         }",
    );
    assert!(
        matches!(swapped, Err(ElabError::TypeMismatch { .. }) | Err(ElabError::KernelRejected { .. })),
        "a source-order swap must fail closed rather than admit a mismatched dictionary: {swapped:?}"
    );
}
