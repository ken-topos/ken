# WP frame — `RT-CONTSPEC-LOWER` (slice 3 of 4 — **the capstone**)

**Node:** `docs/program/issues/RT-CONTSPEC-LOWER.md` · **Owner:** runtime ·
**Size:** L (⚠ see *Sizing* — this is a pre-emptive split candidate)

> ## ✅ READ FIRST — THIS FRAME IS COMPLETE AND THIS NODE IS SHOVEL-READY
>
> **All four slots were filled on 2026-08-01** from the merged slice 2
> (PR #1303, `origin/main = 3cc4fa19`), measured in the landed code rather than
> read off slice 2's frame.
>
> ⛔⛔ **SLOT B CAME BACK NEGATIVE AND IT IS THE MOST IMPORTANT LINE IN THIS
> FRAME: neither of slice 2's gates reaches a lowered call.** `D3` validates the
> descriptor *table*; `D4` is scoped to the *install* path in its own words.
> ⇒ ⭐ **This slice inherits NOTHING at the call site and must build its own
> gates there.** Read *Fixed inputs* before anything else.

---

## Fixed inputs

**Base — a RULE, not a number.** Branch from `origin/main` **containing
[[RT-CONTSPEC-ABI]]** — that is, containing `abi.rs` blob
`23b9f5d778bf98fbb2907cf087bf06da30d82e7d`. ⛔ Not from slice 2's branch, ⛔ not
from a preservation ref, ⛔ not from any SHA quoted in this file: `main` moves,
and `3cc4fa19` below is a *measurement timestamp*, not your base.
⚠ **This exact confusion cost a kickoff in slice 2** — re-derive the base at
pickup and state the SHA you derived.

### ✅ SLOT A — the ABI surface slice 2 landed

**Measured by the Steward on `origin/main = 3cc4fa19` (PR #1303), not read off
slice 2's frame.** All of it lives in
`crates/ken-runtime/src/cranelift_backend/planning/static_transition/abi.rs`.

| what | exact form |
|---|---|
| the `D1` arm | `AbiUnitDefinition::ContinuationSpecialization { specialization: ContinuationSpecializationId }` |
| the descriptor | `AbiContinuationDescriptor { definition, header: AbiFrameHeader, slots: DenseRange, inputs: DenseRange }` |
| per-input authority | `AbiContinuationInputAuthority { ordinal: u32, source_owner: PredeclaredFunctionId, referent_affinity: DenseRange }` |
| storage on `AbiPlane` | `continuation_descriptors: Vec<AbiContinuationDescriptor>` · `continuation_slots: Vec<AbiSlot>` · `continuation_inputs: Vec<AbiContinuationInputAuthority>` |
| install | `install_continuation_specialization_abi(&mut AbiPlane, &[PlannedContinuationSpecialization]) -> Result<(), CraneliftBackendError>` |
| the `D3` gate | `AbiPlane::validate_continuation_specializations(&self, &[PlannedContinuationSpecialization]) -> Result<(), CraneliftBackendError>` |

⭐ **`ContinuationSpecializationId(u32)` is the index into
`continuation_descriptors`**, and `install` refuses when the constructed id and
the positional index disagree. **Slices and ranges, not owned collections** —
`DenseRange` into the three shared vectors is the addressing model, and this
slice must keep it.

⚠⚠ **Every one of these is `pub(super)` — visible only inside
`planning::static_transition`.** This slice's work is in that module already
(`static_transition.rs` calls both `install_…` and `validate_…` today), so it
composes. ⛔ **But if a deliverable turns out to need any of them from outside
that module, that is a HARD STOP to route — ⛔ not a visibility widening.**

### ⛔⛔ SLOT B — ANSWERED, AND THE ANSWER IS "NEITHER"

**Both of slice 2's gates stop at the descriptor table. ⛔ Neither reaches a
lowered call, so this slice INHERITS NOTHING at the call site and must build its
own.**

- **`D3` — `validate_continuation_specializations`** checks the descriptor
  population against the planner population: count equality, `id == index`, and
  the arm's identity. ⇒ It validates **the table**, ⛔ not any use of it.
- **`D4` — the zero-allocation gate** is scoped to the **install path** in its
  own words (`abi.rs:597–603`): the four backing vectors are reserved to their
  exact closed populations before the first descriptor is constructed, so
  *capacity growth while appending* is the observable it refuses; the validator
  is separately noted as allocating nothing on its success path. ⇒ **It says
  nothing about allocation on a lowered call.**

⚠ **`D4`'s positive control is `SKIP_CONTINUATION_ABI_PREFLIGHT`, a
`#[cfg(test)]` thread-local** (`abi.rs:589–595`) that skips the preflight so the
first descriptor grows storage. ⭐ Sound as a control for what it covers; ⛔ **it
is not evidence about the lowered path and must not be cited as such.**

⇒ ⭐⭐ **This is the concrete hole `AC-1`…`AC-4` exist to close.** An owner,
lifetime, affinity or allocation property that holds for a descriptor and not
for the call consuming it is exactly what this slice would otherwise ship.

### ✅ SLOT C — the residual after slice 2's review is EMPTY

**Slice 2 was approved on the FIRST round** — QA `evt_74mmepexexppy`, Architect
`evt_5396qdh7a7p32`, Decision `dec_77b33m1pbahng` resolved, no rejects and
nothing deferred into this slice.

⭐ **Against slice 1's four rounds, that is the staging working**, and it is the
reason to keep this slice's controls to the same standard rather than relaxing
them: the cheapness of slice 2's review was bought by slices 0–1 having already
fixed what it could assume.

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

✅ **SLOT D — measured at `3cc4fa19`: no live contention.**
`RT-BACKEND-MODULE-SPLIT` touches exactly these two files and is sequenced
**after** this capstone; nothing else is in flight against
`crates/ken-runtime/src/cranelift_backend/planning/`.

⚠ **Re-derive at pickup anyway** — this is a measurement, and the doc track
runs concurrently.

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
