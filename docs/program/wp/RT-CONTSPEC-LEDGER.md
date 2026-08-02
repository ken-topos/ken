# WP frame — `RT-CONTSPEC-LEDGER` (ContinuationSpecialization seam 3 of 4)

Node: `docs/program/issues/RT-CONTSPEC-LEDGER.md`. Campaign:
`docs/program/16-recursive-descent-retirement.md`. Owner: runtime ring.
Authority: Architect ownership/sizing ruling `evt_1yymw1gdszpbs`, outcome (c),
seam 3.

Seam 2 turned the mechanism on for the lowering population. This seam closes the
**D7 ledger** over a population that is, for the first time in this campaign,
complete. The governing authorities are already ruled — this seam **applies**
them and does not re-derive them.

> ## The `46d29783` lineage remains an ORACLE
>
> `46d29783`, `1aef3192`, `9d58df12`, and
> `refs/preserved/rt-contspec-lower-held-core-rs = 88972207` are preserved and
> **may not be merged, rebased onto, or cherry-picked wholesale.** This seam
> branches from `main` after seam 2 lands and carries only its own delta.

## The population

The Architect's matrix assigns **17 rows** to existing D7 atomic
ledger/population closure: exact source and synthesized-aggregate ledger gaps,
plus the D7 closure, representation and lifetime controls.

As at seam 2, the durable census differentiates only part of it. Counted over
`docs/program/wp/RT-CONTSPEC-LOWER-D8.md` at `46d29783`:

| first refusal in the census | rows | census owner label |
|---|---:|---|
| `source boundary-use ledger missing` | 12 | `D7 closure port` |
| synthesized-aggregate ledger gaps | 0 | inside the undifferentiated 39 |
| representation / lifetime controls | 0 | inside the undifferentiated 39 |

⇒ **5 of the 17 are inside the 39 rows the census records only as "ownership
matrix pending."** Selecting from the uncorrected census gives 12 rows and
drops 5.

> ### BINDING — the scope oracle is the corrected census
>
> Seam 1's `D4` is what differentiates those rows. **Do not start row selection
> from `46d29783`.** The 12 above are stated so you can sanity-check the
> corrected census, not so you can proceed without it.

## Fixed inputs

Measured at `origin/main = 767bf20f`.

| input | measured value |
|---|---|
| seam 2 | `RT-CONTSPEC-ACTIVATE`, must be `merged` before this starts |
| the corrected census | seam 1 `D4` — 138 rows, one per failing test, no `ownership matrix pending` placeholder |
| the ruled authorities | the graph-derived D7 authorities already ruled; this seam cites them, it does not reopen them |
| prior-slice surfaces | `planning/static_transition.rs`, `planning/static_transition/abi.rs`, `planning/static_transition/semantic_ir.rs`, `planning.rs`, `boundary_value.rs`, `boundary_value_clif.rs` — frozen at their `main` blobs |
| baseline suite | `scripts/ken-cargo test -p ken-runtime --lib` |

Reproduce, read-only:

```sh
git rev-parse HEAD                       # prove the tree BEFORE anything else
git show origin/main:docs/program/issues/RT-CONTSPEC-ACTIVATE.md | grep '^status:'
```

## Two preconditions on every suite run, carried from seams 1 and 2

Both produced a false hard stop on seam 1 (`evt_3q972fhrnsr0b`, ruled
`evt_1pt7rmmw2k5d0`). Neither is optional.

1. **Prove the tree in the same shell as the run.** `git rev-parse HEAD`
   immediately before the suite; quote its output as the base. A `git switch`
   onto a branch held by another worktree fails silently inside an `&&` chain.
2. **Build before you test.** `crates/ken-runtime` is
   `crate-type = ["rlib", "staticlib"]`; `cargo test --lib` never emits
   `libken_runtime.a`, and without it `ken_runtime_staticlib()` fails ~40 rows
   with a `Toolchain` error whose text names ken-host.

   ```sh
   scripts/ken-cargo build -p ken-runtime
   scripts/ken-cargo test  -p ken-runtime --lib
   ```

> ## THE TRAP THIS SEAM SITS DIRECTLY ON TOP OF — read before writing any proof
>
> **A proof over an incomplete population is vacuous, and every control over it
> passes.** That is exactly what rejected `RT-JOIN-DISPOSITION`'s `27f9dca2`: one
> production site bypassed the recording call, so a whole class proved over an
> empty list and every assertion was green.
>
> ⇒ **Every proof this seam adds over a population owes a paired control that
> REDS when a member is omitted from that population.** A control that merely
> passes when the proof holds is not evidence — it is the failure mode itself.
>
> This is a ledger seam. Its entire content is claims of the form "every X is
> recorded." That sentence is true of the empty set. **The omission control is
> what makes it say anything.**

