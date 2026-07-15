//! L4 `export` acceptance (`33 §3.2/§4`, ADR 0016).

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use ken_elaborator::layout::format_ken;
use ken_elaborator::parser::parse_decls;
use ken_elaborator::{Decl, ElabEnv, ElabError, ExportForm};
use ken_kernel::Term;

const CONFORMANCE_SEED: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../conformance/surface/declarations/seed-namespace-export.md"
));

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

struct FixtureRoot(PathBuf);

impl FixtureRoot {
    fn new(label: &str) -> Self {
        let serial = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "ken-l4-export-{label}-{}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create L4 fixture root");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn write(&self, relative: &str, source: &str) {
        let path = self.0.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create fixture parent");
        }
        fs::write(path, source).expect("write L4 fixture");
    }
}

impl Drop for FixtureRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn body_const(env: &ElabEnv, name: &str) -> ken_kernel::GlobalId {
    let (_, body) = env
        .env
        .transparent_body(env.globals[name])
        .unwrap_or_else(|| panic!("{name} must be transparent"));
    match body {
        Term::Const { id, .. } => id,
        other => panic!("{name} must be a canonical constant reference, got {other:?}"),
    }
}

fn load_entry(label: &str, files: &[(&str, &str)]) -> Result<ElabEnv, ElabError> {
    let root = FixtureRoot::new(label);
    for (path, source) in files {
        root.write(path, source);
    }
    let mut env = ElabEnv::new().expect("base environment");
    env.elaborate_module_from_roots(&[root.path().to_path_buf()], "Entry")?;
    Ok(env)
}

#[test]
fn parser_distinguishes_facade_and_in_scope_forms_with_renames() {
    let decls = parse_decls("export M (foo, Bar as baz) export foo, Bar as baz")
        .expect("both export forms parse");
    match &decls[0] {
        Decl::ExportDecl {
            form: ExportForm::Facade { module, items },
            ..
        } => {
            assert_eq!(module, "M");
            assert_eq!(items[0].name, "foo");
            assert_eq!(items[0].rename, None);
            assert_eq!(items[1].name, "Bar");
            assert_eq!(items[1].rename.as_deref(), Some("baz"));
        }
        other => panic!("expected facade export, got {other:?}"),
    }
    match &decls[1] {
        Decl::ExportDecl {
            form: ExportForm::InScope { items },
            ..
        } => {
            assert_eq!(items[0].name, "foo");
            assert_eq!(items[1].name, "Bar");
            assert_eq!(items[1].rename.as_deref(), Some("baz"));
        }
        other => panic!("expected in-scope export, got {other:?}"),
    }

    let single = parse_decls("export M").expect("bare M is an in-scope item");
    assert!(matches!(
        &single[0],
        Decl::ExportDecl {
            form: ExportForm::InScope { items },
            ..
        } if items.len() == 1 && items[0].name == "M"
    ));
}

#[test]
fn facade_publishes_without_binding_while_in_scope_export_republishes_binding() {
    let mut facade = ElabEnv::new().expect("base environment");
    facade
        .elaborate_file(
            "module M { pub const foo : Nat = Zero } \
             module P { export M (foo) } \
             import P (foo) const observed : Nat = foo",
        )
        .expect("facade publishes M.foo to P's clients");
    assert_eq!(body_const(&facade, "observed"), facade.globals["M.foo"]);
    assert!(!facade.globals.contains_key("P.foo"));

    let mut does_not_bind = ElabEnv::new().expect("base environment");
    match does_not_bind.elaborate_file(
        "module M { pub const foo : Nat = Zero } \
         module P { export M (foo) const bad : Nat = foo }",
    ) {
        Err(ElabError::UnboundName { name, .. }) => assert_eq!(name, "foo"),
        other => panic!("facade body lookup must be UnboundName(foo), got {other:?}"),
    }

    let mut in_scope = ElabEnv::new().expect("base environment");
    in_scope
        .elaborate_file(
            "module M { pub const foo : Nat = Zero } \
             module P { import M (foo) export foo const local : Nat = foo } \
             import P (foo) const observed : Nat = foo",
        )
        .expect("in-scope export republishes an existing body binding");
    assert_eq!(body_const(&in_scope, "P.local"), in_scope.globals["M.foo"]);
    assert_eq!(body_const(&in_scope, "observed"), in_scope.globals["M.foo"]);
}

