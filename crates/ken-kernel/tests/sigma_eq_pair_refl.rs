//! Regression for observational equality at Sigma carriers.
//!
//! `Eq (Sigma A B) p q` reduces to a Sigma proof whose second component is
//! checked under the first-component equality proof binder. The codomain must
//! therefore weaken outer references to `p`/`q` under that binder.

use ken_kernel::{
    check, declare_inductive, CtorSpec, GlobalEnv, GlobalId, InductiveSpec, KernelError, Level,
    Term,
};

struct BoolIds {
    bool_: GlobalId,
}

fn bool_env() -> (GlobalEnv, BoolIds) {
    let mut env = GlobalEnv::new();
    let bool_ = declare_inductive(&mut env, |_| InductiveSpec {
        level_params: vec![],
        params: vec![],
        indices: vec![],
        level: Level::zero(),
        constructors: vec![
            CtorSpec {
                args: vec![],
                target_indices: vec![],
            },
            CtorSpec {
                args: vec![],
                target_indices: vec![],
            },
        ],
    })
    .expect("Bool");
    (env, BoolIds { bool_ })
}

fn bool_ty(ids: &BoolIds) -> Term {
    Term::indformer(ids.bool_, vec![])
}

fn pair_bool_bool(ids: &BoolIds) -> Term {
    let bool_ty = bool_ty(ids);
    Term::sigma(bool_ty.clone(), ken_kernel::subst::weaken(&bool_ty, 1))
}

fn pair_refl_component_proof(pair: Term) -> Term {
    Term::pair(
        Term::Refl(Box::new(Term::proj1(pair.clone()))),
        Term::Refl(Box::new(Term::proj2(pair))),
    )
}

#[test]
fn sigma_eq_component_proof_checks_under_first_proof_binder() {
    let (env, ids) = bool_env();
    let pair_ty = pair_bool_bool(&ids);
    let mut ctx = ken_kernel::Context::new();
    ctx.push(pair_ty.clone());

    let s = Term::var(0);
    let eq_pair_ss = Term::Eq(Box::new(pair_ty), Box::new(s.clone()), Box::new(s.clone()));
    let proof = pair_refl_component_proof(s);

    assert!(
        check(&env, &ctx, &proof, &eq_pair_ss).is_ok(),
        "component proof for Eq (Pair Bool Bool) s s must check"
    );
}

#[test]
fn sigma_eq_component_proof_does_not_prove_unrelated_pair_equality() {
    let (env, ids) = bool_env();
    let pair_ty = pair_bool_bool(&ids);
    let mut ctx = ken_kernel::Context::new();
    ctx.push(pair_ty.clone()); // t = Var(0)
    ctx.push(pair_ty.clone()); // s = Var(0), t = Var(1)

    let s = Term::var(0);
    let t = Term::var(1);
    let eq_pair_st = Term::Eq(Box::new(pair_ty), Box::new(s.clone()), Box::new(t));
    let proof = pair_refl_component_proof(s);

    assert!(
        matches!(
            check(&env, &ctx, &proof, &eq_pair_st),
            Err(KernelError::BadEliminator(_)) | Err(KernelError::TypeMismatch { .. })
        ),
        "component refl proof for s must not prove unrelated s = t"
    );
}
