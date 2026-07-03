//! K1.5 W-style (Π-bound) IH acceptance — `ken-interp`'s half of
//! `docs/program/wp/State-effect-build.md` (VAL2 #10 / OQ-C·C2).
//!
//! `elim_reduce`'s pre-existing IH loop only recognized a DIRECT recursive
//! constructor position (arg type's head IS the family). `ITree`'s `Vis :
//! (Resp e → ITree r) → ITree r` is Π-BOUND (K1.5 W-style, `36 §4.5.6` lift
//! (c), `ken-kernel/src/inductive.rs` `recursive_args`) — the recursive
//! occurrence is the CODOMAIN of a function type, invisible to the old
//! `is_recursive_arg` check, so `runState`'s `elim_ITree` fold never applied
//! an IH to a `Vis` node at all (silently stuck via `apply`'s catch-all).
//!
//! This adds `recursive_arg_arity` (peels leading `Term::Pi`s, same
//! detection the kernel already uses) and a new `EvalVal::IhClosure` — a
//! native curried function of arity `nb` that, once saturated with the `nb`
//! branch values, threads them into the recursive field and folds through
//! `elim_reduce` (mirrors the kernel's term-level IH `λb̄. elim_D … (a_j b̄)`,
//! `iota_reduct`). These tests exercise the mechanism standalone against a
//! synthetic W-style `ITree`, per the frame's "testable on its own" note —
//! full `runState`/EFF6 AC2–4 only flip green once Team Language's
//! elaborator-side lift (dependent response + coproduct + derived stdlib)
//! integrates on top of this.
//!
//! Kernel untouched (AC1): this file only exercises `ken-kernel`'s existing
//! `declare_inductive`/`Term` API to build test fixtures; zero `ken-kernel`
//! source changes ride with this WP (verify via `git diff -- crates/ken-kernel/`).

use ken_interp::eval::{eval, EvalStore, EvalVal};
use ken_kernel::{declare_inductive, CtorSpec, GlobalEnv, GlobalId, InductiveSpec, Level, Term};

// ── shared fixtures ─────────────────────────────────────────────────────────

struct BoolEnv {
    id: GlobalId,
    false_id: GlobalId,
    true_id: GlobalId,
}

/// `data Bool = False | True`.
fn mk_bool(env: &mut GlobalEnv) -> BoolEnv {
    let id = declare_inductive(env, |_| InductiveSpec {
        level_params: vec![],
        params: vec![],
        indices: vec![],
        level: Level::zero(),
        constructors: vec![
            CtorSpec { args: vec![], target_indices: vec![] },
            CtorSpec { args: vec![], target_indices: vec![] },
        ],
    })
    .unwrap();
    let decl = env.inductive(id).unwrap();
    BoolEnv { id, false_id: decl.constructors[0].id, true_id: decl.constructors[1].id }
}

struct NatEnv {
    id: GlobalId,
    zero_id: GlobalId,
    suc_id: GlobalId,
}

/// `data Nat = Zero | Suc Nat`.
fn mk_nat(env: &mut GlobalEnv) -> NatEnv {
    let id = declare_inductive(env, |nat| InductiveSpec {
        level_params: vec![],
        params: vec![],
        indices: vec![],
        level: Level::zero(),
        constructors: vec![
            CtorSpec { args: vec![], target_indices: vec![] },
            CtorSpec { args: vec![Term::indformer(nat, vec![])], target_indices: vec![] },
        ],
    })
    .unwrap();
    let decl = env.inductive(id).unwrap();
    NatEnv { id, zero_id: decl.constructors[0].id, suc_id: decl.constructors[1].id }
}

struct ITreeEnv {
    id: GlobalId,
    ret_id: GlobalId,
    vis_id: GlobalId,
}

