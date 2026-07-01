# Git Workflow & Integration Authority

How the Ken teams share one workspace, branch, review, merge, and stay
coordinated. The defining choice: **agents never touch GitHub.** They work in a
single shared clone via per-agent worktrees and do **local git only**. A single
**publisher** identity — operated by the Integrator — is the federation's only
GitHub-network actor (push, CI, merge, fetch).

Two systems, one split of responsibility:

- **GitHub** — system of record for **code and CI**: the canonical `main`,
  branches the publisher pushes, the build/conformance/clean-room checks, and
  protected merges.
- **mootup** — system of record for **coordination and review**: per-team
  spaces, the **merge/review Decisions**, and **notifications** of fresh `main`.

They are bridged by artifact references: a mootup Event can carry a branch or
commit reference, so a merge and its review Decision live in both places without
either being authoritative over the other's domain.

Governing rules (non-negotiable):

1. **`main` is always green.** CI (build + conformance) runs on GitHub's CPU and
   gates the merge **before** it lands. Nothing merges red.
2. **Agents do local git only — no credentials.** The **Integrator** is the sole
   GitHub identity: it pushes branches, reads CI, merges, and fetches `main`.
3. **Teams never merge.** Teams produce commits on a local branch; the
   Integrator merges.
4. **Clean-room is enforced at the merge gate.** No AGPLv3 or other
   copyleft-derived code enters `main`; the **Spec enclave** grounds the spec
   in permissive references and first principles (see `01-strategy.md`).
5. **One source of truth per concern.** Code/CI → GitHub. Coordination + the
   review/merge Decision → mootup. Don't duplicate authority.

This doc is realized by work package **F1** (workspace, worktrees, the publisher
identity, branch protection) and depends on the GitHub repo existing — which it
does (`ken-topos/ken`) — and the shared dev workspace being provisioned.

---

## 1. The shared workspace (how N agents avoid clobbering each other)

The whole fleet runs in **one devcontainer** with **one clone**. Per-team
devcontainers would buy nothing — same disk, RAM, and CPU, and the same in-clone
deconfliction problem. Edits are segregated by **git worktrees**, not by
separate checkouts.

- **One worktree per active agent**, under `.worktrees/<agent>`. An agent edits
  only inside its own worktree; the single `.git` is shared, so every branch and
  every fetched ref is visible to all worktrees at once.
- **No worktree ever sits on `main`.** An idle agent rests on its current
  `wp/<ID>` branch or a `<role>/idle` home branch. The **publisher's** worktree
  stays on `main` permanently and is the only one that does. Create a WP branch
  with `git branch wp/<ID>-<slug> main`, never `git checkout -b` from `main`
  (which would move a worktree onto `main`).
- **Rebase onto `origin/main` at startup.** The publisher does all network I/O;
  when it fetches, the shared `origin/main` ref updates for *every* worktree. So
  each agent picks up merges with a local `git rebase origin/main` — **no
  network and no credentials required** (public-repo reads aside, agents never
  need even that — the ref is already local).
- **Commit before you hand off.** Never hand off uncommitted work: the next
  agent and the publisher can only see committed state. Handing off a dirty
  worktree is a protocol violation.
- **Worktree/`main` mismatch is a named stall class.** A change is **not live**
  for other agents until it is **merged to `main` AND they have rebased**. "It
  works in my worktree" is the classic false-resolved bug — verify against
  `origin/main`, not your own tree.
- **Build/test only via `scripts/ken-cargo`, scoped to your crate** — never raw
  `cargo` or `--workspace` (COORDINATION §12). The full-workspace build runs in
  CI, on GitHub's CPU, not on the laptop.

---

## 2. Branch & WP model

- **One work package = one branch** `wp/<WP-ID>-<slug>` — e.g.
  `wp/K1-core-type-theory`, `wp/V3-prover-backend`. Exploratory spikes:
  `spike/<team>/<topic>`. Work packages — definition, lifecycle, and ownership
  (the Steward) — are in `03-program-of-work.md`.
- **The ring is sequential (COORDINATION §0), so one WP branch is handed between
  worktrees.** The implementer commits to `wp/<ID>` in its worktree, then
  returns to its home branch (freeing the branch); QA checks `wp/<ID>` out in
  *its* worktree, verifies, commits any small repairs, and returns home. No
  per-role sub-branches are needed because only one agent edits at a time.
- **Small PRs merge faster and keep `main` green.** A large WP (e.g. K1, V3) is
  split into a short series of `wp/<ID>` branches, each merged on its own.
- The WP branch is the unit the **publisher pushes for CI** and the **Integrator
  squash-merges** (one commit per WP).

---

## 3. Credentials & the publisher (the one GitHub identity)

