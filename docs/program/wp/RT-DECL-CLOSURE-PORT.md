# RT-DECL-CLOSURE-PORT — port transparent declaration closures to callable units

**A transparent declaration whose body is a closure seed is a retained residual,
and *any* retained residual routes the **whole object** to the monolithic
`RecursiveDescent` root. That root now exceeds Cranelift's per-function ceiling.
This node makes those declarations separately owned callable units so the
residual can be retired.**

**Owner:** Team Runtime (`runtime-leader` + `runtime-implementer` +
`runtime-qa`). **Branch:** `wp/RT-DECL-CLOSURE-PORT`. **Size:** L.
**Risk:** high — this is the last B2F migration seam, and its acceptance is a
*behavioural* compile, not a code-shape assertion.

⛔ **Read `docs/program/16-recursive-descent-retirement.md` first.** This node is
the **keystone** of that seven-node campaign, and the campaign doc carries the
three traps that bind every node in it — including **Trap 3**, which this frame's
`AC-2` now discharges explicitly. This frame predates the campaign doc and does
not repeat its content.

**Status:** Steward frame, shovel-ready.

## ⭐⭐ REORDERED 2026-07-29 — THIS NODE IS NOW **NEXT**, NOT THIRD

**Steward disposition `evt_5mtkdft1nxmwp`, on [[NATIVE-HANDLE-CARRIER]]'s hard
stop.** The order was `RT-JOIN-DISPOSITION → NATIVE-HANDLE-CARRIER → this node`.
`RT-JOIN-DISPOSITION` merged (`main = af056a78`); **`NATIVE-HANDLE-CARRIER` then
hit this node's `AC-1` row** and is preserved at
`85dcee259dc65f9e3c1d625c0ee0ed8342577492` (tree `b7cf9041`) pending this node.
⇒ **`RT-JOIN-DISPOSITION` → this node → `NATIVE-HANDLE-CARRIER` resume.**

⭐ **The premise that put `NATIVE-HANDLE-CARRIER` first was measured false.** It
was *"NHC is 5/6 green and cheap to finish."* NHC's own `AC-1` — full parity, no
partial — is **unreachable on any tree** until the ceiling below falls. It is not
blocked on a formality; it cannot complete.

⚠ **The cost that is real and is not hidden:** this node rewrites `core.rs`, so
`85dcee25` needs a **second** rebase on resume. Its `D1` already proved that
machinery on this exact branch (`git range-diff` 3/3 `=`, no side choice, four-file
provenance settled), so the cost is bounded.

⭐ **Reversing this is cheap:** run `NATIVE-HANDLE-CARRIER` first instead and it
stops again at the same row. That is what makes the reorder safe rather than a
preference.

⭐ **On the Linux ABI I critical path.** Sole blocker of [[PX8-ERRID-ALLOC]] →
[[PX8-ERRID-SCOPE]] → [[PX8]]; `PX8` gates 15 of that program's 19 nodes.

---

## 1. Fixed inputs

| path | blob at `origin/main = eefca112` |
|---|---|
| `crates/ken-runtime/src/cranelift_backend/lowering/core.rs` | `f7bc0d0354d8b8d6f7aa68176846b7b05e5a8514` |
| `crates/ken-runtime/src/cranelift_backend/lowering/units.rs` | `f57215905ad715cab67b580781d078a614e20dfd` |
| `crates/ken-cli/tests/rt_parity_native.rs` | `b2df2bbd00644b907cae5d05efa76edd9df1b3f2` |

**Grounding:** Architect ruling `evt_3t7t27e3rv8cx`, measured in a detached
scratch worktree with diagnostic-only labels against exact `ad7298fb`
(tree `77ece013`). ⭐ No candidate or production change survived that worktree —
the diagnosis cost nothing and left nothing behind.

## 2. The mechanism, at exact anchors

**The selector** — `core.rs:181-197`, `select_body_emission_authority`:

```rust
if recursive_descent_residual(expr)
    .or_else(|| declarations.values().find_map(declaration_recursive_descent_residual))
    .is_some()
{ BodyEmissionAuthority::RecursiveDescent } else { BodyEmissionAuthority::FunctionizedUnits }
```

⇒ **Whole-program, all-or-nothing.** One residual anywhere and the entire object
takes the monolithic root.

