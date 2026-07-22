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

# Librarian QA (thr_15yrvjrpap9td, third pass): a `key = value`-shaped line
# at column 0 INSIDE a still-open multi-line `sources = [ ... ]` array
# desyncs the two manifest consumers. Rust's `parse_manifest` never
# reinterprets a line as a field once it is inside array continuation — it
# just accumulates raw text and quote-extracts from it, so `kind = "status"`
# sitting inside the array is swallowed as literal text and `"status"` is
# extracted as a spurious extra `sources` entry, while the document's real,
# final `kind` is whatever the LAST proper `kind =` line outside the array
# set it to. Both awk parsers in this script instead match `/^kind[[:space:]
# ]*=/` unconditionally at column 0, with no notion of "am I inside an open
# array" — so the exact same line flips their view of the document's `kind`
# instead. Live repro (librarian, scratch commit `1fab9704`): this spoofed a
# document's `kind` to `status` in the awk's eyes only, which made the new
# content-currency check (gate 7) treat it as exempt and silently drop a
# genuinely drifted cited source. Rejected outright here, once, before any
# awk touches the manifest — closing the ambiguity is simpler and safer than
# trying to make three independent parsers agree on how to resolve it.
MALFORMED_ARRAY_LINE="$(awk '
  BEGIN { open = 0 }
  {
    if (open) {
      if ($0 ~ /^[a-z_]+[[:space:]]*=/) { print NR": "$0 }
      if ($0 ~ /\]/) { open = 0 }
      next
    }
    if ($0 ~ /=[[:space:]]*\[[[:space:]]*$/) { open = 1 }
  }
' "$MANIFEST")"
if [ -n "$MALFORMED_ARRAY_LINE" ]; then
  echo "gen-doc-status: library/manifest.toml has a field-looking line" >&2
  echo "  (\"key = value\" at column 0) inside a still-open multi-line array" >&2
  echo "  — the manifest's two consumers (this script's awk, the Rust gate's" >&2
  echo "  parser) do not agree on where the array ends for a shape like this," >&2
  echo "  which can spoof a document's kind or smuggle an extra source:" >&2
  echo "$MALFORMED_ARRAY_LINE" | sed 's/^/  /' >&2
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

# --- REVISION must survive a squash-merge onto main, checked ON THE BRANCH,
# --- BEFORE publish (DOC-CURRENCY-ANCHOR hotfix) ---------------------------
#
# Landed post-merge outage (thr_15yrvjrpap9td, 2026-07-22): the checks above
# only ever verify REVISION is an ancestor of *local HEAD* — on a WP branch,
# that includes the branch's own not-yet-merged commits, which VANISH as
# ancestors the moment the publisher squash-merges (the squash commit's sole
# parent is the pre-merge `main` tip, never the branch). A branch-local
# REVISION value resolves fine on the branch, at every review round, in CI —
# and only fails once HEAD becomes `main` after the last check anyone runs.
#
# Librarian QA (thr_15yrvjrpap9td, hotfix re-review, live commit `61f07dc1`):
# the first cut of this hotfix's regression tested only a fully SYNTHETIC
# repo — it proved the SCRIPT's post-squash behavior in the abstract but
# never touched this repository's actual `library/REVISION`, so a
# branch-local value here would still pass every existing check, reproducing
# the exact outage undetected. This check closes that: while still ON THE
# BRANCH, verify REVISION is an ancestor of `origin/main` (not merely of
# local HEAD) — `origin/main` only ever moves forward on this repository's
# linear history, so a commit that is genuinely on it now stays on it
# forever, and a commit that ISN'T yet (a branch-local one) is caught here,
# before publish, instead of after.
#
# Librarian QA (thr_15yrvjrpap9td, hotfix fold-2 re-review): a "best-effort,
# skip if no anchor resolves" version of this check fails OPEN in exactly
# the shallow/no-configured-remote topology it exists to guard. Live proof:
# delete `refs/remotes/origin/main`, point `origin` at an unreachable repo,
# set REVISION to a genuine branch-local commit — the "best-effort" check
# silently skipped and the script exited 0, reproducing the outage this
# whole mechanism exists to prevent. A trust-anchor check that degrades to
# a no-op when the anchor is unavailable is not a check.
#
# Fixed: resolving the anchor now ends in exactly two states — a verified
# ref used for the ancestry check, or an explicit diagnostic and exit 1.
# There is no third, silent-pass state. The fetch destination is explicit
# (`main:refs/remotes/origin/main`), not assumed from a bare `fetch origin
# main` — the resulting ref is verified afterward either way, not trusted
# because the fetch command exited 0.
if ! git -C "$REPO_ROOT" rev-parse --verify -q refs/remotes/origin/main >/dev/null 2>&1; then
  git -C "$REPO_ROOT" fetch --quiet origin main:refs/remotes/origin/main 2>/dev/null || true
