use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn ken_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ken"))
}

fn fixture(name: &str, contents: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    fs::write(&path, contents).unwrap();
    path
}

#[test]
fn fmt_rewrites_plain_and_literate_sources_through_landed_entry_points() {
    let plain = fixture(
        "fmt_rewrite.ken",
        "fn id (f : Nat -> Nat) (x : Nat) : Nat = f   (x)\n",
    );
    let literate = fixture(
        "fmt_rewrite.ken.md",
        "Prose -> unchanged.\n```ken\nfn id (f : Nat -> Nat) (x : Nat) : Nat = f   (x)\n```\n",
    );
    let output = Command::new(ken_bin())
        .arg("fmt")
        .arg(&plain)
        .arg(&literate)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(plain).unwrap(),
        "fn id (f : Nat → Nat) (x : Nat) : Nat = f (x)\n"
    );
    assert_eq!(
        fs::read_to_string(literate).unwrap(),
        "Prose -> unchanged.\n```ken\nfn id (f : Nat → Nat) (x : Nat) : Nat = f (x)\n```\n"
    );
}

#[test]
fn fmt_check_names_every_offender_and_never_writes() {
    let first = fixture("fmt_check_first.ken", "const first   : Nat = Zero\n");
    let second = fixture("fmt_check_second.ken", "const second   : Nat = Zero\n");
    let before_first = fs::read(&first).unwrap();
    let before_second = fs::read(&second).unwrap();
    let output = Command::new(ken_bin())
        .arg("fmt")
        .arg("--check")
        .arg(&first)
        .arg(&second)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(first.to_str().unwrap()), "{stderr}");
    assert!(stderr.contains(second.to_str().unwrap()), "{stderr}");
    assert_eq!(fs::read(&first).unwrap(), before_first);
    assert_eq!(fs::read(&second).unwrap(), before_second);
}

#[test]
fn fmt_fails_loudly_for_unsupported_paths_and_must_parse_bodies() {
    let unsupported = fixture("fmt_unsupported.txt", "not Ken\n");
    let invalid = fixture(
        "fmt_invalid.ken.md",
        "```ken\nconst unfinished : Nat =\n```\n",
    );
    for (path, needle) in [
        (unsupported, "unsupported path"),
        (invalid, "formatting error"),
    ] {
        let output = Command::new(ken_bin())
            .arg("fmt")
            .arg(path)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains(needle));
    }
}

#[test]
#[ignore = "kenfmt strict frozen-corpus gate paused per operator 2026-07-13; catalog reformat is being reworked incrementally — re-enable after rework (see docs/program/IMPLEMENTATION-PROGRESS.md)"]
fn strict_frozen_corpus_gate_is_green() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut catalog = Vec::new();
    collect(&repository.join("catalog"), ".ken.md", &mut catalog);
    let mut rosetta = Vec::new();
    collect(&repository.join("examples/rosetta"), ".ken", &mut rosetta);
    catalog.sort();
    rosetta.sort();
    assert!(
        catalog.len() >= 14,
        "catalog literate corpus fell below floor 14 (observed {})",
        catalog.len()
    );
    assert!(
        rosetta.len() >= 16,
        "Rosetta corpus fell below floor 16 (observed {})",
        rosetta.len()
    );

    let boundary =
        repository.join("catalog/packages/Capability/Verify/ProofErasureBoundaryChecker.ken");
    assert!(boundary.is_file());
    let output = Command::new(ken_bin())
        .arg("fmt")
        .arg("--check")
        .args(&catalog)
        .arg(boundary)
        .args(&rosetta)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "frozen corpus is not canonical: {}",
        String::from_utf8_lossy(&output.stderr)
    );
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
