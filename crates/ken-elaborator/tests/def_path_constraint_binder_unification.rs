//! Def-path `where` clauses share the constrained-instance binder machinery.
//! These acceptance tests vary declaration keyword, naming mode, contract
//! scope, separator compatibility, and fail-closed boundaries.

use ken_elaborator::{error::ElabError, ElabEnv};

fn elab(env: &mut ElabEnv, source: &str) -> Result<ken_kernel::GlobalId, ElabError> {
    env.elaborate_decl(source)
}

fn setup_flag(env: &mut ElabEnv) {
    elab(env, "class Flag A { tag : Bool }").unwrap();
    elab(env, "instance Flag Int { tag = True }").unwrap();
    elab(env, "instance Flag Bool { tag = False }").unwrap();
    elab(env, "instance Flag a { tag = True }").unwrap();
    // Instance lookup is keyed by the surface head variable spelling; keep a
    // second generic key so the auto `da`/`db` path exercises both names.
    elab(env, "instance Flag b { tag = True }").unwrap();
}

#[test]
fn all_def_keywords_share_named_auto_and_sole_dictionary_bindings() {
    let mut env = ElabEnv::new().unwrap();
    setup_flag(&mut env);
    let before = env.env.trusted_base();

    let auto = elab(
        &mut env,
        "fn auto_names (a : Type) (b : Type) (x : Bool) : Bool where Flag a, Flag b = match da.tag { True |-> db.tag ; False |-> db.tag }",
    );
    assert!(
        auto.is_ok(),
        "auto da/db names must resolve in fn: {auto:?}"
    );

    let explicit = elab(
        &mut env,
        "view explicit_name : Bool where (chosen : Flag Int), (other : Flag Bool) = chosen.tag",
    );
    assert!(
        explicit.is_ok(),
        "an explicit dictionary name must resolve in view: {explicit:?}"
    );

    let sole = elab(
        &mut env,
        "const sole_alias : Bool where (chosen : Flag Int) = d.tag",
    );
    assert!(sole.is_ok(), "sole d alias must resolve in const: {sole:?}");

    let proc = elab(
        &mut env,
        "proc proc_name : Bool where (chosen : Flag Int) visits [Console] = chosen.tag",
    );
    assert!(proc.is_ok(), "explicit name must resolve in proc: {proc:?}");

    assert_eq!(
        before,
        env.env.trusted_base(),
        "def-path dictionary binding must add no trusted-base entries"
    );
}

#[test]
fn dictionaries_scope_over_contracts_refinement_and_body_but_not_siblings() {
    let mut env = ElabEnv::new().unwrap();
    setup_flag(&mut env);

    let scoped = elab(
        &mut env,
        "fn contract_scope (x : Bool) : { r : Bool | Equal Bool r chosen.tag } \
         requires Equal Bool chosen.tag True \
         ensures Equal Bool result chosen.tag \
         where (chosen : Flag Int) = chosen.tag",
    );
    assert!(
        scoped.is_ok(),
        "named dictionary must resolve in requires, ensures, refinement, and body: {scoped:?}"
    );

    let leaked = elab(
        &mut env,
        "const sibling_cannot_see_chosen : Bool = chosen.tag",
    );
    assert!(
        matches!(leaked, Err(ElabError::UnresolvedCon { ref name, .. }) if name == "chosen"),
        "dictionary names must not leak to sibling declarations: {leaked:?}"
    );
}

#[test]
fn semicolon_compatibility_and_fail_closed_naming_and_type_boundaries() {
    let mut env = ElabEnv::new().unwrap();
    setup_flag(&mut env);

    let legacy = elab(
        &mut env,
        "const semicolon_compat : Bool where (first : Flag Int); (second : Flag Bool) = first.tag",
    );
    assert!(
        legacy.is_ok(),
        "existing semicolon spelling must remain accepted: {legacy:?}"
    );

    let collision = elab(
        &mut env,
        "fn ambiguous (a : Type) : Bool where Flag a, Flag a = True",
    );
    assert!(
        matches!(collision, Err(ElabError::ParseError { .. })),
        "same-variable automatic dictionary names must reject: {collision:?}"
    );

    let mistyped = elab(
        &mut env,
        "const dictionary_type_checked : Bool where (chosen : Flag Int) = d.tag",
    );
    assert!(
        mistyped.is_ok(),
        "the correctly typed dictionary path is a control: {mistyped:?}"
    );
    let wrong_use = elab(
        &mut env,
        "const mistyped_dictionary_use : Int where (chosen : Flag Int) = d.tag",
    );
    assert!(
        matches!(wrong_use, Err(ElabError::KernelRejected { .. })),
        "a dictionary projection at the wrong type must fail through the checked path: {wrong_use:?}"
    );
}
