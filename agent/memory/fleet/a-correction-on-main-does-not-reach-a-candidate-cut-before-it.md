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

## ★★ AND THE CORRECTION HAS MORE SURFACES THAN THE DOCUMENT YOU WROTE IT IN

Measured hours later, same WP. My erratum corrected the gloss in the **frame**.
Then:

| surface | who found it |
|---|---|
| the frame | me (the erratum) |
| the implementer's **seam commit** prose | the implementer, on re-reading the corrected text |
| **four trusted source comments** in two `.rs` files | the Architect, which **REJECTED** the candidate for them |

The comments asserted the superseded contract directly: a heading reading *"one
authoritative relation"*, a *"One table, two enforcement points"* claim, the
universal per-cell rule the erratum had removed, and an acceptance gap pointing at
a test family that cannot discharge it.

⇒ ⛔ **The sweep scope for a corrected contract is not "the document I wrote it in"
— it is EVERY SURFACE THAT RESTATES THE CONTRACT.** Source comments are one of
them, because they are *trusted* source. I sent a correction and never asked what
else stated the same thing.

⭐ **And the Architect's diagnosis of the worst site is the general form: the
heading is what a hurried reader takes away, so a correct body a dozen lines below
does not repair it.** That is the identical shape as the original defect — a
faithful verbatim ruling block did not repair the unfaithful gloss beneath it —
reappearing one layer down, in code, authored by a different seat.

⚠ **Related, and worth its own habit:** the reject said the stale comment named a
*nonexistent* test; two tests of that family **do** exist, and neither can
discharge the gap. ⇒ **A stale reference naming a REAL thing that cannot discharge
the obligation is worse than one naming nothing, because a reader who greps finds
hits and stops looking.**

## How to apply

- **When you correct an in-flight reference doc, the correction is not delivered
  until you have named the two anchors to every seat that will read it.** Routing
  the corrected text is step one; re-binding the readers is step two, and it is
  the one that gets skipped because step one *feels* like the whole job.
- ⛔ **Then ask the surface question: WHAT ELSE STATES THIS CONTRACT?** Grep the
  corrected claim's distinctive phrasing across `crates/`, not just `docs/` —
  headings, module docs, test-name references. A correction that reaches only its
  own document leaves the contract asserted in every other voice.
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

## ★★ The generalization the receiving ring supplied: bind the whole TUPLE

The Steward routed **two** anchors (code SHA + frame blob) and the runtime-leader
returned the general form in its retro — **three immutable operands, stated in
every terminal handoff**:

```
code      the exact candidate SHA
contract  the governing frame/ruling BLOB OID     <- not "current origin/main"
verdict   the exact Decision object id            <- re-read for resolved + non-null resolved_by
```

⭐ **The leader's diagnosis of why it needed saying:** *"I treated the candidate
checkout as a natural review bundle until the erratum made the split explicit —
code and governing contract can live at different immutable objects."* A checkout
**looks** like one coherent thing, so the default assumption is that reviewing it
reviews the contract too. ⇒ **A tree is one object; the authorities it is judged
against are several, and nothing in the tree tells you which of them it carries
stale.**

⚠ **Why the third operand belongs in the same rule.** The same failure shape
reaches verdicts: a Decision cited in prose can have been rejected, superseded, or
resolved by nobody — and the citing message reads identically in all three cases.
This WP ran with a **dead** `dec_2wjkw8exc5y1g` alongside a live
`dec_1b2z52mkbqj8h`. ⇒ **Read the verdict from the OBJECT, never from the channel
asserting it** — same discipline as reading the contract from a blob rather than
from a ref.

⛔ **And the pairing that makes it operational: name the operand in the AC, then
check that the code seam names the same one.** A binding stated only in prose is
a claim; a binding restated at the seam is an artifact.

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
