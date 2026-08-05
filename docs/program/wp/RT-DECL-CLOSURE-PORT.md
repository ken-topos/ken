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
traps that bind every node in it — including **Trap 3**, which this frame's
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

| path | blob at `origin/main = eefca112` (original pin) | blob at `origin/main = d5294410` (2026-08-03) |
|---|---|---|
| `crates/ken-runtime/src/cranelift_backend/lowering/core.rs` | `f7bc0d0354d8b8d6f7aa68176846b7b05e5a8514` | **`b1867461424742a6352b7758fa7fb24a020dfdfe` — MOVED** |
| `crates/ken-runtime/src/cranelift_backend/lowering/units.rs` | `f57215905ad715cab67b580781d078a614e20dfd` | **`fbbb575e052a53656d0b29a263fb3d929e5976e6` — MOVED** |
| `crates/ken-cli/tests/rt_parity_native.rs` | `b2df2bbd00644b907cae5d05efa76edd9df1b3f2` | `b2df2bbd00644b907cae5d05efa76edd9df1b3f2` — unchanged |

> ⚠ **Two of the three pinned blobs moved, and that is expected, not a stop.**
> Measured by the Steward 2026-08-03. Four merges touched `core.rs` and
> `units.rs` since the `eefca112` pin: `RT-JOIN-DISPOSITION` (`2f1b8897`),
> `RT-CONTSPEC-ASSEMBLY` (`da2ef32d`), `RT-WORKER-BIND` (`867cac7a`) and
> `RT-CONTSPEC-ACTIVATE` (`0a6e34cc`).
>
> **No deliverable, AC, or hard stop in this frame keys off these blob values** —
> they are a provenance record of what was measured when the mechanism was
> diagnosed, not a freshness gate. ⛔ **A blob mismatch here is not a hard stop
> and must not be reported as one.** Re-measure the anchors in section 2 against
> your own tree and quote `git rev-parse HEAD` beside them; the anchors are
> addresses to re-find, and this node rewrites `core.rs` anyway.

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

  > **Fixture topology — Steward ruling `evt_7cpaj2p98j6vq`, 2026-08-03.**
  > `AC-1` records that **`main` alone passes the row** (`evt_5mtkdft1nxmwp`),
  > which is why two deltas were required. ⇒ **An enumeration on bare `main` is a
  > false clear and is not the reading.** Measure each governed delta —
  > `ad7298fb` and `85dcee25` — in a **detached scratch worktree**, exactly as
  > the Architect grounded this frame. **That is not a merge, rebase, or
  > cherry-pick**; the banned-scope clause governs objects entering the candidate
  > lineage, and `85dcee25` being preserved makes it unmergeable, not unreadable.
  > Carry the committed instrument in rather than rebuilding it per tree; label
  > each result by its delta and **never union the two sets**. ⛔ If a delta will
  > not apply or build, record it **unmeasured** with the reason and route it —
  > a conflict resolved so a row passes measures the resolver.
  >
  > ⛔⛔ **A control proving "all N variants are reachable" is BLIND to
  > short-circuiting and is not a weaker version of the exact-set control.**
  > Measured 2026-08-03 on the rebuilt instrument (`38b05ac9`): under the
  > short-circuit mutation the exact-set control went red while the
  > all-five-reachable control **stayed green**, because each witness fires
  > exactly one variant, so short-circuiting cannot change its answer. ⇒ **A
  > later node that re-proves only reachability has re-proved nothing about
  > short-circuiting.** The exact-set assertion is the load-bearing one.
- **`D2`** — **Planner-owned callable declaration units.** Transparent
  closure-seed declarations become separately owned callable units rather than
  bodies recursively lowered into the generated root.
- **`D3`** — **Typed capture / parameter / result / trap transport** across that
  unit boundary.
- **`D4`** — **`DeclarationRef` calls** to those units
  (`core.rs:148`, `:7238` are the existing reference sites).
- **`D2a`** — **Function-unit population substitution.** A closure-seed
  transparent declaration contributes its declaration-owned `CallableDeclaration`
  and **no separate `SchedulingEntry` function**. Added 2026-08-04; **`D5` does
  not complete until this lands.** Full specification in the section below.
- **`D5`** — **Complete owner/phase validation**, in place **before**
  `TransparentDeclarationClosure` is removed from the retained residual.
- **`D5a`** — **Continuation elimination for the planned closure-valued
  constructor field.** **Every planner-issued continuation call is emitted from
  its exact generated emission context at its ruled seat**, instead of
  letting the closure cross a unit boundary. Added 2026-08-04. **Recut the same
  day into four ordered checkpoints** (`evt_5a0q3m9tnkh8e`, outcome (c)
  mis-sizing): population substitution, ledger lifetime, join reachability, then
  activation and the discriminators. **`D6` is not retried until checkpoint 4 is
  green.** Full specification in the section below.
- **`D6`** — Remove the residual variant, and only then re-run `AC-1`.

### `D2a` — the population substitution D5 could not run without

**Added 2026-08-04. Architect ruling `evt_3twrm71vck49d`, grounded on exact
`5e61d64096af6d2ea6b4186c22eda7274585b104` / tree
`b9c7d58329e91fc24ed19339211536f347b244e4`. Steward recut of `D2`, which this
frame had ordered as closed.**

**Same WP, same atomic scope. This is not a new graph node, a residual, a
disposition, a carrier lane, or an atomic participant** — it is a correction to
`D2`'s function-unit population and ownership representation, exposed by `D5`.

#### What `D5` hit

`D5`'s required positive control cannot be discharged, and the reason is neither
the selector nor `D5`. Under the lawful `cfg(test)` witness, a closure-seed
transparent declaration is refused at `boundary_transfer_admissibility` — *"a
closure cannot cross the boundary"* — because its now-unreachable zero-arity
`SchedulingEntry` unit **is still emitted with the closure as its body**. The
planner emits two functions for one source declaration:

1. the old zero-input `SchedulingEntry` at the closure occurrence, and
2. the new parameter/capture-bearing `CallableDeclaration` at the body target.

Triangulated, not inferred: the **referenced** and the **unreferenced** closure
declaration are refused identically, while a non-closure thunk compiles. **The
unreferenced row is the discriminating one** — with no call site, the refusal
cannot be about the call. The thunk row is the positive control on the harness:
the `FunctionizedUnits` lane and the witness both work.

`D4` proves every reference to that declaration resolves to unit (2). There is
**no lawful runtime meaning for unit (1)**: it cannot call the callable unit
without the missing parameters and captures, cannot return the closure, and
cannot become a no-op without changing program meaning. ⇒ This is not an
emission-whitelist problem and not something activation can paper over.

#### Why it is `D2`'s and not `D6`'s

**Do not retire, suppress, replace, or special-case the vestigial
`SchedulingEntry` inside `D6`.** `D6` remains exactly the one production
activation action already ruled: retire `TransparentDeclarationClosure`, then
re-run the closed `D5`/`AC-1` evidence. Retiring the residual alone puts every
closure-seed declaration on this lane and into this same refusal, so `D6` as
chartered cannot discharge activation either.

Folding the population rewrite into `D6` would activate a representation whose
positive validation had **never been runnable**, which is precisely the ordering
`AC-3` exists to prevent.

#### The closed partition `D2a` must establish

- the process/root entry remains one `SchedulingEntry`;
- a transparent **non-closure** declaration remains one zero-input
  `SchedulingEntry`;
- a transparent `Closure` or `LexicalClosure` declaration contributes **exactly
  one** declaration-owned `CallableDeclaration`, at its exact forward
  `StaticBody` target, and contributes **no separate `SchedulingEntry`
  function**;
- an anonymous closure retains exactly one `ClosureBody`.

**Derive that split from the existing exact declaration occurrence plus its one
forward `StaticBody` relation.** No source whitelist, reverse body search,
call-site reachability test, or "referenced declaration" filter is authorized.

The declaration occurrence remains the callable unit's ownership, provenance,
and `D3` signature authority. It must not become an unowned semantic node or a
second emitted definition. **For the exact declaration-owned pair, the retained
`StaticBody` relation is a definition/signature relation, not a second emitted
cross-unit `StaticBody` call.** Preserve `D3`'s boundary-layout validation over
that relation; exclude only this exact declaration-owned pair from the emitted
static-body-call population. Anonymous closure-body edges keep their existing
call and ownership law.

**The semantic partition, the ABI descriptors, the declared/defined function
population, and the emittable call population must all state the same
one-for-one result.** Each of these is rejected, because each leaves a phantom
owner or definition in another plane: merely skipping the old function in
`emittable_units`; leaving an undefined descriptor; emitting a trampoline;
changing its body.

#### Controls — the complete discriminator matrix, at minimum

- referenced and unreferenced lexical closure declarations;
- seed closure declarations;
- a non-closure transparent thunk;
- an anonymous closure;
- the root scheduling entry;
- missing, duplicate, wrong-owner, and wrong-class forward-body relations;
- a mutation retaining the obsolete closure-declaration scheduling unit;
- exact semantic-unit = ABI-descriptor = emitted-definition equality, and exact
  declaration-owned `StaticBody`-definition versus anonymous
  `StaticBody`-call populations.

**Re-run the `D2`, `D3`, and `D4` controls after the substitution.** `D4`'s
target and ordered-input contract does not change.

#### The falsified evidence, which must be edited rather than annotated

`docs/program/wp/RT-DECL-CLOSURE-PORT-D2.md` states that **the unit population
is unchanged** — *"D2 reclassifies, it does not add or remove"* — and that the
durable equality `functions.len() == entries.len() + StaticBody edge count`
stays green. **Both are falsified for the newly ported declaration-owned
class.** Correct the operative sentences on this lineage as part of `D2a`; an
appended note saying the earlier claim is wrong does not replace the claim, and
a later reader takes the first statement as the record.

#### Sequencing

`5e61d640` is a clean **preservation checkpoint, not a candidate**. Its
split-domain validator and executable controls may be retained; `D5` is not
complete. After this recut publishes:

1. repair the `D2` population on this lineage;
2. prove the population / ownership / call-edge matrix above;
3. promote `D5`'s transition sentinel into the real checked positive, and
   complete the SCC, admission, transplant, duplicate, omission, and mutual-SCC
   controls;
4. only after `D5` closes, perform `D6`'s residual retirement and re-run `AC-1`.

#### Upstream checked-plan refusals stay upstream

Confirmed by the same ruling. Interface composition, segment site,
frame-template set, and `occurrence_binding_fingerprint` mutations whose first
compile-path refusal is the existing `OrientedSubcontinuationPlanV1::validate`
**must remain attributed there.** Do not duplicate those predicates in
`validate_declaration_unit_call`, and do not relabel the upstream diagnostic as
a `D5`-local first refusal.

`D5` owes an end-to-end control that the mutation **reaches** the canonical
upstream validator and that **no declaration-unit call is emitted**. Its local
validator owns only the new composition facts: exact checked occurrence to
planner-issued declaration reference; exact symbol, target, and class; exact
immutable-descriptor to function-local-call-record equality; the ordered `D4`
input run; SCC and admission facts not already canonically closed upstream; and
exact planned/consumed/emitted set closure.

After the population correction, **re-run those controls**, so that a rejection
cannot be credited to the now-removed phantom `SchedulingEntry`.

### `D5a` — eliminate the ruled closure field, do not carry it

**Added 2026-08-04. Architect ruling `evt_44k5h9z49nf9b`, grounded on exact
preservation checkpoint `08cb257ff5118a89b72b4018e3e1c4d1733a03d7`. Steward
recut; `08cb257f` is preservation-only and never a candidate.**

**Same WP, same atomic scope. Not a new graph node, and not part of `D6`.**
**Two populations, and the frame means different things by them.** The
**provenance/discovery population** is the planner-issued producer-owner
population inside this same node, comprising the `CallableDeclaration` and
recursively discovered `ClosureBody` members — that is where a continuation
call's source occurrence is found. The **emission-owner population** is the
generated specialization-context domain: the owners that actually possess the
operands and emit the call. They are not the same set and neither is derivable
from the other by casting. The callee/call mechanism is the already-landed
continuation specialization. Splitting either side into a third atomic
participant would misstate the defect: it is a composition gap between those
populations and the landed mechanism, not a new disposition or representation
family.

#### What `D6` hit

`D6`'s retirement selects `FunctionizedUnits` correctly and runs `D5` unhooked,
then reddens the landed object-link fixture
`nested_post_effect_checked_recursor_reaches_success_and_retains_exact_trap_provenance`:
a closure-valued `Construct` field refuses at boundary transfer. The
discriminating probe holds lane and declaration shape fixed — a plain field
compiles, an anonymous `LexicalClosure` field refuses. The `RecursiveDescent`
lane had no unit boundary, which is why this never surfaced before.

**⛔ Do not recast that landed end-to-end regression as expected behavior.**

#### It is an existing continuation-specialization edge, not constructor data

The fixture's producer is exactly a checked `ComputationalMatch`; scrutinee
constructor `ITree::Vis(Unit, LexicalClosure)`; case `ITree::Vis` with recursive
position `1`; the closure at position `1` invoked through the checked IH route.
⇒ That is the planner's existing `ContinuationSpecialization` population —
matching producer constructor, selected alternative, ruled recursive position,
static worker body and captures, exact consumer continuation. **The closure is
not observable constructor data.**

#### The binding mechanism — eliminate the callable before the boundary

**⛔ CORRECTED 2026-08-04, Architect disposition `evt_c29j2reptyd2`, grounded on
exact localization checkpoint `c8f3a75ef5da6f8dddffb1eca05d389bb590502c` (the
Architect independently re-ran the localization test: 1/1, same six-entry
chronology and census). The mechanism family below was right; this section's
original singular owner/seat premise was wrong, and the corrected contract
replaces it rather than qualifying it.** ⛔ **The contract below has itself
since been corrected on its emission half by `evt_609am4v7cdt5b` — read it
together with "The emission-owner correction" and this section's own corrected
clauses, never as the last word.**

**What the localization proved.** `fn2 @36` is a planned `ClosureBody`; it
enters, lowers its result, and is **the first unit to refuse** during boundary
transfer. Its source result and producer are one exact occurrence
(`result_root=36`, `construct=36`), and the planner has **already issued** that
edge's causal identity — `owner=fn2`, `continuation=10`, `alternative=1`,
`recursive_position=1`, `target=CSId(1)`. `claim_and_call_continuation` has
**zero** entries with entry traced ahead of every branch. `fn3 @40`, the
`CallableDeclaration`, never enters, because ordinary-unit emission aborts at
`fn2` first — and the census separately carries the distinct `fn3` token.

⇒ **This is a two-owner planned population** — a claim about **discovery and
provenance**, not about emission. It is not a missing token and not one token
observed at the wrong owner. The fixed-point planner issues causal calls to
**every producer owner it discovers**, including a recursively discovered
`ClosureBody` result root; ordinary unit bodies are defined **before** the later
`CallableDeclaration`. ⛔ **Do not read "must discharge its own token" as
"raw `fn2` emits the call."** The discovered owner is where the occurrence is
found; which function *emits* is settled by the emission-owner correction
below, and for `CSId(1)` that is `Specialization(CSId(0))`, not raw `fn2`.

##### The contract

1. **Owner population — ⛔ SUPERSEDED 2026-08-04 by `evt_609am4v7cdt5b`; see
   "The emission-owner correction" below.** As written this clause said every
   planner-issued continuation call is owned **and emitted** by its exact
   `producer_owner`, with `CallableDeclaration` and recursively discovered
   `ClosureBody` as one unprivileged population. **The population claim
   stands; binding emission to the raw producer owner does not.** Activation
   falsified it: raw `fn2` cannot possess the operands of the call this clause
   asks it to emit. Read clause 1 only through the correction section.

2. **Two consumption seats, one mechanism.**
   - Where lowering **retains** the active computational frame and the exact
     Construct/alternative/position facts, and the call is **same-owner**, the
     existing in-context producer seat remains lawful and keeps it.
   - Where fixed-point discovery **detaches** a producer as an ordinary unit
     result, the seat is after lowering that exact retained result and
     **before** `transfer_unit_result_into_carrier`, allocation, publication, or
     join. The direct call's result replaces the producer constructor there.
     ⛔ **For the measured cross-owner call this seat is a position, not an
     owner.** It executes **in the explicit generated context** that retains the
     enclosing continuation environment, with that call's continuation inputs
     live across it. The raw `ClosureBody` does not become the final emission
     owner merely because this seat exists inside its body.

3. **Authority before emission.** Planning must **project** the already-issued
   identity, or exact ordered identity set, onto the exact
   `(producer_owner, producer_result_origin, producer_construct_origin)` result
   edge **before function definition.** This is exposure of existing planner
   authority, not new discovery — **but see the emission-owner correction: the
   bounded planner/schema work that distinguishes the owner domains IS now
   authorized, and this clause no longer forbids it.** ⛔ Lowering may not
   search globally,
   reverse-derive a consumer, mint from reached syntax, choose the first token,
   or use the emitted closure shape as a selector. For this measured edge the
   projection has **exactly one** member; a zero member, a duplicate, or an
   unresolved multi-member composition is a **hard stop, never a preference
   rule.**

4. **Exact call construction.** At the detached-result seat: validate the
   specialized result's planned constructor identity and field run; take
   ordinary operands from that exact result with the ruled recursive field
   omitted; append only the planner-projected ordered captures, read from **the
   immediately available slots of the generated emission context** while the
   record's **root provenance is retained separately** and is never substituted
   for them; claim the opaque identity under the **generalized emission owner**;
   declare and call the target in **that generated `Function`**; and substitute
   the returned value before transfer. **Factor the existing direct-call/claim
   machinery after identity resolution** rather than pretending the unavailable
   active-frame selector exists at this seat. ⛔ **"The exact unit environment"
   and "the current defining owner" are the two readings that stopped
   `bffc7f5b`** — raw `fn2` holds one word and cannot supply `fn3[1]`. Neither
   phrase is the rule here.

5. **Closure remains global and affine.** The existing
   planned/resolved/declared/claimed/emitted equality must close over the
   **generalized emission-owner domain** — every emission-owner identity,
   predeclared and specialization alike — **not over raw `fn2`/`fn3` function
   IDs.** Each planned continuation call is emitted **exactly once** from its
   own emission owner, against its own decoded target. ⛔ Raw `fn2` is
   **provenance**, not one of the final emitting functions. **⛔ CORRECTED
   2026-08-04 by `evt_5a0q3m9tnkh8e`: this clause previously said raw `fn2`
   "keeps its ordinary unit and simply loses this caller." That is false in the
   measured case and forbade the population substitution the ruling requires.
   Whether the raw worker keeps an emitted Function is decided by the
   post-retarget executable call graph — see "The raw-template versus
   executable-unit population" below.** Omission, owner transplant, redirect,
   duplicate claim or emission, result-edge disagreement, and capture
   disagreement all remain fail-closed.

6. **Negative boundary.** `Wrap(LexicalClosure)` without a consuming eliminator
   remains a **permanent** escape refusal. ⛔ No closure word, handle, tag,
   capsule, durable store, indirect call, runtime selector, new carrier lane, or
   `same_recursive_argument_shapes` transplant is authorized. ⚠ **The ruled
   planner-interned generated emission context is none of these** and is
   expressly authorized by `evt_609am4v7cdt5b`; this clause does not reach it.
   The generic
   `CallableCapsuleEscape -> EscapeForbidden` rule is unchanged, and
   [[RT-CONTSPEC-LEDGER]]'s general four-axis mapping work is not absorbed here.

#### The emission-owner correction

**Added 2026-08-04. Architect ruling `evt_609am4v7cdt5b`, reproduced
independently on exact `bffc7f5b73dadf3e2f0110cf960a8d3187495717`. This
supersedes clause 1's emission binding and unblocks the planner/schema work the
clauses above forbade. Disposition: bounded planner and frame correction.**

**The semantic continuation edge is real; `CSId(1)` as presently owned by raw
`fn2` is not emittable.** ⛔ Do not delete the edge, reverse-map `fn3`
coordinates into `fn2`, or widen `fn2` until the current token happens to fit.

The focused trace reaches the detached seat, claims `CSId(1)` under `fn2`, then
reports `producer=fn2 consumer=fn3`, `inputs=[(fn3,0),(fn3,1)]`, and an `fn2`
environment of **one parameter and zero captures**. The first refusal is the
source-owner mismatch; the bypass probe's out-of-range refusal is the same
physical fact, not a labelling artifact.

##### Three authorities the planner currently conflates

1. **source-occurrence provenance owner** — raw `fn2`, because the nested
   producer is textually in body 36;
2. **root input provenance owner** — `fn3`, whose parameter and capture values
   populate the continuation environment;
3. **immediate emission and availability owner** — the already-interned
   continuation-specialization execution context that selected and invoked
   `fn2`.

