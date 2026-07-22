# Current briefing (live — read this first on every Steward resume)

> **This file is LIVE STATE ONLY.** When something here stops being true,
> move it to `diary/YYYY/Mon/DD.md` — do not append a newer block above it.
> Appending is what grew the old tracker to 2.23 MB.
> History: [`INDEX.md`](INDEX.md) · Work items: `docs/program/issues/*.md`

**As of 2026-07-22 ~13:00Z. OPERATOR IS PRESENT.**

## Standing state

- **`origin/main = 7c6e03c8`.** Green.
- **★ FOUR TRACKS RUN CONCURRENTLY** (operator directive 2026-07-22 ~12:30Z —
  *"double the rate of use… as soon as we can parallelize to two tracks on the
  implementation side, we should"*). **Idle is not the default-correct state.**

| track | ring | work | state |
|---|---|---|---|
| impl 1 | runtime | **RT-SPLIT slice 7** — the last slice | active |
| impl 2 | verify | **BUDGET-EFF** | active, `ken-host` landed |
| doc | doc | **DOC-W1-2** (Wave 1, slice 2 of 5) | active |
| spec | enclave | **ABI-REVOKE behavioral contract** | active |

- **Build side is capped at TWO implementation tracks** (operator). Doc is the
  standing exception; the enclave is not a build team, so it does not count
  against the cap.
- **⛔ The binding constraint is NOT the two-track cap — it is the READY
  QUEUE.** `kernel`, `language`, `ergo`, and `foundation` are all idle and have
  **zero** ready items owned by them; every ready WP is runtime-, doc-,
  spec- or steward-owned. **Opening a third implementation track requires
  authoring WPs for those rings, not operator permission.** That authoring is
  T1 enclave/Steward work and is the real cost. **Surface this to the operator
  before promising throughput the queue cannot supply.**
- **Do not kick a WP while the operator has an open question below.**

### ★ QA bound-verdict attestations are now on `origin` (2026-07-22 ~12:53Z)

The commit-your-verdict workaround (used when a QA seat's convo outbound dies)
left verdicts on **one local ref in one clone** — `a4473ab0` had **no second
copy anywhere off this box**, and `handoff-gate-compact.sh` has already
hard-reset that exact branch once (`preserved/runtime-qa-work-7c86db36`).
Pushed to durable refs, each verified by `ls-remote`:

```
attest/runtime-qa-verdicts   a4473ab0   (53501ffe is an ancestor — both carried)
attest/ergo-qa-verdicts      cf791c7f
attest/verify-qa-verdicts    04efa001
```

**This fixes durability, NOT discoverability** — a branch name nobody watches
is still a pointer someone must deliver. **Transcribing these into the repo
proper is the actual close and needs a WP.** ⛔ Do not reset, clean, or
re-anchor `runtime-qa/work`, `ergo-qa/work`, or `verify-qa/work`.

★ **The transferable lesson:** *when a workaround relocates a failure mode, the
new location inherits none of the scrutiny the old one had.* The pattern fixed
a real selection error and quietly moved the fragility from the message layer
to the storage layer, where nobody was looking.

> ### 🚨 INFRA ESCALATION FOR THE OPERATOR — `runtime-qa` convo outbound is DEAD
>
> **Inbound works** (it receives mentions and reviews normally); **it cannot
> POST.** Unchanged across 4 watchdog ticks and 2 `/mcp` reconnect attempts.
> Server-side is healthy — every other seat posts normally.
>
> ⛔ **Do NOT relay its self-posts to close a provenance gate.** The gate exists
> precisely so an attestation is *not* Steward-sourced; a relayed self-post is
> a contradiction and banks the gap permanently.
>
> **The working path, and it is better than a relay:** QA **commits its verdict
> to its own branch** through the shared clone, and reviewers read it by object.
> Slice 5 closed this way — `53501ffe`,
> `docs/program/qa-triage/RT-SPLIT-slice5-runtime-qa-verdict.md`, binding
> `APPROVE` on `744bda14` / base `1f70a71b`. Precedent existed already
> (`ergo-qa @ cf791c7f`, `verify-qa @ 04efa001`). **A relay verifies the
> selection; a commit eliminates the selection.**

### ▶ Build track — RT-SPLIT (Runtime ring) — **SLICE 7, THE LAST ONE**

**Slice 6 merged `@ 7c6e03c8` — six of seven done, budget 22/24, retros in.**
`cranelift_backend.rs` is down to **1,445 lines** from 22,081; `core.rs` and
`lowering/mod.rs` were **0-byte diffs** (the no-re-touch rule has now held
across five consecutive slices).

⛔ **Slices 6 and 7 were REVERSED mid-series** (Architect `evt_2j4gnwffr7h63`):
`artifact::api` led, artifact internals follow. Artifact-first would have cost
7 `pub(super)` widenings (22→29 vs cap 24) because api-destined callers were
still in the residual parent — **a visibility forecast from the FINAL module
graph says nothing about the TRANSITIONAL graph a slice compiles against.**

**Slice 7 is kicked** (`evt_1aj141722jfq3`) on `wp/rt-split-7-artifact-internals`
off `7c6e03c8`. Four things bind it:

1. **Delete all six scaffold imports in `artifact/mod.rs`** — and **`api.rs`
   must NOT change at that point. If it does, the scaffold was wrong.** The
   adversary will test exactly this on merge; it makes slice 6 retroactively
   falsifiable by slice 7's diff.
2. Module comment → *"final ruled users span `lowering` and `artifact::api`"*.
3. **Re-run the final-destination census** — recompute from rows in code with
   the blocking assertion; do not implement the number the frame states.
4. **Placement changes ONLY on a changed DIRECT-use population** — the
   quantifier, not a paraphrase.

**`cranelift_backend/**` is slice 7's exclusive territory**, and its landing
**releases Verify's deferred BUDGET-EFF half** (the native reifier in
`lowering/core.rs`). Slice 7 is therefore the release point for the other
track, not just the end of this one.

### ▶ Build track 2 — BUDGET-EFF (Verify ring)

**Scope split ruled `evt_2sw8883abc3m4`** after I asserted path-disjointness
**from a string literal** and `verify-implementer` caught it (my grep matched
`"ctor:prelude::TransferCount::PrivateTransferCount"` — a constructor *name*
inside quotes — while the real reifier sits in `cranelift_backend/lowering/`).

- **In scope:** `ken-host`, `ken-interp`, **AC-3 oracle rewrite**
- **⛔ Deferred until slice 7 lands:** anything under `cranelift_backend*`

`ken-host` piece landed on `wp/BUDGET-EFF @ f4a86050` — two-field inseparable
`TransferCountV1` carrier per Architect ruling `dec_1m6xdwjp2ttyn`, 46/46 green.
`ken-interp` in progress.

★ **Load-bearing oracle shape:** capped-full alone (raw 8, effective 4, count 4
→ remaining 0) is satisfiable by the wrong shortcut `effective := count`. The
suite needs **both capped-full AND capped-short** to discriminate.

★ **RULE ADOPTED:** *path-disjointness is re-derived BY THE RECEIVING RING at
pickup and reported before implementation starts — never asserted by me in a
kickoff.*

> ### ⛔ SLICE 6 IS BLOCKED ON THE ARCHITECT'S FIDELITY REVIEW, NOT ON THE RING
>
> The **12-item frame sweep is written and committed** (`06d72bde`). It folds
> the post-slice-5 rulings into
> `docs/program/wp/rt-split-cranelift-backend.md`. **@architect reviews the
> exact published SHA for fidelity; then `runtime-leader` may kick slice 6.**
>
> **The three rulings that reshape slice 6:**
>
> 1. **`verify_cranelift_function` is LOWERING-owned** (`evt_3tgaw9ws44fqg`),
>    not artifact-owned. It has exactly **one** production consumer. The two
>    test adapters are therefore **symmetric across the boundary** — the JIT
>    bridge in `artifact/mod.rs` (slice 6), the verifier bridge
>    `verify_cranelift_function_for_artifact_tests` in `lowering/mod.rs`
>    (landed slice 5). ⛔ **Slice 6 must NOT re-touch `lowering/mod.rs`** — the
>    transitional facade alias exists so it does not have to.
> 2. **Classify by COMPILATION REACH, never by a production/`cfg(test)`
>    binary** (`evt_2mexay4h5tr6y`, frame §10.4b). A predicate satisfiable with
>    `test = false` is production-reachable, **and a feature name containing
>    `test-support` does not change that** — `ken-cli` enables
>    `px8-ds-test-support` on its ordinary `ken-runtime` dependency. Every sweep
>    run against this file — mine, the adversary's, the implementer's —
>    partitioned on the two-cell binary and was blind to the third cell in the
>    same way.
> 3. **Residual-parent closure is TWO-LAYERED** (frame AC-9): a full
>    source-coverage partition **and** a macro-aware declaration/`cfg` ledger,
>    plus a configuration matrix. **Neither substitutes for the other** — line
>    coverage passed a semicolon mis-split; a declaration regex missed the
>    indented `thread_local!` statics. ⛔ **Every "exactly two" / "closed by
>    construction" claim is WITHDRAWN.**

> ⚠ **A gate that cannot see its own subject.** `with_px8ds_retired_flat_order`
> is bare `pub`, re-exported via `lib.rs:39`, consumed cross-crate by
> `crates/ken-cli/tests/px8ta_oriented_subcontinuation.rs`. The feature is
> **default-off**, so AC-2's rustdoc dump reads 338→338 whether the facade
> re-export is right, wrong, or **missing** — *incapable*, not weak — and
> `-p ken-runtime` compiles the block away. **Omitting it leaves every local
> gate GREEN and another crate broken, caught only by CI.** Only local
> evidence: `scripts/ken-cargo test -p ken-cli --test
> px8ta_oriented_subcontinuation`. ⛔ **NOT `--workspace`.**

### ▶ Doc track — WAVE 1 IS UNBLOCKED

**`DOC-CURRENCY-ANCHOR` is CLOSED** (acceptance re-derived against the landed
`scripts/gen-doc-status.sh` on `origin/main`, not from a working tree). The
outage that made `main` red on its own documentation gate is resolved; the
three-fold history is in [`2026/Jul/22.md`](2026/Jul/22.md) and the closure
record. **`DOC-W1` is now ungated** — its `status: draft` means *framed and
releasable*, and it needs the handoff gate before a kick.

### ⏭ Steward's own queue under the away-window directive

1. ✅ **DONE — Waves 1 and 2 framed** (`issues/DOC-W1.md`, `issues/DOC-W2.md`,
   `origin/main @ a87e1cae`). Both `status: draft` meaning **GATED, not
   unscoped** — the schema has no `blocked` state. Wave 1b and
   `tasks/ffi-and-platform.md` are deferred with reasons stated in-file.
2. ✅ **DONE — the six-wave body** is `12-documentation-program.md` §4a/§4b/§4c.
   §4a is the load-bearing part: DOC-W0's eight findings tabulated as one
   defect class, with the three carries every wave frame inherits.
   **⏭ Remaining: Wave 1b and Waves 3–6 are a MAP, framed only when their
   predecessor's exit condition is actually met.**

> ✅ **PUBLISHED — the §7 *ledger-is-an-output* heuristic is on `main`**
> (verified by CONTENT at `docs/program/wp/rt-split-cranelift-backend.md:329`,
> not by branch-ahead). The old "held, unpublished" note is discharged.
> The duty-assignment lesson that sat here has moved to
> [`2026/Jul/22.md`](2026/Jul/22.md); its rule is durable: **an instruction
> whose verb names a capability I hold and the recipient does not is mine to
> execute** — *push, compact, merge, mint*.

3. ✅ **`issues/ABI-REVOKE.md` — FRAMED**, and it is **not shovel-ready by
   design.** Grounding found a blocking prerequisite the charter does not
   name: `62 §4` ties the membrane to a controlling **`36 §4` space cell**,
   and **no `36 §4` space exists in any runtime crate.** The `Space` in
   `ken-runtime/src/store.rs:198` is the **`44 §3` arena reclamation unit** — a
   different concept sharing the word, which greps positively and is why the
   charter's *"fold into PX7"* read as plausible. **Routed to the Architect for
   a design pass + ADR before any sizing.** PX7's generation table guards
   use-after-close; revocation must deny a handle that is **still valid**.
4. `BUDGET-EFF` — shovel-ready, queued behind RT-SPLIT, ahead of ABI-M1.
5. ⏭ **Queued small fix:** rustfmt drift at `crates/ken-runtime/src/store.rs:602`
   — **pre-existing on `origin/main`**, verified there and not introduced by
   RT-SPLIT (the implementer reverted rather than bundling it — correct call).
   One hunk. **Touches `crates/` ⇒ needs the normal ring gates**; a standalone
   item behind RT-SPLIT, never a rider.
6. ⏭ **`agent/COORDINATION.md §12a` — `git stash` is now fleet law.** The stash
   stack is **one stack shared by ~70 worktrees**; a bare `pop` takes whichever
   agent parked last. ⚠ **Do not reap the existing entries** — they belong to
   other seats.
7. ⏭ Sweep ~20 stale `/tmp/ken-*` worktrees. **ASK before touching any with
   tracked changes.**

### ⛔ PUBLISH DISCIPLINE — tightened 2026-07-22 after invalidating a Decision

I moved `origin/main` under RT-SPLIT slice 2's merge Decision **22 seconds**
after it opened, with a docs-only publish. Third occurrence. **`list_decisions`
run at the top of the work answers a question about the wrong moment**, and no
adjacent check catches a 22-second race. **So the trigger is earlier: hold
Steward publishes whenever a build ring holds a QA-APPROVED CANDIDATE, not
merely an open Decision.** Re-check immediately before the publisher call; if a
publish is urgent, **announce the window in-channel first.** Path disjointness
is irrelevant — §14 is about identity, not conflict.

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

## The completion program — written, NOT started · COVERAGE VERIFIED

`docs/program/10-linux-abi-completion.md` — **on `main`**. Four tracks:
**ABI-R** reconcile, **ABI-A** availability promotion, **ABI-M** manifest
(native-target only), **ABI-S** synchronous floor, plus **Track T** the
committed exit (PX10/PX11/PX12).

> **⚠ IDs RENAMED 2026-07-22 (operator).** Tracks R/A/M/S now carry an
> **`ABI-` prefix** — the bare `A3` collided with `issues/A3.md`
> (catalog-coverage walker) and `R1`-`R3` collided with the adversary's
> finding labels. **`PX9`-`PX12` keep their charter IDs.** `L` was rejected
> as a prefix: `L1`-`L7` are existing WPs.

> **★ COVERAGE ANSWER (operator asked 2026-07-21; verified file-by-file):
> 0 of 18 items have an issue.** The only live node of §5's graph is
> **PX8**, its *root*. Everything downstream of `PX8 -> ABI-R3` and
> `PX8 -> PX9` is unframed. §9 of that document is the record.
>
> **AND the document had a hole:** the charter's **runtime revocation
> membrane** (`09` §5) is absent from it. `RevocationHandle { revoked: bool }`
> (`ken-elaborator/src/capabilities.rs:256`) is still the static contract —
> **its own doc comment says the runtime membrane is DEFERRED** — and there is
> **zero** revocation code in `ken-host`/`ken-runtime`/`ken-interp`/`catalog`.
> PX7's generation-checked handle table is a *different* property
> (use-after-close, not withdrawal of delegated authority). **L2 assumes it**
> (§8.1 gate 9). **AWAITING OPERATOR: fold into Track ABI-S / split as its own
> WP / accept as a known limitation.**

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
   ⛔ **AC-3 REWRITTEN — my earlier pin was VOID.** The R1 oracle's conclusion
   compares two values computed from its OWN constants (`RAW-count` vs
   `effective-count` = `4 == 0`) and **never reads a reifier field**, so it
   fails on ANY implementation and its failure at `e892777c` confirmed
   nothing. "Must pass unchanged" is **withdrawn**; the oracle must be
   **rewritten to observe the mechanism**. ⛔ **"Confirmed by execution" was
   also FALSE** — the defect rests on **source inspection** of the two
   reifiers. ★ **An adversary repro demonstrates a defect; a completion oracle
   defines correctness. Pinning does not convert one into the other — verify
   an oracle observes the mechanism BEFORE making it an AC.** Detail:
   `docs/program/wp/BUDGET-EFF-remaining-bounded-by-effective-request.md`
2. **SEAL-2** — re-anchor on current `main`; evidence
   `adversary/SEAL2-repros@70a603da`.
3. Then STR-BIJ → enclave (`ready`, S) · F1 → Architect (`ready`) · F3 ·
   A3 · F4 · RT-SPLIT (**L**, 22k-line `cranelift_backend.rs`).

> ### ⚠ WHERE THIS QUEUE CAME FROM, AND THE `#N` TRAP
>
> **These items are the gap between what was actually DELIVERED and what
> `research/linux-abi-ii-work-program-proposal.md` ASSUMES was already done**
> (operator, 2026-07-21). That proposal is the second Linux-ABI campaign; this
> series fills the hole in front of it. Read the proposal before sizing any of
> them — an item only makes sense against what it assumes exists.
>
> **`#37` / `#39` are indices in a PRE-RESTART STEWARD TASK LIST. They are NOT
> GitHub issue numbers.** Six issue files asserted them as `github:`
> references and the tracker propagated that into a dedicated GitHub column,
> where a task-list index read as a verified external reference — and
> `github: 38` pointed at whatever real issue #38 happens to be. **Corrected:
> `github: null` on both survivors**, with the provenance stated in-file.
>
> **`#38`/`#32`/`#24`/`#25` are DROPPED** — they carried nothing but a number
> (operator: *"no use to anyone"*). Do not resurrect them; there is nothing to
> resurrect. The old `GH-` filename prefix baked the wrong origin into the
> identifier itself — `identifiers-are-claim-artifacts`, in a schema field.

**Readiness is thin behind BUDGET-EFF — only two items are releasable.**
STR-BIJ (`ready`, enclave) and F1 (`ready`, runtime → Architect first).
The rest are `draft`; **A3 has no owner, no size, and no brief** and blocks
F4. ⚠ Verify "no brief" claims by *reading `docs/program/wp/`*, not by
globbing on the ID — the F3 brief is `F2F3-reducer-degrade.md` (it covers F2
and F3 together) and I mis-reported it as missing once already.

## In flight

**`DOC-W0` — ✅ MERGED `origin/main @ 6be9754b` (PR #830), 2026-07-22 ~01:43Z.**
Verified by content: all 8 blobs byte-identical to reviewed `d56abbb1`;
`revision_resolved` in `scripts/gen-doc-status.sh`; both shallow-clone
regressions present by name. The fleet's first `library/` tree. Retros were
being collected at time of writing — **check they are in before treating the
issue as closed.**

**Nine review rounds, six findings, and NOT ONE was a different kind of
mistake.** Every one was a **proxy standing in for the property**:

| # | proxy checked | property that mattered | found by |
|---|---|---|---|
| 1 | rejects a *fake* revision | **accepts a real one, in CI's env** | CI (red) |
| 2 | test clones `file://{repo_root}` | an **independent** history source | librarian |
| 3 | `cat-file` says object present | present **AND** ancestry provable | architect |
| 4 | symlink not *discovered* | symlink **rejected and reported** | architect |
| 5 | SHA reviewed + approved | SHA **on `origin`** (see below) | steward |
| 6 | process fix *agreed to* | seat **can perform it** (see below) | doc-author |

**5 and 6 were mine.** #3 held because the Architect built an isolated depth-1
probe rather than reasoning from source. **What finally stopped the recursion
was naming the predicate once** (`revision_resolved()` = object present AND
ancestry provable) and deriving self-heal, every deepen checkpoint, the
unshallow fallback, and all diagnostics from it — not any individual fix.

**⇒ Carry for DOC-W1 and every gate after it: when a gate depends on an
environment property (history depth, credentials, checkout topology), state
that precondition as a NAMED PREDICATE before writing the check.** A gate whose
precondition is unwritten gets discovered one CI-red at a time, each round
closing an instance and leaving the next layer live.

**New behavior worth watching:** `gen-doc-status.sh` now performs **network
fetches inside a test gate**. Fail-closed-on-unreachable-origin was verified,
but hermeticity under a flaky remote was not reasoned through. Flagged to the
adversary at merge.

**`SPEC-38-ERRATUM` — CLOSED.** Merged `origin/main @ e5a400c7` (PR #827),
retros in. Enclave carry: *keep semantic target / conformance oracle /
implementation mechanism as **separate scopes**; re-anchor with both
current-base and reviewed-subtree byte-identity checks.*
**This unblocks `BUDGET-EFF`, which stays PARKED pending operator go.** The
closure-mechanism call (reply-carries-effective vs. host-caps-the-request-
record) is an **Architect** decision routing *with* that release, not before.

**⛔ `BUDGET-EFF` AC-3 was UNSATISFIABLE and is corrected on `main`.** `count`
cancels on both sides, reducing it to `8 == 4` — no implementation could ever
have discharged it. **And it cannot be fixed in place:** `remaining` does not
occur in `ken-host/src/effect_v1.rs` where the oracle lives; it is built at
`ken-interp/src/eval.rs:4934-4935` and
`ken-runtime/src/cranelift_backend.rs:13081-13082`. The rewrite is **two new
tests** — budget it as plumbing. **The R1 defect itself still stands** on
source inspection; only its demonstration was broken. Branch aligned on
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

Recently landed and verified by content: **#827** (SPEC-38-ERRATUM →
`e5a400c7`), **#828** (AC-3 correction + ABI gap status → `9fb90aab`),
**#818** (Q-RESIDUE), **#819**
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
> A deliberate `git gc` is needed during a genuinely quiet window (fleet
> idle, no WP in flight). **Never run it while a team is working.**
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
> **★ The third one recurred TWICE on 2026-07-22** — once reading `doc-author`
> as never-engaged (it had already finished), once rousing it *while it was
> mid-fix* with `is_symlink_escape` already in its diff. **Interrupting a
> working agent is forbidden by the playbook and I did it anyway**, because a
> `tail -4` showed a bare `❯`. The spinner renders **above** the prompt. A
> `WORKING` check must grep the whole capture for `esc to interrupt` — never
> judge from the last few lines.
>
> The third one bit me *while running the check designed to catch the first*.
> Had I trusted it I would have stacked a duplicate kickoff on a working seat.
> **Always `capture-pane -S -` piped through `grep -v '^[[:space:]]*$'` before
> concluding anything about a seat.** A new seat's first launch is exactly when
> nobody is watching — the consent modal for
> `--dangerously-load-development-channels` blocks silently and
> indefinitely.

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

## ⛔ "COMMITTED" IS NOT "REACHABLE" — publish, then verify ON MAIN

**2026-07-22, caught by the adversary, not by me.** I corrected `BUDGET-EFF`'s
AC-3, announced *"all four folds are in"*, and it was true **about
`steward/work` only**. `COORDINATION §15` sends rings to **`main`**, so for
the whole window a ring picking up the WP would have read the **unsatisfiable**
AC-3. Five commits, zero publishes.

**The rule, mechanically:** a Steward doc edit is not done at `git commit`. It
is done when `git grep '<plain phrase>' origin/main -- <file>` returns the new
text. Route via `§6a` (corpus branch off *current* `origin/main` → publisher
path → **verify by content**).

Two amplifiers, both real:
- **The publisher prints `merge command succeeded` on a failed push** — that is
  the open `PUB-VERIFY` issue. Its exit code is worthless; only content on
  `main` counts.
- **`git grep` is case-sensitive and false-negatives on a phrase spanning a
  line break.** Three greps false-negatived on 2026-07-22 alone. Grep a short,
  lowercase, single-line fragment — never a sentence, never across `**bold**`.

Same family as the whole week: **verify the mechanism, not a proxy.**
"Committed" is the proxy; "on `main`" is the mechanism.

## ⛔ A `git_request` SHA IS *REVIEWED*, NOT *PUBLISHED* — `ls-remote` FIRST

**2026-07-22, DOC-W0, caught with ~30 seconds to spare.** `doc-leader` sent a
`git_request` for `d56abbb1`. **`origin` was still at `8f14ff83` — the exact
SHA that had already failed CI on PR #830.** Four folds lived only in
doc-author's worktree. Running the publisher as requested would have rebuilt
the known-red commit and looked like a *regression of an already-fixed bug*.

**Why nothing upstream catches it:** the agent worktrees **share one object
store** (`/workspaces/ken/.git`), so an unpushed SHA resolves **perfectly** for
`git log`, `git diff`, `merge-base --is-ancestor`, `git grep`, and a full
detached test run. **Librarian exact-SHA QA and Architect exact-identity review
both passed on a commit that was not on `origin`.** `git ls-remote` is the
*only* check that separates reviewed from published.

**What actually exposed it:** a **number disagreeing with a report** — my scope
diff printed **900** lines for the gate test where doc-author reported ~1200.
Cross-checking a reported magnitude against my own measurement caught it; no
identity check I ran did.

**⇒ HARD PRECONDITION of every publish, before `scripts/scripted-pr-automerge.sh`:**

```sh
git ls-remote origin refs/heads/<branch>   # MUST equal the requested SHA
```

If it does not match, **push it yourself** — mint via
`.devcontainer/mint-gh-token.sh`, then
`git push https://x-access-token:$TOKEN@github.com/ken-topos/ken.git <sha>:refs/heads/<branch>`,
and **re-verify with `ls-remote` after**. Often a clean fast-forward (the stale
head is an ancestor) — check before assuming a force is needed.

**⛔ DO NOT delegate this push to the authoring seat. NO BUILD SEAT HAS GITHUB
CREDENTIALS** — only the scripted publisher and the Steward. I issued exactly
that carry, doc-author accepted it, then *tested* it and hit `could not read
Username for 'https://github.com'`. **A process rule assigned to a seat that
cannot execute it is worse than the gap it closes** — everyone believes it is
handled. **Verify a seat CAN do a thing before making it their duty.** QA seats
may carry "candidate SHA present on `origin`" as a **detection** item; the
remedy always routes to the Steward.

## Standing discipline

A success signal says a thing **ran**, never that it did what you meant —
**verify by content**. `git rev-parse --abbrev-ref HEAD` must read
`steward/work` before any write. Local builds are **targeted only** via
`scripts/ken-cargo -p <crate>` — **never `--workspace`** (it OOMs the box).
Workspace-green means green in **CI**.
