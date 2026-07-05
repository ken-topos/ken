---
scope: fleet
audience: (see scope README)
source: private memory `reason-in-agent-team-hours-not-human-days`
---

# Reason in agent-team wall-clock hours, never human work-days

**Operator correction (2026-06-30):** When estimating how long remaining work
takes, reason in **agent-team wall-clock hours**, never human work-days or
human-effort. The fleet runs 24/7 — no nights, breaks, or context-switches — so
"~12 hours of enclave-bottlenecked work" means **~12 wall-clock hours**, NOT
"1-2 days." I'd smuggled a human work-day frame onto a continuous machine
system; the operator flagged it ("10-16 hours is not 1-2 days. It's 10-16 hours.
You're thinking in human terms again.").

**Why:** ~24 agent-team-hours produced ~**10 human-years** of effort (the
operator's calibration) — roughly **800-1000x** a single person. So a
"human-months" WP (e.g. a native compiler backend, self-hosting) compresses to
**fleet-hours-to-low-days**, not weeks. My "weeks for Tier B" was the same
human-anchoring error, worse.

**How to apply:**
1. **Project from the OBSERVED ring cadence**, not priors. Measured: the
   spec-elaboration ring (Steward frame → spec-author → conformance-validator →
   merge) is ~24-31 min/WP and *accelerating* as the discipline goes routine.
   Builds run in parallel behind it.
2. **The real rate-limiters are structural, not human:** (a) the **single serial
   enclave** (deliberately kept single-focused — the hard floor: N remaining
   spec-WPs x ~30 min); (b) **dependency chains** (X3 native + S1/S2 self-host
   need L-complete, so they serialize *after* the surface regardless of
   throughput); (c) **shared-subscription capacity** (the whole fleet is
   Anthropic-direct on one subscription — a credit/rate window can stall every
   agent at once); (d) rework if a hard soundness bug escapes (none have).
3. **Both my and the operator's priors are human-calibrated and likely wrong**
   for this system — hold estimates loosely and update from the data. The next
   load-bearing data point is always the most recent ring time.

Worked example (the recalibration): Tier A = the full language verified+secured+
seam-ready (~16 remaining spec-WPs) ≈ **~10-16 wall-clock hours**, not "1-2
days". Whole program incl. native backend + self-hosting ≈ **~2-4 fleet-days**,
not weeks. See steward coldstart infra checks (shared-subscription stall).
