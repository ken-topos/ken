#!/usr/bin/env bash
# ken context-awareness — PreToolUse nudge (in-house; no external dependency).
#
# Nudges ONLY the self-compacting singletons (steward/architect/integrator/
# librarian) — the roles that compact via request_context_reset. Team roles
# compact via `moot` at WP boundaries, so they are gated out (silent no-op);
# they still get the statusline readout, just no nudge.
#
# Reads the session context % stashed by ctx-statusline.sh; on crossing UP into
# the warn (>=70%) or high (>=85%) band, injects a one-shot checkpoint-and-seam
# reminder via hookSpecificOutput.additionalContext. Debounced by band; resets
# when context drops (post-compaction) so the next climb re-nudges.
#
# FAIL-SAFE: every path exits 0; stdout is emitted ONLY when actually nudging
# (empty stdout = the tool proceeds untouched). Malformed input → silent no-op.
json="$(cat)"

# Role: prefer CONVO_ROLE (set by moot's launcher); fall back to the worktree
# name in .cwd, so detection works even if the env is not inherited.
role="${CONVO_ROLE:-}"
if [ -z "$role" ]; then
  role="$(printf '%s' "$json" | jq -r '.cwd // empty' 2>/dev/null \
          | grep -oE '\.worktrees/[^/]+' | head -1 | sed 's#.*/##')"
fi
case "$role" in
  steward|architect|integrator|librarian) ;;
  *) exit 0 ;;    # not a self-compacting singleton → no nudge
esac

sid="$(printf '%s' "$json" | jq -r '.session_id // empty' 2>/dev/null)"
[ -z "$sid" ] && sid="nosession"
pctfile="/tmp/cc-ctx-${sid}.pct"
bandfile="/tmp/cc-ctx-${sid}.band"

[ -r "$pctfile" ] || exit 0
pct="$(cat "$pctfile" 2>/dev/null)"
case "$pct" in ''|*[!0-9]*) exit 0 ;; esac   # integer-only guard

WARN=70; HIGH=85
if   [ "$pct" -ge "$HIGH" ]; then cur=high
elif [ "$pct" -ge "$WARN" ]; then cur=warn
else                              cur=ok
fi
last="ok"; [ -r "$bandfile" ] && last="$(cat "$bandfile" 2>/dev/null)"
printf '%s' "$cur" > "$bandfile" 2>/dev/null

msg=""
if   [ "$cur" = "warn" ] && [ "$last" = "ok" ]; then
  msg="Context-awareness: usage at ${pct}%. Past the checkpoint-and-seam threshold. At the NEXT clean work-unit seam (nothing mid-flight — no WP mid-frame, no half-posted handoff, a harvest/review just closed), bring your durable checkpoint current, then self-compact via request_context_reset. Autocompact backstop is ~84%; you have room to pick a good seam."
elif [ "$cur" = "high" ] && [ "$last" != "high" ]; then
  msg="Context-awareness: usage at ${pct}% — autocompact (~84%) is close. Checkpoint your durable state now and self-compact (request_context_reset) at the first safe point; do not start a new large sub-task."
fi
[ -z "$msg" ] && exit 0

jq -cn --arg m "$msg" \
  '{hookSpecificOutput:{hookEventName:"PreToolUse",additionalContext:$m}}' 2>/dev/null || exit 0
exit 0
