---
id: RT-CONTSPEC-SUBSTRATE
title: "ContinuationSpecialization slice 0 — re-derive and independently gate the DORMANT D7 substrate: closed case-emission reachability, exact occurrence/owner/lifetime authority, pre-allocation closure"
status: ready
owner: runtime
size: M
gate: none
depends_on: []
blocks: [RT-CONTSPEC-PLANNER]
github: null
origin: "Architect evt_6wkw2c7ykjxsy (2026-08-01), answering the Steward's base fork at evt_1bh3p4wx76wtv with outcome (C). Slice 0 of the staged recut ruled at evt_4t09329vdrf. Steward-authored; agents cannot create tracked work per COORDINATION §2."
---

> # ⭐ SLICE 0 OF 4 — IT EXISTS BECAUSE NEITHER AVAILABLE BASE WAS LAWFUL
>
> The Steward asked the Architect which base slice 1 should branch from and got
> a third answer (`evt_6wkw2c7ykjxsy`). Both obvious options were defective:
>
> | option | why it fails |
> |---|---|
> | branch slice 1 from `93746ada` | ⛔ it is a **proved semantic ORACLE, not a lawful integration base** — not an ancestor of `main`, and itself carrying six Runtime paths at **+2,467/−526**. It would hide unmerged, un-QA'd `D7` implementation inside a planner-labelled PR — ⭐ **recreating the exact sizing/reviewability defect the recut exists to fix.** |
> | branch slice 1 from `main` | ⛔ forces it to **duplicate or assume** the case-emission/lifetime authorities it is meant to *consume*. |
>
> ⇒ **This slice lands those authorities on `main`, dormant and independently
> gated, so slice 1 can simply consume them.**

## The branch chain, as ruled

1. **slice 0 — this node** — from then-current `origin/main`
2. slice 1 [[RT-CONTSPEC-PLANNER]] — from `main` **after slice 0 lands**
3. each later slice from `main` after its predecessor lands

⛔ **No slice branches from another slice's branch, and no slice branches from a
preservation ref.**

## ▶ THE FRAME IS WRITTEN

`docs/program/wp/RT-CONTSPEC-SUBSTRATE.md`.

## Scope, as ruled

**Extract, re-derive and independently gate only the already-accepted dormant
`D7` substrate that later slices need:** the closed case-emission reachability
facts, the exact occurrence/owner/lifetime authority, and their pre-allocation
closure.

⛔ **Banned, by name, in the ruling:**

- ⛔ **Do not cherry-pick or land `93746ada` wholesale.** ⭐ *"Extract from it
  claim-by-claim and re-prove each slice."*
- ⛔ **Do not activate dynamic continuation transport.**
- ⛔ **Do not include the rejected post-join / static-worker mechanism.**

⚠ **It is not a new semantic node**, carrier lane, disposition, or participant —
it is an implementation slice inside the same `D7` + [[RT-RECURSOR-TRANSPORT]]
atomic scope.

## ⛔ The two prototype objects are NOT interchangeable

The Architect settled this explicitly when the freeze fingerprint failed to match
the audit snapshot:

| object | standing |
|---|---|
| `c4ff4f92…` | the **audit-time snapshot** only |
| `465fab90` / patch `d04a64a7…` | the **authoritative later freeze**, on `origin/preserved/rt-recursor-freeze-465fab90` |

⭐ **The retainability finding was pinned to the NAMED DESIGN STRUCTURES, not to
approval of any exact patch.** ⇒ `465fab90` is prototype/reference material with
⛔ **no acceptance transfer** — extract claim-by-claim and re-prove.
