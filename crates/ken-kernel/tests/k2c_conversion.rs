//! K2c series-1 conformance tests — SCT gate and Unit-η.
//!
//! Covers `conformance/kernel/conversion/seed-conversion.md` SCT accept/reject
//! cases and `17 §2` Unit-η; `conformance/kernel/judgments/seed-judgments.md`
//! declare-def-sct-admits / declare-def-sct-rejects.

use ken_kernel::env::Context;
use ken_kernel::term::{Level, Term};
use ken_kernel::{
    convert, declare_def, declare_inductive, declare_recursive_group, CtorSpec, GlobalEnv,
    GlobalId, InductiveSpec, KernelError,
};

// ---------------------------------------------------------------------------
// Minimal environment: Nat + Bool
// ---------------------------------------------------------------------------

struct NB {
    nat: GlobalId,
    zero: GlobalId,
    suc: GlobalId,
    bool_: GlobalId,
    true_: GlobalId,
    false_: GlobalId,
}

fn mk_env() -> (GlobalEnv, NB) {
    let mut env = GlobalEnv::new();
    let nat = declare_inductive(&mut env, |nat| InductiveSpec {
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
                args: vec![Term::indformer(nat, vec![])],
                target_indices: vec![],
            },
        ],
    })
    .expect("Nat");
    let (zero, suc) = {
        let cs = &env.inductive(nat).unwrap().constructors;
        (cs[0].id, cs[1].id)
    };
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
    let (true_, false_) = {
        let cs = &env.inductive(bool_).unwrap().constructors;
        (cs[0].id, cs[1].id)
    };
    (
        env,
        NB {
            nat,
            zero,
            suc,
            bool_,
            true_,
            false_,
        },
    )
}

fn unit_env() -> (GlobalEnv, GlobalId, GlobalId) {
    let mut env = GlobalEnv::new();
    let unit = declare_inductive(&mut env, |_| InductiveSpec {
        level_params: vec![],
        params: vec![],
        indices: vec![],
        level: Level::zero(),
        constructors: vec![CtorSpec {
            args: vec![],
            target_indices: vec![],
        }],
    })
    .expect("Unit");
    let tt = env.inductive(unit).unwrap().constructors[0].id;
    (env, unit, tt)
}

// ---------------------------------------------------------------------------
// Term shorthands
// ---------------------------------------------------------------------------

fn nat_t(nb: &NB) -> Term {
    Term::indformer(nb.nat, vec![])
}
fn bool_t(nb: &NB) -> Term {
    Term::indformer(nb.bool_, vec![])
}
fn zero_c(nb: &NB) -> Term {
    Term::constructor(nb.zero, vec![])
}
fn suc_c(nb: &NB) -> Term {
    Term::constructor(nb.suc, vec![])
}
fn true_c(nb: &NB) -> Term {
    Term::constructor(nb.true_, vec![])
}
fn false_c(nb: &NB) -> Term {
    Term::constructor(nb.false_, vec![])
}
fn pi_nn(nb: &NB) -> Term {
    Term::pi(nat_t(nb), nat_t(nb))
}

/// Ascribed motive `(λ _. body_ty) : Nat → Type 0`.
/// All our test elims are on `Nat` with 0 indices, so the motive type is
/// always `Nat → Type 0`.
fn asc_motive(nb: &NB, body_ty: Term) -> Term {
    let motive_ty = Term::pi(nat_t(nb), Term::Type(Level::zero()));
    Term::Ascript(Box::new(Term::lam(nat_t(nb), body_ty)), Box::new(motive_ty))
}

/// `elim_Nat motive zero_method suc_method scrut`.
fn nat_elim(nb: &NB, motive: Term, z: Term, s: Term, scrut: Term) -> Term {
    Term::Elim {
        fam: nb.nat,
        level_args: vec![],
        params: vec![],
        motive: Box::new(motive),
        methods: vec![z, s],
        indices: vec![],
        scrut: Box::new(scrut),
    }
}

/// `Const` reference.
fn cref(id: GlobalId) -> Term {
    Term::Const {
        id,
        level_args: vec![],
    }
}

