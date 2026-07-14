//! AX-2: opaque labels are readable audit metadata, never kernel inputs.

use ken_kernel::{check, declare_postulate, Context, GlobalEnv, Term};

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

    assert_eq!(left.trusted_base().entries()[0].name, "left semantic owner");
    assert_eq!(
        right.trusted_base().entries()[0].name,
        "right semantic owner"
    );
}
