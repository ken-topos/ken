# WP frame — `RT-CONTSPEC-PLANNER` (slice 1 of 3)

**Owner:** team Runtime · **Size:** M · **Node:**
`docs/program/issues/RT-CONTSPEC-PLANNER.md`

> ## ⛔⛔ READ FIRST — WHAT THIS SLICE IS FOR
>
> This is **not** a new design. The causal `ContinuationSpecialization`
> mechanism was ruled at `evt_7dhwrk26ks9m0` and **that ruling is binding and
> unchanged.** This frame exists because the Architect's second WIP audit
> (`evt_4t09329vdrf`) found the *delivery* mis-sized — outcome **(c)** — after
> two runs on one contract produced a cumulative **10 files, +10,193/−2,047**
> with **no candidate and no proved checkpoint at any point**.
>
> ⭐ **The property that makes this slice worth cutting is that it can be wrong
> in only one way.** A planner defect here cannot hide behind a lowering defect,
> because nothing is lowered. That is the entire design of the split.

---

## Fixed inputs

| input | value |
|---|---|
| base | ⏳ **PENDING** — routed to the Architect at `evt_1bh3p4wx76wtv`. ⛔ **This frame is not releasable until it is filled in.** |
| proved semantic base | `93746adaaef845f6c857b6c007aeac336c6c800c` |
| prototype reference | `origin/preserved/rt-recursor-freeze-465fab90` = `465fab90767a808edac79e665a1055b81206720b`, tree `aa7571a0828e067852075ce93418c1656e10ad96`, 173 files, +4267/−7885 |
| binding ruling | `evt_7dhwrk26ks9m0` (mechanism) · `evt_4t09329vdrf` (sizing) |
| campaign | `docs/program/16-recursive-descent-retirement.md` |

> ### ⭐ THE PROTOTYPE IS AN INPUT, NOT A CHECKPOINT
>
> The Architect confirmed the frozen prototype's **direction** is right —
> *"do not throw this design work away"* — and named four things in it as
> retainable: the exact projection schema, the full-key `intern_specialization`,
> the exact call-token schema and its plumbing, and the explicit
> `ContinuationSpecialization` unit arm.
>
> ⛔ **"Retainable" is not "accepted."** That state has **no** acceptance
> evidence: no candidate, no QA route, no green checkpoint, and its `--lib`
> green check was taken against a tree that also carried semantic probes. ⇒
> **Read it, re-derive what this slice needs, and land it with this frame's own
> controls.** ⛔ A wholesale port that cites the Architect's approval of the
> direction as evidence for the content is exactly the move this frame forbids.

---

## Deliverables

**D1 — exact ordered projection.** `ContinuationInputProjection` carries, per
input: producer / consumer / source owner; source ABI position and provenance
class; ordinal; carrier, ownership and storage owner; boundary phase, operation,
`Need` and `Avail`; closed referent affinity; and ordinary ABI position.

**D2 — the key is the ordered vector.** `ContinuationSpecializationKey` carries
the **ordered projection vector**, ⛔ not a count and ⛔ not a summary.

**D3 — interning before discovery, with an immutable key.**
`intern_specialization` performs **full-key equality** and inserts the immutable
full key **before** any further discovery.
⛔ **Banned outright, by name:** `same_static_unit_identity`, cross-owner
`reuse_specialization`, and any `max` / `|=` mutation of an already-assigned
specialization. These are the defects the first run shipped; they are not
refactor targets, they must not exist.

**D4 — exact causal edge token identity.**
`ContinuationSpecializationCallToken` binds the producer owner, the exact
result / `Construct` alternative / sequence, the exact target, and the exact
residual/worker provenance.

**D5 — finite recursion and closure.** The fixed point terminates, and both
closure properties hold and are asserted: **keys ↔ units** and **planned-edge**
closure.

---

## Acceptance criteria

⭐ **Every AC below is discharged at the PLANNER level.** ⛔ No AC requires a
lowered call, a native binary, or a runtime exit code — if one appears to, it is
misread.