Exactly **one** credentialed GitHub identity exists: the **publisher**, operated
by the **Integrator**. It is the federation's whole GitHub-network surface —
pushing `wp/<ID>` branches to trigger CI, reading check results, merging, and
fetching `main`. No other agent holds credentials or runs `gh`.

- **Why one identity.** It collapses the per-team-account apparatus to a single
  account, removes the secret-exposure surface across the fleet, and lets every
  build/spec agent stay entirely GitHub-unaware — they only know local git and
  mootup.
- **The CPU offload is preserved — as a pre-merge gate.** The heavy work (full
  `--workspace` build + the conformance suite + the clean-room scan) still runs
  on **GitHub Actions**, on GitHub's CPU. Because the publisher pushes the WP
  branch *before* the merge, CI runs *before* the merge and gates it. The 8-core
  laptop never runs a full-workspace build. Moving to local merges did **not**
  move compute back onto the box — only one identity now drives the push.

---

## 4. Review & merge — in mootup, gated by CI

Review is a **mootup Decision**, not a GitHub PR approval: the reviewers
(Architect, Spec) hold no GitHub accounts, so they review the **diff from the
shared local branch** and vote in mootup.

- When QA approves a WP, the owning **leader** opens a merge **Decision**
  (`propose_decision`, naming the WP ID + `wp/<ID>` branch) in the integration
  space, mentioning the **Architect** (always) and **Spec** (only if it touches
  `/spec`, `/conformance`, or a designated soundness path), and asks the
  **Integrator** to publish the branch.
- The **Architect** (+ **Spec** on its paths) read the diff locally (`git diff
  origin/main...wp/<ID>`) and vote the Decision — a blocking review names the
  concern and the alternative; an approval is a real judgment.
- The **Integrator publishes** `wp/<ID>` → CI runs build+test · conformance ·
  clean-room · path-guard on GitHub.
- **Merge gate — all must hold:** the Decision is approved (Architect always +
  Spec on its paths), CI is green, the clean-room check is green, and no passed
  roadmap gate (G0–G8) regresses. The Integrator then **squash-merges on
  GitHub** — branch protection requires the green checks and restricts the merge
  to the publisher — and **fetches**, so `origin/main` updates for all
  worktrees.
- The merge **Decision resolves** on the merge, giving an auditable log aligned
  1:1 with merges (`list_decisions`).

Domain correctness is gated *before* this, by the team's QA step in the ring —
it is not a separate reviewer. **Teams do not merge.**

---

## 5. mootup mapping

- **One space per team** (`ken-kernel`, `ken-verify`, `ken-language`,
  `ken-runtime`, `ken-ergo`, `ken-foundation` — build teams — plus `ken-spec`,
  the clean-room enclave). The team leader is the accountable participant;
  members set presence with `update_status`. (Research is not a standing team —
  the Steward dispatches it ad hoc; see
  `agent/playbooks/federation/steward.md`.)
- **One integration space** (`ken-integration`) where the federation roles —
  Steward, Architect, Integrator, Librarian — live, **linked** to every team
  space (`link_space` / `create_linked_team`) so cross-space context flows.
- **All GitHub I/O is the Integrator's**, so every GitHub signal reaches the
  fleet only because the Integrator mirrors it into mootup mentioning the actor
  whose move it is (agents never see GitHub; see `agent/COORDINATION.md §14`).
  An optional `ken-ci` webhook→mootup bridge can automate the CI-result mirror;
  until it exists the Integrator posts it by hand. The map:

  | Event | mootup message (type) | space | mentions | posted by |
  |---|---|---|---|---|
  | WP QA-approved, ready to merge | `decision` (open) + `review_request` | integration | Architect (+Spec on its paths) | leader |
  | Integrator published branch; CI running | `status_update` | team | — | Integrator |
  | CI red | `blocked` | team | implementer | Integrator |
  | CI green + Decision approved | `decision` (merge) | integration | Integrator | Integrator |
  | Merged to `main` | `status_update` (ship) | integration | affected team leaders | Integrator |

  The branch/commit rides each message as an artifact reference; the *detail*
  (the diff, the CI log) is fetched on demand — by the reviewer locally, or by
  the Integrator via its identity. Workers never watch CI: only the Integrator
  can see it, and it surfaces red/green as a mention (`agent/COORDINATION.md
  §14`).
- **The merge Decision is the review record.** Opening it *is* the review
  request; the Architect/Spec votes are the review; the Integrator resolves it
  on merge or rejection. No GitHub PR approval is involved.
- **Architecture decisions (ADRs) are also mootup Decisions** — proposed in the
  integration space, resolved by the operator/Integrator, then committed to
  `docs/adr/`. The mootup Decision is the discussion+ratification record; the
  committed ADR is the durable artifact.
- **Notification of fresh `main`:** on merge, the Integrator posts a ship Event
  in `ken-integration` and **mentions** the leaders of impacted team spaces,
  with the merged WP, the commit, and rebase guidance. Team leaders have members
  rebase onto the already-fetched `origin/main` (no network) and fan the update
  in.
