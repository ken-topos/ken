//! I-4 §D: anonymous boundary-header parser and independent reader seams.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use ken_elaborator::lexer::{Lexer, Token};
use ken_elaborator::{
    parser, BoundaryHeader, BoundaryKind, CapabilityDecl, Decl, ElabEnv,
    ElabError,
};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

struct FixtureRoot(PathBuf);

impl FixtureRoot {
    fn new(label: &str) -> Self {
        let serial = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let label = format!("ken-i4d-{label}-{}-{serial}", std::process::id());
        let path = std::env::temp_dir().join(label);
        fs::create_dir_all(&path).expect("create I-4 §D fixture root");
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
        fs::write(path, source).expect("write fixture");
    }
}

impl Drop for FixtureRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn parsed_header(source: &str) -> BoundaryHeader {
    let decls = parser::parse_decls(source).expect("boundary source parses");
    match decls.as_slice() {
        [Decl::BoundaryDecl {
            kind,
            admits,
            capabilities,
            ..
        }] => BoundaryHeader {
            kind: *kind,
            admits: admits.clone(),
            capabilities: capabilities.clone(),
        },
        other => panic!("expected one boundary AST node, got {other:?}"),
    }
}

fn fs(authority: &str) -> Vec<CapabilityDecl> {
    vec![CapabilityDecl {
        family: "FS".to_string(),
        authority: authority.to_string(),
    }]
}

#[test]
fn boundary_keywords_are_reserved_and_four_program_shapes_are_independent() {
    let tokens = Lexer::lex("program package admits capabilities")
        .expect("boundary keywords lex")
        .into_iter()
        .map(|(token, _)| token)
        .collect::<Vec<_>>();
    assert_eq!(
        tokens,
        vec![
            Token::KwProgram,
            Token::KwPackage,
            Token::KwAdmits,
            Token::KwCapabilities,
            Token::Eof,
        ]
    );

    assert_eq!(
        parsed_header("program"),
        BoundaryHeader {
            kind: BoundaryKind::Program,
            admits: None,
            capabilities: None,
        }
    );
    assert_eq!(
        parsed_header("program admits Core.Laws, Data.Map"),
        BoundaryHeader {
            kind: BoundaryKind::Program,
            admits: Some(vec!["Core.Laws".to_string(), "Data.Map".to_string()]),
            capabilities: None,
        }
    );
    assert_eq!(
        parsed_header("program capabilities FS APartial"),
        BoundaryHeader {
            kind: BoundaryKind::Program,
            admits: None,
            capabilities: Some(fs("APartial")),
        }
    );
    assert_eq!(
        parsed_header("program admits P capabilities FS AFull"),
        BoundaryHeader {
            kind: BoundaryKind::Program,
            admits: Some(vec!["P".to_string()]),
            capabilities: Some(fs("AFull")),
        }
    );
}

#[test]
fn boundary_header_is_accepted_before_imports_and_rejected_after_them() {
    let decls = parser::parse_decls("program\nimport P")
        .expect("unit-head boundary precedes imports");
    assert!(matches!(decls.first(), Some(Decl::BoundaryDecl { .. })));
    assert!(matches!(decls.get(1), Some(Decl::ImportDecl { .. })));

    assert!(matches!(
        parser::parse_decls("import P\nprogram"),
        Err(ElabError::ParseError { msg, .. })
            if msg.contains("must be the first unit header")
    ));
    assert!(matches!(
        parser::parse_decls("fn keep (x : Int) : Int = x\npackage"),
        Err(ElabError::ParseError { msg, .. })
            if msg.contains("must be the first unit header")
    ));
}

#[test]
fn package_boundary_has_admission_only_and_capabilities_fail_closed() {
    assert_eq!(
        parsed_header("package"),
        BoundaryHeader {
            kind: BoundaryKind::Package,
            admits: None,
            capabilities: None,
        }
    );
    assert_eq!(
        parsed_header("package admits Core.Laws"),
        BoundaryHeader {
            kind: BoundaryKind::Package,
            admits: Some(vec!["Core.Laws".to_string()]),
            capabilities: None,
        }
    );
    assert!(matches!(
        parser::parse_decls("package capabilities FS AFull"),
        Err(ElabError::PackageCapabilitiesNotAllowed { .. })
    ));
    assert!(matches!(
        parser::parse_decls("package admits P capabilities FS AFull"),
        Err(ElabError::PackageCapabilitiesNotAllowed { .. })
    ));
}