#[test]
fn unresolved_sources_fail_closed_at_the_export_site() {
    let mut in_scope = ElabEnv::new().expect("base environment");
    assert!(matches!(
        in_scope.elaborate_file("module P { export missing }"),
        Err(ElabError::UnboundName { ref name, .. }) if name == "missing"
    ));

    let mut facade = ElabEnv::new().expect("base environment");
    match facade
        .elaborate_file("module M { pub const kept : Nat = Zero } module P { export M (missing) }")
    {
        Err(ElabError::UnboundName { name, .. }) => assert_eq!(name, "M.missing"),
        other => panic!("missing facade member must fail closed, got {other:?}"),
    }
}

#[test]
fn rename_and_same_identity_paths_preserve_the_defined_at_global_id() {
    let mut env = ElabEnv::new().expect("base environment");
    env.elaborate_file(
        "module M { pub const foo : Nat = Zero } \
         module P { export M (foo as bar) } \
         import M (foo) import P (bar) \
         const direct : Nat = foo const via_facade : Nat = bar",
    )
    .expect("rename changes only the published spelling");
    let canonical = env.globals["M.foo"];
    assert_eq!(body_const(&env, "direct"), canonical);
    assert_eq!(body_const(&env, "via_facade"), canonical);
    assert!(!env.globals.contains_key("P.bar"));

    let mut same_name = ElabEnv::new().expect("base environment");
    same_name
        .elaborate_file(
            "module M { pub const foo : Nat = Zero } \
             module P { export M (foo) } \
             import M (foo) import P (foo) const observed : Nat = foo",
        )
        .expect("two paths to one identity are idempotent");
    assert_eq!(
        body_const(&same_name, "observed"),
        same_name.globals["M.foo"]
    );
}

#[test]
fn distinct_identities_collide_at_reexport_in_both_source_orders() {
    for body in [
        "pub const foo : Nat = Zero export M (foo)",
        "export M (foo) pub const foo : Nat = Zero",
    ] {
        let mut env = ElabEnv::new().expect("base environment");
        let source = format!("module M {{ pub const foo : Nat = Zero }} module P {{ {body} }}");
        match env.elaborate_file(&source) {
            Err(ElabError::ReExportCollision {
                surface_name,
                existing,
                incoming,
                ..
            }) => {
                assert_eq!(surface_name, "foo");
                assert_ne!(existing, incoming);
                assert!([existing.as_str(), incoming.as_str()].contains(&"M.foo"));
                assert!([existing.as_str(), incoming.as_str()].contains(&"P.foo"));
            }
            other => panic!("distinct public identities must collide, got {other:?}"),
        }
    }
}

#[test]
fn facade_loader_edge_participates_in_cycle_detection() {
    let root = FixtureRoot::new("cycle");
    root.write("A.ken", "export B (value)");
    root.write("B.ken", "import A\npub const value : Bool = True");
    let mut env = ElabEnv::new().expect("base environment");
    match env.elaborate_module_from_roots(&[root.path().to_path_buf()], "A") {
        Err(ElabError::ImportCycle { cycle, .. }) => {
            assert_eq!(cycle, vec!["A", "B", "A"])
        }
        other => panic!("expected facade ImportCycle A -> B -> A, got {other:?}"),
    }
    assert_eq!(env.loaded_module_count(), 0);
}

