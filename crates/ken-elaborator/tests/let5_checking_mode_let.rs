//! LET-5 — checking-mode propagation through local `let` and `old`.

use ken_elaborator::ElabEnv;
use ken_kernel::Term;

fn elaborate(source: &str) -> Result<Term, ken_elaborator::error::ElabError> {
    let mut env = ElabEnv::new().expect("base environment");
    let id = env.elaborate_decl(source)?;
    let ken_kernel::Decl::Transparent { body, .. } = env
        .env
        .lookup(id)
        .expect("successful declaration is registered")
    else {
        panic!("fixture must elaborate to a transparent definition");
    };
    Ok(body.clone())
}

#[test]
fn singleton_let_preserves_checked_match_goal() {
    let core = elaborate(
        "lemma singleton_match (n : Nat) : Equal Nat n n = \
         let x : Nat = n in \
         match x { Zero ↦ Refl; Suc k ↦ Refl; }",
    )
    .expect("the let body must retain its Omega-valued checked-match goal");
    assert!(matches!(core, Term::Lam(_, ref body) if matches!(body.as_ref(), Term::Let { .. })));
}

#[test]
fn singleton_let_preserves_checked_introduction_goal() {
    let core = elaborate(
        "lemma singleton_refl (n : Nat) : Equal Nat n n = \
         let x : Nat = n in Refl",
    )
    .expect("the let body must retain the goal needed to check Refl");
    assert!(matches!(core, Term::Lam(_, ref body) if matches!(body.as_ref(), Term::Let { .. })));
}

#[test]
fn unannotated_let_infers_rhs_before_checking_body() {
    elaborate(
        "lemma inferred_rhs (n : Nat) : Equal Nat n n = \
         let x = n in Refl",
    )
    .expect("an unannotated RHS must retain the existing infer-first behavior");
}

#[test]
fn checking_mode_let_still_rejects_a_wrong_rhs() {
    let error = elaborate(
        "lemma wrong_rhs (n : Nat) : Equal Nat n n = \
         let x : Bool = n in Refl",
    )
    .expect_err("an annotated let RHS must still be checked against its annotation");
    assert!(matches!(
        error,
        ken_elaborator::error::ElabError::KernelRejected {
            error: ken_kernel::KernelError::TypeMismatch { .. },
            ..
        }
    ));
}

#[test]
fn grouped_let_preserves_checked_body_goal() {
    elaborate(
        "lemma grouped_refl (n : Nat) : Equal Nat n n = \
         let x : Nat = n; y : Nat = x in Refl",
    )
    .expect("binding count must not affect checking-mode goal propagation");
}

#[test]
fn old_transparently_preserves_its_child_goal() {
    let mut env = ElabEnv::new().expect("base environment");
    env.elaborate_decl_v1(
        "space proc old_checked (n : Nat) : Nat \
         ensures Equal (Nat → Nat) (old (λx. x)) (λx. x) = n",
    )
    .expect("old must pass the function goal through to its transparent child");
}
