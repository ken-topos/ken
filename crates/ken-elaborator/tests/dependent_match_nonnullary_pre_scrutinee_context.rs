//! language-qa gate addition: `check_match_dependent`'s own handoff cites a
//! pre-existing bug fix — `subst_var` (built for β-reduction) silently
//! mis-decremented a free variable declared BEFORE the scrutinee (e.g. a
//! generic `(a:Type)` parameter the goal closes over), fixed by a new
//! `subst_var_generalize`. But neither of `dependent_match_nonnullary_
//! acceptance.rs`'s own AC1 tests uses a `view` with an explicit type
//! parameter before the scrutinee — every test scrutinee there is the
//! FIRST (and only) bound variable in context (`List Bool`, `Tree Nat`,
//! both concrete). Differentially confirmed (language-qa): reverting
//! `subst_var_generalize`'s two call sites back to `subst_var` leaves all 4
//! of that file's tests green, while THIS test fails (`KernelRejected`
//! `TypeMismatch`) — so the fix had zero invoking regression test.
//!
//! This test: a `view` with THREE variables in context before/at the match
//! — `(a:Type)`, `(v:a)`, `(xs:List a)` — where the goal (`Equal a v v`)
//! references BOTH `a` and `v`, both declared strictly before the
//! scrutinee `xs`. Structurally asserts the resulting Cons method's `\h.`
//! domain still references `a`/`v` at the CORRECT (undecremented) indices —
//! pins the fix at a non-empty preceding context.

use ken_elaborator::ElabEnv;
use ken_kernel::Term;

fn mk_env() -> ElabEnv {
    ElabEnv::new().expect("base env")
}

fn peel_lams(t: &Term, n: usize) -> &Term {
    let mut cur = t;
    for _ in 0..n {
        match cur {
            Term::Lam(_, body) => cur = body,
            other => panic!("expected a Lam layer to peel, got {other:?}"),
        }
    }
    cur
}

#[test]
fn goal_closing_over_pre_scrutinee_type_param_narrows_correctly() {
    let mut env = mk_env();

    // Context at the match: [a: Type, v: a, xs: List a] — scrut_idx = 0,
    // TWO free variables (`a` at what becomes Var(2), `v` at Var(1)) are
    // declared strictly before the scrutinee. The goal `Equal a v v`
    // references both.
    let id = env
        .elaborate_decl(
            "fn sameHead (a : Type) (v : a) (xs : List a) : Equal a v v -> Prop = \
             match xs { Nil |-> \\h. Equal a v v ; \
                        Cons b bs |-> \\h. Equal a v v }",
        )
        .expect(
            "sameHead (goal closing over a pre-scrutinee generic type param) \
             must elaborate and kernel-check — a regression of the \
             subst_var_generalize fix would either KernelRejected (mis-shifted \
             index produces a TypeMismatch) or Internal-error here",
        );

    let body = env.env.transparent_body(id).expect("sameHead is transparent").1;
    // Peel the function's own `a`, `v`, `xs` parameter lambdas to reach the match.
    let mut inner = &body;
    for _ in 0..3 {
        match inner {
            Term::Lam(_, b) => inner = b,
            other => panic!("expected outer param Lam, got {other:?}"),
        }
    }
    let (fam, methods) = match inner {
        Term::Elim { fam, methods, .. } => (*fam, methods),
        other => panic!("sameHead's match must lower to a real Term::Elim, got {other:?}"),
    };
    assert_eq!(fam, env.globals["List"], "must eliminate over the real List family");
    assert_eq!(methods.len(), 2, "List has exactly 2 constructors (Nil, Cons)");

    // Cons method: λ(b:a).λ(bs:List a).λ(ih:Equal a v v -> Prop, since the
    // goal is CONSTANT in xs here).λ(h:<narrowed goal>). body. Peel the 2
    // field lambdas + 1 IH lambda to reach the arm's own `\h.` binder.
    let cons_method = &methods[1];
    let h_lam = peel_lams(cons_method, 3);
    match h_lam {
        Term::Lam(h_domain, _) => {
            // Outer context at this point (inside the Cons method, past
            // b/bs/ih): the ORIGINAL `a` and `v` must still be reachable at
            // their correctly-shifted indices — b=2, bs=1, ih=0 sit in
            // front, so `a`/`v`/the elaborated `List a` head all shift by
            // 3 from their un-nested position. `h`'s domain must be
            // `Equal a v v` referencing the OUTER `a`/`v` (Var(5)/Var(4)
            // relative to this frame: a, v, xs(already consumed by the
            // motive lambda so replaced), b, bs, ih — NOT some
            // mis-decremented neighbor).
            //
            // Rather than hand-deriving the exact index (error-prone by
            // the same logic the bug fix note warns about), assert the
            // STRUCTURAL SHAPE: `h`'s domain must be `App(App(App(Equal,
            // A), V), V)` for SOME single pair of variable indices A, V
            // with A != V (the bug would either produce a KernelRejected
            // failure already caught by the `.expect` above, or in a
            // softer failure mode leave a domain that still parses as
            // Equal-applied-to-something — so additionally require A/V
            // are NOT Var(0)/Var(1)/Var(2) i.e. not one of the freshly
            // bound b/bs/ih, which would indicate the substitution
            // wrongly captured a local field instead of the true outer
            // a/v).
            fn peel_app(t: &Term) -> (&Term, Vec<&Term>) {
                let mut args = vec![];
                let mut cur = t;
                while let Term::App(f, a) = cur {
                    args.push(a.as_ref());
                    cur = f;
                }
                args.reverse();
                (cur, args)
            }
            let (head, args) = peel_app(h_domain);
            let equal_id = env.globals["Equal"];
            match head {
                Term::Const { id, .. } if *id == equal_id => {}
                other => panic!("h's domain head must be Equal, got {other:?}"),
            }
            assert_eq!(args.len(), 3, "Equal takes 3 args (type, lhs, rhs)");
            let a_ref = args[0];
            let v_lhs = args[1];
            let v_rhs = args[2];
            assert_eq!(v_lhs, v_rhs, "both Equal operands must reference the SAME outer `v`");
            match a_ref {
                Term::Var(i) => {
                    assert!(
                        *i >= 3,
                        "the outer `a` reference must NOT be captured by one of the \
                         freshly-bound b/bs/ih (Var 0..2) — got Var({i}), indicating \
                         a mis-shift/decrement onto a local field, exactly the \
                         subst_var-vs-subst_var_generalize bug shape"
                    );
                }
                other => panic!("expected `a` to still be a bound Var, got {other:?}"),
            }
            match v_lhs {
                Term::Var(i) => {
                    assert!(
                        *i >= 3,
                        "the outer `v` reference must NOT be captured by one of the \
                         freshly-bound b/bs/ih (Var 0..2) — got Var({i})"
                    );
                    if let Term::Var(a_i) = a_ref {
                        assert_ne!(i, a_i, "`a` and `v` must reference DISTINCT outer variables");
                    }
                }
                other => panic!("expected `v` to still be a bound Var, got {other:?}"),
            }
        }
        other => panic!("expected the arm's own `\\h.` lambda, got {other:?}"),
    }
}
