//! V2 obligation-generation conformance tests.
//!
//! Pins `22-obligations.md` — the obligation triple (§1), extraction algorithm
//! (§5), absent-clause scan (§2.5), and V2→V3 interface (§6). Cases are
//! grounded in the landed spec/conformance corpus, NOT the prototype.
//!
//! Placeholder discipline: tests whose full implementation requires K2 `Eq`,
//! eliminator walking, or V3+ infrastructure are tagged
//! `// [placeholder — reifies in <WP>]` and assert the structural property
//! that is checkable today.

use ken_elaborator::{v2_extract, ElabEnv, ObligationKind, ProvKind};
use ken_kernel::{Level, Term};

fn mk_env() -> ElabEnv {
    ElabEnv::new().expect("base env")
}

/// Declare `name : Pi(Nat, Omega 0)` — a unary Nat predicate.
fn decl_nat_pred(env: &mut ElabEnv, name: &str) {
    let nat_id = *env.globals.get("Nat").expect("Nat");
    let nat_ty = Term::indformer(nat_id, vec![]);
    let pred_ty = Term::pi(nat_ty, Term::omega(Level::Zero));
    env.declare_postulate_raw(name, pred_ty).expect("decl_nat_pred");
}

// ======================================================================
// F.3 — Regression: non-spec program yields empty obligation set
// ======================================================================

/// verify/obligations/non-spec-program-empty-obligation-set  (soundness)
///
/// A program with no spec clauses and no partial primitives yields the empty
/// obligation set; V1/V0 elaboration is unchanged (`22 §8`).
#[test]
fn non_spec_program_empty_obligation_set() {
    let mut env = mk_env();
    let elab_res = env
        .elaborate_decl_v1("view id_nat (n : Nat) : Nat = n")
        .expect("plain view must elaborate");
    let ex = v2_extract(&elab_res);
    assert!(
        ex.obligations.is_empty(),
        "no-spec program → empty obligation set"
    );
}

// ======================================================================
// A.2 — Postcondition emits substituted goal
// ======================================================================

/// verify/obligations/postcondition-emits-substituted-goal
///
/// A straight-line body `view f (n : Nat) : Nat ensures SomeProp result = n`
/// emits one obligation `⟨id, (n:Nat) ⊢ SomeProp n, prov⟩` — `result`
/// replaced by the body `n` (`22 §2.2`).
#[test]
fn postcondition_emits_substituted_goal() {
    let mut env = mk_env();
    decl_nat_pred(&mut env, "SomeProp");

    let elab_res = env
        .elaborate_decl_v1("view f (n : Nat) : Nat ensures SomeProp result = n")
        .expect("view with ensures should elaborate");

    let ex = v2_extract(&elab_res);
    assert_eq!(ex.obligations.len(), 1, "one ensures → one obligation");

    let triple = &ex.obligations[0];
    // context = [Nat] (the parameter telescope)
    assert_eq!(triple.context.len(), 1, "context has one entry: n : Nat");

    // phi = SomeProp(Var(0)) — `result` substituted by body `n` (= Var(0) in ctx)
    let someprop_id = *env.globals.get("SomeProp").unwrap();
    let expected_phi = Term::app(Term::const_(someprop_id, vec![]), Term::var(0));
    assert_eq!(
        triple.phi, expected_phi,
        "phi = SomeProp(n): result substituted by body, not left free"
    );

    // goal_closed = Pi(context, phi)
    let expected_closed = Term::pi(triple.context[0].clone(), triple.phi.clone());
    assert_eq!(triple.goal_closed, expected_closed, "goal_closed = Pi(Nat, phi)");

    // provenance
    assert!(
        matches!(triple.provenance.kind, ProvKind::Ensures { index: 0 }),
        "provenance kind = Ensures(0)"
    );
    // stable id
    assert_eq!(triple.id.0, "f.ensures.0", "stable id keyed on name + kind + index");
}

