---
scope: fleet
audience: (see scope README) — anyone writing a doc comment, mechanism
  prose, or an in-code claim about order/bound/complexity; reviewers of
  the same
source: `crates/ken-runtime/src/values.rs` (`Drop` impl / `detach_children`),
  2026-07-24 — same defect made twice in one WP, in an assertion and in a
  doc comment, with opposite outcomes
---

# A mechanism claim in a comment is structurally exempt from execution

In one WP the **same error** was made twice, in two positions, with
opposite outcomes:

- **In an assertion.** `assert_eq!(compound_subvalues, 8)` — the subject
  has **7**. Miscounted by forgetting that `Bytes` and `String` are
  compounds. It **died on its first run, in under a minute, unassisted.**
- **In a doc comment.** `Drop` "dismantles the tree **breadth-first** onto
  an explicit heap stack" — but `detach_children`
  (`crates/ken-runtime/src/values.rs`, grep the symbol) pushes children onto
  a `Vec` and `Vec::pop` takes the most recently pushed, so the walk is
  **LIFO, depth-first**. It passed self-review, passed QA's full mechanism
  audit, and was caught only by the Architect reading the comment against
  the code.

⛔ **Identical defect class — writing what the code was believed to do, not
what it does — and the difference in outcome was not care, attention, or
review quality. It was whether the claim sat somewhere that could be
EXECUTED.**

**Why:** a doc comment on a trusted source is the one place where a
mechanism claim is exempt from every instrument a project owns. It is not
under-tested; it is **untestable in place**. And it is not inert: the
Architect's ruling was that breadth-first and depth-first have **different
live-frontier memory bounds** (a LIFO worklist holds the unvisited siblings
along the active root-to-node path; a FIFO frontier holds a whole level),
so the comment handed the next maintainer the wrong contract to reason
from. This is also why an approve-then-block sequence can have **both
votes defensible**: QA verified everything that runs, and the defect was in
the region where nothing runs.

**How to apply:**
- ⭐ **Name the operation that makes the claim true.** The wrong comment
  asserted an adjective and cited nothing. The corrected one says *`Vec::pop`
  takes the most recently pushed, **therefore** depth-first* — falsifiable
  by anyone in one glance, with no test, no tooling, no reviewer expertise.
  **Adjective-only mechanism prose should read as unsourced**, the way an
  unlabelled number already reads as an estimate.
- **Write mechanism prose FROM the code, not from the intent.** The wrong
  comment came from the picture in the author's head ("flatten the tree
  onto a heap stack") while the code was doing something else. Comment and
  code authored in the same minute is exactly when this happens — the
  intent is louder than the text.
- ⚠ **The trigger is narrow: an order, a bound, or a complexity class.**
  Those are what maintainers reason *from*. This is not a licence to pin
  every comment with a test (absurd) — and specifically **not** "review
  comments harder", because a check that cannot be reliably performed at a
  seat belongs in the artifact's form instead.
- **When put on notice about one such claim, audit the siblings.** Blocked
  on one traversal-order claim, the other two in the same WP (streaming
  pre-order encoder, postorder `Clone`) were re-derived and the diff was
  grepped for further instances. Both held and the one site was the only
  one — which made "only one place was wrong" a *checked* statement instead
  of an assumption.

Related: [[a-rule-far-from-the-point-of-work-does-not-fire]] for the mirror
case (prose that is true but positioned where nothing consults it).
