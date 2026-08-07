---
id: RT-ENTRY-TRAP-PX7O
title: "px7o heterogeneous eliminator frames: native traps at the explicit entry (RuntimeTrap(4), exit 1) where the interpreter returns exit 7 -- the entry-trap family the de Bruijn repair did NOT clear"
status: closed
owner: runtime
size: TBD
gate: none
depends_on: [RT-SRCBODY-BIND-ORDER]
blocks: []
github: null
origin: Measured at frozen base 21fd46dc by the RT-SRCBODY-BIND-ORDER D10 differential (evt_2jc88hbzfskpm), row 15. Steward-filed (agents cannot create tracked work per COORDINATION §2).
---

> ## CLOSED 2026-08-07 — FALSE PREMISE. `px7o` PASSES on the candidate.
>
> **`D12` (`evt_2n9wq8xyj0aa1`) measured the complete surface: `px7o`
> `nested_err_payload_reaches_both_real_executors` is GREEN at `aa032cc2`.**
> It fails only at the base. The de Bruijn binding repair **cleared it**, along
> with `px7o` `nested_ok`, `px7p` selected-fields, and the `px8h`
> payload-direction rows — **six base failures fixed by the candidate.**
>
> **This node existed because I resolved an ambiguity by guessing.** CI reports
> the bare name `nested_err_payload_reaches_both_real_executors`, which is
> defined in **two** binaries (`px7n` and `px7o`). I attributed it to `px7o`.
> **It was `px7n`** — the frame-marker refusal owned by
> [[RT-FRAME-MARKER-ONCE]]. The `D10` differential measured `px7o` at the
> **base**, where it does fail, and I carried that forward as if it described
> the candidate.
>
> ⇒ **The claim I drew from it was also wrong.** I reported that the binding
> repair cleared the `px4b` instance of the `-4` sentinel but **not** this one,
> and that the repair was therefore possibly incomplete. **It cleared both.**
>
> **The correct lesson is the one this node already stated and I did not apply
> to myself:** a bare test name shared by two binaries names neither. Resolve
> the binary before attributing the failure — measuring at the base does not
> tell you which row is red at the tip.
>
> **`D11`'s `px7o` annotation must be REMOVED**; the row is green and skipping
> it would hide a working repair.

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
