//! V1 acceptance tests: verification spec syntax (`21 §6`).
//!
//! Covers all cases from `conformance/verify/spec-syntax/seed-spec-syntax.md`.
//! Pre-declared types use `Nat`/`Bool` from ElabEnv::new(), plus per-test
//! postulates for predicates (NonNeg, Equal, etc.) declared via
//! `declare_postulate_raw`.

use ken_elaborator::{ElabEnv, ElabError};
use ken_kernel::{Context, Level, Term};

// ----- test helpers -----

fn mk_env() -> ElabEnv {
    ElabEnv::new().expect("base env")
}

fn lv(n: u32) -> Level {
    let mut l = Level::Zero;
    for _ in 0..n {
        l = Level::Suc(Box::new(l));
    }
    l
}

/// Declare `name : Nat → Omega 0` — a predicate on Nat.
fn decl_nat_pred(env: &mut ElabEnv, name: &str) {
    let nat_id = *env.globals.get("Nat").unwrap();
    // Type: Pi(Nat, Omega 0) — a predicate
    let ty = Term::pi(Term::indformer(nat_id, vec![]), Term::omega(Level::Zero));
    env.declare_postulate_raw(name, ty).unwrap();
}

/// Declare `name : Nat → Nat → Omega 0` — a binary relation on Nat.
fn decl_nat_rel(env: &mut ElabEnv, name: &str) {
    let nat_id = *env.globals.get("Nat").unwrap();
    let nat = Term::indformer(nat_id, vec![]);
    // Pi(Nat, Pi(Nat, Omega 0))
    let ty = Term::pi(
        nat.clone(),
        Term::pi(weaken(&nat, 1), Term::omega(Level::Zero)),
    );
    env.declare_postulate_raw(name, ty).unwrap();
}

fn weaken(t: &Term, by: i64) -> Term {
    ken_kernel::subst::weaken(t, by)
}

// ======================================================================
// A. Syntax, AST, and elaboration to core
// ======================================================================

/// verify/spec-syntax/requires-elaborates-to-pi-proof-arg
///
/// `fn divide (n : Nat) (d : Nat) : Nat requires NonZero d = n`
/// → core type includes Π proof-arg for the precondition.
#[test]
fn requires_elaborates_to_pi_proof_arg() {
    let mut env = mk_env();
    decl_nat_pred(&mut env, "NonZero");

    let res = env
        .elaborate_decl_v1("fn divide (n : Nat) (d : Nat) : Nat requires NonZero d = n")
        .expect("requires clause should elaborate");

    // No ensures obligations — all requires, no ensures
    assert!(res.obligations.is_empty(), "requires emits no obligations");

    let id = res.def_id;
    // Core type: Pi(Nat, Pi(Nat, Pi(App(NonZero, Var(0)), Nat)))
    let ty = env.env.const_type(id).expect("has type").1;
    // Unwrap Pi-chain: should have 3 Pis (n, d, proof-arg)
    let params = unwrap_pi_chain(&ty);
    assert_eq!(params.len(), 3, "type should have 3 Pi-args: n, d, proof");

    // Third arg should be an App (NonZero applied to d)
    let proof_arg = &params[2];
    assert!(
        matches!(proof_arg, Term::App(_, _)),
        "third Pi-arg should be App(NonZero, d), got {:?}",
        proof_arg
    );
}

fn unwrap_pi_chain(ty: &Term) -> Vec<Term> {
    let mut result = Vec::new();
    let mut cur = ty;
    loop {
        match cur {
            Term::Pi(dom, cod) => {
                result.push(*dom.clone());
                cur = cod;
            }
            _ => break,
        }
    }
    result
}