// ======================================================================
// A.1 — Refinement-introduction emits predicate (return-type case)
// ======================================================================

/// verify/obligations/refinement-introduction-emits-phi  (partial — return-type case)
///
/// `view mkPos (n : Nat) : { k : Nat | NonNeg k } = n` emits one obligation
/// `⟨id, (n:Nat) ⊢ NonNeg n, prov⟩` — the return-type refinement predicate
/// with the body substituted (`22 §2.1`).
///
/// General case (value at arbitrary refinement-typed site) is
/// `[placeholder — reifies in V3]` pending carrier-tracking through elaboration.
#[test]
fn refinement_return_type_emits_phi() {
    let mut env = mk_env();
    decl_nat_pred(&mut env, "NonNeg");

    let elab_res = env
        .elaborate_decl_v1("view mkPos (n : Nat) : { k : Nat | NonNeg k } = n")
        .expect("refinement return type should elaborate");

    let ex = v2_extract(&elab_res);
    assert_eq!(
        ex.obligations.len(),
        1,
        "refinement return type → one obligation (§2.1)"
    );

    let triple = &ex.obligations[0];
    assert_eq!(triple.context.len(), 1, "context: n : Nat");

    let nonneg_id = *env.globals.get("NonNeg").unwrap();
    let expected_phi = Term::app(Term::const_(nonneg_id, vec![]), Term::var(0));
    assert_eq!(
        triple.phi, expected_phi,
        "phi = NonNeg n: body substituted for refinement variable k/result"
    );
}

// ======================================================================
// A.3 (placeholder) — Precondition obligation at call site
// ======================================================================

/// verify/obligations/precondition-obligation-at-call-not-in-body  (soundness)
/// `[placeholder — reifies in V3]`
///
/// Call-site precondition obligations require tracking proof-arg Π positions
/// and walking App nodes, which needs K2 Eq for the hypothesis context.
/// The absence-of-body-obligation is already verified by
/// `body_requires_assumed_not_reobligated`.
#[test]
fn precondition_at_call_not_in_body_placeholder() {
    // [placeholder — reifies in V3]: call-site App walking + Eq in Γ
    // What IS verifiable now: the callee emits no obligation for its own
    // precondition (it's an assumption via Π proof-arg, not a goal).
    let mut env = mk_env();
    decl_nat_pred(&mut env, "Nonzero");

    // Predicate on first param `n` — exercises the fixed de Bruijn shift path.
    let elab_res = env
        .elaborate_decl_v1(
            "view safe_f (n : Nat) (d : Nat) : Nat requires Nonzero n = d",
        )
        .expect("view with requires should elaborate");

    let ex = v2_extract(&elab_res);
    assert!(
        ex.obligations.is_empty(),
        "callee emits no obligation for its own precondition (it is an assumption)"
    );
    // [placeholder — reifies in V3]: emit ⟨id, Γ_call ⊢ Nonzero arg, prov⟩ at each call site
}

// ======================================================================
// A.4 (placeholder) — Partial primitive emits non-zero obligation
// ======================================================================

/// verify/obligations/partial-primitive-emits-nonzero-obligation
/// `[placeholder — reifies in V4]`
///
/// Partial primitive side-condition obligations (`35 §3`) require recognizing
/// specific primitive GlobalIds and forming `Eq`-typed side conditions.
#[test]
fn partial_primitive_nonzero_placeholder() {
    // [placeholder — reifies in V4]: PrimReduction::Op recognition + Eq side conditions
    // Structural: verify that existing operations do NOT emit spurious obligations.
    let mut env = mk_env();
    let elab_res = env
        .elaborate_decl_v1("view f (n : Nat) : Nat = n")
        .expect("plain view elaborates");
    let ex = v2_extract(&elab_res);
    assert!(
        ex.obligations.is_empty(),
        "no partial primitive → no spurious obligation"
    );
}

