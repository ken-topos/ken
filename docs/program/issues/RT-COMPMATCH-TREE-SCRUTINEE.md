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
