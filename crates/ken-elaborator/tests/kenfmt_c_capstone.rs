//! WP C — executable soundness gate over the frozen reformat corpus.

use std::fs;
use std::path::{Path, PathBuf};

use ken_elaborator::layout::{display_width, format_ken, CANONICAL_WIDTH, INDENT_WIDTH};
use ken_elaborator::lexer::Token;
use ken_elaborator::lossless::parse_lossless;
use ken_elaborator::{extract_ken_md, format_ken_md};

#[test]
fn canonical_live_corpus_is_a_fixed_point() {
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
    assert!(!literate.is_empty(), "literate corpus must not be empty");
    assert!(!plain.is_empty(), "plain corpus must not be empty");

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
