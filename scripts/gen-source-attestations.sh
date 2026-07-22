#!/usr/bin/env bash
# gen-source-attestations.sh — render a PROPOSED library/SOURCE-ATTESTATIONS
# ledger from the current manifest-cited source set at HEAD.
#
# SRC-ATTEST Part 1 (docs/program/wp/SRC-ATTEST-currency-substrate.md), the
# non-automation boundary, Librarian-authoritative:
#
#   "The ledger MAY and SHOULD have a deterministic generator... What must
#    remain human is the AUTHORIZATION to update and commit its output."
#
# So this script is a SEPARATE entry point from scripts/gen-doc-status.sh,
# and it NEVER writes the real ledger path. It always writes a `.proposed`
# sibling (`library/SOURCE-ATTESTATIONS.proposed`) and stops — installing it
# is a deliberate, separate act by whoever reviewed the changed sources
# (the Librarian), never something this script or any CI/build step does
# automatically. "Regenerate whenever HEAD differs" must stay impossible;
# keeping generation and installation as two file paths, not one flag, is
# what makes that true by construction rather than by convention.
#
# Usage: scripts/gen-source-attestations.sh
#   Writes library/SOURCE-ATTESTATIONS.proposed. Never touches
#   library/SOURCE-ATTESTATIONS itself. Review the diff
#   (`diff library/SOURCE-ATTESTATIONS library/SOURCE-ATTESTATIONS.proposed`),
#   record which changed sources/pages were revalidated, then:
#     mv library/SOURCE-ATTESTATIONS.proposed library/SOURCE-ATTESTATIONS
#   and commit it deliberately.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MANIFEST="$REPO_ROOT/library/manifest.toml"
PROPOSED_FILE="$REPO_ROOT/library/SOURCE-ATTESTATIONS.proposed"

if [ ! -f "$MANIFEST" ]; then
  echo "gen-source-attestations: manifest not found: $MANIFEST" >&2
  exit 1
fi

# Required population: identical extraction to scripts/gen-doc-status.sh's
# CITED_SOURCES awk (every `sources` entry of a non-`status`-kind document,
# using each record's FINAL `kind`, anchor stripped, deduped) — kept as a
# literal duplicate rather than a shared sourced function, so a change to
# one is visible as a diff against the other rather than silently applying
# to both; `status_md_generation_is_idempotent` plus the ledger-population
# tests in crates/ken-cli/tests/library_documentation_gates.rs are the
# safety net that would catch the two drifting apart.
REQUIRED_PATHS="$(awk '
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
' "$MANIFEST" | sed 's/#.*//;s/^library\/REVISION$//' | sort -u | sed '/^$/d')"

OBJECT_FORMAT="$(git -C "$REPO_ROOT" rev-parse --show-object-format 2>/dev/null || echo sha1)"

# Same canonical-path test as scripts/gen-doc-status.sh's checker (kept as
# a literal duplicate, per this file's own extraction-duplication note
# above) — Architect finding dec_1n8mxg2b0m54w: `git ls-tree`/Rust both
# resolve `docs/./x` and `docs//x` to the same blob as `docs/x`, so
# attesting a noncanonical spelling would let a later noncanonical
# manifest citation match it as a raw string while aliasing the real path.
path_is_noncanonical() {
  local p="$1" part
  [ -z "$p" ] && return 0
  while IFS= read -r part; do
    case "$part" in
      ""|.|..) return 0 ;;
    esac
  done <<<"$(printf '%s' "$p" | tr '/' '\n')"
  return 1
}

BAD=""
ROWS=""
while IFS= read -r path; do
  [ -z "$path" ] && continue
  if path_is_noncanonical "$path"; then
    BAD="${BAD}  - ${path} (escapes the repository or is not canonical spelling)
"
    continue
  fi
  entry="$(git -C "$REPO_ROOT" ls-tree HEAD -- "$path" 2>/dev/null)"
  mode="$(printf '%s' "$entry" | awk '{print $1; exit}')"
  oid="$(printf '%s' "$entry" | awk '{print $3; exit}')"
  if [ -z "$oid" ]; then
    BAD="${BAD}  - ${path} (does not exist at HEAD)
"
    continue
  fi
  if [ "$mode" = "120000" ]; then
    BAD="${BAD}  - ${path} (symlink source — cite the real file it resolves to instead)
"
    continue
  fi
  if [ "$mode" != "100644" ] && [ "$mode" != "100755" ]; then
    BAD="${BAD}  - ${path} (not a regular tracked file — mode ${mode})
"
    continue
  fi
  ROWS="${ROWS}${oid}	${path}
"
done <<<"$REQUIRED_PATHS"

if [ -n "$BAD" ]; then
  echo "gen-source-attestations: cannot attest the following cited source(s):" >&2
  printf '%s' "$BAD" >&2
  echo "  Fix the manifest citation or the source itself before generating." >&2
  exit 1
fi

{
  echo "# ken-source-attestation-v1 object-format=${OBJECT_FORMAT}"
  printf '%s' "$ROWS" | sort -t'	' -k2
} > "$PROPOSED_FILE"

echo "gen-source-attestations: wrote $PROPOSED_FILE"
echo "  Review it (diff library/SOURCE-ATTESTATIONS library/SOURCE-ATTESTATIONS.proposed)," >&2
echo "  record which changed sources/pages were revalidated, then:" >&2
echo "    mv library/SOURCE-ATTESTATIONS.proposed library/SOURCE-ATTESTATIONS" >&2
echo "  and commit it deliberately. This script never installs it for you." >&2