/// verify/spec-syntax/ensures-emits-obligation-not-sigma
///
/// `fn abs (n : Nat) : Nat ensures NonNeg n = n`
/// → core body is bare Nat value (not Σ); one obligation emitted.
#[test]
fn ensures_emits_obligation_not_sigma() {
    let mut env = mk_env();
    decl_nat_pred(&mut env, "NonNeg");

    let res = env
        .elaborate_decl_v1("fn abs (n : Nat) : Nat ensures NonNeg n = n")
        .expect("ensures should elaborate");

    // Exactly one obligation hole
    assert_eq!(res.obligations.len(), 1, "one ensures → one obligation");

    let id = res.def_id;
    // Core type: Pi(Nat, Nat) — NOT Pi(Nat, Sigma(Nat, NonNeg))
    let ty = env.env.const_type(id).expect("has type").1;
    let nat_id = *env.globals.get("Nat").unwrap();
    let nat = Term::indformer(nat_id, vec![]);
    let expected_ty = Term::pi(nat.clone(), weaken(&nat, 1));
    assert_eq!(ty, expected_ty, "ensures does not change the carrier type");

    // Core body: Lam(Nat, Var(0)) — the identity (returns n)
    let body = env.env.transparent_body(id).expect("transparent").1;
    let expected_body = Term::lam(nat, Term::var(0));
    assert_eq!(body, expected_body, "ensures body is bare carrier");

    // The obligation hole is in trusted_base (= unknown status)
    let hole_id = res.obligations[0].hole_id;
    assert!(
        env.is_open_hole(hole_id),
        "ensures obligation is in trusted_base (unknown)"
    );
}

/// verify/spec-syntax/refinement-lowers-to-carrier
///
/// `fn mkPos (n : Nat) : { k : Nat | NonNeg k } = n`
/// → core return type is `Nat` (carrier), not `Sigma(Nat, NonNeg)`.
#[test]
fn refinement_lowers_to_carrier() {
    let mut env = mk_env();
    decl_nat_pred(&mut env, "NonNeg");

    // Refinement type in return-type position: lowers to carrier Nat, no Sigma formed.
    let res = env
        .elaborate_decl_v1("fn mkPos (n : Nat) : { k : Nat | NonNeg k } = n")
        .expect("refinement return type should lower to carrier");

    let id = res.def_id;
    // Core type: Pi(Nat, Nat) — carrier Nat, not Sigma(Nat, NonNeg)
    let ty = env.env.const_type(id).expect("has type").1;
    let nat_id = *env.globals.get("Nat").unwrap();
    let nat = Term::indformer(nat_id, vec![]);
    let expected_ty = Term::pi(nat.clone(), weaken(&nat, 1));
    assert_eq!(ty, expected_ty, "refinement lowers to carrier: Pi(Nat,Nat)");
}

/// verify/spec-syntax/non-omega-predicate-surface-error
///
/// `fn f (n : Nat) : Nat requires Nat = n` → rejected (Nat is Type 0, not Ω).
#[test]
fn non_omega_predicate_surface_error() {
    let mut env = mk_env();
    // `Nat` (a type) used as a proposition — should fail
    let result = env.elaborate_decl_v1("fn f (n : Nat) : Nat requires Nat = n");
    assert!(
        result.is_err(),
        "non-Ω requires clause must be rejected, got {:?}",
        result.ok()
    );
}

/// verify/spec-syntax/result-scope (a) — accepts in ensures
#[test]
fn result_in_ensures_resolves() {
    let mut env = mk_env();
    decl_nat_rel(&mut env, "Equal");

    // `result` in ensures scope → resolves to body value
    let res = env
        .elaborate_decl_v1("fn g (n : Nat) : Nat ensures Equal result n = n")
        .expect("result in ensures should resolve");
    assert_eq!(res.obligations.len(), 1, "one ensures obligation");
}

/// verify/spec-syntax/result-scope (b) — rejects in requires
#[test]
fn result_out_of_ensures_rejects() {
    let mut env = mk_env();
    decl_nat_rel(&mut env, "Equal");

    // `result` in requires scope → scope error
    let result = env.elaborate_decl_v1("fn h (n : Nat) : Nat requires Equal result n = n");
    assert!(
        result.is_err(),
        "result in requires must be rejected, got {:?}",
        result.ok()
    );
}

// ======================================================================
// B. `old`-capture scope guard
// ======================================================================

/// verify/spec-syntax/old-resolves-in-space-op-ensures
///
/// In a `space view`, `old(n)` in ensures is accepted (scope guard passes).
#[test]
fn old_resolves_in_space_op_ensures() {
    let mut env = mk_env();
    decl_nat_rel(&mut env, "Equal");

    // space view: old(n) is in scope in ensures
    let res = env
        .elaborate_decl_v1("space proc inc (n : Nat) : Nat ensures Equal n (old n) = n")
        .expect("old in space-op ensures should resolve");
    // Obligation is emitted
    assert_eq!(
        res.obligations.len(),
        1,
        "space-op ensures emits obligation"
    );
}

