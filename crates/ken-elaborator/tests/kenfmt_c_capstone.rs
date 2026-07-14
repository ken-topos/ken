//! WP C — executable soundness gate over the frozen reformat corpus.

use std::fs;
use std::path::{Path, PathBuf};

use ken_elaborator::layout::{display_width, format_ken, CANONICAL_WIDTH, INDENT_WIDTH};
use ken_elaborator::lexer::Token;
use ken_elaborator::lossless::parse_lossless;
use ken_elaborator::{extract_ken_md, format_ken_md};

// Historical, discharged capstone-C migration baseline. New files must not be
// added here: a file created after the frame has no honest pre-capstone line
// count. Live-corpus canonicity is covered by the fixed-point test below.
const FRAME_LINE_COUNTS: &[(&str, usize)] = &[
    ("catalog/guide/decomposition-abstraction.ken.md", 165),
    ("catalog/guide/proof-techniques.ken.md", 367),
    ("catalog/guide/surface-reference.ken.md", 535),
    ("catalog/packages/Capability/Console/Console.ken.md", 32),
    ("catalog/packages/Capability/FS/FS.ken.md", 36),
    ("catalog/packages/Capability/Parsing/Parsing.ken.md", 742),
    (
        "catalog/packages/Capability/Verify/ProofErasureBoundaryChecker.ken",
        44,
    ),
    ("catalog/packages/Core/EffectfulClasses.ken.md", 6714),
    ("catalog/packages/Core/EmptyDec.ken.md", 323),
    ("catalog/packages/Core/LawfulClasses.ken.md", 2217),
    ("catalog/packages/Core/LawfulFunctors.ken.md", 512),
    ("catalog/packages/Core/NatArith.ken.md", 240),
    ("catalog/packages/Core/OrdNat.ken.md", 356),
    ("catalog/packages/Core/Transport.ken.md", 179),
    ("catalog/packages/Data/Collections/Collections.ken.md", 1446),
    ("catalog/packages/Data/Collections/Map.ken.md", 15352),
    (
        "catalog/packages/Data/Collections/StringBijection.ken.md",
        52,
    ),
    ("catalog/packages/Data/NonEmpty/NonEmpty.ken.md", 170),
    ("catalog/packages/Data/Sums/Sums.ken.md", 321),
    ("catalog/packages/Data/Validation/Validation.ken.md", 367),
    ("catalog/packages/Text/Codec/Codec.ken.md", 99),
    ("catalog/packages/Text/Numeric/Numeric.ken.md", 248),
    ("catalog/packages/Text/StringKeys/StringKeys.ken.md", 112),
    (
        "examples/rosetta/accumulator-factory/accumulator-factory.ken",
        132,
    ),
    ("examples/rosetta/ackermann/ackermann.ken", 143),
    ("examples/rosetta/closures/closures.ken", 66),
    ("examples/rosetta/factorial/factorial.ken", 158),
    ("examples/rosetta/fibonacci/fibonacci.ken", 166),
    ("examples/rosetta/fizzbuzz/fizzbuzz.ken", 258),
    ("examples/rosetta/gcd/gcd.ken", 218),
    ("examples/rosetta/hailstone/hailstone.ken", 167),
    ("examples/rosetta/hello-world/hello-world.ken", 10),
    (
        "examples/rosetta/letter-frequency/letter-frequency.ken",
        114,
    ),
    ("examples/rosetta/merge-sort/merge-sort.ken", 111),
    ("examples/rosetta/mutual-recursion/mutual-recursion.ken", 38),
    ("examples/rosetta/palindrome/palindrome.ken", 56),
    ("examples/rosetta/read-file-lines/read-file-lines.ken", 166),
    ("examples/rosetta/rpn-calculator/rpn-calculator.ken", 125),
    ("examples/rosetta/tree-traversal/tree-traversal.ken", 37),
];

