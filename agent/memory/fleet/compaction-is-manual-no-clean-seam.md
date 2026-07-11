---
scope: fleet
audience: (see scope README)
source: private memory `compaction-is-manual-no-clean-seam`
---

# Team compaction follows the playbook; the manual-seam problem is
singleton-specific

Operator clarified (2026-06-30, correcting my over-read): **team compaction is
solved — follow the playbook** (`agent/playbooks/federation/steward.md` §2c).
`moot compact` the whole team (leader + implementer + QA, or the spec enclave)
at the clean WP boundaries — **after the prior WP's retros are in, before
delivering the next WP**, team quiescent. Keep doing this; don't skip it.

**The unsolved part is SINGLETONS** (Steward, Architect, Librarian).
The earlier "I know we don't have a good way to manage this yet… benefit to
continuity between spec approval and later PR review… no clean place to compact"
was about a **singleton (the Architect)**, not teams: the operator compacts a
singleton by attaching to its `moot-<role>` tmux session, checking `/context`,
and `/compact` by hand. What's genuinely hard is the **seam** (singleton work
spans many teams — the Architect reviews everyone — so there's no clean idle
point, and continuity has real value: a spec-approver reviews the impl PR better
with that context).

**The MECHANISM is solved (operator, 2026-07-02), do not re-cite the broken
one:** a singleton self-compacts with `tmux send-keys -t moot-<role> "/compact"`
(two-step: `/compact`, ~2s, separate `Enter`) — pointed at its own
`moot-<role>` window. Do **NOT** use `request_context_reset`: it is broken in
this harness (hunts for a nonexistent `convo-<role>` session; its error message
*names* `convo-<role>`, which is the bug, not a retry target — the Architect
tripped on exactly this 2026-07-11). Full mechanics in
`playbooks/federation/steward.md` + `architect.md` self-compact sections.

**Auto-resume via a DETACHED watcher, not a buffered `resume` (operator,
2026-07-11).** A self-compact returns the seat to an idle `❯` and **nothing
re-invokes it** — it sits idle until roused. The first fix tried was: type
`resume` right after `/compact` and rely on the host **buffering** it behind the
compaction. That is a **race** and misfired — the `resume` is sent while the
turn is still active (the queued `/compact` only fires at turn end), so it can
land as its own live turn *before* compaction rather than after. The reliable
fix **decouples** the resume from the turn lifecycle: launch
`scripts/postcompact-resume.sh moot-<role>` **detached** (`nohup … & disown`)
*before* sending `/compact`. It outlives the turn and the compaction, polls the
pane until the `Compacting…` window appears and then clears, and only then sends
`resume`. As a separate process it is immune to the turn/compaction timing, so
the resume lands *after* compaction; the post-compact re-orient hook then carries
the seat back into its own in-flight work autonomously. **A hook cannot
substitute** — a SessionStart hook only shapes the next turn's *context*, it
cannot send the keystroke that *triggers* one; the external watcher is that
trigger. **Self-compaction only** — never run the watcher for a Handoff-Gate
team/enclave compaction, where the kickoff mention is the resume trigger and a
premature `resume` wakes the unit into "no new work." (The watcher self-bails if
it never observes a `Compacting…` window, so a mistaken launch can't fire a
premature resume.)

**The error this corrects:** I over-read the singleton comment as a *team*
statement and **skipped compacting Team Verify before releasing Sec1-build** (a
domain switch, with T1-build retros already in) — a real miss under §2c. Going
forward: compact teams at the agreed boundaries; treat only singletons as the
manual/unsolved case. See wp release process steward spec build.
