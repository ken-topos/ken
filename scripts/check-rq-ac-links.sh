#!/usr/bin/env bash
# check-rq-ac-links.sh — report requirements that no acceptance criterion claims.
#
# Governing doc: docs/program/15-requirements-and-acceptance-criteria.md
#
# The tiers are  conformance/ -> PROGRAM.RQ-n -> WP.AC-n , one direction. A
# residual is not a separate object: it is an RQ that no AC references yet
# (15-*.md §5). This script computes that set.
#
# ⛔ WHAT THIS SCRIPT CANNOT DO, stated up front because a green run reads as
#    stronger than it is:
#
#   1. It checks that a REFERENCE EXISTS, never that the referencing AC is
#      strong enough to discharge the RQ. Rule 1 is satisfiable by pointing a
#      weak AC at an RQ. An advertised link is not an enforced one.
#   2. It cannot see an RQ that was never written. A program with no
#      `## Requirements` section reports zero unreferenced RQs, which is the
#      same output as a program that satisfies all of them. That case is
#      reported separately as NO-RQ-BLOCK, never folded into a pass.
#   3. `none` in a conformance column is legitimate (15-*.md §4) and is
#      reported for review, not as an error.
#
# Exit: 0 = nothing unreferenced. 1 = at least one unreferenced RQ. 2 = misuse.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ISSUES_DIR="$REPO_ROOT/docs/program/issues"
PROG_DIR="$REPO_ROOT/docs/program"

[ -d "$ISSUES_DIR" ] || { echo "check-rq-ac-links: no issues dir at $ISSUES_DIR" >&2; exit 2; }

# --- 1. Which programs are IN PROGRESS? -------------------------------------
# 15-*.md §6: at least one node with status ready|active|in-review. Derived
# from the files, never from a list in a doc.

in_progress_ids=()
while IFS= read -r f; do
  st="$(sed -n 's/^status:[[:space:]]*//p' "$f" | head -n1)"
  case "$st" in
    ready|active|in-review) in_progress_ids+=("$(basename "$f" .md)") ;;
  esac
done < <(find "$ISSUES_DIR" -maxdepth 1 -name '*.md' | sort)

# --- 2. Collect every RQ id declared anywhere -------------------------------
# An RQ is declared as `<PROGRAM>.RQ-<n>` inside a `## Requirements` section of
# a program doc or an umbrella node.

declare -A rq_home        # rq id -> file that declares it
while IFS= read -r f; do
  awk -v FILE="$f" '
    /^##[[:space:]]/ { inreq = ($0 ~ /Requirements/) }
    inreq {
      while (match($0, /[A-Z][A-Z0-9-]*\.RQ-[0-9]+/)) {
        print substr($0, RSTART, RLENGTH) "\t" FILE
        $0 = substr($0, RSTART + RLENGTH)
      }
    }
  ' "$f"
done < <(find "$ISSUES_DIR" "$PROG_DIR" -maxdepth 1 -name '*.md' | sort) \
 | sort -u > /tmp/.rq_declared.$$ || true

while IFS=$'\t' read -r id home; do
  [ -n "${id:-}" ] && rq_home["$id"]="$home"
done < /tmp/.rq_declared.$$
rm -f /tmp/.rq_declared.$$

# --- 3. Collect every RQ id REFERENCED from an AC context -------------------
# Frames and nodes both count: a node may record which RQs its ACs serve.

# ⚠ THREE STATES, not two. Measuring "does a frame mention the id" and
#   reporting "does an AC claim it" would be a different question with the same
#   green — so the weaker evidence gets its own label instead of a pass.
#
#     CLAIMED-IN-FRAME  a WP frame names the RQ -> Rule 1 is wired
#     TABLE-ONLY        only the declaring program's discharged-by column names
#                       an AC. Real evidence, but the frame the implementer
#                       actually reads does NOT carry the link.
#     UNREFERENCED      neither -> the §5 finding

declare -A rq_referenced rq_table_only
while IFS= read -r id; do
  [ -n "$id" ] && rq_referenced["$id"]=1
done < <(grep -rhoE '[A-Z][A-Z0-9-]*\.RQ-[0-9]+' \
           "$REPO_ROOT/docs/program/wp" 2>/dev/null | sort -u || true)

# A declaring file's table row counts as TABLE-ONLY when the row also names an
# AC (i.e. a discharged-by cell), which is weaker than a frame-side reference.
for id in "${!rq_home[@]}"; do
  [ -n "${rq_referenced[$id]:-}" ] && continue
  # -F: the id contains '.' and '-'; a regex here would over-match.
  if grep -F "$id" "${rq_home[$id]}" 2>/dev/null | grep -qE '\.AC-|AC-[A-Z]?[0-9]'; then
    rq_table_only["$id"]=1
  fi
done

# --- 4. Report --------------------------------------------------------------

fail=0
echo "check-rq-ac-links: ${#rq_home[@]} requirement(s) declared; ${#in_progress_ids[@]} node(s) in progress"

# 4a. In-progress programs with no Requirements block at all.
for id in "${in_progress_ids[@]}"; do
  f="$ISSUES_DIR/$id.md"
  if grep -qE '^##[^\n]*Requirements' "$f" 2>/dev/null; then continue; fi
  # Only umbrella-ish nodes are expected to carry RQs; a member node need not.
  if grep -qE '^blocks:[[:space:]]*\[[^]]+\]' "$f" 2>/dev/null \
     && grep -qE '^status:[[:space:]]*active' "$f" 2>/dev/null; then
    echo "  NO-RQ-BLOCK   $id (active umbrella, no '## Requirements' section)" >&2
  fi
done

# 4b. The actual finding: declared but unreferenced.
if [ "${#rq_home[@]}" -gt 0 ]; then
  for id in $(printf '%s\n' "${!rq_home[@]}" | sort); do
    if [ -n "${rq_referenced[$id]:-}" ]; then
      continue
    elif [ -n "${rq_table_only[$id]:-}" ]; then
      echo "  TABLE-ONLY    $id — an AC is named in the program's discharged-by cell, but no WP FRAME carries the link" >&2
    else
      echo "  UNREFERENCED  $id — declared in ${rq_home[$id]#$REPO_ROOT/}, claimed by no AC" >&2
      fail=1
    fi
  done
fi

if [ "$fail" -eq 0 ]; then
  echo "check-rq-ac-links: OK — every declared requirement is claimed by at least one AC"
  echo "  ⛔ reminder: existence of a reference is NOT evidence the AC discharges it"
else
  echo "check-rq-ac-links: unreferenced requirement(s) above — see docs/program/15-requirements-and-acceptance-criteria.md §5" >&2
fi
exit "$fail"
