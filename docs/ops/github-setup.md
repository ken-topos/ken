# GitHub setup

How `ken-topos/ken` (public OSS) is wired for the **single-publisher** git model
(`../program/04-git-and-integration.md`,
`../adr/0003-credential-free-publisher.md`). The whole fleet runs in one shared
clone; **agents do local git only** and the **Integrator** is the sole GitHub
identity. So the GitHub-side setup is small: one credentialed identity + branch
protection. The larger per-team-identity apparatus is **deferred** (see
"Graduation" below).

## Identity model: one publisher

Exactly **one** GitHub identity has credentials in the environment — the
**publisher**, operated by the Integrator. Everything else
(build/spec/federation agents) commits locally in worktrees and never
authenticates to GitHub.

Pick **one** of:

- **A `ken-ci` GitHub App** installed on the repo (preferred): installation
  tokens are short-lived and auto-rotating — safer on a public repo than a
  long-lived PAT. Permissions (repo): Contents RW, Pull requests RW, Checks RW,
  Actions read, Metadata read. (Not Administration.) The Integrator agent
  authenticates as the App installation.
- **A single machine-user account** `ken-integrator` (gmail `+tag`, 2FA, one
  fine-grained PAT scoped to this repo), held by the Integrator's harness.
  Simpler to stand up; rotate the PAT periodically.

Either way it is the **only** secret in the fleet. The Integrator uses it to
push `wp/<ID>` branches, read CI (`gh pr checks`), merge, and fetch `main`.

Per-team **attribution** without per-team accounts: encode the team in the
branch (`wp/<WP-ID>-<slug>`), a `team:<name>` label, and a `Team:` commit
trailer. Dashboards and the path-guard key off these, not the GitHub author.

## Branch protection / ruleset on `main`

- Require a PR; **block direct push, force-push, deletion**; **linear history**.
- **Required status checks** (from `.github/workflows/ci.yml`): `build + test`,
  `conformance suite`, `clean-room provenance check`, `path-guard`. This is what
  keeps the offloaded CI a **pre-merge gate** — a branch can't merge until its
  checks are green.
- **Restrict who can push and merge** to the **publisher** identity. This — not
  CODEOWNERS — is what makes the Integrator the sole merger. (CODEOWNERS is
  inert in this model; review is a mootup Decision, see `04`.)
- **Squash-only** merges.
- A merge queue is **not** needed: one publisher serialises merges by
  construction. (It returns at graduation, below.)

## CI

`.github/workflows/ci.yml` runs on the publisher's `wp/<ID>` push (via the PR it
opens) and on `push` to `main`. The heavy jobs (full `--workspace` build, the
conformance suite) run on **GitHub-hosted runners** — this is the compute
offload that keeps the 8-core laptop free (`compute-budget.md` lever 4).
`concurrency` (cancel-in-progress) drops superseded runs. Once the conformance
suite is heavy (F2+), consider a fast subset on branch pushes and the full run
before merge.

## Status checklist (do these to go live)

- [ ] Create the publisher identity — either the `ken-ci` App (install on
  `ken-topos/ken`) **or** the `ken-integrator` machine-user (+tag, 2FA, PAT).
- [ ] Point the Integrator agent's harness at the publisher token.
- [ ] Configure branch protection / ruleset as above (required checks +
  push/merge restricted to the publisher + linear history + squash).
- [ ] Confirm `.github/workflows/ci.yml` runs and its check names match the
  required-status-check list.

> Do **not** create per-team accounts now. They belong to the graduation below.

## Graduation: per-team identities + GitHub-PR review

When **external contributors** arrive or **per-team GitHub review** becomes
load-bearing, move review from mootup Decisions onto GitHub PRs. That step adds:
per-team machine-user accounts, CODEOWNERS-driven review routing, a merge queue,
and a draft→ready automation. The full runbook for that is
`runbook-gh-identities.md` — **deferred until then** (ADR 0003). The
local-worktree discipline (Layer A in `04 §1`) is unchanged across the
graduation.
