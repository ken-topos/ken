---
id: RT-UNIT-CLOSURE-CONVERT
title: "Activate function-unit closure conversion for predeclared units — a retained nested body's free de Bruijn references become declared typed capture slots, reconstructed at unit entry from exact caller operands"
status: draft
owner: runtime
size: TBD
gate: none
depends_on: [RT-CONTSRC-PRODUCER-LOCAL]
blocks: []
github: null
origin: Architect disposition evt_56jh63qntwtfe (2026-08-05) classifying the moved D4a reds as a frame/substrate boundary rather than a bounded D3b repair, and expressly leaving the checkpoint-versus-node call to the Steward. Steward scope recut evt_7he9qv8wbv1yq. Steward-filed per COORDINATION §2.
---

## The gap, exactly

`define_unit_body` builds a predeclared unit's lowering environment **only** from
that unit's declared `Parameter`/`Capture` slots. It then lowers a retained
nested body whose free de Bruijn references **exceed that declared run**.

⇒ The body is reachable, but the function unit does not carry the lexical values
its body requires. Measured symptom at `bc371f13`:

```
Var: no runtime binding for index 2   env_len=2  defining=Predeclared(PredeclaredFunctionId(3))
```

⭐ **One uniform gap, not five.** The five deliberate `D4a` coordinate reds
**moved** here rather than clearing when `D3b` landed both lowering arms —
they now fail past the emission seam, and `D4a`'s own shifted fixture reaches
the identical boundary at `index=3`.

## ⛔⛔ SIZE THIS FROM WHAT `B2R` LEFT INERT — NOT FROM THE MECHANISM LIST

**This is the one thing that will be got wrong.** The Architect's disposition
names the lawful mechanism — planner-issued exact free-variable identities,
declared typed capture slots in the unit descriptor, exact caller operands per
call edge, reconstruction at unit entry, caller/callee equality checked before
emission — and notes it is *"the substance already promised by
`RT-FNSPLIT-B2R` D2/D3/D5"*.

⛔ **Read from that list alone, this sizes as `L` and that is wrong.**

[[RT-FNSPLIT-B2R]] is **`merged`** (PR #967). Its scope section
*"Inert only — the already-ruled scaffold escape"* landed the closure-conversion
**contract** as production code while banning any executable edge:

- declarative ABI/layout/ownership types, descriptor construction and the
  **validators may be production code** — and are;
- production retains exactly one root `FunctionBuilder` and one
  `define_function`, with **zero** new callable target unit, call edge, dispatch
  edge, callback, flag or alternate entry;
- ⛔ no encoder/decoder or helper creating a second live body-emission authority.

The validator that **rejects missing capture slots, extra capture slots and
mismatched slots** is already on `main`. So are the typed frame slots for free
variables.

⇒ **This node ACTIVATES merged substrate for a population that did not exist
until `D4a` admitted it.** It does not invent closure conversion.

⭐ **Therefore `D1` is an inventory, and it is the sizing instrument**: measure,
at the base, exactly what `B2R` left inert, which of the five mechanism elements
already exist as production types/validators, and which genuinely need an
executable edge. ⛔ Do not size the node before that answer exists — and note
that `B2F`, `C3-ACTIVATION` and `RT-NATIVE-FNSPLIT` are **also all merged**, so
the live-boundary work `B2R` deferred to `B2F` may already be landed too.
⚠ `B2R`'s own anchors section warns every anchor in that chain has moved at
least once. Re-derive; do not read the numbers.

## ⛔ Four repairs that are FORBIDDEN, and why

From the Architect's disposition. Each fabricates a binding and violates the
existing no-implicit-tail ABI law:

| forbidden | what it fakes |
|---|---|
| pad the environment vector | a value that was never passed |
| shift the `Var` | a different variable than the source names |
| copy an ambient caller tail | an implicit suffix the ABI law forbids |
| reuse continuation-call inputs as implicit unit captures | conflates the continuation contract with the unit's |

⇒ There is no relax option. That is why this is a node rather than a waiver.

## Why a node and not a checkpoint

Preference order is relax, fold, then cut (`agent/playbooks/federation/steward.md` §4).

- **Relax** — unavailable, per the table above; and this is a genuine
  correctness failure (emitted code carrying a value the body did not name), ⛔
  **not** a would-`main`-go-red concern.
- **Fold** — ⛔ **no open sibling exists.** Checked at file time, not assumed:
  `RT-FNSPLIT-B2R`, `RT-FNSPLIT-B2F`, `RT-FNSPLIT-C3-ACTIVATION` and
  `RT-NATIVE-FNSPLIT` are **all `merged`**.
- **Cut** — and out of [[RT-CONTSRC-PRODUCER-LOCAL]] rather than into it. That
  node is already four checkpoints past its own recut, was itself cut out of
  [[RT-DECL-CLOSURE-PORT]]'s `D7`, and is the sole gate on all seven nodes of
  the `RecursiveDescent` retirement campaign.

## The dependency direction, stated so it is not read as a cycle

This node builds **on** `bc371f13`, so `depends_on` names
[[RT-CONTSRC-PRODUCER-LOCAL]]. But that node's **candidate cannot close until
this one lands** — seven rows are red at `bc371f13`, five of them here.

⛔ That gate is recorded in `RT-CONTSRC-PRODUCER-LOCAL`'s frame **as prose, not
as a `depends_on` edge**, because an edge both ways is a cycle the generator
cannot resolve. Same pattern as `RT-CONTSRC-PRODUCER-LOCAL` itself against
`RT-DECL-CLOSURE-PORT`: both `active`, one branch, sequenced by the frame.

## Status: `draft`, and the frame is OWED

⛔ **This is framing debt, unlike [[RT-CONTSRC-CALLABLE-CONTRACT]]'s deliberate
`draft`.** That node is off the critical path; this one gates a candidate. The
frame is the Steward's next action on this node, and its fixed inputs must be
measured at the base then — `bc371f13` is unmerged and the anchors move.

⇒ Promote to `ready` when `docs/program/wp/RT-UNIT-CLOSURE-CONVERT.md` exists
with the `D1` inventory specified against a named base.

## What must not be lost

Runtime **preserves `bc371f13` exactly** and makes **no unit-frame edit**
meanwhile (Architect, same disposition). The `D3b` arms, crossed-pair refusals
and consumer mutations are **directionally accepted evidence** — QA approved
their fidelity to the current frame. ⛔ That is not the same as `D3b` being
complete, and the crossed-pair law's own premise is under measurement in
`RT-CONTSRC-PRODUCER-LOCAL`'s `EntryAbi` checkpoint.