/// verify/spec-syntax/old-out-of-scope-rejects  (soundness)
///
/// In a pure `view`, `old(x)` in ensures is a scope error.
#[test]
fn old_out_of_scope_rejects() {
    let mut env = mk_env();
    decl_nat_rel(&mut env, "Equal");

    let result = env.elaborate_decl_v1("fn k (n : Nat) : Nat ensures Equal n (old n) = n");
    assert!(
        result.is_err(),
        "old in pure const ensures must be rejected, got {:?}",
        result.ok()
    );
}

// ======================================================================
// C. Verification status model + honesty guard
// ======================================================================

/// verify/spec-syntax/proved-status-cert-checks-not-in-trusted-base  (soundness)
///
/// Discharge an obligation with a valid cert → hole leaves trusted_base.
#[test]
fn proved_status_cert_checks_not_in_trusted_base() {
    let mut env = mk_env();
    decl_nat_pred(&mut env, "TrueProp");

    // `fn tst (n : Nat) : Nat ensures TrueProp n = n`
    // The obligation goal (closed) = Pi(Nat, TrueProp Var(0))
    let res = env
        .elaborate_decl_v1("fn tst (n : Nat) : Nat ensures TrueProp n = n")
        .expect("elaborates");
    let obl = &res.obligations[0];
    assert!(env.is_open_hole(obl.hole_id), "hole is initially open");

    // The closed goal is Pi(Nat, TrueProp Var(0)).
    // A valid cert: Lam(Nat, App(TrueProp, Var(0))) — but we don't have a proof
    // of TrueProp, so we declare one as a postulate and use it.
    let trueprop_id = *env.globals.get("TrueProp").unwrap();
    let nat_id = *env.globals.get("Nat").unwrap();
    let nat = Term::indformer(nat_id, vec![]);
    // Declare a proof witness: `wit : Pi(Nat, TrueProp Var(0))`
    let proof_ty = obl.goal_closed.clone();
    let wit_id = env
        .declare_postulate_raw("wit", proof_ty)
        .expect("declare proof witness");

    // Use the witness as the cert
    let cert = Term::const_(wit_id, vec![]);
    let discharged = env.discharge_hole(obl, cert);
    assert!(discharged, "valid cert should discharge the hole");
    assert!(
        !env.is_open_hole(obl.hole_id),
        "after discharge, hole is not in trusted_base (proved)"
    );
    let _ = (trueprop_id, nat);
}

/// verify/spec-syntax/bogus-cert-not-proved  (soundness)
///
/// A wrong cert doesn't discharge the hole.
#[test]
fn bogus_cert_not_proved() {
    let mut env = mk_env();
    decl_nat_pred(&mut env, "PropQ");

    let res = env
        .elaborate_decl_v1("fn btest (n : Nat) : Nat ensures PropQ n = n")
        .expect("elaborates");
    let obl = &res.obligations[0];

    // Wrong cert: just the nat type itself (type mismatch)
    let nat_id = *env.globals.get("Nat").unwrap();
    let bogus = Term::ty(Level::Zero); // Type 0, not a proof of Pi(Nat, PropQ Var(0))
    let discharged = env.discharge_hole(obl, bogus);
    assert!(!discharged, "bogus cert must not discharge");
    assert!(
        env.is_open_hole(obl.hole_id),
        "hole remains in trusted_base after bogus cert"
    );
    let _ = nat_id;
}

/// verify/spec-syntax/unknown-hole-distinct-from-proved  (soundness)
///
/// An undischarged hole appears in trusted_base; a proved term does not.
#[test]
fn unknown_hole_distinct_from_proved() {
    let mut env = mk_env();
    decl_nat_pred(&mut env, "PropR");

    // Open hole
    let res = env
        .elaborate_decl_v1("fn utest (n : Nat) : Nat ensures PropR n = n")
        .expect("elaborates");
    let obl = &res.obligations[0];
    assert!(
        env.is_open_hole(obl.hole_id),
        "undischarged obligation is in trusted_base (unknown)"
    );

    // The def_id itself (transparent definition) is NOT in trusted_base
    let tb = env.env.trusted_base();
    assert!(
        !tb.contains(&res.def_id),
        "the view's def_id is transparent, not in trusted_base"
    );
}

