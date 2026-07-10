//! `State-effect-build` (VAL2 #10 / OQ-C·C2) — Team Language (elaborator)
//! lane acceptance.
//!
//! Scope: the elaborator-only obligations `git diff -- crates/ken-elaborator/`
//! actually discharges on its own — each hand-built kernel declaration
//! (`effects::state`) type-checks in isolation. This is deliberately NOT the
//! EFF6 conformance corpus (AC2-4): those require driving `runState` through
//! the REAL interpreter, which only exists once Team Runtime's `elim_reduce`
//! K1.5 IH lift (merged to `main`, `5c8dac0`) integrates with this lift on
//! the shared `wp/State-effect-build` branch. AC1 (kernel-untouched) and AC5
//! (no interior mutability) are verified out-of-band by `git diff`/`grep`,
//! not by a unit test here. AC6 (no regression) is `cargo test --workspace`.

use ken_elaborator::effects::state::{declare_bind, declare_get, declare_itree, declare_put, declare_resp_state, declare_resp_coproduct, declare_run_state, declare_state_op, declare_coproduct};
use ken_kernel::{declare_inductive, infer, normalize, CtorSpec, GlobalEnv, GlobalId, InductiveSpec, Level, Term};
use ken_kernel::env::Context;

fn lv0() -> Level {
    Level::zero()
}

fn mk_unit(env: &mut GlobalEnv) -> (GlobalId, GlobalId) {
    let unit = declare_inductive(env, |_| InductiveSpec {
        level_params: vec![],
        params: vec![],
        indices: vec![],
        level: lv0(),
        constructors: vec![CtorSpec { args: vec![], target_indices: vec![] }],
    })
    .unwrap();
    let decl = env.inductive(unit).unwrap();
    (unit, decl.constructors[0].id)
}

fn apply(head: Term, args: &[Term]) -> Term {
    args.iter().fold(head, |f, a| Term::app(f, a.clone()))
}

#[test]
fn full_state_prelude_declares_and_typechecks() {
    let mut env = GlobalEnv::new();
    let (unit_id, mkunit_id) = mk_unit(&mut env);

    let (itree_id, ret_id, vis_id) = declare_itree(&mut env).expect("ITree");
    let (state_op_id, get_id, put_id) = declare_state_op(&mut env).expect("StateOp");
    let (coproduct_id, inl_id, inr_id) = declare_coproduct(&mut env).expect("Coproduct");
    let resp_state_id = declare_resp_state(&mut env, state_op_id, unit_id).expect("resp_state");
    let resp_coproduct_id = declare_resp_coproduct(&mut env, coproduct_id).expect("resp_coproduct");
    let bind_id = declare_bind(&mut env, itree_id, vis_id).expect("bind");
    let run_state_id = declare_run_state(
        &mut env, itree_id, ret_id, vis_id, state_op_id, get_id, put_id, coproduct_id, inl_id, inr_id,
        resp_state_id, resp_coproduct_id, unit_id, mkunit_id,
    )
    .expect("runState");

    let ctx = Context::new();
    let get_ctor_id = env.inductive(state_op_id).unwrap().constructors[0].id;
    // Get : StateOp s takes StateOp's OWN param s explicitly (constructor
    // application supplies the family's params, then its own ctor args).
    let get_at_unit = apply(Term::constructor(get_ctor_id, vec![]), &[Term::indformer(unit_id, vec![])]);
    let resp_state_app = apply(
        Term::const_(resp_state_id, vec![]),
        &[Term::indformer(unit_id, vec![]), get_at_unit],
    );
    let ty = infer(&env, &ctx, &resp_state_app).expect("resp_state application should typecheck");
    assert_eq!(normalize(&env, &ctx, &ty), Term::ty(lv0()));

    // resp_coproduct is now the GENERAL `(g h:Type)->(rg:g->Type)->(rh:h->Type)->
    // Coproduct g h -> Type` (`effect-composition` D1) — 4 explicit args before the
    // still-curried `Coproduct g h -> Type` result.
    let const_unit_fn = Term::lam(Term::indformer(unit_id, vec![]), Term::indformer(unit_id, vec![]));
    let resp_coproduct_app = apply(
        Term::const_(resp_coproduct_id, vec![]),
        &[
            Term::indformer(unit_id, vec![]),
            Term::indformer(unit_id, vec![]),
            const_unit_fn.clone(),
            const_unit_fn,
        ],
    );
    let ty2 = infer(&env, &ctx, &resp_coproduct_app).expect("resp_coproduct (partially applied) should typecheck");
    match ty2 {
        Term::Pi(_, _) => {}
        other => panic!("expected a Pi type, got {other:?}"),
    }

    let get_fn_id = declare_get(
        &mut env, itree_id, ret_id, vis_id, state_op_id, get_id, coproduct_id, inl_id, resp_coproduct_id, resp_state_id, unit_id,
    )
    .expect("get");
    let put_fn_id = declare_put(
        &mut env, itree_id, ret_id, vis_id, state_op_id, put_id, coproduct_id, inl_id, resp_coproduct_id, resp_state_id, unit_id,
        mkunit_id,
    )
    .expect("put");

    assert!(env.lookup(bind_id).is_some(), "bind should be registered as a real decl");
    assert!(env.lookup(run_state_id).is_some(), "runState should be registered as a real decl");
    assert!(env.lookup(get_fn_id).is_some(), "get should be registered as a real decl");
    assert!(env.lookup(put_fn_id).is_some(), "put should be registered as a real decl");
    let _ = itree_id;
}
