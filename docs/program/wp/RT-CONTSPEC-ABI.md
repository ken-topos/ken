# WP frame — `RT-CONTSPEC-ABI` (slice 2 of 4)

**Owner:** team Runtime · **Size:** M · **Node:**
`docs/program/issues/RT-CONTSPEC-ABI.md`

> ## ⛔⛔ READ FIRST
>
> The `ContinuationSpecialization` mechanism is **already ruled**
> (`evt_7dhwrk26ks9m0`) and ⛔ **nothing semantic is open here.** This slice
> exists because the Architect's second WIP audit (`evt_4t09329vdrf`) returned
> outcome **(c) — the delivery was mis-sized**, and cut the work into four
> staged slices.
>
> ⭐ **This slice gives the specialization a REPRESENTATION. It does not give it
> a CALLER.** Slice 3 activates lowering; here the descriptor must be
> constructible, checkable, and **dead**.

## Fixed inputs

| input | value |
|---|---|
| **base** | ⭐ **`origin/main` as of when you cut the branch — whatever SHA that is, provided it contains this frame AND [[RT-CONTSPEC-PLANNER]] (slice 1) has landed.** ⛔ Not a SHA copied from this table, ⛔ not `93746ada`, ⛔ not a preservation ref, ⛔ not another slice's branch. |
| the planner closure | whatever slice 1 landed. ⭐ **Read it at your base; it is authority over this frame** — this frame was written while slice 0 was still in flight. |
| semantic oracle (read-only) | `93746adaaef845f6c857b6c007aeac336c6c800c` — ⛔ **an oracle, not a base** |
| prototype reference | `origin/preserved/rt-recursor-freeze-465fab90` (`465fab90`) — ⛔ **no acceptance transfer** |
| binding rulings | `evt_7dhwrk26ks9m0` (mechanism) · `evt_4t09329vdrf` (sizing) · `evt_6wkw2c7ykjxsy` (the slice chain) |
| campaign | `docs/program/16-recursive-descent-retirement.md` — ⭐ **its five traps bind this node; read them before this frame.** |

> ### ⭐ THE BASE IS A RULE, NOT A NUMBER — and this cost a kickoff already
>
> Slice 0's frame named a base SHA. The frame's **own doc-only publish** moved
> `main` past that number, `runtime-leader` correctly refused to kick on the
> conflict (`evt_2gmw71nth2bg4`), and it had to be reissued as a derivation
> (`evt_4nyyczv0k0evt`). ⛔ **Do not reintroduce a fixed base SHA here.**

---

## What is on `main` today — measured by the Steward at `4d19c5dd`

⭐ **`ContinuationSpecialization` has ZERO occurrences anywhere in `crates/`.**
This slice introduces it as a representation for the first time.

The types it extends all live in
`crates/ken-runtime/src/cranelift_backend/planning/static_transition/abi.rs`:

| item | line | what it is today |
|---|---|---|
| `AbiUnitDefinition` | `:288` | ⭐ **exactly two arms** — `SchedulingEntry` and `ClosureBody` |
| `AbiDescriptor` | `:400` | one unit's complete representation contract; carries `definition` |
| `AbiPlane` | `:414` | one descriptor per `PredeclaredFunction`, plus their slots |
| `AbiSlot` | `:340` | `kind` / `carrier` / `ownership` / `storage_owner` / width / align / `ordinal` |
| `AbiOwnership` | `:220` | the stated **lifetime / aliasing / transfer / reclamation** modes |
| `AbiStorageOwner` | `:204` | storage ownership |
| `AbiCaptureProvenance` | `:256` | capture provenance |
| `CONVENTION_SLOTS` | `:427` | ⭐ the **no-implicit-caller-environment-tail** invariant: a frame's slot count is exactly `parameters + captures + CONVENTION_SLOTS` |

⚠ **Re-derive every line number at your base.** They were measured at
`4d19c5dd` and this file is churn-prone.

---

## Deliverables

**D1 — the explicit unit/descriptor projection.** A third `AbiUnitDefinition`
arm for the continuation specialization, carrying its own defining identity
rather than being encoded as a `ClosureBody` with a marker.
⭐ **The arm is the point:** the ruled mechanism is an explicit unit, ⛔ not a
flag on an existing one.

**D2 — the exact ordinary ABI.** The specialization's slots, in the existing
kind order (parameters, captures, result, control, trap, store), each with its
carrier, ownership, and storage owner drawn from the existing vocabularies.
⛔ **No new carrier or ownership mode may be invented to make a slot fit** — if
one is genuinely required, that is a hard stop (see below), not a widening.

**D3 — the owner / lifetime / affinity gates.** For each slot the descriptor
projects, the owner, lifetime and closed referent affinity are checked against
the planner closure slice 1 landed, and a disagreement is **refused**.

