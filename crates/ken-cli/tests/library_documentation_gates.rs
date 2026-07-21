//! Wave 0 documentation gates (`docs/program/issues/DOC-W0.md` deliverable
//! 5, proposal "Documentation gates" 1/2/3/6), plus what librarian QA
//! (`thr_74hvpkqnxjp9q`) found the first cut left open:
//!
//! 1. the manifest covers every `library/` document and every manifest
//!    path exists;
//! 1b. every manifest record declares the complete required shape (kind,
//!     audience, authority, availability, sources, validation, owner —
//!     all non-empty) and no `path` repeats (AC1: a page whose fields
//!     can silently go missing is not "declaring what it is").
//! 2. internal links resolve to a real file **and a real anchor**
//!    (same-file or cross-file), and external links are syntactically
//!    well-formed;
//! 3. every manifest `sources` entry's path exists, and its `#anchor` (if
//!    any) names a real heading in that file — the drift gate D1 requires;
//! 6. every registered document labels an `availability` of exactly
//!    current/partial/planned/unavailable.
//!
//! Targeted `scripts/ken-cargo -p ken-cli` check, not an out-of-band
//! script (doc-leader kickoff, `thr_74hvpkqnxjp9q`). Each gate below is
//! proven to fail on a planted violation in the DOC-W0 handoff — see the
//! before/after pasted there; this file is the gate's resting (green)
//! state.

use std::collections::{BTreeSet, HashSet};
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    // crates/ken-cli -> repo root is two levels up.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

// --- a hand-rolled parser for library/manifest.toml's controlled subset ---
//
// Only what the manifest actually uses: a run of `[[document]]` tables,
// each with flat `key = "scalar"` fields and `key = [ "a", "b" ]` array
// fields (which may span multiple lines). Not a general TOML parser —
// deliberately, to avoid a new workspace dependency for a fixed, self-
// authored schema (the same "no new dependency for a controlled format"
// call `scripts/gen-progress.sh` makes for issue frontmatter).

#[derive(Debug, Clone, Default)]
struct DocEntry {
    path: String,
    kind: String,
    audience: Vec<String>,
    authority: String,
    availability: String,
    sources: Vec<String>,
    validation: Vec<String>,
    owner: String,
}

fn extract_quoted_strings(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = s;
    while let Some(open) = rest.find('"') {
        let after_open = &rest[open + 1..];
        let Some(close) = after_open.find('"') else {
            break;
        };
        out.push(after_open[..close].to_string());
        rest = &after_open[close + 1..];
    }
    out
}

