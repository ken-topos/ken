---
id: RT-UNIT-CLOSURE-CONVERT
title: "Activate function-unit closure conversion for predeclared units — a retained nested body's free de Bruijn references become declared typed capture slots, reconstructed at unit entry from exact caller operands"
status: closed
owner: runtime
size: TBD
gate: none
depends_on: [RT-CONTSRC-PRODUCER-LOCAL]
blocks: []
github: null
origin: Architect disposition evt_56jh63qntwtfe (2026-08-05) classifying the moved D4a reds as a frame/substrate boundary rather than a bounded D3b repair, and expressly leaving the checkpoint-versus-node call to the Steward. Steward scope recut evt_7he9qv8wbv1yq. Steward-filed per COORDINATION §2.
---

**Frame:** `docs/program/wp/RT-UNIT-CLOSURE-CONVERT.md`.

## The gap, exactly

`define_unit_body` builds a predeclared unit's lowering environment **only**
from that unit's declared `Parameter`/`Capture` slots. It then lowers a
retained nested body whose free de Bruijn references **exceed that declared
run**.

The body is reachable, but the function unit does not carry the lexical values
its body requires. Measured symptom at `bc371f13`:

```
Var: no runtime binding for index 2   env_len=2  defining=Predeclared(PredeclaredFunctionId(3))
```

**One uniform gap, not five.** The five deliberate `D4a` coordinate reds
**moved** here rather than clearing when `D3b` landed both lowering arms — they
now fail past the emission seam, and `D4a`'s own shifted fixture reaches the
identical boundary at `index=3`.

## Size this from what `B2R` left inert, not from the mechanism list

**This is the one thing that will be got wrong.** The Architect's disposition
names the lawful mechanism — planner-issued exact free-variable identities,
declared typed capture slots in the unit descriptor, exact caller operands per
call edge, reconstruction at unit entry, caller/callee equality checked before
emission — and notes it is *"the substance already promised by
`RT-FNSPLIT-B2R` D2/D3/D5"*.

**Read from that list alone, this sizes as `L` and that is wrong.**

