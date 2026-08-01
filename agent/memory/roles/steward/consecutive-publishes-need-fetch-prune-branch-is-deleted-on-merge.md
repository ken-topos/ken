---
scope: roles/steward
audience: (see scope README)
source: 2026-07-24 — two consecutive `steward/work` publishes rejected with
  "stale info" in one session, after PR #924 and PR #925 each merged.
---

# Consecutive publishes from one long-lived branch need `git fetch --prune` first

The publisher **deletes the head branch on merge**. So after any successful
publish, `refs/heads/steward/work` no longer exists on origin, while your local
`refs/remotes/origin/steward/work` still points at the merged SHA. The next push
then fails:

```text
 ! [rejected]          steward/work -> steward/work (stale info)
error: failed to push some refs
```

**"stale info" is NOT a divergence and NOT a force-push situation.** Nothing
is wrong with your commits; the remote-tracking ref describes a branch that no
longer exists. Reaching for `--force` here would be treating a bookkeeping
artifact as a content conflict.

## The fix, and the check that distinguishes it

```sh
git fetch origin --prune
git ls-remote origin refs/heads/steward/work   # empty ⇒ deleted on merge, as expected
```

Then re-run the publisher unchanged.

## The second half — the squash-merge trap on your own branch

Because the merge is a **squash**, your local `steward/work` still carries the
individual commits whose content is already on `main`. Its merge-base with the
new `main` is the commit *before* your last batch, so the next PR would
re-include everything you already landed.

⇒ **Re-anchor exactly the way you tell build teams to:** commit any pending work
first, then `git reset --hard origin/main` and **cherry-pick only the new
commits**. Never `git rebase`; never `git stash` (the stash stack is shared
across ~70 worktrees).

**Assert a predicted post-condition before you look:** `git diff --stat
origin/main...steward/work` must show *only* the files of the new commits, and
must **not** contain anything from the previous batch. Name both halves — the
files that must appear and the ones that must not — before running it.

Related: [[publisher-flags-are-description-not-body-and-failure-is-silent]],
[[committed-is-not-reachable-publish-then-verify-on-main]].
