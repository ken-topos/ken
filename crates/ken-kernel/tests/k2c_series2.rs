//! K2c series-2 conformance tests — the three observational-reduction seams
//! that K2 left sound-stuck and series-2 completes. Every case is
//! discriminating (verdict-flip). Grounded in `16-observational.md` §2.2,
//! §3.2, §4.1, §5.1 and `seed-obs-completion.md`.
//!
//! Seam 1  — `cast` at an inductive index change (§3.2 index rewrite)
//! Seam 1b — `Eq` at an inductive dependent telescope (§2.2)
//! Seam 2  — `J` at a dependent (non-constant) motive (§4.1)
//! Seam 3  — quotient respect schema for Type-target motive (§5.1)

use ken_kernel::env::Context;
use ken_kernel::inductive::peel_app;
use ken_kernel::term::{Level, LevelVar, Term};
use ken_kernel::{
    convert_type, declare_def, declare_inductive, declare_postulate, infer, whnf, CtorSpec,
    GlobalEnv, GlobalId, InductiveSpec,
};

const L: LevelVar = LevelVar(0);
fn lvar() -> Level {
    Level::Var(L)
}

struct Std {
    nat: GlobalId,
    zero: GlobalId,
    suc: GlobalId,
    unit: GlobalId,
    tt: GlobalId,
    vec_: GlobalId,
    vnil: GlobalId,
    vcons: GlobalId,
}

