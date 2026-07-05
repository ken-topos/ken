---
scope: build/leaders
audience: (see scope README)
source: private memory `batched-plan-needs-explicit-self-continue-autonomy`
---

# A batched plan needs explicit self-continue autonomy

When the Steward confirms a **batched** multi-deliverable plan to a build
implementer (e.g. "batch unit 2 — land laws 1/2/3/5 on the branch, one gate at
the end"), **explicitly grant self-continue-through-the-batch autonomy in the
same message**: "continue to the next deliverable without waiting for a per-item
proceed-signal; stop only for the end-of-batch handoff or a genuine hard-stop."

**Why:** the implementer's default cadence is to commit a deliverable, post
"continuing to X next," then **end its turn and wait for a Steward "proceed"
signal** — the "continuing next" is aspirational, not self-executing. Without an
explicit autonomy grant, EACH deliverable pauses waiting for a nudge, and each
pause is a silent stall (~20 min observed) that the leader's ring watchdog can
miss (leader also idle/event-driven). Live: Map capstone unit 2 — I confirmed
"batch unit 2" at 00:21 but did NOT say "self-continue without per-law proceed";
law 1→2 needed a manual "proceed to law 2" nudge, and law 3 then sat ~20 min
idle until the operator flagged it. Diagnosis ruled out the obs eq termination
cycle hang (no running recheck process, clean idle prompt) — it was purely the
missing autonomy grant.

**How to apply:** (1) at plan-confirmation time for any batched/multi-item
build, state the self-continue autonomy + the exact hard-stop bar (a recheck
that *hangs* / a genuinely-new mechanism the spec doesn't settle) + the one
end-of-batch handoff. (2) Treat an implementer sitting idle *between* batched
deliverables (status says "continuing to X" but pane is a clean empty prompt, no
build process) as a **stall to nudge**, distinct from a mid-work quiet. (3) The
leader's ring watchdog should catch inter-deliverable idle, not just intra-item
silence. Sibling of reason in agent team hours not human days (both are about
not stalling the 24/7 fleet on avoidable serial round-trips).
