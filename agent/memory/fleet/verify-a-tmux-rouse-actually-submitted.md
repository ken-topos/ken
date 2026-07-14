---
scope: fleet
audience: (see scope README) — anyone rousing a seat via tmux
source: I-6 stall, 2026-07-14 — a ring idled 25 minutes on an unsent rouse
---

# A tmux rouse can land in the input buffer and never submit — VERIFY IT

`tmux send-keys -t moot-<agent> -l '<text>'` followed by `tmux send-keys -t
moot-<agent> Enter` **does not reliably submit.** The text lands in the pane's
input line as `› [Pasted Content NNNN chars]…` and **just sits there**. The agent
is idle. You believe you kicked it.

**This is indistinguishable from a Codex seat silently ending its turn** — the
pane looks quiet either way — so it gets misread as "quiescent, nothing to do,"
and the lane stalls until someone notices.

**It cost 25 minutes of a live WP:** `runtime-implementer` released the I-6
branch on request, and the hand-back message never submitted. The seat sat idle
waiting for a branch it had just given away, while the Steward's watchdog read
the ring as quiet. The same thing had already happened to `language-leader` in
the same hour.

## The rule

**After every rouse, capture the pane WIDE and confirm it is actually running.**

```sh
tmux send-keys -t moot-<a> -l '<text>'; sleep 1
tmux send-keys -t moot-<a> Enter;      sleep 3
tmux capture-pane -p -t moot-<a> | grep -v '^$' | tail -20   # WIDE. Not tail -3.
```

- ✅ **`• Working (Ns • esc to interrupt)`** → it submitted.
- ❌ **`› [Pasted Content …]`** still on the input line, **and no `Working`
  line** → **it did NOT submit.** Send a bare `Enter` to that pane and re-check.
- ⚠️ **`• Queued follow-up inputs` / `Messages to be submitted after next tool
  call`** → the agent **is busy** and your message **is** queued. **This is
  fine.** **DO NOT RESEND — you will double-queue it.**

## ★ `tail -3` WILL LIE TO YOU — the `Working` line renders ABOVE the queue block

The two states above are distinguished by a line that a narrow tail **cuts off**.
A busy seat with a queued message renders:

```
• Working (41s • esc to interrupt)          ← the line that matters
• Messages to be submitted after next tool call
  ↳ <your text>
• Queued follow-up inputs
› Explain this codebase                     ← placeholder; input is EMPTY
```

**`tail -3` shows you only the bottom — the queue block and an empty input line
— which reads exactly like a dead pane.** I misread this and nearly re-sent a
rouse to a seat that was already working on it. **Capture at least 20 lines.**
(The identical trap exists for `/compact`: the `Compacting…` progress bar also
renders *above* the input, so a narrow tail shows a stale `❯` and reads as a
false "did not land.")

**Do not report a lane as quiescent on a pane read alone.** Cross-check with
`git -C .worktrees/<a> log --oneline origin/main..HEAD` + `status --short`. A
seat that is *idle with your text unsent* and a seat that *finished its turn*
look the same from the outside, and only the git state distinguishes "never
started" from "done."

## Recovering a genuinely stuck turn

If the seat is `Working` on a stale turn and your messages sit queued behind it
(e.g. an implementer stuck 16 minutes in re-orientation without touching a free
branch): **send `Escape`** — the pane itself says *"press esc to interrupt and
send immediately."* That interrupts the model turn and flushes the queue
(`• Model interrupted to submit steer instructions`). **Verify the tree is clean
first** — Escape discards in-flight work.

Sibling of [[pane-suggestion-text-is-not-agent-state]] and
[[codex-seats-do-wake-real-gap-is-reply-to-eventid]]. Same family: **"sent" is
not "received," and "quiet" is not "done."**
