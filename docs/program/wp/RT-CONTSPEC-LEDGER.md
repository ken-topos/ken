# WP frame — `RT-CONTSPEC-LEDGER` (ContinuationSpecialization seam 3 of 4)

Node: `docs/program/issues/RT-CONTSPEC-LEDGER.md`. Campaign:
`docs/program/16-recursive-descent-retirement.md`. Owner: runtime ring.
Authority: Architect ownership/sizing ruling `evt_1yymw1gdszpbs`, outcome (c),
seam 3.

Seam 2 turns continuation emission on. This seam makes the **boundary-use
ledger** record something: today it carries four compile-time constants in every
production build, so it distinguishes no two continuation inputs from each
other.

> ## RECUT NOTICE — the 17-row census selection is RETIRED as this seam's subject
>
> **Steward, 2026-08-02.** The previous cut told you to select exactly 17
> D7-owned rows from the corrected census and make them pass. That instruction
> carried the same defect found and withdrawn on seam 2
> (`evt_2zhx69f2fw07w`, Architect confirmation `evt_66t42tapvdbsj`).
>
> The 138-row census is a **first-refusal record from the held `1aef3192`
> lineage** — a tree carrying a mechanism that was never merged and may not be.
> It can say what once failed and why. It **cannot name a current source
> authority**, and its rows are green on the lawful base.
>
> ⇒ **Selecting from the census is retired as a scoping mechanism for this
> campaign.** This seam's subject is now stated positively, from what the lawful
> base does and does not do, with a discriminator that can fail on the
> candidate. The census is not an input to any deliverable or AC below.

> ## The `46d29783` lineage remains an ORACLE
>
> `46d29783`, `1aef3192`, `9d58df12`, and
> `refs/preserved/rt-contspec-lower-held-core-rs = 88972207` are preserved and
> **may not be merged, rebased onto, or cherry-picked wholesale.** This seam
> branches from `main` after seam 2 lands and carries only its own delta.

## The subject, measured

All line numbers below are at `main = cef564f1`, in
`crates/ken-runtime/src/cranelift_backend/planning/static_transition.rs`.

`ContinuationInputProjection` (`:450`) carries 15 fields. Four of them are the
boundary-use ledger:

| field | enum | variants | production-reachable |
|---|---|---|---|
| `boundary_phase` | `BoundaryUsePhase` `:417` | `OperationalCarrier`, `SpecializedValue` | **`OperationalCarrier` only** |
| `boundary_operation` | `BoundaryUseOperation` `:424` | `Forward`, `Retain` | **`Forward` only** |
| `boundary_need` | `BoundaryUseNeed` `:431` | `PreserveValue`, `PreserveCallableIdentity` | **`PreserveValue` only** |
| `boundary_avail` | `BoundaryUseAvail` `:438` | `Value`, `Callable` | **`Value` only** |

In each of the four enums **the second variant is `#[cfg(test)]`**. It does not
exist in a production build.

`ContinuationInputProjection` has **exactly one construction site**:
`exact_continuation_projection` (`:2754`). It hardcodes all four fields to the
one non-test variant (`:2764-2767`):

```rust
boundary_phase:     BoundaryUsePhase::OperationalCarrier,
boundary_operation: BoundaryUseOperation::Forward,
boundary_need:      BoundaryUseNeed::PreserveValue,
boundary_avail:     BoundaryUseAvail::Value,
```

⇒ **Every continuation input the planner produces carries the same boundary-use
tuple.** The ledger has one row shape and records no distinction.

### What the existing control does and does not prove

There is already a discrimination control over these fields
(`continuation_keys_equal_under_mutation` `:2832`, driven from the omission
harness at `:12355-12405`). It proves, per field, that the interning key
separates two units differing only in that field, and that suppressing the field
conflates them. **That proof is real and it must keep passing.**

But `ContinuationProjectionOmission` (`:531`) is itself `#[cfg(test)]`, and
`mutate_projection_field` reaches the distinct value by flipping to the
`#[cfg(test)]` variant (`:12312-12321`). So the control proves the key
discriminates **over a value production cannot construct.**

