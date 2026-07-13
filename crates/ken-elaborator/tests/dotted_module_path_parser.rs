//! `catalog-taxonomy-paths-imports` WP, P2(a) — `import`/`import … as`/
//! selective `import`/`use`/`module` all accept a dotted module path
//! (`ConId` then zero-or-more `.ConId`), mirroring the catalog's Section >
//! Domain path↔import identity (`docs/program/07-catalog-style-guide.md`).
//! Module stays path-inferred (P2b) — these tests only exercise the parser,
//! never assert a mandatory in-file `module` header.

use ken_elaborator::parser::parse_decls;
use ken_elaborator::{Decl, ImportKind};

fn single_decl(src: &str) -> Decl {
    let mut decls = parse_decls(src).expect("source must parse");
    assert_eq!(decls.len(), 1);
    decls.remove(0)
}

#[test]
fn import_accepts_a_dotted_module_path() {
    let decl = single_decl("import Data.Collections.Map");
    let Decl::ImportDecl { module, kind, .. } = decl else {
        panic!("expected ImportDecl, got {decl:?}");
    };
    assert_eq!(module, "Data.Collections.Map");
    assert!(matches!(kind, ImportKind::Qualified));
}

#[test]
fn import_as_accepts_a_dotted_module_path() {
    let decl = single_decl("import Data.Collections.Map as M");
    let Decl::ImportDecl { module, kind, .. } = decl else {
        panic!("expected ImportDecl, got {decl:?}");
    };
    assert_eq!(module, "Data.Collections.Map");
    match kind {
        ImportKind::Aliased(alias) => assert_eq!(alias, "M"),
        other => panic!("expected Aliased, got {other:?}"),
    }
}

#[test]
fn selective_import_accepts_a_dotted_module_path() {
    let decl = single_decl("import Core.LawfulClasses (eq, Ord)");
    let Decl::ImportDecl { module, kind, .. } = decl else {
        panic!("expected ImportDecl, got {decl:?}");
    };
    assert_eq!(module, "Core.LawfulClasses");
    match kind {
        ImportKind::Selective(items) => {
            assert_eq!(items.len(), 2);
            assert_eq!(items[0].name, "eq");
            assert_eq!(items[0].rename, None);
            assert_eq!(items[1].name, "Ord");
            assert_eq!(items[1].rename, None);
        }
        other => panic!("expected Selective, got {other:?}"),
    }
}

#[test]
fn use_accepts_a_dotted_module_path() {
    let decl = single_decl("use Capability.Parsing.Parsing");
    let Decl::ImportDecl { module, kind, .. } = decl else {
        panic!("expected ImportDecl, got {decl:?}");
    };
    assert_eq!(module, "Capability.Parsing.Parsing");
    assert!(matches!(kind, ImportKind::Open));
}

#[test]
fn module_decl_accepts_a_dotted_module_path() {
    let decl = single_decl("module Data.Collections { fn id (x : Bool) : Bool = x }");
    let Decl::ModuleDecl { name, decls, .. } = decl else {
        panic!("expected ModuleDecl, got {decl:?}");
    };
    assert_eq!(name, "Data.Collections");
    assert_eq!(decls.len(), 1);
}

// Non-regression: single-component (undotted) forms must still parse
// exactly as before the taxonomy WP.
#[test]
fn undotted_forms_still_parse() {
    let decl = single_decl("import Nat");
    let Decl::ImportDecl { module, kind, .. } = decl else {
        panic!("expected ImportDecl, got {decl:?}");
    };
    assert_eq!(module, "Nat");
    assert!(matches!(kind, ImportKind::Qualified));

    let decl = single_decl("use Nat");
    let Decl::ImportDecl { module, .. } = decl else {
        panic!("expected ImportDecl, got {decl:?}");
    };
    assert_eq!(module, "Nat");

    let decl = single_decl("module Nat { fn id (x : Bool) : Bool = x }");
    let Decl::ModuleDecl { name, .. } = decl else {
        panic!("expected ModuleDecl, got {decl:?}");
    };
    assert_eq!(name, "Nat");
}

// A module path component must be `ConId` (uppercase-initial, `31 §1`) — a
// lowercase segment is not a valid module path, distinct from expression-
// position `.field` projection (`parse_dotted`).
#[test]
fn import_rejects_a_lowercase_path_component() {
    let err = parse_decls("import Data.collections")
        .expect_err("a lowercase module path component must be rejected");
    let msg = format!("{err}");
    assert!(msg.contains("expected uppercase constructor name"), "{msg}");
}
