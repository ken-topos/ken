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

> ## ✅ FULLY RULED — nothing is held. Architect `evt_48dxvb2yrwpad`.
>
> `D1` was briefly held on `evt_5vd31pvbrgdf3` and is now answered: **repurpose
> the sole `PredeclaredFunction` table** as the real function-unit table, and
> **remove the misleading per-node row** — there must be exactly one table whose
> name claims "function," and `B2R` attaches signatures to it.
>
> ⛔ **AND THE RULING CORRECTED MY SEED DERIVATION — see the next section. An
> earlier revision of this frame said the heads were the `ClosureBody` nodes.
> That is WRONG, and building it would have produced the wrong unit set and an
> unsatisfiable edge law.**

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

## The unit set is DERIVED from the plan graph — and here are the right seeds

The unit set is **not invented by this node**; it is read off the plan graph. But
the seeds are **not** what an obvious reading suggests.

### ⛔⛔ `TransitionKind::ClosureBody` IS NOT THE HEAD. It is the body's RETURN.

Read the planner (`static_transition.rs:831-846`, and identically `:848-871` for
`LexicalClosure`) in **order**:

```rust
let body_return = self.control_node(TransitionKind::ClosureBody, …)?;  // 1. made FIRST
self.edge(body_return, self.terminal, EdgeKind::Continue)?;            // 2. exits to the
                                                                      //    SHARED terminal
let body = self.plan_expr(body, ctx, body_return, EdgeKind::Continue, 0)?;
                                          // 3. the body is planned TOWARD body_return
…
self.edge(node.entry, body.entry, EdgeKind::StaticBody)?;   // 4. the boundary edge
                                                           //    targets body.entry
```

⇒ The `ClosureBody` node is the body's **return successor**, planned *before* the
body and reached *after* it. The head is the **`StaticBody` edge's target**
(`body.entry`). The unit **owns** its `ClosureBody` return node; it is not seeded
by it.

⭐ **A frame revision of mine said "root ∪ `ClosureBody` heads." It was wrong, and
it would have failed twice over:** the unit set would have been seeded on return
nodes instead of entries, **and** the edge law would have been *unsatisfiable*,
because step 2 above is a **non-`StaticBody` edge from a body-owned node to the
shared terminal.** ⇒ **This is the third time this chain has been bitten by a
marker that names something adjacent to what the reader wants** (`self.lower_expr(`
was a receiver spelling, not the call population; `RuntimeExpr::Closure` was one
of two capture arms). **Read the construction order, not the name.**

### The ruled seeds (Architect `evt_48dxvb2yrwpad`)

```
1. every scheduling entry in StaticTransitionPlan.entries   (static_transition.rs:216)
     = the program root plus each transparent declaration body
2. every TARGET (edge.to) of an EdgeKind::StaticBody edge   (:845, :871)
     = each retained closure-body entry
```

Give these seeds **dense** `PredeclaredFunctionId`s. Validate that entries are
unique (there is already a uniqueness check at `:1158`), that `StaticBody` targets
are unique, and that the **two seed sets are disjoint**. ⛔ **A duplicate is a
planner error, not a deduplication opportunity.**

⚠ `entries.first()` is a **scheduling** entry — `:1047` already warns about this
in-source. Do not treat position 0 as privileged beyond what that comment says.

### ⭐ Two shared exit sentinels sit OUTSIDE the exclusive partition

`self.terminal` is a **single shared node**, and the unique `Terminal` and
`TrapTerminal` are reachable from **many** units by design.

⇒ Classify them as **shared exit templates, outside the exclusive function-owner
partition.** An edge to one lowers as **the current unit's local return/trap,
never as a cross-owner call.** Any *other* multiply-owned or unowned node is a
planner error.

⚠ Without this exception the ownership partition is not merely incomplete — it is
**contradictory**, since every unit's return path ends at the same node.

### Ownership

For each seed, traverse outgoing edges **without crossing `StaticBody`**. Every
ordinary node must be reachable from **exactly one** seed. Record the owner on the
semantic descriptor (or an equally canonical dense node-owner arena). ⛔ **Do not
maintain a second hand-authored population** — a map *derived* from the graph
cannot drift from it, whereas a parallel table needs its own agreement checker.

