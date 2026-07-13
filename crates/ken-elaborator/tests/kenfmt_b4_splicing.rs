//! WP B4 — literate `.ken.md` canonicalization and byte-exact splicing.

use std::fs;
use std::path::{Path, PathBuf};

use ken_elaborator::error::ElabError;
use ken_elaborator::format::canonicalize_lexed_tokens;
use ken_elaborator::lossless::parse_lossless;
use ken_elaborator::{extract_ken_md, format_ken_md, KenMdFenceRole};

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
    let extraction = extract_ken_md(source).expect("formatted Markdown must extract");
    let mut out = String::new();
    let mut cursor = 0usize;
    for fence in &extraction.fences {
        out.push_str(&source[cursor..fence.body_range.start]);
        cursor = fence.body_range.end;
    }
    out.push_str(&source[cursor..]);
    out
}

#[test]
fn ac1_extractor_exposes_all_four_roles_and_marker_spans_additively() {
    let markdown = "Prose before\n\
```ken\nconst source : Nat = Zero\n```\n\
```ken ignore\nconst ignored : Nat = Zero\n```\n\
```text\nconst other : Nat = Zero\n```\n\
```ken reject\nconst rejected : Nat = missing\n```\n\
```ken example\nconst example : Nat = Zero\n```\n\
Prose after\n";
    let extraction = extract_ken_md(markdown).expect("all canonical roles extract");

    assert_eq!(
        extraction
            .fences
            .iter()
            .map(|fence| fence.role)
            .collect::<Vec<_>>(),
        vec![
            KenMdFenceRole::Source,
            KenMdFenceRole::Ignore,
            KenMdFenceRole::Reject,
            KenMdFenceRole::Example,
        ]
    );
    assert_eq!(extraction.compiled_ranges.len(), 1);
    assert_eq!(extraction.reject_ranges.len(), 1);
    assert_eq!(extraction.example_ranges.len(), 1);
    assert!(!extraction.source.contains("ignored"));

    for fence in &extraction.fences {
        let opener = &markdown[fence.opener_span.start..fence.opener_span.end];
        assert!(opener.starts_with("```ken"));
        assert_eq!(
            &markdown[fence.closer_span.start..fence.closer_span.end],
            "```"
        );
        assert!(markdown.is_char_boundary(fence.body_range.start));
        assert!(markdown.is_char_boundary(fence.body_range.end));
    }
}

#[test]
fn ac2_parse_first_role_gate_has_both_fallback_and_hard_error_orientations() {
    let parseable = "fn id (x : Nat) : Nat = (x)\n";
    for role in ["ken", "ken ignore", "ken reject", "ken example"] {
        let document = format!("```{role}\n{parseable}```\n");
        let formatted = format_ken_md(&document).expect("parseable bodies fully format");
        assert!(formatted.contains("fn id (x : Nat) : Nat = (x)\n"));
    }

    let ignore = "```ken ignore\nfn unfinished (f : Nat -> Nat) : Nat =\n```\n";
    let formatted_ignore = format_ken_md(ignore).expect("incomplete ignore falls back");
    assert_eq!(
        formatted_ignore,
        "```ken ignore\nfn unfinished (f : Nat → Nat) : Nat =\n```\n"
    );

    let reject = "```ken reject\nconst broken : Nat = 0 <= -- keep -> in comment\n```\n";
    let formatted_reject = format_ken_md(reject).expect("syntax reject falls back");
    assert_eq!(
        formatted_reject,
        "```ken reject\nconst broken : Nat = 0 ≤ -- keep -> in comment\n```\n"
    );
    assert_eq!(
        canonicalize_lexed_tokens("temporal T { x -> y }\n\"keep ->\" @ ->")
            .expect("recovering token canon"),
        "temporal T { x -> y }\n\"keep ->\" @ →"
    );

    for role in ["ken", "ken example"] {
        let document = format!("```{role}\nconst broken : Nat =\n```\n");
        match format_ken_md(&document).expect_err("tangled/checked body must parse") {
            ElabError::ParseError { msg, span } => {
                assert!(msg.contains(&format!("`{role}` fence body")), "{msg}");
                assert!(span.start >= document.find("const broken").unwrap());
            }
            other => panic!("expected role-specific ParseError, got {other:?}"),
        }
    }
}