#[test]
fn imported_identity_clashes_are_latent_fail_closed_but_same_identity_is_idempotent() {
    let mut distinct = ElabEnv::new().expect("base environment");
    match distinct.elaborate_file(
        "module M { pub const foo : Nat = Zero } \
         module N { pub const foo : Nat = Suc Zero } \
         import M (foo) import N (foo)",
    ) {
        Err(ElabError::AmbiguousReference { name, sources, .. }) => {
            assert_eq!(name, "foo");
            assert!(sources.contains(&"M.foo".to_string()));
            assert!(sources.contains(&"N.foo".to_string()));
        }
        other => panic!("unused distinct imports must reject at binding, got {other:?}"),
    }

    let mut prelude = ElabEnv::new().expect("base environment");
    assert!(matches!(
        prelude.elaborate_file("module M { pub def Bool = Nat } import M (Bool)"),
        Err(ElabError::AmbiguousReference { ref name, .. }) if name == "Bool"
    ));
}

#[test]
fn loader_executes_import_identity_clash_and_rename_discriminators() {
    let modules = [
        ("M.ken", "pub const foo : Nat = Zero\n"),
        ("N.ken", "pub const foo : Nat = Suc Zero\n"),
    ];
    for (label, imports) in [
        ("distinct-mn", "import M (foo)\nimport N (foo)\n"),
        ("distinct-nm", "import N (foo)\nimport M (foo)\n"),
    ] {
        let files = [modules[0], modules[1], ("Entry.ken", imports)];
        match load_entry(label, &files) {
            Err(ElabError::AmbiguousReference { name, sources, .. }) => {
                assert_eq!(name, "foo");
                assert!(sources.contains(&"M.foo".to_string()));
                assert!(sources.contains(&"N.foo".to_string()));
            }
            Err(other) => panic!("{label} rejected with the wrong error: {other}"),
            Ok(_) => panic!("{label} must reject both distinct origins"),
        }
    }

    let renamed = load_entry(
        "distinct-rename",
        &[
            modules[0],
            modules[1],
            (
                "Entry.ken",
                "import M (foo)\n\
                 import N (foo as nfoo)\n\
                 const from_m : Nat = foo\n\
                 const from_n : Nat = nfoo\n",
            ),
        ],
    )
    .expect("renaming the second distinct identity resolves the clash");
    assert_eq!(
        body_const(&renamed, "Entry.from_m"),
        renamed.globals["M.foo"]
    );
    assert_eq!(
        body_const(&renamed, "Entry.from_n"),
        renamed.globals["N.foo"]
    );
    assert_ne!(renamed.globals["M.foo"], renamed.globals["N.foo"]);

    match load_entry(
        "prelude-reject",
        &[
            ("M.ken", "pub def Bool = Nat\n"),
            ("Entry.ken", "import M (Bool)\n"),
        ],
    ) {
        Err(ElabError::AmbiguousReference { name, sources, .. }) => {
            assert_eq!(name, "Bool");
            assert!(sources.contains(&"M.Bool".to_string()));
            assert!(sources.contains(&"<prelude>.Bool".to_string()));
        }
        Err(other) => panic!("prelude clash rejected with the wrong error: {other}"),
        Ok(_) => panic!("a selective import may not shadow the prelude"),
    }

    let prelude_rename = load_entry(
        "prelude-rename",
        &[
            ("M.ken", "pub def Bool = Nat\n"),
            (
                "Entry.ken",
                "import M (Bool as MBool)\n\
                 const from_m : Type = MBool\n\
                 const from_prelude : Type = Bool\n",
            ),
        ],
    )
    .expect("renaming preserves both the user and prelude identities");
    assert_eq!(
        body_const(&prelude_rename, "Entry.from_m"),
        prelude_rename.globals["M.Bool"]
    );
    let (_, prelude_body) = prelude_rename
        .env
        .transparent_body(prelude_rename.globals["Entry.from_prelude"])
        .expect("prelude control is transparent");
    assert!(
        matches!(
            &prelude_body,
            Term::IndFormer { id, .. } if *id == prelude_rename.globals["Bool"]
        ),
        "bare Bool must remain the registered prelude type, got {prelude_body:?}"
    );
    assert_ne!(
        prelude_rename.globals["M.Bool"],
        prelude_rename.globals["Bool"]
    );
}

