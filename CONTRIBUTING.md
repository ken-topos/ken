# Contributing to Ken

Ken is built by multiple agent teams coordinating through
[convo/mootup](https://mootup.io) spaces and GitHub. This file is the short
version; the authoritative workflow is
[`05-git-and-integration.md`](05-git-and-integration.md), and the per-role
behavioral playbooks live under [`agent/playbooks/`](agent/playbooks/).

## The rules that matter

1. **`main` is always green.** Nothing merges unless the build + conformance
   suite pass in CI.
2. **Teams open PRs; teams never merge.** A single **Integrator** reviews and
   merges, then notifies affected team leaders of fresh `main`.
3. **Clean room.** Implement from `/spec` and conformance tests — never from the
   AGPLv3 prototype's source. See [`CLEAN-ROOM.md`](CLEAN-ROOM.md). Only **Team
   Spec** reads the prototype.

## Branch & PR

- Branch per work package: `wp/<WP-ID>-<slug>` (e.g. `wp/K1-core-type-theory`).
  WP IDs are in [`04-program-of-work.md`](04-program-of-work.md).
- One work package (or one reviewable sub-task) per PR; keep PRs small.
- The PR must: target `main`, cite its WP ID + the acceptance criteria it
  satisfies, cite **spec sources** (not prototype source), be conformance-green,
  and request review from CODEOWNERS + the Integrator.
- Don't click merge. When the PR is review-ready, the team leader opens a convo
  Decision (PR URL attached); the Integrator resolves it on merge.

## Coordination (convo/mootup)

- Each team has its own space; the Integrator's space links them all.
- One thread per work item; reply in-thread; mention an agent **iff** the next
  move is theirs.
- After you hand off, **stop** — set status and wait for a notification. Workers
  do not poll; the team leader's watchdog catches stalls.

See [`agent/playbooks/`](agent/playbooks/) for the full per-role discipline.
