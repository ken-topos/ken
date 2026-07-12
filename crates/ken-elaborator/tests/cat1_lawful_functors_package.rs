//! CAT-1 package checks for the landed lawful-functors source.
//! This loads the real package files through the production elaborator path.

use ken_elaborator::ElabEnv;
use ken_kernel::Term;

const COLLECTIONS_KEN_MD: &str = include_str!("../../../catalog/packages/Data/Collections/Collections.ken.md");
const TRANSPORT_KEN_MD: &str = include_str!("../../../catalog/packages/Core/Transport.ken.md");
const LAWFUL_FUNCTORS_KEN_MD: &str =
    include_str!("../../../catalog/packages/Core/LawfulFunctors.ken.md");

fn mk_env_with_lawful_functors() -> ElabEnv {
    let mut env = ElabEnv::new().expect("base env construction failed");
    env.elaborate_ken_md_file(TRANSPORT_KEN_MD)
        .expect("catalog/packages/Core/Transport.ken must elaborate");
    env.elaborate_ken_md_file(COLLECTIONS_KEN_MD)
        .expect("catalog/packages/Data/Collections/Collections.ken.md must elaborate");
    env.elaborate_ken_md_file(LAWFUL_FUNCTORS_KEN_MD)
        .expect("catalog/packages/Core/LawfulFunctors.ken.md must elaborate");
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

    for (class, head, global) in [
        ("Functor", "List", "Functor_instance_List"),
        ("Functor", "Option", "Functor_instance_Option"),
        ("Foldable", "List", "Foldable_instance_List"),
        ("Foldable", "Option", "Foldable_instance_Option"),
    ] {
        let instance_id = env
            .globals
            .get(global)
            .copied()
            .unwrap_or_else(|| panic!("{global} should be registered"));
        env.env
            .const_type(instance_id)
            .unwrap_or_else(|| panic!("{global} should have a kernel type"));
        assert!(
            env.class_env.instance_search(class, head).is_some(),
            "coherence key should include {class}/{head}"
        );
    }
}

#[test]
fn lawful_functors_source_cites_landed_laws_without_axiom() {
    assert!(
        LAWFUL_FUNCTORS_KEN_MD.contains("instance Monoid (List a)"),
        "package must use the parametric List Monoid instance head"
    );
    assert!(
        !LAWFUL_FUNCTORS_KEN_MD.contains("instance Monoid (List Nat)"),
        "package must not leave the old closed List Nat Monoid instance"
    );
    assert!(
        LAWFUL_FUNCTORS_KEN_MD.contains("assoc      = proof assoc for list_append a")
            && LAWFUL_FUNCTORS_KEN_MD
                .contains("left_unit  = proof left_unit for list_append a")
            && LAWFUL_FUNCTORS_KEN_MD
                .contains("right_unit = proof right_unit for list_append a"),
        "law fields should cite the existing generic List proofs"
    );
    assert!(
        !LAWFUL_FUNCTORS_KEN_MD.contains("= Axiom"),
        "lawful-functors package must not fill laws with Axiom"
    );
    assert!(
        LAWFUL_FUNCTORS_KEN_MD.contains("class Functor (f : Type → Type)")
            && LAWFUL_FUNCTORS_KEN_MD.contains("id_law     : (a : Type) → (x : f a)")
            && LAWFUL_FUNCTORS_KEN_MD.contains("fusion_law : (a : Type) → (b : Type) → (c : Type)")
            && LAWFUL_FUNCTORS_KEN_MD.contains("(g : b → c) → (h : a → b) → (x : f a)"),
        "Functor should use the settled single pointwise law fields"
    );
    assert!(
        !LAWFUL_FUNCTORS_KEN_MD.contains("map_id")
            && !LAWFUL_FUNCTORS_KEN_MD.contains("map_comp")
            && !LAWFUL_FUNCTORS_KEN_MD.contains("pointfree"),
        "Functor should not add a point-free duplicate law surface"
    );
    assert!(
        LAWFUL_FUNCTORS_KEN_MD.contains("instance Functor List")
            && LAWFUL_FUNCTORS_KEN_MD.contains("instance Functor Option")
            && LAWFUL_FUNCTORS_KEN_MD.contains("instance Foldable List")
            && LAWFUL_FUNCTORS_KEN_MD.contains("instance Foldable Option"),
        "D3 should provide List and Option Functor/Foldable instances"
    );
    assert!(
        LAWFUL_FUNCTORS_KEN_MD.contains("fold_map_coherence")
            && LAWFUL_FUNCTORS_KEN_MD.contains("fold_map_step")
            && LAWFUL_FUNCTORS_KEN_MD.contains("foldr_to_list"),
        "Foldable should pin fold_map through the selected Monoid and to_list reconstruction laws"
    );
}
