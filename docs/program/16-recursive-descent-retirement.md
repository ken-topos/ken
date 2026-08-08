# Recursive-descent retirement — the B2F migration, finished

**Operator directive, 2026-07-29:** *"Prioritize replacement of RecursiveDescent.
Create WPs to migrate the remaining residual classes and schedule them. Again,
this is a crucial efficiency issue we should close and we should not let it
linger in a half-migrated state. That just carries tech debt for no benefit."*

⭐ **This campaign closes a migration the code itself calls temporary.**
`select_body_emission_authority` is documented as *"The one **temporary** B2F
migration selector"* (`lowering/core.rs:174`). It has been temporary long enough
to grow a per-function code-size wall on the lane it was supposed to be
retiring.

> ### ⭐⭐ INDEPENDENTLY CORROBORATED — the Architect reached the same conclusion from a different direction
>
> The §5a **12th-entry predicate check** (`evt_6vw2j1c5sqzka`, 2026-07-29)
> partitioned twelve hard stops and found **no single shared predicate** — but it
> named, as one of five subfamilies covering **entries 4–6 and the port aspect of
> entry 11**:
>
> > *"executable-boundary closure is incomplete. A static identity or semantic
> > seat exists, but the `FunctionizedUnits` selector, carrier, consumer, join, or
> > callable-declaration port cannot transport it through complete emission."*
>
> ⇒ **That is this campaign's subject, arrived at by partitioning the failure
> history rather than from the directive.** The campaign's grounding is therefore
> two independent sources — the operator's directive and the Architect's
> partition — not one. ⛔ It does **not** authorize a representation recut; the
> predicate answer was `independent/mixed` and every proved subfamily keeps its
> own routed repair.

---

## 1. What the selector actually does

```rust
if recursive_descent_residual(expr)
    .or_else(|| declarations.values().find_map(declaration_recursive_descent_residual))
    .is_some()
{ BodyEmissionAuthority::RecursiveDescent } else { BodyEmissionAuthority::FunctionizedUnits }
```

**Whole-program and all-or-nothing.** *Any* retained residual anywhere in an
object routes the **entire object** to the monolithic `RecursiveDescent` root,
where declaration bodies are recursively lowered *into* one generated function
rather than reached as separately owned callable units.

⇒ That root is what exceeds Cranelift's per-function ceiling
(`Compilation error: Code for function is too large`), and it is the efficiency
cost this campaign exists to remove.

## 2. The five residual classes and who owns each

| class (`core.rs:41-57`) | what it is | node |
|---|---|---|
| `TransparentDeclarationClosure` | a transparent declaration whose body is a closure seed | [`RT-DECL-CLOSURE-PORT`](issues/RT-DECL-CLOSURE-PORT.md) |
| `SeedClosureCall` | a `Call` whose **callee** is the retained non-lexical closure form | [`RT-SEED-CALL-PORT`](issues/RT-SEED-CALL-PORT.md) |
| `ProducerMatchCall` | an ordinary producer `Match` whose scrutinee is directly a `Call` | [`RT-PRODUCER-MATCH-PORT`](issues/RT-PRODUCER-MATCH-PORT.md) |
| `MatchScrutineeRecursor` | an ordinary `Match` consuming an **active computational recursor** | [`RT-RECURSOR-TRANSPORT`](issues/RT-RECURSOR-TRANSPORT.md) |
| `LexicalCallArgumentRecursor` | a lexical unit call whose **argument** is an active computational recursor | ← *same node* |
| — | delete the selector, the enum, the authority, and the lane | [`RT-DESCENT-RETIRE`](issues/RT-DESCENT-RETIRE.md) |

### ⭐ Why two classes share one node, and one does not

**`MatchScrutineeRecursor` and `LexicalCallArgumentRecursor` are one mechanism in
two syntactic positions.** The code says so itself, in
`LexicalCallArgumentRecursor`'s own doc comment:

> *"The recursive result still carries invocation-local scope/return-hole state.
> Passing it through a separately declared lexical unit is not one of the
> completed functionized ports."*

Both classes fire on an **active computational recursor** — a
`ComputationalMatch` with a case carrying non-empty `recursive_positions` — whose
result carries invocation-local scope/return-hole state across a boundary. One
occupies a match scrutinee, the other a lexical call argument. ⇒ **Retiring one
without the other would build the transport twice.** They are folded, per
`docs/PRINCIPLES.md` *subsume-don't-proliferate*.

