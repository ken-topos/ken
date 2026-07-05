//! CAT-1 D2 package checks for the landed `Semigroup`/`Monoid` source.
//! This loads the real package files through the production elaborator path.

use ken_elaborator::ElabEnv;
use ken_kernel::Term;

const COLLECTIONS_KEN: &str = include_str!("../../../packages/collections/collections.ken");
const TRANSPORT_KEN: &str = include_str!("../../../packages/transport/transport.ken");
const LAWFUL_FUNCTORS_KEN: &str =
    include_str!("../../../packages/lawful-functors/lawful_functors.ken");

fn mk_env_with_lawful_functors() -> ElabEnv {
    let mut env = ElabEnv::new().expect("base env construction failed");
    env.elaborate_file(COLLECTIONS_KEN)
        .expect("packages/collections/collections.ken must elaborate");
    env.elaborate_file(TRANSPORT_KEN)
        .expect("packages/transport/transport.ken must elaborate");
    env.elaborate_file(LAWFUL_FUNCTORS_KEN)
        .expect("packages/lawful-functors/lawful_functors.ken must elaborate");
    env
}

#[test]
fn lawful_functors_package_elaborates_with_parametric_list_monoid() {
    let env = mk_env_with_lawful_functors();

    let instance_id = env
        .globals
        .get("Monoid_instance_List")
        .copied()
        .expect("Monoid (List a) should register by bare List head");
    let (_, ty) = env
        .env
        .const_type(instance_id)
        .expect("registered instance should have a kernel type");

    assert!(
        matches!(ty, Term::Pi(_, _)),
        "Monoid (List a) should elaborate as a Pi-typed generic dictionary, got {ty:?}"
    );
    assert!(
        env.class_env.instance_search("Monoid", "List").is_some(),
        "coherence key should be Monoid/List, not a closed element type"
    );
}

#[test]
fn lawful_functors_source_cites_generic_list_proofs_without_axiom() {
    assert!(
        LAWFUL_FUNCTORS_KEN.contains("instance Monoid (List a)"),
        "package must use the parametric List Monoid instance head"
    );
    assert!(
        !LAWFUL_FUNCTORS_KEN.contains("instance Monoid (List Nat)"),
        "package must not leave the old closed List Nat Monoid instance"
    );
    assert!(
        LAWFUL_FUNCTORS_KEN.contains("assoc      = list_assoc a")
            && LAWFUL_FUNCTORS_KEN.contains("left_unit  = list_left_unit a")
            && LAWFUL_FUNCTORS_KEN.contains("right_unit = list_right_unit a"),
        "law fields should cite the existing generic List proofs"
    );
    assert!(
        !LAWFUL_FUNCTORS_KEN.contains("= Axiom"),
        "lawful-functors package must not fill laws with Axiom"
    );
}
