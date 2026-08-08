---
id: RT-COMPMATCH-TREE-SCRUTINEE
title: "ComputationalMatch refuses a tree-producing scrutinee that is not Bool or a constructor (rt_span_prov)"
status: draft
owner: runtime
size: TBD
gate: none
depends_on: [RT-SRCBODY-BIND-ORDER]
blocks: []
github: null
origin: Measured by the RT-SRCBODY-BIND-ORDER D12 complete no-fail-fast enumeration (evt_2n9wq8xyj0aa1). Fails at frozen base 21fd46dc as well as at the candidate, so it is pre-existing base debt and not a regression. Steward-filed (agents cannot create tracked work per COORDINATION §2).
---

> ## THE FRAME IS OWED. `draft`, NOT startable.
>
> It exists so a skipped CI row has an owner. **A skipped row measures nothing;
> this node owns un-skipping it.** Size is `TBD` deliberately.

## Exact signature

```text
ComputationalMatch: tree-producing match scrutinee is not Bool or a constructor
```

## IT IS THE DESCENT CAMPAIGN'S ONLY REAL-PROGRAM WITNESS FOR THIS CLASS

Steward, 2026-08-08, `evt_27jwdbz9h2t4c`, routed from
[[RT-SPECIALIZED-ACTIVE-RESUME]]'s cross-crate census.

That census ran the descent campaign's boundary instrument inside `ken-cli` and
`ken-verify` with `--include-ignored`. **The sole failure was this node** — and
that makes it load-bearing for a question the campaign has carried since node
#6b.

The campaign's Trap 1 is *"a hand-built `RuntimeExpr` fixture proves the
classifier sees the class, not that a real Ken program exhibits it."* Its
population is two hand-built values. **This node is a real Ken program producing
a non-constructor `ComputationalMatch` scrutinee** — the same class, failing at a
**different consumer**.

> ### IT SPLITS A QUESTION THAT WAS BEING CARRIED AS ONE
>
> - *Do real programs exhibit this shape?* **Yes** — this node is the witness.
> - *Is the descent campaign's specific cell reachable in production?* **No**,
>   and that is a property of the `#[cfg(test)]` activation seam, **not of the
>   language.**
>
> Only the first was ever what Trap 1 was about. Recording the split here
> because this node is where a reader arrives holding the witness.

**This does not make the node startable and does not change its owner.** It
raises what closing it is worth: it is the only place the class can be studied
on a real program rather than a fixture.

## Why it is not one of the released owners

**Distinct refusal class.** Not an effect-seat \`Need\`/\`Avail\` membership
question, so it is neither [[RT-CARRIER-BYTESPAN-OBSERVE]] nor
[[RT-CARRIED-RESOURCE-SCALAR]]. This is the scrutinee-shape refusal named in
[[RT-CONTSRC-PRODUCER-LOCAL]]'s two-population split, where it was the
signature of the **five** rows rather than the \`AC-1\` row.

## Provenance

**Fails at frozen base `21fd46dc`.** The complete surface is 40 candidate
failures, all of which fail at the base — **zero regressions**, and the
candidate additionally **fixes six** base failures. Enumerated with
`--no-fail-fast`, which is a closed enumeration because fail-fast is per
**binary**, not per test.
