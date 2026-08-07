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

## LIVE — 2026-08-07 ~02:1xZ · ONE blocker left: nine `scenario.rs` rows, with the operator

**The GitHub outage is over. CI works. `main` is still `3015aafd`.**

### The candidate is exonerated and `ken-cli` is GREEN

**Candidate: `wp/RT-SRCBODY-BIND-ORDER` at `8696e8c5`. HELD — not routed for
review.** `wp/RT-DECL-CLOSURE-PORT-typed-units` frozen at `21fd46dc`.

**`D12` complete `--no-fail-fast` enumeration** (a closed enumeration, because
fail-fast is per **binary**, not per test): **40 candidate failures, every one
also fails at base `21fd46dc`. ZERO REGRESSIONS.** The candidate additionally
**FIXES SIX** base failures.

**31 authorized rows annotated at `8696e8c5`** — annotation-only, 290+/21-,
nine files, each row carrying its **exact signature**, **owning node**, and
**"fails at base `21fd46dc`"**. Result:
`ken-cargo test -p ken-cli --no-fail-fast` = **120 passed / 0 failed / 34
ignored**. **`px7o` is UN-ignored and passing 3/0.**

### THE ONLY THING LEFT

**Nine `ken-verify` LIB UNIT TESTS in `scenario.rs`** — unit tests over
production source, not integration parity rows. **With the operator.** Their
skip authorization plausibly aimed at the integration/CI-gate surface, and this
is a different kind of debt. **DO NOT ANNOTATE `scenario.rs` until they rule.**

⇒ On their answer: add the nine to `8696e8c5` **or** close the set at 31.
**One review cycle either way** — QA then Architect on the exact final SHA. A
new SHA voids `dec_wyn3kvzhs9at`; read Decisions from the **object**.

### THE NUMBER IS 31 — my off-by-one, twice

I said 39, then 30. **`D12`'s 40 are CANDIDATE failures and `px7o` is NOT among
them — it PASSES.** Removing its wrong annotation removed no row from the
failing set. **40 = 30 `ken-cli` + 10 `ken-verify`**; only **9** of `ken-verify`
are the held `scenario.rs` tests, and `px8f_write_partition` is an authorized
integration row already annotated. **40 − 9 = 31**, plus the four `px4b` = **35
`#[ignore]` total.** The ring caught this and escalated the arithmetic while
following the unambiguous substantive instruction — the right precedence.

### `RT-ENTRY-TRAP-PX7O` IS CLOSED — false premise, mine

CI reports the bare name `nested_err_payload_reaches_both_real_executors`,
defined in **two** binaries. I attributed it to `px7o`; the red one is **`px7n`**
(owned by [[RT-FRAME-MARKER-ONCE]]). **`D10` measured `px7o` at the BASE, where
it does fail, and I carried that forward as if it described the tip.** I also
told the operator the repair "cleared `px4b` but not `px7o`, so it may be
incomplete" — **it cleared both.** Do not re-file the node; do not re-skip
`px7o`. **A bare test name shared by two binaries names neither.**

### Owners, all filed

`BytesPointerLength` → [[RT-CARRIER-BYTESPAN-OBSERVE]]; **`ResourceScalar` →
[[RT-CARRIED-RESOURCE-SCALAR]]** (same refusal shape, **different need** — never
call these byte-span); frame marker → [[RT-FRAME-MARKER-ONCE]]; closure lane →
[[RT-CLOSURE-BOUNDARY-LANE]]; `ComputationalMatch` →
[[RT-COMPMATCH-TREE-SCRUTINEE]]; `ProcessExitStatus` →
[[RT-PROCESS-EXIT-STATUS]]. **An unmatched row gets its OWN node, never a
nearest fit.**

### Still unexplained — do not paper over it

**CI showed 14 where local shows 40.** Fail-fast plus sharding explains the
direction; it is **not proven**. The ring correctly refused to size an
annotation on that discrepancy.

