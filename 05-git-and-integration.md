# Git Workflow & Integration Authority

How the Ken teams branch, review, merge, and stay coordinated. Two systems, one
split of responsibility:

- **GitHub** — source of truth for **code and review**: branches, PRs, CI,
  protected `main`.
- **convo / mootup** — source of truth for **coordination**: per-team spaces,
  cross-team **Decisions**, and **notifications** of fresh `main`.

They are bridged by artifact references: a convo Event can carry a GitHub PR URL
(convo is designed for this — see its README "artifact references"), so a PR and
its merge decision live in both places without either being authoritative over
the other's domain.

Governing rules (non-negotiable):

1. **`main` is always green.** Nothing merges unless the conformance suite passes.
2. **Teams never merge to `main`.** Teams open PRs; a single **Integrator**
   merges.
3. **Clean-room is enforced at the merge gate.** No AGPL-derived code enters
   `main`; only **Team Spec** mediates prototype knowledge (see `02-strategy.md`).
4. **One source of truth per concern.** Code/review → GitHub. Decisions/notify →
   convo. Don't duplicate authority.

This doc is realized by work package **F1** (repo, branch protection, CODEOWNERS,
templates) and depends on the GitHub repo existing — which it does not yet
(`~/src/ken` is a plain directory).

---

## 1. Repository model

- **One mono-repo** (`ken`) on GitHub. A compiler + kernel + conformance suite
  belongs together; "fresh commits on `main`" is one trunk.
- **`main` is protected:** no direct pushes, no force-push, no team merges;
  linear history; required status checks; required review from the Integrator
  (and from the owning team via CODEOWNERS).
- **CODEOWNERS maps workstreams → teams.** Each crate/directory has an owning
  team, so a PR touching it auto-requests that team's review. Example:

  ```
  /kernel/         @ken-topos/team-kernel
  /elaborator/     @ken-topos/team-verify
  /prover/         @ken-topos/team-verify
  /lang/           @ken-topos/team-language
  /runtime/        @ken-topos/team-runtime
  /interp/         @ken-topos/team-runtime
  /spec/           @ken-topos/team-spec
  /conformance/    @ken-topos/team-spec
  /docs/adr/       @ken-topos/team-foundation
  * @ken-topos/integration                 # Integrator reviews everything
  ```

- **CI (GitHub Actions, assumed):** build + the conformance suite + a clean-room
  provenance check run on every PR; all are required to be green to merge.

---

## 2. Branch & PR model

- **Branch naming ties to work packages:** `wp/<WP-ID>-<slug>` — e.g.
  `wp/K1-core-type-theory`, `wp/V3-prover-backend`. Exploratory spikes:
  `spike/<team>/<topic>`. The WP IDs are from `04-program-of-work.md`.
- **One work package (or one reviewable sub-task) per PR.** Small PRs merge
  faster and keep `main` green. A large WP (e.g. K1, V3) is split into a series.
- **Every PR must:** target `main`; cite its WP ID and the acceptance criteria it
  satisfies; be conformance-green in CI; pass the clean-room + path-guard checks;
  carry **Architect** approval (always) and **Spec** approval (when it touches
  `/spec`, `/conformance`, or a designated soundness path); then be merged by the
  **Integrator**. Domain correctness is gated *before* the PR by the team's QA
  step in the ring — it is not a separate GitHub reviewer. **Teams do not click
  merge.**
- **PR template** (F1 deliverable) prompts for: WP ID, acceptance criteria met,
  spec source (not prototype source), cross-team impact, and a conformance note.

---

## 3. The Integration authority ("the Integrator")

A **single agent** (DeepSeek V4 Pro — see `agent/MODELS.md`) with **sole merge
rights to `main`**. The Integrator is deliberately *mechanical*: the deep
correctness and architectural review is the **Architect's** job (Opus), which is
why the Integrator can run on a light model. The **Steward** is the escalation
point for cross-team conflicts; Pat decides scope and anything crossing an ADR.
Responsibilities:

1. **Confirm the required reviews are present:** the Architect approved (always),
   and the Spec enclave approved if the PR touched its paths. The Integrator does
   not perform the design review itself; domain correctness was gated pre-PR by
   the owning team's QA in the ring.
2. **Enforce the clean-room gate:** confirm the PR introduces no AGPL-derived
   code and cites spec sources, not prototype source. Reject otherwise.
3. **Require conformance-green** (CI) and **serialize merges** through a merge
   queue so `main` never goes red from interacting PRs.
4. **Merge** (squash, with the WP ID in the commit subject, e.g.
   `K1: dependent Pi/Sigma kernel core (#123)`).
5. **Verify, then notify:** confirm the merge landed and CI is green, then notify
   affected team leaders of the fresh `main` commit (see §4) with a one-line
   changelog and **rebase guidance** for any team whose active branch is impacted.
6. **Guard the gates:** tag G0–G8 milestones (`03-roadmap.md`) when their
   acceptance criteria are met; refuse merges that would regress a passed gate.

The Integrator is a *gatekeeper and notifier*, not a designer or reviewer-of-record
for design. Design judgment lives with the Architect; scope and process with the
Steward and Pat.

