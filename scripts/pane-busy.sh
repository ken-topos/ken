#!/usr/bin/env bash
# pane-busy.sh — classify moot-* panes as BUSY or idle, with two layers of
# self-verification that both run on EVERY invocation.
#
# Usage:
#   scripts/pane-busy.sh                 # sweep every moot-* session
#   scripts/pane-busy.sh runtime-qa ...  # specific seats
#   scripts/pane-busy.sh --quiet         # print only BUSY seats
#
# Exit codes (all explicit — never fall off the end of a conditional):
#   0  verdicts reported and both checks passed
#   2  SELF-TEST failed  — the detector cannot see its own pane
#   3  ARM CHECK failed  — a pattern stopped matching its own shape
#   4  IDENTITY unknown  — cannot determine which pane we are in
#
# ── WHY THIS EXISTS ──────────────────────────────────────────────────────────
#
# The watchdog's busy detector produced FIVE consecutive false-IDLE readings.
# The asymmetry looked structural: the detector is a disjunction of POSITIVE busy
# signals, so any state left un-enumerated defaults to "idle" — and a false idle
# is the reading that makes you interrupt real work.
#
# ⛔ THE "AND NEVER A FALSE BUSY" HALF OF THAT CLAIM WAS FALSE, and this script
#   asserted it for hours. On 2026-07-22 the BACKGROUND arm reported
#   `runtime-implementer` and `verify-qa` BUSY on every 15-minute watchdog tick
#   while both sat idle — a COMPLETED turn (`Baked for 9m 3s`) still holding an
#   ORPHANED shell (`· 1 shell still running`). The operator caught it by looking
#   at the panes; the watchdog reported "all clear" throughout.
#
#   Two lessons, now enforced by controls below rather than by this comment:
#   (a) the false-BUSY direction is the EXPENSIVE one. A false idle costs one
#       spurious nudge. A false BUSY costs an INVISIBLE STALL — and it silences
#       the very backstop that exists to notice.
#   (b) "never observed X" is a statement about the observer. This detector's
#       live self-test runs in its OWN pane, which is BUSY by definition while
#       the script executes, so it was structurally incapable of ever observing a
#       false BUSY. The absence of that reading was a property of the instrument,
#       not of the world — and it got written down as if it were a fact.
#
# Adding arms is unfalsifiable maintenance. Assertions replace it — and every arm
# now carries a NEGATIVE control, not only a positive one.
#
# ── LAYER 1: THE SELF-TEST (does the detector see the live known case?) ──────
#
# This script runs inside a pane. That pane is BUSY, by definition, whenever the
# script is running. If the detector classifies its OWN pane as idle it is
# falsified on the spot, at zero cost, against certain ground truth.
#
# ⛔ THE IDENTITY MUST BE ASKED, NEVER DEFAULTED.
#    `self="${MOOT_ROLE:-steward}"` was a severe defect (adversary, on 82c85f90;
#    reproduced before fixing). MOOT_ROLE is unset in every environment checked —
#    including the Steward's, where the default was correct only by coincidence.
#    For every other caller the oracle silently tested SOMEONE ELSE'S pane, whose
#    state it has no knowledge of. Measured: with the SPINNER arm destroyed and
#    MOOT_ROLE unset, a fully broken detector PASSED its self-test and reported
#    confident verdicts, because the Steward happened to be busy.
#    ⇒ That is worse than having no self-test, because it ships an assurance.
#    The one value you must never guess is the identity the whole oracle rests
#    on. `tmux display-message -p '#S'` answers it for free.
#
# ── LAYER 2: THE ARM CHECKS (does each arm still match its own shape?) ───────
#
# The self-test can only ever prove that AT LEAST ONE arm fires on the self pane.
# A spinner is present there almost always, so the BACKGROUND arm could break and
# stay silently broken — and that arm covers the seat with no spinner and no
# timer, which is the most dangerous one to interrupt. Measured: destroying
# BACKGROUND left the self-test passing while a live seat flipped BUSY -> idle.
#
# ⇒ The self-test proves the detector can SEE the one known case. It does NOT
#   prove the detector WORKS. Each arm therefore carries a positive control AND a
#   negative control over recorded fixtures, asserted every run.
#
# ── MEASURED FACTS behind the patterns (each was an assumption until checked) ─
#
# 1. NEVER pipe the capture through `tail -N`. The spinner renders ~7 lines above
#    the bottom, above the prompt box and the ctx/permissions footer, so a narrow
#    tail cannot see it NO MATTER how many arms the pattern has. (`capture-pane
#    -S -N` is NOT a tail — it captures through the visible bottom — so it is
#    safe.)
# 2. The `❯` prompt box renders while BUSY. It is a constant, not an idle signal.
# 3. `esc to interrupt` appears on only ~1 busy pane in 3 — a Tip line takes that
#    slot otherwise. Not a reliable discriminator alone.
# 4. The spinner GLYPH is not stable: the same message type was counted rendering
#    under two different leading glyphs. A glyph class (`^[✻✽✳]`) is itself an
#    enumeration with no closure proof. Anchor on the SHAPE instead.
# 5. ★ PRESENT PARTICIPLE + PARENS = running. PAST TENSE = finished.
#       "✻ Catapulting… (12m 40s · ↓ 45.9k)"  -> BUSY  (active turn)
#       "✻ Worked for 13m 42s"                -> idle  (completed summary)
#    A bare elapsed-time pattern conflates these and reports FINISHED seats as
#    busy — a false BUSY, which makes the watchdog skip a genuinely stalled seat.
# 6. Background work has NO spinner and NO timer of its own ("2 shells still
#    running"). Busy, and the most dangerous state to interrupt.
# 7. Do not echo these patterns into a pane you are scanning: a naive alternation
#    matches its own source text and reports a false BUSY.
#    ⚠ This immunity is ARM-SCOPED, and an earlier version of this note claimed
#    it script-wide — a claim wider than the instrument backing it. SPINNER is
#    immune because a raw pattern string has no `… (`. BACKGROUND had NO shape to
#    anchor on, so it matched plain PROSE: "the pane showed 2 shells still
#    running when I checked" read BUSY — and it fired on real panes discussing
#    this very script. Both arms are now line-anchored on a leading non-ASCII
#    glyph (optionally indented), which is glyph-AGNOSTIC and so does not
#    reintroduce a glyph enumeration.
#    ⛔ The anchor deliberately allows leading whitespace. Anchoring hard at
#    column 0 would reject an indented continuation line (`  ⎿ 2 shells still
#    running`) — trading a rare false BUSY for a false IDLE, which by point 5 is
#    the dangerous direction. Measured: every live genuine occurrence is
#    glyph-led; the indented form is kept as insurance, not observed necessity.
#
# 8. ⚠ THE ARM CONTROLS ARE A REGRESSION GUARD, NOT A CURRENCY GUARD. They test
#    each arm against RECORDED strings, so if the live UI text drifts, the
#    fixtures stay green AND the self-test still passes via SPINNER — and
#    BACKGROUND can rot silently exactly as before. They prove the patterns are
#    self-consistent, not that they still describe reality. The only currency
#    check is re-measuring against live panes.