**The residual this node retires** — `core.rs:155-172`,
`declaration_recursive_descent_residual`: a `RuntimeDeclarationKind::Transparent`
whose `body` is `RuntimeExpr::Closure { .. }` or `LexicalClosure { .. }` yields
`RecursiveDescentResidual::TransparentDeclarationClosure` (`core.rs:56`, minted
at `:163`).

**What the Architect measured on the failing fixture:**

```text
authority=RecursiveDescent
residual=TransparentDeclarationClosure
declaration=...::buffer_nat_to_int residual=TransparentDeclarationClosure
declaration=...::main             residual=TransparentDeclarationClosure

PX8_ERRID_DIAGNOSTIC RecursiveDescent root:
Compilation error: Code for function is too large
```

⇒ The oversized function is **the `RecursiveDescent` root itself** — not a
functionized unit, not the root adapter, not a fixed helper graph.
**`FunctionizedUnits` declares and defines *zero* semantic units on this route.**

## 3. ⭐⭐ THE TRAP — retiring this residual may not change the authority

⛔ **`TransparentDeclarationClosure` is ONE OF FIVE** residual variants
(`core.rs:41-57`): `ProducerMatchCall`, `MatchScrutineeRecursor`,
`LexicalCallArgumentRecursor`, `SeedClosureCall`, `TransparentDeclarationClosure`.
The selector takes the **first** residual it finds, and the expression walk
(`recursive_descent_residual(expr)`) is consulted **before** the declaration walk.

⇒ ⭐ **Retiring this one residual does not entail that the fixture reaches
`FunctionizedUnits`.** Another variant may be present and simply unreported,
because the selector short-circuits at the first hit.

**This is why `AC-1` is a compile, not a code-shape assertion.** A deliverable
that removes the residual and reports success while the fixture still fails is
the exact failure mode this section exists to prevent.

⚠ **Before building anything, measure the full residual set** on the fixture —
enumerate *every* variant that fires, not the first. If others fire, ⛔ **stop
and report**: the node's scope is then wrong and it is mine to re-cut, not yours
to widen.

## 4. Deliverables

- **`D1`** — **Full-residual diagnostic first.** A temporary or permanent
  enumeration reporting every residual variant present on a given program, not
  the short-circuited first. Run it on the fixture and record the complete set.
  ⛔ Do not proceed to `D2` until `D1`'s result is posted.
- **`D2`** — **Planner-owned callable declaration units.** Transparent
  closure-seed declarations become separately owned callable units rather than
  bodies recursively lowered into the generated root.
- **`D3`** — **Typed capture / parameter / result / trap transport** across that
  unit boundary.
- **`D4`** — **`DeclarationRef` calls** to those units
  (`core.rs:148`, `:7238` are the existing reference sites).
- **`D5`** — **Complete owner/phase validation**, in place **before**
  `TransparentDeclarationClosure` is removed from the retained residual.
- **`D6`** — Remove the residual variant, and only then re-run `AC-1`.

### ⭐⭐ `D7` — THE CLOSED `Carried`-CONSUMER MATRIX

**Added 2026-07-29, Architect `evt_6h6vzqw7ydra8`.**

⛔ **This is one deliverable, not two repairs, and it lands ATOMICALLY with
`483ef7ab`.** ⛔ **Do not file the two observed refusals as separate nodes.**

**Why the matrix is the unit.** `LoweringOperand::Carried` is **not** new here —
`C1` introduced it. What this node newly activates is a **production seed**: a
declared-unit result now enters the already-ruled phase-bearing lowering graph.
`C1 §2h`'s governing contract is transitive **full phase closure** across
environment insertion, recursive call, branch/join forwarding, and result
propagation, ⛔ **with no reachability whitelist.**

⚠ **The implementation does not mechanically close that population.** At exact
`483ef7ab` there are **at least 24** explicit calls to `specialized_at`,
`specialized_ref_at`, `specialized_env_at`, or `specialized_join_arm`, plus
direct exhaustive `LoweringOperand` matches. Their free-form diagnostic strings
identify sites but prove **neither population closure nor lawful disposition**.

⭐⭐ **The seven parity rows reached two sites BY ACCIDENT. Two fixes would
establish only that two witnesses moved.** ⇒ **The reached set is evidence,
never the population.**

