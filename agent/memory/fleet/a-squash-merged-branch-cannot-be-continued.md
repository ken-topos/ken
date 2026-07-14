---
scope: fleet
audience: (see scope README) — anyone who publishes an artifact on a branch and
  then expects someone else to keep working on that same branch
source: LET-3 Phase 2, 2026-07-14 — the Steward published a WP frame from
  `wp/let3-p2-map-acc-lookup`, the frame squash-merged, the Steward's worktree
  stayed parked on the branch, and Team Foundation was hard-blocked from taking
  the WP it had just been kicked
---

# A squash-merged branch cannot be "continued" — and a parked checkout blocks a ring

**Two branches must never share a name across a merge boundary.** If you publish
artifact A on branch `B`, `B` squash-merges, and you then tell someone to
"continue on `B`" — you have pointed them at a **branch that no longer means
what you think.**

## What squash-merge actually leaves behind

- The **remote** branch is auto-deleted on merge. `git ls-remote --heads origin B`
  → **empty**.
- The **local** ref survives, dangling **ahead** of `origin/main`, while **its
  content is already in `main`** under a *different* SHA.
- So `git merge-base --is-ancestor <B's tip> origin/main` says **"not an
  ancestor."** That reads as *unmerged work* and it is **not** — it is a **stale
  leftover**. (This is the squash-merge trap: **branch-ahead ⇏ unmerged.** Verify
  merges by **content**, never by SHA.)

**A team told to "continue" that branch is being handed a dangling ref whose diff
against `main` is meaningless.**

## ★ And the part that actually blocks the ring: one branch, one worktree

**Git will not check out the same branch in two worktrees.** So if you are *still
sitting on* the branch you just published, the receiving team **physically cannot
check it out.** They are hard-blocked — and the block has nothing to do with
their work, their context, or their kickoff.

**The tell is a leader that acks the kickoff and then does not start**, reporting
the branch is held elsewhere. **A leader that holds and asks, instead of forking a
competing branch or resetting yours, is doing the right thing** — that is the
correct escalation, not a stall. Treat it as such and unblock it fast.

## The rule

> **The branch you PUBLISH from and the branch the team BUILDS on are two
> different branches, with two different names.**
>
> - Publisher's branch: `wp/<ID>-frame` — merges, dies, is never continued.
> - Team's branch: `wp/<ID>-<slug>` — cut **fresh from `origin/main`** *after* the
>   frame lands (the artifact is already on `main`; that is the point of merging it).
>
> **And switch your own worktree OFF the published branch the moment you publish
> it.** A publisher parked on a branch is a silent blocker for whoever you handed
> it to.

## Why this hid for so long

The Steward playbook literally said the team *"continues `wp/<ID>-<slug>` for the
implementation"* — the same name the frame was authored and merged on. **That
instruction is self-contradictory after a squash-merge**, and it had survived
because Stewards kept *accidentally* using distinct names (AX-2 got
`wp/ax2-frame` + `wp/ax2-…-build` purely by chance). **The one time the names
actually collided, it blocked a ring within minutes of kickoff.**

**The general shape:** a rule that only works when you happen to do something the
rule never told you to do is **not a working rule** — it is a latent bug with good
luck. When you find one, fix the rule; do not congratulate the luck.

Sibling of [[verify-a-tmux-rouse-actually-submitted]] (delivery ≠ engagement) and
of the squash-merge trap in `steward.md §2c`. Same family: **the handoff is not
complete until the receiver can actually act on it.**
