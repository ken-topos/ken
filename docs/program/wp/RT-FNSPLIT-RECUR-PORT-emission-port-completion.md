# `RT-FNSPLIT-RECUR-PORT` — emission-port completion for the governed bracket family

**Owner:** Runtime ring · **Size:** L
**Depends on:** `RT-FNSPLIT-B2F` (merged) · **Blocks:** `RT-SCALE-B`

> ## ⛔⛔ READ BEFORE SLICING — node carries the contract, this the recipe
>
> **Node:** `docs/program/issues/RT-FNSPLIT-RECUR-PORT.md` — objective,
> deliverables `D1`–`D5`, acceptance criteria `AC-1`–`AC-6`, the bookkeeping, and
> the sizing question. ⛔ **The ACs are there and they bind.** This file is the
> executable half: slicing order, control recipes, and the traps.
>
> ⚠ **Both artifacts carry the two hard prohibitions below**, deliberately
> duplicated. A rule recorded only in the node does not fire, because the frame
> is what an implementer slices from.

> ## ⛔ PROHIBITION 1 — DO NOT MAKE THE FAMILY PASS BY CHANGING THE FAMILY
>
> The discriminator is `planning/static_transition.rs::nested_resource_bracket`,
> **unmodified**: a recursive `ComputationalMatch` with `recursive_positions:
> vec![0]` and trap arms.
>
> **Architect, `evt_14eq3v2g0v1hm`:** *"A family made functionizable merely by
> deleting recursion or traps would be a different benchmark and could not answer
> this frame's question."*
>
> ⇒ ⛔ **Deleting recursion, dropping trap arms, or substituting a non-bracket
> synthetic does not discharge `AC-1`.** ⭐ The control must run the **same
> governed source** the scaling gate names.

> ## ⛔ PROHIBITION 2 — `D3` NARROWS THE CONDITION; IT MUST NOT INVERT THE DEFAULT
>
> `select_body_emission_authority` is closed, exhaustive and **fail-closed**, and
> that is `B2F`'s landed property. ⛔ **An unhandled or unknown source shape must
> still select the retained authority.**
>
> ⚠ **The cheap error here is a one-line inversion**: turning
> `requires_recursive_descent_authority` from *"true for these conditions"* into
> *"false unless these conditions"*. That reads as a narrowing and is a
> **default-flip** — it would admit every future unhandled shape into
> functionized emission silently.
>
> ⇒ `AC-4` needs a **positive control**: a source shape that no arm handles must
> be observed selecting `RecursiveDescent`. ⛔ A test that only checks the
> handled shapes cannot see this.

## ⭐ SLICE 0 — MEASURE THE SIZING INPUT FIRST, and report it

> **Does the `PX8-ERRID-ALLOC` failing fixture route through `RecursiveDescent`?**

⛔ **Unmeasured, and it is not being assumed in either direction.** Both the
fixture and the selector are on `main`, so this is cheap.

- **If yes** — the retained authority is on the critical path to the Cranelift
  wall; `D1`/`D2` are load-bearing for `PX8` itself, not only for the
  measurement. Size holds at `L`.
- **If no** — the wall may already be cleared for what `PX8` needs, and this node
  may reduce to the narrower slice `RT-SCALE-B` requires. ⭐ **Raise that to the
  Steward as a re-scope**; do not silently build the larger thing.

⚠ **Report the result either way, in the channel, before designing `D1`.** ⛔ Do
not carry it to a retro — it is a sizing input, not a lesson.

## Slicing order

| slice | deliverable | why this order |
|---|---|---|
| `S0` | the measurement above | it can re-scope everything after it |
| `S1` | `D1` — recursive positions as declared unit calls | the growth axis; everything else is smaller |
| `S2` | `D2` — trap arms | independent condition in `requires_recursive_descent_authority`; can red separately |
| `S3` | `D3` — narrow the selector, with the fail-closed default proven intact | ⛔ **after** `S1`/`S2`, so each removed condition has a working port behind it |
| `S4` | `D4` — the governed family selects `FunctionizedUnits` at every `n` in `3..7` | the node's stated exit |
| `S5` | `D5` — re-state the remaining retained residual, closed and explicit | leaves the next reader a true statement of what is still unported |

⛔ **Do not fold `S3` forward.** Narrowing the selector before the port exists
admits a shape the emitter cannot yet handle — that is a fail-open, and it is the
same defect class `B2F`'s control 4 was blocked on.

## Control recipes

- **`AC-2` (the load-bearing one).** A recursive position must lower as a
  **declared unit call**. ⭐ The mutation that proves it: make one recursive
  position re-lower inline into the caller's body. ⛔ **That mutation must red**,
  and it must red at an assertion about *call structure*, not about output value
  — an inlined re-lowering computes the same answer, which is exactly why a
  value-only assertion is blind to it.
- **`AC-3`.** Two bodies, identical except one carries a `Trap` arm. ⭐ **Both**
  must functionize. ⛔ A control that only exercises the trap-carrying body
  cannot distinguish "traps are now supported" from "traps were never the
  blocker."
- **`AC-5`.** `B2F`'s inventory controls must pass **unmodified** — two-pointer
  ABI, two-field services record, two-field call-frame envelope, three-field root
  ingress consumed only by the public adapter, role-keyed static ingress. ⛔ If a
  repair here requires touching them, that is a hard stop, not a judgment call.
- **`AC-6` — the real exit.** ⭐ The point is **not** that the selector flipped;
  it is that `RT-SCALE-B`'s harness can now collect every `D2` metric for every
  `n` in `3..7` on this family. Demonstrate the collection, not just the
  selection.

## ⚠ Traps carried from this chain, measured not forecast

- **A negative control passes for any reason.** `B2F`'s control 4 was green while
  accepting the exact evasion it existed to detect. ⛔ Every control here that
  asserts *"X cannot happen"* needs a paired positive control proving it **reds
  on the real path**.
- **`/tmp` leak, live.** ~700 MB per full `-p ken-runtime` run; `temp_output_dir`
  never cleans up. ⛔ It surfaces as an unrelated `No space left on device` —
  triage on the **error production raised**, never on the test names.
- ⛔ **Targeted `scripts/ken-cargo` only — never `--workspace`.** Workspace-green
  and `--locked` mean **green in CI** (`agent/COORDINATION.md` §12).

## ⭐ `D4` of `RT-SCALE-B` is already landed and is an input, not a blocker

`docs/program/rt-scale-b-d4-analytical-model.md` (Architect) records the
**conditional** analytical model and states this port-incomplete gate explicitly.
⭐ It predicts an achievable affine emission model for strict LIFO nesting.

⛔ **It is not a verdict and must not be treated as one** — the classification of
bad-constants-on-`O(n)` versus residual super-linearity is *unreachable* until
this node lands and `RT-SCALE-B` runs. ⚠ **Runtime's future measured table is
evidence against that model, never its source.**

## Standing

⛔ Stop for a genuine frame-unsatisfied hard stop or a clean merge-ready handoff.
**Hard-stop count of record = 13**; the next armed research pull is **`#15`**, so
a stop here fires none. The authoritative counter is
`docs/program/issues/RT-NATIVE-FNSPLIT.md`'s **ARMED §5a RESEARCH-CONSULT
TRIGGER** line, which wins on any disagreement — ⛔ read it at the point of a
stop, never a count transcribed into a frame.
