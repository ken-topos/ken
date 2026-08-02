# WP frame — `RT-CONTSPEC-ASSEMBLY` (ContinuationSpecialization seam 1 of 4)

Node: `docs/program/issues/RT-CONTSPEC-ASSEMBLY.md`. Campaign:
`docs/program/16-recursive-descent-retirement.md`. Owner: runtime ring.
Authority: Architect ownership/sizing ruling `evt_1yymw1gdszpbs`, outcome (c).

> ## READ THIS BEFORE ANYTHING ELSE — the held work is an ORACLE, not a base
>
> `RT-CONTSPEC-LOWER` did not produce a repairable candidate. Its lineage
> **replaced the landed slice 0-2 blobs with older cumulative WIP.** Preserve
> `46d29783` (census), `1aef3192` (parent),
> `9d58df12` (accepted mechanism), and
> `refs/preserved/rt-contspec-lower-held-core-rs = 88972207`.
>
> **None of them may be merged, rebased onto, or cherry-picked wholesale.** The
> accepted helper is re-established **by hand onto current `main`**.
>
> If that instruction feels wasteful, read the measurement below: it is why the
> 138-row run cannot be interpreted at all on the held tree.

## The fact this seam exists to establish

Measured by the Architect against the rule-derived base `b66dea6a`, and
**independently re-verified by the Steward** at `origin/main = 40f8757d`:

| surface | held lineage vs `b66dea6a` | blob on `main` | blob at `b66dea6a` |
|---|---|---|---|
| `planning/static_transition.rs` | +20,199 / -6,120 | `6725971d` | `6725971d` |
| `planning/static_transition/abi.rs` | +748 / -87 | `23b9f5d7` | `23b9f5d7` |
| `planning/static_transition/semantic_ir.rs` | +75 / -48 | — | — |
| `planning.rs` | +21 / -9 | — | — |
| `boundary_value.rs` | +32 / -28 | — | — |
| `boundary_value_clif.rs` | +57 / -2 | — | — |

`main` and `b66dea6a` agree exactly on the two surfaces checked. **The held
lineage is the outlier, not `main`.**

⇒ **76 of the 138 failing rows are evidence against the assembly boundary**, not
a warrant to reopen three merged slices. 56 rows name slice-1 planner closure,
15 name the planner-to-ABI seam, 3 name slice-0 substrate, 2 name the
planner-to-lowering handoff. On this lineage every one of them is explained by
forbidden prior-slice replacement.

## Fixed inputs

Measured at `origin/main = 40f8757d`.

| input | value |
|---|---|
| base for this seam | `origin/main`, **not** any preserved object |
| slice 0/1/2 surfaces | at their landed blobs; this seam changes none of them |
| the accepted mechanism | `CheckedFrameBranchScope` plus its feature-gated harness, accepted and frozen by the Architect at `9d58df12` |
| the census oracle | `46d29783`, tree `e81d315d`, 136 semantic class-3 rows + 2 host `ENOSPC` |
| `/tmp` capacity | cleared 2026-08-02 from 99 percent to 23 percent, 6.0G free. The two `ENOSPC` rows are **host artifacts with no semantic verdict** |

## Deliverables

- **D1 — the extracted helper.** `CheckedFrameBranchScope` and its
  feature-gated harness, re-established on current `main`,
  **unactivated**. No call site
  selects it; no behaviour changes.
- **D2 — the untouched-surface proof.** A blob-identity table showing every
  slice 0-2 surface listed above at the candidate equals its blob on the
  candidate's merge base. **Blob identity, not a diff summary** — a `--stat`
  showing no change is not the same claim.
- **D3 — the composition record.** A short statement of how the helper composes
  with the landed planner/ABI/substrate interfaces **as they are**, naming any
  interface it needs that does not exist.
  ⛔ **The helper must be CONNECTED to the live state it claims to encapsulate,
  and the harness must exercise THAT state — not a set of its own
  construction.** Specifically: `CheckedFrameBranchScope` must operate on
  `Lowering::consumed_subcontinuation_frames`
  (`lowering/mod.rs:1046`, `BTreeSet<(u64, u64)>`), and `D3` must name the field
  and show the connection. A helper that compiles beside the live field without
  touching it does **not** discharge this.
- **D4 — the recut census correction.** `docs/program/wp/RT-CONTSPEC-LOWER-D8.md`
  currently describes **39 rows** as only "assertion after unruled production
  activation" / "ownership matrix pending", although the captured run
  contains their exact refusals — including the 17
  source-position-outside-ABI-input rows
  and 3 synthesized-ledger rows. **Replace that undifferentiated family with the
  Architect's ownership matrix.** This is a documentation repair of an existing
  artifact, and it is in scope precisely because a later seam would otherwise
  inherit a census that hides its own population.

## Acceptance criteria

- **AC-1 — every slice 0-2 surface is blob-identical to the merge base.**
  *Control:* `git rev-parse <candidate>:<path>` against
  `git rev-parse <merge-base>:<path>` for each of the six surfaces. Every pair
  must be equal. **This is the seam's central claim; it is not discharged by a
  green suite.**