⚠ **`SeedClosureCall` is deliberately NOT folded into
[`RT-DECL-CLOSURE-PORT`](issues/RT-DECL-CLOSURE-PORT.md), even though it is
close.** Both concern closure seeds becoming callable units, and
`RT-DECL-CLOSURE-PORT`'s `D2`/`D3`/`D4` build exactly that machinery — so
`SeedClosureCall` may turn out to be **largely or wholly subsumed**. ⭐ That is a
*prediction, not a measurement*, and folding on an unmeasured prediction is the
error that held a ring for a day on 2026-07-28. `RT-SEED-CALL-PORT` therefore
exists as its own node whose **`D1` may legitimately return "already retired"**,
at which point it closes for free. ⛔ A node that closes cheaply on evidence is
correct; a fold that was wrong is expensive.

## 3. ⛔⛔ THE CAMPAIGN'S BINDING TRAPS — EVERY ONE BINDS EVERY NODE

The first two follow from one fact: **the selector short-circuits at the first
residual it finds**, consulting the expression walk before the declaration walk.

### Trap 1 — you cannot measure your own class while an earlier class fires

`recursive_descent_residual` returns `Option<_>` and every combinator is
`.or_else(...)`. So a program that fires *your* class and also an earlier one
reports only the earlier one. ⇒ ⛔ **You cannot enumerate your class's real
population by reading what the selector reports.**

⭐ **Consequence, and it is a hard requirement:** every node's `D1` must
enumerate **every** residual firing on the measured programs, not the
short-circuited first. This is the same `D1` obligation
`RT-DECL-CLOSURE-PORT` already carries; the enumeration should be built **once**
by whichever node runs first and reused, not rebuilt per node.

**IT HAS BEEN BUILT. REUSE IT — do not build a second one.** Measured at
`origin/main = 606efa93` (2026-08-08), landed by `RT-SRCBODY-BIND-ORDER`
(`7ca5cfc0`):

| what | where |
|---|---|
| entry point, over expression plus declaration map | `lowering/core.rs:598` `enumerate_recursive_descent_residuals` |
| non-short-circuiting walk, `BTreeSet`, no wildcard arm | `core.rs:616` `collect_recursive_descent_residuals` |
| exact-set control, `assert_eq!` over four variants | `core/tests/control.rs:10849` `d1_the_enumerator_reports_every_variant_not_the_first` |

**Two standing obligations survive its existence, and they are what each `D1`
still owes.** First, **run the control by name on your tree** — later
deliverables rewrite `core.rs` underneath it, so presence is not greenness.
Second, **an instrument that can see a class is not a population**: the
committed witnesses are hand-built `RuntimeExpr` values in tests, not Ken
programs, so they prove the walk reports the class and say nothing about
whether any real program fires it.

**`RT-SEED-CALL-PORT` asserted this instrument did not exist until 2026-08-08**,
having inherited that fact from `RT-DECL-CLOSURE-PORT` — where it was true — and
never re-derived it after a later node landed one. **Re-derive a claim about the
tree against current `main` at each use.**

### Trap 2 — ⭐ the later nodes are riskier than they look

As classes retire, program shapes that **never reached `FunctionizedUnits`
before** begin to. Those shapes have never been emitted on that lane, never been
scale-measured on it, and never exercised its invariants.

⚠ **This is not hypothetical — it already happened once.** Hard stop #21
(`NATIVE-HANDLE-CARRIER`, 2026-07-29) was the *first program shape* to violate a
fail-closed join-accounting invariant that `RT-FNSPLIT-RECUR-PORT` had landed
green, producing [`RT-JOIN-DISPOSITION`](issues/RT-JOIN-DISPOSITION.md). ⇒
**Expect one such stop per class retired, and do not price the later nodes as if
the earlier ones de-risked them.** They enlarge the exposed population instead.

⛔ **Do not treat a hard stop in this campaign as a defect in the node that
found it.** It is the fail-closed machinery doing its job on a newly reachable
population. Route it; do not work around it.

#### ⭐⭐ AND THE OBVIOUS `AC-1` CANNOT SEE IT

⛔ **Every node's `AC-1` quantifies over the programs that FIRE its class. This
trap's population is the COMPLEMENT of that set.** The rows that break never
fired your class at all — they were already green, and your port newly routes
them onto `FunctionizedUnits`. ⇒ An `AC-1` of the form *"every program `D1`
named as firing this class compiles and passes"* is **structurally blind to the
hazard described above**, and reads green while the port regresses `main`.

**Measured 2026-07-29 on [`RT-DECL-CLOSURE-PORT`](wp/RT-DECL-CLOSURE-PORT.md).**
It discharged its selector gate on both governed deltas
(`authority=FunctionizedUnits`, `residuals=none`) — the size wall was genuinely
gone. A **delta-free** baseline then returned **1/7**: five rows green on `main`
hit a carried-scrutinee producer-`Match` refusal, and a sixth hit a distinct
closure-capture refusal. ⭐ **The port was not additive — it regressed `main`.**
Every prior measurement had carried a delta, so the regression set was invisible
to all of them.

