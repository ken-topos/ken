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

## Operator rulings — 2026-07-21 ~12:45Z. SETTLED, do not reopen.

**On Linux ABI II** (`research/linux-abi-ii-work-program-proposal.md`):

- **No "ratification."** The charter is a **planning document, not a
  commitment**. Nobody outside the project is watching, and nothing depends
  on our timelines or stated intentions. **I had imported a governance ritual
  that does not apply — do not re-raise status-correction as a decision.**
- **Where there is a gap between what was anticipated and what was done, fill
  the gap first.** Hence `docs/program/10-linux-abi-completion.md`.
- **L2-1: no cross-compilation.** A very late feature, deferred behind a long
  line of other work. Manifest v2 = family-scoped and generated, **not**
  cross-target.
- **L2-0: all desirable, nothing deferred.** All nine
  `RepresentedUnavailable` operations get promoted.
- **Timing, timelines, and budget are the OPERATOR'S domain.** They monitor
  and adjust. **Do not reason about schedule or cost.**
- ★ **My lane is token efficiency in terms of delivered work.** That is the
  axis to optimize and the one to report on.

**Still genuinely open (lower stakes, no re-ask):**

- **CI-TRACKER-GATE** — the publisher app lacks `workflows` permission, so
  the tracker's CI gate cannot be pushed. Until it lands, tracker
  correctness is a convention, not a property.
- **Provider concentration** — only `runtime-implementer` and `adversary` are
  on the Anthropic pool.

## The completion program — written, NOT started

`docs/program/10-linux-abi-completion.md` (commit `f2b98c37`, unpublished).
Five tracks — **R** reconcile, **A** availability promotion, **M** manifest
(native-target only), **S** synchronous floor, **T** the committed exit
(PX10/PX11/PX12).

Verified against `main`, not taken from the advisory: 22 ops, 13
`NativeTested` / 9 `RepresentedUnavailable`, no process/socket/poll family in
any state, PX7 landed — and **PX9 (cross-domain `System.Error`) is chartered
but undelivered**, which the advisory's "good filesystem floor" phrasing
obscured. PX9 gates most of Track T.

**Not started.** Next step when the operator says go: decompose the tracks
into `docs/program/issues/` entries.

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
