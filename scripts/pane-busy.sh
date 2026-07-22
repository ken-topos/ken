#!/usr/bin/env bash
# pane-busy.sh — classify moot-* panes as BUSY or idle, with a built-in self-test.
#
# Usage:
#   scripts/pane-busy.sh                 # sweep every moot-* session
#   scripts/pane-busy.sh runtime-qa ...  # specific seats
#   scripts/pane-busy.sh --quiet         # print only BUSY seats
#
# ⛔ THE SELF-TEST IS THE POINT — read this before changing the patterns.
#
# This script runs inside a pane. That pane is BUSY, by definition, whenever
# this script is running. So the detector has a permanently available oracle:
#
#     If it classifies its OWN pane as idle, it is falsified — on the spot,
#     every run, at zero cost, against certain ground truth.
#
# That converts "have I enumerated every busy state?" — unfalsifiable, and the
# question that cost five successive false-IDLE misses — into a standing
# assertion that fires the moment the answer is no. It does not require the
# enumeration to be complete. It requires only that the detector can see the one
# pane whose answer is already known. (adversary, 2026-07-22.)
#
# ── Why the patterns are what they are (all measured, not assumed) ────────────
#
# 1. NEVER pipe the capture through `tail -N`. The spinner renders ~7 lines
#    above the bottom, above the `❯` box and the ctx/permissions footer. A
#    `tail -4`/`tail -5` window cannot see it NO MATTER how many arms the
#    pattern has — the arm is present and outside the window. (`capture-pane
#    -S -N` is safe: it captures from N lines of scrollback THROUGH the visible
#    bottom, so it is not a tail.)
#
# 2. The `❯` prompt box renders while BUSY. It is a constant, not an idle
#    signal. Any rule that reads "reached the prompt box ⇒ no busy marker" is
#    reading a constant.
#
# 3. `esc to interrupt` covers only ~1 pane in 3 — a Tip line takes that slot
#    otherwise. It is not a reliable discriminator on its own.
#
# 4. The spinner GLYPH varies (✻ ✽ ✳ ✶ *) and renders differently per pane, so
#    a glyph-class anchor misses panes. Anchor on the SHAPE instead:
#    `<glyph> <Participle>… (<n>m <n>s`.
#
# 5. ★ PRESENT PARTICIPLE + PARENS = running. PAST TENSE = finished.
#       "✻ Catapulting… (12m 40s · ↓ 45.9k)"  -> BUSY   (active turn)
#       "✻ Worked for 13m 42s"                -> idle   (completed summary)
#    A bare elapsed-time pattern conflates these and reports finished seats as
#    busy. The `… \(` is what separates them.
#
# 6. Background work has NO spinner and NO timer of its own — a seat blocked on
#    two long `cargo` runs shows only "Cogitated for 36s · 2 shells still
#    running". That is BUSY and it is the most dangerous seat to interrupt.
#
# 7. Do not echo these patterns into the pane you are scanning: a naive
#    alternation matches its own source text and reports a false BUSY. The
#    shape-anchor below is immune (a raw pattern string has no `… (`), which is
#    a reason to prefer it beyond window-independence.
#
# Every one of the five historical misses was a false IDLE; none was a false
# BUSY. That asymmetry is structural: the detector is a disjunction of POSITIVE
# busy signals, so any state left un-enumerated defaults to "stalled" — and a
# false idle is the one that makes you interrupt real work.

set -uo pipefail

SPINNER='^.{1,3} [A-Z][a-z]+… \([0-9]+(m [0-9]+)?s'
BACKGROUND='[0-9]+ shells? still running|Waiting for [0-9]+ background'

pane_is_busy() { # $1 = role; echoes BUSY|idle
  local out
  out=$(tmux capture-pane -t "moot-$1" -p 2>/dev/null) || { echo "idle"; return; }
  if grep -qE "$SPINNER" <<<"$out" || grep -qE "$BACKGROUND" <<<"$out"; then
    echo "BUSY"
  else
    echo "idle"
  fi
}

QUIET=0
roles=()
for a in "$@"; do
  case "$a" in
    --quiet) QUIET=1 ;;
    *) roles+=("$a") ;;
  esac
done

if [ ${#roles[@]} -eq 0 ]; then
  mapfile -t roles < <(tmux list-sessions -F '#S' 2>/dev/null | sed -n 's/^moot-//p' | sort)
fi

# ── SELF-TEST — must run before any verdict is reported ──────────────────────
self="${MOOT_ROLE:-steward}"
if [ "$(pane_is_busy "$self")" != "BUSY" ]; then
  echo "⛔ SELF-TEST FAILED: detector classified its own pane (moot-$self) as idle." >&2
  echo "   This pane is BUSY by definition — the script is running in it." >&2
  echo "   The detector is falsified. Do NOT trust the verdicts below; fix the" >&2
  echo "   patterns (see the header) before acting on any seat." >&2
  exit 2
fi

for r in "${roles[@]}"; do
  v=$(pane_is_busy "$r")
  if [ "$QUIET" = 1 ]; then
    [ "$v" = "BUSY" ] && printf '%-24s %s\n' "$r" "$v"
  else
    printf '%-24s %s\n' "$r" "$v"
  fi
done
