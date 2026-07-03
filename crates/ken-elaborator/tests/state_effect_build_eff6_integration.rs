//! `State-effect-build` (VAL2 #10 / OQ-C·C2) — EFF6 integration check (AC2).
//!
//! Team Runtime's `elim_reduce` K1.5 IH extension (`5c8dac0`, merged to
//! `main`) is already present on this rebased branch, so the FULL fold —
//! Team Language's lifted `ITree`/`runState` through Team Runtime's real
//! `elim_reduce` — can be driven end-to-end here, ahead of the Integrator's
//! formal assembly. This is NOT a hand-fed oracle: `next` is built once as a
//! real `bind (get ()) (\n. bind (put (Suc n)) (\_. Ret n))`-shaped term
//! (using the prelude's registered `bind`/`get`/`put`/`Ret`, never a
//! by-hand-constructed result), then `runState`'d and evaluated through
//! `ken_interp::eval::eval` for real. Uses `Nat` (`Zero`/`Suc`) as the state
//! type `s` (no numeric-literal machinery needed) and a fresh 0-constructor
//! `Empty` type for `F` (no other effects) — an adaptation of AC2's `Int`
//! example, same mechanism, same discriminating shape (old value returned,
//! state incremented).

use ken_elaborator::ElabEnv;
use ken_kernel::{declare_inductive, GlobalId, InductiveSpec, Level, Term};
use ken_kernel::env::GlobalEnv;

fn apply_all(head: Term, args: &[Term]) -> Term {
    args.iter().fold(head, |f, a| Term::app(f, a.clone()))
}

fn declare_empty(env: &mut GlobalEnv) -> GlobalId {
    declare_inductive(env, |_| InductiveSpec {
        level_params: vec![],
        params: vec![],
        indices: vec![],
        level: Level::zero(),
        constructors: vec![],
    })
    .expect("0-constructor Empty should be admitted")
}

