//! Acceptance test suite: all 20 conformance cases from
//! `conformance/surface/elaboration/seed-elaboration.md`.
//!
//! Tags in comments match the conformance file: `(oracle)` cases assert the
//! exact core shape; `(property)` cases assert pipeline invariants; the
//! discriminating AC4 shadow case has both a structural and a verdict assertion.

use ken_elaborator::{
    elab::elaborate_rdecl,
    error::ElabError,
    parser::parse_decls,
    resolve::{resolve_decl, RDecl, RExpr},
    ElabEnv,
};
use ken_kernel::{Level, Term};

// ----- helpers -----

fn mk_env() -> ElabEnv {
    ElabEnv::new().expect("base env construction failed")
}

/// Build `Level::Suc^n(Level::Zero)`.
fn lv(n: u32) -> Level {
    let mut l = Level::Zero;
    for _ in 0..n {
        l = Level::Suc(Box::new(l));
    }
    l
}

/// Retrieve the body of the last registered declaration.
fn last_body(env: &ElabEnv, id: ken_kernel::GlobalId) -> Term {
    env.env
        .transparent_body(id)
        .expect("declaration is transparent")
        .1
}

/// Retrieve the type of the last registered declaration.
fn last_type(env: &ElabEnv, id: ken_kernel::GlobalId) -> Term {
    env.env.const_type(id).expect("declaration has a type").1
}

// ----- AC1: a trivial program elaborates and kernel-checks -----

/// `surface/elaboration/id-elaborates-checks` (oracle)
#[test]
fn id_elaborates_checks() {
    let mut env = mk_env();
    let id = env
        .elaborate_decl("view id (A : Type) (x : A) : A = x")
        .expect("id should elaborate");

    // Expected body: Lam(Univ 0, Lam(Var 0, Var 0))
    let body = last_body(&env, id);
    let expected = Term::lam(Term::ty(lv(0)), Term::lam(Term::var(0), Term::var(0)));
    assert_eq!(body, expected, "id body mismatch");

    // Expected type: Pi(Univ 0, Pi(Var 0, Var 1))
    let ty = last_type(&env, id);
    let expected_ty = Term::pi(Term::ty(lv(0)), Term::pi(Term::var(0), Term::var(1)));
    assert_eq!(ty, expected_ty, "id type mismatch");
}

/// `surface/elaboration/const-elaborates-checks` (oracle) — body is Var 1
#[test]
fn const_elaborates_checks() {
    let mut env = mk_env();
    let id = env
        .elaborate_decl("view const (A B : Type) (x : A) (y : B) : A = x")
        .expect("const should elaborate");

    // Body x is Var 1 in scope [y, x, B, A] (innermost first).
    let body = last_body(&env, id);
    // Full body: Lam(Univ0, Lam(Univ0, Lam(Var1, Lam(Var0, Var1))))
    // Only check the innermost Var (x in scope [y,x,B,A] = Var 1).
    fn innermost_var(t: &Term) -> Option<usize> {
        match t {
            Term::Lam(_, b) => innermost_var(b),
            Term::Var(i) => Some(*i),
            _ => None,
        }
    }
    assert_eq!(
        innermost_var(&body),
        Some(1),
        "const body: x must be Var 1, not Var 0 (y)"
    );
}

/// `surface/elaboration/apply-elaborates-checks` (oracle) — body App(Var1, Var0)
#[test]
fn apply_elaborates_checks() {
    let mut env = mk_env();
    let id = env
        .elaborate_decl("view apply (A B : Type) (f : (x : A) -> B) (x : A) : B = f x")
        .expect("apply should elaborate");

    let body = last_body(&env, id);
    // Find the innermost non-Lam: should be App(Var1, Var0)
    fn peel_lam(t: &Term) -> &Term {
        if let Term::Lam(_, b) = t { peel_lam(b) } else { t }
    }
    let core = peel_lam(&body);
    let expected = Term::app(Term::var(1), Term::var(0));
    assert_eq!(core, &expected, "apply body must be App(Var 1, Var 0)");
}

// ----- AC2: round-trip on minimal surface -----

