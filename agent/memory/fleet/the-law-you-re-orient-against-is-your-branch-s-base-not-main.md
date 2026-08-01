---
name: the-law-you-re-orient-against-is-your-branch-s-base-not-main
description: "Re-orienting from your own worktree reads COORDINATION as of your branch's base, not as of main — so a directive published since your last rebase is invisible, and nothing about reading the file tells you how stale it is."
scope: fleet
---

# The law you re-orient against is your branch's base, not `main`

**Measured 2026-07-24 (adversary, post-compaction re-orient).** The
SessionStart hook says: read `agent/COORDINATION.md`. Every agent does that
from its **own worktree** — which is its own `<role>/work` branch, sitting on
whatever `origin/main` was when that branch was last rebased. So the file you
read is not the law; it is **a snapshot of the law taken at your branch's
base**, and nothing about reading it tells you how old it is.

That day `bf00f1a9` had published four operator directives that had **never
reached `main`** before. Two of them — `§8a` (Architect and Librarian review in
parallel) and `§10⁻a` (the adversary channel is report-only and scoped to
`crates/` + catalog) — were absent from the copy I re-oriented against. `§10⁻a`
**defines my lane**. Had I not cross-read `origin/main`, I would have hunted two
merges (`agent/`, `docs/program/`) that the current law puts out of scope, and
filed a correct, grounded, entirely impermissible report. The Steward's own
commit message names this exact failure at fleet scale: *"after a power-cycle
restart the whole fleet re-oriented against a COORDINATION missing two of
them."*

⇒ **Re-orient from `origin/main`, not from your checkout.** After `git fetch
origin main`:

```sh
git diff --stat $(git merge-base HEAD origin/main) origin/main -- agent/
git show origin/main:agent/COORDINATION.md | grep -n '^## \|^### '
```

The header listing is the cheap version and it is usually enough: **a section
you have never seen is the signal.** Read those sections from `origin/main`
before acting; only then decide whether a full rebase is worth it.

**The generalization, which is why this sits at fleet scope: an artifact that
governs you is the one artifact you cannot safely read from your own tree.**
Code you are editing *should* be read locally — that is the point of the
worktree. Law, playbooks, and memory are the inverse: they are authored by
someone else, on another branch, and land without touching anything you own, so
**no local signal goes red when your copy of them goes stale.** Same family as
`COORDINATION §7a` (mutable external state is tested at point of use, never
cited) — a stale governing document is mutable external state that happens to
be spelled as a file in your checkout.

**Corollary for anyone publishing law:** a directive that lives only on a local
working branch is not in force, however correct it is. The fleet's compliance is
bounded by what reached `main`, and a restart re-derives every seat's behavior
from exactly that.

Related: [[compact-wiped-memory-reflog-first]],
[[multi-worktree-cwd-drift-phantom-diff]],
[[preventive-findings-are-unfalsifiable-so-keep-them-cheap]].
