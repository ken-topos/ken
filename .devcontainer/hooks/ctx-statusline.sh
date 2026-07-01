#!/usr/bin/env bash
# ken context-awareness — statusLine bridge (in-house; no external dependency).
#
# Claude Code passes the statusLine command a JSON blob on stdin that includes
# `.context_window.used_percentage`. We stash that % in a session-scoped temp
# file so the PreToolUse nudge hook (ctx-nudge.sh) can read it — PreToolUse input
# does NOT carry context usage itself, so the statusline is the only surface that
# does. We also print a minimal human status line (`ctx N% · Model`).
#
# Best-effort by design: it never aborts the turn. A broken statusline is merely
# cosmetic (no tool is gated on it), so every failure path falls back gracefully.
json="$(cat)"
pct="$(printf '%s' "$json"   | jq -r '.context_window.used_percentage // empty' 2>/dev/null)"
sid="$(printf '%s' "$json"   | jq -r '.session_id // empty'                      2>/dev/null)"
model="$(printf '%s' "$json" | jq -r '.model.display_name // empty'              2>/dev/null)"
[ -z "$sid" ]   && sid="nosession"
[ -z "$model" ] && model="?"
if [ -n "$pct" ]; then
  printf '%s' "$pct" > "/tmp/cc-ctx-${sid}.pct" 2>/dev/null
  printf 'ctx %s%% · %s' "$pct" "$model"
else
  printf 'ctx --%% · %s' "$model"
fi
