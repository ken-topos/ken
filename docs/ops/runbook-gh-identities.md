# Runbook: set up the GitHub identities

Operator runbook (for Pat) to stand up the identities the agent workflow needs.
Reference/rationale is `github-setup.md`; this is the **ordered manual procedure**.
Do the phases in order — the ordering avoids lockout and CODEOWNERS dead-ends
(branch protection goes on **last**, after a smoke test).

Notation: replace `<you>` with your gmail local-part; each `<you>+ken-x@gmail.com`
is a distinct GitHub-eligible address that delivers to your inbox. Steps marked
**[manual]** cannot be scripted (GitHub forbids programmatic account creation and
cross-user token minting); **[gh]** steps can use the `gh` CLI as org owner.

Identities to create: **1 GitHub App** + **4 required accounts**
(`ken-architect`, `ken-spec-author`, `ken-spec-qa`, `ken-integrator`).
`ken-steward` / `ken-librarian` are optional — add when those roles go live.
Build teams **author via the App**, so they need no accounts.

---

## Phase 0 — Prereqs (5 min)

- [ ] You are an **owner** of the `ken-topos` org.
- [ ] `gh` CLI installed and authed as your owner account: `gh auth status`.
- [ ] A password manager with **TOTP support** (you'll create 2FA for ~4
      accounts) and a safe place for each account's **recovery codes**.
- [ ] A secret store for tokens + the App private key (NOT the repo).

---

## Phase 1 — Create the machine-user accounts  [manual] (~10 min each)

For each of `ken-architect`, `ken-spec-author`, `ken-spec-qa`, `ken-integrator`:

- [ ] In a **separate browser profile / incognito window** (so sessions don't
      collide), sign up at github.com with email `<you>+ken-<role>@gmail.com`
      and username `ken-<role>`.
- [ ] Verify the email (it lands in your normal inbox).
- [ ] **Enable 2FA immediately** (TOTP). Save the recovery codes to your manager.
      Many orgs require 2FA — an account without it will be blocked from the org.
- [ ] In the profile bio, note it's a machine account operated by you (ToS
      etiquette; machine users are allowed, mass-bot signups are not).

Gotchas:
- Space the signups out a little; rapid-fire account creation can trip abuse
  detection.
- If `+tag` signup is ever rejected, fall back to distinct emails or gmail
  dot-variants (`ken.architect@…`).

---

## Phase 2 — Add the accounts to the org  [gh or manual] (~5 min)

- [ ] Invite each as a **Member** (not Owner):
      `gh api -X POST /orgs/ken-topos/invitations -f email='<you>+ken-<role>@gmail.com' -f role='direct_member'`
      (or org → People → Invite member).
- [ ] From **each account's** browser session, accept the invitation.
- [ ] Confirm each shows 2FA-enabled in org → People (if the org enforces 2FA).

---

## Phase 3 — Teams + repo access  [gh] (~5 min)

Code owners are only honored if their team has **write access** to the repo —
this is the #1 silent failure. Create the reviewer/merger teams, add members,
grant write.

- [ ] Create teams:
      ```
      for t in architect team-spec integration; do
        gh api -X POST /orgs/ken-topos/teams -f name="$t" -f privacy=closed >/dev/null
      done
      ```
- [ ] Add members:
      ```
      gh api -X PUT /orgs/ken-topos/teams/architect/memberships/ken-architect
      gh api -X PUT /orgs/ken-topos/teams/team-spec/memberships/ken-spec-author
      gh api -X PUT /orgs/ken-topos/teams/team-spec/memberships/ken-spec-qa
      gh api -X PUT /orgs/ken-topos/teams/integration/memberships/ken-integrator
      ```
- [ ] Grant each team **write** on the repo (so CODEOWNERS requests them):
      ```
      for t in architect team-spec integration; do
        gh api -X PUT /orgs/ken-topos/teams/$t/repos/ken-topos/ken -f permission=push
      done
      ```
- [ ] (Optional/future) create the build-team teams `team-kernel … team-foundation`
      and `steward`,`librarian` the same way when those roles go live; they may
      start empty.

---

## Phase 4 — Create the `ken-ci` GitHub App  [manual] (~15 min)

Org → **Settings → Developer settings → GitHub Apps → New GitHub App** (create it
**under the org**, not your personal account).

- [ ] Name `ken-ci`; homepage = the repo URL.
- [ ] **Webhook: uncheck Active** for now (re-enable later only for the convo
      bridge).
- [ ] **Repository permissions:** Contents = **Read & write**; Pull requests =
      **Read & write**; Checks = **Read & write**; Actions = **Read**; Metadata =
      **Read** (auto). Leave everything else No access.
