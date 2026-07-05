---
scope: fleet
audience: (see scope README)
source: private memory `reviewers-review-branches-not-prs`
---

# Every federation agent except the Integrator reviews branches, not PRs

In the Ken federation only the **Integrator** has `gh`/GitHub access. Every
other agent — Architect, QA, leaders — works **local git only** in the shared
object store (one repo, many worktrees). So a reviewer reviews by checking out
the **branch ref + SHA** (`wp/T1-build @ 7d6ed91`), and a review request framed
as a **PR number** (`look at PR #110`) is a **no-op** for it: it has no `gh` to
resolve the PR to anything.

**Why:** observed T1-build (2026-06-30). The operator compacted the Architect
and asked it to "look at PR #110"; the Architect can't reach GitHub, so it sat
wedged at its prompt. The operator pulled the branch from GitHub and handed it
over directly → the Architect reviewed the local code and ✅'d in minutes
(catching the §14 stale-base trap, so it was healthy — just pointed at an
unreachable resource). Compaction + an unreachable-resource framing is what
wedged it.

**How to apply:** nudge any non-Integrator reviewer with the **branch ref +
SHA**, never a PR number — mine, a leader's, or the operator's. The federation's
own merge-Decision flow already does this right (the verify-leader's nudge cited
`wp/T1-build @ 7d6ed91`); the gap is specifically *manual* prompts that borrow
GitHub's PR framing. Corollary of "only the Integrator touches GitHub"
(COORDINATION §14) — candidate sharpening: a reviewer/merge nudge references the
local branch, because the reviewer has no gh. Pairs with the post-compaction
re-orientation discipline (a freshly-compacted reviewer is most fragile to a
malformed handoff). See playbooks state mechanism not intent.
