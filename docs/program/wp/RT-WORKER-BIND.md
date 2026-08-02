# WP frame — `RT-WORKER-BIND` (static-worker binding and transport substrate)

Node: `docs/program/issues/RT-WORKER-BIND.md`. Campaign:
`docs/program/16-recursive-descent-retirement.md`. Owner: runtime ring.
Authority: Architect outcome (c) `evt_2anwskscqz5fg` and the complete
substrate interface ruling `evt_4bgwqgydycd34`, both grounded on exact held
`dd0ca60e` / tree `f7a4495984dd`.

`RT-CONTSPEC-ACTIVATE` stopped because lowering cannot bind a worker's carried
capture operands into a selected semantic body. **This node builds that
binding, and proves it without continuation specialization anywhere near it.**

## The five judgments this frame makes, so you do not have to

### 1. The representation is RULED and CLOSED — do not redesign it

```rust
enum LoweringEnvironmentBinding {
    Value(LoweringOperand),
    StaticWorker(StaticWorkerBinding),
}

struct StaticWorkerBinding {
    closure_origin: StaticOriginId,
    body_origin: StaticOriginId,
    declared_arity: u32,
    captures: Vec<LoweringOperand>,
}
```

**Spelling may vary. The discriminants, the fields, and the separation may
not.** The Architect closed this choice against current source: environments
are `[LoweringOperand]`, `Var` clones one operand, `Lowered::Closure` requires
`Vec<Lowered>`, `LoweringOperand` is the closed
`Specialized | Carried` phase boundary, and `call_declared_unit_target`
already accepts ordered operands through the exact unit descriptor.

⛔ This is **not** a new `Lowered` variant, a third `LoweringOperand` arm, an
extra `ComputationalRecursorClosure` child, a carrier `Record`, a planned
aggregate, an inverse phase conversion, or a runtime callable lane. `Lowered`,
`LoweringOperand`, AC-C4's single-field license, and the boundary ABI all stay
unchanged.

### 2. `StaticWorkerBinding` is COMPILER-ONLY

No runtime word, tag, layout, vtable, descriptor or environment pointer, or
callable identity. **Captures stay `LoweringOperand`; a carried capture stays
carried.** The whole point is that the binding never becomes a value.

⛔ A `Lowered::StaticWorkerRef` plus a side table is **explicitly rejected** —
it could enter a `Vec<Lowered>` aggregate before any later escape check
noticed.

### 3. ⭐ THIS BINDING IS NOT AFFINE — and your instinct will say otherwise

A static worker binding is an **ordinary lexical callable**. **Unused is
lawful. Called twice is lawful.**

⛔ **Do not add a consumed-worker set, an exact-once token, or a
required-empty use ledger.** That would change source semantics and duplicate
an authority that belongs to `RT-CONTSPEC-ACTIVATE`, not here.

⚠ **Why this is called out this hard:** the frozen seam spent four stops
building exactly such an affine ledger, and that work is fresh. Carrying the
pattern across is the most likely single error in this node. Completion here is
**structural**, not affine — see `D7`.

### 4. ONE binding authority

Every lexical environment reaching `lower_expr` uses the sum — saved ordinary
and computational eliminator environments, pending-`Let` environments, and
recursor outer environments. Existing binders install `Value`; the outer spine
forwards unchanged.

⛔ **No parallel operand environment and no de-Bruijn side map.** Either
creates two binding authorities, and then the question "what is bound here" has
two answers.

### 5. ⭐ THE WITNESS MUST CONTAIN ZERO CONTINUATION MACHINERY

This is the load-bearing requirement and the reason this node exists as a
separate node at all.

The witness is an **ordinary FunctionizedUnits program**: a normal unit
receives a real ABI input (hence genuine `Carried`); an ordinary `Let` binds a
lexical closure capturing it, preferably with two ordered captures; the body
calls `Var(0)`; and the linked result distinguishes the selected worker body
and capture order.

⛔ The fixture contains **no** `ComputationalMatch`, continuation
specialization, `ContinuationSpecializationId`, `ContinuationCallIdentity`,
continuation descriptor, or continuation token. ⛔ **A test-only constructor or
a planner census is not the witness** — it must traverse production planning,
ordinary unit declaration, production lowering, object emission, linking, and
execution.

⇒ If the only way you can demonstrate the substrate runs through continuation
specialization, **stop and route** (hard stop 6). That is not a testing
inconvenience; it means this is a cumulative branch rather than a substrate,
which is the shape that killed `RT-CONTSPEC-LOWER`.

## Fixed inputs

Measured at `origin/main = ce3296d8`.

| input | measured value |
|---|---|
| the held seam | `RT-CONTSPEC-ACTIVATE` at `status: draft`, `dd0ca60e` **preservation-only, not a base** |
| the occurrence authority | `retained_body_occurrence`, `child_occurrence`, `case_body_occurrence` in `lowering/core.rs` |
| the call emitter | `call_declared_unit_target`, already accepting ordered operands through the exact unit descriptor |
| the operand boundary | `LoweringOperand = Specialized(Lowered) \| Carried(CarriedBoundaryWord)`, closed |
| the unit population | existing `emittable_units`; this node **projects** it, never extends it |

