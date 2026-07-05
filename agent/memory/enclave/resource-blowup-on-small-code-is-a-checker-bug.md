---
scope: enclave
audience: (see scope README)
source: private memory `resource-blowup-on-small-code-is-a-checker-bug`
---

# A resource blowup on small source code is a bug to fix at the root

Operator (Pat) ruling, 2026-07-03, on the Map-capstone `toListOrdered` OOM (~12
GB SIGKILL kernel-checking a *small* proof over *small* input): **"12 GB
consumed by such small snippets of code is a bug, not a resource restriction.
The language is non-functional with that kind of resource usage."** Plus the
root-cause prior: **"almost certainly an unbounded recursion bug with an
allocation in the path"** (the textbook small-input + small-code + huge-memory
signature).

**Why:** a usable resource profile on ordinary-sized source is a **functionality
bar**, not a nice-to-have. A trusted checker that blows up on small code is
"non-functional" regardless of correctness. Throwing memory/infra at it (a
bigger box, a non-sandbox run, swap) **hides the bug** — and if the recursion is
genuinely unbounded, no finite memory ever completes it, so a bigger box is not
just papering-over but **futile**.

**How to apply:**
- When a perf/resource wall appears in the trusted checker/evaluator on
  small-to-normal input, classify it as a **root-cause bug to FIX**, default to
  the general fix in the checker — NOT a resource-budget escalation to the
  operator, and NOT a "make every author hand-contort their proof around it"
  workaround. This *strengthens* perf primitive vs fix the evaluator fork: fix
  the evaluator/checker, don't grow the trust root, don't push the cost onto
  users.
- **Take the "bigger budget / non-sandbox run" option off the table** for this
  class. Do not hold it as a resolution or escalate for it. (I was about to hold
  a swap-backed in-sandbox escalation lever; Pat's ruling killed it.)
- The **general checker fix subsumes the family** (every future proof of that
  shape) — the subsume-don't-proliferate win. A proof-restructure that fixes the
  *one* proof but leaves the checker pathological on the next same-shaped proof
  **does NOT satisfy the "language functional" bar** → the general checker fix
  is a required follow-on *even if* the restructure also works.
- **Diagnostic tell** (cheap, capture it): under a memory `ulimit`, watch
  whether memory climbs **monotonically with no plateau** (⇒ unbounded recursion
  — the bug) vs **rises to a finite peak** (⇒ bounded-but-expensive / un-shared
  re-traversal). "Small input + small code + huge alloc" ⇒ suspect an unbounded
  recursion with an allocation on the path first.
- **Soundness constraint on the fix** (Architect, non-negotiable): a
  memoization/sharing/lazy-whnf conversion-perf fix must make the trusted check
  **COMPLETE and PASS cheaply**, deciding **exactly the same convertibility** —
  never skip/weaken/bypass a check to "fit" (that converts a resource wall into
  a soundness hole, conformance hand feeds the deliverable trust-root
  integrity). Fixed ≠ skipped.

Both this and perf primitive vs fix the evaluator fork point the same way:
outer-ring/checker resource weakness is fixed in the checker, never budgeted
around or pushed onto the trust root or the user.
