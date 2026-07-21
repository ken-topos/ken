#!/usr/bin/env bash
# gen-progress.sh — regenerate docs/program/IMPLEMENTATION-PROGRESS.md from
# the frontmatter of every docs/program/issues/*.md work-item file.
#
# This is the single source of truth for the tracker: status table,
# releasable frontier, blockers, and gate progress are all DERIVED from the
# issue files, never hand-edited. The hand-written preamble (purpose +
# status legend) is preserved verbatim as a template header below.
#
# Plain bash + standard tools (awk, sed, grep, sort) — no new dependencies.
#
# Usage: scripts/gen-progress.sh [--check]
#   (no args)  regenerate docs/program/IMPLEMENTATION-PROGRESS.md in place.
#   --check    write to a temp file and diff against the committed file;
#              exit non-zero if they differ (used by CI for idempotency).

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ISSUES_DIR="$REPO_ROOT/docs/program/issues"
OUT_FILE="$REPO_ROOT/docs/program/IMPLEMENTATION-PROGRESS.md"

MODE="${1:-write}"
if [ "$MODE" = "--check" ]; then
  MODE="check"
fi

# --- frontmatter parsing (shared shape with check-issue-schema.sh) -------

extract_frontmatter() {
  awk '
    /^---[ \t]*$/ { c++; next }
    c == 1        { print }
    c >= 2        { exit }
  ' "$1"
}

get_field() {
  printf '%s\n' "$1" | { grep -E "^${2}:" || true; } | head -n1 \
    | sed -E "s/^${2}:[[:space:]]*//"
}

strip_quotes() {
  local v="$1"
  v="${v%\"}"; v="${v#\"}"
  printf '%s' "$v"
}

list_items() {
  local s="$1"
  s="${s#\[}"; s="${s%\]}"
  [ -z "$s" ] && return 0
  printf '%s\n' "$s" | tr ',' '\n' | sed -E 's/^[[:space:]]+//; s/[[:space:]]+$//' \
    | while IFS= read -r item; do
        [ -n "$item" ] && printf '%s\n' "$(strip_quotes "$item")"
      done
}

if [ ! -d "$ISSUES_DIR" ]; then
  echo "gen-progress: issues directory not found: $ISSUES_DIR" >&2
  exit 1
fi