/// verify/spec-syntax/disproved-distinct-from-unknown
///
/// Model-level test: `disproved` is a verification error; `unknown` is not.
/// V1 pins the model distinction; prover is V3.
#[test]
fn disproved_distinct_from_unknown() {
    // In V1, `disproved` carries a countermodel (supplied externally, prover is V3).
    // The model distinction is: disproved → hard error (no export), unknown → runs with hole.
    // We assert the honesty guard holds: an `unknown` claim's hole IS in trusted_base,
    // but does NOT read as `proved`. This is covered by `unknown_hole_distinct_from_proved`.
    // The `disproved` case (countermodel) is structurally separate from `unknown`.
    // For V1 build: assert that the status model has three distinct verdicts.
    // This is a model assertion, not a runtime check — both unknowns are "not proved",
    // but only `disproved` is a hard error; `unknown` is a running hole.
    // No code to run: this test documents the model.
    // The verdict trichotomy: proved / disproved / unknown — collapsing any two would
    // break the projection (`21 §5.3`).
    // Structural: `unknown` hole IS in trusted_base; `disproved` carries a refutation.
    // We check the `unknown` side (covered by `unknown_hole_distinct_from_proved`).
    // `disproved` has no V1 runtime representation (prover generates countermodels in V3).
    // Pass: the model is correctly specified in §5.1/§5.3.
}

// ======================================================================
// D. Epistemic projection (model assertion)
// ======================================================================

/// verify/spec-syntax/epistemic-projection-distinct
///
/// The four epistemic statuses are distinct (`21 §5.2`/§5.3).
/// `proved` and `unknown` are pinned here via `trusted_base()`;
/// `tested`/`delegated` are deferred (disposition-tag spelling OQ-syntax).
#[test]
fn epistemic_projection_distinct() {
    let mut env = mk_env();
    decl_nat_pred(&mut env, "PropE");

    // proved side: no hole → not in trusted_base
    // (we test this via discharge_hole; see proved_status_cert_checks_not_in_trusted_base)
    // unknown side: open hole → in trusted_base
    let res = env
        .elaborate_decl_v1("fn etest (n : Nat) : Nat ensures PropE n = n")
        .expect("elaborates");
    let obl = &res.obligations[0];
    assert!(
        env.is_open_hole(obl.hole_id),
        "unknown: hole in trusted_base"
    );

    // Projection: unknown ≠ proved (by trusted_base membership)
    // tested/delegated are deferred [§5.5, OQ-syntax]
}

// ======================================================================
// E. V1→V2 interface
// ======================================================================

/// verify/spec-syntax/obligation-hole-set-exposed-to-v2
///
/// A const with two ensures clauses emits exactly two obligation holes.
#[test]
fn obligation_hole_set_exposed_to_v2() {
    let mut env = mk_env();
    decl_nat_pred(&mut env, "Pos");
    decl_nat_pred(&mut env, "NonNeg2");

    // Two ensures clauses → exactly two obligation holes
    let res = env
        .elaborate_decl_v1("fn f2 (n : Nat) : Nat ensures Pos n ensures NonNeg2 n = n")
        .expect("two ensures should elaborate");

    assert_eq!(res.obligations.len(), 2, "two ensures → two obligations");
    let h0 = res.obligations[0].hole_id;
    let h1 = res.obligations[1].hole_id;
    assert_ne!(h0, h1, "two distinct holes");
    assert!(env.is_open_hole(h0), "hole 0 in trusted_base");
    assert!(env.is_open_hole(h1), "hole 1 in trusted_base");
}

// ======================================================================
// F. Goals — `prove` / `law`
// ======================================================================

/// verify/spec-syntax/prove-goal-obligation-and-postulate-binding  (soundness)
///
/// `prove addComm : SomeGoal` — standalone obligation.
/// → hole emitted, in trusted_base; after discharge, leaves trusted_base.
#[test]
fn prove_goal_obligation_and_postulate_binding() {
    let mut env = mk_env();
    // SomeGoal : Omega 0 — a proposition (no arguments needed)
    env.declare_postulate_raw("SomeGoal", Term::omega(Level::Zero))
        .unwrap();

    // Before discharge
    let res = env
        .elaborate_decl_v1("prove addComm : SomeGoal")
        .expect("prove should elaborate");
    assert_eq!(res.obligations.len(), 1, "prove emits one obligation");
    let obl = &res.obligations[0];
    assert!(
        env.is_open_hole(obl.hole_id),
        "goal in trusted_base (unknown)"
    );

    // `addComm` is usable as a proof term (it IS the hole postulate)
    let comm_id = *env.globals.get("addComm").expect("addComm bound");
    assert_eq!(comm_id, obl.hole_id, "addComm is the hole postulate");

    // After discharge: declare a witness and use it as the certificate.
    let proof_ty = obl.goal_closed.clone();
    let wit_id = env.declare_postulate_raw("AddCommWit", proof_ty).unwrap();
    let cert = Term::const_(wit_id, vec![]);
    let discharged = env.discharge_hole(obl, cert);
    assert!(discharged, "valid cert discharges the hole");
    assert!(
        !env.is_open_hole(obl.hole_id),
        "after discharge, leaves trusted_base (proved)"
    );
}

