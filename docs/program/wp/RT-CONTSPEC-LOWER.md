# WP frame — `RT-CONTSPEC-LOWER` (slice 3 of 4 — **the capstone**)

**Node:** `docs/program/issues/RT-CONTSPEC-LOWER.md` · **Owner:** runtime ·
**Size:** L (⚠ see *Sizing* — this is a pre-emptive split candidate)

> ## ⛔⛔ READ FIRST — THIS FRAME IS **HALF-WRITTEN ON PURPOSE**
>
> ⏳ The node stays `status: draft` until the slots in *Fixed inputs* are
> filled. ⛔ **Do not pick this up yet.**
>
> **Why it exists now anyway:** operator standing policy (§2a-bis) requires a
> framed successor while a node is with a team, and [[RT-CONTSPEC-ABI]] is with
> the team now. ⭐ **Everything slice 2's outcome cannot change is written
> below and is final.** The parts slice 2 genuinely fixes are marked
> **`▢ SLOT`** and are the only work owed at its merge — minutes, not an hour.
>
> ⚠ **The original instruction was "frame this last, or it will be sized
> against a planner and ABI that do not exist yet."** That reasoning is sound
> for the *interface* facts and is preserved as the slots. ⛔ It was **not**
> grounded for the risk posture, the banned scope, the witness gate, or the
> tracker discipline — none of which slice 2 can move. Those are written.

---

## Fixed inputs

**Base — a RULE, not a number.** Branch from `origin/main` **after
[[RT-CONTSPEC-ABI]] has landed on it**. ⛔ Not from slice 2's branch, ⛔ not
from a preservation ref, ⛔ not from any SHA quoted in this file.
⚠ **This exact confusion cost a kickoff in slice 2** — re-derive the base at
pickup and state the SHA you derived.

**▢ SLOT A — the ABI surface slice 2 landed.** At slice 2's merge the Steward
measures and writes in: the `AbiUnitDefinition` arm's exact name and
constructor, the descriptor accessor this slice consumes, and the slot order
`D2` fixed. ⛔ Do not guess these from slice 2's frame — its frame states
intent, and the merged code is the authority.

**▢ SLOT B — which of slice 2's gates bind at the call site.** `D3` (owner /
lifetime / affinity) and `D4` (zero-allocation) are descriptor-level in slice 2.
⚠ **Whether each extends to the lowered path is a real question, not a
formality** — an owner gate that holds for a descriptor and not for the call
that consumes it is a hole this slice would ship.

**▢ SLOT C — the residual after slice 2's review.** Slice 1 needed four review
rounds and the accepted surface moved every round. Whatever slice 2's review
ratifies or defers lands here.

---

## ⚠ This slice carries the campaign's real risk, and that is deliberate

**Everything that needs a running native binary to prove is here, and nothing
that does not need one is.** That is the split the Architect's audit finding
demands: the first 30-hour run and the corrected run both hid their defects
inside a breadth where **a planner error and a lowering error were
indistinguishable from the outside**. Slices 0–2 exist to make that distinction
free by the time this slice runs.

⭐ **This is the only slice that ACTIVATES.** Slices 0, 1 and 2 are dormant by
construction; if anything in them was wrong, it becomes observable here.

---

## Scope, as ruled

1. **Attach the exact call token at each producer alternative.**
2. **Emit the direct call/return BEFORE the identity-erasing join.** ⭐ The
   ordering is the mechanism — after the join the identity this whole campaign
   computes is already gone.
3. **Remove the dynamic case's active-scalar/search route.**
4. **Close nested recursion.**
5. **Close the ledgers.**
6. **Run the witness:** E-1 / E-5 / E-7, the full **19-row** population, and
   literal all-check CI.

---

## Acceptance criteria

**AC-1 — the direct call is emitted before the join**, with a control that
distinguishes *before* from *after*. ⚠ A test that merely shows a direct call
happened does **not** discriminate the two orderings — and the ordering is the
entire ruled mechanism.

**AC-2 — the dynamic case's active-scalar/search route is gone**, shown by a
case that previously took it.

**AC-3 — nested recursion closes**, with a case whose nesting depth is
load-bearing. ⛔ A depth-1 fixture does not measure nesting.

**AC-4 — the full 19-row population passes**, each row named. ⛔ An aggregate
pass is not evidence for the rows: a differential over an aggregate passes
while one of N contributors defects.

**AC-5 — the 761 witness gate.**
`fs_read_at_malformed_offset_narrows_to_invalid_offset` must produce
`InvalidOffset`, and its sibling at `crates/ken-cli/tests/rt_parity_native.rs:544`
is covered by the same question:
> ⛔⛔ **Did the trap become `InvalidOffset` because the defect was FIXED, or
> because the assertion MOVED?** State which, and cite the production error site
> that raises it. ⚠ Classify by **the error production raised**, never by the
> test name.

**AC-6 — activation is observable.** With this slice landed, behaviour that was
unchanged through slices 0–2 now changes in exactly the ruled way, and the frame
names which observable moves.

