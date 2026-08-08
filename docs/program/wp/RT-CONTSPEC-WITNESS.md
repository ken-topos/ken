# WP frame — `RT-CONTSPEC-WITNESS` (ContinuationSpecialization seam 4 of 4)

Node: `docs/program/issues/RT-CONTSPEC-WITNESS.md`. Campaign:
`docs/program/16-recursive-descent-retirement.md`. Owner: runtime ring.
Authority: Architect ownership/sizing ruling `evt_1yymw1gdszpbs`, outcome (c),
seam 4 — the terminal seam.

Seams 1-3 built and proved the mechanism. This seam **measures the result on a
lawful assembly** and closes the campaign out. **It closes only itself** -- see
`D6`; the inherited claim that it closes other nodes was withdrawn 2026-08-08.

> ## RECUT 2026-08-03 — the census is a HISTORICAL RECORD here, not a worklist
>
> **Steward.** The prior cut had `AC-1` require every one of the corrected
> census's 138 rows to reach a terminal disposition, and its population table
> asserted that 6 formerly shadowed rows have "their causal roots cleared in
> seams 2 and 3." **That premise is false and it is settled, not pending.**
> Seam 2 was recut off the census entirely (`evt_2zhx69f2fw07w`, Architect
> confirmation `evt_66t42tapvdbsj`) and merged that way at `0a6e34cc`; seam 3
> was recut off it for the same reason, and its landed frame says in its own
> operative text that *"the census is not an input to any deliverable or AC
> below."* **Neither seam clears any census row.**
>
> As written, every row's disposition depended on clearing that will never
> happen — an obligation with no possible discharge.
>
> ⇒ **The reconciliation stays, and its meaning changes.** The 138 rows are
> accounted for as a **record of what once failed in the held `1aef3192`
> lineage**, not as measurements of work seams 1-3 performed. `AC-1`, `AC-2`,
> `D2` and the population table below are all restated on that footing. **No row's
> disposition may be stated as, or made to depend on, a seam having cleared its
> causal root.**
>
> **Why this was recut before seam 3 merged**, where the earlier note said to
> wait for it: the question the wait was for is already answered. Seam 2 is
> landed and cleared zero rows, and seam 3's census disposition is fixed in its
> own merged frame rather than pending its execution. Holding the node at `draft`
> past that point would only guarantee an unframed successor at seam 3's merge.
> ⇒ **The node is now `ready`.** Release still gates on `RT-CONTSPEC-LEDGER`
> merging, which is a `depends_on` edge, not a framing debt.

> ## The `46d29783` lineage remains an ORACLE
>
> `46d29783`, `1aef3192`, `9d58df12`, and
> `refs/preserved/rt-contspec-lower-held-core-rs = 88972207` are preserved and
> **may not be merged, rebased onto, or cherry-picked wholesale.** This seam
> branches from `main` after seam 3 lands and carries only its own delta.

> ## THE RULE THAT KEEPS THIS SEAM FROM ABSORBING THE CAMPAIGN
>
> **Any fresh planner or ABI failure on the lawful assembly routes back as a new
> exact interface hard stop. It is not repaired inside closeout.**
>
> That is the Architect's wording, and it is the boundary that stops seam 4
> becoming a second cumulative branch — the exact failure that killed
> `RT-CONTSPEC-LOWER`. A closeout seam is the most tempting place in a campaign
> to "just fix the last thing." **It is also the place where doing so destroys
> the evidence that seams 1-3 produced**, because a repair here is unattributable
> to any seam's population.

## The three populations

| population | rows | what makes it measurable now |
|---|---:|---|
| the native population | run on a lawful assembly | seams 1-3 established the assembly; the held lineage never had one |
| formerly shadowed rows | 6 | **nothing cleared their causal roots** — see below; they are read against the lawful base, not against a clearing that happened |
| host `ENOSPC` rows | 2 | disk capacity, not semantics — see below |

The six shadowed rows are the subtle population, and the subtlety is not the one
the earlier cut named. **A shadowed row is not a failing row and it is not a
passing row — it is an unmeasured one.** Its assertion never ran, because an
earlier refusal stopped the test **in the held `1aef3192` tree**.

