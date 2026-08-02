# WP frame — `RT-CONTSPEC-WITNESS` (ContinuationSpecialization seam 4 of 4)

Node: `docs/program/issues/RT-CONTSPEC-WITNESS.md`. Campaign:
`docs/program/16-recursive-descent-retirement.md`. Owner: runtime ring.
Authority: Architect ownership/sizing ruling `evt_1yymw1gdszpbs`, outcome (c),
seam 4 — the terminal seam.

Seams 1-3 built and proved the mechanism. This seam **measures the result on a
lawful assembly** and closes the campaign's held nodes. It is the only seam that
closes anything other than itself.

> ## HELD FOR RECUT — do not start this seam. Its node is `draft` on purpose.
>
> **Steward, 2026-08-02.** `AC-1` requires every one of the corrected census's
> 138 rows to reach a terminal disposition, and the frame's row table asserts
> that 6 formerly shadowed rows have "their causal roots cleared in seams 2 and
> 3." **That premise is now false.** Seam 2 was recut off the census entirely
> (`evt_2zhx69f2fw07w`, Architect confirmation `evt_66t42tapvdbsj`), and seam 3
> was recut off it for the same reason on 2026-08-02 — its subject is now the
> boundary-use vocabulary's production reachability, and it selects no census
> rows. **Neither seam clears any census row, and this is settled, not
> pending.**
>
> The defect here is narrower than seam 3's. Reconciling the 138 rows is a
> reasonable bookkeeping deliverable in itself — but it must be stated as
> **accounting for a historical record**, not as measuring work the earlier
> seams performed. As written it would have every row's disposition depend on
> clearing that will never happen.
>
> The node is `draft` so this frame cannot enter the frontier. The Steward
> recuts it after seams 2 and 3.

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
| formerly shadowed rows | 6 | their causal roots cleared in seams 2 and 3; until then their assertions were shadowed by a first refusal and said nothing |
| host `ENOSPC` rows | 2 | disk capacity, not semantics — see below |

The six shadowed rows are the subtle population. **A shadowed row is not a
failing row and it is not a passing row — it is an unmeasured one.** Its
assertion never ran because an earlier refusal stopped the test. Reclassifying
them means running them and recording what they actually say, which may be
either verdict. **A shadowed row that turns out to fail is a finding, not a
regression**, and it does not retroactively invalidate seams 1-3.

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

Measured at `origin/main = 767bf20f`.

| input | measured value |
|---|---|
| seam 3 | `RT-CONTSPEC-LEDGER`, must be `merged` before this starts |
| the corrected census | seam 1 `D4`, as carried through seams 2 and 3 |
| the 761 witness gate | `fs_read_at_malformed_offset_narrows_to_invalid_offset`, and its sibling at `crates/ken-cli/tests/rt_parity_native.rs:544` |
| nodes closed on merge | this node, `RT-RECURSOR-TRANSPORT`, `RT-DECL-CLOSURE-PORT` |
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
- **D2 — the six formerly shadowed rows reclassified.** One row each: the first
  refusal that shadowed it, the seam that cleared that root, and what the
  assertion says now. Verdicts may be either way.
- **D3 — the two host rows rerun under confirmed capacity**, with the `df -h`
  output that established it, or recorded as unmeasured with the reason.
- **D4 — the 761 discrimination.** The commit that made
  `fs_read_at_malformed_offset_narrows_to_invalid_offset` green, and whether it
  fixed the defect or moved the assertion. Same for the
  `rt_parity_native.rs:544` sibling.
- **D5 — the campaign closeout record**: what the four seams established, what
  the corrected census's 138 rows resolved to, and what remains open.
- **D6 — the three-node tracker closure**, in one commit: this node,
  `RT-RECURSOR-TRANSPORT`, `RT-DECL-CLOSURE-PORT`.

## Acceptance criteria

- **AC-1 — every one of the corrected census's 138 rows has a terminal
  disposition.** Resolved by a seam, reclassified here, or explicitly open with
  an owner. No row is unaccounted for.
  *Control:* the 138-row set against the union of the four seams' dispositions.
- **AC-2 — the six shadowed rows are reclassified with a stated verdict each**,
  and no row is recorded as "still shadowed."
  *Control:* `D2`'s six rows; grep for shadow-language returns nothing.
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
- **AC-7 — `D6` closes exactly three nodes in one commit**, and each closed node
  names this seam as what closed it.
  *Control:* the tracker diff; three `status` flips, one commit.
- **AC-8 — CI green** on the merge.

## Banned scope

- **No planner or ABI repair, at all.** This is the seam where that boundary
  matters most; see the rule above.
- **No repair of a shadowed row that turns out to fail.** Record it and route
  it. A failing shadowed row is a measurement this campaign existed to produce.
- **No semantic inference from a host disk failure.**
- **No edit to any prior-slice surface** (`AC-6`).
- **No merge, rebase, or wholesale cherry-pick of any preserved object.**
- **No closing of any node other than the three named** (`AC-7`).
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
3. `D4` discrimination, `D5` closeout record, `D6` three-node closure.

## Hard stops

Stop and route to the Steward, do not improvise, if any of these hold:

1. **A fresh planner or ABI failure appears on the lawful assembly.** Route it as
   a new exact interface hard stop with its first refusal quoted.
2. **A formerly shadowed row fails.** Record and route; do not repair.
3. **`D4` cannot discriminate fixed-from-moved** for either 761 test. That is a
   real finding about the campaign's evidence, and it belongs to the Architect.
4. **The census's 138 rows do not fully account** (`AC-1`). A row with no
   disposition means a seam's population boundary was wrong.
5. **Disk capacity cannot be established** for `D3` and no seat is free to
   release it.
6. **Closing the three nodes reveals a fourth that depended on them.** The DAG
   edge is wrong and the closure is the Steward's call.
