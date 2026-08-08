---
id: RT-MATCH-RECURSOR-CONSUMERS
title: "Complete the MatchScrutineeRecursor consumer repair in Position A — the D2 increment closed one witness, not the population"
status: ready
owner: runtime
size: M
gate: none
depends_on: []
blocks: [RT-RECURSOR-TRANSPORT]
github: null
origin: Architect re-rule evt_3r4j14fv1jtj2 (2026-08-08) on the nine-expression census evt_16cmej481q7ns, partitioning RT-RECURSOR-TRANSPORT hard stop 4 by measured residual population. Row 6 (d8d) is a D2 completeness defect in Position A, not a lexical-successor row. Campaign docs/program/16-recursive-descent-retirement.md node #6d. Steward-filed (agents cannot create tracked work per COORDINATION §2).
---

> # THIS NODE EXISTS BECAUSE `D2`'s RECORD OVERCLAIMED — NOT BECAUSE `D3`
> # BROKE SOMETHING
>
> **The distinction changes what you are looking for, so read it before `D0`.**
> Row 6's refusal reproduces at exact `D2` `8efdfdb3`, with the `D2` repair
> active and production still selecting `RecursiveDescent`. **It is not
> downstream of `D3`'s retirement** — `D3` merely made it unavoidable.
>
> ⇒ `D2` is a **completeness/scope defect**, not a later sizing discovery. The
> production change at `8efdfdb3` is **sound and stays** — it correctly closes
> the exact `D1` A witness at the exact `resume_active_continuation` seat. What
> was false is its record's claim that *"position A closes"* and that *"both
> lanes now agree on position A."*
>
> **Do not revert the `D2` mechanism.** This node completes it.

## What it is

**Row 6, `d8d`**, the composed binding-site fixture. It enumerates exactly
`{MatchScrutineeRecursor}` — it was never in the `LexicalCallArgumentRecursor`
population, and B-only exclusion is not merely weak for it but **inapplicable**:
the hook's own `debug_assert` refuses an exclusion of a variant that is not in
the set.

Under **A-only exclusion** at `8efdfdb3` it reaches `FunctionizedUnits` and
refuses:

```
Unsupported(UnsupportedLowering {
  construct: "RecursiveBackedge",
  reason: "protocol machinery is never a source value at a boundary" })
```

## The population is the production predicate; `d8d` is a floor

**`D0` closes the population from the production `MatchScrutineeRecursor`
predicate — not from `d8d`'s spelling.** One fixture is a witness, never a
perimeter, and this node was created precisely because a one-witness result was
read as a class-wide property.

## `D1` — activate and attribute before any repair

**A-only exclusion is the activation seam**, proven at `8efdfdb3`. Use the
existing one-variant hook exactly as designed.

- Reproduce each row's **exact first refusal**.
- The ordinary retained run stays green.
- Record **exact activation denominators**, so a refusal cannot be credited when
  the selector or harness never reached the path.
- **Trace each red to the first missing or mis-consumed static fact**, and
  attribute the owner. A rendered refusal string is a symptom, not a cause.

**Banned:** simultaneous exclusion of both variants, a generalized hook, any
`#[ignore]`, and reinterpreting a retained `RecursiveDescent` run as activation.

## The guard that may not be weakened

**`RecursiveBackedge` remains protocol-only and may never become an accepted
source boundary value.** The lawful repair makes the protocol get **consumed or
represented at its owner before the value boundary**; it does not teach the
downstream guard to accept the forbidden state, and it does not make the marker
boundary-transferable.

Also banned as mechanisms: any fallback to `RecursiveDescent`, `BoundaryUse`,
`PlannedEffectSeat` widening, a lowering-minted token, and invocation-local
activation/resume/return-hole state in ABI data.

## Why it goes FIRST, ahead of [[RT-LEXICAL-RECURSOR-CONSUMERS]]

**It closes the claim the `D2` record correction is in the act of narrowing.**
That correction says the A population is *still open at `d8d`*; this node is
what closes it. Landing B first would leave that open statement standing longer
for no gain.

**Do not fold the two nodes together.** The exact residual producer, activation
hook, observed boundary and completion owner all differ. If the two `D1` causal
partitions later prove one exact shared production root, **route a subsumption
proposal before coding** — Runtime may not infer it from shared retirement
timing or shared syntax. Conversely, **materially distinct authorities are a
hard stop** for either node's provisional size.

## Size

**`M`, provisional**, and the provisional part is real: `d8d` is one measured
expression and `D0` may find the A population materially wider. **Return the
partition before coding** if it does.

## Sequence

1. `10369776252861e8b15e613576256a3682c70066` stays **held evidence only**.
2. The bounded `D2` record correction lands (a fresh child over `8efdfdb3`; no
   production or test-logic change; fresh SHA, QA, Architect, Decision).
3. **This node releases and merges.**
4. [[RT-LEXICAL-RECURSOR-CONSUMERS]] releases and merges.
5. [[RT-RECURSOR-TRANSPORT]] `D3` resumes from the resulting `main`, reapplies
   the retirement and the `AC-2b` dispositions, and proves all six old-green
   rows green **without exclusion**.

Both successor nodes block `D3`. [[RT-DESCENT-RETIRE]] remains downstream.

## The edge that is not in the frontmatter

This node's base is **post-`D2`-correction `main`**, and that correction is a
*partial* merge of [[RT-RECURSOR-TRANSPORT]], not its completion. A `depends_on`
naming that node would be a **cycle**, since its `D3` is blocked on this one.
`depends_on` is empty; the base is stated here and in the frame. The
machine-checked edge is `blocks: [RT-RECURSOR-TRANSPORT]`.
