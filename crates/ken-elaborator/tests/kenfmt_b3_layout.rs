//! WP B3 — document algebra, deterministic layout, and preservation gates.

use std::fs;
use std::path::{Path, PathBuf};

use ken_elaborator::layout::{display_width, format_ken};
use ken_elaborator::lossless::parse_lossless;
use ken_elaborator::ElabEnv;

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

#[test]
fn ac2_ac3_mandatory_breaks_separators_and_no_alignment() {
    let source = "module M { fn choose (x : Bool) : Bool = match x { True |-> True; False |-> False }; const tiny : Nat = Zero; }";
    let expected = "module M {\n  fn choose (x : Bool) : Bool =\n    match x {\n      True ↦ True;\n      False ↦ False\n    };\n  const tiny : Nat = Zero\n}\n";
    assert_eq!(format_ken(source).unwrap(), expected);
}

#[test]
fn ac4_redundant_parentheses_drop_but_precedence_parentheses_stay() {
    let source = "fn redundant (a : Int) (b : Int) : Int = (a + b)\nfn required (a : Int) (b : Int) (c : Int) : Int = (a + b) * c\n";
    let formatted = format_ken(source).unwrap();
    assert!(formatted.contains("= a + b\n"));
    assert!(formatted.contains("= (a + b) * c\n"));
    assert_eq!(ast_shape(source), ast_shape(&formatted));

    let mut original = ElabEnv::new().unwrap();
    let mut canonical = ElabEnv::new().unwrap();
    assert!(original.elaborate_file(source).is_ok());
    assert!(canonical.elaborate_file(&formatted).is_ok());
    assert_eq!(original.env.trusted_base(), canonical.env.trusted_base());
}

#[test]
fn ac5_comments_are_retained_and_force_breaks() {
    let source = "-- lead\nfn keep (x : Int) : Int = (\n  -- middle\n  x\n) -- tail\n";
    let formatted = format_ken(source).unwrap();
    assert!(formatted.contains("-- lead\nfn keep"));
    assert!(formatted.contains("-- middle"));
    assert!(formatted.contains("-- tail"));
    assert_eq!(ast_shape(source), ast_shape(&formatted));
}

#[test]
fn ac5_interstitial_comment_forces_the_application_group() {
    let source = "const combined : Nat = combine left -- keep this edge\nright\n";
    let expected =
        "const combined : Nat =\n  combine\n    left\n    -- keep this edge\n    right\n";
    assert_eq!(format_ken(source).unwrap(), expected);
    assert_eq!(format_ken(expected).unwrap(), expected);
}

#[test]
fn ac5_trailing_comment_threshold_has_both_orientations() {
    let code = "const a : Nat = Zero";
    let fit_comment = format!("--{}", "x".repeat(88 - display_width(code) - 2 - 2));
    let overflow_comment = format!("{fit_comment}x");

    let fitting = format_ken(&format!("{code}  {fit_comment}\n")).unwrap();
    assert_eq!(fitting, format!("{code}  {fit_comment}\n"));

    let moved = format_ken(&format!("{code}  {overflow_comment}\n")).unwrap();
    assert_eq!(moved, format!("{overflow_comment}\n{code}\n"));
}

#[test]
fn ac6_independent_oracle_mandatory_forms_are_exact_fixed_points() {
    let sum = "data OptionNat = None | Some Nat";
    let sum_expected = "data OptionNat =\n  None\n  | Some Nat\n";
    assert_eq!(format_ken(sum).unwrap(), sum_expected);
    assert_eq!(format_ken(sum_expected).unwrap(), sum_expected);

    let nested = "fn choose (outer : Sum) : Nat = match outer { Left |-> match inner { Only |-> value }; Right |-> other }";
    let nested_expected = "fn choose (outer : Sum) : Nat =\n  match outer {\n    Left ↦\n      match inner {\n        Only ↦ value\n      };\n    Right ↦ other\n  }\n";
    assert_eq!(format_ken(nested).unwrap(), nested_expected);
    assert_eq!(format_ken(nested_expected).unwrap(), nested_expected);

    let compound =
        "fn compute (x : Nat) : Nat = let y = match x { Zero |-> 0; Suc n |-> n } in finish y";
    let compound_expected = "fn compute (x : Nat) : Nat =\n  let y =\n    match x {\n      Zero ↦ 0;\n      Suc n ↦ n\n    }\n  in\n    finish y\n";
    assert_eq!(format_ken(compound).unwrap(), compound_expected);
    assert_eq!(format_ken(compound_expected).unwrap(), compound_expected);
}

#[test]
fn ac7_whole_catalog_is_parse_preserved_idempotent_and_width_bounded() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("catalog");
    let mut paths = Vec::new();
    collect_sources(&root, &mut paths);
    paths.sort();

    let mut checked = 0usize;
    for path in paths {
        let source = fs::read_to_string(&path).unwrap();
        if path.extension().and_then(|extension| extension.to_str()) == Some("ken") {
            check_unit(&path.display().to_string(), &source);
            checked += 1;
        } else {
            for (index, body) in ken_fence_bodies(&source).into_iter().enumerate() {
                if parse_lossless(body).is_ok() {
                    check_unit(&format!("{} fence {index}", path.display()), body);
                    checked += 1;
                }
            }
        }
    }
    assert!(checked > 0, "catalog gate found no parseable Ken units");
}

fn check_unit(label: &str, source: &str) {
    let formatted = format_ken(source).unwrap_or_else(|error| panic!("{label}: {error:?}"));
    parse_lossless(&formatted).unwrap_or_else(|error| {
        panic!("{label}: formatted output does not parse: {error:?}\n{formatted}")
    });
    assert_eq!(
        ast_shape(source),
        ast_shape(&formatted),
        "{label}: AST drift"
    );
    assert_eq!(
        format_ken(&formatted).unwrap(),
        formatted,
        "{label}: formatter is not byte-idempotent"
    );
    for (line_index, line) in formatted.lines().enumerate() {
        assert!(
            display_width(line) <= 88 || indivisible_overflow(line),
            "{label}: breakable line {} is {} columns: {line}",
            line_index + 1,
            display_width(line)
        );
    }
}

fn indivisible_overflow(line: &str) -> bool {
    line.split_whitespace().any(|word| display_width(word) > 88)
}

fn collect_sources(directory: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
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
