# Current briefing (live — read this first on every Steward resume)

> ## ⛔ HOW TO READ THIS FILE, AND WHEN TO DISTRUST IT
>
> **`origin/main` outranks this file, always.** ⛔ If anything below tells you to
> do something `git fetch origin` shows as landed, **this file is stale and the
> repository is right.** Re-read fresh, in this order:
>
> 1. `git fetch origin && git rev-parse origin/main`
> 2. the LIVE block below — ⛔ **only** the LIVE block
> 3. the open tasks (⛔ do not re-derive priority from memory)
> 4. for what is HELD, DEFERRED, or WHOSE it is: **the node**
>    (`docs/program/issues/*.md`), its operative block — ⛔ never this file
>
> ⭐ **This file is a resume POINTER, not an archive. Git is the archive.** When a
> window closes its block is **deleted**, not demoted to a "superseded" section —
> ⛔ a superseded block left in the file gets read by someone, eventually.

> ### ⚠ REWRITTEN 2026-07-26 ~19:4xZ — 2866 lines → this. Read the bound.
>
> The prior content (~2700 lines of windows stacked back to 2026-07-21) is at blob
> **`c26ee67f29d42690f461d43fe15e21c2202a31df`** — `git show c26ee67f`. Nothing was
> lost; it was archived to git with this pointer.
>
> ⛔ **HONEST BOUND ON THE AUDIT: I did not read all 2866 lines.** I read every
> heading, the blocks claiming to be authoritative, and then **scanned** the
> remainder for sole-source markers, decision ids, held items, and preserved refs.
> ⇒ **That is a scan, not an exhaustive audit**, and its surface was my own idea of
> what "load-bearing" looks like. A reader who needs something from before
> 2026-07-26 should assume it is in `c26ee67f`, not that it was considered.
>
> ⭐ **What the scan found is why the rewrite was worth doing: two blocks that
> advertised themselves as authoritative were WRONG** (see *Corrections*), and a
> hand-maintained list of 6 preserved refs when origin held **26**.

## LIVE — 2026-08-06 ~20:3xZ · PR #1528 open, CI QUEUED, publisher must be re-run

**Verify `origin/main` before trusting anything below.** At writing it is
`3015aafd`; the merge below will move it.

### The operator ruled. Gate readiness is CLOSED.

Operator, 2026-08-06, verbatim: *"mark the tests to skip (add comment why), land
the PR, and continue work, restoring the tests as work allows."*

The campaign-sizing question — whether the 212-commit branch should have been
restructured — **is still parked with the operator** and blocks nothing.

### BLOCKED ON A GITHUB ACTIONS OUTAGE — not on the candidate, not on a seat

**Incident `qcvjkzcs7j74`, Critical, open since 2026-08-06T15:22Z.** Webhook
triggers throttled to roughly 15%. **The merge cannot land until Actions
recovers**, and no action here changes that.

**The tell, so nobody re-diagnoses it:** PR #1528 has a `ken-ci` **check-suite**
with **no workflow run behind it** and `updated_at` frozen at `created_at`, and
**no workflow run exists repo-wide since 12:11Z**. The workflow is `active`; my
token cannot read `actions/permissions` (403), so repo settings were ruled out
by the repo-wide gap rather than by reading them.

To resume: read the Actions component at
`https://www.githubstatus.com/api/v2/summary.json`, confirm a run finally
exists, then re-run the publisher on the same SHA. If the suite still never
dispatches after recovery, closing and reopening the PR is the standard nudge —
**ask the operator first.**

### State of the publish

| item | value |
|---|---|
| candidate | `21fd46dce478ed3a1a17622c33212e20ca545991` |
| its parent | `b914c7ff`, the previously approved candidate |
| Decision | `dec_4w8wn4ymn32cm` — **resolved**, Architect approved on cast |
| PR | **#1528**, open, `MERGEABLE`, head = `21fd46dc` |
| CI | check-suite `ken-ci` **created and QUEUED** at 20:19:06Z |

