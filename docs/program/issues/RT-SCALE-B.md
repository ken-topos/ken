---
id: RT-SCALE-B
title: "Boundary B — the full n=3..7 emission measurement, the research-grounded analytical model, and the operator scaling verdict that gates RT-NATIVE-FNSPLIT's merge"
status: ready
owner: runtime
size: L
gate: none
depends_on: [RT-SCALE-A, RT-FNSPLIT-B2F]
blocks: [RT-NATIVE-FNSPLIT]
github: null
origin: Operator scaling-gate directive 2026-07-23 (evt_4btfhwqhah1ye), requirements 1-3; relocated to the recut by `docs/program/wp/RT-NATIVE-FNSPLIT-recut.md` as Boundary B, whose metric list survived the 2026-07-24 B1/B2 split (Architect evt_49bnspfb74tne + addendum evt_3b2a75fcaegja) as B2's. Research dispatch for the analytical half: evt_62fqpe7pfvym4. Steward-filed 2026-07-26 (agents cannot create tracked work per COORDINATION §2) because the gate had acceptance criteria and no tracked node.
---

> ## ▶ THE FRAME IS WRITTEN — read it, not this file
>
> `docs/program/wp/RT-SCALE-B-emission-scaling-verdict.md`

> ## ⭐ `ready` 2026-07-28, AND IT SUBSUMES `RT-FNSPLIT-B2B`
>
> **`status: draft` → `ready`.** The frame is shovel-ready; the only thing this
> node waits on is `RT-FNSPLIT-B2F` merging, which the `depends_on` edge already
> expresses. ⇒ It now enters the releasable frontier **automatically** the moment
> `B2F` lands, instead of needing a Steward pass first.
>
> **[[RT-FNSPLIT-B2B]] is `closed` as subsumed into this node.** It was the same
> deliverable: its `AC1.1′`–`AC1.5′` map one-to-one onto `AC-B1`–`AC-B4`, its
> metric list is `D2`, its differential suite is `D3`, and its closing action is
> this node's `blocks` edge on `RT-NATIVE-FNSPLIT`.
>
> ⛔ **The origin line below contains a false premise, kept visible rather than
> rewritten:** *"the gate had acceptance criteria and no tracked node"* — `B2B`
> **was** the tracked node, filed by the Steward the day before. This node
> survives the fold because it has a written frame and the **analytical half**
> (`D4`, Architect) that `B2B` never carried; four things `B2B` carried and this
> frame lacked are folded into the frame.

## ⛔ THIS IS THE NODE THAT DECIDES THE EFFORT

`RT-NATIVE-FNSPLIT` **does not merge** until this node returns a verdict. Every
other node in the chain — `B1R`, `B2A-C`, `B2A-S`, `B2O`, `B2R`, `B2V`, `B2F`,
`B2O-CHECK` — is machinery **built to be measured here**.

⭐ **The whole reason the chain was recut is that this measurement did not
exist.** 33 hard-stops of correct, converging semantic work were accumulating on
a representation that provably could not reach the gate. **Every individual
ruling was right; the thing they were accumulating into was not.** The verdict
below is what stops that from happening a second time.

## Three requirements, from the operator directive verbatim in substance

1. **Empirical scaling harness** (Runtime, **permanent tests**) — n = 3..7, each
   under a bounded fail-safe harness, reporting compile wall-time, peak RSS, and
   internal counts.
2. **Analytical scaling model** (**Architect**) — predicted order of growth vs.
   n, and specifically whether ~103 s / ~4 GB at n=4 is **bad constants on an
   O(n) mechanism** or **residual super-linearity** (⇒ a further mechanism gap).
   Must be **research-grounded** (dispatch `evt_62fqpe7pfvym4`).
3. **Verdict** — either **(a)** empirically **and** analytically **linear O(n)**
   plus a plan to reduce the constants, or **(b)** a **research-supported**
   reason growth is inherently super-linear, plus an **explicit operator
   ceiling / acceptability decision.**

⛔ **Requirement 3 is not the Runtime ring's call to make alone.** Outcome (b)
routes to the **operator** through the Steward. Do not close this node with a
verdict that only the ring has read.

## What this node does NOT own

- ⛔ **Boundary A's planner metrics.** Those are [[RT-SCALE-A]]. **Neither
  boundary may stand in for the other.**
- ⛔ **`B2F`'s `AC-G0` helper-count denominator.** That is a **Θ(1)-per-module
  growth invariant** (6 definitions / 8 declarations) and is already answered;
  it is **not** the n=3..7 empirical table and does not discharge any part of
  this node.

Gates the [[NATIVE-HANDLE-CARRIER]] fast-follow and [[PX8-F-CAP-41]] too.
