//! KW-THEOREM AC-1: exact-candidate structural source and oracle sweep.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum SourceClass {
    FencedKen,
    RawKen,
    EvaluationResults,
    EvaluationFixture,
}

impl SourceClass {
    const ALL: [Self; 4] = [
        Self::FencedKen,
        Self::RawKen,
        Self::EvaluationResults,
        Self::EvaluationFixture,
    ];

    fn is_markdown(self) -> bool {
        matches!(self, Self::FencedKen)
    }
}

const ORACLE_PATH: &str = "crates/ken-elaborator/tests/kw_theorem_source_oracle.rs";

struct PopulationExclusion {
    path: &'static str,
    reason: &'static str,
}

// This is the closed complement to the content-derived population. The oracle
// contains planted retired-spelling controls, so scanning its own fixtures
// would turn those controls into candidate findings.
const POPULATION_EXCLUSIONS: &[PopulationExclusion] = &[PopulationExclusion {
    path: ORACLE_PATH,
    reason: "the oracle contains planted retired-spelling controls",
}];

struct NonFencedOracle {
    path: &'static str,
    class: SourceClass,
    reason: &'static str,
}

// These sources intentionally carry Ken text without Markdown fences. Keeping
// them in one named inventory makes that exceptional representation explicit.
const NON_FENCED_ORACLES: &[NonFencedOracle] = &[NonFencedOracle {
    path: "library/agents/evaluations/fixtures/proof-terminals.txt",
    class: SourceClass::EvaluationFixture,
    reason: "the proof-terminal task embeds complete Ken declarations as plain text",
}];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OccurrenceDisposition {
    OrdinaryEnglish,
    IntentionalResidual,
}

struct AllowedOccurrences {
    path: &'static str,
    lines: &'static [(usize, usize)],
    disposition: OccurrenceDisposition,
    reason: &'static str,
}