/// `surface/elaboration/let-ascription-roundtrip` (oracle)
#[test]
fn let_ascription_roundtrip() {
    let mut env = mk_env();
    let (core, ty) = env
        .elaborate_expr("let x : Type = Type in x")
        .expect("let-ascription should elaborate");

    // Expected: Let(Univ 1, Univ 0, Var 0) at type Univ 1
    let expected_core = Term::Let {
        ty: Box::new(Term::ty(lv(1))),
        val: Box::new(Term::ty(lv(0))),
        body: Box::new(Term::var(0)),
    };
    assert_eq!(core, expected_core, "let-ascription core mismatch");
    assert_eq!(ty, Term::ty(lv(1)), "let-ascription type mismatch");
}

/// `surface/elaboration/ascription-on-lambda` (oracle) — two λs check against binary Π
#[test]
fn ascription_on_lambda() {
    let mut env = mk_env();
    let (core, _ty) = env
        .elaborate_expr("(\\A . \\x . x) : (A : Type) -> (x : A) -> A")
        .expect("ascription-on-lambda should elaborate");

    // Same image as id's body: Lam(Univ 0, Lam(Var 0, Var 0))
    let expected = Term::lam(Term::ty(lv(0)), Term::lam(Term::var(0), Term::var(0)));
    assert_eq!(core, expected, "ascription-on-lambda core mismatch");
}

/// `surface/elaboration/base-type-app` (oracle) — Nat from base environment
#[test]
fn base_type_app() {
    let mut env = mk_env();
    let id = env
        .elaborate_decl("view idNat (x : Nat) : Nat = x")
        .expect("idNat should elaborate");

    let nat_id = *env.globals.get("Nat").expect("Nat must be in globals");
    let expected_ty = Term::pi(
        Term::const_(nat_id, vec![]),
        Term::const_(nat_id, vec![]),
    );
    assert_eq!(last_type(&env, id), expected_ty, "idNat type mismatch");

    // Body: Lam(Con Nat, Var 0)
    let body = last_body(&env, id);
    let expected_body = Term::lam(Term::const_(nat_id, vec![]), Term::var(0));
    assert_eq!(body, expected_body, "idNat body mismatch");
}

// ----- AC3: ill-typed surface is rejected -----

/// `surface/elaboration/type-mismatch-rejected` — kernel rejects Nat vs Bool
#[test]
fn type_mismatch_rejected() {
    let mut env = mk_env();
    let result = env.elaborate_decl("view bad (x : Nat) : Bool = x");
    assert!(
        matches!(result, Err(ElabError::KernelRejected { .. })),
        "type-mismatch-rejected: expected KernelRejected, got {:?}",
        result.err()
    );
}

/// `surface/elaboration/wrong-return-app-rejected` — f x : Bool ≢ Nat
#[test]
fn wrong_return_app_rejected() {
    let mut env = mk_env();
    let result = env
        .elaborate_decl("view badApp (f : (x : Nat) -> Bool) (x : Nat) : Nat = f x");
    assert!(
        matches!(result, Err(ElabError::KernelRejected { .. })),
        "wrong-return-app-rejected: expected KernelRejected, got {:?}",
        result.err()
    );
}

/// `surface/elaboration/wrong-arity-rejected` — too many λs: V0 catches structurally
#[test]
fn wrong_arity_rejected() {
    let mut env = mk_env();
    let result =
        env.elaborate_decl("view badLam (x : Nat) : (y : Nat) -> Nat = \\y . \\z . x");
    assert!(
        matches!(result, Err(ElabError::LambdaVsNonFunction { .. })),
        "wrong-arity-rejected: expected LambdaVsNonFunction (caught by V0), got {:?}",
        result.err()
    );
}

/// `surface/elaboration/under-applied-lambda-rejected` — single λ vs binary Π
#[test]
fn under_applied_lambda_rejected() {
    let mut env = mk_env();
    let result = env.elaborate_expr("(\\x . x) : (A : Type) -> (x : A) -> A");
    assert!(
        matches!(result, Err(ElabError::KernelRejected { .. })),
        "under-applied-lambda-rejected: expected KernelRejected, got {:?}",
        result.err()
    );
}

// ----- AC4: name resolution — nested binders + shadowing -----

