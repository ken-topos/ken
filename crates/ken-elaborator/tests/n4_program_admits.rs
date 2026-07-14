//! N4 Lane-B acceptance: anonymous admission boundaries and source-world gate.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use ken_elaborator::{parser, BoundaryKind, Decl, ElabEnv, ElabError};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

struct FixtureRoot(PathBuf);

impl FixtureRoot {
    fn new(label: &str) -> Self {
        let serial = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("ken-n4-{label}-{}-{serial}", std::process::id()));
        fs::create_dir_all(&path).expect("create N4 fixture root");
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
        fs::write(path, source).expect("write N4 fixture");
    }
}

impl Drop for FixtureRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn load(root: &FixtureRoot, entry: &str) -> Result<ElabEnv, ElabError> {
    let mut env = ElabEnv::new().expect("base environment");
    env.elaborate_module_from_roots(&[root.path().to_path_buf()], entry)?;
    Ok(env)
}

#[test]
fn anonymous_headers_parse_structurally_and_named_headers_reject_at_name() {
    let expected_admits = vec!["Core.Laws".to_string(), "Data.Map".to_string()];
    let program = parser::parse_decls("program admits Core.Laws, Data.Map").unwrap();
    assert!(matches!(
        program.as_slice(),
        [Decl::BoundaryDecl {
            kind: BoundaryKind::Program,
            admits,
            capabilities,
            ..
        }] if admits.as_ref() == Some(&expected_admits)
            && capabilities.is_none()
    ));

    let package = parser::parse_decls("package").unwrap();
    assert!(matches!(
        package.as_slice(),
        [Decl::BoundaryDecl {
            kind: BoundaryKind::Package,
            admits,
            capabilities,
            ..
        }] if admits.is_none() && capabilities.is_none()
    ));

    for (source, expected_start) in [("program App admits P", 8), ("package Lib", 8)] {
        match parser::parse_decls(source) {
            Err(ElabError::NamedBoundaryHeader { span, .. }) => {
                assert_eq!(
                    span.start, expected_start,
                    "reject points at the name token"
                );
            }
            other => panic!("named boundary must be a syntax reject, got {other:?}"),
        }
    }
}

fn write_two_provider_fixture(root: &FixtureRoot, admits: &str) {
    root.write(
        "P.ken",
        "class RenderP a { tag : Bool }\n\
         instance RenderP Int { tag = True }\n",
    );
    root.write(
        "Q.ken",
        "class RenderQ a { tag : Bool }\n\
         instance RenderQ Bool { tag = False }\n",
    );
    root.write(
        "Entry.ken",
        &format!(
            "program admits {admits}\n\
             import P\n\
             import Q\n\
             fn useP (x : Int) : Int where RenderP Int = x\n\
             fn useQ (x : Bool) : Bool where RenderQ Bool = x\n"
        ),
    );
}

#[test]
fn admitted_ambient_resolution_records_distinct_provider_provenance() {
    let root = FixtureRoot::new("admitted");
    write_two_provider_fixture(&root, "P, Q");

    let env = load(&root, "Entry").expect("both explicitly admitted providers resolve");
    let resolutions = &env.class_env.resolution_provenance;
    assert_eq!(resolutions.len(), 2);
    assert_eq!(resolutions[0].defining_package, "P");
    assert_eq!(resolutions[0].class_name, "RenderP");
    assert_eq!(resolutions[0].head_type, "Int");
    assert_eq!(resolutions[1].defining_package, "Q");
    assert_eq!(resolutions[1].class_name, "RenderQ");
    assert_eq!(resolutions[1].head_type, "Bool");
    assert_ne!(resolutions[0].instance_id, resolutions[1].instance_id);
}

#[test]
fn unadmitted_direct_dispatch_rejects_after_selection_and_one_line_admit_flips() {
    let root = FixtureRoot::new("unadmitted");
    write_two_provider_fixture(&root, "P");

    let mut env = ElabEnv::new().expect("base environment");
    match env.elaborate_module_from_roots(&[root.path().to_path_buf()], "Entry") {
        Err(ElabError::UnadmittedInstance {
            defining_package,
            class,
            head_type,
            instance_id,
            ..
        }) => {
            assert_eq!(defining_package, "Q");
            assert_eq!(class, "RenderQ");
            assert_eq!(head_type, "Bool");
            assert_eq!(
                env.class_env.instances[&("RenderQ".to_string(), "Bool".to_string())].instance_id,
                instance_id,
                "the diagnostic names the canonical instance selected by real search"
            );
        }
        other => panic!("expected structured UnadmittedInstance for Q, got {other:?}"),
    }

    let control = FixtureRoot::new("admitted-control");
    write_two_provider_fixture(&control, "P, Q");
    load(&control, "Entry").expect("adding only Q to admits flips the verdict");
}

