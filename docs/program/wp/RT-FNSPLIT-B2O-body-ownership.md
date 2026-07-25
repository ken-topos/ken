# `RT-FNSPLIT-B2O` — static body ownership

**One assignable deliverable: a total, validated occurrence → function-unit
ownership mapping in the semantic plane. Inert — no emitted unit, no call edge.**

- **Issue:** `docs/program/issues/RT-FNSPLIT-B2O.md`
- **Parent:** `RT-NATIVE-FNSPLIT` (symptom-inventory entry 2, first of three nodes)
- **Successor:** `RT-FNSPLIT-B2R` (representation + call-ABI contract), then
  `RT-FNSPLIT-B2F` (the atomic live switch)
- **Origin:** Architect ruling `evt_842spc7t6js1` item 5 plus the one-owner half
  of item 6, on research advisory `evt_531c4k52mshrn`. Steward-filed under the
  ruling's grant of re-slicing and sequencing ownership.

> ## ⏳ ONE DELIVERABLE IS PENDING AN ARCHITECT ANSWER — `evt_5vd31pvbrgdf3`
>
> **D1 below (the function population) is held.** Everything else in this frame
> is settled and can be started. The held question is whether the real unit set
> re-populates the existing `PredeclaredFunction` table (with a rename, since its
> current population is not what the name promises) or lands beside it as a new
> table. **Do not start `D1` until that is answered here.** `D2`–`D7` do not
> depend on it.

---

## ⛔ READ FIRST — the function population DOES NOT EXIST YET

This is the finding that makes this node necessary, and it is counter-intuitive
because the plane already contains a type called `PredeclaredFunction`.

**Measured on `origin/main = 9d515c9d`:**

```
semantic_ir.rs:534-536   program        = SemanticProgramId(planned_node.0)
                         capture_layout = CaptureLayoutId(planned_node.0)
                         function       = PredeclaredFunctionId(planned_node.0)
semantic_ir.rs:527       one loop iteration per planned node, one
                         PredeclaredFunction pushed per iteration
```

So `plane.functions` is an **alias of the node table**, and that equality is
**enforced in three places** — this is not incidental drift:

```
semantic_ir.rs validate()    self.functions.len() != nodes.len()  -> planner error
                             program.records.len  != 1            -> planner error
static_transition.rs:2239    assert_eq!(plan.semantic.functions.len(),
                                        plan.nodes.len())
```

And `StaticNode.transition` is a `TransitionKind` (`static_transition.rs:79-91`):
`Terminal`, `TrapTerminal`, `Evaluate`, `Sequence`, `Branch`, `CaseTest`,
`ClosureBody`, `ProducerWrapper`, `SourceReturnResume`, `ProducerTail`,
`CompletedTail`. Those are **abstract-machine steps.**