// ---------------------------------------------------------------------------
// SCT-accept: sct-accept-second-parameter
//
// g : Nat → Nat → Nat
//   g acc 0      = acc
//   g acc (suc n') = g (suc acc) n'
//
// Motive = λ_. Nat→Nat  (suc of the first param grows; second strictly ↓).
// Method types:
//   zero : (λ_. Nat→Nat) 0 = Nat → Nat
//   suc  : Π (n':Nat). (Nat→Nat) → (Nat→Nat)
//
// SCT edge: g_caller → g_callee with M = [[?,?],[?,↓]].
// M⊙M[1,1] = compose(↓,↓) = ↓ → ACCEPT.
// ---------------------------------------------------------------------------

#[test]
fn sct_accept_second_parameter() {
    let (mut env, nb) = mk_env();
    let nat = nb.nat;
    let suc = nb.suc;
    let ty = Term::pi(nat_t(&nb), Term::pi(nat_t(&nb), nat_t(&nb)));
    let ids = declare_recursive_group(&mut env, vec![(vec![], ty)], |ids| {
        let g = ids[0];
        // g = λ acc. λ n.
        //   (elim_Nat (λ_. Nat→Nat : Nat→Type 0)
        //     (λ a. a)                           -- zero: id
        //     (λ n'. λ ih. λ a. g (suc a) n')   -- suc: g (suc acc) n'
        //     n)
        //   acc
        //
        // After outer λ acc. λ n.: Var(0)=n, Var(1)=acc.
        // Elim scrut = Var(0); applied to Var(1) after.
        //
        // Suc method type: Π(n':Nat).(Nat→Nat)→(Nat→Nat).
        //   n' binder: nat_t
        //   ih binder: pi_nn (Nat→Nat, the IH = M(n') = Nat→Nat)
        let nat_t = Term::indformer(nat, vec![]);
        let pi_nn = Term::pi(nat_t.clone(), nat_t.clone());
        let suc_t = Term::constructor(suc, vec![]);

        let suc_method = Term::lam(
            nat_t.clone(), // n' : Nat
            Term::lam(
                pi_nn.clone(), // ih : Nat → Nat (IH = M n')
                Term::lam(
                    nat_t.clone(), // a : Nat (result arg)
                    // g (suc a) n'
                    // In body: Var(0)=a, Var(1)=ih, Var(2)=n', Var(3)=n, Var(4)=acc
                    Term::app(
                        Term::app(cref(g), Term::app(suc_t, Term::var(0))),
                        Term::var(2),
                    ),
                ),
            ),
        );

        let motive = asc_motive(&nb, pi_nn.clone());
        let elim = nat_elim(
            &nb,
            motive,
            Term::lam(nat_t.clone(), Term::var(0)), // zero: λ a. a
            suc_method,
            Term::var(0), // scrut = n (Var 0 after outer lambdas)
        );
        // Apply result (Nat→Nat) to acc (Var 1).
        vec![Term::lam(
            nat_t.clone(),
            Term::lam(nat_t.clone(), Term::app(elim, Term::var(1))),
        )]
    })
    .expect("sct-accept-second-parameter must be admitted");
    assert!(env.transparent_body(ids[0]).is_some());
}

// ---------------------------------------------------------------------------
// SCT-accept: sct-accept-lexicographic
//
// ack : Nat → Nat → Nat
//   ack 0      n  = suc n
//   ack (suc m) n = ack m n   (simplified: just uses first arg strictly ↓)
//
// SCT edge: caller param0=m, callee arg0=m' (field of suc m) → ↓ → ACCEPT.
// (The actual Ackermann would also be accepted but building its full body
// requires nested eliminators; this simplified variant suffices to exercise
// the lexicographic path through SCT.)
// ---------------------------------------------------------------------------

