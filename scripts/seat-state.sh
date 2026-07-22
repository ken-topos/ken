#!/usr/bin/env bash
# seat-state.sh — report whether each named moot seat is WORKING, QUEUED,
# STRANDED, or IDLE. Use this instead of hand-rolling a capture-pane grep.
#
# ⛔ WHY THIS EXISTS. Three hand-rolled sweeps in one day misread live seats as
# idle, twice leading to a rouse of a working agent (forbidden by the playbook).
# Both defects were in the INSTRUMENT, not the fleet:
#
#   1. Grepping a fixed spinner WORD. The verb is randomized — "Infusing…",
#      "Slithering…", "Clauding…", "Sautéed", "Cogitated", "Brewed". Keying on
#      any one of them reads every busy seat as dead.
#   2. Capturing WIDE and then `tail`-ing NARROW. The spinner renders ABOVE the
#      prompt, so `capture-pane -S -200 | tail -7` throws away the exact lines
#      the wide capture was taken to see. A wide capture followed by a narrow
#      tail is a narrow capture.
#
# The load-bearing signal is the DURATION/TOKEN SIGNATURE — "(9m 34s · ↓ 23.6k
# tokens)" — which every spinner carries regardless of its verb, plus the
# literal "esc to interrupt". Grep the WHOLE capture for it. Never tail.
#
# ⚠ `-S` MUST BE NEGATIVE. A positive value returns ~1 line and reads the
# entire fleet as dead.
#
# Usage: scripts/seat-state.sh [seat ...]        (default: all moot-* sessions)

set -uo pipefail

SCROLLBACK="${SCROLLBACK:--400}"

# The three signatures, in precedence order. A spinner elapsed-time/token
# readout is position-independent and verb-independent, which is the point.
SPINNER='\([0-9]+m? ?[0-9]*s · [^)]*\)'
INTERRUPT='esc to interrupt'
QUEUED='Queued follow-up|up to edit queued|Messages to be submitted'
STRANDED='\[Pasted Content'

seats=("$@")
if [ ${#seats[@]} -eq 0 ]; then
  mapfile -t seats < <(tmux list-sessions -F '#S' 2>/dev/null | sed -n 's/^moot-//p')
fi

# Self-test the detector before trusting an empty sweep. An all-idle result is
# a claim about the instrument as much as about the fleet.
if ! printf 'Infusing… (9m 34s · ↓ 23.6k tokens)\n' | grep -qE "$SPINNER"; then
  echo "DETECTOR SELF-TEST FAILED — the spinner pattern does not match a known-good line." >&2
  exit 2
fi

for s in "${seats[@]}"; do
  cap=$(tmux capture-pane -p -S "$SCROLLBACK" -t "moot-$s" 2>/dev/null)
  if [ -z "$cap" ]; then
    # A pane that captures EMPTY at full scrollback is usually alive and blocked
    # on a consent modal rendered at the buffer's start, not dead.
    printf '%-24s %s\n' "$s" "EMPTY-CAPTURE (check for a consent modal; capture from -S -)"
    continue
  fi

  state=IDLE
  if grep -qE "$SPINNER" <<<"$cap" || grep -qF "$INTERRUPT" <<<"$cap"; then
    state=WORKING
  elif grep -qE "$STRANDED" <<<"$cap"; then
    state='STRANDED (composer never submitted — send a bare Enter)'
  fi

  if grep -qE "$QUEUED" <<<"$cap"; then
    state="$state +QUEUED (do NOT resend)"
  fi

  printf '%-24s %s\n' "$s" "$state"
done
