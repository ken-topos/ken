//! Acceptance coverage for the bare attached-proof selector atom.
//!
//! Pins `spec/30-surface/32-grammar.md` and
//! `spec/30-surface/33-declarations.md` section 8.2.

use ken_elaborator::{
    parser::{parse_decls, parse_expr},
    Decl, ElabEnv, Expr,
};

fn selector_bytes(source: &str) -> Vec<u8> {
    match parse_expr(source).unwrap_or_else(|error| panic!("{source:?} should parse: {error}")) {
        Expr::EAttachedProofRef {
            subject,
            proof_name,
            ..
        } => [subject.as_bytes(), b"\0", proof_name.as_bytes()].concat(),
        other => panic!("{source:?} should be an attached-proof selector, got {other:?}"),
    }
}

#[test]
fn bare_grouped_and_canonical_selectors_have_identical_ast_and_elaboration() {
    let bare = selector_bytes("proof p for s");
    let grouped = selector_bytes("(proof p for s)");
    let canonical = selector_bytes("s::p");
    assert_eq!(bare, grouped);
    assert_eq!(bare, canonical);

    let mut env = ElabEnv::new().expect("base env construction failed");
    env.elaborate_file(
        r#"
        fn s (x : Int) : Int = x
        proof p for s (x : Int) : Equal Int (s x) x = Refl
        lemma via_bare (x : Int) : Equal Int (s x) x = proof p for s x
        lemma via_grouped (x : Int) : Equal Int (s x) x = (proof p for s) x
        lemma via_canonical (x : Int) : Equal Int (s x) x = s::p x
        "#,
    )
    .expect("all three selector spellings should elaborate");

    let body = |name: &str| {
        env.env
            .transparent_body(env.globals[name])
            .unwrap_or_else(|| panic!("{name} should be transparent"))
            .1
    };
    assert_eq!(body("via_bare"), body("via_grouped"));
    assert_eq!(body("via_bare"), body("via_canonical"));
}

#[test]
fn application_wraps_outside_the_single_path_selector_subject() {
    let parsed = parse_expr("proof p for Mod.s a b").expect("applied bare selector should parse");
    match parsed {
        Expr::EApp(outer_fun, outer_arg, _) => {
            assert!(matches!(*outer_arg, Expr::EVar(ref name, _) if name == "b"));
            match *outer_fun {
                Expr::EApp(inner_fun, inner_arg, _) => {
                    assert!(matches!(*inner_arg, Expr::EVar(ref name, _) if name == "a"));
                    assert!(matches!(
                        *inner_fun,
                        Expr::EAttachedProofRef {
                            ref subject,
                            ref proof_name,
                            ..
                        } if subject == "Mod.s" && proof_name == "p"
                    ));
                }
                other => panic!("expected the inner application spine, got {other:?}"),
            }
        }
        other => panic!("expected ((proof p for Mod.s) a) b, got {other:?}"),
    }
}

#[test]
fn recursive_attached_proof_can_use_its_bare_selector() {
    let mut env = ElabEnv::new().expect("base env construction failed");
    env.elaborate_file(
        r#"
        fn leq_nat (m : Nat) (n : Nat) : Bool =
          match m {
            Zero |-> True ;
            Suc m2 |-> match n { Zero |-> False ; Suc n2 |-> leq_nat m2 n2 }
          }
        proof refl for leq_nat (x : Nat) : Equal Bool (leq_nat x x) True =
          match x { Zero |-> Proved ; Suc x2 |-> proof refl for leq_nat x2 }
        "#,
    )
    .expect("descending recursive proof through a bare self-selector should elaborate");

    assert!(env.globals.contains_key("leq_nat::refl"));
}

#[test]
fn declaration_head_and_bare_selector_body_are_parsed_and_elaborated_separately() {
    let source = r#"
        fn s (x : Int) : Int = x
        proof p for s (x : Int) : Equal Int (s x) x = Refl
        proof q for s (x : Int) : Equal Int (s x) x = proof p for s x
    "#;
    let declarations = parse_decls(source).expect("proof declaration with selector body parses");
    let declaration = declarations
        .last()
        .expect("q declaration should be present");
    match declaration {
        Decl::AttachedProofDecl {
            proof_name,
            subject,
            body,
            ..
        } => {
            assert_eq!(proof_name, "q", "declaration-position proof names the head");
            assert_eq!(subject, "s", "declaration-position `for` names the subject");
            assert!(matches!(
                body,
                Expr::EApp(fun, arg, _)
                    if matches!(
                        &**fun,
                        Expr::EAttachedProofRef {
                            subject,
                            proof_name,
                            ..
                        } if subject == "s" && proof_name == "p"
                    ) && matches!(&**arg, Expr::EVar(name, _) if name == "x")
            ));
        }
        other => panic!("expected an attached-proof declaration, got {other:?}"),
    }

    let mut env = ElabEnv::new().expect("base env construction failed");
    env.elaborate_file(source)
        .expect("declaration head and bare-selector body should elaborate without cross-talk");
    assert!(env.globals.contains_key("s::q"));
}
