---
id: RT-FRAME-MARKER-ONCE
title: "Checked Runtime frame marker is consumed more than once under a nested computational eliminator"
status: draft
owner: runtime
size: TBD
gate: none
depends_on: [RT-SRCBODY-BIND-ORDER]
blocks: []
github: null
origin: Measured at frozen base 21fd46dc by the RT-SRCBODY-BIND-ORDER D10 differential (evt_2jc88hbzfskpm). All 16 CI failures at aa032cc2 fail at the base too -- ZERO bind-order flips -- so this is pre-existing base debt, not a regression. Steward-filed (agents cannot create tracked work per COORDINATION §2).
---

> ## THE FRAME IS OWED. This node is `draft` and NOT startable.
>
> It exists so that a **skipped CI row has an owner**. A skipped row measures
> nothing; the node that owns it owns **un-skipping** it. Size is `TBD`
> deliberately -- nothing measured bounds the repair, and a guessed size on this
> campaign has been wrong every time it was guessed.

## Exact signature

```text
OrientedSubcontinuationPlanV1: checked Runtime frame marker was consumed more than once
```

## Rows it owns

- \`px7n_nested_computational_eliminator\` \`nested_err_payload_reaches_both_real_executors\`

## Why this is NOT [[RT-CARRIER-BYTESPAN-OBSERVE]]

**Different mechanism entirely.** This is a planner exact-once violation on a
frame marker, refused at object emission. It has no effect seat, no \`Avail\`
membership test, and no carrier observation in it.

## Provenance

**Fails at frozen base `21fd46dc`, so it is not caused by the de Bruijn
binding repair.** Measured per row with `--no-fail-fast`; see the hazard note
in the D10 handback -- `cargo test` with several `--test` flags is fail-fast
**per binary**, and a partial run reads as a complete one.
