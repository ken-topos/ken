//! Constrained instance dictionaries bind in instance fields and are applied
//! recursively when a concrete use-site resolves the instance.

use ken_elaborator::{error::ElabError, ElabEnv, RType};
use ken_kernel::{Decl, Term};

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
    let instance_id = elab(&mut env, source).unwrap();

    let info = env
        .class_env
        .instances
        .get(&("Pick".to_string(), "Pair".to_string()))
        .expect("the constrained Pair instance must be registered");
    assert_eq!(info.head_param_count, 2, "Pair abstracts a then b");
    assert!(matches!(
        info.constraints.as_slice(),
        [first, second]
            if matches!(first.head_type, RType::RVarTy(1, ref name, _) if name == "a")
                && matches!(second.head_type, RType::RVarTy(0, ref name, _) if name == "b")
    ));
    let declaration = env.env.lookup(instance_id).unwrap();
    let (instance_ty, instance_body) = match declaration {
        Decl::Transparent { ty, body, .. } => (ty, body),
        other => panic!("instance must be a transparent kernel-checked definition: {other:?}"),
    };
    assert_eq!(
        pi_count(instance_ty),
        4,
        "type parameters precede two dictionary Pis"
    );
    assert_eq!(
        lam_count(instance_body),
        4,
        "value closes over the same four binders"
    );

    let use_site = elab(
        &mut env,
        "const pickedPair : Pair Int Bool where Pick (Pair Int Bool) = d.select",
    );
    assert!(
        use_site.is_ok(),
        "two-constraint instance must recursively resolve and kernel-check at its use site: {use_site:?}"
    );
}

fn pi_count(term: &Term) -> usize {
    match term {
        Term::Pi(_, body) => 1 + pi_count(body),
        _ => 0,
    }
}

fn lam_count(term: &Term) -> usize {
    match term {
        Term::Lam(_, body) => 1 + lam_count(body),
        _ => 0,
    }
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

    let unique_names = elab(
        &mut env,
        "instance Goal (Pair a b) where (qa : Q a), (rb : R b) { }",
    );
    assert!(
        unique_names.is_ok(),
        "pairwise-distinct explicit names must accept: {unique_names:?}"
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
fn compound_named_and_bare_d_boundaries_are_discriminating() {
    let mut env = ElabEnv::new().unwrap();
    elab(&mut env, "class Wrap A { select : A }").unwrap();
    elab(&mut env, "class Holder A { select : A }").unwrap();
    elab(&mut env, "class Multi A { select : A }").unwrap();

    let compound = elab(
        &mut env,
        "instance Holder (List a) where (wa : Wrap (List a)) { select = wa.select }",
    );
    assert!(
        compound.is_ok(),
        "a compound constraint with an explicit name must bind and project: {compound:?}"
    );
    elab(&mut env, "instance Wrap (List Bool) { select = Nil Bool }").unwrap();
    let compound_use = elab(
        &mut env,
        "const held : List Bool where Holder (List Bool) = d.select",
    );
    assert!(
        compound_use.is_ok(),
        "the explicit compound dictionary must resolve recursively at use: {compound_use:?}"
    );

    let bare_d_multi = elab(
        &mut env,
        "instance Multi (Pair a b) where Wrap a, Wrap b { select = d.select }",
    );
    assert!(
        matches!(bare_d_multi, Err(ElabError::UnresolvedCon { ref name, .. }) if name == "d"),
        "bare d must be unbound with multiple constraints: {bare_d_multi:?}"
    );
}

#[test]
fn independently_named_ill_typed_field_is_kernel_rejected() {
    let mut env = ElabEnv::new().unwrap();
    elab(&mut env, "class Pick A { select : A }").unwrap();
    let ill_typed = elab(
        &mut env,
        "instance Pick (Pair a b) where Pick a, Pick b { select = True }",
    );
    assert!(
        matches!(ill_typed, Err(ElabError::KernelRejected { .. })),
        "an ill-typed field must be rejected by the kernel-checked field path: {ill_typed:?}"
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
