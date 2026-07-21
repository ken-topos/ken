---
id: MODELS-TIER
title: "agent/MODELS.md — Roles-to-tier column is a DEFAULT, not uniform"
status: ready
owner: steward
size: S
gate: none
depends_on: []
blocks: []
github: null
origin: operator correction, 2026-07-21 (agent/MODELS.md commentary)
---

Doc-only fix to `agent/MODELS.md`: the Roles column in the tier table reads
as if build roles are uniformly T2, which is a DEFAULT, not the landed
seating. Documents Runtime's actual (inverted) seating —
`runtime-implementer` is T1, `runtime-leader` is T2 — because implementation
is the hard part on that team, not coordination. Fleet playbook accuracy
only, feeds no gate.

Held on branch `wp/MODELS-TIER-erratum @ 55576c05` (doc-only, +32/−0 lines
per the tracker) — "ready to publish" per the current slim tracker as of
this migration. Note: an earlier `STEWARD-DECISION-LOG.md` entry (PR #793)
records this branch as already published and verified on `main`; the slim
tracker, which this migration treats as authoritative on disagreement, still
lists it as held. This discrepancy is flagged here rather than resolved by
guessing — verify against `origin/main` before treating either source as
current.
