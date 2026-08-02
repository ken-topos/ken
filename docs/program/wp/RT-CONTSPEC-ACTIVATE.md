# WP frame — `RT-CONTSPEC-ACTIVATE` (ContinuationSpecialization seam 2 of 4)

Node: `docs/program/issues/RT-CONTSPEC-ACTIVATE.md`. Campaign:
`docs/program/16-recursive-descent-retirement.md`. Owner: runtime ring.
Authority: Architect ownership/sizing ruling `evt_1yymw1gdszpbs`, outcome (c),
seam 2.

This is the seam that turns the mechanism on. Seam 1 put the accepted helper on
the landed substrate and proved it changed nothing; this seam makes it the
authority for one bounded population and proves exactly that population moved.

> ## The `46d29783` lineage is still an ORACLE, not a base
>
> Everything seam 1's banner says continues to bind here. `46d29783` (census),
> `1aef3192` (parent), `9d58df12` (accepted mechanism), and
> `refs/preserved/rt-contspec-lower-held-core-rs = 88972207` are preserved and
> **may not be merged, rebased onto, or cherry-picked wholesale.** This seam
> branches from `main` after seam 1 lands and carries only its own delta.

## The population, and why it cannot be enumerated by grep alone

The Architect's matrix assigns **37 rows** to capstone lowering
activation/consumption, by four first-refusal kinds: inactive emitted owner,
missing `JoinArm` use, duplicate declared-unit call, and runtime discriminator
failure.

The durable census at `46d29783` labels only **35** of them. Counted over
`docs/program/wp/RT-CONTSPEC-LOWER-D8.md` at that commit:

| first refusal in the census | rows | census owner label |
|---|---:|---|
| `lowering boundary use has no active emitted owner` | 31 | `CONTSPEC lower` |
| `lowering JoinArm boundary-use ledger missing` | 4 | `CONTSPEC lower` |
| duplicate declared-unit call | 0 | not yet differentiated |
| runtime discriminator failure | 0 | not yet differentiated |

⇒ **The remaining 2 lower-owned rows are inside the 39 rows the census records
only as "assertion after unruled production activation" / "ownership matrix
pending."** That is the fidelity defect the Architect named, and **seam 1's `D4`
is what repairs it.**

The partition closes exactly, which is why this is arithmetic and not an
estimate. Differentiated rows: 37 + 12 + 1 + 31 + 4 + 12 + 2 = **99**.
Undifferentiated: **39**. Total **138**. The 39 decompose across the matrix as
planner 19, ABI 2, substrate 3, planned-use 2, **lower 2**, D7 ledger 5, cascade
6 — summing to 39, and every population then reaching its matrix total.

> ### BINDING — the scope oracle is `D4`'s corrected census, not the `46d29783` one
>
> **Do not start row selection until seam 1's `D4` is on `main`.** Selecting from
> the uncorrected census gives you 35 rows and silently drops 2. The frame states
> the 35 so you can sanity-check `D4`, **not** so you can proceed without it.

> ### THE 37/37 COLLISION — read this before you grep for "37"
>
> **"37" names two disjoint populations in this campaign.** In the census,
> `producer callable identity is not a Closure` is **37 rows owned by
> `CONTSPEC planner`**. In the Architect's matrix, **lower-owned is 37 rows.**
> They share a number and nothing else.
>
> The planner population sits inside the 56-row slice-1 planner closure, which is
> **forbidden prior-slice surface** for this seam. Grepping the census for `37`,
> or filtering on the wrong owner column, hands you precisely the rows this seam
> may not touch. **Select by first-refusal kind and owner label together, never
> by count.**

## Fixed inputs

Measured at `origin/main = 4e6b0744`.

| input | measured value |
|---|---|
| seam 1 | `RT-CONTSPEC-ASSEMBLY`, must be `merged` before this starts |
| the corrected census | seam 1 `D4`, path fixed by that seam; the 138 rows with one row per failing test and no `ownership matrix pending` placeholder |
| the accepted helper | `CheckedFrameBranchScope` plus its feature-gated harness, established on `main` by seam 1 and already carrying the existing per-branch scoping at the `lower_forked_branch` forks. **It changes no emission authority** — that is this seam's job, and it is what "activation" means here (ruled `evt_5p6exqgphrwxj`) |
| lowering entry point | `crates/ken-runtime/src/cranelift_backend/lowering/core.rs` |
| prior-slice surfaces | `planning/static_transition.rs`, `planning/static_transition/abi.rs`, `planning/static_transition/semantic_ir.rs`, `planning.rs`, `boundary_value.rs`, `boundary_value_clif.rs` — all frozen at their `main` blobs |
| baseline suite | `scripts/ken-cargo test -p ken-runtime --lib` |

Reproduce, read-only:

```sh
git rev-parse HEAD                       # prove the tree BEFORE anything else
git log --oneline -1 origin/main
git show origin/main:docs/program/issues/RT-CONTSPEC-ASSEMBLY.md | grep '^status:'
```

## Two preconditions on every suite run, carried from seam 1

Both produced a false hard stop on seam 1 (`evt_3q972fhrnsr0b`, ruled
`evt_1pt7rmmw2k5d0`). Neither is optional here.

1. **Prove the tree in the same shell as the run.** `git rev-parse HEAD`
   immediately before the suite; quote its output as the base. A `git switch`
   onto a branch held by another worktree fails **silently** inside an `&&`
   chain and the chain runs the suite in the old tree.