fn parse_manifest(src: &str) -> Vec<DocEntry> {
    let mut entries = Vec::new();
    let mut current: Option<DocEntry> = None;
    let mut lines = src.lines().peekable();

    while let Some(raw_line) = lines.next() {
        let line = raw_line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if line == "[[document]]" {
            if let Some(entry) = current.take() {
                entries.push(entry);
            }
            current = Some(DocEntry::default());
            continue;
        }
        let Some(entry) = current.as_mut() else {
            continue;
        };
        let Some((key, mut value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        value = value.trim();

        // Multi-line array: opens with `[` but has no closing `]` on this line.
        let mut array_text = String::new();
        if value.starts_with('[') && !value.contains(']') {
            array_text.push_str(value);
            array_text.push('\n');
            for cont in lines.by_ref() {
                array_text.push_str(cont);
                array_text.push('\n');
                if cont.contains(']') {
                    break;
                }
            }
            value = array_text.trim();
        }

        match key {
            "path" => entry.path = extract_quoted_strings(value).pop().unwrap_or_default(),
            "kind" => entry.kind = extract_quoted_strings(value).pop().unwrap_or_default(),
            "audience" => entry.audience = extract_quoted_strings(value),
            "authority" => {
                entry.authority = extract_quoted_strings(value).pop().unwrap_or_default()
            }
            "availability" => {
                entry.availability = extract_quoted_strings(value).pop().unwrap_or_default()
            }
            "sources" => entry.sources = extract_quoted_strings(value),
            "validation" => entry.validation = extract_quoted_strings(value),
            "owner" => entry.owner = extract_quoted_strings(value).pop().unwrap_or_default(),
            _ => {}
        }
    }
    if let Some(entry) = current.take() {
        entries.push(entry);
    }
    entries
}

fn load_manifest() -> Vec<DocEntry> {
    let manifest_path = repo_root().join("library/manifest.toml");
    let src = std::fs::read_to_string(&manifest_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", manifest_path.display()));
    let entries = parse_manifest(&src);
    assert!(
        !entries.is_empty(),
        "library/manifest.toml parsed to zero [[document]] entries — parser or manifest is broken"
    );
    entries
}

/// Every `.md` file under `library/`, repo-relative with forward slashes.
fn library_markdown_files() -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![repo_root().join("library")];
    let root = repo_root();
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()))
        {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
                let rel = path.strip_prefix(&root).unwrap();
                out.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    out.sort();
    out
}

// GitHub-style heading slug: lowercase; drop everything that is not a
// letter, digit, space, hyphen, or underscore; spaces -> hyphens. Matches
// the anchors already used by `research/librarian-documentation-program-
// proposal.md`'s own worked example
// (`docs/PRINCIPLES.md#1-ken-is-a-software-engineering-language-not-a-programming-language`).
fn slugify(heading: &str) -> String {
    let lower = heading.trim().to_lowercase();
    let filtered: String = lower
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
        .collect();
    filtered.replace(' ', "-")
}

/// Every heading anchor a file exposes. A heading may be inside a
/// blockquote (`> ### …`, as `docs/PRINCIPLES.md`'s transient-principle
/// block uses) — strip one leading `> ` before checking for `#`.
fn heading_anchors(contents: &str) -> BTreeSet<String> {
    let mut anchors = BTreeSet::new();
    for line in contents.lines() {
        let mut l = line.trim_start();
        if let Some(rest) = l.strip_prefix("> ") {
            l = rest;
        }
        if l.starts_with('#') {
            let text = l.trim_start_matches('#').trim();
            if !text.is_empty() {
                anchors.insert(slugify(text));
            }
        }
    }
    anchors
}

fn split_source(source: &str) -> (&str, Option<&str>) {
    match source.split_once('#') {
        Some((path, anchor)) => (path, Some(anchor)),
        None => (source, None),
    }
}

// --- gate 1: manifest coverage + path existence ---------------------------

#[test]
fn gate1_manifest_covers_every_document_and_every_path_exists() {
    let entries = load_manifest();
    let root = repo_root();

    let registered: HashSet<String> = entries.iter().map(|e| e.path.clone()).collect();
    let on_disk: Vec<String> = library_markdown_files();

    let mut missing_from_manifest = Vec::new();
    for path in &on_disk {
        if !registered.contains(path) {
            missing_from_manifest.push(path.clone());
        }
    }
    assert!(
        missing_from_manifest.is_empty(),
        "library/*.md file(s) with no manifest.toml [[document]] entry: {missing_from_manifest:?}"
    );

    let mut dangling_entries = Vec::new();
    for entry in &entries {
        assert!(!entry.path.is_empty(), "a [[document]] entry has no `path`");
        if !root.join(&entry.path).is_file() {
            dangling_entries.push(entry.path.clone());
        }
    }
    assert!(
        dangling_entries.is_empty(),
        "manifest.toml [[document]] path(s) that do not exist on disk: {dangling_entries:?}"
    );
}

// AC1 ("a new page cannot land without declaring what it is, what grounds
// it, and how its currency is checked"): every field the manifest record
// promises must actually be present, and the manifest's own "exactly one
// [[document]] entry" contract must hold. Librarian QA (thr_74hvpkqnxjp9q,
// finding 2): a field silently missing must fail this gate even when
// every other gate stays green — `sources = []` in particular means "what
// grounds it" is not mechanically declared, so `sources` is required
// non-empty, not merely present.
#[test]
fn gate_manifest_records_declare_the_complete_required_shape_and_unique_paths() {
    let entries = load_manifest();
    let mut bad = Vec::new();
    let mut seen_paths: HashSet<String> = HashSet::new();

    for entry in &entries {
        let label = if entry.path.is_empty() {
            "<no path>".to_string()
        } else {
            entry.path.clone()
        };
        if entry.path.is_empty() {
            bad.push(format!("{label}: missing `path`"));
        }
        if entry.kind.is_empty() {
            bad.push(format!("{label}: missing `kind`"));
        }
        if entry.audience.is_empty() {
            bad.push(format!("{label}: missing `audience`"));
        }
        if entry.authority.is_empty() {
            bad.push(format!("{label}: missing `authority`"));
        }
        if entry.availability.is_empty() {
            bad.push(format!("{label}: missing `availability`"));
        }
        if entry.sources.is_empty() {
            bad.push(format!(
                "{label}: missing `sources` — what grounds this page is not declared"
            ));
        }
        if entry.validation.is_empty() {
            bad.push(format!("{label}: missing `validation`"));
        }
        if entry.owner.is_empty() {
            bad.push(format!("{label}: missing `owner`"));
        }
        if !entry.path.is_empty() && !seen_paths.insert(entry.path.clone()) {
            bad.push(format!(
                "{label}: duplicate [[document]] entry — the manifest promises exactly one"
            ));
        }
    }

    assert!(
        bad.is_empty(),
        "manifest record(s) with a missing required field or a duplicate path:\n{}",
        bad.join("\n")
    );
}

// --- gate 2: links valid ---------------------------------------------------

fn markdown_links(contents: &str) -> Vec<String> {
    // Inline links only: `[text](target)`. Sufficient for the small,
    // hand-authored Wave 0 corpus; no reference-style links are in use.
    let mut out = Vec::new();
    let bytes = contents.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'[' {
            if let Some(close_bracket) = contents[i..].find(']') {
                let after = i + close_bracket + 1;
                if contents.as_bytes().get(after) == Some(&b'(') {
                    if let Some(close_paren) = contents[after..].find(')') {
                        let target = &contents[after + 1..after + close_paren];
                        out.push(target.to_string());
                        i = after + close_paren + 1;
                        continue;
                    }
                }
            }
        }
        i += 1;
    }
    out
}

