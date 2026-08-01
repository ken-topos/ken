# The "additional safety checks" modal needs NO answer — leave it alone

**Operator, 2026-07-29, verbatim:** *"Note that you don't need to select an
option on the 'additional safety checks' prompt. It will keep working
regardless."*

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

## THE ACTION IS: DO NOTHING

**This is not a stall, and it is not a delivery failure.** The request
proceeds while the prompt is on screen. ⇒ In a pane sweep, a seat showing this
modal is **working** — do not classify it as blocked, do not send it a keypress,
do not nudge it in the channel.

**Doing nothing is also the safest option**, because the only way to harm the
seat here is to interact with it (below).

## Why you must not reflex-`Enter` it

Option **1 is pre-selected**. The standing repair for a *stranded Codex
delivery* — a bare `Enter` — therefore **silently downgrades the seat's model**,
undoing an operator seating directive with no error, no log line, and no channel
event.

The two shapes look alike: a seat sitting still, apparently waiting on a
keypress. ⇒ **Before sending `Enter` to an apparently-stuck Codex seat, capture
the tail and check for a numbered option list.** If one is present, the seat is
not stranded — leave it. Any pane-sweep script that sends a bare `Enter` must
refuse when it sees numbered options.

## RETRACTED — "an unanswered modal blocks the turn"

An earlier version of this lesson claimed an unanswered modal **blocks** the turn
and froze the elapsed counter (`Working (8m 05s)` → `(8m 19s)` across 12 minutes
of wall clock), and prescribed clearing it as an urgent repair. **The operator's
ruling supersedes that.** The work continues; whatever the frozen counter was, it
was not a stopped turn.

⇒ **A frozen elapsed counter under this modal is NOT evidence of a blocked
seat.** That inference manufactured an urgent intervention out of a healthy seat
— which is the expensive direction, since the intervention itself is what can
downgrade the model.