#[test]
fn canonical_frozen_corpus_is_a_39_file_fixed_point() {
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
    let mut enumerated = Vec::new();
    collect(&repository.join("catalog"), ".ken.md", &mut enumerated);
    collect(
        &repository.join("examples/rosetta"),
        ".ken",
        &mut enumerated,
    );
    enumerated.push(
        repository.join("catalog/packages/Capability/Verify/ProofErasureBoundaryChecker.ken"),
    );
    let mut enumerated = enumerated
        .into_iter()
        .map(|path| {
            path.strip_prefix(&repository)
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect::<Vec<_>>();
    let mut oracle = FRAME_LINE_COUNTS
        .iter()
        .map(|(path, _)| (*path).to_owned())
        .collect::<Vec<_>>();
    enumerated.sort();
    oracle.sort();
    let missing = oracle
        .iter()
        .filter(|path| enumerated.binary_search(path).is_err())
        .collect::<Vec<_>>();
    assert!(
        missing.is_empty(),
        "historical line-count oracle paths must remain in the live corpus: {missing:?}"
    );

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
    assert_eq!(frame_total, 32_594, "frame line-count oracle drifted");
    assert!(
        canonical_total <= frame_total * 3,
        "whole corpus pathologically expanded from {frame_total} to {canonical_total} lines"
    );
}

#[test]
fn balanced_corpus_rejects_the_known_over_width_splay_shape() {
    let flat_field = "fusion_law : (a : Type) → (b : Type) → (c : Type) → (g : b → c) → (h : a → b) → (x : f a) → Equal (f c) (map a c (comp a b c g h) x) (map b c g (map a b h x))";
    assert!(
        display_width(flat_field) > CANONICAL_WIDTH,
        "negative fixture must exercise the doesn't-fit path"
    );
    let pre_fix = "class Functor (f : Type → Type) {\n  fusion_law : (a : Type)\n  → (b : Type)\n  → (c : Type)\n  → (g : b\n  → c)\n  → (h : a\n  → b)\n  → (x : f\n  a)\n  → Equal (f c) (map a c (comp a b c g h) x) (map b c g (map a b h x))\n}\n";
    parse_lossless(pre_fix).expect("pre-fix splay fixture must remain valid Ken");
    let rejected = splay_violations(pre_fix);
    assert!(
        rejected
            .iter()
            .any(|violation| violation.contains("mid-arrow")),
        "detector did not reject the known over-width mid-arrow splay: {rejected:?}"
    );
    assert!(
        rejected
            .iter()
            .any(|violation| violation.contains("missing +2")),
        "detector did not reject the known low-level indentation splay: {rejected:?}"
    );

    let balanced = format_ken(&format!(
        "class Functor (f : Type → Type) {{ {flat_field} }}"
    ))
    .expect("balanced control must format");
    assert!(
        splay_violations(&balanced).is_empty(),
        "balanced control was rejected:\n{balanced}"
    );

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

    for path in plain {
        let source = fs::read_to_string(&path).unwrap();
        assert_no_splay(&path.display().to_string(), &source);
    }
    for path in literate {
        let source = fs::read_to_string(&path).unwrap();
        let extraction = extract_ken_md(&source).unwrap();
        for (index, fence) in extraction.fences.iter().enumerate() {
            let body = &source[fence.body_range.clone()];
            if parse_lossless(body).is_ok() {
                assert_no_splay(&format!("{} fence {index}", path.display()), body);
            }
        }
    }
}

fn assert_no_splay(label: &str, source: &str) {
    let violations = splay_violations(source);
    assert!(violations.is_empty(), "{label}: {}", violations.join("; "));
}

fn splay_violations(source: &str) -> Vec<String> {
    let parsed = parse_lossless(source).expect("balance detector requires parseable Ken");
    let tokens = parsed.tokens();
    let mut matching = vec![None; tokens.len()];
    let mut open: Vec<usize> = Vec::new();
    for (position, token) in tokens.iter().enumerate() {
        match token.kind {
            Token::LParen => open.push(position),
            Token::RParen => {
                if let Some(start) = open.pop() {
                    matching[start] = Some(position);
                }
            }
            _ => {}
        }
    }

    let mut line_starts = vec![0usize];
    for (offset, byte) in source.bytes().enumerate() {
        if byte == b'\n' && offset + 1 < source.len() {
            line_starts.push(offset + 1);
        }
    }
    let line_indent = |line: usize| {
        source[line_starts[line]..]
            .chars()
            .take_while(|ch| matches!(ch, ' ' | '\t'))
            .map(|ch| if ch == '\t' { INDENT_WIDTH } else { 1 })
            .sum::<usize>()
    };

    let mut violations = Vec::new();
    let mut parens: Vec<usize> = Vec::new();
    for (position, token) in tokens.iter().enumerate() {
        let line = line_starts
            .partition_point(|start| *start <= token.span.start)
            .saturating_sub(1);
        let prefix = &source[line_starts[line]..token.span.start];
        let starts_line = prefix.chars().all(char::is_whitespace);
        let line_end = line_starts.get(line + 1).copied().unwrap_or(source.len());
        let closes_only = source[line_starts[line]..line_end]
            .trim()
            .chars()
            .all(|ch| matches!(ch, ')' | ']' | '}' | ',' | ';'));

        if starts_line {
            if let Some(&start) = parens.last() {
                let opener = &tokens[start];
                let open_line = line_starts
                    .partition_point(|line_start| *line_start <= opener.span.start)
                    .saturating_sub(1);
                let open_indent = line_indent(open_line);
                if line > open_line
                    && !matches!(token.kind, Token::RParen)
                    && !closes_only
                    && line_indent(line) < open_indent + INDENT_WIDTH
                {
                    violations.push(format!(
                        "line {}: missing +2 continuation indentation inside parenthesized group",
                        line + 1
                    ));
                }
                if line > open_line && matches!(token.kind, Token::Arrow) {
                    if let Some(close) = matching[start] {
                        let flat = source[opener.span.start..tokens[close].span.end]
                            .split_whitespace()
                            .collect::<Vec<_>>()
                            .join(" ");
                        if display_width(&flat) <= CANONICAL_WIDTH.saturating_sub(open_indent) {
                            violations.push(format!(
                                "line {}: mid-arrow break inside a parenthesized group that fits flat",
                                line + 1
                            ));
                        }
                    }
                }
            }
        }

        match token.kind {
            Token::LParen => parens.push(position),
            Token::RParen => {
                parens.pop();
            }
            _ => {}
        }
    }
    violations
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
