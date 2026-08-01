#!/usr/bin/env bash
# Sweep every `moot-<role>` pane for a STRANDED DELIVERY and submit it.
#
# ⚠ TWO stranding shapes, not one. This script originally keyed only on the
# `[Pasted Content …]` marker. On 2026-07-26 `moot compact <role>` was measured
# leaving `› /compact` sitting UNSUBMITTED on the Architect's composer while
# printing `Sent /compact to moot-architect`. That shape has no paste marker, so
# the sweep walked straight past it — and it is the QUIETER failure: the seat
# keeps billing stale context through every later turn and nothing reports it,
# whereas a stranded mention at least blocks a ring visibly.
# ⇒ **A backstop keyed to ONE stranding shape is blind to another that causes the
# same failure.** Classification now lives in `classify-pane-composer.py`, which
# is controlled by `test-classify-pane-composer.sh` — add a shape there, not by
# widening a regex here.
#
# ⛔⛔ AND THE MARKER IS OFTEN ABSENT ENTIRELY. Measured on `moot-doc-author`
# 2026-08-01: a stranded mention rendered as `› <channel source="convo" …
# event_type="mention"…` with the terminal wrapping `[Pasted` / `Content` across
# lines, so the marker was not on the composer line at all. The classifier's
# `startswith("[Pasted Content")` returned `other` and this sweep walked past a
# seat holding a QA rejection with its ring blocked. ⚠ It had been failing
# INTERMITTENTLY as a function of terminal width, so its silence carried no
# information. Fixed in `classify-pane-composer.py` (the DELIVERY pattern), pinned
# by `strand-mention`/`strand-interval`/`near-miss` in the control file.
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
# ⚠ HONEST RESIDUAL — A BUSY SEAT IS REPORTED, NEVER REPAIRED. A stranded
# delivery on a seat that is mid-turn is real and this script will not fix it:
# pressing Enter into a live turn is how a `/compact` destroys in-flight work.
# So `busy` is a "come back later", and it means ⛔ **one sweep is not a clean
# bill of health for the fleet** — a run that reports four BUSY seats has
# answered nothing about those four. Re-run when they go idle.
#
# ⚠ AND A FALSE BUSY WOULD MAKE THIS SCRIPT BLIND while still printing
# reassuring output, which is why the busy detector is positional (pane tail
# only) and why `test-classify-pane-composer.sh` pins BOTH directions —
# `busy-*` rows and `done-*`/`echo-duration` rows. ⛔ Never "fix" a false wedged
# report by widening the busy guard; that trades a visible wrong answer for a
# silent one.
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

CLASSIFY="$(dirname "$0")/classify-pane-composer.py"

wedged=()
verdicts=()

while read -r session; do
    [[ "$session" == "$SELF" ]] && continue

    # ⛔ `-e` is REQUIRED. Without the escape sequences an idle pane's own
    # suggestion text ("Explain this codebase") is indistinguishable from a real
    # delivery, and submitting it sends the agent an instruction nobody wrote.
    # ⛔ `-S -50` is NOT optional either, and for a reason that took two live
    # misfires to learn: the status/spinner line renders ABOVE the composer, so a
    # capture whose topmost line is the composer structurally cannot hold the
    # evidence that the seat is busy. It does not return "unknown" — it returns a
    # confident IDLE. The classifier anchors on the LAST prompt-glyph line, so
    # extra scrollback is harmless; too little is not.
    pane="$(tmux capture-pane -e -t "$session" -p -S -50 2>/dev/null || true)"

    verdict="$(printf '%s' "$pane" | python3 "$CLASSIFY" 2>/dev/null || echo unreadable)"

    case "$verdict" in
        paste|slash:*)
            wedged+=("$session")
            verdicts+=("$verdict")
            [[ "$DRY_RUN" == 1 ]] && continue
            tmux send-keys -t "$session" Enter
            ;;
        busy)
            # ⛔ NEVER submit into a live turn — an Enter on a stranded `/compact`
            # here destroys in-flight work. Reported, not repaired, because a
            # stranded delivery on a busy seat is a real thing that needs the
            # Steward's eyes rather than the script's reflex.
            echo "sweep: $session — BUSY (mid-turn); not touching it"
            ;;
        unreadable)
            # ⛔ Was silently `continue`d as an empty pane, which read exactly like
            # a healthy seat in the final report. An unreadable pane is an
            # unanswered question, so say so.
            echo "sweep: $session — pane UNREADABLE or classifier failed; NOT a clear seat"
            ;;
        *)
            # queued (healthy, will be consumed) · ghost (the UI's own text) ·
            # other (⛔ unattributable — never submit) · clear.
            ;;
    esac
done < <(tmux list-sessions -F '#{session_name}' 2>/dev/null | grep '^moot-')

if [[ ${#wedged[@]} -eq 0 ]]; then
    echo "sweep: clear — no stranded deliveries"
    exit 0
fi

if [[ "$DRY_RUN" == 1 ]]; then
    for i in "${!wedged[@]}"; do
        printf 'sweep: WEDGED (dry-run, not repaired): %s [%s]\n' "${wedged[$i]}" "${verdicts[$i]}"
    done
    exit 0
fi

# Verify the repair landed — a delivery still sitting there did not take the
# Enter and needs a human. Reporting "repaired" without re-reading the pane would
# be exactly the fabricated-confidence bug this fleet keeps hitting.
sleep 3
for i in "${!wedged[@]}"; do
    session="${wedged[$i]}"
    pane="$(tmux capture-pane -e -t "$session" -p -S -50 2>/dev/null || true)"
    after="$(printf '%s' "$pane" | python3 "$CLASSIFY" 2>/dev/null || echo unreadable)"
    case "$after" in
        paste|slash:*)
            echo "sweep: $session — STILL WEDGED after Enter (${after}); needs manual attention"
            ;;
        *)
            echo "sweep: $session — repaired (${verdicts[$i]} submitted)"
            ;;
    esac
done
