---
id: RT-SCALE-A
title: "Boundary A — re-derive the planner census for n=3..7 against the COMPLETED factored representation, superseding the provisional outer-planner numbers"
status: active
owner: runtime
size: M
gate: none
depends_on: [RT-FNSPLIT-B2A-S]
blocks: [RT-SCALE-B]
github: null
origin: Operator scaling-gate directive 2026-07-23 (evt_4btfhwqhah1ye), relocated to the recut by `docs/program/wp/RT-NATIVE-FNSPLIT-recut.md` §"THE COMPLETE n=3..7 EMPIRICAL GATE MOVES TO THE RECUT" as Boundary A. Steward-filed 2026-07-26 (agents cannot create tracked work per COORDINATION §2) because the gate had acceptance criteria and no tracked node — it lived only as prose inside RT-NATIVE-FNSPLIT.md and the recut frame.
---

> ## ▶ THE FRAME IS WRITTEN — read it, not this file
>
> `docs/program/wp/RT-SCALE-A-planner-census.md`

## Why this node exists — the requirement had no owner

The operator's scaling gate is a **merge condition on `RT-NATIVE-FNSPLIT`**. Its
Boundary A half carries five acceptance criteria (`AC1.1′`–`AC1.5′`) and a named
metric list, and until 2026-07-26 **no tracked node owned it.** It existed only
as prose inside `RT-NATIVE-FNSPLIT.md` and the recut frame.

⛔ **That is the `KW-THEOREM` failure shape.** There, the frame *correctly named*
the formatter's CI-only corpus coupling — and four exact-SHA reviews plus every
targeted local check still approved a noncanonical corpus, because CI was the
first operative control. All three rings' retros converged independently on the
same fix: **a requirement that only a distant gate can observe must become an
executable step at the point of work.** A merge condition with no node is the
same defect one level up — nothing sequences it, nothing releases it, and no
team is holding it.

## ⚠ Boundary A is NOT unstarted — and its numbers are NOT a baseline

A Boundary A census **landed at `647a2e5b`**. The recut frame is explicit that it
is **true only for the OUTER planner and remains PROVISIONAL for the completed
representation.**

⛔ **Do not cite `87/115/143/171/199`, `K=8`, or widths `12/32/16` as a settled
baseline.** This node's deliverable is the **re-derivation against the completed
factored representation**, which supersedes those numbers. A re-run that
reproduces them has not thereby confirmed them — it has to be measured on the
completed object either way.

## ⛔ What this node must NOT be asked for

**CLIF instructions/bytes and full compile wall-time / peak RSS cannot be
required here.** There is no lowering at this boundary. Requiring them of a
pre-lowering planner census is a **category error**, and the recut frame says so
in those terms. Those metrics belong to [[RT-SCALE-B]].

⛔ **Neither boundary may stand in for the other**, and ⛔ **a post-failure
prefix cannot substitute for any boundary.**

## Sequencing note the ring may return to the Steward

`depends_on: [RT-FNSPLIT-B2F]` encodes *"the factored representation is
complete."* ⚠ **If the ring determines the planner representation is complete
earlier than `B2F`** — Boundary A is a planner census and `B2F` is the emission
switch-over — **that is a re-sequencing question for the Steward, not a judgment
call to make in-flight** (COORDINATION §2). Come back and I will re-sequence.