**The publisher exited non-zero and that is NOT a candidate failure.** It waited
322s, found **zero check runs** because the suite was still `queued`, printed
*"no checks reported on the branch"* and died. **A queued suite and a suite that
never triggered are indistinguishable to `gh pr checks`**, and the script treats
both as fatal.

⇒ **Resume by re-running the publisher on the same SHA.** `gh pr list --head`
finds #1528, so it will not create a second PR; it proceeds to poll and merge.
Do **not** re-push, do not open a new PR, do not re-vote.

```sh
export GH_TOKEN="$(/workspaces/ken/.devcontainer/mint-gh-token.sh)"
scripts/scripted-pr-automerge.sh \
  --target 21fd46dce478ed3a1a17622c33212e20ca545991 \
  --title "RT-DECL-CLOSURE-PORT + RT-CONTSRC-PRODUCER-LOCAL: continuation source coordinate and typed-unit port; five known rows ignored" \
  --description-file <the file below>
```

Description body is preserved at
`docs/program/diary/pr-1528-description.md`. Check the suite first:

```sh
gh api repos/swe-toolkit/ken/commits/21fd46dc.../check-suites \
  --jq '.check_suites[] | "\(.app.slug) \(.status) \(.conclusion//"-")"'
```

### THE PUBLISH REF IS FROZEN AT `21fd46dc`. A live trap was disarmed here.

**The ring committed `c4112237` (RT-ENTRY-TRAP-254 `D6`) onto
`wp/RT-DECL-CLOSURE-PORT-typed-units` — which is the HEAD OF APPROVED PR
#1528.** Local moved to `c4112237` while origin stayed at `21fd46dc`.

**The publisher force-pushes `refs/heads/$head_branch` by design, and
`resolve_branch` derives `head_sha` from the BRANCH, not from `--target`.** So
the queued re-run would have pushed `c4112237`, moved the PR head off the
approved SHA, and voided `dec_4w8wn4ymn32cm`.

**Disarmed 2026-08-06:** `D6` preserved at **`wp/RT-ENTRY-TRAP-254-d6`**
(`c4112237`, nothing lost); `wp/RT-DECL-CLOSURE-PORT-typed-units` reset to
`21fd46dc`, matching origin. Ring told to work on the `-d6` branch.

⇒ **BEFORE ANY PUBLISHER RE-RUN, assert the branch is still at the approved
SHA:**

```sh
git rev-parse wp/RT-DECL-CLOSURE-PORT-typed-units   # MUST be 21fd46dc...
```

**`--target <sha>` does NOT protect you.** The publisher resolves the SHA to a
branch and then re-reads that branch's tip.

### RT-ENTRY-TRAP-254 — `D3` WITHDRAWN, `D4` measured, `D5` released

- **`D3` (the `abi_word` instrument) is WITHDRAWN, not deferred.** The only
  encoding that retains trapping reddens a **committed**
  `identity_preserved: false` `RootProcessSentinel` invariant and the Family-4
  `-4` closed-default pin. **Steward ruling: do NOT retire either pin** —
  spending a permanent detector for a one-off diagnostic, and the sentinel's
  meaning at the process ABI boundary is the Architect's question, not a
  diagnostic convenience. Not opened, because it is not needed.
- **`D4` was measured BEFORE the instrument was reverted**, which is why the
  hard-stop cost almost nothing. Identity 5 localizes the outer
  `match input { MkProcessInput ... }` of declared unit `process_discriminator`,
  at **generic** `lower_carried_match`, `core.rs:10420`. **The input boundary
  word takes the borrowed-opaque lane and fails the one `ProcessInput`
  constructor case, selecting the closed default.**
