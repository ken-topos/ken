//! KM-literal-trust-accounting acceptance.
//!
//! Checked surface literals are values supplied by parsed source syntax; they
//! must not look like new trusted primitive assumptions. Real primitives,
//! foreigns, explicit `Axiom` holes, and open obligations remain visible.

use std::collections::HashSet;

use ken_elaborator::{foreign::trusted_base_delta, ElabEnv};
use ken_kernel::{declare_def, declare_primitive, GlobalEnv, GlobalId, Level, PrimReduction, Term};

fn trusted_set(env: &ken_kernel::GlobalEnv) -> HashSet<GlobalId> {
    env.trusted_base().into_iter().collect()
}

fn new_entries(after: &HashSet<GlobalId>, before: &HashSet<GlobalId>) -> HashSet<GlobalId> {
    after.difference(before).copied().collect()
}

#[test]
fn checked_numeric_and_string_literals_are_trust_accounting_neutral() {
    let mut env = ElabEnv::new().expect("base env");
    let base = trusted_set(&env.env);

    let ids = env
        .elaborate_file(
            r#"
            const km_lit_int : Int = 1
            const km_lit_string : String = "true"
            const km_lit_encoded : Bytes = bytes_encode "and"
            "#,
        )
        .expect("literal package snippet must elaborate");

    let after = trusted_set(&env.env);
    assert!(
        new_entries(&after, &base).is_empty(),
        "checked numeric/string literals must not add trusted_base entries; got {:?}",
        new_entries(&after, &base)
    );

    let bytes_encode = env.globals["bytes_encode"];
    for id in ids {
        let delta = trusted_base_delta(&env.env, id);
        let unexpected = new_entries(&delta, &base);
        assert!(
            unexpected.is_empty(),
            "literal definition must not add non-base trusted_base_delta entries; got {unexpected:?}"
        );
    }

    let encoded_id = env.globals["km_lit_encoded"];
    let encoded_delta = trusted_base_delta(&env.env, encoded_id);
    assert!(
        encoded_delta.contains(&bytes_encode),
        "real preexisting primitive op bytes_encode must remain visible in delta"
    );
}

#[test]
fn literal_classification_is_the_only_primitive_accounting_exclusion() {
    let mut env = GlobalEnv::new();
    let type0 = Term::ty(Level::Zero);

    let real_primitive = declare_primitive(
        &mut env,
        vec![],
        type0.clone(),
        PrimReduction::Op {
            symbol: "km_real_primitive",
        },
    )
    .expect("real primitive admits");
    assert!(
        env.trusted_base().contains(&real_primitive),
        "real primitive operation must remain in trusted_base"
    );

    let alias = declare_def(
        &mut env,
        vec![],
        type0.clone(),
        Term::const_(real_primitive, vec![]),
    )
    .expect("alias over primitive type admits");
    let alias_delta = trusted_base_delta(&env, alias);
    assert!(
        alias_delta.contains(&real_primitive),
        "real primitive reached by a transparent definition must remain in trusted_base_delta"
    );

    let literal = declare_primitive(&mut env, vec![], type0, PrimReduction::Literal)
        .expect("literal declaration admits");
    assert!(
        !env.trusted_base().contains(&literal),
        "literal classification alone is accounting-neutral"
    );
}

#[test]
fn foreign_axiom_and_open_obligation_trust_entries_still_count() {
    let mut env = ElabEnv::new().expect("base env");

    env.elaborate_decl_v1(r#"foreign km_foreign : Int -> Int = "ffi_symbol" "ffi_lib" pure"#)
        .expect("foreign declaration must elaborate");
    let foreign_id = env.globals["km_foreign"];
    assert!(
        env.env.trusted_base().contains(&foreign_id),
        "foreign postulate must remain in trusted_base"
    );

    let (_, foreign_ty) = env.env.const_type(foreign_id).expect("foreign has a type");
    let foreign_alias = declare_def(
        &mut env.env,
        vec![],
        foreign_ty,
        Term::const_(foreign_id, vec![]),
    )
    .expect("transparent alias over foreign admits");
    assert!(
        trusted_base_delta(&env.env, foreign_alias).contains(&foreign_id),
        "foreign postulate reached by a transparent definition must remain in delta"
    );

    let base_after_foreign = trusted_set(&env.env);
    let axiom_id = env
        .elaborate_decl("const km_axiom_value : Int = Axiom")
        .expect("Axiom expression must elaborate");
    let after_axiom = trusted_set(&env.env);
    let axiom_new = new_entries(&after_axiom, &base_after_foreign);
    assert_eq!(
        axiom_new.len(),
        1,
        "explicit Axiom must add exactly one trusted hole, got {axiom_new:?}"
    );
    let axiom_delta = trusted_base_delta(&env.env, axiom_id);
    assert!(
        axiom_new.iter().all(|id| axiom_delta.contains(id)),
        "Axiom hole used by the definition must remain in trusted_base_delta"
    );

    let int_id = env.globals["Int"];
    let prop_ty = Term::pi(Term::const_(int_id, vec![]), Term::omega(Level::Zero));
    env.declare_postulate_raw("KMProp", prop_ty)
        .expect("predicate postulate setup");
    let base_after_prop = trusted_set(&env.env);
    let result = env
        .elaborate_decl_v1("fn km_open_obligation (n : Int) : Int ensures KMProp result = n")
        .expect("ensures obligation declaration must elaborate");
    assert!(
        !result.obligations.is_empty(),
        "ensures clause must emit an open obligation"
    );
    let after_obligation = trusted_set(&env.env);
    let obligation_new = new_entries(&after_obligation, &base_after_prop);
    for obligation in &result.obligations {
        assert!(
            obligation_new.contains(&obligation.hole_id),
            "open obligation hole must remain visible in trusted_base"
        );
    }
}
