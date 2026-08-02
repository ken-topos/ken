---
id: RT-CONTSPEC-LEDGER
title: "ContinuationSpecialization seam 3 — D7 ledger and population closure: the exact source and synthesized-aggregate ledger rows plus the representation and lifetime controls, from the graph-derived authorities already ruled"
status: ready
owner: runtime
size: M
gate: none
depends_on: [RT-CONTSPEC-ACTIVATE]
blocks: [RT-CONTSPEC-WITNESS]
github: null
origin: "Architect ownership/sizing ruling evt_1yymw1gdszpbs (2026-08-02), outcome (c) on RT-CONTSPEC-LOWER, seam 3 of four. Steward-filed (agents cannot create tracked work per COORDINATION section 2)."
---

# Seam 3 — close the D7 ledger over a population that is finally complete

Population: the Architect's **17 rows** — exact source and synthesized-aggregate
ledger gaps, plus the D7 closure, representation and lifetime controls. The
governing authorities are already ruled; this seam applies them, it does not
re-derive them.

Gates exact planned/emitted equality and the existing negative discriminators.

## The trap this seam sits directly on top of

Campaign trap 3: **a proof over an incomplete population is vacuous, and every
control over it passes.** That is exactly what rejected `RT-JOIN-DISPOSITION`'s
`27f9dca2` — one production site bypassed the recording call, so a whole class
proved over an empty list.

⇒ Any proof this seam adds over a population owes a paired control that
**reds when a member is omitted from the population**, not merely one that
passes when the proof holds.

Branches from `main` after seam 2 lands, and carries only its own delta.

## What the frame settles that this node must not be read without

**The scope oracle is seam 1's `D4`, not the `46d29783` census.** That census
differentiates only **12** of the 17 D7-owned rows, all under
`source boundary-use ledger missing`. The other 5 — the synthesized-aggregate
ledger gaps and the representation/lifetime controls — sit inside the 39 rows it
records as "ownership matrix pending." Selecting from it drops 5 rows silently.

The frame is `docs/program/wp/RT-CONTSPEC-LEDGER.md` — written 2026-08-02 while
seam 1 is in flight. Node is `ready`; release is gated on seam 2 merging, not on
further framing.
