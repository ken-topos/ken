//! `effect-composition` D5 §3 — the synthetic-third peel probe (Architect-
//! recommended, AC7-EXEMPT white-box mechanism unit test, NOT the acceptance
//! path — `effect-composition-conformance.md` §2/§3). Proves `run_io`'s
//! `Coproduct`-peel routes an ARBITRARY base tag, closing the structural leg's
//! residual for the terminal driver itself: {FS,Console} alone can only
//! witness a set-of-2 adversary, never an arbitrary depth/tag.
//!
//! This file intentionally hand-constructs `InL`/`InR`-wrapped `EvalVal`s —
//! that is exactly what makes it a MECHANISM probe of `run_io`'s peel, not
//! an acceptance e2e (AC7 governs the acceptance path only, §2).

use ken_elaborator::ElabEnv;
use ken_interp::{run_io, ConsoleIds, EvalStore, EvalVal, RunIoError, CoproductIds};
use ken_kernel::{declare_inductive, CtorSpec, GlobalId, InductiveSpec, Level};
use std::rc::Rc;

/// A synthetic third op-family, unknown to `run_io`'s Console/FS base
/// table — a single nullary `Ping` constructor.
fn declare_ping(elab: &mut ElabEnv) -> GlobalId {
    let ping_ind = declare_inductive(&mut elab.env, |_| InductiveSpec {
        level_params: vec![],
        params: vec![],
        indices: vec![],
        level: Level::zero(),
        constructors: vec![CtorSpec { args: vec![], target_indices: vec![] }],
    })
    .expect("Ping");
    elab.env.inductive(ping_ind).unwrap().constructors[0].id
}

fn wrap(id: GlobalId, coproduct_ids: &CoproductIds, on_left: bool) -> EvalVal {
    let tag = if on_left { coproduct_ids.inl_id } else { coproduct_ids.inr_id };
    let inner = EvalVal::Ctor { id, args: Rc::new(vec![]), slot: 0 };
    // InL/InR ctor_arity = 2 params (g,h, both Unknown fillers here — the
    // peel never inspects them) + 1 arg (the payload) = 3.
    EvalVal::Ctor {
        id: tag,
        args: Rc::new(vec![EvalVal::Unknown, EvalVal::Unknown, inner]),
        slot: 0,
    }
}

/// A `Vis` node whose op is `InL (Ping)` (single-level wrap): the peel must
/// strip exactly one layer, landing on `Ping` — reported verbatim in
/// `UnknownEffect` (the base table has no `Ping` arm).
#[test]
fn single_wrap_peels_to_the_synthetic_base_tag() {
    let mut elab = ElabEnv::new().expect("env");
    let ping_id = declare_ping(&mut elab);
    let coproduct_ids = CoproductIds { inl_id: elab.prelude_env.inl_id, inr_id: elab.prelude_env.inr_id };
    let console_ids = ConsoleIds {
        itree_id: elab.prelude_env.itree_id,
        ret_id: elab.prelude_env.ret_id,
        vis_id: elab.prelude_env.vis_id,
        write_id: elab.prelude_env.write_id,
        unit_id: elab.prelude_env.unit_id,
        params_len: 3,
    };
    let op = wrap(ping_id, &coproduct_ids, true);
    let k = EvalVal::Unknown;
    let vis = EvalVal::Ctor {
        id: elab.prelude_env.vis_id,
        args: Rc::new(vec![EvalVal::Unknown, EvalVal::Unknown, EvalVal::Unknown, op, k]),
        slot: 0,
    };
    let mut store = EvalStore::new();
    let result = run_io(vis, &console_ids, None, Some(&coproduct_ids), &elab.env, &mut store);
    match result {
        Err(RunIoError::UnknownEffect(EvalVal::Ctor { id, args, .. })) => {
            assert_eq!(id, ping_id, "the peel must land exactly on the synthetic Ping tag");
            assert!(args.is_empty(), "Ping is nullary — no stray wrapper args should survive the peel");
        }
        other => panic!("expected UnknownEffect(Ping); got {other:?}"),
    }
}

