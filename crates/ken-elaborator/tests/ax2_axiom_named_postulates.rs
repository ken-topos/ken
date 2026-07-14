//! AX-2: declaration sugar and readable postulate provenance.

use std::collections::BTreeSet;

use ken_elaborator::layout::format_ken;
use ken_elaborator::parser::parse_decls;
use ken_elaborator::{Decl, ElabEnv};

#[test]
fn axiom_surface_parses_formats_and_elaborates_as_a_named_postulate() {
    let source = "axiom assumed_top : Top";
    let declarations = parse_decls(source).expect("axiom declaration parses");
    assert!(matches!(
        declarations.as_slice(),
        [Decl::AxiomDecl { name, .. }] if name == "assumed_top"
    ));
    assert_eq!(
        format_ken(source).expect("axiom declaration formats"),
        "axiom assumed_top : Top\n"
    );

    let mut env = ElabEnv::new().expect("base environment builds");
    env.elaborate_file(source)
        .expect("axiom declaration elaborates through the lemma lane");
    let entries = env.env.trusted_base();
    assert!(
        entries
            .entries()
            .iter()
            .any(|entry| entry.name == "assumed_top"),
        "trusted-base audit must expose the declared axiom name"
    );
}

#[test]
fn repeated_expression_axioms_share_the_owner_label_but_not_identity() {
    let mut env = ElabEnv::new().expect("base environment builds");
    env.elaborate_file(
        "lemma choose (x : Top) (y : Top) : Top = x\n\
         lemma shared : Top = choose Axiom Axiom",
    )
    .expect("both expression-position Axiom terms elaborate");

    let entries = env.env.trusted_base();
    let shared = entries
        .entries()
        .iter()
        .filter(|entry| entry.name == "shared")
        .collect::<Vec<_>>();
    assert_eq!(shared.len(), 2, "both Axiom occurrences retain provenance");
    assert_eq!(
        shared.iter().map(|entry| entry.id).collect::<BTreeSet<_>>().len(),
        2,
        "shared provenance is not shared identity"
    );
}

#[test]
fn instance_field_axiom_uses_the_canonical_owner_path() {
    let mut env = ElabEnv::new().expect("base environment builds");
    env.elaborate_file(
        "class Witness A { evidence : Top }\n\
         instance Witness Int { evidence = Axiom }",
    )
    .expect("instance field Axiom elaborates");

    let entries = env.env.trusted_base();
    assert!(
        entries
            .entries()
            .iter()
            .any(|entry| entry.name == "Witness.Int.evidence"),
        "instance-field provenance must be Class.HeadType.field"
    );
}

#[test]
fn standalone_api_requires_and_preserves_its_caller_owner() {
    let mut env = ElabEnv::new().expect("base environment builds");
    env.elaborate_expr("standalone_assumption", "Axiom : Top")
        .expect("checking-mode Axiom remains legal through the standalone API");
    assert!(
        env.env
            .trusted_base()
            .entries()
            .iter()
            .any(|entry| entry.name == "standalone_assumption")
    );

    env.elaborate_file("lemma choose_api (x : Top) (y : Top) : Top = x")
        .expect("standalone fixture helper elaborates");
    env.elaborate_expr(
        "standalone_shared_owner",
        "choose_api Axiom Axiom",
    )
    .expect("both standalone Axiom operands elaborate");
    let trusted = env.env.trusted_base();
    let shared = trusted
        .entries()
        .iter()
        .filter(|entry| entry.name == "standalone_shared_owner")
        .collect::<Vec<_>>();
    assert_eq!(shared.len(), 2);
    assert_ne!(shared[0].id, shared[1].id);
}

#[test]
fn module_qualification_is_the_axiom_owner() {
    let mut env = ElabEnv::new().expect("base environment builds");
    env.elaborate_file("module Claims { pub axiom admitted : Top }")
        .expect("module-owned axiom elaborates");
    assert!(
        env.env
            .trusted_base()
            .entries()
            .iter()
            .any(|entry| entry.name == "Claims.admitted")
    );
}
