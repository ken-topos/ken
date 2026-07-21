#!/usr/bin/env bash
# check-issue-schema.sh — validate docs/program/issues/*.md frontmatter.
#
# Fails non-zero (with a per-file diagnostic on stderr) when any issue file:
#   - is missing a required frontmatter field
#   - has an unknown `status` value
#   - has an `id` that does not match its filename
#   - has a `depends_on`/`blocks` entry that names an id no file defines
#   - duplicates an `id` already defined by another file
#
# Plain bash + standard tools (awk, sed, grep) — no new dependencies.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ISSUES_DIR="${1:-$REPO_ROOT/docs/program/issues}"

REQUIRED_FIELDS=(id title status owner size gate depends_on blocks github origin)
VALID_STATUSES=(draft ready active in-review merged closed)

fail=0

# --- helpers -----------------------------------------------------------

extract_frontmatter() {
  # Print only the YAML frontmatter block (between the first two `---` lines).
  awk '
    /^---[ \t]*$/ { c++; next }
    c == 1        { print }
    c >= 2        { exit }
  ' "$1"
}

get_field() {
  # $1 = frontmatter text, $2 = field name -> prints the raw value (may be "").
  # Never fails (a missing field is reported by the caller, not by grep's
  # no-match exit status), so this is safe under `set -e -o pipefail`.
  printf '%s\n' "$1" | { grep -E "^${2}:" || true; } | head -n1 \
    | sed -E "s/^${2}:[[:space:]]*//"
}

strip_quotes() {
  local v="$1"
  v="${v%\"}"; v="${v#\"}"
  printf '%s' "$v"
}

list_items() {
  # $1 = "[A, B]" or "[]" -> one item per line, trimmed, quotes stripped.
  local s="$1"
  s="${s#\[}"; s="${s%\]}"
  [ -z "$s" ] && return 0
  printf '%s\n' "$s" | tr ',' '\n' | sed -E 's/^[[:space:]]+//; s/[[:space:]]+$//' \
    | while IFS= read -r item; do
        # Force a trailing newline on every emitted item (including the
        # last) so `while read` at the call site never silently drops it.
        [ -n "$item" ] && printf '%s\n' "$(strip_quotes "$item")"
      done
}

is_valid_status() {
  local s="$1" v
  for v in "${VALID_STATUSES[@]}"; do
    [ "$s" = "$v" ] && return 0
  done
  return 1
}

if [ ! -d "$ISSUES_DIR" ]; then
  echo "check-issue-schema: issues directory not found: $ISSUES_DIR" >&2
  exit 1
fi

shopt -s nullglob
files=("$ISSUES_DIR"/*.md)

if [ ${#files[@]} -eq 0 ]; then
  echo "check-issue-schema: no issue files found in $ISSUES_DIR" >&2
  exit 1
fi

# --- pass 1: per-file field validation + id collection ------------------

all_ids=()
declare -A id_to_file

for f in "${files[@]}"; do
  base="$(basename "$f" .md)"
  fm="$(extract_frontmatter "$f")"

  if [ -z "$fm" ]; then
    echo "FAIL $f: no YAML frontmatter block found (expected --- ... ---)" >&2
    fail=1
    continue
  fi

  file_ok=1
  declare -A field_val=()

  for field in "${REQUIRED_FIELDS[@]}"; do
    val="$(get_field "$fm" "$field")"
    field_val["$field"]="$val"
    if [ -z "$val" ]; then
      echo "FAIL $f: missing required field '$field'" >&2
      fail=1
      file_ok=0
    fi
  done

  id_val="$(strip_quotes "${field_val[id]:-}")"
  status_val="$(strip_quotes "${field_val[status]:-}")"

  if [ -n "$id_val" ] && [ "$id_val" != "$base" ]; then
    echo "FAIL $f: id '$id_val' does not match filename '$base'" >&2
    fail=1
    file_ok=0
  fi

  if [ -n "$status_val" ] && ! is_valid_status "$status_val"; then
    echo "FAIL $f: unknown status '$status_val' (valid: ${VALID_STATUSES[*]})" >&2
    fail=1
    file_ok=0
  fi

  if [ -n "$id_val" ]; then
    if [ -n "${id_to_file[$id_val]:-}" ]; then
      echo "FAIL $f: duplicate id '$id_val' (already defined by ${id_to_file[$id_val]})" >&2
      fail=1
      file_ok=0
    else
      id_to_file["$id_val"]="$f"
      all_ids+=("$id_val")
    fi
  fi

  unset field_val
done

# --- pass 2: depends_on / blocks reference existing ids ------------------

id_exists() {
  local target="$1" i
  for i in "${all_ids[@]}"; do
    [ "$i" = "$target" ] && return 0
  done
  return 1
}

for f in "${files[@]}"; do
  fm="$(extract_frontmatter "$f")"
  [ -z "$fm" ] && continue

  for field in depends_on blocks; do
    raw="$(get_field "$fm" "$field")"
    [ -z "$raw" ] && continue
    while IFS= read -r ref; do
      [ -z "$ref" ] && continue
      if ! id_exists "$ref"; then
        echo "FAIL $f: $field references unknown id '$ref'" >&2
        fail=1
      fi
    done < <(list_items "$raw")
  done
done

if [ "$fail" -ne 0 ]; then
  echo "check-issue-schema: FAILED" >&2
  exit 1
fi

echo "check-issue-schema: OK (${#all_ids[@]} issue files, all valid)"
exit 0