// ======================================================================
// A.5 — Prove and Law emit one obligation per goal
// ======================================================================

/// verify/obligations/prove-and-law-emit-one-obligation-per-goal
///
/// `prove` emits one obligation; `law` with N fields emits N obligations,
/// each keyed by field name (`22 §2.4`).
#[test]
fn prove_and_law_emit_one_obligation_per_goal() {
    let mut env = mk_env();
    env.declare_postulate_raw("SomeGoal", Term::omega(Level::Zero))
        .unwrap();

    // prove case: one obligation
    let elab_prove = env
        .elaborate_decl_v1("prove addComm : SomeGoal")
        .expect("prove should elaborate");
    let ex_prove = v2_extract(&elab_prove);
    assert_eq!(ex_prove.obligations.len(), 1, "prove → one obligation");
    assert_eq!(ex_prove.obligations[0].id.0, "addComm.prove", "stable id");
    assert!(
        matches!(ex_prove.obligations[0].provenance.kind, ProvKind::Prove),
        "provenance = Prove"
    );

    // law case: three fields → three obligations
    env.declare_postulate_raw("AssocProp", Term::omega(Level::Zero))
        .unwrap();
    env.declare_postulate_raw("UnitLProp", Term::omega(Level::Zero))
        .unwrap();
    env.declare_postulate_raw("UnitRProp", Term::omega(Level::Zero))
        .unwrap();

    let elab_law = env
        .elaborate_decl_v1(
            "law Monoid (m) { assoc : AssocProp ; unitL : UnitLProp ; unitR : UnitRProp }",
        )
        .expect("law should elaborate");
    let ex_law = v2_extract(&elab_law);
    assert_eq!(ex_law.obligations.len(), 3, "law with 3 fields → 3 obligations");

    let ids: Vec<&str> = ex_law
        .obligations
        .iter()
        .map(|o| o.id.0.as_str())
        .collect();
    assert!(ids.contains(&"Monoid.law.assoc"), "stable id for assoc field");
    assert!(ids.contains(&"Monoid.law.unitL"), "stable id for unitL field");
    assert!(ids.contains(&"Monoid.law.unitR"), "stable id for unitR field");

    for obl in &ex_law.obligations {
        assert!(
            matches!(obl.provenance.kind, ProvKind::LawField { .. }),
            "all law obligations have LawField provenance"
        );
    }
}

// ======================================================================
// B.1 — Refined parameter is a Γ-hypothesis, not a definition-site obligation
// ======================================================================

/// verify/obligations/refined-param-is-hypothesis-not-obligation  (soundness)
///
/// A refined parameter `(n : { k : Nat | IsNonNeg k })` emits **no**
/// definition-site obligation — it lowers to the carrier and contributes
/// a hypothesis to Γ for the caller's burden (`22 §2.5.1`, §3).
#[test]
fn refined_param_is_hypothesis_not_obligation() {
    let mut env = mk_env();
    decl_nat_pred(&mut env, "IsNonNeg");

    // The refined parameter: lowers to carrier at the def site, no obligation.
    let elab_res = env
        .elaborate_decl_v1("view f (n : { k : Nat | IsNonNeg k }) : Nat = n")
        .expect("refined param view should elaborate");

    let ex = v2_extract(&elab_res);
    assert!(
        ex.obligations.is_empty(),
        "refined parameter is Γ-hypothesis at the def site, not an obligation"
    );
    // Γ-hypothesis contribution ((_ : IsNonNeg n) ∈ Γ) is
    // [placeholder — reifies in V3] pending path-sensitive Γ accumulation.
}

// ======================================================================
// B.2 — Body requires: assumed in Π, not re-obligated
// ======================================================================

