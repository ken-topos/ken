#!/usr/bin/env bash
# gen-doc-status.sh — regenerate library/STATUS.md from library/manifest.toml
# and the explicit, recorded revision in library/REVISION.
#
# library/STATUS.md is GENERATED, never hand-edited (docs/program/12-
# documentation-program.md §2: "a date is not evidence of currency — a
# source revision is"). This script is the single source of truth for it,
# mirroring scripts/gen-progress.sh's shape: plain bash + standard tools,
# a --check mode for CI idempotency, no new dependencies.
#
# Why the revision is an EXPLICIT INPUT, not `git rev-parse HEAD` computed
# here (librarian QA finding 1, thr_74hvpkqnxjp9q, second pass): computing
# HEAD live is self-referential for the very commit that introduces or
# updates STATUS.md — you must generate it BEFORE that commit exists, so
# the embedded HEAD is always the *parent* commit, and checking out the
# finished commit and regenerating always disagrees with what got
# committed. A content hash "solves" that but silently swaps the settled
# contract (DOC-W0/the proposal: "a repository/source revision") for a
# different one that also loses coverage of cited-but-unmodified source
# bytes. The fix librarian asked for: the revision is a small, explicit,
# committed INPUT (`library/REVISION`) that a human (or the Librarian's
# as-built pass) sets deliberately when they have validated the corpus
# against that revision — never auto-derived from live git state, so
# there is no cycle, and `--check` regenerates from that exact recorded
# value and diffs, which is trivially idempotent (AC3) because the input
# didn't change.
#
# Usage: scripts/gen-doc-status.sh [--check]
#   (no args)  regenerate library/STATUS.md in place from library/REVISION.
#   --check    write to a temp file and diff against the committed file;
#              exit non-zero if they differ (AC3: unchanged tree -> no-op).

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MANIFEST="$REPO_ROOT/library/manifest.toml"
REVISION_FILE="$REPO_ROOT/library/REVISION"
OUT_FILE="$REPO_ROOT/library/STATUS.md"

MODE="${1:-write}"
if [ "$MODE" = "--check" ]; then
  MODE="check"
fi

if [ ! -f "$MANIFEST" ]; then
  echo "gen-doc-status: manifest not found: $MANIFEST" >&2
  exit 1
fi

if [ ! -f "$REVISION_FILE" ]; then
  echo "gen-doc-status: $REVISION_FILE not found — record the validated" >&2
  echo "  revision explicitly (e.g. \`git rev-parse HEAD > library/REVISION\`" >&2
  echo "  run BEFORE committing) rather than deriving it here." >&2
  exit 1
fi

REVISION="$(tr -d '[:space:]' < "$REVISION_FILE")"

# Librarian QA (thr_74hvpkqnxjp9q, third pass): a shape check alone lets
# `library/REVISION` hold forty zeroes — a value that LOOKS like a commit
# id but isn't one. Require a full 40-hex id, that it resolves to a real
# commit object, and that it is an ancestor of the current tree (so it
# genuinely describes an earlier, checkable state of this repository, not
# an arbitrary hex string or a commit from an unrelated future/fork).
if ! printf '%s' "$REVISION" | grep -qE '^[0-9a-f]{40}$'; then
  echo "gen-doc-status: library/REVISION must be a full 40-hex commit id, got: '${REVISION}'" >&2
  exit 1
fi
if ! git -C "$REPO_ROOT" cat-file -e "${REVISION}^{commit}" 2>/dev/null; then
  echo "gen-doc-status: library/REVISION '${REVISION}' does not resolve to a real commit object" >&2
  exit 1
fi
if ! git -C "$REPO_ROOT" merge-base --is-ancestor "$REVISION" HEAD 2>/dev/null; then
  echo "gen-doc-status: library/REVISION '${REVISION}' is not an ancestor of the current tree (HEAD)" >&2
  exit 1
fi

# --- manifest parsing -----------------------------------------------------
# library/manifest.toml is a small, hand-controlled TOML subset: a run of
# `[[document]]` tables, each with flat `key = "value"` scalar fields. This
# single-pass awk parser depends on that shape, not on general TOML — it is
# not meant to read arbitrary TOML.

TMP_TABLE="$(mktemp)"
TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_TABLE" "$TMP_OUT"' EXIT

awk '
  function field(line,  v) {
    v = line
    sub(/^[a-z]+[[:space:]]*=[[:space:]]*"/, "", v)
    sub(/".*/, "", v)
    return v
  }
  /^\[\[document\]\]/ {
    if (path != "") print path "\t" kind "\t" authority "\t" availability
    path = ""; kind = ""; authority = ""; availability = ""
    next
  }
  /^path[[:space:]]*=/         { path = field($0) }
  /^kind[[:space:]]*=/         { kind = field($0) }
  /^authority[[:space:]]*=/    { authority = field($0) }
  /^availability[[:space:]]*=/ { availability = field($0) }
  END { if (path != "") print path "\t" kind "\t" authority "\t" availability }
' "$MANIFEST" > "$TMP_TABLE"

ROWS=""
COUNT=0
while IFS=$'\t' read -r path kind authority availability; do
  [ -z "$path" ] && continue
  COUNT=$((COUNT + 1))
  ROWS="${ROWS}| \`${path}\` | ${kind} | ${authority} | ${availability} |
"
done < "$TMP_TABLE"

# --- render -----------------------------------------------------------

render() {
  cat <<EOF
# Library status

**Generated by \`scripts/gen-doc-status.sh\` — do not hand-edit.** Currency
is this file's one job: it is anchored to a repository revision, never a
typed date (docs/program/12-documentation-program.md §2).

**Validated revision:** \`${REVISION}\`

Recorded explicitly in \`library/REVISION\`, not derived from \`git
rev-parse HEAD\` at generation time. A live-HEAD anchor is self-
referential for the commit that introduces or updates this file; an
explicit, deliberately-set input has no such cycle, and \`--check\`
regenerating from the same recorded value reproduces this file
byte-for-byte (AC3). Bump \`library/REVISION\` only when you have
revalidated the corpus against that commit.

## Registered documents

Every row below is one \`[[document]]\` entry in \`library/manifest.toml\`.
A document with no row here has no manifest entry and fails gate 1.

| Path | Kind | Authority | Availability |
|---|---|---|---|
${ROWS}
**Total:** ${COUNT} registered document(s).

## Regenerating

\`\`\`
scripts/gen-doc-status.sh          # regenerate in place from library/REVISION
scripts/gen-doc-status.sh --check  # verify committed file matches that input (CI)
\`\`\`
EOF
}

if [ "$MODE" = "check" ]; then
  render > "$TMP_OUT"
  if ! diff -u "$OUT_FILE" "$TMP_OUT"; then
    echo "gen-doc-status --check: library/STATUS.md is stale — run scripts/gen-doc-status.sh" >&2
    exit 1
  fi
  echo "gen-doc-status --check: library/STATUS.md is current."
else
  render > "$OUT_FILE"
  echo "gen-doc-status: wrote $OUT_FILE"
fi