#[test]
fn loader_executes_same_identity_carveout_and_distinct_source_flip() {
    for (label, imports) in [
        ("same-mp", "import M (foo)\nimport P (foo)\n"),
        ("same-pm", "import P (foo)\nimport M (foo)\n"),
    ] {
        let entry = format!("{imports}const observed : Nat = foo\n");
        let env = load_entry(
            label,
            &[
                ("M.ken", "pub const foo : Nat = Zero\n"),
                ("P.ken", "export M (foo)\n"),
                ("Entry.ken", &entry),
            ],
        )
        .unwrap_or_else(|error| panic!("{label} must accept one canonical identity: {error}"));
        assert_eq!(body_const(&env, "Entry.observed"), env.globals["M.foo"]);
        assert!(!env.globals.contains_key("P.foo"));
    }

    for (label, imports) in [
        ("different-mp", "import M (foo)\nimport P (foo)\n"),
        ("different-pm", "import P (foo)\nimport M (foo)\n"),
    ] {
        let files = [
            ("M.ken", "pub const foo : Nat = Zero\n"),
            ("N.ken", "pub const foo : Nat = Suc Zero\n"),
            ("P.ken", "export N (foo)\n"),
            ("Entry.ken", imports),
        ];
        match load_entry(label, &files) {
            Err(ElabError::AmbiguousReference { name, sources, .. }) => {
                assert_eq!(name, "foo");
                assert!(sources.contains(&"M.foo".to_string()));
                assert!(sources.contains(&"N.foo".to_string()));
            }
            Err(other) => panic!("{label} rejected with the wrong error: {other}"),
            Ok(_) => panic!("{label} must reject the changed facade source"),
        }
    }
}

#[test]
fn loader_executes_facade_and_in_scope_body_scope_discriminator() {
    let facade = load_entry(
        "facade-publishes",
        &[
            ("M.ken", "pub const foo : Nat = Zero\n"),
            ("P.ken", "export M (foo as bar)\n"),
            ("Entry.ken", "import P (bar)\nconst observed : Nat = bar\n"),
        ],
    )
    .expect("facade export is a loader edge and publishes its renamed identity");
    assert_eq!(
        body_const(&facade, "Entry.observed"),
        facade.globals["M.foo"]
    );
    assert!(!facade.globals.contains_key("P.bar"));

    match load_entry(
        "facade-does-not-bind",
        &[
            ("M.ken", "pub const foo : Nat = Zero\n"),
            ("P.ken", "export M (foo)\nconst local_use : Nat = foo\n"),
            ("Entry.ken", "import P (foo)\n"),
        ],
    ) {
        Err(ElabError::UnboundName { name, .. }) => assert_eq!(name, "foo"),
        Err(other) => panic!("facade body lookup rejected with the wrong error: {other}"),
        Ok(_) => panic!("facade body lookup must reject as UnboundName(foo)"),
    }

    let in_scope = load_entry(
        "in-scope-binds",
        &[
            ("M.ken", "pub const foo : Nat = Zero\n"),
            (
                "P.ken",
                "import M (foo)\n\
                 export foo\n\
                 const local_use : Nat = foo\n",
            ),
            ("Entry.ken", "import P (foo)\nconst observed : Nat = foo\n"),
        ],
    )
    .expect("in-scope export republishes an existing body binding");
    assert_eq!(
        body_const(&in_scope, "P.local_use"),
        in_scope.globals["M.foo"]
    );
    assert_eq!(
        body_const(&in_scope, "Entry.observed"),
        in_scope.globals["M.foo"]
    );
}