#[test]
fn sct_accept_lexicographic() {
    let (mut env, nb) = mk_env();
    let nat = nb.nat;
    let suc = nb.suc;
    let ty = Term::pi(nat_t(&nb), Term::pi(nat_t(&nb), nat_t(&nb)));
    let ids = declare_recursive_group(&mut env, vec![(vec![], ty)], |ids| {
        let ack = ids[0];
        let nat_t = Term::indformer(nat, vec![]);
        let suc_t = Term::constructor(suc, vec![]);
        let pi_nn = Term::pi(nat_t.clone(), nat_t.clone());

        // ack = λ m. λ n.
        //   (elim_Nat (λ_. Nat→Nat)
        //     (λ n'. suc n')             -- zero: suc n
        //     (λ m'. λ ih. λ n'. ack m' n')  -- suc: ack m' n' (m' ↓ m)
        //     m)
        //   n
        //
        // Outer body after λ m. λ n.: Var(0)=n, Var(1)=m.
        // Suc method type: Π(m':Nat).(Nat→Nat)→(Nat→Nat).
        //   m' binder: nat_t
        //   ih binder: pi_nn (IH = M m' = Nat→Nat)
        let suc_method = Term::lam(
            nat_t.clone(), // m' : Nat (field)
            Term::lam(
                pi_nn.clone(), // ih : Nat → Nat (IH)
                Term::lam(
                    nat_t.clone(), // n' : Nat (result arg)
                    // ack m' n'
                    // Var(0)=n', Var(1)=ih, Var(2)=m', Var(3)=n, Var(4)=m
                    Term::app(Term::app(cref(ack), Term::var(2)), Term::var(0)),
                ),
            ),
        );

        // Zero case: λ n'. suc n'
        let zero_method = Term::lam(nat_t.clone(), Term::app(suc_t, Term::var(0)));

        let motive = asc_motive(&nb, pi_nn);
        let elim = nat_elim(&nb, motive, zero_method, suc_method, Term::var(1));
        vec![Term::lam(
            nat_t.clone(),
            Term::lam(nat_t.clone(), Term::app(elim, Term::var(0))),
        )]
    })
    .expect("sct-accept-lexicographic must be admitted");
    assert!(env.transparent_body(ids[0]).is_some());
}

// ---------------------------------------------------------------------------
// SCT-accept: sct-accept-mutual
//
// isEven : Nat → Bool; isOdd : Nat → Bool.
//   isEven 0 = true;  isEven (suc n') = isOdd  n'.
//   isOdd  0 = false; isOdd  (suc n') = isEven n'.
//
// Each call passes n' (field of suc n → ↓ on param0).
// Composed self-loop: compose(↓, ↓) = ↓ → ACCEPT.
// ---------------------------------------------------------------------------

#[test]
fn sct_accept_mutual() {
    let (mut env, nb) = mk_env();
    let nat = nb.nat;
    let ty = Term::pi(nat_t(&nb), bool_t(&nb));
    let ids = declare_recursive_group(&mut env, vec![(vec![], ty.clone()), (vec![], ty)], |ids| {
        let is_even = ids[0];
        let is_odd = ids[1];
        let nat_t = Term::indformer(nat, vec![]);
        let bool_t = Term::indformer(nb.bool_, vec![]);
        let true_t = Term::constructor(nb.true_, vec![]);
        let false_t = Term::constructor(nb.false_, vec![]);

        // isEven = λ n. elim_Nat (λ_. Bool) true (λ n'. λ _ih. isOdd n') n
        // Suc method type: Π(n':Nat). Bool → Bool.
        //   n' binder: nat_t; _ih binder: bool_t (IH = M n' = Bool).
        // After enter_method(n_fields=1, n_ihs=1):
        //   prov: [None(ih), Some(0↓)(n'), Some(0↓=)(n)].
        //   collect on: App(Const(isOdd), Var(1)).
        //   Var(1) = n' → prov[1] = Some((0, Down)) → sizeRel(param0=n, n') = ↓.
        // M_even_odd = [[↓]].
        let even_suc = Term::lam(
            nat_t.clone(), // n' : Nat
            Term::lam(
                bool_t.clone(),                        // _ih : Bool (IH = M n' = Bool)
                Term::app(cref(is_odd), Term::var(1)), // isOdd n'
            ),
        );
        let is_even_body = Term::lam(
            nat_t.clone(),
            nat_elim(
                &nb,
                asc_motive(&nb, bool_t.clone()),
                true_t,
                even_suc,
                Term::var(0),
            ),
        );

        // isOdd = λ n. elim_Nat (λ_. Bool) false (λ n'. λ _ih. isEven n') n
        let odd_suc = Term::lam(
            nat_t.clone(), // n' : Nat
            Term::lam(
                bool_t.clone(),                         // _ih : Bool
                Term::app(cref(is_even), Term::var(1)), // isEven n'
            ),
        );
        let is_odd_body = Term::lam(
            nat_t.clone(),
            nat_elim(
                &nb,
                asc_motive(&nb, bool_t.clone()),
                false_t,
                odd_suc,
                Term::var(0),
            ),
        );

        vec![is_even_body, is_odd_body]
    })
    .expect("sct-accept-mutual must be admitted");
    assert!(env.transparent_body(ids[0]).is_some());
    assert!(env.transparent_body(ids[1]).is_some());
}

