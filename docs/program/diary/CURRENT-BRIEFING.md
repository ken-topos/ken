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

## ▶▶ LIVE — 2026-07-27 ~18:0xZ · 4 merges; 3 FRAMES WRITTEN, NOT KICKED

**`origin/main` at last check: `b55d292c`, with PR #1117 (`V4-RESIDUAL`) in its
publisher window.** ⛔ Verify it; do not trust this line.

### ⭐⭐ THE ONE THING TO DO NEXT

Three WP frames are **written, committed to `steward/work`, and NOT released.**
Publishing them and kicking their rings is the whole of the next action — it is
`§00` delivery and nothing outranks it.

| ring | frame | state |
|---|---|---|
| **Ergo** | `SURF-IDENT-TR39-R1` | written, unkicked |
| **Language** | `SURF-SPACE-CELLS-P1` | written, unkicked |
| **Runtime** | `RT-FNSPLIT-C1` (frame pre-existed) | dependency cleared, unkicked |

Sequence: V4 lands → verify by blob → publish my doc bundle (`steward/work`
commits `781ffee1`, `a2bbde28`, `6aa5c230`, `a5dbdc6b`; ⛔ cut a fresh
`wp/steward-<slug>` from `origin/main` and cherry-pick — never publish from
`steward/work`) → **then the Handoff Gate for each ring: compact EVERY member,
verify each drop, THEN post the kickoff.**

⚠ **Ergo and Language both touch `crates/ken-elaborator/src/lexer.rs`** (Ergo:
the identifier rule; Language: `becomes`/`mut` tokens). Different functions, but
I told Language in `SURF-SPACE-CELLS-P1 §5` that I would **sequence rather than
let them collide.** Honor that.

### ✅ Landed this window — four merges, each blob-verified

| PR | `main` | what |
|---|---|---|
| #1114 | `78f1f74b` | `MAP-TRANSPORT-CODEC-R1` determination — no codec is required today |
| #1115 | `aea07d62` | `EFF-SPACE-ENSURES-PRESTATE` — `old` fails closed (Shape B) |
| #1116 | `b55d292c` | `RT-VALUE-TOTALITY-P3` — `Value` `Debug` is depth-total |
| #1117 | in flight | `V4-RESIDUAL` — binder-child paths rejected |

### ▶ Lane state

| ring | state |
|---|---|
| **Kernel** | ✅ **building** — `KERNEL-NESTED-IND` D3b+D4. D3a already merged |
| **Verify** | `V4-RESIDUAL` in the publisher; ring free after |
| **Runtime** | free; `RT-FNSPLIT-C1` releasable, awaiting my kickoff |
| **Language · Ergo** | free; frames written, awaiting my kickoff |
| **Foundation** | ⛔ **whole ring idle and legitimately blocked** — see task `#144` |
| **Doc** | idle, correct — `DOC-ATTEST-LIVING` is operator-held |
| **Spec enclave** | `STR-NFC-CONSTRUCTION` awaiting Architect resolution |

### ⛔ OWED TO THE OPERATOR — four items, none self-resolvable

1. **`MAP-TRANSPORT-CODEC` candidate 3** — a wire format for a **non-Ken peer**.
   Candidates 1 and 2 were measured and answered *no*; this one is a **roadmap
   call and is not answerable from the repository.** The node is closed
   `not-needed`; if this comes back *yes* it reopens with a fresh frame.
2. ⛔ **`SPEC-MISSION-GROUNDING` `AC-M3` names a pass I am forbidden to request.**
   The AC says the adversary refutation pass is owed; `COORDINATION §10⁻a`
   forbids the Steward from asking the adversary to hunt anything. **Two
   operator-authored artifacts conflict.** Needs the operator to dispatch the
   adversary directly or re-route the AC. **Raised, unanswered.**
