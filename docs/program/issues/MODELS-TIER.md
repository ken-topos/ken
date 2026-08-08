---
id: MODELS-TIER
title: "agent/MODELS.md — the Runtime seating is the fleet-wide norm, not an exception"
status: draft
owner: steward
size: S
gate: none
depends_on: []
blocks: []
github: null
origin: operator correction, 2026-07-21 (agent/MODELS.md commentary)
---

> ## STATUS CORRECTED `ready` TO `draft` — 2026-08-08, Steward
>
> **`ready` means shovel-ready: a written frame, dependencies merged.**
> This node was not, because
> no frame exists. Steward-owned process work, deprioritized under
> playbook §1 while build teams are running.
>
> **The correction is not a downgrade of the work.** A node advertising
> startable work it does not have makes the backlog read deeper than it
> is, and that depth is exactly what a Steward reads to decide whether a
> team is idle for want of work or for want of a lane.

**The original erratum has LANDED** — `agent/MODELS.md` on `origin/main`
now carries "★ The Roles column is a DEFAULT, not the landed seating
(operator, 2026-07-21)", merged as `f4f8e06a`. Verified by content against
`origin/main`, not inferred from a branch or a tracker line.

The branch `wp/MODELS-TIER-erratum @ 55576c05` is a **stale local
leftover**, not held work: it has a zero diff against `origin/main` for
`agent/MODELS.md`, and it exists only locally (not on origin). Do not
publish it. It can be deleted.

## What actually remains

A narrower reframing. The landed text calls Runtime **"the standing
exception, and it is INVERTED vs the table"**, but the full-fleet sweep
showed `implementer = T1` / `leader = QA = T2` is the **fleet-wide
convention** — Runtime is the norm, not a deviation. One paragraph, doc-only,
no gate.

## Provenance note

The migration that created this file flagged a conflict between two sources:
the decision log (now folded into [`diary/`](../diary/INDEX.md)) recorded
this as already published, while the slim tracker still listed it "held."
The tracker was carrying a stale claim — restructuring preserves staleness.
The decision log was right. Resolved by checking `origin/main` directly,
which is the only source that settles a landed-or-not question.