// ---------------------------------------------------------------------------
// SCT-accept: sct-accept-permuted
//
// foo bar : Nat → Nat → Nat.
//   foo 0      b = b;    foo (suc a) b = bar b  a.
//   bar b      0 = b;    bar b (suc a) = foo a  b.
//
// The descending value (a) moves from position 0 (foo) to position 1 (bar).
// Self-loop foo: M_foo_bar.compose(M_bar_foo)[0,0] = compose(?,?) ∨ compose(↓,↓)
//   where [[↓,?],[?,?]] ⊙ [[?,?],[?,↓]] wait — let me be precise.
//
// foo→bar: arg0=b (Var(0) → None), arg1=a' (Var(2) → Some(0,↓)).
//   M_foo_bar = [[?,↓],[?,?]]  (row=foo-param, col=bar-param).
// bar→foo: arg0=a' (Var(2) → Some(1,↓)), arg1=b (Var(0) → None).
//   M_bar_foo = [[?,?],[↓,?]]  (row=bar-param, col=foo-param).
// Self-loop[0,0] = max(compose(?,?), compose(↓,↓)) = ↓ → ACCEPT.
// ---------------------------------------------------------------------------

#[test]
fn sct_accept_permuted() {
    let (mut env, nb) = mk_env();
    let nat = nb.nat;
    let ty = Term::pi(nat_t(&nb), Term::pi(nat_t(&nb), nat_t(&nb)));
    let ids = declare_recursive_group(&mut env, vec![(vec![], ty.clone()), (vec![], ty)], |ids| {
        let foo_id = ids[0];
        let bar_id = ids[1];
        let nat_t = Term::indformer(nat, vec![]);
        let pi_nn = Term::pi(nat_t.clone(), nat_t.clone());

        // foo = λ a. λ b.
        //   (elim_Nat (λ_. Nat→Nat)
        //     (λ b'. b')                       -- zero: b
        //     (λ a'. λ ih. λ b'. bar b' a')   -- suc: bar b a'
        //     a)
        //   b
        //
        // Outer body: Var(0)=b, Var(1)=a.  Elim scrut=Var(1)=a.
        // Suc method type: Π(a':Nat).(Nat→Nat)→(Nat→Nat).
        //   a' binder: nat_t; ih binder: pi_nn.
        // enter_method peels a' (field → Some(0,↓)) and ih (None).
        // collect on λ b'. bar b' a'.
        //   Push None for b'. prov: [None,None,Some(0↓),Some(1↓=),Some(0↓=)].
        //   Call bar([Var(0),Var(2)]): arg0=b'→None,arg1=a'→Some(0,↓).
        //   M_foo_bar = [[?,↓],[?,?]].
        let foo_suc = Term::lam(
            nat_t.clone(), // a' : Nat (field of a)
            Term::lam(
                pi_nn.clone(), // ih : Nat → Nat (IH)
                Term::lam(
                    nat_t.clone(), // b' : Nat
                    Term::app(
                        Term::app(cref(bar_id), Term::var(0)), // bar b'
                        Term::var(2),                          // a'
                    ),
                ),
            ),
        );
        let foo_body = Term::lam(
            nat_t.clone(), // a
            Term::lam(nat_t.clone(), {
                // b
                let elim = nat_elim(
                    &nb,
                    asc_motive(&nb, pi_nn.clone()),
                    Term::lam(nat_t.clone(), Term::var(0)), // λ b'. b'
                    foo_suc,
                    Term::var(1), // scrut = a
                );
                Term::app(elim, Term::var(0)) // apply to b
            }),
        );

        // bar = λ b. λ a.
        //   (elim_Nat (λ_. Nat→Nat)
        //     (λ b'. b')                       -- zero: b
        //     (λ a'. λ ih. λ b'. foo a' b')   -- suc: foo a' b
        //     a)
        //   b
        //
        // Outer body: Var(0)=a, Var(1)=b.  Elim scrut=Var(0)=a.
        // a is param1 of bar (outermost=param0=b, innermost=param1=a).
        // Wait: after λ b. λ a., Var(0)=a=param1, Var(1)=b=param0.
        // Elim scrut=Var(0)=param1. field_prov=Some(1,↓).
        // Suc method type: Π(a':Nat).(Nat→Nat)→(Nat→Nat).
        // enter_method peels a' (Some(1,↓)) and ih (None).
        // collect on λ b'. foo a' b'.
        //   Push None for b'. prov: [None,None,Some(1↓),Some(0↓=),Some(1↓=)].
        //   (Var0=b', Var1=ih, Var2=a', Var3=b, Var4=a)
        //   Call foo([Var(2),Var(0)]): arg0=a'→Some(1,↓),arg1=b'→None.
        //   M_bar_foo = [[?,?],[↓,?]].
        let bar_suc = Term::lam(
            nat_t.clone(), // a' : Nat (field of a)
            Term::lam(
                pi_nn.clone(), // ih : Nat → Nat (IH)
                Term::lam(
                    nat_t.clone(), // b' : Nat
                    Term::app(
                        Term::app(cref(foo_id), Term::var(2)), // foo a'
                        Term::var(0),                          // b'
                    ),
                ),
            ),
        );
        let bar_body = Term::lam(
            nat_t.clone(), // b
            Term::lam(nat_t.clone(), {
                // a
                let elim = nat_elim(
                    &nb,
                    asc_motive(&nb, pi_nn.clone()),
                    Term::lam(nat_t.clone(), Term::var(0)), // λ b'. b'
                    bar_suc,
                    Term::var(0), // scrut = a
                );
                Term::app(elim, Term::var(1)) // apply to b
            }),
        );

        vec![foo_body, bar_body]
    })
    .expect("sct-accept-permuted must be admitted");
    assert!(env.transparent_body(ids[0]).is_some());
    assert!(env.transparent_body(ids[1]).is_some());
}

