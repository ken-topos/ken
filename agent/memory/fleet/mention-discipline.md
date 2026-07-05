---
scope: fleet
audience: all agents
source: COORDINATION §2 (operator directive 2026-07-03, evt_2g69q1w71yvtq,
  wp/coord-mention-discipline @ b85e2f5); merges former private memories
  `mention-only-for-question-or-action` + `convo-mention-id-must-be-grepped-not-typed`
---

# Mention discipline

Two independent rules govern every `@mention` on the convo bus — **whether/why**
to mention, and **who** (the correct id). Both fail quietly, so both are habits,
not checks.

## Whether / why — mention IFF a question or a next action

Mention an agent **IFF** (a) you are asking them a question, or (b) you expect a
specific next move from them. **A mention is never an acknowledgment.** Silence
is acceptance — if your only reason to name someone is that you received / agree
with / are proceeding on their message, mention **no one** and post **nothing**.

- A status/checkpoint report mentions nobody.
- A reviewer's APPROVE needs no ack.
- "Packaging X / relaying to Y" is just **done** — mention Y iff Y moves next,
  never announced back to the requester.
- On a substantive routing post (decision, finding, handoff), mention **only the
  one actor whose move is next**, not the observers / requester / CC list.

Fast self-check before naming anyone: *"does this person have a move to make
because I posted?"* If no, drop the mention — often the whole post.

**Why:** a mention that expects no move is pure noise that trains the fleet to
tune mentions out, which then buries the ones that do need action. Honesty-about-
the-boundary, applied to attention.

## Who — grep the id, never type it

A `@mention` id (`agt_…`) must be **grepped from a fresh `orientation()` /
`list_participants` roster**, never typed from memory or pattern-matched to a
familiar-looking one.

**Why:** `post_response` / `reply_to` with a wrong `agt_…` in `mentions` **does
not error** — the message posts fine, it just notifies nobody. There is no error
surface (unlike a bad path or a failing test); the mistake is invisible until
someone notices the silence.

**How:** copy the target's `participant_id` immediately before composing the
mention, every time, even when you're sure. A recurring failure mode is typing
**your own id** by muscle memory (it feels familiar) — and you are never the
actor who acts on your own post, so your own id in a routing mention is always
wrong.
