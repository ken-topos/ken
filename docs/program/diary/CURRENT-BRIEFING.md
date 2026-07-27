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

## ▶▶ LIVE — 2026-07-27 ~03:4xZ · `main` GREEN; enclave owes retros, then `STR-BIJ`

**`origin/main` at last check: `13004a63`.** ⛔ Verify it; do not trust this line.

### ▶ Where the thread actually is

`SPEC-31-WIDTH-ERRATUM` is **merged** — PR **#1054** (`main` was `c5281fc3`),
node flipped `merged` in PR **#1055** (`main` now `13004a63`). Both verified by
**blob identity with a pre-merge control that differs**, never by ancestry.
#1054 was published **without** `--doc-only` (it touches `conformance/`), so the
full CI gate ran and passed — a second non-doc-only merge clearing CI.

⛔ **The node is NOT closed: spec-author and conformance-validator retros are
owed** (called `evt_q75v9dhj6st7`, re-led `evt_52pkravrkn887`, then delivered by
direct pane write). ⛔ **Do not compact the enclave until they post** — a
compaction eats an unposted retro. `STR-BIJ` is next and is released only after
retros, behind the full Handoff Gate.

⚠ **Measured 2026-07-27: two convo mentions and a bare `Enter` did NOT wake
`spec-leader`; only writing into its pane did.** The composer held its **idle
placeholder**, so nothing was stranded — the event never arrived at all. ⭐ The
bare-`Enter` recovery presumes something is *sitting on* the composer; against an
empty one it is a **no-op that looks exactly like a successful nudge**. Check for
a `Working`/spinner transition after any nudge, not just that you sent it.

### ✅ THE RED IS GONE — and here is the evidence, because the claim is load-bearing

The block that stood here said **`main` IS RED** and *"every non-doc-only merge is
blocked."* ⛔ **Both were true when written and are false now.** `LIB-GATE-DECOUPLE`
landed and removed the CI coupling, per the operator's ruling.

⭐ **The proof is not a green badge — it is a merge that could not have happened.**
PR **#1052** (integrator retirement) was published **without `--doc-only`**
precisely because it touches `ci.yml` and a functional case arm. The publisher
waited its full pre-poll window, read the checks, and reported
*"PR #1052 checks passed and merge command succeeded."* ⇒ A **non**-doc-only merge
cleared CI end to end. That is the discriminating observation; a doc-only merge
would have proved nothing, since `--doc-only` skips CI.

### ▶ Lane state

