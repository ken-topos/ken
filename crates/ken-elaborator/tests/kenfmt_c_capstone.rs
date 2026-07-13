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

type Digest = (usize, u64);

struct PlainOracle {
    path: &'static str,
    ast: Digest,
    elab: Digest,
}

struct LiterateOracle {
    path: &'static str,
    non_body: Digest,
    roles: Digest,
    bodies: Digest,
    elab: Digest,
}

// Generated from the exact pre-reformat frame, 4276c9e4c736f72661fa7dc0689d1050efd8a493.
// These values deliberately live in the candidate so the preservation gate does
// not depend on Git history being present in a shallow CI checkout.
const PLAIN_FRAME_ORACLE: &[PlainOracle] = &[
    PlainOracle {
        path: "catalog/packages/Capability/Verify/ProofErasureBoundaryChecker.ken",
        ast: (2523, 15097096754884164626),
        elab: (529, 1133664933836613887),
    },
    PlainOracle {
        path: "examples/rosetta/accumulator-factory/accumulator-factory.ken",
        ast: (10005, 6474529705028527871),
        elab: (530, 1269093779773768664),
    },
    PlainOracle {
        path: "examples/rosetta/ackermann/ackermann.ken",
        ast: (13893, 14114019858815701120),
        elab: (530, 13691892354280438385),
    },
    PlainOracle {
        path: "examples/rosetta/closures/closures.ken",
        ast: (3836, 14116684374655137527),
        elab: (38, 7880510533857600124),
    },
    PlainOracle {
        path: "examples/rosetta/factorial/factorial.ken",
        ast: (15294, 9547857932267199579),
        elab: (530, 17758657664400445908),
    },
    PlainOracle {
        path: "examples/rosetta/fibonacci/fibonacci.ken",
        ast: (16248, 4772379726581069503),
        elab: (530, 7064407290575852589),
    },
    PlainOracle {
        path: "examples/rosetta/fizzbuzz/fizzbuzz.ken",
        ast: (23483, 13885462354896307810),
        elab: (530, 14506539258707426598),
    },
    PlainOracle {
        path: "examples/rosetta/gcd/gcd.ken",
        ast: (20057, 1654998761782227445),
        elab: (530, 10312116879069670277),
    },
    PlainOracle {
        path: "examples/rosetta/hailstone/hailstone.ken",
        ast: (13313, 17083283526094292166),
        elab: (530, 2104710637584015422),
    },
    PlainOracle {
        path: "examples/rosetta/letter-frequency/letter-frequency.ken",
        ast: (12893, 16311549745736640173),
        elab: (530, 2104710637584015422),
    },
    PlainOracle {
        path: "examples/rosetta/merge-sort/merge-sort.ken",
        ast: (9579, 13903258188282562203),
        elab: (38, 7880510533857600124),
    },
    PlainOracle {
        path: "examples/rosetta/mutual-recursion/mutual-recursion.ken",
        ast: (2684, 18132765894208952090),
        elab: (529, 622099709342506226),
    },
    PlainOracle {
        path: "examples/rosetta/palindrome/palindrome.ken",
        ast: (3609, 11723712112429823809),
        elab: (38, 7880510533857600124),
    },
    PlainOracle {
        path: "examples/rosetta/read-file-lines/read-file-lines.ken",
        ast: (12404, 1426488223853805935),
        elab: (529, 17974579862108406758),
    },
    PlainOracle {
        path: "examples/rosetta/rpn-calculator/rpn-calculator.ken",
        ast: (11034, 1116992010601463573),
        elab: (529, 17974579862108406758),
    },
    PlainOracle {
        path: "examples/rosetta/tree-traversal/tree-traversal.ken",
        ast: (3348, 16418674890690876012),
        elab: (38, 7880510533857600124),
    },
];

