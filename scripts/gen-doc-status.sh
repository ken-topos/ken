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

# Environment assumption this check makes, stated explicitly (Steward,
# thr_74hvpkqnxjp9q, PR #830 CI failure): `cat-file -e`/`merge-base` need
# the commit OBJECT present in the local object database, not merely a
# valid-looking hex string — but CI's default checkout (and any local
# `git clone --depth=N`) is SHALLOW, so a real ancestor's object can be
# genuinely absent even though it truly is an ancestor in the repository's
# full history. "Not present in this shallow clone" and "not a real
# commit" are indistinguishable from `cat-file -e`'s exit code alone, so a
# bare shallow clone would silently condemn a valid revision — which is
# exactly what broke PR #830 (a real ancestor of `main`, rejected only
# because the CI runner's checkout never fetched it).
#
# Architect finding (thr_74hvpkqnxjp9q, CI-red re-review): the object
# being PRESENT is not the whole predicate. A shallow clone can fetch
# `$REVISION` as its own separate shallow root (e.g. an earlier, narrower
# fetch, or a shallow-since boundary) while never fetching the commits
# connecting it to HEAD — `cat-file -e` then succeeds but `merge-base
# --is-ancestor` cannot prove ancestry, and the ORIGINAL code below only
# ever triggered self-heal on `cat-file` failing, so this state skipped
# deepening entirely and fell straight through to a false "not an
# ancestor" rejection of a genuine ancestor. Fixed by making `resolved()`
# require BOTH conditions — object present AND ancestry provable — and
# re-checking both after every deepen/unshallow step, not just the first.
#
# Escalating deepen rather than always paying for a full `--unshallow`
# (which every CI run would otherwise trigger unconditionally, since
# CI's checkout is shallow by default and this gate runs on every PR).
# `library/REVISION` is normally bumped to a recent ancestor on each
# rebase (see the fold history in this WP), so a modest deepen resolves
# the common case; an old anchor still resolves via `--unshallow`.
revision_resolved() {
  git -C "$REPO_ROOT" cat-file -e "${REVISION}^{commit}" 2>/dev/null \
    && git -C "$REPO_ROOT" merge-base --is-ancestor "$REVISION" HEAD 2>/dev/null
}

SELF_HEAL_ATTEMPTED=0
if [ "$(git -C "$REPO_ROOT" rev-parse --is-shallow-repository 2>/dev/null)" = "true" ] \
   && ! revision_resolved; then
  SELF_HEAL_ATTEMPTED=1
  for DEPTH in 50 500 5000 50000; do
    git -C "$REPO_ROOT" fetch --quiet --deepen="$DEPTH" origin 2>/dev/null || true
    revision_resolved && break
  done
  if ! revision_resolved; then
    git -C "$REPO_ROOT" fetch --quiet --unshallow origin 2>/dev/null || true
  fi
fi

if ! git -C "$REPO_ROOT" cat-file -e "${REVISION}^{commit}" 2>/dev/null; then
  if [ "$SELF_HEAL_ATTEMPTED" = "1" ]; then
    # Librarian QA (thr_74hvpkqnxjp9q, CI-red fold): distinguish "we
    # deepened a shallow clone and the object is still missing" — which
    # points at the REMOTE (unreachable, or the object genuinely isn't
    # there) — from the plain shape-only case below, where deepening was
    # never even attempted because the checkout already had full history.
    # Conflating them into one message hides which side of the fence the
    # failure is on. Keeps the "does not resolve to a real commit object"
    # phrase as a common substring with the plain case so a caller
    # matching on that text doesn't need to special-case this branch.
    echo "gen-doc-status: library/REVISION '${REVISION}' does not resolve to a real commit object" >&2
    echo "  even after deepening this shallow clone (tried --deepen=50/500/5000/50000" >&2
    echo "  and --unshallow against 'origin') — either the object does not exist" >&2
    echo "  upstream, or 'origin' was unreachable" >&2
  else
    echo "gen-doc-status: library/REVISION '${REVISION}' does not resolve to a real commit object" >&2
  fi
  exit 1
fi
if ! git -C "$REPO_ROOT" merge-base --is-ancestor "$REVISION" HEAD 2>/dev/null; then
  if [ "$SELF_HEAL_ATTEMPTED" = "1" ]; then
    # Architect finding: the object can be present (a separate shallow
    # root) while the connecting history to HEAD is still missing. If
    # deepening/unshallowing ran and ancestry STILL can't be proven, full
    # history is now present (or fetching failed) — this is no longer a
    # "maybe just needs more history" case, so say so distinctly from the
    # plain (never-shallow) case below.
    echo "gen-doc-status: library/REVISION '${REVISION}' is not an ancestor of the current tree" >&2
    echo "  (HEAD) even after deepening this shallow clone — the object was present but" >&2
    echo "  the connecting history was not, and still isn't after fetching; this revision" >&2
    echo "  is genuinely not an ancestor, or 'origin' was unreachable during the fetch" >&2
  else
    echo "gen-doc-status: library/REVISION '${REVISION}' is not an ancestor of the current tree (HEAD)" >&2
  fi
  exit 1
