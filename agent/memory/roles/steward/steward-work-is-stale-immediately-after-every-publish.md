---
scope: roles/steward
audience: (see scope README)
source: 2026-07-25, hit twice in one session after two consecutive publishes
---

# `steward/work` is stale the instant a publish merges, every single time

The scripted publisher squash-merges, so `origin/main` gains a **new** SHA
that is not a descendant of the pushed tip, and GitHub **auto-deletes the
merged head branch**. Both halves bite:

- `git merge-base --is-ancestor origin/main HEAD` → **fails** on the next
  commit.
- `origin/steward/work` becomes a **stale local tracking ref pointing at a
  deleted remote branch**, so the publisher's `--force-with-lease` fails with
  `! [rejected] (stale info)` — which reads like a permissions or race
  problem and is neither.

**Why it matters:** hit twice in one session, and the first was
mis-attributed. Worse, a stale base is the
[[stale-base-candidate-silently-reverts-everything-landed-since]] hazard —
the publisher's clean-merge gate caught it the second time, but that gate is
a backstop, not a plan.

**How to apply — immediately after every publish, before authoring anything
new:**

```sh
git fetch --prune origin        # --prune is what clears the deleted-branch ref
git reset --hard origin/main    # only safe when the worktree is clean
```

If commits were already made on the stale base, re-root instead of
rebasing: preserve the tip (`git branch -f preserved/... <tip>`), compute
`git diff --name-only origin/main <tip>`, `git reset --hard origin/main`,
then `git checkout <tip> -- <those files>` and re-commit. Verify with `git
diff --quiet <tip>` that content is unchanged. ⚠ Never `git stash` here
(the stash stack is shared across every worktree in this repo).

The re-root is safe **only** because the publish already landed the
identical content — confirm with `git diff --quiet <published-tip>
origin/main` first, and check sibling survival (`git diff --quiet
<approved-sha> origin/main -- crates/`) whenever the batch was cut from a
branch lacking someone else's code.

## 2026-08-05 — the cleanliness guard did NOT save me, and the check's shape is why

I ran the re-root **proactively**, to pre-empt the stale-base publisher gate
before it fired — and destroyed two uncommitted frame edits doing it.

**The guard "only safe when the worktree is clean" was already written here and
I had read it.** What defeated it was *how I verified*:

```sh
git reset --hard origin/main -q; echo "dirty: $(git status --porcelain | wc -l)"
```

⇒ **The cleanliness check ran AFTER the destructive command, in the same
compound command.** It printed `dirty: 0` — and that is exactly what it prints
whether the tree was clean beforehand or the reset had just made it so. **A
post-hoc clean reading is not evidence; it is the reset's own signature**, and
it reads as confirmation at the moment you most want reassurance.

**How to apply:**

- **Check dirtiness in a SEPARATE call that completes before you type the
  reset.** Never in the same compound command, and never after it.
- **Commit first, then re-root.** The correct order for continuing after a
  squash merge is `commit` → `fetch --prune` → `reset --hard origin/main` →
  `cherry-pick <your commit>`. Committing costs nothing and makes the reset
  non-destructive by construction, which beats remembering to check.
- **The proactive case is the dangerous one.** Running the re-root *because the
  gate failed* means your work is already committed (that is what was rejected).
  Running it *to prevent* the gate failing catches you mid-edit — same command,
  opposite safety.

Sibling of [[handoff-gate-hard-reset-destroys-uncommitted-seat-work]] and
[[a-process-count-matches-your-own-shell-when-the-command-embeds-the-path]] —
both are instruments whose reading is contaminated by the act of taking it.
