---
scope: fleet
audience: (see scope README) — anyone binding a reviewer to a candidate (leaders,
  the Steward), anyone reviewing one (QA, adversary), and anyone correcting a
  shared reference doc while work is in flight
source: RT-FNSPLIT-B2V, 2026-07-26. The Steward corrected two of its own reader-aid
  glosses on `RULING R5` (PR #1009) while the candidate was in flight; the
  candidate's base predated the erratum. Caught before QA rebound, by a blob
  check rather than by anyone noticing.
---

# A correction on `main` does not reach a candidate cut before it

⛔ **When you correct a shared reference document — a WP frame, a transcribed
ruling, a playbook — the correction lands on `main` and stops there. Every
in-flight candidate cut before it still carries the old text, and the reviewer who
checks out that candidate reads the corrected-away version.**

## What happened

I landed an ERRATUM replacing two overstated glosses on a transcribed Architect
ruling. The replacement was a real **edit**, not an appended note — verified: the
wrong sentence is absent from `main`. Then:

| frame blob | where | has the ERRATUM? |
|---|---|---|
| `0728ce37` | `origin/main` | ✅ authoritative |
| `61af6841` | base `ee226c5e` **and candidate `898fdb5c`** | ⛔ the superseded gloss, live at `:1248` |

The candidate does not touch the frame, so git carries the old blob forward
**correctly**. ⇒ ⭐ **The merge is clean and the content is stale, and those are not
in tension** — no conflict, no warning, nothing to notice.

## ★★ THE ASYMMETRY IS THE WHOLE LESSON

The **implementer** was fine — it had been *told* the erratum landed and re-read
`main`. It said so explicitly: *"both were found by re-reading the corrected text
rather than by acting on the version in my context."* It even found a second thing
it had built wrong, from that re-read.

⛔ **QA had no such prompt.** Checking out the approved SHA and reading the frame
from that tree is the **normal, correct** reviewer behaviour — and it silently
yields the text that was corrected away. **Nothing in the tree looks old.** The
seat doing exactly the right thing is the seat that gets the stale document.

⇒ So the failure is not carelessness and cannot be fixed by care. **A "the erratum
is authoritative as of `main=X`" note in the routing message does not fix it
either**, because the reader's own tree disagrees with the note and the tree is
what they open. The note is a claim; the checkout is an artifact.

## The rule: bind TWO anchors, and bind them BY BLOB

**A review instruction must name both operands separately:**

```
code   at the candidate SHA        898fdb5c
frame  at blob                    0728ce37   (docs/program/wp/<WP>.md)
```

⭐ **And name the OID, not "current `origin/main`" — measured within the hour.**
The leader bound QA by blob. Twenty minutes later I published an unrelated doc-only
PR and `main` moved. Because the binding named `0728ce37`, the move was a
**non-event**: the blob was identical at the old main, the new main, and
`origin/main`, so nothing had to be re-derived. Had the instruction said *"the
frame at current `main`"*, that phrase would now point somewhere new and QA would
have had to work out whether it mattered.

⇒ ⛔ **A binding phrased against a moving ref is invalidated by any publish; a
binding phrased against a blob survives a change to the very ref it was written
against.** This is the same discipline as verifying a landing by blob identity
rather than by ancestry, applied to *instructions* instead of *evidence*.

## How to apply

- **When you correct an in-flight reference doc, the correction is not delivered
  until you have named the two anchors to every seat that will read it.** Routing
  the corrected text is step one; re-binding the readers is step two, and it is
  the one that gets skipped because step one *feels* like the whole job.
- **Check it mechanically rather than remembering whether the base predates the
  fix:** `git rev-parse <candidate>:<doc>` vs `git rev-parse origin/main:<doc>`.
  Different OIDs ⇒ the candidate's reader will see the old text.
- ⛔ **Confirm the correction was an EDIT, with a negative control.** Grep the
  superseded phrase at `main` (must be **absent**) *and* at the candidate (must be
  **present**). Absent at both means your needle is wrong, not that you are safe.
- ⚠ **A visible ERRATUM box beats a silent prose swap** — anyone who already read
  the old version needs to know it moved. But the box lives on `main` too, so it
  does not solve the in-flight problem either. It is honesty for future readers,
  not delivery to current ones.

## Positioning

- **The diff-claim sibling:** *state a claim about a change against the anchor your
  reader actually holds, not the one you hold.* Same family, but that one is about
  a **diff**; this is about the reader's **working copy of a document**, which they
  did not choose and cannot see is stale.
- **The appended-correction sibling:** *a later note saying a deliverable is false
  does not replace the deliverable* — the superseded text stays operative and is
  the copy positioned to be obeyed. ⭐ **This case is sharper: the replacement
  genuinely happened, correctly, and still did not reach the reader.** Doing the
  correction properly is necessary and not sufficient.
- [[a-scope-exclusion-bounds-edits-not-verification]] — the sibling shape: there a
  *constant* went stale under a re-anchor; here a *document* goes stale under a
  base that never moved.
- [[the-law-you-re-orient-against-is-your-branch-s-base-not-main]] — the general
  form this is an instance of.
- [[live-review-candidate-goes-stale-reanchor-sha]] — the code-side counterpart.
