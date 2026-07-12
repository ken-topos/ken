//! Proof-vocabulary admission: proofs share the definition SCC/SCT seam.

use ken_elaborator::{ElabEnv, ElabError};
use ken_kernel::KernelError;

fn env() -> ElabEnv {
    ElabEnv::new().expect("base environment")
}

#[test]
fn structural_and_forward_proof_definitions_elaborate() {
    let mut elab = env();
    let before = elab.env.trusted_base();
    elab.elaborate_file(
        r#"
        lemma self_refl (x : Nat) : Equal Nat x x =
          match x { Zero |-> Proved ; Suc x2 |-> self_refl x2 }
        lemma use_later (x : Int) : Equal Int (later x) x = later_refl x
        fn later (x : Int) : Int = x
        lemma later_refl (x : Int) : Equal Int (later x) x = Refl
        "#,
    )
    .expect("structural proof and forward references elaborate");
    assert_eq!(
        before,
        elab.env.trusted_base(),
        "proofs add no trusted base"
    );
}

#[test]
fn public_module_definitions_receive_the_same_forward_admission() {
    let mut elab = env();
    elab.elaborate_file(
        r#"
        module M {
          pub lemma use_later (x : Int) : Equal Int (later x) x = later_refl x
          pub fn later (x : Int) : Int = x
          pub lemma later_refl (x : Int) : Equal Int (later x) x = Refl
        }
        "#,
    )
    .expect("public definitions must use scope-wide forward admission");
    assert!(elab.globals.contains_key("M.use_later"));
    assert!(elab.globals.contains_key("M.later"));
    assert!(elab.globals.contains_key("M.later_refl"));
}

#[test]
fn mutual_scc_orders_dependencies_from_non_entry_members() {
    let mut elab = env();
    elab.elaborate_file(
        r#"
        fn a (n : Nat) : Nat = match n { Zero |-> Zero ; Suc m |-> b m }
        fn b (n : Nat) : Nat = match n { Zero |-> w ; Suc m |-> a m }
        const w : Nat = Zero
        "#,
    )
    .expect("a non-entry SCC member's forward dependency must be elaborated first");
    assert!(elab.globals.contains_key("a"));
    assert!(elab.globals.contains_key("b"));
    assert!(elab.globals.contains_key("w"));
}

#[test]
fn homogeneous_mutual_proofs_admit_but_non_descending_proofs_fail_at_sct() {
    let mut elab = env();
    elab.elaborate_file(
        r#"
        lemma left (n : Nat) : Equal Nat n n =
          match n { Zero |-> Proved ; Suc m |-> right m }
        lemma right (n : Nat) : Equal Nat n n =
          match n { Zero |-> Proved ; Suc m |-> left m }
        "#,
    )
    .expect("homogeneous descending proof SCC must pass SCT");

    let err = env()
        .elaborate_file("lemma bad (n : Nat) : Equal Nat n n = bad n")
        .expect_err("non-descending proof self recursion must fail closed");
    assert!(matches!(
        err,
        ElabError::KernelRejected {
            error: KernelError::NotTerminating(_),
            ..
        }
    ));
}

#[test]
fn attached_proof_uses_occurs_applied_and_mixed_cycles_fail_closed() {
    let mut elab = env();
    elab.elaborate_file(
        r#"
        fn id (x : Int) : Int = x
        proof refl for id (x : Int) : Equal Int (id x) x = Refl
        proof refl_via_sibling for id (x : Int) : Equal Int (id x) x = id::refl x
        "#,
    )
    .expect("attached sibling proof and applied-subject claim elaborate");

    let err = env()
        .elaborate_file(
            "fn id (x : Int) : Int = x\n\
             proof unrelated for id (x : Int) : Equal Int x x = Refl",
        )
        .expect_err("an attached proof must mention its subject applied in its claim");
    assert!(matches!(err, ElabError::TypeMismatch { reason, .. } if reason.contains("applied")));

    let err = env()
        .elaborate_file(
            "fn computational (n : Nat) : Equal Nat n n = proof_member n\n\
             lemma proof_member (n : Nat) : Equal Nat n n = computational n",
        )
        .expect_err("mixed fn/proof recursive SCC stays explicitly deferred");
    assert!(matches!(err, ElabError::TypeMismatch { reason, .. } if reason.contains("mixed")));
}

#[test]
fn computational_keywords_reject_omega_results_and_proved_names_top() {
    let diagnostic = "`fn`/`const` compute; use `lemma`/`proof` for an Ω-valued definition";

    for declaration in [
        "fn bad_fn (x : Nat) : Equal Nat x x = Refl",
        "const bad_const : Equal Bool True True = Proved",
        "const bad_inferred = Proved",
    ] {
        let err = env()
            .elaborate_decl(declaration)
            .expect_err("computational keyword must reject an Omega-valued result");
        assert!(
            matches!(err, ElabError::TypeMismatch { ref reason, .. } if reason == diagnostic),
            "expected vocabulary diagnostic, got {err:?}"
        );
    }

    let mut elab = env();
    elab.elaborate_decl("const inferred_nat = Zero")
        .expect("an inferred computational result must remain accepted");
    elab.elaborate_file(
        "lemma accepted (x : Nat) : Equal Nat x x = Refl\n\
         lemma top_token : Top = Proved",
    )
    .expect("lemma/proved must admit the Omega-valued proof vocabulary");
    assert!(elab.globals.contains_key("Top"));
    assert!(elab.globals.contains_key("Proved"));
    assert!(!elab.globals.contains_key("tt"));
}
