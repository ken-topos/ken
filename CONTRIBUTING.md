# Contributing to Ken

Ken is built by multiple agent teams coordinating through
[mootup](https://mootup.io) spaces and GitHub. This file is the short
version; the authoritative workflow is
[`docs/program/04-git-and-integration.md`](docs/program/04-git-and-integration.md), and the per-role
behavioral playbooks live under [`agent/playbooks/`](agent/playbooks/).

## The rules that matter

1. **`main` is always green.** Nothing merges unless the build + conformance
   suite pass in CI.
2. **Teams prepare branches; teams never merge.** The scripted publisher path
   opens and merges PRs under Steward/operator control, then the Steward routes
   any downstream notification.
3. **Clean room.** Implement from `/spec` and conformance tests — never from
   AGPLv3 or other copyleft source. See [`CLEAN-ROOM.md`](CLEAN-ROOM.md). The
   **Spec enclave** (Opus, Anthropic-hosted) is the only team that may consult
   copyleft references, and only for behavior/approach, never code structure.

## Branch & PR

- Branch per work package: `wp/<WP-ID>-<slug>` (e.g. `wp/K1-core-type-theory`).
  WP IDs are in [`docs/program/03-program-of-work.md`](docs/program/03-program-of-work.md).
- One work package (or one reviewable sub-task) per PR; keep PRs small.
- The PR must: target `main`, cite its WP ID + the acceptance criteria it
  satisfies, cite **spec sources** (not prototype source), be conformance-green,
  and carry the required federation review/Decision record.
- Don't click merge. When the branch is review-ready, the team leader opens the
  required mootup Decision; the scripted publisher path handles PR publication
  and merge after the gate resolves.

## Coordination (mootup)

- Each team has its own space; the Steward coordinates cross-team flow.
- One thread per work item; reply in-thread; mention an agent **iff** the next
  move is theirs.
- After you hand off, **stop** — set status and wait for a notification. Workers
  do not poll; the team leader's watchdog catches stalls.

See [`agent/playbooks/`](agent/playbooks/) for the full per-role discipline.