const LITERATE_FRAME_ORACLE: &[LiterateOracle] = &[
    LiterateOracle {
        path: "catalog/guide/decomposition-abstraction.ken.md",
        non_body: (6774, 3760008385915963939),
        roles: (53, 10037688677702532272),
        bodies: (3524, 11193913127275033442),
        elab: (529, 885254910282465358),
    },
    LiterateOracle {
        path: "catalog/guide/proof-techniques.ken.md",
        non_body: (12912, 8927004873038155629),
        roles: (146, 17497744662216630657),
        bodies: (12972, 6188435893361228945),
        elab: (38, 12239898488860970838),
    },
    LiterateOracle {
        path: "catalog/guide/surface-reference.ken.md",
        non_body: (20630, 13796131843174927509),
        roles: (212, 14668379460646231996),
        bodies: (17351, 4443136453603011700),
        elab: (38, 12239898488860970838),
    },
    LiterateOracle {
        path: "catalog/packages/Capability/Parsing/Parsing.ken.md",
        non_body: (10656, 9580184160329500348),
        roles: (40, 2923901368470146633),
        bodies: (73232, 13977598132591303649),
        elab: (530, 18374774688282540110),
    },
    LiterateOracle {
        path: "catalog/packages/Core/EffectfulClasses.ken.md",
        non_body: (23505, 16647807678531932950),
        roles: (364, 12870271445831248943),
        bodies: (875178, 7999271338717185508),
        elab: (38, 7880510533857600124),
    },
    LiterateOracle {
        path: "catalog/packages/Core/EmptyDec.ken.md",
        non_body: (9560, 1523905977475764228),
        roles: (136, 2392300118228111225),
        bodies: (14900, 9097841185403983372),
        elab: (529, 17974579862108406758),
    },
    LiterateOracle {
        path: "catalog/packages/Core/LawfulClasses.ken.md",
        non_body: (28213, 2509163055710560037),
        roles: (100, 12414575537886237477),
        bodies: (246977, 3763836438363746828),
        elab: (38, 7880510533857600124),
    },
    LiterateOracle {
        path: "catalog/packages/Core/LawfulFunctors.ken.md",
        non_body: (8907, 13212373154402607305),
        roles: (60, 9335717978284175997),
        bodies: (42224, 8745952158824692795),
        elab: (38, 7880510533857600124),
    },
    LiterateOracle {
        path: "catalog/packages/Core/NatArith.ken.md",
        non_body: (1219, 5638983632013839077),
        roles: (21, 16010722412329868616),
        bodies: (32893, 362208107803323714),
        elab: (38, 7880510533857600124),
    },
    LiterateOracle {
        path: "catalog/packages/Core/OrdNat.ken.md",
        non_body: (8086, 16656890420579233212),
        roles: (83, 12639759097982950488),
        bodies: (22440, 13484820139639534015),
        elab: (38, 1137836956143451629),
    },
    LiterateOracle {
        path: "catalog/packages/Core/Transport.ken.md",
        non_body: (5940, 8496629964281118246),
        roles: (72, 838572123642843480),
        bodies: (7035, 10535524234487272318),
        elab: (529, 622099709342506226),
    },
    LiterateOracle {
        path: "catalog/packages/Data/Collections/Collections.ken.md",
        non_body: (16778, 3748699175877912106),
        roles: (70, 2549630713001227532),
        bodies: (141573, 1942631580699224185),
        elab: (38, 7880510533857600124),
    },
    LiterateOracle {
        path: "catalog/packages/Data/Collections/Map.ken.md",
        non_body: (31872, 3843303894275102233),
        roles: (190, 5921493409085836996),
        bodies: (1801133, 456685768879496307),
        elab: (38, 7880510533857600124),
    },
    LiterateOracle {
        path: "catalog/packages/Data/Sums/Sums.ken.md",
        non_body: (7105, 1944399919816828236),
        roles: (30, 5737283672692455396),
        bodies: (27454, 14247200435161758205),
        elab: (530, 10312116879069670277),
    },
];

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
fn actual_frozen_reformat_matches_frame_semantics_and_literate_bytes() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let actual_paths = frozen_corpus_paths(&repository);
    let mut expected_paths = PLAIN_FRAME_ORACLE
        .iter()
        .map(|oracle| oracle.path)
        .chain(LITERATE_FRAME_ORACLE.iter().map(|oracle| oracle.path))
        .chain(["examples/rosetta/hello-world/hello-world.ken"])
        .map(str::to_owned)
        .collect::<Vec<_>>();
    expected_paths.sort_unstable();
    assert_eq!(
        actual_paths, expected_paths,
        "the frozen 31-file set drifted"
    );

    for oracle in PLAIN_FRAME_ORACLE {
        let source = fs::read_to_string(repository.join(oracle.path)).unwrap();
        assert_eq!(
            fingerprint(&ast_shape(&source)),
            oracle.ast,
            "{}: AST drift from frame",
            oracle.path
        );
        assert_eq!(
            fingerprint(&format!("{:?}", elaborate(&source, false))),
            oracle.elab,
            "{}: elaboration-outcome drift from frame",
            oracle.path
        );
    }

    for oracle in LITERATE_FRAME_ORACLE {
        let source = fs::read_to_string(repository.join(oracle.path)).unwrap();
        let extraction = extract_ken_md(&source).unwrap();
        let roles = extraction
            .fences
            .iter()
            .map(|fence| format!("{:?}", fence.role))
            .collect::<Vec<_>>();
        let bodies = extraction
            .fences
            .iter()
            .map(|fence| {
                let body = &source[fence.body_range.clone()];
                parse_lossless(body).ok().map(|_| ast_shape(body))
            })
            .collect::<Vec<_>>();
        assert_eq!(
            fingerprint(&non_body_bytes(&source)),
            oracle.non_body,
            "{}: prose/marker bytes drift from frame",
            oracle.path
        );
        assert_eq!(
            fingerprint(&format!("{roles:?}")),
            oracle.roles,
            "{}: fence roles drift from frame",
            oracle.path
        );
        assert_eq!(
            fingerprint(&format!("{bodies:?}")),
            oracle.bodies,
            "{}: per-body AST drift from frame",
            oracle.path
        );
        assert_eq!(
            fingerprint(&format!("{:?}", elaborate(&source, true))),
            oracle.elab,
            "{}: elaboration-outcome drift from frame",
            oracle.path
        );
    }
}

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
    assert_eq!(literate.len(), 14);
    assert_eq!(plain.len(), 17);

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

fn frozen_corpus_paths(repository: &Path) -> Vec<String> {
    let mut literate = Vec::new();
    collect(&repository.join("catalog"), ".ken.md", &mut literate);
    let mut plain = Vec::new();
    collect(&repository.join("examples/rosetta"), ".ken", &mut plain);
    plain.push(
        repository.join("catalog/packages/Capability/Verify/ProofErasureBoundaryChecker.ken"),
    );
    let mut paths = literate
        .into_iter()
        .chain(plain)
        .map(|path| {
            path.strip_prefix(repository)
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned()
        })
        .collect::<Vec<_>>();
    paths.sort_unstable();
    paths
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

fn fingerprint(value: &str) -> (usize, u64) {
    let hash = value
        .as_bytes()
        .iter()
        .fold(0xcbf29ce484222325, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
        });
    (value.len(), hash)
}
