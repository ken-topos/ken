# `RT-CONTSRC-PRODUCER-LOCAL` `D3c` — the `EntryAbi` availability measurement

Released by `evt_2f30t4866ensx` (Runtime leader), on the Architect's
disposition `evt_56jh63qntwtfe` and the Steward's scope recut
`evt_7he9qv8wbv1yq`. Built over preserved `bc371f13`.

Commits: `3a9841c2` (the observatory and the divergence half) → `33e210c5`
(the zero-depth agreement half) → `09921479` (hoisting the oracle's positive
control).

**No production edit.** The projection's `RootIsImmediate` arm still copies
`source_abi_position` into `immediate_slot`, and the emission seam still reads
`producer_env` at that slot. Nothing in this checkpoint changes a lowering or
planning decision; every line added is `#[cfg(test)]`.

`D3b`'s re-cut, `D4b`, the candidate, QA of the node, `D6` closure, `AC-4`,
`#27`/case-emission, the call-result SCC and downstream `D7` remain held.

## The result, first

**The position moves.** At a real predeclared emission seat under one
intervening binder, an entry-ABI value's root ABI position is **not** its
immediate position, and reading `producer_env` there yields a different value
of the identical lowering shape.

```
source_abi_position          = 0
defining_abi_operands[0]     = specialized-scalar(v15)   <- the entry oracle
producer_env                 = [ specialized-scalar(v44),     <- index 0
                                 specialized-scalar(v15),     <- index 1
                                 specialized-scalar(v21) ]
measured immediate position  = 1
```

Production reads `producer_env[0]`. That slot holds `v44` — the producer-local
host-effect result the intervening binder pushed — where the entry parameter
`v15` belongs. `v15` is still present, exactly once, one position further in.

The emission environment is 3 long against an entry ABI run of 2. The
displacement is the binder.

## Why this is the silent class, not a crash

Everything a consumer could check at that seam agrees:

- **the index is in bounds** — `0 < 3`, so no bounds guard fires;
- **the shape is identical** — both are `specialized-scalar`, so no contract or
  carrier check can separate them;
- **`D3b`'s own consistency law passes** — it requires
  `immediate_slot == source_abi_position`, and they are both `0`.

So the seam does not refuse. It emits a call carrying a well-formed operand of
exactly the right contract and **the wrong value**. That is the failure class
the checkpoint text names when it rejects Option 3, and it is the reason the
Architect put `D3b`'s premise in doubt rather than its fidelity.

## The four conditions, each discharged

The measurement is
`d3c_an_entry_abi_root_position_is_not_the_immediate_position_under_a_binder`
in `lowering/core/tests/control.rs`.

**1. One real predeclared emission under an intervening binder.**
`governed_nested_resource_bracket(3)` compiled through
`recursive_port_process_compiles` — the production entry, and an **existing**
production planner population that five landed controls already compile.
⛔ No fixture is authored for this checkpoint. The seat's emission environment
is longer than its entry ABI operand run, which is what "under a binder" means
observationally, and the selection is on that property rather than on an
ordinal.

**2. Both root domains in the same required vector.** The seat's vector is
`[ProducerLocal, EntryAbi]` — one input of each. Recorded as counts on the
observation and asserted, so a population that lost one domain would fail the
selection loudly rather than quietly measuring a single-domain seat.

**3. An independent lowering-side oracle.** Production already records the
entry ABI operands in ABI-position order at unit entry: `D5a`'s
`defining_abi_operands` is built from the *same single slot walk* that seeds
the entry environment, so "index `i` is ABI position `i`" holds there by
construction rather than by two walks agreeing. It is keyed by ABI position and
never by an environment index.

⇒ Comparing it against `producer_env[source_abi_position]` compares **two
independently derived answers to "which value is this"**. ⛔ No planner re-walk,
no index arithmetic, no fixture-authored expected index, no direct
construction. The identity reported is the Cranelift SSA `Value`, for `D4a`'s
reason: carrier, phase and lowering shape all agree between the entry parameter
and the local binding that displaces it.

**4. Substituting `source_abi_position` flips, on identity.**
`D3cPositionSelection::SourceAbiPosition` moves the position the instrument
reads from the measured one to the root ABI position; the observed operand
changes from `v15` to `v44`. The control separately proves the flip is not an
artefact: the root position is asserted **in bounds** before the comparison,
and the two operands are asserted to share a lowering shape after it.

## The discriminating half — zero binder depth AGREES