⛔ **Their causal roots were never cleared and no seam will clear them.** The
refusals that shadowed them belong to a mechanism that was never merged and may
never be. So the question "what does this row say now that its root is cleared"
has no referent, and you must not ask it.

⇒ **The question that does have a referent: does the test the row names exist on
the lawful base, and what does it do there?** That is a fact about `main`, which
you can measure, and it is entirely independent of the held tree's refusal. Three
dispositions are available per row and they are exhaustive:

- **superseded** — the row's refusal was a property of the held mechanism, and no
  test on the lawful base carries the assertion. The row records a failure of
  something that does not exist here. This is expected to be the common case and
  it is a complete disposition, not a dodge.
- **live, with a current verdict** — a test on the lawful base carries the
  assertion. Run it and record what it says, either way.
- **open, with a named owner** — you cannot determine which of the two it is.

**A row that turns out to fail on the lawful base is a finding, not a
regression**, it does not retroactively invalidate seams 1-3, and it is routed
rather than repaired.

### The two host rows are a capacity precondition, not a semantic one

`/tmp` reached 99 percent on 2026-08-01 and produced linker
`No space left on device` failures that sat inside the 138-row census looking
like semantic rows.

**The underlying cause is fixed, not merely cleared.** On 2026-08-02 the
operator directed build scratch onto the repo volume, and `scripts/ken-cargo`
now exports `TMPDIR=/workspaces/ken/tmp` — roughly 20 GB free against the
7.8 GB `/tmp` tmpfs that the runtime suites were exhausting. See
`docs/ops/compute-budget.md` §1a.

**Still confirm capacity in the same shell as the run**, and confirm it on the
volume the run will actually use:

```sh
scripts/ken-cargo build -p ken-runtime   # exports TMPDIR; do this first
df -h /workspaces
```

⇒ **A host row that fails with a disk error is not a semantic result and no
semantic inference may be drawn from it.** Rerun it with capacity, or record it
as unmeasured. Do not classify it. ⚠ **If one of these two rows still fails with
`No space left on device` on the new volume, that is a finding worth routing** —
20 GB is not consumed by a lib suite, so the run would be reaching a temp path
that does not honour `TMPDIR`.

## Fixed inputs

Measured at `origin/main = 767bf20f`; the seam-2 and census rows re-stated at
`origin/main = 5d430082` on 2026-08-03. **Re-measure on your own tree before any
edit and quote it** — seams 2 and 3 both edit `static_transition.rs`, so
addresses move.

| input | measured value |
|---|---|
| seam 2 | `RT-CONTSPEC-ACTIVATE`, merged at `0a6e34cc`; clears **zero** census rows |
| seam 3 | `RT-CONTSPEC-LEDGER`, must be `merged` before this starts; its frame selects **zero** census rows |
| the corrected census | seam 1 `D4` — a first-refusal record of the held `1aef3192` lineage, **not** a worklist carried through seams 2 and 3 |
| the 761 witness gate | `fs_read_at_malformed_offset_narrows_to_invalid_offset`, and its sibling at `crates/ken-cli/tests/rt_parity_native.rs:544` |
| nodes closed on merge | **this node only, and the Steward flips it post-merge at M7.** The candidate carries no tracker `status:` change. `RT-RECURSOR-TRANSPORT` must NOT be closed -- see `D6` |
| prior-slice surfaces | the six, frozen at their `main` blobs |

## Two preconditions on every suite run, carried from seams 1-3

Both produced a false hard stop on seam 1 (`evt_3q972fhrnsr0b`, ruled
`evt_1pt7rmmw2k5d0`).

1. **Prove the tree in the same shell as the run** — `git rev-parse HEAD`
   immediately before the suite, quoted as the base.
2. **Build before you test** — `scripts/ken-cargo build -p ken-runtime` then
   `scripts/ken-cargo test -p ken-runtime --lib`. A `Toolchain`-stage
   `ObjectLinkerPackagingError` is an environment finding, not a baseline one.