fi
if ! git -C "$REPO_ROOT" rev-parse --verify -q refs/remotes/origin/main >/dev/null 2>&1; then
  echo "gen-doc-status: cannot establish the origin/main trust anchor —" >&2
  echo "  refs/remotes/origin/main does not resolve, and fetching 'main' from" >&2
  echo "  'origin' failed. REVISION's ancestry can only be verified against" >&2
  echo "  local HEAD without this anchor, which is exactly the check that let" >&2
  echo "  a branch-local commit reach main once already. Fix 'origin' access" >&2
  echo "  (or the local ref) — this check does not skip." >&2
  exit 1
fi
if ! git -C "$REPO_ROOT" merge-base --is-ancestor "$REVISION" refs/remotes/origin/main 2>/dev/null; then
  echo "gen-doc-status: library/REVISION '${REVISION}' is not an ancestor of" >&2
  echo "  origin/main — it resolves on this branch right now, but a branch-local" >&2
  echo "  commit does not survive a squash-merge onto main. REVISION must name a" >&2
  echo "  commit that is already on main (e.g. the branch's merge base), never a" >&2
  echo "  commit that exists only on this branch." >&2
  exit 1
fi

# --- currency: REVISION must certify something about the CORPUS, not just --
# --- name a real ancestor commit (DOC-CURRENCY-ANCHOR) ---------------------
#
# Everything above establishes that REVISION names a real commit and that
# it is an ancestor of HEAD — and nothing more. Grounded, un-mutated, on
# `origin/main @ 6be9754b` (adversary, `evt_6c9mhr3tg9pfg`): `STATUS.md`
# stamped "Validated revision e5a400c7", and `git ls-tree e5a400c7 --
# library/` returns ZERO entries — the corpus is stamped validated at a
# revision where it did not yet exist, and every check above still passes.
# REVISION alone is a PROVENANCE anchor only (SRC-ATTEST Part 1, below) — it
# is checked here for the same bootstrap reason, not as a claim about cited
# source bytes.

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
# attested by `library/SOURCE-ATTESTATIONS`, a squash-stable whole-source
# ledger (SRC-ATTEST Part 1). The `REVISION → HEAD` cited-byte diff this
# comment used to describe is GONE — the Librarian proved it impossible for
# any WP that itself edits a cited source (evt_6t6wz1aw18291): no commit can
# be simultaneously an ancestor of `origin/main` (survives a squash) and a
# holder of the not-yet-merged bytes that make the citation current. A
# candidate-time BLOB-OID attestation dissolves that: the blob survives even
# though the branch commit does not, so the ledger can be updated in the
# same commit that changes the cited source, and `REVISION` no longer needs
# to certify cited content at all — it stays a provenance/bootstrap anchor
# only (checked above). Replacing rather than layering this on top of the
# old check is deliberate: keeping both preserves the exact impossibility
# this WP exists to dissolve.
#
# Librarian QA (thr_15yrvjrpap9td, first pass): an earlier cut of the OLD
# check blanket-skipped every `library/`-prefixed source, which silently
# exempted `library/STATUS.md`'s own declared `sources`
# (`library/manifest.toml`, `library/REVISION`) from the very token
# (`source-currency`) its manifest record claimed to carry — a hidden
# exception contradicting AC-1's own text, not the issue's sanctioned
# "visibly weakened" branch. The ledger population below inherits that fix
# unchanged, two ways:
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
# Librarian QA (thr_15yrvjrpap9td, second pass): TOML duplicate keys are
# invalid but NEITHER consumer here rejects them, and an earlier cut of
# this awk decided "is this record's sources checked" the instant a
# `sources = [...]` block closed — using whatever `kind` had been seen SO
# FAR. Live repro (scratch probe `e9927bec`): `kind = "status"` placed
# right before `sources`, `kind = "portal"` restored right after it — the
# Rust gate's `parse_manifest` (which, like the render awk elsewhere in
# this file, keeps whatever `key =` value it saw LAST for the whole
# record) computes a final `kind` of `portal` and expects `source-currency`
# to apply; this awk, deciding at `sources`-close time, saw `status` and
# silently dropped the source from `CITED_SOURCES` — a body-drifted cited
# source stayed green end-to-end. Fixed by making the decision at the
# SAME point Rust makes it: buffer this record's sources text and defer
# the "checked or not" decision to the record's END (the next
# `[[document]]`, or EOF) — using the FINAL `kind` for the whole record,
# identically to the Rust parser and to this file's own render awk, so a
# duplicate/out-of-order `kind` line can no longer desync the two.
CITED_SOURCES="$(awk '
  function flush_record() {
    if (kind != "status" && buf != "") {
      line = buf
      while (match(line, /"[^"]*"/)) {
        print substr(line, RSTART + 1, RLENGTH - 2)
        line = substr(line, RSTART + RLENGTH)
      }
    }
    kind = ""; buf = ""; capture = 0
  }
  /^\[\[document\]\]/ { flush_record(); next }
  /^kind[[:space:]]*=/ {
    k = $0
    sub(/^kind[[:space:]]*=[[:space:]]*"/, "", k)
    sub(/".*/, "", k)
    kind = k
    next
  }
  /^sources[[:space:]]*=/ { capture = 1; buf = "" }
  capture { buf = buf "\n" $0 }
  capture && /\]/ { capture = 0 }
  END { flush_record() }
' "$MANIFEST" | sed 's/#.*//' | sort -u)"

# --- ledger: library/SOURCE-ATTESTATIONS is the source-currency authority -
# (SRC-ATTEST Part 1). This block is READ-ONLY — it never writes the ledger
# file. Generation is a separate entry point (`scripts/gen-source-
# attestations.sh`), invoked only by the Librarian after semantic review,
# and it never writes the real ledger path either (it writes a `.proposed`
# sibling) — see that script's header for why the two are kept apart.
TMP_LEDGER_PAIRS="$(mktemp)"
trap 'rm -f "$TMP_LEDGER_PAIRS"' EXIT

LEDGER_FILE="$REPO_ROOT/library/SOURCE-ATTESTATIONS"
if [ ! -f "$LEDGER_FILE" ]; then
  echo "gen-doc-status: $LEDGER_FILE not found — every manifest-cited source" >&2
  echo "  needs an attested blob OID. Run scripts/gen-source-attestations.sh" >&2
  echo "  to produce a proposed ledger for review, then commit it as" >&2
  echo "  library/SOURCE-ATTESTATIONS once the Librarian has reviewed it." >&2
  exit 1
fi

OBJECT_FORMAT="$(git -C "$REPO_ROOT" rev-parse --show-object-format 2>/dev/null || echo sha1)"
case "$OBJECT_FORMAT" in
  sha1) OID_RE='^[0-9a-f]{40}$' ;;
  sha256) OID_RE='^[0-9a-f]{64}$' ;;
  *) OID_RE='^[0-9a-f]{40,64}$' ;;