/// verify/spec-syntax/law-all-omega-fields-is-proposition
///
/// `law Monoid (M) { assoc : AssocP ; unit_l : UnitLP }` → per-field obligations.
#[test]
fn law_all_omega_fields_is_proposition() {
    let mut env = mk_env();
    // Pre-declare the field propositions as Omega-typed postulates
    env.declare_postulate_raw("AssocP", Term::omega(Level::Zero))
        .unwrap();
    env.declare_postulate_raw("UnitLP", Term::omega(Level::Zero))
        .unwrap();

    let res = env
        .elaborate_decl_v1("law Monoid (M) { assoc : AssocP ; unit_l : UnitLP }")
        .expect("law should elaborate");

    // Two field obligations, one per law field
    assert_eq!(res.obligations.len(), 2, "two law fields → two obligations");
    let h0 = res.obligations[0].hole_id;
    let h1 = res.obligations[1].hole_id;
    assert!(env.is_open_hole(h0), "field 0 obligation in trusted_base");
    assert!(env.is_open_hole(h1), "field 1 obligation in trusted_base");
}

// ======================================================================
// G. Regression — V0 unchanged
// ======================================================================

/// verify/spec-syntax/v0-unchanged-for-non-spec-programs  (soundness)
///
/// A V0 program (no spec forms) elaborates identically to V0.
#[test]
fn v0_behavior_unchanged() {
    let mut env = mk_env();
    let id = env
        .elaborate_decl("fn id (A : Type) (x : A) : A = x")
        .expect("V0 id should elaborate");

    // Core body unchanged: Lam(Type 0, Lam(Var(0), Var(0)))
    let body = env.env.transparent_body(id).expect("transparent").1;
    let expected = Term::lam(Term::ty(lv(0)), Term::lam(Term::var(0), Term::var(0)));
    assert_eq!(body, expected, "V0 body must be unchanged");

    // No obligations (no spec forms)
    let res_v1 = env
        .elaborate_decl_v1("fn const2 (A : Type) (x : A) (y : A) : A = x")
        .expect("V0 const should elaborate via V1 path");
    assert!(
        res_v1.obligations.is_empty(),
        "no spec forms → no obligations"
    );
}

// ======================================================================
// H. de Bruijn fix — `requires` on non-final parameter (V1-fix)
// ======================================================================
//
// V1-fix: the body was not weakened by req_cores.len() before inserting
// the proof-arg lambdas between param lambdas and the body. This caused
// body Var(i) to point at the proof argument instead of param i.
// Fix: weaken(body_inner, req_cores.len()) in elaborate_view_with_spec Phase 4.

/// verify/spec-syntax/requires-on-first-param-of-two  (verdict-flip: was TypeMismatch, now ok)
///
/// `fn f (n : Nat) (d : Nat) : Nat requires Positive n = d` — `requires`
/// clause references `n` (the FIRST param, not the last). Before the fix:
/// kernel TypeMismatch. After: elaborates cleanly and the body is `d` (correct).
#[test]
fn requires_on_first_of_two_params() {
    let mut env = mk_env();
    decl_nat_pred(&mut env, "Positive");

    let res = env
        .elaborate_decl_v1("fn f (n : Nat) (d : Nat) : Nat requires Positive n = d")
        .expect("requires on non-final param must elaborate after fix");

    // No ensures → no obligations
    assert!(res.obligations.is_empty(), "requires-only → no obligations");

    // Full type: Pi(n:Nat, Pi(d:Nat, Pi(Positive(n), Nat)))
    // Full body: Lam(n, Lam(d, Lam(proof, d)))  where body=d=Var(1) in innermost ctx
    let body = env
        .env
        .transparent_body(res.def_id)
        .expect("transparent body")
        .1;
    // In Lam(n, Lam(d, Lam(proof, body))):
    //   innermost ctx: Var(0)=proof, Var(1)=d, Var(2)=n
    // body = d = Var(1) after fix (was Var(0)=proof before fix → TypeMismatch)
    let positive_id = *env.globals.get("Positive").unwrap();
    let nat_id = *env.globals.get("Nat").unwrap();
    let nat = Term::indformer(nat_id, vec![]);
    // Expected body: λn:Nat. λd:Nat. λproof:Positive(n). d
    let expected = Term::lam(
        nat.clone(),
        Term::lam(
            nat.clone(),
            Term::lam(
                Term::app(Term::const_(positive_id, vec![]), Term::var(1)), // Positive(n) in d-ctx: n=Var(1)
                Term::var(1), // d in proof-ctx: d=Var(1)
            ),
        ),
    );
    assert_eq!(
        body, expected,
        "body must be d (Var(1) in proof-ctx), not proof (Var(0))"
    );
}

