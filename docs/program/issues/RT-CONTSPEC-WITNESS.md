---
id: RT-CONTSPEC-WITNESS
title: "ContinuationSpecialization seam 4 — integrated witness and closeout: the native population, the six formerly shadowed rows reclassified, the two host rows rerun, and the tracker closure of every node still open in the set"
status: ready
owner: runtime
size: M
gate: none
depends_on: [RT-CONTSPEC-LEDGER]
blocks: [RT-RECURSOR-TRANSPORT]
github: null
origin: "Architect ownership/sizing ruling evt_1yymw1gdszpbs (2026-08-02), outcome (c) on RT-CONTSPEC-LOWER, seam 4 of four — the terminal seam that inherits RT-CONTSPEC-LOWER's tracker closure (three nodes as originally scoped; two as at 2026-08-08, RT-DECL-CLOSURE-PORT having merged separately). Steward-filed (agents cannot create tracked work per COORDINATION section 2)."
---

# Seam 4 — the closeout, and the only seam that closes other nodes

Runs only after seams 1-3 land. Three deliverable populations:

- the **native population** run on a lawful assembly;
- the **six formerly shadowed rows**, which become measurements only once their
  causal roots have cleared, and are reclassified here;
- the **two host `ENOSPC` rows**, rerun after capacity is available. The Steward
  cleared `/tmp` from 99 percent to 23 percent on 2026-08-02; confirm capacity
  before the run rather than assuming it held.

## The rule that keeps this seam from absorbing the campaign

**Any fresh planner or ABI failure on the lawful assembly routes back as a new
exact interface hard stop. It is not repaired inside closeout.** That is the
Architect's wording and it is the boundary that stops seam 4 becoming a second
cumulative branch.

## Closure

When this merges it closes, in one tracker commit, **every node in the set that
is still open at that point**. That obligation moved here from
RT-CONTSPEC-LOWER, which does not continue as a single node.

**Corrected 2026-08-08:** the set was written as three — itself,
[[RT-RECURSOR-TRANSPORT]] and [[RT-DECL-CLOSURE-PORT]] — and is now **two**.
[[RT-DECL-CLOSURE-PORT]] merged separately under the operator's partial-merge
policy and is already `merged`, which is terminal. The frame's `AC-7` pinned
*exactly three* flips, which a correct implementer would have failed; it now
asks for the set measured at the base.

Also carried forward unchanged: the **761 witness gate**.
`fs_read_at_malformed_offset_narrows_to_invalid_offset` must produce
`InvalidOffset`, and its sibling at
`crates/ken-cli/tests/rt_parity_native.rs:544` is covered by the same open
question — did the trap become `InvalidOffset` because the defect was fixed, or
because the assertion moved?

Branches from `main` after seam 3 lands.

## What the frame settles that this node must not be read without

**A shadowed row is unmeasured, not passing and not failing.** Reclassifying the
six means running them and recording what they actually say — either verdict. **A
shadowed row that turns out to fail is a finding, not a regression**, it does not
invalidate seams 1-3, and it is routed rather than repaired.

**The 761 gate is an open question, not a checkbox.** Both tests were observed
green on `b66dea6a`. That does not close it: the question is whether the trap
became `InvalidOffset` because the defect was fixed or **because the assertion
moved**, and a green run cannot tell the two apart. The frame requires naming the
commit and picking one.

The frame is `docs/program/wp/RT-CONTSPEC-WITNESS.md`.

**Recut 2026-08-03 and now `ready`.** It was held at `draft` from 2026-08-02
because its frame treated the `46d29783` first-refusal census as a worklist whose
rows seams 2 and 3 would clear. They do not: seam 2 was recut off the census
entirely (`evt_2zhx69f2fw07w`, Architect confirmation `evt_66t42tapvdbsj`) and
merged that way at `0a6e34cc`, and seam 3's landed frame selects no census rows.
Every row's disposition therefore depended on a clearing that will never happen.

The recut keeps the reconciliation and changes its footing: the 138 rows are
**accounted for as a record of what once failed in the held `1aef3192`
lineage**, each `superseded`, `live` with a current verdict on the lawful base,
or `open` with an owner. **No row's disposition may depend on a seam having
cleared its causal root.**

Release gates on [[RT-CONTSPEC-LEDGER]] merging — a `depends_on` edge, not a
framing debt.