#[test]
fn every_boundary_surface_failure_has_its_named_diagnostic() {
    assert!(matches!(
        parser::parse_decls("program App"),
        Err(ElabError::NamedBoundaryHeader { name, .. }) if name == "App"
    ));
    assert!(matches!(
        parser::parse_decls("program capabilities Net AFull"),
        Err(ElabError::UnknownCapabilityFamily { family, .. })
            if family == "Net"
    ));
    assert!(matches!(
        parser::parse_decls("program capabilities FS ARoot"),
        Err(ElabError::InvalidCapabilityAuthority {
            family,
            authority,
            ..
        }) if family == "FS" && authority == "ARoot"
    ));
    assert!(matches!(
        parser::parse_decls("program capabilities FS APartial, FS AFull"),
        Err(ElabError::DuplicateCapabilityFamily { family, .. })
            if family == "FS"
    ));

    // The invalid family is behind a valid admission clause: this cannot pass
    // merely because the parser stopped after its first recognised clause.
    assert!(matches!(
        parser::parse_decls("program admits P capabilities Net AFull"),
        Err(ElabError::UnknownCapabilityFamily { family, .. })
            if family == "Net"
    ));
}

fn write_provider(root: &FixtureRoot) {
    root.write(
        "P.ken",
        "class Mark a { tag : Bool }\n\
         instance Mark Int { tag = True }\n",
    );
}

fn write_entry(root: &FixtureRoot, admits: bool) {
    let admits_clause = if admits { "admits P\n" } else { "" };
    root.write(
        "Entry.ken",
        &format!(
            "program\n\
             {admits_clause}\
             capabilities FS AFull\n\
             import P\n\
             fn useP (x : Int) : Int where Mark Int = x\n"
        ),
    );
}

#[test]
fn loader_projects_both_readers_and_admits_clause_changes_real_dispatch() {
    let accepted = FixtureRoot::new("admitted");
    write_provider(&accepted);
    write_entry(&accepted, true);
    let mut accepted_env = ElabEnv::new().expect("base environment");
    accepted_env
        .elaborate_module_from_roots(&[accepted.path().to_path_buf()], "Entry")
        .expect("parsed admits reaches real instance dispatch");
    assert_eq!(
        accepted_env.boundary_header(),
        Some(&BoundaryHeader {
            kind: BoundaryKind::Program,
            admits: Some(vec!["P".to_string()]),
            capabilities: Some(fs("AFull")),
        })
    );
    assert_eq!(
        accepted_env
            .class_env
            .resolution_provenance
            .last()
            .expect("real dictionary search records provenance")
            .defining_package,
        "P"
    );

    let rejected = FixtureRoot::new("unadmitted");
    write_provider(&rejected);
    write_entry(&rejected, false);
    let mut rejected_env = ElabEnv::new().expect("base environment");
    assert!(matches!(
        rejected_env.elaborate_module_from_roots(
            &[rejected.path().to_path_buf()],
            "Entry"
        ),
        Err(ElabError::UnadmittedInstance {
            defining_package,
            class,
            head_type,
            ..
        }) if defining_package == "P" && class == "Mark" && head_type == "Int"
    ));
}

#[test]
fn direct_file_reader_preserves_manifest_without_minting_or_trust_growth() {
    let mut env = ElabEnv::new().expect("base environment");
    let before: BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    env.elaborate_file(
        "program capabilities FS ANone\n\
         fn keep (x : Int) : Int = x\n",
    )
    .expect("direct source file elaborates");
    assert_eq!(
        env.boundary_header(),
        Some(&BoundaryHeader {
            kind: BoundaryKind::Program,
            admits: None,
            capabilities: Some(fs("ANone")),
        })
    );
    let after: BTreeSet<_> = env.env.trusted_base().into_iter().collect();
    assert_eq!(before, after, "surface header adds no trusted declaration");
}