- [ ] Create the App. **Note the App ID.**
- [ ] **Generate a private key** → download the `.pem`. Store it in your secret
      manager — it is a credential equal to all the App's powers; never commit it.
- [ ] **Install App** → only `ken-topos/ken`. Note the **installation ID**
      (visible in the install URL / `gh api /app/installations` later).
- [ ] Put App ID + installation ID + the `.pem` where the harness can read them
      (the harness mints short-lived installation tokens from these for the
      build-team authoring agents).

---

## Phase 5 — Fine-grained PATs for the accounts  [manual] (~5 min each)

The reviewer/merger agents act as their account via a PAT. From **each account's**
Settings → Developer settings → **Fine-grained tokens** → Generate:

- [ ] Resource owner = `ken-topos`; **Only select repositories** = `ken-topos/ken`.
- [ ] Permissions:
      - `ken-architect`, `ken-spec-author`, `ken-spec-qa`: **Pull requests: RW**,
        Contents: Read.
      - `ken-integrator`: **Contents: RW**, **Pull requests: RW** (needed to merge
        + drive the queue).
- [ ] Expiry: 90 days (or your policy). **Set a calendar reminder to rotate.**
- [ ] Copy the token once → store in the harness/secret store keyed to that agent.

(The build-team authoring agents use the **App** installation token from Phase 4,
not a PAT.)

---

## Phase 6 — Verify CODEOWNERS resolves  [manual] (~5 min)

- [ ] On github.com, edit `.github/CODEOWNERS` in the web editor (don't save) —
      GitHub shows a **"Owners" / syntax** check inline; confirm **no
      "unknown owner / no write access"** warnings for `@ken-topos/architect` and
      `@ken-topos/team-spec`. Fix team membership/access (Phase 3) if any appear.

---

## Phase 7 — Smoke test BEFORE locking down  (~10 min)

Do this while `main` is still unprotected, so you can iterate freely.

- [ ] Open a throwaway **draft PR** touching a normal path (e.g. a README typo)
      as the App (or manually). Confirm the 4 CI checks run.
- [ ] Mark it ready; confirm `ken-architect` is auto-requested as reviewer.
- [ ] Approve as `ken-architect`; confirm a non-Integrator account **cannot** be
      the one to merge once protection is on (you'll re-verify in Phase 8).
- [ ] Touch `/spec/...` in another throwaway PR; confirm `team-spec` is *also*
      auto-requested. Close both PRs.

---

## Phase 8 — Branch protection / ruleset (DO LAST)  [gh or manual]

Repo → Settings → Branches → add a rule for `main` (or use a ruleset). Set:

- [ ] Require a pull request before merging; **require review from Code Owners**;
      **dismiss stale approvals** on new commits; require conversation resolution.
- [ ] Require status checks to pass: `build + test`, `conformance suite`,
      `clean-room provenance check`, `path-guard`. (These are placeholder jobs
      today — they pass trivially; that's fine as a scaffold.)
- [ ] **Require linear history**; **require merge queue**; allow **squash only**.
- [ ] **Restrict who can merge** (push to `main`) to the **`integration`** team.
      This is what makes `ken-integrator` the sole merger.
- [ ] Block force-pushes and deletions.
- [ ] Leave **"include administrators" OFF** during early phases so you can
      intervene if the bootstrap wedges; turn it on once the loop is stable.

`gh` starting point (adjust; the protection payload is fiddly — UI is fine):
```
gh api -X PUT /repos/ken-topos/ken/branches/main/protection --input protection.json
```
where `protection.json` sets `required_status_checks.contexts` to the four check
names above, `required_pull_request_reviews.require_code_owner_reviews=true` +
`dismiss_stale_reviews=true`, `required_linear_history=true`,
`restrictions.teams=["integration"]`, `allow_force_pushes=false`.

- [ ] Enable the **merge queue** for `main` (Settings → branch rule → merge queue)
      so `ci.yml`'s `merge_group` trigger re-checks against latest `main`.
- [ ] Re-run the Phase-7 smoke test end to end: draft → green → ready → Architect
      approve → **only `ken-integrator` can merge** → merged.

---

## Appendix — inventory to record (in your secret store, not here)

| Account | Email | Team(s) | Token location | 2FA recovery |
|---|---|---|---|---|
| ken-architect | `<you>+ken-architect@…` | architect | | |
| ken-spec-author | `<you>+ken-spec-author@…` | team-spec | | |
| ken-spec-qa | `<you>+ken-spec-qa@…` | team-spec | | |
| ken-integrator | `<you>+ken-integrator@…` | integration | | |
| ken-ci (App) | — | — | App ID + install ID + .pem | — |

Rotation: PATs every 90 days; App key if ever exposed. Each agent's harness maps
its convo identity → the right GitHub credential (App token for authors; the role
PAT for Architect / Spec / Integrator).