**The ruling:** derive and validate **one closed consumer-edge matrix**. Every
edge that can receive a lowering operand has **exactly one** disposition; every
disposition names a **real** edge; each is consumed **exactly once**. ⛔ A
missing edge must fail **planning/compilation before `ObjectEmission`**.

#### The required partition — names may vary, this partition may not

| edge | rule |
|---|---|
| **Forwarding** | environment insertion, recursive lowering, result propagation, planned join. `Carried` passes **unchanged**; a join keeps using the ruled `JoinResultRepresentation` token — ⛔ do not invent a second phase plan |
| **Callable-capture** | an **invocation-local compiler control capsule** retains static body/parameter identity while **each captured operand retains its own phase**. ⭐ This is forwarding plus an owner/lifetime invariant — ⛔ **not a value representation** |
| **Semantic eliminator** | `Match`, `ComputationalMatch`, `Project`, or anything that semantically **reads** the value. `Carried` takes **one** emitted-helper route; `Specialized` keeps its existing route |
| **Specialized-only leaf** | lawful **only** where the planner **proves** no `Carried` seed can reach that exact edge under the selected authority. ⛔ *"It has not happened in the corpus"* and a fail-closed `specialized_at` are **NOT** such proofs. A reachable `Carried` edge must gain an emitted semantic route **or** remain a **named selector residual** — ⛔ it may not fail late during emission |
| **Escape-forbidden** | a whole runtime-local closure/control capsule, trap-protocol object, or other non-value **cannot cross a unit/publication boundary**. Refusal occurs **before** allocation, helper invocation, or partial publication |

#### How the population is derived — ⛔ not by grep

Derive the **source-edge** population beside the static-origin/child-occurrence
graph, by an **exhaustive, wildcard-free traversal of every `RuntimeExpr`
variant and every operand-bearing child role.** Lowering-only edges with no
source child use a **separate closed enum**. ⛔ Free-form strings remain
diagnostic labels only. ⭐ **The only route into `specialized_at` /
`specialized_ref_at` / `specialized_env_at` must consume the exact typed
disposition for that edge.**

**Validation owes a bijection and exact-once consumption *before function
definition*:** every planned operand edge exists · every real edge is planned ·
⛔ no duplicate, wrong owner, wrong phase, omitted edge, or unconsumed token.
⭐ **Adding a `RuntimeExpr` form must break the exhaustive planner; adding an
operand role to an existing form must break the edge/child-role closure or its
omission control.**

#### Row: the closure-capture cell (`core.rs:7370`)

⛔⛔ **This refusal is NOT authority to** restore `PersistentClosure`, mint a
carried closure, decode a carrier into a closure template, or pass a whole
closure through the declared-unit ABI. **The closure boundary still forbids all
four.**

Lawful representation — an **invocation-local compiler control capsule**:

- the outer `Closure` / `DeclarationClosure` stays `LoweringOperand::Specialized`
  and retains **only** static body origin, params, callable identity;
- its **`captures` edge becomes phase-bearing** (`LoweringOperand` per capture),
  so a carried capture **stays carried**;
- the capsule is **unconditionally non-transferable**, and the admissibility /
  escape check rejects it **before inspecting or emitting its captures**;
- on `FunctionizedUnits`, the planner-selected callable target receives
  parameters **then** captures through the **existing** typed unit-input ABI;
  carried captures pass as their existing word, specialized captures cross the
  one-way producer **exactly once**;
- owner, phase, capture position/order, invocation lifetime, result and trap
  validation remain **exact**.

⚠ **This is a narrow amendment to `C1`'s former "single-field license".** The
governing distinction remains **phase identity, not transitive Rust
containment.** The capture is still a `LoweringOperand`: ⛔ it does **not**
acquire a `LoweredVariant`, `BoundaryDisposition`, encoding policy, inverse
conversion, carrier tag, durable slot, or independent callable identity.
⇒ `C1`'s text saying every `Closure`/`DeclarationClosure` child stays `Lowered`
is **superseded for capture edges only**, by the newly proved reachable
population.

#### Row: the producer-`Match` cell

