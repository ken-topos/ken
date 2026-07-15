//! CC1 (`Data.Collections.NonEmpty` + `Data.Sums.Validation`) acceptance —
//! `docs/program/wp/cc1-nonempty-validation.md`.
//!
//! These packages depend on the existing catalog rather than the bare
//! prelude.  Match the catalog's established DS-7/DS-8 validation model:
//! elaborate the dependency closure in order into one shared `ElabEnv`, then
//! elaborate both CC1 entries (including every checked literate fence).

use std::collections::BTreeSet;

use ken_elaborator::ElabEnv;

const TRANSPORT_KEN_MD: &str = include_str!("../../../catalog/packages/Core/Logic/Transport.ken.md");
const COLLECTIONS_KEN_MD: &str =
    include_str!("../../../catalog/packages/Data/Collections/Derived.ken.md");
const LAWFUL_CLASSES_KEN_MD: &str =
    include_str!("../../../catalog/packages/Core/Classes/LawfulClasses.ken.md");
const LAWFUL_FUNCTORS_KEN_MD: &str =
    include_str!("../../../catalog/packages/Core/Classes/LawfulFunctors.ken.md");
const EFFECTFUL_CLASSES_KEN_MD: &str =
    include_str!("../../../catalog/packages/Core/Classes/EffectfulClasses.ken.md");
const NONEMPTY_KEN_MD: &str =
    include_str!("../../../catalog/packages/Data/Collections/NonEmpty.ken.md");
const VALIDATION_KEN_MD: &str =
    include_str!("../../../catalog/packages/Data/Sums/Validation.ken.md");

fn dependency_env() -> ElabEnv {
    let mut env = ElabEnv::empty().expect("prelude bootstrap");
    env.elaborate_ken_md_file(TRANSPORT_KEN_MD)
        .expect("Core/Logic/Transport.ken.md must elaborate first");
    env.elaborate_ken_md_file(COLLECTIONS_KEN_MD)
        .expect("Data/Collections/Derived.ken.md must elaborate second");
    env.elaborate_ken_md_file(LAWFUL_CLASSES_KEN_MD)
        .expect("Core/Classes/LawfulClasses.ken.md must elaborate third");
    env.elaborate_ken_md_file(LAWFUL_FUNCTORS_KEN_MD)
        .expect("Core/Classes/LawfulFunctors.ken.md must elaborate fourth");
    env.elaborate_ken_md_file(EFFECTFUL_CLASSES_KEN_MD)
        .expect("Core/Classes/EffectfulClasses.ken.md must elaborate fifth");
    env
}

fn assert_transparent_globals(env: &ElabEnv, names: &[&str]) {
    for name in names {
        let id = *env
            .globals
            .get(*name)
            .unwrap_or_else(|| panic!("expected checked global `{name}`"));
        assert!(
            env.env.transparent_body(id).is_some(),
            "`{name}` must be a real transparent, kernel-checked term"
        );
    }
}

#[test]
fn ordered_dependency_closure_elaborates_both_packages_and_all_laws() {
    let mut env = dependency_env();

    env.elaborate_ken_md_file(NONEMPTY_KEN_MD)
        .expect("Data/Collections/NonEmpty.ken.md and every checked fence must elaborate");
    assert_transparent_globals(
        &env,
        &[
            "nonempty_head",
            "nonempty_tail",
            "nonempty_to_list",
            "nonempty_map",
            "nonempty_append",
            "nonempty_append::assoc",
            "Semigroup_instance_NonEmpty",
        ],
    );

    env.elaborate_ken_md_file(VALIDATION_KEN_MD)
        .expect("Data/Sums/Validation.ken.md and every checked fence must elaborate");
    assert_transparent_globals(
        &env,
        &[
            "validation_map",
            "validation_pure",
            "validation_ap",
            "validation_map::id",
            "validation_map::fusion",
            "validation_ap_id",
            "validation_ap_hom",
            "validation_ap_ich",
            "validation_ap_cmp",
            "validation_map_coh",
            "Functor_instance_Validation",
            "Applicative_instance_Validation",
        ],
    );

    let extracted = ken_elaborator::literate::extract_ken_md(VALIDATION_KEN_MD)
        .expect("Validation.ken.md must extract");
    assert!(
        extracted.example_ranges.iter().any(|range| {
            let example = &VALIDATION_KEN_MD[range.clone()];
            example.contains("checked_record")
                && example.contains("expected_errors")
                && example.contains("both_errors_accumulate")
        }),
        "the checked example must prove that both independent errors accumulate"
    );
}

#[test]
fn cc1_checked_code_has_zero_axiom_and_zero_trusted_base_delta() {
    for (name, source) in [
        ("NonEmpty.ken.md", NONEMPTY_KEN_MD),
        ("Validation.ken.md", VALIDATION_KEN_MD),
    ] {
        let extracted =
            ken_elaborator::literate::extract_ken_md(source).expect("CC1 source must extract");
        assert!(
            !extracted.source.contains("Axiom"),
            "{name}'s tangled checked code must contain no Axiom"
        );
        for range in extracted
            .example_ranges
            .iter()
            .chain(extracted.reject_ranges.iter())
        {
            assert!(
                !source[range.clone()].contains("Axiom"),
                "{name}'s checked example/reject fences must contain no Axiom"
            );
        }
    }

    let mut env = dependency_env();
    let before: BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    env.elaborate_ken_md_file(NONEMPTY_KEN_MD)
        .expect("NonEmpty.ken.md must elaborate");
    env.elaborate_ken_md_file(VALIDATION_KEN_MD)
        .expect("Validation.ken.md must elaborate");
    let after: BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    assert_eq!(
        before, after,
        "CC1 must add no primitive, opaque constant, postulate, or Axiom"
    );
}

#[test]
fn validation_deliberately_has_no_monad_instance() {
    let extracted = ken_elaborator::literate::extract_ken_md(VALIDATION_KEN_MD)
        .expect("Validation.ken.md must extract");
    assert!(
        !extracted.source.contains("instance Monad (Validation"),
        "Validation accumulates independent errors and must not define Monad"
    );
}
