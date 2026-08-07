---
id: RT-ENTRY-TRAP-PX7O
title: "px7o heterogeneous eliminator frames: native traps at the explicit entry (RuntimeTrap(4), exit 1) where the interpreter returns exit 7 -- the entry-trap family the de Bruijn repair did NOT clear"
status: draft
owner: runtime
size: TBD
gate: none
depends_on: [RT-SRCBODY-BIND-ORDER]
blocks: []
github: null
origin: Measured at frozen base 21fd46dc by the RT-SRCBODY-BIND-ORDER D10 differential (evt_2jc88hbzfskpm), row 15. Steward-filed (agents cannot create tracked work per COORDINATION §2).
---

> ## THE FRAME IS OWED. This node is `draft` and NOT startable.
>
> It exists so a skipped CI row has an owner. A skipped row measures nothing;
> this node owns **un-skipping** it. Size is `TBD` deliberately.

## Exact signature

`px7o_heterogeneous_eliminator_frames.rs:118:5`, `assertion left == right`:

| | native | interpreter |
|---|---|---|
| terminal_error | `Some(RuntimeTrap(4))` | — |
| terminal_exit | `ControlledTrap` | `ReturnedError` |
| exit_status | `1` | `7` |
| stdout | — | `"seed:err-payload"` |
| effect_trace | empty | three `ConsoleWrite`/`ConsoleFlush` |
| stderr | `ken native trap: explicit entry trap` | — |

**The ONLY runtime-behavioural row of the sixteen.** The other fifteen refuse
before execution, at object emission — they never run, so they cannot exhibit a
behavioural symptom at all.

## THE POINT OF THIS NODE, and it is not the row

`RuntimeTrap(4)` is the **`-4` explicit-entry-trap sentinel** — the same family
that [[RT-ENTRY-TRAP-254]] chased through `D0`-`D9` and that
[[RT-SRCBODY-BIND-ORDER]]'s de Bruijn binding repair **did** clear for the
`px4b` instance (its `D4` un-skipped that row and greened it).

⇒ **This instance was not cleared.** So either the repair is incomplete, or
`px7o` reaches the same sentinel by a different cause.

**That is unmeasured and it must be measured, not argued.** Do not assume it is
the same defect because it shares a sentinel and a stderr string — matching a
signature to a mechanism without measuring the path is the exact inference the
Architect refuted twice on this campaign (`evt_7v61ed5pn9q3t`,
`evt_m36y2zegby7m`).

## The first question, when this is framed

**Does `px7o` still fail at `aa032cc2`?** It fails at the base. CI's failure
list at `aa032cc2` carries the **bare** name `nested_err_payload_reaches_both_-
real_executors`, which is defined in **two** binaries (`px7n` and `px7o`), so
which one failed there is **ambiguous from the CI list alone**. Resolve that
first — if `px7o` passes at `aa032cc2`, this node is already discharged and
closes.

## Provenance

Fails at frozen base `21fd46dc`, so it is not caused by the binding repair.
