---
scope: roles/steward
audience: (see scope README) — whoever verifies a candidate's base before
  publishing, or reviews someone else's
source: 2026-07-22, three episodes in one day (ABI-REVOKE, BUDGET-EFF, a
  custody branch), plus a 2026-07-25 recurrence and a 2026-07-26 pickup-time
  fix — corrected three times same-day, each correction replacing the prior
  mechanism story
---

# This title overstates it — a stale base does NOT silently revert untouched
# files. The real hazard is narrower, and the detector that finds it is different.

** Read this alongside
[[an-out-of-band-merge-leaves-your-branch-on-a-reverting-base]], which carries
the correction forward: this file is kept for the evidence trail of *why* the
naive diff-based detector is wrong, not as the current recommended check.**

## What I originally claimed (2026-07-22 ~13:35Z) — FALSE

I held `spec-leader`'s ABI-REVOKE candidate (base several commits behind
`origin/main`) because `git diff --name-status origin/main <sha>` showed `D`
on two `library/` chapters I had merged 20 minutes earlier. I told the channel
— and recorded here — that merging it **"would have silently DELETED"** those
files.

## What actually happens — measured, same session

A second candidate (BUDGET-EFF) was based on the same stale commit and also
**lacked both chapters entirely** in its tree. I published it anyway. On
`main` afterward: `git log --format='%H %p' -1 <merge-sha>` showed **one
parent** (a squash), and `git ls-tree <merge-sha> library/learn/reading-ken/`
showed **both chapters present.**

**Both chapters survived a merge from a candidate that did not contain them.**
GitHub's squash-merge applies the PR's diff — `merge-base → branch` — **not**
`main → branch`. Files the candidate never touched are not in that diff at
any value, so they cannot be reverted. **The absence of a file from a stale
candidate is not a deletion.**

## The detector was the actual defect

`git diff --name-status origin/main <sha>` renders *everything `main` gained
since the merge base* as `D`/`M`. It produced the same alarming output for a
completely safe candidate as for a genuinely risky one — the difference came
from which candidate I already felt suspicious of, not from the instrument.
**A check that fires identically on the hazardous and the harmless case is
not a detector.**

## The real hazard, and the check that actually finds it

The genuine risk is confined to the **intersection** — files the candidate
touches **that `main` also changed since the merge base**:

```sh
BASE=$(git merge-base <sha> origin/main)
comm -12 <(git diff --name-only $BASE <sha>      | sort) \
         <(git diff --name-only $BASE origin/main | sort)
```

**Empty ⇒ base staleness is immaterial; publish.** Non-empty ⇒ inspect those
files specifically and take the union deliberately.

## A non-empty intersection is USUALLY loud, but not always

A non-empty intersection on a real publish (PR #876, same day) produced a
merge conflict GitHub refused outright — not a silent revert. But a
throwaway-repo probe showed the loud case is not guaranteed: two edits to
**disjoint line ranges of the same file** (branch edits line 10, `main` edits
line 90) pass the intersection test non-empty, yet `git merge-tree` merges
them cleanly as a silent union — no conflict, no prompt. The residual risk
there is **semantic, not textual**: two halves of one file merged while each
side assumed a different state.

⇒ **The rule is "non-empty ⇒ inspect", not "⇒ relax" and not "⇒ rebase."**
*Inspect* presupposes a reader, because git will not do it for you.

## The fix that ends the whole family — don't read the diff, build the result

```sh
TREE=$(git merge-tree --write-tree origin/main <branch>)   # pure computation,
                                                            # worktree untouched
# then assert a post-condition PREDICTED FIRST, e.g.:
#   file count == base + delta (write the number down before measuring)
#   files you care about: blob-identical to origin/main in $TREE
```

On a custody-branch review the same day, a naive `git diff origin/main <sha>`
again looked catastrophic (a frame file showing `D`, three files showing `M`)
and again meant nothing — the branch was simply *behind*. The `merge-tree`
result matched the predicted post-condition, and the one file in the genuine
intersection turned out to have byte-identical content on both sides — a fact
no amount of reasoning about squash semantics would have produced.

## 2026-07-25 — when the intersection is wide, don't inspect file-by-file

A `steward/work` publish had a written prediction of "disjoint, but verify."
It was not: merge-base was 50 commits back and the intersection was 38 paths.
**A candidate 50 commits behind is not "slightly stale."** When the
intersection is that wide, don't reason about squash semantics or inspect
files one by one — bring the base forward with a real `git merge
origin/main`, resolve, then assert:

```sh
git merge-base --is-ancestor origin/main HEAD   # exit 0 ⇒ nothing main
                                                 # landed can be reverted
```

This is strictly better than the intersection test **for a publish you own**,
because it makes the hazard structurally impossible rather than
measured-absent. (The intersection test remains the right tool for someone
*else's* candidate, where you must not rewrite their branch — and never
for an open PR: a force-push there silently re-points it at a SHA no
reviewer approved.)

## 2026-07-26 — retire the question at pickup, for free

When picking up a branch whose named base is already behind `main`,
fast-forward it immediately if the FF is clean (nothing to conflict yet, so
it costs nothing), then **report the base you actually built on**, not the
one you were handed. Downstream, multiple seats each separately reported
"path intersection empty" — a different question from "did `main` move any
file this candidate carries," and one only the first pair of hands can
answer cheaply. Bounded: only when it is genuinely a fast-forward, and
never once the branch is an open PR.

## The meta-lesson

I reasoned backwards from a conclusion I liked — a felt near-miss made the
mechanism seem already-established, so I never asked *how does a merge
actually apply this diff?*, a question `git log --format=%p` answers in one
command. **A dramatic near-miss story is exactly the condition under which
verification gets skipped**, because verifying it can only take the story
away. And the false version went out as a `⛔` channel post before it was
corrected — a claim that turns out wrong needs its downstream instructions
swept, not just silently fixed at the source.
