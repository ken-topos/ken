---
name: a-disclosed-deferral-gets-guard-rails-written-for-the-whole
description: When work splits and one half is deliberately deferred, the artifacts AROUND the deferred half get authored as if the whole thing shipped — the doc claims both, the adjacent comment argues the defect away, the test suite covers only the landed half. On any disclosed deferral, audit that triple before the second half starts.
metadata:
  type: feedback
---

BUDGET-EFF landed its host/interp half and **deliberately deferred the native
reifier** — disclosed, tracked, and correct. The divergence itself was never
the finding. What I found on `main @ b9c23a6b` is that all three artifacts
surrounding the deferred half had been written for the *completed* version:

1. **The normative doc asserted both.** `effect_v1.rs:2081-2083` said
   `remaining` (**both reifiers**) is `effective_request - get()` — present
   tense, and false, because only the interp one was. It is also exactly the
   sentence an auditor would treat as the invariant's statement of record.
2. **The comment adjacent to the defect argued the defect away.**
   `core.rs:5081` passed the value with `.expect("positioned request bounds
   were narrowed before dispatch")`. "Narrowed" there means
   `narrow_native_int_u64`, an Int→u64 **width** cast — *not* the budget
   clamp. An implementer arriving at the fix site reads it as "already
   effective" and changes nothing.
3. **The test suite could not see it.** Four `budget_eff_capped_*` tests on
   the interp side, **zero** native counterparts — and the binary that would
   host one (`rt_parity_native.rs`) is **skipped in CI**. So a green run on
   the native WP carries no information about the axis at all.

**Why this shape recurs.** A split is decided *before* the work, so whoever
writes the doc, the comment, or the AC is describing the design — which is
whole. The deferral is a schedule fact, and schedule facts don't survive into
prose the way design facts do. Nobody is careless; the artifacts are simply
authored at the granularity of the intent rather than of what landed.

**Why it is dangerous specifically for the second half.** Each of the three
independently *removes* a reason to look. The doc says done, so no audit; the
comment says fine, so no edit; the suite is green, so no failure. The half
that most needs guard rails is the one whose guard rails all point the other
way — and the person who will meet them is the one implementing it, alone.

## What to do

On **any** disclosed deferral, before the second half starts, check the
triple and say which of the three is currently lying:

- **doc/comment** — grep the invariant's prose for universals ("both", "every",
  "always") and test each against what actually landed;
- **the fix site's neighbours** — read the comment a future implementer will
  read *at that line*, and ask whether it argues for or against the change;
- **the oracle** — not "is there a test?" but *would a test on this axis run,
  and would it have failed before?* A skipped suite is worse than a missing
  one, because the missing one is visible.

**File it as an input, not a defect report.** Timing is the whole value: the
same three facts delivered after the second half merges are a post-mortem;
delivered before it starts they are the acceptance criterion. Related:
[[a-conjunction-finding-gets-silently-decomposed]] (the AC keyed on one
construct and lost the conjunction) and
[[a-repro-is-evidence-not-a-completion-oracle]].