#[test]
fn ac2_runstate_next_post_increment_through_real_interp() {
    let mut env = ElabEnv::new().expect("prelude should elaborate (Team Language + Team Runtime integrated)");
    let empty_id = declare_empty(&mut env.env);

    let p = env.prelude_env.clone();
    let nat_ty = Term::indformer(p.nat_id, vec![]);
    let empty_ty = Term::indformer(empty_id, vec![]);
    // RespEmpty : Empty -> Type = \_. Unit (never invoked -- Empty is uninhabited).
    let resp_empty = Term::lam(empty_ty.clone(), Term::indformer(p.unit_id, vec![]));

    // Op = Sum (StateOp Nat) Empty ; Resp = resp_sum Nat Empty RespEmpty.
    let sum_ty = apply_all(
        Term::indformer(p.sum_id, vec![]),
        &[Term::app(Term::indformer(p.state_op_id, vec![]), nat_ty.clone()), empty_ty.clone()],
    );
    let resp_ty = apply_all(
        Term::const_(p.resp_sum_id, vec![]),
        &[nat_ty.clone(), empty_ty.clone(), resp_empty.clone()],
    );

    // get () : ITree Op Resp Nat
    let get_call = apply_all(
        Term::const_(p.get_fn_id, vec![]),
        &[nat_ty.clone(), empty_ty.clone(), resp_empty.clone(), Term::constructor(p.mkunit_id, vec![])],
    );
    // \n. bind Op Resp Nat Nat (put (Suc n)) (\_. Ret Op Resp Nat n)   -- ctx [n] (len=1): n=Var(0).
    let put_call_ctx1 = apply_all(
        Term::const_(p.put_fn_id, vec![]),
        &[
            nat_ty.clone(),
            empty_ty.clone(),
            resp_empty.clone(),
            Term::app(Term::constructor(p.suc_id, vec![]), Term::var(0)),
        ],
    );
    // The inner continuation for the outer bind: \_:Unit. Ret Op Resp Nat n, ctx [n,_] (len=2): n=Var(1).
    let ret_n_ctx2 = apply_all(
        Term::constructor(p.ret_id, vec![]),
        &[
            apply_all(
                Term::indformer(p.sum_id, vec![]),
                &[Term::app(Term::indformer(p.state_op_id, vec![]), nat_ty.clone()), empty_ty.clone()],
            ),
            apply_all(Term::const_(p.resp_sum_id, vec![]), &[nat_ty.clone(), empty_ty.clone(), resp_empty.clone()]),
            nat_ty.clone(),
            Term::var(1),
        ],
    );
    let inner_cont = Term::lam(Term::indformer(p.unit_id, vec![]), ret_n_ctx2);
    // inner bind : ITree Op Resp Unit -> (Unit -> ITree Op Resp Nat) -> ITree Op Resp Nat
    let inner_bind = apply_all(
        Term::const_(p.bind_id, vec![]),
        &[sum_ty.clone(), resp_ty.clone(), Term::indformer(p.unit_id, vec![]), nat_ty.clone(), put_call_ctx1, inner_cont],
    );
    // outer: \n. inner_bind, wrapped as the continuation for the OUTER bind over `get`.
    let outer_cont = Term::lam(nat_ty.clone(), inner_bind);
    let next_body = apply_all(
        Term::const_(p.bind_id, vec![]),
        &[sum_ty.clone(), resp_ty.clone(), nat_ty.clone(), nat_ty.clone(), get_call, outer_cont],
    );

    let mut store = ken_interp::eval::EvalStore::new();
    let val = ken_interp::eval::eval(&[], &next_body, &env.env, &mut store);

    // Since F = Empty (no other effects), `next` never performs an unhandled
    // Vis; the whole fold reduces directly to `Pair(old_n, new_n)` (a=Nat,
    // s=Nat -- the Sigma pair `runState`/AC2 promise) once we thread `Zero`.
    let run_state_app = apply_all(
        Term::const_(p.run_state_id, vec![]),
        &[
            nat_ty.clone(),
            empty_ty.clone(),
            resp_empty.clone(),
            nat_ty.clone(),
            Term::constructor(p.zero_id, vec![]),
        ],
    );
    let full_term = Term::app(run_state_app, next_body.clone());
    let result = ken_interp::eval::eval(&[], &full_term, &env.env, &mut store);

    fn nat_count(env: &GlobalEnv, v: &ken_interp::eval::EvalVal, zero_id: GlobalId, suc_id: GlobalId) -> u64 {
        use ken_interp::eval::EvalVal;
        let _ = env;
        match v {
            EvalVal::Ctor { id, args, .. } if *id == zero_id && args.is_empty() => 0,
            EvalVal::Ctor { id, args, .. } if *id == suc_id && args.len() == 1 => 1 + nat_count(env, &args[0], zero_id, suc_id),
            other => panic!("expected a Nat Ctor chain, got {other:?}"),
        }
    }

    // `runState`'s codomain is `ITree F RespF (Sigma A S)` — since `F = Empty`
    // is uninhabited, the ONLY possible value is `Ret (Pair result state)`
    // (a genuine `Vis` would require an impossible `Empty` op); unwrap the
    // `Ret` to reach the pair.
    let pair = match &result {
        ken_interp::eval::EvalVal::Ctor { id, args, .. } if *id == p.ret_id => {
            args.last().cloned().expect("Ret must carry its value")
        }
        other => panic!("runState 0 next must reduce to Ret (Pair result state) (F=Empty admits no Vis), got {other:?}"),
    };
    match pair {
        ken_interp::eval::EvalVal::Pair { fst, snd, .. } => {
            let result_val = nat_count(&env.env, &fst, p.zero_id, p.suc_id);
            let final_state = nat_count(&env.env, &snd, p.zero_id, p.suc_id);
            assert_eq!(result_val, 0, "AC2: result must be the OLD (pre-increment) value");
            assert_eq!(final_state, 1, "AC2: final state must be incremented");
        }
        other => panic!("Ret's value must be a Sigma pair (result, final-state), got {other:?}"),
    }
    let _ = val; // `next` alone (unapplied to s0) is checked only by constructing it above.
}
