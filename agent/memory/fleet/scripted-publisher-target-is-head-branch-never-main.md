---
scope: fleet
audience: (see scope README) — anyone invoking `scripts/scripted-pr-automerge.sh`
source: private memory `scripted-publisher-target-is-head-branch-never-main`;
  incident 2026-07-12, flag surface confusion 2026-07-22
---

# `scripted-pr-automerge.sh --target` is the HEAD branch to push, never `main`

`scripts/scripted-pr-automerge.sh --target X` treats **X as the HEAD
branch it pushes** (grep `resolve_branch` and the `git push
--force-with-lease` call — currency-checked 2026-07-28, still present, no
longer at the old line numbers, so grep rather than cite a line), and
`--base main` is **hardcoded** at the `gh pr create` call. So `--target` is
NEVER `main`. Always pass the **candidate WP branch** (e.g.
`wp/adr0014-fold`), which the script pushes, opens a PR from into main, and
squash-merges.

**The foot-gun (hit 2026-07-12):** `--target main` was passed once.
`resolve_branch` matched a **stale local `main`** (`d26270e2`, never kept
current), and the force-push line force-pushed it over `origin/main`
(`f24519f5`) — a **force-push that regressed the remote backward**,
dropping several landed PRs/ADRs from main's tip. The doc-only merge path
never even ran; the damage was the pre-merge head push. A
`+ f24519f5...d26270e2 main -> main (forced update)` line (note the `+`
and "forced update") is the tell — a healthy doc-only publish of a
candidate never force-updates main.

**Recovery that worked:** the dropped commits still existed in the object
store, and the old remote head (`d26270e2`) was an **ancestor** of the
intended candidate (`7fc800fc`), so restoring was a **pure fast-forward**:
`git update-ref refs/heads/main 7fc800fc` (`main` was checked out in the
root worktree, so `branch -f` is refused — `update-ref` moves the ref
without touching that working tree), then re-running the script `--target
main` so the force-push line pushed the *forward* ref. Output
`d26270e2..7fc800fc main -> main` with **no `+`** = clean ff = restored.
Then reconciling the root worktree: its ref moved but index/worktree were
stale — `git -C <root-worktree> reset --hard <sha>` (NOT `checkout -- .`,
which restores from the stale index and makes it worse), preserving local
untracked config via a scratch backup.

**Rule:** publisher `--target` = the candidate branch, full stop. Before
any publish, `git rev-parse main` vs `origin/main` — if local `main` is
behind, never let it become the push source.

## The exact flag surface — read `usage()`, don't recall it

The script's argument parser has rejected guesses on flag names before
(`--branch`, `--body` — neither exists). The real usage, confirmed current
2026-07-28 (`scripts/scripted-pr-automerge.sh`, `usage()` at the top of the
file):

```
scripts/scripted-pr-automerge.sh \
  --target <sha-or-branch> \
  --title <pr-title> \
  (--description <text> | --description-file <path>) \
  [--doc-only]
```

**There is no `--branch` and no `--body`.** The description is
`--description` / `--description-file`, and for anything multi-paragraph
use `--description-file` — a long `--description` with newlines is a
quoting trap.

`--doc-only` merges immediately; **omit it** whenever the diff touches
`scripts/` or `crates/` (even a docs-motivated fix often does), and the
script then waits ~CI-duration + 10% and polls checks before merging.

**Read the `usage()` block at the top of the script rather than recalling
the flags** — it is short and settles it in one read.