> ## THE 761 WITNESS GATE — an open question, not a checkbox
>
> `fs_read_at_malformed_offset_narrows_to_invalid_offset` must produce
> `InvalidOffset`. It was observed passing on the capstone base `b66dea6a`
> (2026-08-01 19:48), along with its sibling.
>
> **That observation does not close the gate, because the question is why.** Did
> the trap become `InvalidOffset` because the defect was fixed, or **because the
> assertion moved**? A green test says nothing about which. This seam owes the
> discrimination, and the discrimination is a `git log`/`git diff` question about
> the assertion's own history, not another run.
>
> ⇒ **`D4` must name the commit that made the test green and say which of the two
> it did.** "Both pass" is not an answer to this AC.

## Deliverables

- **D1 — the native population run** on the lawful assembly, with full pass/fail
  counts and the proving `git rev-parse HEAD`.
- **D2 — the six formerly shadowed rows reclassified against the lawful base.**
  One row each: the first refusal that shadowed it in the held tree, whether the
  lawful base carries a test bearing that assertion, and the resulting
  disposition — `superseded`, `live` with its current verdict, or `open` with an
  owner. ⛔ **Do not write "the seam that cleared this root" for any row. No
  seam cleared any root**, and a row claiming one is a defect in `D2`, not a
  finding about the seam.
- **D3 — the two host rows rerun under confirmed capacity**, with the `df -h`
  output that established it, or recorded as unmeasured with the reason.
- **D4 — the 761 discrimination.** The commit that made
  `fs_read_at_malformed_offset_narrows_to_invalid_offset` green, and whether it
  fixed the defect or moved the assertion. Same for the
  `rt_parity_native.rs:544` sibling.