> ### ⭐ THE TRAP, STATED PLAINLY
>
> `D1` of `RT-FNSPLIT-B2F` says *"one closed target function per static planned
> function/origin."* Read against today's table, **"one per
> `PredeclaredFunction`" means one Cranelift function per machine transition
> state** — a population far larger than the target, and it would read as
> *literal compliance with the ruling.*
>
> ⛔ This is the same shape as every prior defeat in this chain: **a name
> promising a static notion, populated with something else.** Pointer identity,
> then runtime configuration, then "the carrier exists so the property holds"
> (#5), then "TOTAL is true so composition holds" (#8). ⭐ **A type's existence
> is not evidence about its population. Count it.**

Every `SemanticProgram` also spans **exactly one** record, so nothing in the
plane groups a body's occurrences either.

## The good news — the boundary is ALREADY MARKED, so the unit set is derived

The unit set is **not invented by this node**; it is read off the plan graph:

```
static_transition.rs:834,851   TransitionKind::ClosureBody   (both closure arms)
static_transition.rs:845,871   EdgeKind::StaticBody          (the boundary edge)
```

⇒ **function units = the root ∪ the `ClosureBody` heads**, and **ownership = the
reachability partition of nodes under non-`StaticBody` edges from each head.**
`StaticBody` is exactly the cross-owner edge.

This matters for honesty as much as for cost: an ownership map *derived* from the
plan graph cannot drift from the graph, whereas a second hand-maintained table
can — and would then need its own agreement checker.

## ⭐ AND THE OWNER FIELD ALREADY EXISTS — it is currently an identity alias

`SemanticDescriptor` (`semantic_ir.rs:459-466`) already has a `function:
PredeclaredFunctionId` field, one descriptor per planned node / per origin. Today
it is filled with `PredeclaredFunctionId(planned_node.0)` — an identity alias
carrying no information.

**So the core change is not a new structure. It is: repopulate `functions` as the
real unit set, and make `descriptor.function` name the OWNING unit instead of the
node's own index.** Consequences, all of them wanted:

- **Totality is structural, not asserted.** There is already exactly one
  descriptor per origin, so every occurrence has exactly one owner **by
  construction**. ⚠ Pin it anyway — see `AC-2` on why "structural" is not
  self-verifying.
- **Exclusivity is structural too** — one field, not a list.
- The two axes are **separable**: this node moves the *functions* axis only.
  `programs`/`records` stay one-to-one, so `child_origin`
  (`semantic_ir.rs:664-700`, which destructures `[record]` and *requires* exactly
  one) is **untouched**, and `B2A-C`'s landed correspondence keeps working
  unchanged. ⛔ Do not widen this node to the programs axis.

## What this node does NOT do

- ⛔ **No emitted unit, no call edge, no dispatch, no callback, no flag, no
  alternate entry.** Production keeps **exactly** the existing one root
  `FunctionBuilder` and one `define_function`.
- ⛔ **No signature, no slot layout, no calling convention, no ownership or
  lifetime rules.** All of that is `RT-FNSPLIT-B2R`. This node answers *where are
  the boundaries*; `B2R` answers *what may cross them*.
- ⛔ **No removal of any lowering authority.** `D6`'s removal of cross-owner
  re-emission is `B2F`'s atomic switch. This node **describes** the boundary that
  `B2F` will later enforce.
- ⛔ **No container-spelling blacklist** for the `B2A-S` entry-keyed-store
  residual. Ruled: that arm stays review-enforced.

## Deliverables

### ⏳ D1 — the function unit population *(HELD, see the box at the top)*

Establish the real unit set: **root ∪ `ClosureBody` heads**, derived from the
plan graph rather than hand-listed. Whether this re-populates
`PredeclaredFunction` (renamed) or lands as a new table is the held question.

⛔ **Whichever it is, the type whose population is one-per-transition-node must
not remain in the plane under a name that promises "function."** If both tables
end up existing, `B2R` item 3 cannot say which one names a signature.

### D2 — `descriptor.function` becomes the owner pointer

Fill `SemanticDescriptor.function` with the **owning** unit, computed by the
reachability partition, replacing the `PredeclaredFunctionId(planned_node.0)`
identity alias.

### D3 — the boundary predicate, as a checkable graph property

State and check, over the whole plan graph:

- every **non-`StaticBody`** edge has `owner(from) == owner(to)`;
- every **`StaticBody`** edge has `owner(from) != owner(to)`, and `to` **is** a
  unit head;
- every unit head is the target of exactly one `StaticBody` edge, **except the
  root**, which is a head and the target of none.

### D4 — validation, as a planner error before emission

Extend `SemanticPlane::validate` so a violated ownership invariant is a
**planner error**. ⛔ **Never a fallback, never a repair, never a warning** — the
ruling is explicit that failure precedes emission and that there is no fallback
to the old specializer after partial emission.

### D5 — re-baseline the three enforced equalities to PREDICTED numbers

`D1` makes `functions.len() != nodes.len()` **true**, so the three sites listed
above go red **by design**:

| site | what it currently enforces |
|---|---|
| `semantic_ir.rs` `validate()` | `functions.len() == nodes.len()` |
| `static_transition.rs:2239` | `assert_eq!(functions.len(), nodes.len())` |
| `static_transition.rs:1687` | `helper_definitions: functions.len()` — a **reported metric** |

⛔ **Do not delete or weaken any of the three.** ⛔ **Do not re-baseline to
whatever the new numbers happen to be.**

⇒ **Predict each new value from the design, record the prediction and its reason
in the test/comment, and only then measure.** A count that differs from the
prediction is **a finding to route, not a number to update**. ★ This is the
`pin-a-property` discipline (`agent/playbooks/tools/pin-a-property.md`).

⚠ `:1687` is the subtle one: `helper_definitions` is a **metric other things
read**, not a test. Changing the function population changes a reported figure.
Inventory its consumers before you move it, and say in the WP what the figure
means afterwards.

### D6 — the 59-call disposition, BY OWNER, as a derived report

Report each of the **59** tokenized production calls into `lower_expr` against
the ownership mapping: for each call, the owner of the occurrence it lowers and
whether that call crosses an owner boundary.

⛔ **This is a report, not the authority.** The ruling is explicit: *disposition
is per occurrence ownership and reaching path, not one row per source site*, and
**the five provenance classes are evidence inputs, not the authority
partition.** The withdrawn `AC-5` failed precisely because it keyed disposition
to the site; for 14 caller-dependent sites the answer is a function of the
**reaching path**, so the same site can appear on both sides.

⚠ **The census is a tokenized count, not a `self.`-spelled one.** The root call
is `compiler.lower_expr(` at `core.rs:188` and it takes `root_static_origin` — it
**seeds** the descent and is not traversal. A census derived from
`grep -c 'self\.lower_expr('` returns 58 and silently loses the program's entry
point. **Use the tokenizer** (`identifier_occurrences`,
`lowering/core/tests/control.rs:3529`).

### D7 — the inert-scaffold controls

Both cfg configurations pin the unchanged production emitted-unit census and
**zero** executable edge into functionized emission.

## Acceptance criteria

**AC-1 — inert in production, in BOTH configurations.** The production emitted
unit census is unchanged (`core.rs` 1 builder / 1 definition / 2 declarations;
`lowering/mod.rs`, `planning.rs`, `planning/static_transition.rs`,
`semantic_ir.rs` all zero), under `cfg(test)` **and** not. ⚠ `core.rs` carries
**22 inline `#[cfg(test)]` attributes inside production functions**, so "both
configurations" has real surface in the file you are touching — it is not a
formality.

**AC-2 — totality is pinned, not merely structural.** Assert every origin
resolves to an in-range owner. ⭐ **"It is total by construction" is exactly the
claim #5 was defeated on** — the carrier existed and the property did not follow.
A structural guarantee still needs a check that *fires* if the construction
changes.

**AC-3 — ⛔ COMPOSITION, BIDIRECTIONALLY. This is the #8 lesson and it is the AC
most likely to be under-served.** Hard-stop #8 was **predictable from the
question the frame asked**: the census answered `TOTAL` and was *true*, but the
mechanism needed **closure under parent→child reachability**, a different
property — `ComputationalMatch` filed its occurrence on a different node from the
entry its parent pointed at, so totality held while composition failed.

⇒ Pin, **per variant, on a real instance**:

- descending parent → child **within** an owner stays in that owner;
- an owner boundary crossed on the way **down** is the same boundary seen on the
  way **back up**;
- `owner(child_origin(p, i))` agrees with `owner(p)` for every non-boundary
  child, **for every `SemanticOpcode` variant**, not for a sampled few.

⭐ **Compose the accessor with itself, per variant, before threading it
anywhere.** Rigour does not supply relevance: write down the mechanism
obligation the measurement is meant to discharge, then check the implication.

**AC-4 — the unit set is exactly root ∪ `ClosureBody` heads, with a POSITIVE
control.** ⚠ A negative check ("no extra units") passes for any reason,
including because nothing reached the checker. So include a control that **adds a
closure** and observes **exactly one** additional unit, and a control that adds a
non-closure expression and observes **zero** additional units.

**AC-5 — the boundary predicate is enforced, with an evasion attempt per pin.**
For each pin in `D3`, attempt a **compile-preserving** evasion — construct a plan
that violates the invariant and confirm the planner errors. ⛔ A pin that
enumerates spellings is not a proof of the property.

**AC-6 — the three re-baselined pins carry their predictions.** Each records the
predicted number and the reason, dated, in the test or comment.

**AC-7 — `B2A-C`'s correspondence is untouched.** `child_origin` still requires
exactly one record per program; `programs`/`records` stay one-to-one.
`B2A-S`'s `AC-4` pin on the `origin → expression` lookup count still holds — if a
second consumer appears, route through `retained_body_occurrence`
(`lowering/core.rs:4176`) or re-baseline **explicitly**, never silently.

**AC-8 — no-regression means GREEN IN CI.** ⛔ Locally: targeted only, via
`scripts/ken-cargo` with `-p ken-runtime` or `--test <name>`. **Never
`--workspace`, never `--locked`, never the conformance suite on this box** — the
full gate runs in CI on GitHub and the scripted publisher polls those exact
checks.

## ⚠ Anchors — RE-DERIVE ON YOUR OWN BASE BEFORE YOU TRUST ONE

Every anchor in this chain has moved at least once; `lower_expr` alone went
`:3847 → :4255 → :4333`. Measured on `9d515c9d`:

| fact | location |
|---|---|
| `lower_expr` definition | `lowering/core.rs:4333` |
| synthesized root call | `lowering/core.rs:188` (`compiler.lower_expr`) |
| tokenized production call population | **59** |
| `StaticNode` / `TransitionKind` | `planning/static_transition.rs:164`, `:79` |
| `ClosureBody` heads | `planning/static_transition.rs:834`, `:851` |
| `StaticBody` edges | `planning/static_transition.rs:845`, `:871` |
| `SemanticDescriptor` | `planning/static_transition/semantic_ir.rs:459` |
| `PredeclaredFunction` | `…/semantic_ir.rs:449` |
| id aliasing | `…/semantic_ir.rs:534-536` |
| `child_origin` accessor | `…/semantic_ir.rs:664` |
| enforced equality | `…/semantic_ir.rs` `validate()`; `static_transition.rs:2239` |
| `helper_definitions` metric | `static_transition.rs:1687` |
| tokenizer for the census | `lowering/core/tests/control.rs:3529` |
| emitted-unit census pin | `lowering/core/tests/control.rs:3336` |

⚠ **`crates/ken-backend-native` does not exist.** The research advisory
(`evt_531c4k52mshrn`) cites every path under that prefix; its line numbers are
accurate but its paths are not (erratum `evt_3k9xam3ws9pgz`). The real roots are
`crates/ken-runtime/src/cranelift_backend/{lowering/core.rs, lowering/mod.rs,
planning/static_transition/semantic_ir.rs}`.

## Hard-stop protocol

**Raise a hard-stop the moment the frame is unsatisfiable as written — before
writing code.** That is exactly what happened at #9 and it cost nothing to
unwind, because the branch was clean. **Count of record = 9** (Steward holds it;
on any disagreement the parent issue's count line wins).

⛔ **The next armed research pull is #12.** #9's is consumed.

⚠ When you stop, **preserve your evidence and report the exact SHA.** Build seats
have no GitHub credential by design; the Steward pushes WP branches. Evidence
that exists only in your worktree exists on **one local ref with zero off-box
copies** — that is the state the #9 stop nearly ended in.

## Rebase and handoff discipline

- The WP branch is `wp/RT-FNSPLIT-B2O-body-ownership`, cut from `origin/main` at
  kickoff. Report the exact SHA on every fold; the Steward pushes it.
- ⚠ `wp/RT-FNSPLIT-B2F-functionization` at `fbe206a7` is the **#9 evidence ref**,
  based on `3891b7aa` deliberately. ⛔ **Do not merge, squash, rebase, or build on
  it.** It is a droppable evidence object.
- ⛔ **Never `git stash`** — `refs/stash` is shared across ~70 worktrees. Commit
  to your `<role>/work` branch or add a worktree instead.
- A frame amendment that is not on a fetchable ref **has not happened**. If you
  are told the frame changed, fetch it before acting on it.

## ⭐ Pin discipline — this chain has now spent nine hard-stops on it

1. **Predict, then measure.** A baseline re-fit to observed output measures
   nothing.
2. **A negative check passes for any reason** — pair it with a positive control
   that *would* fire.
3. **A structural pin that enumerates spellings is not a proof of the property**
   — attempt a compile-preserving evasion of each pin.
4. **Never let a census key on a spelling standing in for a population.** This
   frame's own predecessor said 58 because it counted `self.lower_expr(`; the
   root call is spelled `compiler.lower_expr(` and is the entry point. The same
   defect appeared one layer up in `AC-G0` (source sites where emitted units
   belonged) and again in the capture population (`RuntimeExpr::Closure` where
   `LexicalClosure` behaves differently).
5. **A measured property can be TRUE and not entail what the mechanism needs.**
   State `MEASURED / CLAIMED / THE GAP` as its own sentence.
