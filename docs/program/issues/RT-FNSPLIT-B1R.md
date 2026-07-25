---
id: RT-FNSPLIT-B1R
title: "RT-NATIVE-FNSPLIT Boundary B1R — encode the occurrence-local semantic material B1 counted but never stored (repair of landed B1)"
status: active
owner: runtime
size: L
gate: none
depends_on: []
blocks: [RT-FNSPLIT-B2A]
github: null
origin: RT-FNSPLIT-B2A carrier hard-stop (runtime-implementer evt_6fm274bx4q6hb, hard-stop #4 on the recut chain); Architect classification evt_7d5v99mh8n9cc ruling option (B) as a REPRESENTATION RECUT ahead of B2a, with symptom-inventory entry 1 appended at a8eaba91. Slice cut by the Steward 2026-07-25.
---

> ## The authoritative frame is `docs/program/wp/RT-NATIVE-FNSPLIT-recut-B1R-semantic-material.md`
>
> Read that, not this file. This entry exists so the tracker and the dependency
> graph see the work.

## Why this WP exists — landed B1 contradicts the B1 frame

`RT-FNSPLIT-B2A` hard-stopped before writing any code: the semantic plane has no
static occurrence carrier into emission, and the only demonstrated bridge is the
retained `415b5aa7` oracle keyed on `expr as *const RuntimeExpr` — B1's
explicitly rejected pointer-origin recovery.

The Architect classified the cause as a **representation defect in landed B1**,
not B2a plumbing:

- **B1 D3** requires material held out of line by dense ranges/IDs;
  `build_semantic_plane` manufactures `0..source_material_elements` ordinal
  **placeholders** and stores no atoms and no source-child origins.
- **B1 D4** forbids emission-time body reconstruction; with no material, that is
  the only way to emit.

★ **On the record** (`evt_7d5v99mh8n9cc`): *"I approved B1 while reading the
counted placeholder arena as the material arena; that review conclusion was
wrong, and this hard-stop exposes it."* ⇒ This is **B1's unfinished second
half**, sequenced ahead of B2a.

## Why option (A) was rejected

Threading `lower_expr(builder, origin, expr, env)` preserves static helper
identity but leaves the cloned `RuntimeExpr` as the authoritative body: the plane
could name the emitted unit but not supply or verify it, and a same-shaped
body/origin cross-wire would pass today's shape/count checks. That is **two
authorities** — the exact condition B2a exists to remove.

## Scope boundary (Steward's slice call — Architect to confirm at review)

**B1R adds the origin carrier; `RT-FNSPLIT-B2A` removes the `RuntimeExpr` body
and closes the origin-driven emission seam.** The removal is what forces the
6201-line `lowering/core.rs` edit, whose known failure mode is an unreviewable
diff. ⛔ **B1R must make the origin-driven seam POSSIBLE, not build it.** The old
emission path stays authoritative at the end of this WP, by design.

## Symptom inventory — LIVE, entry 1 recorded

The chain's inventory is append-only and carries across slices. Entry 1
(`a8eaba91`): *retained body selection — keyed on cloned `RuntimeExpr` pointer
identity* — which **already reduces to the chain's predicate**, *a dynamic
property must not name static code*. That reduction is why this is a recut rather
than a ruling. **Hard-stop count = 4; next Research pull = #6.**

## Chain position

| piece | SHA |
|---|---|
| Boundary A — the planner | `647a2e5b` |
| `RT-PLANNER-DIAGNOSTIC-K` | `36dd61f6` |
| Boundary B1 — semantic-IR plane + sole builder | `5554b33f` |
| `RT-PLANNER-ATTRIB-K` (J1 spin-off) | `5015bc71` |
| **B1R — semantic material (repair of B1)** | **this WP** |
| B2a — emission port (held, re-anchors after B1R) | `RT-FNSPLIT-B2A` |
| B2b — census + growth verdict, closes the campaign | `RT-FNSPLIT-B2B` |