**AC-7 — deliverables are production types.** ⛔ No deliverable may be
`#[cfg(test)]`. ⚠ `AbiDescriptorShape` and `AbiPlane::shape` are existing
test-only probe infrastructure — you may **use** them in a control, ⛔ never
land a deliverable behind them.

**AC-8 — no regression, green in CI on GitHub.** ⛔ Not a local `--workspace`
run; local work is `scripts/ken-cargo` scoped to the crate touched
(`COORDINATION §12`).

> ### ⭐⭐ THE REVIEW BUDGET GOES ON WHETHER EACH CONTROL CAN DISCRIMINATE
>
> **Measured in slice 1: two of its three rejects turned on a DEGENERATE
> CONTROL, not on wrong production code.** One fixture had a single worker and a
> single capture, so both wrong answers equalled `1` — green against green.
> Another had a `unit()` case body, so descriptor-count truncation was invisible
> to it.
>
> ⭐ **The model is slice 1's final fixture:** `Var(4)` makes ordinal 2
> load-bearing, so exact production gives `[1,0,1]`, truncation gives `[1,0]`,
> and descriptor restatement gives `[0,1]` — **three distinguishable wrong
> answers, each named in advance.**
>
> ⇒ For every AC above, name the wrong answers it separates. ⛔ An AC that
> cannot name one is not yet a control.

---

## ⛔ Banned scope

- ⛔ **No planner or ABI re-work.** If slice 1's closure or slice 2's descriptor
  is wrong, that is a **hard stop routed back**, ⛔ not a repair inside this
  slice. ⭐ The whole point of the staging is that this slice inherits them
  proved.
- ⛔ **No post-join / static-worker mechanism** — that arm is rejected.
- ⛔ **No new carrier, ownership, or storage-owner variant** invented to make a
  slot fit.
- ⛔ **No semantic probes, no ungated prints, no formatter sweep.** The literal
  210/211 returns, the `CONTSPEC_*` markers, the bare `eprintln!` at
  `planning/static_transition.rs:8549`, and the 177-file churn must not appear.
  ⚠ **A probe that changes what the program returns is not a diagnostic** — one
  has already been reported as a semantic finding in this campaign.
- ⛔ **No module split.** `RT-BACKEND-MODULE-SPLIT` contends on exactly these
  files and is sequenced **after** this capstone.
- ⛔ **No CI gate asserting facts about source lines** (operator test policy —
  tests measure behaviour).

---

## ⭐⭐ Tracker discipline — this merge closes THREE nodes in ONE commit

When this merges it closes **itself**, [[RT-RECURSOR-TRANSPORT]] and
[[RT-DECL-CLOSURE-PORT]], in ⛔ **one** tracker commit.

⛔ **Do not flip them separately**, and ⛔ **do not describe this slice's
recursor code as a `D7` deliverable** — `D7` is the atomic mechanism these four
slices implement, not a thing any one slice delivers.

---

## ⚠ Sizing — this is a pre-emptive split candidate, and the decision is owed

**Size `L`, and that is a flag, not a plan.** Slice 1 was `M` and needed four
review rounds. This slice carries lowering, nested recursion, the ledgers, and
the entire native witness — and the operator's WP-integrity rule targets an
implementer turn completing **under an hour**.

⭐ **The candidate seam, if it splits:**

| part | contents | provable without? |
|---|---|---|
| **3a — activate** | token attach · direct call before the join · remove the active-scalar/search route · nested recursion | needs its own witness subset |
| **3b — close** | the ledgers · full 19-row population · E-1/E-5/E-7 · literal all-check CI | inherits 3a activated |

⚠ **The seam is not obviously clean** — activation without its witness is
unproved, so 3a cannot simply defer all of the native run to 3b.

⇒ **The split decision is made AT slice 2's merge, with the Architect, informed
by slice 2's actual surface.** ⛔ It is not made now, and it is not made by the
implementer mid-turn. ⭐ If the first turn on this node runs past 60 minutes
without a hard stop, that is the WIP-audit trigger and **outcome (c) —
restructure into smaller WPs — is explicitly on the table.**

---

## Contention

`RT-BACKEND-MODULE-SPLIT` touches exactly these files and is sequenced after
this capstone. ⚠ **▢ SLOT D — re-derive contention at pickup**: any node that
lands between now and slice 2's merge may touch the lowering path.

---

## Hard stops

1. **Slice 1's planner closure or slice 2's descriptor is wrong.** ⇒ Route back;
   ⛔ do not repair here.
2. **A slot requires a new carrier, ownership, or storage-owner variant.** ⇒
   Route; ⛔ that is a widening, not a fit.
3. **The 761 witness turns out to pass because the assertion moved.** ⇒ ⛔ Stop
   and route — that is a false green on the campaign's headline gate, not a
   detail.
4. **The 19-row population cannot be run without a full-workspace local build.**
   ⇒ Route to the Steward for a CI path; ⛔ never run `--workspace` locally
   (`COORDINATION §12`).
