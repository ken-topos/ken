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

**The unsolved part is SINGLETONS** (Steward, Architect, Integrator, Librarian).
The earlier "I know we don't have a good way to manage this yet… benefit to
continuity between spec approval and later PR review… no clean place to compact"
was about a **singleton (the Architect)**, not teams: the operator compacts a
singleton by attaching to its `moot-<role>` tmux session, checking `/context`,
and `/compact` by hand. The playbook says singletons self-compact via
`request_context_reset`, but in practice that isn't a smooth process — and
singleton work spans many teams (the Architect reviews everyone), so there's no
clean idle seam and continuity has real value (a spec-approver reviews the impl
PR better with that context).

**The error this corrects:** I over-read the singleton comment as a *team*
statement and **skipped compacting Team Verify before releasing Sec1-build** (a
domain switch, with T1-build retros already in) — a real miss under §2c. Going
forward: compact teams at the agreed boundaries; treat only singletons as the
manual/unsolved case. See wp release process steward spec build.