⇒ ⭐ **The discrimination is proved and unreachable.** That is this seam's whole
subject: not a missing mechanism, but an instantiated key over an
uninstantiated vocabulary. Do not read the green control as evidence the ledger
works — read it as the harness this seam finally gives a production population
to.

## Fixed inputs, and the re-measurement you owe before any edit

Measured at `main = cef564f1`. **Seam 2 is permitted to edit
`static_transition.rs`, so these line numbers and possibly the construction site
may move under you.** The four-constants property is what matters, not the
addresses.

| input | measured value |
|---|---|
| seam 2 | `RT-CONTSPEC-ACTIVATE`, must be `merged` before this starts |
| the ruled authorities | the graph-derived D7 authorities already ruled; this seam cites and applies them, it does not reopen them |
| the ledger's production tuple | `OperationalCarrier / Forward / PreserveValue / Value`, at the single construction site |
| the discrimination harness | `continuation_keys_equal_under_mutation`, retained and kept green |
| prior-slice surfaces | `planning/static_transition/abi.rs`, `planning/static_transition/semantic_ir.rs`, `boundary_value.rs`, `boundary_value_clif.rs` — frozen at their `main` blobs |
| baseline suite | `scripts/ken-cargo test -p ken-runtime --lib` |

**Run this first and quote its output as `D1`. If it disagrees with the table
above, stop and route — do not adapt the frame yourself.**

```sh
git rev-parse HEAD
F=crates/ken-runtime/src/cranelift_backend/planning/static_transition.rs
grep -n "ContinuationInputProjection {" $F        # expect ONE construction site
grep -n -A3 "enum BoundaryUse" $F | grep -c "cfg(test)"   # expect 4
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

> ## THE TRAP THIS SEAM SITS ON — and it is already realized here
>
> **A proof over an incomplete population is vacuous, and every control over it
> passes.** That is what rejected `RT-JOIN-DISPOSITION`'s `27f9dca2`: one
> production site bypassed the recording call, so a class proved over an empty
> list and every assertion was green.
>
> ⇒ **This seam does not merely risk that shape — it inherits it.** The four
> boundary controls are green today over a population of one constant tuple.
> Adding a ledger claim above them without widening the population would leave
> every new assertion green for the same reason.
>
> **So every population this seam proves over owes a control that REDS when a
> member is dropped from it**, and the boundary-use population owes, in
> addition, a control that reds when the vocabulary collapses back to one value.

## Deliverables

- **D1 — the re-measurement**, per the block above: the construction-site count,
  the four `cfg(test)` gates, and `git rev-parse HEAD`, quoted. Written before
  any edit.
- **D2 — the production-reachable vocabulary.** Remove the `#[cfg(test)]` gate
  from the four second variants and have the planner **issue** them from real
  planner facts at the construction site. ⛔ **Which planner condition selects
  which variant comes from the ruled D7 authorities — you apply them, you do not
  invent the mapping.** If they do not settle it, that is hard stop 2.
- **D3 — the ledger closure** over the now-distinguishable inputs: the exact
  source and synthesized-aggregate ledger rows, closed from those same ruled
  authorities.
- **D4 — the representation and lifetime controls**, applied from the same ruled
  authorities.
- **D5 — the boundary-use census on the candidate.** Over a fixed fixture, the
  distinct boundary-use tuples the planner actually issues, with a count. This
  is the discriminator: **1 on the base, greater than 1 on the candidate.**
- **D6 — the omission controls.** For each population `D3` or `D4` proves over,
  one control that reds when a member is dropped from it. **One per population,
  not one for the seam.**
- **D7 — the collapse control.** One control that reds if the four fields revert
  to a single tuple across the fixture — the failure this seam exists to fix,
  stated so a later change cannot silently undo it.

## Acceptance criteria

- **AC-1 — `D5` shows strictly more than one distinct boundary-use tuple on the
  candidate, and exactly one on the base.**
  *Control:* the census run on both trees, with `git rev-parse HEAD` in each
  block. ⛔ **A candidate census of 1 fails the seam** — it means the vocabulary
  is still unreachable and every ledger claim above it is vacuous.