⛔ **Branch from `main`.** Do not branch from, rebase onto, or cherry-pick
`dd0ca60e` or any preserved `RT-CONTSPEC-*` object.

## Deliverables

- **D1 — the environment binding sum**, threaded through every lexical
  environment reaching `lower_expr`. Existing binders install `Value`; the
  outer spine forwards unchanged.
- **D2 — one construction route.** A private retained-occurrence projection
  for an exact `Closure`/`LexicalClosure`: closure origin is the exact
  occurrence origin, body origin is exact child 0, lexical capture `i` is exact
  child `1 + i`, declared arity is the source parameter count, and seed versus
  lexical capture provenance stays distinct during validation. Use
  `retained_body_occurrence` / `child_occurrence` — **no source re-walk, no
  global closure search.**
  The constructor requires, **before installation**: capture count equals the
  retained definition; the current function has a declared static-body target
  for `body_origin`; `DeclaredUnitCall.origin == body_origin`; descriptor
  parameter count equals `declared_arity`; descriptor capture count equals the
  capture vector; and slots, offsets, and frame bytes come unchanged from that
  descriptor. **Missing, duplicate, wrong-body, wrong-arity, and wrong-capture
  facts all reject before worker-call emission.**
  For ordinary `Let`, a binder-lowering helper preserves existing
  `Value(lower_expr(...))`. A lexical closure with all-specialized captures may
  preserve existing `Lowered::Closure`; **one with any carried capture installs
  `StaticWorker`.** ⛔ Classify all captures exhaustively first — never select
  from syntax spelling or from a partially emitted `specialized_at` failure.
- **D3 — the sole consumer, and non-escape.** The only consumer is
  `RuntimeExpr::Call` with an exact `Var(index)` callee resolving to
  `StaticWorker`: validate argument count against `declared_arity`, lower
  explicit arguments in source order, append stored captures **without phase
  conversion**, obtain this function's `DeclaredUnitCall` by exact body origin,
  and use `call_declared_unit_target`. ⛔ No `call_indirect`, runtime
  selection, tag or layout dispatch, environment decode, or body re-lowering.
  `Var` in every value-producing position accepts **only** `Value`.
  `StaticWorker` used as a result, aggregate field, primitive or effect
  argument, stored value, projection subject, match scrutinee, or ordinary call
  argument **fails closed before carrier transfer.** Match the sum
  exhaustively; **no wildcard.**
- **D4 — function-local target transport.** ⛔ **`FuncRef` does not go in the
  binding** — it belongs to one Cranelift `Function`. Index the existing unit
  bundle's already-validated static-body units by exact body origin for their
  existing `FuncId` and ABI descriptor; this is a **projection of
  `emittable_units`**, not a new unit or call-edge population. Missing or
  duplicate body origins reject. Each generated function declares its required
  worker `FuncId`s into that function and receives fresh `DeclaredUnitCall`s;
  ordinary units may reuse their graph-derived
  `FunctionLocalRefs.unit_calls`. Expose the same local declaration operation
  for a separately emitted caller. ⛔ **Never copy a `FuncRef` between
  functions.**
- **D5 — multiple workers and positions.** The substrate is position-agnostic
  and supports any number of `StaticWorker` bindings in one environment,
  preserving supplied binder order.
- **D6 — nested-worker closure.** A static worker body may bind and call
  another captured static worker using the same representation: the inner
  definition comes from its exact retained occurrence, its captures are the
  outer function's value operands **including carried ones**, and its target is
  declared afresh into the outer worker function.
- **D7 — structural completion.** Every requested target resolves uniquely
  against the existing bundle before body definition; every call uses a
  `DeclaredUnitCall` owned by the current function and its exact descriptor; no
  `StaticWorker` escapes the callee-only arm; and binding scopes end
  structurally with their lexical environment. ⛔ **No affine ledger** —
  judgment 3.
- **D8 — the independent witness and its companions.** The ordinary
  FunctionizedUnits program of judgment 5, plus: capture omission and
  reordering; wrong body, arity, descriptor, and capture count; two same-shape
  workers; nested worker; result/aggregate/value-position escape; fresh
  per-function `FuncRef`; **an unused binding that succeeds**; and **a
  twice-called binding that succeeds.**

## Acceptance criteria

- **AC-1 — the representation matches the ruled shape**: two binding
  discriminants, the four `StaticWorkerBinding` fields, captures typed
  `LoweringOperand`. *Control:* read the type. A third operand arm, a new
  `Lowered` callable arm, or a side table fails outright.
- **AC-2 — no runtime representation of a worker exists.** *Control:* the
  binding carries no word, tag, layout, vtable, descriptor pointer, or callable
  identity, and no carried capture is phase-converted anywhere on the path.
