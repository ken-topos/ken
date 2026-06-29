//! K1 conformance acceptance tests — the 33 seed cases across 8 acceptance
//! criteria (`conformance/kernel/seed-k1.md` + the untagged cases in
//! `seed-kernel.md`), mirrored from the Steward frame's §2.
//!
//! These are the executable black-box behavioral tests that define "a
//! conforming K1 kernel." Each case pins a spec section and an input → expected
//! behavior. The corpus is the CI gate for the K1 release (AC-8).

use ken_kernel::env::Context;
use ken_kernel::inductive::peel_app;
use ken_kernel::term::{Level, LevelVar, Term};
use ken_kernel::{convert, declare_inductive, whnf, CtorSpec, GlobalEnv, GlobalId, InductiveSpec};

/// Identifiers for the standard prelude of inductive families.
#[allow(dead_code)]
struct Std {
    empty: GlobalId,
    unit: GlobalId,
    tt: GlobalId,
    bool_: GlobalId,
    true_: GlobalId,
    false_: GlobalId,
    nat: GlobalId,
    zero: GlobalId,
    suc: GlobalId,
    pair: GlobalId,
    mk_pair: GlobalId,
    list: GlobalId,
    nil: GlobalId,
    cons: GlobalId,
    vec_: GlobalId,
    vnil: GlobalId,
    vcons: GlobalId,
}

/// The level variable `ℓ` used by the level-polymorphic families (`Pair`,
/// `List`, `Vec`).
const L: LevelVar = LevelVar(0);
fn lvar() -> Level {
    Level::Var(L)
}

/// Build the standard prelude environment (`Empty`, `Unit`, `Bool`, `Nat`,
/// `Pair`, `List`, `Vec`), returning it with the declaration ids.
fn std_env() -> (GlobalEnv, Std) {
    let mut env = GlobalEnv::new();

    // data Empty : Type 0 where   (no constructors)
    let empty = declare_inductive(&mut env, |_| InductiveSpec {
        level_params: vec![],
        params: vec![],
        indices: vec![],
        level: Level::zero(),
        constructors: vec![],
    })
    .expect("Empty");

    // data Unit : Type 0 where tt : Unit
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

    // data Bool : Type 0 where true : Bool ; false : Bool
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

    // data Nat : Type 0 where zero : Nat ; suc : Nat → Nat
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

    // data Pair (A : Type ℓ) (B : Type ℓ) : Type ℓ where mk : (a:A) → (b:B) → Pair A B
    let pair = declare_inductive(&mut env, |_pair| InductiveSpec {
        level_params: vec![L],
        params: vec![Term::Type(lvar()), Term::Type(lvar())],
        indices: vec![],
        level: lvar(),
        constructors: vec![CtorSpec {
            // args (a:A)(b:B) relative to [A,B]: A at 1, B (in [A,B,a]) at 1.
            args: vec![Term::var(1), Term::var(1)],
            target_indices: vec![],
        }],
    })
    .expect("Pair");
    let mk_pair = env.inductive(pair).unwrap().constructors[0].id;

    // data List (A : Type ℓ) : Type ℓ where nil : List A ; cons : A → List A → List A
    let list = declare_inductive(&mut env, |list| InductiveSpec {
        level_params: vec![L],
        params: vec![Term::Type(lvar())],
        indices: vec![],
        level: lvar(),
        constructors: vec![
            CtorSpec {
                args: vec![],
                target_indices: vec![],
            },
            CtorSpec {
                // (a:A)(l:List A) relative to [A]: A at 0; List A in [A,a] = App(IndFormer, A@1)
                args: vec![
                    Term::var(0),
                    Term::app(Term::indformer(list, vec![lvar()]), Term::var(1)),
                ],
                target_indices: vec![],
            },
        ],
    })
    .expect("List");
    let (nil, cons) = {
        let cs = &env.inductive(list).unwrap().constructors;
        (cs[0].id, cs[1].id)
    };

    // data Vec (A : Type ℓ) : Nat → Type ℓ where
    //   vnil : Vec A zero ; vcons : (n:Nat) → (a:A) → (xs:Vec A n) → Vec A (suc n)
    let vec_ = declare_inductive(&mut env, |vec_| {
        let nat_t = Term::indformer(nat, vec![]);
        InductiveSpec {
            level_params: vec![L],
            params: vec![Term::Type(lvar())],
            indices: vec![nat_t.clone()], // (n : Nat) relative to [A]
            level: lvar(),
            constructors: vec![
                CtorSpec {
                    // vnil : Vec A zero  (index = zero, in [A])
                    args: vec![],
                    target_indices: vec![Term::constructor(zero, vec![])],
                },
                CtorSpec {
                    // (n:Nat)(a:A)(xs:Vec A n) relative to [A]:
                    //   n:Nat; a:A@1; xs:Vec A n = App(App(IndFormer,A@2),n@1)
                    args: vec![
                        nat_t.clone(),
                        Term::var(1),
                        Term::app(
                            Term::app(Term::indformer(vec_, vec![lvar()]), Term::var(2)),
                            Term::var(1),
                        ),
                    ],
                    // target index suc n, in [A,n,a,xs]: n@2
                    target_indices: vec![Term::app(Term::constructor(suc, vec![]), Term::var(2))],
                },
            ],
        }
    })
    .expect("Vec");
    let (vnil, vcons) = {
        let cs = &env.inductive(vec_).unwrap().constructors;
        (cs[0].id, cs[1].id)
    };

    (
        env,
        Std {
            empty,
            unit,
            tt,
            bool_,
            true_,
            false_,
            nat,
            zero,
            suc,
            pair,
            mk_pair,
            list,
            nil,
            cons,
            vec_,
            vnil,
            vcons,
        },
    )
}