- **Cross-team dependencies** (the graph in `02-roadmap.md`/`03`) are
  coordinated via the linked spaces and mentions: when WP-B depends on WP-A
  landing, the B-team leader watches A's merge Decision and is mentioned on its
  resolution.

---

## 6. The merge lifecycle (end to end)

```
0. (intra-team ring) implementer commits to wp/<WP-ID>-<slug> in its worktree,
   scoped builds via ken-cargo → returns to home branch → QA checks the branch
   out, verifies (small repairs ok) → leader packages. Domain correctness is
   gated here, before the merge.
1. Leader opens the merge Decision in ken-integration (mentioning Architect
   always + Spec on its paths), naming the WP ID + wp/<ID> branch, and asks the
   Integrator to publish.
2. Integrator pushes wp/<ID> to GitHub → CI runs: build+test · conformance ·
   clean-room · path-guard (on GitHub's CPU). concurrency:cancel-in-progress
   kills superseded runs on new pushes.
3. Architect (+Spec on its paths) read the diff locally and vote the Decision.
4. CI green AND Decision approved → **the Integrator rebases wp/<ID> onto the
   CURRENT `origin/main` and confirms CI is green on that rebased tip** before
   squash-merging (branch protection: required checks + merge restricted to the
   publisher), one commit with the WP ID in the subject, then fetches so
   origin/main updates for all. **The rebase-before-merge step is mandatory, not
   optional** (operator-directed 2026-07-01): a branch whose CI ran against a
   *stale* base can be green in isolation yet break `main` when combined with
   another branch that landed meanwhile — e.g. one branch adds an enum variant
   while a parallel branch adds a `match` over it; each PR is green against its
   own base, the merged tree is non-exhaustive. Rebasing onto current `main` and
   re-running the full-workspace CI tests the *actual* post-merge tree and
   catches these parallel-branch collisions. (This is the fix for the red-main
   incident of 2026-07-01: `NumericLitVal::Str` added by one VAL1 branch,
   unhandled in `ken-cli lit_to_eval` — invisible until both landed.)
5. Integrator verifies main green, resolves the merge Decision, posts the ship
   Event in ken-integration mentioning only affected team leaders with rebase
   guidance, and sweeps the merged wp/<ID> branch. Steward digests the log;
   the operator hears only gate-level news.
6. Impacted teams rebase active branches onto the new origin/main (no network).
7. Owning team runs the retro (each working agent posts a `retro`; the leader
   collects + hands "retros in" to the Steward). The WP is not *done* until this
   lands (agent/COORDINATION.md §10).
```

`main` stays green at every step because CI gates the branch **before** the
merge, and branch protection refuses a merge whose required checks aren't green.
**But CI-on-the-branch is only sufficient if the branch is rebased onto current
`origin/main` first** (step 4) — otherwise a green branch built on a stale base
can still red `main` on merge (the parallel-collision failure above). Green CI +
current base together are the guarantee; green CI alone is not.

---

## 7. Clean-room reminder in the git context

The merge gate is where clean-room compliance is *mechanically* enforced:

- Implementation work must derive from **spec sources** (`/spec`, conformance
  tests), not from any copyleft `file:line`. The merge Decision records this;
  the Integrator checks it.
- The CI **clean-room/provenance check** scans for copied AGPL text and flags
  license headers. (Design in F1; can start as a denylist + manual review.)
- The **Spec enclave** is the only team that may consult copyleft references
  (for behavior and approach only, under the leakage recheck); its branches
  touch `/spec` and `/conformance`, never implementation crates. This is
  visible in the team's branch/review boundaries.

---

## 8. Setup & graduation

Decided: org/repo **`ken-topos/ken`** (public OSS); **GitHub Actions** CI;
**squash** merges; **one publisher identity** (operated by the Integrator). The
single-publisher mechanics — the one account/App, branch protection (required
checks + merge restricted to the publisher), and CI wiring — live in
**`docs/ops/github-setup.md`**.

- **Deferred graduation — the full GH-PR apparatus.** Per-team GitHub
  identities, CODEOWNERS-driven review routing, and a merge queue (the runbook
  in `docs/ops/runbook-gh-identities.md`) are **not** set up now. They are taken
  up only when **external contributors** arrive or **per-team GitHub review**
  becomes load-bearing — at which point review moves from mootup Decisions onto
  GitHub PRs. Until then the single publisher + mootup review is the whole
  model. The decision is recorded in `../adr/0003-credential-free-publisher.md`.
- **mootup bridge (optional):** a GitHub-webhook → mootup bridge that mirrors
  the §5 CI-result events. Because only the Integrator sees GitHub, the bridge
  merely saves it manual mirroring; the workflow does not depend on it.