esac

LEDGER_HEADER="$(head -n1 "$LEDGER_FILE")"
case "$LEDGER_HEADER" in
  "# ken-source-attestation-v1 object-format=${OBJECT_FORMAT}") : ;;
  "# ken-source-attestation-v1 object-format="*)
    echo "gen-doc-status: $LEDGER_FILE declares an object-format that does" >&2
    echo "  not match this repository's ('${OBJECT_FORMAT}'): '${LEDGER_HEADER}'" >&2
    exit 1
    ;;
  *)
    echo "gen-doc-status: $LEDGER_FILE has no recognized" >&2
    echo "  '# ken-source-attestation-v1 object-format=...' header line" >&2
    exit 1
    ;;
esac

# Required population = every unique manifest-cited path (anchor stripped),
# same source list content-currency has always used, sorted+deduped.
REQUIRED_PATHS="$(printf '%s\n' "$CITED_SOURCES" | sed 's/#.*//;s/^library\/REVISION$//' | sort -u | sed '/^$/d')"

# Ledger rows, tab-separated `<oid>\t<path>`, after the one header line.
# Exact shape enforced before any semantic check: exactly one tab per row,
# a well-formed OID for this repository's object format, sorted, unique
# paths, no path escaping the repository (leading `/`, `..` component).
LEDGER_ROWS="$(tail -n +2 "$LEDGER_FILE")"
LEDGER_PATHS=""
PREV_PATH=""
BAD_ROW=""
while IFS= read -r row; do
  [ -z "$row" ] && continue
  tab_count=$(printf '%s' "$row" | tr -cd '\t' | wc -c)
  if [ "$tab_count" -ne 1 ]; then
    BAD_ROW="${BAD_ROW}  - malformed row (expected exactly one tab): ${row}
