---
id: RT-FNSPLIT-B2A
title: "RT-NATIVE-FNSPLIT Boundary B2a — make the semantic plane load-bearing for emission (behaviour-preserving port)"
status: ready
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

> ## ⛔ HELD — flipped `active` → `ready` behind `RT-FNSPLIT-B1R` (2026-07-25)
>
> **This WP hard-stopped before any code** (`evt_6fm274bx4q6hb`, hard-stop #4 on
> the recut chain). The Architect classified the cause as a **representation
> defect in landed B1**, not B2a plumbing (`evt_7d5v99mh8n9cc`), and ruled it a
> **recut ahead of B2a** rather than an in-slice ruling.
>
> ⇒ `RT-FNSPLIT-B1R` encodes the occurrence-local semantic material that B1
> counted but never stored. **B2a re-anchors and performs the emission port after
> B1R lands.** The branch `wp/RT-FNSPLIT-B2A-emission-port` is clean at
> `5015bc71` and no code change is authorized on it.
>
> ⚠ **This frame is now stale in one respect and it is my defect, not the
> ring's:** fixed input #1 asserted *"B1's plane is the representation"* and
> stated the seam by quoting `lowering/core.rs:33-35`'s promise — *"Phase 2 will
> consume this plan for emission"* — **as if the bridge existed.** It is a
> promise in a comment, not a mechanism. **Re-derive that input against the
> landed B1R plane at re-anchor; do not read it forward.**

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
