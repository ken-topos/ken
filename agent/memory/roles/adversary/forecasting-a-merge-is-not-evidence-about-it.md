---
name: forecasting-a-merge-is-not-evidence-about-it
description: A rebase forecast read off a snapshot is a claim about content a still-in-flight commit can invalidate; only structural disjointness (path sets) survives.
metadata:
  type: feedback
---

I told the RT-SPLIT ring a pending rebase "should be trivial or automatic since
the modified lines are disjoint." It was not — git conflicted, and
`runtime-implementer` resolved it by hand. Their correction is the rule:
**a prediction about a merge isn't evidence about a merge.**

What I actually checked was true: both branches touched
`cranelift_backend.rs:33-42`, and the lines each one *modified* were different.
The defect was that **slice 4 was still in flight**, and it went on to add
`use lowering::core::*;` into that exact block. My window was the file *as it
existed*; my claim was about the file *as it would exist*.

**Why:** this is the same generator as the whole
`verify-the-mechanism-not-a-proxy` family — an instrument scoped narrower than
the claim it certifies — but on the **time** axis rather than the scope axis, so
the usual "did I enumerate the whole namespace?" check does not catch it. The
cost was not the wrong guess; it was that "trivial/disjoint" propagated into the
ring as the default reading, and *identity-only* is a materially lighter review
posture than *hand-resolved three-way merge*. A forecast from an observer seat
gets read as a finding.

**How to apply:**

- **Do not forecast a merge at all while either side is unlanded.** If asked,
  say what is structurally true and label the rest as unchecked.
- **Distinguish structural from snapshot claims** — this is the reusable part:
  - **Path-set disjointness is structural.** Two commits touching disjoint files
    cannot conflict *regardless of what either contains*. Safe to state.
  - **Line-disjointness inside a shared block is a content claim.** It is only
    as current as the moment you read it, and any in-flight commit on either
    side can invalidate it. Never state it as a prediction.
- **When a hand-resolved merge does land, verify it whole-file, not hunk-scoped**
  (`[[line-closure-check-must-compare-whole-files-not-diff-hunks]]`): a line can
  vanish from *outside* both conflict markers, and a diff re-pairs the retained
  code around it so the hunk reads clean. Line-count closure plus the intended
  net delta is the check that catches it.
- A hand resolution is **new content**, not a replayed patch. It earns an
  original-edit review, not a rebase-succeeded nod.

Related: [[a-repro-is-evidence-not-a-completion-oracle]] (same distinction
between a thing that *suggests* and a thing that *establishes*),
[[auditing-conformance-silently-ratifies-the-artifact]].
