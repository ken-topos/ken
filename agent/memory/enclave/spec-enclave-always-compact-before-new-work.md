---
scope: enclave
audience: (see scope README)
source: private memory `spec-enclave-always-compact-before-new-work`
---

# The spec enclave compacts unconditionally before every new work unit

**Operator rule (Pat, 2026-07-04, after correcting the enclave TWICE):** the
spec enclave is **ALWAYS COMPACTED BEFORE NEW WORK** — same unconditional rule
as build teams ("no exceptions"). There is **no before-work ctx threshold** for
the enclave. The "high ctx threshold is 33%" is a **mid-flight ceiling only**
(if a unit drifts over 33% *while working* with no handoff in sight, compact at
the next safe seam); it is **never** a reason to skip the before-new-work
compaction because a unit is "still under it."

**Why:** the Opus enclave is the single most expensive unit in the fleet, and
every turn re-bills stale context. A per-role before-work threshold *invites*
the "they're still under it / I'll compact at the next seam / let me wait for X
first" rationalization — which is exactly how the failure recurs. Removing the
threshold removes the rationalization. Compaction is not lossy for what matters
(the summary keeps recent detail; the agent re-fetches sources from the
filesystem at pickup).

**The failure this fixed (mine):** I let **CV run uncompacted through *five*
unrelated work units** — FS-driver conformance → FS Phase-2 gate →
challenge-suite run → drills → reconcile — to **67%**, because I treated the
enclave as a continuous reviewer I could keep feeding and kept deferring
compaction ("compact at the flip seam," "wait for the retro first"). Pat
compacted CV manually and hardened the rule. This was a **repeat** of the
2026-07-03 "left CV at 60%, reviews at merge" correction — the repeat is the
tell that a threshold was the loophole.

**How to apply:**
- **Every new work unit handed to the enclave is a fresh compact-first kickoff**
  — no matter how "warm" or "under threshold" the context looks. The enclave is
  NOT a continuous reviewer whose context you may accumulate; each unit boundary
  is a compaction trigger.
- **Do not even read the ctx% to decide** — the answer before new work is always
  compact.
- **Retro-ordering unchanged but drive it NOW:** the prior retro must be posted
  before you compact (compaction eats an un-posted retro), so **nudge the retro
  to completion immediately** and then compact — "waiting for the retro" is a
  reason to *finish the retro now*, never to defer the compaction or let the
  unit take on new work uncompacted.
- Sibling of playbooks state mechanism not intent (compact mechanically at the
  seam, never on a story about ctx level). Lives in `steward.md` §2c
  handoff-gate step 4 + the step-2 enclave-handoff note.
