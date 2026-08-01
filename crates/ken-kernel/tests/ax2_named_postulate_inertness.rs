//! AX-2: opaque labels are readable audit metadata, never kernel inputs.

use ken_kernel::{check, declare_postulate, Context, Decl, GlobalEnv, GlobalId, Term};

fn trusted_ids(env: &GlobalEnv) -> Vec<GlobalId> {
    env.trusted_base()
}

fn opaque_name(env: &GlobalEnv, id: GlobalId) -> &str {
    match env.lookup(id) {
        Some(Decl::Opaque { name, .. }) => name,
        other => panic!("trusted postulate {id:?} did not resolve as opaque: {other:?}"),
    }
}

#[test]
fn changing_only_an_opaque_label_changes_audit_text_not_typing() {
    let mut left = GlobalEnv::new();
    let mut right = GlobalEnv::new();
    let left_ty = Term::const_(left.top_id(), vec![]);
    let right_ty = Term::const_(right.top_id(), vec![]);

    let left_id = declare_postulate(
        &mut left,
        "left semantic owner".to_string(),
        vec![],
        left_ty.clone(),
    )
    .expect("left postulate admits");
    let right_id = declare_postulate(
        &mut right,
        "right semantic owner".to_string(),
        vec![],
        right_ty.clone(),
    )
    .expect("right postulate admits");

    assert_eq!(left_id, right_id, "labels do not participate in identity");
    assert!(check(
        &left,
        &Context::new(),
        &Term::const_(left_id, vec![]),
        &left_ty,
    )
    .is_ok());
    assert!(check(
        &right,
        &Context::new(),
        &Term::const_(right_id, vec![]),
        &right_ty,
    )
    .is_ok());

    assert_eq!(opaque_name(&left, left_id), "left semantic owner");
    assert_eq!(opaque_name(&right, right_id), "right semantic owner");
}

#[test]
fn trusted_base_stays_id_shaped_and_resolves_shared_labels() {
    let mut env = GlobalEnv::new();
    let before = trusted_ids(&env).len();
    let ty = Term::const_(env.top_id(), vec![]);

    let first = declare_postulate(
        &mut env,
        "shared semantic owner".to_string(),
        vec![],
        ty.clone(),
    )
    .expect("first postulate admits");
    let second = declare_postulate(
        &mut env,
        "shared semantic owner".to_string(),
        vec![],
        ty,
    )
    .expect("second postulate admits");

    let ids = trusted_ids(&env);
    assert_eq!(ids.len(), before + 2, "count semantics stay unchanged");
    assert_ne!(first, second, "owner labels are not declaration identity");
    assert!(ids.contains(&first));
    assert!(ids.contains(&second));
    assert_eq!(opaque_name(&env, first), "shared semantic owner");
    assert_eq!(opaque_name(&env, second), "shared semantic owner");
}
