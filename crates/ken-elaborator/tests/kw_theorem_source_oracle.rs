//! KW-THEOREM AC-1: exact-candidate structural source and oracle sweep.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum SourceClass {
    LiterateKen,
    RawKen,
    ConformanceSeed,
    EvaluationResults,
    EvaluationFixture,
}

impl SourceClass {
    const ALL: [Self; 5] = [
        Self::LiterateKen,
        Self::RawKen,
        Self::ConformanceSeed,
        Self::EvaluationResults,
        Self::EvaluationFixture,
    ];

    fn is_markdown(self) -> bool {
        matches!(self, Self::LiterateKen | Self::ConformanceSeed)
    }
}

#[derive(Debug, Eq, PartialEq)]
struct RetiredDeclaration {
    line: usize,
    text: String,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn git(args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_root())
        .output()
        .unwrap_or_else(|error| panic!("git {} must run: {error}", args.join(" ")));
    assert!(
        output.status.success(),
        "git {} failed:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("git output must be UTF-8")
}

fn classify(path: &str) -> Option<SourceClass> {
    if path.ends_with(".ken.md") {
        return Some(SourceClass::LiterateKen);
    }
    if path.ends_with(".ken") {
        return Some(SourceClass::RawKen);
    }
    if path.starts_with("conformance/")
        && path.ends_with(".md")
        && path
            .rsplit('/')
            .next()
            .is_some_and(|name| name.starts_with("seed-"))
    {
        return Some(SourceClass::ConformanceSeed);
    }
    if path.starts_with("library/agents/evaluations/")
        && path.ends_with(".toml")
        && path
            .rsplit('/')
            .next()
            .is_some_and(|name| name.starts_with("results-"))
    {
        return Some(SourceClass::EvaluationResults);
    }
    if path == "library/agents/evaluations/fixtures/proof-terminals.txt" {
        return Some(SourceClass::EvaluationFixture);
    }
    None
}

fn candidate_inputs() -> (String, BTreeMap<SourceClass, Vec<(String, String)>>) {
    let candidate = git(&["rev-parse", "HEAD"]).trim().to_owned();
    let tree = git(&["ls-tree", "-r", "--name-only", &candidate]);
    let mut inputs: BTreeMap<SourceClass, Vec<(String, String)>> = BTreeMap::new();

    for path in tree.lines() {
        let Some(class) = classify(path) else {
            continue;
        };
        let object = format!("{candidate}:{path}");
        inputs
            .entry(class)
            .or_default()
            .push((path.to_owned(), git(&["show", &object])));
    }

    let populated = inputs.keys().copied().collect::<BTreeSet<_>>();
    assert_eq!(
        populated,
        SourceClass::ALL.into_iter().collect(),
        "the exact candidate must populate every structural source/oracle class"
    );
    (candidate, inputs)
}

fn markdown_ken_lines(source: &str) -> Result<Vec<(usize, &str)>, String> {
    #[derive(Clone, Copy)]
    struct Fence {
        ticks: usize,
        is_ken: bool,
    }

    let mut fence = None;
    let mut ken_lines = Vec::new();

    for (index, line) in source.lines().enumerate() {
        let line_number = index + 1;
        let indent = line.bytes().take_while(|byte| *byte == b' ').count();
        let markdown = if indent <= 3 { &line[indent..] } else { line };

        match fence {
            None => {
                let ticks = markdown.bytes().take_while(|byte| *byte == b'`').count();
                if ticks >= 3 {
                    let info = markdown[ticks..].trim_end();
                    fence = Some(Fence {
                        ticks,
                        is_ken: matches!(info, "ken" | "ken ignore" | "ken reject" | "ken example"),
                    });
                }
            }
            Some(open) => {
                let ticks = markdown.bytes().take_while(|byte| *byte == b'`').count();
                if ticks >= open.ticks && markdown[ticks..].trim().is_empty() {
                    fence = None;
                } else if open.is_ken {
                    ken_lines.push((line_number, line));
                }
            }
        }
    }

    if fence.is_some() {
        return Err("unterminated Markdown fence".to_owned());
    }
    Ok(ken_lines)
}

fn declaration_lines(class: SourceClass, source: &str) -> Result<Vec<(usize, &str)>, String> {
    if class.is_markdown() {
        markdown_ken_lines(source)
    } else {
        Ok(source
            .lines()
            .enumerate()
            .map(|(i, line)| (i + 1, line))
            .collect())
    }
}

fn is_retired_declaration(line: &str) -> bool {
    let mut words = line.trim_start().split_whitespace();
    let mut head = words.next();
    if head.is_some_and(|word| word.eq_ignore_ascii_case("pub")) {
        head = words.next();
    }
    head.is_some_and(|word| word.eq_ignore_ascii_case("lemma"))
}

fn retired_declarations(
    class: SourceClass,
    source: &str,
) -> Result<Vec<RetiredDeclaration>, String> {
    Ok(declaration_lines(class, source)?
        .into_iter()
        .filter(|(_, line)| is_retired_declaration(line))
        .map(|(line, text)| RetiredDeclaration {
            line,
            text: text.trim().to_owned(),
        })
        .collect())
}

fn retired_occurrence_offsets(source: &str) -> Vec<usize> {
    source
        .to_ascii_lowercase()
        .match_indices("lemma")
        .map(|(offset, _)| offset)
        .collect()
}

// MEASURED: an exact `git ls-tree` population of every tracked `.ken` and
// `.ken.md`, plus derived conformance seed Markdown and evaluation oracles,
// read from the candidate object and scanned only inside Ken fences when
// Markdown is literate.
// CLAIMED: no retired standalone declaration spelling survives in a Ken source
// or non-source oracle except the enumerated AC-2(d)/AC-4 negative control.
// THE GAP: this source oracle observes declaration heads, not parser semantics;
// the same-harness AC-4 test separately pins lexing, full elaboration,
// formatting, and the exact negative diagnostic.
#[test]
fn exact_candidate_has_no_unclassified_retired_declarations() {
    let (candidate, inputs) = candidate_inputs();
    let mut findings = Vec::new();

    for (class, files) in inputs {
        for (path, source) in files {
            for finding in retired_declarations(class, &source)
                .unwrap_or_else(|error| panic!("{candidate}:{path}: {error}"))
            {
                findings.push((path.clone(), finding));
            }
        }
    }

    assert_eq!(
        findings,
        vec![(
            "conformance/surface/declarations/seed-named-proof-claims.md".to_owned(),
            RetiredDeclaration {
                line: 43,
                text: "lemma kw_theorem_refl (x : Bool) : Equal Bool x x = Refl".to_owned(),
            },
        )],
        "every residual retired declaration must be an enumerated AC-2(d) control"
    );
}

#[test]
fn planted_retired_declaration_is_seen_in_every_structural_class() {
    let controls = [
        (
            SourceClass::LiterateKen,
            "lemma is ordinary prose here\n```ken\nlemma literate_control : Top = Axiom\n```\n",
        ),
        (
            SourceClass::RawKen,
            "lemma raw_control : Top = Axiom\n",
        ),
        (
            SourceClass::ConformanceSeed,
            "lemma is ordinary prose here\n  ```ken reject\n  lemma seed_control : Top = Axiom\n  ```\n",
        ),
        (
            SourceClass::EvaluationResults,
            "answer = \"\"\"\nlemma results_control : Top = Axiom\n\"\"\"\n",
        ),
        (
            SourceClass::EvaluationFixture,
            "lemma fixture_control : Top = Axiom\n",
        ),
    ];

    for (class, source) in controls {
        let findings =
            retired_declarations(class, source).expect("planted control must be classifiable");
        assert_eq!(
            findings.len(),
            1,
            "{class:?} must expose exactly its planted declaration"
        );
        assert!(
            findings[0].text.contains("_control"),
            "{class:?} must detect the planted declaration, not Markdown prose"
        );
    }
}

#[test]
fn occurrence_probe_is_case_insensitive_plural_possessive_and_identifier_safe() {
    let probe = "LEMMA lemmas lemma's lemma_identifier";
    assert_eq!(
        retired_occurrence_offsets(probe).len(),
        4,
        "the occurrence probe must not lose case, plural, possessive, or derived rows"
    );
}
