---
scope: roles/steward
audience: (see scope README)
source: 2026-07-30 — `RT-RECURSOR-TRANSPORT` sat on the RELEASABLE FRONTIER
  as unassigned Runtime work for a day while it was in flight
---

# An atomic sibling node needs `active`, not a dependency edge

Two nodes were being built as **one candidate on one branch in one PR** —
their mechanisms turned out to be the same thing, so they could not land
apart. One was correctly `active`. The other was `status: ready` with
`depends_on: []`.

`scripts/gen-progress.sh` computes the frontier as **`ready` AND every
`depends_on` entry merged** — so the in-flight node was listed on the
**RELEASABLE FRONTIER as unassigned work available to release.** Its
correctly-`active` sibling masked it: the tracker looked coherent because the
node the ring talked about was right.

⛔ **The fix is NOT a dependency edge.** A `depends_on` edge encodes *after*.
These are **siblings in one atomic set**, not a sequence — an edge would
misstate the relationship and, worse, would still be false once the shared PR
merged. **`status: active` is the only correct lever**, and it is the one
thing that keeps a node off the frontier.

**How to apply:**

- **`§2c` step 8 flips the node `active` at kickoff. For an atomic set, flip
  EVERY member.** The step reads naturally as one node per kickoff, which is
  exactly how the second one gets missed.
- **Write the atomicity INTO both nodes**, including that `depends_on: []` is
  deliberate, or the next reader "fixes" it. Also record that the branch name
  names only one of the two — whoever publishes must not describe one node's
  code as the other's deliverable.
- **At the merge, both nodes flip `merged` in ONE commit.**
- ⭐ **Audit the frontier against what the rings are actually doing, not
  against the node you last touched.** A stale `ready` is not cosmetic — it
  is the population the next sequencing pass reads, and it invites releasing
  work that is already out.

Sibling of [[a-frame-with-no-tracker-node-is-equally-consistent-with-done]] and
[[a-fully-framed-node-can-be-withheld-from-the-frontier-by-its-own-draft-status]]
— the frontier is computed from frontmatter, so every frontier defect is a
frontmatter defect, and the generator never disagrees with you loudly.
