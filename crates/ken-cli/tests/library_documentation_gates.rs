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
//! 1c. `validation` is a closed, known vocabulary that names exactly all
//!     registered record-validation gates that apply to that record — not
//!     free prose (AC1: "how its currency is checked" must be mechanical).
//! 1d. no manifest scalar/array-string value contains a literal `|` —
//!     the generator's row transport delimiter — so gate and generator
//!     can't silently disagree about where one field ends and the next
//!     begins.
//! 2. internal links resolve to a real file **and a real anchor**
//!    (same-file or cross-file), and external links are syntactically
//!    well-formed;
//! 3. every manifest `sources` entry's path exists, and its `#anchor` (if
//!    any) names a real heading in that file — the drift gate D1 requires;
//! 6. every registered document labels an `availability` of exactly
//!    current/partial/planned/unavailable.
//! 7. every manifest `sources` entry cited by a non-`status`-kind document
//!    is byte-unchanged between `library/REVISION` and `HEAD` — `revision_
//!    resolved()` (DOC-W0) only proves `REVISION` names a real ancestor
//!    commit; it never reads a cited source's bytes AT that revision, so
//!    it is blind to content drift under an unchanged heading
//!    (`DOC-CURRENCY-ANCHOR`). A `status`-kind document (`STATUS.md`) is
//!    exempt from this token entirely (it carries `generated-current`
//!    instead — always regenerated fresh, so idempotency subsumes it), and
//!    `library/REVISION` itself is the one path-level exemption (self-
//!    referential by construction); nothing else is exempted by path. A
//!    symlinked source is rejected outright rather than diffed through —
//!    `git diff` on a symlink path compares the symlink's own target-path
//!    blob, not the resolved file's content. Enforced in `scripts/gen-doc-
//!    status.sh`, verified here by driving the real script against
//!    synthetic fixtures — also covers the bootstrap case: `REVISION` must
//!    name a point at or after `library/manifest.toml`'s own introduction,
//!    not merely an ancestor.
//!
//! Targeted `scripts/ken-cargo -p ken-cli` check, not an out-of-band
//! script (doc-leader kickoff, `thr_74hvpkqnxjp9q`). Each gate below is
//! proven to fail on a planted violation in the DOC-W0 handoff — see the
//! before/after pasted there; this file is the gate's resting (green)
//! state.
//!
//! Two substrate-soundness properties Architect review added
//! (`dec_4hrvf6bkce8fk`): this parser's `[[document]]`/`key =`
//! recognition is anchored at column 0, byte-identical to
//! `scripts/gen-doc-status.sh`'s awk grammar, so a manifest record either
//! parses the same way on both sides or is rejected by gate 1b on
//! neither — the two can no longer silently disagree. And every path
//! (document `path`, `sources`, internal links) resolves through
//! `resolve_confined`, which rejects an absolute target or a `..` climb
//! past the repository root before ever touching the filesystem, so an
//! existing host file outside the repo can't satisfy a manifest entry.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Component, Path, PathBuf};

fn repo_root() -> PathBuf {
    // crates/ken-cli -> repo root is two levels up.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

// --- repository-path confinement (Architect finding 2, thr_74hvpkqnxjp9q) -
//
// `root.join(rel)` has a sharp Rust `PathBuf` gotcha: if `rel` is
// ABSOLUTE, `join` doesn't concatenate — it REPLACES the base entirely
// (`PathBuf::push` docs). So a manifest `path`/`source` or an internal
// link of `/etc/passwd` silently resolved to the real host file
// `/etc/passwd`, existence-checks and all — an anti-drift gate that is
// host-dependent, not repository-confined. A lexical `..` climb has the
// same effect without even needing an absolute string. Fixed by
// normalizing PURELY LEXICALLY (no filesystem access, so it rejects an
// escape even when the target doesn't exist) and requiring the result
// stay under the repository root.

fn lexically_normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

// The lexical check alone is not enough: every consumer (`Path::is_file`,
// `Path::exists`, `read_to_string`, `path.is_dir`) resolves symlinks when
// it touches the filesystem, so an in-repository symlink whose target is
// outside the repository passes the lexical prefix check (the symlink
// itself is an ordinary path component under `library/`) and then reads
// straight through to a real host file — a green-but-host-dependent bypass
// of the same confinement boundary (Architect, `thr_74hvpkqnxjp9q`, third
// round). Fixed by canonicalizing whenever the lexically-confined target
// exists (canonicalization fully resolves symlinks) and re-checking
// containment against the canonicalized repository root. A target that
// does not exist cannot leak anything yet — the lexical check already
// rejected an absolute/`..` escape for it, and the "does this exist"
// checks downstream correctly report the rest as missing.
fn is_symlink_escape(path: &Path, repo_root: &Path) -> bool {
    match (path.canonicalize(), repo_root.canonicalize()) {
        (Ok(canon), Ok(canon_root)) => !canon.starts_with(&canon_root),
        _ => false,
    }
}

/// Resolve `rel` against `base`, confined to `repo_root`: rejects an
/// absolute `rel`, any `..` climb that lands outside `repo_root`, and any
/// existing target a symlink component resolves outside `repo_root`.
/// Returns the normalized absolute path if it stays confined, `None`
/// otherwise. A legitimate cross-tree relative link (e.g.
/// `library/README.md` citing `../catalog/packages/README.md`) still
/// resolves fine — only an escape past `repo_root` itself is rejected.
fn resolve_confined(base: &Path, rel: &str, repo_root: &Path) -> Option<PathBuf> {
    if rel.is_empty() {
        return None;
    }
    let normalized = lexically_normalize(&base.join(rel));
    let repo_root_norm = lexically_normalize(repo_root);
    if !normalized.starts_with(&repo_root_norm) {
        return None;
    }
    if is_symlink_escape(&normalized, repo_root) {
        return None;
    }
    Some(normalized)
}

fn resolve_controlled_path(repo_root: &Path, rel: &str, owner: &str) -> PathBuf {
    resolve_confined(repo_root, rel, repo_root).unwrap_or_else(|| {
        panic!(
            "{owner}: controlled path {rel:?} is absolute, escapes the repository, \
             or resolves through an escaping symlink"
        )
    })
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
    // Architect finding 1 (thr_74hvpkqnxjp9q): this parser used to `.trim()`
    // every line before recognizing a `[[document]]` header or a `key =`
    // field, but `gen-doc-status.sh`'s awk companion anchors both at
    // column 0 (`/^\[\[document\]\]/`, `/^path[[:space:]]*=/`, …) — an
    // INDENTED field passed this gate while the generator silently
    // dropped it. The two must accept identical input. Fixed here by
    // matching awk's column-0 anchoring exactly: only a comment/blank
    // check trims; `[[document]]` and `key =` recognition run against the
    // UNTRIMMED line, so a leading space, tab, or anything else before
    // the token makes it invisible to both parsers alike, not just one.
    let mut entries = Vec::new();
    let mut current: Option<DocEntry> = None;
    let mut lines = src.lines().peekable();

    while let Some(raw_line) = lines.next() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if raw_line.starts_with("[[document]]") {
            if let Some(entry) = current.take() {
                entries.push(entry);
            }
            current = Some(DocEntry::default());
            continue;
        }
        let Some(entry) = current.as_mut() else {
            continue;
        };
        // Column-0 anchor: an indented `key = value` line is not a field
        // in either parser (see the fn-level note above).
        if raw_line.starts_with(char::is_whitespace) {
            continue;
        }
        let Some((key, mut value)) = raw_line.split_once('=') else {
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

/// Result of walking `library/`: every `.md` file found (repo-relative,
/// forward slashes), and every symlink found (same form) — file or
/// directory, at any depth.
struct LibraryWalk {
    markdown_files: Vec<String>,
    symlinks: Vec<String>,
}

/// Walks `library/`, never following a symlink (`DirEntry::file_type()`
/// reports the symlink itself, unlike `path.is_dir()`/`path.is_file()`,
/// which follow it — so a symlinked directory is never descended into and
/// a symlinked file is never opened). Architect finding (`thr_74hvpkqnxjp9q`,
/// fourth round): NOT following a symlink is not the same as REJECTING
/// one. An earlier fix made `library_markdown_files` silently `continue`
/// past any symlink — safe against the escape, but it made every symlink
/// under `library/` invisible to gate 1 rather than invalid, so an
/// unregistered `library/rogue.md` symlink (or worse, `library/guide ->
/// ../catalog/guide`, smuggling the not-yet-fence-gated guide tree under
/// the product portal ahead of its Wave-0 ordering constraint) would pass
/// every coverage gate simply by never being seen. Fixed by recording
/// every symlink encountered instead of dropping it, so gate 1 can fail
/// closed on it explicitly.
fn walk_library() -> LibraryWalk {
    let mut markdown_files = Vec::new();
    let mut symlinks = Vec::new();
    let mut stack = vec![repo_root().join("library")];
    let root = repo_root();
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()))
        {
            let entry = entry.expect("dir entry");
            let file_type = entry.file_type().expect("dir entry file type");
            let path = entry.path();
            let rel = path.strip_prefix(&root).unwrap().to_string_lossy().replace('\\', "/");
            if file_type.is_symlink() {
                symlinks.push(rel);
                continue;
            }
            if file_type.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
                markdown_files.push(rel);
            }
        }
    }
    markdown_files.sort();
    symlinks.sort();
    LibraryWalk { markdown_files, symlinks }
}

