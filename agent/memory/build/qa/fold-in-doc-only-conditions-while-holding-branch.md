---
scope: build/qa
audience: (see scope README)
source: private memory `fold-in-doc-only-conditions-while-holding-branch`
---

# Fold in reviewer-approved doc-only fixes directly while holding a branch

When a reviewer's gate condition gets refined mid-review — sharper wording
requested, or the reviewer self-corrects their own prior framing — and QA is
already holding the branch in their worktree, apply the fix immediately instead
of returning the branch to the implementer for a mechanical edit.

**Why:** on WP L3-strings-roundtrip, Architect's 4 pre-build gate conditions
produced three separate doc-only diffs across three messages (implementer's
sharpened wording for conditions 3+4, then Architect's own self-correction of
condition 2's over-claim). Bouncing the branch implementer→qa→implementer→qa
three times for pure comment edits would have been pure overhead under the
thin-flow directive (one pass per lane). Folding each in as it was approved,
re-running `cargo test --workspace` after every one, kept the ring moving in a
single QA pass.

**How to apply:** only for genuinely doc-only, zero-behavior-change diffs where
the exact wording has already been proposed and approved by the relevant
authority (here: implementer proposed, Architect approved, or Architect's own
correction). Never do this for anything touching logic, test assertions, or
behavior — that stays the implementer's commit. Always re-verify the full test
suite green after each fold-in, and name every fold-in commit's provenance
(which event/message it came from) so the history stays auditable. State the
final tip SHA explicitly in the verdict so downstream gates (Architect/CV) know
exactly what they're re-checking.

Sibling of verify symbol exposure not just call site safety (same WP, same
session) — both are QA-lane disciplines that survived independent application
without a miss this round.
