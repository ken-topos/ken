//! N3 Lane-B acceptance: per-item import rename plus fail-closed top-level
//! local/import and prelude clashes (`33 §3.2-3.3`).

use ken_elaborator::parser::parse_decls;
use ken_elaborator::{Decl, ElabEnv, ElabError, ImportKind};
use ken_kernel::Term;

fn mk_env() -> ElabEnv {
    ElabEnv::new().expect("prelude construction")
}

fn assert_ambiguous<T>(result: Result<T, ElabError>, name: &str) {
    match result {
        Err(ElabError::AmbiguousReference {
            name: actual,
            sources,
            ..
        }) => {
            assert_eq!(actual, name);
            assert!(
                sources.len() >= 2,
                "clash must retain both sources: {sources:?}"
            );
        }
        Err(other) => panic!("expected AmbiguousReference for {name}, got {other:?}"),
        Ok(_) => panic!("expected AmbiguousReference for {name}, got success"),
    }
}

#[test]
fn parser_distinguishes_item_rename_from_module_alias_and_rejects_hiding() {
    let renamed = parse_decls("import M (foo as bar)").expect("per-item rename parses");
    let Decl::ImportDecl {
        kind: ImportKind::Selective(items),
        ..
    } = &renamed[0]
    else {
        panic!("expected selective import, got {:?}", renamed[0]);
    };
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].name, "foo");
    assert_eq!(items[0].rename.as_deref(), Some("bar"));

    let aliased = parse_decls("import M as N").expect("module alias parses");
    assert!(matches!(
        &aliased[0],
        Decl::ImportDecl { kind: ImportKind::Aliased(alias), .. } if alias == "N"
    ));

    assert!(
        matches!(
            parse_decls("import M hiding (foo)"),
            Err(ElabError::ParseError { .. })
        ),
        "`hiding` has no import production"
    );
}

#[test]
fn local_import_clash_is_order_independent_and_latent() {
    for source in [
        "module M { pub const foo : Int = 0 } import M (foo) const foo : Int = 1",
        "module M { pub const foo : Int = 0 } const foo : Int = 1 import M (foo)",
    ] {
        let mut env = mk_env();
        assert_ambiguous(env.elaborate_file(source), "foo");
    }

    // Persisted root scope supplies the cross-call form of the same two orders.
    let mut import_first = mk_env();
    import_first
        .elaborate_file("module M { pub const foo : Int = 0 } import M (foo)")
        .expect("import installs");
    assert_ambiguous(import_first.elaborate_file("const foo : Int = 1"), "foo");

    let mut local_first = mk_env();
    local_first
        .elaborate_file("module M { pub const foo : Int = 0 } const foo : Int = 1")
        .expect("local installs");
    assert_ambiguous(local_first.elaborate_file("import M (foo)"), "foo");
}

#[test]
fn deselection_and_rename_resolve_the_clash_with_distinct_identities() {
    let mut deselected = mk_env();
    deselected
        .elaborate_file(
            "module M { pub const foo : Int = 0 pub const kept : Int = 2 } \
             import M (kept) const foo : Int = 1 const local_ref : Int = foo",
        )
        .expect("omitting M.foo leaves the local as sole binding");
    let local = deselected.globals["foo"];
    let (_, body) = deselected
        .env
        .transparent_body(deselected.globals["local_ref"])
        .expect("local_ref transparent");
    assert!(matches!(body, Term::Const { id, .. } if id == local));

    let mut renamed = mk_env();
    renamed
        .elaborate_file(
            "module M { pub const foo : Int = 0 } \
             import M (foo as imported_foo) \
             const foo : Int = 1 \
             const local_ref : Int = foo \
             const imported_ref : Int = imported_foo",
        )
        .expect("renamed import and local coexist");
    let local = renamed.globals["foo"];
    let imported = renamed.globals["M.foo"];
    assert_ne!(local, imported);
    for (reference, expected) in [("local_ref", local), ("imported_ref", imported)] {
        let (_, body) = renamed
            .env
            .transparent_body(renamed.globals[reference])
            .expect("reference transparent");
        assert!(
            matches!(body, Term::Const { id, .. } if id == expected),
            "{reference} must preserve the selected declaration identity; got {body:?}"
        );
    }
}

#[test]
fn prelude_is_unshadowable_but_lexical_binders_still_shadow_imports() {
    let mut prelude_clash = mk_env();
    assert_ambiguous(prelude_clash.elaborate_file("def Bool = Nat"), "Bool");

    let mut renamed_local = mk_env();
    renamed_local
        .elaborate_file("def LocalBool = Bool")
        .expect("renaming the local resolves a prelude clash");
    assert_ne!(
        renamed_local.globals["LocalBool"],
        renamed_local.globals["Bool"]
    );

    let mut lexical = mk_env();
    lexical
        .elaborate_file(
            "module M { pub const foo : Nat = Zero } \
             import M (foo) fn lexical (foo : Nat) : Nat = foo",
        )
        .expect("parameter shadowing remains lexical innermost-wins");
    let (_, body) = lexical
        .env
        .transparent_body(lexical.globals["lexical"])
        .expect("lexical transparent");
    assert!(
        matches!(&body, Term::Lam(_, inner) if matches!(inner.as_ref(), Term::Var(0))),
        "function body must resolve to its de Bruijn parameter, got {body:?}"
    );
}