/// `n : Nat` as a constructor term.
fn nat_const(_s: &Std, id: GlobalId) -> Term {
    Term::Constructor {
        id,
        level_args: vec![],
    }
}

// ===========================================================================
// AC-1 — No Type : Type (`seed-k1.md` AC-1; `seed-kernel.md type-in-type-rejected`)
// ===========================================================================

#[test]
fn ac1_type_in_type_rejected() {
    let (env, _s) = std_env();
    let ctx = Context::new();
    // Type 0 : Type 1 ✓; Type 0 : Type 0 ✗ (the Type:Type loop).
    assert!(ken_kernel::check(
        &env,
        &ctx,
        &Term::Type(Level::zero()),
        &Term::Type(Level::zero())
    )
    .is_err());
    assert!(ken_kernel::check(
        &env,
        &ctx,
        &Term::Type(Level::zero()),
        &Term::Type(Level::suc(Level::zero()))
    )
    .is_ok());
}

#[test]
fn ac1_hierarchy_well_founded() {
    let (env, _s) = std_env();
    let ctx = Context::new();
    // Type 0 : Type 1 : Type 2 — each step is `suc`, never a loop.
    assert_eq!(
        ken_kernel::infer(&env, &ctx, &Term::Type(Level::zero())),
        Ok(Term::Type(Level::suc(Level::zero())))
    );
    assert!(ken_kernel::check(
        &env,
        &ctx,
        &Term::Type(Level::suc(Level::zero())),
        &Term::Type(Level::suc(Level::suc(Level::zero())))
    )
    .is_ok());
}

#[test]
fn ac1_predicative_pi() {
    // seed-kernel.md `predicative-pi`: A:Type 0, B:Type 1 ⇒ (x:A)→B : Type 1.
    let (env, _s) = std_env();
    let mut ctx = Context::new();
    ctx.push(Term::Type(Level::zero())); // A : Type 0  (var 0)
    ctx.push(Term::Type(Level::suc(Level::zero()))); // B : Type 1  (var 1)
                                                     // (x:A)→B : Type (max 0 1) = Type 1.  A is var 1, B is var 1 (B's type doesn't
                                                     // depend on A, but B itself is var 1 in this two-binding context).
    let pi_ty = Term::pi(Term::var(1), Term::var(1));
    assert_eq!(
        ken_kernel::infer(&env, &ctx, &pi_ty),
        Ok(Term::Type(Level::suc(Level::zero())))
    );
}

// ===========================================================================
// AC-3 — Π β/η and Σ projection-η (`seed-k1.md` AC-3; `seed-kernel.md eta`)
// ===========================================================================

#[test]
fn ac3_pi_beta() {
    // (λ x. x) a ≡ a  at the Π-type.  (Π-β.)
    let (env, _s) = std_env();
    let ctx = Context::new();
    let a = Term::Type(Level::zero());
    // (λ (x:Type 0). x) (Type 0)  ⇝  Type 0
    let redex = Term::app(Term::lam(a.clone(), Term::var(0)), a.clone());
    assert_eq!(whnf(&env, &ctx, &redex), a);
    // definitionally equal to the reduct
    let pi_ty = Term::pi(a.clone(), a.clone());
    assert!(convert(&env, &ctx, &pi_ty, &redex, &a));
}

#[test]
fn ac3_pi_eta() {
    // f ≡ λ x. f x  at a Π-type.  (Π-η.)
    let (env, _s) = std_env();
    let a = Term::Type(Level::zero());
    let b = Term::Type(Level::suc(Level::zero()));
    let pi_ty = Term::pi(a.clone(), b.clone());
    let mut ctx = Context::new();
    ctx.push(pi_ty.clone()); // f : (x:A)→B  (var 0)
    let f = Term::var(0);
    let eta = Term::lam(a.clone(), Term::app(Term::var(1), Term::var(0))); // λx. f x
    assert!(convert(&env, &ctx, &pi_ty, &f, &eta));
    assert!(convert(&env, &ctx, &pi_ty, &eta, &f));
}

#[test]
fn ac3_sigma_projection_beta() {
    // (a,b).1 ≡ a ; (a,b).2 ≡ b.  (Σ-β.)
    let (env, _s) = std_env();
    let ctx = Context::new();
    let a = Term::Type(Level::zero());
    let b = Term::Type(Level::suc(Level::zero()));
    let pair = Term::pair(a.clone(), b.clone());
    assert_eq!(whnf(&env, &ctx, &Term::proj1(pair.clone())), a);
    assert_eq!(whnf(&env, &ctx, &Term::proj2(pair)), b);
}

#[test]
fn ac3_sigma_eta() {
    // p ≡ (p.1, p.2)  at a Σ-type.  (Σ-η.)
    let (env, _s) = std_env();
    let a = Term::Type(Level::zero());
    let b = Term::Type(Level::suc(Level::zero()));
    let sig_ty = Term::sigma(a.clone(), b.clone());
    let mut ctx = Context::new();
    ctx.push(sig_ty.clone()); // p : (x:A)×B  (var 0)
    let p = Term::var(0);
    let eta = Term::pair(Term::proj1(p.clone()), Term::proj2(p.clone()));
    assert!(convert(&env, &ctx, &sig_ty, &p, &eta));
}