⇒ **Two obligations on every remaining port node in this campaign:**

- ⭐ **`D0` — run the target suite on the base with NO delta applied**, before
  building, and record which rows are green. That set is the regression
  baseline. ⛔ A measurement carrying your own delta structurally cannot produce
  it.
- ⭐ **Factor `AC-1` in two**, because one criterion cannot carry both claims:
  - **`AC-1a` — the ceiling moved.** The selector reports
    `authority=FunctionizedUnits` / `residuals=none` on the governed programs.
  - **`AC-1b` — the objects still build.** Every row green in `D0` is still
    green. ⛔ This is the criterion that catches the regression, and ⛔ it is
    **not** discharged by `AC-1a`.

⚠ **CI does catch this** — `rt_parity_native` has run as its own job since
2026-07-22. But it catches it *after* a QA cycle and a full CI run, on a
candidate already cut. `D0` costs one suite run before any code is written.

### Trap 3 — a proof over an incomplete population is vacuous

**Measured 2026-07-29, and it rejected a candidate that was otherwise sound.**
`RT-JOIN-DISPOSITION`'s `27f9dca2` built a completed-CFG *materialized-but-dead*
proof — entry reachability, live predecessor input, reachable block-param use —
quantified over `materialized_join_blocks`. One production site
(`lower_dynamic_host_result_match`) created a **real** planned CLIF merge and
appended its parameters **directly**, bypassing `append_planned_join_params`.

⇒ That block was never recorded, so for the whole HostResult class the proof ran
over an **empty list and passed**. A real merge would have been classified
"metadata-only materialization," and all three CFG obligations were **vacuous**.
Architect ruling `evt_24esnraje522r`.

⭐ **The shape generalizes past that node, and this campaign is full of
population-quantified proofs** — residual enumerations, reached-case unions,
materialized-block sets, and [`RT-DESCENT-RETIRE`](issues/RT-DESCENT-RETIRE.md)'s
"no residual fires anywhere."

⛔ **So whenever a node adds a proof over a population, the paired obligation is
a control that REDS WHEN A MEMBER IS OMITTED FROM THE POPULATION** — not merely
one that passes when the proof holds. ⚠ The sound-proof-over-incomplete-population
failure is silent by construction: **every control over it passes**, which is
exactly why it survives review.

### Trap 4 — ⭐⭐ a red CI job is not evidence that the atomic set is too small

**Measured 2026-07-30, and it nearly re-scoped two nodes for nothing.** The
`RT-DECL-CLOSURE-PORT` `D7` + `RT-RECURSOR-TRANSPORT` atomic candidate reached CI
as PR #1251 and came back **8 of 12 red**, against a `main` that was **12 of 12
green** — so the failures were genuinely the lineage's, not inherited. Two of the
refusals named shapes that *look* like later nodes' territory, and the
`RT-DECL-CLOSURE-PORT` frame itself already said `AC-4` was *"unreachable until
its consumers are complete."* ⇒ The obvious reading was **fold the successors in
and land a bigger set.**

⛔ **That reading was wrong.** Architect `evt_21gpwrsewyxax`: CI falsified **that
SHA's sufficiency, not the node partition.** Both refusal classes were in-scope
defects of the pair — a capture contract projected down to `capture_count`, and a
`#26` population omission falling through to a late generic refusal.

⭐⭐ **THE TEST, and it binds every node in this campaign:**

> **A node joins an atomic landing only when a failure fires THAT NODE'S OWN
> PRODUCING PREDICATE.**

For the two remaining syntactic residuals that means, concretely:

| node | its producing predicate |
|---|---|
| [`RT-SEED-CALL-PORT`](wp/RT-SEED-CALL-PORT.md) | a `Call` whose callee is the retained non-lexical closure form |
| [`RT-PRODUCER-MATCH-PORT`](wp/RT-PRODUCER-MATCH-PORT.md) | an ordinary producer `Match` whose scrutinee is directly a `Call` |

⛔ **A test name is not ownership, and neither is a pre-port refusal text.** Both
mislead in the same direction here: a row that failed at the base with one
refusal can reach a *different* wall after the port, and the base text then names
the wrong owner. ⇒ Attribute by **the predicate the failure actually fires on the
candidate**, which means the per-row **first-refusal map** Trap 2 already
requires — ⛔ never the aggregate, never the test name.

