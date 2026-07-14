#!/usr/bin/env bash
# Sweep every `moot-<role>` pane for a STRANDED PASTE and submit it.
#
# The failure this repairs: a convo mention is delivered into a seat's composer
# as `› [Pasted Content NNNN chars] …` and is NEVER SUBMITTED. `post_response`
# returning an event_id proves the EVENT exists — it does not prove any agent
# READ it. The seat then sits looking perfectly healthy while its ring blocks.
# On 2026-07-14 this fired SIX times (architect x3, kernel-implementer, CV x2).
# The documented repair has always been "send a bare Enter to that pane", which
# only works if a human or the Steward happens to look. This makes it a check.
#
# A paste that is already QUEUED ("Messages to be submitted after next tool
# call") is HEALTHY — the seat is busy and will consume it. Do not resend: that
# double-delivers. Only a paste still sitting on the `›` composer line is wedged.
#
# Usage:
#   scripts/sweep-wedged-panes.sh            # repair, report what it touched
#   scripts/sweep-wedged-panes.sh --dry-run  # report only
#
# Exit status is 0 whether or not anything was wedged; the report is the output.
set -euo pipefail

DRY_RUN=0
[[ "${1:-}" == "--dry-run" ]] && DRY_RUN=1

# Never Enter our own pane — that would submit the Steward's own composer.
SELF="moot-steward"

wedged=()

while read -r session; do
    [[ "$session" == "$SELF" ]] && continue

    pane="$(tmux capture-pane -t "$session" -p 2>/dev/null || true)"
    [[ -z "$pane" ]] && continue

    # Already queued behind an active turn: the seat WILL consume it. Leave it.
    if grep -qF 'Messages to be submitted after next tool call' <<<"$pane"; then
        continue
    fi

    # A paste still on the composer prompt line has never been submitted.
    if grep -qE '^[[:space:]]*›[[:space:]]*\[Pasted Content' <<<"$pane"; then
        wedged+=("$session")
        [[ "$DRY_RUN" == 1 ]] && continue
        tmux send-keys -t "$session" Enter
    fi
done < <(tmux list-sessions -F '#{session_name}' 2>/dev/null | grep '^moot-')

if [[ ${#wedged[@]} -eq 0 ]]; then
    echo "sweep: clear — no stranded pastes"
    exit 0
fi

if [[ "$DRY_RUN" == 1 ]]; then
    printf 'sweep: WEDGED (dry-run, not repaired): %s\n' "${wedged[*]}"
    exit 0
fi

# Verify the repair landed — an unsubmitted paste that is still there did not
# take the Enter, and needs a human. Reporting "repaired" without re-reading the
# pane would be exactly the fabricated-confidence bug this fleet keeps hitting.
sleep 3
for session in "${wedged[@]}"; do
    pane="$(tmux capture-pane -t "$session" -p 2>/dev/null || true)"
    if grep -qE '^[[:space:]]*›[[:space:]]*\[Pasted Content' <<<"$pane"; then
        echo "sweep: $session — STILL WEDGED after Enter; needs manual attention"
    else
        echo "sweep: $session — repaired (paste submitted)"
    fi
done