fn is_well_formed_external_url(url: &str) -> bool {
    let Some((scheme, rest)) = url.split_once("://") else {
        return false;
    };
    if scheme != "http" && scheme != "https" {
        return false;
    }
    let host = rest.split(['/', '?', '#']).next().unwrap_or("");
    !host.is_empty() && host.contains('.')
}

#[test]
fn gate2_links_are_valid() {
    // Librarian QA (thr_74hvpkqnxjp9q, finding 3): a link's `#anchor` must
    // resolve too, same-file or cross-file — not just the file it points
    // at. `introduction.md#no-such-heading` is broken exactly like
    // `nonexistent.md` is; both mean the reader lands nowhere real.
    let root = repo_root();
    let mut broken = Vec::new();

    for rel_path in library_markdown_files() {
        let abs_path = root.join(&rel_path);
        let contents = std::fs::read_to_string(&abs_path).expect("read library markdown file");
        let file_dir = abs_path.parent().expect("file has a parent dir");
        let own_anchors = heading_anchors(&contents);

        for link in markdown_links(&contents) {
            if link.starts_with("http://") || link.starts_with("https://") {
                if !is_well_formed_external_url(&link) {
                    broken.push(format!("{rel_path}: malformed external link {link:?}"));
                }
                continue;
            }

            let (target_path, anchor) = split_source(&link);

            if target_path.is_empty() {
                // Same-file anchor-only link, e.g. `#no-such-heading`.
                if let Some(anchor) = anchor {
                    if !own_anchors.contains(anchor) {
                        broken.push(format!(
                            "{rel_path}: same-file anchor '#{anchor}' not found (have: {own_anchors:?})"
                        ));
                    }
                }
                continue;
            }

            let resolved = file_dir.join(target_path);
            if !resolved.exists() {
                broken.push(format!(
                    "{rel_path}: link target does not exist: {link:?} (resolved {})",
                    resolved.display()
                ));
                continue;
            }
            if let Some(anchor) = anchor {
                let target_contents =
                    std::fs::read_to_string(&resolved).expect("read link target file");
                let target_anchors = heading_anchors(&target_contents);
                if !target_anchors.contains(anchor) {
                    broken.push(format!(
                        "{rel_path}: link anchor '#{anchor}' not found in {target_path} \
                         (have: {target_anchors:?})"
                    ));
                }
            }
        }
    }
    assert!(broken.is_empty(), "broken link(s):\n{}", broken.join("\n"));
}