[[RT-FNSPLIT-B2R]] is **`merged`** (PR #967). Its scope section *"Inert only —
the already-ruled scaffold escape"* landed the closure-conversion **contract**
as production code while banning any executable edge:

- declarative ABI/layout/ownership types, descriptor construction and the
  **validators may be production code** — and are;
- production retains exactly one root `FunctionBuilder` and one
  `define_function`, with **zero** new callable target unit, call edge,
  dispatch edge, callback, flag or alternate entry;
- no encoder/decoder or helper creating a second live body-emission authority.

The validator that **rejects missing capture slots, extra capture slots and
mismatched slots** is already on `main`. So are the typed frame slots for free
variables.

So **this node activates merged substrate for a population that did not exist
until `D4a` admitted it.** It does not invent closure conversion.

**Therefore `D1` is an inventory, and it is the sizing instrument**: measure,
at the base, exactly what `B2R` left inert, which of the five mechanism
elements already exist as production types/validators, and which genuinely need
an executable edge. Do not size the node before that answer exists — and note
that `B2F`, `C3-ACTIVATION` and `RT-NATIVE-FNSPLIT` are **also all merged**, so
the live-boundary work `B2R` deferred to `B2F` may already be landed too.
`B2R`'s own anchors section warns every anchor in that chain has moved at least
once. Re-derive; do not read the numbers.

**Measured at `origin/main` `5e36e193` and carried into the frame:**
`CaptureSlot` is `{ ordinal: u32 }` — the merged substrate carries counts,
ordinals and layouts, **not free-variable identities**. Confirm or refute that
at the real base; it is the difference between a small node and a large one.

## Four repairs that are forbidden, and why

From the Architect's disposition. Each fabricates a binding and violates the
existing no-implicit-tail ABI law:

| forbidden | what it fakes |
|---|---|
| pad the environment vector | a value that was never passed |
| shift the `Var` | a different variable than the source names |
| copy an ambient caller tail | an implicit suffix the ABI law forbids |
| reuse continuation-call inputs as implicit unit captures | conflates the continuation contract with the unit's |

There is no relax option. That is why this is a node rather than a waiver.

## Why a node and not a checkpoint

Preference order is relax, fold, then cut
(`agent/playbooks/federation/steward.md` §4).

- **Relax** — unavailable, per the table above; and this is a genuine
  correctness failure (emitted code carrying a value the body did not name),
  **not** a would-`main`-go-red concern.
- **Fold** — **no open sibling exists.** Checked at file time, not assumed:
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

That gate is recorded in `RT-CONTSRC-PRODUCER-LOCAL`'s frame **as prose, not as
a `depends_on` edge**, because an edge both ways is a cycle the generator
cannot resolve. Same pattern as `RT-CONTSRC-PRODUCER-LOCAL` itself against
`RT-DECL-CLOSURE-PORT`: both `active`, one branch, sequenced by the frame.

## ⛔ CLOSED 2026-08-05 — the premise was FALSE. Read this before anything below.

**`closed` means resolved-without-landing.** Nothing here was built, and
**nothing below this section should be built.** Everything from "The gap,
exactly" onward is retained as the record of a premise that measurement
retired — ⛔ **it is history, not a specification.** Nothing depends on this
node (`blocks: []`, and no other node's `depends_on` names it), so closing it
strands no work.

**What three measurement passes established**, on
`wp/RT-DECL-CLOSURE-PORT-typed-units` (records at
`docs/program/wp/RT-UNIT-CLOSURE-CONVERT-D1{,b,c}.md`):

| pass | finding |
|---|---|
| `D1` `bc754c03` | the runtime closure-conversion substrate is **complete** — all five `RT-FNSPLIT-B2R` elements present, none a stub, live for 127 closures |
| `D1b` `a8b66c5c` | production's capture basis is `(0..runtime_depth).map(Var)` — **positional and total by construction**; nothing inspects a body |
| `D1c` `e27d297a` | the five failing units **never reach that path**: zero records across the whole `ken-runtime` **and** `ken-elaborator` lib suites. The empty list is a **fixture literal** in `test_objects.rs` |

**Architect ruling `evt_5g7kaec1xzaf6` then settled the contract:**
`LexicalClosure.captures` must be **total** for its body's ambient lexical
demand, with no lawful undeclared caller tail. ⇒ The two fixtures are
**malformed**; there is **no closure-conversion substrate gap** and this node
had nothing to activate.

⭐ **The residual work — correcting those two fixtures — is
[[RT-CONTSRC-PRODUCER-LOCAL]] `D5`**, folded there because it is small, in the
same crate, on the same branch, and is that node's own candidate gate. ⛔ Do not
re-open this node to hold it.

⛔ **Do not resurrect the title's mechanism.** "Free de Bruijn references become
declared typed capture slots" describes work the ruling says must **not** be
done — no `CaptureSlot` identity field, no synthesized capture, no padding or
`Var` shifting, no caller tail.

## Superseded: Status `active` — this node does NOT wait for a merge

**Released `D1` only, 2026-08-05, from exact `b3ba2820` on
`wp/RT-DECL-CLOSURE-PORT-typed-units`** (Steward sequencing ruling; Architect
disposition `evt_gqph7jhjeybx` discharging `RT-CONTSRC-PRODUCER-LOCAL` `D4b`
and referring the release boundary here).

⛔ **This section previously said the node "enters the frontier when
`RT-CONTSRC-PRODUCER-LOCAL` merges, with no Steward pass in between." That was
false, and read literally it was a deadlock** — that node's candidate cannot
close until this one lands (the section above, and its own frame), so waiting
for its merge means waiting forever. The sentence was the standing
one-release-ahead phrasing applied to a pair it does not govern.

**The two nodes are one atomic set on one branch, not a sequence.** They land
in one candidate, and at that merge **both flip `merged` in one commit**. The
`depends_on` edge is retained because this node genuinely builds **on**
`RT-CONTSRC-PRODUCER-LOCAL`'s landed checkpoints — it states checkpoint order,
which is true, and **not** merge order, which is not. `status: active` is what
keeps this node off the releasable frontier; `gen-progress.sh` computes that
frontier as `ready` **and** every `depends_on` entry merged, so a stale `ready`
here would advertise in-flight work as available to release.

This is the same shape as `RT-CONTSRC-PRODUCER-LOCAL` against
`RT-DECL-CLOSURE-PORT`, live in this corpus today: `active`, with an unmerged
`depends_on`, building on the shared branch.

## What must not be lost

Runtime **preserves `bc371f13` exactly** and makes **no unit-frame edit**
meanwhile (Architect, same disposition). The `D3b` arms, crossed-pair refusals
and consumer mutations are **directionally accepted evidence** — QA approved
their fidelity to the current frame. That is not the same as `D3b` being
complete, and the crossed-pair law's own premise is under measurement in
`RT-CONTSRC-PRODUCER-LOCAL`'s `D3c` `EntryAbi` checkpoint.