- **`D5` RETURNED, no commit, no repair.** The failing comparison is the
  **class** comparison at `core.rs:10420`: expected
  `BoundaryClass::BorrowedOpaque` (8), observed `BoundaryClass::Constructor`
  (4). A compile-time-gated probe scoped to only the single-case
  `ProcessInput::MkProcessInput` match returned exit `Some(4)`; sibling `List`
  and `Prod` untouched. **Class is the borrowed-lane entry condition**, so
  kind/tag/arity are never evaluated —
  `brif(class == BorrowedOpaque, borrowed, represented)` diverts first. The
  generic declared-unit parameter is **carrier-represented, not a borrowed
  handle**.
- **ARCHITECT RULED (`evt_1ayrezmann8zz`): the CONSUMER is correct; this ingress
  must remain `BorrowedOpaque`.** Class 4 is not a lawful relabelling. The
  producer chain admits no lawful step that turns the borrow into a
  `Constructor`: `units.rs:2738-2742` → `units.rs:3749-3765` →
  `transfer_into_carrier` `mod.rs:6128-6203` → `mod.rs:6998-7007` /
  `mod.rs:9688-9692`, disposition `InvocationBorrowed / BorrowedOpaque`.
  **The defect is on the PRODUCER / call-input path** — wrong operand delivered
  to parameter 0 (stronger hypothesis), or an unauthorized reconstruction.
- **TWO CORRECTIONS, one of them the Steward's.**
  **(a) "Carrier-represented" and "borrowed handle" are NOT opposites** — a
  `BorrowedNativeValue` crosses a declared unit *as* a carrier word whose
  representation is `InvocationBorrowed / BorrowedOpaque`; the class-8 node is
  the carrier wrapper around the host pointer. The ring framed them as
  alternatives and **the Steward relayed that framing without catching it.**
  **(b) "Two stacked failures" is REJECTED and the rejection is right** — one
  wrong constructor word explains both observations: it is class 4, and then its
  non-`MkProcessInput` tag correctly defaults in the represented branch.
  **The Steward was about to hold a repair against an unmeasured second defect.
  Do not carry that framing forward.**
- **BANNED (Architect):** do not change or remove the generic borrowed-lane
  test; do not teach the consumer that class 4 means borrowed process input; do
  not patch the observed node's class in place — that makes a constructor
  payload be read as a host pointer and is a confused-deputy hole.
- **`D7` RELEASED (`evt_5veydsvsv7b1n`) — a PROVENANCE/KEYED measurement, not
  another class probe.** Measure the represented word's observed constructor
  **tag and field count** against the planner-issued `MkProcessInput` case
  identity and against every constructor live at the caller-side argument
  occurrence (especially `MkProgramCaps` and `Failure`); record the caller-side
  argument origin and the callee parameter-0 slot ordinal. **The three-way
  partition sizes the repair:** another constructor ⇒ wrong operand, repair the
  call-input producer; `MkProcessInput` but a differing identity ⇒ constructor
  identity authority mismatch; identity equal and field count 3 ⇒ the
  closed-default report is itself incomplete and the next selector must be
  measured. **Repair and recut are the Steward's on that return.**
- **`D7` RETURNED, no change from `c4112237`. Bucket 1: another-constructor /
  wrong-operand.** Planner `MkProcessInput` identity `3470333575222` (low byte
  54), binders 3. **Observed identity differs, low byte 52, field count 1 vs an
  expected 3.** The **arity independently rules out** `MkProcessInput` and the
  authority-mismatch bucket — two axes from different evidence, which is why
  this one is trustworthy. No live named case at this seat has low byte 52:
  `ProcessInput` 54/3, `List::Cons` 36/2, `List::Nil` 35/0, `Prod::MkProd` 38/2.
  **So the observed constructor is currently UNNAMED.**
- **The provenance half is INCOMPLETE** and the implementer stopped rather than
  absorb another build — **correct behaviour, the one-hour target working; do
  not train it out.** Unrecorded: the observed constructor's name, the
  caller-side argument origin, the callee parameter-0 slot ordinal.