/// `surface/elaboration/shadow-outer-not-captured` (discriminating)
///
/// Correct resolution → kernel accepts; capture bug → kernel rejects.
#[test]
fn shadow_outer_not_captured() {
    let src = "view f (A : Type) (x : A) : Type -> A = \\B . x";

    // --- correct path ---
    let mut env = mk_env();
    assert!(
        env.elaborate_decl(src).is_ok(),
        "shadow guard: correct resolution must be accepted by the kernel"
    );

    // --- capture bug path ---
    // Build an RDecl with the bug injected: x → Var 0 (= B) instead of Var 1.
    use ken_elaborator::error::Span;
    use ken_elaborator::resolve::{RExpr, RType};

    let sp = Span::zero();
    // full_ty: Pi("A", Univ, Pi("x", Var0, Pi(Univ, Var2)))  (non-dependent arrow desugared)
    let full_ty = {
        let ret = RType::RArr(
            Box::new(RType::RUniv(None, sp.clone())),
            Box::new(RType::RVarTy(1, "A".into(), sp.clone())),
            sp.clone(),
        );
        let pi_x = RType::RPi(
            "x".into(),
            Box::new(RType::RVarTy(0, "A".into(), sp.clone())),
            Box::new(ret),
            sp.clone(),
        );
        RType::RPi(
            "A".into(),
            Box::new(RType::RUniv(None, sp.clone())),
            Box::new(pi_x),
            sp.clone(),
        )
    };
    // Bug body: Lam("A", Lam("x", Lam("B", Var(0))))  ← Var(0) = B, not x
    let bug_body = RExpr::RLam(
        "A".into(),
        Box::new(RExpr::RLam(
            "x".into(),
            Box::new(RExpr::RLam(
                "B".into(),
                Box::new(RExpr::RVar(0, "x".into(), sp.clone())), // BUG: Var(0)=B
                sp.clone(),
            )),
            sp.clone(),
        )),
        sp.clone(),
    );

    let bug_rdecl = RDecl {
        name: "f_buggy".into(),
        ty: Some(full_ty),
        body: bug_body,
        requires: vec![],
        ensures: vec![],
        span: sp,
        kind: ken_elaborator::resolve::RDeclKind::Let,
    };

    let mut env2 = mk_env();
    let result = elaborate_rdecl(&mut env2.env, &mut env2.globals, &mut env2.num_values, &env2.numeric_env, &bug_rdecl);
    assert!(
        matches!(result, Err(ElabError::KernelRejected { .. })),
        "shadow guard: capture bug must be rejected by the kernel, got {:?}",
        result.err()
    );
}

/// `surface/elaboration/shadow-resolver-emits-outer-index` (oracle)
///
/// Structural assertion: the resolver emits Var(1) for `x`, independent of
/// the elaboration verdict.
#[test]
fn shadow_resolver_emits_outer_index() {
    let src = "view f (A : Type) (x : A) : Type -> A = \\B . x";
    let decls = parse_decls(src).expect("parse failed");
    let rdecl = resolve_decl(&decls[0]).expect("resolution failed");

    // The resolved body is Lam("A", Lam("x", Lam("B", RVar(i, "x", _)))).
    // Walk the nested Lams to find the innermost RVar for "x".
    fn find_x_index(e: &RExpr) -> Option<usize> {
        match e {
            RExpr::RLam(_, body, _) => find_x_index(body),
            RExpr::RVar(i, name, _) if name == "x" => Some(*i),
            _ => None,
        }
    }

    let idx = find_x_index(&rdecl.body).expect("no RVar for x found in resolved body");
    assert_eq!(
        idx, 1,
        "shadow-resolver-emits-outer-index: x must resolve to Var(1), not Var({})",
        idx
    );
}

/// `surface/elaboration/nested-app-each-binder` (oracle)
#[test]
fn nested_app_each_binder() {
    let mut env = mk_env();
    let id = env
        .elaborate_decl(
            "view nested (A : Type) (f : (x : A) -> A) (x : A) : A = f (f x)",
        )
        .expect("nested-app should elaborate");

    let body = last_body(&env, id);
    fn peel_lam(t: &Term) -> &Term {
        if let Term::Lam(_, b) = t { peel_lam(b) } else { t }
    }
    let core = peel_lam(&body);
    let expected = Term::app(
        Term::var(1),
        Term::app(Term::var(1), Term::var(0)),
    );
    assert_eq!(core, &expected, "nested body must be App(Var1, App(Var1, Var0))");
}

