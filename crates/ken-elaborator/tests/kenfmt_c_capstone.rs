//! WP C — executable soundness gate over the frozen reformat corpus.

use std::fs;
use std::path::{Path, PathBuf};

use ken_elaborator::layout::format_ken;
use ken_elaborator::lossless::parse_lossless;
use ken_elaborator::{extract_ken_md, format_ken_md, ElabEnv};

#[derive(Debug, PartialEq, Eq)]
enum ElabOutcome {
    Success {
        declarations: usize,
        trusted: String,
    },
    Failure {
        variant: String,
    },
}

#[test]
fn actual_frozen_reformat_preserves_ast_elaboration_and_literate_non_body_bytes() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut literate = Vec::new();
    collect(&repository.join("catalog"), ".ken.md", &mut literate);
    let mut plain = Vec::new();
    collect(&repository.join("examples/rosetta"), ".ken", &mut plain);
    plain.push(
        repository.join("catalog/packages/Capability/Verify/ProofErasureBoundaryChecker.ken"),
    );
    literate.sort();
    plain.sort();
    assert_eq!(literate.len(), 14);
    assert_eq!(plain.len(), 17);

    for path in plain {
        let source = fs::read_to_string(&path).unwrap();
        let formatted =
            format_ken(&source).unwrap_or_else(|error| panic!("{}: {error:?}", path.display()));
        assert_eq!(
            ast_shape(&source),
            ast_shape(&formatted),
            "{}",
            path.display()
        );
        assert_eq!(
            elaborate(&source, false),
            elaborate(&formatted, false),
            "{}: elaboration outcome drift",
            path.display()
        );
        assert_eq!(
            format_ken(&formatted).unwrap(),
            formatted,
            "{}",
            path.display()
        );
    }

    for path in literate {
        let source = fs::read_to_string(&path).unwrap();
        let formatted =
            format_ken_md(&source).unwrap_or_else(|error| panic!("{}: {error:?}", path.display()));
        assert_eq!(
            non_body_bytes(&source),
            non_body_bytes(&formatted),
            "{}",
            path.display()
        );
        let before = extract_ken_md(&source).unwrap();
        let after = extract_ken_md(&formatted).unwrap();
        assert_eq!(
            before.fences.len(),
            after.fences.len(),
            "{}",
            path.display()
        );
        for (left, right) in before.fences.iter().zip(&after.fences) {
            assert_eq!(left.role, right.role, "{}", path.display());
            let left_body = &source[left.body_range.clone()];
            let right_body = &formatted[right.body_range.clone()];
            if parse_lossless(left_body).is_ok() {
                assert_eq!(
                    ast_shape(left_body),
                    ast_shape(right_body),
                    "{}",
                    path.display()
                );
            }
        }
        assert_eq!(
            elaborate(&source, true),
            elaborate(&formatted, true),
            "{}: elaboration outcome drift",
            path.display()
        );
        assert_eq!(
            format_ken_md(&formatted).unwrap(),
            formatted,
            "{}",
            path.display()
        );
    }
}

fn elaborate(source: &str, literate: bool) -> ElabOutcome {
    let mut environment = ElabEnv::new().unwrap();
    let result = if literate {
        environment.elaborate_ken_md_file(source)
    } else {
        environment.elaborate_file(source)
    };
    match result {
        Ok(ids) => ElabOutcome::Success {
            declarations: ids.len(),
            trusted: format!("{:?}", environment.env.trusted_base()),
        },
        Err(error) => ElabOutcome::Failure {
            variant: format!("{:?}", std::mem::discriminant(&error)),
        },
    }
}

fn ast_shape(source: &str) -> String {
    let parsed = parse_lossless(source).expect("source must parse");
    erase_debug_spans(format!("{:?}", parsed.typed_decls()))
}

fn erase_debug_spans(mut debug: String) -> String {
    const PREFIX: &str = "Span { start: ";
    while let Some(start) = debug.find(PREFIX) {
        let Some(relative_end) = debug[start..].find(" }") else {
            break;
        };
        debug.replace_range(start..start + relative_end + 2, "Span");
    }
    debug
}

fn non_body_bytes(source: &str) -> String {
    let extraction = extract_ken_md(source).unwrap();
    let mut out = String::new();
    let mut cursor = 0usize;
    for fence in extraction.fences {
        out.push_str(&source[cursor..fence.body_range.start]);
        cursor = fence.body_range.end;
    }
    out.push_str(&source[cursor..]);
    out
}

fn collect(directory: &Path, suffix: &str, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect(&path, suffix, out);
        } else if path.to_string_lossy().ends_with(suffix) {
            out.push(path);
        }
    }
}