**AC-1 — D1–D5 are implemented and the banned identities of D3 do not exist.**

**AC-2 — the omission mutation.** Drop **one field** from the projection.
⇒ A test must go red. ⛔ It must be red because two specializations that differ
only in that field now collide — ⛔ **not** merely because a struct literal fails
to compile. Run it **per field**; a mutation that only breaks compilation is
**not** a control and does not count.

**AC-3 — the collision mutation.** Make `intern_specialization` compare on a
**prefix** of the key instead of the full key. ⇒ Red.

**AC-4 — the value-not-key mutation.** Re-introduce mutation of an assigned
specialization (the `max` / `|=` shape). ⇒ Red.

**AC-5 — termination.** The fixed point terminates on the nested
inner→outer chain, and the test that proves it would **not** terminate under a
deliberately weakened decreasing measure.
⚠ ⛔ **Do not rest the termination argument on anything that shrinks only at
compile time** — a runtime value that grows destroys exactly that argument, and
this campaign has already been bitten by that shape once.

**AC-6 — dormancy, stated as a behavioural property.** With this slice landed,
**observable program behaviour is unchanged**: the existing native and interp
suites produce the same results as at the base. ⛔ The planner may compute the
new structure; nothing may consume it.

**AC-7 — no regression, in CI.** Green on GitHub CI.
⛔ **Not** a local `--workspace` run — local work is `scripts/ken-cargo` scoped
to the crate touched, per `COORDINATION §12`.

> ### ⛔ ON THE CONTROLS — the failure this frame is built to prevent
>
> ⭐ **A mutation that reddens a test proves the test sees the mutation. It does
> not prove the test sees the PROPERTY.** For each of AC-2/3/4, name **which**
> test went red and **what wrong answer** it produced — a wrong specialization
> selected, two units conflated, an edge planned to the wrong target. ⛔ "The
> suite went red" is not a discharge.
>
> ⚠ And a control whose mutation only breaks **compilation** is measuring the
> type system, not your mechanism.

---

## ⛔ Banned scope

- ⛔ **No lowering activation.** Nothing in `lowering/` may consume the planner's
  output on a live path.
- ⛔ **No real-witness claim.** The 761 witness, the three-way differential, and
  the 19-row population all belong to slice 3. ⛔ Do not cite a witness result as
  evidence for anything in this slice.
- ⛔ **No dynamic branch activation**, and ⛔ no callable/control identity in
  runtime data — that is slice 2's boundary and it stays closed here.
- ⛔ **No semantic probes and no ungated prints.** The literal 210/211 returns at
  `lowering/core.rs:6540+` / `:8920+`, the `CONTSPEC_*` markers, and the bare
  `eprintln!` at `planning/static_transition.rs:8549` must not reappear.
  ⚠ **A probe that changes what the program returns is not a diagnostic** — the
  campaign has already had one reported as a semantic finding.
- ⛔ **No formatter sweep.** The 177-file churn is being neutralized; do not
  reintroduce it. Touch the files this slice needs and no others.

---

## Contention

**None.** Team Runtime holds this alone. The doc track runs concurrently by
standing exception and touches `library/` and `agent/`, never `crates/`.
⚠ `RT-BACKEND-MODULE-SPLIT` (campaign node #8) contends on exactly these files
and is deliberately sequenced **after** `RT-DESCENT-RETIRE` — ⛔ do not begin any
module split inside this slice.

---

## Hard stops

⭐ **Route a hard stop; do not push through one.** This node exists *because* a
seat pushed through. If the planner closure cannot be expressed without
activating something this frame bans, **that is a hard stop and the frame is
what is wrong** — say so and it will be re-cut. ⛔ Do not widen the slice to make
its own AC reachable.

⏱ **Target: this slice completes, or hard-stops, inside one turn.** That is a
sizing target the Steward is accountable for, ⛔ **not** an acceptance criterion
and ⛔ not something QA checks. If it is missed, the sizing was wrong.