/// `data ITree (R : Type 0) = Ret R | Vis (Bool -> ITree R)`.
/// The response domain is fixed to `Bool` — the dependent `E.Resp e` lift
/// ((a) in the frame) is Team Language's elaborator-side job; this test
/// exercises the W-style FOLD mechanism, which is response-domain-agnostic.
fn mk_itree(env: &mut GlobalEnv, bool_id: GlobalId) -> ITreeEnv {
    let id = declare_inductive(env, |itree| InductiveSpec {
        level_params: vec![],
        params: vec![Term::Type(Level::zero())],
        indices: vec![],
        level: Level::zero(),
        constructors: vec![
            // Ret : (r : R) -> ITree R      [in ctx [R]: R = Var(0)]
            CtorSpec { args: vec![Term::var(0)], target_indices: vec![] },
            // Vis : (Bool -> ITree R) -> ITree R
            //   in ctx [R]: Pi(Bool, App(ITree, Var(1)))
            CtorSpec {
                args: vec![Term::pi(
                    Term::indformer(bool_id, vec![]),
                    Term::app(Term::indformer(itree, vec![]), Term::var(1)),
                )],
                target_indices: vec![],
            },
        ],
    })
    .unwrap();
    let decl = env.inductive(id).unwrap();
    ITreeEnv { id, ret_id: decl.constructors[0].id, vis_id: decl.constructors[1].id }
}

fn ctor(id: GlobalId) -> Term {
    Term::Constructor { id, level_args: vec![] }
}
fn fmr(id: GlobalId) -> Term {
    Term::indformer(id, vec![])
}
fn ret_t(ret_id: GlobalId, r_ty: Term, val: Term) -> Term {
    Term::app(Term::app(ctor(ret_id), r_ty), val)
}
fn vis_t(vis_id: GlobalId, r_ty: Term, k: Term) -> Term {
    Term::app(Term::app(ctor(vis_id), r_ty), k)
}

/// `elim_ITree` is untyped at the value layer (`eval` never reads `motive`
/// for `Term::Elim` — only `fam`/`methods`/`scrut`), so every test below
/// passes this inert placeholder rather than a genuinely well-typed motive.
fn inert_motive() -> Term {
    Term::Type(Level::zero())
}

// ═══════════════════════════════════════════════════════════════════════════
// 1 — the IH actually recurses at depth 2 (not just depth 1)
// ═══════════════════════════════════════════════════════════════════════════

/// `size : ITree Bool -> Nat` via `elim_ITree`, `size (Ret r) = 0`,
/// `size (Vis k) = suc (size (k False))` — driven TWO levels deep:
/// `Vis (λ_. Vis (λ_. Ret True))`. A mechanism that only fires the W-ι once
/// (e.g. treats the IH as already-final instead of re-foldable) would return
/// `1` here instead of the correct `2` — the depth-1-vs-depth-2 verdict flip
/// a single-level test cannot see.
#[test]
fn wstyle_ih_folds_at_depth_two() {
    let mut env = GlobalEnv::new();
    let b = mk_bool(&mut env);
    let it = mk_itree(&mut env, b.id);
    let nat = mk_nat(&mut env);
    let mut store = EvalStore::new();

    let r_ty = fmr(b.id);

    // mr = λ(r:Bool). zero
    let mr = Term::lam(fmr(b.id), ctor(nat.zero_id));
    // mv = λ(k:Bool→ITree Bool). λ(ih:Bool→Nat). suc (ih False)
    let mv = Term::lam(
        Term::pi(fmr(b.id), Term::app(fmr(it.id), r_ty.clone())),
        Term::lam(
            Term::pi(fmr(b.id), fmr(nat.id)),
            Term::app(ctor(nat.suc_id), Term::app(Term::var(0), ctor(b.false_id))),
        ),
    );

    // innermost: Ret True
    let inner_ret = ret_t(it.ret_id, r_ty.clone(), ctor(b.true_id));
    // level 2: Vis (λ_:Bool. Ret True)
    let k2 = Term::lam(fmr(b.id), inner_ret);
    let level2 = vis_t(it.vis_id, r_ty.clone(), k2);
    // level 1: Vis (λ_:Bool. level2)
    let k1 = Term::lam(fmr(b.id), level2);
    let tree_term = vis_t(it.vis_id, r_ty.clone(), k1);

    let elim = Term::Elim {
        fam: it.id,
        level_args: vec![],
        params: vec![r_ty],
        motive: Box::new(inert_motive()),
        methods: vec![mr, mv],
        indices: vec![],
        scrut: Box::new(tree_term),
    };

    let result = eval(&[], &elim, &env, &mut store);
    let expected = eval(
        &[],
        &Term::app(ctor(nat.suc_id), Term::app(ctor(nat.suc_id), ctor(nat.zero_id))),
        &env,
        &mut store,
    );
    assert_eq!(result, expected, "size of a depth-2 Vis chain must be 2 (suc (suc zero))");
}