// --- gate 3: every manifest `sources` path + anchor exists ----------------

#[test]
fn gate3_every_manifest_source_path_and_anchor_exists() {
    let entries = load_manifest();
    let root = repo_root();
    let mut bad = Vec::new();

    for entry in &entries {
        for source in &entry.sources {
            let (path, anchor) = split_source(source);
            let abs = root.join(path);
            if !abs.is_file() {
                bad.push(format!(
                    "{}: source path does not exist: {source:?}",
                    entry.path
                ));
                continue;
            }
            if let Some(anchor) = anchor {
                let contents = std::fs::read_to_string(&abs).expect("read cited source file");
                let anchors = heading_anchors(&contents);
                if !anchors.contains(anchor) {
                    bad.push(format!(
                        "{}: source anchor '#{anchor}' not found in {path} (have: {:?})",
                        entry.path, anchors
                    ));
                }
            }
        }
    }
    assert!(
        bad.is_empty(),
        "manifest source(s) with a missing path or stale anchor:\n{}",
        bad.join("\n")
    );
}

// --- gate 6: every document labels a valid availability -------------------

#[test]
fn gate6_every_document_labels_a_valid_availability() {
    const VALID: &[&str] = &["current", "partial", "planned", "unavailable"];
    let entries = load_manifest();
    let mut bad = Vec::new();
    for entry in &entries {
        if !VALID.contains(&entry.availability.as_str()) {
            bad.push(format!(
                "{}: availability {:?} is not one of {VALID:?}",
                entry.path, entry.availability
            ));
        }
    }
    assert!(
        bad.is_empty(),
        "document(s) with a missing/invalid availability label:\n{}",
        bad.join("\n")
    );
}

// --- AC3: STATUS.md generation is idempotent on an unchanged tree ---------

#[test]
fn status_md_generation_is_idempotent() {
    let root = repo_root();
    let script = root.join("scripts/gen-doc-status.sh");
    let output = std::process::Command::new("bash")
        .arg(&script)
        .arg("--check")
        .current_dir(&root)
        .output()
        .expect("run scripts/gen-doc-status.sh --check");
    assert!(
        output.status.success(),
        "library/STATUS.md is stale relative to library/manifest.toml — rerun \
         scripts/gen-doc-status.sh. stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

// --- authority class is one of D1's closed set -----------------------------

#[test]
fn every_document_declares_a_closed_authority_class() {
    const VALID: &[&str] = &[
        "derived-reference",
        "explanatory",
        "tutorial",
        "how-to",
        "status",
        "normative-pointer",
    ];
    let entries = load_manifest();
    let mut bad = Vec::new();
    for entry in &entries {
        if !VALID.contains(&entry.authority.as_str()) {
            bad.push(format!(
                "{}: authority {:?} is not one of the D1 closed set {VALID:?}",
                entry.path, entry.authority
            ));
        }
    }
    assert!(
        bad.is_empty(),
        "document(s) with an invalid authority class:\n{}",
        bad.join("\n")
    );
}

#[test]
fn slugify_matches_the_proposals_own_worked_anchor() {
    // research/librarian-documentation-program-proposal.md's own manifest
    // example cites this exact anchor — pin the algorithm against it so a
    // future slugify change can't silently drift from the citations
    // already written against it.
    assert_eq!(
        slugify("1. Ken is a *software-engineering* language, not a programming language"),
        "1-ken-is-a-software-engineering-language-not-a-programming-language"
    );
}