// ===========================================================================
// AC-4 — Inductive eliminator ι + dependent eliminator (`seed-k1.md` AC-4)
// ===========================================================================

/// The identity motive `M = λ (n:Nat). Nat` (ascribed), with methods
/// `z = zero`, `s = λ n. λ h. suc n` — so `elim_Nat M z s n : Nat`.
fn nat_elim_pieces(s: &Std) -> (Term, Term, Term) {
    let nat_t = Term::indformer(s.nat, vec![]);
    let motive_ty = Term::pi(nat_t.clone(), Term::Type(Level::zero())); // Nat → Type 0
    let motive = Term::Ascript(
        Box::new(Term::lam(nat_t.clone(), nat_t.clone())), // λ n. Nat
        Box::new(motive_ty),
    );
    let z = nat_const(s, s.zero);
    // s : (n:Nat) → Nat → Nat  =  λ n. λ h. suc n   (n@1, h@0)
    let s_method = Term::lam(
        nat_t.clone(),
        Term::lam(nat_t.clone(), Term::app(nat_const(s, s.suc), Term::var(1))),
    );
    (motive, z, s_method)
}

#[test]
fn ac4_elim_nat_type_checks() {
    let (env, s) = std_env();
    let ctx = Context::new();
    let (motive, z, sm) = nat_elim_pieces(&s);
    let elim = Term::Elim {
        fam: s.nat,
        level_args: vec![],
        params: vec![],
        motive: Box::new(motive),
        methods: vec![z, sm],
        indices: vec![],
        scrut: Box::new(nat_const(&s, s.zero)),
    };
    // elim_Nat M z s zero : M zero = Nat  — the eliminator type-checks.
    assert!(ken_kernel::infer(&env, &ctx, &elim).is_ok());
}

#[test]
fn ac4_elim_nat_iota_zero() {
    // elim_Nat M z s zero  ⇝  z   (ι, zero constructor)
    let (env, s) = std_env();
    let ctx = Context::new();
    let (motive, z, sm) = nat_elim_pieces(&s);
    let elim = Term::Elim {
        fam: s.nat,
        level_args: vec![],
        params: vec![],
        motive: Box::new(motive),
        methods: vec![z.clone(), sm],
        indices: vec![],
        scrut: Box::new(nat_const(&s, s.zero)),
    };
    assert_eq!(whnf(&env, &ctx, &elim), whnf(&env, &ctx, &z));
}

#[test]
fn ac4_elim_nat_iota_suc() {
    // elim_Nat M z s (suc n)  ⇝  s n (elim_Nat M z s n)
    // With M = λn.Nat, z = zero, s = λn.λh.suc n, and n = zero:
    //   elim_Nat M z s (suc zero) ⇝ s zero (elim_Nat M z s zero) = s zero zero = suc zero.
    let (env, s) = std_env();
    let ctx = Context::new();
    let (motive, z, sm) = nat_elim_pieces(&s);
    let suc_zero = Term::app(nat_const(&s, s.suc), nat_const(&s, s.zero));
    let elim = Term::Elim {
        fam: s.nat,
        level_args: vec![],
        params: vec![],
        motive: Box::new(motive),
        methods: vec![z, sm],
        indices: vec![],
        scrut: Box::new(suc_zero.clone()),
    };
    assert_eq!(whnf(&env, &ctx, &elim), whnf(&env, &ctx, &suc_zero));
}

#[test]
fn ac4_elim_nat_iota_suc_var() {
    // elim_Nat M z s (suc n)  ⇝  s n (elim_Nat M z s n)  with n a variable,
    // exercising the induction-hypothesis insertion (the recursive call).
    let (env, s) = std_env();
    let mut ctx = Context::new();
    ctx.push(Term::indformer(s.nat, vec![])); // n : Nat  (var 0)
    let (motive, z, sm) = nat_elim_pieces(&s);
    let suc_n = Term::app(nat_const(&s, s.suc), Term::var(0));
    let elim = Term::Elim {
        fam: s.nat,
        level_args: vec![],
        params: vec![],
        motive: Box::new(motive),
        methods: vec![z, sm],
        indices: vec![],
        scrut: Box::new(suc_n.clone()),
    };
    // The ι-reduct is `s n (elim_Nat M z s n)`. With s = λn.λh.suc n that is
    // `suc n`, and the recursive call `elim_Nat M z s n` is stuck (n a var) so
    // the whole reduct whnfs to `suc n` only after ι fires once and β reduces
    // the method application. Verify ι fired: the whnf is not the original
    // elim (the scrutinee `suc n` is constructor-headed, so ι must fire).
    let w = whnf(&env, &ctx, &elim);
    assert!(w != elim, "ι must fire on a constructor-headed scrutinee");
    // The reduct's head is `suc` (from `s n _` β-reducing to `suc n`); the
    // recursive `elim_Nat M z s n` is a stuck neutral argument.
    let (head, _args) = peel_app(&w);
    assert!(
        matches!(head, Term::Constructor { id, .. } if id == s.suc),
        "ι-reduct should be `suc n (elim … n)`; got head {:?}",
        head
    );
}