⭐ The divergence alone would be equally consistent with an oracle that never
lines up, which would establish nothing. So the same armed window also compiles
the `D5a` `px8tr_nested_post_effect` witness, which reaches predeclared
emission seats at **zero** binder depth and compiles **green**.

At those seats the emission environment holds the entry ABI operand at its own
root position, position for position. Two such rows were observed against the
one shifted row, and the control refuses to proceed if the zero-depth set is
empty.

⇒ The entry oracle is **correct wherever the projection's assumption holds, and
divergent exactly where a binder has been pushed.** This is the Architect's
reading confirmed as measured rather than inferred: the two coincide only at
zero binder depth, and every population before `D4a` was at zero binder depth,
which is why nothing ever had to tell them apart.

## Mutations — six, each red at its own line

Run against the committed tree and reverted byte-clean (`git diff` empty after
each).

| mutation | proves | attributed to |
|---|---|---|
| the entry oracle reads the emission environment instead of the entry ABI walk | the oracle's **independence** is load-bearing; collapse the two answers into one and the divergence vanishes | the core `assert_ne!` measurement |
| the emission environment is recorded as the entry ABI run | the environment observed really is the **emission seat's**, not the entry run | the population selection — no shifted seat exists |
| the substitution arm is made the identity | the `SourceAbiPosition` mutation is **not** a second spelling of the exact route | condition 4's position assertion |
| the entry oracle is starved (index out of range) | the "recorded nothing" positive control is **live** | the hoisted oracle control |
| the shifted population is not compiled | the divergence comes from the bracket population, not from the witness | the population selection |
| the operand read ignores the selection | condition 4's **operand-identity** half is live independently of its position half | condition 4's operand assertion |

⭐ **The starved-oracle mutation initially fired at the zero-depth agreement
assertion, not at the guard written for it.** That guard was sitting after both
halves and was therefore not load-bearing. It was hoisted to lead, and to cover
every observed row rather than the shifted one alone; the mutation then
attributed correctly. Recorded because a positive control that a mutation
reaches only by accident is not a proven one.

## Promise class

**Durable invariant.** The control asserts a *relation* between two
independently derived answers to "which value is this", and pins no literal
index, count or SSA word. If a later checkpoint corrects the representation so
that the two agree, this control is the thing that must be re-cut deliberately,
and its failure is the correction announcing itself.

## MEASURED / CLAIMED / THE GAP

**MEASURED.** Compiling `governed_nested_resource_bracket(3)` and the
`px8tr_nested_post_effect` witness through the production planner and lowering
path yields three entry-ABI continuation inputs at predeclared emission seats.
At the one seat whose vector holds both root domains and whose environment
exceeds its entry ABI run, the operand production's own entry walk recorded for
ABI position 0 is not the operand the emission environment holds at index 0; it
is present exactly once, at index 1; and the operand at index 0 is in bounds
and of the identical lowering shape. At the two seats where the environment
equals the entry ABI run, the two agree position for position.

**CLAIMED.** The `RootIsImmediate` copy of `source_abi_position` into
`immediate_slot` is unsound at nonzero lexical depth, and `D3b`'s
three-lawful/three-crossed pairing law rests on it. A predeclared emitter
reading `producer_env[source_abi_position]` under a binder obtains a different
value than the one the coordinate names.

**THE GAP.** This measures **one** population at **one** depth (a shift of
exactly one binder), and it does not measure whether any *currently accepted*
program reaches this seat — the population it was found in already fails
downstream, at the unit-body environment boundary recorded at `D3b` and now
carried by [[RT-UNIT-CLOSURE-CONVERT]]. So the defect is proved **reachable in
the planner and lowering planes**, and is **not** shown to be reachable in a
program that compiles green today. ⛔ Whether that distinction bounds the
severity is a scope question, not mine to answer here.

It also says nothing about how the corrected representation should be spelled.
Per the checkpoint, the correction is structural — immediate availability is
orthogonal to root provenance, and entry-frame availability is lawful only
where the immediate environment really is the entry frame. ⛔ No numeric
equality, constant offset, padding, reverse search or fallback may bridge the
domains, and none is attempted here.

## Suite

`ken-runtime` lib: **726 passed / 7 failed / 1 ignored** — `bc371f13`'s seven
reds unchanged (the two standing `D0` reds plus the five former `D4a` reds at
their downstream `Var: no runtime binding` boundary), plus this control. No
regression. The workspace build, the `--locked` gate and conformance are CI's.