- **D5 — the campaign closeout record**: what the four seams established, the
  full 138-row disposition table (`superseded` / `live` with verdict / `open`
  with owner — one row each, this is `AC-1`'s control), and what remains open.
  ⛔ **State plainly in it that the census recorded first refusals of a mechanism
  that was never merged**, so a later reader does not mistake `superseded` for
  work that was silently dropped.
- **D6 — NO TRACKER WORK IS OWED BY THE RING. There is nothing to do here.**

  > **Rewritten 2026-08-08 by the Steward, twice, and the second correction is
  > the one that mattered.** Raised by `runtime-leader` asking a narrow workflow
  > question about candidate `baa45832`.
  >
  > **This said "the three-node tracker closure: this node,
  > `RT-RECURSOR-TRANSPORT`, `RT-DECL-CLOSURE-PORT`."** Both other entries are
  > wrong, for different reasons, and I removed only the first one on my first
  > pass:
  >
  > - `RT-DECL-CLOSURE-PORT` is already `merged`, which is terminal. Nothing to
  >   flip.
  > - **`RT-RECURSOR-TRANSPORT` MUST NOT BE CLOSED BY THIS NODE.** It is
  >   `status: ready`, **`size: L`**, and **its mechanism has never been built**
  >   — both classes it owns are live in production at `core.rs` (grep
  >   `MatchScrutineeRecursor` and `LexicalCallArgumentRecursor`). It is a
  >   `depends_on` of [[RT-DESCENT-RETIRE]]. **Closing it would mark unbuilt
  >   work resolved and unblock the lane deletion while two classes can still
  >   select it** — which is `RT-DESCENT-RETIRE`'s own banned scope: *"a partial
  >   deletion is strictly worse than none: it removes the fallback while a
  >   class can still select it."*
  >
  > **How the error survived my first correction, stated because it is the
  > reusable part:** I re-derived the *count* (three to two) and inherited the
  > *membership* unexamined. The list came from `RT-CONTSPEC-LOWER`, written
  > when `RT-RECURSOR-TRANSPORT` was to land in the same atomic candidate; the
  > recut split them and nobody re-derived the set. **Fixing the arithmetic of
  > an inherited premise reads as having checked it.**
  >
  > ⇒ **What remains of `D6` is this node's own flip, and that is the
  > Steward's at `merge-procedure.md` M7, after the publisher completes.**
  > **Put no `status:` change in the candidate.** A node cannot truthfully
  > record its own merge — the event it is recording has not happened, and a
  > rejected or superseded branch then carries a tracker asserting work landed.

- **D7 — INHERITED 2026-08-03 from [[RT-CONTSPEC-ACTIVATE]]: the behavioural
  target-dependence witness.** Architect ruling `evt_bz62dah3ecp0`. On the
  integrated, executed assembly, prove that **selecting the wrong same-shaped
  target changes or fails an observable result.** "Same-shaped" is
  `RT-WORKER-BIND`'s landed definition — same declared arity and same capture
  count. The redirect resolves the exact target first, then selects a *distinct*
  target on that predicate; never on origin inequality or ABI layout.

  **Why it is here and not in seam 2.** `ACTIVATE` closes at the emission seam:
  it proves the planner-issued target equals the emitted direct-call target. That
  is not a behavioural oracle — it passes even if the call result is discarded,
  both targets alias one body, or the callee body is malformed. Seam 2's own
  population cannot supply the stronger witness: six shapes measured at
  `a84dbfba` all refuse at the ordinary-`Closure` boundary before any answer is
  observable. **This seam is the first one that runs an integrated native
  assembly, so it is the first that can execute the witness lawfully.**

  ⛔ **The obligation was deferred, not discharged.** `ACTIVATE` merging does not
  close it. If this seam cannot execute it either, that is a routed finding, not
  a silent drop.

  ⚠ **AMENDED 2026-08-03 20:20 (`evt_6bf2mmehjzy3k`) — the two-target
  precondition is now this seam's to satisfy.** `ACTIVATE`'s generated function
  has exactly one entry in `continuation_calls`, so a same-shaped redirect
  refuses before reaching the call seam and `ACTIVATE` no longer attempts it.
  ⇒ **This seam's integrated fixture must supply at least two distinct
  same-shaped targets in one lawful callable population**, and the redirect must
  reach the seam and change or fail the observed result. ⛔ A pre-call "found no
  distinct same-shaped call target" refusal is a missing fixture precondition,
  not a discharge — and it is the precondition this seam owns.

## Acceptance criteria

- **AC-1 — every one of the corrected census's 138 rows has a terminal
  disposition, as a historical record.** Each row is `superseded` (its refusal
  belonged to the held mechanism and the lawful base carries no test bearing the
  assertion), `live` with a current verdict measured on the lawful base, or
  `open` with a named owner. No row is unaccounted for.
  *Control:* the 138-row set against `D5`'s disposition table — every row appears
  exactly once and carries one of the three.
  ⛔ **A disposition of the form "resolved by seam N" is invalid for every row
  and fails this AC.** Seams 2 and 3 were both recut off the census and resolve
  nothing in it; a row so marked means the false premise was reintroduced.
- **AC-2 — the six shadowed rows carry a stated disposition each**, and no row is
  recorded as "still shadowed" or as awaiting a clearing.
  *Control:* `D2`'s six rows; a grep for shadow-language and for
  cleared-root language both return nothing.
- **AC-3 — `D4` names a commit and picks one of the two explanations** for each
  of the two 761 tests. A green run does not discharge this AC.
  *Control:* read `D4` against `git log` on the named test.
- **AC-4 — the two host rows are either rerun under stated capacity or recorded
  as unmeasured.** Neither may be classified semantically.
  *Control:* `D3`, with its `df -h` evidence.
- **AC-5 — no fresh planner or ABI failure was repaired in this seam.** If one
  appeared, it is routed, not fixed.
  *Control:* the candidate's path list — no planner or ABI file appears.
- **AC-6 — the prior-slice surfaces are blob-identical to the merge base.**
  *Control:* `git rev-parse` per surface, candidate against merge base.
- **AC-7 — the candidate contains NO tracker `status:` change.**
  *Control:* `git diff` over `docs/program/issues/` on the candidate is empty of
  `status:` lines. **Discharged by the handback stating that you made no tracker
  change, not by a tracker diff.**
  **Rewritten 2026-08-08** — this previously required flips the ring must not
  make. This node's own flip is the Steward's at M7; `RT-RECURSOR-TRANSPORT`
  must not be closed at all (see `D6`), and `RT-DECL-CLOSURE-PORT` is already
  terminal.
- **AC-8 — CI green** on the merge.

- **AC-9 — INHERITED from [[RT-CONTSPEC-ACTIVATE]] `D4` item 4 (2026-08-03,
  `evt_bz62dah3ecp0`): a wrong same-shaped target changes or fails an observable
  result.** Redirect a causal token to the other same-shaped target — same
  declared arity, same capture count — on the integrated executed assembly; the
  observed result must change or fail.
  *Control:* `D7`, with a **committed** `cfg(test)`-gated switch on the exact
  production branch whose red reproduces from the committed tree. ⛔ A hand-run
  mutation does not discharge this.
  ⛔ **A green that shows only "which target was claimed changed" does not
  discharge this AC** — that is `ACTIVATE`'s `AC-2`, already met, and it observes
  the mutation changing the field it mutates. This AC needs the **result**.
  ⚠ If the integrated assembly cannot execute the witness, say so with the
  refusal and route it. Do not weaken `boundary_transfer_admissibility`,
  fabricate a durable or borrowed closure lane, change planner or ABI
  populations, or fall back on a `0/0` witness — all remain banned outright.

## Banned scope

- **No planner or ABI repair, at all.** This is the seam where that boundary
  matters most; see the rule above.
- **No repair of a shadowed row that turns out to fail.** Record it and route
  it. A failing shadowed row is a measurement this campaign existed to produce.
- **No semantic inference from a host disk failure.**
- **No edit to any prior-slice surface** (`AC-6`).
- **No merge, rebase, or wholesale cherry-pick of any preserved object.**
- **No tracker `status:` change of any kind in the candidate** (`AC-7`). This
  previously read *"no closing of any node other than the three named"*, which
  presumed a closure set `D6` no longer has. The ring closes nothing; the
  Steward flips this node alone, post-merge, at `merge-procedure.md` M7.
- **No test asserting facts about source or documentation lines** (operator test
  policy). `D2`, `D4` and `D5` are review artifacts, not gates.

## Contention

Runtime is single-threaded. `D1` and `D3` both need the shared build lock; probe
without blocking first. **Targeted only** — never `--workspace`. `D3` also needs
disk headroom, so check `df -h /workspaces` before taking the lock, and **do not
reclaim scratch while any seat holds the build turn.**

## Sizing

**Size `M`.** Mostly measurement and record-keeping, with one genuine
investigation (`D4`). The risk is not length but **scope creep at the boundary**:
every fresh failure this seam surfaces will look like a small fix.

⇒ **Commit at these three checkpoints and post the exact SHA at each:**

1. `D1` native population run.
2. `D2` and `D3` — reclassification and host rerun.
3. `D4` discrimination and `D5` closeout record. **`D6` is not a checkpoint --
   it owes the ring no work at all.**

## Hard stops

Stop and route to the Steward, do not improvise, if any of these hold:

1. **A fresh planner or ABI failure appears on the lawful assembly.** Route it as
   a new exact interface hard stop with its first refusal quoted.
2. **A formerly shadowed row fails.** Record and route; do not repair.
3. **`D4` cannot discriminate fixed-from-moved** for either 761 test. That is a
   real finding about the campaign's evidence, and it belongs to the Architect.
4. **A census row resists all three dispositions** (`AC-1`) — you can neither
   show the lawful base carries no test bearing its assertion, nor find one to
   run, nor name an owner for it. ⛔ **This is no longer evidence that a seam's
   population boundary was wrong**, which is what the prior cut inferred; seams 2
   and 3 have no census population to be wrong about. It means the census row
   itself is unreadable, which is a finding about seam 1's `D4` record.
5. **Disk capacity cannot be established** for `D3` and no seat is free to
   release it.
6. **You find a node that looks like it should be closed by this seam.** Do not
   close it and do not propose a set. `D6` closes this node only, the Steward
   performs it post-merge, and a DAG edge that appears wrong is the Steward's
   call -- routing it is correct, acting on it is not. **This is how the
   `RT-RECURSOR-TRANSPORT` error nearly landed.**