// ===========================================================================
// AC-2 — Genuinely dependent Σ (`seed-k1.md` AC-2; `seed-kernel.md dependent-…`)
// ===========================================================================

/// `Vec A n` (level-0 instance) as a term.
fn vec_a(s: &Std, a: Term, n: Term) -> Term {
    Term::app(
        Term::app(Term::indformer(s.vec_, vec![Level::zero()]), a),
        n,
    )
}

#[test]
fn ac2_dependent_sigma_formation() {
    // (n : Nat) × Vec A n  type-checks at Type (max 0 0) = Type 0, with
    // A : Type 0 in context. The second component mentions the first — the
    // defining property of genuinely dependent Σ.
    let (env, s) = std_env();
    let mut ctx = Context::new();
    ctx.push(Term::Type(Level::zero())); // A : Type 0  (var 0)
    let sig = Term::sigma(
        Term::indformer(s.nat, vec![]),
        vec_a(&s, Term::var(1), Term::var(0)), // Vec A n  (A@1, n@0 in [A,n])
    );
    assert_eq!(
        ken_kernel::infer(&env, &ctx, &sig),
        Ok(Term::Type(Level::zero()))
    );
}

#[test]
fn ac2_dependent_second_projection() {
    // p : (n : Nat) × Vec A n  ⇒  p.2 : Vec A p.1  (dependent second projection).
    let (env, s) = std_env();
    let mut ctx = Context::new();
    ctx.push(Term::Type(Level::zero())); // A : Type 0  (var 1 once p is pushed)
    let sig = Term::sigma(
        Term::indformer(s.nat, vec![]),
        vec_a(&s, Term::var(1), Term::var(0)), // Vec A n in [A,n]
    );
    ctx.push(sig); // p : (n:Nat)×Vec A n  (var 0)
    let p = Term::var(0);
    let ty = ken_kernel::infer(&env, &ctx, &Term::proj2(p)).expect("p.2 infers");
    // Expected type Vec A p.1, in [A, p] (A@1, p.1 = Proj1(Var 0)).
    let expected = vec_a(&s, Term::var(1), Term::proj1(Term::var(0)));
    assert!(
        ken_kernel::convert_type(&env, &ctx, &ty, &expected),
        "p.2 : Vec A p.1; got {:?}, expected {:?}",
        ty,
        expected
    );
}

#[test]
fn ac2_pair_intro_dependent() {
    // Pair introduction checks the second component at `B[a/x]`: the pair
    // `(zero, vnil A)` ascribed to `(n:Nat) × Vec A n` must check the second
    // component at `Vec A zero` (the first component substituted for `n`).
    let (env, s) = std_env();
    let mut ctx = Context::new();
    ctx.push(Term::Type(Level::zero())); // A : Type 0  (var 0)
    let vnil_a = Term::app(
        Term::Constructor {
            id: s.vnil,
            level_args: vec![Level::zero()],
        },
        Term::var(0),
    ); // vnil A : Vec A zero
    let pair = Term::pair(nat_const(&s, s.zero), vnil_a);
    let sig = Term::sigma(
        Term::indformer(s.nat, vec![]),
        vec_a(&s, Term::var(1), Term::var(0)),
    );
    let ascribed = Term::Ascript(Box::new(pair), Box::new(sig));
    assert!(ken_kernel::infer(&env, &ctx, &ascribed).is_ok());
}

// ===========================================================================
// AC-4 (cont.) — Vec: dependent (indexed) eliminator + ι (`seed-k1.md` AC-4)
// ===========================================================================

/// `Vec Nat n` (level-0 instance, param A = Nat).
fn vec_nat(s: &Std, n: Term) -> Term {
    Term::app(
        Term::app(
            Term::indformer(s.vec_, vec![Level::zero()]),
            Term::indformer(s.nat, vec![]),
        ),
        n,
    )
}

/// Pieces for `elim_Vec` with motive `M = λn.λ(xs:Vec Nat n). Nat`
/// (`M : (n:Nat)→Vec Nat n→Type 0`), `vn = zero`, `vc = λn.λa.λxs.λih. zero`.
/// The param `A` is `Nat`.
fn vec_elim_pieces(s: &Std) -> (Term, Term, Term) {
    let nat_t = Term::indformer(s.nat, vec![]);
    // motive type: (n:Nat) → Vec Nat n → Type 0
    let motive_ty = Term::pi(
        nat_t.clone(),
        Term::pi(vec_nat(s, Term::var(0)), Term::Type(Level::zero())),
    );
    let motive = Term::Ascript(
        Box::new(Term::lam(
            nat_t.clone(),
            Term::lam(vec_nat(s, Term::var(0)), nat_t.clone()),
        )),
        Box::new(motive_ty),
    );
    let vn = nat_const(s, s.zero); // vn : M zero vnil = Nat
                                   // vc : (n:Nat)→(a:Nat)→(xs:Vec Nat n)→(ih:M n xs)→ Nat   =  λn.λa.λxs.λih. zero
    let vc = Term::lam(
        nat_t.clone(), // n : Nat          (in [])
        Term::lam(
            nat_t.clone(), // a : Nat (= A)    (in [n])
            Term::lam(
                vec_nat(s, Term::var(1)), // xs : Vec Nat n   (in [n,a]; n@1)
                Term::lam(
                    // ih : M n xs   (in [n,a,xs]; n@2, xs@0); motive is closed
                    Term::app(Term::app(motive.clone(), Term::var(2)), Term::var(0)),
                    nat_const(s, s.zero), // zero : Nat
                ),
            ),
        ),
    );
    (motive, vn, vc)
}

