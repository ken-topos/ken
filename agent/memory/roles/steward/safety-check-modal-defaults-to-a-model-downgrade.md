# The "additional safety checks" modal defaults to a MODEL DOWNGRADE

**Measured 2026-07-28** on `runtime-implementer`, minutes after the operator had
reseated it to `gpt-5.6-sol high` **by name**.

A Codex seat mid-turn can render:

```
Additional safety checks
This request requires additional safety checks, which can take extra time.
Hang tight or retry with a faster model for a quicker response, though it
may be less capable of handling complex requests.

› 1. Retry with a faster model
  2. Keep waiting
  3. Learn more
```

## ✅ THE OPERATOR HAS RULED — "Keep waiting", standing

**Operator, 2026-07-28, verbatim:** *"You gave the correct answer. We are willing
to wait."*

⇒ ⭐ **This is settled input. Do not re-decide it per occurrence, and do not
escalate it.** Send `Down`, then `Enter`. Option **2**, every time.

## ⛔ Why the reflex is wrong here

Option **1 is pre-selected**. The standing repair for a stranded Codex
delivery — a **bare `Enter`** — therefore **silently downgrades the seat's
model**, undoing an operator seating directive with no error, no log line, and
no channel event. ⚠ The repair for one failure mode is a **directive violation**
in this one, and the two shapes look alike: a seat sitting still, waiting on a
keypress.

⇒ **Before sending `Enter` to an apparently-stuck Codex seat, capture the tail
and check for a numbered option list.** If one is present, read which line
carries `›` and what option 1 does. Any pane-sweep script that sends a bare
`Enter` must refuse when it sees numbered options.

## ⭐ The seat was never stalled

After confirming *Keep waiting*, the footer read `Working (6m 49s • esc to
interrupt)` with the safety-check text as an **inline note beneath the live
spinner**, and the counter kept advancing — 6m49s → 7m30s → 9m55s across the
session's samples, one monotonic turn.

⚠ **The modal is a latency notice, not a block.** Confirm liveness by counter
**advancement**, never by the presence of a spinner glyph.
