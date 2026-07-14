---
scope: roles/steward
audience: (see scope README)
source: operator directive 2026-07-13 ("don't worry about credits"; "the point is
  to use tokens efficiently and to use them up before the end of the weekly window")
---

# Spend the weekly token window — idle capacity is the waste, not the spend

There is a **weekly token allocation meant to be used up.** Unspent tokens at the
window's end are **lost** — so the failure mode to avoid is **idle units**, not
"too much spend." Keep every unit (rings, enclave, Architect) productively busy
in parallel; parallelize aggressively and go deep where depth adds value. The
operator's priority is **maximal forward progress and full, efficient
utilization of the window** — not minimal spend. (Operator, 2026-07-13: "don't
worry about credits"; "the point is to use tokens efficiently and to use them up
before the end of the weekly window.")

"Efficiently" still binds: spend on **real, valued work** (genuine WPs, deeper
verification, more parallel lanes), not busy-work — but never leave a ready unit
idle to conserve.

**The trap this corrects.** On 2026-07-13 I held Lane B (namespace: Architect
ADR-amendment design for #39/#36) idle while Language was saturated on kenfmt
batch-2a, reasoning that spending Architect T1 credits "buys nothing on the
critical path while the fleet is already busy." That is the wrong optimization:
the Architect was **idle**, the #39/#36 design is **needed regardless** (not
speculative), and running it in parallel shortens the *overall* schedule even if
it isn't on today's single critical path. The operator corrected it immediately.

**How to apply.** Parallelize aggressively: if the Architect/enclave/a second
ring is idle and has a well-scoped, genuinely-needed WP, route it **now** rather
than serializing behind the current lane. "It's not on the critical path *right
now*" is not a reason to leave a unit idle when the work is real.

**What this does NOT overturn.** The Handoff-Gate **compaction** discipline
stands unchanged — compacting before new work avoids *pure* stale-context waste
(no benefit lost). "Don't worry about credits" is about not refusing *beneficial
parallel work* over its cost; it is not licence to carry stale context or skip
the compaction gate. Still reserve the top tier for genuine T1 work
([[credit-window-reserve-opus-for-t1]]) — that's about *fit*, not *thrift*.