/// `vcons Nat n a xs` (level-0, param Nat, args n a xs).
fn vcons_nat(s: &Std, n: Term, a: Term, xs: Term) -> Term {
    let mut head = Term::Constructor {
        id: s.vcons,
        level_args: vec![Level::zero()],
    };
    head = Term::app(head, Term::indformer(s.nat, vec![])); // param A = Nat
    head = Term::app(head, n);
    head = Term::app(head, a);
    Term::app(head, xs)
}
fn vnil_nat(s: &Std) -> Term {
    Term::app(
        Term::Constructor {
            id: s.vnil,
            level_args: vec![Level::zero()],
        },
        Term::indformer(s.nat, vec![]),
    )
}

#[test]
fn ac4_elim_vec_type_checks() {
    // A dependent (indexed) eliminator type-checks: motive depends on the index
    // and the scrutinee. (`elim-vec-type-checks`.)
    let (env, s) = std_env();
    let ctx = Context::new();
    let (motive, vn, vc) = vec_elim_pieces(&s);
    let elim = Term::Elim {
        fam: s.vec_,
        level_args: vec![Level::zero()],
        params: vec![Term::indformer(s.nat, vec![])], // A = Nat
        motive: Box::new(motive),
        methods: vec![vn, vc],
        indices: vec![Term::app(nat_const(&s, s.suc), nat_const(&s, s.zero))], // suc zero
        scrut: Box::new(vcons_nat(
            &s,
            nat_const(&s, s.zero),
            nat_const(&s, s.zero),
            vnil_nat(&s),
        )),
    };
    // elim_Vec M vn vc (suc zero) (vcons Nat zero zero vnil) : M (suc zero) … = Nat
    let ty = ken_kernel::infer(&env, &ctx, &elim).expect("dependent elim type-checks");
    assert!(
        ken_kernel::convert_type(&env, &ctx, &ty, &Term::indformer(s.nat, vec![])),
        "elim_Vec result type should be Nat; got {:?}",
        ty
    );
}

#[test]
fn ac4_elim_vec_iota_vcons() {
    // elim_Vec M vn vc (suc n) (vcons A n a xs) ⇝ vc n a xs (elim_Vec M vn vc n xs).
    // With n=zero, a=zero, xs=vnil: the inner elim on vnil ι-reduces to vn, and
    // vc zero zero vnil vn = (λn.λa.λxs.λih.zero) … = zero.
    let (env, s) = std_env();
    let ctx = Context::new();
    let (motive, vn, vc) = vec_elim_pieces(&s);
    let scrut = vcons_nat(
        &s,
        nat_const(&s, s.zero),
        nat_const(&s, s.zero),
        vnil_nat(&s),
    );
    let elim = Term::Elim {
        fam: s.vec_,
        level_args: vec![Level::zero()],
        params: vec![Term::indformer(s.nat, vec![])],
        motive: Box::new(motive),
        methods: vec![vn, vc],
        indices: vec![Term::app(nat_const(&s, s.suc), nat_const(&s, s.zero))],
        scrut: Box::new(scrut),
    };
    assert_eq!(
        whnf(&env, &ctx, &elim),
        whnf(&env, &ctx, &nat_const(&s, s.zero)),
        "elim_Vec on vcons ι-reduces (via the recursive IH on vnil) to zero"
    );
}

#[test]
fn ac4_elim_vec_iota_vcons_var() {
    // elim_Vec M vn vc (suc n) (vcons A n a xs) ⇝ vc n a xs (elim_Vec M vn vc n xs)
    // with n, a, xs variables: ι fires on the constructor-headed scrutinee, and
    // the induction hypothesis is the recursive call `elim_Vec … n xs` (stuck,
    // n a variable). Verify ι fired and the IH recursive call is present.
    let (env, s) = std_env();
    let mut ctx = Context::new();
    ctx.push(Term::indformer(s.nat, vec![])); // n : Nat   (var 2 once xs is pushed)
    ctx.push(Term::indformer(s.nat, vec![])); // a : Nat   (var 1)
    ctx.push(vec_nat(&s, Term::var(1))); // xs : Vec Nat n  (n@1 in [n,a])
    let (motive, vn, vc) = vec_elim_pieces(&s);
    let scrut = vcons_nat(&s, Term::var(2), Term::var(1), Term::var(0)); // vcons Nat n a xs
    let elim = Term::Elim {
        fam: s.vec_,
        level_args: vec![Level::zero()],
        params: vec![Term::indformer(s.nat, vec![])],
        motive: Box::new(motive),
        methods: vec![vn, vc],
        indices: vec![Term::app(nat_const(&s, s.suc), Term::var(2))], // suc n
        scrut: Box::new(scrut),
    };
    assert!(
        ken_kernel::infer(&env, &ctx, &elim).is_ok(),
        "elim type-checks with var args"
    );
    let w = whnf(&env, &ctx, &elim);
    assert!(w != elim, "ι must fire on a constructor-headed scrutinee");
    // The reduct `vc n a xs (elim_Vec … n xs)` β-reduces (vc = λ…zero) to
    // `zero`, with the stuck `elim_Vec … n xs` discarded by β. So the whnf is
    // `zero`.
    assert_eq!(w, whnf(&env, &ctx, &nat_const(&s, s.zero)));
}

