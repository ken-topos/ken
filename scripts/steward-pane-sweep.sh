#!/usr/bin/env bash
# Steward watchdog STEP 1 fact-gatherer. Prints one row per seat and DECIDES NOTHING.
# Reporting red is not refusing: every judgment call belongs to the Steward reading it.
#
# Columns: seat | LIVE/DONE/(no-footer)/[SAFETY-MODAL] | ok/STRAND
#
# Busy is matched by VERB FAMILY, never by a verb. Both families are open-ended --
# live verbs seen include Composing, Actualizing, Baked, Grooving, Cooking, Wibbling;
# finished ones include "Worked for", "Cogitated for", "Baked for". Keying on any single
# verb has produced a false idle on a live seat AND a wait loop that ran against a seat
# that had already stopped. LIVE = ellipsis / "esc to interrupt" / a progress bar.
# DONE = past tense + "for Nm Ns".
#
# The STRAND column only sees pasted/channel text. Plain instruction text in a composer
# is INVISIBLE to it and reports "ok" -- the only sound test is to type two throwaway
# characters and see whether they replace the line (empty) or append (real input).
#
# (no-footer) is a rendering state, not idle. Codex seats render no timing footer in
# either direction and always land there; that is expected and is NOT evidence of idleness.

set -uo pipefail

SEATS=${*:-"runtime-leader runtime-implementer runtime-qa architect foundation-leader foundation-implementer"}

for p in $SEATS; do
  b=$(tmux capture-pane -t "moot-$p" -p -S -200 2>/dev/null)
  if [ -z "$b" ]; then
    printf "%-24s %-14s | %s\n" "$p" "(no-session)" "-"
    continue
  fi
  f=$(printf '%s' "$b" | tail -14)

  if printf '%s' "$f" | grep -qE '…|esc to interrupt|▰'; then
    s=LIVE
  elif printf '%s' "$f" | grep -qE ' for [0-9]+m ?[0-9]*s'; then
    s=DONE
  elif printf '%s' "$b" | grep -q 'Retry with a faster model'; then
    s='[SAFETY-MODAL]'
  else
    s='(no-footer)'
  fi

  c=$(printf '%s' "$f" | grep -E '^[›❯]' | tail -1)
  if printf '%s' "$c" | grep -qE 'Pasted Content|<channel>'; then
    strand=STRAND
  else
    strand=ok
  fi

  printf "%-24s %-14s | %s\n" "$p" "$s" "$strand"
done