- **`D8` RELEASED (`evt_1zp55czkaws7h`):** those three, plus settle the
  one-field adjacent **`Failure`** producer **BY IDENTITY**. It was reported as
  *consistent with* the word and **unmeasured**, which was the right way to
  report it — **"field count 1" is shared by every one-field constructor in
  scope, so consistency ranks it a CANDIDATE, not the answer.** A refuted
  candidate is worth as much as a confirmed one: it would mean the operand comes
  from somewhere nobody has enumerated.
- **No repair authorized.** Knowing *which* wrong operand arrives does not yet
  say *why*, and that gap is where the repair scope lives. Recut is the
  Steward's.
- **`D8` COMPLETE, no branch change. THE WRONG OPERAND IS NAMED:**
  `ProgramCaps::MkProgramCaps`, identity **52**, minted at `StaticOriginId(3)`,
  one field, arriving at `process_discriminator` **parameter-0, slot ordinal
  0** where the borrowed process root belongs. Confirms the Architect's
  stronger hypothesis — wrong operand delivered, not an unauthorized
  reconstruction.
- **The `Failure` candidate is REFUTED BY IDENTITY**, not waved off:
  `StaticOriginId(109)`, identity `3697966841899`, low byte 43 vs observed 52.
  The refutation is what leaves exactly one candidate standing.
- Frame layout: 0 Parameter, 1 Parameter, 2 Result, 3 Control, 4 Trap, 5 Store.
  Call-seat scheduling entries are origins 6 and 109. **The caller
  argument-occurrence ID is still uncaptured.**
- **`D9` RELEASED (`evt_5hnq7741yz410`) and it is THE LAST MEASUREMENT.** Two
  items: (1) **why origin 3 is selected for parameter 0** — name the step that
  builds the argument list, and capture the argument-occurrence ID linking the
  scheduling entries to the delivered operand; (2) **settle the
  identity-magnitude anomaly.** `MkProcessInput` is `3470333575222`, `Failure`
  is `3697966841899`, `MkProgramCaps` is **52** — not the same order of
  magnitude. **Is 52 a lawfully interned identity or an unset/defaulted field?**
  If minting is inconsistent, "the wrong operand arrived" may be the wrong
  diagnosis and "the identity was never set" the right one. **One field is
  consistent with a real `MkProgramCaps`, and that consistency is exactly what
  would hide a defaulted value.**
- **ON `D9`'s RETURN: frame the repair, or hand the mechanism to the Architect.
  NOT another measurement.** We are at `D9` with one commit landed, which is
  the recuts-produce-labels-rather-than-merges shape the playbook warns about.
  Each measurement did genuinely narrow — trap, class, bucket, named operand —
  but the next artifact must be a repair or a ruling.
- **KEEP IN VIEW, do not act on:** `process_discriminator` is called as a
  **functionized declaration closure**, and the branch being merged is
  `RT-DECL-CLOSURE-PORT` — *port declaration closures to the functionized lane*.
  **If `D9` locates the selection defect inside that call path, the defect lives
  in the work being landed** and goes to the Architect, not into a
  Steward-scoped repair. **This does NOT reopen the publish** — the operator
  ruled, the row is skipped, and this node owns it.
- **`D6` landed** (stale carried-scrutinee reachability comment), on
  `wp/RT-ENTRY-TRAP-254-d6`. Targeted: 778 passed / 2 named pre-existing / 1
  ignored.

### Origin's WP ref was DIVERGENT and the force-update was safe

`origin/wp/RT-DECL-CLOSURE-PORT-typed-units` stood at `03f0510c` — **34 commits
that were neither ancestor nor descendant** of the candidate, left by the failed
`fc758323` publish. **`git cherry 21fd46dc 03f0510c` returned `-` for all 34**,
so every one is patch-equivalent-present in the candidate. The publisher's
`--force-with-lease` lost nothing. **Checked before the push, not after.**

### The five rows that ship marked `#[ignore]`

