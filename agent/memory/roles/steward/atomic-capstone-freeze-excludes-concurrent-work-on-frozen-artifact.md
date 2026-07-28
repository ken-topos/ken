---
scope: roles/steward
audience: (see scope README) — anyone sequencing a lane against a scheduled
  freeze window
source: 2026-07-13, operator correction (Pat) while sequencing the Ken CLI
  mega-effort — the kenfmt capstone C catalog-reformat freeze
---

# An atomic whole-artifact freeze excludes every concurrent lane that lands in it

I proposed starting the catalog-closure lane (new catalog packages,
Foundation-owned) **immediately** on an idle ring — reasoning it was
independent, zero-trust-delta, and unblocked, so it could run in parallel
with the kenfmt capstone C (a Steward-scheduled window that reformats the
**entire catalog atomically** and turns on the strict `ken fmt --check` gate
in one merge). The operator corrected: *"Kick foundation after B4-C lands.
We want no catalog changes in flight during C."*

**Why the plan was wrong:** new catalog packages landing concurrently would
(a) be un-reformatted files racing the freeze, (b) invalidate C's
whole-catalog preview and its "green day-one" gate, and (c) create
merge/rebase churn against the largest blast-radius merge on the roadmap.
"Independent and unblocked" is **not** sufficient — the deciding axis is
**does the lane's output land in the artifact being frozen.**

**The rule:** when a WP performs an atomic whole-artifact rewrite behind a
freeze window (catalog reformat, mass migration, a corpus-wide
canonicalize), that window is **exclusive over the artifact** — hold every
lane whose builds touch that artifact until the freeze lifts, regardless of
how independent they look. Partition concurrent work by *which artifact it
lands in*:

- lanes that touch the **frozen artifact** (there, `catalog/`) → **hold
  until after the capstone merges**;
- lanes that touch **disjoint artifacts** (there, `crates/` + `spec/` — a
  host-ABI/effects program, contract design, spec-prerequisite cleanup) →
  **safe to run concurrently** with the freeze.

**How to apply:** before kicking any lane during (or just before) a
scheduled capstone/freeze, ask *"do this lane's merges land in the artifact
the capstone rewrites?"* If yes, hold it behind the capstone; if no
(disjoint artifact), run it. Design/framing work and disjoint-crate work are
always safe; the frozen artifact's own content is exclusive to the
capstone.

This is the scheduling twin of
[[entrypoint-abi-change-is-never-corpus-disjoint]] — a follow-on where a
lane declared "crates-only, catalog-disjoint" turned out to force edits into
the very corpus a freeze was protecting, for a reason a crate-location
argument could not see.