fn std_env() -> (GlobalEnv, Std) {
    let mut env = GlobalEnv::new();

    let nat = declare_inductive(&mut env, |nat| InductiveSpec {
        level_params: vec![],
        params: vec![],
        indices: vec![],
        level: Level::zero(),
        constructors: vec![
            CtorSpec { args: vec![], target_indices: vec![] },
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

    let unit = declare_inductive(&mut env, |_| InductiveSpec {
        level_params: vec![],
        params: vec![],
        indices: vec![],
        level: Level::zero(),
        constructors: vec![CtorSpec { args: vec![], target_indices: vec![] }],
    })
    .expect("Unit");
    let tt = env.inductive(unit).unwrap().constructors[0].id;

    let vec_ = declare_inductive(&mut env, |vec_| {
        let nat_t = Term::indformer(nat, vec![]);
        InductiveSpec {
            level_params: vec![L],
            params: vec![Term::Type(lvar())],
            indices: vec![nat_t.clone()],
            level: lvar(),
            constructors: vec![
                CtorSpec {
                    args: vec![],
                    target_indices: vec![Term::constructor(zero, vec![])],
                },
                CtorSpec {
                    // (n:Nat)(a:A)(xs:Vec A n) relative to [A]
                    args: vec![
                        nat_t.clone(),
                        Term::var(1),
                        Term::app(
                            Term::app(Term::indformer(vec_, vec![lvar()]), Term::var(2)),
                            Term::var(1),
                        ),
                    ],
                    // target index: suc n, in [A,n,a,xs] n=Var(2)
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

    (env, Std { nat, zero, suc, unit, tt, vec_, vnil, vcons })
}

// --- helpers -----------------------------------------------------------------

fn nat_t(s: &Std) -> Term {
    Term::indformer(s.nat, vec![])
}
fn zero_t(s: &Std) -> Term {
    Term::constructor(s.zero, vec![])
}
fn suc_t(s: &Std, n: Term) -> Term {
    Term::app(Term::constructor(s.suc, vec![]), n)
}
fn vec_t(s: &Std, a: Term, n: Term) -> Term {
    Term::app(Term::app(Term::indformer(s.vec_, vec![Level::zero()]), a), n)
}
fn vcons_t(s: &Std, a: Term, n: Term, elem: Term, xs: Term) -> Term {
    Term::app(
        Term::app(
            Term::app(
                Term::app(Term::constructor(s.vcons, vec![Level::zero()]), a),
                n,
            ),
            elem,
        ),
        xs,
    )
}

// =============================================================================
// Seam 1 — cast at an inductive index change (§3.2 index rewrite)
// =============================================================================

/// `cast (Vec A (suc n)) (Vec A (suc m)) e (vcons A n a xs)`
/// ⇝ `vcons A m a (cast (Vec A n) (Vec A m) ... xs)` — discriminating.
///
/// Discriminant: the naive ill-typed reduct `vcons A m a xs` (bare xs, no
/// sub-cast) differs structurally from the correct reduct at args[3].
#[test]
fn cast_inductive_index_rewrite() {
    let (mut env, s) = std_env();
    let ctx = Context::new();

    // n, m : Nat (neutral postulates)
    let n_id = declare_postulate(&mut env, vec![], nat_t(&s)).unwrap();
    let n = Term::Const { id: n_id, level_args: vec![] };
    let m_id = declare_postulate(&mut env, vec![], nat_t(&s)).unwrap();
    let m = Term::Const { id: m_id, level_args: vec![] };

    // A : Type 0; a : A (postulates)
    let big_a_id = declare_postulate(&mut env, vec![], Term::Type(Level::zero())).unwrap();
    let big_a = Term::Const { id: big_a_id, level_args: vec![] };
    let a_id = declare_postulate(&mut env, vec![], big_a.clone()).unwrap();
    let small_a = Term::Const { id: a_id, level_args: vec![] };

    // xs : Vec A n (postulate)
    let xs_ty = vec_t(&s, big_a.clone(), n.clone());
    let xs_id = declare_postulate(&mut env, vec![], xs_ty.clone()).unwrap();
    let xs = Term::Const { id: xs_id, level_args: vec![] };

    // e : Eq Type (Vec A (suc n)) (Vec A (suc m)) (postulate — cast ignores it)
    let suc_n = suc_t(&s, n.clone());
    let suc_m = suc_t(&s, m.clone());
    let vec_a_suc_n = vec_t(&s, big_a.clone(), suc_n.clone());
    let vec_a_suc_m = vec_t(&s, big_a.clone(), suc_m.clone());
    let e_ty = Term::Eq(
        Box::new(Term::Type(Level::zero())),
        Box::new(vec_a_suc_n.clone()),
        Box::new(vec_a_suc_m.clone()),
    );
    let e_id = declare_postulate(&mut env, vec![], e_ty).unwrap();
    let e = Term::Const { id: e_id, level_args: vec![] };

    // scrutinee: vcons A n a xs : Vec A (suc n)
    let scrut = vcons_t(&s, big_a.clone(), n.clone(), small_a.clone(), xs.clone());

    let cast_term = Term::Cast(
        Box::new(vec_a_suc_n),
        Box::new(vec_a_suc_m),
        Box::new(e),
        Box::new(scrut),
    );
    let result = whnf(&env, &ctx, &cast_term);

    // Must be constructor-headed at vcons
    let (head, args) = peel_app(&result);
    assert!(
        matches!(head, Term::Constructor { id, .. } if id == s.vcons),
        "result must be vcons-headed; got {:?}",
        result
    );
    assert_eq!(args.len(), 4, "vcons has 4 args (A, n, a, xs)");

    // args[1] = m (forced index)
    assert!(
        convert_type(&env, &ctx, &args[1], &m),
        "forced index arg should be m; got {:?}",
        args[1]
    );

    // args[3] must be a Cast (not bare xs) — structural discriminant
    let vec_a_n = vec_t(&s, big_a.clone(), n.clone());
    let vec_a_m = vec_t(&s, big_a.clone(), m.clone());
    match &args[3] {
        Term::Cast(src, tgt, _, _) => {
            assert!(
                convert_type(&env, &ctx, src, &vec_a_n),
                "sub-cast source should be Vec A n"
            );
            assert!(
                convert_type(&env, &ctx, tgt, &vec_a_m),
                "sub-cast target should be Vec A m"
            );
        }
        other => panic!("expected Cast in xs position (discriminant), got {:?}", other),
    }
}

/// Neutral index k ⇒ stuck; §3.2 guard fires. Discriminates from the
/// index-rewrite case on the guard axis.
#[test]
fn cast_inductive_open_index_stuck() {
    let (mut env, s) = std_env();
    let ctx = Context::new();

    let n_id = declare_postulate(&mut env, vec![], nat_t(&s)).unwrap();
    let n = Term::Const { id: n_id, level_args: vec![] };
    let k_id = declare_postulate(&mut env, vec![], nat_t(&s)).unwrap();
    let k = Term::Const { id: k_id, level_args: vec![] };

    let big_a_id = declare_postulate(&mut env, vec![], Term::Type(Level::zero())).unwrap();
    let big_a = Term::Const { id: big_a_id, level_args: vec![] };
    let a_id = declare_postulate(&mut env, vec![], big_a.clone()).unwrap();
    let small_a = Term::Const { id: a_id, level_args: vec![] };

    let xs_id = declare_postulate(&mut env, vec![], vec_t(&s, big_a.clone(), n.clone())).unwrap();
    let xs = Term::Const { id: xs_id, level_args: vec![] };

    // cast (Vec A (suc n)) (Vec A k) e (vcons A n a xs) — k is neutral
    let suc_n = suc_t(&s, n.clone());
    let vec_a_suc_n = vec_t(&s, big_a.clone(), suc_n.clone());
    let vec_a_k = vec_t(&s, big_a.clone(), k.clone());
    let e_ty = Term::Eq(
        Box::new(Term::Type(Level::zero())),
        Box::new(vec_a_suc_n.clone()),
        Box::new(vec_a_k.clone()),
    );
    let e_id = declare_postulate(&mut env, vec![], e_ty).unwrap();
    let e = Term::Const { id: e_id, level_args: vec![] };

    let scrut = vcons_t(&s, big_a, n, small_a, xs);
    let cast_term = Term::Cast(
        Box::new(vec_a_suc_n),
        Box::new(vec_a_k),
        Box::new(e),
        Box::new(scrut),
    );
    let result = whnf(&env, &ctx, &cast_term);

    // Must remain a Cast (neutral/stuck) — NOT a constructor
    assert!(
        matches!(result, Term::Cast(..)),
        "open index k should leave cast stuck; got {:?}",
        result
    );
    assert!(
        !matches!(result, Term::Constructor { .. }),
        "must not fabricate a constructor past a neutral index"
    );
}

// =============================================================================
// Seam 1b — Eq at an inductive dependent telescope (§2.2)
// =============================================================================

/// `Eq (Vec A (suc n)) (vcons A n a xs) (vcons A n' a' xs')`
/// ⇝ Σ-conjunction whose third conjunct transports xs via Cast.
///
/// Discriminant: a wrong kernel (no transport) would return None (stuck) or
/// emit a conjunct without Cast in the xs position.
#[test]
fn eq_inductive_dependent_telescope() {
    let (mut env, s) = std_env();
    let ctx = Context::new();

    let big_a_id = declare_postulate(&mut env, vec![], Term::Type(Level::zero())).unwrap();
    let big_a = Term::Const { id: big_a_id, level_args: vec![] };

    let n_id = declare_postulate(&mut env, vec![], nat_t(&s)).unwrap();
    let n = Term::Const { id: n_id, level_args: vec![] };
    let np_id = declare_postulate(&mut env, vec![], nat_t(&s)).unwrap();
    let np = Term::Const { id: np_id, level_args: vec![] };

    let a_id = declare_postulate(&mut env, vec![], big_a.clone()).unwrap();
    let small_a = Term::Const { id: a_id, level_args: vec![] };
    let ap_id = declare_postulate(&mut env, vec![], big_a.clone()).unwrap();
    let small_ap = Term::Const { id: ap_id, level_args: vec![] };

    let xs_id =
        declare_postulate(&mut env, vec![], vec_t(&s, big_a.clone(), n.clone())).unwrap();
    let xs = Term::Const { id: xs_id, level_args: vec![] };
    let xsp_id =
        declare_postulate(&mut env, vec![], vec_t(&s, big_a.clone(), np.clone())).unwrap();
    let xsp = Term::Const { id: xsp_id, level_args: vec![] };

    let suc_n = suc_t(&s, n.clone());
    let vec_a_suc_n = vec_t(&s, big_a.clone(), suc_n);

    let lhs = vcons_t(&s, big_a.clone(), n.clone(), small_a.clone(), xs.clone());
    let rhs = vcons_t(&s, big_a.clone(), np.clone(), small_ap.clone(), xsp.clone());

    let eq_term = Term::Eq(Box::new(vec_a_suc_n), Box::new(lhs), Box::new(rhs));
    let result = whnf(&env, &ctx, &eq_term);

    // Must NOT be neutral (still an Eq with inductive head)
    assert!(
        !matches!(result, Term::Eq(..)),
        "Eq at same-ctor should reduce, not stay stuck; got {:?}",
        result
    );

    // Result is a Σ-chain (conjunction). Flatten it.
    fn flatten_sigma(t: &Term) -> Vec<&Term> {
        match t {
            Term::Sigma(first, rest) => {
                let mut v = vec![first.as_ref()];
                v.extend(flatten_sigma(rest));
                v
            }
            other => vec![other],
        }
    }
    let conjuncts = flatten_sigma(&result);
    // Vec has 3 ctor args (n, a, xs) → 3 conjuncts (strip_trailing_top removes Top)
    assert!(
        conjuncts.len() >= 3,
        "expected ≥3 conjuncts, got {}; result = {:?}",
        conjuncts.len(),
        result
    );

    // Third conjunct (xs position): must be an Eq with Cast on the LHS
    let third = conjuncts[2];
    match third {
        Term::Eq(ty, lhs, _rhs) => {
            // type should be Vec A n' (target side)
            let vec_a_np = vec_t(&s, big_a.clone(), np.clone());
            assert!(
                convert_type(&env, &ctx, ty, &vec_a_np),
                "third conjunct Eq type should be Vec A n'; got {:?}",
                ty
            );
            // lhs should be a Cast (not bare xs)
            assert!(
                matches!(lhs.as_ref(), Term::Cast(..)),
                "third conjunct LHS should be Cast (not bare xs); got {:?}",
                lhs
            );
        }
        other => panic!("third conjunct should be Eq; got {:?}", other),
    }
}

// =============================================================================
// Seam 2 — J at a dependent (non-constant) motive (§4.1)
// =============================================================================

/// `J (λb. λ_. Vec A (suc b)) base e_j` where `e_j : Eq Nat n m` and `n`, `m`
/// are **neutral** (postulates). motive is non-constant since `Vec A (suc n)
/// ≢ Vec A (suc m)`. Seam-2 fires J-cast; seam-1 fires on the outer index.
///
/// Discriminant: bug = stuck `Term::J`; correct = constructor-headed `vcons`.
#[test]
fn j_dependent_motive_fires() {
    let (mut env, s) = std_env();
    let ctx = Context::new();

    let big_a_id = declare_postulate(&mut env, vec![], Term::Type(Level::zero())).unwrap();
    let big_a = Term::Const { id: big_a_id, level_args: vec![] };
    let small_a_id = declare_postulate(&mut env, vec![], big_a.clone()).unwrap();
    let small_a = Term::Const { id: small_a_id, level_args: vec![] };

    // n, m : Nat — neutral postulates so Eq Nat n m stays neutral (not ⇝ Bottom)
    let n_id = declare_postulate(&mut env, vec![], nat_t(&s)).unwrap();
    let n = Term::Const { id: n_id, level_args: vec![] };
    let m_id = declare_postulate(&mut env, vec![], nat_t(&s)).unwrap();
    let m = Term::Const { id: m_id, level_args: vec![] };

    // motive: λ(b:Nat). λ(_:Type). Vec A (suc b)
    // Under 2 binders: b=Var(1), _=Var(0). suc b = App(suc, Var(1)).
    let suc_b = suc_t(&s, Term::var(1));
    let motive = Term::Lam(
        Box::new(nat_t(&s)),
        Box::new(Term::Lam(
            Box::new(Term::Type(Level::zero())),
            Box::new(vec_t(&s, big_a.clone(), suc_b)),
        )),
    );

    // base : Vec A (suc n) — concrete vcons so seam-1 fires on the outer cast
    let vnil_a = Term::app(
        Term::constructor(s.vnil, vec![Level::zero()]),
        big_a.clone(),
    );
    let base = vcons_t(&s, big_a.clone(), n.clone(), small_a.clone(), vnil_a);

    // e_j : Eq Nat n m — neutral equality (n, m neutral ⇒ Eq Nat n m stays Eq)
    let eq_j_ty = Term::Eq(
        Box::new(nat_t(&s)),
        Box::new(n.clone()),
        Box::new(m.clone()),
    );
    let e_j_id = declare_postulate(&mut env, vec![], eq_j_ty).unwrap();
    let e_j = Term::Const { id: e_j_id, level_args: vec![] };

    let j_term = Term::J(Box::new(motive), Box::new(base), Box::new(e_j));
    let result = whnf(&env, &ctx, &j_term);

    // J must NOT stay stuck (discriminant: bug leaves Term::J neutral)
    assert!(
        !matches!(result, Term::J(..)),
        "J with non-constant motive must fire (seam 2); got stuck Term::J"
    );

    // J-cast fired and seam-1 fired on the outer Vec index rewrite (suc n →
    // suc m); result is vcons-headed.
    let (head, _args) = peel_app(&result);
    assert!(
        matches!(head, Term::Constructor { id, .. } if id == s.vcons),
        "J result should be vcons-headed after full reduction; got {:?}",
        result
    );
}

// =============================================================================
// Seam 3 — quotient respect schema for Type-target motive (§5.1)
// =============================================================================

/// Type-target quotient elim: well-typed respect ⇒ accepted; ill-typed ⇒
/// rejected. Discriminant flips the verdict.
///
/// `motive` and `method` are **transparent definitions** (not opaque postulates)
/// so that `App(motive, [x])` reduces to a concrete type during type-checking.
/// This is necessary because the kernel builds the cong/cast expected type with
/// `Refl(m_x)` as the cast-equality proof, which only type-checks when
/// `m_x ≡ m_y` is decidable by whnf (regularity).
///
/// Respect proofs are bare lambdas, checked in check-mode by infer_quot_elim.
#[test]
fn quotient_respect_type_target() {
    let (mut env, s) = std_env();
    let ctx = Context::new();

    let unit_t = Term::indformer(s.unit, vec![]);
    let nat_ty = nat_t(&s);
    let zero_c = zero_t(&s);

    // R : Unit → Unit → Ω_0  (postulate; needed for check_quotient_rel)
    let rel_ty = Term::pi(
        unit_t.clone(),
        Term::pi(unit_t.clone(), Term::Omega(Level::zero())),
    );
    let rel_id = declare_postulate(&mut env, vec![], rel_ty).unwrap();
    let rel = Term::Const { id: rel_id, level_args: vec![] };

    // quot_ty = Unit/R
    let quot_ty = Term::Quot(Box::new(unit_t.clone()), Box::new(rel.clone()));

    // q : Unit/R
    let q_id = declare_postulate(&mut env, vec![], quot_ty.clone()).unwrap();
    let q = Term::Const { id: q_id, level_args: vec![] };

    // M : Unit/R → Type 0  **transparent** (= λ_. Nat) so App(M, [x]) ⇝ Nat.
    // Transparency is critical: with opaque M, m_x=App(M,[x]) and m_y=App(M,[y])
    // are not convertible, so Refl(m_x) can't prove Eq(Type_0, m_x, m_y).
    let motive_fn_ty = Term::pi(quot_ty.clone(), Term::Type(Level::zero()));
    let motive_fn_body = Term::Lam(Box::new(quot_ty.clone()), Box::new(nat_ty.clone()));
    let motive_id = declare_def(&mut env, vec![], motive_fn_ty, motive_fn_body).unwrap();
    let motive = Term::Const { id: motive_id, level_args: vec![] };

    // f : Unit → Nat  **transparent** (= λ_. zero) so App(f, x) ⇝ zero.
    let method_fn_ty = Term::pi(unit_t.clone(), nat_ty.clone());
    let method_fn_body = Term::Lam(Box::new(unit_t.clone()), Box::new(zero_c.clone()));
    let method_id = declare_def(&mut env, vec![], method_fn_ty, method_fn_body).unwrap();
    let method = Term::Const { id: method_id, level_args: vec![] };

    // h_ty = R x y at depth 2 (x=Var(1), y=Var(0)).
    let h_ty = Term::app(Term::app(rel.clone(), Term::var(1)), Term::var(0));

    // r_valid = λx. λy. λh. refl(zero)
    // At depth 3, expected body = Eq(Nat, zero, Cast(Nat, Nat, Refl(Nat), zero))
    //                           = Eq(Nat, zero, zero)  [Cast reduces by regularity]
    // so refl(zero) is a valid proof.
    let r_valid = Term::Lam(
        Box::new(unit_t.clone()),
        Box::new(Term::Lam(
            Box::new(unit_t.clone()),
            Box::new(Term::Lam(
                Box::new(h_ty.clone()),
                Box::new(Term::Refl(Box::new(zero_c.clone()))),
            )),
        )),
    );

    let valid_elim = Term::QuotElim {
        motive: Box::new(motive.clone()),
        method: Box::new(method.clone()),
        respect: Box::new(r_valid),
        scrut: Box::new(q.clone()),
    };
    let ok = infer(&env, &ctx, &valid_elim);
    assert!(ok.is_ok(), "valid respect proof must be accepted; err = {:?}", ok);

    // r_bad = λx. λy. λh. zero  (returns Nat, not an Eq — wrong type)
    let r_bad = Term::Lam(
        Box::new(unit_t.clone()),
        Box::new(Term::Lam(
            Box::new(unit_t.clone()),
            Box::new(Term::Lam(
                Box::new(h_ty),
                Box::new(zero_c.clone()),
            )),
        )),
    );
    let bad_elim = Term::QuotElim {
        motive: Box::new(motive),
        method: Box::new(method),
        respect: Box::new(r_bad),
        scrut: Box::new(q),
    };
    let err = infer(&env, &ctx, &bad_elim);
    assert!(
        err.is_err(),
        "invalid respect proof must be rejected; got Ok({:?})",
        err
    );
}

/// Direction discriminant for the seam-3 Cast: `Cast(M[y], M[x], _, f_y)` vs
/// the pre-fix wrong direction `Cast(M[x], M[y], _, f_y)`.
///
/// Uses a Vec-indexed motive so `cast_at_inductive` fires for BOTH directions
/// (no stuck case), making the wrong-direction observable as a different result
/// value rather than an opaque neutral.
///
/// *   **Correct** `Cast(m_y, m_x, _, f_y)` → `vcons A m a (Cast … xs)` —
///     forced index is **m** (the TARGET `m_x` index); xs gets a sub-cast.
/// *   **Wrong**   `Cast(m_x, m_y, _, f_y)` → `vcons A n a xs` — forced
///     index is **n** (the SOURCE `m_x` index); xs is unchanged. Result type
///     is `m_y`, not `m_x` — exactly the type error the seam-3 bug introduced.
///
/// Verdict flip: correct→forced index = `suc_m`; wrong→forced index = `suc_n`.
#[test]
fn quotient_respect_direction_cast() {
    let (mut env, s) = std_env();
    let ctx = Context::new();

    // A, a, n, m : neutral postulates.
    let big_a_id = declare_postulate(&mut env, vec![], Term::Type(Level::zero())).unwrap();
    let big_a = Term::Const { id: big_a_id, level_args: vec![] };
    let a_id = declare_postulate(&mut env, vec![], big_a.clone()).unwrap();
    let small_a = Term::Const { id: a_id, level_args: vec![] };
    let n_id = declare_postulate(&mut env, vec![], nat_t(&s)).unwrap();
    let n = Term::Const { id: n_id, level_args: vec![] };
    let m_id = declare_postulate(&mut env, vec![], nat_t(&s)).unwrap();
    let m = Term::Const { id: m_id, level_args: vec![] };

    // xs : Vec A n (tail of f_y).
    let xs_id =
        declare_postulate(&mut env, vec![], vec_t(&s, big_a.clone(), n.clone())).unwrap();
    let xs = Term::Const { id: xs_id, level_args: vec![] };

    // f_y = vcons A n a xs : Vec A (suc n)  (method-at-y; its type is m_y).
    let suc_n = suc_t(&s, n.clone());
    let suc_m = suc_t(&s, m.clone());
    let m_y = vec_t(&s, big_a.clone(), suc_n.clone()); // M[y] = Vec A (suc n)
    let m_x = vec_t(&s, big_a.clone(), suc_m.clone()); // M[x] = Vec A (suc m)
    let f_y = vcons_t(&s, big_a.clone(), n.clone(), small_a.clone(), xs.clone());

    // Proof witnesses (postulates); cast ignores the proof (§3.4).
    let e_correct_ty = Term::Eq(
        Box::new(Term::Type(Level::zero())),
        Box::new(m_y.clone()),
        Box::new(m_x.clone()),
    );
    let e_correct = Term::Const {
        id: declare_postulate(&mut env, vec![], e_correct_ty).unwrap(),
        level_args: vec![],
    };
    let e_wrong_ty = Term::Eq(
        Box::new(Term::Type(Level::zero())),
        Box::new(m_x.clone()),
        Box::new(m_y.clone()),
    );
    let e_wrong = Term::Const {
        id: declare_postulate(&mut env, vec![], e_wrong_ty).unwrap(),
        level_args: vec![],
    };

    // CORRECT direction: Cast(m_y, m_x, e, f_y) — transport f_y from M[y] to M[x].
    let result_correct = whnf(
        &env,
        &ctx,
        &Term::Cast(
            Box::new(m_y.clone()),
            Box::new(m_x.clone()),
            Box::new(e_correct),
            Box::new(f_y.clone()),
        ),
    );
    let (head_c, args_c) = peel_app(&result_correct);
    assert!(
        matches!(head_c, Term::Constructor { id, .. } if id == s.vcons),
        "correct-direction cast must fire to vcons; got {:?}",
        result_correct
    );
    // args[1] = tail length n' such that result = vcons A n' a (Cast xs) : Vec A (suc n').
    // Correct direction forces n' = m (from TARGET j_bar = suc m; inner arg = m).
    assert!(
        convert_type(&env, &ctx, &args_c[1], &m),
        "correct direction: tail-length arg must be m (TARGET's inner); got {:?}",
        args_c[1]
    );
    // xs position carries a Cast (not bare xs) — because a_ty_j = Vec A n ≢ b_ty_j = Vec A m.
    assert!(
        matches!(&args_c[3], Term::Cast(..)),
        "correct direction: xs position must be Cast; got {:?}",
        args_c[3]
    );

    // WRONG direction (pre-fix bug): Cast(m_x, m_y, e, f_y).
    // cast_at_inductive still fires (both Vec), but forces index n (from SOURCE
    // j_bar in the wrong direction = suc n; inner arg = n), so result type is
    // Vec A (suc n) = m_y instead of m_x — the schema's RHS has the wrong type.
    let result_wrong = whnf(
        &env,
        &ctx,
        &Term::Cast(
            Box::new(m_x.clone()),
            Box::new(m_y.clone()),
            Box::new(e_wrong),
            Box::new(f_y.clone()),
        ),
    );
    let (head_w, args_w) = peel_app(&result_wrong);
    assert!(
        matches!(head_w, Term::Constructor { id, .. } if id == s.vcons),
        "wrong-direction cast fires (cast_at_inductive is structural)"
    );
    // Wrong direction forces tail-length n (from TARGET j_bar of wrong cast = suc n).
    assert!(
        convert_type(&env, &ctx, &args_w[1], &n),
        "wrong direction: tail-length arg must be n (wrong-direction result); got {:?}",
        args_w[1]
    );
    assert!(
        !convert_type(&env, &ctx, &args_w[1], &m),
        "wrong direction must NOT produce m (that would be the correct cast)"
    );
    // xs is bare (no sub-cast): b_ty_j for xs collapses to a_ty_j = Vec A n
    // because the wrong direction targets its own SOURCE index, not the schema's target.
    assert!(
        !matches!(&args_w[3], Term::Cast(..)),
        "wrong direction: xs must be bare (no sub-cast for wrong-direction transport)"
    );
}

// =============================================================================
// Regression — existing seams must be unaffected
// =============================================================================

/// Constant-motive J (the K2 headline case) still reduces to `base`.
#[test]
fn j_constant_motive_still_reduces() {
    let (mut env, _s) = std_env();
    let ctx = Context::new();

    let big_a_id = declare_postulate(&mut env, vec![], Term::Type(Level::zero())).unwrap();
    let big_a = Term::Const { id: big_a_id, level_args: vec![] };
    let a_id = declare_postulate(&mut env, vec![], big_a.clone()).unwrap();
    let small_a = Term::Const { id: a_id, level_args: vec![] };
    let b_id = declare_postulate(&mut env, vec![], big_a.clone()).unwrap();
    let b_val = Term::Const { id: b_id, level_args: vec![] };

    // e : Eq A a b (non-refl postulate)
    let e_ty = Term::Eq(
        Box::new(big_a.clone()),
        Box::new(small_a.clone()),
        Box::new(b_val.clone()),
    );
    let e_id = declare_postulate(&mut env, vec![], e_ty).unwrap();
    let e = Term::Const { id: e_id, level_args: vec![] };

    // constant motive: λ_. λ_. A  (ignores both args)
    let motive = Term::Lam(
        Box::new(big_a.clone()),
        Box::new(Term::Lam(Box::new(Term::Type(Level::zero())), Box::new(big_a.clone()))),
    );
    // base : A
    let j_term = Term::J(Box::new(motive), Box::new(small_a.clone()), Box::new(e));
    let result = whnf(&env, &ctx, &j_term);

    // Constant motive: P(a, refl a) ≡ P(b, e) ≡ A → cast reduces by
    // regularity → base.
    assert!(
        !matches!(result, Term::J(..)),
        "constant-motive J must still fire"
    );
    // For a constant motive, the cast's source ≡ target (A ≡ A), so
    // regularity fires immediately and the result IS `small_a` (the base).
    assert!(
        convert_type(&env, &ctx, &result, &small_a),
        "constant-motive J result should be the base; got {:?}",
        result
    );
}

/// Ω-target quotient elim is still respect-free (K2 regression).
///
/// All lambdas declared as postulates so `infer` can resolve their types.
#[test]
fn quotient_omega_target_respect_free() {
    let (mut env, s) = std_env();
    let ctx = Context::new();

    let unit_t = Term::indformer(s.unit, vec![]);

    // R : Unit → Unit → Ω_0  (postulate)
    let rel_ty = Term::pi(
        unit_t.clone(),
        Term::pi(unit_t.clone(), Term::Omega(Level::zero())),
    );
    let rel_id = declare_postulate(&mut env, vec![], rel_ty).unwrap();
    let rel = Term::Const { id: rel_id, level_args: vec![] };

    // quot_ty = Unit/R
    let quot_ty = Term::Quot(Box::new(unit_t.clone()), Box::new(rel));
    let q_id = declare_postulate(&mut env, vec![], quot_ty.clone()).unwrap();
    let q = Term::Const { id: q_id, level_args: vec![] };

    // M : Unit/R → Ω_0  (postulate; Ω-target → type_target = false)
    let motive_ty = Term::pi(quot_ty.clone(), Term::Omega(Level::zero()));
    let motive_id = declare_postulate(&mut env, vec![], motive_ty).unwrap();
    let motive = Term::Const { id: motive_id, level_args: vec![] };

    // f : (x:Unit) → M [x]  (postulate)
    let method_ty = Term::pi(
        unit_t.clone(),
        Term::app(motive.clone(), Term::QuotClass(Box::new(Term::var(0)))),
    );
    let method_id = declare_postulate(&mut env, vec![], method_ty).unwrap();
    let method = Term::Const { id: method_id, level_args: vec![] };

    // respect is a dummy (Ω-target → raw_well_formed check only): tt
    let respect = Term::constructor(s.tt, vec![]);

    let elim = Term::QuotElim {
        motive: Box::new(motive),
        method: Box::new(method),
        respect: Box::new(respect),
        scrut: Box::new(q),
    };
    let result = infer(&env, &ctx, &elim);
    assert!(
        result.is_ok(),
        "Ω-target quotient elim must still be accepted; err={:?}",
        result
    );
}
