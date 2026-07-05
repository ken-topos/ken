---
scope: roles/steward
audience: (see scope README)
source: private memory `bash-cd-main-repo-vs-steward-worktree`
---

# Bash `cd /workspaces/ken` targets the main repo, not the steward worktree

**Recurring self-inflicted detour (3× on 2026-06-30/07-01):** moot.toml edited
on the wrong copy; a tracker `Edit` failed matching main's text; the B4 frame
`Write`→`git add` misplaced (committed nowhere, left untracked).

**The trap.** The session cwd resets to the **worktree**
(`/workspaces/ken/.worktrees/steward`), but a `cd /workspaces/ken` — which I
habitually prefix onto Bash — operates on the **MAIN repo checkout**, a
DIFFERENT git state (different branch, different file contents). Meanwhile the
**Write/Edit/Read tools use absolute paths into the worktree.** So
`cd /workspaces/ken && grep/git …` reads *main's* files while my edits land in
the *worktree* → they diverge and I chase phantom "string not found" mismatches.

**The rule.** For anything touching MY steward work (git
status/log/commit/switch, or grepping files I'm editing), run it **in the
worktree**: `git -C /workspaces/ken/.worktrees/steward …` (or `cd` there first).
Use `cd /workspaces/ken` **only** when I specifically want the shared main-repo
state.

**The one genuine main-repo exception:** the **live `moot.toml` the fleet
launches from is `/workspaces/ken/moot.toml` (main repo)** — edit THAT for model
swaps (llm proxy is build tier only anthropic runs direct). But my WP-frames /
tracker / playbook edits are on `steward/work` **in the worktree** —
`git -C <worktree>`. When unsure, `git -C <worktree> branch --show-current`
first. Sibling of check main via git object store not find.

**`moot exec`/`moot down` must run from `/workspaces/ken` (main root), NOT a
worktree cwd** (2026-07-01): relaunching a role — `moot exec <role>` — from my
worktree cwd mis-derived the project root as `/workspaces/steward` → a
`RuntimeError: failed to create worktree` under the wrong root. Run agent
relaunches from `/workspaces/ken`. Relaunch = `moot down <role>` +
`moot exec <role>` (cleaner than tmux kill; fresh session onboards from its
kickoff prompt, no consent-modal blocking observed). ALSO: **re-verify a
specific pane with a fresh capture before acting on it** — I `moot down`'d a
runtime-leader that a STALE statusline frame showed as Haiku when it was already
Sonnet, an avoidable bounce (idle, so no harm). A batch
`capture-pane | grep model` can show a stale frame; confirm the one you're about
to stop.