The fixed point descends from a newly interned specialization into
`worker.body_origin`, then **forgets that generated execution context** and
re-reads the raw occurrence owner. `CSId(0)` holds the two `fn3` continuation
inputs, but its ordinary static-worker call passes only the worker's explicit
argument and source captures, so raw `fn2` receives one word and the
continuation inputs are dropped before its nested producer seat. **No lowering
lookup can recover them lawfully.**

##### The binding correction

The existing continuation-specialization mechanism remains the mechanism; it
gains an **explicit generated emission context.** That context is **not** a
runtime carrier, selector, disposition, or third atomic participant.

- The discovery frontier and causal ledgers must **distinguish an ordinary
  predeclared owner from a continuation-specialization owner.** ⛔ Do not cast
  or alias one ID domain into the other.
- Descending into the selected worker body from an interned specialization
  **retains that specialization as the immediate emission owner.** The raw
  body's predeclared owner remains **provenance only**.
- Every input record distinguishes **root provenance** (`fn3[0]`, `fn3[1]`)
  from its **immediate available slot** in the enclosing specialization
  environment.
- The generated specialization execution must keep those continuation inputs
  **live across the exact checked-IH worker execution.** A nested producer
  emits and claims its exact direct continuation call **from that generated
  context**, before result transfer, allocation, publication, or an
  identity-erasing join.
- Materialize this as an **explicit planner-interned, continuation-specialized
  producer execution context.** It may be a generated definition subordinate to
  the existing specialization identity. ⛔ It must not mutate or union the raw
  `ClosureBody` ABI, and must not fuse a generic runtime suffix into the raw
  source body.
- **The same raw worker reached under two continuation identities yields two
  distinct generated contexts** — never one widened raw ABI, never a runtime
  choice.
- The planned/resolved/declared/claimed/emitted exact-set closure extends over
  this **generalized emission-owner domain**.

For this witness the corrected nested call retains root provenance at `fn3`,
while its immediate source slots belong to the specialization context that
already carries those two values. **Raw `fn2` is no longer asked to emit a call
whose operands it cannot possess.**

##### What `bffc7f5b` keeps, and what it owes

`bffc7f5b` is a **non-candidate evidence checkpoint.** Retained and accepted
through this recut: pre-definition result-edge projection; the detached
post-result, pre-transfer seat; common exact claim/call factoring; exact
constructor, result and field validation; and the honest unexercised
fail-closed scaffolding. **Rejected is only the assumption that raw `fn2` is
both the nested edge's provenance owner and its immediate emission and
source-slot owner.** The stop was taken at the correct boundary, and no further
control work can be load-bearing until the positive route exists.

The corrected checkpoint owes discriminators for: root provenance versus
immediate owner and position; two continuation identities selecting the same
raw worker; **unchanged ordinary `fn2` ABI** — which per `evt_5a0q3m9tnkh8e`
means the raw worker's **descriptor and source binding** are preserved so a
generated context can validate and lower the same body, **not** that the raw
body is defined in an environment lacking the continuation inputs; missing,
duplicate, and transplanted generated-owner bindings; recursive
intern-before-descent; and the existing permanent stored-closure negative.

**Still not a new node.** This is the distinct-owner-with-an-exact-token branch
of the published decision table — a same-WP `D5a` frame recut, not a new
disposition, carrier lane, or third atomic participant, and **no change to Ken
or runtime semantics or to the continuation-specialization mechanism itself**.
⚠ **This sentence is not a bar on the authorized schema work.** The bounded
planner and schema correction that distinguishes the emission-owner domains,
including the planner-interned generated context, is expressly authorized by
`evt_609am4v7cdt5b`. What is unchanged is the *semantics*, not the planner's
record shapes.

#### The `08cb257f` probe must keep rejecting — it is not the positive

That probe returns `Wrap(LexicalClosure)` and has **no consuming computational
eliminator**, so its closure really is stored as a constructor value. Its
plain-field half proves the lane works; its closure-field half proves the
generic escape prohibition. It proves nothing about a carrier capability
arriving. ⛔ **Its comments and trigger must not say this row should turn green
before `D6`.**

The load-bearing discriminator is instead:

1. the exact planned `Vis` recursive field **compiles** by direct continuation
   elimination;
2. the same-shaped **unplanned, nonrecursive, returned, or stored** closure
   field **rejects before allocation or publication**;
3. omitting, transplanting, redirecting, or failing to claim the exact causal
   identity **fails** the existing planned/resolved/declared/claimed/emitted
   closeout.

#### The checkpoint, and what it gates

- under the existing selector witness, the landed object fixture reaches the
  exact direct continuation call **from its exact generated emission context**
  at the ruled seat, with root provenance and immediate available slot recorded
  separately;
- each call receives its exact declared continuation target, and the existing
  whole-pass planned/resolved/declared/claimed/emitted closure closes over the
  **generalized emission-owner domain**, each call emitted exactly once;
- the ordinary raw `fn2` ABI is **unchanged**, and two continuation identities
  over one raw worker produce two distinct generated contexts;
- the generic closure-valued constructor negative is preserved.

#### The four-checkpoint recut — `D5a` is mis-sized as one turn

**Added 2026-08-04. Architect ruling `evt_5a0q3m9tnkh8e`: contract reading (B),
strengthened to a closed population rule, plus outcome (c) sizing at the
delivery-frame level. Grounded on exact
`cbdf9a2a9630517c2355dc487b51cba5be20e713`.**

**Still one semantic node and one eventual atomic candidate** — not a new
disposition, carrier lane, or third atomic participant. What changes is the
**delivery order inside `D5a`**: four ordered, independently reviewable
checkpoints in place of one turn.

**`cbdf9a2a` is accepted as a preservation-only generated-context checkpoint,
never a candidate.** Sound and retained through this recut: the owner domains,
context interning, unchanged raw ABI, provenance/immediate-slot split, the
retarget, and the generated `Function` direction.

##### The raw-template versus executable-unit population

Raw `fn2` is a **source template** here, not necessarily an emitted `Function`.
"Unchanged ordinary `fn2` ABI" preserves the raw worker's **descriptor and
source binding** so generated contexts can validate and lower the same body. It
does **not** require defining that body in an environment known to lack the
continuation inputs.

The governing population is the **post-retarget executable call graph**:

- if **at least one** exact final call edge still targets the raw worker, its
  ordinary `Function` remains **declared and defined**, and the permanent
  unconsumed-closure refusal still governs that raw execution;
- if **every** exact final call edge is retargeted to generated contexts, the
  raw worker remains descriptor, provenance and template authority and is
  **absent from the emitted-`Function` population**.

⛔ **CORRECTED 2026-08-04 by `evt_3s0nf03nkmz0a`. This section previously
asserted "for the measured `fn2`, the second branch applies." That was
premature, and checkpoint 1 falsified it on the graph that exists there.** The
measured checkpoint-1 census, reproduced independently by the Architect, is:

```text
template_only={}
executable=[fn0, fn1, fn2, fn3]
```

**Branch one is the correct answer for the partial graph at checkpoint 1**, and
the checkpoint-1 algorithm is sound substrate. Its *population answer* is
**provisional** until the last retargeting seat is closed in checkpoint 4.
⛔ **The final census decides `fn2`'s branch — not this frame's earlier
prediction, and not the current partial graph.**

**Why branch one holds there.** A second, distinct route reaches origin 36:
`lower_source_machine_with_continuation ->
call_declared_recursive_position_unit(origin 36)`. It is a planner-derived
**carried source-machine recursive predecessor**, separate from the static-worker
call already retargeted through the generated context. It is real and it is
live: removing its graph edge would make the live invocation lose its declared
target, so **neither global suppression nor reclassification as dead is
lawful.**

**Two axes, and they must not be conflated.**

| question | answer, and where it is settled |
|---|---|
| Does the semantic recursive predecessor exist and remain reachable? | Yes — planner-derived, stays in the closed graph. Checkpoint 3 answers only this. |
| Which final `Function` does that predecessor call? | At checkpoint 1, raw `fn2`. That is the current lowering binding, **not** proof that raw `fn2` is the final callee. Settled in checkpoint 4. |

Checkpoint 3's origin-25 rule **preserves** this route and makes downstream
reachability honest for it. It does not delete it, make it unreachable, or
decide that its current raw callee is permanent — origin 25 is *downstream* of
the invocation, and ⛔ **reachability repair cannot retarget the invocation that
supplies the predecessor.**

⛔ **"A `StaticBody` call edge is never superseded" is too broad for the
completed mechanism.** The **source edge and its provenance are never deleted**.
The **emitted callee edge may be retargeted** when exact planner authority
proves that this carried invocation executes in a generated continuation
context. Any implementation comment or table stating the broader claim is to be
narrowed to that.

⛔ **Do not merely skip `define_function` after declaring an "emittable" raw
unit.** That mints an undefined phantom and falsifies the declared/defined
census. Planning must separate **raw-body descriptor authority** from
**executable-unit membership**, and the constructor's raw identity and arity
validation must be **separate** from the generated context `FuncRef` the call
uses. ⛔ **"One context exists" is not a global suppression predicate** — a
mixed raw-plus-context caller population **retains** the raw `Function`.

**Clause 6 is unchanged.** A genuinely raw, unconsumed `Wrap(LexicalClosure)`
still refuses. The contextual execution does not cross that boundary; it
eliminates the callable in the generated context.

##### One cross-pass ledger lifetime

`define_unit_bodies` presently closes the
planned/resolved/declared/claimed/emitted equality **before**
`define_continuation_bodies` and `define_continuation_context_bodies`. It
therefore reports one planned token absent before the generated context has had
any chance to declare, claim, or emit it.

Open the one ledger **before the first generated `Function`**, and close it only
**after** ordinary units, continuation specializations, and generated contexts
have all been defined and recorded. ⛔ **One global equality** — not per-pass
partial equalities, and not a mirrored second ledger.

With that close moved later, the next reached edge is the existing
checked-computational-IH carried-marker refusal while defining the
specialization body. That measurement used diagnostic bypasses and **authorizes
no weakening**: the refusal itself names the already-ruled emitted-helper route.
This recut carries that activation seam as checkpoint 4. ⛔ **No compile-time
template recovery from a carried word.**

##### Recursive-return join reachability — origin 25

**Origin 25 is a stale reachability classification, not a new residual.**
Independently reproduced by the Architect:

```text
required_join_origins(fn3) = {10, 14, 25}
join(25) = NativeScalarPair, has_continuing_predecessor = true
```

Origin 25 is the ordinary `Result` `Match` in the `ITree::Ret` arm owned by
`fn3`. The initial scrutinee selects `Vis`, but the newly emitted exact
continuation call supplies the recursive return edge on which `Ret` is
reachable. Lowering currently **both** consumes and materializes origin 25
**and** dispositions its subtree as statically unselected from the initial
selection. The finished CFG correctly exposes that contradiction.

⛔ **Keep the `materialized-but-dead` validator.** Do not bypass it, force the
block dead, or delete origin 25.

Final match-case reachability is the **union of the initial selection and every
planner-issued source-machine recursive predecessor**. An initial static
selection may disposition a case **only when no planned recursive or dynamic
predecessor can select it**. A specialized re-entry keeps its exact selected
case; a carried or dynamic re-entry retains the planner's closed reachable case
population. This is bounded existing `D7`/join-disposition work **inside this
same semantic node**.

##### The delivery order

| # | checkpoint | what it closes |
|---|---|---|
| 1 | raw-template versus executable-unit population substitution | the descriptor/call-target split and the no-phantom census |
| 2 | one cross-pass continuation ledger lifetime | the exact-set closeout across all three definition passes |
| 3 | recursive-return join reachability | origin 25, retaining the existing CFG validator |
| 4 | generated-context and checked-IH activation | three ordered steps below: carried-invocation binding, final executable-population close, then activation and evidence |

**Landed so far:** checkpoint 1 at exact
`8e07bab89762dab79e041fab768485e8d331abec` (accepted as checkpoint-1 evidence
and preservation; its branch-one population answer is **provisional**), and
checkpoint 2 at exact `e5762c5c` (one ledger lifetime, before/after measured
under the ruling's own diagnostic bypasses). Both are preservation checkpoints,
never candidates. **Checkpoints 2 and 3 are semantically unchanged by the
2026-08-04 correction.**

Each checkpoint must **preserve `cbdf9a2a`'s sound generated-context work** and
must be **commit-clean before the next begins**. ⛔ **No red-versus-red
discriminator counts as evidence** — the ruled discriminators land against the
positive route in checkpoint 4, never before.

##### Checkpoint 4, in three ordered steps

**Added 2026-08-04, Architect ruling `evt_3s0nf03nkmz0a`. This is a retarget of
one exact emitted call while preserving its semantic predecessor — not deletion
of the edge**, and it must use the existing continuation-specialization and
generated-context mechanism.

1. **Carried-invocation binding.** Before lowering, project an **exact** binding
   from this planner-issued source-machine recursive invocation to its exact
   continuation-specialized generated execution context. ⛔ The lookup must be
   keyed by **the invocation's causal identity and retained source
   coordinates** — never merely by body origin, ABI shape, "a context exists",
   or first match. **Zero or multiple bindings is a hard stop.** If the exact
   causal identity is not available at the source-machine seat, **planning must
   expose it before definition; lowering may not reconstruct it.**
2. **Final executable-population close.** Re-run checkpoint 1's post-retarget
   population and the declared/defined/no-phantom census **after that binding
   exists**. If another exact raw call still remains, `fn2` **stays branch one**
   and its permanent raw closure refusal governs that route. **Only if every
   emitted invocation has an exact generated-context target** does `fn2` become
   descriptor/provenance/template-only, i.e. branch two.
3. **Checked-IH activation and evidence.** Then discharge the already-recorded
   carried-marker activation, and run the positive route plus **all** ruled
   discriminators against the fully closed route.

⛔ **This authorizes no runtime selector, closure carrier, raw-ABI widening,
global body suppression, template recovery from a carried word, or choice by
shape.**

Any of these three steps may be split into its own bounded checkpoint if it does
not fit one turn.

**`D5a` is CLOSED.** Completed at exact
`2ed97dd1cdbd0a76dbd49520f78047e8fde9d8e9` and fresh-QA approved against base
`25fccc7e` (`evt_7rvdp1qcxd8xt`): 664 passing, zero failed; the nine ruled
discriminators borne; all five formerly unexercised detached-seat guards
exercised by reaching mutations; the final census measured as the **mixed**
population `template_only={StaticOriginId(36)}`, `executable=[fn0, fn1, fn3]`,
asserted as the relation `executable = emittable minus template_only` rather
than as a literal list. That settles checkpoint 1's explicitly provisional
branch-one answer as **branch two**.

One boundary is recorded rather than hidden:
`d5a_reading_the_root_position_as_the_immediate_slot_is_currently_undetectable`
passes as a transition sentinel for a known swapped-coordinate residual. The
candidate does not claim such a swap is caught.

**CORRECTED 2026-08-04 by `evt_4m2qk2fehm6vg`: this frame previously said `D6`
is "retried unchanged" as "one production action". That clause is now false,
and its falsity is load-bearing** — a reader who keeps it will take the `D6a`
repair below as unauthorized scope creep and refuse it.

`D6` was performed at exact `1e5daa7b`, and its one action **succeeded and is
isolated**: `TransparentDeclarationClosure` and both producers are retired, the
selector witness is fully removed, the remaining residuals and their
fail-closed classifier are unchanged, all 31 unhooked calls and 32 `D5`/`D5a`
controls are green, the governed enumeration is empty and selects
`FunctionizedUnits`, and the ruled shape-change row is now a positive with
`RecursiveDescent`'s separate negative retained. **Then the functionized
artifact ran for the first time and trapped.** `D6` is therefore **not closed**,
and the repair may not be taken under the one-action clause. It is `D6a`.

**Resume base — the single place this frame states it.** The resume point is
exact `ae45e804`, which carries `D6`'s activation (rebased content-identical as
`e3891003`) plus `D6a`'s accepted downstream carried consumer. It supersedes
`1e5daa7b`, which this block named before the predecessor-edge ruling, and
`e5762c5c` at checkpoint 3 before that.

Every earlier SHA on this lineage — `1e5daa7b`, `2ed97dd1`, `3e58490e`,
`86bdb5cb`, `5758dd93`, `016b786c`, `d65ee15c`, `9f12da78`, `b6d13351`,
`e5762c5c`, `8e07bab8`, `cbdf9a2a`, `a765b8d3`, `c8f3a75e`, `08cb257f`,
`bffc7f5b` — is a preservation or evidence checkpoint and **never a
candidate**; reverting to any discards accepted work. The localization trace is
discharged: the `D5a` discriminators now bear its claims. The candidate and QA
routes stay closed until both halves of `D6a` are green.

⛔ **This block has now been restated three times in one day, and twice a stale
copy of it outlived the measurement that retired it. If a later ruling moves
the resume point, edit HERE and grep the whole frame for the old SHA — an
appended correction elsewhere does not replace this sentence.**

#### `same_recursive_argument_shapes` is obsolete on this lane — no guard was lost

`same_recursive_argument_shapes` is **not a Ken semantic law and not a declared
function-unit ABI predicate.** Its only production uses guard
`RecursiveDescent`'s same-function CFG backedges, where one fixed run of
specialized `Lowered` block parameters must represent every loop iteration;
there, turning `None` into `Some(Int)` changes the compile-time template and
must reject.

Functionized calls hold a different representation contract: every declared
parameter is one `AbiSlotKind::Parameter` with `AbiCarrier::ValueWord`; the
descriptor is independent of the particular runtime constructor; each actual
argument is transferred through the boundary encoder at the call; and graph
admissibility still rejects genuinely non-transferable children. ⇒ `None` and
`Some(Int)` are **two lawful values of one declared slot**, not an ABI shape
disagreement.

`recursive_declaration_shape_change_hits_typed_boundary` going green is
therefore **the intended removal of a `RecursiveDescent` implementation
restriction**, not a lost guard. On the `D6` lineage, rename and reframe it as a
**positive** proving the functionized recursive declaration accepts the variant
change through one `ValueWord` parameter, and retain a **separate negative** for
an actually non-transferable value graph or a descriptor/input disagreement.
⛔ Do not transplant `same_recursive_argument_shapes` into the declared-call
path; keep it for every remaining `RecursiveDescent` backedge.

### `D6a` — thread the closed answer-route fact into the carried eliminator

**Added 2026-08-04. Architect ruling `evt_4m2qk2fehm6vg`, grounded on exact
`1e5daa7b2bde4f070e87cbdff69997cd65353e46`. Steward recut. `1e5daa7b` is the
`D6` activation and red-evidence checkpoint, preservation-only and never a
candidate.**

**Same WP, same atomic scope, one bounded post-activation checkpoint. Not a new
graph node**, not a new residual, disposition, carrier lane, runtime callable
selector, or third atomic participant. The node gate was applied and returns
"no node": the constraint arguing for one is this frame's own one-action
sentence, which the ruling has just retired. The seam is an existing
`D7`/source-machine semantic-consumer edge that `D6` activation exposed for the
first time.

#### What `D6` activation exposed

`D6`'s own action is sound. What it removed was the last thing standing between
the fixture and execution, so the governed runtime witness compiled, linked,
and then exited 1 with `ken native trap: explicit entry trap`, empty trap
provenance, and no `DeforestedAnswerResumed` event. **Its harness never used
the selector witness, so this is the first execution of the functionized
artifact.** Every prior checkpoint could prove emission or refusal, never a
runtime answer.

The first causal source seat is the `LoweringOperand::Carried` arm of
`SourceContinuation::ComputationalMatchScrutinee` in `lowering/core.rs`:

- the source continuation holds a field
  `answer_route: SourceComputationalAnswerRoute` — **CORRECTED 2026-08-04 by
  `evt_4n6anh6431w1p`: this bullet originally said the source continuation
  "still holds the closed fact", and the section below built on that. It is
  false for the post-activation path. That seat is one producer of the route,
  not its owner. See "The route is a predecessor-edge fact" below, which is
  the operative statement;**
- its **specialized** arm reads that fact, and on `CheckedSelectedRecursor`
  routes an unmatched constructor through the unique guarded `ITree::Ret`
  answer path, recording `DeforestedAnswerResumed`;
- its **carried** arm builds `ComputationalEliminatorFrame` **without that
  field**, and `ComputationalEliminatorFrame` has no answer-route member;
- so `lower_carried_computational_match_inner` compares the carried tag only
  against the ordinary `ITree::Ret` / `ITree::Vis` case identities and seals
  the ordinary default.

On this witness the checked recursive worker returns carried `Result::Ok` —
the checked answer the specialized route sends to the unique `ITree::Ret`
continuation. The carried route instead asks whether `Result::Ok` is literally
an `ITree` constructor, which it is not, and takes `PX8-TR checked ITree
recursor default`. The unit writes that trap identity to its `TrapWord`, the
root adapter maps a process trap to `-4`, and the starter prints the trap line.