set -uo pipefail

SPINNER='^.{1,3} [A-Z][a-z]+… \([0-9]+(m [0-9]+)?s'
BACKGROUND='^[[:space:]]*[^ -~[:space:]].*([0-9]+ shells? still running|Waiting for [0-9]+ background)'

# ── Arm checks: positive AND negative control per arm ────────────────────────
# A negative check passes for any reason, so an arm matching nothing would sail
# through a positives-only suite. Each arm must ACCEPT its own shape and REJECT
# the shape it exists to be distinguished from.
arm_checks() {
  local fail=0 d
  # SPINNER must ACCEPT an active turn, under varying glyphs
  for d in '✻ Catapulting… (12m 40s · ↓ 45.9k tokens)' \
           '─ Frolicking… (1m 8s · ↓ 3.7k tokens)' \
           '✽ Incubating… (8m 47s)'; do
    grep -qE "$SPINNER" <<<"$d" || { echo "ARM SPINNER failed to ACCEPT: $d" >&2; fail=1; }
  done
  # SPINNER must REJECT a completed turn (the false-BUSY guard) and pane chrome
  for d in '✻ Worked for 13m 42s' \
           '✻ Crunched for 1m 11s' \
           '  ctx 15% · Sonnet 5' \
           '❯ check for new mentions'; do
    grep -qE "$SPINNER" <<<"$d" && { echo "ARM SPINNER wrongly ACCEPTED: $d" >&2; fail=1; }
  done
  # BACKGROUND must ACCEPT a turn BLOCKED on a shell — no spinner, NO timer.
  # The absence of a `for <time>` completion is what makes it live.
  for d in '✻ Cogitated · 2 shells still running' \
           '  ⎿ 2 shells still running' \
           '✻ Waiting for 1 background agent to finish'; do
    grep -qE "$BACKGROUND" <<<"$d" || { echo "ARM BACKGROUND failed to ACCEPT: $d" >&2; fail=1; }
  done
  # BACKGROUND must REJECT a completed turn
  for d in '✻ Worked for 13m 42s' '  ctx 15% · Sonnet 5' \
           'the pane showed 2 shells still running when I checked' \
           'I wrote: Waiting for 1 background agent to finish'; do
    grep -qE "$BACKGROUND" <<<"$d" && { echo "ARM BACKGROUND wrongly ACCEPTED: $d" >&2; fail=1; }
  done
  # ⛔⛔ THE FALSE-BUSY CONTROL — the case this script got WRONG in production.
  # 2026-07-22: `runtime-implementer` and `verify-qa` were reported BUSY on every
  # 15-minute watchdog tick while both sat IDLE. The operator caught it; the
  # script never could. Both panes read:
  #     ✻ Baked for 9m 3s · 1 shell still running
  # A COMPLETED turn (`Verb for <time>`) that ORPHANED a background shell. The
  # phrase "1 shell still running" persists indefinitely after the turn ends, so
  # the seat reads BUSY forever and the watchdog never nudges it.
  #
  # ★ The previous version of this suite listed exactly these two strings as
  #   things BACKGROUND must ACCEPT. The bug was not an un-enumerated case —
  #   it was ENCODED AS THE SPECIFICATION, and the suite passed green proving it.
  #
  # ★★ Why it survived a self-test: the live check runs against THIS pane, which
  #   is BUSY by definition while the script executes. That is a positive control
  #   and ONLY a positive control — it can confirm "BUSY reads BUSY" and can
  #   never confirm "idle reads idle". A false IDLE costs one spurious nudge; a
  #   false BUSY costs an INVISIBLE STALL. The arm with no negative control was
  #   the arm that failed, in the expensive direction.
  for d in '✻ Baked for 9m 3s · 1 shell still running' \
           '✻ Crunched for 3m 14s · 1 shell still running' \
           '✻ Cogitated for 36s · 2 shells still running'; do
    classify_pane_text "$d" | grep -qx idle ||
      { echo "FALSE-BUSY CONTROL: completed turn + orphan shell classified BUSY: $d" >&2; fail=1; }
  done
  # …and the end-to-end positive control, so the fix above cannot be "return idle".
  for d in '✻ Catapulting… (12m 40s · ↓ 45.9k tokens)' \
           '✻ Cogitated · 2 shells still running'; do
    classify_pane_text "$d" | grep -qx BUSY ||
      { echo "BUSY CONTROL: live turn classified idle: $d" >&2; fail=1; }
  done
  return $fail
}