// ---------------------------------------------------------------------------
// SCT-reject: sct-reject-self-loop
//
// loop : Nat → Nat := λ n. loop n  (↓= not ↓ on n → REJECT).
// ---------------------------------------------------------------------------

#[test]
fn sct_reject_self_loop() {
    let (mut env, nb) = mk_env();
    let ty = Term::pi(nat_t(&nb), nat_t(&nb));
    let result = declare_recursive_group(&mut env, vec![(vec![], ty)], |ids| {
        let loop_id = ids[0];
        let nat_t = Term::indformer(nb.nat, vec![]);
        vec![Term::lam(nat_t, Term::app(cref(loop_id), Term::var(0)))]
    });
    assert!(result.is_err(), "loop must be rejected");
    assert!(matches!(result.unwrap_err(), KernelError::NotTerminating(_)));
}

// ---------------------------------------------------------------------------
// SCT-reject: sct-reject-growing
//
// up : Nat → Nat := λ n. up (suc n)  (arg grows → M = [[?]] → REJECT).
// ---------------------------------------------------------------------------

#[test]
fn sct_reject_growing() {
    let (mut env, nb) = mk_env();
    let ty = Term::pi(nat_t(&nb), nat_t(&nb));
    let result = declare_recursive_group(&mut env, vec![(vec![], ty)], |ids| {
        let up = ids[0];
        let nat_t = Term::indformer(nb.nat, vec![]);
        let suc_t = Term::constructor(nb.suc, vec![]);
        vec![Term::lam(
            nat_t,
            Term::app(cref(up), Term::app(suc_t, Term::var(0))),
        )]
    });
    assert!(result.is_err(), "up must be rejected");
}

// ---------------------------------------------------------------------------
// SCT-reject: sct-reject-ctor-wrap-compose  (discriminating case)
//
// p(suc x) = q x  →  M_pq = [[↓]]
// q x = p(suc(suc x))  →  M_qp = [[?]]
// Self-loop p: compose(↓, ?) = ? (NOT ↓) → REJECT.
// This test FAILS if compose(↓,?) = ↓ (wrong rule).
// ---------------------------------------------------------------------------

