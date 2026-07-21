# Current briefing (live — read this first on every Steward resume)

> **This file is LIVE STATE ONLY.** When something here stops being true,
> move it to `diary/YYYY/Mon/DD.md` — do not append a newer block above it.
> Appending is what grew the old tracker to 2.23 MB.
> History: [`INDEX.md`](INDEX.md) · Work items: `docs/program/issues/*.md`

**As of 2026-07-21 ~12:30Z. Operator is BACK and engaged.**

## Standing state

- Fleet is **SINGLE-THREADED**. Nothing is owed to any ring; every idle ring
  is **correct**, not a stall.
- `origin/main = cd3606a1`. Nothing is blocked.
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

- **Provider concentration** — only `runtime-implementer` and `adversary` are
  on the Anthropic pool.

**CI-TRACKER-GATE is RESOLVED.** The operator granted the app `workflows:
write`; verified present in the installation's permission set, and a
workflow-bearing push was accepted. Close the issue once PR #804 lands.

> ★ **Diagnosing a `workflows`-permission rejection.** A freshly minted token
> is NOT enough — but neither is assuming staleness. `mint-gh-token.sh` with
> its final extraction changed from `['token']` to `['permissions']` prints
> the installation's **actual** grants. That converts "the push failed, why?"
> into a direct answer. Note the publisher only mints a new token when `gh`
> is not already authenticated, so a cached ~1h token keeps its old scopes;
> force a fresh mint before concluding anything.

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

## ⏸ TRACK Q — ACTIVE. Q1 done; Q2 blocked on the operator (2026-07-21)

The operator authorized **Q1 and Q2** (program 11 Track Q). Q1 is **landed**
(`cd3606a1`). **Q2 is blocked on two operator answers — do not start either
without them, and do not infer past them:**

1. **Go for Q2a?** Proposed split, awaiting approval:
   - **Q2a — mechanical scan.** Advisory §5's risk patterns are *syntactic
     smells* (bare count literals, source-text greps, `#[ignore]`, `Err(_)`,
     wall-clock asserts): scriptable across all 1909 tests, **no domain
     knowledge and no ownership question**. Output is a candidate queue.
   - **Q2b — domain triage of only Q2a's candidates.** The hard half ("is
     this literal a contract value or today's repo state?") genuinely needs
     the owning team, but runs on a small fraction.
2. **`ken-cli` and `ken-foundation` ownership** — genuinely undocumented.
   `CODEOWNERS` declares itself inert and maps no crate; `agent/teams/` holds
   only `foundation/`, which never names a crate. **`ken-foundation` is the
   trap: its NAME says Foundation, its CONTENTS are the K3 value model, and
   the sole WP citing it says Runtime.** Do not infer an owner into the
   tracker.

> ### ★ Why Q2 is NOT routed per-crate
>
> The test distribution is brutally concentrated — `ken-elaborator` alone is
> **1052 of 1909 (55%)**, while `ken-verify` has 24. Routing by crate hands
> one team the majority and is a queue behind Language, not a fan-out.
> **This is the C9 lesson again: partition by the work unit, not by the
> container.** (C9 sharded by test rather than by crate for exactly this
> reason.)

**Q1's finding, so it is not re-derived:** the advisory splits into a
**review** checklist (§9) and an **authoring** workflow (§6). §9 had been in
`qa.md` since 2026-07-18; `implementer.md` referenced the advisory **not at
all**. The gap was *routing* — guidance reached reviewers, never authors.
Fixed by a **pointer** in `implementer.md` step 4, never a copy.

## My queue, in order

0. **Track Q above** — Q2 the moment the operator answers.
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

**Nothing in flight.** Branch aligned on `origin/main`, clean, no orphaned
polls.

Recently landed and verified by content: **#810** (restore
`px8f_buffer_native`, scope the dedicated jobs), **#811** (SHA-pin
`install-action`), **#812** (Q1 + RT-PARITY closure).

## ⚠ FLEET IS MID-RESEAT — leader / implementer / QA seats → Sonnet 5

The operator is reseating the build-team seats (**not** spec-leader). Seats
were cycling as of ~19:00Z.