## Deliverables

- **D1 — the ledger closure.** The exact source and synthesized-aggregate ledger
  rows, closed from the already-ruled graph-derived authorities.
- **D2 — the representation and lifetime controls**, applied from the same ruled
  authorities.
- **D3 — the selected population, written before any edit.** One row per member
  of the 17: test name, first refusal, and the corrected-census owner label that
  put it in scope. **Authored from seam 1's `D4`.**
- **D4 — exact planned/emitted equality**, gated, plus the existing negative
  discriminators retained and shown still discriminating.
- **D5 — the omission controls.** For each population `D1` or `D2` proves over,
  one control that reds when a member is dropped from it. **One per population,
  not one for the seam.**
- **D6 — the before/after row disposition** for the 17, plus a single aggregate
  line stating that no row outside the 17 changed status.

## Acceptance criteria

- **AC-1 — `D3` has exactly 17 rows**, each tracing to a corrected-census row
  with a D7 ledger, representation, or lifetime label.
  *Control:* the `D3` name set against the corrected census, both directions. A
  `D3` of 12 rows fails this AC — it means the uncorrected census was used.
- **AC-2 — the 17 selected rows pass on the candidate.**
  *Control:* the run output with pass/fail counts and `git rev-parse HEAD` in the
  same block. `--no-run` does not discharge this.
- **AC-3 — no row outside the 17 changes status**, in either direction.
  *Control:* the full pass/fail set on this seam's base against the candidate's,
  differenced; the symmetric difference must be exactly the 17.
- **AC-4 — every `D5` omission control reds under its own omission.** Drop one
  member from the population; the control must go red. Restore it; green.
  *Control:* each omission run shown red, then reverted. **A control that stays
  green under its own omission fails this AC and, with it, the seam's central
  claim.** Commit the real fix before any mutation proof, and reset after.
- **AC-5 — `D4`'s negative discriminators still discriminate.** Each existing
  negative control is shown red under the condition it exists to catch.
  *Control:* the mutation run per discriminator.
- **AC-6 — the prior-slice surfaces are blob-identical to the merge base.**
  *Control:* `git rev-parse <candidate>:<path>` against
  `git rev-parse <merge-base>:<path>` for each of the six surfaces.
- **AC-7 — CI green** on the merge. Workspace-green means green in CI, never a
  local `--workspace` run.

## Banned scope

- **No planner or ABI repair.** A planner- or ABI-worded refusal on the lawful
  assembly is a new interface fact and routes back as an exact hard stop under
  seam 4's rule.
- **No re-derivation of the D7 authorities.** They are ruled. This seam applies
  them; disagreeing with one is a hard stop, not an edit.
- **No lowering activation work.** That was seam 2 and it is merged.
- **No edit to any prior-slice surface** (`AC-6`).
- **No merge, rebase, or wholesale cherry-pick of any preserved object.**
- **No test asserting facts about source or documentation lines** (operator test
  policy). `D3` and `D6` are review artifacts, not gates.

## Contention

Runtime is single-threaded. Take the shared build lock for `AC-2`/`AC-3`; probe
without blocking first. **Targeted only:** `scripts/ken-cargo test -p ken-runtime
--lib`. **Never `--workspace`** — the full-workspace build, the `--locked` gate
and conformance run in CI.

## Sizing

**Size `M`.** Smaller than seam 2: the authorities are ruled and the population
is fixed, so the work is application rather than design. The one thing that can
inflate it is `D5` — an omission control per population, not per seam.

⇒ **Commit at these three checkpoints and post the exact SHA at each:**

1. `D3` written from the corrected census — no production edit yet.
2. `D1` plus `D2` closure.
3. `D4` equality gate, `D5` omission controls with their proofs, then `D6`.

If checkpoint 2 runs past an hour, stop and route; the recut is the Steward's.

## Hard stops

Stop and route to the Steward, do not improvise, if any of these hold:

1. **The corrected census does not yield exactly 17 D7-owned rows.** The
   partition is wrong and selection cannot proceed from an unreproducible count.
2. **A `D5` omission control cannot be written for some population** — meaning
   the population has no observable membership. That is the vacuity trap in its
   original form and it is a design question, not an implementation one.
3. **A ruled D7 authority appears wrong.** Route it; do not apply an authority
   you believe is incorrect, and do not silently correct it.
4. **A planner- or ABI-worded refusal appears.** New interface fact.
5. **A row outside the 17 changes status in either direction** (`AC-3`).
6. **Closing the ledger requires touching a prior-slice surface.** Interface
   fact, exactly as at seams 1 and 2.