// KW-THEOREM AC-2's review-visible leave/residual classification. Each tuple is
// `(line, occurrence count)`, so a new occurrence on an existing line also
// reddens instead of inheriting that line's prior classification.
const ALLOWED_OCCURRENCES: &[AllowedOccurrences] = &[
    AllowedOccurrences {
        path: "agent/memory/fleet/a-vacuous-law-has-zero-trust-delta.md",
        lines: &[(44, 1), (57, 1), (61, 1)],
        disposition: OccurrenceDisposition::OrdinaryEnglish,
        reason: "mathematical prose names a helper result",
    },
    AllowedOccurrences {
        path: "catalog/guide/proof-techniques.ken.md",
        lines: &[(8, 1), (274, 1), (348, 1)],
        disposition: OccurrenceDisposition::OrdinaryEnglish,
        reason: "proof guidance names helper results",
    },
    AllowedOccurrences {
        path: "catalog/guide/surface-reference.ken.md",
        lines: &[(372, 1)],
        disposition: OccurrenceDisposition::OrdinaryEnglish,
        reason: "surface guidance names a helper result",
    },
    AllowedOccurrences {
        path: "catalog/packages/Core/Classes/EffectfulClasses.ken.md",
        lines: &[
            (189, 1),
            (615, 1),
            (1069, 1),
            (1172, 1),
            (1180, 1),
            (1183, 1),
            (1185, 1),
            (1190, 1),
            (1243, 1),
            (1346, 1),
            (1514, 1),
            (1863, 1),
            (3352, 1),
        ],
        disposition: OccurrenceDisposition::OrdinaryEnglish,
        reason: "package prose names mathematical helper results",
    },
    AllowedOccurrences {
        path: "catalog/packages/Data/Collections/Derived.ken.md",
        lines: &[(673, 1), (674, 1), (849, 1)],
        disposition: OccurrenceDisposition::OrdinaryEnglish,
        reason: "package prose names mathematical helper results",
    },
    AllowedOccurrences {
        path: "catalog/packages/Data/Collections/Map.ken.md",
        lines: &[
            (3910, 1),
            (3931, 1),
            (5662, 1),
            (7604, 1),
            (9533, 1),
            (12427, 1),
            (13594, 1),
        ],
        disposition: OccurrenceDisposition::OrdinaryEnglish,
        reason: "package prose names mathematical helper results",
    },
    AllowedOccurrences {
        path: "catalog/packages/Data/Numeric/Nat/Order.ken.md",
        lines: &[(256, 1), (309, 1)],
        disposition: OccurrenceDisposition::OrdinaryEnglish,
        reason: "package prose names mathematical helper results",
    },
    AllowedOccurrences {
        path: "conformance/surface/declarations/seed-named-proof-claims.md",
        lines: &[(31, 1), (43, 1), (49, 1), (51, 1)],
        disposition: OccurrenceDisposition::IntentionalResidual,
        reason: "AC-2(d)/AC-4 retains the retired spelling in its anchor, source, and diagnostic",
    },
    AllowedOccurrences {
        path: "docs/program/07-catalog-style-guide.md",
        lines: &[(157, 1), (158, 1), (205, 1), (431, 1), (443, 1)],
        disposition: OccurrenceDisposition::OrdinaryEnglish,
        reason: "catalog guidance names helper results",
    },
    AllowedOccurrences {
        path: "docs/program/diary/2026/Jul/21.md",
        lines: &[
            (370, 1),
            (383, 1),
            (655, 1),
            (705, 1),
            (751, 1),
            (1261, 1),
            (1283, 1),
            (1405, 1),
        ],
        disposition: OccurrenceDisposition::OrdinaryEnglish,
        reason: "historical narrative names mathematical helper results",
    },
    AllowedOccurrences {
        path: "docs/program/wp/KM-sigma-eq-pair-refl.md",
        lines: &[(50, 1)],
        disposition: OccurrenceDisposition::OrdinaryEnglish,
        reason: "work-package prose names a helper result",
    },
    AllowedOccurrences {
        path: "docs/program/wp/SPAN-SEAL-buffer-span-producer-closure.md",
        lines: &[(16, 1), (59, 1), (112, 2)],
        disposition: OccurrenceDisposition::OrdinaryEnglish,
        reason: "work-package prose names helper results",
    },
    AllowedOccurrences {
        path: "docs/program/wp/SURF-named-proof-claims.md",
        lines: &[(147, 2), (179, 1)],
        disposition: OccurrenceDisposition::OrdinaryEnglish,
        reason: "work-package prose names helper results",
    },
    AllowedOccurrences {
        path: "docs/program/wp/let3-catalog-let-pilot.md",
        lines: &[(58, 1), (59, 1)],
        disposition: OccurrenceDisposition::OrdinaryEnglish,
        reason: "work-package prose names a helper result",
    },
    AllowedOccurrences {
        path: "docs/program/wp/str-bij-overclaim-erratum.md",
        lines: &[(53, 1), (135, 1)],
        disposition: OccurrenceDisposition::OrdinaryEnglish,
        reason: "work-package prose names a helper result",
    },
    AllowedOccurrences {
        path: "library/learn/reading-ken/02-types-contracts-and-proofs.md",
        lines: &[(102, 1), (130, 1)],
        disposition: OccurrenceDisposition::OrdinaryEnglish,
        reason: "learning prose names mathematical helper results",
    },
    AllowedOccurrences {
        path: "spec/30-surface/33-declarations.md",
        lines: &[(381, 1), (584, 1)],
        disposition: OccurrenceDisposition::OrdinaryEnglish,
        reason: "normative prose names mathematical helper results",
    },
    AllowedOccurrences {
        path: "spec/30-surface/38-ffi-io.md",
        lines: &[(615, 1)],
        disposition: OccurrenceDisposition::OrdinaryEnglish,
        reason: "normative prose names a mathematical helper result",
    },
];

#[derive(Debug, Eq, PartialEq)]
struct RetiredOccurrence {
    line: usize,
    column: usize,
    text: String,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn git_output(args: &[&str]) -> Vec<u8> {
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
    output.stdout
}

fn git(args: &[&str]) -> String {
    String::from_utf8(git_output(args)).expect("git output must be UTF-8")
}

fn population_exclusion(path: &str) -> Option<&'static PopulationExclusion> {
    POPULATION_EXCLUSIONS
        .iter()
        .find(|exclusion| exclusion.path == path)
}

fn is_evaluation_results(source: &str) -> bool {
    let mut has_schema = false;
    let mut has_run = false;
    let mut has_answer = false;
    for line in source.lines().map(str::trim) {
        has_schema |= line == "schema_version = 1";
        has_run |= line == "[[run]]";
        has_answer |= line == "answer = \"\"\"";
    }
    has_schema && has_run && has_answer
}

fn classify(path: &str, source: &str) -> Result<Option<SourceClass>, String> {
    if population_exclusion(path).is_some() {
        return Ok(None);
    }
    if path.ends_with(".ken") {
        return Ok(Some(SourceClass::RawKen));
    }
    if markdown_ken_lines(source)?.saw_ken_fence {
        return Ok(Some(SourceClass::FencedKen));
    }
    if is_evaluation_results(source) {
        return Ok(Some(SourceClass::EvaluationResults));
    }
    if let Some(oracle) = NON_FENCED_ORACLES.iter().find(|oracle| oracle.path == path) {
        assert!(
            !oracle.reason.is_empty(),
            "every non-fenced oracle needs a representation reason"
        );
        return Ok(Some(oracle.class));
    }
    Ok(None)
}