- **AC-2 — the Runtime lib suite is green.** `scripts/ken-cargo test -p
  ken-runtime --lib`, compiled **and run**. `--no-run` does not discharge this —
  that exact substitution is what let the previous node reach a 138-failure
  surprise.
  *Control:* the run output, pass/fail counts shown.
- **AC-3 — nothing is activated.** No production path selects the helper; the
  emission authority selected for every program is unchanged from the merge
  base.
  *Control:* a before/after authority comparison on the governed programs, both
  taken on this seam's own base.
- **AC-4 — the candidate's diff touches no planner, ABI, or substrate file.**
  *Control:* the path list of `git diff --name-only <merge-base> <candidate>`.
  Any of the six surfaces appearing here fails the seam outright.
- **AC-5 — `D4`'s corrected census carries one row per failing
  test**, each with test name, first refusal, phase, D8 class, causal root or
  cascade membership,
  and owning contract. No row may read "ownership matrix pending".
  *Control:* row count equals 138; grep for the placeholder text returns
  nothing.
- **AC-6 — CI green** on the merge. Workspace-green means green in CI, never a
  local `--workspace` run.

## Banned scope

- **No activation of any kind.** That is seam 2.
- **No edit to any slice 0-2 surface** (`AC-4`). If the helper cannot compose
  without one, that is **hard stop 1**, not a small exception.
- **No planner or ABI repair**, however well-evidenced a refusal looks. On a
  lawful assembly such a refusal is a new interface fact; on this one it is
  expected noise.
- **No merge, rebase, or wholesale cherry-pick of any preserved object.**
- **No D7 ledger work** — that is seam 3.
- **No test asserting facts about source or documentation lines** (operator
  test policy).

## Contention

Runtime is single-threaded and this seam edits `lowering/` only. Take the shared
build lock for `AC-2`; probe without blocking first. **Targeted only:**
`scripts/ken-cargo test -p ken-runtime --lib`. **Never `--workspace`** — the
full-workspace build, the `--locked` gate and conformance run in CI.

### Two preconditions on every `AC-2` run, and on any baseline you report

Both of these produced a false hard stop 2 on this seam (`evt_3q972fhrnsr0b`,
ruled `evt_1pt7rmmw2k5d0`). Neither is optional.

**1. Stand in the right tree, and prove it in the same shell.** Run
`git rev-parse HEAD` immediately before the suite and quote its output as the
base. A `git switch` onto a branch already checked out in another worktree fails
**silently** when it sits inside an `&&` chain, and the chain then runs the suite
in the old tree. Measured here: the leader's worktree was 204 commits behind the
base its report named. ⇒ **State the base you measured, never the base you
intended.**

**2. Build before you test — `cargo test --lib` does not emit the staticlib.**
`crates/ken-runtime/Cargo.toml` declares
`crate-type = ["rlib", "staticlib"]`.
`cargo test --lib` builds only the rlib for the harness, so
`libken_runtime.a` is never produced. `object_linker_packaging.rs:1211`
`ken_runtime_staticlib()` then finds no archive and every row that links one
fails with a `Toolchain` error
whose text names ken-host — stale wording; the function looks for
`libken_runtime.a`. In a worktree that has never run a build this is **~40
failures that say nothing about the base.** So:

```sh
scripts/ken-cargo build -p ken-runtime     # materializes libken_runtime.a
scripts/ken-cargo test  -p ken-runtime --lib
```

⇒ **A `Toolchain`-stage `ObjectLinkerPackagingError` is an environment finding,
not a baseline finding.** Check for `target/debug/**/libken_runtime*.a` before
routing one as a hard stop.

## Sizing

**Size `M`, and it is deliberately the smallest seam.** It adds no behaviour.
Its whole content is: put the accepted helper on the landed
substrate, prove the substrate did not move, and correct the census.

If it turns out large, the reason will be `D3` — the helper needing interfaces
the landed slices do not expose. **That is information, not scope**, and it
routes (hard stop 1).

## Hard stops

Stop and route to the Steward, do not improvise, if any of these hold:

1. **The helper cannot compose without editing a prior-slice surface.** That is
   an **interface fact**, and it is the single most valuable thing this seam can
   discover. It is **not** permission to import the cumulative WIP, and it is
   not a small exception to `AC-4`. Report the exact interface needed.
   ⛔ **A helper that composes with NOTHING does not satisfy this stop — it
   evades it.** As written, this stop was vacuously satisfiable by a dormant,
   disconnected helper, and that is exactly what happened on `a7ab9efa`
   (rejected, `dec_6xa6c5v0y9q6g`): the frozen helper keyed
   `(Option<PredeclaredFunctionId>, u64, u64)` while the live field is
   `BTreeSet<(u64, u64)>`, nothing joined them, and the harness built its own
   triple-key set. **The green run proved dormancy, not composition.** The stop
   fires only when the helper is connected to the live field and *then* cannot
   proceed. See `D3`.
2. **The Runtime lib suite is red on current `main` before your delta.** Then
   the baseline is not what this recut assumes and nothing downstream is
   interpretable. Report the failing rows.
3. **The census cannot be corrected to one row per test** because the captured
   run does not contain a row's exact refusal. Report which rows and what is
   missing; do not synthesize a classification.
4. **Any preserved object turns out to be unreachable.** Report it before doing
   anything else — the census oracle is the only record of the 138-row run.
