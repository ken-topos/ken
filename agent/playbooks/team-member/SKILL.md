---
name: ken-team-member
description: Workflow discipline for any Ken building-team agent (Kernel, Verify, Language, Runtime, Foundation, Ergo, Research). Handoffs, status, escalation, PR hygiene.
---

# Ken team member

You build one team's work packages. Read `../../COORDINATION.md` first — it has
the cross-cutting law (no-poll, mentions, status, decisions, grounding). This
playbook adds the building-agent specifics.

## Your loop

1. Pick up a work package (WP) from `../../../04-program-of-work.md` as assigned
   by your team leader. One WP (or one reviewable sub-task) at a time.
2. Branch `wp/<WP-ID>-<slug>` off the latest `main`.
3. Implement **from `/spec` and `/conformance`** — never from prototype source
   (`../../../CLEAN-ROOM.md`). If the spec is silent and the answer is
   structurally determined, resolve and cite it; if it's a genuine fork, escalate
   to Team Spec.
4. Write the common-case tests before you hand off. Keep the PR small.
5. Open the PR; cite the WP ID, the acceptance criteria met, and your spec
   sources. Do **not** merge.
6. **Hand off, then stop.** Post a structured handoff (below), set status, and
   wait for a notification. Do not poll.

## The handoff template (prevents the silent handoff)

When work is ready for the next actor (reviewer, dependent team, or the
Integrator via your team leader), post in-thread, mentioning **only** the next
actor:

```
pr_ready: <WP-ID> <one-line what>
- branch: wp/<WP-ID>-<slug>   PR: <url>
- did: <2-3 bullets>
- spec: <spec §/file this implements>
- next: <what the next actor needs to do>
- watch: <anything risky / cross-team interface touched>
```

Then **stop**. Do not wait for an ack; do not re-ping. If you are genuinely
blocked, set status to blocked and post a `blocked`-typed note mentioning the one
agent who can unblock you.

## Discipline that the convo team learned the hard way

- **Don't author outside your lane to "save a round."** If something in another
  team's crate looks wrong, file a `bug`-typed note to that team (cap your own
  investigation at ~5 minutes) and continue. Cross-lane "quick fixes" produce
  duplicated, half-informed work because you lack the owning agent's context.
- **A non-blocking bug never stops the pipeline.** File it, keep going.
- **Propose a Decision only for real tradeoffs** (kernel/semantics/API). Bug
  fixes are not decisions.
- **Re-resolve thread IDs after a context reset** before replying.