// ═══════════════════════════════════════════════════════════════════════════
// 2 — State-shaped pair fold: threading rehearses `runState`'s (result, state)
// ═══════════════════════════════════════════════════════════════════════════

/// A `runState`-shaped fold, `Bool -> Pair Bool Bool` (`S -> (A × S)` at
/// `A = S = Bool`): `Ret r` yields `(r, s)` for the incoming state `s`;
/// `Vis k` denotes a `get`-like op — the response IS the current state `s`,
/// threaded into `k` AND carried forward as the next state. Run from two
/// different initial states (`False`/`True`, AC4's discriminator) — a
/// mis-threaded fold (e.g. one that drops `s` and re-derives a constant)
/// would collapse both runs to the same pair; a correctly-threaded fold does
/// not.
#[test]
fn wstyle_pair_valued_fold_threads_state_through_vis() {
    let mut env = GlobalEnv::new();
    let b = mk_bool(&mut env);
    let it = mk_itree(&mut env, b.id);
    let mut store = EvalStore::new();

    let r_ty = fmr(b.id);

    // mr = λ(r:Bool). λ(s:Bool). Pair(r, s)
    let mr = Term::lam(
        fmr(b.id),
        Term::lam(fmr(b.id), Term::Pair(Box::new(Term::var(1)), Box::new(Term::var(0)))),
    );
    // mv = λ(k:Bool→ITree Bool). λ(ih:Bool→Bool→Pair Bool Bool). λ(s:Bool).
    //        (ih s) s     -- "get": feed current state as the response,
    //                        thread it forward as the next state too.
    let mv = Term::lam(
        Term::pi(fmr(b.id), Term::app(fmr(it.id), r_ty.clone())),
        Term::lam(
            Term::pi(
                fmr(b.id),
                Term::pi(fmr(b.id), Term::Pair(Box::new(fmr(b.id)), Box::new(fmr(b.id)))),
            ),
            Term::lam(fmr(b.id), Term::app(Term::app(Term::var(1), Term::var(0)), Term::var(0))),
        ),
    );

    // tree = Vis (λs. Ret s)  — a single `get` that returns the state as the
    // result (post-increment-style read, one level).
    let k = Term::lam(fmr(b.id), ret_t(it.ret_id, r_ty.clone(), Term::var(0)));
    let tree_term = vis_t(it.vis_id, r_ty.clone(), k);

    let elim = Term::Elim {
        fam: it.id,
        level_args: vec![],
        params: vec![r_ty],
        motive: Box::new(inert_motive()),
        methods: vec![mr, mv],
        indices: vec![],
        scrut: Box::new(tree_term),
    };

    let fold_val = eval(&[], &elim, &env, &mut store);

    // Run from s0 = False: expect (False, False).
    let false_val = eval(&[], &ctor(b.false_id), &env, &mut store);
    let run_false = ken_interp::apply(fold_val.clone(), false_val.clone(), &env, &mut store);
    match &run_false {
        EvalVal::Pair { fst, snd, .. } => {
            assert_eq!(**fst, false_val, "run from False: result must be False");
            assert_eq!(**snd, false_val, "run from False: final state must be False");
        }
        other => panic!("expected a Pair; got {:?}", other),
    }

    // Run from s0 = True: expect (True, True) — DIFFERENT from the False run,
    // proving the state was actually threaded (not a hard-coded constant).
    let true_val = eval(&[], &ctor(b.true_id), &env, &mut store);
    let run_true = ken_interp::apply(fold_val, true_val.clone(), &env, &mut store);
    match &run_true {
        EvalVal::Pair { fst, snd, .. } => {
            assert_eq!(**fst, true_val, "run from True: result must be True");
            assert_eq!(**snd, true_val, "run from True: final state must be True");
        }
        other => panic!("expected a Pair; got {:?}", other),
    }
    assert_ne!(run_false, run_true, "two initial states must yield two independent pairs (AC4 shape)");
}

