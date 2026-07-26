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

## ▶▶ LIVE — 2026-07-26 ~19:4xZ · ⛔ `main` IS RED; ONE WP CLEARS IT

**`origin/main` at last check: `e9987c6a`.** ⛔ Verify it; do not trust this line.

### ⛔⛔ THE ONE THING THAT MATTERS

**`main` is RED**, and **not** for the reason an earlier window gave. Measured at
`11b21039` — a tree containing **no** candidate:

```
scripts/ken-cargo test -p ken-cli --test library_documentation_gates
test result: FAILED. 29 passed; 2 failed
```

| failing test | asserts |
|---|---|
| `registered_record_validation_gates_run` (`:636`, panics `:1048`) | 12 cited sources drifted from their attestations |
| `agent_library_manifest_schema_contract_and_measurements_hold` (`:3356`) | `author-package.md` `measured_tokens` — 480 declared, 459 actual |

**Cause: my own `--doc-only` merge `95bc855c` (PR #1031).** It edited 11 cited
catalog sources + `docs/program/07-catalog-style-guide.md` without regenerating
`library/SOURCE-ATTESTATIONS`, and `library/agents/tasks/author-package.md`
without recomputing its token count. ⛔ **`--doc-only` skips CI, so nothing
reported either.**

⇒ **Every non-doc-only merge is blocked.** Fix =
[`LIB-GATE-DECOUPLE`](../issues/LIB-GATE-DECOUPLE.md), **operator-ruled: "remove
the CI coupling."** ⛔ **Do NOT regenerate the ledger** — that option was
considered and rejected.

### ▶ Lane state

| ring | state |
|---|---|
| **Verify** | ▶ **LIVE** on `LIB-GATE-DECOUPLE`, kicked at `e9987c6a` (`evt_1bhzvkrc1gmm0`), leader confirmed `Working` |
| **Runtime** | ⛔ **STOPPED by operator order.** No node. `B2F` held. ⛔ Do NOT re-anchor `B2E`/`B2F` — retire them and write fresh |
| **Language** | idle, correct. Candidate approved and queued; told it is innocent of the red (`evt_6fp26ys3h0r5f`) |
| **Spec enclave** | idle, correct. Next work = the `SPEC-STORE-SPLIT` frame (unwritten) |
| **Doc · Foundation · Kernel · Ergo** | idle |

### ▶ Queued behind the red

- **`KW-ORACLE-REMOVE`** exact `68c3d870`, base `720b0e17`. `dec_200k7z471z9x6`
  **resolved** by the Architect. PR #1035 failed CI **on the pre-existing red, not
  on the candidate.** ⇒ Republish **WITHOUT `--doc-only`** once `main` is green.
- **Publisher message fix** — `preserved/steward-publisher-msg-fix = 921e042b`
  (also `steward/work` HEAD). ⛔ Touches `scripts/`, so it **must clear CI** and
  ⛔ **must not ride a `--doc-only` merge.**

### ▶ What I owe next, in order

1. Verify's candidate → Decision → publish → blob-verify → retros.
2. Republish `68c3d870` **without** `--doc-only`.
3. Publish the publisher message fix (needs CI).
4. Author the `SPEC-STORE-SPLIT` frame shovel-ready; **§2c gate — compact all
   three enclave seats unconditionally** — then kick the enclave.
5. Fold task `#94`'s hard-stop-`#11` measurements into a durable node, then delete
   the task.

### ⛔ Still operator-HELD

**`DOC-ATTEST-LIVING`** — ⛔ **do not release, do not re-ask.** Node:
`docs/program/issues/DOC-ATTEST-LIVING.md`.
⚠ `DOC-GATE-NEEDLE` was also listed as operator-HELD in the old file; **it has
since merged (#1019) and closed.** That marker was stale.

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

## ▶ Preserved refs — ⛔ QUERY, do not maintain a list here

The old file hand-maintained 6. **Origin holds 26.** ⇒ A hand-list of refs is a
floor that reads as a population:

```sh
git ls-remote origin 'refs/heads/preserved/*'
```

⭐ Notable, because their content exists **nowhere else**:
`preserved/b2e-rejected-source-oracle` = `159f4109` and
`wp/RT-FNSPLIT-B2E-boundary-value-elimination` = `e1b540e2` (Runtime's five
stopped days — ⛔ **delete neither**) ·
`preserved/rt-fnsplit-b2f-hardstop-{9,10,11}-evidence` ·
`preserved/architect-state-*` (a state branch that never merges).

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