/// verify/obligations/body-requires-assumed-not-reobligated
///
/// `requires φ` inside the body is an **assumption** (a Π proof-arg in the
/// type), not re-emitted as the function's own obligation (`22 §2.5.2`, §3).
#[test]
fn body_requires_assumed_not_reobligated() {
    let mut env = mk_env();
    decl_nat_pred(&mut env, "NonzeroP");

    // Predicate on first param `n` — exercises the fixed de Bruijn shift path.
    let elab_res = env
        .elaborate_decl_v1(
            "view safe_f (n : Nat) (d : Nat) : Nat requires NonzeroP n = d",
        )
        .expect("view with requires should elaborate");

    let ex = v2_extract(&elab_res);
    assert!(
        ex.obligations.is_empty(),
        "requires clause is a Π proof-arg assumption, not a function-body obligation"
    );
    // [placeholder — reifies in V3]: the caller's obligation ⟨id, Γ_call ⊢ NonzeroP arg, prov⟩
}

// ======================================================================
// B.3 — Present cert yields zero new open obligations
// ======================================================================

/// verify/obligations/present-cert-yields-zero-new-obligations  (soundness)
///
/// A `prove` whose certificate is supplied and checks removes the hole from
/// `trusted_base()` — the open obligation count drops to zero (`22 §2.5.3`,
/// `21 §5.4`). Undischarged prove stays as one open obligation.
#[test]
fn present_cert_yields_zero_new_obligations() {
    let mut env = mk_env();
    env.declare_postulate_raw("SomeGoal2", Term::omega(Level::Zero))
        .unwrap();

    // Undischarged: one open obligation
    let elab_res = env
        .elaborate_decl_v1("prove openProof : SomeGoal2")
        .expect("prove elaborates");
    let ex_open = v2_extract(&elab_res);
    assert_eq!(ex_open.obligations.len(), 1, "undischarged → 1 obligation");
    assert!(
        env.is_open_hole(ex_open.obligations[0].hole_id),
        "hole in trusted_base (unknown)"
    );

    // Discharge with a witness
    let proof_ty = elab_res.obligations[0].goal_closed.clone();
    let wit = env.declare_postulate_raw("Wit2", proof_ty).unwrap();
    let cert = Term::const_(wit, vec![]);
    let discharged = env.discharge_hole(&elab_res.obligations[0], cert);
    assert!(discharged, "valid cert discharges the hole");

    // After discharge: hole leaves trusted_base → zero open obligations
    assert!(
        !env.is_open_hole(ex_open.obligations[0].hole_id),
        "hole removed from trusted_base"
    );
    let open_count = ex_open
        .obligations
        .iter()
        .filter(|o| env.is_open_hole(o.hole_id))
        .count();
    assert_eq!(open_count, 0, "zero open obligations after discharge");
}

// ======================================================================
// B.4 — Forgetful coercion emits nothing
// ======================================================================

/// verify/obligations/forgetful-coercion-emits-nothing
///
/// Using a value at the carrier type (forgetting a refinement) emits no
/// obligation — the forgetful direction `{x:A|φ} ≤ A` is free (`22 §2.5.4`).
/// Structural: V1's `elab_type(TRefine) = carrier` makes this automatic.
#[test]
fn forgetful_coercion_emits_nothing() {
    let mut env = mk_env();
    // A spec-free view whose body is used at a carrier type emits no obligation.
    let elab_res = env
        .elaborate_decl_v1("view id_nat2 (n : Nat) : Nat = n")
        .expect("elaborates");
    let ex = v2_extract(&elab_res);
    assert!(
        ex.obligations.is_empty(),
        "forgetful direction (carrier-only) emits no obligation"
    );
    // [placeholder — reifies in V3+]: forgetful coercion tracking at expression sites
}

// ======================================================================
// B.5 — Trivial clause still emits obligation (the counter-rule)
// ======================================================================

