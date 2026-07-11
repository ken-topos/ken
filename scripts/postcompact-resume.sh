#!/usr/bin/env bash
# postcompact-resume.sh — reliably fire a `resume` into a pane AFTER its
# self-`/compact` completes, so a self-compacting agent auto-continues instead
# of sitting idle at an empty prompt.
#
# WHY THIS EXISTS (the race it fixes)
#   The naive mechanism — send `/compact`+Enter, then IMMEDIATELY send
#   `resume`+Enter from the same turn and rely on the host buffering the resume
#   behind the compaction — is unreliable. The `resume` is sent while the
#   agent's turn is still active (the queued `/compact` fires only at turn END),
#   so whether `resume` buffers behind the compaction or lands as its own live
#   turn is a race on host timing. Observed to misfire (resume surfaces as an
#   early "user sent a message while working" instead of cleanly post-summary).
#
#   The fix: DECOUPLE the resume-send from the agent's turn lifecycle. This
#   script is launched DETACHED (it outlives the turn and the compaction),
#   watches the pane until compaction actually completes, and only THEN sends
#   the resume. A separate process is immune to the turn/compaction lifecycle.
#
# USAGE (as the agent's LAST action, BEFORE it sends its own `/compact`):
#   nohup scripts/postcompact-resume.sh moot-<role> >/tmp/pcr-<role>.log 2>&1 &
#   disown
#   # ...then send `/compact`+Enter and end the turn.
#
# The watcher is already polling when `/compact` fires, catches the
# `Compacting…` window, waits for it to clear, and sends `resume`. It is
# self-terminating (bounded by COMPLETE_TIMEOUT) — not a leaking loop.
#
# ARGS / ENV
#   $1  TARGET   tmux session/pane to drive     (default: moot-steward)
#   $2  MESSAGE  text to send after compaction   (default: resume)
#   APPEAR_TIMEOUT    max secs to wait for `Compacting…` to appear (default 60)
#   COMPLETE_TIMEOUT  max secs to wait for it to clear             (default 300)
#   SETTLE            secs to let the prompt settle before sending (default 3)
#   POLL              poll interval secs                           (default 2)
set -u

TARGET="${1:-moot-steward}"
MESSAGE="${2:-resume}"
APPEAR_TIMEOUT="${APPEAR_TIMEOUT:-60}"
COMPLETE_TIMEOUT="${COMPLETE_TIMEOUT:-300}"
SETTLE="${SETTLE:-3}"
POLL="${POLL:-2}"

pane() { tmux capture-pane -p -t "$TARGET" 2>/dev/null | tail -25; }
# The live compaction indicator Claude Code renders is "Compacting…" (with a
# spinner). Match it case-insensitively; the present participle avoids matching
# the post-compact reorient text ("compacted", "compaction").
is_compacting() { pane | grep -qiE 'compacting'; }

log() { printf '[postcompact-resume %s] %s\n' "$TARGET" "$*"; }

# 1) Wait for compaction to START — DO NOT send resume before `/compact` has
#    fired, or we reproduce the exact early-resume race we exist to prevent.
log "waiting for compaction to start (<= ${APPEAR_TIMEOUT}s)"
start=$SECONDS
saw=0
while (( SECONDS - start < APPEAR_TIMEOUT )); do
  if is_compacting; then saw=1; log "compaction started"; break; fi
  sleep 1
done
if (( saw == 0 )); then
  # Compaction never became visible. Rather than risk an early resume, bail —
  # the agent is still holding its turn, or /compact never fired. A missed
  # resume is recoverable (the agent can be roused); a premature resume is the
  # bug. Exit non-zero so the log shows it.
  log "compaction never observed within ${APPEAR_TIMEOUT}s; NOT sending resume (bailing to avoid an early-resume race)"
  exit 3
fi

# 2) Wait for compaction to COMPLETE — the indicator clears and stays clear.
log "waiting for compaction to complete (<= ${COMPLETE_TIMEOUT}s)"
start=$SECONDS
while (( SECONDS - start < COMPLETE_TIMEOUT )); do
  if ! is_compacting; then
    sleep "$SETTLE"
    is_compacting || { log "compaction complete"; break; }
  fi
  sleep "$POLL"
done

# 3) Send the resume — split text and Enter to avoid the fused-keystroke race
#    that can leave the line unsent.
log "sending '${MESSAGE}'"
tmux send-keys -t "$TARGET" -l "$MESSAGE"
sleep 1
tmux send-keys -t "$TARGET" Enter

# 4) Verify it was accepted; one retry if the pane still looks idle.
sleep 4
if pane | grep -qiE 'esc to interrupt|working|cogitat|brewed|forging|percolat'; then
  log "resume accepted (agent is working)"
else
  log "resume not visibly accepted; retrying once"
  tmux send-keys -t "$TARGET" -l "$MESSAGE"
  sleep 1
  tmux send-keys -t "$TARGET" Enter
fi
log "done"
