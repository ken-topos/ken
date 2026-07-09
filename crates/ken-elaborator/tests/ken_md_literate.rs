use ken_elaborator::error::ElabError;
use ken_elaborator::lexer::{Lexer, Token};
use ken_elaborator::{extract_ken_md, ElabEnv};

fn token_kinds(src: &str) -> Vec<Token> {
    Lexer::lex(src)
        .expect("source should lex")
        .into_iter()
        .map(|(tok, _)| tok)
        .filter(|tok| *tok != Token::Eof)
        .collect()
}

#[test]
fn extraction_preserves_length_newlines_and_utf8() {
    let md = "Intro café\n```ken\nconst answer : Nat = Zero\n```\n尾\n";
    let extracted = extract_ken_md(md).expect("extracts");

    assert_eq!(extracted.source.len(), md.len());
    assert_eq!(
        extracted.source.matches('\n').count(),
        md.matches('\n').count()
    );
    assert_eq!(
        &extracted.source[md.find("const answer").unwrap()..][.."const answer".len()],
        "const answer"
    );
    assert!(!extracted.source.contains("café"));
    assert!(!extracted.source.contains("尾"));
}

#[test]
fn token_stream_matches_supported_fence_contents() {
    let md = "\
prose
```ken
const first : Nat = Zero
```
more prose
```ken
fn id (x : Nat) : Nat = x
```
";
    let extracted = extract_ken_md(md).expect("extracts");
    let concatenated = "const first : Nat = Zero\nfn id (x : Nat) : Nat = x\n";

    assert_eq!(token_kinds(&extracted.source), token_kinds(concatenated));
}