All in `crates/ken-cli/tests/px4b_native_production.rs`, Linux-only. Each
comment carries the exact observed signature, the owning node, and the
branch-introduced provenance (absent at merge base `e6b4a13b` and `main`
`3015aafd`).

| test | owner |
|---|---|
| `fs_write_and_read_resume_through_the_native_capability` | `RT-CARRIER-BYTESPAN-OBSERVE` |
| `fs_scope_denial_reaches_ken_as_the_named_error` | `RT-CARRIER-BYTESPAN-OBSERVE` |
| `canonical_fs_identity_exactly_matches_across_real_producers_and_drift_fails` | `RT-CARRIER-BYTESPAN-OBSERVE` |
| `linked_console_broken_pipe_reaches_ken_instead_of_signal_termination` | `RT-CARRIER-BYTESPAN-OBSERVE` |
| `public_source_observes_raw_argv_environment_cwd_bytes_in_field_order` | `RT-ENTRY-TRAP-254` |

Suite at `21fd46dc`: **14 passed / 0 failed / 5 ignored, no sixth row.**

**A skipped row measures nothing.** Greenness here is achieved by not asking the
question. That is the whole reason both successor frames make un-skipping a
deliverable rather than a courtesy.

### Owed the moment the merge lands, in order

1. **M6** — blob-identity verify every changed path against `origin/main`;
   then `git reset --hard origin/main` on `steward/work`, which is stale the
   instant any publish lands.
2. **M7** — flip `RT-DECL-CLOSURE-PORT` and `RT-CONTSRC-PRODUCER-LOCAL` to
   `merged`, run `scripts/gen-progress.sh`, publish doc-only.
3. **M8** — notify the Adversary. **This merge carries code, so the step is
   required.** Look the id up at post time with `scripts/moot-actor-id.sh
   adversary`.
4. **Librarian** — `crates/ken-runtime/src/cranelift_backend.rs` is a **cited
   source** in `library/SOURCE-ATTESTATIONS`. Routes to the Librarian **after**
   the merge, never into a ring's frame.
5. **M9** — the stay-one-release-ahead check. Already satisfied: both
   successors are `ready` with written frames.

### Both successors are framed. The ring will not idle.

**`RT-CARRIER-BYTESPAN-OBSERVE`** — `ready`, size L, frame at
`docs/program/wp/RT-CARRIER-BYTESPAN-OBSERVE.md`. Base is **`main`**, not the
branch: the publisher squashes, so `21fd46dc` is not an ancestor of `main`
afterwards. Two findings from grounding it, neither known when the rows were
measured:

- **The `BytesPointerLength` seat population is SIX, not the three that fail.**
  `host_effect_seat_contract` binds one `bytes` tuple at six
  `(operation, ordinal)` pairs. `FsWriteFile Argument(2)`, `FsChangeMode
  Argument(0)` and `FsOpen Argument(0)` are unmeasured. Repairing three leaves
  an identical seat refusing identically; flipping six asserts a capability
  nobody measured — and a shared tuple is exactly what makes it a bad
  discriminator. `AC-4` requires a per-seat disposition over all six.
- **The carrier reads a carried byte value one byte at a time and cannot read
  its extent at all.** `BOUNDARY_LOCAL_HELPERS` has `ken_boundary_byte_local` by
  index and a `store_bytes_len` **writer**, and no length reader anywhere.
  `ken_boundary_int_view_local` is the precedent for the missing shape and
  `narrow_carried_int_u64` for its reader. **A per-index reader does not
  establish that a contiguous pointer can be produced** — that is `D1`, and a
  negative answer is a representation boundary that returns to the Architect.

**`RT-ENTRY-TRAP-254`** — `ready`, size **S and DIAGNOSIS ONLY**; the repair is a
separate cut on its return. Two things its frame settled:

- **The exit `1` is not the defect and must not be investigated.** The linked
  shim ends `if (value < 0) return 1;`, so every negative sentinel collapses to
  1 and the exit code cannot distinguish `-1` from `-4`. Only the stderr line
  can. The fact that matters is that the entrypoint returned **`-4`**.
