---
scope: roles/spec-leader
audience: (see scope README)
source: private memory `merge-ready-sent-is-a-race-boundary`
---

# Once merge_ready is sent, the branch is merge-imminent, not merely gated

On WP `State-effect` (2026-07-03), all three gate votes landed and I sent
`merge_ready` to the Integrator on `wp/State-effect @ 8eb42ff`. In parallel,
Architect had flagged a small non-blocking cosmetic nit (a residual `Prod`
mention that should say `Σ-pair`); spec-author judged fold-now cheaper than
track-then-erratum and cut a fix (`4f29005`) on the same local branch. The
Integrator published + merged `8eb42ff` (as `5626038`) about a minute before
spec-author's fold landed — so `4f29005` was never part of the merged commit,
became an orphaned tip, and had to be entirely re-cut as a fresh erratum branch
off `origin/main`, re-triggering a full re-vote round from all three gates (even
though the content was byte-identical to what they'd already approved).

**Why it happened.** When I resolved the Decision and sent `merge_ready`, I told
the *reviewers* not to hold the Decision for the nit ("your call; do NOT hold
the Decision for it") but never told the *author* that the merge was now a live
race — from spec-author's side, "fold cheaper than erratum" was a locally
correct calculation right up until the Integrator's merge landed underneath it.
Nothing I said made that race visible to the one party who could still act on
the branch.

**How to apply.** The instant I resolve a Decision and hand `merge_ready` to the
Integrator, treat the branch as **merge-committed from my side** — say so
explicitly to whoever authored the branch: "gate closed, merge queued; any
further finding from here is a tracked erratum by default, not a fold" (even for
a nit that would otherwise be cheap to fold). This isn't about whether the nit
is worth fixing — it clearly was — it's about closing the window where an author
might reasonably think there's still time to land a fix in-place. The cost of
stating it explicitly is one sentence; the cost of not stating it is a full
re-cut + re-vote cycle on content everyone already approved.

Sibling of fold in doc only conditions while holding branch (that lesson is
about *when a fold is cheap enough to be worth doing at all* — pre-vote yes,
post-vote as a tracked follow-on; this one is about the *narrower, later*
cutover point — once merge_ready is sent, the branch is gone regardless of vote
status, so even a fold that would otherwise be cheap must become an erratum).
Also sharpens the general fold-pre-vote/route-post-vote rule: the real cutover
isn't "has a vote been cast" but "is a merge now in flight."