⚠ **And the reason this trap is expensive rather than merely wrong:** widening an
atomic set is nearly irreversible in practice. It lengthens the critical path,
re-opens settled frames, and every node in front of the widened set compounds.
Reading a red job as a partition defect spends that cost to fix an implementation
bug.

### Trap 5 — preflight follows a synthesized aggregate

**Architect ruling `evt_1zq4fkh6a1jv5`**, generalizing the defect that rejected
`dec_pg3yyzhx085j` (`evt_3qywmvj31e7y9`). ⭐ **The scope is the allocation
mechanism, not a node name** — `RT-DECL-CLOSURE-PORT`'s worker environment was
the *witness*, never the boundary.

> **Trap 5 — preflight follows a synthesized aggregate, not a node or
> callable-unit name.** Creating or calling a separately emitted unit does not by
> itself create a capture-environment aggregate. But whenever a remaining node
> synthesizes a `Constructor` or `Record` environment, planning must, before any
> allocation or publication, issue one exact occurrence record keyed by the exact
> callable/environment owner identity and phase, aggregate identity/class/arity,
> and ordered child provenance; close the possible referent-owner set of every
> transitive child; select `PersistentGround` only when every possible child is
> immediate or persistent and `InvocationAggregate` when any possible child is
> invocation-owned; and reject every forbidden, unrepresented, or protocol-only
> child. Lowering consumes the move-only exact occurrence authority before
> allocation and does not rediscover ownership. The existing escape guard remains
> defense in depth. If the port allocates no aggregate, record that fact; do not
> mint an artificial environment or token merely because a callable unit exists.

**Why this is not new law.** `RT-DECL-CLOSURE-PORT.md:1290-1311` already requires
an exact planner-issued occurrence record for **each** `Constructor` or `Record`
occurrence. The `7b860005` worker environment was one new `Record` allocation
site that **bypassed** that existing law.

⛔ **And why a blanket "one environment token per callable unit" would be FALSE.**
At exact `6b9fe2bf` two FunctionizedUnits paths create and call units while
allocating no capture `Record` at all: the direct lexical-closure call builds an
ordered `args + captures` ABI-input vector and calls the declared unit
(`core.rs:7281-7317`), and the ordinary lowered-closure call extends `call_env`
with captures and calls the unit (`:7344-7395`). ⇒ A per-unit token would
manufacture state the mechanism does not need.

**What binds each remaining node:**

| node | what binds |
|---|---|
| [`RT-SEED-CALL-PORT`](wp/RT-SEED-CALL-PORT.md) · [`RT-PRODUCER-MATCH-PORT`](wp/RT-PRODUCER-MATCH-PORT.md) | If the lawful port is only typed parameter/capture/result transport and allocates no aggregate, the obligation is **vacuous — record that fact.** If either port synthesizes a `Constructor`/`Record` capture environment, the complete preflight / owner-meet / exact-token law binds that site **before its first allocation.** |
| [`RT-DESCENT-RETIRE`](wp/RT-DESCENT-RETIRE.md) | A **deletion** node, ⛔ not authorized to invent a new environment. It must preserve every surviving FunctionizedUnits allocation law. ⭐ If deleting the old lane **exposes** a synthesized aggregate site that lacks the law, that is a **hard stop** — ⛔ not permission to delete around it. |

⚠ **This is a framing invariant, not a new AC**, and it authorizes no change to
any active repair.

## 4. Schedule

Runtime is single-threaded, and **every node here edits `lowering/core.rs`**, so
this is a strict sequence, not a fan-out.