2. **Build before you test.** `crates/ken-runtime` is
   `crate-type = ["rlib", "staticlib"]` and `cargo test --lib` never emits
   `libken_runtime.a`. Without it, `ken_runtime_staticlib()`
   (`object_linker_packaging.rs:1211`) finds no archive and ~40 rows fail with a
   `Toolchain` error whose text names ken-host. Run:

   ```sh
   scripts/ken-cargo build -p ken-runtime
   scripts/ken-cargo test  -p ken-runtime --lib
   ```

⇒ **A `Toolchain`-stage `ObjectLinkerPackagingError` is an environment finding,
not a baseline finding.** Check for `target/debug/**/libken_runtime*.a` before
routing one as a hard stop.

## Deliverables

- **D1 — the activation.** Direct call **before** the identity-erasing join;
  active emitted owner; affine call occurrence; `JoinArm` consumption. This is
  the whole mechanism content of the seam.
- **D2 — the selected population, written down before any edit.** One row per
  member of the 37, each carrying: test name, first refusal, and the `D4` owner
  label that put it in scope. **Authored from `D4`, not from `46d29783`.**
- **D3 — wrong-owner and duplicate-use controls.** A negative control that fails
  when the emitted owner is wrong, and one that fails when a call occurrence is
  used twice. These are the discriminators that keep `D1` from passing for the
  wrong reason.
- **D4 — the before/after row disposition.** For each of the 37: its status on
  this seam's own base, and its status on the candidate. Rows outside the 37 get
  a single aggregate line stating the count unchanged.

## Acceptance criteria

- **AC-1 — every row in `D2` traces to a `D4` row with a lower-owned label**, and
  `D2` has exactly 37 rows.
  *Control:* the `D2` name set against the corrected census, both directions.
  A `D2` of 35 rows fails this AC — it means the uncorrected census was used.
- **AC-2 — the 37 selected rows pass on the candidate.**
  *Control:* the run output, pass/fail counts shown, `git rev-parse HEAD` in the
  same block. `--no-run` does not discharge this.
- **AC-3 — no row outside the 37 changes status.** Neither direction: a row that
  starts failing is a regression, and a row that starts passing means the
  activation reached outside its population.
  *Control:* the full pass/fail set on this seam's base against the candidate's,
  differenced. The symmetric difference must be exactly the 37.
- **AC-4 — the prior-slice surfaces are blob-identical to the merge base.**
  *Control:* `git rev-parse <candidate>:<path>` against
  `git rev-parse <merge-base>:<path>` for each of the six surfaces. Any
  inequality fails the seam outright.
- **AC-5 — `D3`'s controls fail when mutated.** Flip the emitted owner; the
  wrong-owner control must go red. Use a call occurrence twice; the duplicate-use
  control must go red. **A control that stays green under its own mutation is not
  a control.**
  *Control:* both mutations run and shown red, then reverted. Commit the real
  fix before any mutation proof, and reset after.
- **AC-6 — CI green** on the merge. Workspace-green means green in CI, never a
  local `--workspace` run.

## Banned scope

- **No planner or ABI repair**, however well-evidenced a refusal looks. On a
  lawful assembly a planner- or ABI-worded refusal is a **new interface fact**:
  it routes back as an exact hard stop under seam 4's rule. It is not repaired
  here, and it is not a reason to reopen a merged slice.
- **No D7 population expansion.** The 17 ledger and representation rows are seam
  3. Widening into them rebuilds the mis-sizing that produced this recut.
- **No edit to any prior-slice surface** (`AC-4`).
- **No merge, rebase, or wholesale cherry-pick of any preserved object.**
- **No test asserting facts about source or documentation lines** (operator test
  policy). `D2` and `D4` are review artifacts, not gates.

## Contention

Runtime is single-threaded and this seam edits `lowering/` only. Take the shared
build lock for `AC-2`/`AC-3`; probe without blocking first. **Targeted only:**
`scripts/ken-cargo test -p ken-runtime --lib`. **Never `--workspace`** — the
full-workspace build, the `--locked` gate and conformance run in CI.

## Sizing

**Size `L`, and that is the one thing about this seam worth watching.** The
Architect fixed the seam boundaries; the risk is not the boundary but the turn
length. An uninterrupted implementer run past 60 minutes is an indication the
work wants a further cut.

⇒ **Commit at these four checkpoints, and post the exact SHA at each.** They are
natural boundaries, not a re-cut, and they mean a long seam never sits as one
uninspectable turn:

1. `D2` written from the corrected census — no production edit yet.
2. `D1` direct-call-before-join and active emitted owner.
3. `D1` affine call occurrence and `JoinArm` consumption.
4. `D3` controls plus their mutation proofs, then `D4`.

If checkpoint 2 alone runs past an hour, **stop and route** — that is a sizing
finding, and the recut is the Steward's.

## Hard stops

Stop and route to the Steward, do not improvise, if any of these hold:

1. **`D4`'s corrected census does not yield exactly 37 lower-owned rows.** The
   partition is then wrong, and selection cannot proceed from a count nobody can
   reproduce.
2. **The activation cannot be made without touching a prior-slice surface.**
   That is an interface fact, exactly as it was at seam 1 — not a small
   exception.
3. **A planner- or ABI-worded refusal appears on the lawful assembly.** New
   interface fact; route it, do not repair it.
4. **A row outside the 37 changes status in either direction** (`AC-3`). The
   population boundary is wrong and the matrix needs the Architect, not a scope
   adjustment.
5. **A `D3` control stays green under its own mutation** (`AC-5`). The control is
   not measuring what it claims and the activation is unproved.
