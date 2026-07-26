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

printf '\n%d passed, %d failed\n' "$pass" "$fail"
[[ "$fail" -eq 0 ]]