### Why the set kept growing, recorded because I missed it once

**`cargo test` and CI are both fail-fast PER BINARY.** One failure and the
binary's remaining tests never run and never report, so **annotating a row
un-hides the next row in the same binary.** The implementer caught this during
`D10` and reported it; **I then scoped `D11` off CI's truncated list and called
it complete.** `D12`'s whole-package `--no-fail-fast` run is the closed
enumeration that ended the peeling.

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

### RT-ENTRY-TRAP-254 is CLOSED (superseded). The repair is RT-SRCBODY-BIND-ORDER.

**Architect mechanism ruling `evt_7yfs6qxp9hm5b`.** The `D0`-`D9` chain found a
**general multi-parameter source-body binding permutation**; the skipped
`ProcessInput` row is **one discriminator** for it.

**THE DEFECT.** `lowering/units.rs:3701-3790` does **one slot-order walk doing
two jobs**: it records `defining_abi_operands` in ABI descriptor order
(**correct**) and pushes the same operands into `env` in that order
(**wrong**). A declaration body reads **de Bruijn-NEAREST-FIRST**, and
`core.rs:14705-14714` **already states** reverse-then-append. So
`main(input, caps)` gives `env = [input, caps]` while the body names `input` as
`Var(1)` ⇒ `Var(1)` reads `ProgramCaps`. **A bug fix restoring a stated
contract, not a mechanism change.**

**REPAIR:** keep the ABI run and `defining_abi_operands` unchanged; build the
semantic env as `reverse(Parameter run) ++ Capture run in D3 order`.

**`D9`'s ATTRIBUTION WAS REFUTED and the refutation is load-bearing.** It blamed
the common transfer coordinate. `call_declared_unit_target` **already pairs
positionally**, `carry_call_input` cannot select a sibling or change position,
and **a carried word bypasses `transfer_into_carrier` entirely** — so a caller
occurrence there cannot change which word occupies slot 0. ⇒ **Per-argument
transfer coordinates are BANNED: a design change that would leave the defect
intact.** Also banned: reversing the process root's ABI roles, rewriting
continuation specializations, touching `carry_source_call_inputs`,
`carry_call_input`, `call_declared_unit_target`, or `mod.rs:5958-5978`.

**BLAST RADIUS — AGGREGATE-NESS IS NOT CAUSAL.** Not one row, and **not "every
aggregate through `call_declared_unit_target`" — that framing was the Steward's
and is wrong.** The class is **every activated non-root functionized source-body
unit with at least two parameters whose body distinguishes parameter
positions**; it surfaces for ints, bools, capabilities, borrowed handles or
constructors. The 97-`Constructor` census does **not** bound it. **Unary units,
unused parameters and equal values MASK it.** Operator told; **it does not alter
the publish ruling by itself** but is materially larger in logical scope.

**`D2` is not optional:** generated contexts claim **byte-for-byte equivalence**
with the raw unit while installing parameter-then-capture order
(`units.rs:2523-2547`). Fixing the unit alone makes that committed claim
**false** — worse than the original defect, because the claim is what a reader
relies on.

**Four controls, and control 1 is the important one:** a two-parameter
declaration with distinct **NONAGGREGATE** values reading both positions
(proves the fix is not aggregate-shaped); the `ProcessInput`/`ProgramCaps`
discriminator; a root-adapter control proving its ABI-role order was **not**
reversed; raw-worker vs generated-context equivalence on a body that
**distinguishes** its parameters (a unary body proves nothing — unary units are
invariant under reversal). **Expect CI reds and attribute each individually;
never re-baseline.**

**`D6` landed** (stale carried-scrutinee reachability comment) at `c4112237` on
`wp/RT-ENTRY-TRAP-254-d6` — follow-up PR when Actions returns.

### RT-SRCBODY-BIND-ORDER in flight — candidate `5d388e37`, QA HELD