fn candidate_inputs() -> (String, BTreeMap<SourceClass, Vec<(String, String)>>) {
    let candidate = git(&["rev-parse", "HEAD"]).trim().to_owned();
    let tree = git(&["ls-tree", "-r", "--name-only", &candidate]);
    let mut inputs: BTreeMap<SourceClass, Vec<(String, String)>> = BTreeMap::new();

    for path in tree.lines() {
        let object = format!("{candidate}:{path}");
        let Ok(source) = String::from_utf8(git_output(&["show", &object])) else {
            // Non-UTF-8 blobs cannot carry UTF-8 Ken source or fence info.
            continue;
        };
        let Some(class) =
            classify(path, &source).unwrap_or_else(|error| panic!("{candidate}:{path}: {error}"))
        else {
            continue;
        };
        inputs
            .entry(class)
            .or_default()
            .push((path.to_owned(), source));
    }

    let populated = inputs.keys().copied().collect::<BTreeSet<_>>();
    assert_eq!(
        populated,
        SourceClass::ALL.into_iter().collect(),
        "the exact candidate must populate every structural source/oracle class"
    );
    (candidate, inputs)
}

struct MarkdownKenLines<'a> {
    saw_ken_fence: bool,
    lines: Vec<(usize, &'a str)>,
}

