---
id: SEAL-2
title: carrier producer closure, over a derived enumeration
status: draft
owner: foundation
size: M
gate: none
depends_on: [RT-PARITY, BUDGET-EFF]
blocks: []
github: null
origin: evt_74mjc4txd9y1e
---

Closes a derived-enumeration gap in the `SPAN-SEAL` producer-closure oracle:
every namespace, every result position, every source root, derived from the
elaborator's own structure so a new namespace is a build break. Adversary
findings S1+S2 on `SPAN-SEAL cd4184b8`, both against a WP that closed with no
live defect — this is hardening the oracle, not fixing a contradiction.

Not yet framed to a branch. Base will be re-anchored on `origin/main` at kick
time (RT-PARITY was in flight and touches adjacent prose when this was
drafted). Blocked by RT-PARITY closing and, per the current tracker's
sequencing, by BUDGET-EFF (a live defect outranks closing a gate with none).
RT-PARITY has since merged.

Full brief: [`docs/program/wp/SEAL-2-carrier-producer-enumeration-closure.md`](../wp/SEAL-2-carrier-producer-enumeration-closure.md).
