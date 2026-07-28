---
id: RT-FNSPLIT-RECUR-PORT
title: "emission-port completion — the governed nested-bracket family (recursive ComputationalMatch + trap arms) must select FunctionizedUnits, so RT-SCALE-B can measure the completed population"
status: ready
owner: runtime
size: L
gate: none
depends_on: [RT-FNSPLIT-B2F]
blocks: [RT-SCALE-B]
github: null
origin: RT-SCALE-B frame-unsatisfied hard stop #13, raised by runtime-implementer (evt_1wszmnre17w1e) and ruled structurally by the Architect (evt_14eq3v2g0v1hm, stop order evt_4yg9kzq46sn4b). Escalated to the Steward by runtime-leader (evt_3fmxnax486b52) because the repair is outside RT-SCALE-B and RT-FNSPLIT-B2F is merged. Steward-filed 2026-07-28 (agents cannot create tracked work per COORDINATION §2); Steward ruling evt_37fwa49tk6dhj.
---

> ## ⛔⛔ WHY THIS NODE EXISTS — `RT-SCALE-B` CANNOT LAWFULLY MEASURE WITHOUT IT
>
> **`RT-SCALE-B` is the node that decides whether the whole `RT-NATIVE-FNSPLIT`
> effort worked.** It is paused clean at `466a9fa7` with no edits, and it stays
> paused until this lands.
>
> **The determination is structural, not empirical** (Architect
> `evt_14eq3v2g0v1hm`). On `origin/main = 466a9fa7`:
>
> 1. `planning/static_transition.rs::nested_resource_bracket` builds each bracket
>    as a `RuntimeExpr::ComputationalMatch` with `recursive_positions: vec![0]`,
>    carrying trap arms.
> 2. `lowering/core.rs::requires_recursive_descent_authority` returns true for any
>    non-empty `recursive_positions`, **and independently** for any
>    `RuntimeExpr::Trap` reached in the source tree.
> 3. `select_body_emission_authority` therefore selects
>    `BodyEmissionAuthority::RecursiveDescent`.
> 4. `compile_expr_into_module` declares and defines `UnitBundle` bodies **only**
>    for `FunctionizedUnits`; `RecursiveDescent` executes the retained monolithic
>    `compiler.lower_expr` root instead.
>
> ⇒ ⭐ **`B2F`'s functionized population is never produced for the family the
> scaling gate names.** Both remaining options are unlawful as `D1`/`D2`:
> reporting retained-authority metrics as the *completed representation* is a
> **fabricated population**, and reporting missing functionized metrics as
> `could_not_determine` is an **`AC-B1` failure**. ✅ Runtime refused both.

> ## ⛔ `RT-FNSPLIT-B2F` IS NOT DEFECTIVE — DO NOT REOPEN IT
>
> **The residual was approved as part of `B2F`.** Decision `dec_5y5ekba06d1t7`
> says so in terms:
>
> > *"the closed `RecursiveDescent` residual covers recursive computational
> > matches, `Match`-with-`Call` scrutinees, traps, seed `Closure` calls, and
> > top-level transparent closure declarations."*
>
> ⇒ **`B2F` landed exactly its approved scope.** The gap is that `RT-SCALE-B`'s
> prescribed discriminator family lives **inside that approved residual** — two
> nodes each individually correct and **jointly inconsistent**.
>
> ⚠ **That inconsistency is the Steward's**, not the ring's: `RT-SCALE-B` was
> released as shovel-ready without checking its prescribed family against the
> authority `B2F` was scoped to deliver. ⛔ Recorded so no one re-derives it as a
> `B2F` defect and reopens a merged node.

> ## ⛔ THE BENCHMARK IS NOT BEING AMENDED — this was a real fork, decided
>
> `runtime-leader` correctly surfaced two paths (`evt_3fmxnax486b52`): repair the
> port, **or** obtain a scope amendment changing the benchmark. ⭐ **The port
> side is taken.**
>
> **The Architect already ruled the substitution inadequate on the merits:**
>
> > *"A family made functionizable merely by deleting recursion or traps would be
> > a different benchmark and could not answer this frame's question."*
>
> ⭐⭐ **And there is a harder reason than measurement fidelity.** This chain
> exists because `PX8-ERRID-ALLOC` died on `Cranelift backend failure: Code for
> function is too large`. ⇒ **If recursive `ComputationalMatch` and trap arms
> still lower through the retained monolithic root, the wall is not cleared for
> those constructs — regardless of what we choose to measure.** Amending the
> benchmark would move the *thermometer*, ⛔ not the *fever*.
>
> ⚠ **ONE INPUT IS UNMEASURED AND IS NOT BEING ASSERTED:** whether the
> `PX8-ERRID-ALLOC` failing fixture **provably** routes through
> `RecursiveDescent`. ⛔ It has not been measured. ⭐ It is the input that could
> change this node's **size** — see the open question below. It does **not**
> change whether the node is needed, because `RT-SCALE-B` is blocked either way.

> ## ✅ THIS IS THE FRAME'S OWN PRESCRIBED HANDLING, not a Steward invention
>
> `docs/program/wp/RT-SCALE-B-emission-scaling-verdict.md` armed this exact
> symptom at release:
>
> > *"A new entry that reduces to that same predicate means the **port** is
> > incomplete — it routes back to the emission-port work rather than being ruled
> > here, and ⛔ it does **not** become a verdict of super-linearity."*
>
> ⇒ ⭐ **The armed symptom fired and the frame's prescribed route is this node.**