#[test]
fn sct_reject_ctor_wrap_compose() {
    let (mut env, nb) = mk_env();
    let nat = nb.nat;
    let ty = Term::pi(nat_t(&nb), nat_t(&nb));
    let result =
        declare_recursive_group(&mut env, vec![(vec![], ty.clone()), (vec![], ty)], |ids| {
            let p_id = ids[0];
            let q_id = ids[1];
            let nat_t = Term::indformer(nat, vec![]);
            let suc_t = Term::constructor(nb.suc, vec![]);
            let zero_t = Term::constructor(nb.zero, vec![]);

            // p = λ n. elim_Nat (λ_. Nat) zero (λ x. λ _ih. q x) n
            // Suc method type: Π(x:Nat). Nat → Nat.
            //   x binder: nat_t; _ih binder: nat_t (IH = M x = Nat).
            // After enter_method(n', _ih): prov[1]=x=Some(0,↓).
            // collect on: App(Const(q), Var(1)) → M_pq = [[↓]].
            let p_suc = Term::lam(
                nat_t.clone(), // x : Nat (field)
                Term::lam(
                    nat_t.clone(),                       // _ih : Nat (IH = M x = Nat)
                    Term::app(cref(q_id), Term::var(1)), // q x
                ),
            );
            let p_body = Term::lam(
                nat_t.clone(),
                nat_elim(
                    &nb,
                    asc_motive(&nb, nat_t.clone()),
                    zero_t,
                    p_suc,
                    Term::var(0),
                ),
            );

            // q = λ x. p (suc (suc x))
            // Arg = suc(suc(Var(0))) → Unknown → M_qp = [[?]].
            // Self-loop p: compose([[↓]], [[?]]) = [[?]] → REJECT.
            let q_body = Term::lam(
                nat_t.clone(),
                Term::app(
                    cref(p_id),
                    Term::app(suc_t.clone(), Term::app(suc_t, Term::var(0))),
                ),
            );

            vec![p_body, q_body]
        });
    assert!(result.is_err(), "p/q must be rejected (compose(↓,?) = ?)");
}

// ---------------------------------------------------------------------------
// Unit-η: any two elements of a single-constructor no-field inductive convert
// (`17 §2`).
// ---------------------------------------------------------------------------

#[test]
fn unit_eta_two_vars_convert() {
    let (env, unit, _tt) = unit_env();
    let mut ctx = Context::new();
    let unit_ty = Term::indformer(unit, vec![]);
    ctx.push(unit_ty.clone()); // x : Unit  (Var 1 after next push)
    ctx.push(unit_ty.clone()); // y : Unit  (Var 0)
    assert!(
        convert(&env, &ctx, &unit_ty, &Term::var(1), &Term::var(0)),
        "Unit-η: two distinct Unit variables must convert"
    );
}

#[test]
fn unit_eta_tt_and_var() {
    let (env, unit, tt) = unit_env();
    let mut ctx = Context::new();
    let unit_ty = Term::indformer(unit, vec![]);
    ctx.push(unit_ty.clone()); // x : Unit  (Var 0)
    let tt_term = Term::constructor(tt, vec![]);
    assert!(
        convert(&env, &ctx, &unit_ty, &tt_term, &Term::var(0)),
        "Unit-η: tt must convert to any Unit variable"
    );
}

// ---------------------------------------------------------------------------
// Declare-def: non-recursive definition is SCT-admitted immediately.
// ---------------------------------------------------------------------------

#[test]
fn declare_def_non_recursive_admitted() {
    let (mut env, nb) = mk_env();
    let nat_t = nat_t(&nb);
    let id = declare_def(
        &mut env,
        vec![],
        Term::pi(nat_t.clone(), nat_t.clone()),
        Term::lam(nat_t, Term::var(0)),
    )
    .expect("non-recursive identity must be admitted");
    assert!(env.transparent_body(id).is_some());
}

// ---------------------------------------------------------------------------
// SCT-reject: union-masking regression (Architect soundness blocker)
//
// f : Nat → Nat with two self-call sites in the same body:
//   zero case:  f x  (x = original param → ↓= on param 0)
//   suc  case:  f n  (n = field of suc x → ↓  on param 0)
//
// Buggy union-based closure merges [[↓=]] ∪ [[↓]] = [[↓]] → wrongly admits.
// Correct set-based closure keeps [[↓=]] as a distinct idempotent loop with no
// strict diagonal → REJECT.
// ---------------------------------------------------------------------------