The mechanism from `evt_5catd48dv8db6` **remains correct**, but as **one matrix
row — not separate ownership.** The carried-`Match` dispatcher gets **ordinary
and producer continuation modes** sharing class / tag / field-count / case /
default / arity / static-origin / join handling. Producer mode lowers the
selected ordinary-`Match` case with `lower_computational_producer_expr` **under
the complete remaining eliminator stack**. ⛔ No eager ordinary body followed by
an outer merge; ⛔ no reconstructed `Lowered` template.

⚠ [[RT-PRODUCER-MATCH-PORT]] and [[RT-SEED-CALL-PORT]] **still own their
syntactic residual retirements later** — ⛔ but **not** this repair, and ⛔ not a
second capture transport. ⭐ Their `D1` may find the transport already present.

#### ⛔⛔ Atomicity — why this cannot be a separate node

`RT-DECL` alone **regresses existing green rows**; a consumer node **cannot**
merge first with a reaching production witness; and `RT-DECL` **cannot** merge
first. ⇒ ⭐ **A nominal graph node with no independent safe merge boundary is a
label, not a node.** One candidate, one merge.

## 5. Acceptance criteria

- **`AC-1` (the only one that decides the node).** `scripts/ken-cargo test -p
  ken-cli --test rt_parity_native
  fs_write_at_malformed_offset_narrows_to_invalid_offset` **compiles and passes**.
  ⛔ Not "the residual is gone" — **the object builds.**

  ⛔⛔ **AMENDED 2026-07-29 — `AC-1` NOW REQUIRES THIS ROW GREEN ON *TWO*
  INDEPENDENT DELTAS, NOT ONE.** As originally written it read *"on a tree
  carrying `ad7298fb`'s semantic delta"* — **Foundation's delta only.**
  1. A tree carrying **`ad7298fb`**'s semantic delta ([[PX8-ERRID-ALLOC]]).
  2. A tree carrying **`85dcee25`**'s semantic delta ([[NATIVE-HANDLE-CARRIER]],
     the `Resource Buffer → BufferHandle` carrier migration).

  ⭐ **`ad7298fb` IS the measurement object for delta 1 — `e65c81b5` is NOT to be
  run, and that is settled, not a shortcut.** `e65c81b5` is the pre-rebase
  Foundation WIP; it sits on a **pre-repair** tree, and applying the `D1`
  enumerator to it conflicts in `core.rs`. ⛔ **Do not hand-resolve that conflict
  to "complete" this AC** — a resolution chosen so a row passes measures the
  resolver, which is the defect that rejected `27f9dca2`.
  ⇒ Equivalence was **measured, not asserted**: `git range-diff
  5404108a..e65c81b5 eef0cb06..ad7298fb` maps the three-commit series in order
  with final `e65c81b5 = ad7298fb` **patch-equivalent** (`D1` report,
  `evt_24dbrgg36w6by`, 2026-07-29). ⭐ Recorded here because an in-thread
  justification is not a durable deliverable, and without it a later reader
  checking `AC-1` against `e65c81b5` literally would block on work that was
  correctly done.

  ⭐ **Why the second one was added, and why omitting it would have made the
  reorder worthless.** Both deltas reach this same ceiling independently, and
  `main` alone passes the row (measured: `evt_5mtkdft1nxmwp`). With one delta in
  the AC, **this node could land fully green and `NATIVE-HANDLE-CARRIER` would
  resume and still be red** — after the queue was reordered to fix exactly that.
  ⇒ The single-delta AC does not measure that the ceiling is gone; it measures
  that *one program* got under it.

  ⚠ **A third program is known to sit near the ceiling**:
  `docs/program/issues/CI-SKIPPED-NATIVE-TESTS.md` records this row as the only
  one of seven opening **two nested resource brackets** where every sibling opens
  one, and as the 250.5s timing outlier. ⛔ Do not add it as a third required
  delta — two independent deltas is the control; ⭐ **but if either delta needs a
  size concession to pass, that is a reportable finding that this node did not
  remove the ceiling, only lowered the program under it.**
  ### ⛔⛔ `AC-1` IS FACTORED — 2026-07-29

  On the Architect's recommendation (`evt_5catd48dv8db6`).

  **`AC-1` as written bundles two properties, and only the first is this node's.**

  | | property | owner | state |
  |---|---|---|---|
  | **`AC-1a`** | the `RecursiveDescent` ceiling no longer takes these programs — **both** deltas select `FunctionizedUnits` with `residuals=none` | **this node** | ✅ **DISCHARGED** |
  | **`AC-1b`** | the same two rows **compile and pass** end to end | **this node's `D7` consumer matrix** (below) — it lands **atomically** with `483ef7ab` | ⛔ **not yet discharged** |

  ⚠ **`AC-1b`'s owner was re-ruled on 2026-07-29.** It was briefly filed to
  [[RT-PRODUCER-MATCH-PORT]] (`evt_5catd48dv8db6`); the matrix ruling
  (`evt_6h6vzqw7ydra8`) **superseded that on per-cell ownership** and returned it
  here as `D7`. ⛔ Do not route it to a successor node.

  ### ✅ `AC-1a` DISCHARGED 2026-07-29 at `483ef7ab` (tree `b41794b4`)

  ```text
  RT_DECL_CLOSURE_PORT_D1 authority=FunctionizedUnits
  RT_DECL_CLOSURE_PORT_D1 residuals=none
  ```

  Measured on **both** required deltas — Foundation `ad7298fb` and NHC
  `85dcee25` (detached, no-commit, no preserved ref moved) — each then reaching
  the **identical** downstream refusal (`evt_69ebt7hwg8508`).

  ⭐ **Why two deltas agreeing is the point.** One delta would show *one program*
  got under the ceiling. Two independent deltas, both clearing every residual and
  both stopping at one shared downstream wall, show **the ceiling moved**. That is
  exactly what the two-delta amendment above was added to distinguish, and it is
  the only reason this node can be said to have succeeded at all.

  ⚠ **This is not a code-shape claim.** It is a selector outcome on the exact
  governed deltas, from an instrument carrying a **causal** mutation control
  (`AC-2`, below). ⛔ It is equally not a claim that the programs run — that is
  `AC-1b`, and it is somebody else's.

  ### ⛔⛔ `AC-1b` CANNOT BE DISCHARGED HERE — THE NODE IS HELD

  ⛔ **Do not route `483ef7ab` to QA.**

  **Measured 2026-07-29 (`evt_1b1v2qjy82epm`):** targeted `rt_parity_native` on
  clean `483ef7ab` with **neither** delta applied is **1/7**.

  - **five** rows that are **green on `main` today** hit the producer-`Match`
    carried-scrutinee population (Architect-filed under
    [[RT-PRODUCER-MATCH-PORT]], hard stop **#22**);
  - one — `buffer_allocate_malformed_capacity_narrows_to_invalid_bounds` — hits a
    **distinct carried closure-capture** refusal (hard stop **#23**, owner
    classification open).

  ⇒ ⛔ **The port is NOT additive: it regresses `main`.** `AC-4` (workspace green
  in CI) is therefore unreachable, and this node **cannot become a candidate**
  until its consumers are complete. ⛔ No QA route, no fresh review, no approval.

  ⭐ **The baseline proxy that found this was not in the original plan and it
  changed the outcome.** Every measurement before it carried a delta, so the
  regression was invisible to all of them — and the factoring above was, before
  it ran, about to be used to argue this node could land. ⚠ **A port measured only
  with its consumers' inputs applied has not been measured against `main`.**

  ### ⭐⭐ WHAT THIS NODE OWES ITS SUCCESSORS — the consumer-side enumeration

  This node introduced a **new representation** — a declaration result crossing
  the callable-unit boundary as `LoweringOperand::Carried` — and **did not
  enumerate the consumers that must eliminate it.** Both refusals above are the
  same cell type (*a `Carried` value reaching a consumer built only for
  specialized shapes*), found in two different consumers, and found **only**
  because seven parity rows happened to reach them.

  ⛔ **Two cells found by two test rows is not a bounded population.** The
  producer side got an enumerator with an omission-reds control; ⭐ **the consumer
  side owes the same discipline and does not yet have it.** See
  `docs/program/16-recursive-descent-retirement.md` §4.

- **⭐⭐ `AC-7` — `D7`'s evidence net (Architect `evt_6h6vzqw7ydra8`). All eight.**
  1. Record the **complete static edge matrix** and a **separate** reached-edge
     trace. ⛔ **Do not equate them.**
  2. ⭐⭐ **THE LOAD-BEARING CONTROL — omit one *real* member** (closure capture,
     producer `Match`, **or one internal forwarding edge**) and prove validation
     **reds before emission.** ⚠ Without this, every other item on this list
     passes over whatever population the instrument happens to see.
  3. Replace producer-`Match` producer mode with ordinary mode → recreate the
     **exact** refusal / wrong continuation.
  4. Re-specialize a carried closure capture → recreate the **exact**
     `BoundaryCarrier` refusal.
  5. Prove carried capture **order, owner, phase, invocation lifetime** —
     perturb **each independently**.
  6. Prove invalid carrier class/tag/arity/default **and** whole-closure escape
     still fail closed **before** side effects or publication.
  7. Pin that **all current unwrapping sites consume typed dispositions**;
     adding an unplanned unwrapping **cannot compile or validate**.
  8. **Baseline targeted parity returns 7/7**, and the exact Foundation and NHC
     governed rows compile and pass.

  ⛔ **Still separately required and NOT subsumed by the above:** `C1` carrier
  controls · `RT-JOIN-DISPOSITION` controls · exhaustive residual enumeration ·
  `AC-6` · the remaining mutations · QA · a **fresh exact-SHA review**.

- **`AC-2`.** `D1`'s complete residual enumeration is recorded in the tree, with
  the fixture's full set named. If the set is larger than
  `{TransparentDeclarationClosure}`, that is a reportable finding, not a silent
  scope widening.

  ⛔⛔ **"Complete" is asserted here, and it owes TWO POSITIVE CONTROLS.** This
  AC as written passes on whatever set `D1` happens to emit — including a set
  that is missing a variant, which is campaign **Trap 3**
  (`docs/program/16-recursive-descent-retirement.md`).
  1. **The enumerator must be shown NOT to short-circuit.** Run it on a program
     that fires **two or more** variants and show it reports **all** of them.
     ⭐ This is the control that matters most: the enumerator's entire purpose is
     to defeat the selector's `.or_else(...)` short-circuit, and if it silently
     retained that behaviour it would report **exactly one** variant — which is
     precisely the result everyone expects to see, so nothing would look wrong.
  2. **Each variant must be shown reachable by the instrument.** For every one of
     the five variants, name a program the enumerator reports it on. A variant no
     program in the corpus reaches is a **reportable gap in the measurement**, not
     a variant that does not fire.

  ⚠⚠ **This instrument is the most leveraged object in the campaign.** The
  campaign doc directs that it be built **once here and reused** by
  [[RT-SEED-CALL-PORT]], [[RT-PRODUCER-MATCH-PORT]], [[RT-RECURSOR-TRANSPORT]]
  and [[RT-DESCENT-RETIRE]]. ⇒ A gap in it does not stay local — **every
  downstream node inherits it**, and [[RT-DESCENT-RETIRE]]'s "no residual fires
  anywhere" would then be vacuous at exactly the moment it authorizes deleting
  the lane.

  ### ✅ `AC-2` DISCHARGED 2026-07-29 at `93a6903b` (tree `c11bb8b0`)

  `D1` report `evt_24dbrgg36w6by`, accepted by `runtime-leader`
  `evt_3a595dcnam7f8`. **Both positive controls landed, and the first is
  *causal*, not asserted:**

  1. **Does not short-circuit** — the control exercises all five individual
     witnesses **plus a compound all-five population**, `1/1` green. Mutating the
     `report` visitor to short-circuit (`continue true → stops false`) turns it
     **red at the compound assertion**, observing only
     `{ProducerMatchCall, TransparentDeclarationClosure}` instead of five. Exited
     101; restored byte-identically.
  2. **All five variants reachable** — each has a named witness in the control.

  ⭐ **This is the campaign's shared instrument, now controlled.** Downstream
  nodes ([[RT-SEED-CALL-PORT]], [[RT-PRODUCER-MATCH-PORT]],
  [[RT-RECURSOR-TRANSPORT]], [[RT-DESCENT-RETIRE]]) inherit it **and this
  evidence** — ⚠ but re-prove it cheaply at each point of use, since `D2`–`D6`
  rewrite `core.rs` underneath it.

  **Scope fork result — the hard stop did NOT fire.** Both governed deltas select
  `authority = RecursiveDescent` and report **only**
  `TransparentDeclarationClosure`, on `buffer_nat_to_int` and `main`, then reach
  the known size failure. ⇒ ⭐ **This node is confirmed as the fix for both held
  candidates**, which is what the reorder assumed and did not wait for.
- **`AC-3`.** `D5`'s owner/phase validation is present and fails closed **before**
  `D6` lands. A commit ordering that removes the residual first fails this AC.
- **`AC-4` (no-regression).** Workspace green **in CI** — ⛔ never a local
  `--workspace` run (`COORDINATION §12`).
- **`AC-5`.** The exhaustive-match fail-closed property at `core.rs:59-65` is
  preserved: a new `RuntimeExpr` form must still be unable to compile until the
  classifier assigns it. ⛔ Do not replace the exhaustive match with a wildcard.
- **`AC-6` — ⭐ MEASURE THE ROOT'S COST. Added 2026-07-29 (operator: this part
  of the compiler must be both correct *and* efficient).** Record, for the
  fixture, the **emitted function count** and the **per-function code-size
  distribution** under each authority — the `RecursiveDescent` root before, and
  the `FunctionizedUnits` population after. Post the table.

  ⭐ **Why this AC exists:** [[RT-SCALE-B]] returned verdict **(a)** — linear,
  no exponent — but it was **bounded to the governed recursive resource-bracket
  populations and excluded the mutually exclusive `RecursiveDescent` root**
  (Architect, `evt_3t7t27e3rv8cx`). ⇒ **The monolithic root has never been
  scale-measured.** `"Code for function is too large"` is that unmeasured cost
  surfacing as a hard ceiling instead of as a curve. This node is the first
  point where both authorities can be measured on the same program.

  ⛔ **Report the measurement; do not tune to a threshold, and do not pin a
  number.** No target figure is set here and none may be inferred — a pinned
  size number would rot at the next merge, and the AC is discharged by the
  table existing and being routed to the Steward, not by any value in it.
  ⛔ A regression in either figure is a **reportable finding**, not a licence to
  widen this node's scope.