// ═══════════════════════════════════════════════════════════════════════════
// 3 — RTP1's dead-code-skip fix still applies correctly at nb >= 1
// ═══════════════════════════════════════════════════════════════════════════

/// A `Vis` method that never references its IH binder must still be skipped
/// (RTP1 (B') `term_var_free`) — even though the position is now W-style
/// (`nb = 1`, not the direct `nb = 0` case RTP1 was originally proven
/// against). Discriminator: a regression that mis-indexes the de Bruijn
/// liveness check across the now-arity-tagged `rec_positions` list could
/// misfire and either get stuck or return the wrong constant.
///
/// The tree's continuation is deliberately self-referential (`λ_. Var(0)`,
/// i.e. it would loop forever if `elim_reduce` ever actually forced it) —
/// if the liveness skip regressed and the IH were eagerly computed anyway,
/// this test hangs/stack-overflows instead of failing cleanly, which is
/// itself the point: the skip must be honored, not just tolerated by luck.
#[test]
fn wstyle_unused_ih_is_dead_code_skipped() {
    let mut env = GlobalEnv::new();
    let b = mk_bool(&mut env);
    let it = mk_itree(&mut env, b.id);
    let nat = mk_nat(&mut env);
    let mut store = EvalStore::new();

    let r_ty = fmr(b.id);
    let mr = Term::lam(fmr(b.id), ctor(nat.zero_id));
    // mv = λk. λih. suc zero   -- ih (Var 0) is NEVER referenced.
    let mv = Term::lam(
        Term::pi(fmr(b.id), Term::app(fmr(it.id), r_ty.clone())),
        Term::lam(Term::pi(fmr(b.id), fmr(nat.id)), Term::app(ctor(nat.suc_id), ctor(nat.zero_id))),
    );

    let k = Term::lam(fmr(b.id), Term::var(0)); // self-referential; never forced
    let tree_term = vis_t(it.vis_id, r_ty.clone(), k);

    let elim = Term::Elim {
        fam: it.id,
        level_args: vec![],
        params: vec![r_ty],
        motive: Box::new(inert_motive()),
        methods: vec![mr, mv],
        indices: vec![],
        scrut: Box::new(tree_term),
    };

    let result = eval(&[], &elim, &env, &mut store);
    let expected = eval(&[], &Term::app(ctor(nat.suc_id), ctor(nat.zero_id)), &env, &mut store);
    assert_eq!(result, expected, "unused W-style IH must be skipped, not eagerly folded");
}

// ═══════════════════════════════════════════════════════════════════════════
// 4 — a single constructor mixing a DIRECT and a W-style recursive position
// ═══════════════════════════════════════════════════════════════════════════

