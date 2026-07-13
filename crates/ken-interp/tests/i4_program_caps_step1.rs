//! I-4B Step 1: the checked, authority-parametric `ProgramCaps` record.

use ken_kernel::{Decl, Level, Term};

#[test]
fn program_caps_is_authority_parametric_and_adds_zero_trust() {
    let mut elab = ken_elaborator::ElabEnv::new().expect("prelude registers");
    let auth_id = *elab.globals.get("Auth").expect("Auth registered");
    let cap_id = *elab.globals.get("Cap").expect("Cap registered");
    let program_caps_id = *elab
        .globals
        .get("ProgramCaps")
        .expect("ProgramCaps registered");
    let constructor_id = *elab
        .globals
        .get("MkProgramCaps")
        .expect("MkProgramCaps registered");

    let inductive = match elab.env.lookup(program_caps_id) {
        Some(Decl::Inductive(inductive)) => inductive,
        other => panic!("ProgramCaps must be a checked inductive, got {other:?}"),
    };
    assert_eq!(inductive.level, Level::Zero);
    assert_eq!(inductive.params, vec![Term::indformer(auth_id, vec![])]);
    assert!(inductive.indices.is_empty());
    assert_eq!(inductive.constructors.len(), 1);
    let constructor = &inductive.constructors[0];
    assert_eq!(constructor.id, constructor_id);
    assert_eq!(
        constructor.args,
        vec![Term::app(Term::const_(cap_id, vec![]), Term::var(0))]
    );

    assert!(
        !elab.env.trusted_base().contains(&program_caps_id),
        "ProgramCaps must be ordinary kernel-checked Ken"
    );
    assert!(
        !elab.env.trusted_base().contains(&constructor_id),
        "MkProgramCaps must be ordinary kernel-checked Ken"
    );

    let before = elab.env.trusted_base();
    elab.elaborate_decl(
        "fn retain_program_caps (a : Auth) (caps : ProgramCaps a) : ProgramCaps a = caps",
    )
    .expect("authority-parametric ProgramCaps consumer kernel-checks");
    assert_eq!(
        before,
        elab.env.trusted_base(),
        "ProgramCaps and its consumers must add zero trusted-base entries"
    );
}