#[test]
fn admitted_reexport_carries_only_the_named_instance_surface() {
    let root = FixtureRoot::new("instance-carry");
    root.write(
        "Q.ken",
        "pub class Render a { tag : Bool }\n\
         pub data Widget = MkWidget\n\
         instance Render Widget { tag = True }\n\
         pub class HiddenMark a { mark : Bool }\n\
         pub data Hidden = MkHidden\n\
         instance HiddenMark Hidden { mark = False }\n",
    );
    for (carried, direct) in [("Render", "Widget"), ("Widget", "Render")] {
        root.write(
            "P.ken",
            &format!("package admits Q\nexport Q ({carried})\n"),
        );
        root.write(
            "Entry.ken",
            &format!(
                "program admits P\n\
                 import P ({carried})\n\
                 import Q ({direct})\n\
                 fn use_widget (x : Widget) : Widget where Render Widget = x\n"
            ),
        );

        let mut accepted = ElabEnv::new().expect("base environment");
        accepted
            .elaborate_module_from_roots(&[root.path().to_path_buf()], "Entry")
            .unwrap_or_else(|error| {
                panic!("re-exporting key participant {carried} must carry Q's dictionary: {error}")
            });
        let resolution = accepted
            .class_env
            .resolution_provenance
            .last()
            .expect("implicit dispatch records provenance");
        assert_eq!(resolution.defining_package, "Q");
        assert_eq!(resolution.class_name, "Render");
        assert_eq!(resolution.head_type, "Q.Widget");
    }

    root.write("P.ken", "package admits Q\nexport Q (Render, Widget)\n");

    root.write(
        "Entry.ken",
        "program admits P\n\
         import P (Render, Widget)\n\
         import Q (HiddenMark, Hidden)\n\
         fn use_hidden (x : Hidden) : Hidden where HiddenMark Hidden = x\n",
    );
    let mut rejected = ElabEnv::new().expect("base environment");
    match rejected.elaborate_module_from_roots(&[root.path().to_path_buf()], "Entry") {
        Err(ElabError::UnadmittedInstance {
            defining_package,
            class,
            head_type,
            ..
        }) => {
            assert_eq!(defining_package, "Q");
            assert_eq!(class, "HiddenMark");
            assert_eq!(head_type, "Q.Hidden");
        }
        other => panic!("non-reexported transitive instance must reject, got {other:?}"),
    }
}

#[test]
fn abstract_reexport_preserves_hidden_constructor_boundary() {
    let mut visible_type = ElabEnv::new().expect("base environment");
    visible_type
        .elaborate_file(
            "module M { pub data Token = MkToken } \
             module P { export M (Token) } \
             import P (Token) fn keep (x : Token) : Token = x",
        )
        .expect("the original abstract type identity remains usable");

    let mut hidden_ctor = ElabEnv::new().expect("base environment");
    match hidden_ctor.elaborate_file(
        "module M { pub data Token = MkToken } \
         module P { export M (Token) } \
         import P (Token) \
         fn open (x : Token) : Nat = match x { MkToken ↦ Zero }",
    ) {
        Err(ElabError::UnresolvedCon { name, .. }) => assert_eq!(name, "MkToken"),
        other => panic!("re-export must not widen hidden constructors, got {other:?}"),
    }
}

#[test]
fn formatter_is_fixed_point_and_export_adds_no_trust() {
    assert!(CONFORMANCE_SEED.contains("facade-reexport-preserves-global-id"));
    let source = "export M (foo, Bar as baz)\n\nexport foo, Bar as baz\n";
    let formatted = format_ken(source).expect("export forms format");
    assert_eq!(
        format_ken(&formatted).expect("formatted export reparses"),
        formatted
    );
    let reparsed = parse_decls(&formatted).expect("formatted export AST parses");
    assert!(matches!(
        &reparsed[0],
        Decl::ExportDecl {
            form: ExportForm::Facade { .. },
            ..
        }
    ));
    assert!(matches!(
        &reparsed[1],
        Decl::ExportDecl {
            form: ExportForm::InScope { .. },
            ..
        }
    ));

    let mut env = ElabEnv::new().expect("base environment");
    let before = env.env.trusted_base();
    env.elaborate_file("module M { pub const foo : Nat = Zero } module P { export M (foo) }")
        .expect("re-export elaborates away");
    assert_eq!(env.env.trusted_base(), before);
    assert!(!env.globals.contains_key("P.foo"));
}