/// verify/obligations/trivial-clause-still-emits-obligation  (soundness)
///
/// Both a trivially-true and a real-burden `ensures` clause emit an
/// obligation. Emission is keyed on clause **presence**, never on a
/// triviality heuristic (`22 §2.5` counter-rule).
#[test]
fn trivial_clause_still_emits_obligation() {
    let mut env = mk_env();
    decl_nat_pred(&mut env, "TrivProp");
    decl_nat_pred(&mut env, "RealProp");

    // Trivial clause: still emits an obligation (provable by refl / trivial proof)
    let elab1 = env
        .elaborate_decl_v1("view f_triv (n : Nat) : Nat ensures TrivProp result = n")
        .expect("elaborates");
    let ex1 = v2_extract(&elab1);
    assert_eq!(ex1.obligations.len(), 1, "trivial clause emits 1 obligation");

    // Real-burden clause: emits an obligation
    let elab2 = env
        .elaborate_decl_v1("view f_real (n : Nat) : Nat ensures RealProp result = n")
        .expect("elaborates");
    let ex2 = v2_extract(&elab2);
    assert_eq!(ex2.obligations.len(), 1, "real-burden clause emits 1 obligation");

    // Neither yields no obligation — the counter-rule holds
    assert!(!ex1.obligations.is_empty(), "trivial clause does NOT yield empty set");
    assert!(!ex2.obligations.is_empty(), "real burden does NOT yield empty set");
}

// ======================================================================
// B.6 — Exhaustive traversal: no catch-all silent skip
// ======================================================================

/// verify/obligations/exhaustive-traversal-no-silent-skip  (soundness)
///
/// The `match` in `lift_obligation` has NO catch-all `_ ⇒ skip` arm —
/// every `ObligationKind` variant is handled explicitly. Adding a new
/// variant without an arm is a **compile error**, satisfying §2.5's
/// exhaustiveness property (`22 §2.5`).
///
/// This test verifies all currently-known variants produce non-empty
/// provenance (structural: each arm is reachable and returns a `ProvKind`).
#[test]
fn exhaustive_traversal_no_silent_skip() {
    let mut env = mk_env();
    decl_nat_pred(&mut env, "EProp");
    env.declare_postulate_raw("EGoal", Term::omega(Level::Zero))
        .unwrap();

    // Ensures → ProvKind::Ensures (arm 1)
    let r1 = env
        .elaborate_decl_v1("view ef (n : Nat) : Nat ensures EProp result = n")
        .unwrap();
    assert!(v2_extract(&r1)
        .obligations
        .iter()
        .all(|o| matches!(o.provenance.kind, ProvKind::Ensures { .. })));

    // Prove → ProvKind::Prove (arm 2)
    let r2 = env.elaborate_decl_v1("prove ep : EGoal").unwrap();
    assert!(v2_extract(&r2)
        .obligations
        .iter()
        .all(|o| matches!(o.provenance.kind, ProvKind::Prove)));

    // LawField → ProvKind::LawField (arm 3)
    let r3 = env
        .elaborate_decl_v1("law ELaw (m) { ef1 : EGoal }")
        .unwrap();
    assert!(v2_extract(&r3)
        .obligations
        .iter()
        .all(|o| matches!(o.provenance.kind, ProvKind::LawField { .. })));

    // [placeholder — reifies in V3/V4]: CallPrecond and PartialPrim arms
}

// ======================================================================
// C.1 (placeholder) — Match branch Γ carries scrutinee equation
// ======================================================================

/// verify/obligations/match-branch-gamma-carries-scrutinee-equation  (soundness)
/// `[placeholder — reifies in V3]`
///
/// Path-sensitive Γ accumulation for match/elim requires `Eq` (`16 §2`) to
/// form the scrutinee equation `(_ : Eq A s (cₖ fields))` — a K2 prerequisite.
#[test]
fn match_branch_gamma_scrutinee_equation_placeholder() {
    // [placeholder — reifies in V3]: Elim walking + Eq-typed scrutinee equations in Γ
    // Structural: verify the current extractor does not crash on a branchy body.
    let mut env = mk_env();
    decl_nat_pred(&mut env, "MatchProp");
    // A straight-line view (no match) as a baseline — no Elim to walk.
    let elab_res = env
        .elaborate_decl_v1("view f_match (n : Nat) : Nat ensures MatchProp result = n")
        .unwrap();
    let ex = v2_extract(&elab_res);
    assert_eq!(ex.obligations.len(), 1, "straight-line body: 1 obligation");
    // Scrutinee equation in Γ for a branchy body [placeholder — reifies in V3]
}