- **AC-2 — no `#[cfg(test)]` gate remains on any of the four second variants**,
  and each is constructed on a non-test path.
  *Control:* the four declarations, plus the construction site showing each
  variant issued from a planner fact rather than a literal.
- **AC-3 — the existing discrimination harness still passes**, unmodified in
  what it asserts.
  *Control:* `continuation_keys_equal_under_mutation` green, and its assertions
  byte-unchanged. Widening its reachability must not weaken its claim.
- **AC-4 — every `D6` omission control reds under its own omission.** Drop one
  member from the population; the control must go red. Restore it; green.
  *Control:* each omission run shown red, then reverted. **A control that stays
  green under its own omission fails this AC and, with it, the seam's central
  claim.** Commit the real fix before any mutation proof, and reset after.
- **AC-5 — the `D7` collapse control reds when the four fields are forced back
  to the base tuple.**
  *Control:* the forced-collapse mutation shown red, then reverted. This is the
  positive control for `AC-1`.
- **AC-6 — no test asserts a fact about source, catalog, or documentation
  lines.** `D1` and `D5` are review artifacts, not gates (operator test policy).
  *Control:* a read of the added tests.
- **AC-7 — the prior-slice surfaces are blob-identical to the merge base.**
  *Control:* `git rev-parse <candidate>:<path>` against
  `git rev-parse <merge-base>:<path>` for each of the four surfaces.
- **AC-8 — CI green** on the merge. Workspace-green means green in CI, never a
  local `--workspace` run.

## Banned scope

- **No planner or ABI repair.** A planner- or ABI-worded refusal on the lawful
  assembly is a new interface fact and routes back as an exact hard stop.
- **No re-derivation of the D7 authorities.** They are ruled. This seam applies
  them; disagreeing with one is a hard stop, not an edit.
- **No lowering activation work.** That was seam 2 and it is merged.
- **No edit to any prior-slice surface** (`AC-7`).
- **No census selection.** The 138-row census is not an input here.
- **No merge, rebase, or wholesale cherry-pick of any preserved object.**

## Contention

Runtime is single-threaded. Take the shared build lock for the suite runs; probe
without blocking first. **Targeted only:** `scripts/ken-cargo test -p ken-runtime
--lib`. **Never `--workspace`** — the full-workspace build, the `--locked` gate
and conformance run in CI.

## Sizing

**Size `M`.** The authorities are ruled, so `D3`/`D4` are application rather
than design. The two things that can inflate it are `D2` — if the ruled
authorities turn out not to settle the variant mapping — and `D6`, which is one
omission control per population, not per seam.

⇒ **Commit at these three checkpoints and post the exact SHA at each:**

1. `D1` re-measurement plus `D2` production-reachable vocabulary, with the `D5`
   census showing the count move off 1. No ledger claims yet.
2. `D3` plus `D4` closure.
3. `D6` omission controls and `D7` collapse control with their mutation proofs.

**Expect to end your turn at each checkpoint.** Post the SHA and wait for the
leader rather than assuming one turn spans all three. If any checkpoint runs
past an hour, stop and route; the recut is the Steward's.

## Hard stops

Stop and route to the Steward, do not improvise, if any of these hold:

1. **The `D1` re-measurement disagrees with the fixed-inputs table** — the
   construction site count is not 1, or the four `cfg(test)` gates are not
   there. Seam 2 moved something this frame depends on.
2. **The ruled D7 authorities do not determine which planner condition selects
   which boundary-use variant.** That is a design question and it is not yours.
   ⛔ Do not pick a plausible mapping to keep moving.
3. **The vocabulary cannot be made production-reachable** without touching a
   prior-slice surface or the planner. Interface fact, exactly as at seams 1
   and 2.
4. **A `D6` omission control cannot be written for some population** — meaning
   the population has no observable membership. That is the vacuity trap in its
   original form and it is a design question.
5. **A ruled D7 authority appears wrong.** Route it; do not apply an authority
   you believe is incorrect, and do not silently correct it.
6. **A planner- or ABI-worded refusal appears.** New interface fact.
