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

### ⭐⭐ `D7` — THE CLOSED BOUNDARY-OPERAND SEMANTIC-CLOSURE MATRIX

**Added 2026-07-29, Architect `evt_6h6vzqw7ydra8`. ⭐ RECUT 2026-07-29 at hard
stop #24, Architect `evt_3ayvrada4c0nj`.**

⛔ **This is one deliverable, not three repairs, and it lands ATOMICALLY with
`483ef7ab`.** ⛔ **Do not file the observed refusals as separate nodes.**

> #### ⛔⛔ RECUT — THE MATRIX WAS NAMED TOO NARROWLY. READ THIS BEFORE THE ROWS.
>
> `D7` was framed as a **`Carried`-consumer** matrix. Its own exhaustive
> derivation then exposed hard stop **#24** — a real edge outside all five
> dispositions — and the Architect ruled that the *framing*, not just the
> partition, was wrong.
>
> ⭐⭐ **The governing predicate (entry-15 answer — YES, one predicate).** For
> every operand edge `e` that may cross a function-unit **owner** or lowering
> **phase**, define:
>
> - **`Need(e)`** — what the downstream semantic operation can observe;
> - **`Avail(e)`** — what the selected ABI/disposition guarantees after crossing.
>
> > **The closed obligation is `Need(e) ⊆ Avail(e)`, or planning must ELIMINATE
> > that runtime edge completely before emission.**
>
> This is **owner/phase semantic sufficiency**, and it classifies all three stops
> as one family:
>
> | stop | `Need(e)` | why it failed |
> |---|---|---|
> | #22 | constructor identity, field order, arity | specialized-only read had none for a carried scrutinee |
> | #23 | each capture's value, phase, order, owner, lifetime | the capsule asserted captures were specialized |
> | #24 | callable **body identity** plus captured state | `Forwarding` can move only the whole capsule — forbidden |
>
> ⇒ ⭐ **These are representation-contract defects, not three syntax-site bugs**,
> and ⛔ they do **not** imply one universal runtime carrier. Some obligations are
> met by a runtime representation; **#24 is met by proving the callable edge
> ABSENT from the runtime ABI.**
>
> ⛔⛔ **The concrete matrix defect is narrower than "`CallArgument` is wrong":
> SOURCE-CHILD ROLE ALONE DOES NOT DETERMINE DISPOSITION.** An ordinary value
> argument and a statically known callable whose identity will be invoked have the
> **same syntactic role and different semantic needs**. ⇒
> `SourceOperandRole::disposition(self)` is **insufficient authority**. Planning
> must derive a typed semantic obligation from the **exact parent, child, callee
> target, parameter ordinal, and downstream parameter-use closure**. ⭐ Role
> remains **inventory data, not the verdict**.

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

#### The required SIX-way partition — names may vary, this partition may not

⛔ **The five-way partition is SUPERSEDED.** The new lane may **not** be
collapsed into `Forwarding`, `CallableCapture`, or `SpecializedOnlyLeaf`.