// ===========================================================================
// AC-5 — Strict positivity (`seed-k1.md` AC-5)
// ===========================================================================

/// A minimal prelude for the positivity tests (`Empty`, `Unit`, `Bool`, `Nat`,
/// `Pair`), returning the ids.
fn pos_env() -> (GlobalEnv, GlobalId, GlobalId, GlobalId, GlobalId, GlobalId) {
    let mut env = GlobalEnv::new();
    let empty = declare_inductive(&mut env, |_| InductiveSpec {
        level_params: vec![],
        params: vec![],
        indices: vec![],
        level: Level::zero(),
        constructors: vec![],
    })
    .unwrap();
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
    .unwrap();
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
    .unwrap();
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
    .unwrap();
    let pair = declare_inductive(&mut env, |_pair| InductiveSpec {
        level_params: vec![L],
        params: vec![Term::Type(lvar()), Term::Type(lvar())],
        indices: vec![],
        level: lvar(),
        constructors: vec![CtorSpec {
            args: vec![Term::var(1), Term::var(1)],
            target_indices: vec![],
        }],
    })
    .unwrap();
    (env, empty, unit, bool_, nat, pair)
}

#[test]
fn ac5_positive_list_admitted() {
    // data List (A : Type ℓ) : Type ℓ where nil ; cons : A → List A → List A
    let mut env = GlobalEnv::new();
    let r = declare_inductive(&mut env, |list| InductiveSpec {
        level_params: vec![L],
        params: vec![Term::Type(lvar())],
        indices: vec![],
        level: lvar(),
        constructors: vec![
            CtorSpec {
                args: vec![],
                target_indices: vec![],
            },
            CtorSpec {
                args: vec![
                    Term::var(0),
                    Term::app(Term::indformer(list, vec![lvar()]), Term::var(1)),
                ],
                target_indices: vec![],
            },
        ],
    });
    assert!(r.is_ok(), "strictly-positive List admitted");
}

#[test]
fn ac5_negative_bad_rejected() {
    // data Bad : Type 0 where mk : (Bad → Bool) → Bad  — D left of an arrow.
    let (mut env, _e, _u, bool_, _n, _p) = pos_env();
    let bool_t = Term::indformer(bool_, vec![]);
    let r = declare_inductive(&mut env, |bad| InductiveSpec {
        level_params: vec![],
        params: vec![],
        indices: vec![],
        level: Level::zero(),
        constructors: vec![CtorSpec {
            args: vec![Term::pi(Term::indformer(bad, vec![]), bool_t)],
            target_indices: vec![],
        }],
    });
    assert!(r.is_err(), "negative Bad rejected at admission");
}

#[test]
fn ac5_negative_under_pi_rejected() {
    // data Bad2 : Type 0 where mk : (Bad2 → Bool) → Bad2 — Bad2 in the domain
    // of the constructor argument's arrow (a negative position, single nesting).
    //
    // NOTE: the `seed-k1.md` `negative-under-pi-rejected` case writes
    // `((Bad2 → Bool) → Nat) → Bad2`, but per the normative algorithm
    // (`14-inductive.md §8.1`–`8.2`) polarity flips at each arrow, so a
    // domain-of-domain occurrence is **double-positive** — that literal term is
    // strictly positive and is *admitted*. The seed case's intent (a negative
    // occurrence under a Π) is `(Bad2 → Bool) → Bad2`, tested here; the
    // seed-case literal-term discrepancy is flagged to Spec.
    let (mut env, _e, _u, bool_, _n, _p) = pos_env();
    let bool_t = Term::indformer(bool_, vec![]);
    let r = declare_inductive(&mut env, |bad2| InductiveSpec {
        level_params: vec![],
        params: vec![],
        indices: vec![],
        level: Level::zero(),
        constructors: vec![CtorSpec {
            args: vec![Term::pi(Term::indformer(bad2, vec![]), bool_t)],
            target_indices: vec![],
        }],
    });
    assert!(r.is_err(), "negative Bad2 (D in an arrow domain) rejected");
}

#[test]
fn ac5_nested_double_positive_admitted() {
    // The seed case's literal term `((Bad2 → Bool) → Nat) → Bad2`: Bad2 sits in
    // the domain-of-domain, which is double-positive per §8.1 — strictly
    // positive, so it is *admitted*. (Documents the §8.2 behavior the seed case
    // mislabels; flagged to Spec.)
    let (mut env, _e, _u, bool_, nat, _p) = pos_env();
    let bool_t = Term::indformer(bool_, vec![]);
    let nat_t = Term::indformer(nat, vec![]);
    let r = declare_inductive(&mut env, |bad2| InductiveSpec {
        level_params: vec![],
        params: vec![],
        indices: vec![],
        level: Level::zero(),
        constructors: vec![CtorSpec {
            args: vec![Term::pi(
                Term::pi(Term::indformer(bad2, vec![]), bool_t),
                nat_t,
            )],
            target_indices: vec![],
        }],
    });
    assert!(
        r.is_ok(),
        "((Bad2 → Bool) → Nat) → Bad2 is strictly positive (double-positive); admitted per §8.2"
    );
}