- **`5d388e37`** meets `D1`, `D2`, `D4`, all four controls red-before-green.
- **`D3` control 4 and `AC-3` were AMENDED by the Steward** (`evt_gpekyt7jzb67`).
  The required population does not exist — no body is present at both hosts,
  retargeted raw workers are template-only, **every generated-context worker is
  unary**. The ring reported the weakness instead of widening the fixture.
  **`AC-3` as originally written was UNDISCHARGEABLE and that was a frame defect
  of mine.**
- **The ruling, one fact:** `reverse([p]) ++ captures` is identical to the
  parameter-then-capture order already installed, so **`D2` is INERT at unary
  arity.** ⇒ the obligation becomes an **ACTIVATION GATE** on the first
  multi-parameter generated-context worker, shipped as a **TRANSITION
  SENTINEL** that asserts the measured population and reddens by itself.
  **Non-vacuity required:** observed RED against a hand-added two-parameter
  worker, then restored. **`AC-3` must NOT be recorded as "equivalence
  verified"** — equivalence is unfalsifiable at unary arity.
- **NOT authorized:** changing the checked IH call-site arity to manufacture the
  fixture. Population expansion into a checked mechanism, and the constraint
  demanding it was **this frame's own prose**.
- **ARCHITECT is reviewing `D3c` unrequested** (`evt_28gv50xst6sqf`): tracing
  `RootIsImmediate` from its stored coordinate into the **post-`D1` semantic
  environment**. **This is the right question and I did not think to ask it** —
  `D1` reorders the semantic env, and `D3c`'s per-consumer availability claims
  are keyed to that env, so a claim could now name the wrong value. **QA is
  correctly held pending it. Do not release QA or alter `D3c` until it lands.**

### Compaction verification: a LOW ctx is proof, a HIGH ctx is NOT disproof

**Measured 2026-08-06 on runtime-implementer.** `handoff-gate-compact.sh`
returned, the pane still read `ctx 27%`, and a **full-stream** grep for
`Compacting|Context compacted` returned **0**. I resent `/compact` to that one
pane and ctx went **27% -> 7%** — while the marker grep *still* returned **0**.

⇒ **The marker text is transient and its absence proves nothing** (the progress
bar clears). **The ctx number showing a DROP is conclusive; the ctx number
showing HIGH is inconclusive**, because it can be a stale render.

**So verify in this order:** ctx dropped ⇒ done. Otherwise resend to that one
pane and re-check ctx. Do not conclude "did not compact" from an absent marker,
and do not conclude "did compact" from the script returning.

### GATE MISS, MINE: I released a new node without the before-work compaction

**`COORDINATION §15` / `steward/compaction.md`: always compact before new work,
no exceptions, no threshold, ctx unread.** I released `RT-SRCBODY-BIND-ORDER`
after roughly fifteen consecutive diagnostic releases to the same ring **and
never ran the handoff gate once.** The **implementer asked for the seam** —
the backstop caught what the gate should have.

⇒ **The gate is the fix, the ctx scan is only the backstop.** When the scan (or
a seat's own request) is what catches it, **the gate already failed upstream.**
Run `scripts/handoff-gate-compact.sh <every member>` at each new-node release.

**Mechanism note for next time:** the script does `git reset --hard
origin/main` on each worktree. It was safe here **only because** all three
runtime worktrees sat on their own `<role>/work` branches at `3015aafd ==
origin/main`, so no protected ref was checked out. **Check that first** — the
frozen publish ref `wp/RT-DECL-CLOSURE-PORT-typed-units` (`21fd46dc`) and
`wp/RT-ENTRY-TRAP-254-d6` (`c4112237`) must never be the checked-out branch when
it runs. **And note the base mismatch:** this node's base is `21fd46dc`, not
`origin/main`, so the reset puts worktrees on the wrong base and the ring must
re-checkout.

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