| edge | rule |
|---|---|
| **Forwarding** | environment insertion, recursive lowering, result propagation, planned join. `Carried` passes **unchanged**; a join keeps using the ruled `JoinResultRepresentation` token — ⛔ do not invent a second phase plan |
| **Callable-capture** | an **invocation-local compiler control capsule** retains static body/parameter identity while **each captured operand retains its own phase**. ⭐ This is forwarding plus an owner/lifetime invariant — ⛔ **not a value representation**. ⭐ **Gained one new member #26** (`StaticRecursorWorkerResidual`) on 2026-07-29 — see the row below; ⛔ the member is new, the **disposition is not** |
| **`StaticCallableElimination`** ⭐ NEW (#24) | a statically known callable passed as a transparent-declaration argument. ⛔ The callable edge is **eliminated from the runtime ABI** — planner-owned, **out-of-line** callee specialization with **lifted captures**. Lawfulness conditions below; ⛔ this is **not** the prohibited specialized fallback |
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

#### ⭐⭐ Row (#24): callable as declaration argument

**The measured edge.** Source occurrence **`StaticOriginId(1031)`**:
`LexicalClosure { captures: [Var(0)..Var(8)], params: ["arg0"], … }` supplied as
a transparent declaration `CallArgument`. Classifying it `Forwarding` makes
`call_declared_unit_target` transfer the **whole capsule** into the declared-unit
ABI, and `boundary_transfer_admissibility` **correctly** refuses before
allocation/publication.

**Lawful only when ALL of these are planner-proved before any function
definition:**

- the call target is **one exact** transparent declaration/callable unit;
- the callable argument resolves to **one static** `Closure`/`LexicalClosure`
  body origin — ⛔ **never** to a runtime-observed `Lowered` value;
- the callable parameter's **complete use-closure** is known: invocation consumes
  the static binding · forwarding to another transparent specialized call
  propagates it · an unused binding is **erased** · ⛔ return, storage,
  construction, effect passage, comparison, or any other value use is **rejected
  before ABI allocation**;
- **every** captured operand has a closed **lifted environment**: ordinary
  captures get typed ABI slots; a statically known callable capture is
  **recursively** a static binding with its own lifted captures; ⛔ a dynamically
  selected callable **cannot enter this lane**.

⛔⛔ **This is NOT the prohibited specialized fallback.** It is selected **in the
static plan**, has a complete emitted body and ABI, ⛔ never invokes
`specialized_at`, ⛔ never inspects a JIT-time value to choose code, and ⛔ never
falls back to recursive descent after `FunctionizedUnits` emission has begun.

##### Exact emitted shape

**Interned specialization key — compile-time identities only:**

```text
(base callee owner/origin,
 ordered [(parameter ordinal, callable body origin, declared arity, capture provenance) …])
```

⛔⛔ **Capture VALUES are not in the key.** Each exact call-site edge maps to the
key; identical keys **may reuse one unit**.

**Runtime ABI of the specialized unit** — separately emitted, **out of line**:

1. the non-callable parameters, **in original parameter order**;
2. then **lifted capture slots**, grouped by eliminated callable-parameter
   ordinal and capture declaration order;
3. the existing result/control/trap/store convention.

⛔ **The callable parameter slot is structurally ABSENT. There is ZERO
code-pointer, tag, selector, vtable, descriptor, trampoline, closure handle, or
other callable-identity word.** Inside the unit, a **compiler-only** callable
binding environment maps that parameter to its static body unit and lifted
capture slots; invocation emits a **direct declared-unit call** with invocation
arguments followed by those captures. ⛔ The base declaration body is **not**
copied into the caller.

**Finite fixed point, ⛔ not clone-on-visit.** Keys range only over finite planned
declaration origins, parameter ordinals, and static callable body origins.
**Intern before enqueue**; recursive use of the same key **reuses the same unit**.
⛔ Checked cardinality/capacity failure is **loud before emission** and never
changes semantics or selects a fallback.

⚠ **This extends the ABI/unit partition, not only the operand disposition.**
`AbiUnitDefinition`'s comment claims its **two seed classes are exhaustive**; a
callable specialization is a planner-derived emitted unit and must become an
**explicit closed arm** with its own owner, descriptor, call edges, and
validation. ⛔ **Do not smuggle it through `SchedulingEntry`, `ClosureBody`, or
`TransparentDeclarationClosure`.**

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

#### ⭐⭐ Row (#26): the static-recursor-worker residual — a LOWERING-ONLY edge

**Added by Architect ruling `evt_5c9ys1my7hr51` (2026-07-29).** ⛔ **This is ONE
NEW MEMBER of the already-ruled six-way matrix — not a seventh disposition, not a
new representation family, and not a new WP.** The partition above is unchanged.

**The measured edge.** Bounded witness on
`fs_read_at_malformed_offset_narrows_to_invalid_offset`, path:

```text
ComputationalRecursorClosure / static-worker residual / Closure / Captures[Carried x7]
```

producer `RecursorProducerOriginId(0)`, sibling `1`, generated-unit owner, worker
`StaticOriginId(723)`, arity `1`, `dynamic_splices=1`, `open_obligations=0`,
pre-allocation parent/descriptor counters `0 / 0`.

| axis | value |
|---|---|
| edge kind | **lowering-only** — no source child ⇒ it lives in the **separate closed enum**, per *How the population is derived* above |
| suggested name | `StaticRecursorWorkerResidual` |
| **disposition** | **existing `CallableCapture`** |
| keyed by | exact **recursor parent** · **producer origin** · **sibling / recursive position** · **worker body origin** · **declared arity** · **capture provenance** |
| mechanism owner | ⭐ [[RT-RECURSOR-TRANSPORT]], **not this node** — see its §8 |

⛔ **NOT `StaticCallableElimination`.** There is **no callable declaration
parameter / use-closure** here; the recursor plan **already selects the direct
worker**. ⛔ Not `Forwarding`. ⛔ **It does not make the `Closure` a value** — it
retains body/parameter identity in **compiler plan material** while the ordered
captures retain their own phase.

⛔⛔ **THE GENERIC ARM IS UNCHANGED.**
`LoweringOnlyOperandEdge::CallableCapsuleEscape -> EscapeForbidden` remains
correct for **every unproved or value-position whole `Closure`.** ⛔ Do not
relabel it, and ⛔ do not weaken it to admit this member. ⭐ **#26 is an exception
proved per-edge by a planner token, never a softening of the default** — the
control that holds that line is [[RT-RECURSOR-TRANSPORT]]'s sharpened `AC-15`
whole-`Closure` negative pair.

**Matrix omission control (this node's obligation).** Omit or reclassify the real
static-worker-residual member and **planning must fail before function/object
emission.** ⛔ It **may not fall through** to the late generic `Closure` refusal —
⭐ a late refusal is the **wrong failure**, and it would satisfy a naive "it still
rejects" assertion while proving the matrix is *not* closed.

⚠ **The lawful transport for this edge is specified in
[[RT-RECURSOR-TRANSPORT]] §8c, not here** — five validation obligations that must
all discharge **before any allocation**, plus the `Carried x7` carrier-`Record`
envelope rule. ⛔ Do not restate it in this frame; one operative statement, one
owner.

#### ⛔⛔ Atomicity — why this cannot be a separate node

`RT-DECL` alone **regresses existing green rows**; a consumer node **cannot**
merge first with a reaching production witness; and `RT-DECL` **cannot** merge
first. ⇒ ⭐ **A nominal graph node with no independent safe merge boundary is a
label, not a node.** One candidate, one merge.

⭐ **#24 does not change this.** It folds atomically into `RT-DECL` + `D7` for the
same reason: `RT-DECL` remains baseline-red without the closure, and this
representation is **activated by `RT-DECL`'s unit port**. #24 stays in the
existing representation-closure subfamily.

#### ⛔⛔ Prohibitions — forbidden at every depth (`evt_3ayvrada4c0nj` §4)

⛔ whole closure/capsule transfer or publication **at any depth** · ⛔ carrier
tag/codec/template, runtime selector, function pointer, vtable, descriptor,
trampoline, or hidden side table · ⛔ declaration **inlining** into the caller ·
⛔ choosing a unit from a **runtime value or capture value** · ⛔ a source-origin
**whitelist**, residual/selector weakening, specialized-only **late** refusal, or
**size concession** · ⛔ partial frame allocation **before** the
specialization/use-closure validates.

⭐ Owner, body origin, parameter ordinal, arity, capture order, phase, storage
owner, and lifetime must **all** be typed plan material and **revalidated at the
call edge**. The existing one-way carrier remains the **only** transfer for
ordinary lifted captures. The whole capsule remains **`EscapeForbidden`** before
the first allocation/store/publication.

#### ⛔ THE SIXTH BASELINE ROW IS UNRULED — do NOT repair it from this ruling

`buffer_allocate_malformed_capacity_narrows_to_invalid_bounds` independently
still reaches `Match: scrutinee is not a constructor value`. ⚠ **That text does
NOT establish its operand phase, origin, or rejecting arm.**

⇒ After the recut is authorized and #24 is implemented, **rerun the exact row and
attribute it to the real matrix edge BEFORE changing any code.** ⛔ Do not infer
collateral. ⛔ Do not repair it as part of #24. ⭐ It may or may not be the same
family — that is a measurement nobody has taken.

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

  > #### ⭐⭐ SUPERSEDED 2026-07-29 BY ATOMIC ASSEMBLY — Architect `evt_5zr53v2dp86md`
  >
  > **The hold below was correct and its resolution is now ruled.** ⛔ It is no
  > longer *"until its consumers are complete"* — that reading was a **cycle**:
  > this node waited on [[RT-PRODUCER-MATCH-PORT]], which waited on
  > [[RT-SEED-CALL-PORT]], which waited on this node. ⚠ And not a merely
  > procedural one — `rt_parity_native` is its own CI job, so the publisher gate
  > is **mechanical**.
  >
  > ⇒ **Ruled resolution: assemble ATOMICALLY with [[RT-RECURSOR-TRANSPORT]] on
  > this `D7` lineage** — one branch on `820d3e53`, **one candidate, one PR**, and
  > **both tracker nodes flip `merged` together.** Neither can go green alone:
  > `D7`'s parity gate needs the reached successor, and the successor has **no
  > reaching production witness** on the pre-`D7` tree.
  >
  > ⛔ **Atomic assembly does NOT relabel the recursor mechanism as `D7`.** It is
  > owned, reviewed and accepted as [[RT-RECURSOR-TRANSPORT]]. ⛔ Do not fold its
  > ACs into this frame's, and ⛔ do not describe recursor code as a `D7`
  > deliverable in the candidate description.
  >
  > ⛔ **`AC-1` IS NOT DISCHARGEABLE BY THIS NODE ALONE.** The row it names,
  > `fs_write_at_malformed_offset_narrows_to_invalid_offset`, is **measured** in
  > the per-row `D0` below as failing on the base with the `ComputationalMatch`
  > refusal — i.e. it is in the successor's population. ⇒ `AC-1` is discharged
  > **on the atomic candidate**, never on a `D7`-only tree. ⚠ Read this before
  > treating a red `AC-1` row as a `D7` defect.
  >
  > ⛔ **No `D7`-only adjustment to `820d3e53` is authorized.** Preservation
  > point: `820d3e53014899da50e7d8fab0584b8c267c5874`, tree
  > `5faee6ef816ce35369a2eadee5f4de305834ad85`, parent `79029d4c`.

  ⛔ **Do not route `483ef7ab` to QA.**

  **Measured 2026-07-29 (`evt_1b1v2qjy82epm`):** targeted `rt_parity_native` on
  clean `483ef7ab` with **neither** delta applied is **1/7**.

  - **five** rows that are **green on `main` today** hit the producer-`Match`
    carried-scrutinee population (Architect-filed under
    [[RT-PRODUCER-MATCH-PORT]], hard stop **#22**);
  - one — `buffer_allocate_malformed_capacity_narrows_to_invalid_bounds` — hits a
    **distinct carried closure-capture** refusal (hard stop **#23**, owner
    classification open).

  #### ⭐⭐ THE EXACT PER-ROW `D0` — measured, and it replaces the aggregate

  **`evt_3tc0zm7smx9h2`, on detached `483ef7ab`** (no active ref moved, scratch
  worktree removed, log `/tmp/rt-d0-483ef7ab.log`). Still exactly **1/7**, now
  **with the named population**:

  | row | base result |
  |---|---|
  | `buffer_freeze_malformed_span_is_unconstructible_at_the_landed_surface` | **PASS** |
  | `buffer_allocate_malformed_capacity_narrows_to_invalid_bounds` | FAIL — `BoundaryCarrier: a closure capture is a specialized-only surface…` |
  | `fs_read_at_malformed_offset_narrows_to_invalid_offset` | FAIL — `ComputationalMatch: tree-producing match scrutinee is not Bool or a constructor` |
  | `fs_read_at_malformed_offset_without_read_right_narrows_to_invalid_offset` | FAIL — same |
  | `fs_read_at_malformed_window_narrows_to_invalid_bounds` | FAIL — same |
  | `fs_write_at_malformed_offset_narrows_to_invalid_offset` | FAIL — same |
  | `fs_write_at_malformed_offset_without_write_right_narrows_to_invalid_offset` | FAIL — same |

  ⭐⭐ **Why the aggregate was not enough, and this is the general lesson.**
  `AC-1b` is a **per-row** property — *every row green in `D0` is still green*. The
  prior record was the scalar `1/7`, and a `1/7`-then vs `1/7`-now agreement is
  **count** agreement: ⛔ **one of six contributors can defect underneath a total
  that holds.** ⚠ The scalar also could not settle which rows belonged to which
  owner — deriving that mapping by elimination against `AC-1` produced a
  **contradiction**, and only the row map resolved it. ⇒ ⛔ **Never record a
  baseline as a count.**

  ⚠ **Attribution correction.** The five `fs_*` rows' **base** refusal is the
  `ComputationalMatch` producer-shape text above, but the edge they **reach after
  the `D7` delta** is [[RT-RECURSOR-TRANSPORT]]'s recursor boundary, not
  [[RT-PRODUCER-MATCH-PORT]]'s syntactic residual (Architect
  `evt_5zr53v2dp86md`). ⛔ Do not read the base text as the owner.

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

- **⭐⭐ `AC-24A`–`AC-24F` — the `StaticCallableElimination` discriminators
  (Architect `evt_3ayvrada4c0nj`). All six. ⛔ These are IN ADDITION to `AC-7`,
  which remains required in full.**

  - **`AC-24A` — callable identity, THE PRIMARY DISCRIMINATOR.** One program
    calls the **same** transparent declaration at **two** call sites with two
    lexical closures having the **same signature, same capture count/types, and
    identical capture values**, but **different bodies** producing observably
    different constructor tags. The result must be the **ordered pair** of those
    tags, and the plan must contain **two distinct** callable-binding
    keys/body origins. ⭐ **Mutating both call edges to one key, or swapping
    their body origins, must red this oracle.** ⇒ This is what distinguishes the
    selected mechanism from one shared body with lost identity, and from one
    global binding.
  - **`AC-24B` — captures are runtime INPUTS, not specialization material.** Two
    calls use the **same** callable body/key with **different capture values**,
    produce different results, and the specialization census contains **one**
    interned unit. Swapping capture order, owner, phase, or lifetime must fail
    validation or the observable oracle. ⛔ **Capture values appearing in the key
    is a failure.**
  - **`AC-24C` — exact ABI and out-of-line proof.** For every specialized unit,
    descriptor slots equal **exactly** the remaining ordinary parameters + the
    recursively lifted captures + convention slots, and every input slot has
    typed source provenance. ⛔ Adding one selector/identity slot, transferring a
    capsule, or deleting one capture must red the exact census. The unit and its
    graph-derived call edge must exist **independently of the caller** —
    replacing the call edge with caller-body emission must red the control.
  - **`AC-24D` — complete use-closure / the negative pair.** Invocation and
    specialization-forwarding **pass**. Returning, storing, constructing,
    effect-passing, or otherwise observing the callable parameter **fails in
    planning before descriptor construction/function definition**. A
    **runtime-selected** `If`/`Match` between two closures likewise cannot enter
    this lane and must fail at that **same pre-emission boundary** — ⛔ not at
    `ObjectEmission`, and ⛔ not through a fallback.
  - **`AC-24E` — finite recursion.** A recursive transparent declaration reusing
    the same callable binding produces **one** interned specialization state and
    **terminates planning**. ⛔ Mutating interning/deduplication so recursion
    clones the unit must fail the unit census/cycle invariant **before emission**.
    Multiple static bindings remain distinct and finite.
  - **`AC-24F` — matrix causality.** Omitting the **real** origin-1031
    `StaticCallableElimination` member, reclassifying it as `Forwarding`, or
    deleting its parameter-use obligation must fail the matrix
    **bijection/use-closure before emission**.

  ⛔ **Everything `AC-7` already required remains required:** the real
  lexical-capture omission control · producer/ordinary `Match` ·
  typed-only unwrap · capture perturbations · whole-capsule escape ·
  baseline **7/7** · the exact Foundation/NHC rows · `C1` controls ·
  `RT-JOIN-DISPOSITION` controls · `AC-6` · every mutation control.

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

> ### ⛔ §5a — ARMED AND CURRENT AS OF 2026-07-29. Re-read this line at every stop.
>
> **Count of record: 26** · entries **17** · **next research pull = `#27`** ·
> **next predicate check = 18th entry**. ⇒ ⭐ **Neither trigger is due — but both
> are now ONE stop away.**
>
> ⛔⛔ **BOTH TRIGGERS FIRE TOGETHER AT `#27` / the 18th entry.** That is the
> simultaneous-due condition this section already records as a past lapse. ⭐ It is
> foreseen now rather than discovered later: **at the next stop on this chain, run
> the research pull AND the predicate check, in the same pass.** ⚠ Do not let the
> next stop's urgency defer either — that is exactly how the line went stale for
> three stops before.
>
> **#26 (2026-07-29):** [[RT-RECURSOR-TRANSPORT]] stopped on a recurred ordinary-
> `Closure` refusal inside the recursor split (`evt_6stmz1wsg17pd` →
> `evt_51fnetbggve7s`). Architect ruled `evt_5c9ys1my7hr51`: attribution closed to
> the existing recursor node, **one new matrix member #26 under the existing
> `CallableCapture` disposition** (⛔ not a seventh lane, not a new node), and
> `c45a59a9` is preservation-only **and** the next repair base. ⭐ The §8 one-shot
> bounded protocol fired and is now **spent for that edge**.
>
> **#25 (2026-07-29):** `D7` stopped at the ruled successor boundary
> (`evt_5pep3tvxb5etv` → `evt_68rfv4fm41nhx`). Architect ruled
> `evt_5zr53v2dp86md`: the edge is [[RT-RECURSOR-TRANSPORT]]'s existing
> predicate, `D1` = outcome **(b)**, `820d3e53` remains the preservation point.
>
> ⚠ **This line stood at `21`/`12` for three stops.** #22, #23 and #24 all landed
> against a stale anchor. ⭐ An armed trigger that is not re-read is not armed —
> that is the exact lapse `steward.md §5a`'s ⚠ note describes, and it is why both
> triggers came due simultaneously instead of one at a time.
>
> **Discharged at #24 (2026-07-29):** research pull **fired** and the predicate
> check **asked**, both in `evt_qak9f1thjkw7`, before the Architect ruled.
> Research confirmed awake (`capture-pane`, §5a duty 3). Cadence re-anchored from
> #24 ⇒ next pull `#27`, next predicate check the 18th entry.

⛔ The `NATIVE-HANDLE-CARRIER` stop that reordered this node is **not** #22: it
routed a red row to the node that already owned it, and no new mechanism failed.

**The four stops on this node's chain where a matrix cell was MISSING, for the
predicate check:**

| # | entry | mechanism that failed |
|---|---|---|
| 22 | 13 | producer-`Match` carried scrutinee — tree-producing scrutinee not `Bool`/constructor |
| 23 | 14 | carried closure-capture — `Carried` reaching a consumer built only for specialized shapes |
| 24 | 15 | **callable as transparent-declaration `CallArgument`** — `StaticOriginId(1031)`, a `LexicalClosure` with no lawful disposition among the ruled five |
| 26 | 17 | **static-recursor-worker residual** — `StaticOriginId(723)`, a `Closure` in the recursor split's `Captures[Carried x7]`, with no lawful **member** among the ruled matrix's cells |

⭐ #22 and #23 were already ruled one shape, and folding them is what produced
`D7` — which is what **found** #24. ⛔ No repair of the two known cells would
have surfaced it. **The reached set is evidence, never the population.**

> ⚠⚠ **CARRY THIS INTO THE `#27` PREDICATE CHECK — a four-instance pattern is now
> visible, and naming it is not the same as resolving it.**
>
> #22, #23, #24 and #26 are all the **same shape of failure**: a matrix or
> partition asserted closed, met a **real** edge it had no lawful cell for, and
> refused **late** rather than at planning. Each was individually ruled a
> legitimate new cell/member — and that is exactly what makes the pattern easy to
> keep absorbing one ruling at a time.
>
> ⭐ **The question `#27` owes is not "what is the next cell?" but "why does the
> closure keep failing to be closed?"** — i.e. whether the derivation that claims
> exhaustiveness is itself wrong, rather than merely incomplete on a fourth
> occasion. ⛔ That is an **Architect** question about the derivation, not a
> Steward re-cut and not a ring's to answer, and ⛔ it is **not** grounds to add a
> node (operator node gate: interrogate the constraint, do not presume a node).
>
> ⚠ **Steward note, flagged as inference not ruling:** #25 is deliberately absent
> from this table — it was a *boundary* stop with no missing cell, so the table's
> "stops where a cell was missing" claim stays exact. ⛔ Do not renumber to make
> the two sequences line up; the stop count and the entry count measure different
> things and always have.

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