---

## 4. convo / mootup mapping

- **One space per team** (`ken-kernel`, `ken-verify`, `ken-language`,
  `ken-runtime`, `ken-ergo`, `ken-foundation` — build teams — plus `ken-spec`,
  the clean-room enclave). The team leader is the accountable participant;
  members set presence with `update_status`. (Research is not a standing team —
  the Steward dispatches it ad hoc; see `agent/playbooks/federation/steward.md`.)
- **One integration space** (`ken-integration`) where the federation roles —
  Steward, Architect, Integrator, Librarian — live, **linked** to every team
  space (`link_space` / `create_linked_team`) so
  cross-space context flows.
- **PRs surface as convo Events** in the integration space, carrying the GitHub
  PR URL as an artifact reference (`share` / `post_response`). The Integrator (or
  a webhook bridge, §6) posts them.
- **Merge approvals are convo Decisions.** When a PR is review-ready the owning
  team `propose_decision` ("merge wp/K1 …", PR URL attached); the Integrator
  `resolve_decision` on merge or rejection. This yields an auditable decision log
  aligned 1:1 with GitHub merges (`list_decisions`).
- **Architecture decisions (ADRs) are also convo Decisions** — proposed in the
  integration space, resolved by Pat/Integrator, then committed to `docs/adr/`.
  The convo Decision is the discussion+ratification record; the committed ADR is
  the durable artifact.
- **Notification of fresh `main`:** on merge, the Integrator posts an Event in
  `ken-integration` and **mentions** the leaders of impacted team spaces
  (`reply_to` / mention), with the merged WP, the commit, and rebase guidance.
  Team leaders pull/rebase and fan the update into their own space.
- **Cross-team dependencies** (the graph in `03-roadmap.md`/`04`) are coordinated
  via the linked spaces and mentions: when WP-B depends on WP-A landing, the
  B-team leader watches A's Decision and is mentioned on its resolution.

---

## 5. The merge lifecycle (end to end)

```
0. (intra-team ring) implementer builds+tests scoped via ken-cargo → QA verifies
   → leader packages. Domain correctness is gated here, before the PR.
1. Leader opens a DRAFT PR from wp/<WP-ID>-<slug>, under the team's GitHub
   identity (the ken-ci App; see docs/ops/github-setup.md), citing WP ID +
   acceptance criteria + spec sources.
2. CI runs on the draft: build+test · conformance · clean-room · path-guard.
   concurrency:cancel-in-progress kills superseded runs on new pushes.
3. CI green → auto-transition draft → ready-for-review (the ken-ci App).
4. "Ready" fires CODEOWNERS review requests: Architect (always) + Spec (only if
   /spec, /conformance, or a designated soundness path is touched). The leader
   posts a `review_request` Event with the PR URL, mentioning the reviewer(s).
5. Reviewers approve on GitHub. A change request → push fixes → CI re-runs →
   stale approvals dismissed → re-review on green. (No draft toggle needed.)
6. Required approvals + green + merge queue → INTEGRATOR (sole merge identity)
   squash-merges via the queue (re-checks against latest main). The merge
   Decision resolves.
7. Integrator verifies the merge landed + CI green, then posts the ship Event in
   ken-integration, mentioning only the affected team leaders with rebase
   guidance. Steward digests the merge log; Pat hears only gate-level news.
8. Impacted teams rebase active branches on the new main.
```

`main` stays green at every step because (a) CI gates the PR and (b) the merge
queue re-checks against the latest `main` before landing.

---

## 6. Setup & automation

Decided: org/repo **`ken-topos/ken`** (public OSS); **GitHub Actions** CI;
**squash** merges. Identities follow the **App-plus-accounts** model — a `ken-ci`
GitHub App for automation + agent authoring, and a small set of machine-user
accounts for the CODEOWNERS-eligible reviewers and the restricted merger. The
full mechanics — App permissions, the ~5 accounts (`+tag` emails), branch
protection, merge queue, CI concurrency, and the auto-ready automation — live in
**`docs/ops/github-setup.md`**.

Still optional:
- **convo bridge (nice-to-have):** a GitHub-webhook → convo bridge that auto-posts
  PR-opened / merged Events and opens/resolves the merge Decision. Until it
  exists, the Integrator posts via the convo MCP tools manually — the workflow
  does not depend on the bridge.
- **Per-team owning-review:** off to start (the QA ring covers domain
  correctness). Add per-team leader accounts + CODEOWNERS crate lines only if
  review quality later warrants it.

---

## 7. Clean-room reminder in the git context

The merge gate is where clean-room compliance is *mechanically* enforced:

- Implementation PRs must cite **spec sources** (`/spec`, conformance tests), not
  prototype `file:line`. The PR template asks for this; the Integrator checks it.
- The CI **clean-room/provenance check** scans for copied AGPL text and flags
  license headers. (Design in F1; can start as a denylist + manual review.)
- **Team Spec** is the only team that reads the AGPL prototype to produce specs;
  its PRs touch `/spec` and `/conformance`, never implementation crates. This is
  visible in CODEOWNERS and in the branch/PR boundaries.