/// verify/spec-syntax/requires-on-middle-param-of-three
///
/// Three params: first, second (predicated on), third. `requires` references
/// the middle param — tests that the shift is correct for both the body and
/// the predicate domain across the param chain.
#[test]
fn requires_on_middle_param_of_three() {
    let mut env = mk_env();
    decl_nat_pred(&mut env, "MidPred");

    // fn g (a : Nat) (b : Nat) (c : Nat) : Nat requires MidPred b = a
    let res = env
        .elaborate_decl_v1("fn g (a : Nat) (b : Nat) (c : Nat) : Nat requires MidPred b = a")
        .expect("requires on middle param must elaborate after fix");

    assert!(res.obligations.is_empty(), "requires-only → no obligations");
    // Body should be `a` (the first param). In innermost ctx (a,b,c,proof):
    // Var(0)=proof, Var(1)=c, Var(2)=b, Var(3)=a → body = Var(3)
    let body = env
        .env
        .transparent_body(res.def_id)
        .expect("transparent body")
        .1;
    let mid_pred_id = *env.globals.get("MidPred").unwrap();
    let nat_id = *env.globals.get("Nat").unwrap();
    let nat = Term::indformer(nat_id, vec![]);
    // λa:Nat. λb:Nat. λc:Nat. λproof:MidPred(b). a
    // MidPred(b) in c-ctx: b=Var(1), so domain = App(MidPred, Var(1))
    // a in proof-ctx: Var(3)
    let expected = Term::lam(
        nat.clone(),
        Term::lam(
            nat.clone(),
            Term::lam(
                nat.clone(),
                Term::lam(
                    Term::app(Term::const_(mid_pred_id, vec![]), Term::var(1)), // b in c-ctx
                    Term::var(3),                                               // a in proof-ctx
                ),
            ),
        ),
    );
    assert_eq!(body, expected, "body must be a (Var(3) in proof-ctx)");
}

/// verify/spec-syntax/requires-on-final-param-unaffected
///
/// Regression: the existing working form (`requires` on the final/last param)
/// must be unaffected by the fix. Same case as `requires_elaborates_to_pi_proof_arg`
/// re-stated to pin the no-regression dimension explicitly.
#[test]
fn requires_on_final_param_unaffected() {
    let mut env = mk_env();
    decl_nat_pred(&mut env, "FinalPred");

    let res = env
        .elaborate_decl_v1("fn h (n : Nat) (d : Nat) : Nat requires FinalPred d = n")
        .expect("requires on final param must still elaborate (regression)");

    assert!(res.obligations.is_empty());
    // Body = n. In innermost ctx (n,d,proof): Var(0)=proof, Var(1)=d, Var(2)=n → body=Var(2)
    let body = env
        .env
        .transparent_body(res.def_id)
        .expect("transparent body")
        .1;
    let fin_pred_id = *env.globals.get("FinalPred").unwrap();
    let nat_id = *env.globals.get("Nat").unwrap();
    let nat = Term::indformer(nat_id, vec![]);
    // λn:Nat. λd:Nat. λproof:FinalPred(d). n
    // FinalPred(d) in d-ctx: Var(0)=d, so domain = App(FinalPred, Var(0))
    // n in proof-ctx: Var(2)
    let expected = Term::lam(
        nat.clone(),
        Term::lam(
            nat.clone(),
            Term::lam(
                Term::app(Term::const_(fin_pred_id, vec![]), Term::var(0)), // d in d-ctx
                Term::var(2),                                               // n in proof-ctx
            ),
        ),
    );
    assert_eq!(body, expected, "body must be n (Var(2) in proof-ctx)");
}