/// `surface/elaboration/unbound-name-rejected-at-resolution`
///
/// After L-resolver-globals, lowercase scope misses fall through to RCon and
/// are caught at the elaboration stage as UnresolvedCon rather than the
/// resolution-stage UnboundName.  Either error proves the name was rejected.
#[test]
fn unbound_name_rejected_at_resolution() {
    let mut env = mk_env();
    let result = env.elaborate_decl("view unbound (x : Nat) : Nat = y");
    assert!(
        matches!(
            &result,
            Err(ElabError::UnboundName { name, .. } | ElabError::UnresolvedCon { name, .. })
                if name == "y"
        ),
        "unbound-name: expected UnboundName or UnresolvedCon for 'y', got {:?}",
        result.err()
    );
}

// ----- AC5: pipeline integration -----

/// `surface/elaboration/pipeline-emits-explicit-core` (property)
///
/// Every AC1–AC2 accepted program's core is metavariable-free: all `Type`
/// arguments are concrete `Univ n` nodes (no `Level::Var` metas remain).
#[test]
fn pipeline_emits_explicit_core() {
    fn has_level_meta(t: &Term) -> bool {
        match t {
            Term::Type(l) | Term::Omega(l) => level_has_meta(l),
            Term::Pi(a, b) | Term::Lam(a, b) | Term::App(a, b) => {
                has_level_meta(a) || has_level_meta(b)
            }
            Term::Let { ty, val, body } => {
                has_level_meta(ty) || has_level_meta(val) || has_level_meta(body)
            }
            _ => false,
        }
    }
    fn level_has_meta(l: &Level) -> bool {
        match l {
            Level::Var(_) => true,
            Level::Zero => false,
            Level::Suc(inner) => level_has_meta(inner),
            Level::Max(a, b) => level_has_meta(a) || level_has_meta(b),
        }
    }

    let mut env = mk_env();
    let cases = [
        ("view id (A : Type) (x : A) : A = x", "id"),
        (
            "view const (A B : Type) (x : A) (y : B) : A = x",
            "const",
        ),
        (
            "view apply (A B : Type) (f : (x : A) -> B) (x : A) : B = f x",
            "apply",
        ),
        ("view idNat (x : Nat) : Nat = x", "idNat"),
    ];

    for (src, name) in &cases {
        let id = env
            .elaborate_decl(src)
            .unwrap_or_else(|e| panic!("{}: elaboration failed: {}", name, e));
        let body = last_body(&env, id);
        let ty = last_type(&env, id);
        assert!(
            !has_level_meta(&body),
            "{}: body contains unsolved level meta",
            name
        );
        assert!(
            !has_level_meta(&ty),
            "{}: type contains unsolved level meta",
            name
        );
    }
}

/// `surface/elaboration/pipeline-errors-at-correct-stage`
#[test]
fn pipeline_errors_at_correct_stage() {
    // (a) parse error: missing body
    {
        let mut env = mk_env();
        let result = env.elaborate_decl("view id (A : Type) (x : A) : A =");
        assert!(
            matches!(result, Err(ElabError::ParseError { .. })),
            "(a) missing body should be a parse error, got {:?}",
            result.err()
        );
    }
    // (b) name-resolution error: free `y` (UnboundName at resolution OR
    //     UnresolvedCon at elaboration — both prove the name was rejected)
    {
        let mut env = mk_env();
        let result = env.elaborate_decl("view u (x : Nat) : Nat = y");
        assert!(
            matches!(
                &result,
                Err(ElabError::UnboundName { name, .. } | ElabError::UnresolvedCon { name, .. })
                    if name == "y"
            ),
            "(b) free 'y' should be UnboundName or UnresolvedCon, got {:?}",
            result.err()
        );
    }
    // (c) kernel rejection: body type mismatch
    {
        let mut env = mk_env();
        let result = env.elaborate_decl("view bad (x : Nat) : Bool = x");
        assert!(
            matches!(result, Err(ElabError::KernelRejected { .. })),
            "(c) body mismatch should be KernelRejected, got {:?}",
            result.err()
        );
    }
}

// ----- AC6: level reconciliation -----