⚠ There is already a reachability computation seeded from `entries` at
`static_transition.rs:1275` — read it before writing a second one.

## ⭐ AND THE OWNER FIELD ALREADY EXISTS — it is currently an identity alias

`SemanticDescriptor` (`semantic_ir.rs:459-466`) already has a `function:
PredeclaredFunctionId` field, one descriptor per planned node / per origin. Today
it is filled with `PredeclaredFunctionId(planned_node.0)` — an identity alias
carrying no information.

**So the core change is not a new structure. It is: repopulate `functions` as the
real unit set, and make that field name the OWNING unit instead of the node's own
index.** Consequences, all of them wanted:

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

> ### ⛔⛔ BUT THE FIELD'S TYPE CANNOT TELL THE TRUTH — and this is the subtlest
> ### requirement in the frame
>
> `function: PredeclaredFunctionId` **cannot represent the two shared exit
> sentinels.** They are not functions, so no `PredeclaredFunctionId` describes
> them.
>
> ⛔ **And you may NOT reach for an `Option`, a sentinel index, or a reserved
> "invalid" id.** The sentinels are **explicit shared exit templates** — they are
> **not missing data** and **not target functions**. An `Option<…>` says "absent",
> which is a third thing that is false.
>
> ⇒ **Replace the field with an exhaustive closed classification**, e.g.
>
> ```rust
> enum SemanticOwner {
>     Function(PredeclaredFunctionId),
>     Terminal,
>     TrapTerminal,
> }
> ```
>
> Every ordinary descriptor is `Function`; **only the unique two sentinel
> descriptors** take a shared-exit variant.
>
> ⭐ **Why this is worth a whole box:** a taxonomy with no cell for the honest
> answer *reads as complete*. That is exactly how the withdrawn `AC-5` passed
> review — its two-way classification had no cell for "depends on the reaching
> path", so it could have been filled in completely and still been wrong. Here the
> same defect would live in a **type**, where it is even harder to see: the code
> would compile, every descriptor would carry a `PredeclaredFunctionId`, and two
> of them would be lies.

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

### D1 — the function unit population, in the ONE table named "function"

Repopulate `PredeclaredFunction` as the **real** function-unit table, with dense
`PredeclaredFunctionId`s over the ruled seeds:

```
all scheduling entries in plan.entries       (root + each transparent declaration)
  UNION
all TARGETS (edge.to) of EdgeKind::StaticBody edges   (each retained body entry)
```

⛔ **Remove the misleading per-node function row. Do not leave the node-alias
table beside a new `FunctionUnit` table** — there must be **exactly one** table
whose name claims "function," because `B2R` attaches signatures and frame layouts
to it and cannot be told which of two to use.

The node-level semantic definition already lives in `SemanticDescriptor` +
`SemanticProgram` + `SemanticRecord`. **Retain those node-scoped artifacts** and
move any non-redundant field from the old per-node function row into them.

Validate: entries unique, `StaticBody` targets unique, and the **two seed sets
disjoint**. ⛔ **A duplicate is a planner error, not a deduplication
opportunity.**

### D2 — the owner classification on every descriptor

Replace the `function: PredeclaredFunctionId` identity alias with the **exhaustive
closed classification** described in the box above —
`SemanticOwner::{Function(PredeclaredFunctionId), Terminal, TrapTerminal}` or an
equivalent — filled from the reachability partition.

⛔ **No `Option`, no reserved invalid id, no sentinel index.** The two shared
exits are neither functions nor missing data.

### D3 — the edge laws, as checkable graph properties

State and check, over the whole plan graph:

- a **non-`StaticBody`** edge either stays **within one `Function` owner** **or
  targets `Terminal`/`TrapTerminal`**;
- a **`StaticBody`** edge crosses from a `Function` owner to a **distinct function
  seed**;
- **each `StaticBody` target has exactly one incoming `StaticBody` edge**;
- **each top-level scheduling entry has none.**