fi

# --- currency: REVISION must certify something about the CORPUS, not just --
# --- name a real ancestor commit (DOC-CURRENCY-ANCHOR) ---------------------
#
# Everything above establishes that REVISION names a real commit and that
# it is an ancestor of HEAD — and nothing more. `library/STATUS.md`'s claim
# is "the corpus was validated as of REVISION"; neither fact above reads a
# single byte of anything the corpus cites. Grounded, un-mutated, on
# `origin/main @ 6be9754b` (adversary, `evt_6c9mhr3tg9pfg`): `STATUS.md`
# stamped "Validated revision e5a400c7", and `git ls-tree e5a400c7 --
# library/` returns ZERO entries — the corpus is stamped validated at a
# revision where it did not yet exist, and every check above still passes.
#
# Two DISTINCT properties, checked separately so their diagnostics don't
# get conflated (AC-2 asks for exactly this distinguishability):

# (a) library/'s own corpus must already exist at REVISION — otherwise
# nothing was there to validate. This is the bootstrap gap made explicit:
# a REVISION set before library/manifest.toml was ever introduced is not a
# stale-but-once-valid revision, it is a revision that never had a corpus
# to certify anything about.
if ! git -C "$REPO_ROOT" cat-file -e "${REVISION}:library/manifest.toml" 2>/dev/null; then
  echo "gen-doc-status: library/REVISION '${REVISION}' predates library/'s own" >&2
  echo "  introduction — library/manifest.toml did not exist at that revision, so" >&2
  echo "  nothing was there to validate. This is distinct from a cited source" >&2
  echo "  drifting (below): REVISION must point at or after the commit that" >&2
  echo "  first introduced library/manifest.toml." >&2
  exit 1
fi

# (b) every manifest `sources` entry cited by a NON-generated document —
# the external claims the corpus actually rests its authority on — must be
# byte-unchanged between REVISION and HEAD. Librarian QA (thr_15yrvjrpap9td,
# first pass): an earlier cut of this check blanket-skipped every
# `library/`-prefixed source, which silently exempted `library/STATUS.md`'s
# own declared `sources` (`library/manifest.toml`, `library/REVISION`) from
# the very token (`source-currency`) its manifest record claimed to carry —
# a hidden exception contradicting AC-1's own text, not the issue's
# sanctioned "visibly weakened" branch. Fixed two ways:
#
# - `library/manifest.toml` is NOT exempted by path — it is bound like any
#   other source, but ONLY for documents whose `kind` is not `status`.
#   Nothing currently cites it except `STATUS.md` itself, so this has no
#   live effect today; it stops being a silent carve-out and becomes a
#   principled per-document-kind rule instead.
# - `library/REVISION` remains the ONE exemption, and it is structural, not
#   a convenience: it is `STATUS.md`'s own anchor value, not an external
#   claim, and its file content differs from itself by construction on
#   every legitimate bump (the parent-commit self-reference this script's
#   header explains) — checking it would fail every time REVISION is used
#   correctly. `STATUS.md`'s manifest record visibly narrows its own claim
#   (`crates/ken-cli/tests/library_documentation_gates.rs`,
#   `applicable_validation_tokens`): a `kind = "status"` document does not
#   carry `source-currency` at all — its freshness is what `generated-
#   current` (idempotency) already establishes, which subsumes "unchanged
#   since REVISION" for a document that is, by definition, always
#   regenerated fresh from the current working tree.
#
# Extraction lives entirely in the awk below (no `grep -o` stage this
# time — an earlier cut piped through one and had to guard its "zero
# matches" exit-1 with `|| true` under `set -o pipefail`/`set -e`; folding
# the quoted-string extraction into awk itself avoids re-introducing that
# trap). It emits one bare source path per line, tracking each document's
# `kind` field (which the manifest always states BEFORE `sources`,
# matching this file's other single-pass, controlled-subset parsers) so a
# `status`-kind document's own sources are excluded from extraction
# entirely, not filtered out downstream by path.
CITED_SOURCES="$(awk '
  /^\[\[document\]\]/ { kind = "" }
  /^kind[[:space:]]*=/ {
    k = $0
    sub(/^kind[[:space:]]*=[[:space:]]*"/, "", k)
    sub(/".*/, "", k)
    kind = k
  }
  /^sources[[:space:]]*=/ { capture = 1; buf = "" }
  capture { buf = buf "\n" $0 }
  capture && /\]/ {
    capture = 0
    if (kind != "status") {
      line = buf
      while (match(line, /"[^"]*"/)) {
        print substr(line, RSTART + 1, RLENGTH - 2)
        line = substr(line, RSTART + RLENGTH)
      }
    }
  }
' "$MANIFEST" | sed 's/#.*//' | sort -u)"

# Librarian QA (thr_15yrvjrpap9td, first pass, finding 2): a cited source
# that is a SYMLINK passes `git diff --quiet REVISION HEAD -- path` by
# comparing the symlink's own (unchanged) target-path blob, never the
# real file's content the symlink resolves to — demonstrated live: a
# manifest source pointing at a symlink whose TARGET body changed stayed
# green end-to-end. `git diff`/`cat-file` operate on git's tracked blob for
# that path, which for a symlink IS the target-path string, not the
# resolved content — this is the exact same escape class gate 1's
# `walk_library` already rejects (fail closed on a symlink rather than
# silently reading through it). Checked via the tracked git MODE
# (`120000` = symlink), at both endpoints — a source that is a regular
# file at REVISION but a symlink at HEAD (or vice versa) is just as
# unverifiable as one that is a symlink throughout.
DRIFTED=""
while IFS= read -r path; do
  [ -z "$path" ] && continue
  case "$path" in
    library/REVISION) continue ;;
  esac
  mode_head="$(git -C "$REPO_ROOT" ls-tree HEAD -- "$path" 2>/dev/null | awk '{print $1; exit}')"
  mode_rev="$(git -C "$REPO_ROOT" ls-tree "$REVISION" -- "$path" 2>/dev/null | awk '{print $1; exit}')"
  if [ "$mode_head" = "120000" ] || [ "$mode_rev" = "120000" ]; then
    DRIFTED="${DRIFTED}  - ${path} (symlink source — content-currency cannot verify through a\n    symlink indirection; cite the real file the symlink resolves to instead)