/// A DEEPER wrap (`InR (InL (Ping))`) — the peel must recurse through BOTH
/// layers, proving it isn't a single-strip shortcut. This is the "arbitrary
/// depth/tag" witness the structural grep (leg 1) can't provide on its own.
#[test]
fn nested_wrap_peels_through_multiple_layers_to_the_synthetic_base_tag() {
    let mut elab = ElabEnv::new().expect("env");
    let ping_id = declare_ping(&mut elab);
    let coproduct_ids = CoproductIds { inl_id: elab.prelude_env.inl_id, inr_id: elab.prelude_env.inr_id };
    let console_ids = ConsoleIds {
        itree_id: elab.prelude_env.itree_id,
        ret_id: elab.prelude_env.ret_id,
        vis_id: elab.prelude_env.vis_id,
        write_id: elab.prelude_env.write_id,
        unit_id: elab.prelude_env.unit_id,
        params_len: 3,
    };
    let inner = wrap(ping_id, &coproduct_ids, true); // InL Ping
    let op = {
        // InR (InL Ping) — wrap `inner` in InR by hand (mirrors `wrap`'s shape).
        EvalVal::Ctor {
            id: coproduct_ids.inr_id,
            args: Rc::new(vec![EvalVal::Unknown, EvalVal::Unknown, inner]),
            slot: 0,
        }
    };
    let k = EvalVal::Unknown;
    let vis = EvalVal::Ctor {
        id: elab.prelude_env.vis_id,
        args: Rc::new(vec![EvalVal::Unknown, EvalVal::Unknown, EvalVal::Unknown, op, k]),
        slot: 0,
    };
    let mut store = EvalStore::new();
    let result = run_io(vis, &console_ids, None, Some(&coproduct_ids), &elab.env, &mut store);
    match result {
        Err(RunIoError::UnknownEffect(EvalVal::Ctor { id, .. })) => {
            assert_eq!(id, ping_id, "the peel must recurse through BOTH InR/InL layers to reach Ping");
        }
        other => panic!("expected UnknownEffect(Ping); got {other:?}"),
    }
}

/// `coproduct_ids = None` disables peeling entirely (BV6: pre-composition callers
/// unaffected) — a wrapped op with no `CoproductIds` supplied is dispatched
/// AS-IS (its top-level ctor id, `inl_id`, doesn't match Console's
/// `write_id` either, so this still surfaces `UnknownEffect`, but on the
/// UNPEELED wrapper, not `Ping`) — proving the peel is opt-in, not always-on.
#[test]
fn no_sum_ids_disables_peeling() {
    let mut elab = ElabEnv::new().expect("env");
    let ping_id = declare_ping(&mut elab);
    let coproduct_ids = CoproductIds { inl_id: elab.prelude_env.inl_id, inr_id: elab.prelude_env.inr_id };
    let console_ids = ConsoleIds {
        itree_id: elab.prelude_env.itree_id,
        ret_id: elab.prelude_env.ret_id,
        vis_id: elab.prelude_env.vis_id,
        write_id: elab.prelude_env.write_id,
        unit_id: elab.prelude_env.unit_id,
        params_len: 3,
    };
    let op = wrap(ping_id, &coproduct_ids, true);
    let k = EvalVal::Unknown;
    let vis = EvalVal::Ctor {
        id: elab.prelude_env.vis_id,
        args: Rc::new(vec![EvalVal::Unknown, EvalVal::Unknown, EvalVal::Unknown, op, k]),
        slot: 0,
    };
    let mut store = EvalStore::new();
    let result = run_io(vis, &console_ids, None, None, &elab.env, &mut store);
    match result {
        Err(RunIoError::UnknownEffect(EvalVal::Ctor { id, .. })) => {
            assert_eq!(id, coproduct_ids.inl_id, "with coproduct_ids=None the wrapper must reach dispatch UNPEELED");
        }
        other => panic!("expected UnknownEffect(InL-wrapped); got {other:?}"),
    }
}
