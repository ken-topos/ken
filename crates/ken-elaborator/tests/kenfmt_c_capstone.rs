//! WP C — executable soundness gate over the frozen reformat corpus.

use std::fs;
use std::path::{Path, PathBuf};

use ken_elaborator::layout::format_ken;
use ken_elaborator::lossless::parse_lossless;
use ken_elaborator::{extract_ken_md, format_ken_md};

const FRAME_LINE_COUNTS: &[(&str, usize)] = &[
    ("catalog/guide/decomposition-abstraction.ken.md", 147),
    ("catalog/guide/proof-techniques.ken.md", 331),
    ("catalog/guide/surface-reference.ken.md", 528),
    ("catalog/packages/Capability/Parsing/Parsing.ken.md", 653),
    (
        "catalog/packages/Capability/Verify/ProofErasureBoundaryChecker.ken",
        33,
    ),
    ("catalog/packages/Core/EffectfulClasses.ken.md", 2743),
    ("catalog/packages/Core/EmptyDec.ken.md", 300),
    ("catalog/packages/Core/LawfulClasses.ken.md", 1717),
    ("catalog/packages/Core/LawfulFunctors.ken.md", 470),
    ("catalog/packages/Core/NatArith.ken.md", 245),
    ("catalog/packages/Core/OrdNat.ken.md", 313),
    ("catalog/packages/Core/Transport.ken.md", 177),
    ("catalog/packages/Data/Collections/Collections.ken.md", 1113),
    ("catalog/packages/Data/Collections/Map.ken.md", 6710),
    ("catalog/packages/Data/Sums/Sums.ken.md", 248),
    (
        "examples/rosetta/accumulator-factory/accumulator-factory.ken",
        94,
    ),
    ("examples/rosetta/ackermann/ackermann.ken", 96),
    ("examples/rosetta/closures/closures.ken", 68),
    ("examples/rosetta/factorial/factorial.ken", 102),
    ("examples/rosetta/fibonacci/fibonacci.ken", 108),
    ("examples/rosetta/fizzbuzz/fizzbuzz.ken", 160),
    ("examples/rosetta/gcd/gcd.ken", 154),
    ("examples/rosetta/hailstone/hailstone.ken", 108),
    ("examples/rosetta/hello-world/hello-world.ken", 9),
    ("examples/rosetta/letter-frequency/letter-frequency.ken", 97),
    ("examples/rosetta/merge-sort/merge-sort.ken", 100),
    ("examples/rosetta/mutual-recursion/mutual-recursion.ken", 25),
    ("examples/rosetta/palindrome/palindrome.ken", 53),
    ("examples/rosetta/read-file-lines/read-file-lines.ken", 122),
    ("examples/rosetta/rpn-calculator/rpn-calculator.ken", 110),
    ("examples/rosetta/tree-traversal/tree-traversal.ken", 34),
];

#[test]
fn canonical_frozen_corpus_is_a_31_file_fixed_point() {
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
    assert!(
        literate.len() >= 14,
        "literate corpus fell below floor 14 (observed {})",
        literate.len()
    );
    assert!(
        plain.len() >= 17,
        "plain corpus fell below floor 17 (observed {})",
        plain.len()
    );

    for path in plain {
        let source = fs::read_to_string(&path).unwrap();
        let formatted =
            format_ken(&source).unwrap_or_else(|error| panic!("{}: {error:?}", path.display()));
        assert_eq!(formatted, source, "{}", path.display());
        assert_no_zero_indent_continuation(&path.display().to_string(), &source);
    }

    for path in literate {
        let source = fs::read_to_string(&path).unwrap();
        let formatted =
            format_ken_md(&source).unwrap_or_else(|error| panic!("{}: {error:?}", path.display()));
        assert_eq!(formatted, source, "{}", path.display());
        let extraction = extract_ken_md(&source).unwrap();
        for (index, fence) in extraction.fences.iter().enumerate() {
            let body = &source[fence.body_range.clone()];
            if parse_lossless(body).is_ok() {
                assert_no_zero_indent_continuation(
                    &format!("{} fence {index}", path.display()),
                    body,
                );
            }
        }
    }
}

#[test]
fn canonical_reformat_has_no_pathological_line_expansion() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut frame_total = 0usize;
    let mut canonical_total = 0usize;
    for &(path, frame_lines) in FRAME_LINE_COUNTS {
        let canonical_lines = fs::read_to_string(repository.join(path))
            .unwrap()
            .lines()
            .count();
        frame_total += frame_lines;
        canonical_total += canonical_lines;
        assert!(
            canonical_lines * 2 <= frame_lines * 9,
            "{path}: pathological expansion from {frame_lines} to {canonical_lines} lines"
        );
    }
    assert_eq!(frame_total, 17_168, "frame line-count oracle drifted");
    assert!(
        canonical_total <= frame_total * 3,
        "whole corpus pathologically expanded from {frame_total} to {canonical_total} lines"
    );
}

fn assert_no_zero_indent_continuation(label: &str, source: &str) {
    const TOP_LEVEL_PREFIXES: &[&str] = &[
        "program", "package", "view", "const", "fn", "proc", "space", "prove", "prop", "lemma",
        "proof", "law", "data", "def", "foreign", "temporal", "class", "instance", "derive",
        "module", "import", "use", "pub", "--", "}",
    ];
    for (index, line) in source.lines().enumerate() {
        if line.is_empty() || line.starts_with(' ') {
            continue;
        }
        assert!(
            TOP_LEVEL_PREFIXES.iter().any(|prefix| {
                line == *prefix
                    || line
                        .strip_prefix(prefix)
                        .is_some_and(|rest| rest.starts_with(char::is_whitespace))
            }),
            "{label}: required continuation at column zero on line {}: {line}",
            index + 1
        );
    }
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