"
    continue
  fi
  if ! git -C "$REPO_ROOT" cat-file -e "${REVISION}:${path}" 2>/dev/null; then
    DRIFTED="${DRIFTED}  - ${path} (does not exist at REVISION)
"
    continue
  fi
  if ! git -C "$REPO_ROOT" diff --quiet "$REVISION" HEAD -- "$path" 2>/dev/null; then
    DRIFTED="${DRIFTED}  - ${path}
"
  fi
done <<<"$CITED_SOURCES"

if [ -n "$DRIFTED" ]; then
  echo "gen-doc-status: cited source(s) changed between REVISION and HEAD — the" >&2
  echo "  currency claim is no longer backed by evidence for:" >&2
  printf '%s' "$DRIFTED" >&2
  echo "  Re-validate the corpus against the new content, then bump" >&2
  echo "  library/REVISION to reflect that." >&2
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

# Field separator is `|`, not a tab. Bash `read` (and, with some awk
# implementations, `OFS`) treats a tab as an IFS *whitespace* character
# regardless of a custom `IFS=$'\t'` setting — it collapses runs of it
# and drops empty fields between them, silently SHIFTING every field
# after a blank one. `|` isn't in that whitespace class, so an empty
# `authority`/`availability` (the exact shape a malformed manifest
# record produces) round-trips as a genuinely empty field, not a
# shifted one. Verified: `printf 'a|b||c\n' | { IFS='|' read ...; }`
# preserves the empty third field; the tab form did not.
awk '
  function field(line,  v) {
    v = line
    sub(/^[a-z]+[[:space:]]*=[[:space:]]*"/, "", v)
    sub(/".*/, "", v)
    return v
  }
  /^\[\[document\]\]/ {
    if (path != "") print path "|" kind "|" authority "|" availability
    path = ""; kind = ""; authority = ""; availability = ""
    next
  }
  /^path[[:space:]]*=/         { path = field($0) }
  /^kind[[:space:]]*=/         { kind = field($0) }
  /^authority[[:space:]]*=/    { authority = field($0) }
  /^availability[[:space:]]*=/ { availability = field($0) }
  END { if (path != "") print path "|" kind "|" authority "|" availability }
' "$MANIFEST" > "$TMP_TABLE"

ROWS=""
COUNT=0
# Librarian QA (thr_74hvpkqnxjp9q, fifth pass): a 5th `extra` `read`
# variable does NOT catch every embedded `|` — when the smuggled `|` is
# the LAST character of the last field (e.g. `availability = "current|"`),
# the trailing empty remainder `read` would assign to `extra` is
# discarded, not preserved, so `extra` comes back empty and the row
# silently passes with the delimiter stripped from the value. `read`'s
# field-slurping is the wrong tool for "did this row have the right shape
# at all" — count the delimiter directly instead. A well-formed row has
# EXACTLY three `|` characters (four fields); any other count — whether
# from a smuggled `|` in a middle field or a trailing one — is rejected
# before `read` ever gets to (mis)parse it.
while IFS= read -r row; do
  [ -z "$row" ] && continue
  pipe_count=$(printf '%s' "$row" | tr -cd '|' | wc -c)
  if [ "$pipe_count" -ne 3 ]; then
    echo "gen-doc-status: a manifest row has $pipe_count '|' characters" >&2
    echo "  (expected exactly 3 — one field separator too many or too few," >&2
    echo "  which means a scalar smuggled the transport delimiter): $row" >&2
    exit 1
  fi
  IFS='|' read -r path kind authority availability <<<"$row"
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
