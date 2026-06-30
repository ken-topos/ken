//! L-resolver-globals acceptance tests.
//!
//! Pins: `docs/program/wp/L-resolver-globals.md` AC1–AC3.
//!
//! The fix: `EVar` in expression position falls through to `RCon` on scope
//! miss — same fallback `TVar` uses in type position. No new resolver state;
//! the elaborator's existing `RCon` handler resolves the name against globals.

use ken_elaborator::{error::ElabError, ElabEnv};
use ken_kernel::{whnf, Context, GlobalId, Term};

fn mk_env() -> ElabEnv {
    ElabEnv::new().expect("base env construction failed")
}

fn elab_ok(env: &mut ElabEnv, src: &str) -> GlobalId {
    env.elaborate_decl(src)
        .unwrap_or_else(|e| panic!("elab_ok failed: {e}"))
}

// ─────────────────────────────────────────────────────────────────────────────
// AC1 — cross-declaration lowercase global resolves
//
// Before the fix: `EVar("forty_two", ...)` misses the local scope → UnboundName.
// After the fix: falls through to RCon → elaborator finds it in globals.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ac1_cross_decl_lowercase_global_resolves() {
    let mut env = mk_env();
    // First declaration: registers "forty_two" in globals.
    elab_ok(&mut env, "let forty_two : Int = 42");
    // Second declaration: body references "forty_two" — was UnboundName pre-fix.
    let result = env.elaborate_decl("view use_global : Int = forty_two");
    assert!(
        result.is_ok(),
        "cross-decl lowercase global should resolve; got: {:?}",
        result.err()
    );
}

#[test]
fn ac1_absent_name_still_errors() {
    // Discriminating pair: a name that is neither local nor global still errors.
    // Post-fix the error is UnresolvedCon (elaboration stage) not UnboundName
    // (resolution stage) — both prove the name was rejected.
    let mut env = mk_env();
    let result = env.elaborate_decl("view uses_absent : Int = no_such_global");
    assert!(
        matches!(
            result,
            Err(ElabError::UnboundName { .. } | ElabError::UnresolvedCon { .. })
        ),
        "absent name should still error (UnboundName or UnresolvedCon); got: {:?}",
        result
    );
}

#[test]
fn ac1_two_decl_chain() {
    // Verify a chain of two cross-decl references.
    let mut env = mk_env();
    elab_ok(&mut env, "let base_val : Int = 10");
    let mid_id = elab_ok(&mut env, "view mid_val : Int = base_val");
    let top_id = elab_ok(&mut env, "view top_val : Int = mid_val");
    // Both second and third declarations must have resolved successfully.
    assert_ne!(mid_id, top_id);
}

// ─────────────────────────────────────────────────────────────────────────────
// AC2 — locals still shadow globals (regression guard)
//
// A locally-bound parameter with the same name as a global resolves to de-Bruijn
// 0 (the param), not the global.  Assert the resolved TARGET, not just "compiles."
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ac2_local_param_shadows_global() {
    let mut env = mk_env();

    // Declare a global named "pp" of type Int.
    let int_id = *env.globals.get("Int").expect("Int pre-declared");
    let pp_id = env
        .declare_postulate_raw("pp", Term::const_(int_id, vec![]))
        .expect("declare pp");

    // View with a PARAMETER also named "pp". Body `pp` must resolve to the param.
    let shadow_id =
        elab_ok(&mut env, "view shadow_view (pp : Int) : Int = pp");

    // Extract the transparent body: should be Lam(Int, Var(0)).
    let (_, body) = env
        .env
        .transparent_body(shadow_id)
        .expect("shadow_view is transparent");

    // Apply body to a distinguishing term (Int type itself) and β-reduce.
    // · If body = Lam(Int, Var(0))        → result is Int_const  (local shadow ✓)
    // · If body = Lam(Int, Const(pp_id))  → result is pp_const   (global leak ✗)
    let int_const = Term::const_(int_id, vec![]);
    let applied = whnf(
        &env.env,
        &Context::new(),
        &Term::app(body, int_const),
    );
    assert!(
        matches!(&applied, Term::Const { id, .. } if *id == int_id),
        "body should return its argument (local Var(0)), not the global pp; \
         got: {:?} (pp_id={:?}, int_id={:?})",
        applied,
        pp_id,
        int_id
    );
}

#[test]
fn ac2_local_let_binding_shadows_global() {
    let mut env = mk_env();

    // Global named "inner".
    let int_id = *env.globals.get("Int").expect("Int pre-declared");
    let inner_g_id = env
        .declare_postulate_raw("inner", Term::const_(int_id, vec![]))
        .expect("declare inner global");

    // View whose body has a local `let inner = 7` — that local should shadow.
    // The body resolves "inner" inside the let-binding to de-Bruijn 0, not the global.
    let shadow_id = elab_ok(
        &mut env,
        "view let_shadow : Int = let inner : Int = 7 in inner",
    );

    // The body should evaluate to the literal 7, not the postulate inner_g_id.
    let (_, body) = env
        .env
        .transparent_body(shadow_id)
        .expect("let_shadow is transparent");
    let reduced = whnf(&env.env, &Context::new(), &body);
    // The literal 7 is opaque but is NOT the global postulate inner_g_id.
    assert!(
        !matches!(&reduced, Term::Const { id, .. } if *id == inner_g_id),
        "let-bound local should shadow global; \
         got the global postulate instead: {:?}",
        reduced
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC3 — full suite no-regression (all existing tests remain green)
//
// This file runs alongside the existing acceptance suites; Cargo collects all
// test binaries. No explicit re-run here — "cargo test -p ken-elaborator" covers
// the full regression signal.
//
// One positive smoke-test: a ConId (uppercase global) still works as before.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ac3_uppercase_con_still_resolves() {
    let mut env = mk_env();
    elab_ok(&mut env, "data MaybeI = NoI | SomeI Int");
    let result = env.elaborate_decl("view get_no_i : MaybeI = NoI");
    assert!(
        result.is_ok(),
        "uppercase constructor reference should still resolve; got: {:?}",
        result.err()
    );
}

#[test]
fn ac3_type_position_global_unchanged() {
    // Type-position global resolution (TVar fallback) must be unaffected.
    let mut env = mk_env();
    elab_ok(&mut env, "data MyInt = MkMyInt Int");
    let result = env.elaborate_decl("let v : MyInt = MkMyInt 3");
    assert!(
        result.is_ok(),
        "type-position lowercase global (MyInt) should still resolve; got: {:?}",
        result.err()
    );
}