> ### ★ NEW TRAP — a reseated agent re-posts an ALREADY-CLOSED retro
>
> `kernel-leader` came up on a fresh seat and posted a §10 retro for
> **KTR-1, which closed 2026-07-14 with retros already in** (`65d68cfc`,
> PR #675). Not an error on its part — it reported what its context showed.
> But **counting such a re-post inflates the promotion ladder.**
>
> **Verify every post-reseat retro against the RECORDED state, never the
> report:** `docs/program/issues/<ID>.md` frontmatter (`status: closed`), or
> the diary for WPs predating the issue system. Expect more of these as the
> remaining seats come up.
>
> Contrast RT-PARITY, where the leader's near-identical announcement *was*
> actionable: its retros were genuinely in and only the frontmatter lagged at
> `merged`. **The two look the same from the outside — only the recorded
> state tells them apart.**

**Do not kick any WP until the reseat is complete and the operator releases
one.** Delivering into a seat that is about to restart is the transport
failure the Handoff Gate exists to prevent.

## Programs written, NOT started

- `docs/program/10-linux-abi-completion.md` — the work Linux ABI II presumes.
  Tracks R/A/M/S/T; **PX9 gates most of Track T**.
- `docs/program/11-test-suite-and-ci-remediation.md` — **Track C is DONE.
  CI went 47 min -> ~8 min, landed `8b09fb95`.** Skip the three slow native
  binaries + nextest + shard x4 + `opt-level = 2` on deps + rust-cache
  removed. **Do not shard further** — per-shard parallelism already fell
  3.96x -> 2.5x, so 8 shards would buy ~90s for double the compute. The next
  real reduction must come from `CI-SKIPPED-NATIVE-TESTS` getting faster.
  Details and full scorecard in §1a/§1b. **Track Q (the QA-advisory sweep)
  is untouched and still the actual point of the program.**

  **Skipped-test restoration — measured, not guessed** (§1c/§1d,
  `CI-SKIPPED-NATIVE-TESTS`). Any job finishing under the ~471s critical
  shard costs **zero** wall clock, and that headroom is the budget:
  - `px8f_write_partition` ✅ restored, own `native-slow` job. **C6 gave it
    −22.7%** (309s→239s) vs 8.4% suite-wide — C6's benefit is concentrated
    in cranelift-heavy code, exactly as predicted.
  - `px8f_buffer_native` ✅ restored, own `native-buffer` job. **Measured on
    run 29850680007: Test step 149s, job 224s** — well under the 482s
    critical shard, so it costs zero wall clock, as predicted.
  - `rt_parity_native` — **a ONE-TEST problem.** Parallelizes fine (7 tests,
    266.7s wall / 470.6s CPU), but
    `fs_write_at_malformed_offset_narrows_to_invalid_offset` takes **221.4s**
    vs **42.2s** for its near-identical sibling. Fix that one and the binary
    lands ~90s. **Do not just re-enable it** — today it fits by ~1s, which
    is noise.

  ✅ **Dedicated jobs are now scoped** (`-p <crate> --test <name>`), not
  `--workspace` — that was compiling all 200 test binaries to run one. Now
  **confirmed by isolation**: `px8f_write_partition`'s Test step went
  **241s → 129s (−112s)** across runs 29850405231 → 29850680007 with
  scoping as the only variable, against a ~124s estimate. The `Build` step
  stays `--workspace`: it is only ~65s and it is what proves the workspace
  compiles under `--locked`.

  **Only `rt_parity_native` is still skipped, and the target is ONE test.**

  > ⚠ **The `native-buffer` number was right for the wrong reason — do not
  > cite it as evidence C6 generalizes.** I projected ~240s for that job and
  > measured 224s. But that projection was of an *unscoped* job, and the
  > scoping change (−112s) landed in the **same PR**. Unscoped, the job
  > would have been ~336s — the projection was ~40% high and was rescued by
  > an unrelated bundled change. **Two changes in one PR made a wrong
  > prediction look confirmed.** C6's −22.7% on `px8f_write_partition`
  > remains the only clean measurement of C6 on cranelift-heavy code; it is
  > a single data point, not an established scaling law. Sibling of
  > `green-vs-green does not confirm a fix` — a number matching its forecast
  > is not evidence the reasoning behind the forecast was sound.

  > ★ **I had the CI diagnosis backwards, and the operator caught it.** I
  > claimed a cold dependency build dominated the wall clock. Measured:
  > **build 47s, test execution 44m14s — 95% of the run.** The error was
  > reasoning from *"there is no cache"* (true) to *"the build is the cost"*
  > (never checked) without opening a single run log. The logs were
  > available the entire time. **An explanation for why something COULD be
  > slow is not evidence that it IS.**

  Measured distribution: `cargo test` walks its **200 test binaries strictly
  in series**, and **three of them — nine tests — are 56.5% of the whole
  run** (`rt_parity_native` 14m41s, `px8f_buffer_native` 5m10s in a *single*
  test, `px8f_write_partition` 5m09s in a *single* test). The bottom 150
  binaries total **48 seconds**. All three fat binaries do a real native
  codegen-and-link per test case.

  > **Operator ruling 2026-07-21: remove `Swatinem/rust-cache` as part of
  > C6** (tracked as C8). No measurable benefit, and it is a third-party
  > dependency with access to the build — a supply-chain surface taken on
  > for nothing. **A dependency must earn its place.** My counter-argument
  > (it absorbs C6's rebuild) was weak: it defended a dependency on an
  > untested hypothesis and priced only time, never trust.
  >
  > ⚠ **C6 and C8 are in latent tension** — C6 can only *increase*
  > dependency compile time, and C8 makes every run pay it. **The C6 run
  > must report the Build step**, not just test numbers. Thresholds are
  > pre-committed in §3b; if the build blows up, **return it to the operator
  > with the number** rather than quietly reinstating the cache.
  >
  > ⚠ **C2 added `taiki-e/install-action@nextest`, unpinned — same class of
  > exposure.** It is defensible because nextest earns it (it fixes the
  > actual problem) where the cache did not, but **pin it to a commit SHA**.

  **Next steps are C2 → C6 → C7, re-measuring between each:** nextest (one
  global pool replaces the serial walk), `[profile.dev.package."*"]
  opt-level = 2` (cranelift runs its codegen unoptimized — **hypothesis,
  test it in CI, do not merge on plausibility**), and splitting the two
  1-test binaries (unsubdividable, so they become the critical path the
  moment C2 lands). C1 landed and bought ~5s — **do not report it as a
  throughput win**.

  ⚠ C7 and Q7 are one edit from two sides — splitting the native binaries
  for parallelism, and giving temp dirs per-test ownership so that
  parallelism is safe. Do them together or expect a flake that reads as
  "nextest broke the suite."

  **`scripts/ci-test-timings.sh <run-id>`** regenerates the per-binary table
  from any run's log. Granularity is the binary; per-`#[test]` needs C2.

Next step for either program when the operator says go: decompose its tracks
into `docs/program/issues/` entries.

## Tooling traps — distrust a clean negative

- ⛔⛔ **AFTER EVERY MERGE, RE-BASE `steward/work` ONTO `main` BEFORE THE
  NEXT COMMIT.** Cost three publish cycles on 2026-07-21 — the same trap
  each time. `main` merges are **squash** merges, so `steward/work` never
  contains the resulting commit: its merge base stays at the *previous*
  main, and GitHub's three-way merge then conflicts on any file both sides
  touched, even when the content is compatible.

  > ★ **`git diff origin/main..HEAD` will NOT warn you.** A two-dot diff
  > shows the **net difference**; a merge asks a **different question** —
  > what happens when both sides' changes are replayed from a shared base.
  > A clean two-dot diff next to a conflicting merge is not a contradiction.
  > **The check that actually predicts a conflict is**
  > `git merge-base --is-ancestor origin/main HEAD` — if that fails, rebase
  > *before* committing anything further.

  Recipe (content-preserving, verified three times):
  `git tag -f preserved/<sha> HEAD` → `git reset --hard origin/main` →
  `git checkout <old-sha> -- <changed files>` → regenerate the dashboard →
  commit. Then confirm with `git diff <old-sha> HEAD`: the **only** expected
  delta is `IMPLEMENTATION-PROGRESS.md`'s timestamp line.
  ⚠ Do **not** verify with a bare `git diff` after `git checkout -- <path>`
  — that stages the files, so unstaged `git diff` reads empty and looks like
  a mismatch. Compare **commit to commit**.

- ⛔ **`scripts/scripted-pr-automerge.sh` exits 0 on failure** (4 times on
  2026-07-21). Its **first attempt after any merge always fails** with
  `stale info`, because the merge deletes the origin head branch and stales
  the local ref. **Always `git fetch origin --prune` before publishing.**
  ⚠ Its `--description-file` must exist **before** the call — a heredoc
  inside the same `&&` chain does not reliably land, and the script reports
  `description file not found` and exits.
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