- **AC-3 — `StaticWorker` cannot escape the callee-only arm.** *Control:* the
  escape companions in `D8` — result, aggregate field, and value position —
  each **fail closed**, and the match on the sum has **no wildcard**.
- **AC-4 — the independent witness contains no continuation machinery and runs
  end to end.** *Control:* grep the fixture for `ComputationalMatch`,
  `ContinuationSpecializationId`, `ContinuationCallIdentity`, and continuation
  descriptor/token spellings — all absent — and the test links and **executes**,
  rather than asserting over a planner census.
- **AC-5 — the negative controls bite.** At least one mutation **restores
  current carried-capture narrowing and reds the positive**, and at least one
  **redirects a same-shape target and reds behavior**. *Control:* run both
  mutations and show the red. ⛔ A companion that cannot be made to fail is not
  a control — see `agent/memory/` on negative checks needing a positive
  control.
- **AC-6 — two same-shape workers are genuinely distinguished.** Two
  same-arity, same-capture-count workers at distinct de-Bruijn slots with
  behaviorally distinct bodies and captures, both called; **swapping either
  body or capture order changes the linked result or rejects.**
  *Control:* the swap mutations in `D5`/`D8`.
- **AC-7 — the nested positive depends on BOTH levels.** Its result must depend
  on outer **and** inner body and capture order. *Control:* mutate each
  independently; each must move the result.
- **AC-8 — completion is structural, with no affine ledger.** *Control:* the
  unused-binding and twice-called companions both **succeed**, and no consumed
  set, once-token, or required-empty ledger exists in the diff.
- **AC-9 — no `FuncRef` crosses a function.** *Control:* the fresh
  per-function `FuncRef` companion.
- **AC-10 — continuation specialization is untouched.** *Control:* the
  candidate's path list and diff — no continuation token, key, or descriptor
  changes, no edit to held Activate work, no boundary ABI change, no deletion
  of the consumer route.
- **AC-11 — CI green** on the merge. **Targeted locally only**
  (`COORDINATION §12`) — never `--workspace`, never `--locked` on this box.

## Banned scope

- **No activation of continuation specialization**, and no change to its
  tokens, keys, or descriptors. Activate remains the sole future consumer of
  the four-field claim.
- **No edit to held `RT-CONTSPEC-ACTIVATE` work**, and **no deletion of the
  consumer route** — that is Activate's to remove when it resumes.
- **No boundary ABI change.** No carrier or aggregate addition.
- **No affine worker ledger** (judgment 3, `AC-8`).
- **No third `LoweringOperand` arm, no new `Lowered` callable arm, no runtime
  worker representation, no inverse phase conversion, no second source walk, no
  binding side map.**
- **No reopening of the preallocated worker-environment representation.** That
  is the Architect's alternative disposition and it is not this node's to take.
- **No merge, rebase, or wholesale cherry-pick of any preserved object.**
- **No test asserting facts about source or documentation lines** (operator
  test policy). Every control here is behavioural.

## Contention

Runtime is single-threaded and this node holds the shared build turn for its
witness runs. **Targeted only** — `scripts/ken-cargo` scoped to the crate or
suite you touched. Check `df -h /workspaces` before taking the lock, and do not
reclaim scratch while another seat holds the build turn.

## Sizing

**Size `L`.** Eight deliverables, but they are one contract the Architect ruled
through completion — the size is real work, not unresolved design. The risk is
not the representation, which is closed; it is **judgment 3** (importing
Activate's affine instinct) and **judgment 5** (reaching for continuation
machinery to demonstrate the thing).

⇒ **Commit at these four checkpoints and post the exact SHA at each:**

1. `D1` environment sum threaded, `D2` construction route with all six
   pre-installation rejections.
2. `D3` sole consumer and non-escape, `D4` function-local transport.
3. `D8` the independent witness plus its mutation companions. **Post the
   witness fixture's content with this checkpoint** — judgment 5 is the one a
   reviewer must see directly rather than take on report.
4. `D5` multiple workers, `D6` nested worker, `D7` structural completion.

**Expect to end your turn at each checkpoint.** If any checkpoint runs past an
hour, stop and route.

## Hard stops

Stop and route to the Steward, do not improvise, if any of these hold. The
Architect named the first seven; each means the ruled contract does not fit the
source, which is a finding, not a thing to work around.

1. **A third operand arm** is needed.
2. **A new `Lowered` callable arm** is needed.
3. **A runtime worker representation** is needed.
4. **An inverse phase conversion**, a **second source walk**, or a **binding
   side map** is needed.
5. **A non-projective planner or ABI fact** is needed — something the existing
   bundle does not already carry.
6. **The independent witness cannot be built without continuation machinery**
   (judgment 5). This is the most important stop in the node.
7. **An affine worker ledger** appears necessary to make completion work.
8. **Reopening the preallocated aggregate** appears necessary. That is the
   Architect's second disposition and an operator-visible fork — route it, do
   not take it.
