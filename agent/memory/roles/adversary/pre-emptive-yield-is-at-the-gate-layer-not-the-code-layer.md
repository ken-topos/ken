---
name: pre-emptive-yield-is-at-the-gate-layer-not-the-code-layer
description: When you file into work a competent seat is about to start, the code-level defect is the part they will find themselves in pre-work verification — your durable yield is at the layer the seat cannot unilaterally change: the AC vocabulary, the gate's blind spots, the scope definition. Aim there.
metadata:
  type: feedback
---

I filed a pre-emptive finding on `CB-HYGIENE` minutes before the branch was
cut. It had two kinds of content, and I ranked them wrong.

**What I led with — the code-level blind spot.** `:90-91` is a `pub use`
behind a *default-off* feature whose sole consumer is a test in another
crate, so the WP's own named gate (`ken-cargo test -p ken-runtime`) never
compiles the line, and a rename there is green locally. I thought that was
the headline.

**`runtime-implementer` found it independently, in their own pre-work
verification, before the relay even landed.** Which is what a competent seat
*does* — they read the file they are about to edit.

**What actually only I could contribute** was the layer above: AC #2 (*"no WP
token in any non-test region"*) is **unsatisfiable** under AC #4 (*"move +
comment edit only"*); the required classification vocabulary —
removed/restated/relocated — had **no cell for "left in place"** although the
brief demanded leaves be visible; the two-class taxonomy had no cell for
*production code under a test-support feature gate*; and "non-test region"
was defined by line range rather than cfg-condition. All four were adopted.

**Why the seat does not produce these.** An implementer who hits an
unsatisfiable AC reports a **blocker** — correctly, it's not theirs to
rewrite. They do not redesign the classification vocabulary, because the
brief is an input to them and a *product* to the author. So the AC layer has
exactly one reader positioned to attack it before the work starts, and it
isn't the person doing the work.

⇒ **The heuristic: before filing into imminent work, ask which of my items
survives the assumption that the implementer is careful.** A code-level
defect in the file they are about to open usually does not. A defect in the
gate that will certify their work always does — the gate is the thing nobody
re-derives, because passing it *is* the definition of done.

This is not "don't report the code finding" — reporting it cost a paragraph,
it corroborated their independent read, and corroboration from a second
construction has its own value (cf. verify-qa planting their own variants
rather than re-running the implementer's). It is about **which item to lead
with, and which to spend the grounding effort on**.

Refines [[the-post-merge-yield-is-vantage-not-seat-quality]] — that one says
the yield comes from the vantage rather than the seat; this one names *where*
that vantage still has an edge once a competent seat is actively looking at
the same file. Sibling of
[[a-conjunction-finding-gets-silently-decomposed]] (read the resulting AC as
an adversary reads any gate) and
[[a-disclosed-deferral-gets-guard-rails-written-for-the-whole]].
