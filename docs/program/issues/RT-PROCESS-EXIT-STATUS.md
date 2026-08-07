---
id: RT-PROCESS-EXIT-STATUS
title: "ProcessExitStatus refusal in the escape lane (rt_escape r2_cross_buffer_freeze_fails_closed_with_invalid_bounds)"
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
ProcessExitStatus refusal, rt_escape r2_cross_buffer_freeze_fails_closed_with_invalid_bounds
```

## Why it is not one of the released owners

**Fits none of the five released owners** — the ring said so explicitly rather
than forcing it into the nearest one, which was the right call. Capture the
exact wording from the D12 handback before framing; the signature above is the
class, not yet the verbatim text.

## Provenance

**Fails at frozen base `21fd46dc`.** The complete surface is 40 candidate
failures, all of which fail at the base — **zero regressions**, and the
candidate additionally **fixes six** base failures. Enumerated with
`--no-fail-fast`, which is a closed enumeration because fail-fast is per
**binary**, not per test.
