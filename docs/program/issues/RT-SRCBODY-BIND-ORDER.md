---
id: RT-SRCBODY-BIND-ORDER
title: "Functionized source-body units install the parameter run in ABI order where the body reads de Bruijn-nearest-first, so every multi-parameter source body binds its parameters permuted"
status: merged
owner: runtime
size: M
gate: none
depends_on: [RT-CONTSRC-PRODUCER-LOCAL]
blocks: []
github: null
origin: Architect mechanism ruling evt_7yfs6qxp9hm5b (2026-08-06), on the RT-ENTRY-TRAP-254 D0-D9 diagnosis chain. Supersedes RT-ENTRY-TRAP-254, whose failing row is one discriminator for this defect. Steward-filed (agents cannot create tracked work per COORDINATION §2).
---

> ## THE DEFECT, EXACTLY — ABI order was used as de Bruijn lexical order
>
> **Architect `evt_7yfs6qxp9hm5b`.** Frame:
> `docs/program/wp/RT-SRCBODY-BIND-ORDER.md`.
>
> The contract is already stated and is not in dispute:
>
> - the process adapter constructs the declaration call in **source argument
>   order** `[ProcessInput, ProgramCaps]` (`object_linker_packaging.rs:797-809`);
> - the declaration call centralizes `inputs = arguments in parameter order ++
>   captures in D3 order` (`core.rs:14898-14976`);
> - the ABI descriptor is parameter ordinal 0, parameter ordinal 1, then captures
>   (`planning/static_transition/abi.rs:1551-1585`);
> - **but a declaration body environment is de Bruijn-NEAREST-FIRST**, and the
>   recursive-descent implementation states and implements exactly that: reverse
>   source arguments, then append captures (`core.rs:14705-14714`).
>
> **The functionized unit entry violates that last conversion.** At
> `lowering/units.rs:3701-3790`, one slot-order walk does two jobs and only one
> of them is right: it records `defining_abi_operands` in ABI position order
> (**correct**), and pushes the same operands into `env` in that same order
> (**incorrect**).
>
> For the observed fixture the root adapter passes `[input, caps]` correctly;
> the functionized `main(input, caps)` installs `env = [input, caps]` while the
> erased body names `input` as `Var(1)` and `caps` as `Var(0)`. **So `Var(1)`
> reads `ProgramCaps`**, and the later unary call then correctly places that
> wrong value in `process_discriminator` parameter slot 0.
>
> **This explains every `D9` fact without an unpaired call argument.**

> ## THE SEAM `D9` NAMED WAS THE WRONG ONE. Do not repair there.
>
> `D9` attributed selection to the common transfer coordinate. **The Architect
> refuted that**, and the refutation is load-bearing because the obvious repair
> follows from the wrong attribution:
>
> `carry_source_call_inputs` (`mod.rs:5837-5860`) iterates the **already-ordered**
> input vector and calls `carry_call_input` once per member; that returns a
> `Carried` member unchanged and transfers a `Specialized` one. **It cannot
> select a sibling or change vector position.** `call_declared_unit_target`
> (`mod.rs:6128+`) then walks the descriptor's `Parameter | Capture` slots
> consuming `inputs[0]`, `inputs[1]`, ... in exact order — **that is already
> positional argument-to-slot pairing.** The wrapper at `mod.rs:5958-5978` only
> resolves a target and delegates an already-assembled slice; **line 5975 is not
> an argument-selection operation.**
>
> **And a carried word bypasses `transfer_into_carrier` entirely**, so adding a
> caller occurrence to the common transfer coordinate **cannot change which
> carried word occupies slot 0**.
>
> ⇒ **BANNED: per-argument transfer coordinates.** They are a
> provenance/ownership design change **and they would leave this defect
> intact.** The existing common-coordinate rule stands; its aggregate
> self-authorization argument was about **ownership**, not positional binding.

> ## BLAST RADIUS — and AGGREGATE-NESS IS NOT CAUSAL
>
> **Broader than one aggregate row, and not "every aggregate through
> `call_declared_unit_target`."** Both of those framings are wrong, and the
> second was the Steward's.
>
> **The affected class:** every **activated non-root functionized source-body
> unit with at least two parameters whose body distinguishes parameter
> positions**. Its parameter run is installed in the wrong semantic order.
>
> Consequences can surface for **integers, booleans, capabilities, borrowed
> handles, constructors, or any mixture** — the class is decided by **arity and
> positional use**, not by representation.
>
> **What MASKS it:** unary units are invariant under reversal; unused
> parameters, equal values, and same-representation uses all hide it.
>
> **What does NOT bound it:** the measured 97 specialized `Constructor`
> parameters from `call_static_worker` are real aggregate traffic and they
> self-authorize as their comment says — **that census does not prove those 97
> calls are misdelivered, and the measured absence of aggregate specialized
> captures says nothing about parameter order.** Do not cite either as a bound.
>
> **The operator-facing statement, verbatim from the Architect:**
>
> > activation exposed a general multi-parameter source-body binding
> > permutation; the skipped `ProcessInput` row is one discriminator for it. The
> > defect is not an aggregate-ownership failure and is not confined by the
> > prior nonaggregate ownership census.
>
> **It does not alter the prior publish ruling by itself**, but it is
> **materially larger in logical scope** than the single observed row.

## Why this is a node and not a fold

`RT-ENTRY-TRAP-254` is a node about **one failing row**. This defect is a
**general binding permutation** for which that row is one discriminator. Keeping
the repair under a node whose title names a single test would understate the
scope to every future reader, and the four required controls are mostly not
about that row at all.

⇒ **`RT-ENTRY-TRAP-254` is superseded and closed.** Its `D0`-`D9` chain is the
evidence that produced this node, and **its skipped row moves here** as control
2 of 4.

## This is a BUG FIX, not a design change

Architect, explicitly: *"That is a bug fix restoring an already-stated binding
contract, not a design change."* The contract at `core.rs:14705-14714` already
says reverse-then-append; one code path does not do it. **Do not frame, size, or
review this as a mechanism change** — and do not let its breadth be mistaken for
one.
