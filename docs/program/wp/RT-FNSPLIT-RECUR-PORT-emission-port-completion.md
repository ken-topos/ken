# `RT-FNSPLIT-RECUR-PORT` — emission-port completion for the governed bracket family

**Owner:** Runtime ring · **Size:** XL (was `L`; `D6`+`D7` added by ruling `#14`)
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
> ### ⚠⚠ AMENDED BY RULING `#14` (`evt_3629v1gy7fwqq`) — READ THIS FIRST
>
> ⭐ **"Unmodified" now means the CORRECTED canonical helper**, spelled out as
> `D7` in the node. ⛔ **The malformed one-operand `BufferFreeze(Var(0))` helper
> is RETIRED** — it was a planning-only raw `RuntimeExpr` that could not state
> its own named contract, so it was never a well-formed instance of this
> benchmark. Reproducing it is now the violation.
>
> ⛔ **Everything else below stands unchanged and still binds:** deleting
> recursion, dropping trap arms, or substituting a non-bracket synthetic does
> not discharge `AC-1`. ⚠ `recursive_positions = [0]`, **every** trap arm, the
> `n=3..7` family, and LIFO bracket behavior are all still load-bearing.
>
> ⇒ ⛔ **`AC-7` is the control that keeps this honest** — a structural fixture
> pinning the corrected shape. Without it, "correcting the family" and
> "substituting a different benchmark" are indistinguishable from the outside.
>
> ---
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

## ✅ SLICE 0 — DONE. Affirmative.

> **Does the `PX8-ERRID-ALLOC` failing fixture route through `RecursiveDescent`?**
> ✅ **YES** — `authority=RecursiveDescent function=ken_nc23_entrypoint`,
> measured on clean `b6bac1a8` with an environment-gated probe placed right
> after `select_body_emission_authority`, then reversed (`git diff --quiet`
> passes, HEAD unmoved). ⭐ That is the right shape for a measurement slice.

⇒ **The retained authority is on the critical path to the Cranelift wall;
`D1`/`D2` are load-bearing for `PX8` itself, not only for the measurement.**
⛔ **No re-scope.** The later re-size to `XL` came from ruling `#14` adding two
deliverables, ⚠ **not** from this measurement.

## ✅ SLICE 1 — DONE, and it is the win this chain exists for

At depth 3 the governed source selects `FunctionizedUnits`, the recursive
lexical-closure position resolves to its planner-declared body unit and emits a
**direct unit call**, and re-entry closes as a **static-origin-keyed CFG
backedge** rather than an inline re-lowering. ⇒ **`AC-2`'s growth property is
demonstrated.**

⚠⚠ **But `D7` rewrites the family that result was measured on.** ⛔ `AC-2` is
now a **preservation** obligation: re-run the inline-re-lowering mutation
against the **corrected** source. ⛔ **Do not carry the old green forward.**

## Slicing order

⚠ **The ids are deliverable-derived, not sequence numbers.** `S6`/`S7` were
added by ruling `#14` and **execute next — before `S2` resumes** — because the
family they correct is the input to every remaining slice.

| slice | deliverable | why this order |
|---|---|---|
| ✅ `S0` | the measurement above | it could re-scope everything after it |
| ✅ `S1` | `D1` — recursive positions as declared unit calls | the growth axis; everything else is smaller |
| ▶ `S6` | `D7` — replace the malformed helper with the corrected four-seat canonical family | ⛔ **first of the remainder.** `D6`'s admission rule is stated per *seat 0 / seat 3*, and those seats do not exist until this lands |
| ▶ `S7` | `D6` — the narrow carried resource-token seat in `lower_process_host_effect` | unblocks compilation past `BoundaryCarrier`; without it `AC-1` has no complete bundle and `AC-6` has nothing to collect |
| `S2` | `D2` — trap arms | independent condition in `requires_recursive_descent_authority`; can red separately |
| `S3` | `D3` — narrow the selector, with the fail-closed default proven intact | ⛔ **after** the ports exist, so each removed condition has a working port behind it |
| `S4` | `D4` — the governed family selects `FunctionizedUnits` at every `n` in `3..7` | the node's stated exit |
| `S5` | `D5` — re-state the remaining retained residual, closed and explicit | leaves the next reader a true statement of what is still unported |

⛔ **Do not fold `S3` forward.** Narrowing the selector before the port exists
admits a shape the emitter cannot yet handle — that is a fail-open, and it is the
same defect class `B2F`'s control 4 was blocked on.

⛔ **Do not fold `S7` into `S6`.** They red on different things: `S6` is a
source-shape correction provable by a structural fixture with no lowering
involved, `S7` is a lowering admission rule. Landing them as one commit makes
`AC-7` and `AC-8` indistinguishable if the result is red.

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

### Added by ruling `#14` — the recipes for `S6`/`S7`

- **`AC-7` — a STRUCTURAL fixture, no lowering.** Pin, on the corrected source
  tree itself: the IH argument **is** the allocation result · exactly **four**
  `BufferFreeze` operands · seats 0 and 3 are the **same closure parameter** ·
  seats 1 and 2 are literals `0` and `1` · recursion and **every** trap arm
  retained. ⛔⛔ **Build it from semantic binder roles and then audit the
  generated indices — do NOT copy the raw `Var` indices out of the ruling.**
  ⚠ Copying a guessed index is precisely how `BufferFreeze(Var(0))` got here.
- **`AC-8` — the mutation that proves the port is live.** Force **either**
  resource seat back to specialized-only; ⛔ **it must red on the governed
  carried route.** ⚠ A control that only shows the carried route working cannot
  tell a live port from a path that never needed one — the `B2F` control-4
  failure class again.
- **`AC-9` — fail-closed on the new seat, three ways.** A carried
  **wrong-class / non-`BorrowedOpaque`** value must fail ⭐ **before any host
  request is issued**; a **carried `start`** must fail closed; a **carried
  `length`** must fail closed. ⛔ One of the three passing does not cover the
  other two.
- **`AC-10` — provenance stays real.** The existing `PX8-SPAN-PROV`
  same-shape / two-buffer discriminator must stay green, and substituting a
  **distinct span-origin token must red**. ⛔ **The encoder must not derive seat
  3 from seat 0** — if it does, seat 3 is decoration and this discriminator is
  vacuous no matter what colour it reports.
- ⛔ **The license is a closed list.** ⛔ Do not treat every `Carried` host
  operand as a scalar · do not reconstruct or fabricate `Lowered::ResourceToken`
  · do not add a `Lowered` variant containing a carrier · do not mint a new
  carrier tag, class, identity, ABI field, service, envelope field, or ingress
  lane · do not synthesize bounds or provenance in the wire encoder · do not
  widen **any other** host operation without an independently demonstrated
  carried seat. ⚠ **Any of these is a hard stop, not a judgment call** — and
  `#15` is armed.

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
**Hard-stop count of record = 15**, and ⚠⚠ **`#15` HAS FIRED — the armed
research consult is OPEN; the next armed multiple is `#18`.** ⛔ **`S6`/`S7` are
paused pending the Architect's `#15` ruling**, which cannot issue until entry 6
answers the shared-predicate check. The authoritative counter is the
**`COUNT OF RECORD` block at the head of `§5a`** in
`docs/program/issues/RT-NATIVE-FNSPLIT.md`, which wins on any disagreement —
⛔ read it at the point of a stop, never a count transcribed into a frame,
**including this one**.