"
    continue
  fi
  oid="${row%%$'\t'*}"
  path="${row#*$'\t'}"
  if ! printf '%s' "$oid" | grep -qE "$OID_RE"; then
    BAD_ROW="${BAD_ROW}  - malformed OID for object-format=${OBJECT_FORMAT}: ${row}
"
    continue
  fi
  case "$path" in
    /*|*..*)
      BAD_ROW="${BAD_ROW}  - path escapes the repository: ${row}
"
      continue
      ;;
  esac
  if [ -n "$PREV_PATH" ] && [ "$(printf '%s\n%s' "$PREV_PATH" "$path" | sort | head -n1)" != "$PREV_PATH" ]; then
    BAD_ROW="${BAD_ROW}  - ledger rows are not sorted by path at: ${row}
"
  fi
  if [ "$path" = "$PREV_PATH" ]; then
    BAD_ROW="${BAD_ROW}  - duplicate path row: ${row}
"
  fi
  PREV_PATH="$path"
  LEDGER_PATHS="${LEDGER_PATHS}${path}
"
  # oid:path pairs, one per line, for the per-path lookup below.
  printf '%s\t%s\n' "$oid" "$path" >> "$TMP_LEDGER_PAIRS"
done <<<"$LEDGER_ROWS"

if [ -n "$BAD_ROW" ]; then
  echo "gen-doc-status: $LEDGER_FILE has malformed row(s):" >&2
  printf '%s' "$BAD_ROW" >&2
  exit 1
fi

LEDGER_PATHS="$(printf '%s' "$LEDGER_PATHS" | sort -u)"

MISSING_FROM_LEDGER="$(comm -23 <(printf '%s\n' "$REQUIRED_PATHS") <(printf '%s\n' "$LEDGER_PATHS"))"
EXTRA_IN_LEDGER="$(comm -13 <(printf '%s\n' "$REQUIRED_PATHS") <(printf '%s\n' "$LEDGER_PATHS"))"
if [ -n "$MISSING_FROM_LEDGER" ] || [ -n "$EXTRA_IN_LEDGER" ]; then
  echo "gen-doc-status: library/SOURCE-ATTESTATIONS does not exactly match the" >&2
  echo "  current manifest-cited source set:" >&2
  if [ -n "$MISSING_FROM_LEDGER" ]; then
    echo "  missing from ledger (cited, not attested):" >&2
    printf '%s\n' "$MISSING_FROM_LEDGER" | sed 's/^/    - /' >&2
  fi
  if [ -n "$EXTRA_IN_LEDGER" ]; then
    echo "  stale in ledger (attested, no longer cited):" >&2
    printf '%s\n' "$EXTRA_IN_LEDGER" | sed 's/^/    - /' >&2
  fi
  echo "  Run scripts/gen-source-attestations.sh, review, and commit the" >&2
  echo "  updated ledger." >&2
  exit 1
fi

# Librarian QA (thr_15yrvjrpap9td, first pass, finding 2), preserved under
# the ledger: a cited source that is a SYMLINK at HEAD is unverifiable
# through the indirection (its tracked blob IS the target-path string, not
# the resolved content) — the exact escape class gate 1's `walk_library`
# already rejects. Checked via the tracked git MODE (`120000`), at HEAD.
DRIFTED=""
while IFS= read -r pair; do
  [ -z "$pair" ] && continue
  ledger_oid="${pair%%$'\t'*}"
  path="${pair#*$'\t'}"
  head_entry="$(git -C "$REPO_ROOT" ls-tree HEAD -- "$path" 2>/dev/null)"
  mode_head="$(printf '%s' "$head_entry" | awk '{print $1; exit}')"
  oid_head="$(printf '%s' "$head_entry" | awk '{print $3; exit}')"
  if [ "$mode_head" = "120000" ]; then
    DRIFTED="${DRIFTED}  - ${path} (symlink source — content-currency cannot verify through a symlink indirection; cite the real file it resolves to instead)
"
    continue
  fi
  if [ -z "$oid_head" ]; then
    DRIFTED="${DRIFTED}  - ${path} (does not exist at HEAD)
"
    continue
  fi
  if [ "$mode_head" != "100644" ] && [ "$mode_head" != "100755" ]; then
    DRIFTED="${DRIFTED}  - ${path} (not a regular tracked file at HEAD — mode ${mode_head})
"
    continue
  fi
  if [ "$oid_head" != "$ledger_oid" ]; then
    DRIFTED="${DRIFTED}  - ${path} (attested ${ledger_oid}, actual ${oid_head})
"
  fi
done < "$TMP_LEDGER_PAIRS"

if [ -n "$DRIFTED" ]; then
  echo "gen-doc-status: cited source(s) changed since their last attestation —" >&2
  echo "  the currency claim is no longer backed by evidence for:" >&2
  printf '%s' "$DRIFTED" >&2
  echo "  Re-validate the corpus against the new content, then run" >&2
  echo "  scripts/gen-source-attestations.sh and commit the updated ledger." >&2
  exit 1
fi

# Attested source-set digest: a single value `STATUS.md` can render distinct
# from `REVISION` (the provenance anchor, above), so a reader isn't left
# inferring "which value actually backs source-currency" from prose alone —
# rendered next to REVISION, not in place of it (SRC-ATTEST Part 1 row 6).
LEDGER_DIGEST="$(sha256sum "$LEDGER_FILE" | awk '{print $1}')"

# --- manifest parsing -----------------------------------------------------
# library/manifest.toml is a small, hand-controlled TOML subset: a run of
# `[[document]]` tables, each with flat `key = "value"` scalar fields. This
# single-pass awk parser depends on that shape, not on general TOML — it is
# not meant to read arbitrary TOML.

TMP_TABLE="$(mktemp)"
TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_TABLE" "$TMP_OUT" "$TMP_LEDGER_PAIRS"' EXIT

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
is this file's one job: it is anchored to two DISTINCT values, never a
typed date (docs/program/12-documentation-program.md §2).

**Provenance revision:** \`${REVISION}\`

Recorded explicitly in \`library/REVISION\`, not derived from \`git
rev-parse HEAD\` at generation time. A live-HEAD anchor is self-
referential for the commit that introduces or updates this file; an
explicit, deliberately-set input has no such cycle, and \`--check\`
regenerating from the same recorded value reproduces this file
byte-for-byte (AC3). This value is a **bootstrap anchor only** — it
proves \`library/\` already existed at some point on \`main\`'s history.
**It does not, by itself, certify any cited source's bytes** (SRC-ATTEST
Part 1) — that claim is the attested source-set digest below.

**Attested source-set digest:** \`${LEDGER_DIGEST}\`

The SHA-256 of \`library/SOURCE-ATTESTATIONS\`, the ledger binding every
manifest-cited source to its exact blob OID at the commit the Librarian
last reviewed it. This is the source-currency authority: it is
squash-stable (a blob OID survives even though the commit that introduced
it does not), so it can be updated in the very commit that changes a
cited source, unlike \`REVISION\`. Regenerate a proposed ledger with
\`scripts/gen-source-attestations.sh\`; only the Librarian commits it.

## Registered documents

Every row below is one \`[[document]]\` entry in \`library/manifest.toml\`.
A document with no row here has no manifest entry and fails gate 1.

| Path | Kind | Authority | Availability |
|---|---|---|---|
${ROWS}
**Total:** ${COUNT} registered document(s).

## Regenerating

\`\`\`
scripts/gen-doc-status.sh              # regenerate this file in place (CI-checked)
scripts/gen-doc-status.sh --check      # verify committed file matches (CI)
scripts/gen-source-attestations.sh     # render a PROPOSED ledger for review;
                                        # only the Librarian commits it
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
