//! WP N1 Lane B: fail-closed duplicate definitions in one compilation unit.

use ken_elaborator::{ElabEnv, ElabError};
use ken_kernel::Term;

fn expect_duplicate(source: &str, expected_name: &str) {
    let mut env = ElabEnv::new().expect("base environment");
    match env.elaborate_file(source) {
        Err(ElabError::DuplicateDefinition { name, .. }) => {
            assert_eq!(
                name, expected_name,
                "the diagnostic must identify the collision"
            );
        }
        Err(other) => panic!("expected DuplicateDefinition, got {other:?}"),
        Ok(ids) => panic!("duplicate definition unexpectedly elaborated as {ids:?}"),
    }
}

fn contains_j(term: &Term) -> bool {
    match term {
        Term::J(..) => true,
        Term::App(function, argument) => contains_j(function) || contains_j(argument),
        Term::Pi(domain, codomain)
        | Term::Lam(domain, codomain)
        | Term::Sigma(domain, codomain)
        | Term::Ascript(domain, codomain) => contains_j(domain) || contains_j(codomain),
        Term::Let { ty, val, body } => contains_j(ty) || contains_j(val) || contains_j(body),
        _ => false,
    }
}

#[test]
fn duplicate_ordinary_definition_uses_specific_variant_and_payload() {
    expect_duplicate(
        "fn keep (x : Bool) : Bool = x\nfn keep (x : Bool) : Bool = x",
        "keep",
    );
}

#[test]
fn class_and_constructor_share_the_duplicate_definition_funnel() {
    expect_duplicate(
        "class Eq a { eq : a -> a -> Bool }\n\
         fn J (x : Bool) : Bool = x\n\
         data Marker = Eq\n\
         lemma eq_sugar (a : Type) (x : a) : Eq a x x = Refl\n\
         lemma j_sugar\n\
           (ty : Type) (a : ty) (b : ty) (q : Equal ty a b)\n\
           : Equal ty a b = J (\\b' _. Equal ty a b') Refl q",
        "Eq",
    );
}

#[test]
fn lower_arity_eq_j_and_real_sugar_remain_jointly_live() {
    let mut env = ElabEnv::new().expect("base environment");
    let ids = env
        .elaborate_file(
            "class Eq a { eq : a -> a -> Bool }\n\
             fn J (x : Bool) : Bool = x\n\
             data Marker = Only\n\
             lemma eq_sugar (a : Type) (x : a) : Eq a x x = Refl\n\
             lemma j_sugar\n\
               (ty : Type) (a : ty) (b : ty) (q : Equal ty a b)\n\
               : Equal ty a b = J (\\b' _. Equal ty a b') Refl q",
        )
        .expect("the Lane A arity-gated positive control must remain accepted");

    let (_, body) = env
        .env
        .transparent_body(ids[4])
        .expect("j_sugar is transparent");
    assert!(
        contains_j(&body),
        "the three-argument J site must lower to a real Term::J, got {body:?}"
    );
}

#[test]
fn totally_reserved_sugar_names_keep_the_existing_hard_error() {
    for (source, expected_name) in [
        ("const Refl : Bool = True", "Refl"),
        ("const Axiom : Bool = True", "Axiom"),
        ("fn absurd (x : Bool) : Bool = x", "absurd"),
    ] {
        let mut env = ElabEnv::new().expect("base environment");
        match env.elaborate_file(source) {
            Err(ElabError::ParseError { msg, .. }) => {
                assert!(msg.contains(expected_name));
                assert!(msg.contains("reserved surface sugar"));
            }
            Err(other) => panic!("expected reserved-sugar ParseError, got {other:?}"),
            Ok(ids) => panic!("reserved name unexpectedly elaborated as {ids:?}"),
        }
    }
}
