# Current briefing (live — read this first on every Steward resume)

> **This file is LIVE STATE ONLY.** When something here stops being true,
> move it to `diary/YYYY/Mon/DD.md` — do not append a newer block above it.
> Appending is what grew the old tracker to 2.23 MB.
> History: [`INDEX.md`](INDEX.md) · Work items: `docs/program/issues/*.md`

**As of 2026-07-21 ~12:30Z. Operator is BACK and engaged.**

## Standing state

- Fleet is **SINGLE-THREADED**. Nothing is owed to any ring; every idle ring
  is **correct**, not a stall.
- `origin/main = 4a06cf90`. Nothing is blocked.
- **Do not kick a WP while the operator has an open question below.**

## Operator rulings — 2026-07-21 ~12:45Z. SETTLED, do not reopen.

**On Linux ABI II** (`research/linux-abi-ii-work-program-proposal.md`):

- **No "ratification."** The charter is a **planning document, not a
  commitment**. Nobody outside the project is watching, and nothing depends
  on our timelines or stated intentions. **I had imported a governance ritual
  that does not apply — do not re-raise status-correction as a decision.**
- **Where there is a gap between what was anticipated and what was done, fill
  the gap first.** Hence `docs/program/10-linux-abi-completion.md`.
- **L2-1: no cross-compilation. CROSS-PLATFORM IS INDEFINITELY DEFERRED**
  (restated by the operator 2026-07-21 after I re-raised it). A very late
  feature, behind a long line of other work. Manifest v2 = family-scoped and
  generated, **not** cross-target.

  ⛔ **This ruling ALREADY ANSWERS any non-linux finding — do not route one
  back as a scoping question.** I did exactly that with "`ken-host` has never
  compiled on any non-linux target" (28 `cfg(not(target_os = "linux"))`
  fail-closed sites, never built, `abi_v1.rs:747`). **Under this ruling that
  finding is inert**: the lane is deferred, nothing builds it, and the defect
  cannot bite. It is dead code for a deferred target, not an open decision.
  Record such findings as *observations against a deferred lane* and stop —
  a settled ruling is a fixed input, never a question to re-ask.
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

## ✅ TRACK Q IS COMPLETE — nothing outstanding (2026-07-21 ~21:55Z)