- **`254` IS the correct expectation — that open obligation is DISCHARGED.** The
  test sets `K` to byte `0xfe` under `env_clear()` and asserts `254`, a second
  arm at `253`, and `assert_ne!` between them. Those are legitimate non-negative
  exit codes: the program observes a raw process byte and returns it, and
  `return (int)value` passes non-negative values through. The program is meant
  to compute a byte and traps instead. **Do not repair the row by changing the
  expectation** — the frame bans it as the cheapest available false fix.

### Standing bans that survive this window

- **Do not fold `RT-ENTRY-TRAP-254` into byte-span** because "bytes" appears in
  its test name. That is the vocabulary inference the Architect refuted
  (`evt_7v61ed5pn9q3t`). Shared root cause is **unmeasured**.
- **Do not justify `RT-CARRIER-BYTESPAN-OBSERVE` from the historical
  `c7410b79` `BoundaryCarrier` signature.** Same refutation.
- **Read a Decision from the object, never from a message.** Measured again this
  window: the Architect posted "resolving on cast" and the object still read
  `proposed` for ~30s.

## ⛔ CORRECTIONS — two claims the old file made that were FALSE

⭐ Both were **time-varying state wearing a permanent-looking hat** — the exact
failure the heartbeat prompt bans. Recorded so the *shape* is recognisable, not
just the instances.

### 1. ⛔ "ARMED COUNTERS — the SOLE count of record" was stale AND retired

It read `RT-NATIVE-FNSPLIT: hard-stop 10 · next research pull #11` and `Architect
production blocks: 6 · next check #9`. **Both numbers were behind**, and the chain
they counted **is retired** — the operator stopped the FNSPLIT effort on
2026-07-26 and `SPEC-STORE-SPLIT` replaces it.

⛔ **A counter calling itself "the SOLE count of record" is the worst thing to
leave stale**: it invites a reader to trust it *instead of* measuring. ⇒ **There
are no armed counters now.** When the re-cut program exists, its node owns its
counts.

### 2. ⛔ "TRANSPORT — convo MCP mostly DEAD" is FALSE

The old block claimed only `set_interval`/`subscribe` survived and routed all
reads through scratchpad HTTP scripts. **Measured across this entire session:
`orientation`, `list_decisions`, `post_response`, `list_participants` all work
over MCP.** Tracked as task `#110` because **the heartbeat prompt still repeats
the claim.**

**What IS true — the part worth keeping:**

- ⛔ **NEVER call `mcp__convo__get_transcript`.** Its `limit` does not bound the
  response and it takes the stdio connection down with it. Operator prohibition;
  fleet law in `AGENTS.md`.
- ⚠ **Mentions arrive TRUNCATED** — a doorbell, not a message. Fetch full text via
  the HTTP read path, with **your own** credential.
- ⚠ **`list_decisions` can exceed the result cap** and spill to a file — grep the
  file rather than retrying the call.
- ⛔ `claude mcp list` reporting `convo: ✔ Connected` **is not evidence** — it
  health-checks a fresh process.

## ▶ Preserved refs — ⛔ QUERY LOCALLY. `origin` carries `main` ONLY.

> ### ⛔ THIS SECTION WAS FALSE AS WRITTEN. Both halves.
>
> It said *"Origin holds 26"* and gave `git ls-remote origin
> 'refs/heads/preserved/*'` as the query. **Operator ruling, 2026-07-26:** *"clean
> up all of the non-main branches at origin."* ⇒ **All 63 non-`main` origin
> branches are deleted.** That `ls-remote` now returns **nothing**, and a reader
> running it would conclude the work was lost.

**Measured 2026-07-27 — the query is local, and the population is larger, not
smaller:**

```sh
git for-each-ref 'refs/heads/preserved/*'    # 78 refs
git ls-remote --heads origin                 # refs/heads/main — and nothing else
```