**D4 — the zero-allocation negative gate.** A check that constructing and
validating a specialization descriptor introduces **no allocation on the
boundary path**.
⚠ **No such gate exists on `main` today** — the Steward measured zero hits for
`zero_alloc` / `no_alloc` across `crates/ken-runtime`. ⇒ **You are creating it,
not extending one.** ⭐ State the property **behaviourally** (what an allocating
choice would do that a non-allocating one does not), ⛔ not as an assertion
about source lines — operator test policy bans source-line oracles.

⭐ **All four land DORMANT.** They may be constructed, projected, and checked;
⛔ nothing on a live path may consume them.

---

## Acceptance criteria

⭐ **Every AC below is discharged at the DESCRIPTOR level.** ⛔ No AC requires a
lowered call, a branch route, a join, a native binary, or a runtime exit code.
If one appears to, it is misread — that is slice 3.

**AC-1 — D1–D4 implemented**, each independently gated with its own assertion,
⛔ not merely as an input to something else.

**AC-2 — the existing frame invariant still holds for the new arm.** A
specialization unit's slot count is exactly
`parameters + captures + CONVENTION_SLOTS` (`abi.rs:427`), with **no implicit
caller-environment tail**. **Control:** a case that adds a capture and shows the
count moves by exactly one.

**AC-3 — the disagreement mutation.** Make the descriptor project an owner,
lifetime, or affinity that **contradicts** the planner closure. ⇒ A test must go
red, and it must go red **because the gate refused a wrong answer** — name which
test, and what wrong pairing it accepted before the gate existed.
⛔ **A mutation that only breaks compilation is not a control**; it measures the
type system, not this mechanism.

**AC-4 — the zero-allocation gate has a positive control.** Show it goes red
when an allocating choice is made.
⚠ ⭐ **A negative check passes for any reason, including never running.** An
"asserts no allocation" gate that has never been observed failing is not
evidence — the campaign has been bitten by exactly this shape.

**AC-5 — dormancy, behaviourally.** With this slice landed, observable program
behaviour is unchanged: the existing native and interp suites give the same
results as at the base. ⭐ The descriptor may exist and be checked; nothing may
consume it.

**AC-6 — deliverables are production types.** ⛔ **No deliverable may be
`#[cfg(test)]`.** ⚠ `AbiDescriptorShape` (`abi.rs:390`) and `AbiPlane::shape`
(`abi.rs:437`) are **existing test-only probe infrastructure**, and a prior WP's
`AC-6` already requires executable probes to stay test-only. ⇒ You may **use**
them in a control; ⛔ you may not land a deliverable behind them, or the
mechanism is unbuildable in production.

**AC-7 — no regression, green in CI on GitHub.** ⛔ Not a local `--workspace`
run; local work is `scripts/ken-cargo` scoped to the crate touched
(`COORDINATION §12`).

---

## ⛔ Banned scope

- ⛔ **No dynamic branch activation.** The branch route, the join, and the
  witness are slice 3 and must be able to fail on their own.
- ⛔ **Zero callable/control identity in runtime data.** ⭐ **This is the
  boundary that keeps the slice dormant** — a descriptor that carries a callable
  identity is already half-activated.
- ⛔ **No planner re-work.** If slice 1's closure is wrong, that is a hard stop
  routed back, ⛔ not a repair inside this slice.
- ⛔ **No post-join / static-worker mechanism** — that arm is rejected.
- ⛔ **No new carrier, ownership, or storage-owner variant** invented to fit a
  slot. See hard stops.
- ⛔ **No semantic probes, no ungated prints, no formatter sweep.** The literal
  210/211 returns, the `CONTSPEC_*` markers, the bare `eprintln!` at
  `planning/static_transition.rs:8549`, and the 177-file churn must not appear.
  ⚠ **A probe that changes what the program returns is not a diagnostic** — one
  has already been reported as a semantic finding in this campaign.
- ⛔ **No module split.** `RT-BACKEND-MODULE-SPLIT` contends on exactly these
  files and is sequenced after the capstone.

---

## Contention

**None.** Team Runtime holds this alone. The doc track runs concurrently by
standing operator exception and touches `library/` and `agent/`, never
`crates/`. ⚠ Re-measure at pickup.

---

## Hard stops

⭐ **Route a hard stop; do not push through one.** This node exists *because* a
seat pushed through one.

Two are specifically foreseeable here:

1. **A slot genuinely needs a carrier or ownership mode that does not exist.**
   That is a representation question above this slice's pay grade — ⛔ do not
   invent the variant to make `D2` reachable.
2. **The planner closure cannot answer an owner/lifetime/affinity question
   `D3` must ask.** That is slice 1 being incomplete, ⛔ not licence to compute
   the answer here.

⛔ **In both cases: do not widen the slice to make its own AC reachable** — say
so and it will be re-cut.

⏱ **Target: complete or hard-stop inside one turn.** A Steward sizing target,
⛔ not an acceptance criterion and ⛔ not something QA checks. If it is missed,
the sizing was wrong.
