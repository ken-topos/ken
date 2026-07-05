---
scope: roles/steward
audience: (see scope README)
source: private memory `playbooks-state-mechanism-not-intent`
---

# Playbooks must state the mechanism explicitly, not just the intent

For any agent (leaders especially), a playbook that states **intent without
mechanism** is a latent failure. After a compaction, an agent reconstructs its
operating knowledge from the playbook — and if the mechanism is implicit, it
improvises, often wrong. This is model-agnostic: even a capable model rebuilds
from the playbook post-compaction, so the playbook must be self-sufficient.

**Concrete (2026-06-29):** spec-leader's playbook said *"hand each WP to
spec-author to author"* (intent) but never *how*. Freshly compacted, it read
"hand to" as **spawn** and called `claude(prompt)` / the Agent tool to "launch"
its already-running teammates — which starts a fresh, unconfigured Claude that
fails **"503 provider not configured."** spec-leader then mis-escalated it to "a
fleet-wide outage" and proposed re-authoring the trust-root spec elsewhere. The
operator caught it (teammates were healthy in tmux); root cause was delegation
mechanics, and it had **recurred** across leaders.

**Why:** compact wiped memory reflog first — compaction wipes operational
memory; the playbook is the restore point, so it must be **self-sufficient**.

**How to apply:** when authoring/promoting corpus, write the *mechanism* next to
the *intent* — the exact tool call, not just the goal. Fix here: "assign by
**`post_response` mention** of the persistent peer; **never** spawn / `claude()`
/ Agent-tool a teammate" — pinned in spec-leader + build-leader playbooks
**and** COORDINATION §2 (fleet-wide invariant). General rule: every delegation,
query, handoff in this federation is a **mootup mention**; agents are persistent
peers, never sub-agents anyone launches. Audit playbooks for other
intent-without- mechanism gaps (the build leaders got delegation right only by
luck until this).
