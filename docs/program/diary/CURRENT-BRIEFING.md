# Current briefing (live — read this first on every Steward resume)

> **This file is LIVE STATE ONLY.** When something here stops being true,
> move it to `diary/YYYY/Mon/DD.md` — do not append a newer block above it.
> Appending is what grew the old tracker to 2.23 MB.
> History: [`INDEX.md`](INDEX.md) · Work items: `docs/program/issues/*.md`

**As of 2026-07-21 ~12:30Z. Operator is BACK and engaged.**

## Standing state

- Fleet is **SINGLE-THREADED**. Nothing is owed to any ring; every idle ring
  is **correct**, not a stall.
- `origin/main = 62643287`. Nothing is blocked.
- **Do not kick a WP while the operator has an open question below.**

## Awaiting the operator — asked, unanswered. Do NOT re-ask.

**Linux ABI II** (`research/linux-abi-ii-work-program-proposal.md`) — briefed
12:15Z. Four decisions are genuinely theirs:

| # | Decision | My recommendation |
|---|---|---|
| 1 | Ratify "foundation and descriptor tranche" as the first campaign's honest status | **Yes, now** — costs nothing, unblocks planning from truth |
| 2 | `io_uring` v1 in scope | **Yes in principle**, gate framing on an Architect ruling on the atomics-privacy claim |
| 4 | Public C ABI as a companion program | **Yes — and don't start it**; it competes with the real committed debt |
| 7 | MMIO as L2-8 | **Confirm, last** |

★ **The finding that matters:** charter `09-posix-linux-abi-campaign.md` says
"COMMIT THROUGH PX-E", but PX10/PX11/PX12 (process, socket, `epoll`) are
**absent** from the closed 22-operation `HostOpV1` catalog — verified against
`main`, not taken from the report. What landed is a foundation + descriptor
tranche, not the committed campaign. **L2-0 (track zero) is already my
queue**: BUDGET-EFF / SEAL-2 / RT-ESCAPE / RT-SPLIT *are* the PX8 chain it
names.

Also open, lower stakes:

- **CI-TRACKER-GATE** — the publisher app lacks `workflows` permission, so
  the tracker's CI gate cannot be pushed. Until it lands, tracker
  correctness is a convention, not a property.
- **Provider concentration** — only `runtime-implementer` and `adversary` are
  on the Anthropic pool.

## My queue, in order

1. **BUDGET-EFF** — Handoff Gate the Spec enclave (spec-leader, spec-author,
   conformance-validator). **Spec erratum FIRST**: `38` self-contradicts
   (`:404-405`/`:443-444` say *effective*, `:419-420`/`:438-440` say
   *requested*), so a code-first fix re-derives the defect from a broken
   citation. It is a **plumbing gap, not a formula fix** —
   `TransferCountV1::new(read, effective)` validates then **discards**
   `effective`, so neither reifier can compute the bound. Two closures with
   different blast radii ⇒ **Architect call**. Oracle
   `adversary/R1-effective-request-repro @ 06bb9538` is pinned as AC-3 and
   must pass **unchanged**. Detail:
   `docs/program/wp/BUDGET-EFF-remaining-bounded-by-effective-request.md`
2. **SEAL-2** — re-anchor on current `main`; evidence
   `adversary/SEAL2-repros@70a603da`.
3. Then F1 (#37) → Architect · STR-BIJ → enclave · A3 · F4 · F3 · RT-SPLIT.
   **STOP at #38/#32/#24/#25** — title-only; reconstruct **with** the
   operator, never invent scope.

## In flight

- **PR #803** — the pre-commit hook. In its ~40-minute CI wait. Verify with
  `git cat-file -p origin/main:.githooks/pre-commit`, not by exit code.

## Tooling traps — distrust a clean negative

- ⛔ **`scripts/scripted-pr-automerge.sh` exits 0 on failure** (4 times on
  2026-07-21). Its **first attempt after any merge always fails** with
  `stale info`, because the merge deletes the origin head branch and stales
  the local ref. **Always `git fetch origin --prune` before publishing.**
  Redirect its output to a file — a `| tail` pipe block-buffers it to 0
  bytes. Afterwards it sleeps ~40 min polling a PR that may already have
  merged; verify `origin/main` by content and kill the orphan. Tracked as
  issue `PUB-VERIFY`.
- **A piped exit code belongs to the last command in the pipeline**, not to
  `git`. Verify `HEAD` moved.
- **Branch-ahead ⇏ unmerged** (squash-merge trap). Verify by content.
- **Concurrent subagents in one worktree share `.git`** — path-disjoint is
  **not** commit-disjoint. Two raced the index on 2026-07-21. Use
  `isolation: "worktree"`.
- **`convo` posting can fail while the channel stays up.** An absent post is
  not a stalled agent — check the pane.
- **A literal-string grep is a proxy, not the property.** "Four tools" had
  been upgraded to "Five tools"; the grep read as content loss and nearly
  discarded good work. Grep the theme, then read the hit.
- Liveness: `tmux capture-pane -p -S -300 -t moot-<seat>` — **`-S` must be
  negative**; a positive value returns ~1 line and reads every seat as dead.
- `Press up to edit queued messages` = **busy + queued. Do not resend.**

## Standing discipline

A success signal says a thing **ran**, never that it did what you meant —
**verify by content**. `git rev-parse --abbrev-ref HEAD` must read
`steward/work` before any write. Local builds are **targeted only** via
`scripts/ken-cargo -p <crate>` — **never `--workspace`** (it OOMs the box).
Workspace-green means green in **CI**.