shopt -s nullglob
files=("$ISSUES_DIR"/*.md)
if [ ${#files[@]} -eq 0 ]; then
  echo "gen-progress: no issue files found in $ISSUES_DIR" >&2
  exit 1
fi

# Sort files by basename for a deterministic (idempotent) generation order.
IFS=$'\n' sorted_files=($(printf '%s\n' "${files[@]}" | sort)); unset IFS

declare -A f_title f_status f_owner f_size f_gate f_deps f_blocks f_github f_origin
ids=()

for f in "${sorted_files[@]}"; do
  fm="$(extract_frontmatter "$f")"
  id="$(strip_quotes "$(get_field "$fm" id)")"
  [ -z "$id" ] && { echo "gen-progress: $f has no 'id' field, skipping" >&2; continue; }
  ids+=("$id")
  f_title["$id"]="$(strip_quotes "$(get_field "$fm" title)")"
  f_status["$id"]="$(strip_quotes "$(get_field "$fm" status)")"
  f_owner["$id"]="$(strip_quotes "$(get_field "$fm" owner)")"
  f_size["$id"]="$(strip_quotes "$(get_field "$fm" size)")"
  f_gate["$id"]="$(strip_quotes "$(get_field "$fm" gate)")"
  f_deps["$id"]="$(get_field "$fm" depends_on)"
  f_blocks["$id"]="$(get_field "$fm" blocks)"
  f_github["$id"]="$(strip_quotes "$(get_field "$fm" github)")"
  f_origin["$id"]="$(strip_quotes "$(get_field "$fm" origin)")"
done

is_closed_status() {
  case "$1" in
    merged|closed) return 0 ;;
    *) return 1 ;;
  esac
}

# --- build generated sections into a temp file --------------------------

GEN="$(mktemp)"
trap 'rm -f "$GEN"' EXIT

{
cat <<'PREAMBLE'
# Implementation progress — the build backbone

**Owned by the Steward** (`agent/playbooks/federation/steward.md §2a`). This
file tracks execution **against the implementation DAG**
(`05-implementation-dag.md`), the build's analog of `spec/SPEC-PROGRESS.md`.
It **survives compaction**: on a cold start or after a compact, read this
first, then continue from the frontier (below). Update it **every synthesis
pass and on every WP state change**. The plan lives in `05`; this file
tracks *progress against it*. Run until complete, blocked, or instructed
(§2b).

**This file holds CURRENT STATE ONLY, and it is GENERATED** — edit
`docs/program/issues/*.md` and re-run `scripts/gen-progress.sh`; hand edits
here are overwritten. The full chronicle — every prior "live state"
snapshot, the detailed evidence trail for every merged WP, and the
day-by-day session logs back to project start — lives in
[`diary/`](diary/INDEX.md). If you need *why* a past call was made, or the
mechanism detail behind a closed WP, start there;
[`diary/CURRENT-BRIEFING.md`](diary/CURRENT-BRIEFING.md) carries the live
operator briefing and the Steward's resume state.

**Status legend:** `draft` (not framed / deps unmet) · `ready` (deps met,
unassigned) · `active` (a team is building) · `in-review` (PR open / QA / CI)
· `merged` (landed + retro in) · `closed` (resolved without landing, e.g. a
superseded or withdrawn item). Gates: see `05-implementation-dag.md`.

**★ GENERATED FILE — do not hand-edit.** This file is regenerated from the
frontmatter of every `docs/program/issues/*.md` work-item file by
`scripts/gen-progress.sh`. To change tracked status, edit the relevant
`docs/program/issues/<ID>.md` file and re-run the generator. CI checks that
the committed file matches the generator's output.

PREAMBLE

printf -- '## Last generated\n\n'
printf '%s — from %d issue file(s) in `docs/program/issues/`.\n\n' \
  "$(date -u '+%Y-%m-%d %H:%M:%SZ')" "${#ids[@]}"

printf -- '## Work-item status\n\n'
printf '%s\n' '| ID | Title | Status | Owner | Size | Gate | GitHub |'
printf '%s\n' '|---|---|---|---|---|---|---|'
for id in "${ids[@]}"; do
  gh="${f_github[$id]}"
  [ "$gh" = "null" ] || [ -z "$gh" ] && gh="—"
  printf '| `%s` | %s | %s | %s | %s | %s | %s |\n' \
    "$id" "${f_title[$id]}" "${f_status[$id]}" "${f_owner[$id]}" \
    "${f_size[$id]}" "${f_gate[$id]}" "$gh"
done
printf '\n'

printf -- '## Releasable frontier\n\n'
printf 'Items whose status is `ready` and whose every `depends_on` entry is\n'
printf 'itself `merged` or `closed` (i.e. nothing left blocking a kickoff):\n\n'
frontier_found=0
for id in "${ids[@]}"; do
  [ "${f_status[$id]}" = "ready" ] || continue
  all_deps_done=1
  while IFS= read -r dep; do
    [ -z "$dep" ] && continue
    dep_status="${f_status[$dep]:-}"
    if [ -z "$dep_status" ] || ! is_closed_status "$dep_status"; then
      all_deps_done=0
    fi
  done < <(list_items "${f_deps[$id]}")
  if [ "$all_deps_done" -eq 1 ]; then
    printf -- '- `%s` — %s\n' "$id" "${f_title[$id]}"
    frontier_found=1
  fi
done
[ "$frontier_found" -eq 0 ] && printf -- '- (none currently)\n'
printf '\n'

printf -- '## Blockers\n\n'
printf 'Items not yet `merged`/`closed` whose `depends_on` names an id that\n'
printf 'is itself not yet `merged`/`closed`:\n\n'
blockers_found=0
for id in "${ids[@]}"; do
  is_closed_status "${f_status[$id]}" && continue
  while IFS= read -r dep; do
    [ -z "$dep" ] && continue
    dep_status="${f_status[$dep]:-unknown}"
    if ! is_closed_status "$dep_status"; then
      printf -- '- `%s` blocked by `%s` (status: %s)\n' "$id" "$dep" "$dep_status"
      blockers_found=1
    fi
  done < <(list_items "${f_deps[$id]}")
done
[ "$blockers_found" -eq 0 ] && printf -- '- (none currently)\n'
printf '\n'

printf -- '## Gate progress\n\n'
printf 'Work items grouped by the gate (`05-implementation-dag.md`) they\n'
printf 'feed; `none`/`TBD` gates are omitted here (see the status table above\n'
printf 'for every item, gated or not):\n\n'
gates_found=0
# Collect distinct, non-trivial gate values (sorted).
mapfile -t distinct_gates < <(
  for id in "${ids[@]}"; do printf '%s\n' "${f_gate[$id]}"; done \
    | grep -vix -E 'none|tbd|' | sort -u
)
for g in "${distinct_gates[@]:-}"; do
  [ -z "$g" ] && continue
  gates_found=1
  printf -- '- **%s**:' "$g"
  for id in "${ids[@]}"; do
    [ "${f_gate[$id]}" = "$g" ] || continue
    printf ' `%s` (%s)' "$id" "${f_status[$id]}"
  done
  printf '\n'
done
[ "$gates_found" -eq 0 ] && printf -- '- No item in the current queue cites a named gate.\n'
printf '\n'

cat <<'FOOTER'
## Archive & diary

- The complete build chronicle — every prior live-state snapshot, the full
  evidence trail behind every merged WP back to project start — and the
  day-to-day session narrative both live in [`diary/`](diary/INDEX.md), one
  file per day under `diary/YYYY/Mon/DD.md`. See
  [`diary/CURRENT-BRIEFING.md`](diary/CURRENT-BRIEFING.md) for the live
  operator briefing and Steward resume state.
- Per-item briefs, where they exist, live under
  [`wp/`](wp/) and are linked from the corresponding
  `docs/program/issues/<ID>.md` file.
FOOTER
} > "$GEN"

if [ "$MODE" = "check" ]; then
  # The "Last generated" line embeds a live timestamp by design (it is
  # informational, not load-bearing), so idempotency is judged on content
  # with that one line normalized out of both sides.
  TS_PATTERN='^[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}:[0-9]{2}Z — from [0-9]+ issue file'
  NORM_GEN="$(mktemp)"; NORM_OUT="$(mktemp)"
  trap 'rm -f "$GEN" "$NORM_GEN" "$NORM_OUT"' EXIT
  grep -vE "$TS_PATTERN" "$GEN" > "$NORM_GEN"
  grep -vE "$TS_PATTERN" "$OUT_FILE" > "$NORM_OUT" 2>/dev/null || true
  if diff -q "$NORM_GEN" "$NORM_OUT" >/dev/null 2>&1; then
    echo "gen-progress --check: OK ($OUT_FILE is up to date)"
    exit 0
  else
    echo "gen-progress --check: STALE — $OUT_FILE does not match generator output." >&2
    echo "Run scripts/gen-progress.sh and commit the result. Diff (timestamp line ignored):" >&2
    diff -u "$NORM_OUT" "$NORM_GEN" >&2 || true
    exit 1
  fi
else
  cp "$GEN" "$OUT_FILE"
  echo "gen-progress: wrote $OUT_FILE ($(wc -c < "$OUT_FILE") bytes, ${#ids[@]} issues)"
fi
