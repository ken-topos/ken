# ADR 0003 — Agents do local git only; a single publisher is the GitHub gateway

- **Status:** Accepted
- **Date:** 2026-06-22
- **Deciders:** the operator

## Context

The federation runs many agents in **one devcontainer with one clone**, edits
segregated by per-agent **git worktrees** (the model proven in the operator's
other agent system). The open question was the **credentials + merge model**: do
agents each hold GitHub credentials and push their own PRs (per-team identities,
CODEOWNERS-routed review, branch-protection merge) — or does some lighter scheme
apply?

Two forces pulled against each other:

1. **Credential surface.** Per-agent GitHub identities mean ~5+ machine-user
   accounts (2FA, PATs) or a multi-permission App, all set up before any code is
   written — operational weight and a fleet-wide secret-exposure surface.
2. **Compute offload.** The 8-core/16 GB laptop must not run the full-workspace
   build or the conformance suite; that work is **offloaded to GitHub Actions**.
   For a *verified* language we want CI green **before** code reaches `main` (a
   pre-merge gate), not merely a post-merge tripwire.

The other system ran **credential-free**: agents did local git only, the human
pushed to GitHub manually, and the team "merge" was into *local* `main`. That
removes credentials entirely but also removes the pre-merge CI gate (nothing
pushes a branch for CI before the merge), which the pure form would sacrifice.

## Decision

**Agents do local git only; exactly one credentialed "publisher" identity —
operated by the Integrator — is the federation's sole GitHub-network actor.**

- Every build/spec/federation agent works in its own worktree and runs **local
  git only**: commit to a `wp/<ID>` branch, rebase onto the already-fetched
  `origin/main`. No `gh`, no push, no fetch, no token, no PR.
- The **Integrator** holds the only GitHub credentials. It **pushes** the team's
  `wp/<ID>` branch to GitHub (so CI runs **before** the merge — the offloaded
  pre-merge gate), **reads** CI, **merges** under branch protection, and
  **fetches** `main` so the shared ref updates for all worktrees.
- **Review is a mootup Decision, not a GitHub PR approval.** The Architect
  (always) and Spec (on its paths) read the diff from the shared local branch
  and vote the merge Decision. The merge gate is: Decision approved + CI green +
  clean-room green + no gate regression.

## Rationale

1. **Keeps the compute offload as a pre-merge gate.** The heavy build +
   conformance run on GitHub's CPU, *before* the merge, because the publisher
   pushes the branch. The laptop never runs a full-workspace build. (This is
   what pure local-merge would have given up.)
2. **Collapses the credential surface to one.** One identity instead of per-team
   accounts; the whole `runbook-gh-identities.md` apparatus is deferred. Most
   agents stay entirely GitHub-unaware — they know only local git and mootup.
3. **`main` stays mechanically always-green.** Branch protection (required
   checks + merge restricted to the publisher) enforces it, not convention.
4. **Review fits the fleet as it is.** Reviewers are agents without GitHub
   accounts; a mootup Decision is the natural gate, and ken already records
   merge approvals as Decisions (COORDINATION §5).

## Consequences

- **The Integrator is a load-bearing singleton** — the only GitHub identity and
  the only CI-watcher. Its watchdog and liveness backstop matter more (it is the
  middle of the three liveness layers, COORDINATION §13).
- **CODEOWNERS is inert** in this model (GitHub requests reviews only on PRs the
  team would open). It is retained as ownership documentation and a graduation
  artifact; review routing is done by who the leader mentions on the Decision.
- **ADR 0002 (wait idle for CI) still holds:** there *is* a pre-merge CI wait —
  it begins when the Integrator publishes the branch.
- The git model is documented in `../program/04-git-and-integration.md`; the
  single-publisher mechanics in `../ops/github-setup.md`.

## Revisit if (graduation to per-agent PRs)

Move to per-team GitHub identities + CODEOWNERS-routed PR review + a merge queue
(the deferred `../ops/runbook-gh-identities.md`) when **external contributors
arrive** (they need real GitHub PRs) **or** per-team GitHub review becomes
load-bearing. At that point review moves from mootup Decisions onto GitHub PRs;
the local-worktree discipline (Layer A) is unchanged either way.
