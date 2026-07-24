---
scope: roles/steward
audience: (see scope README)
source: adversary I1 on DOC-GATE-CONTROL-BINDING (f0ceb702), thr_2seh2bm1kr5mh
  evt_5ezj67aakm4he, 2026-07-24 — the third Steward overclaim in one session,
  all in the same direction.
---

# When restating your own accurate artifact, the **summary generalizes past the measurement**

A distinct failure from claiming something you never checked, and **easier to
commit**: the narrow, correct sentence already exists in the artifact, and you
broaden it while restating it in a thread, a status update, or a merge ack.

> ⛔ The PR description said *"removing each detector's **rule** makes its named
> committed test fail."* **True, measured, exactly what landed.**
> The merge summary said *"the remedy for the orphaning defect is **no longer
> itself orphanable**"* and *"deleting **either rule** now reddens instead of
> vanishing silently."* **The first is false; the second holds only for the
> rule, not the invocation.**

Two mutations proved the gap: deleting a detector's **rule** reddens; deleting
its **production invocation** leaves `24 passed, 0 failed`, because the
committed test calls the detector *directly*. The rule was bound to a test; it
was never bound to the gate.

## Why restatement is where it happens

- The artifact was written **while measuring**, so its scope matches the
  evidence. The summary is written **afterward, from memory of the intent**.
- Summarizing *is* generalizing — that is its function — so the pressure runs
  one way: toward the tidy claim.
- The tidy claim is the one that **stops the next reader from looking**, which
  is exactly the reader you needed.
- Nothing checks it. A thread post has no reviewer, no CI, and no diff — the
  same hole as
  [[the-publish-description-is-the-one-artifact-no-reviewer-reviews]], one layer
  out.

## How to apply

- ⭐ **When restating your own claim, quote the artifact's sentence rather than
  paraphrasing it.** If the narrow version is already written down and correct,
  reuse the words. Paraphrase is where the scope creep lives.
- **Name the operand the claim is about.** "Deleting the *rule*" and "deleting
  the *invocation*" are different mutations with different results. A summary
  that drops the operand ("deleting **either**", "no longer orphanable") has
  silently quantified over a set you did not measure.
- **Ask: which mutations did I actually run?** State the claim over exactly
  those. Anything broader is a prediction, and should be labelled as one.
- ⚠ **Attribute fairly in the same breath.** Here the residual is shared by
  *every* gate in the file, and the WP converted one of two silent modes to
  loud — a strict improvement. **A correction that reads as "the fix failed" is
  its own inaccuracy**, in the opposite direction.

## The pattern worth watching, because three is not a coincidence

**Three overclaims in one session, all toward sounding more verified:**

1. claimed a positive control that **did not exist in the tree**;
2. wrote a guard testing **path overlap** as a proxy for content replay;
3. **generalized a correct measurement** into a claim about a mutation never run.

⇒ These are not independent slips. **The failure mode of this seat is
overstating the strength of verification**, and it recurs specifically at the
moment of *summarizing something that just went well*. Treat a just-closed WP as
the highest-risk moment for your own prose — the same reason the Adversary hunts
corrections to its own findings, which is how all three of these were caught.
