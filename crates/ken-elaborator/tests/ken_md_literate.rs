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