fn markdown_ken_lines(source: &str) -> Result<MarkdownKenLines<'_>, String> {
    #[derive(Clone, Copy)]
    struct Fence {
        ticks: usize,
        is_ken: bool,
    }

    let mut fence = None;
    let mut ken_lines = Vec::new();
    let mut saw_ken_fence = false;

    for (index, line) in source.lines().enumerate() {
        let line_number = index + 1;
        let indent = line.bytes().take_while(|byte| *byte == b' ').count();
        let markdown = if indent <= 3 { &line[indent..] } else { line };

        match fence {
            None => {
                let ticks = markdown.bytes().take_while(|byte| *byte == b'`').count();
                if ticks >= 3 {
                    let info = markdown[ticks..].trim();
                    let is_ken =
                        matches!(info, "ken" | "ken ignore" | "ken reject" | "ken example");
                    saw_ken_fence |= is_ken;
                    fence = Some(Fence { ticks, is_ken });
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

    if fence.is_some_and(|open| open.is_ken) {
        return Err("unterminated Markdown fence".to_owned());
    }
    Ok(MarkdownKenLines {
        saw_ken_fence,
        lines: ken_lines,
    })
}

fn occurrence_lines(class: SourceClass, source: &str) -> Result<Vec<(usize, &str)>, String> {
    if class.is_markdown() {
        markdown_ken_lines(source)?;
    }
    Ok(source
        .lines()
        .enumerate()
        .map(|(i, line)| (i + 1, line))
        .collect())
}

fn retired_occurrences(class: SourceClass, source: &str) -> Result<Vec<RetiredOccurrence>, String> {
    let mut findings = Vec::new();
    for (line, text) in occurrence_lines(class, source)? {
        for offset in retired_occurrence_offsets(text) {
            findings.push(RetiredOccurrence {
                line,
                column: offset + 1,
                text: text.trim().to_owned(),
            });
        }
    }
    Ok(findings)
}

fn retired_occurrence_offsets(source: &str) -> Vec<usize> {
    source
        .to_ascii_lowercase()
        .match_indices("lemma")
        .map(|(offset, _)| offset)
        .collect()
}

fn retired_findings(
    candidate: &str,
    inputs: BTreeMap<SourceClass, Vec<(String, String)>>,
) -> Vec<(String, RetiredOccurrence)> {
    let mut findings = Vec::new();
    for (class, files) in inputs {
        for (path, source) in files {
            for finding in retired_occurrences(class, &source)
                .unwrap_or_else(|error| panic!("{candidate}:{path}: {error}"))
            {
                findings.push((path.clone(), finding));
            }
        }
    }
    findings
}

fn occurrence_census(findings: &[(String, RetiredOccurrence)]) -> BTreeMap<(String, usize), usize> {
    let mut census = BTreeMap::new();
    for (path, finding) in findings {
        *census.entry((path.clone(), finding.line)).or_default() += 1;
    }
    census
}

fn allowed_occurrence_census() -> BTreeMap<(String, usize), usize> {
    let mut census = BTreeMap::new();
    for file in ALLOWED_OCCURRENCES {
        assert!(!file.reason.is_empty(), "{} needs a reason", file.path);
        assert!(
            matches!(
                file.disposition,
                OccurrenceDisposition::OrdinaryEnglish | OccurrenceDisposition::IntentionalResidual
            ),
            "{} needs an AC-2 disposition",
            file.path
        );
        for &(line, count) in file.lines {
            assert!(
                census.insert((file.path.to_owned(), line), count).is_none(),
                "{}:{line} is classified twice",
                file.path
            );
        }
    }
    census
}

// MEASURED: every UTF-8 file in the exact `git ls-tree` candidate is classified
// by Ken content, with one named self-exclusion, structurally recognized
// evaluation results, and one named non-fenced fixture representation; all
// lines of every selected file are scanned for retired occurrences.
// CLAIMED: every retired occurrence in tracked Ken source or a source oracle is
// enumerated as either ordinary-English leave or intentional AC-2(d) residual.
// THE GAP: this source oracle observes tracked text and Ken-fence structure, not
// parser semantics; the same-harness AC-4 test separately pins lexing, full
// elaboration, formatting, and the exact negative diagnostic.
// Promise class: durable invariant, with the AC-2(d) negative control as its
// explicitly enumerated compatibility vector.
#[test]
fn exact_candidate_has_no_unclassified_retired_occurrences() {
    let (candidate, inputs) = candidate_inputs();
    let findings = retired_findings(&candidate, inputs);

    assert_eq!(
        occurrence_census(&findings),
        allowed_occurrence_census(),
        "every retired occurrence must have an exact AC-2 leave/residual classification; \
         findings: {findings:#?}"
    );
}

// Promise class: durable invariant.
#[test]
fn occurrence_scan_reaches_every_population_class_beyond_declaration_heads() {
    let controls = [
        (
            SourceClass::FencedKen,
            "Retired lemmas remain forbidden in prose.\n```ken\nconst name : String = \
             \"lemma_identifier\"\n```\n",
        ),
        (
            SourceClass::RawKen,
            "const names : String = \"lemmas lemma_identifier\"\n",
        ),
        (
            SourceClass::EvaluationResults,
            "answer = \"\"\"\nconst names : String = \"lemmas lemma_identifier\"\n\"\"\"\n",
        ),
        (
            SourceClass::EvaluationFixture,
            "const names : String = \"lemmas lemma_identifier\"\n",
        ),
    ];

    let mut inputs = BTreeMap::new();
    for (class, source) in controls {
        inputs
            .entry(class)
            .or_insert_with(Vec::new)
            .push((format!("{class:?}-control"), source.to_owned()));
    }
    let findings = retired_findings("planted-control", inputs);
    for class in SourceClass::ALL {
        let class_path = format!("{class:?}-control");
        let class_findings = findings
            .iter()
            .filter(|(path, _)| path == &class_path)
            .count();
        assert_eq!(
            class_findings, 2,
            "{class:?} must expose both planted occurrences beyond a declaration head"
        );
    }
}

// Promise class: durable invariant.
#[test]
fn occurrence_probe_is_case_insensitive_plural_possessive_and_identifier_reaching() {
    let probe = "LEMMA lemmas lemma's lemma_identifier";
    assert_eq!(
        retired_occurrence_offsets(probe).len(),
        4,
        "the occurrence probe must not lose case, plural, possessive, or derived rows"
    );
}

// Promise class: durable invariant.
#[test]
fn new_markdown_with_a_ken_fence_enters_without_registration() {
    let source = r#"
prose
```ken
const fresh : Nat = Zero
```
"#;
    assert_eq!(
        classify("new/unregistered/source.md", source).unwrap(),
        Some(SourceClass::FencedKen)
    );
}

// Promise class: transition sentinel. Any new exclusion requires review of the
// closed complement and an independently causal control.
#[test]
fn population_exclusions_are_closed_named_and_causal() {
    assert_eq!(POPULATION_EXCLUSIONS.len(), 1);
    let exclusion = &POPULATION_EXCLUSIONS[0];
    assert_eq!(exclusion.path, ORACLE_PATH);
    assert!(!exclusion.reason.is_empty());

    let planted = r#"
```ken
const names : String = "lemma_control"
```
"#;
    assert_eq!(classify(exclusion.path, planted).unwrap(), None);
    assert_eq!(
        classify("crates/ken-elaborator/tests/not_the_oracle.rs", planted).unwrap(),
        Some(SourceClass::FencedKen),
        "without the named self-exclusion the population must grow by this file"
    );
}

// Promise class: durable invariant.
#[test]
fn spaced_ken_fence_info_is_classified() {
    let source = r#"
``` ken
const spaced : Nat = Zero
```
"#;
    assert_eq!(
        classify("docs/spaced.md", source).unwrap(),
        Some(SourceClass::FencedKen)
    );
    assert_eq!(
        markdown_ken_lines(source).unwrap().lines,
        vec![(3, "const spaced : Nat = Zero")]
    );
}

// Promise class: durable invariant.
#[test]
fn whole_tree_population_reaches_outside_adversary_roots() {
    let source = r#"
```ken
const tooling_control : Nat = Zero
```
"#;
    assert_eq!(
        classify("tooling/oracle-control.md", source).unwrap(),
        Some(SourceClass::FencedKen)
    );
}