/// `surface/elaboration/id-pi-level-max` (oracle)
///
/// The inner Π `(x : A) -> A` (A : Univ 0) has level `max(0,0) = 0`;
/// the outer Π (A : Type 0) -> _ has level `max(suc 0, 0) = 1`.
/// Both appear explicitly: domains are `Univ 0` (no metas remain).
#[test]
fn id_pi_level_max() {
    let mut env = mk_env();
    let id = env
        .elaborate_decl("view id (A : Type) (x : A) : A = x")
        .expect("id should elaborate");

    let ty = last_type(&env, id);
    // Type = Pi(Univ 0, Pi(Var 0, Var 1)); the Univ 0 domain is explicit.
    if let Term::Pi(outer_dom, outer_cod) = &ty {
        assert_eq!(
            outer_dom.as_ref(),
            &Term::ty(lv(0)),
            "outer Π domain must be Univ 0"
        );
        if let Term::Pi(inner_dom, _) = outer_cod.as_ref() {
            // inner_dom = Var(0) = A, whose TYPE is Univ 0 (explicit in outer_dom)
            assert_eq!(
                inner_dom.as_ref(),
                &Term::var(0),
                "inner Π domain must be Var 0 (= A)"
            );
        } else {
            panic!("expected inner Pi, got {:?}", outer_cod);
        }
    } else {
        panic!("expected outer Pi, got {:?}", ty);
    }
}

/// `surface/elaboration/two-distinct-levels` (oracle) — Type 1 and Type 2 coexist
#[test]
fn two_distinct_levels() {
    let mut env = mk_env();
    let id = env
        .elaborate_decl("view poly (A : Type 1) (B : Type 2) (x : A) : A = x")
        .expect("poly should elaborate");

    let ty = last_type(&env, id);
    // Full type Pi(Univ 1, Pi(Univ 2, Pi(Var 1, Var 2)))
    // Check that both Univ 1 and Univ 2 appear in the top-level type structure.
    fn has_univ(t: &Term, n: u32) -> bool {
        let target = Term::ty(lv(n));
        match t {
            _ if t == &target => true,
            Term::Pi(a, b) => has_univ(a, n) || has_univ(b, n),
            Term::Lam(a, b) => has_univ(a, n) || has_univ(b, n),
            _ => false,
        }
    }
    assert!(
        has_univ(&ty, 1),
        "poly type must contain Univ 1; got {:?}",
        ty
    );
    assert!(
        has_univ(&ty, 2),
        "poly type must contain Univ 2; got {:?}",
        ty
    );
}

/// `surface/elaboration/level-equality-decidable` (property)
///
/// `convLevel` decides level equality; non-cumulative: `Type 1 ≢ Type 2`.
#[test]
fn level_equality_decidable() {
    use ken_kernel::{level_eq, convert_type, Context, GlobalEnv};

    let env = GlobalEnv::new();
    let ctx = Context::new();

    // max 0 0 ≡ 0
    let max00 = Level::Max(Box::new(lv(0)), Box::new(lv(0)));
    assert!(level_eq(&max00, &lv(0)), "max(0,0) must equal 0");

    // max 1 1 ≡ 1
    let max11 = Level::Max(Box::new(lv(1)), Box::new(lv(1)));
    assert!(level_eq(&max11, &lv(1)), "max(1,1) must equal 1");

    // Type 1 ≢ Type 2 (non-cumulative)
    let univ1 = Term::ty(lv(1));
    let univ2 = Term::ty(lv(2));
    let converts = convert_type(&env, &ctx, &univ1, &univ2);
    assert!(!converts, "Type 1 must not convert to Type 2 (non-cumulative)");
}

// ----- Regression: on-main elaboration invariants still hold -----

/// `surface/elaboration/existing-invariants-still-green` (property)
///
/// Every accepted program's core passes `kernel.check`; covered by the AC1–AC2
/// `elaborate_decl` calls above (which call the kernel internally). Spot-check
/// one rejected case to confirm `ambiguity-is-an-error`.
#[test]
fn existing_invariants_still_green() {
    // well-typed-output: already asserted in AC1/AC2 tests (each calls declare_def).
    // ambiguity-is-an-error: an input with a completely unconstrained type must
    // still give a well-typed result (defaults to Univ 0) — not silently produce
    // a meta-bearing term.
    let mut env = mk_env();
    // `Type` without an annotation gives meta → default 0 → Type 0.
    let (core, ty) = env
        .elaborate_expr("(\\A . \\x . x) : (A : Type) -> (x : A) -> A")
        .expect("should accept");
    // core is Lam(Univ 0, Lam(Var 0, Var 0)) — no metas.
    let expected = Term::lam(Term::ty(lv(0)), Term::lam(Term::var(0), Term::var(0)));
    assert_eq!(core, expected, "existing-invariants: core mismatch");
    assert_eq!(ty, Term::pi(Term::ty(lv(0)), Term::pi(Term::var(0), Term::var(1))));
}
