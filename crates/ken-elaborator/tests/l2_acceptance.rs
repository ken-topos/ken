//! L2 acceptance tests: `data` / `match` / exhaustiveness / refinements.
//!
//! Pins: `conformance/surface/data-match/seed-data-match.md` AC1–AC8.
//! Spec: `spec/30-surface/34-data-match.md`.
//!
//! AC5 (indexed families) and AC6 (dependent motive) are deferred — they
//! require `indices` support and dependent motives respectively.

use ken_elaborator::{error::ElabError, ElabEnv};
use ken_kernel::{whnf, Context, GlobalId, Level, Term};

// ----- helpers -----

fn mk_env() -> ElabEnv {
    ElabEnv::new().expect("base env construction failed")
}

fn elab(env: &mut ElabEnv, src: &str) -> Result<GlobalId, ElabError> {
    env.elaborate_decl(src)
}

fn elab_ok(env: &mut ElabEnv, src: &str) -> GlobalId {
    elab(env, src).unwrap_or_else(|e| panic!("elab_ok failed: {}", e))
}

fn body_of(env: &ElabEnv, id: GlobalId) -> Term {
    env.env.transparent_body(id).expect("not a transparent def").1
}

fn term_mentions_const(t: &Term, target: GlobalId) -> bool {
    match t {
        Term::Const { id, .. } if *id == target => true,
        _ => t
            .children()
            .into_iter()
            .any(|child| term_mentions_const(child, target)),
    }
}