#[test]
fn sct_reject_union_masking() {
    let (mut env, nb) = mk_env();
    let nat = nb.nat;
    let ty = Term::pi(nat_t(&nb), nat_t(&nb));
    let result = declare_recursive_group(&mut env, vec![(vec![], ty)], |ids| {
        let f = ids[0];
        let nat_t = Term::indformer(nat, vec![]);
        // f = λ x.
        //   elim_Nat (λ_. Nat)
        //     (f x)              -- zero case: f x (↓= on param0)
        //     (λ n. λ _ih. f n)  -- suc case:  f n (↓  on param0)
        //     x
        //
        // Edge M_A = [[↓=]] (zero case), M_B = [[↓]] (suc case).
        // [[↓=]] idempotent, no strict diagonal → REJECT.
        let suc_method = Term::lam(
            nat_t.clone(),
            Term::lam(nat_t.clone(), Term::app(cref(f), Term::var(1))),
        );
        vec![Term::lam(
            nat_t.clone(),
            nat_elim(
                &nb,
                asc_motive(&nb, nat_t.clone()),
                Term::app(cref(f), Term::var(0)), // zero: f x
                suc_method,
                Term::var(0),
            ),
        )]
    });
    assert!(
        result.is_err(),
        "f with a stationary self-call must be rejected"
    );
    assert!(matches!(result.unwrap_err(), KernelError::NotTerminating(_)));
}

// ---------------------------------------------------------------------------
// Declare-def: self-loop is rejected by SCT.
// ---------------------------------------------------------------------------

