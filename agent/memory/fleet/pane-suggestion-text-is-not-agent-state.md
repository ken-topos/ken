---
scope: fleet
audience: (see scope README)
source: private memory `pane-suggestion-text-is-not-agent-state`
---

# The tmux pane's gray suggestion text is not agent state

**Operator, 2026-06-30.** The gray text shown after the `❯` prompt in an agent's
tmux pane (e.g. `❯ /compact`, `❯ waiting for next WP kickoff`, `❯ continue`,
`❯ rebase onto origin/main`) is **Claude Code's suggestion of what to prompt the
model with next** — **NOT** typed input, **NOT** the agent's status/state. It
does not fire without an explicit Enter, and `Escape`/`C-u` don't clear it (it
isn't in the input buffer). **In general, disregard any text in that line.**

**Why:** I read a stray gray `❯ /compact` on kernel-leader's pane as a possible
wedge / leftover input and burned calls (`Escape`, then `C-u`) trying to clear
it before posting a kickoff. There was nothing to clear.

**How to apply (sharpens the capture-pane watchdog — steward §7a):** when
diagnosing wedge-vs-working via `tmux capture-pane`, the **`❯ <gray text>` line
is noise — ignore it.** Read the **actual transcript above the prompt**:
`Spelunking…`/`esc to interrupt`/a rising token count = working; the agent's
last real output followed by no activity = idle. Corroborate with
`list_participants` status + `last_seen_at`, never with the suggestion line.
(Earlier I read implementer suggestion lines as status and happened to be right
— but that was luck; the suggestion is not the signal.)
