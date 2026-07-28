---
id: RT-FNSPLIT-B2A
title: "RT-NATIVE-FNSPLIT Boundary B2a — make the semantic plane load-bearing for emission (behaviour-preserving port)"
status: closed
owner: runtime
size: L
gate: none
depends_on: [RT-FNSPLIT-B1R]
blocks: []
github: null
origin: recut frame docs/program/wp/RT-NATIVE-FNSPLIT-recut.md; Boundary B split at an Architect review gate (evt_49bnspfb74tne, addendum evt_3b2a75fcaegja). B2 further split into B2a/B2b by the Steward 2026-07-25 on the runtime ring's own B1 retro carry — "keep representation checkpoints separate from a retained emission port".
---

> ## Authoritative frame: `wp/RT-NATIVE-FNSPLIT-recut-B2a-emission-port.md`
>
> Read that, not this file. This entry exists so the tracker and the dependency
> graph see the work.

> ## ⛔ RETIRED 2026-07-25 — SUPERSEDED BY `RT-FNSPLIT-B2A-S` + `RT-FNSPLIT-B2F`
>
> **The Architect retired this frame outright** (`evt_6h5gw5c503n5z`, amendment
> `evt_25ynt8615r9sk`). It is kept for lineage; ⛔ **do not build from it.**
>
> - **`RT-FNSPLIT-B2A-S`** — defunctionalize retained body selection (the
>   static-origin tag + one closed consumer). Retires inventory entry 1.
> - **`RT-FNSPLIT-B2F`** — per-static-origin Cranelift target functions, atomic
>   with switch-over, differential equivalence, and old-authority removal.
>
> ⚠ Its ACs were **re-walked and placed deliberately**, not copied: see each
> successor node. `AC-4`/`AC-5` → `B2A-S`; `AC-1`/`AC-2`/`AC-3`/`AC-7`/`AC-8` →
> `B2F`.