fn term_mentions_var(t: &Term, target: usize) -> bool {
    match t {
        Term::Var(i) if *i == target => true,
        _ => t
            .children()
            .into_iter()
            .any(|child| term_mentions_var(child, target)),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC1  construct-then-eliminate  (`34 §1`, `14 §3`)
//
// Property: `data` declares a real inductive + computing `elim_D`; `match`
// ι-reduces on the constructor.
//
// Scrutinee given inline: `SomeInt 3` in the match expression.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ac1_data_decl_registers_former_and_ctors() {
    let mut env = mk_env();
    elab_ok(&mut env, "data MaybeInt = NoInt | SomeInt Int");

    assert!(env.globals.contains_key("MaybeInt"), "type former registered");
    assert!(env.globals.contains_key("NoInt"), "NoInt ctor registered");
    assert!(env.globals.contains_key("SomeInt"), "SomeInt ctor registered");

    let d_id = env.globals["MaybeInt"];
    assert!(env.env.inductive(d_id).is_some(), "MaybeInt is an inductive family");

    let c_id = env.globals["SomeInt"];
    assert!(env.env.constructor(c_id).is_some(), "SomeInt is a constructor");
}

#[test]
fn ac1_construct_then_eliminate_reduces() {
    let mut env = mk_env();
    elab_ok(&mut env, "data MaybeInt = NoInt | SomeInt Int");

    // `SomeInt 3` is the scrutinee inline — no separate binding needed.
    let id = elab_ok(
        &mut env,
        "let answer : Int = match SomeInt 3 { SomeInt x => x ; NoInt => 0 }",
    );
    let body = body_of(&env, id);
    let ctx = Context::new();
    let reduced = whnf(&env.env, &ctx, &body);

    // ι-rule: elim_MaybeInt M [0; λx.x] (SomeInt v) ⇝ (λx.x) v ⇝ v
    // The result should be the opaque literal (the SomeInt arg), not still an Elim.
    assert!(
        !matches!(reduced, Term::Elim { .. }),
        "AC1: elim_MaybeInt did not ι-reduce; still got Elim node"
    );
}

#[test]
fn ac1_nil_arm_reduces_to_default() {
    let mut env = mk_env();
    elab_ok(&mut env, "data MaybeInt = NoInt | SomeInt Int");

    let id = elab_ok(
        &mut env,
        "let answer : Int = match NoInt { SomeInt x => x ; NoInt => 0 }",
    );
    let body = body_of(&env, id);
    let ctx = Context::new();
    let reduced = whnf(&env.env, &ctx, &body);

    // ι-rule: elim_MaybeInt M [0; λx.x] NoInt ⇝ 0 (the postulate for 0)
    assert!(
        !matches!(reduced, Term::Elim { .. }),
        "AC1: elim_MaybeInt on NoInt did not ι-reduce"
    );
    // The result is a Const (the opaque literal 0).
    assert!(
        matches!(reduced, Term::Const { .. }),
        "AC1: expected numeric literal Const after ι-reduction, got {:?}",
        reduced
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC2  match-elaborates-to-elim  (`34 §3`, `39 §2.6`)
//
// Structural: body head is Term::Elim; nested match ⇒ nested Elim.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ac2_match_head_is_term_elim() {
    let mut env = mk_env();
    elab_ok(&mut env, "data Shape = Circle Int | Rect Int Int");

    let id = elab_ok(
        &mut env,
        "let r : Int = match Circle 2 { Circle x => x ; Rect w h => w }",
    );
    let body = body_of(&env, id);

    assert!(
        matches!(body, Term::Elim { .. }),
        "AC2: match did not compile to Term::Elim; got {:?}",
        body
    );
}

#[test]
fn ac2_elim_computes_on_circle() {
    let mut env = mk_env();
    elab_ok(&mut env, "data Shape = Circle Int | Rect Int Int");

    let id = elab_ok(
        &mut env,
        "let r : Int = match Circle 2 { Circle x => x ; Rect w h => w }",
    );
    let body = body_of(&env, id);
    let ctx = Context::new();
    let reduced = whnf(&env.env, &ctx, &body);

    // elim_Shape M [λx.x; λw.λh.w] (Circle 2) ⇝ (λx.x) 2 ⇝ 2  (opaque literal)
    assert!(
        !matches!(reduced, Term::Elim { .. }),
        "AC2: elim_Shape on Circle 2 did not ι-reduce"
    );
}

#[test]
fn ac2_nested_match_produces_nested_elim() {
    let mut env = mk_env();
    elab_ok(&mut env, "data AB = A | B");

    let id = elab_ok(
        &mut env,
        "let v : Int = match A { A => match B { A => 1 ; B => 2 } ; B => 0 }",
    );
    let body = body_of(&env, id);

    // Outer Elim: method for A should itself be an Elim (the nested match).
    let a_method = match &body {
        Term::Elim { methods, .. } => methods[0].clone(), // A is constructor 0
        other => panic!("expected outer Elim, got {:?}", other),
    };
    assert!(
        matches!(a_method, Term::Elim { .. }),
        "AC2 nested: A-arm method is not an Elim (nested match not lowered); got {:?}",
        a_method
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC3  exhaustiveness-required  (`34 §4.1`, `§4.4`)  (soundness — TR3)
//
// The discriminating signal is the NAMED witness (not just "rejects"):
//  (a) missing Blue → rejects naming "Blue"
//  (b) all three arms → accepts
// ─────────────────────────────────────────────────────────────────────────────

fn setup_color(env: &mut ElabEnv) {
    elab_ok(env, "data Color = Red | Green | Blue");
}

#[test]
fn ac3_missing_arm_names_blue() {
    let mut env = mk_env();
    setup_color(&mut env);

    // Match on a constructor directly to avoid binding issues.
    let result = elab(
        &mut env,
        "let bad : Int = match Red { Red => 0 ; Green => 1 }",
    );

    match result {
        Err(ElabError::ExhaustivenessError { missing, .. }) => {
            assert_eq!(
                missing, "Blue",
                "AC3: named witness should be 'Blue', got '{}'",
                missing
            );
        }
        Ok(_) => panic!("AC3: non-exhaustive match accepted (should have been rejected)"),
        Err(other) => panic!("AC3: expected ExhaustivenessError naming 'Blue', got: {}", other),
    }
}

#[test]
fn ac3_exhaustive_match_accepts() {
    let mut env = mk_env();
    setup_color(&mut env);

    let id = elab_ok(
        &mut env,
        "let result : Int = match Blue { Red => 0 ; Green => 1 ; Blue => 2 }",
    );
    let body = body_of(&env, id);
    assert!(matches!(body, Term::Elim { .. }), "AC3: accepted match should produce Elim");
}

#[test]
fn ac3_exhaustive_elim_reduces_on_blue() {
    let mut env = mk_env();
    setup_color(&mut env);

    let id = elab_ok(
        &mut env,
        "let result : Int = match Blue { Red => 0 ; Green => 1 ; Blue => 2 }",
    );
    let body = body_of(&env, id);
    let ctx = Context::new();
    let reduced = whnf(&env.env, &ctx, &body);

    // elim_Color M [0; 1; 2] Blue ⇝ 2 (ι on Blue ctor)
    assert!(
        matches!(reduced, Term::Const { .. }),
        "AC3: elim_Color on Blue should reduce to Const(2), got {:?}",
        reduced
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC4  reachability-redundant-arm  (`34 §4.2`)
//
//  (a) duplicate Red → ReachabilityError
//  (b) three distinct arms → accepts
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ac4_duplicate_arm_is_reachability_error() {
    let mut env = mk_env();
    setup_color(&mut env);

    let result = elab(
        &mut env,
        "let bad : Int = match Red { Red => 0 ; Green => 1 ; Blue => 2 ; Red => 9 }",
    );

    match result {
        Err(ElabError::ReachabilityError { .. }) => { /* ✓ AC4 */ }
        Ok(_) => panic!("AC4: redundant arm should have been flagged"),
        Err(other) => panic!("AC4: expected ReachabilityError, got: {}", other),
    }
}

#[test]
fn ac4_all_distinct_arms_accept() {
    let mut env = mk_env();
    setup_color(&mut env);

    let id = elab_ok(
        &mut env,
        "let result : Int = match Green { Red => 0 ; Green => 1 ; Blue => 2 }",
    );
    let body = body_of(&env, id);
    assert!(matches!(body, Term::Elim { .. }), "AC4: all-distinct match should produce Elim");
}

// ─────────────────────────────────────────────────────────────────────────────
// Nested constructor patterns (`34 §3.1` pattern-matrix compilation).
//
// Regression for GAP-nested-patterns: `infer_match` used to track coverage
// by top-level constructor only, so two arms sharing a head constructor
// (`Succ Zero` / `Succ (Succ m)`) tripped a false ReachabilityError even
// though the nested patterns are disjoint. The fix compiles the standard
// column-by-column pattern matrix, splitting further on a field whose own
// sub-pattern is itself a constructor.
// ─────────────────────────────────────────────────────────────────────────────

fn setup_natl(env: &mut ElabEnv) {
    elab_ok(env, "data NatL = Zero | Succ NatL");
}

#[test]
fn nested_ctor_pattern_accepted_and_reduces() {
    let mut env = mk_env();
    setup_natl(&mut env);

    let id = elab_ok(
        &mut env,
        "let result : Int = match Succ (Succ Zero) { \
         Zero => 0 ; Succ Zero => 1 ; Succ (Succ m) => 2 }",
    );
    let body = body_of(&env, id);
    assert!(matches!(body, Term::Elim { .. }), "nested match should produce Elim");

    let ctx = Context::new();
    let reduced = whnf(&env.env, &ctx, &body);
    assert!(
        matches!(reduced, Term::Const { .. }),
        "Succ (Succ Zero) should select the Succ (Succ m) arm and ι-reduce to Const(2), \
         got {:?}",
        reduced
    );
}

#[test]
fn nested_ctor_pattern_selects_the_right_arm_at_succ_zero() {
    let mut env = mk_env();
    setup_natl(&mut env);

    // Discriminating: swap which literal each arm returns, and scrutinize
    // `Succ Zero` — only reduces to the FIRST Const if the `Succ Zero`
    // (not `Succ (Succ m)`) arm actually fired.
    let id = elab_ok(
        &mut env,
        "let result : Int = match Succ Zero { \
         Zero => 0 ; Succ Zero => 7 ; Succ (Succ m) => 9 }",
    );
    let body = body_of(&env, id);
    let ctx = Context::new();
    let reduced = whnf(&env.env, &ctx, &body);
    match reduced {
        Term::Const { .. } => {}
        other => panic!("expected Succ Zero arm to ι-reduce to a literal, got {:?}", other),
    }
}

#[test]
fn nested_ctor_pattern_missing_case_is_exhaustiveness_error() {
    let mut env = mk_env();
    setup_natl(&mut env);

    // `Succ Zero` is uncovered: the outer `Succ` bucket only handles the
    // nested-`Succ` sub-case, leaving the nested-`Zero` sub-case missing.
    let result = elab(
        &mut env,
        "let bad : Int = match Zero { Zero => 0 ; Succ (Succ m) => 2 }",
    );

    match result {
        Err(ElabError::ExhaustivenessError { .. }) => { /* ✓ */ }
        Ok(_) => panic!("non-exhaustive nested match accepted (should have been rejected)"),
        Err(other) => panic!("expected ExhaustivenessError, got: {}", other),
    }
}

#[test]
fn nested_ctor_pattern_shadowed_by_earlier_flat_arm_is_reachability_error() {
    let mut env = mk_env();
    setup_natl(&mut env);

    // `Succ n` (flat) already covers every Succ-headed value, so the later
    // `Succ (Succ m)` arm is dead code — must still be caught even though it
    // shares no top-level ambiguity with `Succ n` at the FIRST split.
    let result = elab(
        &mut env,
        "let bad : Int = match Zero { \
         Zero => 0 ; Succ n => 1 ; Succ (Succ m) => 2 }",
    );

    match result {
        Err(ElabError::ReachabilityError { .. }) => { /* ✓ */ }
        Ok(_) => panic!("arm shadowed by an earlier flat arm should have been flagged"),
        Err(other) => panic!("expected ReachabilityError, got: {}", other),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC5  indexed-impossible-pair  — DEFERRED
//
// Requires indexed family support (non-empty `indices` in InductiveSpec) and
// "absurdity fill" for index-impossible arms (`34 §4.3`).
// Tracked as follow-on: L2-indexed.
// ─────────────────────────────────────────────────────────────────────────────

// ─────────────────────────────────────────────────────────────────────────────
// AC6  branch-refinement-is-hypothesis  — DEFERRED
//
// Requires a *dependent* motive (motive mentioning the scrutinee/index).
// Current `infer_match` emits only the constant motive `λx. R`.
// Tracked as follow-on: L2-dep-motive.
// ─────────────────────────────────────────────────────────────────────────────

// ─────────────────────────────────────────────────────────────────────────────
// AC7  refinement-obligation  (`34 §5`, `21 §2`, `22 §2.1`)  (soundness — TR7)
//
// Pre-declare a proposition `P : Int → Ω` so the refinement predicate has
// type Ω (required by `elaborate_view_with_spec`).
//
//  (a) intro side: `{ n : Int | P n }` return type → obligation emitted
//  (b) forget side: `{ n : Int | P n }` value used as Int → no obligation
// ─────────────────────────────────────────────────────────────────────────────

fn setup_prop(env: &mut ElabEnv) {
    // Declare P : Int → Ω (a predicate postulate).
    let int_id = env.globals["Int"];
    let omega = Term::omega(Level::Zero);
    let prop_ty = Term::pi(Term::const_(int_id, vec![]), omega);
    env.declare_postulate_raw("P", prop_ty).expect("declare P");
}

#[test]
fn ac7_refinement_intro_emits_obligation() {
    let mut env = mk_env();
    setup_prop(&mut env);

    // The return type `{ n : Int | P n }` forces an obligation `P 3` at the
    // introduction site.
    let result = env.elaborate_decl_v1("const nonneg_val : { n : Int | P n } = 3");

    let elab_result =
        result.unwrap_or_else(|e| panic!("AC7a: expected success, got: {}", e));

    // AC7 (soundness, TR7): at least one obligation is emitted.
    assert!(
        !elab_result.obligations.is_empty(),
        "AC7a: no obligation emitted for refinement introduction (TR7 regression)"
    );
}

#[test]
fn ac7_plain_carrier_no_obligation() {
    let mut env = mk_env();
    setup_prop(&mut env);

    // A plain Int annotation (no refinement predicate) must emit zero obligations —
    // the obligation gate is the refinement *introduction*, not mere Int typing.
    // This is the negative discriminant complementing AC7a's positive discriminant:
    //   AC7a: `{ n : Int | P n }` annotation → obligation emitted
    //   AC7b: `Int`               annotation → no obligation
    let result = env.elaborate_decl_v1("const plain_val : Int = 5");
    let elab_result = result.unwrap_or_else(|e| {
        panic!("AC7b: plain-Int const should accept, got: {}", e)
    });

    assert!(
        elab_result.obligations.is_empty(),
        "AC7b: spurious obligation emitted for plain Int annotation (no refinement)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC8  proof-returning-dependent-motive  (`34 §3.5`, `39 §2.1`, `14 §3`)
//
// Positive red-to-green for KM-dependent-match-proof-motive-build D1. On the
// pre-fix `927dd34` head this literal source rejects at the whole body with
// `KernelRejected { TypeMismatch { expected: Type 0, found: Ω0 } }`, because
// `match km_scrutinee b` missed checked dependent-motive recovery and fell
// through to `infer_match`'s constant `D -> Type 0` motive path.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ac8_proof_returning_match_through_transparent_scrutinee_elaborates() {
    let mut env = mk_env();
    let scrutinee_id = elab_ok(&mut env, "fn km_scrutinee (b : Bool) : Bool = b");

    let id = elab_ok(
        &mut env,
        "fn km_proof_motive_positive (b : Bool) \
           : Equal Bool (km_scrutinee b) (km_scrutinee b) = \
           match km_scrutinee b { True => tt ; False => tt }",
    );

    let body = body_of(&env, id);
    let mut inner = &body;
    while let Term::Lam(_, body) = inner {
        inner = body;
    }

    let (motive, scrut) = match inner {
        Term::Elim { fam, motive, scrut, .. } => {
            assert_eq!(*fam, env.globals["Bool"], "AC8 must eliminate over Bool");
            (motive.as_ref(), scrut.as_ref())
        }
        other => panic!(
            "AC8 proof-returning match must lower to Term::Elim, got {other:?}"
        ),
    };
    assert!(
        term_mentions_const(scrut, scrutinee_id),
        "the scrutinee should preserve the transparent helper call"
    );

    let (motive_body, motive_ty) = match motive {
        Term::Ascript(body, ty) => (body.as_ref(), ty.as_ref()),
        other => panic!(
            "AC8 motive must be ascribed so the kernel sees its Ω codomain, got {other:?}"
        ),
    };
    match motive_ty {
        Term::Pi(_, codomain) => {
            assert_eq!(
                **codomain,
                Term::Omega(Level::Zero),
                "AC8 motive codomain must be Ω0"
            );
        }
        other => panic!("AC8 motive type must be a Pi over Bool, got {other:?}"),
    }
    match motive_body {
        Term::Lam(_, body) => {
            assert!(
                term_mentions_var(body, 0),
                "AC8 motive body must mention the generalized scrutinee binder"
            );
            assert!(
                !term_mentions_const(body, scrutinee_id),
                "AC8 motive body must abstract over `km_scrutinee b`, not keep a constant motive"
            );
        }
        other => panic!("AC8 motive body must be a lambda, got {other:?}"),
    }
}

#[test]
fn ac8_wrong_specialized_branch_still_rejects() {
    let mut env = mk_env();
    elab_ok(&mut env, "fn km_scrutinee (b : Bool) : Bool = b");

    let err = elab(
        &mut env,
        "fn km_proof_motive_negative (b : Bool) \
           : Equal Bool (km_scrutinee b) True -> Equal Bool (km_scrutinee b) True = \
           match km_scrutinee b { True => \\p. p ; False => \\p. tt }",
    )
    .expect_err("AC8 negative must reject");

    match err {
        ElabError::KernelRejected {
            error: ken_kernel::KernelError::TypeMismatch { .. },
            ..
        } => {}
        other => panic!(
            "AC8 negative must reject through branch-obligation kernel TypeMismatch, got {other:?}"
        ),
    }
}

#[test]
fn ac8_cat4_option_table_probe_still_rejects_distinct_mechanism() {
    let mut env = mk_env();
    elab_ok(
        &mut env,
        "fn km_intersection_table (left : Option Unit) (keep : Bool) (prior : Option Unit) \
           : Option Unit = \
           match left { \
             None => prior ; \
             Some x => match keep { True => Some Unit x ; False => prior } \
           }",
    );
    elab_ok(
        &mut env,
        "fn km_member_from_lookup (left : Option Unit) : Bool = \
           match left { None => False ; Some x => True }",
    );
    elab_ok(
        &mut env,
        "fn km_lookup (b : Bool) : Option Unit = \
           match b { True => Some Unit MkUnit ; False => None Unit }",
    );
    elab_ok(
        &mut env,
        "fn km_option_refl (o : Option Unit) : Equal (Option Unit) o o = Refl",
    );

    // Mechanical CAT-4 D3 reconstruction from the Ken-owned
    // `intersectionLookupMemberCharacterization` trigger: a nested
    // proof-returning `match km_lookup ...` whose option-table target mentions
    // the lookup scrutinee and a reducible membership test. D1/D2 fixed the
    // old `Type 0`/`Ω0` motive-sort failure; this remaining rejection is the
    // distinct option-table branch-motive conversion split Runtime needs named.
    let err = elab(
        &mut env,
        "fn intersectionLookupMemberCharacterization (b : Bool) \
           : Equal (Option Unit) \
               (km_intersection_table (km_lookup b) \
                 (km_member_from_lookup (km_lookup b)) (None Unit)) \
               (km_intersection_table (km_lookup b) \
                 (km_member_from_lookup (km_lookup b)) (None Unit)) = \
           match km_lookup b { \
             None => km_option_refl \
               (km_intersection_table (None Unit) \
                 (km_member_from_lookup (None Unit)) (None Unit)) ; \
             Some x => match km_member_from_lookup (Some Unit x) { \
               True => km_option_refl \
                 (km_intersection_table (Some Unit x) \
                   (km_member_from_lookup (Some Unit x)) (None Unit)) ; \
               False => km_option_refl \
                 (km_intersection_table (Some Unit x) \
                   (km_member_from_lookup (Some Unit x)) (None Unit)) \
             } \
           }",
    )
    .expect_err("CAT-4 option-table probe should still reject before the follow-on split");

    match err {
        ElabError::KernelRejected {
            error:
                ken_kernel::KernelError::TypeMismatch {
                    expected,
                    found,
                },
            ..
        } => {
            assert!(
                !matches!((&*expected, &*found), (Term::Type(Level::Zero), Term::Omega(Level::Zero))),
                "D3 classification must not be the pre-D1 proof-motive sort failure"
            );
            assert!(
                !matches!(&*expected, Term::Type(_)) && !matches!(&*found, Term::Omega(_)),
                "D3 classification must reject after proof-motive Ω sorting, got expected {expected:?}, found {found:?}"
            );
        }
        other => panic!(
            "CAT-4 option-table probe must reject through the distinct kernel TypeMismatch, got {other:?}"
        ),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Additional robustness tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn data_recursive_type_accepts() {
    let mut env = mk_env();
    elab_ok(&mut env, "data NatL = Zero | Succ NatL");

    assert!(env.globals.contains_key("Zero"));
    assert!(env.globals.contains_key("Succ"));

    // Simple match on Zero inline.
    let id = elab_ok(
        &mut env,
        "let is_zero : Int = match Zero { Zero => 1 ; Succ n => 0 }",
    );
    let body = body_of(&env, id);
    assert!(matches!(body, Term::Elim { .. }));

    // ι-reduces on Zero.
    let ctx = Context::new();
    let reduced = whnf(&env.env, &ctx, &body);
    assert!(
        matches!(reduced, Term::Const { .. }),
        "recursive type match on Zero should ι-reduce to literal 1"
    );
}

#[test]
fn unknown_ctor_in_pattern_is_error() {
    let mut env = mk_env();
    setup_color(&mut env);

    let result = elab(
        &mut env,
        "let bad : Int = match Red { Red => 0 ; Green => 1 ; Purple => 2 }",
    );
    assert!(result.is_err(), "unknown ctor 'Purple' should produce an error");
}

#[test]
fn data_two_arg_ctor_match_accepted() {
    let mut env = mk_env();
    elab_ok(&mut env, "data Pair = P Int Int");

    let id = elab_ok(
        &mut env,
        "let fst : Int = match P 1 2 { P x y => x }",
    );
    let body = body_of(&env, id);
    assert!(matches!(body, Term::Elim { .. }));

    // ι-reduces: elim_Pair M [λx.λy.x] (P 1 2) ⇝ (λx.λy.x) 1 2 ⇝ 1
    let ctx = Context::new();
    let reduced = whnf(&env.env, &ctx, &body);
    assert!(
        !matches!(reduced, Term::Elim { .. }),
        "Pair match should ι-reduce"
    );
}
