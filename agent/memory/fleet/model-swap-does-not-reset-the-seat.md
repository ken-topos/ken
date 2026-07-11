---
scope: fleet
audience: (see scope README)
source: private memory `model-swap-does-not-reset-the-seat`
---

# A model swap does NOT reset a seat — its context is intact

When the operator swaps a seat's model (e.g. `gpt-5.6-terra` → `gpt-5.6-sol`,
done by connecting to the `moot-<role>` tmux pane), it **only changes the model
parameter on the completion call to the backend** — it does **not** restart the
session, clear the conversation, or drop the seat's in-context thread state. The
seat keeps everything: its playbook load, its memory, its in-flight assignment,
and the thread history it had read.

**So when you rouse a just-swapped seat, do NOT tell it to re-orient / re-read
its thread / "pick up after a reset."** That instruction is wrong and wasteful —
it already has all of that. Just give it the normal continue nudge (point at its
durable in-thread assignment and say proceed).

**Why this bit (2026-07-11):** the operator swapped `foundation-implementer` to
`gpt-5.6-sol` mid-task; I saw it idle at its prompt and mis-read the swap as a
seat reset, rousing it with a long "your session reset, re-orient, re-read the
thread" preamble. The operator corrected: *"a model swap does not reset the
seat. It just changes a param in the completion call."* An idle swapped seat is
just a **no-poll seat awaiting its next rouse** (normal), not a reset one —
diagnose it as an ordinary idle-with-work stall and rouse it to continue.
Sibling of [[pane-suggestion-text-is-not-agent-state]] (don't over-read a pane's
surface for agent state).