⭐ **A branch on one local ref is the NORMAL state of preserved work, not an
exposure.** ⛔ Do not raise an unpushed ref as a finding.

⛔ **AND THE "EXISTS NOWHERE ELSE" CLAIM WAS WRONG ON EVERY ITEM IT NAMED.** Each
was checked at `origin/main = a1e29284`:

| the old claim | measured |
|---|---|
| `preserved/b2e-rejected-source-oracle` = `159f4109` | ✅ **present locally at that exact SHA** |
| `wp/RT-FNSPLIT-B2E-boundary-value-elimination` = `e1b540e2` | ✅ **present locally at that exact SHA** — ⛔ delete neither |
| `preserved/rt-fnsplit-b2f-hardstop-{9,10,11}-evidence` | ⛔ **no local ref of that name exists** — and it does not need to. Hard-stops #9/#10/#11 are all on `main`, across **12** files (`RT-FNSPLIT-B2{E,F,O,R,V}.md`, `RT-NATIVE-FNSPLIT.md`, `RT-VALUE-TOTALITY.md`, the B2O report + predictions, two WP frames, `diary/2026/Jul/25.md`). `bce75fec` is literally *"make hard-stop #11's evidence durable"*. |
| `preserved/architect-state-*` | ⛔ **wrong prefix** — the refs are `preserved/architect-work-*` (5 locally). A ref name you cannot resolve is not a backup. |

⭐ **The transferable part: a "this exists nowhere else" note is a claim about a
population you did not enumerate, and it decays in both directions at once** — the
copy you were protecting had already landed in the repo, while the ref name you
recorded it under never existed. ⇒ **Re-derive from `for-each-ref` and `git grep`
on `main`; never from a hand-kept list of what is precious.**

## Operator rulings — 2026-07-21 ~12:45Z. ⛔ SETTLED, do not reopen.

⭐ Kept inline deliberately: this is law, and a settled ruling is a **fixed input,
never a question to re-ask.**

- **No "ratification."** The Linux ABI II charter is a **planning document, not a
  commitment.** Nothing outside the project depends on our timelines. ⛔ Do not
  re-raise status-correction as a decision.
- **Where anticipated and done diverge, fill the gap first** — hence
  `docs/program/10-linux-abi-completion.md`.
- **L2-1: no cross-compilation. CROSS-PLATFORM IS INDEFINITELY DEFERRED**
  (restated 2026-07-21 after I re-raised it). Manifest v2 is family-scoped and
  generated, **not** cross-target.
  ⛔ **This ruling ALREADY ANSWERS any non-linux finding** — do not route one back
  as a scoping question. Record such findings as *observations against a deferred
  lane* and stop.
- **L2-0: all desirable, nothing deferred.** All nine `RepresentedUnavailable`
  operations get promoted.
- **Timing, timelines, and budget are the OPERATOR'S domain.** ⛔ Do not reason
  about schedule or cost.
- ★ **My lane is token efficiency in terms of delivered work.** That is the axis
  to optimize and the one to report on.

**Standing test policy (operator, 2026-07-26):** *"Test oracles that assert facts
about source code, catalog, or documentation lines are an invitation for failure
and delay. Tests should focus on behavior."* ⇒ Executable form: **"does an edit
that changes nothing about how any program behaves make this test fail?"**

**Standing gate policy (operator, 2026-07-26):** the library currency ledger is
generated **at version release points**, ⛔ **not enforced per merge.**

**⛔ `origin` CARRIES `main` ONLY (operator, 2026-07-26; restated 2026-07-28).**
A branch living on one local ref is **normal** and is never a finding. ⛔ No
durability sweeps, no pushes of WP or seat branches, no ring reporting an
unpushed ref. The publisher's own candidate-branch push stays — that is how it
opens a PR.

