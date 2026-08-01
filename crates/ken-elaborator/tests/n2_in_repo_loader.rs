//! N2 Lane-B acceptance: lazy in-repo cross-file loading (`33 §3.2`).

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use ken_elaborator::{ElabEnv, ElabError};
use ken_kernel::Term;

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

struct FixtureRoot(PathBuf);

impl FixtureRoot {
    fn new(label: &str) -> Self {
        let serial = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("ken-n2-{label}-{}-{serial}", std::process::id()));
        fs::create_dir_all(&path).expect("create N2 fixture root");
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
        fs::write(path, source).expect("write N2 fixture");
    }
}

impl Drop for FixtureRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn ken_md(source: &str) -> String {
    format!("# Fixture\n\n```ken\n{source}\n```\n")
}

#[test]
fn cross_file_import_resolves_lazily_through_plural_root_api_and_caches() {
    let root = FixtureRoot::new("accept");
    root.write(
        "A.ken.md",
        &ken_md("import B\n\nconst answer : Bool = B.value"),
    );
    root.write("B.ken.md", &ken_md("pub const value : Bool = True"));
    // An invalid, unimported unit proves discovery follows import edges rather
    // than eagerly scanning the root.
    root.write("C.ken.md", "```ken\nthis is not Ken\n```\n");

    let mut env = ElabEnv::new().expect("base environment");
    let roots = vec![root.path().to_path_buf()];
    let ids = env
        .elaborate_module_from_roots(&roots, "A")
        .expect("A imports and resolves B.value");

    assert_eq!(env.loaded_module_count(), 2, "only A and its B edge load");
    let answer = env.globals["A.answer"];
    let value = env.globals["B.value"];
    let (_, body) = env
        .env
        .transparent_body(answer)
        .expect("A.answer is transparent");
    assert!(
        matches!(body, Term::Const { id, .. } if id == value),
        "A.answer must resolve to the canonical B.value GlobalId"
    );

    let decl_count = env.env.decls().count();
    let cached = env
        .elaborate_module_from_roots(&roots, "A")
        .expect("second load reuses the cache");
    assert_eq!(
        cached, ids,
        "the cached unit returns the same declaration ids"
    );
    assert_eq!(
        env.env.decls().count(),
        decl_count,
        "cache prevents re-elaboration"
    );
    assert_eq!(env.loaded_module_count(), 2);
}

#[test]
fn import_cycle_rejects_with_entry_rooted_closed_payload_before_admission() {
    let root = FixtureRoot::new("cycle");
    root.write(
        "A.ken.md",
        &ken_md("import B\n\nconst answer : Bool = B.value"),
    );
    root.write(
        "B.ken.md",
        &ken_md("import A\n\npub const value : Bool = True"),
    );

    let mut env = ElabEnv::new().expect("base environment");
    let roots = vec![root.path().to_path_buf()];
    match env.elaborate_module_from_roots(&roots, "A") {
        Err(ElabError::ImportCycle { cycle, .. }) => {
            assert_eq!(cycle, vec!["A", "B", "A"]);
        }
        other => panic!("expected ImportCycle A -> B -> A, got {other:?}"),
    }
    assert_eq!(
        env.loaded_module_count(),
        0,
        "cyclic units never enter the cache"
    );
    assert!(!env.globals.contains_key("A.answer"));
    assert!(!env.globals.contains_key("B.value"));
}

#[test]
fn plural_api_fail_closed_until_multi_root_precedence_is_defined() {
    let root_a = FixtureRoot::new("root-a");
    let root_b = FixtureRoot::new("root-b");
    root_a.write("A.ken", "pub const value : Bool = True");

    for roots in [vec![], vec![root_a.0.clone(), root_b.0.clone()]] {
        let mut env = ElabEnv::new().expect("base environment");
        let err = env
            .elaborate_module_from_roots(&roots, "A")
            .expect_err("N2 accepts exactly one entry in the plural root input");
        assert!(matches!(err, ElabError::ParseError { .. }));
    }
}

#[test]
fn dotted_module_path_maps_to_its_unique_leaf_file() {
    let root = FixtureRoot::new("dotted");
    root.write(
        "Entry.ken",
        "import Data.Collections.Flag\nconst answer : Bool = Data.Collections.Flag.value",
    );
    root.write(
        "Data/Collections/Flag.ken",
        "pub const value : Bool = True",
    );

    let mut env = ElabEnv::new().expect("base environment");
    let roots = vec![root.path().to_path_buf()];
    env.elaborate_module_from_roots(&roots, "Entry")
        .expect("dotted path resolves through N-1 directories to one leaf");
    assert!(env.globals.contains_key("Data.Collections.Flag.value"));
    assert!(env.globals.contains_key("Entry.answer"));
}
