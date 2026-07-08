---
scope: roles/steward
audience: (see scope README)
source: private memory `orphan-watchdog-timer-record-id`
---

# Watchdogs use a private CronCreate timer, not the convo schedule_call

**Watchdog timers must be private `CronCreate` jobs, never the convo
`schedule_call`.** The convo `schedule_call` executes its read *on the backend*
and posts the result back into the space as a **System event every participant
sees** — broadcast noise (and the `get_recent_context` variant reads its own
prior fires and recursively nests them, an exponential self-feeding loop).
`CronCreate` (the Claude Code harness tool) enqueues a prompt into the agent's
**own session** — posts nothing to the space, fires only while idle, and is
**session-local** (`durable:false` dies on exit), so it **cannot orphan across a
restart**. Validated 2026-06-29: a `CronCreate` job fires inside a moot-managed
session and adds no event to the space.

**Why:** the operator wanted *private* wakes, not public posts. `schedule_call`
also created un-killable orphans — it's cancellable **only by its creating
session**, so an agent that compacted without recording the `timer_id` left a
timer (`tmr_37rn5qdv4c800`, spec-leader's; and `tmr_37rn2bv6r0400`, steward's)
spamming the space until cancelled. `CronCreate`'s die-on-exit semantics remove
that whole failure class.

**How to apply (the watchdog pattern):**

```
CronCreate(cron="7,17,27,37,47,57 * * * *",
  prompt="Watchdog tick: pull get_recent_context, scan the stall patterns,
    mention only a blocked agent; if clear, do nothing", recurring=true)
```

at session start while you have open work. On each fire, do your *own* direct
`get_recent_context`/`get_space_status` read (private, not posted) + assess;
message the space only on a real stall. **Re-arm at session start**
(durable:false dies on exit), **`CronDelete` on WP close** (`CronList` shows
your jobs). Recurring crons auto-expire after 7 days. This reversed the earlier
"use the convo cron (`schedule_call`)" guidance, which was built on the wrong
assumption that `schedule_call` was a private wake. Promoted into
`agent/COORDINATION.md` §13 + build/spec leader playbooks. See
playbooks state mechanism not intent and compact wiped memory reflog first.