> ## ⛔ HARD-STOPPED PRE-CODE (#6) — 2026-07-25 (the stop that retired this frame)
>
> **Kicked, audited, and stopped before a single edit** (`evt_3xzv4xn77na0d`;
> leader confirmed `evt_34y9pnbs8r330`). The frame below is *known wrong*, and
> `ready` would invite releasing it again.
>
> ⇒ **`status: closed`** — flipped from `draft` 2026-07-28. A **retired** node is
> resolved-without-landing, which is what `closed` means; leaving it `draft` kept
> a known-wrong frame sitting in the graph looking like unstarted work.
>
> ### The defect is MINE, and it is a class of framing error worth naming
>
> **B2a's `Retain` and `Replace` lists were inherited from the HELD tree** and
> describe artifacts that are **not in B2a's base**. I re-verified all three
> deciding measurements independently rather than on report:
>
> | claim | verdict on `7151ae58` |
> |---|---|
> | `partition.rs` / `PartitionWorkItem` / `work_?item` | **absent; 0; 0** — live only on `preserved/wp-…-b077eb7a`, **not an ancestor of `main`** (merge-base `8ebe370a`) |
> | "bounded deferred Cranelift functions" to retain | **none** — production `lowering/` has exactly **one** `FunctionBuilder::new` (`core.rs:140`) and **one** `define_function` (`core.rs:202`); all other sites are under `core/tests/` |
> | planner↔emitter coupling | **one symbol** — `plan_static_transition_graph`, built `core.rs:35`, dropped `:204`, zero refs to the plane types outside `planning/` |
>
> ⇒ **The emitted units D2 would re-key do not exist.** Creating them means
> per-transition function declaration, a real calling convention for the 8-field
> `DynamicActivationFrame`, and a persistent-store runtime — then proving
> behaviour preservation across the whole 6201-line SCC. **That is a
> construction, not the behaviour-preserving port this frame asserts**, and
> `AC-2`'s "removed whole-configuration emission path" is not a separable path
> at all: it is `lower_expr`'s entire recursive-descent inliner
> (`core.rs:3847`, 60 call sites).
>
> ★ **The ring did exactly what the frame told it to** — invoked the
> unreviewable-diff stop, and explicitly refused to reinterpret the deliverables
> to fit what was buildable. **That refusal is the valuable output of this
> kickoff.** ⛔ Nothing was edited or committed; the WP ref is free at exactly
> `7151ae58`.
>
> ### ⛔ Re-slice is pending and is the STEWARD's, gated behind two inputs
>
> 1. **The #6 research advisory** — the pull fired; the Architect's ruling is
>    gated *behind* it, not delivered instead of it.
> 2. **One Architect design call the implementer correctly refused to make
>    itself:** in the proposed first slice, resolving `static_origin →
>    &RuntimeExpr` needs the plan to retain occurrence references (a lifetime on
>    `StaticTransitionPlan`). Is a *derived* reference compatible with "never
>    from pointer, content, clone order, or activation" when the identity is the
>    positional ordinal? ⛔ **Not the Steward's call.**
>
> ⚠ **When the re-slice lands, re-walk every AC and control** — several are
> defined on deliverables that will move, and scoping a deliverable out strands
> the ACs defined on it.
>
> ## ✅ (superseded) HOLD RELEASED — `ready` → `active`, kicked 2026-07-25
>
> **`RT-FNSPLIT-B1R` merged as PR #937** and is content-verified on `origin/main`
> = `7151ae58` (a squash lands under a new SHA, so ancestry of the approved
> `e58b3fa6` is **not** the test — the check was the landed content). Retros in,
> WP closed under §10. The blocker this WP was held behind is discharged.
>
> ⛔ **CUT A FRESH BRANCH FROM `origin/main`. Do NOT reuse
> `wp/RT-FNSPLIT-B2A-emission-port`.** That name exists as a **local ref at
> `5015bc71`** — i.e. **pre-B1R** — in the shared object store, so a plain
> `git checkout` of it silently lands you on a base that predates the
> representation this WP consumes. It was never on `origin`. The Steward deleted
> it at kickoff for exactly this reason; if you see it, it is not a resume point.
>
> ### Prior history, retained (this is why the frame reads the way it does)
>
> **This WP hard-stopped before any code** (`evt_6fm274bx4q6hb`, hard-stop #4 on
> the recut chain). The Architect classified the cause as a **representation
> defect in landed B1**, not B2a plumbing (`evt_7d5v99mh8n9cc`), and ruled it a
> **recut ahead of B2a** rather than an in-slice ruling. `RT-FNSPLIT-B1R` then
> hard-stopped at **#5** because the origin **carrier** could not be added
> without editing `lowering/core.rs` — so the carrier moved **here**, as **D0**.
>
> ⚠ **The frame was stale in one respect and it was my defect, not the ring's:**
> fixed input #1 asserted *"B1's plane is the representation"* and stated the seam
> by quoting `lowering/core.rs:33-35`'s promise — *"Phase 2 will consume this plan
> for emission"* — **as if the bridge existed.** It is a promise in a comment, not
> a mechanism; `core.rs:204` unconditionally **drops** the plan. The frame now
> carries that as an explicit finding, but **re-derive it against the landed B1R
> plane at pickup rather than reading it forward.**

## Why B2 is two slices

**B1 landed `5554b33f`** — the closed semantic-IR plane plus its sole exhaustive
builder — and the plane is currently **built, validated, and then dropped**
(`lowering/core.rs:33-35`: *"Phase 2 will consume this plan for emission; until
then the existing emitter remains unchanged"*).

Making it load-bearing means touching a **6201-line** emitter under a
behaviour-preserving mandate. Bundling the growth census into that same slice
would measure a moving target and leave a reviewer unable to separate a
regression from a redesign — the same argument the Architect used to split
Boundary B into B1/B2 in the first place, and the same one the runtime ring
reached independently in its B1 retro.

- **B2a** (this WP) — the port. Behaviour unchanged; the **differential suite**
  is the acceptance argument. ⛔ No growth claim.
- **`RT-FNSPLIT-B2B`** — the full emission census, first and second finite
  differences, and the explicit growth verdict that answers the operator's
  scaling gate `evt_4btfhwqhah1ye`.

## What closing both means

`RT-NATIVE-FNSPLIT` stays `active` until B2b lands. It gates
`NATIVE-HANDLE-CARRIER` → `PX8-F-CAP-41` downstream, and it carries the
operator's standing priority — roughly 36 hours of effort as of 2026-07-25.

## Landed so far

| piece | SHA |
|---|---|
| Boundary A — the planner | `647a2e5b` |
| `RT-PLANNER-DIAGNOSTIC-K` | `36dd61f6` |
| Boundary B1 — semantic-IR plane + sole builder | `5554b33f` |
| `RT-PLANNER-ATTRIB-K` (J1 spin-off) | `5015bc71` |
| **B2a — emission port** | **this WP** |
| **B2b — census + growth verdict** | `RT-FNSPLIT-B2B` |