/// Every `.md` file under `library/`, repo-relative with forward slashes.
fn library_markdown_files() -> Vec<String> {
    walk_library().markdown_files
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

fn check_manifest_coverage() {
    let entries = load_manifest();
    let root = repo_root();

    let walk = walk_library();
    // Architect finding (thr_74hvpkqnxjp9q, fourth round): a symlink under
    // `library/` must fail this gate, not be silently excluded from it —
    // an unregistered symlink otherwise passes coverage by never being
    // seen at all. Fail closed and name every one found.
    assert!(
        walk.symlinks.is_empty(),
        "library/ contains symlink(s), which this inventory rejects rather \
         than silently omits — a symlink cannot be a manifest-covered \
         document nor a container this walk descends into: {:?}",
        walk.symlinks
    );

    let registered: HashSet<String> = entries.iter().map(|e| e.path.clone()).collect();
    let on_disk: Vec<String> = walk.markdown_files;

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

    // Architect finding 2: a document `path` must resolve UNDER
    // `library/`, confined to the repo — reject absolute paths and `..`
    // escapes before ever touching the filesystem, so an existing host
    // file outside the repo can't satisfy a manifest entry.
    let library_root = root.join("library");
    let mut escaping_entries = Vec::new();
    let mut dangling_entries = Vec::new();
    for entry in &entries {
        assert!(!entry.path.is_empty(), "a [[document]] entry has no `path`");
        match resolve_confined(&root, &entry.path, &root) {
            Some(resolved) if resolved.starts_with(&library_root) => {
                if !resolved.is_file() {
                    dangling_entries.push(entry.path.clone());
                }
            }
            _ => escaping_entries.push(entry.path.clone()),
        }
    }
    assert!(
        escaping_entries.is_empty(),
        "manifest.toml [[document]] path(s) that are absolute, escape the \
         repository, or fall outside library/: {escaping_entries:?}"
    );
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
fn invalid_kind_violations(entries: &[DocEntry]) -> Vec<String> {
    const VALID_KINDS: &[&str] = &["explanatory", "portal", "reference", "status", "tutorial"];
    let mut bad = Vec::new();

    for entry in entries {
        if !entry.kind.is_empty() && !VALID_KINDS.contains(&entry.kind.as_str()) {
            let label = if entry.path.is_empty() {
                "<no path>"
            } else {
                &entry.path
            };
            bad.push(format!(
                "{label}: kind {:?} is not one of {VALID_KINDS:?}",
                entry.kind
            ));
        }
    }

    bad
}

fn check_document_kinds() {
    let bad = invalid_kind_violations(&load_manifest());
    assert!(
        bad.is_empty(),
        "manifest document-kind violation(s):\n{}",
        bad.join("\n")
    );
}

fn check_manifest_completeness() {
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
        "manifest record(s) with a missing/invalid required field or a duplicate path:\n{}",
        bad.join("\n")
    );
}

#[test]
fn invalid_kind_detector_rejects_case_variant() {
    let entries = [DocEntry {
        path: "library/fixture.md".to_string(),
        kind: "Status".to_string(),
        ..DocEntry::default()
    }];

    assert_eq!(
        invalid_kind_violations(&entries),
        vec!["library/fixture.md: kind \"Status\" is not one of \
             [\"explanatory\", \"portal\", \"reference\", \"status\", \"tutorial\"]"
            .to_string()]
    );
}

// `validation` names all registered record-validation gates that apply to a
// record. The registry below is the sole owner of each token, its applicability
// predicate, and the executable runner that enforces it. The manifest agreement
// test derives both the closed vocabulary and each record's exact required set
// from that registry.
//
// Librarian QA (thr_15yrvjrpap9td, first pass, finding 1): an earlier cut
// gave EVERY document `source-currency`, including `library/STATUS.md` —
// but `scripts/gen-doc-status.sh`'s implementation blanket-skipped every
// `library/`-prefixed source, so `STATUS.md`'s own declared `sources`
// (`manifest.toml`, `REVISION`) were silently never checked at all. That
// is a hidden exception, not the issue's sanctioned "visibly weakened"
// branch (AC-1). Fixed by making the exemption VISIBLE here instead:
// `source-currency` does not apply to a `kind = "status"` document at
// all — its currency is what `generated-current` (idempotency: it is
// always regenerated fresh from the current working tree) already
// establishes, which subsumes "unchanged since REVISION" for a document
// that has no independent existence apart from its own generation. Every
// OTHER document's `library/`-referencing sources (none exist today, but
// none are exempted by path either) are bound like any other citation.
struct ValidationGate {
    token: &'static str,
    applies: fn(&DocEntry) -> bool,
    run: fn(),
}

fn applies_to_every_record(_: &DocEntry) -> bool {
    true
}

fn applies_to_status_records(entry: &DocEntry) -> bool {
    entry.kind == "status"
}

fn applies_to_non_status_records(entry: &DocEntry) -> bool {
    entry.kind != "status"
}

const VALIDATION_GATES: &[ValidationGate] = &[
    ValidationGate {
        token: "manifest-coverage",
        applies: applies_to_every_record,
        run: check_manifest_coverage,
    },
    ValidationGate {
        token: "manifest-completeness",
        applies: applies_to_every_record,
        run: check_manifest_completeness,
    },
    ValidationGate {
        token: "document-kind",
        applies: applies_to_every_record,
        run: check_document_kinds,
    },
    ValidationGate {
        token: "checked-examples",
        applies: applies_to_checked_example_records,
        run: check_checked_examples,
    },
    ValidationGate {
        token: "links",
        applies: applies_to_every_record,
        run: check_links,
    },
    ValidationGate {
        token: "source-anchors",
        applies: applies_to_every_record,
        run: check_source_anchors,
    },
    ValidationGate {
        token: "availability-label",
        applies: applies_to_every_record,
        run: check_availability_labels,
    },
    ValidationGate {
        token: "authority-class",
        applies: applies_to_every_record,
        run: check_authority_classes,
    },
    ValidationGate {
        token: "source-currency",
        applies: applies_to_non_status_records,
        run: check_source_currency,
    },
    ValidationGate {
        token: "generated-current",
        applies: applies_to_status_records,
        run: check_generated_current,
    },
    ValidationGate {
        token: "transport-delimiter",
        applies: applies_to_every_record,
        run: check_transport_delimiter,
    },
];

#[test]
fn registered_record_validation_gates_run() {
    for gate in VALIDATION_GATES {
        (gate.run)();
    }
}

fn status_record_population_violations(entries: &[DocEntry]) -> Vec<String> {
    let status_records: BTreeSet<&str> = entries
        .iter()
        .filter(|entry| entry.kind == "status")
        .map(|entry| entry.path.as_str())
        .collect();

    if status_records == BTreeSet::from(["library/STATUS.md"]) {
        Vec::new()
    } else {
        vec![format!(
            "status-kind records were {status_records:?}; expected exactly \
             {{\"library/STATUS.md\"}}"
        )]
    }
}

#[test]
fn gate_validation_tokens_are_closed_and_match_applicable_checks() {
    let entries = load_manifest();
    let mut bad = Vec::new();
    let known: BTreeSet<&str> = VALIDATION_GATES.iter().map(|gate| gate.token).collect();
    let status_record_violations = status_record_population_violations(&entries);

    assert_eq!(
        known.len(),
        VALIDATION_GATES.len(),
        "registered record-validation gate tokens must be unique"
    );
    assert!(
        status_record_violations.is_empty(),
        "check_generated_current is hard-coded to gen-doc-status.sh/library/STATUS.md; a \
         second status-kind record needs its own registered runner, not `generated-current`:\n{}",
        status_record_violations.join("\n")
    );

    for entry in &entries {
        for tok in &entry.validation {
            if !known.contains(tok.as_str()) {
                bad.push(format!(
                    "{}: unknown validation token {tok:?} (known: {known:?})",
                    entry.path
                ));
            }
        }
        let declared: BTreeSet<&str> = entry.validation.iter().map(String::as_str).collect();
        let required: BTreeSet<&str> = VALIDATION_GATES
            .iter()
            .filter(|gate| (gate.applies)(entry))
            .map(|gate| gate.token)
            .collect();
        if declared != required {
            bad.push(format!(
                "{}: validation {declared:?} does not exactly match the applicable checks {required:?}",
                entry.path
            ));
        }
    }

    assert!(
        bad.is_empty(),
        "document(s) with an unknown or incomplete validation list:\n{}",
        bad.join("\n")
    );
}

#[test]
fn status_record_population_detector_rejects_second_status_record() {
    let entries = [
        DocEntry {
            path: "library/STATUS.md".to_string(),
            kind: "status".to_string(),
            ..DocEntry::default()
        },
        DocEntry {
            path: "library/second-status.md".to_string(),
            kind: "status".to_string(),
            ..DocEntry::default()
        },
    ];

    let bad = status_record_population_violations(&entries);
    assert_eq!(bad.len(), 1, "expected one population violation: {bad:?}");
    assert!(
        bad[0].contains("library/STATUS.md") && bad[0].contains("library/second-status.md"),
        "violation must name the complete status-record population: {bad:?}"
    );
}

// Librarian QA (thr_74hvpkqnxjp9q, fourth pass): switching the generator's
// row transport from tab to `|` fixed the empty-field collapse but
// introduced an unguarded delimiter collision — `|` is legal in the
// manifest's quoted TOML subset and in a real filename
// (`library/pipe|page.md` regenerated a STATUS row with every column
// shifted, exactly the green-but-generator-disagrees class this fold
// exists to close). Chosen fix (option (b) from the finding): make the
// controlled grammar explicitly reject `|` in every transported scalar,
// enforced here AND independently in `gen-doc-status.sh` itself (so a
// direct script run, not just this gate, fails closed).
fn all_string_fields(entry: &DocEntry) -> Vec<(&'static str, &str)> {
    let mut fields = vec![
        ("path", entry.path.as_str()),
        ("kind", entry.kind.as_str()),
        ("authority", entry.authority.as_str()),
        ("availability", entry.availability.as_str()),
        ("owner", entry.owner.as_str()),
    ];
    fields.extend(entry.audience.iter().map(|s| ("audience", s.as_str())));
    fields.extend(entry.sources.iter().map(|s| ("sources", s.as_str())));
    fields.extend(entry.validation.iter().map(|s| ("validation", s.as_str())));
    fields
}

fn check_transport_delimiter() {
    let entries = load_manifest();
    let mut bad = Vec::new();
    for entry in &entries {
        for (field_name, value) in all_string_fields(entry) {
            if value.contains('|') {
                bad.push(format!(
                    "{}: `{field_name}` contains a literal '|', which \
                     gen-doc-status.sh's row transport uses as its field \
                     separator: {value:?}",
                    entry.path
                ));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "manifest scalar(s) containing the transport delimiter '|':\n{}",
        bad.join("\n")
    );
}

// Librarian QA (thr_15yrvjrpap9td, third pass): a `key = value`-shaped line
// at column 0 INSIDE a still-open multi-line `sources = [ ... ]` array
// desyncs `parse_manifest` (above) from `gen-doc-status.sh`'s awk parsers.
// `parse_manifest`'s array continuation (see its `multi-line array` branch)
// never reinterprets a line as a field once inside an open array — it just
// accumulates raw text and quote-extracts from it, so `kind = "status"`
// sitting inside the array is swallowed as literal text and `"status"` is
// extracted as a spurious extra `sources` entry, while this record's real,
// final `kind` stays whatever the last PROPER `kind =` line (outside the
// array) set it to. `gen-doc-status.sh`'s awk instead matches
// `/^kind[[:space:]]*=/` unconditionally at column 0, with no notion of
// "inside an open array" — so the same line flips ITS view of the
// document's `kind` instead. Live repro (librarian, scratch commit
// `1fab9704`): this spoofed a document's `kind` to `status` in the awk's
// eyes only, exempting it from the new content-currency gate and silently
// dropping a genuinely drifted cited source. Rejected outright — closing
// the ambiguity is simpler than making three independent parsers agree on
// how to resolve it.
fn field_lines_inside_open_arrays(src: &str) -> Vec<String> {
    let mut bad = Vec::new();
    let mut open = false;
    for (i, raw_line) in src.lines().enumerate() {
        if open {
            if let Some((key, _)) = raw_line.split_once('=') {
                let key = key.trim();
                if !key.is_empty()
                    && raw_line.starts_with(|c: char| c.is_ascii_lowercase())
                    && key.chars().all(|c| c.is_ascii_lowercase() || c == '_')
                {
                    bad.push(format!("line {}: {raw_line:?}", i + 1));
                }
            }
            if raw_line.contains(']') {
                open = false;
            }
            continue;
        }
        if let Some((_, value)) = raw_line.split_once('=') {
            let value = value.trim();
            if value.starts_with('[') && !value.contains(']') {
                open = true;
            }
        }
    }
    bad
}

#[test]
fn gate_manifest_rejects_a_field_line_inside_an_open_multiline_array() {
    let manifest_path = repo_root().join("library/manifest.toml");
    let src = std::fs::read_to_string(&manifest_path).expect("read manifest.toml");
    let bad = field_lines_inside_open_arrays(&src);
    assert!(
        bad.is_empty(),
        "manifest.toml has a field-looking line inside a still-open \
         multi-line array — this is the exact shape that desyncs \
         parse_manifest from gen-doc-status.sh's awk parsers:\n{}",
        bad.join("\n")
    );
}

// Mutation proof for the detector itself, on a synthetic manifest string —
// proves the mechanism fires on the librarian's exact reported shape,
// rather than merely asserting the real manifest happens to be clean today.
#[test]
fn field_lines_inside_open_arrays_detects_the_reported_shape() {
    let clean = "[[document]]\npath = \"library/fixture.md\"\nkind = \"portal\"\n\
        authority = \"explanatory\"\navailability = \"current\"\nsources = [\n  \
        \"docs/foo.md\",\n]\n";
    assert!(
        field_lines_inside_open_arrays(clean).is_empty(),
        "detector false-positived on a clean manifest record"
    );

    let malformed = "[[document]]\npath = \"library/fixture.md\"\nkind = \"portal\"\n\
        authority = \"explanatory\"\navailability = \"current\"\nsources = [\nkind = \"status\"\n  \
        \"docs/foo.md\",\n]\n";
    let bad = field_lines_inside_open_arrays(malformed);
    assert_eq!(
        bad.len(),
        1,
        "expected exactly one offending line, got: {bad:?}"
    );
    assert!(
        bad[0].contains("kind") && bad[0].contains("status"),
        "expected the offending line to name the spoofed kind, got: {bad:?}"
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

fn check_links() {
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

            // Architect finding 2: confine link resolution to the repo —
            // an absolute target or a `..` climb past `root` must not
            // resolve to a real host file outside it.
            let Some(resolved) = resolve_confined(file_dir, target_path, &root) else {
                broken.push(format!(
                    "{rel_path}: link target is absolute or escapes the repository: {link:?}"
                ));
                continue;
            };
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

fn check_source_anchors() {
    let entries = load_manifest();
    let root = repo_root();
    let mut bad = Vec::new();

    for entry in &entries {
        for source in &entry.sources {
            let (path, anchor) = split_source(source);
            // Architect finding 2: confine source resolution to the repo.
            let Some(abs) = resolve_confined(&root, path, &root) else {
                bad.push(format!(
                    "{}: source path is absolute or escapes the repository: {source:?}",
                    entry.path
                ));
                continue;
            };
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

fn check_availability_labels() {
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

fn run_status_generator_check() {
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

fn check_source_currency() {
    run_status_generator_check();
}

fn check_generated_current() {
    run_status_generator_check();
}

// --- library/REVISION resolves in a SHALLOW clone (Steward, PR #830) ------
//
// The registered generated-current runner above only ever runs against this
// worktree, which has full history — it could not have caught PR #830's
// CI failure. CI's default `actions/checkout` is SHALLOW (depth 1), so
// `git cat-file -e "${REVISION}^{commit}"` failed for a genuine ancestor
// of `main` purely because the object was never fetched into that
// checkout, not because the revision was invalid. The all-zeros mutation
// proof from an earlier fold only proved the gate REJECTS a fake
// revision; nobody proved it ACCEPTS a real one in the environment where
// it actually runs — that is exactly the half that shipped broken.
//
// Librarian QA (thr_74hvpkqnxjp9q, CI-red fold): a first cut of this test
// cloned `--depth=1` from `file://{repo_root()}` — but in CI, `repo_root`
// IS the shallow checkout under test, so its own `origin` can't supply
// the missing object either, and the test would fail in exactly the
// environment it exists to protect (a self-defeating regression, worse
// than none — it would have permanently blocked this fold from ever
// going green in CI). Fixed by building a fully SYNTHETIC upstream in a
// scratch directory: real git history, the real `gen-doc-status.sh`
// script copied byte-for-byte, its own manifest/REVISION — independent of
// whatever state this test's own checkout happens to be in. The synthetic
// `origin` plays the role CI's real GitHub remote plays for the real
// script: it always has full history, regardless of how shallow the
// checkout that clones from it is.
fn run_git(args: &[&str], cwd: &Path) -> String {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(cwd)
        .env("GIT_AUTHOR_NAME", "doc-w0-fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "doc-w0-fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .output()
        .unwrap_or_else(|e| panic!("run git {args:?}: {e}"));
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn object_present(rev: &str, cwd: &Path) -> bool {
    // `output()`, not `status()`: git's own "not a valid object name"
    // diagnostic on the expected-absent pre-check would otherwise leak to
    // the test harness's terminal even though this call succeeding
    // (returning `false`) is the correct, asserted-for outcome.
    std::process::Command::new("git")
        .args(["cat-file", "-e", &format!("{rev}^{{commit}}")])
        .current_dir(cwd)
        .output()
        .expect("run git cat-file")
        .status
        .success()
}

/// Ledger test helpers (SRC-ATTEST Part 1). Every fixture below needs
/// `library/SOURCE-ATTESTATIONS` to exist unconditionally — the check path
/// now treats a missing ledger as a hard failure regardless of whether the
/// manifest cites anything.
fn object_format(cwd: &Path) -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "--show-object-format"])
        .current_dir(cwd)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "sha1".to_string())
}

/// Content-addressed blob OID of a working-tree file, independent of
/// whether it has been committed yet — matches how the real ledger binds a
/// path to a blob OID.
fn hash_object(repo: &Path, rel_path: &str) -> String {
    run_git(&["hash-object", rel_path], repo)
}

/// Writes `library/SOURCE-ATTESTATIONS` with exactly the given
/// `(oid, path)` rows, sorted by path, matching the real generator's shape.
fn write_ledger(repo: &Path, rows: &[(&str, &str)]) {
    let fmt = object_format(repo);
    let mut sorted: Vec<&(&str, &str)> = rows.iter().collect();
    sorted.sort_by_key(|(_, path)| *path);
    let mut body = format!("# ken-source-attestation-v1 object-format={fmt}\n");
    for (oid, path) in sorted {
        body.push_str(&format!("{oid}\t{path}\n"));
    }
    std::fs::write(repo.join("library/SOURCE-ATTESTATIONS"), body).unwrap();
}

/// Header-only ledger for fixtures whose manifest cites no sources at all.
fn write_empty_ledger(repo: &Path) {
    write_ledger(repo, &[]);
}

/// Builds a fully synthetic upstream in `base/origin`: the real
/// `gen-doc-status.sh` copied byte-for-byte, a minimal `library/`
/// substrate, several commits of unrelated history after the
/// REVISION-anchored commit (so a depth=1 clone of the tip genuinely
/// lacks it), then a final commit pointing `library/REVISION` at that
/// distant ancestor — mirroring how this WP has bumped `library/REVISION`
/// on every rebase fold. Returns `(origin_dir, revision_target, tip)`.
fn build_synthetic_origin(base: &Path) -> (PathBuf, String, String) {
    let origin = base.join("origin");
    std::fs::create_dir_all(&origin).expect("create origin dir");
    run_git(&["init", "--quiet", "-b", "main"], &origin);
    std::fs::create_dir_all(origin.join("scripts")).unwrap();
    std::fs::create_dir_all(origin.join("library")).unwrap();
    let real_script = std::fs::read_to_string(repo_root().join("scripts/gen-doc-status.sh"))
        .expect("read the real gen-doc-status.sh to copy into the fixture");
    std::fs::write(origin.join("scripts/gen-doc-status.sh"), &real_script).unwrap();
    std::fs::write(
        origin.join("library/manifest.toml"),
        "[[document]]\npath = \"library/fixture.md\"\nkind = \"explanatory\"\n\
         authority = \"explanatory\"\navailability = \"current\"\n",
    )
    .unwrap();
    std::fs::write(origin.join("library/fixture.md"), "# Fixture\n").unwrap();
    std::fs::write(origin.join("library/REVISION"), "0".repeat(40)).unwrap();
    write_empty_ledger(&origin);
    run_git(&["add", "-A"], &origin);
    run_git(&["commit", "--quiet", "-m", "initial"], &origin);
    let revision_target = run_git(&["rev-parse", "HEAD"], &origin);

    for i in 0..20 {
        std::fs::write(origin.join(format!("filler-{i}.txt")), format!("filler {i}\n")).unwrap();
        run_git(&["add", "-A"], &origin);
        run_git(&["commit", "--quiet", "-m", &format!("filler {i}")], &origin);
    }
    std::fs::write(
        origin.join("library/REVISION"),
        format!("{revision_target}\n"),
    )
    .unwrap();
    run_git(&["add", "-A"], &origin);
    run_git(
        &["commit", "--quiet", "-m", "anchor REVISION at the distant ancestor"],
        &origin,
    );
    let tip = run_git(&["rev-parse", "HEAD"], &origin);
    (origin, revision_target, tip)
}

fn ancestry_provable(rev: &str, cwd: &Path) -> bool {
    std::process::Command::new("git")
        .args(["merge-base", "--is-ancestor", rev, "HEAD"])
        .current_dir(cwd)
        .output()
        .expect("run git merge-base --is-ancestor")
        .status
        .success()
}

#[test]
fn shallow_clone_self_heals_from_an_independent_full_history_origin() {
    let pid = std::process::id();
    let base = std::env::temp_dir().join(format!("doc-w0-synthetic-{pid}"));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("create scratch base dir");

    struct Cleanup(PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(base.clone());

    let (origin, revision_target, tip) = build_synthetic_origin(&base);

    // The checkout under test: a real depth=1 clone of the SYNTHETIC
    // origin — not of this test's own (possibly shallow, in CI)
    // checkout. Same topology as CI's `actions/checkout`, but the source
    // of truth is self-contained.
    let checkout = base.join("checkout");
    let clone_status = std::process::Command::new("git")
        .args(["clone", "--quiet", "--depth=1"])
        .arg(format!("file://{}", origin.display()))
        .arg(&checkout)
        .status()
        .expect("run git clone --depth=1");
    assert!(clone_status.success(), "git clone --depth=1 failed");

    assert_eq!(
        run_git(&["rev-parse", "HEAD"], &checkout),
        tip,
        "clone did not land on the intended tip commit"
    );
    assert_eq!(
        run_git(&["rev-parse", "--is-shallow-repository"], &checkout),
        "true",
        "test setup did not produce an actually-shallow checkout"
    );
    assert!(
        !object_present(&revision_target, &checkout),
        "test setup: the shallow checkout must NOT already have the \
         REVISION object, or this regression proves nothing"
    );

    // Positive: the real, committed REVISION — a genuine distant ancestor
    // whose object this shallow checkout did not fetch up front — must
    // resolve by self-healing from the synthetic origin.
    let positive = std::process::Command::new("bash")
        .arg(checkout.join("scripts/gen-doc-status.sh"))
        .current_dir(&checkout)
        .output()
        .expect("run gen-doc-status.sh in the shallow checkout");
    assert!(
        positive.status.success(),
        "gen-doc-status.sh failed on a real ancestor revision in a shallow \
         checkout against an independent full-history origin — this is the \
         exact PR #830 CI failure shape. stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&positive.stdout),
        String::from_utf8_lossy(&positive.stderr)
    );
    assert!(
        object_present(&revision_target, &checkout),
        "generator reported success without actually fetching the \
         REVISION object into the checkout"
    );

    // Negative, same checkout: a fake all-zero id must still be rejected
    // — self-healing a shallow clone must not turn into accepting
    // anything just because deepening happened to occur.
    std::fs::write(
        checkout.join("library/REVISION"),
        "0000000000000000000000000000000000000000",
    )
    .expect("overwrite REVISION with a fake id");
    let negative = std::process::Command::new("bash")
        .arg(checkout.join("scripts/gen-doc-status.sh"))
        .current_dir(&checkout)
        .output()
        .expect("run gen-doc-status.sh with a fake REVISION");
    assert!(
        !negative.status.success(),
        "gen-doc-status.sh accepted an all-zero fake REVISION in a shallow \
         checkout — the shallow-clone self-heal must not mask a genuinely \
         invalid revision"
    );
    assert!(
        String::from_utf8_lossy(&negative.stderr).contains("does not resolve to a real commit"),
        "expected the fake-revision diagnostic, got stderr:\n{}",
        String::from_utf8_lossy(&negative.stderr)
    );
}

// Architect finding (thr_74hvpkqnxjp9q, CI-red re-review): object PRESENT
// is not the whole predicate — a shallow clone can fetch `$REVISION` as
// its own separate shallow root (e.g. an earlier, narrower fetch) while
// never fetching the commits connecting it to HEAD. `cat-file -e` then
// succeeds but `merge-base --is-ancestor` cannot prove ancestry. The
// ORIGINAL self-heal only triggered on `cat-file` failing, so this state
// skipped deepening entirely and fell through to a false "not an
// ancestor" rejection of a genuine ancestor. Reproduces that exact
// topology (a normal depth=1 clone of the tip, PLUS a separate depth=1
// fetch of the distant ancestor by itself — object present, no
// connecting history) against the same independent synthetic origin, so
// this test is immune to the same nested-topology blind spot Librarian
// found in the first cut of the sibling test above.
#[test]
fn shallow_clone_self_heals_when_object_present_but_ancestry_unprovable() {
    let pid = std::process::id();
    let base = std::env::temp_dir().join(format!("doc-w0-synthetic-ancestry-{pid}"));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("create scratch base dir");

    struct Cleanup(PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(base.clone());

    let (origin, revision_target, tip) = build_synthetic_origin(&base);

    let checkout = base.join("checkout");
    let clone_status = std::process::Command::new("git")
        .args(["clone", "--quiet", "--depth=1"])
        .arg(format!("file://{}", origin.display()))
        .arg(&checkout)
        .status()
        .expect("run git clone --depth=1");
    assert!(clone_status.success(), "git clone --depth=1 failed");
    assert_eq!(
        run_git(&["rev-parse", "HEAD"], &checkout),
        tip,
        "clone did not land on the intended tip commit"
    );

    // Fetch the REVISION commit as its OWN separate shallow root — the
    // object lands in the object database, but nothing connects it to
    // HEAD's history.
    run_git(
        &["fetch", "--quiet", "--depth=1", "origin", &revision_target],
        &checkout,
    );

    assert_eq!(
        run_git(&["rev-parse", "--is-shallow-repository"], &checkout),
        "true",
        "test setup did not produce an actually-shallow checkout"
    );
    assert!(
        object_present(&revision_target, &checkout),
        "test setup: the separate shallow-root fetch did not land the \
         REVISION object — this regression proves nothing"
    );
    assert!(
        !ancestry_provable(&revision_target, &checkout),
        "test setup: ancestry must NOT be provable yet, or this regression \
         proves nothing (the object being present alone is not the bug)"
    );

    let positive = std::process::Command::new("bash")
        .arg(checkout.join("scripts/gen-doc-status.sh"))
        .current_dir(&checkout)
        .output()
        .expect("run gen-doc-status.sh in the shallow checkout");
    assert!(
        positive.status.success(),
        "gen-doc-status.sh failed when the REVISION object was present but \
         ancestry was not yet provable — self-heal must trigger on EITHER \
         half of the predicate failing, not just object-absence. \
         stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&positive.stdout),
        String::from_utf8_lossy(&positive.stderr)
    );
    assert!(
        ancestry_provable(&revision_target, &checkout),
        "generator reported success without actually fetching the \
         connecting history — ancestry still isn't provable"
    );
}

// --- gate 7: cited source content is unchanged since REVISION (DOC-CURRENCY-
// --- ANCHOR) ----------------------------------------------------------------
//
// The tests above prove `revision_resolved()` correctly establishes "REVISION
// names a real ancestor commit." That is a PROXY for the property
// `library/STATUS.md` actually claims — "the corpus was validated as of
// REVISION" — and a TRUE proxy is exactly the shape that shipped in DOC-W0:
// nine review rounds converged on ever-better true statements about the
// anchor without anyone reading a single cited byte AT it. Grounded,
// un-mutated, on `origin/main @ 6be9754b`: `STATUS.md` stamped "Validated
// revision e5a400c7" while `git ls-tree e5a400c7 -- library/` returns zero
// entries — every check above still passes.
//
// Builds a small self-contained repo (same byte-copy-the-real-script
// pattern as `build_synthetic_origin`) with one document citing an external
// `docs/` file, so the mutation proof can act directly on git history
// rather than needing shallow-clone gymnastics. Since SRC-ATTEST Part 1,
// the currency claim is a `library/SOURCE-ATTESTATIONS` ledger row (a
// blob OID for the cited path) checked against HEAD, not a REVISION-to-HEAD
// diff — the fixture below writes that ledger explicitly.
fn build_currency_fixture(base: &Path) -> (PathBuf, String) {
    let repo = base.join("repo");
    std::fs::create_dir_all(&repo).expect("create repo dir");
    run_git(&["init", "--quiet", "-b", "main"], &repo);
    std::fs::create_dir_all(repo.join("scripts")).unwrap();
    std::fs::create_dir_all(repo.join("library")).unwrap();
    std::fs::create_dir_all(repo.join("docs")).unwrap();
    let real_script = std::fs::read_to_string(repo_root().join("scripts/gen-doc-status.sh"))
        .expect("read the real gen-doc-status.sh to copy into the fixture");
    std::fs::write(repo.join("scripts/gen-doc-status.sh"), &real_script).unwrap();
    std::fs::write(
        repo.join("docs/example.md"),
        "# Example\n\n## A Heading\n\noriginal content\n",
    )
    .unwrap();
    let example_oid = hash_object(&repo, "docs/example.md");
    std::fs::write(
        repo.join("library/manifest.toml"),
        "[[document]]\npath = \"library/fixture.md\"\nkind = \"explanatory\"\n\
         authority = \"explanatory\"\navailability = \"current\"\nsources = [\n  \
         \"docs/example.md#a-heading\",\n]\n",
    )
    .unwrap();
    std::fs::write(repo.join("library/fixture.md"), "# Fixture\n").unwrap();
    std::fs::write(repo.join("library/REVISION"), "0".repeat(40)).unwrap();
    write_ledger(&repo, &[(&example_oid, "docs/example.md")]);
    run_git(&["add", "-A"], &repo);
    run_git(
        &["commit", "--quiet", "-m", "initial: manifest + cited source"],
        &repo,
    );
    let revision = run_git(&["rev-parse", "HEAD"], &repo);

    // Point REVISION at the commit just made — a follow-up commit, matching
    // the self-referential-parent design this script's header explains
    // (REVISION can't name the commit that sets it).
    std::fs::write(repo.join("library/REVISION"), format!("{revision}\n")).unwrap();
    run_git(&["add", "-A"], &repo);
    run_git(&["commit", "--quiet", "-m", "anchor REVISION"], &repo);
    // Librarian QA (thr_15yrvjrpap9td, hotfix fold-2 re-review): the
    // origin/main trust-anchor check is now MANDATORY (fail closed, never
    // silently skipped) -- every fixture that wants a green arm must
    // supply its own synthetic anchor. HEAD is the natural "main" for a
    // fixture with no separate branch/squash concerns.
    set_origin_main_to_head(&repo);
    (repo, revision)
}

/// Points a synthetic `refs/remotes/origin/main` at the fixture's current
/// HEAD -- no real remote needed, `merge-base --is-ancestor` only reads the
/// ref. Fixtures that don't call this deliberately test the missing-anchor
/// path instead.
fn set_origin_main_to_head(repo: &Path) {
    let head = run_git(&["rev-parse", "HEAD"], repo);
    run_git(&["update-ref", "refs/remotes/origin/main", &head], repo);
}

// Plain write mode, not `--check`: these fixtures don't pre-populate a
// committed `library/STATUS.md` to diff against (irrelevant to what's under
// test — the currency checks below run and can fail BEFORE render/--check
// would ever touch that file), so `--check` would spuriously fail on a
// missing comparison file on the recovery/green arms.
fn run_gen_doc_status(repo: &Path) -> std::process::Output {
    std::process::Command::new("bash")
        .arg(repo.join("scripts/gen-doc-status.sh"))
        .current_dir(repo)
        .output()
        .expect("run gen-doc-status.sh")
}

#[test]
fn content_currency_gate_rejects_a_drifted_cited_source_and_recovers() {
    let pid = std::process::id();
    let base = std::env::temp_dir().join(format!("doc-currency-drift-{pid}"));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("create scratch base dir");
    struct Cleanup(PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(base.clone());

    let (repo, _revision) = build_currency_fixture(&base);

    // Green: the cited source is unchanged since REVISION (REVISION is its
    // own immediate ancestor here, so this is trivially true — the baseline
    // that must NOT be flagged).
    let green = run_gen_doc_status(&repo);
    assert!(
        green.status.success(),
        "gen-doc-status.sh failed on an unmutated cited source. \
         stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&green.stdout),
        String::from_utf8_lossy(&green.stderr)
    );

    // Red: mutate the cited source's BODY under an UNCHANGED heading — the
    // exact adversary forward-repro shape (a structural anchor gate stays
    // green while content drifts underneath it).
    std::fs::write(
        repo.join("docs/example.md"),
        "# Example\n\n## A Heading\n\nMUTATED — this must be caught.\n",
    )
    .unwrap();
    run_git(&["add", "-A"], &repo);
    run_git(&["commit", "--quiet", "-m", "mutate cited source"], &repo);

    let red = run_gen_doc_status(&repo);
    assert!(
        !red.status.success(),
        "gen-doc-status.sh accepted a cited source whose body changed \
         under an unchanged heading since REVISION"
    );
    let red_stderr = String::from_utf8_lossy(&red.stderr);
    assert!(
        red_stderr.contains("docs/example.md") && red_stderr.contains("attested"),
        "expected a diagnostic naming the drifted source, got stderr:\n{red_stderr}"
    );

    // Green again: revert the content — proves the gate isn't just
    // permanently red once tripped, and that the check is genuinely keyed
    // on content (the ledger never moved), not on commit count/history shape.
    std::fs::write(
        repo.join("docs/example.md"),
        "# Example\n\n## A Heading\n\noriginal content\n",
    )
    .unwrap();
    run_git(&["add", "-A"], &repo);
    run_git(&["commit", "--quiet", "-m", "revert cited source"], &repo);

    let recovered = run_gen_doc_status(&repo);
    assert!(
        recovered.status.success(),
        "gen-doc-status.sh stayed red after the cited source's content \
         was reverted to match REVISION. stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&recovered.stdout),
        String::from_utf8_lossy(&recovered.stderr)
    );
}

#[test]
fn content_currency_gate_rejects_revision_predating_librarys_own_introduction() {
    let pid = std::process::id();
    let base = std::env::temp_dir().join(format!("doc-currency-bootstrap-{pid}"));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("create scratch base dir");
    struct Cleanup(PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(base.clone());

    let repo = base.join("repo");
    std::fs::create_dir_all(&repo).expect("create repo dir");
    run_git(&["init", "--quiet", "-b", "main"], &repo);

    // Commit 1: a repository that does not have library/ yet at all —
    // this is the state DOC-W0's real REVISION (`e5a400c7`) pointed at.
    std::fs::write(repo.join("README.md"), "pre-library state\n").unwrap();
    run_git(&["add", "-A"], &repo);
    run_git(&["commit", "--quiet", "-m", "pre-library"], &repo);
    let pre_library_revision = run_git(&["rev-parse", "HEAD"], &repo);

    // Commit 2: introduce library/, but (the bug under test) anchor
    // REVISION at the PRE-library commit rather than at-or-after this one.
    std::fs::create_dir_all(repo.join("scripts")).unwrap();
    std::fs::create_dir_all(repo.join("library")).unwrap();
    let real_script = std::fs::read_to_string(repo_root().join("scripts/gen-doc-status.sh"))
        .expect("read the real gen-doc-status.sh to copy into the fixture");
    std::fs::write(repo.join("scripts/gen-doc-status.sh"), &real_script).unwrap();
    std::fs::write(
        repo.join("library/manifest.toml"),
        "[[document]]\npath = \"library/fixture.md\"\nkind = \"explanatory\"\n\
         authority = \"explanatory\"\navailability = \"current\"\nsources = []\n",
    )
    .unwrap();
    std::fs::write(repo.join("library/fixture.md"), "# Fixture\n").unwrap();
    std::fs::write(
        repo.join("library/REVISION"),
        format!("{pre_library_revision}\n"),
    )
    .unwrap();
    write_empty_ledger(&repo);
    run_git(&["add", "-A"], &repo);
    run_git(
        &["commit", "--quiet", "-m", "introduce library/, REVISION mis-anchored"],
        &repo,
    );
    set_origin_main_to_head(&repo);

    let out = run_gen_doc_status(&repo);
    assert!(
        !out.status.success(),
        "gen-doc-status.sh accepted a REVISION that predates \
         library/manifest.toml's own introduction — the exact DOC-W0 shape \
         (STATUS.md stamped validated at a revision where library/ had zero \
         entries)"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("predates library/'s own"),
        "expected the bootstrap-distinguishing diagnostic, got stderr:\n{stderr}"
    );

    // Recovery: re-anchor REVISION at the commit that introduced library/
    // itself (the earliest legitimate value) — must now pass.
    let introducing_commit = run_git(&["rev-parse", "HEAD"], &repo);
    std::fs::write(
        repo.join("library/REVISION"),
        format!("{introducing_commit}\n"),
    )
    .unwrap();
    run_git(&["add", "-A"], &repo);
    run_git(&["commit", "--quiet", "-m", "re-anchor REVISION at library/'s introduction"], &repo);

    let recovered = run_gen_doc_status(&repo);
    assert!(
        recovered.status.success(),
        "gen-doc-status.sh stayed red after REVISION was re-anchored \
         at library/'s own introducing commit. stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&recovered.stdout),
        String::from_utf8_lossy(&recovered.stderr)
    );
}

// Librarian QA (thr_15yrvjrpap9td, first pass, finding 2), reproduced as a
// committed regression rather than left as handoff-only evidence: a cited
// source that is a symlink must be REJECTED outright, not silently
// diffed via its own (target-path) blob — `git diff --quiet` on a symlink
// path compares the symlink's target string, which can stay byte-identical
// while the file it resolves to changes underneath it, so a content-
// currency check that doesn't special-case symlinks would report "clean"
// without ever having read the real content. This fixture proves the
// stronger fail-closed claim: EVEN WITH THE TARGET UNCHANGED, a symlink
// source must still be rejected, because nothing here can distinguish that
// case from the exact one that slips through undetected — "verified"
// through indirection is not verified.
#[cfg(unix)]
#[test]
fn content_currency_gate_rejects_a_symlink_source_even_when_its_target_is_unchanged() {
    use std::os::unix::fs::symlink;

    let pid = std::process::id();
    let base = std::env::temp_dir().join(format!("doc-currency-symlink-{pid}"));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("create scratch base dir");
    struct Cleanup(PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(base.clone());

    let repo = base.join("repo");
    std::fs::create_dir_all(&repo).expect("create repo dir");
    std::fs::create_dir_all(repo.join("scripts")).unwrap();
    std::fs::create_dir_all(repo.join("library")).unwrap();
    std::fs::create_dir_all(repo.join("docs")).unwrap();
    run_git(&["init", "--quiet", "-b", "main"], &repo);
    let real_script = std::fs::read_to_string(repo_root().join("scripts/gen-doc-status.sh"))
        .expect("read the real gen-doc-status.sh to copy into the fixture");
    std::fs::write(repo.join("scripts/gen-doc-status.sh"), &real_script).unwrap();
    std::fs::write(
        repo.join("docs/target.md"),
        "# Target\n\n## A Heading\n\nreal content\n",
    )
    .unwrap();
    symlink(repo.join("docs/target.md"), repo.join("docs/link.md"))
        .expect("create the symlink source probe");
    std::fs::write(
        repo.join("library/manifest.toml"),
        "[[document]]\npath = \"library/fixture.md\"\nkind = \"explanatory\"\n\
         authority = \"explanatory\"\navailability = \"current\"\nsources = [\n  \
         \"docs/link.md\",\n]\n",
    )
    .unwrap();
    std::fs::write(repo.join("library/fixture.md"), "# Fixture\n").unwrap();
    std::fs::write(repo.join("library/REVISION"), "0".repeat(40)).unwrap();
    run_git(&["add", "-A"], &repo);
    run_git(
        &["commit", "--quiet", "-m", "initial: manifest + symlink source"],
        &repo,
    );
    // The ledger must list a row for the symlink path to reach the
    // mode-check at all (a missing row would instead trip the
    // set-equality "missing from ledger" branch, a different diagnostic) —
    // use the path's OWN tracked blob OID (the symlink's target-string
    // blob), exactly what a real generator run against this fixture would
    // produce, since attesting it is possible; the check must still refuse
    // to trust it.
    let link_oid = run_git(&["rev-parse", "HEAD:docs/link.md"], &repo);
    write_ledger(&repo, &[(&link_oid, "docs/link.md")]);
    let revision = run_git(&["rev-parse", "HEAD"], &repo);
    std::fs::write(repo.join("library/REVISION"), format!("{revision}\n")).unwrap();
    run_git(&["add", "-A"], &repo);
    run_git(&["commit", "--quiet", "-m", "anchor REVISION + ledger"], &repo);
    set_origin_main_to_head(&repo);

    // Target is UNCHANGED since the ledger was written — the only variable
    // under test is "is the cited path a symlink", not "did the content
    // drift".
    let out = run_gen_doc_status(&repo);
    assert!(
        !out.status.success(),
        "gen-doc-status.sh accepted a symlink cited as a manifest source, \
         even with its target byte-unchanged since REVISION — content- \
         currency through a symlink is unverifiable and must be rejected \
         outright, not silently diffed via the symlink's own blob"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("symlink source"),
        "expected the symlink-rejection diagnostic, got stderr:\n{stderr}"
    );
}

// Librarian QA (thr_15yrvjrpap9td, second pass), reproduced as a committed
// regression (their scratch probe `e9927bec` was detached, not committed):
// a duplicate, out-of-order `kind` field desynced the two consumers. The
// Rust gate's `parse_manifest` keeps whatever `kind =` value it saw LAST
// for the whole record (same as this file's render awk elsewhere); an
// earlier cut of the shell's source-extraction awk instead decided
// "checked or not" the instant `sources = [...]` closed, using whatever
// `kind` had been seen SO FAR — so `kind = "status"` placed immediately
// before `sources`, with `kind = "explanatory"` restored immediately
// after, made the shell see `status` (skip the source) while the record's
// real, final kind is `explanatory` (source-currency applies). Proves the
// fix: the shell must defer to the record's final `kind`, exactly like
// the Rust parser, so this can no longer stay green.
#[test]
fn content_currency_gate_rejects_drift_hidden_behind_a_duplicate_out_of_order_kind_field() {
    let pid = std::process::id();
    let base = std::env::temp_dir().join(format!("doc-currency-dup-kind-{pid}"));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("create scratch base dir");
    struct Cleanup(PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(base.clone());

    let repo = base.join("repo");
    std::fs::create_dir_all(&repo).expect("create repo dir");
    std::fs::create_dir_all(repo.join("scripts")).unwrap();
    std::fs::create_dir_all(repo.join("library")).unwrap();
    std::fs::create_dir_all(repo.join("docs")).unwrap();
    run_git(&["init", "--quiet", "-b", "main"], &repo);
    let real_script = std::fs::read_to_string(repo_root().join("scripts/gen-doc-status.sh"))
        .expect("read the real gen-doc-status.sh to copy into the fixture");
    std::fs::write(repo.join("scripts/gen-doc-status.sh"), &real_script).unwrap();
    std::fs::write(
        repo.join("docs/example.md"),
        "# Example\n\n## A Heading\n\noriginal content\n",
    )
    .unwrap();
    let example_oid = hash_object(&repo, "docs/example.md");
    // `kind = "status"` right before `sources`, `kind = "explanatory"`
    // (the record's REAL, final kind) restored right after — the exact
    // field placement from the live probe.
    std::fs::write(
        repo.join("library/manifest.toml"),
        "[[document]]\npath = \"library/fixture.md\"\nkind = \"status\"\n\
         authority = \"explanatory\"\navailability = \"current\"\nsources = [\n  \
         \"docs/example.md#a-heading\",\n]\nkind = \"explanatory\"\n",
    )
    .unwrap();
    std::fs::write(repo.join("library/fixture.md"), "# Fixture\n").unwrap();
    std::fs::write(repo.join("library/REVISION"), "0".repeat(40)).unwrap();
    write_ledger(&repo, &[(&example_oid, "docs/example.md")]);
    run_git(&["add", "-A"], &repo);
    run_git(
        &["commit", "--quiet", "-m", "initial: manifest + duplicate kind"],
        &repo,
    );
    let revision = run_git(&["rev-parse", "HEAD"], &repo);
    std::fs::write(repo.join("library/REVISION"), format!("{revision}\n")).unwrap();
    run_git(&["add", "-A"], &repo);
    run_git(&["commit", "--quiet", "-m", "anchor REVISION"], &repo);
    set_origin_main_to_head(&repo);

    // Drift the cited source's body under an unchanged heading, WITHOUT
    // bumping REVISION — this must be caught despite the duplicate-kind
    // decoy.
    std::fs::write(
        repo.join("docs/example.md"),
        "# Example\n\n## A Heading\n\nMUTATED — must be caught despite the decoy.\n",
    )
    .unwrap();
    run_git(&["add", "-A"], &repo);
    run_git(&["commit", "--quiet", "-m", "drift the cited source"], &repo);

    let out = run_gen_doc_status(&repo);
    assert!(
        !out.status.success(),
        "gen-doc-status.sh accepted a drifted cited source because a \
         duplicate, out-of-order `kind` field made the shell's extraction \
         see a stale (`status`) kind at the moment `sources` closed, \
         instead of the record's real final kind (`explanatory`)"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("docs/example.md") && stderr.contains("attested"),
        "expected a diagnostic naming the drifted source, got stderr:\n{stderr}"
    );
}

// --- SRC-ATTEST Part 1 proof matrix, rows 3/4/8 -----------------------------
//
// Rows 1 (drift, ledger unchanged), 2 (candidate-time update goes green), and
// the symlink-row case of row 4 are proved above by the tests already
// adapted to the ledger. These close the remaining rows the frame names as
// required: 3 (citation add/remove -> set mismatch), 4's remaining shapes
// (duplicate/wrong-path row), and 8 (the check path cannot mutate the
// ledger).

#[test]
fn ledger_set_mismatch_when_a_citation_is_added_without_a_ledger_row() {
    let pid = std::process::id();
    let base = std::env::temp_dir().join(format!("doc-currency-add-cite-{pid}"));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("create scratch base dir");
    struct Cleanup(PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(base.clone());

    let (repo, _revision) = build_currency_fixture(&base);

    // A second cited source appears with no corresponding ledger row — the
    // exact "citation add" half of row 3.
    std::fs::write(repo.join("docs/second.md"), "# Second\n\nnever attested\n").unwrap();
    std::fs::write(
        repo.join("library/manifest.toml"),
        "[[document]]\npath = \"library/fixture.md\"\nkind = \"explanatory\"\n\
         authority = \"explanatory\"\navailability = \"current\"\nsources = [\n  \
         \"docs/example.md#a-heading\",\n  \"docs/second.md\",\n]\n",
    )
    .unwrap();
    run_git(&["add", "-A"], &repo);
    run_git(
        &["commit", "--quiet", "-m", "add an unattested citation"],
        &repo,
    );

    let out = run_gen_doc_status(&repo);
    assert!(
        !out.status.success(),
        "gen-doc-status.sh accepted a manifest citation with no matching \
         ledger row"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("missing from ledger") && stderr.contains("docs/second.md"),
        "expected a diagnostic naming the unattested new citation, got \
         stderr:\n{stderr}"
    );
}

#[test]
fn ledger_set_mismatch_when_a_citation_is_removed_but_its_ledger_row_stays() {
    let pid = std::process::id();
    let base = std::env::temp_dir().join(format!("doc-currency-remove-cite-{pid}"));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("create scratch base dir");
    struct Cleanup(PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(base.clone());

    let (repo, _revision) = build_currency_fixture(&base);

    // Drop the citation from the manifest; the ledger still names it — the
    // "citation remove" half of row 3. A stale ledger row is exactly as
    // wrong as a missing one: it asserts an attestation for a claim the
    // corpus no longer makes.
    std::fs::write(
        repo.join("library/manifest.toml"),
        "[[document]]\npath = \"library/fixture.md\"\nkind = \"explanatory\"\n\
         authority = \"explanatory\"\navailability = \"current\"\nsources = []\n",
    )
    .unwrap();
    run_git(&["add", "-A"], &repo);
    run_git(
        &[
            "commit",
            "--quiet",
            "-m",
            "remove the citation, ledger untouched",
        ],
        &repo,
    );

    let out = run_gen_doc_status(&repo);
    assert!(
        !out.status.success(),
        "gen-doc-status.sh accepted a stale ledger row for a citation that \
         no longer exists in the manifest"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("stale in ledger") && stderr.contains("docs/example.md"),
        "expected a diagnostic naming the stale ledger row, got stderr:\n{stderr}"
    );
}

#[test]
fn ledger_rejects_a_duplicate_path_row() {
    let pid = std::process::id();
    let base = std::env::temp_dir().join(format!("doc-currency-dup-row-{pid}"));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("create scratch base dir");
    struct Cleanup(PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(base.clone());

    let (repo, _revision) = build_currency_fixture(&base);

    let example_oid = run_git(&["rev-parse", "HEAD:docs/example.md"], &repo);
    let fmt = object_format(&repo);
    std::fs::write(
        repo.join("library/SOURCE-ATTESTATIONS"),
        format!(
            "# ken-source-attestation-v1 object-format={fmt}\n\
             {example_oid}\tdocs/example.md\n\
             {example_oid}\tdocs/example.md\n"
        ),
    )
    .unwrap();
    run_git(&["add", "-A"], &repo);
    run_git(&["commit", "--quiet", "-m", "duplicate ledger row"], &repo);

    let out = run_gen_doc_status(&repo);
    assert!(
        !out.status.success(),
        "gen-doc-status.sh accepted a ledger with a duplicate path row"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("duplicate path row"),
        "expected the duplicate-row diagnostic, got stderr:\n{stderr}"
    );
}

#[test]
fn ledger_rejects_a_path_escaping_the_repository() {
    let pid = std::process::id();
    let base = std::env::temp_dir().join(format!("doc-currency-escape-row-{pid}"));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("create scratch base dir");
    struct Cleanup(PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(base.clone());

    let (repo, _revision) = build_currency_fixture(&base);

    let example_oid = run_git(&["rev-parse", "HEAD:docs/example.md"], &repo);
    let fmt = object_format(&repo);
    // The escaping row REPLACES the legitimate one, so the only variable
    // under test is row-shape rejection, not a set-mismatch on the real
    // citation (which would fire first and mask this).
    std::fs::write(
        repo.join("library/SOURCE-ATTESTATIONS"),
        format!(
            "# ken-source-attestation-v1 object-format={fmt}\n\
             {example_oid}\t../outside-the-repo.md\n"
        ),
    )
    .unwrap();
    std::fs::write(
        repo.join("library/manifest.toml"),
        "[[document]]\npath = \"library/fixture.md\"\nkind = \"explanatory\"\n\
         authority = \"explanatory\"\navailability = \"current\"\nsources = []\n",
    )
    .unwrap();
    run_git(&["add", "-A"], &repo);
    run_git(&["commit", "--quiet", "-m", "escaping ledger row"], &repo);

    let out = run_gen_doc_status(&repo);
    assert!(
        !out.status.success(),
        "gen-doc-status.sh accepted a ledger row whose path escapes the \
         repository"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("escapes the repository"),
        "expected the path-escape diagnostic, got stderr:\n{stderr}"
    );
}

// Architect finding (dec_1n8mxg2b0m54w, terminal review on `ccf89fda`):
// `git ls-tree`/Rust both normalize `docs/./x` and `docs//x` to the same
// blob as `docs/x` — so a noncanonical manifest citation paired with a
// matching noncanonical ledger row would agree as RAW STRINGS while both
// aliasing the real path, defeating exact set equality's intent (Part 1
// rule 4). These prove the fix rejects both the ledger-row half and the
// manifest-citation half, and that they can't hide behind each other.
#[test]
fn ledger_rejects_a_dot_slash_alias_row_even_though_the_manifest_matches_it() {
    let pid = std::process::id();
    let base = std::env::temp_dir().join(format!("doc-currency-alias-row-{pid}"));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("create scratch base dir");
    struct Cleanup(PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(base.clone());

    let (repo, _revision) = build_currency_fixture(&base);

    let example_oid = run_git(&["rev-parse", "HEAD:docs/example.md"], &repo);
    let fmt = object_format(&repo);
    // Both the manifest citation AND the ledger row use the SAME alias
    // spelling (`docs/./example.md`), so a raw-string set-equality check
    // alone would see them agree and never reach the OID comparison — the
    // exact defect under test.
    std::fs::write(
        repo.join("library/SOURCE-ATTESTATIONS"),
        format!(
            "# ken-source-attestation-v1 object-format={fmt}\n\
             {example_oid}\tdocs/./example.md\n"
        ),
    )
    .unwrap();
    std::fs::write(
        repo.join("library/manifest.toml"),
        "[[document]]\npath = \"library/fixture.md\"\nkind = \"explanatory\"\n\
         authority = \"explanatory\"\navailability = \"current\"\nsources = [\n  \
         \"docs/./example.md\",\n]\n",
    )
    .unwrap();
    run_git(&["add", "-A"], &repo);
    run_git(
        &["commit", "--quiet", "-m", "matching alias citation+row"],
        &repo,
    );

    let out = run_gen_doc_status(&repo);
    assert!(
        !out.status.success(),
        "gen-doc-status.sh accepted a `docs/./example.md` alias even though \
         the manifest citation used the identical alias spelling — raw \
         string agreement between a noncanonical citation and a matching \
         noncanonical ledger row must not substitute for canonical-form \
         enforcement"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("canonical") || stderr.contains("escapes the repository"),
        "expected a canonical-path diagnostic (from either the citation \
         check or the row check), got stderr:\n{stderr}"
    );
}

#[test]
fn ledger_rejects_a_doubled_slash_row() {
    let pid = std::process::id();
    let base = std::env::temp_dir().join(format!("doc-currency-doubleslash-row-{pid}"));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("create scratch base dir");
    struct Cleanup(PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(base.clone());

    let (repo, _revision) = build_currency_fixture(&base);

    let example_oid = run_git(&["rev-parse", "HEAD:docs/example.md"], &repo);
    let fmt = object_format(&repo);
    std::fs::write(
        repo.join("library/SOURCE-ATTESTATIONS"),
        format!(
            "# ken-source-attestation-v1 object-format={fmt}\n\
             {example_oid}\tdocs//example.md\n"
        ),
    )
    .unwrap();
    std::fs::write(
        repo.join("library/manifest.toml"),
        "[[document]]\npath = \"library/fixture.md\"\nkind = \"explanatory\"\n\
         authority = \"explanatory\"\navailability = \"current\"\nsources = []\n",
    )
    .unwrap();
    run_git(&["add", "-A"], &repo);
    run_git(
        &["commit", "--quiet", "-m", "doubled-slash ledger row"],
        &repo,
    );

    let out = run_gen_doc_status(&repo);
    assert!(
        !out.status.success(),
        "gen-doc-status.sh accepted a ledger row with a doubled slash \
         (`docs//example.md`), a path git/Rust normalize to `docs/example.md`"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("canonical") || stderr.contains("escapes the repository"),
        "expected a canonical-path diagnostic, got stderr:\n{stderr}"
    );
}

#[test]
fn check_and_write_modes_never_mutate_the_ledger() {
    let pid = std::process::id();
    let base = std::env::temp_dir().join(format!("doc-currency-check-immutable-{pid}"));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("create scratch base dir");
    struct Cleanup(PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(base.clone());

    let (repo, _revision) = build_currency_fixture(&base);
    let ledger_path = repo.join("library/SOURCE-ATTESTATIONS");
    let before = std::fs::read(&ledger_path).unwrap();

    // Default (write) mode regenerates STATUS.md; the ledger is a read
    // input to it, never a write target.
    let write_out = run_gen_doc_status(&repo);
    assert!(
        write_out.status.success(),
        "gen-doc-status.sh (write mode) failed on a green fixture. \
         stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&write_out.stdout),
        String::from_utf8_lossy(&write_out.stderr)
    );
    assert_eq!(
        std::fs::read(&ledger_path).unwrap(),
        before,
        "gen-doc-status.sh (write mode) mutated library/SOURCE-ATTESTATIONS \
         — the check/generate paths must stay separate entry points (SRC- \
         ATTEST row 8)"
    );

    // `--check` mode.
    let check_out = std::process::Command::new("bash")
        .arg(repo.join("scripts/gen-doc-status.sh"))
        .arg("--check")
        .current_dir(&repo)
        .output()
        .expect("run gen-doc-status.sh --check");
    assert!(
        check_out.status.success(),
        "gen-doc-status.sh --check failed on a freshly-regenerated fixture. \
         stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&check_out.stdout),
        String::from_utf8_lossy(&check_out.stderr)
    );
    assert_eq!(
        std::fs::read(&ledger_path).unwrap(),
        before,
        "gen-doc-status.sh --check mutated library/SOURCE-ATTESTATIONS"
    );
}

#[test]
fn generator_only_ever_writes_the_proposed_sibling_never_the_real_ledger() {
    let pid = std::process::id();
    let base = std::env::temp_dir().join(format!("doc-currency-generator-{pid}"));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("create scratch base dir");
    struct Cleanup(PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(base.clone());

    let (repo, _revision) = build_currency_fixture(&base);
    let real_generator =
        std::fs::read_to_string(repo_root().join("scripts/gen-source-attestations.sh"))
            .expect("read the real gen-source-attestations.sh to copy into the fixture");
    std::fs::write(
        repo.join("scripts/gen-source-attestations.sh"),
        &real_generator,
    )
    .unwrap();

    let ledger_path = repo.join("library/SOURCE-ATTESTATIONS");
    let proposed_path = repo.join("library/SOURCE-ATTESTATIONS.proposed");
    let _ = std::fs::remove_file(&proposed_path);
    let before = std::fs::read(&ledger_path).unwrap();

    // Deliberately stale the real ledger relative to what the generator
    // would compute (drift the cited source, don't touch the ledger), so a
    // generator that silently "fixed" the real file would be observable.
    std::fs::write(
        repo.join("docs/example.md"),
        "# Example\n\n## A Heading\n\nchanged after attestation\n",
    )
    .unwrap();
    run_git(&["add", "-A"], &repo);
    run_git(
        &["commit", "--quiet", "-m", "drift after attestation"],
        &repo,
    );

    let out = std::process::Command::new("bash")
        .arg(repo.join("scripts/gen-source-attestations.sh"))
        .current_dir(&repo)
        .output()
        .expect("run gen-source-attestations.sh");
    assert!(
        out.status.success(),
        "gen-source-attestations.sh failed to render a proposed ledger. \
         stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        proposed_path.is_file(),
        "gen-source-attestations.sh did not write the .proposed sibling"
    );
    assert_eq!(
        std::fs::read(&ledger_path).unwrap(),
        before,
        "gen-source-attestations.sh mutated the REAL ledger — generation \
         must never install its own output; only a reviewed, deliberate \
         commit may (SRC-ATTEST non-automation boundary)"
    );
    let proposed = std::fs::read_to_string(&proposed_path).unwrap();
    assert!(
        proposed.contains("docs/example.md"),
        "proposed ledger did not include the (now drifted) cited source: \
         {proposed}"
    );
}

// --- REVISION must survive a squash-merge, not just resolve on the branch --
//
// Landed post-merge outage (thr_15yrvjrpap9td/thr_sq41qedhmtas, adversary
// evt_504x5h9t6veqq, 2026-07-22): `DOC-CURRENCY-ANCHOR` merged with
// `library/REVISION` naming `cc2af484`, the WP branch's own immediate-parent
// commit -- correct on the branch (where the registered `generated-current`
// runner's status-generator check demanded exactly that bump) and where three
// folds, an Architect approval, a Librarian QA pass, and green CI all checked
// it. The publisher
// **squash-merges**: the merged commit's sole parent is the pre-merge `main`
// tip, so no pre-squash branch commit survives as an ancestor of `main`.
// `origin/main` went CI-red on its own documentation gate the moment the
// squash landed -- a property that was true at every check anyone ran and
// became false only after the last one.
//
// Architect ruling (thr_tq8z3dda5khk, evt_2aj7bxb164cp8): the fix is NOT
// "REVISION must equal the candidate's exact/latest merge base" -- that
// overclaims a single canonical value where the real contract is a
// conjunction any qualifying commit `R` can satisfy:
//   1. `R` is a squash-stable ancestor of the integrated tree -- an
//      already-`main` commit, never a candidate-only parent;
//   2. `library/manifest.toml` exists at `R` (the bootstrap check);
//   3. every current non-status document's cited source blob is
//      byte-identical at `R` and `HEAD` (the content-currency check);
//   4. `STATUS.md` is generated from that exact `R`.
// This test proves ONLY predicate 1's topology distinction: a branch-local
// commit (`C1`) does not survive a squash-merge onto `main`, while a
// commit that is already on `main` (`B`) does, by construction, on this
// repository's linear (no-merge-commit) history -- a squash commit `S`'s
// sole parent is always the pre-merge main tip `T`, and any candidate's
// merge base is always `T` or an ancestor of `T`, hence always an ancestor
// of `S`. A bare `merge-base --is-ancestor` assertion on its own would be
// a WEAKER check than this: it must run against a topology that actually
// has the squash-merge shape, which is what this test constructs, rather
// than against the branch (where the bug was invisible by construction).
//
// ⚠ Residual, stated explicitly rather than left implicit: this test
// CANNOT and does not select which on-`main` ancestor is the "right" one
// among several that would all pass it -- it only distinguishes on-`main`
// from branch-local. Predicates 2-4 above are the independent, separately-
// tested acceptance checks (`gate_manifest_rejects_a_field_line_...`,
// `content_currency_gate_rejects_a_drifted_cited_source_and_recovers`,
// `content_currency_gate_rejects_revision_predating_librarys_own_
// introduction`) that narrow "any ancestor of main" down to a valid one.
// Picking `638fe6d4` specifically over some other qualifying ancestor is a
// review judgment (it was the last reviewed DOC-CURRENCY base and
// demonstrably contains the manifest), not a fact this or any test proves.
#[test]
fn revision_must_survive_a_simulated_squash_merge_not_just_the_branch() {
    let pid = std::process::id();
    let base = std::env::temp_dir().join(format!("doc-currency-squash-{pid}"));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("create scratch base dir");
    struct Cleanup(PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(base.clone());

    let repo = base.join("repo");
    std::fs::create_dir_all(&repo).expect("create repo dir");
    std::fs::create_dir_all(repo.join("scripts")).unwrap();
    std::fs::create_dir_all(repo.join("library")).unwrap();
    run_git(&["init", "--quiet", "-b", "main"], &repo);
    let real_script = std::fs::read_to_string(repo_root().join("scripts/gen-doc-status.sh"))
        .expect("read the real gen-doc-status.sh to copy into the fixture");
    std::fs::write(repo.join("scripts/gen-doc-status.sh"), &real_script).unwrap();
    std::fs::write(
        repo.join("library/manifest.toml"),
        "[[document]]\npath = \"library/fixture.md\"\nkind = \"explanatory\"\n\
         authority = \"explanatory\"\navailability = \"current\"\nsources = []\n",
    )
    .unwrap();
    std::fs::write(repo.join("library/fixture.md"), "# Fixture\n").unwrap();
    std::fs::write(repo.join("library/REVISION"), "0".repeat(40)).unwrap();
    write_empty_ledger(&repo);
    run_git(&["add", "-A"], &repo);
    run_git(
        &["commit", "--quiet", "-m", "B: merge base (simulated main tip)"],
        &repo,
    );
    let b = run_git(&["rev-parse", "HEAD"], &repo);

    // A synthetic `origin/main` ref pointing at B -- no real remote needed,
    // git only needs the ref to exist for `merge-base --is-ancestor` to
    // read it. This is what lets the ON-BRANCH check below run at all.
    run_git(
        &["update-ref", "refs/remotes/origin/main", &b],
        &repo,
    );

    // C1: first branch commit (filler) -- the shape that used to be
    // (incorrectly) treated as "the immediate parent, so it's fine."
    std::fs::write(repo.join("filler-1.txt"), "filler\n").unwrap();
    run_git(&["add", "-A"], &repo);
    run_git(&["commit", "--quiet", "-m", "C1: branch commit"], &repo);
    let c1 = run_git(&["rev-parse", "HEAD"], &repo);

    // Librarian QA (thr_15yrvjrpap9td, hotfix re-review, live commit
    // `61f07dc1`): the first cut of this test proved the script's
    // post-squash behavior only on a FULLY SYNTHETIC repo -- it never
    // touched THIS repository's actual `library/REVISION`, so a
    // branch-local value there would still pass every check unchanged,
    // reproducing the exact outage undetected. This is the primary arm
    // that closes that: on the BRANCH itself (HEAD = C1, no squash yet),
    // with REVISION = C1 (a value that trivially resolves as an ancestor
    // of local HEAD, since C1 IS HEAD) -- the new origin/main check must
    // still reject it, because C1 is not yet on `origin/main`.
    std::fs::write(repo.join("library/REVISION"), format!("{c1}\n")).unwrap();
    let on_branch_negative = run_gen_doc_status(&repo);
    assert!(
        !on_branch_negative.status.success(),
        "gen-doc-status.sh accepted a branch-local REVISION value ON THE \
         BRANCH itself, before any squash -- this is the actual landed \
         outage's precondition: a value that only fails once it's too \
         late to catch before publish"
    );
    assert!(
        String::from_utf8_lossy(&on_branch_negative.stderr).contains("origin/main"),
        "expected the origin/main-ancestry diagnostic, got stderr:\n{}",
        String::from_utf8_lossy(&on_branch_negative.stderr)
    );

    // Same branch state, REVISION = B (the merge base, already on
    // origin/main by construction) -- must pass, proving the check isn't
    // simply rejecting everything.
    std::fs::write(repo.join("library/REVISION"), format!("{b}\n")).unwrap();
    let on_branch_positive = run_gen_doc_status(&repo);
    assert!(
        on_branch_positive.status.success(),
        "gen-doc-status.sh rejected REVISION naming the merge base, on the \
         branch, before any squash. stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&on_branch_positive.stdout),
        String::from_utf8_lossy(&on_branch_positive.stderr)
    );

    // C2: branch tip (the WP's final fold). Its exact content doesn't
    // matter beyond establishing the tree the squash carries forward.
    std::fs::write(repo.join("filler-2.txt"), "filler 2\n").unwrap();
    run_git(&["add", "-A"], &repo);
    run_git(&["commit", "--quiet", "-m", "C2: branch tip"], &repo);

    // S: the synthetic squash commit -- C2's TREE, but B as the SOLE
    // PARENT. This is exactly what a squash-merge produces
    // (`git commit-tree <tree> -p <pre-merge-main-tip>`): a new commit
    // carrying the content, parented on `main`, never on the branch.
    let tree = run_git(&["rev-parse", "HEAD^{tree}"], &repo);
    let s = run_git(
        &["commit-tree", &tree, "-p", &b, "-m", "S: simulated squash merge"],
        &repo,
    );
    run_git(&["checkout", "--quiet", &s], &repo);
    assert_eq!(
        run_git(&["rev-parse", "HEAD"], &repo),
        s,
        "checkout of the synthetic squash commit landed somewhere else"
    );

    // Negative: REVISION names the branch-local commit C1 -- exactly the
    // landed bug. Must be rejected AT S, even though it resolved fine on
    // the branch (that is the entire failure mode: true on the branch,
    // false only after the squash).
    std::fs::write(repo.join("library/REVISION"), format!("{c1}\n")).unwrap();
    let negative = run_gen_doc_status(&repo);
    assert!(
        !negative.status.success(),
        "gen-doc-status.sh accepted REVISION naming a branch-local commit \
         at a simulated squash-merge commit -- this is the exact \
         DOC-CURRENCY-ANCHOR main outage (cc2af484 unreachable after \
         d03151d3's squash)"
    );
    assert!(
        String::from_utf8_lossy(&negative.stderr).contains("is not an ancestor"),
        "expected the ancestry-rejection diagnostic, got stderr:\n{}",
        String::from_utf8_lossy(&negative.stderr)
    );

    // Positive: REVISION names the merge base B -- an ancestor of S by
    // construction (S's sole parent IS B), exactly like a real
    // squash-merge landing on main.
    std::fs::write(repo.join("library/REVISION"), format!("{b}\n")).unwrap();
    let positive = run_gen_doc_status(&repo);
    assert!(
        positive.status.success(),
        "gen-doc-status.sh rejected REVISION naming the branch's merge \
         base at the simulated squash commit -- the fix's core claim \
         (merge base survives squash) does not hold. stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&positive.stdout),
        String::from_utf8_lossy(&positive.stderr)
    );
}

// Librarian QA (thr_15yrvjrpap9td, hotfix fold-2 re-review): a "best-effort,
// skip if no anchor resolves" version of the origin/main check FAILS OPEN
// in exactly the topology it exists to guard. Live proof from the review:
// delete `refs/remotes/origin/main`, point `origin` at an unreachable repo,
// set a real REVISION to a real branch-local commit -- the best-effort
// check silently skipped and `gen-doc-status.sh`/`--check` both exited 0,
// reproducing the outage undetected. Committed here as a permanent
// regression: with NO origin/main ref and NO working `origin` remote at
// all, the script must reject outright with a dedicated diagnostic naming
// the missing anchor -- never silently fall back to the (insufficient)
// HEAD-only ancestry check that caused the original outage.
#[test]
fn missing_origin_main_anchor_is_rejected_not_silently_skipped() {
    let pid = std::process::id();
    let base = std::env::temp_dir().join(format!("doc-currency-no-anchor-{pid}"));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("create scratch base dir");
    struct Cleanup(PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(base.clone());

    let repo = base.join("repo");
    std::fs::create_dir_all(&repo).expect("create repo dir");
    std::fs::create_dir_all(repo.join("scripts")).unwrap();
    std::fs::create_dir_all(repo.join("library")).unwrap();
    run_git(&["init", "--quiet", "-b", "main"], &repo);
    let real_script = std::fs::read_to_string(repo_root().join("scripts/gen-doc-status.sh"))
        .expect("read the real gen-doc-status.sh to copy into the fixture");
    std::fs::write(repo.join("scripts/gen-doc-status.sh"), &real_script).unwrap();
    std::fs::write(
        repo.join("library/manifest.toml"),
        "[[document]]\npath = \"library/fixture.md\"\nkind = \"explanatory\"\n\
         authority = \"explanatory\"\navailability = \"current\"\nsources = []\n",
    )
    .unwrap();
    std::fs::write(repo.join("library/fixture.md"), "# Fixture\n").unwrap();
    std::fs::write(repo.join("library/REVISION"), "0".repeat(40)).unwrap();
    run_git(&["add", "-A"], &repo);
    run_git(&["commit", "--quiet", "-m", "initial"], &repo);
    let revision = run_git(&["rev-parse", "HEAD"], &repo);
    std::fs::write(repo.join("library/REVISION"), format!("{revision}\n")).unwrap();
    run_git(&["add", "-A"], &repo);
    run_git(&["commit", "--quiet", "-m", "anchor REVISION at own parent"], &repo);

    // Deliberately: no `refs/remotes/origin/main`, no `origin` remote
    // configured at all -- `git fetch origin` has nothing to fetch from.
    // REVISION genuinely resolves against local HEAD (it's the immediate
    // parent), so every check that stops at HEAD-ancestry would pass this.
    let out = run_gen_doc_status(&repo);
    assert!(
        !out.status.success(),
        "gen-doc-status.sh accepted a REVISION with no origin/main trust \
         anchor available at all -- it must fail closed here, not fall \
         back to the HEAD-only ancestry check that caused the real outage"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("cannot establish the origin/main trust anchor"),
        "expected the dedicated missing-anchor diagnostic (distinct from \
         the ancestry-rejection one), got stderr:\n{stderr}"
    );
}

// --- authority class is one of D1's closed set -----------------------------

fn check_authority_classes() {
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

// --- symlink escape (Architect finding, thr_74hvpkqnxjp9q, third round) ---
//
// Committed regression, not just handoff evidence: plants a real
// in-repository symlink pointing at a real file outside the repository
// and a real symlinked directory, and proves both `resolve_confined` and
// `library_markdown_files` reject them. Unix-only (`std::os::unix::fs::
// symlink`) — consistent with the rest of this WP's tooling
// (`scripts/gen-doc-status.sh` is bash).
#[cfg(unix)]
#[test]
fn symlink_escape_is_rejected_by_confinement_and_by_the_walk() {
    use std::os::unix::fs::symlink;

    struct Cleanup(Vec<PathBuf>);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            for p in &self.0 {
                let _ = std::fs::remove_file(p);
                let _ = std::fs::remove_dir_all(p);
            }
        }
    }

    let root = repo_root();
    let library = root.join("library");
    let pid = std::process::id();

    // An outside-the-repo target a symlink will point at.
    let outside_file = std::env::temp_dir().join(format!("doc-w0-symlink-target-{pid}.md"));
    std::fs::write(&outside_file, "host content outside the repository\n")
        .expect("write outside probe file");

    // A symlinked FILE under library/ pointing at it.
    let file_link = library.join(format!("__doc_w0_symlink_file_probe_{pid}.md"));

    // A symlinked DIRECTORY under library/ pointing at a tmp dir that
    // itself contains a .md file — proves the walk does not descend.
    let outside_dir = std::env::temp_dir().join(format!("doc-w0-symlink-dir-{pid}"));
    std::fs::create_dir_all(&outside_dir).expect("create outside probe dir");
    std::fs::write(outside_dir.join("leaked.md"), "leaked\n").expect("write leaked probe file");
    let dir_link = library.join(format!("__doc_w0_symlink_dir_probe_{pid}"));

    let _cleanup = Cleanup(vec![
        file_link.clone(),
        dir_link.clone(),
        outside_file.clone(),
        outside_dir.clone(),
    ]);

    symlink(&outside_file, &file_link).expect("create file-symlink probe");
    symlink(&outside_dir, &dir_link).expect("create dir-symlink probe");

    let file_rel = format!("__doc_w0_symlink_file_probe_{pid}.md");
    let dir_rel = format!("__doc_w0_symlink_dir_probe_{pid}");
    assert!(
        resolve_confined(&library, &file_rel, &root).is_none(),
        "resolve_confined followed an in-repo symlink to a file outside the repository"
    );

    let walk = walk_library();
    // Architect finding (thr_74hvpkqnxjp9q, fourth round): a symlink must
    // be REPORTED, not silently omitted from discovery — omission is what
    // let an unregistered/misdirected symlink pass gate 1 by never being
    // seen. Both planted symlinks must show up in `walk.symlinks`.
    assert!(
        walk.symlinks.contains(&format!("library/{file_rel}")),
        "walk_library() silently omitted a symlinked file instead of reporting it: {:?}",
        walk.symlinks
    );
    assert!(
        walk.symlinks.contains(&format!("library/{dir_rel}")),
        "walk_library() silently omitted a symlinked directory instead of reporting it: {:?}",
        walk.symlinks
    );
    assert!(
        !walk.markdown_files.contains(&format!("library/{file_rel}")),
        "walk_library() discovered a symlinked file as an ordinary markdown file"
    );
    assert!(
        !walk.markdown_files.iter().any(|f| f.contains("leaked")),
        "walk_library() walked into a symlinked directory and found a file outside the repository"
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

// --- Wave 2 agent-library gates -------------------------------------------
//
// These are global invariants over a second, deliberately small manifest
// corpus. They are standalone tests rather than VALIDATION_GATES rows:
// document-record applicability is the registry's category, while module
// existence, a pack dependency DAG, and evaluation-to-pack coverage are
// properties of the complete agent-library graph.

#[derive(Debug, Clone, Copy)]
enum ControlledScalarKind {
    String,
    Integer,
    Other,
}

#[derive(Debug, Clone, Default)]
struct ControlledRecord {
    scalars: BTreeMap<String, String>,
    scalar_kinds: BTreeMap<String, ControlledScalarKind>,
    arrays: BTreeMap<String, Vec<String>>,
    arrays_are_strings: BTreeMap<String, bool>,
}

fn array_contains_only_strings(src: &str) -> bool {
    let mut quoted = false;
    let mut escaped = false;
    for ch in src.chars() {
        if quoted {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                quoted = false;
            }
        } else if ch == '"' {
            quoted = true;
        } else if !matches!(ch, '[' | ']' | ',' | ' ' | '\t' | '\r' | '\n') {
            return false;
        }
    }
    !quoted
}

fn parse_controlled_records(src: &str, header: &str) -> Vec<ControlledRecord> {
    let marker = format!("[[{header}]]");
    let mut records = Vec::new();
    let mut current: Option<ControlledRecord> = None;
    let mut lines = src.lines().peekable();

    while let Some(raw_line) = lines.next() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == marker {
            if let Some(record) = current.take() {
                records.push(record);
            }
            current = Some(ControlledRecord::default());
            continue;
        }
        if line.starts_with("[[") {
            if let Some(record) = current.take() {
                records.push(record);
            }
            continue;
        }
        let Some(record) = current.as_mut() else {
            continue;
        };
        let Some((key, mut value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim().to_string();
        value = value.trim();
        let mut array_text = String::new();
        if value.starts_with('[') && !value.contains(']') {
            array_text.push_str(value);
            array_text.push('\n');
            for continuation in lines.by_ref() {
                array_text.push_str(continuation);
                array_text.push('\n');
                if continuation.contains(']') {
                    break;
                }
            }
            value = array_text.trim();
        }
        if value.starts_with('[') {
            record
                .arrays_are_strings
                .insert(key.clone(), array_contains_only_strings(value));
            record.arrays.insert(key, extract_quoted_strings(value));
        } else {
            let (scalar, kind) = if value.starts_with('"') {
                (
                    extract_quoted_strings(value).pop().unwrap_or_default(),
                    ControlledScalarKind::String,
                )
            } else if value.parse::<i64>().is_ok() {
                (value.to_string(), ControlledScalarKind::Integer)
            } else {
                (value.to_string(), ControlledScalarKind::Other)
            };
            record.scalar_kinds.insert(key.clone(), kind);
            record.scalars.insert(key, scalar);
        }
    }
    if let Some(record) = current {
        records.push(record);
    }
    records
}

fn parse_pack_file(src: &str) -> ControlledRecord {
    let wrapped = format!("[[pack-file]]\n{src}");
    parse_controlled_records(&wrapped, "pack-file")
        .pop()
        .expect("pack file parsed to no record")
}

fn controlled_record_value(record: &ControlledRecord) -> serde_json::Value {
    let mut object = serde_json::Map::new();
    for (key, value) in &record.scalars {
        let json = match record.scalar_kinds.get(key) {
            Some(ControlledScalarKind::String) => serde_json::Value::String(value.clone()),
            Some(ControlledScalarKind::Integer) => value
                .parse::<i64>()
                .map(serde_json::Value::from)
                .unwrap_or(serde_json::Value::Null),
            _ => serde_json::Value::Null,
        };
        object.insert(key.clone(), json);
    }
    for (key, values) in &record.arrays {
        let mut items: Vec<_> = values
            .iter()
            .cloned()
            .map(serde_json::Value::String)
            .collect();
        if !record.arrays_are_strings.get(key).copied().unwrap_or(false) {
            items.push(serde_json::Value::Null);
        }
        object.insert(key.clone(), serde_json::Value::Array(items));
    }
    serde_json::Value::Object(object)
}

fn controlled_manifest_value(src: &str) -> serde_json::Value {
    let wrapped = format!("[[manifest-root]]\n{src}");
    let root = parse_controlled_records(&wrapped, "manifest-root")
        .into_iter()
        .next()
        .expect("agent manifest has no root fields");
    let mut value = controlled_record_value(&root);
    let object = value.as_object_mut().expect("controlled root is an object");
    object.insert(
        "module".to_string(),
        serde_json::Value::Array(
            parse_controlled_records(src, "module")
                .iter()
                .map(controlled_record_value)
                .collect(),
        ),
    );
    object.insert(
        "pack".to_string(),
        serde_json::Value::Array(
            parse_controlled_records(src, "pack")
                .iter()
                .map(controlled_record_value)
                .collect(),
        ),
    );
    value
}

fn is_kebab(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn matches_shipped_pattern(value: &str, pattern: &str) -> Result<bool, String> {
    let matched = match pattern {
        "^[0-9a-f]{40}$" => {
            value.len() == 40
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        }
        "^[a-z0-9-]+$" => is_kebab(value),
        "^(core|tasks)/[a-z0-9-]+$" => value
            .split_once('/')
            .is_some_and(|(prefix, rest)| matches!(prefix, "core" | "tasks") && is_kebab(rest)),
        "^library/agents/(core|tasks)/.*\\.md$" => {
            value
                .strip_prefix("library/agents/")
                .and_then(|rest| rest.split_once('/'))
                .is_some_and(|(area, file)| {
                    matches!(area, "core" | "tasks")
                        && !file.is_empty()
                        && file.ends_with(".md")
                })
        }
        "^library/agents/packs/.*\\.toml$" => value
            .strip_prefix("library/agents/packs/")
            .is_some_and(|file| !file.is_empty() && file.ends_with(".toml")),
        other => return Err(format!("unsupported declared schema pattern {other:?}")),
    };
    Ok(matched)
}

fn schema_violations(
    schema: &serde_json::Value,
    instance: &serde_json::Value,
    schema_root: &serde_json::Value,
    location: &str,
) -> Vec<String> {
    schema_violations_with_refs(schema, instance, schema_root, location, &mut Vec::new())
}

fn assert_schema_constraint_violation(
    schema: &serde_json::Value,
    instance: &serde_json::Value,
    schema_root: &serde_json::Value,
    constraint: &str,
) {
    const LOCATION: &str = "schema fixture";
    assert!(
        !LOCATION.contains(constraint),
        "schema fixture location must be independent of constraint needle {constraint:?}"
    );
    let violations = schema_violations(schema, instance, schema_root, LOCATION);
    let needle = format!("{constraint} violation");
    assert!(
        violations.iter().any(|message| message.contains(&needle)),
        "planted {constraint} violation did not fail at that constraint: {violations:?}"
    );
}

fn decode_json_pointer_component(component: &str) -> Result<String, String> {
    let mut decoded = String::new();
    let mut chars = component.chars();
    while let Some(ch) = chars.next() {
        if ch != '~' {
            decoded.push(ch);
            continue;
        }
        match chars.next() {
            Some('0') => decoded.push('~'),
            Some('1') => decoded.push('/'),
            Some(other) => {
                return Err(format!("invalid JSON-pointer escape ~{other}"));
            }
            None => return Err("trailing ~ in JSON-pointer component".to_string()),
        }
    }
    Ok(decoded)
}

fn resolve_local_schema_reference<'a>(
    reference: &str,
    schema_root: &'a serde_json::Value,
) -> Result<&'a serde_json::Value, String> {
    if reference.contains('%') {
        return Err(format!(
            "percent-encoded schema reference {reference:?} is unsupported"
        ));
    }
    let Some(pointer) = reference.strip_prefix('#') else {
        return Err(format!("external schema reference {reference:?} is unsupported"));
    };
    let mut target = schema_root;
    if pointer.is_empty() {
        return Ok(target);
    }
    let Some(pointer) = pointer.strip_prefix('/') else {
        return Err(format!("invalid local schema reference {reference:?}"));
    };
    for encoded_component in pointer.split('/') {
        let component = decode_json_pointer_component(encoded_component)
            .map_err(|message| format!("{reference:?}: {message}"))?;
        target = match target {
            serde_json::Value::Object(object) => object.get(&component),
            serde_json::Value::Array(array) => component
                .parse::<usize>()
                .ok()
                .and_then(|index| array.get(index)),
            _ => None,
        }
        .ok_or_else(|| {
            format!(
                "unresolved local schema reference {reference:?}: missing component {component:?}"
            )
        })?;
    }
    if !target.is_object() {
        return Err(format!(
            "local schema reference {reference:?} resolves to a non-object schema"
        ));
    }
    Ok(target)
}

fn schema_violations_with_refs(
    schema: &serde_json::Value,
    instance: &serde_json::Value,
    schema_root: &serde_json::Value,
    location: &str,
    active_refs: &mut Vec<String>,
) -> Vec<String> {
    let mut bad = Vec::new();
    if let Some(reference_value) = schema.get("$ref") {
        match reference_value.as_str() {
            Some(reference) if active_refs.iter().any(|active| active == reference) => {
                bad.push(format!(
                    "{location}: schema reference cycle through {reference:?}"
                ));
            }
            Some(reference) => {
                match resolve_local_schema_reference(reference, schema_root) {
                    Ok(target) => {
                        active_refs.push(reference.to_string());
                        bad.extend(schema_violations_with_refs(
                            target,
                            instance,
                            schema_root,
                            location,
                            active_refs,
                        ));
                        active_refs.pop();
                    }
                    Err(message) => bad.push(format!("{location}: {message}")),
                }
            }
            None => bad.push(format!("{location}: $ref must be a string")),
        }
    }

    if let Some(expected) = schema.get("const") {
        if instance != expected {
            bad.push(format!(
                "{location}: const violation: expected {expected}, found {instance}"
            ));
        }
    }

    if let Some(expected_type) = schema.get("type").and_then(serde_json::Value::as_str) {
        let type_matches = match expected_type {
            "object" => instance.is_object(),
            "array" => instance.is_array(),
            "string" => instance.is_string(),
            "integer" => instance.as_i64().is_some(),
            other => {
                bad.push(format!("{location}: unsupported declared type {other:?}"));
                false
            }
        };
        if !type_matches {
            bad.push(format!(
                "{location}: type violation: expected {expected_type}, found {instance}"
            ));
            return bad;
        }
    }

    if let Some(minimum) = schema.get("minimum").and_then(serde_json::Value::as_i64) {
        if instance.as_i64().is_some_and(|value| value < minimum) {
            bad.push(format!("{location}: minimum violation: expected at least {minimum}"));
        }
    }
    if let Some(minimum) = schema.get("minLength").and_then(serde_json::Value::as_u64) {
        if instance
            .as_str()
            .is_some_and(|value| value.chars().count() < minimum as usize)
        {
            bad.push(format!("{location}: minLength violation: expected at least {minimum}"));
        }
    }
    if let Some(pattern) = schema.get("pattern").and_then(serde_json::Value::as_str) {
        if let Some(value) = instance.as_str() {
            match matches_shipped_pattern(value, pattern) {
                Ok(true) => {}
                Ok(false) => bad.push(format!(
                    "{location}: pattern violation: {value:?} does not match {pattern:?}"
                )),
                Err(message) => bad.push(format!("{location}: {message}")),
            }
        }
    }
    if let Some(minimum) = schema.get("minItems").and_then(serde_json::Value::as_u64) {
        if instance
            .as_array()
            .is_some_and(|values| values.len() < minimum as usize)
        {
            bad.push(format!("{location}: minItems violation: expected at least {minimum}"));
        }
    }
    if let (Some(items_schema), Some(items)) = (schema.get("items"), instance.as_array()) {
        for (index, item) in items.iter().enumerate() {
            bad.extend(schema_violations_with_refs(
                items_schema,
                item,
                schema_root,
                &format!("{location}[{index}]"),
                active_refs,
            ));
        }
    }

    let deny_additional_properties = match schema.get("additionalProperties") {
        Some(serde_json::Value::Bool(false)) => true,
        Some(serde_json::Value::Bool(true)) | None => false,
        Some(other) => {
            bad.push(format!(
                "{location}: unsupported additionalProperties form {other}"
            ));
            false
        }
    };

    if let Some(object) = instance.as_object() {
        if let Some(required) = schema.get("required").and_then(serde_json::Value::as_array) {
            for field in required {
                let field = field.as_str().expect("schema required field is a string");
                if !object.contains_key(field) {
                    bad.push(format!("{location}: required field {field:?} is missing"));
                }
            }
        }
        let properties = schema
            .get("properties")
            .and_then(serde_json::Value::as_object);
        if let Some(properties) = properties {
            for (field, field_schema) in properties {
                if let Some(value) = object.get(field) {
                    bad.extend(schema_violations_with_refs(
                        field_schema,
                        value,
                        schema_root,
                        &format!("{location}.{field}"),
                        active_refs,
                    ));
                }
            }
        }
        if deny_additional_properties {
            for field in object.keys() {
                if properties.is_none_or(|declared| !declared.contains_key(field)) {
                    bad.push(format!(
                        "{location}: additionalProperties violation: unknown field {field:?}"
                    ));
                }
            }
        }
    }
    bad
}

fn unsupported_schema_keywords(schema: &serde_json::Value, location: &str) -> Vec<String> {
    const SUPPORTED: &[&str] = &[
        "$schema",
        "$id",
        "$ref",
        "$defs",
        "title",
        "type",
        "required",
        "properties",
        "const",
        "pattern",
        "minLength",
        "minimum",
        "items",
        "minItems",
        "additionalProperties",
    ];
    let Some(object) = schema.as_object() else {
        return Vec::new();
    };
    let mut bad = Vec::new();
    for key in object.keys() {
        if !SUPPORTED.contains(&key.as_str()) {
            bad.push(format!("{location}: unsupported schema keyword {key:?}"));
        }
    }
    for container in ["properties", "$defs"] {
        if let Some(children) = object.get(container).and_then(serde_json::Value::as_object) {
            for (name, child) in children {
                bad.extend(unsupported_schema_keywords(
                    child,
                    &format!("{location}.{container}.{name}"),
                ));
            }
        }
    }
    if let Some(items) = object.get("items") {
        bad.extend(unsupported_schema_keywords(
            items,
            &format!("{location}.items"),
        ));
    }
    bad
}

fn record_scalar<'a>(record: &'a ControlledRecord, field: &str) -> &'a str {
    record
        .scalars
        .get(field)
        .map(String::as_str)
        .unwrap_or("")
}

fn duplicate_record_ids(records: &[ControlledRecord]) -> BTreeSet<String> {
    let mut seen = BTreeSet::new();
    let mut duplicates = BTreeSet::new();
    for record in records {
        let id = record_scalar(record, "id").to_string();
        if !seen.insert(id.clone()) {
            duplicates.insert(id);
        }
    }
    duplicates
}

fn controlled_source_violations(root: &Path, owner: &str, source: &str) -> Vec<String> {
    let (path, anchor) = split_source(source);
    let Some(resolved) = resolve_confined(root, path, root) else {
        return vec![format!("{owner}: source {source:?} escapes the repository")];
    };
    if !resolved.exists() {
        return vec![format!("{owner}: source {source:?} does not exist")];
    }
    if let Some(anchor) = anchor {
        if !resolved.is_file() {
            return vec![format!(
                "{owner}: source anchor {anchor:?} requires a file, but {path:?} is not one"
            )];
        }
        let contents = std::fs::read_to_string(&resolved)
            .unwrap_or_else(|e| panic!("read {}: {e}", resolved.display()));
        if !heading_anchors(&contents).contains(anchor) {
            return vec![format!(
                "{owner}: source anchor {anchor:?} does not exist in {path:?}"
            )];
        }
    }
    Vec::new()
}

fn unicode_whitespace_tokens(path: &Path) -> usize {
    std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
        .split_whitespace()
        .count()
}

fn module_contract_violations(path: &Path, src: &str) -> Vec<String> {
    const SECTIONS: [&str; 10] = [
        "## 1. Use when",
        "## 2. Prerequisites",
        "## 3. Current capability",
        "## 4. Canonical forms",
        "## 5. Invariants and prohibitions",
        "## 6. Decision procedure",
        "## 7. Failure signatures",
        "## 8. Validation",
        "## 9. Authority and sources",
        "## 10. Known unavailable or partial behavior",
    ];
    let mut bad = Vec::new();
    let mut previous = 0;
    for (index, heading) in SECTIONS.iter().enumerate() {
        let Some(position) = src.find(heading) else {
            bad.push(format!("{}: missing {heading:?}", path.display()));
            continue;
        };
        if index > 0 && position <= previous {
            bad.push(format!(
                "{}: {heading:?} is out of contract order",
                path.display()
            ));
        }
        let body_start = position + heading.len();
        let body_end = SECTIONS
            .get(index + 1)
            .and_then(|next| src.find(next))
            .unwrap_or(src.len());
        if src[body_start..body_end].trim().is_empty() {
            bad.push(format!("{}: {heading:?} is empty", path.display()));
        }
        previous = position;
    }
    bad
}

fn graph_violations(
    module_ids: &BTreeSet<String>,
    packs: &BTreeMap<String, ControlledRecord>,
) -> Vec<String> {
    fn visit(
        id: &str,
        packs: &BTreeMap<String, ControlledRecord>,
        visiting: &mut BTreeSet<String>,
        visited: &mut BTreeSet<String>,
        bad: &mut Vec<String>,
    ) {
        if visited.contains(id) {
            return;
        }
        if !visiting.insert(id.to_string()) {
            bad.push(format!("circular pack dependency reaches {id:?}"));
            return;
        }
        if let Some(pack) = packs.get(id) {
            for dependency in pack
                .arrays
                .get("dependencies")
                .into_iter()
                .flatten()
            {
                visit(dependency, packs, visiting, visited, bad);
            }
        }
        visiting.remove(id);
        visited.insert(id.to_string());
    }

    let mut bad = Vec::new();
    for (id, pack) in packs {
        for module in pack.arrays.get("includes").into_iter().flatten() {
            if !module_ids.contains(module) {
                bad.push(format!("{id}: included module {module:?} is missing"));
            }
        }
        for dependency in pack
            .arrays
            .get("dependencies")
            .into_iter()
            .flatten()
        {
            if !packs.contains_key(dependency) {
                bad.push(format!("{id}: dependency pack {dependency:?} is missing"));
            }
        }
    }
    let mut visited = BTreeSet::new();
    for id in packs.keys() {
        visit(
            id,
            packs,
            &mut BTreeSet::new(),
            &mut visited,
            &mut bad,
        );
    }
    bad.sort();
    bad.dedup();
    bad
}

fn transitive_modules(
    id: &str,
    packs: &BTreeMap<String, ControlledRecord>,
    out: &mut BTreeSet<String>,
) {
    let pack = &packs[id];
    for dependency in pack
        .arrays
        .get("dependencies")
        .into_iter()
        .flatten()
    {
        transitive_modules(dependency, packs, out);
    }
    out.extend(
        pack.arrays
            .get("includes")
            .into_iter()
            .flatten()
            .cloned(),
    );
}

#[test]
fn agent_library_manifest_schema_contract_and_measurements_hold() {
    let root = repo_root();
    let agent_root = root.join("library/agents");
    let schema_dir = agent_root.join("schemas");
    let schemas: BTreeSet<String> = std::fs::read_dir(&schema_dir)
        .expect("read agent schema directory")
        .map(|entry| entry.expect("schema entry").file_name().to_string_lossy().into())
        .collect();
    assert_eq!(
        schemas,
        BTreeSet::from([
            "agent-manifest.schema.json".to_string(),
            "pack.schema.json".to_string(),
        ]),
        "schemas exist exactly for the two manifest formats this gate consumes"
    );

    let manifest_schema: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(schema_dir.join("agent-manifest.schema.json"))
            .expect("read agent manifest schema"),
    )
    .expect("parse agent manifest schema");
    let pack_schema: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(schema_dir.join("pack.schema.json"))
            .expect("read pack schema"),
    )
    .expect("parse pack schema");
    let mut unsupported = unsupported_schema_keywords(&manifest_schema, "agent manifest schema");
    unsupported.extend(unsupported_schema_keywords(&pack_schema, "pack schema"));
    assert!(
        unsupported.is_empty(),
        "shipped schemas declare unsupported constraints:\n{}",
        unsupported.join("\n")
    );
    let manifest_src =
        std::fs::read_to_string(agent_root.join("manifest.toml")).expect("read agent manifest");
    let manifest_bad = schema_violations(
        &manifest_schema,
        &controlled_manifest_value(&manifest_src),
        &manifest_schema,
        "agent manifest",
    );
    assert!(
        manifest_bad.is_empty(),
        "agent manifest schema violations:\n{}",
        manifest_bad.join("\n")
    );
    let modules = parse_controlled_records(&manifest_src, "module");
    let manifest_packs = parse_controlled_records(&manifest_src, "pack");
    assert_eq!(modules.len(), 10, "Wave 2 requires four core and six task modules");
    assert!(!manifest_packs.is_empty(), "agent manifest has no packs");
    let duplicate_pack_ids = duplicate_record_ids(&manifest_packs);
    assert!(
        duplicate_pack_ids.is_empty(),
        "agent manifest has duplicate pack IDs: {duplicate_pack_ids:?}"
    );

    let mut module_ids = BTreeSet::new();
    let mut module_sizes = BTreeMap::new();
    let mut bad = Vec::new();
    for module in &modules {
        let id = record_scalar(module, "id");
        let path = record_scalar(module, "path");
        if !module_ids.insert(id.to_string()) {
            bad.push(format!("duplicate module id {id:?}"));
        }
        let full_path = resolve_controlled_path(&root, path, id);
        let src = std::fs::read_to_string(&full_path)
            .unwrap_or_else(|e| panic!("read {}: {e}", full_path.display()));
        bad.extend(module_contract_violations(&full_path, &src));
        for source in module.arrays.get("sources").into_iter().flatten() {
            bad.extend(controlled_source_violations(&root, id, source));
        }
        let measured = unicode_whitespace_tokens(&full_path);
        let declared = record_scalar(module, "measured_tokens")
            .parse::<usize>()
            .unwrap_or(0);
        if measured != declared {
            bad.push(format!(
                "{id}: measured_tokens is {declared}, recomputed {measured}"
            ));
        }
        module_sizes.insert(id.to_string(), measured);
    }
    assert!(bad.is_empty(), "agent module manifest violations:\n{}", bad.join("\n"));

    let mut packs = BTreeMap::new();
    for manifest_pack in &manifest_packs {
        let id = record_scalar(manifest_pack, "id");
        let path =
            resolve_controlled_path(&root, record_scalar(manifest_pack, "path"), id);
        let pack = parse_pack_file(
            &std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("read {}: {e}", path.display())),
        );
        let pack_bad = schema_violations(
            &pack_schema,
            &controlled_record_value(&pack),
            &pack_schema,
            id,
        );
        assert!(
            pack_bad.is_empty(),
            "{id}: pack schema violations:\n{}",
            pack_bad.join("\n")
        );
        for source in manifest_pack.arrays.get("sources").into_iter().flatten() {
            let source_bad = controlled_source_violations(&root, id, source);
            assert!(
                source_bad.is_empty(),
                "{id}: pack source violations:\n{}",
                source_bad.join("\n")
            );
        }
        assert_eq!(record_scalar(&pack, "id"), id, "{id}: pack-file id drift");
        for field in ["purpose"] {
            assert_eq!(
                record_scalar(&pack, field),
                record_scalar(manifest_pack, field),
                "{id}: {field} drift between manifest and pack file"
            );
        }
        for field in ["triggers", "exclusions", "includes", "dependencies"] {
            assert_eq!(
                pack.arrays.get(field),
                manifest_pack.arrays.get(field),
                "{id}: {field} drift between manifest and pack file"
            );
        }
        assert!(
            packs.insert(id.to_string(), pack).is_none(),
            "agent manifest has duplicate pack ID {id:?}"
        );
    }
    let graph_bad = graph_violations(&module_ids, &packs);
    assert!(graph_bad.is_empty(), "agent pack graph violations:\n{}", graph_bad.join("\n"));

    for manifest_pack in &manifest_packs {
        let id = record_scalar(manifest_pack, "id");
        let mut includes = BTreeSet::new();
        transitive_modules(id, &packs, &mut includes);
        let measured: usize = includes.iter().map(|module| module_sizes[module]).sum();
        let declared = record_scalar(manifest_pack, "measured_tokens")
            .parse::<usize>()
            .unwrap_or(0);
        assert_eq!(
            measured, declared,
            "{id}: transitive unique module measurement drift"
        );
    }
}

#[test]
fn agent_pack_integrity_rejects_missing_modules_and_cycles() {
    let module_ids = BTreeSet::from(["core/read-ken".to_string()]);
    let clean = parse_pack_file(
        "id = \"clean\"\nincludes = [\"core/read-ken\"]\ndependencies = []\n",
    );
    let mut packs = BTreeMap::from([("clean".to_string(), clean.clone())]);
    assert!(graph_violations(&module_ids, &packs).is_empty());

    packs
        .get_mut("clean")
        .unwrap()
        .arrays
        .insert("includes".to_string(), vec!["core/missing".to_string()]);
    let missing = graph_violations(&module_ids, &packs);
    assert!(
        missing
            .iter()
            .any(|message| message.contains("included module \"core/missing\" is missing")),
        "planted missing module was not rejected at the named graph detector: {missing:?}"
    );

    let a = parse_pack_file("id = \"a\"\nincludes = []\ndependencies = [\"b\"]\n");
    let b = parse_pack_file("id = \"b\"\nincludes = []\ndependencies = [\"a\"]\n");
    let cycle = graph_violations(
        &BTreeSet::new(),
        &BTreeMap::from([("a".to_string(), a), ("b".to_string(), b)]),
    );
    assert!(
        cycle
            .iter()
            .any(|message| message.contains("circular pack dependency")),
        "planted circular dependency was not rejected at the named graph detector: {cycle:?}"
    );
}

// Durable invariant: manifest key spaces stay injective as the corpus grows.
#[test]
fn agent_key_space_detectors_reject_duplicate_pack_and_task_ids() {
    let duplicate_packs = parse_controlled_records(
        "[[pack]]\nid = \"write-pure\"\n[[pack]]\nid = \"write-pure\"\n",
        "pack",
    );
    assert_eq!(
        duplicate_record_ids(&duplicate_packs),
        BTreeSet::from(["write-pure".to_string()]),
        "planted duplicate pack ID was not rejected at the named key-space detector"
    );

    let duplicate_tasks = parse_controlled_records(
        "[[task]]\nid = \"write-pure\"\n[[task]]\nid = \"write-pure\"\n",
        "task",
    );
    assert_eq!(
        duplicate_record_ids(&duplicate_tasks),
        BTreeSet::from(["write-pure".to_string()]),
        "planted duplicate task ID was not rejected at the named key-space detector"
    );
}

// Durable invariant: every constraint class declared by the shipped schemas
// is executable, not decorative metadata.
#[test]
fn agent_schema_contract_rejects_each_declared_constraint_class() {
    let root = repo_root();
    let schema_dir = root.join("library/agents/schemas");
    let pack_schema: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(schema_dir.join("pack.schema.json"))
            .expect("read pack schema"),
    )
    .expect("parse pack schema");
    let clean = "schema_version = 1\n\
                 id = \"write-pure\"\n\
                 purpose = \"Write Ken\"\n\
                 triggers = [\"write\"]\n\
                 exclusions = [\"effects\"]\n\
                 dependencies = []\n\
                 includes = [\"core/write-ken\"]\n";
    assert!(
        schema_violations(
            &pack_schema,
            &controlled_record_value(&parse_pack_file(clean)),
            &pack_schema,
            "clean pack",
        )
        .is_empty(),
        "positive-control pack does not satisfy the shipped schema"
    );

    let mutations = [
        (
            "const",
            clean.replace("schema_version = 1", "schema_version = 999"),
        ),
        (
            "type",
            clean.replace("purpose = \"Write Ken\"", "purpose = 1"),
        ),
        ("pattern", clean.replace("id = \"write-pure\"", "id = \"WritePure\"")),
        ("minItems", clean.replace("triggers = [\"write\"]", "triggers = []")),
        ("minLength", clean.replace("purpose = \"Write Ken\"", "purpose = \"\"")),
        (
            "additionalProperties",
            format!("{clean}unknown_field = \"must reject\"\n"),
        ),
    ];
    for (constraint, source) in mutations {
        assert_schema_constraint_violation(
            &pack_schema,
            &controlled_record_value(&parse_pack_file(&source)),
            &pack_schema,
            constraint,
        );
    }

    let manifest_schema: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(schema_dir.join("agent-manifest.schema.json"))
            .expect("read agent manifest schema"),
    )
    .expect("parse agent manifest schema");
    let module = serde_json::json!({
        "id": "core/write-ken",
        "path": "library/agents/core/write-ken.md",
        "purpose": "Write Ken",
        "triggers": ["write"],
        "prerequisites": [],
        "sources": ["spec/30-surface/33-declarations.md"],
        "revision": "a9860e9c1617e98799ac408fad8ef1e9c7085443",
        "validation": ["schema"],
        "measured_tokens": 0
    });
    assert_schema_constraint_violation(
        &manifest_schema["$defs"]["module"],
        &module,
        &manifest_schema,
        "minimum",
    );

    let closed_without_properties = serde_json::json!({
        "type": "object",
        "additionalProperties": false
    });
    assert_schema_constraint_violation(
        &closed_without_properties,
        &serde_json::json!({"unknown": "field"}),
        &closed_without_properties,
        "additionalProperties",
    );

    let unsupported_additional_properties = serde_json::json!({
        "type": "object",
        "additionalProperties": {"type": "string"}
    });
    let unsupported_form = schema_violations(
        &unsupported_additional_properties,
        &serde_json::json!({"field": "value"}),
        &unsupported_additional_properties,
        "schema fixture",
    );
    assert!(
        unsupported_form
            .iter()
            .any(|message| message.contains("unsupported additionalProperties form")),
        "unsupported additionalProperties schema form was not rejected loudly: \
         {unsupported_form:?}"
    );

    let required_source = clean.replace("purpose = \"Write Ken\"\n", "");
    let required = schema_violations(
        &pack_schema,
        &controlled_record_value(&parse_pack_file(&required_source)),
        &pack_schema,
        "required",
    );
    assert!(
        required
            .iter()
            .any(|message| message.contains("required field \"purpose\"")),
        "planted required violation did not fail at that constraint: {required:?}"
    );

    let clean_pack = controlled_record_value(&parse_pack_file(clean));
    let mut items_schema = pack_schema.clone();
    items_schema["properties"]["triggers"]["items"]["const"] =
        serde_json::json!("different trigger");
    let items = schema_violations(
        &items_schema,
        &clean_pack,
        &items_schema,
        "items traversal",
    );
    assert!(
        items.iter().any(|message| {
            message.contains("items traversal.triggers[0]") && message.contains("const")
        }),
        "planted items traversal violation did not reach the array member: {items:?}"
    );

    let mut properties_schema = pack_schema.clone();
    properties_schema["properties"]["purpose"]["const"] =
        serde_json::json!("different purpose");
    let properties = schema_violations(
        &properties_schema,
        &clean_pack,
        &properties_schema,
        "properties traversal",
    );
    assert!(
        properties.iter().any(|message| {
            message.contains("properties traversal.purpose") && message.contains("const")
        }),
        "planted properties traversal violation did not reach the field: {properties:?}"
    );

    let mut valid_module = module.clone();
    valid_module["measured_tokens"] = serde_json::json!(1);
    let mut dangling_ref_schema = manifest_schema.clone();
    dangling_ref_schema["properties"]["module"]["items"]["$ref"] =
        serde_json::json!("#/$defs/missing-module");
    let dangling_ref = schema_violations(
        &dangling_ref_schema["properties"]["module"],
        &serde_json::json!([valid_module]),
        &dangling_ref_schema,
        "dangling $ref",
    );
    assert!(
        dangling_ref.iter().any(|message| {
            message.contains("unresolved local schema reference")
                && message.contains("missing-module")
        }),
        "planted dangling $ref did not fail at the reference artifact: {dangling_ref:?}"
    );

    let escaped_pointer_schema = serde_json::json!({
        "$ref": "#/$defs/slash~1name~0record",
        "$defs": {
            "slash/name~record": {"type": "string"}
        }
    });
    assert!(
        schema_violations(
            &escaped_pointer_schema,
            &serde_json::json!("resolved"),
            &escaped_pointer_schema,
            "escaped JSON pointer",
        )
        .is_empty(),
        "RFC 6901 escapes in a local $ref were not decoded"
    );

    let sibling_schema = serde_json::json!({
        "$ref": "#/$defs/string",
        "minLength": 2,
        "$defs": {
            "string": {"type": "string"}
        }
    });
    let sibling = schema_violations(
        &sibling_schema,
        &serde_json::json!("x"),
        &sibling_schema,
        "$ref sibling",
    );
    assert!(
        sibling.iter().any(|message| message.contains("minLength")),
        "supported sibling constraint beside $ref was skipped: {sibling:?}"
    );

    let cyclic_schema = serde_json::json!({
        "$ref": "#/$defs/loop",
        "$defs": {
            "loop": {"$ref": "#/$defs/loop"}
        }
    });
    let cycle = schema_violations(
        &cyclic_schema,
        &serde_json::json!("anything"),
        &cyclic_schema,
        "cyclic $ref",
    );
    assert!(
        cycle
            .iter()
            .any(|message| message.contains("schema reference cycle")),
        "planted recursive $ref did not fail at the cycle detector: {cycle:?}"
    );

    for (reference, expected) in [
        ("https://example.invalid/schema", "external schema reference"),
        ("#/$defs/non-schema", "non-object schema"),
    ] {
        let bad_reference_schema = serde_json::json!({
            "$ref": reference,
            "$defs": {
                "non-schema": "not a schema"
            }
        });
        let violations = schema_violations(
            &bad_reference_schema,
            &serde_json::json!("anything"),
            &bad_reference_schema,
            "unsupported $ref",
        );
        assert!(
            violations.iter().any(|message| message.contains(expected)),
            "planted {expected} did not fail at the reference artifact: {violations:?}"
        );
    }

    let mut future_schema = pack_schema.clone();
    future_schema["futureConstraint"] = serde_json::json!(true);
    let future = unsupported_schema_keywords(&future_schema, "future keyword");
    assert!(
        future
            .iter()
            .any(|message| message.contains("futureConstraint")),
        "planted unsupported schema keyword did not fail closed: {future:?}"
    );
}

// Durable invariant: controlled data can name only repository-confined paths.
#[test]
fn agent_controlled_paths_fail_loudly_on_escape() {
    let root = repo_root();
    for raw in ["../outside-repository", "/tmp/outside-repository"] {
        let result = std::panic::catch_unwind(|| {
            resolve_controlled_path(&root, raw, "planted controlled-path violation")
        });
        assert!(
            result.is_err(),
            "planted controlled path {raw:?} was not rejected loudly"
        );
    }
}

#[test]
fn agent_evaluation_tasks_and_packs_cover_each_other_exactly() {
    let root = repo_root();
    let agent_root = root.join("library/agents");
    let manifest =
        std::fs::read_to_string(agent_root.join("manifest.toml")).expect("read agent manifest");
    let declared_packs: BTreeSet<String> = parse_controlled_records(&manifest, "pack")
        .iter()
        .map(|pack| record_scalar(pack, "id").to_string())
        .collect();
    let tasks_src = std::fs::read_to_string(agent_root.join("evaluations/tasks.toml"))
        .expect("read evaluation tasks");
    let tasks = parse_controlled_records(&tasks_src, "task");
    assert_eq!(tasks.len(), 7, "the Wave 2 evaluation corpus must have seven tasks");
    let duplicate_task_ids = duplicate_record_ids(&tasks);
    assert!(
        duplicate_task_ids.is_empty(),
        "evaluation corpus has duplicate task IDs: {duplicate_task_ids:?}"
    );
    let selected_packs: BTreeSet<String> = tasks
        .iter()
        .map(|task| {
            let pack = record_scalar(task, "pack");
            assert!(!pack.is_empty(), "{} selects no pack", record_scalar(task, "id"));
            let fixture = record_scalar(task, "fixture");
            let fixture_path =
                resolve_controlled_path(&root, fixture, record_scalar(task, "id"));
            assert!(
                fixture_path.is_file(),
                "{} fixture does not exist: {fixture}",
                record_scalar(task, "id")
            );
            pack.to_string()
        })
        .collect();
    assert_eq!(
        selected_packs, declared_packs,
        "each task must select exactly one pack and no pack may be unused"
    );
    assert!(
        declared_packs.iter().all(|pack| !pack.contains("refus")),
        "refusal is point 10's cross-cutting property, not its own pack"
    );
    let refusal = tasks
        .iter()
        .find(|task| record_scalar(task, "id") == "refuse-unsupported")
        .expect("missing refusal evaluation task");
    assert!(
        !refusal.arrays.get("must_refuse").unwrap_or(&Vec::new()).is_empty(),
        "refusal task must name prohibited inventions"
    );

    let results_src =
        std::fs::read_to_string(agent_root.join("evaluations/results-2026-07-24.toml"))
            .expect("read cold-context evaluation results");
    let runs = parse_controlled_records(&results_src, "run");
    assert!(
        runs.len() >= tasks.len(),
        "evaluation results must record at least one run per task"
    );
    let task_ids: BTreeSet<String> = tasks
        .iter()
        .map(|task| record_scalar(task, "id").to_string())
        .collect();
    assert_eq!(
        task_ids.len(),
        tasks.len(),
        "evaluation task ID cardinality must equal task record cardinality"
    );
    let mut latest = BTreeMap::new();
    let mut observed_invention_failure = false;
    for run in &runs {
        let task_id = record_scalar(run, "task_id");
        assert!(task_ids.contains(task_id), "result names unknown task {task_id:?}");
        for field in [
            "run_id",
            "variant",
            "seat_id",
            "cold_evidence",
            "pack",
            "correctness",
            "cited_authority",
            "validation",
        ] {
            assert!(
                !record_scalar(run, field).is_empty(),
                "{task_id}: result omits axis/evidence field {field:?}"
            );
        }
        for field in [
            "correctness_evidence",
            "unnecessary_loads",
            "inventions",
            "authority_paths",
            "exact_loads",
        ] {
            assert!(
                run.arrays.contains_key(field),
                "{task_id}: result omits array axis/evidence field {field:?}"
            );
        }
        if !run.arrays["inventions"].is_empty() {
            observed_invention_failure = true;
            assert_ne!(
                record_scalar(run, "correctness"),
                "correct",
                "{task_id}: a run with an invention cannot be scored correct"
            );
        }
        latest.insert(task_id.to_string(), run);
    }
    assert!(
        observed_invention_failure,
        "results must preserve the initial invented-capability failure, not smooth it into a pass"
    );
    assert_eq!(
        latest.keys().cloned().collect::<BTreeSet<_>>(),
        task_ids,
        "results do not close every evaluation task"
    );
    for (task_id, run) in latest {
        assert_eq!(
            record_scalar(run, "correctness"),
            "correct",
            "{task_id}: latest cold result is not correct"
        );
        assert!(
            run.arrays["inventions"].is_empty(),
            "{task_id}: latest cold result invents syntax or capability"
        );
        assert_eq!(
            record_scalar(run, "cited_authority"),
            "complete",
            "{task_id}: latest cold result does not cite complete authority"
        );
    }
}

fn has_checked_examples(src: &str) -> bool {
    src.lines().any(|line| {
        matches!(
            line.trim(),
            "```ken example" | "```ken reject"
        )
    })
}

fn applies_to_checked_example_records(entry: &DocEntry) -> bool {
    let root = repo_root();
    let path = resolve_controlled_path(&root, &entry.path, "checked-example applicability");
    let src = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    has_checked_examples(&src)
}

fn run_checked_markdown(path: &Path, src: &str, index: usize) -> std::process::Output {
    // Agent modules are ordinary `.md` documents, while `ken check` selects
    // literate extraction by the `.ken.md` suffix. Preserve the complete
    // source bytes in a temporary literate file so every checked fence is
    // exercised through the real extractor without changing the
    // product-facing document name.
    let probe = std::env::temp_dir().join(format!(
        "doc-w2-checked-examples-{}-{index}.ken.md",
        std::process::id()
    ));
    std::fs::write(&probe, src).unwrap_or_else(|e| panic!("write {}: {e}", probe.display()));
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_ken"))
        .arg("check")
        .arg(&probe)
        .output()
        .unwrap_or_else(|e| panic!("run ken check {}: {e}", path.display()));
    let _ = std::fs::remove_file(&probe);
    output
}

fn check_checked_examples() {
    let root = repo_root();
    for (index, entry) in load_manifest()
        .into_iter()
        .filter(|entry| applies_to_checked_example_records(entry))
        .enumerate()
    {
        let path =
            resolve_controlled_path(&root, &entry.path, "checked-example execution");
        let src = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let output = run_checked_markdown(&path, &src, index);
        assert!(
            output.status.success(),
            "checked fences failed in {}:\nstdout:\n{}\nstderr:\n{}",
            path.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn checked_examples_detector_rejects_invalid_example_and_stale_reject() {
    let invalid_example = "# Fixture\n\n```ken example\nconst broken : Bool = Missing\n```\n";
    let invalid = run_checked_markdown(Path::new("invalid-example.md"), invalid_example, 10_001);
    assert!(
        !invalid.status.success()
            && String::from_utf8_lossy(&invalid.stderr)
                .contains("a 'ken example' block failed to elaborate"),
        "invalid `ken example` did not fail at the checked-example artifact: {}",
        String::from_utf8_lossy(&invalid.stderr)
    );

    let stale_reject = "# Fixture\n\n```ken reject\nconst valid : Bool = True\n```\n";
    let stale = run_checked_markdown(Path::new("stale-reject.md"), stale_reject, 10_002);
    assert!(
        !stale.status.success()
            && String::from_utf8_lossy(&stale.stderr)
                .contains("a 'ken reject' block unexpectedly elaborated"),
        "stale `ken reject` did not fail at the checked-example artifact: {}",
        String::from_utf8_lossy(&stale.stderr)
    );
}