# A completed turn stamps `Verb for <elapsed>`. A live one stamps `Verb… (`.
# The elapsed counter appears in BOTH, so the counter is NOT a discriminator —
# the preposition is.
COMPLETED='[A-Z][a-z]+ for [0-9]+(m [0-9]+)?s'

classify_pane_text() { # $1 = pane text; echoes BUSY|idle
  local out="$1"
  if grep -qE "$SPINNER" <<<"$out"; then
    echo "BUSY"; return
  fi
  # A background signal counts only on a line that is NOT a completion stamp.
  # `Baked for 9m 3s · 1 shell still running` is a finished turn holding an
  # orphaned shell, not an agent doing work.
  if grep -E "$BACKGROUND" <<<"$out" | grep -qvE "$COMPLETED"; then
    echo "BUSY"; return
  fi
  echo "idle"
}

pane_is_busy() { # $1 = role; echoes BUSY|idle
  local out
  out=$(tmux capture-pane -t "moot-$1" -p 2>/dev/null) || { echo "idle"; return; }
  classify_pane_text "$out"
}

QUIET=0
roles=()
for a in "$@"; do
  case "$a" in
    --quiet) QUIET=1 ;;
    *) roles+=("$a") ;;
  esac
done

# ── LAYER 2 runs FIRST: a broken arm invalidates every verdict below ─────────
if ! arm_checks; then
  echo "⛔ ARM CHECK FAILED: a pattern no longer matches its own recorded shape." >&2
  echo "   The self-test below can only prove ONE arm fires, so it cannot catch" >&2
  echo "   this. Fix the pattern; do NOT trust any verdict from this run." >&2
  exit 3
fi

# ── LAYER 1: identity is ASKED, never defaulted ──────────────────────────────
self=$(tmux display-message -p '#S' 2>/dev/null | sed 's/^moot-//')
if [ -z "$self" ]; then
  echo "⛔ IDENTITY UNKNOWN: cannot determine which pane this is." >&2
  echo "   The self-test rests entirely on knowing that THIS pane is the one" >&2
  echo "   running the script. Refusing to guess — a wrong identity makes the" >&2
  echo "   oracle test someone else's pane and pass vacuously." >&2
  exit 4
fi

if [ "$(pane_is_busy "$self")" != "BUSY" ]; then
  echo "⛔ SELF-TEST FAILED: detector classified its own pane (moot-$self) as idle." >&2
  echo "   This pane is BUSY by definition — the script is running in it." >&2
  echo "   The detector is falsified. Do NOT trust any verdict; fix the patterns" >&2
  echo "   (see the header) before acting on any seat." >&2
  exit 2
fi

if [ ${#roles[@]} -eq 0 ]; then
  mapfile -t roles < <(tmux list-sessions -F '#S' 2>/dev/null | sed -n 's/^moot-//p' | sort)
fi

for r in "${roles[@]}"; do
  v=$(pane_is_busy "$r")
  if [ "$QUIET" = 1 ]; then
    [ "$v" = "BUSY" ] && printf '%-24s %s\n' "$r" "$v"
  else
    printf '%-24s %s\n' "$r" "$v"
  fi
done

# Explicit: the loop's last conditional must not become the exit status.
exit 0
