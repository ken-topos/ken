#!/usr/bin/env bash
# Controls for scripts/classify-pane-composer.py.
#
# Every fixture below is a POSITIVE or NEGATIVE control for one decision the
# classifier makes. Two of them are the reason this file exists:
#
#   ghost-slash  -- suggestion text that IS the allow-listed command. If the
#                   classifier keys on the text instead of the dim attribute it
#                   returns `slash:/compact` here and the sweep compacts a
#                   healthy seat that nobody sent anything to. ⛔ The whole
#                   mechanism rests on this row failing to fire.
#   dim-off      -- `ESC[22m` is dim OFF and ends in `2m`. A substring test for
#                   `2m` inverts on exactly this sequence, so it is the control
#                   for is_dim_sgr() rather than for the sweep.
#
# `arbitrary` is the control for the standing rule that the sweep must never
# submit composer text it cannot attribute to a completed delivery.
set -uo pipefail
cd "$(dirname "$0")/.."

E=$'\x1b'
pass=0
fail=0

check() {
    local name="$1" expect="$2" fixture="$3" got
    got="$(printf '%s' "$fixture" | python3 scripts/classify-pane-composer.py)"
    if [[ "$got" == "$expect" ]]; then
        printf '  ok    %-14s -> %s\n' "$name" "$got"
        pass=$((pass + 1))
    else
        printf '  FAIL  %-14s -> got %-16s want %s\n' "$name" "$got" "$expect"
        fail=$((fail + 1))
    fi
}

echo "classify-pane-composer controls:"

# --- NEGATIVE controls: must never be actioned -------------------------------
check ghost        ghost "${E}[0;1m›${E}[0m ${E}[2mFind and fix a bug in @filename${E}[0m"
check ghost-slash  ghost "${E}[0;1m›${E}[0m ${E}[2m/compact${E}[0m"

# ⛔ THE SAME NEGATIVE CONTROL IN THE *CLAUDE* PROMPT SHAPE. Measured on a live
#    pane 2026-07-26: Claude renders `❯` + U+00A0 NO-BREAK SPACE + dim text,
#    and the separator-skip loop tested `in " \t"`, which U+00A0 does not match.
#    The loop broke before the dim run, is_dim came back False, and `ghost-slash`
#    above became `slash:/compact` -- the sweep would have compacted a healthy
#    seat on its own suggestion text.
#
#    ⭐ `ghost-slash` was CORRECT and still blind: it is written with the Codex
#    glyph and an ASCII space, so the Claude shape was absent from the control
#    POPULATION, not from the detector. ⛔ When you add a prompt shape to
#    PROMPT, add its ghost row here in the SAME commit -- an allow-listed
#    command reaching Enter is the one outcome with no undo.
NB=$' '
check ghost-nbsp   ghost "${E}[39m❯${NB}${E}[2m/compact${E}[0m"
check ghost-claude ghost "${E}[39m❯ ${E}[2m/compact${E}[0m"
check slash-nbsp   slash:/compact "${E}[39m❯${NB}/compact"
check arbitrary    other "${E}[0;1m›${E}[0m rm -rf /"
check clear        clear "${E}[0;1m›${E}[0m"
check queued       queued "${E}[2mMessages to be submitted after next tool call${E}[0m"

# --- POSITIVE controls: must be actioned ------------------------------------
check slash        slash:/compact "${E}[0;1m›${E}[0m /compact"
check slash-claude slash:/compact "${E}[1m❯${E}[0m /compact"
check dim-off      slash:/compact "${E}[0;1m›${E}[0m ${E}[22m/compact${E}[0m"
check paste        paste "${E}[0;1m›${E}[0m [Pasted Content 2841 chars]"

# A queued paste is HEALTHY: the seat is mid-turn and will consume it. Resending
# double-delivers, so `queued` must win over `paste` on the same pane.
check queued-wins  queued "${E}[0;1m›${E}[0m [Pasted Content 91 chars]
Messages to be submitted after next tool call"