// ======================================================================
// C.2 (placeholder) — Let binding adds equation to Γ
// ======================================================================

/// verify/obligations/let-binding-adds-equation-to-gamma
/// `[placeholder — reifies in V3]`
///
/// `let m := n + 1 in …` adds `(m : Int)` and `(_ : Eq Int m (n+1))` to Γ.
/// Requires K2 `Eq` for the propositional equation.
#[test]
fn let_binding_adds_equation_to_gamma_placeholder() {
    // [placeholder — reifies in V3]: Let walking + Eq-typed equation in Γ
    // Structural: the current context carries only the parameter telescope.
    let mut env = mk_env();
    decl_nat_pred(&mut env, "LetProp");
    let elab_res = env
        .elaborate_decl_v1("view f_let (n : Nat) : Nat ensures LetProp result = n")
        .unwrap();
    let ex = v2_extract(&elab_res);
    // With a straight-line body the context is just [Nat] — no let-equation yet.
    assert_eq!(ex.obligations[0].context.len(), 1);
    // Let-equation in Γ [placeholder — reifies in V3]
}

// ======================================================================
// C.3 (placeholder) — Conditional branch adds boolean equation
// ======================================================================

/// verify/obligations/conditional-branch-adds-boolean-equation
/// `[placeholder — reifies in V3]`
///
/// `if c then … else …` adds `Eq Bool c true`/`false` per branch. Requires
/// K2 `Eq` and `elim_Bool` walking.
#[test]
fn conditional_branch_boolean_equation_placeholder() {
    // [placeholder — reifies in V3]: If/elim_Bool walking + Eq Bool equations in Γ
}

// ======================================================================
// D.1 (placeholder) — Recursive function: per-ctor obligation with IH
// ======================================================================

/// verify/obligations/recursive-fn-per-ctor-obligation-with-ih  (soundness)
/// `[placeholder — reifies in V3]`
///
/// The IH `M zᵢ` in the `cons` branch requires reading the eliminator's
/// motive and recursive fields — K2 Eq + Elim walking.
#[test]
fn recursive_fn_per_ctor_obligation_with_ih_placeholder() {
    // [placeholder — reifies in V3]: Elim walking, motive tracking, IH in Γ
}

// ======================================================================
// D.2 (placeholder) — Non-recursive: degenerate motive, no IH
// ======================================================================

/// verify/obligations/nonrecursive-degenerate-no-induction-hypothesis
/// `[placeholder — reifies in V3]`
#[test]
fn nonrecursive_degenerate_no_ih_placeholder() {
    // [placeholder — reifies in V3]: degenerate motive case of recursive-fn test
}

// ======================================================================
// E.1 (placeholder) — Flagship: inductive postcondition hole localization
// ======================================================================

/// verify/obligations/inductive-postcond-hole-localization  (soundness)
/// `[placeholder — reifies in V3]`
///
/// End-to-end: all proofs supplied → fully verified (nothing in trusted_base);
/// one proof removed → exactly one precisely-located open hole.
#[test]
fn inductive_postcond_hole_localization_placeholder() {
    // [placeholder — reifies in V3]: depends on D.1 (per-ctor obligations with IH)
}

// ======================================================================
// F.1 — Provenance and stable ids
// ======================================================================