#[test]
fn ac5_nested_negative_in_application_rejected() {
    // data Bad3 : Type 0 where mk : Pair (Bad3 → Empty) Unit → Bad3
    // — the `occurs` guard on application arguments catches D hidden in `Pair`'s
    // arg (the Architect review's soundness hole).
    let (mut env, empty, unit, _b, _n, pair) = pos_env();
    let empty_t = Term::indformer(empty, vec![]);
    let unit_t = Term::indformer(unit, vec![]);
    let pair_t =
        |a: Term, b: Term| Term::app(Term::app(Term::indformer(pair, vec![Level::zero()]), a), b);
    let r = declare_inductive(&mut env, |bad3| InductiveSpec {
        level_params: vec![],
        params: vec![],
        indices: vec![],
        level: Level::zero(),
        constructors: vec![CtorSpec {
            args: vec![pair_t(
                Term::pi(Term::indformer(bad3, vec![]), empty_t),
                unit_t,
            )],
            target_indices: vec![],
        }],
    });
    assert!(
        r.is_err(),
        "Bad3 (nested negative in application argument) rejected"
    );
}

#[test]
fn ac5_d_in_own_indices_rejected() {
    // data Bad4 : (Bad4 → Empty) → Type 0 where … — D in its own index telescope.
    let (mut env, empty, _u, _b, _n, _p) = pos_env();
    let empty_t = Term::indformer(empty, vec![]);
    let r = declare_inductive(&mut env, |bad4| InductiveSpec {
        level_params: vec![],
        params: vec![],
        indices: vec![Term::pi(Term::indformer(bad4, vec![]), empty_t)],
        level: Level::zero(),
        constructors: vec![],
    });
    assert!(r.is_err(), "Bad4 (D in own indices) rejected");
}

// ===========================================================================
// AC-7 — Decidable checking on the K1 fragment (termination) (`seed-k1.md` AC-7)
// ===========================================================================

#[test]
fn ac7_beta_reduction_terminates() {
    let (env, s) = std_env();
    let ctx = Context::new();
    // A term with nested β-redexes; leftmost-outermost whnf must terminate.
    let t = Term::app(
        Term::lam(Term::Type(Level::zero()), Term::var(0)),
        Term::app(
            Term::lam(Term::Type(Level::zero()), Term::var(0)),
            Term::indformer(s.nat, vec![]),
        ),
    );
    let _ = whnf(&env, &ctx, &t); // returns (does not loop)
}

#[test]
fn ac7_eta_expansion_terminates() {
    // Convert two functions at a Π-type with a nested η opportunity; the
    // type-directed η descent is finite (the type is finite).
    let (env, _s) = std_env();
    let a = Term::Type(Level::zero());
    let pi_ty = Term::pi(a.clone(), a.clone());
    let mut ctx = Context::new();
    ctx.push(pi_ty.clone());
    let f = Term::var(0);
    let g = Term::lam(a.clone(), Term::app(Term::var(1), Term::var(0)));
    let _ = convert(&env, &ctx, &pi_ty, &f, &g); // terminates
}

#[test]
fn ac7_iota_reduction_terminates() {
    // elim_Nat over a deep numeral — ι descends on structurally smaller
    // scrutinees, so it terminates.
    let (env, s) = std_env();
    let ctx = Context::new();
    let (motive, z, sm) = nat_elim_pieces(&s);
    // suc (suc (suc zero))
    let deep = Term::app(
        nat_const(&s, s.suc),
        Term::app(
            nat_const(&s, s.suc),
            Term::app(nat_const(&s, s.suc), nat_const(&s, s.zero)),
        ),
    );
    let elim = Term::Elim {
        fam: s.nat,
        level_args: vec![],
        params: vec![],
        motive: Box::new(motive),
        methods: vec![z, sm],
        indices: vec![],
        scrut: Box::new(deep),
    };
    let _ = whnf(&env, &ctx, &elim); // terminates
}

#[test]
fn ac7_delta_unfolding_terminates() {
    // A chain of transparent definitions c1 := c2, c2 := c3, c3 := zero; δ is
    // acyclic so unfolding terminates.
    let (mut env, s) = std_env();
    let ctx = Context::new();
    let nat_t = Term::indformer(s.nat, vec![]);
    let c3 =
        ken_kernel::declare_def(&mut env, vec![], nat_t.clone(), nat_const(&s, s.zero)).unwrap();
    let c2 = ken_kernel::declare_def(
        &mut env,
        vec![],
        nat_t.clone(),
        Term::Const {
            id: c3,
            level_args: vec![],
        },
    )
    .unwrap();
    let c1 = ken_kernel::declare_def(
        &mut env,
        vec![],
        nat_t,
        Term::Const {
            id: c2,
            level_args: vec![],
        },
    )
    .unwrap();
    let _ = whnf(
        &env,
        &ctx,
        &Term::Const {
            id: c1,
            level_args: vec![],
        },
    ); // terminates at zero
}

#[test]
fn ac7_checking_terminates_k1() {
    // `check`/`infer` on a suite exercising all K1 formers terminate.
    let (env, s) = std_env();
    let ctx = Context::new();
    let suite = [
        Term::Type(Level::zero()),
        Term::pi(Term::Type(Level::zero()), Term::Type(Level::zero())),
        Term::sigma(Term::indformer(s.nat, vec![]), Term::Type(Level::zero())),
        nat_const(&s, s.zero),
        Term::app(nat_const(&s, s.suc), nat_const(&s, s.zero)),
    ];
    for t in &suite {
        let _ = ken_kernel::infer(&env, &ctx, t); // terminates
    }
}