# --- SCROLLBACK ECHOES: the composer is the LAST glyph line ------------------
#
# ⛔ Measured twice on live seats 2026-07-26. A submitted `/compact` stays echoed
# in the transcript for the rest of the session. The classifier iterated FORWARD
# and `break`s on the first match, so it reported a CONSUMED echo as a live
# stranded delivery — and the sweep would have pressed Enter on it.
#
# ⭐ The sharp form of the defect: the verdict was a function of HOW MUCH
# SCROLLBACK THE CALLER CAPTURED. `-S -40` and `-S -300` of the same pane at the
# same instant could disagree, which means it was not measuring the composer.
# ⇒ These rows are `-S`-depth invariance controls, not just echo controls.
check echo-consumed  clear "${E}[0;1m›${E}[0m /compact
• Context compacted
${E}[0;1m›${E}[0m
  gpt-5.6-terra medium"
check echo-above-ghost ghost "${E}[0;1m›${E}[0m [Pasted Content 2841 chars]
• Context compacted
${E}[39m❯${NB}${E}[2m/compact${E}[0m"

# ⛔ AND THE INVERSE, which is the row that stops this being "fixed" by going
# blind: a REAL stranded delivery still has transcript above it, so anchoring on
# the last line must not become "ignore everything". If a future edit makes this
# return `clear`, the sweep has stopped repairing anything and would report a
# healthy fleet forever.
check echo-then-real slash:/compact "${E}[0;1m›${E}[0m /status
• Ran something
${E}[0;1m›${E}[0m /compact"
check echo-then-paste paste "${E}[0;1m›${E}[0m /compact
• Context compacted
${E}[0;1m›${E}[0m [Pasted Content 412 chars]"

# --- BUSY: never submit into a live turn -------------------------------------
#
# ⛔ A high-effort turn shows NEITHER `Working` NOR `esc to interrupt`, only a
# spinner glyph and an elapsed counter. Keying busy-ness on either word reports
# IDLE on a seat that is working — measured on `moot-runtime-implementer`, which
# sat at `✻ Thundering… (14m 13s · ↓ 55.1k tokens)` while a survey called it idle.
# ⚠ The spinner VERB is randomized, so no row here may depend on the word.
check busy-highffort busy "✻ Thundering… (14m 13s · ↓ 55.1k tokens)
${E}[0;1m›${E}[0m"
check busy-codex     busy "• Working (23s • esc to interrupt)
${E}[0;1m›${E}[0m"
check busy-seconds   busy "✽ Scurrying… (24s · ↓ 1.2k tokens)
${E}[0;1m›${E}[0m"
# ⛔ BUSY MUST WIN OVER A STRANDED SLASH. This is the exact live shape from the
# report: an echo/stranded `/compact` on a seat that is mid-turn. Submitting here
# destroys in-flight work, so the verdict must be `busy`, and the Steward looks.
check busy-beats-slash busy "✻ Thundering… (14m 13s · ↓ 55.1k tokens)
${E}[0;1m›${E}[0m /compact"

# --- NOT busy: past-tense forms a FINISHED turn leaves behind ----------------
#
# ⛔ These are the false-BUSY controls, and they matter because a false BUSY makes
# the sweep BLIND — it reports "nothing to repair" and never touches a genuinely
# stranded delivery again. Going blind is the failure mode a careless busy-guard
# introduces, so both directions are pinned.
check done-worked   slash:/compact "─ Worked for 1m 47s ─────────────────
${E}[0;1m›${E}[0m /compact"
check done-cogitate slash:/compact "✻ Cogitated for 34m 18s
${E}[0;1m›${E}[0m /compact"
# A duration inside a scrollback tool echo is far above the status region, so it
# must not read as busy. ⛔ This is why the busy check is positional (pane TAIL)
# rather than a whole-buffer grep for a duration.
check echo-duration slash:/compact "• Ran cargo test (1m 3s)
line
line
line
line
line
line
line
line
line
line
${E}[0;1m›${E}[0m /compact"

# --- MISUSE: fail closed, never `clear` (task #76) ---------------------------
#
# ⛔ `clear` asserts the composer was SEEN and held nothing. An empty buffer
# asserts only that the probe saw nothing at all. Reading the second as the first
# is how a failed `capture-pane` became "nothing to repair".
check empty        unreadable ""
check whitespace   unreadable "

   "
printf '  '
got_argv="$(python3 scripts/classify-pane-composer.py somefile.txt </dev/null 2>/dev/null)"
rc_argv=$?
if [[ "$got_argv" == "unreadable" && "$rc_argv" -ne 0 ]]; then
    printf 'ok    %-14s -> %s (exit %d)\n' "argv-misuse" "$got_argv" "$rc_argv"
    pass=$((pass + 1))
else
    printf 'FAIL  %-14s -> got %s exit %d, want unreadable + nonzero\n' \
        "argv-misuse" "$got_argv" "$rc_argv"
    fail=$((fail + 1))
fi

printf '\n%d passed, %d failed\n' "$pass" "$fail"
[[ "$fail" -eq 0 ]]