#[test]
fn ac3_descending_splices_prevent_multi_fence_offset_drift() {
    let source = "Before\n\
```ken\nfn choose (x : Bool) : Bool = match x { True |-> True; False |-> False }\n```\n\
Middle\n\
```ken ignore\nfn incomplete (f : Nat -> Nat) : Nat =\n```\n\
After\n";
    let formatted = format_ken_md(source).expect("multi-fence document formats");
    let extraction = extract_ken_md(&formatted).expect("formatted document re-extracts");

    assert_eq!(extraction.fences.len(), 2);
    assert_eq!(extraction.fences[0].role, KenMdFenceRole::Source);
    assert_eq!(extraction.fences[1].role, KenMdFenceRole::Ignore);
    assert!(formatted[extraction.fences[0].body_range.clone()].contains("True ↦ True"));
    assert!(formatted[extraction.fences[1].body_range.clone()].contains("Nat → Nat"));
    assert!(formatted.ends_with("```\nAfter\n"));
}

#[test]
fn ac4_prose_markers_and_non_ken_fences_are_byte_identical() {
    let source =
        "A long prose paragraph with ->, |->, and ```ken-looking text that is not a marker.\n\n\n  ```ken\nindented fence-looking prose stays prose\n  ```\n\
```text\nfn untouched (x : Nat) : Nat = x -> x\n```\n\n\
Between fences, preserve  every   byte.\n\
```ken\nfn choose (x : Bool) : Bool = match x { True |-> True; False |-> False }\n```\n\n\n\
```ken ignore\nfn unfinished (f : Nat -> Nat) : Nat =\n```\n\
Tail.\n";
    let formatted = format_ken_md(source).expect("adversarial Markdown formats");

    assert_eq!(non_body_bytes(&formatted), non_body_bytes(source));
    assert!(formatted.contains("  ```ken\nindented fence-looking prose stays prose"));
    assert!(formatted.contains("```text\nfn untouched (x : Nat) : Nat = x -> x\n```"));
}

#[test]
fn ac5_full_document_is_byte_idempotent_and_canonical_input_is_a_noop() {
    let source = "Heading\n```ken\nconst answer : Nat = (Zero)\n```\n";
    let once = format_ken_md(source).expect("formats once");
    assert_eq!(format_ken_md(&once).expect("formats twice"), once);

    let canonical = "Heading\n```ken\nconst answer : Nat = Zero\n```\n";
    assert_eq!(format_ken_md(canonical).unwrap(), canonical);
}

#[test]
fn ac5_empty_fences_are_first_pass_fixed_points_in_all_roles() {
    let source = "```ken\n```\n```ken ignore\n```\n```ken reject\n```\n```ken example\n```\n";
    assert_eq!(format_ken_md(source).unwrap(), source);
}

#[test]
fn ac6_whole_literate_corpus_preserves_prose_ast_and_idempotence_read_only() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut paths = Vec::new();
    for root in ["catalog", "examples/rosetta", "spec"] {
        collect_literate_sources(&repository.join(root), &mut paths);
    }
    paths.sort();

    let mut documents = 0usize;
    let mut parseable_bodies = 0usize;
    for path in paths {
        let source = fs::read_to_string(&path).unwrap();
        let formatted =
            format_ken_md(&source).unwrap_or_else(|error| panic!("{}: {error:?}", path.display()));
        assert_eq!(
            non_body_bytes(&formatted),
            non_body_bytes(&source),
            "{}: prose or marker drift",
            path.display()
        );
        assert_eq!(
            format_ken_md(&formatted).unwrap(),
            formatted,
            "{}: not byte-idempotent",
            path.display()
        );

        let before = extract_ken_md(&source).unwrap();
        let after = extract_ken_md(&formatted).unwrap();
        assert_eq!(before.fences.len(), after.fences.len());
        assert_eq!(
            before
                .fences
                .iter()
                .map(|fence| fence.role)
                .collect::<Vec<_>>(),
            after
                .fences
                .iter()
                .map(|fence| fence.role)
                .collect::<Vec<_>>()
        );
        for (original, canonical) in before.fences.iter().zip(&after.fences) {
            let original_body = &source[original.body_range.clone()];
            if parse_lossless(original_body).is_ok() {
                let canonical_body = &formatted[canonical.body_range.clone()];
                assert_eq!(
                    ast_shape(original_body),
                    ast_shape(canonical_body),
                    "{}: body AST drift",
                    path.display()
                );
                parseable_bodies += 1;
            }
        }
        documents += 1;
    }

    assert!(
        documents > 0,
        "whole-corpus gate found no literate documents"
    );
    assert!(
        parseable_bodies > 0,
        "whole-corpus gate found no parseable fence bodies"
    );
}

fn collect_literate_sources(directory: &Path, out: &mut Vec<PathBuf>) {
    if !directory.exists() {
        return;
    }
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect_literate_sources(&path, out);
        } else if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".ken.md"))
        {
            out.push(path);
        }
    }
}
