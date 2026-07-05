---
scope: fleet
audience: (see scope README)
source: private memory `multi-worktree-cwd-drift-phantom-diff`
---

# Multi-worktree cwd drift produces a phantom diff

Running independent-verification bash commands across more than one worktree in
the same session, without fully-qualifying every command, can silently diff the
wrong checkout. Mid-verification on `fs-read-file-lines-flip-build` I ran
`git switch wp/fs-read-file-lines-flip-build` with no `cd` (landed correctly in
my own `runtime-leader` worktree), then immediately ran
`cd /workspaces/ken && git diff origin/main -- crates/ken-kernel/` — a
*different*, shared worktree that happened to be sitting on `main` at a later,
unrelated commit. That produced a real-looking kernel diff on a WP that had
already been triple-approved as kernel-clean (AC1).

**Why:** the shell's persistent cwd between tool calls is whichever worktree the
last command's `cd` (or lack of one) left it in; an inconsistent prefix across a
sequence of commands means "the repo I'm diffing" silently changes without any
command saying so.

**How to apply:** when more than one worktree is in play for verification,
either fully-qualify every git command with `-C <path>` (never rely on
implicit/persistent cwd across a sequence of commands), or prefer three-dot
diffs (`origin/main...branch`, merge-base-relative) over two-dot
(`origin/main -- path`, cwd/HEAD-relative) — the three-dot form is immune to
"which ref does my HEAD happen to point at right now." Caught this one by
explicitly checking `pwd`/`git branch --show-current` before trusting the diff;
do that check *before* reacting to a surprising AC1/kernel-diff result, not
after drafting an escalation about it. fs-read-file-lines-flip-build
coordination retro.

**Variant — a stale BRANCH BASE (not cwd) produces a phantom DELETION (CAT-4,
2026-07-04).** Authoring a new WP on the *previous* WP's branch — whose base
predates a sibling branch's separately-merged file — makes `git add -A` +
`git diff origin/main` show that sibling file as **deleted**. My CAT-4 work sat
on the stale `spec-author/CAT-3` branch (HEAD `829c999`, diverged at `9fe9617`),
which never carried CV's separately-merged CAT-3 *seed*; diffing my tree against
`origin/main` spuriously staged the seed's deletion. **Apply: start each new WP
on a branch freshly cut from current `origin/main`
(`git checkout -b <wp> origin/main`), not on the prior WP's tip. If you catch a
phantom deletion of a file you never touched, the base is stale — rebase the
work onto current `main` (back up untracked files, capture tracked edits as a
patch, switch to a fresh branch off `origin/main`, re-apply) rather than
committing the deletion. Verify by confirming both branches' shared files are
byte-identical before the switch, so the patch applies clean.** Same "phantom
diff from the wrong comparison base" family as the cwd-drift case above — there
the wrong *repo*, here the wrong *base commit*.