/// verify/obligations/provenance-and-stable-ids
///
/// Each obligation carries provenance (source clause) and a stable id
/// (`22 §1`, `24 §6`). Re-extraction yields the same ids.
#[test]
fn provenance_and_stable_ids() {
    let mut env = mk_env();
    decl_nat_pred(&mut env, "PropA");
    decl_nat_pred(&mut env, "PropB");

    // Two ensures clauses → two obligations with consecutive stable ids
    let elab_res = env
        .elaborate_decl_v1(
            "view f2 (n : Nat) : Nat ensures PropA result ensures PropB result = n",
        )
        .expect("two ensures should elaborate");

    let ex = v2_extract(&elab_res);
    assert_eq!(ex.obligations.len(), 2);

    // Stable ids keyed on name + kind + index
    assert_eq!(ex.obligations[0].id.0, "f2.ensures.0");
    assert_eq!(ex.obligations[1].id.0, "f2.ensures.1");

    // Provenance
    assert!(matches!(
        ex.obligations[0].provenance.kind,
        ProvKind::Ensures { index: 0 }
    ));
    assert!(matches!(
        ex.obligations[1].provenance.kind,
        ProvKind::Ensures { index: 1 }
    ));

    // Stable: re-extraction gives same ids
    let ex2 = v2_extract(&elab_res);
    assert_eq!(ex2.obligations[0].id, ex.obligations[0].id, "id stable on re-extraction");
    assert_eq!(ex2.obligations[1].id, ex.obligations[1].id);
}

// ======================================================================
// F.2 — Decoupled from Σ-sort
// ======================================================================

/// verify/obligations/decoupled-from-sigma-sort  (soundness)
///
/// V2 reads V1's bare-carrier + separate-obligation form — the obligation
/// goal is never a `Σ`-projection (`22 §1.1`, `21 §2`).
#[test]
fn decoupled_from_sigma_sort() {
    let mut env = mk_env();
    decl_nat_pred(&mut env, "PostCond");

    let elab_res = env
        .elaborate_decl_v1("view f_sigma (n : Nat) : Nat ensures PostCond result = n")
        .expect("elaborates");

    let ex = v2_extract(&elab_res);
    assert_eq!(ex.obligations.len(), 1);

    let triple = &ex.obligations[0];
    // phi is App(PostCond, Var(0)) — never Proj1/Proj2 of a Sigma
    assert!(
        !matches!(triple.phi, Term::Proj1(_) | Term::Proj2(_)),
        "obligation phi is not a Sigma projection"
    );
    assert!(
        !matches!(triple.goal_closed, Term::Sigma(_, _)),
        "goal_closed is not a Sigma type"
    );
    // The triple is a standalone Γ ⊢ φ — decoupled from the sort_sigma erratum
}

// ======================================================================
// V2 enrichment: V1 obligations get context + phi + stable id
// ======================================================================

/// V2 enriches V1's closed obligations with open context Γ and phi.
///
/// For a straight-line view with one ensures, the context is the parameter
/// telescope and phi is the substituted predicate.
#[test]
fn v2_enriches_v1_obligations_with_context() {
    let mut env = mk_env();
    decl_nat_pred(&mut env, "QProp");

    let elab_res = env
        .elaborate_decl_v1("view g (n : Nat) : Nat ensures QProp result = n")
        .expect("elaborates");
    let ex = v2_extract(&elab_res);
    assert_eq!(ex.obligations.len(), 1);

    let triple = &ex.obligations[0];
    // goal_closed = Pi(context, phi) — consistency check
    assert_eq!(
        triple.goal_closed,
        Term::pi(triple.context[0].clone(), triple.phi.clone()),
        "goal_closed is precisely Pi(context[0], phi)"
    );
    // V2 gives the hole_id back (consistent with V1)
    assert_eq!(
        triple.hole_id,
        elab_res.obligations[0].hole_id,
        "hole_id consistent with V1"
    );
    // id uses the def name
    assert!(
        triple.id.0.starts_with("g."),
        "stable id uses declaration name"
    );
}
