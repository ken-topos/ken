---
scope: fleet
audience: (see scope README)
source: 2026-07-30, Runtime — candidate `29d74605` handed off to QA with the
  leader not mentioned; both seats then idled for individually correct reasons
---

# A candidate handoff that skips the leader deadlocks the ring

An implementer finished a two-hour turn, committed a candidate, and posted the
handoff with `metadata.mentions = ["<runtime-qa>"]` — **QA only. The leader was
not mentioned.**

QA received it, read it, and **correctly held**: *"Candidate handoff noted;
awaiting Runtime Leader's explicit QA review request."* That is the ring's law —
QA reviews on its leader's request, not on an implementer's say-so. Meanwhile the
leader's last turn was an ack of an unrelated post; **it had no signal that a
candidate existed at all.**

⇒ Both seats idle. Each one's reasoning was right. **The edge between them was
unowned**, and no status line anywhere reported a problem.

⛔ **This is NOT a strand.** The mention was delivered and read; the receiving
composer held only a placeholder. Every delivery-side check passes. The composer
sweep is **structurally blind** to it.

**How to apply:**

- ⭐ **Mention the LEADER on every candidate handoff, not just the next
  worker.** The implementer→QA edge looks like the work edge, but the
  *authority* edge runs through the leader. Naming only the next worker hands
  off the artifact and drops the authority to act on it.
- **The detector is a FROZEN finished-turn counter next to a fresh artifact in
  the channel.** QA's footer read `─ Worked for 1m 40s ─` on every sweep for a
  day while a candidate sat in the channel unreviewed. A counter that does not
  move across two dated sweeps, while events addressed to that seat arrive, is
  a dropped intra-ring edge — ⛔ not a quiet seat.
- **Repair by rousing the seat that OWES THE MOVE — the leader — not the one
  that is waiting.** ⛔ Do not issue the review request on the leader's behalf
  and ⛔ do not re-task QA directly: a relay's words do not carry the ring's
  authority, and doing so leaves the same edge unowned next time. Give the
  leader the facts and let it issue its own kickoff.
- **A ring can be deadlocked with zero error signals.** "Nothing is stranded and
  nobody reports a blocker" is consistent with a ring that cannot proceed.

Sibling of [[self-contained-handoff-paste-verbatim-no-event-id]] and
[[handoff-scope-count-must-match-full-thread]] — those govern a handoff's
**content**; this one governs its **addressees**. A handoff can be perfectly
self-contained and still stall the ring by reaching the wrong set of seats.
⚠ [[pane-suggestion-text-is-not-agent-state]] is why the composer sweep cannot
find this one.