/// `data Mixed = Mk Mixed (Bool -> Mixed) | Leaf` — arg[0] of `Mk` is direct
/// (`nb=0`), arg[1] is W-style (`nb=1`); `Leaf` is a nullary base case used
/// only as an inert field value (never itself the elim scrutinee). Both IHs
/// must compute independently and correctly in the same ι-reduction —
/// regression net for `rec_positions`' index-pairing after the arity-aware
/// rewrite (a transposition bug here would swap which IH is direct vs.
/// deferred).
#[test]
fn mixed_direct_and_wstyle_positions_fold_together() {
    let mut env = GlobalEnv::new();
    let b = mk_bool(&mut env);
    let nat = mk_nat(&mut env);

    let mixed = declare_inductive(&mut env, |mixed| InductiveSpec {
        level_params: vec![],
        params: vec![],
        indices: vec![],
        level: Level::zero(),
        constructors: vec![
            CtorSpec {
                args: vec![
                    Term::indformer(mixed, vec![]), // arg[0]: Mixed (direct)
                    Term::pi(fmr(b.id), Term::indformer(mixed, vec![])), // arg[1]: Bool -> Mixed (W-style)
                ],
                target_indices: vec![],
            },
            CtorSpec { args: vec![], target_indices: vec![] }, // Leaf
        ],
    })
    .unwrap();
    let decl = env.inductive(mixed).unwrap();
    let mk_id = decl.constructors[0].id;
    let leaf_id = decl.constructors[1].id;
    let mut store = EvalStore::new();

    // Motive is a constant Nat. `mm` (the Mk method) receives, in order:
    // direct field, k field, ih_direct : Nat, ih_w : Bool -> Nat. Result:
    // Pair(suc ih_direct, ih_w False) — both IHs exercised, packaged so the
    // test can assert each independently.
    let mm = Term::lam(
        fmr(mixed),
        Term::lam(
            Term::pi(fmr(b.id), fmr(mixed)),
            Term::lam(
                fmr(nat.id),
                Term::lam(
                    Term::pi(fmr(b.id), fmr(nat.id)),
                    Term::Pair(
                        Box::new(Term::app(ctor(nat.suc_id), Term::var(1))),
                        Box::new(Term::app(Term::var(0), ctor(b.false_id))),
                    ),
                ),
            ),
        ),
    );
    // `ml` (the Leaf method) is what each recursive IH call selects when it
    // recurses into a Leaf-headed field — reuse `zero` so both IHs land on
    // the same known Nat value.
    let ml = ctor(nat.zero_id);

    // scrut = Mk Leaf (λ_. Leaf) — both recursive fields are the Leaf base
    // case, so each IH call selects `ml` and folds to `zero`.
    let leaf = ctor(leaf_id);
    let k_fn = Term::lam(fmr(b.id), leaf.clone());
    let scrut = Term::app(Term::app(ctor(mk_id), leaf), k_fn);

    let elim = Term::Elim {
        fam: mixed,
        level_args: vec![],
        params: vec![],
        motive: Box::new(inert_motive()),
        methods: vec![mm, ml],
        indices: vec![],
        scrut: Box::new(scrut),
    };

    let result = eval(&[], &elim, &env, &mut store);
    match result {
        EvalVal::Pair { fst, snd, .. } => {
            // ih_direct = elim_Mixed(Leaf) = ml = zero, so fst = suc zero.
            let expected_fst = eval(&[], &Term::app(ctor(nat.suc_id), ctor(nat.zero_id)), &env, &mut store);
            // ih_w False = elim_Mixed(k_fn False) = elim_Mixed(Leaf) = ml = zero
            // directly (no extra `suc` — `mm` only wraps ih_direct in `suc`).
            let expected_snd = eval(&[], &ctor(nat.zero_id), &env, &mut store);
            assert_eq!(*fst, expected_fst, "direct IH must fold the Leaf field to (suc zero)");
            assert_eq!(*snd, expected_snd, "W-style IH applied to False must fold the Leaf field to zero");
        }
        other => panic!("expected a Pair combining both IHs; got {:?}", other),
    }
}