#[test]
fn multi_provider_direct_use_without_a_boundary_fails_closed() {
    let root = FixtureRoot::new("missing-program");
    write_two_provider_fixture(&root, "P, Q");
    root.write(
        "Entry.ken",
        "import P\n\
         import Q\n\
         fn useP (x : Int) : Int where RenderP Int = x\n",
    );

    let mut env = ElabEnv::new().expect("base environment");
    match env.elaborate_module_from_roots(&[root.path().to_path_buf()], "Entry") {
        Err(ElabError::UnadmittedInstance {
            defining_package,
            class,
            head_type,
            ..
        }) => assert_eq!(
            (defining_package.as_str(), class.as_str(), head_type.as_str()),
            ("P", "RenderP", "Int")
        ),
        other => panic!("multi-provider direct use needs an explicit boundary, got {other:?}"),
    }
}

#[test]
fn package_dependency_is_transitive_for_coherence_but_not_parent_direct_use() {
    let root = FixtureRoot::new("transitive");
    root.write(
        "Q.ken",
        "class QMark a { tag : Bool }\n\
         instance QMark Int { tag = True }\n",
    );
    root.write(
        "P.ken",
        "package admits Q\n\
         import Q\n\
         fn insideP (x : Int) : Int where QMark Int = x\n",
    );
    root.write(
        "Entry.ken",
        "program admits P\n\
         import P\n\
         import Q\n\
         fn directQ (x : Int) : Int where QMark Int = x\n",
    );

    let mut env = ElabEnv::new().expect("base environment");
    match env.elaborate_module_from_roots(&[root.path().to_path_buf()], "Entry") {
        Err(ElabError::UnadmittedInstance {
            defining_package,
            class,
            head_type,
            ..
        }) => {
            assert_eq!(
                (
                    defining_package.as_str(),
                    class.as_str(),
                    head_type.as_str()
                ),
                ("Q", "QMark", "Int")
            );
            assert!(
                env.globals.contains_key("P.insideP"),
                "P's own declared Q dependency resolved before the parent direct-use reject"
            );
            assert!(
                env.class_env
                    .instances
                    .contains_key(&("QMark".to_string(), "Int".to_string())),
                "Q remains present in the unfiltered source coherence registry"
            );
        }
        other => panic!("expected parent's direct Q dispatch to reject, got {other:?}"),
    }
}

#[test]
fn single_source_package_self_admits_without_program() {
    let root = FixtureRoot::new("self-admit");
    root.write(
        "Solo.ken",
        "class SoloMark a { tag : Bool }\n\
         data SoloItem = MkSoloItem\n\
         instance SoloMark SoloItem { tag = True }\n\
         fn useSolo (x : SoloItem) : SoloItem where SoloMark SoloItem = x\n",
    );

    let env = load(&root, "Solo").expect("one source package self-admits");
    let provenance = env
        .class_env
        .resolution_provenance
        .last()
        .expect("real search records provenance");
    assert_eq!(provenance.defining_package, "Solo");
}

#[test]
fn sole_external_source_provider_self_admits_without_boundary() {
    let root = FixtureRoot::new("sole-external-provider");
    root.write(
        "P.ken",
        "class Mark a { tag : Bool }\n\
         instance Mark Int { tag = True }\n",
    );
    root.write(
        "Entry.ken",
        "import P\n\
         fn useP (x : Int) : Int where Mark Int = x\n",
    );

    let env = load(&root, "Entry")
        .expect("a boundary-less closure self-admits its sole source provider");
    let provenance = env
        .class_env
        .resolution_provenance
        .last()
        .expect("real search records the external provider");
    assert_eq!(provenance.defining_package, "P");
    assert_eq!(provenance.class_name, "Mark");
    assert_eq!(provenance.head_type, "Int");
}

#[test]
fn admission_does_not_replace_overlap_or_orphan_gates() {
    let overlap = FixtureRoot::new("overlap");
    overlap.write(
        "P.ken",
        "package\n\
         class Render a { tag : Bool }\n\
         data PItem = MkPItem\n\
         instance Render PItem { tag = True }\n\
         instance Render PItem { tag = False }\n",
    );
    let mut overlap_env = ElabEnv::new().expect("base environment");
    match overlap_env.elaborate_module_from_roots(&[overlap.path().to_path_buf()], "P") {
        Err(ElabError::OverlappingInstances {
            first_span,
            second_span,
            ..
        }) => assert_ne!(
            first_span, second_span,
            "both declaration spans are retained"
        ),
        other => panic!("same-package duplicate must reach OverlappingInstances, got {other:?}"),
    }

    let orphan = FixtureRoot::new("orphan");
    orphan.write("Class.ken", "class Render a { tag : Bool }\n");
    orphan.write("Head.ken", "data RItem = MkRItem\n");
    orphan.write(
        "Bad.ken",
        "import Class\n\
         import Head\n\
         instance Render RItem { tag = True }\n",
    );
    orphan.write(
        "Entry.ken",
        "program admits Bad\n\
         import Bad\n",
    );
    let mut orphan_env = ElabEnv::new().expect("base environment");
    match orphan_env.elaborate_module_from_roots(&[orphan.path().to_path_buf()], "Entry") {
        Err(ElabError::OrphanInstance {
            class, head_type, ..
        }) => assert_eq!((class.as_str(), head_type.as_str()), ("Render", "RItem")),
        other => panic!("admitted orphan must still reject at declaration, got {other:?}"),
    }
}
