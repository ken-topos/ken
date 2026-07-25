---
id: RT-FNSPLIT-B2A
title: "RT-NATIVE-FNSPLIT Boundary B2a — make the semantic plane load-bearing for emission (behaviour-preserving port)"
status: active
owner: runtime
size: L
gate: none
depends_on: [RT-FNSPLIT-B1R]
blocks: [RT-FNSPLIT-B2B]
github: null
origin: recut frame docs/program/wp/RT-NATIVE-FNSPLIT-recut.md; Boundary B split at an Architect review gate (evt_49bnspfb74tne, addendum evt_3b2a75fcaegja). B2 further split into B2a/B2b by the Steward 2026-07-25 on the runtime ring's own B1 retro carry — "keep representation checkpoints separate from a retained emission port".
---

> ## Authoritative frame: `wp/RT-NATIVE-FNSPLIT-recut-B2a-emission-port.md`
>
> Read that, not this file. This entry exists so the tracker and the dependency
> graph see the work.

> ## ✅ HOLD RELEASED — `ready` → `active`, kicked to the Runtime ring 2026-07-25
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