⚠ **Do not write "every head except the root has one incoming `StaticBody`."**
That is wrong for **transparent declaration entries**, which are top-level seeds
too — the root is not the only entry. An earlier revision of this frame said
exactly that.

⛔ **Any other cross-owner edge is a planner error.**

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

**The predictions to write down BEFORE measuring:**

```
functions.len()           = entries.len() + count(StaticBody edges)
descriptors/programs/records  remain EXACTLY nodes.len()   (node-exact, unchanged)
every non-sentinel node   has exactly ONE Function owner
shared-exit population    is exactly { Terminal, TrapTerminal }
cross-owner edges         are exactly the StaticBody edges
```

⚠ `:1687` is the subtle one: `helper_definitions` is a **metric other things
read**, not a test — easy to miss when you read the red as "three tests."
**Rename it to the property it now measures** (e.g. `function_units`). ⛔ *"Helper
definitions" must not silently inherit the old node count* — a name that keeps
reporting a number whose meaning changed underneath it is worse than a rename,
because nothing fails. Inventory its consumers before you move it.

### D6 — the 59-call disposition, BY OWNER, as a derived report

> ## ⛔⛔ ARCHITECT RULING 2026-07-25 (`evt_5yxjd1zqnyvcq`, durable at
> ## `architect/work` `8bff7b7a`) — D6 IS A REPORT AND ITS ROUTE ORACLE IS OUT.
>
> **The authority is the occurrence's `StaticOriginId`, its validated
> `SemanticOwner`, and the planned edge kind.** *"Rust syntax parsing cannot
> prove reachability or authority: it lacks name resolution, macro expansion,
> and indirect-call semantics, and it would freeze a transient lowering
> topology. More fundamentally, **a Rust wrapper or same-named method does not
> create a Ken function-unit boundary.**"*
>
> **Required of this deliverable:**
>
> 1. **Remove `cannot grow silently` and every equivalent reachability claim.**
>    The report must state that **the owner graph is authoritative** and that
>    **Rust-route closure is unmechanized.**
> 2. The **59-call census and the helper observations survive only as
>    explicitly frozen review evidence** — never as authority.
> 3. **Do NOT add `syn`, any new dependency, or a lowering production change.**
> 4. Respin from the **already-reviewed production bytes**. They have not moved
>    since `97db6f0b`.
>
> ### ★★ THE CONTROL INVERTS — READ THIS BEFORE WRITING ANY PIN
>
> > *"A Rust wrapper or nested function relocation must remain **GREEN** for
> > semantic boundary classification, proving source topology is not
> > authority."*
>
> ⇒ **The mutation four folds fought to make RED must now stay GREEN.** A pin
> that reddens when someone adds a Rust wrapper is **measuring the wrong thing
> and reporting success.** Structured controls mutate **graph/owner axes**, not
> source text.
>
> ### ⛔⛔ AND THE PART THAT COST FOUR CANDIDATE SHAs — VERIFIED, NOT ASSERTED
>
> **The claim that consumed folds 2–5 was never an obligation of this frame.**
>
> | where | occurrences of `cannot grow silently` |
> |---|---|
> | **this frame** | **0** — and no AC here has ever required route-set closure |
> | report at `97db6f0b` (first QA-approved tree) | **0** |
> | report at `c59d76ce`, `02afcc3f`, `96627f2a` | **1** — *introduced by the fold* |
>
> The original D6 finding was correct: **a count does not support that claim**,
> because the count is invariant under adding a wrapper. But a claim that
> outruns its evidence has **two** repairs — **strengthen the evidence, or
> narrow the claim** — and *nobody checked whether the claim was required.* It
> was not. **Narrowing was free and available at fold 2, and it was one deleted
> sentence.** Instead the claim was mechanized, and the mechanization then had
> to be defended against evasions no source-text oracle can close.
>
> ★ **This section already said the right thing** — *"⛔ This is a report, not
> the authority"* — **and the folds drifted past it.** An unrequired claim
> acquired the force of a requirement purely by sitting in a deliverable.
> ⇒ **Before hardening any mechanism to support a claim, check whether the
> claim is required at all.** Deleting an over-claim is always cheaper than
> mechanizing it, and it is *always* the right move when no AC asked for it.

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