> ### ⛔⛔ CORRECTED 2026-07-29 — `PX8` IS **NOT** RELEASED AT #3
>
> **#3 absorbs a consumer-matrix deliverable instead.**
>
> This section previously claimed *"`PX8` is released at #3… Foundation resumes
> in parallel from #4 onward and this campaign does not hold it."* **That was
> measured false on 2026-07-29** and the corrected schedule is below.
>
> `RT-DECL-CLOSURE-PORT` did exactly what it promised — **both** governed deltas
> now report `authority=FunctionizedUnits`, `residuals=none`
> (`evt_69ebt7hwg8508`). But the rows `PX8-ERRID-ALLOC` and
> `NATIVE-HANDLE-CARRIER` must *compile* then hit a **newly reachable**
> `FunctionizedUnits` refusal — `ComputationalMatch: tree-producing match
> scrutinee is not Bool or a constructor` — which the Architect filed under
> **`RT-PRODUCER-MATCH-PORT`** (`evt_5catd48dv8db6`, hard stop #22).
> ⇒ **The ABI release gate moved from #3 to `RT-PRODUCER-MATCH-PORT`.**
>
> ⭐⭐ **THE MATRIX RULING (`evt_6h6vzqw7ydra8`) SUPERSEDED PER-CELL OWNERSHIP.**
> The repair is **one closed `Carried`-consumer matrix**, added to **#3** as
> `D7`, and it **lands atomically with `483ef7ab`**. ⛔ **The two observed
> refusals are NOT separate nodes**, and #4/#5 are **not** reordered — they keep
> their syntactic residual retirements and gain nothing here.
>
> ⚠ **Why no node could be split off:** #3 alone regresses existing green rows; a
> consumer node cannot merge first with a reaching production witness; and #3
> cannot merge first. ⇒ ⭐ **A nominal node with no independent safe merge
> boundary is a label, not a node.** ⛔ So the ABI release is **#3's own merge**,
> now carrying `D7`.
>
> ⚠ **The lesson, since this is the second time it bit in two days:** the false
> claim was not a measurement error — it was a **scope inference** ("the ABI
> campaign needs `TransparentDeclarationClosure` retired **and nothing more**")
> written as if measured. The identical shape held Foundation for a day on
> 2026-07-28. ⛔ A release edge asserted from scope reasoning is not a release
> edge until a row compiles.

```mermaid
graph LR
  JD[RT-JOIN-DISPOSITION] --> NHC[NATIVE-HANDLE-CARRIER]
  NHC --> DCP[RT-DECL-CLOSURE-PORT]
  DCP --> SCP[RT-SEED-CALL-PORT]
  SCP --> PMP[RT-PRODUCER-MATCH-PORT]
  DCP === RXT[RT-RECURSOR-TRANSPORT]
  RXT --> RET[RT-DESCENT-RETIRE]
  PMP --> RET
  RET --> SPLIT[RT-BACKEND-MODULE-SPLIT]
  DCP --> ALLOC[PX8-ERRID-ALLOC]
  DCP --> NHC2[NATIVE-HANDLE-CARRIER resume]
  ALLOC --> SCOPE[PX8-ERRID-SCOPE]
  SCOPE --> PX8
```

| # | node | size | why here |
|---|---|---|---|
| 1 | `RT-JOIN-DISPOSITION` | M | ✅ merged; repaired the phase invariant the whole campaign kept hitting |
| 2 | `NATIVE-HANDLE-CARRIER` | M | ⛔ **held at `85dcee25`** — reached #3's ceiling; resumes on #3's merge |
| 3 | `RT-DECL-CLOSURE-PORT` | **L+** | **builds the closure-seed → callable-unit machinery** #4/#5 reuse. ✅ mechanism gate discharged on **both** deltas. ⭐ **Now also carries `D7`, the closed `Carried`-consumer matrix, and holds the ABI release** |
| 4 | `RT-SEED-CALL-PORT` | S–M | cheapest; reuses #3 directly and may close on its own `D1` |
| 5 | `RT-PRODUCER-MATCH-PORT` | M | its **syntactic** `ProducerMatchCall` retirement only — ⛔ **not** the carried-`Match` transport, which is #3's `D7` |
| **3-atomic** | `RT-RECURSOR-TRANSPORT` | L | ⭐⭐ **MOVED 2026-07-29 from #6 — it is `D7`'s reached successor and assembles ATOMICALLY with #3.** `D1` is answered: outcome **(b)** |
| 7 | `RT-DESCENT-RETIRE` | M | delete the selector, enum, authority and lane; bank the win |
| 8 | `RT-BACKEND-MODULE-SPLIT` | M | ⭐ **operator, 2026-07-31** — split the oversized `ken-runtime` backend files. ⛔ **After #7, never before** — see below |

> ### ⭐⭐ REORDERED 2026-07-29 — THE HARDEST NODE MOVED FROM #6 TO #3-ATOMIC
>
> **Architect ruling `evt_5zr53v2dp86md`, on exact `820d3e53`.** `D7` stopped at a
> lawful successor seam whose refusal is `RT-RECURSOR-TRANSPORT`'s predicate
> verbatim (*"a computational recursor closure names an in-flight activation, not a
> transferable value"*). Three consequences:
>
> 1. ⛔ **`RT-PRODUCER-MATCH-PORT` is no longer a prerequisite** for that reached
>    population — `D7` already supplies enough producer-`Match` path to expose the
>    recursor boundary. Its `depends_on` edge to #6 is **removed**. ⚠ Its own
>    `ProducerMatchCall` syntactic retirement remains separate and still owed.
> 2. ⛔⛔ **#3 and the recursor node assemble ATOMICALLY** on the `820d3e53`
>    lineage — one branch, **one candidate, one PR**, both nodes flipping `merged`
>    together. Neither goes green alone: #3's parity gate needs the reached
>    successor, and the successor has no reaching production witness pre-`D7`.
>    ⭐ **This breaks a real cycle** (#3 held on its consumers, which depended on
>    #3), and the cycle was **mechanical** — `rt_parity_native` is its own CI job.
> 3. ⛔ Atomic assembly **does not relabel** the mechanism as `D7`.
>
> ⭐ **The mitigation below worked exactly as designed** — the `D1` probe was
> pulled forward, run against a measured seam rather than a hypothetical, and the
> answer re-cut the schedule. ⚠ It did **not** come back (a): it came back **(b)**,
> *the state need not cross*, which is a **stronger** result than the transport
> this node was originally scoped to build.

> ### ⭐ SECOND RULING, SAME DAY — the base moved; the graph did NOT
>
> The atomic pair is **unchanged**: still #3-atomic, still one candidate, still two
> nodes flipping together. ⛔ **No node was added, no edge changed, and no
> disposition was created.** What changed is where the work resumes from.
>
> | fact | value |
> |---|---|
> | ⭐ **resume base** | `c45a59a9f7bd6a911441e58ebb5e9e303e1bc7ac` (tree `1e3cfe58…`) |
> | its parent | `820d3e53` — so *"the `820d3e53` lineage"* above stays exact |
> | what it did | made the recursor refusal **advance**, then hit an ordinary-`Closure` refusal **inside** the recursor split's `Captures[Carried x7]` |
> | ruling | that `Closure` is **`RT-RECURSOR-TRANSPORT`'s**, as **one new member** of `D7`'s matrix under the **existing** `CallableCapture` disposition |
> | what is owed | an **ordering** repair — validate the whole residual/environment **before** allocating — plus reach through **every** governed recursor position |
>
> ⛔ **Cut from `c45a59a9`, not `820d3e53`.** Cutting from the parent discards a
> ruled-correct advance. `c45a59a9` is preservation-only **and** the repair base;
> both at once, and neither cancels the other.
>
> ⚠ **The one thing here worth reading past the SHAs:** the attribution is
> **population-scoped**. It settles this recursor edge and ⛔ *"does not globally
> attribute every future `Closure` refusal"* — so a later refusal with identical
> text is a fresh question, not a settled one.

> ### ⭐⭐ AND THAT FRESH QUESTION ARRIVED WITHIN THE HOUR — stop `#27`, still no
> ### node, still the same atomic pair
>
> **The population-scope caveat directly above stopped being hypothetical.** The
> ordering repair was built (`07ce6ef1`, tree `ee3f2bb9…`, parent exact
> `c45a59a9`, 4 Runtime files, +777/−121, targeted checks green, static-recursor
> tests 3/3) and then met a **different** `Closure` edge with the **same refusal
> text**: parent `StaticOriginId(655)` / child `650` / body `641`, arity `1`,
> captures **`8`**, inside `transfer_constructor_operands` on a carried
> computational-match path — against the ruled member's worker body `723`,
> `Carried x7`. **No planner-proved token exists for it** (`evt_4tvysmzr6mfpb`).
>
> ⭐ **The caveat is what made the ring stop instead of absorbing it.** "Same text
> as the thing that just got ruled" is the exact reasoning the population scope
> forbids, and it was available and declined.
>
> ⛔⛔ **STILL NO NODE, NO EDGE, NO DISPOSITION, AND NO SCHEDULE CHANGE.** The pair
> is still #3-atomic. The campaign's shape is untouched by this stop.
>
> ⚠ **What IS different: this is the fifth instance of one shape**, and it landed
> where `RT-DECL-CLOSURE-PORT` §5a predicted the next one would. That record's
> `#27` triggers both fired in one pass (`evt_3tx7ndxp5pm4j`) and the question put
> to the Architect is about the **derivation** — *why does the closure keep failing
> to be closed?* — ⛔ not *which cell is next*. Read §5a's discriminator table
> before treating a new cell as the resolution.
>
> ⛔ **The §8 bounded-witness protocol is now spent on BOTH known edges.** There is
> no third charge, so a further refusal outside both populations is an attribution
> question first, with no diagnostic authorized by default.

> ### ⛔⛔ RULED — THE DERIVATION WAS WRONG. Still no node; the SCOPE grew.
>
> **Architect `evt_4p9ne0vcds5hb` + addendum `evt_3gzcnk62v8bzz`, Research advisory
> `evt_62tkq32hrjqmn`.** The `#27` question came back **wrong-derivation**, not
> incomplete: `D7`'s claim to a closed boundary-operand population is **withdrawn**.
>
> ⭐ **The decisive finding is sharper than a missing cell.** `#27`'s edge was
> **already in the source inventory** as `ConstructArgument -> SemanticEliminator`
> — but the real crossing never consumes that cell; it hits a gate that mints an
> identity-free `CallableCapsuleEscape` token. **One event, two independent
> verdicts, two populations.** ⛔ Adding outcomes cannot repair a key that does not
> determine the verdict.
>
> ⛔⛔ **STILL NO NODE, NO EDGE, NO SEVENTH DISPOSITION, NO SCHEDULE CHANGE** — all
> explicitly unauthorized, and the pair is still #3-atomic. ✅ `Need ⊆ Avail`-or-
> eliminate survives as the governing predicate; the six dispositions survive as the
> **codomain**.
>
> ⚠⚠ **BUT THE WORK GREW, AND THAT IS AN OPERATOR-FACING FACT, NOT A NODE
> QUESTION.** What was a bounded ordering repair is now a planner **population
> re-derivation**: one `BoundaryUse` record per static lowering event, raw phase
> transitions made private behind one choke-point API, unforgeable planned-edge
> tokens, `.token()` minting removed from lowering, and a planned-vs-emitted ledger
> comparison before function definition — plus ten controls. ⭐ **`PX8` and the ABI
> program sit behind this pair**, so the cost lands on that critical path. ⛔ The
> Architect forbade splitting it out, so the answer is **not** a node; the honest
> statement is that #3-atomic is now a substantially larger node than when it was
> scheduled.

> ### ⛔⛔ THE RELEASE POINT IS A **CONDITION**, NOT A NODE NUMBER
>
> ⚠ **A draft of this block twice asserted a node id as the release point** —
> first #3, then #4. **Both were the same unmeasured scope inference.** The
> release is #3's merge only because `D7` was *added to* #3; ⛔ it is not a
> property of the number.
>
> **Measured 2026-07-29 (`evt_1b1v2qjy82epm`):** targeted `rt_parity_native` on
> clean `483ef7ab` with **neither** delta is **1/7** — five existing-`main` rows
> hit the producer-`Match` population, and a sixth,
> `buffer_allocate_malformed_capacity_narrows_to_invalid_bounds`, hits a
> **distinct carried closure-capture refusal** that the prior ruling does not
> cover. ⇒ **Two different consumers, not one.**
>
> ⭐ **So the release condition is: every consumer that can receive a `Carried`
> operand eliminates it.** Which node numbers that spans is open until the
> Architect classifies the second refusal. ⛔ Do not write a release edge against
> a node id until a row compiles.

⭐⭐ **THE UNDERLYING SHAPE — one incomplete matrix, not N bugs.**
`RT-DECL-CLOSURE-PORT` introduced a **new representation**: a declaration result
crossing the callable-unit boundary as `LoweringOperand::Carried`. **It did not
enumerate the consumers that must eliminate that representation.** Both refusals
are the same cell type — *a `Carried` value reaching a consumer built only for
specialized shapes* — found in two different consumers, and found only because
seven parity rows happened to reach them.

⇒ ⛔ **Fixing the two known cells does not bound the population.** That is
precisely the failure §3 of this document already names: *a proof over an
incomplete population, where every control passes.* **The paired obligation
here is an enumeration of `Carried`-receiving consumers with a control that reds
when a member is omitted** — the same discipline the residual enumerator gave
the *producer* side, now owed on the *consumer* side.

### ✅ The one scheduling risk worth stating plainly — RETIRED 2026-07-29

**It read:** *the hardest node is sixth, so if `RT-RECURSOR-TRANSPORT` proves
infeasible as scoped we learn it after five nodes of investment — and
"half-migrated" is exactly the state the operator directed us out of.* The stated
mitigation was that its `D1` is **a feasibility probe runnable at any time**,
independent of queue position, whose result re-cuts the schedule.

⭐⭐ **That is what happened, and the risk is now spent.** `D1` was pulled forward
at `D7`'s hard stop #25 — against a **measured** seam rather than a hypothetical —
and the Architect answered it **(b): the state need not cross**
(`evt_5zr53v2dp86md`).
The node moved from #6 to **#3-atomic** on that result.

⚠ **Two things worth keeping from how this went.** First, the probe was only worth
anything because it was **reachable**: `D7` had to advance the edge far enough to
produce a real refusal before the question could be asked at all — a `D1` run
speculatively at #1 would have had no production witness. ⇒ ⭐ *"Runnable at any
time"* was true, but *"informative at any time"* was not. Second, the answer came
back **stronger** than the deliverable was scoped for — the node was framed to
*build a transport*, and the ruling instead **eliminates the crossing**. ⛔ A frame
that had only asked "build the transport" would have had no slot for that answer.

### ⭐ #8 — the module split goes AFTER the capstone (operator, 2026-07-31)

The `ken-runtime` backend files are oversized again. A previous interlude of this
exact shape produced the `cranelift_backend/` directory, and the operator asked
whether to repeat it **now**, as a pause in this campaign, or after it.

**Ruling: after.** ⛔ The split is `RT-BACKEND-MODULE-SPLIT`, node #8, gated on
`RT-DESCENT-RETIRE`. Measured sizes on `main = 1e6eb5c6` (crate: 97,881 lines
across 49 files):

| file | lines |
|---|---|
| `cranelift_backend/lowering/mod.rs` | 11,197 |
| `cranelift_backend/lowering/core/tests/control.rs` | 9,847 (test) |
| `cranelift_backend/lowering/core.rs` | 9,788 |
| `cranelift_backend/planning/static_transition.rs` | 9,034 → **>20,858** in #3-atomic's delta |
| `boundary_value_clif.rs` | 8,691 |

**Three grounds, in order of weight.**

1. ⭐⭐ **#7 SUBTRACTS FROM EXACTLY THESE FILES.** `RT-DESCENT-RETIRE` deletes
   the classifiers, the enum, the authority and the whole lane across
   `lowering/core.rs` (22 occurrences), `core/tests/control.rs` (16),
   `lowering/mod.rs` (4), `planning/static_transition.rs` (3) and
   `object_linker_packaging.rs` (1), and its `D6` retires or re-homes the lane's
   tests. ⇒ Splitting first means **re-homing a lane into new modules and then
   deleting it out of those new homes one node later** — paid twice, and the
   second payment discards the first.
2. **#4 and #5 are CONSUMERS of the transport, not authors of it.** The growth in
   `static_transition.rs` came from *building* the continuation-specialization
   mechanism at #3-atomic. Both remaining ports **ban a second transport** in
   their frames and make needing one a hard stop and a re-cut. ⇒ The peak is
   roughly **now**, not later.
3. ⛔ **They contend on the same files** and cannot run concurrently, so this is
   purely an ordering question — and campaign-first is the order without rework.

⚠ **The counter-hypothesis, stated so it can be revisited:** that large files are
themselves making this work harder. No evidence was found for it here — #3's
three hard stops were **semantic** (a false scalar-recursor premise; the dynamic
same-tag continuation fork; census fidelity), and none gets easier in a smaller
file. ⛔ But that is inferred from reports; the implementer and Architect are
better placed to judge it than the Steward.

⭐ **The one place the counter-hypothesis could still win, and the cheap test.**
#5 is the only remaining node that must do real work inside a 20k-line
`static_transition.rs`. **At #3-atomic's merge, ask the Architect whether a
narrow split of that one file should ride ahead of #5.** One exchange — far
cheaper than an interlude, and it does not disturb this ordering if the answer
is no.

⭐ **And #8 is cheaper than the precedent it is modelled on.**
`static_transition.rs` **already has** a `static_transition/` subdirectory
(`semantic_ir.rs` 2,729, `abi.rs` 1,601). The seam exists, so #8 **extends an
established split** rather than inventing an architecture the way the original
`cranelift_backend` extraction did. ⛔ #8 is deliberately left unframed until #7
merges — the deletion changes where the natural module seams are, so a frame
written now would be sized against a tree that is about to disappear.

## 5. What "done" means

⛔ **Retiring all five residual classes is NOT the finish line.** With every
class retired, the selector still exists, still evaluates, and the
`RecursiveDescent` lane is still compiled in — dead. **That residue is precisely
the tech debt the directive names**, so `RT-DESCENT-RETIRE` is a required node,
not a tidy-up.

Done is: the selector, `RecursiveDescentResidual`,
`BodyEmissionAuthority::RecursiveDescent`, and the recursive-descent emission
lane are **deleted**, and every program compiles through `FunctionizedUnits`.

⭐ **And the efficiency claim is measured, not asserted.**
[`RT-SCALE-B`](wp/RT-SCALE-B-emission-scaling-verdict.md) returned verdict (a) —
linear, no exponent — but it was **bounded to the governed recursive
resource-bracket populations and excluded the mutually exclusive
`RecursiveDescent` root** (Architect, `evt_3t7t27e3rv8cx`). ⇒ **The monolithic
root has never been scale-measured.** `RT-DECL-CLOSURE-PORT.AC-6` takes the
first such measurement; `RT-DESCENT-RETIRE` takes the last. ⛔ Neither pins a
threshold — a pinned size number rots at the next merge. The obligation is that
the numbers exist and are routed.
