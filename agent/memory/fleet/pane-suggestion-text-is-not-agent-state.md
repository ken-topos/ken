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

## ★★ 2026-07-14 — it became a WATCHDOG, and its remedy was DESTRUCTIVE

`language-leader` escalated to the Steward with what it called *"watchdog
evidence"*: the implementer had *"remained `Working` for six minutes on the
generic `Explain this codebase` orientation turn"*, and asked for **pane recovery
(`Escape`/`C-c`)** so the queued SUB-1b assignment could run.

**Every load-bearing word of that was the suggestion line.** `Explain this
codebase` is Claude Code's **stock placeholder in an EMPTY composer** — it
renders **identically whether the agent is idle or deep in work**, which is
exactly why it can be evidence of neither. Scrolled up two screens, the same
pane showed the implementer had **posted its SUB-1b pickup event**
(`evt_4wx6xtx0w5nk3`, into the leader's own thread) and was reading `MODELS.md`
and its implementer playbook. **It was six minutes into the assigned WP.**

**Had the Steward acted on the report, the `Escape` would have destroyed six
minutes of live T1 work on the critical path.**

**The escalation: a false-positive watchdog whose remedy is destructive is worse
than no watchdog at all.** Before recommending an interrupt, you owe **one**
positive check that the agent is *not* working — and two were free here, both
contradicting the alarm: **(a)** the agent's own **posted event**, and **(b)** its
`list_participants` **status** (*"Implementing SUB-1b Route B"*). **An agent that
has posted a pickup event is not stuck on an orientation turn.**

**The only sound stall signal is ACTIVITY, not display: no new tool call in the
pane across two checks several minutes apart.** Same shape as
[[an-oracle-that-greps-a-name-fires-on-prose-that-denies-it]] — *the string is a
CANDIDATE, never the DECISION* — and the reason both bite is the same: **the
cheap surface signal is available, and the real one takes one more look.**
