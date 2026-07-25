#!/usr/bin/env bash
# Report whether an agent pane is BUSY, IDLE, or SUSPECT.
#
# Exists because a hand-rolled busy check at the call site has produced NINE
# recorded misclassifications. Do not inline a busy regex; call this.
#   BUSY  — an active spinner, an interrupt hint, or an explicit wait state
#   IDLE  — no busy signal (a bare "Verb for 1m 39s" is a FINISHED turn)
#   SUSPECT — pane is in copy mode, so a capture may read history
#
# Usage: scripts/pane-busy.sh moot-runtime-implementer [...]
set -uo pipefail

# An ACTIVE spinner always carries a parenthesised elapsed counter, an interrupt
# hint, or an explicit wait state. The verb is RANDOMIZED and never a signal.
# ⛔ No bare-verb arms: "Cogitat" matches the finished "Cogitated for".
BUSY_RE='\([0-9]+m [0-9]+s|\([0-9]+s|esc to interrupt|Compacting conversation|[0-9]+ shells? still running|Running [0-9]+ shell commands|· [0-9]+ shells? ·|Waiting for [0-9]+ background'

for pane in "$@"; do
  if ! tmux has-session -t "$pane" 2>/dev/null && ! tmux list-panes -t "$pane" >/dev/null 2>&1; then
    printf '%-24s %s\n' "$pane" "NOPANE"; continue
  fi
  mode=$(tmux display-message -p -t "$pane" '#{pane_in_mode}' 2>/dev/null)
  # -S - spans all history through the true end, so it is robust to copy mode.
  # ⛔ Never -S -<n>: that is relative to a scrolled top and silently returns
  # plausible, well-formed, STALE text.
  body=$(tmux capture-pane -p -S - -t "$pane" 2>/dev/null | grep -v '^[[:space:]]*$' | tail -25)
  state=IDLE
  hit=$(printf '%s' "$body" | grep -oE "$BUSY_RE" | head -1)
  [ -n "$hit" ] && state=BUSY
  [ "$mode" = "1" ] && [ "$state" = IDLE ] && state=SUSPECT
  ctx=$(printf '%s' "$body" | grep -oE 'ctx [0-9]+%' | tail -1)
  printf '%-24s %-8s %-22s %s\n' "$pane" "$state" "${hit:-}" "${ctx:-}"
done