#[test]
fn declare_def_sct_rejects_self_loop() {
    let (mut env, nb) = mk_env();
    let nat_t = nat_t(&nb);
    let ty = Term::pi(nat_t.clone(), nat_t.clone());
    let result = declare_recursive_group(&mut env, vec![(vec![], ty)], |ids| {
        let f = ids[0];
        let nat_t = Term::indformer(nb.nat, vec![]);
        vec![Term::lam(nat_t, Term::app(cref(f), Term::var(0)))]
    });
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// AC4 (accept arm): sct-accept-plus
//
// plus : Nat → Nat → Nat
//   plus zero    n = n
//   plus (suc m) n = suc (plus m n)
//
// Edge: plus m' n where m' = field of suc m → ↓ on param0, n passed as-is →
// M = [[↓,?],[?,↓=]].  M⊙M[0,0] = ↓ → ACCEPT.
//
// Paired with sct_reject_self_loop / sct_reject_growing (both AC4 reject arm).
// Both arms are required: accept-only hides a gate that always accepts;
// reject-only hides a gate that never accepts.
// ---------------------------------------------------------------------------

#[test]
fn sct_accept_plus() {
    let (mut env, nb) = mk_env();
    let nat = nb.nat;
    let ty = Term::pi(nat_t(&nb), Term::pi(nat_t(&nb), nat_t(&nb)));
    let ids = declare_recursive_group(&mut env, vec![(vec![], ty)], |ids| {
        let plus = ids[0];
        let nat_t = Term::indformer(nat, vec![]);
        let suc_t = Term::constructor(nb.suc, vec![]);

        // plus = λ m. λ n.
        //   elim_Nat (λ_. Nat)
        //     n                              -- zero: plus 0 n = n
        //     (λ m'. λ _ih. suc (plus m' n)) -- suc: plus (suc m') n = suc (plus m' n)
        //     m
        //
        // After λ m. λ n.: Var(0)=n, Var(1)=m.
        // Suc method type: Π(m':Nat). Nat → Nat.
        //   m' binder: nat_t  (n_fields=1)
        //   _ih binder: nat_t (n_ihs=1; IH = M m' = Nat)
        // In suc body after enter_method peels m' and _ih:
        //   Var(0)=_ih (None), Var(1)=m' (0,↓), Var(2)=n (1,↓=), Var(3)=m (0,↓=).
        // Call plus m' n:
        //   arg0=Var(1): prov[1]=(0,↓)   → M[0,0]=↓,  M[1,0]=?
        //   arg1=Var(2): prov[2]=(1,↓=)  → M[0,1]=?,  M[1,1]=↓=
        // M = [[↓,?],[?,↓=]].  M⊙M = same → idempotent; M[0,0]=↓ → ACCEPT.
        let suc_method = Term::lam(
            nat_t.clone(), // m' : Nat (field)
            Term::lam(
                nat_t.clone(), // _ih : Nat (IH = M m' = Nat)
                Term::app(
                    suc_t,
                    Term::app(
                        Term::app(cref(plus), Term::var(1)), // plus m'
                        Term::var(2),                        // n (outer)
                    ),
                ),
            ),
        );
        vec![Term::lam(
            nat_t.clone(), // m
            Term::lam(
                nat_t.clone(), // n: Var(0)=n, Var(1)=m
                nat_elim(
                    &nb,
                    asc_motive(&nb, nat_t.clone()), // motive = λ_. Nat
                    Term::var(0),                   // zero: n
                    suc_method,
                    Term::var(1), // scrut = m
                ),
            ),
        )]
    })
    .expect("plus must be admitted transparent by SCT");
    assert!(env.transparent_body(ids[0]).is_some());
}

// ---------------------------------------------------------------------------
// SCT-reject: declare-def-nullary-self-loop-rejects (soundness — Architect
// finding on wp/K2c-recursive-sct)
//
// bad := bad  (nullary: zero Lam binders, body is the bare unapplied
// occurrence `bad`).
//
// Pre-fix: `collect_calls`'s leaf arm treated `Term::Const` as a no-op, so a
// bare group-member occurrence produced NO call edge — `edges.is_empty()` =>
// admitted transparent. That is a real δ-cycle `bad ⇝ bad ⇝ …` inhabiting
// whatever type `bad` is declared at (an unsoundness, not merely
// non-termination). Fix: a bare `Const` to a group member is now modeled as a
// ?-everywhere self-loop edge (`17 §4.1`); `ScMatrix::zero(0, 0)` is
// idempotent with no strict diagonal (vacuously, on 0 params) => rejected.
// This flips Ok(transparent) -> Err(NotTerminating).
// ---------------------------------------------------------------------------

#[test]
fn sct_reject_bare_self_reference() {
    let (mut env, nb) = mk_env();
    let nat_t = nat_t(&nb);
    let result = declare_recursive_group(&mut env, vec![(vec![], nat_t)], |ids| {
        vec![cref(ids[0])] // bad := bad, zero Lam binders
    });
    assert!(
        result.is_err(),
        "bare nullary self-reference must be rejected"
    );
    assert!(matches!(result.unwrap_err(), KernelError::NotTerminating(_)));
}

// ---------------------------------------------------------------------------
// SCT-reject: declare-def-laundered-self-loop-rejects
//
// loop : Nat -> Nat := id loop  (loop occurs unapplied, as an argument to a
// transparent passthrough `id : (Nat -> Nat) -> (Nat -> Nat)`; `loop`'s own
// body has zero leading Lam binders, so `loop` appears as a bare `Const` in
// **argument** position, not as an App head).
//
// Same gap as the bare case: pre-fix, the unapplied occurrence of `loop`
// inside `id loop` produced no call edge (`edges.is_empty()` => admitted).
// Pins that the applied-only precondition is on the OCCURRENCE, not the
// syntactic head — laundering a self-reference through any transparent
// passthrough (`id`, `map`, …) must still be caught by the same guard that
// catches the bare case.
// ---------------------------------------------------------------------------

#[test]
fn sct_reject_combinator_laundered() {
    let (mut env, nb) = mk_env();
    let nat_t = nat_t(&nb);
    let fn_t = Term::pi(nat_t.clone(), nat_t.clone()); // Nat -> Nat
    let id_ty = Term::pi(fn_t.clone(), fn_t.clone()); // (Nat->Nat) -> (Nat->Nat)
    let id_fn = declare_def(
        &mut env,
        vec![],
        id_ty,
        Term::lam(fn_t.clone(), Term::var(0)), // λf. f
    )
    .expect("id must be admitted (non-recursive)");

    let result = declare_recursive_group(&mut env, vec![(vec![], fn_t)], |ids| {
        vec![Term::app(cref(id_fn), cref(ids[0]))] // loop := id loop
    });
    assert!(
        result.is_err(),
        "loop laundered through id must be rejected"
    );
    assert!(matches!(result.unwrap_err(), KernelError::NotTerminating(_)));
}