## Objective

Make the **same governed** `nested_resource_bracket` family — recursive
`ComputationalMatch` with trap arms, unmodified — select
`BodyEmissionAuthority::FunctionizedUnits` and produce a complete functionized
unit population, so `RT-SCALE-B` can collect `D1`–`D3`/`D5` evidence on the
representation whose growth the chain exists to bound.

⛔ **Not in scope:** deleting recursion or traps from the family; introducing a
non-bracket synthetic; changing what `RT-SCALE-B` measures; reopening `B2F`'s
landed envelope, ABI, services record, or static ingress declaration.

## Deliverables

| id | deliverable |
|---|---|
| `D1` | functionized emission for `ComputationalMatch` with non-empty `recursive_positions` — recursive positions become declared unit calls, not an inlined re-lowering |
| `D2` | functionized emission for `RuntimeExpr::Trap` arms, so a reached `Trap` no longer forces the whole body to the retained authority |
| `D3` | `requires_recursive_descent_authority` narrowed to what genuinely remains unportable, with each removed condition individually justified — ⛔ **the selector stays closed, exhaustive and fail-closed** |
| `D4` | the governed `nested_resource_bracket` family selects `FunctionizedUnits` at every `n` in `3..7`, with the existing selector control updated to assert the new selection **positively** |
| `D5` | the retained-authority residual re-stated: whatever still selects `RecursiveDescent` after this node, named explicitly and closed |

## Acceptance criteria

- **`AC-1`** — the unmodified `nested_resource_bracket(n)` for every `n` in
  `3..7` selects `FunctionizedUnits` and produces a complete `UnitBundle`.
  ⛔ A control that mutates the family to make it pass does not discharge this.
- **`AC-2`** — recursive positions lower as **declared unit calls**. ⛔ A control
  must red if a recursive position is re-lowered inline into the caller's body,
  since that is the growth this chain exists to bound.
- **`AC-3`** — a `Trap` arm inside an otherwise functionizable body does not by
  itself force `RecursiveDescent`. ⛔ Needs a positive control: the same body
  with and without the trap arm, both functionized.
- **`AC-4`** — the selector remains **closed and fail-closed**: an unhandled or
  unknown source shape still selects the retained authority rather than being
  admitted by default. ⛔ **`D3` narrows the condition; it must not invert the
  default.**
- **`AC-5`** — `B2F`'s landed mechanism is unchanged: two-pointer ABI, two-field
  services record, two-field call-frame envelope, three-field root ingress
  consumed only by the public adapter, role-keyed static ingress. ⛔ Inventory
  controls must still pass unmodified.
- **`AC-6`** — `RT-SCALE-B`'s harness can produce every `D2` metric for every
  `n` in `3..7` on this family. ⭐ **This is the node's real exit condition** —
  the point is not that the selector flipped, but that the measurement is now
  collectable.

## ⛔ Open question that sizes this node — measure it FIRST

> **Does the `PX8-ERRID-ALLOC` failing fixture route through
> `RecursiveDescent`?**
>
> ⛔ **Unmeasured.** ⭐ It changes the shape of the work:
>
> - **If yes** — the retained authority is itself on the critical path to the
>   Cranelift wall, and `D1`/`D2` are load-bearing for `PX8`, not only for the
>   measurement. Size holds at **`L`**.
> - **If no** — the wall may already be cleared for the constructs `PX8` needs,
>   and this node may be reducible to the narrower slice `RT-SCALE-B` requires.
>
> ⇒ ⛔ **Measure this before designing `D1`, and report it.** It is cheap — the
> fixture and the selector are both on `main`. ⚠ **Do not defer it to the
> retro**; it is a sizing input, not a lesson.

## Bookkeeping

- **Hard-stop count of record: 13** (this stop). ⛔ **No research pull fires** —
  the armed multiples are `#15`, `#18`, `#21`. The authoritative counter is
  `docs/program/issues/RT-NATIVE-FNSPLIT.md`'s **ARMED §5a RESEARCH-CONSULT
  TRIGGER** line, which wins on any disagreement.
- **Symptom inventory: `ENTRIES` 3 → 4**, appended by the **Architect**
  (*"appends one line per hard-stop, before it rules"*) — it ruled without
  appending. ⛔ **`NEXT PREDICATE CHECK` stays the 6th entry**; the 3rd was
  consumed, answered at entry 2. ⇒ Entry 4 triggers no predicate answer.
- ⛔ **§5a-ii: the shared predicate is the Architect's to name**, and it named it
  at `evt_55bzwnhjpwjrs`. The Steward does not restate it.

## Standing constraints

⛔ **Targeted `scripts/ken-cargo` only — never `--workspace`.** Workspace-green
and `--locked` mean **green in CI** (`agent/COORDINATION.md` §12).
⚠ The C3 archive leak is live: ~700 MB of `/tmp` per full `-p ken-runtime` run,
surfacing as an unrelated `No space left on device`. ⛔ Triage on the **error
production raised**, never on the test names.