**The empty provenance is predicted by the same drop, not a second failure.**
`DeforestedAnswerResumed` is emitted only while lowering the specialized
branch, so it cannot report a runtime-carried choice the emitted CFG never
contained. It is evidence of the phase gap.

#### The complete bounded mechanism, and nothing beyond it

1. **Preserve the existing closed route fact.** Thread the incoming
   `SourceComputationalAnswerRoute` into the carried computational-eliminator
   operation. Do not infer it from a tag, body, constructor spelling, frame id,
   or the presence of a continuation unit. ⛔ **CORRECTED 2026-08-04 by
   `evt_4n6anh6431w1p`: this clause originally said to thread the route "from
   `SourceContinuation::ComputationalMatchScrutinee`", naming that seat as the
   source. It is one producer, not the authority, and the downstream half built
   against this clause is accepted and retained. Where the route comes from is
   ruled in "The route is a predecessor-edge fact" below, and that section
   governs.**
2. **Ordinary case matching remains first and unchanged.** A carried value
   whose exact planner-issued tag matches an ordinary computational case takes
   that case.
3. **The checked-answer fallback is narrow.** Only after no ordinary case
   matches, and only for `CheckedSelectedRecursor`, emit the same guarded
   answer route as the specialized arm: exactly one `ITree::Ret` case with one
   binder, exactly one `ITree::Vis` case, exactly two cases total, and the
   existing no-checked-marker condition on the return body. Feed the **same
   carried word** as the return case's one retained argument and continue
   through the original source control.
4. **No phase forgery.** Do not decode, reconstruct, or convert the carried
   word into `Lowered`; do not recover a template; do not choose a callable
   target at runtime; do not add a constructor whitelist or a name-based
   `Result` special case.
5. **Defaults stay closed.** `DirectScrutinee`, malformed return topology,
   disabled checked-answer routing, unknown or unlawful tags, and every
   unmatched ordinary carried scrutinee retain the existing exact default and
   trap behavior.

#### The evidence, required before candidate or QA routing

- the enabled linked artifact exits 0 through the unique
  return-case-dependent `ExitSuccess`;
- disabling **only** the checked-answer fallback exits 1 through the exact
  planned `PX8-TR checked ITree recursor default`;
- dropping the route fact between the source continuation and the carried
  eliminator recreates this exact red;
- an ordinary matching carried `ITree` constructor still takes its ordinary
  case, ahead of the fallback;
- `DirectScrutinee` plus malformed or ambiguous return topology cannot enter
  the fallback;
- the carried return argument remains `Carried`, and a
  carried-to-specialized/template reconstruction mutation reds;
- the original nontrivial source continuation is observed **after** the return
  case, so a helper that returns an isolated value cannot pass;
- every accepted `D5`/`D5a` control and the exhaustive residual controls stay
  green unhooked.

**Two evidence rules that are the point of this checkpoint, not decoration.**
The compile-time `DeforestedAnswerResumed { actual_constructor }` assertion is
**not truthful evidence of runtime execution on this branch** — keep it for the
specialized branch, and for the carried branch pair the linked exit result with
a separate emission/runtime discriminator. And prove the disabled trap's
**planner trap identity at the unit `TrapWord` and root propagation seat**: the
generic process `-4` string alone is not exact provenance.

#### The route is a predecessor-edge fact — the upstream half

**Added 2026-08-04 by Architect ruling `evt_4n6anh6431w1p`, grounded on exact
`ae45e80427cead5bcdd16de0b459cf1aa93c8e7f` (tree `e2dbc153`, parent
`e3891003`). Steward recut. `ae45e804` is preservation-only, never a
candidate.**

**The downstream half above is ACCEPTED and must be preserved.** Runtime
implemented it faithfully — ordinary carried case matching first, the narrow
`CheckedSelectedRecursor` fallback with the exact two-case/one-`Ret`/one-binder/
no-marker topology, the same carried word passed unchanged, closed defaults, and
a separate `CarriedAnswerRouteEmitted` event that claims emission rather than
runtime choice. The stop was also correct: the seam has the exact frame and
topology but receives `DirectScrutinee`, and deriving otherwise from
`checked_frame_id.is_some()` is forbidden **and would be unsound**.

**The falsified premise.** The route is **not** a property of the
`ComputationalMatch` occurrence or of its checked frame. It is a property of
**the exact predecessor that supplied the answer.** The planner census on this
witness makes the distinction concrete:

```text
CSId(0)  continuation origin 10, emitted from predeclared fn3
CSId(1)  the SAME continuation origin 10, emitted from Specialization(CSId(0))
         the second causal call is owned by that specialization context and
         targets CSId(1)
         both coexist with the ordinary direct scrutinee of origin 10
```

⇒ Neither `StaticOriginId(10)`, frame 7, "a continuation unit exists", nor the
current emission owner selects the route. **Any occurrence-global projection
would mark the ordinary direct predecessor as checked too.**

#### The two lawful producers, and nothing else

1. **The existing checked recursor-layer predecessor.** Its exact
   `RecursorLayerRole::SelectsOccurrence` path supplies
   `CheckedSelectedRecursor`; every other layer role and direct descent
   supplies `DirectScrutinee`.
2. **An actually emitted, exactly claimed continuation-specialization call.**
   The authority is the opaque `ContinuationCallIdentity` consumed at
   `claim_and_call_resolved_continuation`, **after** the owner/affine claim
   succeeds and **after** the emitted callee has been checked against
   `identity.target()`. A result from that exact call is a
   `CheckedSelectedRecursor` answer. ⛔ A static-worker call, a raw unit call,
   an ordinary expression result, or a merely matching continuation origin is
   **not**.

This uses existing planner authority. **No planner population, ABI descriptor,
frame, carrier tag, or runtime selector is added.**

#### The transport contract

Carry a **compiler-only routed answer** on the exact lowering predecessor: the
existing operand paired with `SourceComputationalAnswerRoute`. It is metadata
of the compiler path — **not a field in the runtime word and not a new carrier
lane.**

- Ordinary source evaluation starts `DirectScrutinee`.
- Applying an exact selecting recursor layer changes the routed answer to
  `CheckedSelectedRecursor`.
- Returning from the exact claimed and emitted continuation call changes
  **that call result** to `CheckedSelectedRecursor`.
- The source/composed state transports the pair until the computational-match
  consumer, which threads the incoming route into the already-accepted
  `ComputationalEliminatorFrame`.
- **A function boundary carries only the word.** The **caller** re-attests the
  route from its own exact claimed call identity; the callee never writes a
  hidden route bit.
- If control-flow composition would merge `DirectScrutinee` and
  `CheckedSelectedRecursor` before the consumer, **preserve them as distinct
  predecessor arms. If the current join cannot do that, hard-stop.** ⛔ Do not
  collapse them to either scalar and do not add a runtime discriminator.

The existing field on `SourceContinuation::ComputationalMatchScrutinee` may
remain, for the recursor-layer producer. ⛔ **It is not the sole authority and
must not overwrite a checked route carried by the incoming exact call result.**
This also retires the reading that every other construction site should be set
to `DirectScrutinee` as a default: ordinary evaluation *starts* there, and an
exact producer *raises* it. A site that hard-codes `DirectScrutinee` on a path
an exact call result reaches would erase the fact this checkpoint exists to
transport.

#### Forbidden shortcuts

⛔ Do not derive the route from frame id or presence, match origin, body,
constructor spelling or tag, the current defining or emission owner,
continuation-unit or context existence, ABI shape, test name, or "it is the
only candidate". Do not mark all ordinary descents checked. Do not store the
route in the boundary word, decode the word, or choose a callable at runtime.

#### The discriminators this upstream half owes

These are **in addition to** the eight downstream evidence obligations above,
which stand unchanged.

- the exact claimed `CSId(0)` call result reaches origin 10 as
  `CheckedSelectedRecursor`, emits the carried fallback, and the linked
  witness exits 0; `CSId(1)` is raised later inside that fallback's
  return-case body and reaches no carried consumer on this witness.
  ⛔ **CORRECTED 2026-08-04 by `evt_3hnn2c2jvbkj`: this bullet named `CSId(1)`,
  which is temporally impossible as written — `CSId(1)` is claimed after the
  very fallback the bullet attributed to its result. The implementation's
  predecessor-edge attribution is the lawful one; changing code to make the
  prose true would corrupt the mechanism. The mechanism is identity-parametric
  and unchanged: a result becomes `CheckedSelectedRecursor` only after its
  exact `ContinuationCallIdentity` is claimed and the emitted callee checked
  against `identity.target()`. Only which identity this witness instantiates it
  with was wrong.**
- dropping **only** that call-result route recreates the exact planned default;
- the **same checked frame** with an ordinary direct predecessor stays
  `DirectScrutinee` — this is the control that would have caught the
  occurrence-global projection;
- a raw or static-worker call **cannot mint** the checked route;
- the recursor-layer producer stays green **independently** of the call-result
  producer;
- a mixed-route predecessor fixture either preserves separate arms correctly or
  reaches the explicit merge hard stop;
- planned, claimed and emitted call identity and the routed-consumer evidence
  agree **on the exact identity, not merely on counts**;
- every existing `D5`, `D5a`, `D6` and downstream `D6a` control stays green
  unhooked.

#### Holds

Preserve `1e5daa7b` and `ae45e804`. Do not revert the residual retirement,
reintroduce the selector witness, or undo the accepted downstream carried
consumer — all three are directionally correct. Do not tune joins, resume
behavior, continuation identities, captures, or call operands from a red
evidence checkpoint. Runtime is released from exact `ae45e804` by its leader
once this recut has published. The candidate route, `D6` closure, and the QA
route stay closed until both halves of `D6a` are green.

> ### ⭐⭐ `D7` NOW CARRIES THE EXACT-RECORD RE-DERIVATION, AND IT GATES ANOTHER NODE
>
> **Architect ruling `evt_40ra70t92mjd2`, 2026-08-03, on `RT-CONTSPEC-LEDGER`
> hard stop 2.** That seam needed a mapping from planner facts to the four
> boundary-use variants, found none, and stopped. The ruling sustained the stop:
> **no current lawful mapping authority exists**, and the re-derivation this
> section already names is where it must come from. It is **not** a seventh
> disposition, a carrier lane, or an independent node.
>
> **`D7` must materialize one planner-issued record for every exact boundary-use
> event** after the generated-unit / specialization / continuation fixed point
> closes. Source-backed and synthesized events inhabit the **same closed set**.
> Each record binds:
>
> 1. exact producer and consumer owners and phases;
> 2. source occurrence plus slot/path, or an earned synthesized-edge identity;
> 3. the exact downstream semantic operation and the consumer-derived `Need`;
> 4. the selected disposition and its guaranteed `Avail`; and
> 5. the lifetime/use fact that distinguishes **forwarding** from **retention**.
>
> **Closure is proved by exact planned-set membership and a pre-emission
> planned/consumed bijection.** Missing, duplicate, ambiguous, wrong-owner,
> wrong-phase, wrong-slot/path, and unconsumed records all **fail closed**.
> ⛔ **Reached traces, fixture names, source kind alone, and the historical
> census are evidence only — never selectors.**
>
> ⇒ **`RT-CONTSPEC-LEDGER` is now `depends_on` this node** and stays held after
> its accepted `D1` until this authority lands. Its four fields are a
> **projection** of the record above. ⚠ A disposition need not determine a unique
> tuple; if the binary enums cannot total-project without coercion, that is
> LEDGER's stop to raise, not a reason to widen anything here.

### ⛔⛔ `D7` — ITS CLAIM TO BE A **CLOSED** MATRIX IS WITHDRAWN (2026-07-29)

**Added 2026-07-29, Architect `evt_6h6vzqw7ydra8`. ⭐ RECUT 2026-07-29 at hard
stop #24, Architect `evt_3ayvrada4c0nj`. ⛔⛔ POPULATION AUTHORITY WITHDRAWN
2026-07-29 at hard stop #27, Architect `evt_4p9ne0vcds5hb` + addendum
`evt_3gzcnk62v8bzz`.**

> #### ⛔⛔ READ THIS BEFORE ANY ROW BELOW. THE ROWS ARE STILL THE SEMANTIC
> #### CODOMAIN; THEY ARE NO LONGER A PROOF THAT THE POPULATION IS CLOSED.
>
> **This heading formerly read "THE CLOSED BOUNDARY-OPERAND SEMANTIC-CLOSURE
> MATRIX". That word is RETIRED, not qualified.** The Architect ruled at stop #27
> that **the derivation failed, not merely its latest classification** — and
> withdrew the closure claim rather than adding a fifth cell.
>
> ⛔⛔ **DO NOT ADD A FIFTH CELL, A SEVENTH DISPOSITION, OR A NODE.** All three are
> explicitly unauthorized. ⛔ Do not classify stop #27's origin-650 `Closure` at
> all yet — it must first become a real member of a correctly derived population,
> because classifying it now repeats the defect.
>
> **What the code proved false** (measured on exact `07ce6ef1`, three independent
> ways):
>
> 1. `build_operand_edge_matrix` / `validate_operand_edge_matrix` are exact only
>    over positional **source** children from `RuntimeExpr` + `SourceOperandRole`.
>    That closes one syntax-derived subset, ⛔ not the lowering population.
> 2. `LoweringOnlyOperandEdge::token(self)` mints an `OperandEdgeToken` **from an
>    enum label**, ad hoc at consumer sites. ⇒ Exhaustive matching proves every
>    **named variant** has a disposition; it proves **nothing** about whether every
>    real transfer *has* a variant.
> 3. `StaticRecursorWorkerResidual` is **one global `Option<disposition>` flag**
>    plus on-demand synthesis by source-occurrence search — so the omission
>    mutation removes a *flag*, not a concrete planned edge.
>
> ⭐⭐ **THE DECISIVE POINT — AND IT IS SHARPER THAN "A MEMBER WAS MISSING."** The
> addendum plus Research advisory `evt_62tkq32hrjqmn` establish that #27's edge is
> **already in the source inventory**: every `Construct` child maps to
> `SourceOperandRole::ConstructArgument`, uniformly `SemanticEliminator`. But the
> real crossing in `transfer_constructor_operands` **never consumes that cell** —
> it calls the whole-value gate, whose `Closure` arm fabricates
> `CallableCapsuleEscape.token()` with `parent = child = position = None`.
>
> ⇒ **ONE concrete crossing event receives TWO independent verdicts:** planned
> `(655, 650, ConstructArgument) -> SemanticEliminator`, and lowering-time
> `(Closure, no edge identity) -> EscapeForbidden`. ⛔ **That is two populations
> crossing, not a missing name** — and no amount of adding outcomes or unioning
> another enum repairs it. ⭐ If the declared classifier is `f(role)` and equal
> roles require different outcomes, **adding an outcome does not make `f` a
> function.**
>
> ⚠ **The Architect withdrew part of its OWN prior ruling, and that matters for
> how §5a reads:** #26 being "one new member" survives **only** as a *local
> classification of the measured `723` residual* — ⛔ never as evidence the global
> matrix became closed.
>
> ✅ **What SURVIVES:** `Need(e) ⊆ Avail(e)`-or-eliminate remains the correct
> governing predicate, and the six ruled dispositions remain the current semantic
> **codomain**. ⛔ What is withdrawn is their claim to cover a closed **domain**,
> until it is re-derived.
>
> ⚠⚠ **AND THE PREDICATE HAS A DIRECTION — the code inverted it.** Measured
> 2026-07-30 (Architect `evt_4ev85skm3pdx9`): `boundary_contract(disposition)`
> derives `Need` **from the chosen disposition**, which **reverses** the equation.
> ⭐ **Planning must derive the exact consumer's `Need` FIRST, then select and
> validate an `Avail` that satisfies it.** ⛔ Corollary, binding: `SemanticEliminator`
> and `SpecializedOnlyLeaf` may **NOT** share a contract arm — the first must
> observe the semantic value in **either** phase via emitted carrier helpers, the
> second needs a compile-time template and must refuse `Carried`. Full ruling and
> its evidence list: **the `264 → 262` effect-argument seat section** under §5.

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

> ⚠⚠ **READ "NO REACHABILITY WHITELIST" PRECISELY — it bans an ORACLE, not a
> PROOF.** Architect `evt_1x47ep8rnhk9p` (2026-07-30) ruled that a **closed,
> exact value-flow proof over the planner graph** is the lawful way to eliminate a
> runtime edge, and is ⛔ **not** the whitelist banned here. ⭐ The discriminator
> is the **source of the fact**: a reached trace, an operation catalog, a test
> name, an origin list, a lowering-time search, or *"no producer was observed"* are
> ⛔ **never** proofs — a **monotone fixed point over the producer graph, where any
> unknown yields `Open`**, is. See **the `FsAppendFile` case-emission section**
> under §5.

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