**AC-2 — totality and exclusivity are pinned, not merely structural.** Assert
every **non-sentinel** node resolves to exactly one in-range `Function` owner, and
that the shared-exit population is **exactly** `{Terminal, TrapTerminal}` — no
more, no fewer. Overlap is permitted **nowhere else**; a non-sentinel unowned or
multiply-owned node is a planner error.

⭐ **"It is total by construction" is exactly the claim #5 was defeated on** — the
carrier existed and the property did not follow. A structural guarantee still
needs a check that *fires* if the construction changes.

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
  child, **for every `SemanticOpcode` variant**, not for a sampled few;
- ⭐ **each retained-body unit OWNS its reachable `ClosureBody` return successor**,
  and that successor exits only through the shared terminal. **This is the "up"
  half of the invariant** — the boundary crossed on descent is represented by the
  **callee seed**, and the body's return node stays inside that **callee's** owner.

⛔ **`B2O` must NOT invent a static edge back to the caller.** `B2R` later carries
the dynamic return continuation in the frame. A static return edge here would
manufacture a cross-owner edge that the graph does not have, and `D3` would then
be describing a graph this node created rather than the one the planner builds.

⭐ **Compose the accessor with itself, per variant, before threading it
anywhere.** Rigour does not supply relevance: write down the mechanism
obligation the measurement is meant to discharge, then check the implication.

**AC-4 — the unit set is exactly `plan.entries` ∪ `StaticBody` targets, with
THREE POSITIVE controls.** ⚠ A negative check ("no extra units") passes for any
reason, including because nothing reached the checker.

| control | expected |
|---|---|
| add one retained closure | **exactly one** additional function unit |
| add one transparent declaration | **exactly one** additional function unit |
| add a non-closure expression inside an existing unit | **zero** additional units |

⚠ **The middle row is the one an obvious test set omits.** A closure/non-closure
pair does not exercise the **second top-level seed class** at all, so a
declaration-entry bug would pass every control. ⭐ **Two seed classes require two
positive controls** — this is the "no cell for the honest answer" defect in test
form.

**AC-5 — every law is enforced, with an INDEPENDENT redden control each.** For
each, construct the violation and confirm the planner errors **before emission**.
⛔ A pin that enumerates spellings is not a proof of the property; attempt a
**compile-preserving** evasion of each.

Required, each reddening **on its own**:

1. a missing root entry;
2. a missing **transparent declaration** entry;
3. a missing `StaticBody` target;
4. a **duplicate** `StaticBody` target;
5. a non-`StaticBody` cross-owner edge;
6. an ordinary node owned by **two** seeds (overlap);
7. a **sentinel misclassified as a `Function`**;
8. a `ClosureBody` return successor assigned to the **caller** instead of the
   callee.

⚠ Number 8 is the one that would otherwise ship green: assigning the return node
to the caller is the *intuitive* reading of "the caller resumes here", it produces
a coherent-looking partition, and only the down/up invariant catches it.

**AC-6 — the three re-baselined pins carry their predictions, and the metric is
renamed.** Each records the predicted number and its reason, dated, in the test or
comment; all five predictions in `D5` are written down **before** measuring; and
`helper_definitions` is renamed to the property it now measures. ⚠ Renaming is
part of the AC because this pin is the one that **cannot fail loudly** — a metric
keeps reporting whatever it is given.

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
| **seed class 1** — scheduling entries | `planning/static_transition.rs:216` (`entries`), pushed at `:1728`, uniqueness checked at `:1158`, caveat at `:1047` |
| **seed class 2** — `StaticBody` edge TARGETS | `planning/static_transition.rs:845`, `:871` |
| ⛔ `ClosureBody` = retained-body **RETURN SUCCESSOR**, never a head | `planning/static_transition.rs:834`, `:851` |
| shared exit sentinel | `self.terminal`; `TransitionKind::{Terminal, TrapTerminal}` at `:80-81` |
| existing reachability walk from `entries` | `planning/static_transition.rs:1275` |
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
