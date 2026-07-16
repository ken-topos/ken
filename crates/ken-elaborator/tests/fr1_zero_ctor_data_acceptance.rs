//! FR-1 (`docs/program/wp/ds-1-findings-remediation.md`) acceptance —
//! zero-constructor `data` has a real surface spelling.
//!
//! DS-1's `Empty` was bootstrapped by calling `data::elab_data_decl`
//! directly, bypassing the parser (`crates/ken-elaborator/src/prelude.rs`).
//! These tests exercise a genuinely surface-authored zero-constructor
//! `data` — distinct globals, going through the ordinary
//! `elaborate_decl`/`parse_decls` source-text path — for both surface
//! spellings the parser now admits.

use ken_elaborator::parser::parse_decls;
use ken_elaborator::{Decl, ElabEnv};

// The explicit-family spelling: `data D : Type where { }`.
#[test]
fn explicit_family_zero_ctor_data_elaborates_and_eliminates() {
    let mut env = ElabEnv::empty().expect("prelude bootstrap");
    // `Void` is not a prelude global — a fresh, surface-authored type.
    assert!(!env.globals.contains_key("Void"), "Void must not pre-exist");

    env.elaborate_decl("data Void : Type where { }")
        .expect("a surface-authored zero-constructor explicit data must elaborate");
    assert!(
        env.globals.contains_key("Void"),
        "Void must be a registered global"
    );

    // The general Type-sorted eliminator (empty `match`, large elim) must
    // work over this surface-declared type, not just the prelude's `Empty`.
    env.elaborate_decl("fn absurdVoid (C : Type) (e : Void) : C = match e { }")
        .expect("empty-match eliminator must elaborate over a surface-declared Void");
}

// The legacy spelling: `data D =` (no constructors after `=`).
#[test]
fn legacy_zero_ctor_data_elaborates_and_eliminates() {
    let mut env = ElabEnv::empty().expect("prelude bootstrap");
    assert!(!env.globals.contains_key("Nope"), "Nope must not pre-exist");

    env.elaborate_decl("data Nope =")
        .expect("a surface-authored legacy zero-constructor data must elaborate");
    assert!(
        env.globals.contains_key("Nope"),
        "Nope must be a registered global"
    );

    env.elaborate_decl("fn absurdNope (C : Type) (e : Nope) : C = match e { }")
        .expect("empty-match eliminator must elaborate over a surface-declared Nope");
}

// Parser-level: confirm the AST shape directly (zero ctors, not a parse
// failure) for both spellings, independent of elaboration.
#[test]
fn both_spellings_parse_to_zero_constructor_decls() {
    let mut decls = parse_decls("data Void : Type where { }").expect("explicit form must parse");
    assert_eq!(decls.len(), 1);
    match decls.remove(0) {
        Decl::ExplicitDataDecl { name, ctors, .. } => {
            assert_eq!(name, "Void");
            assert!(ctors.is_empty());
        }
        other => panic!("expected ExplicitDataDecl, got {other:?}"),
    }

    let mut decls = parse_decls("data Nope =").expect("legacy form must parse");
    assert_eq!(decls.len(), 1);
    match decls.remove(0) {
        Decl::DataDecl { name, ctors, .. } => {
            assert_eq!(name, "Nope");
            assert!(ctors.is_empty());
        }
        other => panic!("expected DataDecl, got {other:?}"),
    }
}

// Existing non-empty forms must be entirely unaffected by the gate relax.
#[test]
fn non_empty_data_forms_still_parse_and_elaborate() {
    let mut env = ElabEnv::empty().expect("prelude bootstrap");
    env.elaborate_decl("data Trit = Neg | Zero_ | Pos")
        .expect("legacy non-empty data must still elaborate");
    env.elaborate_decl("data Box (A : Type) : Type where { Mk : A -> Box A }")
        .expect("explicit non-empty data must still elaborate");
}
