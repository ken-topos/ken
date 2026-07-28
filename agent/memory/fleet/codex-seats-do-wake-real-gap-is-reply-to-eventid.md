---
scope: fleet
audience: (see scope README) — anyone diagnosing a Codex/gpt build seat that
  looks unresponsive to a convo mention, or drafting a claim about which
  harnesses can/can't auto-wake
source: private memory `codex-seats-do-wake-real-gap-is-reply-to-eventid`;
  operator correction 2026-07-11
---

# Codex seats DO auto-wake on mentions — the real gap is `reply_to`

Codex/gpt-5.6 build seats were twice reported to the operator as "don't
auto-wake on a convo mention (only Claude-Code seats start a turn)." **That
is FALSE.** Proven by `.moot/logs/channel-<role>.log`: every build seat runs
a `channel_runner` daemon (a `ChannelAdapter`, child of its `codex` harness
process) that holds a healthy WebSocket stream and, on a mention,
**`tmux send-keys`-delivers it** — logged as `convo.channel INFO Pushed
mention notification via tmux: @you mentioned by … (pane_command=codex)`.
A `language-leader` seat received and acted on 20+ mentions this way
(kickoffs, QA verdicts, gate approvals). Delivery to non-Claude harnesses
is a first-class, working path.

**The real dropped-communications bug (operator, 2026-07-11):** the seat
*receives* the mention and tries to answer, but **`reply_to` is rejected by
the convo platform as an invalid event_id** — a response-path, server-side
bug. This is NOT fixable locally and NOT an adapter/delivery issue.
Practical fleet workaround while it's open: answer with a fresh
mention-led `share`/`post_response` (populate the `mentions` array) instead
of `reply_to`, and grep the target id from recent context rather than
echoing one.

**What WAS a real, correctly-fixed facet:** a ruling/answer posted with an
**empty `mentions` array** never triggers tmux delivery — the daemon's
relevance check keys on the mentions array — so the asker never gets woken.
The fix: always mention the next-move owner, even when it's the asker.

So a report of "codex seat unresponsive" has (at least) three distinct
possible causes: (1) empty-mentions → no delivery [check the poster's
`mentions` array]; (2) "codex can't wake" [FALSE as a general claim — check
the daemon log before asserting it]; (3) `reply_to` event_id rejection
[real, platform-owned — use fresh mention-led posts instead].

**How to apply:** never assert a seat "can't wake" from its idle pane
alone — `tail` its `.moot/logs/channel-<role>.log` for `Pushed mention
notification via tmux` first; the daemon log is ground truth. Housekeeping
seen in passing: orphaned/duplicate `channel_runner` processes accumulate
across seat restarts — harmless but worth a periodic reap. Reinforces
[[pane-suggestion-text-is-not-agent-state]].