3. **T3 / `Property`** — there is **no `ken test` subcommand** (`ken-cli`
   dispatches `repl|run|check|native-build|fmt|version|help`) and **no spec
   chapter for the CLI at all** (task `#143`). `Tooling/Testing/Property.ken.md`
   exists but is deterministic finite checks — no randomness, shrinking, or
   seeds, deliberately. ⇒ T3 is blocked on a **design input**, not on code.
4. **Linux ABI** — `ABI-S3`'s three ops landed `RepresentedUnavailable` by
   design and **no Track-A node promotes them**, so `§6`'s exit condition is
   unreachable through `ABI-A1/A2/A3`.

### ⛔ Still operator-HELD

**`DOC-ATTEST-LIVING`** — ⛔ **do not release, do not re-ask.** Node:
`docs/program/issues/DOC-ATTEST-LIVING.md`.

### ⛔⛔ THE `integrator` GHOST — do NOT chase it again

**The seat was RETIRED by PR #1052** (50 files). It has **no tmux session, no
entry in `.moot/actors.json`, and no playbook** — `agent/playbooks/federation/`
holds only adversary, architect, librarian, research, steward.

⚠ **But `orientation()` and `list_participants` still show it**, carrying a
**stale stored status** — *"PR #365 green on head `befc2dc4`, awaiting Steward
routing."* ⭐ **That reads exactly like a live seat blocked on you, and it is
not.** I treated it as a real open query, investigated it, and posted a routing
reply to a seat that cannot read. No one was waiting; nothing was owed.

⇒ ⛔ **A retired seat's last status is indistinguishable from a live seat's
current one.** Before acting on any participant status, check for a tmux session
**and** an `actors.json` entry. Convo has no tool to remove a participant, so
this ghost persists — it is operator/convo-admin item, already raised.

For the record on its content: `befc2dc4` is **on no ref at all**, and
`scripts/scripted-pr-automerge.sh` **is** on `main` (blob `76afaf31`) — the
capability landed and I run it every publish. ⛔ Do not re-propose that commit;
its `COORDINATION.md` / `04-git-and-integration.md` / `steward.md` versions are
from 2026-07-08 and re-landing them would revert weeks of work.

### ⭐ Traps measured this window — positional, so they will recur

1. ⛔ **The decisions read path field is `decision_id`, NOT `id`.** `d.get('id')`
   returns `None` for **every** record, so a lookup reports **NOT FOUND for a
   decision that exists**. I was one step from blocking a merge on this. ⇒ Always
   run a positive control against a decision you know exists.
2. ⛔ **Any non-doc-only publish MUST be `run_in_background: true`.** Full-CI
   polling exceeds the Bash tool's 600000ms cap and the tool kills the publisher
   (exit 143). Doc-only finishes in ~2 min and is safe in the foreground.
3. ⭐⭐ **"Awaiting merge" may already BE merged** — twice this window (Ergo
   `a85c0dc5`, Kernel `5396f9a7`). The publisher **squashes**, so every ancestry
   check says unmerged. Only a blob diff of the candidate's **own** paths against
   `origin/main` discriminates. ⚠ And a path-drift check *actively misleads*: both
   Kernel paths read as "changed on main since the approval base," which normally
   means a stale base — here the thing that changed them was the candidate's own
   already-landed work.
4. ⛔ **Require the exact BRANCH as well as the exact SHA.** Kernel had
   `wp/KERNEL-NESTED-IND` (D1a, `e685570c`) and `wp/KERNEL-NESTED-IND-D3` (the
   approved D3a) live at once, and the approved SHA was **not** an ancestor of the
   branch matching the node name.
5. ⚠ **A node annotation written at merge time can gate another team's frontier.**
   I wrote *"`RT-VALUE-TOTALITY` stays `active` for its remaining scope"* — it had
   none, and that stale `active` was the last unmet `depends_on` of
   `RT-FNSPLIT-C1`. It would have idled Runtime behind a complete node. It
   surfaced only because `runtime-leader` refused to infer a branch and asked.

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