// ===========================================================================
// AC-6 — Subject reduction on the K1 fragment (`seed-k1.md` AC-6)
// ===========================================================================

/// Subject reduction: if `Γ ⊢ t : A` and `t` reduces (to whnf `t'`), then
/// `Γ ⊢ t' : A` (up to conversion). Reducing all the way to whnf tests the
/// property across every β/η/ι/δ step on the path.
fn subject_reduction_holds(env: &GlobalEnv, ctx: &Context, t: &Term) -> bool {
    let a = match ken_kernel::infer(env, ctx, t) {
        Ok(ty) => ty,
        Err(_) => return false,
    };
    let t_red = whnf(env, ctx, t);
    let a_red = match ken_kernel::infer(env, ctx, &t_red) {
        Ok(ty) => ty,
        Err(_) => return false,
    };
    ken_kernel::convert_type(env, ctx, &a, &a_red)
}

#[test]
fn ac6_subject_reduction_pi_beta() {
    // Γ ⊢ ((λ(x:Nat). x) : Nat → Nat) zero : Nat  ;  ⇝ zero : Nat.
    let (env, s) = std_env();
    let ctx = Context::new();
    let nat_t = Term::indformer(s.nat, vec![]);
    let id = Term::Ascript(
        Box::new(Term::lam(nat_t.clone(), Term::var(0))),
        Box::new(Term::pi(nat_t.clone(), nat_t.clone())),
    );
    let t = Term::app(id, nat_const(&s, s.zero));
    assert!(subject_reduction_holds(&env, &ctx, &t));
}

#[test]
fn ac6_subject_reduction_sigma_beta() {
    // (zero, zero) : Nat × Nat ;  (zero, zero).1 ⇝ zero : Nat ; .2 ⇝ zero : Nat.
    let (env, s) = std_env();
    let ctx = Context::new();
    let nat_t = Term::indformer(s.nat, vec![]);
    let sig = Term::sigma(nat_t.clone(), nat_t.clone());
    let pair = Term::Ascript(
        Box::new(Term::pair(nat_const(&s, s.zero), nat_const(&s, s.zero))),
        Box::new(sig),
    );
    assert!(subject_reduction_holds(
        &env,
        &ctx,
        &Term::proj1(pair.clone())
    ));
    assert!(subject_reduction_holds(&env, &ctx, &Term::proj2(pair)));
}

#[test]
fn ac6_subject_reduction_iota_nat() {
    // elim_Nat M z s (suc zero) : M (suc zero) = Nat ;  ι ⇝ … ⇝ suc zero : Nat.
    let (env, s) = std_env();
    let ctx = Context::new();
    let (motive, z, sm) = nat_elim_pieces(&s);
    let suc_zero = Term::app(nat_const(&s, s.suc), nat_const(&s, s.zero));
    let t = Term::Elim {
        fam: s.nat,
        level_args: vec![],
        params: vec![],
        motive: Box::new(motive),
        methods: vec![z, sm],
        indices: vec![],
        scrut: Box::new(suc_zero),
    };
    assert!(subject_reduction_holds(&env, &ctx, &t));
}

#[test]
fn ac6_subject_reduction_k1_property() {
    // Property test across all K1 reduction forms: each well-typed term reduces
    // to a whnf that type-checks at the same type (up to conversion). A
    // deterministic battery covering β, Σ-β, ι, and δ.
    let (mut env, s) = std_env();
    let ctx = Context::new();
    let nat_t = Term::indformer(s.nat, vec![]);
    let id = Term::Ascript(
        Box::new(Term::lam(nat_t.clone(), Term::var(0))),
        Box::new(Term::pi(nat_t.clone(), nat_t.clone())),
    );
    // δ: a transparent def unfolding to zero.
    let _c =
        ken_kernel::declare_def(&mut env, vec![], nat_t.clone(), nat_const(&s, s.zero)).unwrap();
    let delta_term = Term::Const {
        id: _c,
        level_args: vec![],
    };
    let (motive, z, sm) = nat_elim_pieces(&s);
    let two = Term::app(
        nat_const(&s, s.suc),
        Term::app(nat_const(&s, s.suc), nat_const(&s, s.zero)),
    );
    let iota_term = Term::Elim {
        fam: s.nat,
        level_args: vec![],
        params: vec![],
        motive: Box::new(motive),
        methods: vec![z, sm],
        indices: vec![],
        scrut: Box::new(two),
    };
    let suc_zero = Term::app(nat_const(&s, s.suc), nat_const(&s, s.zero));
    let battery = [
        Term::app(id, nat_const(&s, s.zero)), // β
        Term::proj1(Term::Ascript(
            Box::new(Term::pair(nat_const(&s, s.zero), suc_zero.clone())),
            Box::new(Term::sigma(nat_t.clone(), nat_t.clone())),
        )), // Σ-β₁
        Term::proj2(Term::Ascript(
            Box::new(Term::pair(suc_zero.clone(), nat_const(&s, s.zero))),
            Box::new(Term::sigma(nat_t.clone(), nat_t.clone())),
        )), // Σ-β₂
        iota_term,                            // ι
        delta_term,                           // δ
    ];
    for t in &battery {
        assert!(
            subject_reduction_holds(&env, &ctx, t),
            "subject reduction failed for {:?}",
            t
        );
    }
}