| ring | state |
|---|---|
| **Spec enclave** | ✅ `SPEC-31-WIDTH-ERRATUM` **merged** (#1054/#1055). ⛔ **retros owed** — not closed, do not compact. Next: `STR-BIJ` (hold discharged, re-derivation done) |
| **Runtime** | ⛔ **STOPPED by operator order.** No node. ⛔ Do NOT re-anchor `B2E`/`B2F` — retire them and write fresh |
| **Verify · Language · Doc · Foundation · Kernel · Ergo** | idle, correct — nothing released |

⛔ **The tracker shows three items `active`; only one ring is.**
`RT-NATIVE-FNSPLIT` and `RT-VALUE-TOTALITY` read `active` and are **stopped**;
`SPEC-MISSION-GROUNDING` reads `active` and is an **umbrella with no ring on it**.
See the operator escalation below — `gen-progress.sh`'s `VALID_STATUSES` has no
value meaning *"halted under an operator stop-order"*, so the stop exists only as
prose someone has to repeat.

### ✅ Landed this window — five merges, each blob-verified with a control

| PR | `main` | what |
|---|---|---|
| #1048 | `c631841d` | `SPEC-STORE-SPLIT` — Map/Set internal bytes are not observable |
| #1049 | `5b848ad5` | tracker flip |
| #1050 | `6d3f9fb5` | retro carry: a property-removal census must close over **entailment** |
| #1051 | `fd8de255` | Librarian as-built — chapter 06 store mechanism + capacity prose |
| #1052 | `a1e29284` | **integrator seat retired** — 50 files, every operative reference |

⭐ **`SPEC-STORE-SPLIT`'s carry is the strongest thing this window produced, and all
four enclave seats derived it independently:** a property-removal census **by
subject name is not closed under entailment.** The first fold repaired all six
carriers that *say* Map/Set; the retired promise survived in generic clauses
quantified over *closure-free* / *admitted* / *any live* value — whose domains
contain Map/Set. Nine carriers in total, found across three passes. ⭐⭐ **The tell:
repeated population growth across passes means the ENUMERATION METHOD is wrong,
not that the list is longer.** Landed at
`agent/memory/fleet/an-enumeration-needs-a-proven-closure-not-a-better-grep.md`.

### ⛔ OWED TO THE OPERATOR — three items, none self-resolvable

1. **The tracker cannot express an operator stop-order.** `RT-NATIVE-FNSPLIT`,
   `RT-FNSPLIT-B2F`, `RT-VALUE-TOTALITY`, `PX8-F-CAP-41` display **unblocked**
   because their dependency merged, while runtime is under a full stop. Needs
   either a new status value or a `halted:` field. **Raised, unanswered.**
2. ⛔ **`SPEC-MISSION-GROUNDING` `AC-M3` names a pass I am forbidden to request.**
   The AC says *"the adversary refutation pass is still owed"*, and the node's §2
   routes it to the adversary. **`COORDINATION §10⁻a` forbids the Steward from
   asking the adversary to hunt anything** — *"a request for an attack is a
   conversation; the Steward does not make one"* — and scopes the adversary to
   `crates/` and catalog/`library/`, which a spec-vs-mission audit is not. ⇒ Two
   operator-authored artifacts conflict. **I cannot discharge `AC-M3` without the
   operator either dispatching the adversary directly (as happened for the first
   pass) or re-routing the AC.**
3. **The `integrator` participant still exists in the convo space** with a stale
   status citing PR #365 (merged 2026-07-08). The roster, git, worktrees, tmux, and
   every tracked non-chronicle file are clean — but `orientation()` and
   `list_participants` still show the seat to **every** agent, and no MCP tool
   removes a participant. **Operator/convo-admin action.**

### ⛔ Still operator-HELD

**`DOC-ATTEST-LIVING`** — ⛔ **do not release, do not re-ask.** Node:
`docs/program/issues/DOC-ATTEST-LIVING.md`.

### ⚠ UNDURABLE CARRY — this text is the only copy, do not delete it unread

Task `#113` tracks the promotion ladder; this entry is still channel-only, which
is the exact undurability task `#107` exists to fix.

**A CORRECTION RESETS THE FIXED-INPUT AUDIT** (`evt_5h11p8gjjswmp`, Language ring):

> *"The first measurement validated cardinality; it did not validate pass state, and
> no amount of confidence in the corrected prose could bridge that gap. … Re-read
> every executable operand at the corrected exact object, **including facts that
> were introduced by the correction itself**. A correction explains why the prior
> claim was wrong; it does not make its replacement observed."*

⭐ That is why the second `AC-4` error lived **inside** the fix for the first. ⭐⭐ It
is the **same property** as *a clearance names the axes it covers*, seen from the
other end — an arity mismatch that prose collapses. ⇒ ⛔ Promote them as **one**
entry, not two.

⭐ Also load-bearing: *"the 'measure before any edit' boundary kept the red
attributable to `main`."* ⇒ **A deletion that lands without a pre-measurement
destroys the evidence that the base was already broken.**
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
> 'refs/heads/preserved/*'` as the query. **Operator ruling, 2026-07-26:** *"you do
> not need off-box pushes … Also clean up all of the non-main branches at origin."*
> ⇒ **All 63 non-`main` origin branches are deleted.** That `ls-remote` now returns
> **nothing**, and a reader running it would conclude the work was lost.

**Measured 2026-07-27 — the query is local, and the population is larger, not
smaller:**

```sh
git for-each-ref 'refs/heads/preserved/*'    # 78 refs
git ls-remote --heads origin                 # refs/heads/main — and nothing else
```

⭐ **A branch on one local ref is the NORMAL state of preserved work, not an
exposure.** ⛔ Do not raise an unpushed ref as a finding, and do not mint a token
to push one.

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

**⛔ NO OFF-BOX PUSHES (operator, 2026-07-26).** *"You do not need off-box pushes.
In decades of software development, I have never lost a commit to a drive failure.
Also clean up all of the non-main branches at origin."* And on the protocol:
*"keep it simple. the off box push protocol needlessly complicates and wastes time
and tokens."* ⇒ **`origin` carries `main` only.** A branch on one local ref is
**normal**. ⛔ No durability sweeps, no token-minting pushes, no ring reporting an
unpushed ref. The publisher's own candidate-branch push stays — that is how it
opens a PR. `steward.md` §2c step 8b (~80 lines of `ls-remote` sweep + push
recipe) is **deleted**, not amended.

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