> ### ⛔⛔ SUPERSEDED AS *POPULATION AUTHORITY* AT STOP #27 — the PROPERTY above
> ### stands, the MECHANISM that was to establish it does not
>
> ⚠ The three sentences immediately above state the right **property** (one
> disposition per edge, every disposition a real edge, consumed exactly once,
> missing fails before emission). ⛔ **What is withdrawn is the claim that a
> source-child matrix plus a lowering-only enum ESTABLISHES it.** Re-read them as
> the obligation, ⛔ never as the design.
>
> **⭐ REQUIRED REPLACEMENT DERIVATION (Architect `evt_4p9ne0vcds5hb` §3).**
> Re-derive `D7` from the **actual owner/phase transition graph**, after the finite
> generated-unit / specialization / continuation fixed point closes. The planner
> materializes **one exact `BoundaryUse`-shaped record per semantic producer
> occurrence** that moves, stores, publishes, forwards, or semantically inspects a
> phase-bearing operand. Names may vary; each record carries:
>
> **CORRECTED 2026-08-04 by `evt_4cbvtjh9dsv6d`. This clause read "one exact
> record per static lowering event", and the allocation census falsifies it: in
> one compilation, 44 of 123 compile/occurrence pairs emit more than one static
> allocation, up to 26.** The record is not an affine allocation token. It proves
> a universal fact about one semantic producer occurrence — the exact
> possible-owner meet of its ordered children and the lane that meet selects —
> and that proof stays true every time the backend replicates the same source
> lowering under the same emission owner. **Duplicating the proof does not
> consume it.** See the relational closure below, which replaces per-record
> count-one.
>
> - producer owner/phase **and** consumer owner/phase;
> - exact source origins, **or** a planner-interned synthesized-edge identity where
>   no source child exists;
> - unit/ABI slot **or** structural parent→child path;
> - the downstream semantic operation and its `Need`;
> - the selected disposition and guaranteed `Avail`.
>
> ⭐⭐ **Source children and synthesized/lowering-only edges inhabit the SAME
> population.** Static recursor worker, callable specialization, continuation
> worker, deferred constructor field, join, environment insertion, carrier child,
> and ABI input/result are **inserted when planning creates them** — ⛔ none may be
> recovered by a lowering-time search or an enum label minted at its consumer.
>
> **Enforcement is at the choke point, not by exhaustiveness:**
>
> 1. make raw phase transitions **private** behind one API;
> 2. require an **unforgeable exact planned-edge token** for every specialized
>    read, forward, carrier transfer/store, environment insertion, join, and
>    semantic elimination;
> 3. **remove ad hoc `.token()` construction** from lowering;
> 4. make `transfer_constructor_operands` consume the **exact parent edge plus one
>    exact child edge per argument**, and descend into **every** nested child
>    obligation **before allocation**;
> 5. **reject any generated edge added after the plan fixed point closes.**
>
> Validation before function definition compares the exact **planned-edge
> relation** against the exact **emitted-consumption ledger**: no missing, extra,
> duplicate, wrong-owner, wrong-phase, or wrong-child-path edge.
>
> ⛔ **CORRECTED 2026-08-04 by `evt_39b1dzgc85gyf`: this sentence also said "or
> unconsumed edge", and for the aggregate-allocation population that is false.**
> An unused planned record is lawful — see the relation law immediately below.
> A no-unconsumed reading survives only where a planned edge genuinely must be
> consumed; it is **not** a general property of `P`.
>
> #### The aggregate lifecycle law — a relation, not a per-record count
>
> **Binding form, Architect `evt_4cbvtjh9dsv6d`.** Let `P` be the closed set of
> planner-issued semantic aggregate occurrence records, keyed by exact emission
> owner plus semantic producer seat plus role/schema and any genuinely semantic
> child/path discriminator; let `E` be the set of actual static
> aggregate-allocation instructions the backend emits; let `R ⊆ E × P` be the
> checked allocation relation. The required closure is:
>
> **CORRECTED 2026-08-04 by `evt_39b1dzgc85gyf`. The clause that stood here
> required `image(R) = P` exactly, "so an unused planned record still rejects".
> That is falsified: the measured artifact carries 1 to 132 lawfully unused
> records.** `P` is a closed **authorization** population, not a closed
> execution-obligation population — the source half admits every source
> `Construct`/`Record` occurrence whether or not it crosses a carrier here, the
> synthesized half is every Effect seat times every lawful emission owner times
> every allocation-reachable recipe-tree use, and the owner population includes
> every generated specialization whose worker subtree contains the seat.
> **An unused `p` is a dormant proof, not a missing emission, and forcing
> surjectivity would reject ordinary valid artifacts.** Surjectivity is dropped;
> no other exactness law is weakened.
>
> Each event is identified by `AggregateAllocationEvent { function: FuncId,
> result: Value }`. The binding closure is:
>
> 1. **`dom(R) = E` exactly.** Every actual governed allocation event is observed
>    once and has exactly one related record, and no relation entry may name a
>    nonexistent event;
> 2. `R` is single-valued on events and every pair is unique — duplicate or
>    conflicting pairing rejects;
> 3. **`image(R) ⊆ P`.** Every related occurrence is planner-issued under the
>    exact closed population. ⛔ There is deliberately **no** `image(R) = P`
>    requirement;
> 4. before raw allocation the selected record reconciles owner, seat, path, full
>    role/schema, shape, lane, ordered children, and opaque occurrence identity —
>    a failure emits no successful artifact;
> 5. **one `p ∈ P` may govern any number of distinct events, including zero;**
> 6. **body lifecycle is exact and append-only** — every opened `FuncId` body
>    commits exactly once after finalization/verification and before
>    `define_function`; no body remains open; opened and committed `FuncId` sets
>    agree; the whole-pass relation is the exact union of committed body
>    relations; and a second build or commit rejects.
>
> ⛔ **Do not define `E` from the keys of `R`.** That makes `dom(R) = E` true by
> construction and leaves the missing-pair mutation invisible. Event evidence and
> relation evidence stay **separate**: each local body holds an event set and an
> event-to-occurrence map; after the raw allocator returns its `Value`, record
> the event, then record the pair; local close requires exact key equality
> between the two; global commit appends both and records the `FuncId` in
> monotone opened/committed censuses; whole-pass close requires global event-set
> equality with relation keys, opened-body equality with committed-body keys, no
> open local body, and `image(R) ⊆ P`. An equivalent private representation is
> fine **only if** the two sets stay independently mutable enough for the
> negative controls to discriminate.
>
> **The ledger stays `FuncId`-local plus whole-pass.** ⛔ No build counter, no
> planner re-key, no ABI change, and **no control-flow reachability planner** is
> authorized — deriving exact allocation reachability would be a second model of
> lowering control flow, which `D7` must not create.
>
> ##### The controls this law binds
>
> Each must be committed, and each must discriminate:
>
> | must be LAWFUL (closeout succeeds) | must REJECT |
> |---|---|
> | the same numeric `Value` under two distinct `FuncId`s is two events | duplicate or conflicting event pairing |
> | one planner record governing many events | an observed event whose relation insertion is suppressed, at local close |
> | **at least one unused planned record** | a relation entry naming no observed event |
> | | committed relation entries cleared between bodies while event evidence remains |
> | | a suppressed or discarded body commit, via open-versus-committed closure |
> | | a second build or commit of one `FuncId` |
> | | a related occurrence absent from `P` |
> | | allocation with no open body |
> | | wrong owner, seat, path, role, shape, lane, or child — refused **before** raw allocation |
> | | each governed-site wrapper bypass, before a successful artifact definition |
>
> ⭐ **The unused-record row is the one that would have caught this defect**, and
> it is a positive control: it fails if surjectivity is ever reintroduced.
>
> **Count one moves from the record to the actual allocation event.** "Duplicate"
> stays illegal for two planner records naming one semantic producer, and for two
> ledger entries naming one actual allocation instruction. It is **not** illegal
> for two distinct CLIF allocation instructions to use the same exact, reconciled
> lifetime proof.
>
> **The boundary this protects:** minting one planner identity per CLIF emission
> would make the semantic plan predict backend expansion, recursive descent, and
> block cloning. That is the wrong layer and buys no lifetime safety. A
> lowering-generated repetition counter is worse — it becomes identity authority
> *after* planning. **The observed CLIF instruction/result is ledger evidence,
> never planner identity; no lowering counter and no per-emission planner key are
> authorized.**
>
> **Implementation boundary.** Keep template attachment and read-only lookup
> repeatable and outside the ledger. Route every aggregate allocation through one
> checked wrapper — conceptually `emit_planned_aggregate_alloc(current_owner,
> occurrence, shape, children, ...)` — which resolves and fully reconciles the
> record **before** the raw allocation, emits exactly one carrier allocation,
> records the actual CLIF instruction/result paired with the record, and rejects
> a duplicate actual event or a conflicting pairing. **The checked wrapper is the
> sole lawful route for the four governed aggregate-allocation sites; other raw
> carrier allocations do not enter `E`, and this law does not reach unrelated
> scalar or spill carriers.** At Function and whole-pass closeout, compare
> **`dom(R)` against the independently recorded event set, and `image(R)` for
> containment in `P`** — ⛔ **never** a per-record count, and ⛔ **never** an
> image-equality check. A failed definition never closes successfully.
>
> **Still binding, unchanged by this correction:** the owner-axis finding
> (`Predeclared` versus `Specialization` stays in the key and the
> reconciliation) and the site-dependent population finding (every fixed use that
> can allocate needs an exact record, or a static non-allocation proof plus a
> fail-closed guard).
>
> ⭐ Rust
> exhaustiveness stays useful for the set of semantic operations and dispositions —
> but **privacy plus the choke-point token requirement is what proves no real
> transition bypasses the population.**
>
> ### ⛔ THE EVIDENCE THIS OWES — the existing named-member mutation is INSUFFICIENT
>
> | control | what it must show |
> |---|---|
> | **#27 population** | the exact `655 → 650 / body 641 / captures 8` use is in the plan **before lowering**; omitting it fails **planning** before any function/object/carrier allocation. ⛔ It may **not** reach the late `Closure` diagnostic |
> | **bypass** | a test-only attempt to call each raw move/read/store/transfer path **without** an exact token is unrepresentable or fails the pre-emission invariant — in particular a raw mixed-constructor transfer cannot bypass the ledger |
> | **synthesized-edge closure** | adding **or** omitting one generated worker/specialization/continuation edge changes the planned set and produces a missing/unconsumed-edge failure |
> | **structural descent** | an authorized **root** carrier does **not** authorize an unplanned **nested** child; omit a nested edge and **all** allocation/publication counters stay zero |
> | **bijection mutations** | duplicate, transplant, wrong-owner, wrong-phase, wrong-position, and unused tokens **all** reject before function definition |
> | **reached-set independence** | the complete static edge count is identical whether or not the named parity row executes a particular runtime branch |
>
> **⭐ PLUS the addendum's four sharpened #27 controls (`evt_3gzcnk62v8bzz`):**
>
> 1. **single-verdict** — the `655 → 650` crossing has exactly **one** planned
>    authority and the carrier gate consumes **that same record**; there is no
>    second identity-free verdict path;
> 2. **key-sufficiency pair** — two `ConstructArgument` crossings with the same
>    nominal role but different proved semantic/provenance facts take their **two
>    different** required outcomes, and collapsing the key back to role-only
>    **reds the pair**;
> 3. **authority-conflict mutation** — restoring the current split (source
>    `SemanticEliminator` **plus** anonymous capsule `EscapeForbidden`) fails
>    planning **before function definition**;
> 4. **no production token** has an absent parent/child/position or a
>    synthesized-edge identity it did not earn.
>
> ⚠ **Control 2 is the load-bearing one** — it is the only entry that can fail
> while every other control passes, because it tests whether the **key** is
> sufficient rather than whether the **outcomes** are complete. That distinction is
> the whole ruling.

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

##### ⭐⭐ RECUT 2026-07-30 — a SECOND arm: `ContinuationSpecialization`

**Architect `evt_7dhwrk26ks9m0` §2.4**, on preservation `93746ada`. The closed
`AbiUnitDefinition` partition gains **one more explicit arm**, conceptually
**`ContinuationSpecialization`**, owned by [[RT-RECURSOR-TRANSPORT]]'s outcome
(b) dynamic case. ⛔ **This is not a new node, disposition, carrier lane, or
atomic participant** — the six-way *operand-disposition* partition above is
**unchanged**. What changes is the **unit-definition** census.

⛔⛔ **It may not be smuggled through `SchedulingEntry`, `ClosureBody`,
`TransparentDeclarationClosure`, or `StaticCallableSpecialization`.** ⭐ The last
one is the trap, because it is the nearest neighbour and the only existing
planner-interned arm:

| | `StaticCallableSpecialization` (#24, above) | **`ContinuationSpecialization`** (new) |
|---|---|---|
| what is specialized | a **transparent callable PARAMETER** | a **caller-owned recursor CONTINUATION / return hole** |
| who owns the identity | the **callee** declaration's parameter | the **caller**, into which a worker result returns |
| key carries | base callee owner/origin + ordered (parameter ordinal, callable body origin, arity, capture provenance) | producer unit/owner + **exact causal producer-result occurrence** + consumer unit/owner + exact checked continuation/frame or suffix identity + recursor parent and recursive/sibling position + worker body identity, arity, ordered capture provenance |
| shared rule | ⛔ **capture VALUES are never key material**; ⛔ intern **before** enqueue/descent; ⛔ zero callable-identity word in the ABI | identical |

⇒ ⭐ **Reusing `StaticCallableSpecialization` for the continuation case is
exactly the "same shape, different meaning" error that the `442`/`723` witness
punishes** — the two keys have different *causal* content, and a unit arm that
cannot tell them apart cannot prove one worker per call edge.

**Validation this arm owes, before descriptor / environment / function / object
allocation** (`evt_7dhwrk26ks9m0` §2.7) — ⛔ all five, ⛔ none deferrable to
emission:

1. discovered specialization keys **biject** defined units;
2. planned **causal** call edges **biject** emitted **direct** call edges;
3. every call edge names **exactly one** specialization and **exactly one**
   worker;
4. every synthesized/source boundary identity and lifetime/provenance obligation
   is **consumed exactly once**;
5. recursive cycles **fold to already-interned identities**.

⚠ **Item 2 is the load-bearing one.** A post-join clone satisfies 1, 3, 4, and 5
while failing only 2 — and 2 is the item whose control (`E-5` in
[[RT-RECURSOR-TRANSPORT]]) requires a **post-join single-call mutation to
fail**. ⭐ Read it as a claim about **where the edge is formed**, not about how
many units exist.

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
    **distinct carried closure-capture** refusal (hard stop **#23**, ✅ **owner
    ATTRIBUTED 2026-07-30**, see immediately below).

  > #### ✅ HARD STOP #23 IS ATTRIBUTED — Architect `evt_21gpwrsewyxax`, 2026-07-30
  >
  > On PR #1251's CI, `buffer_allocate_malformed_capacity_narrows_to_invalid_bounds`
  > reached the **`StaticRecursorWorker` capture boundary**. ⇒ Its owner is **this
  > `D7` / [[RT-RECURSOR-TRANSPORT]] representation closure** — ⛔ **not** a buffer
  > operation, and ⛔ **not** either later syntactic residual node.
  >
  > ⭐ **What the attribution authorizes:** planning and transporting an **ordinary
  > specialized capture** under the existing mixed-phase contract. ⛔ **What it does
  > NOT authorize:** carrying a **nested callable / control capsule**. If the exact
  > capture proves to be one, ⛔ preserve the zero-allocation refusal and return
  > that concrete edge as a **new hard stop** — do not widen scope to absorb it.
  >
  > ⚠ **This retires the earlier ban on acting on this row.** The row is no longer
  > unowned or out of bounds; it is in this pair's population and must go green.

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

  ⇒ ⛔ **The port is NOT additive: it regresses `main`.**

  > #### ⭐⭐ CORRECTED 2026-07-30 BY CI MEASUREMENT — Architect `evt_21gpwrsewyxax`
  >
  > **The regression is real and still stands. What was wrong is the inference
  > drawn from it.** This paragraph used to continue *"`AC-4` (workspace green in
  > CI) is therefore unreachable, and this node cannot become a candidate until
  > its consumers are complete."* ⛔ **That is retired.** It read a defect in the
  > implementation as a defect in the node partition.
  >
  > ⭐ **What CI actually falsified.** The atomic pair *was* measured together, at
  > exact `4dc120c5` on PR #1251: **8 of 12 checks red**, against a `main`
  > (`e79f7af6`, run `30497524438`) where **all twelve are green**. ⇒ The failures
  > are this lineage's regressions, not inherited. But the ruling is that **CI
  > falsified that SHA's sufficiency, not the partition**: `AC-4` **stays
  > literal**, [[RT-SEED-CALL-PORT]] and [[RT-PRODUCER-MATCH-PORT]] **do not
  > fold**, and the existing `D7` + [[RT-RECURSOR-TRANSPORT]] pair **is still the
  > minimal set**.
  >
  > ⭐⭐ **The test that decides whether another node joins an atomic landing:**
  > **a failure must fire that node's own producing predicate.** None of the
  > reported failures fires `SeedClosureCall` (a `Call` whose callee is the
  > retained non-lexical closure form) or `ProducerMatchCall` (an ordinary
  > producer `Match` whose scrutinee is directly a `Call`). ⛔ **A test name and a
  > pre-port refusal text are not ownership.**

  ⭐ **The baseline proxy that found this was not in the original plan and it
  changed the outcome.** Every measurement before it carried a delta, so the
  regression was invisible to all of them — and the factoring above was, before
  it ran, about to be used to argue this node could land. ⚠ **A port measured only
  with its consumers' inputs applied has not been measured against `main`.**

  ### ⛔⛔ THE BINDING REPAIR BOUNDARY AFTER THE 2026-07-30 RECUT

  **Architect `evt_21gpwrsewyxax`.** Exact `4dc120c5` is an **incomplete
  implementation of this pair**, and both CI refusal classes are **inside** it.
  ⛔ This section is an acceptance obligation, not commentary.

  #### The two diagnosed defects

  1. **`StaticRecursorWorker` is an in-scope capture-contract defect.** The
     unified identity stores the ordered `StaticRecursorCaptureProvenance`, but
     the move-only token exports **only `capture_count`**;
     `validate_static_recursor_worker_residual_identity` revalidates **only that
     count**; and `prepare_planned_static_recursor_worker` then rejects every
     capture that is not **already** `LoweringOperand::Carried`. ⇒ That
     projection **loses the per-capture phase / owner / provenance the frame
     requires.** ⭐ `D7`'s binding capture rule is **narrower and more capable**:
     each capture retains its own phase; an already-carried ordinary value passes
     unchanged; a specialized ordinary value crosses the one-way producer
     **exactly once**; a nested callable / control capsule still fails **before**
     allocation. ⛔ [[RT-RECURSOR-TRANSPORT]] §8c requires an **ordinary
     transferable lane**, ⛔ not *"already carried or reject."*

  2. **The late generic `Closure` refusal on the `fs_*` rows is an in-scope `#26`
     population omission.** This frame already attributes those rows, after the
     port, to [[RT-RECURSOR-TRANSPORT]] rather than the syntactic ProducerMatch
     retirement. ⭐ **The matrix-omission law is explicit: a real static-worker
     residual that is omitted or misclassified must FAIL IN PLANNING, and may
     NOT fall through to the late generic `Closure` refusal.** Exact `4dc120c5`
     reaches that **forbidden late fallback** ⇒ the graph-derived `#26`
     population is **still incomplete**. Repair the **planner-issued exact
     edge**; ⛔ do **not** weaken `CallableCapsuleEscape -> EscapeForbidden`.

  #### What the replacement owes

  - **Carry and revalidate the COMPLETE ORDERED capture contract at
    consumption** — ordinal, source provenance, owner, expected phase / lane,
    lifetime, and exact-once producer authority where needed. ⛔ `capture_count`
    alone is insufficient. Preserve **one** exact static-worker identity.
  - **Preflight the ENTIRE environment before allocation.** Carried ordinary
    captures pass unchanged; specialized ordinary captures use the existing
    one-way producer **exactly once**; nested callable / control captures remain
    **fail-closed with every allocation / publication counter at zero**.
  - **Close the `#26` result-flow population for every CI-reached exact
    recursive-position closure.** An omitted real member must **red in
    planning**; ⛔ no exact member may reach the generic whole-`Closure` arm; and
    the **same** closure **outside** an exact planner-proved edge must **still**
    reach that arm.
  - ⭐⭐ **Record a per-row FIRST-REFUSAL MAP for every currently failing job and
    test — not only the two example rows.** ⛔ Never a count (see the `1/7`
    lesson above). ⛔ **If a third production predicate appears — or an actual
    `SeedClosureCall` or `ProducerMatchCall` appears — HARD STOP and return the
    concrete edge before widening scope.**
  - **Add mixed-phase worker controls:** at least one environment containing
    **both** carried **and** specialized ordinary captures; phase / owner /
    order / omission mutations; the nested-capsule **zero-allocation** negative;
    and the exact-member versus same-closure-outside-edge pair.

  #### ⛔⛔ THE CARRIED SOURCE-MACHINE `Match` ROUTE — Architect `evt_7tqhf9d7wsvzw`

  ⭐ **The hard-stop control two bullets above FIRED and was discharged
  in-scope.** Acting on the authorized `#23` specialized-ordinary capture
  crossing reached origin **271**, where an ordinary `Match` over a `Var` that
  loads the transported capture as `LoweringOperand::Carried` was refused.
  ⛔ **That is NOT a third producing predicate and it adds no node, member,
  disposition, or lane.** `RuntimeExpr::Var(_)` is a *source access form*, not a
  production predicate — partitioning on it would recreate the very derivation
  error `D7` withdrew. The classification is: **producer** = the already-attributed
  `#23` mixed-phase static-worker capture transport; **consumer/owner** = *this*
  node's ordinary `Match` as a `SemanticEliminator`, specifically its
  **source-machine continuation route**, which `D7` already names. The refusal
  fires only because `SourceContinuation::MatchScrutinee` enumerates specialized
  shapes and **has no `Carried` arm**. Atomic membership is unchanged; the buffer
  test is only the **reaching witness** and owns nothing.

  **The binding lawful route.** At `SourceContinuation::MatchScrutinee`, after
  consuming the exact planner-issued `SemanticEliminator` authority, **dispatch on
  phase BEFORE any specialized-shape test**:

  1. `Carried(word)` uses the existing `D7` carrier ABI semantics — runtime
     class / tag / arity, planner-issued case identities, projected children
     **remaining `Carried`**, and the closed default. ⛔ No decode, no
     reconstruction, no `Carried -> Lowered` conversion.
  2. `Specialized(...)` retains every existing source-machine path **unchanged**.
  3. ⛔ **The carried path must preserve the ENTIRE remaining `SourceControl`.**
     ⛔ Do **not** call the direct `lower_carried_match` helper as-is and return
     its isolated result — its case bodies use direct/producer lowering and do
     **not** carry this source continuation. Factor a source-machine carried-match
     route from the existing source branch machinery: split/instantiate the
     source-prefix template, preserve an inherited affine join when nested, mint
     **one predecessor per runtime-selected arm**, lower each selected case with
     `lower_forked_branch` under the **original** `next`, union branch-local frame
     consumption, consume the match's planned join **exactly once** where locally
     owned, and resume the suffix **exactly once**.
  4. ⛔ This authorizes **no** generic `Match` weakening, origin whitelist, buffer
     special case, anonymous token, new carrier lane, or whole callable /
     control-capsule transport.
     > ⚠⚠ **"NEW CARRIER LANE" IS NARROWLY SUPERSEDED — Architect
     > `evt_5ks9da0h0977w` (2026-07-30).** The `MkProgramCaps` lifetime ruling
     > **does** admit **one** explicit `InvocationAggregate` tag over exactly
     > `{Constructor, Record}`. ⭐ Everything else in this clause stands, and the
     > supersession is **only** that row family — ⛔ no new arena, word width,
     > scalar encoding, constructor identity, class, host ABI, or durable lane.
     > See **the `MkProgramCaps` invocation-aggregate section** below.

  **Required evidence before candidate routing** — ⚠ these are acceptance
  obligations, and QA reviews against them:

  - the exact origin-**271** linked-native row advances through the carried
    ordinary source-machine `Match` and reaches its intended `InvalidBounds`
    result;
  - a source-machine ordinary `Match(Var)` over a **carried** represented
    constructor / `HostResult` selects the right case, validates arity, and leaves
    bound fields **`Carried`**;
  - ⭐ a **nontrivial post-match source continuation** is observed, so
    dropping/bypassing `next` **reds** the control;
  - an inherited / nested source join preserves **distinct** affine predecessor
    edges and **exact-once** join-plan consumption;
  - the same **specialized** source-machine match remains green;
  - removing the carried-first route **restores the exact refusal**, and
    identity / arity / field / order mutations **red**.

  ⇒ Every `#23` / `#26` mixed-phase, zero-allocation nested-capsule, population,
  and per-row first-refusal obligation above **remains owed on top of these**.
  Exact `fb8fd881` is **preservation-only, not a candidate**, and is **superseded
  history** — see the governing-base correction immediately below.

  #### GOVERNING BASE — corrected 2026-08-05, Architect `evt_73b1t4nkewftw`

  **The governing base for every remaining `D7` move is exact
  `708875292fdf1d81f13320d61baeb5a84a7e7bbf`**, which descends from `origin/main`
  `e6b4a13b`. **Do not rebase, merge, or cherry-pick `fb8fd881`, `430798bf`,
  `548682c3`, or `42ccd8ec` into it.**

  > **RESUME POINT, 2026-08-05 — exact `6a09ed6840bc334af25015ec57b69ec884e8705c`**
  > (Architect `evt_6vr00htkk6cpf`), a descendant of `70887529` on this lineage.
  > **Preserve it; it is accepted WIP substrate, not a candidate.** Accepted in
  > it: the full 13-operation population, the capability-as-own-seat correction,
  > exact seat/owner coordinates, per-seat `Need`/`Avail`, the key-axis controls,
  > and the independent contract recomputation.
  >
  > **What it is not:** the accepted population/ledger closure. Two clauses of
  > this section were false and are recut below — planned producer/consumer
  > **phase** fields, and the global `planned = consumed` / unused-token
  > rejection. Runtime resumes from `6a09ed68` and **replaces only the coarse
  > ledger/claim lifecycle** before proceeding to operation-specific consumption.
  > The lowering and exact-`Int` slice remain held.

  > **RESUME POINT ADVANCED, 2026-08-05 — exact
  > `ae64f687fe1cc5a0d2bcac8ec78a8fb0b9819443`** (Architect `evt_4enxcfe742ta8`),
  > a descendant of `6a09ed68` on this lineage. **Preserve it; it is accepted
  > WIP substrate, preservation-only, not a candidate.** It supersedes the
  > `6a09ed68` resume point above, which stands as the record of what was
  > accepted when.
  >
  > Accepted in it, cumulative over `727b51a1` (per-visit claim group),
  > `69c68e6e` (pre-definition body close) and `f6958b95` (operation-arm claim
  > consumption): reply projection is lazy and exact-use-driven through the
  > claimed `SiteOperand`; `BufferAllocate` consumes its exact capacity seat with
  > no dense conversion reappearing; the specialized and carried exact-`Int`
  > routes converge on one `(sign, len, limbs)` rule; the persistent branch uses
  > `sign == 0 && len == 1` and reads limb 0 only on that path; malformed
  > carrier state stays distinct from semantic `InvalidBounds`; and the phase,
  > taxonomy, seat, disposition, closure and sign-bit controls are landed.
  >
  > **What it does not discharge:** the exact reaching-row, lowering-closure, and
  > cumulative no-regression obligations — because the linked row never reaches
  > this machinery. See "Required acceptance evidence" item 1, which is recut
  > into ordered checkpoints for exactly that reason.

  > **RESUME POINT ADVANCED AGAIN, 2026-08-05 — exact
  > `50092c59e21720cc8ad7c102fb6ceb925da4ecdd`** (Architect `evt_50tq0x2qy5489`),
  > two commits on `ae64f687`. **Accepted checkpoint 1, preservation-only, not a
  > candidate.** Runtime lib 715 passed / 2 standing reds, unchanged.
  >
  > Accepted in it: both retained callable forms carry `Vec<LoweringOperand>`
  > while their outer capsules stay `Specialized` and unconditionally
  > non-transferable; lexical captures retain their exact phase and seed captures
  > are explicitly `Specialized`; the generic retained-callable seat and
  > `StaticWorkerBinding` share one complete pre-emission capture gate, reusing
  > the existing worker-template and ABI-slot authority to check body/source
  > identity, arity, count, ordinal, provenance, owner, phase/lifetime, and
  > whole-slot equality **before allocation**; every capture consumer is
  > phase-exhaustive; and no callable carrier, marker change, planner population,
  > ABI shape, tag, inverse, or boundary lane was added.
  >
  > **The evidence that makes it an advance, per-row and never a count:** the
  > linked row's first refusal moved off the closure-capture seat, and the five
  > `fs_*` rows keep their exact framed `D0` refusal text — so nothing regressed
  > to buy the advance.
  >
  > **What it does not discharge:** the row still does not reach this node's
  > consumer. Its new first refusal is the Host-`Vis` producer arity
  > inconsistency, which checkpoint `1b` below owns.

  Those tips and `70887529` share merge-base `f7cea8fd` and are **competing
  historical implementations, not independent halves awaiting assembly.** The
  older line lacks the accepted source-aggregate preflight, the typed `Record`
  schema, and the aggregate E/R relation. `70887529` lacks the older
  role/disposition contract machinery — and importing it would **reintroduce the
  role-/disposition-derived schema this frame's own host-effect ruling declares
  false**, which is why the assembly is prohibited rather than merely costly.

  **Every sentence below calling one of those four SHAs "the exact continuation
  base" was stale and is corrected in place.** They remain citable as **evidence
  of what was ruled when, and of the original refusal** — never as a place to
  resume from. This restates, at the point of work, what
  **THE BASE IS NOW DURABLE AT `origin`** already says further down: a preserved
  ref is a recovery point, not a resume instruction.

  **The measurement that forced this correction** — runtime-implementer,
  2026-08-05, `evt_2t4h5v0p7kwph`, stopping before any edit rather than picking an
  interpretation. `fb8fd881` is **not an ancestor of `70887529`**. Of ten symbols
  probed, the six the older line carries — `specialized_source_env_at`,
  `boundary_contract`, `BoundaryUseIdentity`, `SemanticEliminator`,
  `SpecializedOnlyLeaf`, `EffectArgument` — appear in **zero** files on the
  governing base; the four the governing base carries —
  `specialized_operands_at`, `source_aggregate_preflight`, `LoweredRecordField`,
  `AggregateAllocationLedger` — appear in **zero** files on `430798bf`.

  ⇒ **A frame that names a base and separately names what must be preserved has
  told a truth only if the base carries it.** Those two sentences were written
  three lineage-tips apart and were never reconciled; the block above is that
  reconciliation, and it is the reason this recut exists.

  #### ⛔⛔ THE `264 → 262` EFFECT-ARGUMENT SEAT — Architect `evt_4ev85skm3pdx9`

  **Evidenced — not based — on exact `430798bf`** (tree `4653af25`, parent exact
  `fb8fd881`), which advances origin 271 and then first-refuses at:

  ```text
  BoundaryUseIdentity::Source { parent: StaticOriginId(264),
                                child:  StaticOriginId(262),
                                position: 1 }
  role = EffectArgument            disposition = SemanticEliminator
  BoundaryCarrier: an effect argument is a specialized-only surface …
  ```

  **That is the ORIGINAL REFUSAL as printed by machinery that does not exist on
  the governing base**, and it is quoted here only to fix **which semantic seat**
  is at issue: effect origin 264, child origin 262, structural position 1. On
  exact `70887529` the same seat refuses **generically** — a host-effect operand
  is a specialized-only surface — carrying **no identity at all**. ⇒ Read every
  `BoundaryUseIdentity` / `SemanticEliminator` / `SpecializedOnlyLeaf` /
  `EffectArgument` spelling in this section as **the historical vocabulary of the
  evidence**, never as a symbol to produce, remove, or assert against.

  ⛔ **This is an ALREADY-PLANNED `D7` semantic-eliminator seat — not a new
  producing predicate, node, lane, disposition, atomic participant, or lineage
  assembly. Atomic scope remains `D7` + [[RT-RECURSOR-TRANSPORT]].** `#23`
  remains the producer: it delivers a **carried
  exact `Int`** to the capacity seat of the statically selected `BufferAllocate`
  effect. `D7` owns the consumer — a host-effect argument that semantically narrows
  that exact `Int` for the wire request. `RuntimeExpr::Effect` and `EffectArgument`
  were **already** in the static source inventory and the exact edge already
  exists; no Seed/ProducerMatch predicate fires. The buffer parity row is, again,
  only the **reaching witness**.

  ⭐⭐ **THE DEFECT IS THAT TWO ARTIFACTS LIE ABOUT WHAT `SemanticEliminator`
  MEANS.** It has two coupled halves:

  1. **Effect lowering consumes the wrong authority.** `lower_process_host_effect`
     lowers each argument as a `LoweringOperand`, but for every
     non-`BufferFreeze` operation it bulk-converts the **entire vector** *before*
     entering the operation/seat-specific match. ⇒ A token standing for a
     semantic-carrier observation is consumed as authorization for a
     **specialized-only read**, and the carried word is refused before the effect
     can perform its ruled emitted-helper observation.

     **On the governing base this call is spelled `specialized_operands_at`**
     (measured 2026-08-05; the historical spelling `specialized_source_env_at` has
     zero occurrences there). **The mechanism is the same and this half of the
     defect is fully present** — which is what makes sections B and C buildable on
     `70887529` without any lineage assembly.
  2. **⛔ The contract schema is under-keyed and FALSE.**
     `boundary_contract(disposition)` maps `SemanticEliminator | SpecializedOnlyLeaf`
     to **one shared arm** — `consumer_phase = SpecializedValue`,
     `operation = Inspect`, `need = ReadSpecializedTemplate`,
     `avail = SpecializedTemplate`. That contradicts `D7`:
     `SpecializedOnlyLeaf` needs a compile-time template and **must** refuse
     `Carried`; `SemanticEliminator` **must** observe the semantic value in
     *either* phase, using emitted carrier helpers when `Carried`.

  ⭐⭐⭐ **AND NOTE THE DIRECTION OF THE ERROR — it inverts this frame's own
  governing predicate.** A `Need` derived *from the chosen disposition* **reverses
  the equation**: planning must derive the **exact consumer's `Need` first**, then
  select and validate an `Avail` that satisfies it. ⇒ This is **not** a new matrix
  member; it is an **existing member whose recorded contract is under-keyed and
  false** — the same defect class as stop #27, one layer further in.

  ##### A. Re-derive effect uses by SEMANTIC SEAT, not by role

  For **all 13** operations in `CRANELIFT_HOST_EFFECT_CONSUMERS_V1`, planning must
  enumerate every capability / argument seat **after** the operation and the
  conditional capability offset are known. Each exact planned record binds at
  least: effect origin + child origin + structural position · host operation +
  semantic argument ordinal/seat · producer owner/phase + consumer owner/phase ·
  the seat-specific semantic operation and `Need` · the selected disposition and
  guaranteed `Avail`.

  **BUILD A FRESH EFFECT-SEAT AUTHORITY. Do not reconstruct the historical
  vocabulary.** Corrected 2026-08-05, Architect `evt_73b1t4nkewftw`. **This clause
  read "Remove the shared `SemanticEliminator | SpecializedOnlyLeaf` contract
  arm."** That arm does not exist on the governing base, so the instruction has
  **no referent**, and its mutation control would be **vacuous**: restoring an arm
  that was never there proves nothing. Do not produce `BoundaryUseIdentity`,
  `OperandEdgeDisposition`, `SemanticEliminator`, `SpecializedOnlyLeaf`, or
  `EffectArgument` on `70887529`.

  **The requirement the removal was standing in for is kept in full**, and is now
  stated positively. One **planner-issued record**, keyed by the full semantic
  seat:

  - effect origin, exact child origin, and structural position;
  - admitted host operation and semantic ordinal, **after** the conditional
    capability offset;
  - exact producer and consumer **owners**; and
  - the seat-specific semantic operation and consumer-derived `Need`, followed by
    the set of operand **phases in which that `Need` is lawfully dischargeable** —
    the per-seat `Avail`.

  > ##### PHASE IS EVIDENCE, NOT PLANNER AUTHORITY
  >
  > Recut 2026-08-05, Architect `evt_6vr00htkk6cpf`.
  >
  > **This bullet read "producer and consumer owner/phase", and the planned-phase
  > half of it is withdrawn for this effect-seat population.** The declared-ABI
  > witnesses falsify child-join representation as that authority, and the
  > measured substitution the implementer made is the correct one.
  >
  > **Do not invent another statically predicted producer/consumer phase.** Owners
  > stay planned. Admissible phases stay planned, as `Avail`. The actual
  > `SpecializedTemplate`-versus-`CarriedWord` phase is **emission evidence** —
  > observed from the exact operand at the exact consumption seat and checked
  > against that planned `Avail`. It must not be reverse-derived from a child or
  > an ABI result.
  >
  > **Lifetime is not a replacement phase oracle.** These seats immediately
  > observe a value rather than retain it; owner/lifetime validity stays with the
  > typed carrier/helper and aggregate authorities that actually govern the
  > representation.
  >
  > **One correction is owed where the claim moves into the operation-specific
  > arm: retain the observed phase in the independent claim evidence, and bind the
  > returned claim to the same operand/arm that performs the read.** Exact
  > `6a09ed68` checks the phase and then collapses it out of the ledger. **A check
  > whose evidence is discarded, followed by a later independent read, is not the
  > closed relation this gate needs** — the two reads can disagree and nothing
  > sees it.

  **`Need` is derived FIRST from the exact operation/seat; `Avail` is THEN checked
  to satisfy it** — never the reverse, which is the inversion this whole section
  exists to correct. **Operation, ordinal, and `Need` are part of the authority
  and part of its equality.** A generic "argument", a role, a disposition, an
  origin label, or a diagnostic string **is not authority**.

  **Keep this type SEPARATE from the continuation-specific `BoundaryUseNeed` /
  `Avail` projection already present on `70887529`.** Widening that unrelated
  continuation record to carry effect seats **conflates two consumers again** —
  the identical defect one layer out, and the reason a shared arm was wrong in
  the first place.

  Two argument seats with the **same structural kind** but different seats —
  `BufferAllocate.capacity : exact Int → checked u64` versus
  `ConsoleWrite.bytes : Bytes → pointer/length` or
  `ResourceRelease.resource : opaque token → scalar` — **must remain distinct
  records carrying different `Need`s.**

  ##### B. Consume ONE exact seat token inside the operation-specific arm

  ⛔ Do **not** bulk-convert the argument vector through
  `specialized_operands_at`. Each semantic seat phase-dispatches
  **exhaustively, with no wildcard**: `Specialized(value)` uses the existing typed
  specialized reader; `Carried(word)` calls the carrier helper(s) that satisfy
  **that seat's recorded `Need`**; and a seat for which planning cannot prove
  `Need ⊆ Avail` is **eliminated before emission or rejects in planning** — ⛔ it
  may **not** survive to the generic specialized-only diagnostic.

  ⭐ **This is a COMPLETE EFFECT-SEAT CLOSURE**, ⛔ not a `BufferAllocate`,
  origin-264, or test-name whitelist. **A later carried effect seat cannot be
  discovered one CI row at a time.**

  ##### C. Exact lawful capacity transport

  The capacity seat needs only a checked `Int → u64`, ⛔ **not** a `Lowered::Int`
  template. Factor **one** carried exact-`Int` narrowing route over the existing
  emitted carrier ABI:

  - **`ImmediateInt`** — validate that exact tag, obtain the signed scalar through
    the carrier scalar helper, report `valid` **iff** it is nonnegative;
  - **persistent `BoundaryClass::Int`** — obtain the canonical `{sign, len, limbs}`
    through the emitted `int_view` helper, report `valid` **iff** the canonical
    magnitude fits one unsigned limb (`sign == 0 && len == 1`), loading limb 0
    **only** on that valid path;
  - **every other** tag/class, unsealed magnitude, invalid helper status, or
    owner/shape violation **fails closed as a CARRIER ERROR**. ⛔ It must **not**
    be relabelled `InvalidBounds`.

  Return `(u64_value, valid)` **directly in emitted code**. ⛔ Never construct
  `Lowered::Int`, reconstruct a compile-time template, decode in Rust, or inspect
  a JIT-time constant. ⛔ The specialized and carried routes **must share one
  stated range rule** — no second magnitude encoding, no hand-written wide-`Int`
  decoder. The existing `BufferAllocate` narrowing flow then remains
  authoritative: `valid == false` records detail `7`, synthesizes
  `ResourceError::InvalidBounds`, and performs **zero host dispatches**;
  `valid == true` writes the request and dispatches normally.

  ##### Required acceptance evidence — ⚠ QA reviews against these

  1. **Exact reaching row.** The row
     `buffer_allocate_malformed_capacity_narrows_to_invalid_bounds`
     advances through origin 271 **and** the exact semantic seat
     `264 → 262 / position 1`, returns `InvalidBounds`, and records **zero**
     shared-host dispatch events.

     **Corrected 2026-08-05 (`evt_73b1t4nkewftw`): the row must carry the NEW
     planner-issued seat identity for that effect/child/position. The
     `BoundaryUseIdentity` spelling and the old diagnostic string are NOT
     required and must not be reconstructed.**

     **RECUT 2026-08-05 (Architect `evt_4enxcfe742ta8`) — the single before/after
     pair this item used to demand is unmeasurable, because its chronology is
     false on the governing lineage.** It said that *before* the repair the seat
     reaches the generic host-effect specialized-only refusal. The row never gets
     that far: it refuses earlier, at the `#23` producer, as a closure capture.

     **This frame already measured that, and the clause contradicted it.** The
     exact per-row `D0` table above records this row's base result as
     `FAIL — BoundaryCarrier: a closure capture is a specialized-only surface`,
     measured 2026-07-29 on `483ef7ab`. Runtime's 2026-08-05 bisection agrees and
     locates when the current lineage exposed it: `origin/main` `9bceb8c5`
     through `c2be588b` pass, `c85179ce` (`D5a` checkpoint 4 step 3, consuming the
     checked-IH marker at the static-worker call edge) is the first failure, and
     the message is unchanged from there through `ae64f687`.

     **That bisection says when the lineage exposed the refusal, not who owns
     it.** The attribution this frame already carries stands unchanged: producer
     is the `#23` mixed-phase static-worker ordinary-capture transport, consumer
     is this node's exact `264 -> 262 / position 1` semantic-eliminator seat. The
     row stops at the producer, so it has not reached the consumer. **The `D5a`
     checked-IH law — consume the marker on the complete application occurrence
     before the static-worker call is emitted — is accepted, and may not be
     reverted, weakened, or special-cased to make this row advance.**

     **The control is therefore two ordered cumulative checkpoints, not one
     pair.** Neither may be skipped, and the second may not be measured first:

     1. **Make the retained callable's capture edge phase-bearing, and close its
        pre-emission gate** — under the contract this frame already states in
        *What the replacement owes*: the complete ordered capture contract
        revalidated at consumption (ordinal, source provenance, owner, expected
        phase/lane, lifetime, exact-once producer authority — `capture_count`
        alone is insufficient); the entire environment preflighted before
        allocation; carried ordinary captures passing unchanged; specialized
        ordinary captures crossing the existing one-way producer exactly once;
        and nested callable/control capsules still refusing before allocation
        with every allocation and publication counter at zero. `D5a`'s marker
        semantics are preserved, not adjusted.

        > **LOCALIZED 2026-08-05 (Architect `evt_4wgqmy49rtszv`) — this
        > checkpoint said "restore the framed `#23` producer", and the `#23`
        > producer is not what refuses.** Measured on `ae64f687`: `lower_binder`'s
        > mixed-phase route is present and correct, and the row never reaches it.
        > The first refusal is the **generic `LexicalClosure` value arm of
        > `lower_expr`** — `lowering/core.rs`, the site commented *D7, site 2 of
        > 3* — which has no mixed-phase route and folds unconditionally through
        > `specialized_operands_at(.., "a closure capture")`. The exact seat is
        > closure origin `381` / body origin `375`, five ordinary captures with
        > phases `S,S,C,C,C`, at **constructor-field position 1** of `Vis` parent
        > `386`. Body `375` is already in `worker_templates`, so this is a **real
        > planned static-worker member reaching the late generic `Closure` arm** —
        > the case this frame's own matrix-omission law says must fail in
        > planning and may not fall through there.
        >
        > **It is not a capsule**, so the nested-callable hard stop does not fire;
        > `D5a`'s marker semantics were neither reverted, weakened, nor
        > special-cased. The `#23` attribution above still holds for **ownership**
        > — this is that population — but the **repair site** is the
        > closure-capture edge, not the producer.
        >
        > **The mechanism is already framed and needs no exception.** Per the
        > *Row: the closure-capture cell* section, `Closure` and
        > `DeclarationClosure` stay specialized invocation-local control capsules
        > while their **capture edges become `LoweringOperand`**, expressly
        > superseding `C1`/`AC-C4`'s single-field statement for capture edges
        > only. So: `Lowered::Closure.captures` and
        > `Lowered::DeclarationClosure.captures` become `Vec<LoweringOperand>`; a
        > carried capture gains **no** `LoweredVariant`, disposition, encoding,
        > inverse conversion, carrier tag, or durable slot; site-2 lexical
        > captures keep the operands lowering already produced (`S,S,C,C,C` stays
        > `S,S,C,C,C`); seed captures are wrapped explicitly `Specialized`;
        > `lower_binder` and `StaticWorkerBinding` keep their narrower semantics,
        > so a constructor field does **not** become a `StaticWorkerBinding` and
        > the value / aggregate-field / match-scrutinee refusals stay correct.
        > Every consumer of the widened field dispatches
        > `Specialized`/`Carried` **exhaustively** — no wildcard, no
        > `specialized_operands_at` fallback.
        >
        > **The Architect ruled that no recut was required to unblock this**; the
        > correction above is to the checkpoint's own localization, so QA does not
        > verify a producer that was never broken. No scope, node, lane, or
        > disposition changes: no `AC-C4` exception, no callable carrier, no
        > `B2F` lane, no planner-population or ABI-shape widening.
     **1b. Correct the Host-`Vis` computational-IH producer so one application
        has ONE arity.** Added 2026-08-05, Architect `evt_50tq0x2qy5489`, on
        accepted checkpoint 1 (`50092c59`). **This is numbered `1b` rather than
        renumbering the consumer**, so that "checkpoint 2" keeps meaning what
        every post in this thread already uses it to mean.

        > **Why it exists.** Checkpoint 1 did not create this refusal; it made
        > the row *reach* an existing fail-closed check. With the closure-capture
        > seat passed, the row's first refusal is now
        > `OrientedSubcontinuationPlanV1: computational IH invocation marker
        > wraps a call of 1 arguments but its checked template names arity 0`.
        >
        > **One application is described by two arities.** The Host-`Vis`
        > producer records `CheckedComputationalIHCallSeed::arity =
        > checked_arguments.len()` — so a nullary force records `0` —
        > `lower_checked_host_computation` consumes that seed at the same checked
        > count, and then that same `Vis` route builds the complete Runtime call
        > by lowering the checked arguments **and appending
        > `RuntimeExpr::Var(0)`, the host result**. The emitted application
        > therefore carries 1 argument against a template naming 0.
        >
        > **The marker law is correct and is NOT what changes.** A
        > `CheckedComputationalIHCallTemplateV1` describes one complete
        > application, and both marker entry and static-worker consumption
        > compare the wrapped `RuntimeExpr::Call` argument count against the
        > immutable `arity`. ⛔ Do not relax, special-case, or widen those
        > checks to make the row advance — the defect is the producer naming an
        > incomplete application, not the check catching it.

        > **RECUT 2026-08-05 (Architect `evt_68e5svw1eeenp`) — this checkpoint's
        > first statement of what it owes was FALSE, and the implementation was
        > right to contradict it.** It said "make the seed and template name the
        > full erased application" and demanded a **nullary Host-`Vis` be
        > `1 / 1 / 1` at seed, marker body, and consumer. That **collapses two
        > distinct lawful coordinates into one number.**
        >
        > **The honest measured relation is `source-seed 0 / emitted-template 1 /
        > marker-consumer 1`.** Forcing the source seed to `1` is wrong for
        > non-injecting nullary forces, and the first attempt proved it: a
        > producer-global count moved the five non-injecting `fs_*` rows **off
        > their framed `D0` refusal text**. ⛔ Do not contort the code to satisfy
        > the old premise. **The per-row table is what caught this** — a pass/fail
        > total reads `1/7` before and after, identically, and hides it entirely.

        **What the correction owes — two coordinates, named separately.**

        - **Source occurrence arity** — the checked seed-binding count. It binds
          the checked/source application occurrence and **stays as it is.**
        - **Complete emitted arity** — the immutable template / marker / consumer
          count, naming the complete erased Runtime application.
        - **Host-`Vis` relates the two by exactly one planner-known injected
          host-result operand. Every non-injecting route relates them by zero.**

        **The delta may not be an unconstrained count.** Derive it from a closed
        route / injected-operand kind, or an equivalent **exhaustively matched**
        representation; use **checked** addition and conversion; and **retain the
        exact source-binding check** unchanged. An open integer that happens to be
        `1` on the measured path is the same defect in the other direction.

        **Required controls — they must drive the REAL routes.** The prior four
        did not: they hand-constructed the slot, source seed, frame, and raw
        injected operands, never traversed the Host-`Vis` producer, and asserted
        only that a number was less or greater than a separately computed
        `source + 1`. **Neither mutation was ever fed through marker entry or
        static-worker consumption, so neither was shown to refuse — and both
        would stay green if that pre-emission rejection disappeared.** A
        `debug_assert_eq!` over the same arithmetic is not an independent
        release-build oracle.

        - a **real nullary** Host-`Vis` application produces full template arity
          **1**; a **real n-argument** one produces **n + 1**;
        - a **real ordinary non-Host** route remains **n** — the scoping half,
          without which every other row is equally consistent with a global shift;
        - `cfg(test)`-only **omit** and **double-count** mutations **at the
          Host-`Vis` route** each **reach the existing Runtime marker gate and
          refuse before function definition or emission**. Both directions: a
          correction that can only be wrong one way is half-measured.
        - all existing `D5a`, checkpoint-1, and per-row `D0` controls remain
          **cumulative**.

        **Scope.** Existing `D7` / [[RT-RECURSOR-TRANSPORT]] atomic scope. No new
        node, carrier lane, disposition, representation, or third participant.
        Exact `4ec5362c` is **preservation-only partial progress** — its component
        boundary is correct and should be retained; it is not an accepted `1b`.
        **No candidate, no QA route, and no checkpoint-2 work until a corrected
        `1b` is accepted.**

     **1c. Enforce the matrix-omission law in PLANNING.** Authorized by the
        operator 2026-08-05; sequencing confirmed by the Architect at
        `evt_68e5svw1eeenp`.

        > **RECUT 2026-08-05 (Architect `evt_41v9zkjaf52n6`, Research advisory
        > `evt_72mj37q3pecv4`) — this checkpoint's FORWARD reading was false on
        > the current authority plane, and is retired.** It said body `375`'s
        > presence in `worker_templates` while reaching the late generic `Closure`
        > arm is **a planned member falling through**. It is not.
        >
        > **Three populations, measured, and the frame collapsed them:**
        >
        > - **B** — generic `ClosureBody` members;
        > - **C** — edge-local specialization candidates with recursive position
        >   and worker provenance;
        > - **I** — candidates **interned** only after an exact **closed** producer
        >   environment is proved.
        >
        > Closure `381` and **22 measured green peers** are in `B ∩ C`, but their
        > required environment is `Open`, so none is in `I`.
        > `exact_continuation_source_environment` makes that decision **before**
        > interning. **`Open` is positive fail-closed evidence meaning "do not
        > commit this specialization" — not "a planned member was omitted" and not
        > "reject the enclosing program."**
        >
        > ⛔ **So this checkpoint may enforce `I -> exactly one matching B`. It may
        > NOT enforce `B ∩ C -> I or reject`** — that turns inability to prove an
        > environment into source-program invalidity, and would falsely reject 23
        > measured green programs including the landed
        > `contspec_open_and_ambiguous_sources_refuse_only_the_candidate` law.
        >
        > ⛔ **Closure-level membership is not an edge-local obligation.** Measured
        > closure `13` is declined on one edge and interned on another, so a
        > closure-level refusal collapses edge-local opportunity into member-global
        > obligation — a cardinality error independent of the 22 peers.

        **What `1c` is, and it is CLOSED.** Exact
        `179af86350ba7191935fcc9ff902bb166c954339` (on `ca1c4418`) lands the exact
        lawful converse and discharges this checkpoint:

        ```
        interned specialization
            -> exactly one ClosureBody member
               at the defining closure occurrence
               with the exact declared parameter/capture contract
        ```

        Preserve that mechanism and its omission, reclassification, redirection,
        and wrong-contract controls — they are non-vacuous **on an actually
        interned specialization**.

        **Sequencing.** After `1b`, before checkpoint 2, for the reason recorded
        when it was authorized: enforcing in planning moves where the linked row
        first refuses, and `1b`'s evidence is a per-row before/after on that row.

     **1d. The route/availability authority INVENTORY for the linked edge.**
        Added 2026-08-05, Architect `evt_41v9zkjaf52n6`. **This is a bounded
        investigation whose first and possibly only deliverable is an answer, not
        a mechanism.**

        **The open question, stated exactly.** Closure `381`'s late refusal is
        real: it lacks a viable eventual route. But that is **an unmet
        route/availability obligation whose mandatory status is not represented at
        this planning boundary** — not a planned-member omission. `1c`'s converse
        cannot reach it, because `I = ∅` for `381`.

        **First task: inventory, do not build.** Identify whether an **existing
        upstream** semantic authority proves this exact edge **mandatory** and
        supplies an exact, **edge-local** environment derivation.

        - **It must distinguish `381` from the 22 same-state green peers BEFORE
          interning.** No such discriminator exists in the measured plane; if the
          inventory does not find one that already exists, it has not found one.
        - **Projection of that authority is permitted only if it already exists
          and is edge-local.** ⛔ Not derived, not generalized, not reconstructed.

        > ⛔⛔ **HARD STOP, and it is the point of this checkpoint. If satisfying
        > it would mint a NEW representation, population, identity, or planner/ABI
        > authority — STOP and return the concrete edge to the Steward.** That
        > outcome requires a **separate substrate node**, which this ruling
        > expressly does **not** authorize. **The graph shape is the Steward's
        > call, decided FROM the inventory** — deciding it before the inventory
        > exists would be creating a node on speculation, which this frame's own
        > constraint test forbids.

        **Bans carried from the ruling.** ⛔ Do not reject all `Open` real
        members · ⛔ do not retire the candidate-only law · ⛔ do not
        special-case closure `381` · ⛔ do not use corpus identity as a
        predicate · ⛔ do not
        weaken the generic `Closure` refusal · ⛔ do not declare the linked witness
        closed from the converse. **No environment repair is authorized from the
        present facts.**

        **Scope.** Existing `D7` / [[RT-RECURSOR-TRANSPORT]] atomic scope **for
        the inventory only**. It adds no third production predicate.

        **ANSWERED 2026-08-05 by `evt_5kws532ac99c9` — the inventory is
        NEGATIVE, and it is accepted.** 1110 candidate records (1057
        `ken-runtime --lib` + 60 `D0` parity), 612 declines, 489 interns, 169
        distinct declined edges across 34 tests; probe reverted, branch
        unchanged at exact `179af863`. Nine candidate authorities enumerated;
        none both proves the edge mandatory and supplies an exact edge-local
        closed environment. Three findings are load-bearing downstream:

        - `member=true` on **all 612 declines and all 489 interns** — `1c`'s
          finding, now measured over 1101 rows rather than 23 fixtures.
        - `case_emission=None` on all 1110: `build_case_emission_plan` iterates
          `RuntimeExpr::Match` only and never `ComputationalMatch`, so that
          authority is **inapplicable**, not merely insufficient.
        - The ring **declined a near-miss discriminator it could have reported**
          ("`Open` because of an effect result" — true for the witness family,
          false for every green peer). It is perfectly confounded with corpus:
          `Effect` occurs in 0 of 1057 lib ancestor chains and 60 of 60 parity
          chains, so the predicate separates two **test suites**. That is the
          corpus-identity ban in a different spelling, and catching it was
          correct.

     **1e. Mint the host-effect-result continuation input source.** Steward
        scope ruling 2026-08-05, decided FROM the `1d` inventory as `1d`
        required. **This authorizes exactly one representation and supersedes
        `1d`'s no-minting ban for that one only.**

        **Why this is folded into `D7` and NOT cut as a separate node.** `1d`
        said the outcome "requires a separate substrate node." That sentence was
        Steward prose, not a ruling — the Architect left the graph shape to the
        Steward, and this frame's own constraint test says to ask whether the
        constraint is real, not which node it becomes. It is not real: the
        preference order is relax, then fold, then cut; and `179af863` is not on
        `main`, so a separate node would branch from an unmerged branch, gaining
        an entry in the graph and no independent mergeability. **Folding costs
        nothing that cutting would buy.**

        > **FALSIFIED 2026-08-05 by `evt_5ngh190h9b1k5`. The scope ruling below
        > is DEAD and must not be executed — it bans the one slot every `D0` row
        > requires.** Read this block before the struck text under it.
        >
        > **Measured, by a full environment-vector probe cross-checked against
        > the production walk.** Every effect-bearing closure has **two**
        > required `Open` inputs: ordinal 0 is `Let value=Effect`, ordinal 1 is
        > a Match case binder. **The effect-only population is 0**, so closing
        > the effect result closes **zero** edges. The declined closure-edge
        > census is **34 case-binder-only, 4 effect-plus-case-binder, 1
        > `Construct`-only**; the 4 mixed parity edges span all six failing
        > `D0` rows. ⇒ **No `D0` row can move without the case-binder slot the
        > ruling banned.**
        >
        > **And the authorized variant has no lawful ABI seat.** The owner input
        > run admits only `Parameter`/`Capture`; a mid-body effect result is
        > neither. A position for it requires ABI/input-run widening, a new
        > mid-body position population, or an exemption from its only validator
        > — each exceeds `1e`.
        >
        > **Route modality is unaddressable, and not for the reason given
        > below.** The environment never becomes closed, so the hoped-for
        > closed-vs-`Open` discriminator does not materialize at all.
        >
        > ### The defect that produced the ruling, because it will recur
        >
        > **`1d`'s census recorded the DECLINING ordinal — the FIRST `Open` —
        > and the Steward read it as a REQUIREMENT census.** "6 edges are
        > effect-result" meant *6 edges decline first at an effect result*, not
        > *6 edges need only an effect-result slot*. A first-failure statistic
        > cannot support a minimality claim, because it is silent about every
        > input after the one that failed.
        >
        > **The `161` was also the wrong unit** — pair-level/first-`Open`, not
        > closure-edge. Closure-edge scope is 34 lib + 9 parity. The ruling
        > compared a 6 in one unit against a 161 in another and called the
        > result minimal.
        >
        > ⇒ **Before scoping from a census, state the unit and confirm it
        > answers "what does this edge REQUIRE", not "where did it first
        > stop."**

        **The scope, and it is minimal by measurement. FALSIFIED — see above; do
        not execute.** Mint the **host-effect-result** slot only:

        - a fourth `ContinuationInputSource` variant
          (`static_transition.rs:410`, today `Parameter` / `LexicalCapture` /
          `SeedCapture`), and
        - whatever assigns it an ABI position, since
          `validate_continuation_source_slot` resolves a slot by finding it
          among `continuation_owner_entry_sources`. **That second half is not
          optional** — the inventory says so explicitly.

        **The scrutinee-field / case-binder slot is OUT OF SCOPE.** The
        inventory's own counts decide it: the host-effect-result slot covers
        **6** distinct edges, which are exactly the 6 failing `D0` rows; the
        case-binder slot covers **161** distinct edges, all in a `ken-runtime`
        lib corpus standing at **718 passed / 2 failed**. No failing row demands
        it, and closing those edges would newly intern 161 edges on green
        programs. Minting it here would be a change with no measurement behind
        it. If a later row demands it, that is its own question.

        **First deliverable is an ANSWER, not a mechanism — the third time on
        this node, and the pattern has paid twice.** The `1d` hard stop named
        two mintings. The second — a route-modality authority expressing "this
        edge must have a route" — is **not authorized, and is probably not
        needed.** Its only source is `1d`'s own requirement to distinguish `381`
        from the peers *before interning*, and that was a constraint of the
        projection-only framing, which this checkpoint retires. Once the
        environment closes exactly, closed-vs-`Open` **is** the discriminator:
        the witness closes, the 22 peers stay `Open` because their ordinal 0 is
        a case binder, which is a different missing slot and stays missing.

        **Report whether that holds. Do not assume it because the Steward wrote
        it** — three of this node's stops were exactly that error. If a route
        modality is genuinely required, HARD STOP and return the concrete edge;
        that one is a real substrate question and the Steward will cut it a node.

        **ANSWERED, and it does not hold — `evt_5ngh190h9b1k5`.** The ring
        reported exactly as instructed rather than assuming. The prediction
        failed at its root: the environment never closes, so there is no
        closed-vs-`Open` discriminator to appear. Route modality is not
        "probably unnecessary"; it is **unaddressable from this grant**. That
        made this the fourth stop of the same shape, and the instruction to
        measure rather than comply is what caught it — keep writing it that way.

     **1f. `1e` IS WITHDRAWN. The linked row leaves `D7`.** Steward, 2026-08-05,
        on the ring's `evt_5ngh190h9b1k5` and the Architect gate
        `evt_75k8cydbj5127` (REJECT the one-slot representation, accept the
        hard stop).

        **Settled, and not reopened.** There is no lawful ABI seat for a
        mid-body value at `179af863`. The Architect enumerated the exits and
        closed all five: inventing an entry position, widening
        parameters/captures (which would falsely claim the value exists at
        function entry), reusing `AbiSlotKind::Result` (a different boundary
        direction), exempting the variant from
        `validate_continuation_source_slot` (its only exact validator), or
        using `immediate_slot` alone and discarding root provenance. The
        emission seam agrees independently: its exhaustive two-class resolution
        covers an entry value in its root owner and a value captured by a
        generated context, and **a producer-local value created mid-body is a
        third availability class with no authority here.**

        ⇒ **This is a representation and population boundary, not a missing
        enum arm.** It cannot close inside `D7` under any scope `D7` can grant
        itself, and a fourth `ContinuationInputSource` case would be the wrong
        shape even if it were authorized — the enclosing record still requires
        an entry-ABI coordinate that a producer-local value does not have.

        **Ruling 1 — BROAD admission.** The recut represents **every** exact
        producer-local value, and **all** newly representable candidates may
        lawfully intern. Not the four `D0` mixed edges alone.

        **Why, and it is the Architect's own conditional.** Interning only the
        four while leaving the 34 same-shaped case-binder candidates declined
        **requires a real edge-selection authority**, with corpus, closure
        identity, first-`Open` reason, and planned-member status all forbidden
        as substitutes. No such authority exists, and minting one to justify
        treating identical shapes differently is the manufactured discriminator
        this ring already caught once. Broad admission implies **no
        route-modality authority at all** — the question `1d` and `1e` both
        chased is dissolved rather than answered.

        **The consequence, stated plainly:** roughly 34 additional edges newly
        intern, changing emitted code on programs that are green today. That is
        correct — they were declined only because the representation could not
        name their environment — and the per-row `D0` and `718/2` baselines are
        the control that catches it if it is not.

        **Ruling 2 — the work is substrate and it gets its own node:**
        [[RT-CONTSRC-PRODUCER-LOCAL]]. Grounded in a measured capability gap and
        an Architect ruling, which is what `1d`'s node claim lacked. `D7` does
        **not** attempt it.

        **Ruling 3 — `D7`'s disposition.** Checkpoints 1, `1b` and `1c` stand as
        proved. The linked row's obligation, checkpoint 2, and every claim gated
        on the row move to [[RT-CONTSRC-PRODUCER-LOCAL]]. **The frame's standing
        clause that no candidate, QA route, `D6` closure or `AC-4` claim may
        proceed "while the row stands unreached" is RETIRED** — it was written
        when the row was believed reachable here, and it now forbids `D7` from
        ever landing anything. A constraint that can never be discharged by the
        node holding it is not a gate, it is a deadlock.

        **`1e`'s bans are void, not suspended.** They were derived from a
        falsified premise. The case-binder slot is required, and the successor
        node covers both it and the host-effect result as distinct structural
        bindings.

        **Architect gate.** This is a Steward scope ruling plus **one**
        confirming gate on the axis the fork turns on: the Architect confirms
        the **representation design** — the variant's shape and how it earns an
        ABI position. Scope is settled and is not reopened by that gate.

        **Bans.** All `1d` bans carry except the no-minting clause this
        checkpoint supersedes: do not reject all `Open` real members, retire the
        candidate-only law, special-case closure `381`, use corpus identity as a
        predicate, weaken the generic `Closure` refusal, or declare the witness
        closed from the converse. Additionally: do not mint the case-binder slot,
        do not mint a route-modality authority, and do not let the near-miss
        effect-result *provenance* become a production predicate — the slot is a
        representation, not a reason string.

        **Evidence.** Per-row, never a count. The 6 `D0` rows move by their own
        texts; `718/2` and `108/0` do not regress; `1c`'s member-population law
        and its four mutation controls stay intact and non-vacuous.

     2. **Then close the exact `264 -> 262 / position 1` consumer** with the
        retained `ae64f687` mechanism: the exact planned seat is claimed, the
        real row returns `InvalidBounds`, and shared-host dispatch count is
        **zero**. Removing the carried-capacity arm must recreate the refusal at
        that exact seat.

     All existing mixed-phase, owner/order/provenance, zero-allocation capsule,
     `D5a` marker, phase/taxonomy and `AC-4` controls remain **cumulative**
     across all five checkpoints.

     **The row may NOT be replaced by a synthetic or direct carried-capacity
     witness.** Such a witness proves the consumer locally and cannot prove that
     the real producer reaches it — which is the entire content of this item. So
     while the row stands unreached: no replacement reaching witness, no
     acceptance of the closure-capture refusal as standing branch state, and no
     candidate, QA route, `D6` closure, or `AC-4` claim.

     **This is a sequencing and evidence recut only.** It creates no node, lane,
     disposition, or atomic participant; atomic scope remains `D7` +
     [[RT-RECURSOR-TRANSPORT]]. **If restoring `#23` requires changing the `D5a`
     application-marker meaning, inventing a callable carrier, or widening
     planner/ABI beyond the already-framed capture contract — HARD STOP and
     return the concrete edge.**
  2. **Exact-`Int` phase pair.** Specialized and carried inputs produce
     **identical** narrowing outcomes for `-1`, `0`, `1`, `u64::MAX`,
     `u64::MAX + 1`, and a negative wide magnitude — covering **both** carried
     representations: `ImmediateInt` **and** sealed persistent exact `Int`.
  3. **Failure taxonomy.** Negative / out-of-range **valid `Int`s** take the
     semantic `valid == false` lane; wrong tag, wrong class, unsealed/invalid
     magnitude, wrong owner, and non-OK/non-range helper status **fail closed and
     are never converted into `InvalidBounds`**. No invalid-range case dispatches;
     a valid case dispatches **exactly once**.
  4. **Key sufficiency.** Two argument seats with the **same structural kind**
     but a different **operation, ordinal, or `Need`** remain **distinct
     records**. A mutation that **erases operation, erases ordinal, erases
     `Need`, or collapses all argument seats to one contract must reject before
     function definition.** (Corrected 2026-08-05: this read "two exact
     `EffectArgument` seats with the same nominal role" and keyed the mutation on
     role-only/disposition-only — vocabulary with no referent on the governing
     base. The property is unchanged; only the key is restated in terms that
     exist.)
  5. **Semantic-eliminator versus specialized-only discriminator — stated in the
     REAL current mechanisms, not retired enum labels.** A carried
     `BufferAllocate.capacity` operand **passes** through its exact emitted
     helper, while a carried operand at an **actually reached
     `specialized_operands_at` template-reading leaf** still **refuses**. A
     mutation routing the effect seat **back through the generic bulk specialized
     conversion must red the passing half.**

     **This is the non-vacuous equivalent of the impossible "restore the shared
     `boundary_contract` arm" mutation this item used to demand.** That arm does
     not exist on the governing base, so its restoration would have been a
     **no-op, and a no-op mutation is indistinguishable from a well-defended
     one** — it goes green either way and proves nothing.
  6. **Static population closure.** Enumerate every capability/argument seat for
     **all 13** admitted host operations, **independently of which runtime branch
     the parity row reaches**. Omit, duplicate, transplant, change operation,
     change ordinal/conditional base, or change `Need`: each **rejects at the
     planned-set / emitted-ledger gate before function definition**.

     > **Recut 2026-08-05, Architect `evt_6vr00htkk6cpf`: "or leave one token
     > unused" is WITHDRAWN, and so is global `planned = consumed`.** The
     > implementer's measurement is decisive — an un-emitted declaration lawfully
     > leaves planned seats unused. **`P` here is a closed AUTHORIZATION
     > population, not an execution obligation.**
     >
     > Global closure is **`image(claims) ⊆ P`**. An unused member of `P` is
     > **lawful and reported**, never a rejection. This is the same law the
     > aggregate lifecycle already carries in this frame — `dom(R) = E` with
     > `image(R) ⊆ P`, no surjectivity onto `P` — and the two must not drift
     > apart again.
     >
     > **SCOPE — read this before carrying the withdrawal anywhere else.** Both
     > withdrawals govern **the effect-seat population only**. Two other
     > populations in this frame still state a **bijection** with unused-token
     > rejection, and they are **untouched and still correct**: the
     > continuation/boundary-use `#27` population (`655 → 650`, the "bijection
     > mutations" control row) and the case-emission reachability authority's
     > planned-use/emitted-use bijection. Their `P` is an execution obligation;
     > this section's `P` is an authorization. **Do not propagate this
     > withdrawal into them, and do not read their bijection back into the
     > effect seats** — the populations differ in what `P` means, not in rigour.

  6a. **Per-visit claim-group lifecycle — the global relation is necessary but
      NOT sufficient.** Exact `6a09ed68` keys its ledger
      `(FuncId, effect_origin, slot)`, which **merges every repeated lowering of
      one static occurrence in that body**. The implementation measured such
      repeats. **Visit A can claim one subset and visit B the complementary
      subset, and their union passes per-body occurrence completeness while
      neither visit was complete** — and the key discards which phase each visit
      observed. Putting claims inside operation-specific arms makes that
      distinction load-bearing.

      Use a **local claim group per concrete compiler-side lowering visit of one
      effect occurrence**, on the existing local event/relation discipline:

      1. Open one **ledger-minted, lowering-unconstructible** claim-group identity
         **before any seat of that effect occurrence is observed**.
      2. Bind the group to the open function body, the exact effect origin, the
         operation, and its planner-issued seat population.
      3. Claim every planned seat **exactly once within that group**. Each claim
         records the exact planned record **plus the observed phase**, proves
         `observed_phase ∈ Avail`, and is consumed by that seat's exact
         operation-specific read.
      4. **Close the group before host request dispatch or any successful exit.**
         Its claimed slot set must equal the planned slot set for that occurrence.
         **No group may remain open at body or global close.**
      5. **Repeating a complete group for the same static occurrence is lawful.**
      6. Whole-pass close requires every opened group committed and every recorded
         claim related to `P`. It does **not** require every member of `P` to have
         a group.

      **Rejects:** duplicate use of one seat inside a group, a conflicting record
      at one key, a missing seat in any one group, an extra seat, a wrong
      operation/owner/ordinal/`Need`, an `observed_phase` outside `Avail`, and a
      discarded or still-open group.

  6b. **Lifecycle controls — the masking case is the one that decides this.**

      - **Complementary omissions across two repeated visits must reject BOTH
        incomplete groups**, even though their union equals the planned seat set.
        This is the discriminator the `(FuncId, effect_origin, slot)` union
        cannot express; without it the whole per-visit law is unmeasured.
      - An **un-emitted occurrence** passes as reported-unreached.
      - **Two identical complete visits** pass.
      - A **duplicate within one visit** rejects.
      - A **discarded or open group** rejects.
      - An **observed-phase mutation rejects at the exact seat** — this is what
        makes the retained phase evidence load-bearing rather than recorded.
  7. **Lowering closure.** Removing the carried capacity arm **recreates the
     refusal at the exact `264 → 262 / position 1` seat** — on the governing base
     that is the **generic host-effect specialized-only refusal**, not the
     historical identity-bearing diagnostic (corrected 2026-08-05). Restoring the
     bulk non-`BufferFreeze` `specialized_operands_at` gate **reds** the
     effect-seat controls. Existing specialized effect rows and `BufferFreeze`
     phase-bearing seats remain green.
  8. **No regression in the prior closure.** Every origin-271 source-machine
     `Match` control, every `#23` / `#26` mixed-phase and zero-allocation capsule
     control, the complete per-row first-refusal map, and the literal CI-green
     `AC-4` obligation **all remain owed**.

  ⇒ Exact `430798bf` is **preservation-only, not a candidate**, and is
  **superseded history — evidence for the semantic seat and the original refusal
  only.** Corrected 2026-08-05: this sentence read "and is the exact continuation
  base"; **the governing base is exact `70887529`** (GOVERNING BASE, above).
  ⛔ No QA verdict transfers; fresh SHA, fresh QA, and a
  **fresh** Architect Decision are mandatory.

  #### ⛔⛔ THE `FsAppendFile` CASE-EMISSION REACHABILITY AUTHORITY
  #### — Architect `evt_1x47ep8rnhk9p`

  Grounded on exact preservation-only `548682c3` (tree `31505888`, parent exact
  `430798bf`; 7 tracked Runtime files, `+1162/-242`). The delta **preserves** the
  13-operation catalog and **advances** the exact effect seat. Then, at the carried
  source-machine match, lowering **iterates every source case and lowers every
  body**, and the plan — which exports the full case-body list and each canonical
  `case_constructor_identity` — exports **no authority answering which constructor
  identities can reach this exact scrutinee**. So the `FsAppendFile` body
  (`family=decl:rt_parity_buffer_allocate_single::FSOp`, `0x0303` / 771) is emitted
  and the existing out-of-catalog gate **correctly** refuses it: `represented
  unavailable lane`.

  ⛔ **This is a MISSING CASE-EMISSION REACHABILITY AUTHORITY inside `D7`'s
  existing graph-derived `SemanticEliminator`.** It adds **no** producing
  predicate, node, carrier lane, seventh disposition, or atomic participant.
  Atomic scope remains `D7` + [[RT-RECURSOR-TRANSPORT]]; Seed and ProducerMatch do
  not fold; the buffer row is **still** only the reaching witness.

  ⭐⭐⭐ **BOTH CHEAP REACTIONS ARE UNLAWFUL, AND THE SECOND IS THE TRAP:**

  - ⛔ **Admitting `FsAppendFile`** widens `CRANELIFT_HOST_EFFECT_CONSUMERS_V1`
    beyond the ruled 13-operation surface.
  - ⛔ **Dropping a case because its body names an unavailable operation** turns a
    potentially **lawful runtime `FsAppendFile` value** into the match **default**.
    ⭐⭐ **THE CATALOG IS A CAPABILITY BOUNDARY, NOT A REACHABILITY ORACLE.** A test
    name, the branch this execution happened to take, and absence from the reached
    trace prove **nothing** about the static producer population.

  ⇒ **The observed case is NOT YET ENTITLED TO THE WORD *dead*.** It is eliminable
  **only** if the planner **proves** its canonical constructor identity is absent
  from the **closed set of producers that can reach this exact scrutinee**.

  ##### 1. Derive one closed producer-set fact per carried match scrutinee

  Computed **after** the generated-unit / specialization / continuation fixed point
  closes. Names may vary; the abstract result is exactly:

  ```text
  ScrutineeProducerSet = Open | Closed(Set<ConstructorIdentity>)
  ```

  The **key** includes the exact match origin, the exact scrutinee edge/origin, and
  the function owner **and phase**. `Construct` contributes its existing canonical
  `constructor_symbol_identity`; forwarding, environment/capture transfer, joins,
  calls, results, and recursor flow **propagate that same authority**; runtime
  `If` / `Match` alternatives **union** their producers; cycles close by a
  **monotone fixed point**.

  ⛔ **Any ABI ingress, opaque/untracked producer, unknown dynamic source, or
  missing flow edge yields `Open`, and `Open` DOMINATES union.** ⛔⛔ **Never derive
  `Closed` from a reached trace, an operation catalog, a test, an origin whitelist,
  a lowering-time search, or *"no producer was observed."***

  ##### 2. Partition every case ordinal before emission

  A planner-issued, **unforgeable** case-emission record binding at least: match +
  scrutinee origins and owner/phase · case ordinal + body origin + canonical case
  `ConstructorIdentity` · the closed producer-set authority **and its exact flow
  provenance** · `status = Reachable | Eliminated`.

  ⭐ **This is a case-body EMISSION PARTITION — ⛔ not a seventh operand
  disposition.**

  - **`Open` ⇒ NO case may be eliminated.** Every case remains potentially live; if
    a retained body contains an unsupported operation, **planning / object
    construction rejects**.
  - **`Closed(S)` ⇒ a case is `Reachable` iff its canonical identity ∈ `S`.** Emit
    that body and require every nested boundary-use / join token **normally**.
  - **A case outside `S` is `Eliminated`** — ⛔ do not lower its body or mint an
    effect-seat / object-emission use for it. Its **whole same-owner subtree** must
    be accounted for as eliminated in the plan, so the planned-use / emitted-use
    **bijection cannot report or hide anonymous unused tokens**.
  - **The runtime tag chain retains the closed default.** A corrupted or unlawful
    tag for an eliminated case therefore **traps / fails closed**; ⛔ it never
    executes an omitted body.

  ##### 3. Validate the partition BEFORE function definition

  Every source case has **exactly one** record; every record names a **real** case
  of that exact match; `Reachable` and `Eliminated` are **disjoint and cover the
  full case list**. Missing, duplicate, transplanted, wrong
  match/scrutinee/body/ordinal/identity/owner/phase, stale producer provenance, or
  an **`Open → Closed` mutation** ⇒ **rejects before function / object / carrier
  allocation**. ⭐ **The exact reachable-case ledger must EQUAL the emitted-case
  ledger.** Eliminated subtrees must have **no** unaccounted boundary use, join,
  effect seat, or helper call. ⛔ **Lowering CONSUMES this record; it does not
  re-derive reachability.**

  ##### 4. Keep the capability boundary literal

  `CRANELIFT_HOST_EFFECT_CONSUMERS_V1` remains **exactly 13** operations. ⛔ **If
  the closed set at origin 271 actually contains `FsAppendFile`, or if the set is
  `Open`, this repair MAY NOT PRUNE IT** — the exact program remains **unbuildable**
  at the existing represented-unavailable gate. **Return that producer edge as the
  next hard stop.** ⛔ Do not widen the catalog and do not convert the refusal into
  a runtime default. ⭐ **Only a proved `Closed` set excluding `FsAppendFile`
  authorizes omission.**

  ⭐⭐ **AND NOTE WHAT THIS IS:** the exact implementation of `D7`'s **existing**
  law — `Need ⊆ Avail`, **or planning eliminates the runtime edge completely before
  emission**. ⛔ It is **not** the reachability whitelist this frame forbids (see
  the disambiguation at §2's *"no reachability whitelist"* clause); it is a
  **closed, exact value-flow proof over the same planner graph**.

  ##### Required acceptance evidence — ⚠ QA reviews against these

  1. **Exact origin-271 proof.** Record the **complete `Closed` constructor set**
     and **every** producer origin / flow path reaching the exact scrutinee. The
     buffer row may advance **only if `FsAppendFile` is absent by that proof**; it
     then reaches `InvalidBounds` with **zero** `BufferAllocate` dispatches.
  2. **Multi-producer union.** One runtime conditional supplies **at least two
     distinct admitted constructors** to the same carried match; **both** case
     bodies are planned/emitted and either selects correctly. ⭐ This proves the
     producer walk **does not short-circuit on the first constructor**.
  3. **⭐ Actual unavailable-producer negative.** Add a **real** `FsAppendFile`
     constructor on a flow path to the same match. Its case becomes `Reachable` and
     the **unchanged** 13-op gate rejects before function definition / object /
     carrier allocation. ⛔ **A catalog-based filter would incorrectly GREEN this
     control** — it is the discriminator between the lawful proof and the banned
     shortcut.
  4. **Open-source negative.** Feed the match from an opaque / ABI / untracked
     carried source. The set is **`Open`**; **no** case is pruned, and the
     unsupported case **still rejects**.
  5. **Occurrence-key discriminator.** Two matches with **identical case
     spellings** but different exact producer sets receive **different**
     partitions. ⛔ Keying by family, case string, operation, or role **reds the
     pair**.
  6. **Bijection mutations.** Omit, duplicate, transplant, change
     match/scrutinee/body/ordinal/constructor identity/owner/phase/producer
     provenance, change `Open` to `Closed`, or leave a reachable record unused:
     each **rejects before function definition**.
  7. **Eliminated-subtree closure.** An eliminated case containing its **own**
     effect seat, boundary use, and planned join emits **none** of them but remains
     **fully accounted for** by the elimination record. Removing that record,
     omitting one nested member, or **falsely eliminating a truly reachable case**
     **reds before allocation / emission**.
  8. **Closed-default integrity.** Injecting an eliminated or unknown runtime tag
     takes the existing closed default / carrier failure and performs **zero**
     omitted-body effects; tag / arity / field mutations remain red.
  9. **No prior regression.** The specialized match route, the exact 13-operation
     census, the key-sufficiency and disposition discriminators, the origin-271
     continuation/join controls, the `#23`/`#26` mixed-phase and zero-allocation
     controls, the complete CI first-refusal map, and the **literal all-12 CI
     gate** all remain owed.

  ⇒ Exact `548682c3` is **preservation-only, not a candidate**, and is
  **superseded history, cited as evidence only.** Corrected 2026-08-05: this
  sentence read "and is the exact continuation base"; **the governing base is
  exact `70887529`** (GOVERNING BASE, above).
  ⛔ No verdict transfers; fresh SHA, fresh QA, and a **fresh**
  Architect Decision are mandatory.

  #### ⛔⛔ THE `MkProgramCaps` INVOCATION-AGGREGATE REPRESENTATION
  #### — Architect `evt_5ks9da0h0977w` (+ Research advisory `evt_1xtgtqhwyhhtd`)

  Grounded on exact preservation-only `42ccd8ec` (tree `8c16a9d9`, parent exact
  `548682c3`). ⭐ **The case-emission proof of the previous section is ACCEPTED at
  this seam** — origin 271 is `Closed` on canonical `DenseRange { start: 708,
  len: 53 }` through exact `Construct 385→385 / Forward 385→12 /
  Environment 12→270`, and `FsAppendFile` is **absent by that proof**. The next
  refusal is **independent of case reachability**: the row now clears planning and
  object emission, then traps `malformed borrowed process input` / `RuntimeTrap(1)`
  with a **zero effect trace** instead of returning `InvalidBounds`.

  ✅ **THE ESCAPE GUARD IS CORRECT AND STAYS.** The synthetic root adapter
  materializes `MkProgramCaps` as a **persistent** `Lowered::Constructor`, then
  stores an **invocation-lifetime** `BorrowedOpaque` `CapabilityToken` child, and
  `boundary_value_clif::store_field` rejects persistent-parent / invocation-child
  with `BOUNDARY_ERR_ESCAPE` **before host dispatch**. ⛔ `BOUNDARY_ERR_ESCAPE`
  must remain. The rejection is **evidence the representation's two promises
  conflict**, ⛔ not evidence the guard is wrong.

  ⭐⭐⭐ **THE REAL DEFECT: `B2V`'S VARIANT-ONLY REPRESENTATION AUTHORITY CONFLATES
  AGGREGATE SHAPE WITH REFERENT LIFETIME.** The current authority

  ```text
  LoweredVariant::Constructor -> PersistentGround / Constructor
  ```

  is **under-keyed**: `Constructor` answers the **shape** question and proves
  **nothing** about the **referent owner**. ⇒ This exact `MkProgramCaps` occurrence
  **may** cross the unit ABI, be matched, and project its child **during** the
  invocation; it **may not** be published into the persistent store or survive the
  invocation. ⛔ **Copying the 64-bit token, giving the outer value a persistent
  tag, and separately compiling the callee each perform NO ownership conversion**
  — a generated-unit frame slot is **transport, not persistent ownership**.

  ⚠⚠ **AND THIS RULING NARROWLY SUPERSEDES A PRIOR BAN — the only supersession so
  far.** The blanket *"no new carrier lane"* clause **is** narrowed here: this
  repair **does** add **one** explicit admitted tag×class representation row
  family. ⛔ **Hiding that fact is forbidden** — no reusing `PersistentGround`, no
  disguising an aggregate as `InvocationBorrowed / BorrowedOpaque`, and no calling
  a singleton rewrite *"not a lane."* ⭐ [[B2V]] is **not** reopened as a separate
  WP; its closed representation authority is corrected **inside this same atomic
  acceptance surface**. Atomic scope remains `D7` + [[RT-RECURSOR-TRANSPORT]].

  ##### 1. Add ONE explicit invocation-aggregate tag

  Names may vary; its **meaning** may not. Its payload indexes the **existing**
  invocation arena; `referent_owner()` is `InvocationArena`; it is **distinct**
  from borrowed-opaque ingress **and** from `InvocationHostResult`. Admitted node
  classes are **exactly**:

  ```text
  InvocationAggregate × { Constructor, Record }
  ```

  ⛔ **No** new arena, word width, scalar encoding, constructor identity, class,
  host ABI, or durable lane. `HostResult` keeps its existing invocation lane;
  `Closure` remains **forbidden**; Int/Bytes/String and opaque tokens keep their
  representations. ⛔ The numeric tag and **every** ABI/plan identity covering the
  closed tag set must update **through the existing authority**, ⛔ never an
  untracked literal.

  ##### 2. Replace variant-only ownership with a planner-issued occurrence record

  After the exact boundary producer/flow graph closes and **before function
  definition**, derive for **each** `Constructor` or `Record` occurrence: aggregate
  origin + owner/phase · class + constructor/field identity and arity · exact child
  occurrence/position records · **possible referent-owner set for every child** ·
  selected aggregate owner/tag/class.

  ⭐⭐ **THE OWNER LAW IS THE LIFETIME MEET:**

  - select `PersistentGround` **only** when **every** possible transitive child is
    immediate/no-referent or persistent;
  - select `InvocationAggregate` when **any** possible child is invocation-owned;
  - a join or dynamic alternative that **may** produce either durable or
    invocation-owned children selects **invocation ownership conservatively**;
  - an unrepresented / protocol-only / forbidden child **rejects before emission**.

  Empty and all-durable aggregates remain persistent. ⛔ **Constructor spelling,
  source type, test name, root-adapter location, one reached branch, and the
  presence of `ProgramCaps` are NEVER ownership authority.** ⛔ Lowering **consumes**
  the exact record; it does **not** recursively rediscover lifetime while emitting
  fields.

  ##### 3. Preserve transitive containment at construction and escape

  The invocation aggregate may contain immediate, persistent, or same-invocation
  children, and its fields must be validated through the **same** arena/store
  relation before sealing. ⛔ A persistent parent containing **any**
  invocation-owned child — **including an invocation aggregate nested at depth two
  or greater** — still returns `BOUNDARY_ERR_ESCAPE`. ⛔ An invocation aggregate may
  **never** be adopted, interned, or relabelled into the persistent store. ⭐ The
  existing tag-owner escape check remains **defense in depth** even though a valid
  plan must never reach it with a persistent parent / invocation child.

  ##### 4. Keep the capability opaque and invocation-scoped

  `CapabilityToken` remains full-width `InvocationBorrowed / BorrowedOpaque`. ⛔ No
  immediate form, persistent copy, scalar constructor field, new minting path, or
  callee host-context reload. Its **only** legal path is: ingress → declared root
  slot → invocation-owned `MkProgramCaps` field → ordinary carried projection →
  host-effect consumer, **all under the originating invocation's activation
  services**.

  ##### 5. Keep ordinary constructor semantics

  The invocation-owned constructor carries the **same** canonical constructor
  identity, arity, positional fields, class query, carried-`Match` identity chain,
  field projection, and closed default as its persistent twin. Consumers choose
  from the **emitted word's tag/class relation**, ⛔ never from the constructor
  name. ⛔ **No `ProgramCaps` flattening, singleton-constructor elimination,
  lifted-capability special ABI, or unit inlining.** ⭐ Those shortcuts erase the
  source constructor at one synthetic site, bypass identity/arity/default controls,
  and **leave the same Record/nesting matrix hole open**.

  ⭐ **The Research advisory's discriminator is ADOPTED:** a **synchronous
  cross-unit loan is lawful**; persistence, surviving closure/continuation capture,
  async handoff, and cross-invocation retention are **not**. Trap / cancellation
  cleanup ends the same invocation extent and must leave **no** retained aggregate.

  ##### Required acceptance evidence — ⚠ QA reviews against these

  1. **Exact reaching witness.** The root adapter produces canonical
     `MkProgramCaps` as `InvocationAggregate / Constructor`, transfers it through
     the declared root parameter, the carried `Match` selects the exact identity and
     projects the **unchanged full-width** capability, and the buffer row reaches
     `InvalidBounds` with **zero** `BufferAllocate` host dispatches.
  2. **⭐ Same-nominal-shape owner pair.** Two occurrences of the **same**
     constructor identity/arity differing **only** in child lifetime: all-durable
     children ⇒ `PersistentGround / Constructor`; one invocation child ⇒
     `InvocationAggregate / Constructor`. ⛔ Keying by variant, type, constructor
     name, root adapter, or an unconditional shortest/longest lifetime **reds the
     pair**.
  3. **Closed aggregate matrix.** Repeat the owner pair for `Record`; prove the new
     tag admits **exactly** `Constructor` and `Record`. Every other tag×class
     mutation, unknown tag/class, wrong recorded owner, and untracked
     tag-set/ABI-identity mutation **rejects**. Rust builders and emitted helpers
     must derive from and reconcile to **one** representation authority over the
     **full finite product**.
  4. **Transitive and alternative closure.** Put an invocation child below **at
     least two** mixed `Constructor`/`Record` levels; **every** ancestor is
     invocation-owned. An all-persistent twin stays persistent. A runtime
     alternative joining persistent and invocation children selects **invocation**
     ownership and **both** arms project correctly. ⛔ A direct-child-only walk, a
     first-arm rule, or a lowering-time search **reds**.
  5. **⭐ Synchronous-loan versus escape discriminator.** Passing the invocation
     aggregate through a separately emitted parameter/capture/result slot and
     consuming it **before** the outer invocation ends **succeeds**. Attempting
     persistent adoption/publication, returning it across the native invocation
     boundary, storing it in a surviving closure/continuation, or handing it to
     asynchronous work **rejects before persistent identity/publication or host
     dispatch**. ⛔ A frame-slot write alone must **not** be misclassified as
     persistence.
  6. **Originating-invocation closure.** There is **no** legal raw `BoundaryWord`
     bridge between invocation arenas. Destroy invocation A, then attempt **every**
     exposed replay route for A's wrapper/token under invocation B — including equal
     payload/index pressure: **none** may authorize B or resolve through B's
     semantic slots. Exceptional exit likewise leaves **no** retained wrapper and
     performs **no** post-invalidation dispatch.
  7. **Construction ordering and defense.** Missing/duplicate/transplanted child
     record, wrong origin/position/identity/arity/owner/phase, stale owner
     provenance, or a changed selected tag **rejects before function definition
     or carrier allocation**. ⛔ **CORRECTED 2026-08-04 by `evt_39b1dzgc85gyf`:
     this list also required "an unused aggregate record" to reject. It is
     falsified — the measured artifact carries 1 to 132 lawfully unused records,
     and at least one unused planned record must now let closeout SUCCEED.**
     Independently mutating a
     persistent parent to receive an invocation child must still reach exact
     `BOUNDARY_ERR_ESCAPE`; ⛔ weakening or removing that guard **reds**.
  8. **Representation-semantic parity.** Persistent and invocation-owned ordinary
     constructors use the **same** identity/arity/field/default consumer logic.
     Forcing either through `BorrowedOpaque`, `HostResult`, or the other's
     owner-specific allocator **reds at its named assertion**; ⛔ no compile-time
     template recovery and no constructor-name dispatch passes.
  9. **Cumulative closure.** All origin-271 producer-set / case-partition controls,
     the 13-operation seat census, the key and disposition discriminators, the
     `#23`/`#26` mixed-phase and zero-allocation controls, the complete per-CI-row
     first-refusal map, the **`D8` retained-target obligation**, and the **literal
     all-twelve CI gate** remain owed.

  ⇒ Exact `42ccd8ec` is **preservation-only, not a candidate**, and is
  **superseded history, cited as evidence only.** Corrected 2026-08-05: this
  sentence read "and is the exact continuation base", which also contradicted the
  paragraph directly below calling its preserved ref a recovery point rather than
  a resume instruction. **The governing base is exact `70887529`** (GOVERNING
  BASE, above).
  ⛔ No QA verdict **or prior Decision** transfers; fresh SHA,
  fresh QA, and a **fresh** Architect Decision are mandatory.

  ##### ⭐ THE BASE IS NOW DURABLE AT `origin` — and it was NOT until 2026-07-30

  ⚠ Measured 2026-07-30: `42ccd8ec` — the live base this whole atomic candidate
  is being built on — existed on **exactly one local ref**
  (`refs/heads/wp/RT-DECL-CLOSURE-PORT`, the seat's own working branch) with
  **zero refs at `origin`**. ⛔ A handoff-gate hard reset or a worktree reseat
  would have destroyed the continuation base of both nodes at once.

  ⇒ Now recoverable from **`origin/preserved/rt-decl-d7-base-42ccd8ec`**.

  ⛔ **This ref is a recovery point, NOT a resume instruction.** It is
  preservation-only exactly as the paragraph above says; fetching it changes
  nothing about the fresh-SHA / fresh-QA / fresh-Decision requirement.

  ⚠ **The earlier preservation seams in this lineage — `548682c3`, `430798bf` —
  are deliberately NOT pushed.** They are superseded history, and this frame
  cites them as evidence of *what was ruled when*, not as places to resume from.
  ⭐ **A cited SHA needs a durable ref only when an artifact tells someone to
  resume from it**; treating every citation as a preservation obligation would
  put ~200 dead refs on `origin` and bury the handful that matter.

  #### The CI regression population this must clear

  Measured on PR **#1251** at exact `4dc120c5`: **8 of 12 checks red**, where
  `main` at `e79f7af6` (run `30497524438`) is **12 of 12 green** — so every one
  is this lineage's regression, ⛔ none inherited.

  | failing check | first-refusal evidence |
  |---|---|
  | `native-slow (rt_parity_native)` | `StaticRecursorWorker: a static recursor environment capture is not an ordinary carried operand` (`rt-parity-allocate`); `Closure: a closure cannot cross the boundary…` (`rt-parity-read-norights`) |
  | `native-slow (px8f_buffer_native)` | `px8f_buffer_native.rs` — owed in the map |
  | `native-slow (px8f_write_partition)` | owed in the map |
  | `test shard 1/4` … `4/4` | `px4b_native_production` (4 rows), `px7f_resource_native`, `px7l_checked_host_recursive_bind`, `px7m_hostresult_computational_match` |
  | `build + test` | aggregate of the above |

  ⚠ **`AC-4` remains literal: workspace green IN CI.** ⛔ The replacement is
  **not a candidate** until **all twelve** publisher checks are green. Fresh SHA,
  fresh QA, fresh Architect Decision — `dec_2z9v81tc5jt6x` **transfers nothing**.

  ⭐⭐ **And note where this class of failure is visible from.** Every failing row
  is a `crates/ken-cli` **linked-native** test. ⛔ No amount of targeted
  `-p ken-runtime` evidence can see it — per `agent/COORDINATION.md` §12 that is
  CI's job by design. ⇒ **An `AC-1b`- or `AC-4`-style "the objects still build"
  criterion has NO local discharge, and this frame does not claim one.**

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

  **Amended 2026-08-04 (Architect `evt_3twrm71vck49d`).** "Fails closed" is not
  discharged by a validator no accepted input can reach. Measured at
  `5e61d640`: the vestigial `SchedulingEntry` refuses every closure-seed
  declaration before emission, so `D5`'s positive control asserts a blocker
  rather than the property. ⇒ **`D2a` lands first, then `D5`'s transition
  sentinel is promoted into the real checked positive, and only then `D6`.** A
  `D5` whose positive is still a sentinel does not satisfy this AC.

  **Extended 2026-08-04 (Architect `evt_44k5h9z49nf9b`).** `D5a` sits between
  `D5` and `D6` for the same reason one step further out: with the planned
  closure-valued constructor field still crossing a unit boundary, `D6`'s
  retirement reddens a landed end-to-end fixture, so activation would again
  precede a runnable proof. ⇒ The full order is
  **`D2a` → `D5` → `D5a` → `D6` → `D6a`**.

  **Corrected 2026-08-04 (Architect `evt_4m2qk2fehm6vg`).** This clause
  previously ended "and `D6` is retried unchanged once `D5a` is green". `D6`
  ran unchanged at `1e5daa7b` and its own action succeeded; the first execution
  of the functionized artifact then trapped, on an existing carried-consumer
  edge that only activation could expose. So the order does not end at `D6`,
  and **this AC is not satisfied by a green `D6`** — it requires `D6a`'s
  linked-exit evidence. Reading the retired clause as authority to fold the
  `D6a` repair into `D6`, or to refuse it as out of scope, are the two wrong
  readings it produced.
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

> ### ⛔ §5a — ARMED AND CURRENT AS OF 2026-07-30. Re-read this line at every stop.
>
> **Both triggers are ANCHORED TO A NAMED EVENT, not to an index.**
>
> | trigger | next due |
> |---|---|
> | research pull | the **3rd hard stop after `evt_5ks9da0h0977w`** |
> | predicate check | the **3rd hard stop after `evt_5ks9da0h0977w`** |
>
> ⭐ **Count forward from that ruling. ⛔ Do NOT evaluate either trigger against
> an entry index or a matrix count.** Anchoring on an index made both triggers
> unevaluable without a correct history — and the history was wrong twice (see the
> two ⚠ notes below). An event anchor is checkable from the channel alone.
>
> ✅⭐ **BOTH FIRED AND WERE DISCHARGED AT THE `42ccd8ec` STOP (2026-07-30).**
> Leader's classification request `evt_1qz2eh0vxvwkp`; research pull
> `evt_4xpktd1g5t8m3` → advisory `evt_1xtgtqhwyhhtd`; Architect ruling
> `evt_5ks9da0h0977w`, which cites the advisory explicitly. ⭐ **The predicate check
> returned a REPRESENTATION-level answer, which is the outcome `steward.md §5a-ii`
> exists to produce:** the shared predicate is that
> `LoweredVariant::Constructor -> PersistentGround / Constructor` is **under-keyed**
> — it answers aggregate **shape** and never proves **referent owner** — so the
> closure is one explicit `InvocationAggregate` representation row family over the
> closed `{Constructor, Record}` matrix, ⛔ not another enumerated disposition.
>
> ⚠ **THE TWO NUMBERS BELOW MEASURE DIFFERENT POPULATIONS AND MUST NEVER BE READ
> AS ONE CADENCE.** `count of record` tallies **Architect-admitted cells**;
> the research/predicate cadence counts **hard stops**. A stop can occur with no
> admission (and did, at `#27`), so the two advance at different rates. ⛔ The
> superseded line stated them adjacently as `count 26 · next pull #30`, which reads
> as one series and computes the trigger against the wrong population — measured
> 2026-07-30: the ring pulled research correctly by counting its own hard stops
> while that line would have answered *"not due."*
>
> **Count of record: 26** (admitted cells) · **entries: 18 + the four 2026-07-30
> stops, never appended.** ⚠ Whether the `evt_5ks9da0h0977w` admission moves the
> count is **NOT self-evident** — it admits a **B2V representation row family**,
> while the count tallies **ruled dispositions in the codomain**. ⇒ It stays `26`
> pending an Architect statement; ⛔ moving it would be exactly the
> inflation-by-fiat the block below forbids.
>
> ### 📜 HISTORY BELOW THIS LINE — ⛔ no cadence stated past here is live.
>
> **`#27` / the 18th entry (2026-07-29).** Both triggers fired together in one pass
> (`evt_3tx7ndxp5pm4j`) before the Architect ruled — the foreseen simultaneous-due
> condition, discharged rather than deferred. ⛔ **Its re-stagger (pull at `#30`,
> predicate at the 21st entry) is SUPERSEDED** by the event anchor above: four
> further stops landed on 2026-07-30 without either index being advanced, so the
> re-stagger was already unevaluable when the next stop arrived. ⭐ **That is the
> second failure of an index-shaped anchor in this block, which is why the live
> trigger is now keyed to a ruling event instead.**
>
> ⚠ **Count of record stays `26`.** Stop `#27` reached an edge with **no** lawful
> member, and no member has been ruled for it — ⛔ the count moves when the
> **Architect admits** a cell, never when a stop **finds** one missing. Those are
> different events and conflating them would inflate the matrix by fiat.
>
> ⛔⛔ **AND AS OF `evt_4p9ne0vcds5hb` THE COUNT MEASURES NOTHING ABOUT CLOSURE.**
> The population authority is **withdrawn** (see the `D7` deliverable's withdrawal
> block). ⇒ `26` is now a **tally of ruled dispositions in the codomain**, ⛔ not a
> statement that the domain is covered. ⚠ Do not cite it, or its growth, as
> evidence the matrix is closing — that reading is exactly what let four
> individually-correct additions look like convergence.
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

**The FIVE stops on this node's chain where a matrix cell was MISSING, for the
predicate check:**

| # | entry | mechanism that failed |
|---|---|---|
| 22 | 13 | producer-`Match` carried scrutinee — tree-producing scrutinee not `Bool`/constructor |
| 23 | 14 | carried closure-capture — `Carried` reaching a consumer built only for specialized shapes |
| 24 | 15 | **callable as transparent-declaration `CallArgument`** — `StaticOriginId(1031)`, a `LexicalClosure` with no lawful disposition among the ruled five |
| 26 | 17 | **static-recursor-worker residual** — `StaticOriginId(723)`, a `Closure` in the recursor split's `Captures[Carried x7]`, with no lawful **member** among the ruled matrix's cells |
| **27** | **18** | **constructor-operand closure on a carried computational-match path** — parent `StaticOriginId(655)` / child `650` / body `641`, arity `1`, captures **`8`**, refused inside `transfer_constructor_operands` with **no planner-proved token at all** (`evt_4tvysmzr6mfpb`) |
| `42ccd8ec` | 2026-07-30 | ⭐ **persistent-parent / invocation-child escape** — the synthetic root adapter materializes `MkProgramCaps` as a persistent `Lowered::Constructor`, then stores an invocation-lifetime `BorrowedOpaque CapabilityToken`; `store_field` raises `BOUNDARY_ERR_ESCAPE` **correctly** (`evt_1qz2eh0vxvwkp` → ruling `evt_5ks9da0h0977w`) |

⚠ **The last row lands in a DIFFERENT matrix from the five above it, and that is
the finding, not a bookkeeping detail.** Rows 22–27 are missing cells in the
**disposition** matrix — each asks *which ruled disposition covers this edge?*
The `42ccd8ec` row is a missing row in the **B2V representation** matrix: every
disposition was present and lawful, and the authority
`LoweredVariant::Constructor -> PersistentGround / Constructor` still could not
say who **owns the referent**. ⇒ ⭐ **This is why the predicate check produced a
structural closure rather than a sixth enumerated disposition** — the enumeration
was never short a cell; the representation was short a key.

⭐ #22 and #23 were already ruled one shape, and folding them is what produced
`D7` — which is what **found** #24. ⛔ No repair of the two known cells would
have surfaced it. **The reached set is evidence, never the population.**

> ✅⚠⚠ **DISCHARGED AT `#27` — and the pattern is now FIVE. The prediction being
> borne out is itself the evidence.**
>
> ⛔ **This block formerly read "CARRY THIS INTO THE `#27` PREDICATE CHECK" and
> described a four-instance pattern. That instruction is SPENT and that count is
> RETIRED, not qualified** — the check was asked at `#27` in `evt_3tx7ndxp5pm4j`.
>
> #22, #23, #24, #26 **and now #27** are all the **same shape of failure**: a
> matrix or partition asserted closed, met a **real** edge it had no lawful cell
> for, and refused **late** rather than at planning. The first four were each
> individually ruled a legitimate new cell/member — and that is exactly what makes
> the pattern easy to keep absorbing one ruling at a time.
>
> ⭐⭐ **What `#27` adds is not a fifth data point but a CONFIRMED PREDICTION.**
> This block was written before `#27` existed and said the next stop would test
> whether the derivation is wrong rather than incomplete. The next stop then
> arrived, in the predicted shape, on the predicted chain. ⇒ **"Incomplete on one
> more occasion" is now the hypothesis with a failed forecast against it**, and
> the one asserting exhaustiveness owes the stronger account.
>
> ⭐ **The question asked (Architect, `evt_3tx7ndxp5pm4j`): not "what is the next
> cell?" but "why does the closure keep failing to be closed?"** ⛔ An Architect
> question about the derivation — not a Steward re-cut, not a ring's to answer, and
> ⛔ **not** grounds to add a node (operator node gate: interrogate the constraint,
> do not presume a node).
>
> ### ⭐⭐ THE DISCRIMINATOR, WRITTEN BEFORE THE ANSWER — so a fifth absorption
> ### cannot pass as a resolution
>
> ⛔ **A ruling that admits a cell for `#27` and stops there does NOT answer the
> question asked**, and this line exists so that is visible rather than arguable.
> The two hypotheses differ observably:
>
> | | **merely INCOMPLETE** | **the DERIVATION is wrong** |
> |---|---|---|
> | where the new cells fall | scattered, unrelated operand kinds | all on one axis the derivation never split on |
> | what the next stop looks like | unpredictable | predictable from the axis, in advance |
> | the fix | add the cell | re-derive the partition on that axis |
> | the refusal site | varies with the operand | keeps landing **late**, at lowering, never at planning |
>
> ⚠ **Three of the four columns already read to the right on all five instances**,
> and the "refused late rather than at planning" property is common to every one of
> them. ⛔ That is not a ruling — the axis and its soundness are the Architect's —
> but it means the burden has moved, and a cell-only answer should have to say why
> the shared late-refusal property is a coincidence.
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
