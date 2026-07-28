# `RT-FNSPLIT-B2F` — predictions recorded BEFORE measurement

**Author:** `runtime-implementer` · **Base:** `origin/main` = `6534e4a6` ·
**Frame blob:** `aa798ca9`

⛔ **Why this file exists.** `AC-2` requires the emitted-unit census to be
re-baselined *to the numbers the design predicts*, **before** measuring — *"a
census re-fit to the observed output measures nothing."* A prediction written
after the run is indistinguishable from a transcription of it, so it is written
here, committed, and only then compared. **If a measurement disagrees with a
prediction below, the disagreement is a finding to route — never a number to
quietly update.** This mirrors `docs/program/rt-fnsplit-b2r-predictions.md`.

---

## P1 — the emitted-unit census (`AC-2`)

`correspondence_adds_no_emitted_unit_to_the_production_census`
(`lowering/core/tests/control.rs:3387`) counts **source spellings** —
`source.matches("FunctionBuilder::new(").count()` and the two siblings — over a
hand-listed five-row population.

⭐ **The distinction that drives every number below, and it is the one `AC-G0`
was written to catch:** the needles count **how many times a spelling appears in
the file**, never **how many units are emitted**. A loop that emits `n` units
from one call site contributes **exactly 1**. `native_int_clif` already
demonstrates the gap in the other direction — 6 emitted definitions from 5
`FunctionBuilder::new` source sites, because one site is shared by two helpers.

### Predicted rows after `B2F`

| file | builders | definitions | declarations | why |
|---|---|---|---|---|
| `lowering/core.rs` | **1** | **1** | **2** | ⭐ **unchanged.** The root entry function is still declared and defined here; the switch-over changes *what the root body calls*, not how many spellings this file holds |
| `lowering/units.rs` *(new)* | **1** | **1** | **1** | one loop over `emittable_units()`: one forward-declare site, one builder site, one define site |
| `lowering/mod.rs` | 0 | 0 | 0 | unchanged |
| `planning.rs` | 0 | 0 | 0 | unchanged — planning must never emit |
| `planning/static_transition.rs` | 0 | 0 | 0 | unchanged |
| `planning/static_transition/semantic_ir.rs` | 0 | 0 | 0 | unchanged |

### Predicted rows ADDED to state the population (`AC-2`, second clause)

⛔ **These are not new emitters — they are pre-existing ones the census could not
see.** An absent row and a zero row read identically to a reader and only one of
them is a claim.

| file | builders | definitions | declarations | status |
|---|---|---|---|---|
| `planning/static_transition/abi.rs` | **0** | **0** | **0** | inert by construction; the frame already flagged this omission |
| `boundary_value_clif.rs` | **23** | **3** | **3** | ⭐ **a live production emitter in NEITHER the census NOR `BACKEND_PRODUCTION_SOURCES`** |
| `native_int_clif.rs` | **5** | **1** | **3** | the `AC-G0` file; likewise in neither |

---

## P2 — `AC-G0`'s denominator covers THREE emitters, not two

⛔ **The frame's `AC-G0` discussion names `native_int_clif` as *the* excluded
sibling emitter. That is now incomplete.** Measured on `6534e4a6`, `ken-runtime`
holds **three** production Cranelift emitters outside the lowering root:

| emitter | builder sites | emitted definitions | growth in the program |
|---|---|---|---|
| `native_int_clif.rs` | 5 | **6** (`LOCAL_HELPER_COUNT`, `artifact/tests.rs:56`) | ⭐ Θ(1) per native module — **already settled, cite it** |
| `boundary_value_clif.rs` | 23 | *to be measured* | **predicted Θ(1) per module**, same shape |
| `lowering/units.rs` *(new)* | 1 | **one per `PredeclaredFunction`** | ⭐ **Θ(n)** — this node's growth |

**PREDICTION:** `boundary_value_clif`'s emitted population is a **fixed constant
per module**, orthogonal to `B2F`'s per-static-origin Θ(n), on the same grounds
that settle `native_int_clif` — `emit_boundary_value_local_graph` is called once
per compiled module from `lowering/core.rs:92` and takes no program-derived
parameter.

⚠ **NOT CLAIMED, and this is the half `AC-G0` exists to force:** I have **not**
yet counted `boundary_value_clif`'s emitted definitions, and its 23 builder
sites are a **source-site count**, which is precisely the population error the
frame's own `AC-G0` narrative records its author committing. ⛔ **Do not read 23
as an emitted-unit count.** The number to pin is the emitted one, and it is
measured in `S8`, not here.

---

## P3 — the growth verdict's shape

Stated in the Architect's exact required form, and ⛔ never as a blanket bound:

> Total units may be **Θ(n)** while **each function is bounded by its own static
> body/transition contract.**

**PREDICTION:** unit count `== plan.entries.len() + count(StaticBody edges)`,
which is already a planner-enforced equality (`semantic_ir.rs`
`validate_function_units`). ⇒ `B2F` **consumes** that equality as its growth
statement rather than re-deriving one, so the verdict cannot disagree with the
population it is about.

---

## P4 — where I expect to be wrong

⭐ Recorded because a prediction file listing only confident rows is a
transcription with extra steps.

1. **`lowering/core.rs` staying at `1/1/2` is the row most likely to move.** If
   the switch-over needs the root to be declared through the same bundle path as
   every other unit, its declaration migrates out of `core.rs` and the row goes
   to `1/1/1`. **That would be a design change worth reporting, not a number to
   update.**
2. **`lowering/units.rs` at one site each assumes one loop.** If forward
   declaration and definition cannot share a pass — the likeliest cause being
   that a body's emission needs every *other* unit's `FuncId` already declared —
   the file still holds one spelling of each and the row holds. ⚠ If it turns
   out to need two builder sites, the row is wrong and the reason is
   structural.
3. ⛔ **The census is a source-TEXT oracle and I am required to keep it.** The
   frame forbids deleting or weakening it; the operator's 2026-07-26 rule bars
   authoring tests whose subject is repository text. ⇒ I re-baseline it as
   instructed **and** add a behavioural control that counts what a compiled
   module actually contains, so `AC-2`'s property is defended by an oracle that
   a comment or a line-break cannot move. **The text census is retained as a
   tripwire, not as the evidence.**