**⛔ THE `integrator` SEAT IS RETIRED (operator, 2026-07-26).** *"remove any
references to the integrator. that seat was retired weeks ago."* ⇒ Every operative
reference is gone as of PR #1052 (`a1e29284`, 50 files) — PR template, CODEOWNERS,
`ci.yml`, four devcontainer files (including a **functional** `ctx-nudge.sh` case
arm), `COORDINATION.md`, `04-git-and-integration.md`, 40 WP frames, the roster
(29→28), git refs, worktrees. ⭐ **The chronicles keep the word deliberately** —
`docs/program/diary/`, `agent/memory/MIGRATION-LOG.md`,
`docs/program/ds-campaign-judgment-log.md` (17 files, 501 occurrences): there it is
a true account of what the process **was**. **Instructions get corrected; records
stay records.** ⚠ One residual is not mine to clear — the convo **participant**
still exists; see the LIVE block's operator-owed list.

**Canonical width: 96 (operator, 2026-07-26).** *"re 88 v 96. 96 is what it should
be. It was an incomplete revision, apparently."* ⇒ `spec/30-surface/31-lexical.md`
and `CANONICAL_WIDTH` are correct; `conformance/` is the stale side.
`SPEC-31-WIDTH-ERRATUM` reconciles it. ⛔ Do not re-argue the value.

## ▶ Where durable law lives — ⛔ do not restate it here

⭐ **The old file's real defect was restating durable rules inside a diary.** A
rule copied into a briefing drifts from its source and then contradicts it. ⇒
**Point, never copy.**

| what | where |
|---|---|
| federation law, §2c handoff gate, §14 merge gate | `agent/COORDINATION.md` |
| my playbook, publish discipline | `agent/playbooks/federation/steward.md` |
| hard-won operational lessons | `agent/memory/` (`fleet` + `enclave` + `roles/steward/`) |
| model tiers | `agent/MODELS.md` |
| reasoning charter | `docs/PRINCIPLES.md` |
| ⛔ no local `--workspace` builds — CI only | `agent/COORDINATION.md §12` |
| build status against the DAG | `docs/program/IMPLEMENTATION-PROGRESS.md` |
| spec status | `spec/SPEC-PROGRESS.md` |

## ⚠ Standing traps — only the POSITIONAL ones

⭐ Each is here because it fires **at a specific command**. That is the whole test
for belonging in this file rather than in `agent/memory/`.

- ⛔ **Verify landed content by BLOB IDENTITY, never ancestry.** The publisher
  squashes, so an approved SHA is correctly *never* an ancestor of `main`.
- ⛔ **Verify every object you NAME exists at the base you NAME** —
  `git cat-file -e <base>:<path>`, and quote the blob (§2c step 5b).
- ⛔ **`git diff --stat` always exits 0.** Use `--quiet` for an emptiness test.
- ⛔ **The publisher's exit code is the LAUNCHER's** — confirm it exited *and* that
  `main` moved.
- ⛔ **Never `git fetch` while the publisher is inside its merge→verify window** —
  `refs/remotes/origin/main` is shared across ~70 worktrees.
- ⛔ **Never `pkill -f`** (matches your own shell) · **never `git stash`**
  (`refs/stash` is shared) · **never `git checkout <ref> -- .`** (reverts
  uncommitted edits worktree-wide).
- ⛔ **A probe truncated before its filter is not a measurement.** Search the full
  stream; truncate the RESULT.
- ⛔ **Never dump `.moot/actors.json`** to learn its shape — use
  `scripts/moot-actor-id.sh <role>`; the schema-discovery step is what leaks a
  key. Look up a participant id **at post time**, never from memory.
- ⛔ **`steward/work` is stale immediately after every publish** — reset onto the
  squashed `main` before writing anything new.
- ⛔ **A `--doc-only` merge can redden `main` and is structurally unable to notice.**
  After one, **enumerate consumers of the touched paths** — attestation ledger,
  measured-token censuses, source-text oracles. This is how `95bc855c` broke three
  things and reported none.
