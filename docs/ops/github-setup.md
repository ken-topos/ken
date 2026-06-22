# GitHub setup

How `ken-topos/ken` (public OSS) is wired so the federation's PR workflow
(`../program/04-git-and-integration.md`) is enforced, not just documented. Public repo
⇒ org membership and GitHub-hosted Actions minutes are **free**, so the cost of
identities is *operational* (accounts, 2FA, tokens), not billing — which is why
we minimize the account count.

## Identity model: one App + ~5 accounts

GitHub gives exactly two kinds of actor: **user accounts** and **GitHub Apps**.
Apps cannot be team members or CODEOWNERS, so reviewers/merger must be accounts;
authoring can be the App. We split accordingly.

### The `ken-ci` GitHub App (automation + authoring)

One App installed on the repo. Used for: agent **authoring** (push branches, open
PRs), the **draft→ready** auto-transition, driving the **merge queue**, posting
**status**, and the **path-guard** check. App installation tokens are short-lived
and auto-rotating — safer on a public repo than long-lived PATs on many accounts.

- Permissions (repo): Contents RW, Pull requests RW, Checks RW, Actions read,
  Metadata read. (Not Administration.)
- Because the App is **not** a code owner, App-authored PRs raise no
  self-approval conflict — reviewers are requested normally.
- Per-team **attribution** without per-team accounts: encode the team in the
  branch (`wp/<WP-ID>-<slug>`), a `team:<name>` label, and a `Team:` commit
  trailer. The path-guard and dashboards key off these, not the GitHub author.

### Machine-user accounts (CODEOWNERS-eligible reviewers + the merger)

Create via gmail `+tag` (each `<you>+ken-<role>@gmail.com` is a distinct GitHub
account). Give each 2FA and a fine-grained PAT scoped to this repo; the agent
holds its token in the harness. Minimum set:

| Account | GitHub team | Role |
|---|---|---|
| `ken-architect` | `@ken-topos/architect` | required reviewer on every PR |
| `ken-spec-author` | `@ken-topos/team-spec` | Spec review (also the spec author) |
| `ken-spec-qa` | `@ken-topos/team-spec` | second Spec identity, so a spec PR gets a non-author owner approval |
| `ken-integrator` | `@ken-topos/integration` | **sole merger** (branch-protection restricted) |

Optional, add when the role goes live (each also resolves a single-owner
self-approval case by pairing with the Architect in CODEOWNERS):

| Account | GitHub team | Role |
|---|---|---|
| `ken-steward` | `@ken-topos/steward` | owns `/agent` |
| `ken-librarian` | `@ken-topos/librarian` | owns `/docs` |

> Why two Spec accounts: GitHub forbids self-approval, so a spec PR authored by
> one Spec identity needs the other to satisfy the `team-spec` code-owner review.
> The same logic would force a 2nd account per build team **if** you enabled
> owning-team review — which we don't, to start (the QA ring covers it).

Don't mass-create accounts in one burst (abuse detection). Machine users operated
by a human are allowed by GitHub ToS; each must be a real, 2FA'd account.

## GitHub teams

Create under `ken-topos`: `team-kernel`, `team-verify`, `team-language`,
`team-runtime`, `team-ergo`, `team-foundation`, `team-spec`, plus the federation
teams `architect`, `integration`, `steward`, `librarian`. Only the
reviewer/merger teams need member accounts now (per the table); build-team teams
exist for future per-team review and for dashboards, and may start empty.

## Branch protection / ruleset on `main`

- Require a PR; **block direct push, force-push, deletion**; **linear history**.
- **Required status checks** (from `.github/workflows/ci.yml`): `build + test`,
  `conformance suite`, `clean-room provenance check`, `path-guard`.
- **Require review from Code Owners** (drives Architect-always + Spec-on-its-paths
  via CODEOWNERS); **dismiss stale approvals on new commits**; require
  conversation resolution.
- **Require merge queue**; **squash-only** merges.
- **Restrict who can merge** to `@ken-topos/integration` (the Integrator). This —
  not CODEOWNERS — is what makes the Integrator the sole merger.

## Automation (the `ken-ci` App)

- **Auto-ready:** on `check_suite`/required-checks success for a draft PR, flip it
  to ready-for-review (which fires the CODEOWNERS review requests).
- **Merge queue:** the Integrator agent enqueues an approved+green PR; the queue
  re-runs CI against latest `main` before landing (the `merge_group` trigger in
  `ci.yml`).
- **(Optional) convo bridge:** webhook → convo, to auto-post PR/merge Events and
  open/resolve the merge Decision. Until built, the Integrator posts via the convo
  MCP tools manually; the workflow does not depend on it.

## CI cost note

Public-repo Actions are free, but keep runs lean anyway: `concurrency`
(cancel-in-progress, already set) drops superseded runs; once the conformance
suite is heavy (F2+), consider a fast subset on draft pushes and the **full**
conformance run at **merge-queue** time.

## Status checklist (do these to go live)

- [ ] Create the `ken-ci` GitHub App; install on `ken-topos/ken`.
- [ ] Create the 4 required machine-user accounts (+tag, 2FA, PATs).
- [ ] Create the GitHub teams; add the accounts to their teams.
- [ ] Land CODEOWNERS (done) and confirm it resolves (no unknown-team warnings).
- [ ] Configure branch protection / ruleset as above.
- [ ] Enable the merge queue.
- [ ] Wire the App's auto-ready + merge-queue automation.
- [ ] Point each agent's harness at its token (App token for authors; the role
      PAT for Architect / Spec / Integrator).
