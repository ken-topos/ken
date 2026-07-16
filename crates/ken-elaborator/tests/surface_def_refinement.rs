//! SURF-def-refinement: the `type` → `def` declaration-keyword rename.
//!
//! Pins `spec/30-surface/33-declarations.md` §1, `32-grammar.md` §1, and
//! `31-lexical.md` §4, plus the conformance seed
//! `conformance/surface/declarations/seed-def-refinement.md`: `def` is the
//! renamed type-level definition keyword (refinement or plain alias), `type`
//! is reserved (no longer a declaration keyword, no longer a free
//! identifier), and elaboration is byte-identical to the pre-rename `type`
//! form — zero kernel delta.

use ken_elaborator::{ElabEnv, ElabError};
use ken_kernel::{Decl, Term};

fn mk_env() -> ElabEnv {
    ElabEnv::new().expect("base env construction failed")
}

fn elaborate_ok(src: &str) -> ElabEnv {
    let mut env = mk_env();
    env.elaborate_file(src)
        .unwrap_or_else(|e| panic!("source should elaborate: {e}"));
    env
}

fn elaborate_err(src: &str) -> ElabError {
    let mut env = mk_env();
    env.elaborate_file(src)
        .expect_err("source should be rejected")
}

#[test]
fn def_refinement_parses_and_elaborates_without_trusted_base_growth() {
    let mut env = mk_env();
    let before = env.env.trusted_base();

    env.elaborate_file(
        r#"
        fn posP (n : Int) : Prop = IsTrue (leq_int 1 n)
        def Pos = { n : Int | posP n }
        "#,
    )
    .expect("def refinement elaborates");

    assert!(env.globals.contains_key("Pos"));
    assert_eq!(
        before,
        env.env.trusted_base(),
        "a def refinement must not grow the trusted base (frame AC §4.3)"
    );
}

#[test]
fn def_alias_parses_and_elaborates() {
    let env = elaborate_ok(
        r#"
        data DecimalPair = MkDecimalPair Int Int
        def Decimal = DecimalPair
        "#,
    );
    assert!(env.globals.contains_key("Decimal"));
    assert!(env.globals.contains_key("DecimalPair"));
}

#[test]
fn def_refinement_and_def_alias_elaborate_to_the_same_core_decl_as_type_used_to() {
    // The parser's `def` and (formerly) `type` branches both call the same
    // `parse_type_alias_decl` → `Decl::TypeAlias` → `RDeclKind::TypeAlias`
    // path (`crates/ken-elaborator/src/{parser,resolve,elab}.rs`) — a single
    // code path renamed at the keyword-token boundary, so the elaborated core
    // decl is identical to what `type` produced by construction, not just by
    // observation. This test pins the *observable* half: the core `Decl` for
    // a `def` alias is `Transparent` with body exactly the aliased type.
    let alias_env = elaborate_ok(
        r#"
        data DecimalPair = MkDecimalPair Int Int
        def Decimal = DecimalPair
        "#,
    );
    let decimal_id = *alias_env.globals.get("Decimal").unwrap();
    let decimalpair_id = *alias_env.globals.get("DecimalPair").unwrap();
    let decl = alias_env.env.lookup(decimal_id).expect("Decimal registered");
    match decl {
        Decl::Transparent { body, .. } => {
            let expected = Term::indformer(decimalpair_id, vec![]);
            assert_eq!(
                *body, expected,
                "def alias body must be exactly the aliased type's IndFormer term"
            );
        }
        other => panic!("expected a Transparent alias decl, found {other:?}"),
    }
}

#[test]
fn type_keyword_no_longer_parses_as_a_declaration() {
    let err = elaborate_err("type Foo = Int");
    let msg = format!("{err}");
    assert!(
        msg.contains("reserved"),
        "expected a reserved-keyword diagnostic, got: {msg}"
    );
}

#[test]
fn type_is_reserved_not_a_free_identifier() {
    let err = elaborate_err("fn type (x : Int) : Int = x");
    let msg = format!("{err}");
    assert!(
        msg.contains("KwTypeReserved"),
        "expected 'type' to be rejected as its own reserved token (never falling back to an \
         identifier) even outside decl position, got: {msg}"
    );
}

#[test]
fn def_value_position_yields_steering_diagnostic() {
    let err = elaborate_err("def double x = x * 2");
    let msg = format!("{err}");
    assert!(
        msg.contains("'def' defines a type") && msg.contains("'fn'") && msg.contains("'const'"),
        "expected the def-is-a-type steering diagnostic, got: {msg}"
    );
}
