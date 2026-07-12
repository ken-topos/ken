//! WP B1 acceptance: lossless AST + token/trivia source representation.

use std::fs;
use std::path::{Path, PathBuf};

use ken_elaborator::lossless::{parse_lossless, CommentPlacement, FormattableSource, TriviaKind};

fn assert_round_trip(label: &str, source: &str) {
    let lossless = parse_lossless(source)
        .unwrap_or_else(|error| panic!("{label}: source must parse: {error:?}"));
    let rebuilt = lossless.reconstruct();
    assert_eq!(rebuilt.as_bytes(), source.as_bytes(), "{label}: byte drift");

    let reparsed = parse_lossless(&rebuilt)
        .unwrap_or_else(|error| panic!("{label}: rebuilt source must parse: {error:?}"));
    assert_eq!(
        format!("{:?}", lossless.typed_decls()),
        format!("{:?}", reparsed.typed_decls()),
        "{label}: reconstructed source changed the semantic AST"
    );

    let comment_count = lossless
        .trivia()
        .iter()
        .filter(|item| item.kind == TriviaKind::LineComment)
        .count();
    assert_eq!(
        lossless.comment_attachments().len(),
        comment_count,
        "{label}: every comment must have exactly one home"
    );
}

#[test]
fn leading_trailing_and_interstitial_comments_have_stable_unique_homes() {
    let source = "-- leading\n\
const a : Nat = Zero -- trailing\n\
const b : Nat = (\n\
  -- interstitial\n\
  Zero\n\
)\n";
    let lossless = parse_lossless(source).expect("focused ambiguity source must parse");
    assert_eq!(lossless.reconstruct(), source);

    let placements: Vec<_> = lossless
        .comment_attachments()
        .iter()
        .map(|attachment| {
            (
                &source[attachment.comment_span.start..attachment.comment_span.end],
                attachment.placement,
            )
        })
        .collect();
    assert_eq!(
        placements,
        vec![
            ("-- leading", CommentPlacement::Leading),
            ("-- trailing", CommentPlacement::Trailing),
            ("-- interstitial", CommentPlacement::Interstitial),
        ]
    );

    assert_eq!(
        lossless
            .comments_for_span(lossless.typed_decls()[0].span())
            .len(),
        1,
        "the own-line leading comment homes on the following declaration"
    );
    for attachment in lossless.comment_attachments() {
        assert_eq!(
            lossless.comments_for_span(&attachment.home_span).len(),
            1,
            "each focused comment has one span-keyed AST home"
        );
    }
}

#[test]
fn public_consumer_depends_only_on_the_formattable_source_interface() {
    fn consume(source: &dyn FormattableSource) -> (usize, usize, String) {
        (
            source.typed_decls().len(),
            source.comment_attachments().len(),
            source.reconstruct(),
        )
    }

    let source = "const identity (x : Nat) : Nat = x -- kept\n";
    let lossless = parse_lossless(source).expect("source must parse");
    assert_eq!(consume(lossless.as_ref()), (1, 1, source.to_owned()));
}

#[test]
fn whole_catalog_and_every_parseable_ken_fence_round_trip_byte_exactly() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("catalog");
    let mut files = Vec::new();
    collect_sources(&root, &mut files);
    files.sort();

    let mut plain_units = 0usize;
    let mut parseable_fences = 0usize;
    for path in files {
        let source =
            fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        let label = path.display().to_string();
        if path.extension().and_then(|extension| extension.to_str()) == Some("ken") {
            assert_round_trip(&label, &source);
            plain_units += 1;
            continue;
        }

        for (index, fence) in ken_fence_bodies(&source).into_iter().enumerate() {
            if parse_lossless(fence).is_ok() {
                assert_round_trip(&format!("{label} fence {index}"), fence);
                parseable_fences += 1;
            }
        }
    }

    assert!(plain_units > 0, "catalog harness found no .ken units");
    assert!(
        parseable_fences > 0,
        "catalog harness found no parseable Ken fences"
    );
}

fn collect_sources(directory: &Path, out: &mut Vec<PathBuf>) {
    for entry in
        fs::read_dir(directory).unwrap_or_else(|error| panic!("{}: {error}", directory.display()))
    {
        let path = entry
            .expect("catalog directory entry must be readable")
            .path();
        if path.is_dir() {
            collect_sources(&path, out);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("ken")
            || path.to_string_lossy().ends_with(".ken.md")
        {
            out.push(path);
        }
    }
}

fn ken_fence_bodies(source: &str) -> Vec<&str> {
    let mut bodies = Vec::new();
    let mut body_start = None;
    let mut offset = 0usize;

    for line in source.split_inclusive('\n') {
        let text = line.strip_suffix('\n').unwrap_or(line);
        if let Some(start) = body_start {
            if text == "```" {
                bodies.push(&source[start..offset]);
                body_start = None;
            }
        } else if matches!(
            text,
            "```ken" | "```ken ignore" | "```ken reject" | "```ken example"
        ) {
            body_start = Some(offset + line.len());
        }
        offset += line.len();
    }
    bodies
}