**Q-RESIDUE merged: `origin/main @ 64337192` (PR #818)**, from
`wp/Q-RESIDUE-test-rework @ 3f752451`. Verified on main **by content** (the
crate diff against the approved SHA is empty), not by the publisher's exit
code. Issue closed, all three retros in, adversary notified
(`evt_4g7qasxqdy5s8`). Q1 and Q2 merged earlier the same day.

**The fleet is idle and home-clean, and that is CORRECT** — the queue below
is unstarted pending the operator's direction. Do not kick any of it
without their go.

> ### ⚠ THE STALL THAT LOOKED LIKE QUIESCENCE — expect this again
>
> Track Q sat still for ~20 minutes not because it was done but because
> **the architect's composer had two Q-RESIDUE vote requests stacked as
> `[Pasted Content …]` and never submitted.** No `Working`, no turn, no
> vote, and nothing in the system surfaces it. A bare `Enter` released it.
>
> **"The fleet is quiescent" is an observation with at least two causes.**
> Single-threading makes idle rings normal, which is exactly what
> camouflages a wedged pane. On any quiet tick, `capture-pane -S -200` the
> architect specifically and look for stacked pastes — the composer strands
> there repeatedly. See `sweep-wedged-panes-misses-stacked-paste-form`.

> ### ⚠ AND A GREP THAT NEARLY MIS-REPORTED THE MERGE
>
> Verifying the merge, `grep -c 'examples.len() == 5'` on main returned
> **1** — reading as "the frozen count survived." It had not: the match was
> inside a **comment documenting its removal**; the live assertion is
> `!examples.is_empty()`. **Grepping a name instead of the mechanism** —
> the precise failure class this WP existed to remove, committed while
> verifying the fix for it. Read the context, never the count.

> ### ★★ THE RESULT WORTH KEEPING FROM THIS WP
>
> **AC-2's mutation proof caught a bad test before it shipped, on its first
> application.** A first draft of the settlement-ordering test
> *"only hand-sequenced the two helper functions instead of invoking the
> real `unsafe extern "C"` entrypoint, so it wouldn't have caught a real
> regression."* It would have sat **green through an actual defect** — a
> test exercising a **proxy** instead of the **mechanism**, exactly the
> class this WP existed to remove.
>
> The final discriminator is confirmed **three times independently**:
> implementer's authoring run, QA's independent re-execution on the branch,
> and the captured panic (`abi_v1.rs:1590`, `left: 0, right: 1`).
>
> **Require the mutation proof on every future test-rework WP.** It is the
> only step that distinguishes a grounded assertion from a green one.
>
> ⚠ **But do not over-read the three runs.** They are three confirmations
> of **one** discriminator, not three discriminators — all three flipped
> the same seam. A wrong seam produces exactly this agreement. Same shape
> as `differential-oracle-is-blind-to-a-shared-premise`. Raised with the
> adversary at merge; unresolved by design, it is theirs to attack.

> ### ⚠ I MIS-READ THIS WP'S DIFF AND TOLD THE FLEET
>
> I flagged `abi_v1.rs` (+71) as a **production-surface ABI change** in a
> test-rework WP. It is **entirely inside `mod tests`.** I inferred it from
> the file path and line count without opening the diff — same shape as the
> §1 CI misdiagnosis. I corrected it in the flag itself and told the ring to
> **vote on the branch, not on my summary.** They did, and grounded their
> review against real code (`abi_v1.rs:824-841` + the C call site).

## ✅ TRACK Q — DONE (2026-07-21). Only Q-RESIDUE remains, and it is an S.

**Q1 landed. Q2 complete: 428 triaged, 100% classified, six rings in
parallel.** Result: `docs/program/qa-triage/FINDINGS.md`.

| class | count | share |
|---|---:|---:|
| durable-invariant | 392 | **91.6%** |
| compat-vector | 19 | 4.4% |
| transition-sentinel | 7 | 1.6% |
| UNCLASSIFIABLE | 10 | 2.3% |

**Q3–Q7 folded by the operator into ONE S** — `docs/program/issues/Q-RESIDUE.md`
(status `ready`, owner `runtime`). **Not kicked; awaiting operator go.**

> ### ★★ Q4 AND Q7 WERE EMPTY — THE LESSON, NOT JUST THE RESULT
>
> 147 tests flagged for asserting an outcome without naming the variant:
> **every one sound.** All 27 wall-clock flags: **sound.** Both tracks had
> been sized from **scan hit counts**, and hit counts carried almost no
> signal about defects. Authorizing Q3–Q7 off the totals would have reworked
> ~300 correct tests.
>
> **⛔ Do not re-derive Q-RESIDUE's scope from `scripts/qa-risk-scan.py`.**
> It emits a **review queue, not a defect list**. The inventory in the issue
> is the whole of the work.

> ### ★★ THREE DEFECTS IN MY OWN INSTRUMENTS — ALL FOUND BY OTHERS
>
> 1. **The scanner fabricated a test.** An unanchored `#[test]` matched the
>    attribute *in prose* (`rt_parity_native.rs:3` is a doc comment). The
>    phantom swallowed 480 lines of helpers. **Foundation found it by
>    reading source.** `--self-test` passed throughout — it only checked
>    files that HAVE the patterns, never one that would INVENT a row. It now
>    has a negative arm (`NEVER_A_TEST`).
> 2. **"Two counts agreed" was an echo, not corroboration.** I cited the
>    scanner reproducing the documented 1909 as proof it wasn't dropping
>    tests. Both used the same naive match, so both counted prose mentions.
>    True total **1905**. A differential oracle is blind to a shared premise.
> 3. **The aggregator read the wrong file** — Ergo parsed 23 vs a reported
>    71 (glob matched QA's partial share, not the leader's assembly). Caught
>    only because the leader's own count disagreed.
>
> **Every one was caught by an INDEPENDENT source, none by my own checks —
> which were built on the same premises as the things they checked.**

> ### ★ TRANSPORT: a mention proves the EVENT exists, never that it was READ
>
> **Five of six Q2 kickoffs silently failed to deliver.** Repair: `tmux
> send-keys` a pointer to the `evt_…` (point at it, never restate it).
>
> **It reproduces INSIDE a ring.** Kernel stalled at 50/72 — its leader
> delegated, the implementer never got it, and the leader went idle
> believing it had handed off. **Silence and done look identical from here.**
>
> **⛔ Do NOT detect "working" by grepping a spinner word** — the verb is
> randomized ("Gitifying…", "Calculating…", "Crunched for…"). Key on the
> duration/token signature: `\([0-9]+m? ?[0-9]*s · [^)]*\)`. Grepping a
> fixed word read all six busy leaders as dead and nearly caused six
> duplicate re-rouses.

## ▶ THE DOC PROGRAM IS RUNNING — a SECOND, CONCURRENT track (2026-07-21 ~22:5xZ)

`origin/main @ 7610d2a1`. **`DOC-W0` is `active`** and released to a new
**doc team** (`evt_1m7j5qvvm2p2m`). This is the fleet's **one standing
exception to single-threading** (operator): the doc track runs **concurrently**
with build work, because doc WPs touch `library/` and `agent/`, not `crates/`.
**The exception is contention-free-ness, not priority** — a doc WP that would
touch a path a build WP holds defers.

| seat | tier | agent id |
|---|---|---|
| `doc-leader` | T2 Sonnet 5 | `agt_37w6sznc4nw00` |
| `doc-author` | T2 Sonnet 5 | `agt_37w6t02849400` |
| `librarian` | **T1 `gpt-5.6-sol`** | (existing) — the team's **QA** |

**★ Judgment is concentrated on the REVIEWING end, not the authoring end** —
inverted from every other team, deliberately. Documentation fails by being
*confidently wrong*, not badly written; a page whose citation does not carry
its claim reads perfectly. That is a grounding problem, which is where T1 pays.

Frame: `docs/program/12-documentation-program.md` (§0 team, §1 four **settled**
decisions — do not reopen). Overlays: `agent/teams/doc/{leader,implementer}.md`.
There is **no `doc-qa` seat** and no `agent/teams/doc/qa.md`.

**Seat provisioning, if it ever needs repeating:** `moot init` is NOT usable —
its only incremental option is `--force`, which rotates keys for **all**
already-adopted agents and would kill every live seat. Use the API directly:
`POST /api/agents` → `POST /api/registration-tickets/{id}/exchange` for the
plaintext key → write into `.moot/actors.json` (gitignored). PAT is at
`.mootup/credentials`. OAS: `local/refs/convo/docs/api/openapi.yaml`
(operator-sanctioned read; clean-room bars `local/refs/` for *writing Ken's
code*, which this is not).

## My queue, in order

0. **BUDGET-EFF** — Handoff Gate the Spec enclave (spec-leader, spec-author,
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

**`DOC-W0` — doc team, active.** Nothing else. Branch aligned on
`origin/main`, clean, no orphaned polls. Build fleet idle and home-clean,
which is **correct**, not a stall.

**Also queued, not started: `Q-CLAIM-CLOSURE`** (`issues/Q-CLAIM-CLOSURE.md`,
`ready`, owner runtime) — the adversary's post-merge findings on Q-RESIDUE.
Advisory, **no live defects**. Its generator is worth reading before framing
any future rework WP: *the ACs took the TEST as the unit when the load-bearing
unit was the CLAIM*, so a rework could strengthen one claim, mutation-prove it,
and silently drop its siblings while fully satisfying the criteria. R1 (ABI
fact inventory has no independent anchor — both sides of the check come from
one generator) is the one to sequence first.

**CLOSED — not a scoping question.** `ken-host` has never compiled on any
non-linux target (`abi_v1.rs:747`, `?` on an `Option` in a `Result`-returning
fn; pre-existing since PX5 `049628f8`, adversary confirmed by extracting and
compiling it). 28 `cfg(not(target_os = "linux"))` fail-closed sites, never
built. **Cross-platform is indefinitely deferred (operator) — so this is dead
code for a deferred lane and cannot bite.** Recorded as an observation; no
action, no decision pending. See the L2-1 ruling above.

Recently landed and verified by content: **#818** (Q-RESIDUE), **#819**
(Track Q closeout), **#820** (doc program frame), **#821** (doc team),
**#822** (librarian T1 + DOC-W0 release).

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

> ### ⛔⛔ `git maintenance` CAN STARVE THE WHOLE BOX — config now guards it
>
> **2026-07-21: `git pack-objects` consumed all 8 cores; load hit 14.** Root
> cause is structural and will recur if the config is ever reset:
>
> ```
> maintenance.lock  -> .git/worktrees/<name>/maintenance.lock   PER-WORKTREE
> gc.pid            -> .git/gc.pid                              shared
> objects/          -> .git/objects                             shared
> ```
>
> **`git maintenance` locks PER-WORKTREE but repacks the SHARED object
> store.** A run in one worktree is invisible to a run in another, so the
> concurrency ceiling is **the worktree count — 30**, each defaulting to all 8
> cores. The legacy `git gc --auto` path did *not* have this hole (`gc.pid` is
> common, so the second bails). `git maintenance` lost that protection.
>
> **Guard now set repo-locally** (covers all worktrees via the shared store):
> `maintenance.auto=false`, `gc.auto=0`, `pack.threads=2`,
> `pack.windowMemory=256m`. **`maintenance.auto=false` is the load-bearing
> one** — capping threads alone still allows 30 × 2 contending.
>
> **Consequence you are now carrying:** loose objects accumulate forever.
> A deliberate `git gc` is needed during a genuinely quiet window (fleet idle,
> no WP in flight). **Never run it while a team is working.**
>
> ⚠ **Trigger to avoid:** a `git add -A` run from `/workspaces/ken` (the MAIN
> worktree) sweeps untracked `.cache/`, `.targets/`, `.tmp-*` into the object
> store, blowing past the loose-object threshold and firing maintenance from
> every worktree at once. **A `cd` chained before a broad `git add` silently
> changes which repo it applies to.** Those blobs are still present as
> unreachable objects pending a prune.

> ### ⛔ A PANE SNAPSHOT IS NOT AGENT STATE — three variants seen in ONE day
>
> | symptom | actual state | repair |
> |---|---|---|
> | stacked `[Pasted Content …]`, no `Working` | alive, **never submitted** | bare `Enter` |
> | pane **entirely blank** (even `-S -200`) | alive, blocked on a **consent modal** rendered at the buffer's START | capture from `-S -` and `grep -v '^\s*$'`, then `Enter` |
> | empty prompt, looks idle | **actively working** — narrow `tail` caught a gap between renders | capture WIDE before repairing |
>
> The third one bit me *while running the check designed to catch the first*.
> Had I trusted it I would have stacked a duplicate kickoff on a working seat.
> **Always `capture-pane -S -` piped through `grep -v '^[[:space:]]*$'` before
> concluding anything about a seat.** A new seat's first launch is exactly when
> nobody is watching — the consent modal for `--dangerously-load-development-channels`
> blocks silently and indefinitely.

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