#[test]
fn only_exact_ken_fences_compile() {
    let md = "\
```ken ignore
const ignored : Nat = Zero
```
```text
const alsoIgnored : Nat = Zero
```
```ken
const kept : Nat = Zero
```
";
    let extracted = extract_ken_md(md).expect("extracts");

    assert!(extracted.source.contains("const kept"));
    assert!(!extracted.source.contains("const ignored"));
    assert!(!extracted.source.contains("const alsoIgnored"));
    assert_eq!(extracted.compiled_ranges.len(), 1);
}

#[test]
fn token_spans_keep_original_markdown_offsets() {
    let md = "markdown before\n```ken\nconst kept : Nat = Zero\n```\n";
    let extracted = extract_ken_md(md).expect("extracts");
    let const_offset = md.find("const").unwrap();
    let (_, span) = Lexer::lex(&extracted.source)
        .expect("lexes")
        .into_iter()
        .find(|(tok, _)| *tok == Token::KwConst)
        .expect("const token");

    assert_eq!(span.start, const_offset);
}

#[test]
fn elaboration_errors_report_original_markdown_offsets() {
    let md = "heading\n```ken\nconst bad : Nat = x\n```\n";
    let expected = md.find("x").unwrap();
    let mut env = ElabEnv::new().expect("env");
    let err = env
        .elaborate_ken_md_file(md)
        .expect_err("unbound name rejects");

    match err {
        ElabError::UnboundName { name, span } => {
            assert_eq!(name, "x");
            assert_eq!(span.start, expected);
        }
        ElabError::UnresolvedCon { name, span } => {
            assert_eq!(name, "x");
            assert_eq!(span.start, expected);
        }
        other => panic!("expected name-resolution error, got {other:?}"),
    }
}

#[test]
fn declarations_may_not_straddle_compiled_fences() {
    let md = "\
```ken
pub
```
```ken
const visible : Nat = Zero
```
";
    let mut env = ElabEnv::new().expect("env");
    let err = env
        .elaborate_ken_md_file(md)
        .expect_err("split pub must reject");

    match err {
        ElabError::ParseError { msg, span } => {
            assert!(
                msg.contains("expected") || msg.contains("found"),
                "unexpected parse message: {msg}"
            );
            assert!(span.start >= md.find("pub").unwrap());
        }
        other => panic!("expected ParseError, got {other:?}"),
    }
}

#[test]
fn ordinary_ken_elaboration_is_unchanged() {
    let mut env = ElabEnv::new().expect("env");
    env.elaborate_file("const plain : Nat = Zero")
        .expect("ordinary .ken source still elaborates through the existing path");
}

// AC2 — `ken reject` whose body fails to elaborate: the overall run succeeds
// (the expected failure is not surfaced as an error).
#[test]
fn ken_reject_with_invalid_body_check_passes() {
    let md = "\
```ken
const kept : Nat = Zero
```
```ken reject
const bad : Nat = undefinedName
```
";
    let mut env = ElabEnv::new().expect("env");
    env.elaborate_ken_md_file(md)
        .expect("a reject block that correctly fails to elaborate must not fail the run");
}

// AC3 — `ken reject` whose body is unexpectedly *valid*: the run must fail,
// and the error must name the block (its role and original byte range).
#[test]
fn ken_reject_with_stale_valid_body_check_fails() {
    let md = "\
```ken
const kept : Nat = Zero
```
```ken reject
const stale : Nat = Zero
```
";
    let expected_start = md.find("const stale").unwrap();
    let mut env = ElabEnv::new().expect("env");
    let err = env
        .elaborate_ken_md_file(md)
        .expect_err("an unexpectedly-valid reject block must fail the run");

    match err {
        ElabError::ParseError { msg, span } => {
            assert!(
                msg.contains("reject"),
                "error should name the 'ken reject' role: {msg}"
            );
            assert!(
                span.start <= expected_start && expected_start < span.end,
                "span {:?} should cover the stale block's body (starting at {})",
                span,
                expected_start
            );
        }
        other => panic!("expected ParseError naming the stale reject block, got {other:?}"),
    }
}

// AC4 — `ken example` with a valid body: the run succeeds and the body is
// NOT tangled into the module (does not appear in `compiled_ranges`/`source`,
// and removing the fence leaves `extraction.source` unchanged).
#[test]
fn ken_example_with_valid_body_check_passes_and_is_not_tangled() {
    let md = "\
```ken
const kept : Nat = Zero
```
```ken example
const shown : Nat = Zero
```
";
    let without_example = "\
```ken
const kept : Nat = Zero
```
";
    let with_extraction = extract_ken_md(md).expect("extracts");
    let without_extraction = extract_ken_md(without_example).expect("extracts");

    assert_eq!(with_extraction.compiled_ranges.len(), 1);
    assert!(!with_extraction.source.contains("shown"));
    assert_eq!(with_extraction.example_ranges.len(), 1);
    assert_eq!(
        with_extraction.source[..without_extraction.source.len()],
        without_extraction.source[..],
        "removing the example fence must not change the tangled source"
    );

    let mut env = ElabEnv::new().expect("env");
    env.elaborate_ken_md_file(md)
        .expect("a valid example block must not fail the run");
}

// AC5 — `ken example` whose body fails to elaborate: the run must fail,
// citing the block.
#[test]
fn ken_example_with_invalid_body_check_fails() {
    let md = "\
```ken
const kept : Nat = Zero
```
```ken example
const broken : Nat = undefinedName
```
";
    let expected_start = md.find("const broken").unwrap();
    let mut env = ElabEnv::new().expect("env");
    let err = env
        .elaborate_ken_md_file(md)
        .expect_err("an example block that fails to elaborate must fail the run");

    match err {
        ElabError::ParseError { msg, span } => {
            assert!(
                msg.contains("example"),
                "error should name the 'ken example' role: {msg}"
            );
            assert!(
                span.start <= expected_start && expected_start < span.end,
                "span {:?} should cover the broken block's body (starting at {})",
                span,
                expected_start
            );
        }
        other => panic!("expected ParseError naming the broken example block, got {other:?}"),
    }
}

// AC7 — an unrecognized `ken`-prefixed role hard-errors at extraction time
// rather than silently falling back to prose.
#[test]
fn unrecognized_ken_role_is_a_hard_extraction_error() {
    let md = "\
```ken bogus
const whatever : Nat = Zero
```
";
    let err = extract_ken_md(md).expect_err("an unrecognized 'ken' role must hard-error");
    match err {
        ElabError::ParseError { msg, .. } => {
            assert!(
                msg.contains("bogus") || msg.contains("unrecognized"),
                "error should name the unrecognized role: {msg}"
            );
        }
        other => panic!("expected ParseError for the unrecognized role, got {other:?}"),
    }
}

// AC8 — offset/UTF-8 safety: reject/example ranges are valid byte offsets
// into the original `.ken.md` source even with non-ASCII prose around them.
#[test]
fn checked_fence_ranges_are_utf8_safe_with_non_ascii_prose() {
    let md = "café intro 尾\n\
```ken reject\n\
const bad : Nat = undefinedName\n\
```\n\
更多 prose\n\
```ken example\n\
const shown : Nat = Zero\n\
```\n\
末尾\n";
    let extracted = extract_ken_md(md).expect("extracts with non-ASCII prose");

    assert_eq!(extracted.reject_ranges.len(), 1);
    assert_eq!(extracted.example_ranges.len(), 1);
    for range in extracted
        .reject_ranges
        .iter()
        .chain(extracted.example_ranges.iter())
    {
        assert!(md.is_char_boundary(range.start));
        assert!(md.is_char_boundary(range.end));
    }
    assert_eq!(extracted.source.len(), md.len());
    assert_eq!(
        extracted.source.matches('\n').count(),
        md.matches('\n').count()
    );
}

// AC6 — `ken ignore` stays unchecked: a body that would fail to elaborate is
// never surfaced as an error (no pass/fail signal either way).
#[test]
fn ken_ignore_with_elaboration_breaking_body_is_never_checked() {
    let md = "\
```ken ignore
const bad : Nat = undefinedName
```
";
    let mut env = ElabEnv::new().expect("env");
    env.elaborate_ken_md_file(md)
        .expect("an 'ken ignore' block must never be checked, regardless of its body");
}
