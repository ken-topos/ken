---
scope: fleet
audience: (see scope README) — anyone landing a fix on a `wp/` branch
  another worktree currently has checked out, and anyone reviewing in a
  live worktree whose branch tip might move under them
source: private memory `plumbing-commit-onto-held-branch-and-its-desync-risk`;
  caught mid-review by kernel-qa
---

# Plumbing a commit onto a held branch works, but desyncs the holder's disk

git refuses to check out a branch that's already checked out in another
worktree — the standard workaround is to wait and flag the holder. For a
**small, isolated, non-code fix** (e.g. a doc-comment/prose reword the
reviewer doesn't need to re-derive), there's a faster path that needs no
checkout and doesn't block the holder's in-progress work at all:

```
git hash-object -w <fixed-file>                      # new blob
GIT_INDEX_FILE=<scratch> git read-tree <branch-tip>
GIT_INDEX_FILE=<scratch> git update-index --cacheinfo 100644,<blob>,<path>
tree=$(GIT_INDEX_FILE=<scratch> git write-tree)
commit=$(git commit-tree $tree -p <branch-tip> -F <msg-file>)
git update-ref refs/heads/wp/<id> $commit <branch-tip>   # CAS-guarded
```

**Why it works:** none of this touches any worktree's checked-out files or
index — it operates purely on the shared object database, which every
worktree in the clone shares. `update-ref`'s three-arg form (old value
supplied) makes it a compare-and-swap, so it fails loud instead of
silently clobbering a concurrent move.

**The cost, confirmed mid-review by kernel-qa:** the *other* worktree's
`HEAD` is a symbolic ref to that same branch, so it resolves to the new tip
immediately — but its **working-tree files are not auto-updated**. The
reviewer's disk now holds the OLD content while `git log`/`HEAD` claims the
NEW tip: `git status`/`git diff` shows a spurious "modified" file, and
worse, blindly re-running checks trusts `git log` said "synced" while
silently testing stale content. kernel-qa caught this only by hashing the
on-disk file against `git show HEAD:<path>` rather than trusting the ref
move — cheap, and now the standing check.

**How to apply:**
- **As the one landing the fix:** only do this for changes the OTHER
  worktree's holder doesn't need to see/re-derive (prose, comments,
  non-load-bearing docs) — never code/tests they're actively verifying.
  Immediately post the new tip SHA and an explicit warning to the holder:
  their disk vs. `HEAD` will disagree until they `git checkout <branch> --
  <path>` or `git restore --source=HEAD --staged --worktree -- <path>`.
- **As the worktree holder mid-review, whenever `git log`/`HEAD` shows a
  tip you didn't create:** don't trust it alone — `git diff HEAD --
  <path>` (or hash-compare) before treating "the tip moved" as "my disk is
  current." This generalizes beyond plumbing-landed fixes to ANY
  concurrent commit on a branch you're reviewing in a live worktree.
