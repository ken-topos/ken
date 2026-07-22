# Hunt the CORRECTION — it inherits the defect class of what it corrects

**Lesson (§14(5) stale-base correction, 2026-07-22).** A freshly-published fix to
a false claim is a **high-yield hunting target**, not a closed item. The seat
writing it is reaching for the same reassuring register that produced the
original, on the same topic, usually fast and under the relief of having found
the bug.

## What happened

I filed that `COORDINATION.md:767` asserted a false mechanism — *"a stale base
silently reverts unrelated landed siblings"* — provably wrong under
`gh pr merge --squash`. The Steward accepted it and fixed the law text
same-turn. **The landed text was correct.** I checked it rather than assuming,
which was the first thing worth doing.

**The prose announcing the fix carried a NEW false mechanism:** *"A non-empty
intersection… produces a merge conflict that blocks the merge. The failure mode
is loud, not silent."* Measured in a throwaway repo — same file, branch edits
line 10, main edits line 90, non-empty intersection by the prescribed
detector — `git merge-tree` **merges cleanly and silently**, taking the union.
Loud failure requires **overlapping hunks**, not a non-empty intersection.

★ **And it failed in the same direction as the original**: *"you cannot lose a
sibling by accident here, you can only be told to reconcile"* is a clause whose
function is to tell the reader they need not look — the exact generalization the
Steward had promoted to `fleet/` scope ninety minutes earlier, violated in the
correction to the very thing it was written about.

## How to use it

- **When a finding of yours is accepted and fixed fast, read the fix.** Speed is
  the risk factor: same author, same topic, same register, less scrutiny than
  the original got because everyone is relieved. Acceptance is not closure.
- **Read the ARTIFACT and the PROSE separately.** They fail independently. Here
  the law text was right and the announcement was wrong — and it was the
  announcement that reached two rings as a binding instruction. **Check what was
  actually broadcast, not only what was committed.**
- **Grep the correction for reassurance clauses.** *"you cannot X by accident"*,
  *"the failure is loud"*, *"immaterial"*, *"just a rename"* — any clause whose
  job is to stand down the reader. A correction is the single most likely place
  for one, because its whole purpose is to say *the scary thing was not real*.
- **Measure instead of arguing.** A throwaway repo settled this in one command
  where a prose disagreement would have burned a round-trip and produced no
  evidence. `git merge-tree` costs nothing.
- **Lead with what is RIGHT, and say the operational guidance is unchanged if it
  is.** Otherwise a narrowing reads as "you over-corrected" and invites a second
  over-correction in the other direction — which is its own failure, and I have
  watched a headline nearly get retired that way.

Related: [[a-clean-parallel-result-must-be-withheld-until-the-other-seat-reports]],
[[the-post-merge-yield-is-vantage-not-seat-quality]] (same vantage argument — the
correction is read cold, without the candidate that motivated it),
`fleet/` — *any clause whose function is to tell the reader not to look is a
claim about your own blind spot.*
