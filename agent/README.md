# Ken agent workflow

Ken is built by multiple agent teams coordinating through
[convo/mootup](https://mootup.io) spaces and GitHub PRs. This directory holds the
behavioral discipline those agents follow.

- **[`COORDINATION.md`](COORDINATION.md)** — cross-cutting rules for *every* Ken
  agent (message types, mention discipline, status, decisions, threads, the
  retro→memory-audit pipeline, topology invariance, the no-poll rule). Read this
  first.
- **`playbooks/`** — per-role skills:
  - [`team-member`](playbooks/team-member/SKILL.md) — any building-team agent.
  - [`team-leader`](playbooks/team-leader/SKILL.md) — one per team space; runs the
    watchdog and keeps the pipeline moving.
  - [`integrator`](playbooks/integrator/SKILL.md) — the single merge/notify
    authority for `main`.
  - [`spec`](playbooks/spec/SKILL.md) — Team Spec, the clean-room mediator.

These patterns are lifted and adapted from the workflow skills the convo team
developed by dogfooding convo on a different (SaaS) codebase. The coordination
nuances transfer; the SaaS/tooling specifics were dropped. The governing git
model is [`../05-git-and-integration.md`](../05-git-and-integration.md).