## 6. ⛔ Banned scope

- ⛔ **Deleting the selector residual** without the port. Named and banned by the
  ruling as an unproved shortcut.
- ⛔ **Selectively inlining fewer declarations.** Same — banned by name.
- ⛔ **A second [[PX8-ERRID-ALLOC]] size reduction.** The feature delta is
  **exonerated**; shrinking its identity mapping trades semantics for bytes.
- ⛔ **Retiring the other four residual variants.** If `D1` shows they fire,
  report it — do not absorb them. Each is its own migration seam.
- ⛔ **Reopening [[RT-NATIVE-FNSPLIT]] or [[RT-SCALE-B]].** Both closed on gates
  that were met. `RT-SCALE-B` explicitly excluded the `RecursiveDescent` root,
  which is why its verdict is untouched by this.

## 7. Hard stop

Report and stop if `D1` shows residuals beyond `TransparentDeclarationClosure` on
the fixture, or if the port lands and `AC-1` still fails **on either delta**.
⛔ Do not attempt a size reduction in either case.

**§5a count of record: 21**, entries **12**, next predicate check the **15th
entry**, next research pull **#24** — carried on [[NATIVE-HANDLE-CARRIER]].
⛔ The `NATIVE-HANDLE-CARRIER` stop that reordered this node is **not** #22: it
routed a red row to the node that already owned it, and no new mechanism failed.

## 8. What landing this closes

**Two held nodes, not one.**

- [[PX8-ERRID-ALLOC]] is released the moment this merges — its candidate
  `ad7298fb` is rebased and preserved, and Foundation owes no rebuild, only a
  re-run. That in turn releases [[PX8-ERRID-SCOPE]] and clears the last of `PX8`'s
  three blockers on this path.
- [[NATIVE-HANDLE-CARRIER]] resumes from preserved `85dcee25` — **11 of 12
  `rt_parity_native` rows already green**, `D1` rebase complete, the identity arm
  re-derived. It owes a second rebase over this node's `core.rs` rewrite, then
  `AC-2`'s Big-identity mutation and the two `AC-4` positive-red controls, which
  the hard stop pre-empted.

⭐ **The selector's own doc comment calls it "the one *temporary* B2F migration
selector" (`core.rs:174`).** This node is that migration finishing, not a new
mechanism.
