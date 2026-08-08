# RT-RECURSOR-TRANSPORT — retire the two live recursor residual classes

Owner: Runtime. Size: **M, provisional** — see Sizing.
Authority: Architect recut ruling `evt_237tbdsacqbk4` (2026-08-08), answering
the Steward's re-derivation request `evt_4hr31qp6ab5xg`.

**Read `docs/program/16-recursive-descent-retirement.md` first** — the campaign
context and the five traps that bind every node in this arc.

> # RECUT 2026-08-08 — THIS FRAME REPLACES ITS PREDECESSOR ENTIRELY
>
> The previous frame was 730 lines written across five superseded recuts, against
> a world where `RT-DECL-CLOSURE-PORT` `D7` had not landed and the
> ContinuationSpecialization seams did not exist. **It is not amended and it is
> not context. Its contract, its ordering rule, and its base are all withdrawn**
> — see the node file's recut banner for the three withdrawals in full.
>
> **The short version of what changed:** the hard internal mechanism the old
> `L`-sized text expected this node to *invent* has since **landed**.
> Continuation specializations already bind exact producer occurrence and
> alternative, worker provenance, continuation origin and recursive position,
> emission owner, typed inputs, and an opaque causal call identity. Lowering
> already has checked carried-match and lexical declared-unit call transports.
>
> ⇒ **Your first job is to find out how much of this node is already done.**
> That is `D1`, and it is genuinely open — it may close a class for free.

## 1. Fixed inputs

**Measure all of these yourself at your pinned base.** The values below were
measured by the Steward at `d9b2eb38` on 2026-08-08 and are **anchors to
re-find, never values to check** — this node's own base is later than that by
construction, and `WITNESS` moves the very files involved.

| input | as measured 2026-08-08, at `d9b2eb38` |
|---|---|
| the two live variants | `MatchScrutineeRecursor`, `LexicalCallArgumentRecursor`, both in `lowering/core.rs` — declared in the `RecursiveDescentResidual` enum, classified in `recursive_descent_residual`, and collected into a `found` set |
| retired siblings | `TransparentDeclarationClosure`, `SeedClosureCall`, `ProducerMatchCall` — three of five |
| `BoundaryUse` | **zero hits in `crates/`**. Surviving references are historical docs only |
| `D7`'s landed authority | `PlannedEffectSeat` — the record in `planning/static_transition.rs`, derived by `build_host_effect_seat_plan` over admitted host-effect occurrences, consumed through a claim ledger whose `close` refuses an unclosed visit and refuses `committed != opened` |
| your base | **not fixed here.** Branch from `main` after [[RT-CONTSPEC-WITNESS]] merges, and pin it in your first checkpoint post |

**Cite by grep-able phrase, not by line number.** Every coordinate the previous
frame carried for these two variants is now wrong — it said `core.rs:96-105` and
`core.rs:125-136`; they had moved to `:591` and `:598` before this recut and will
move again under `WITNESS`. A coordinate is a time-sensitive operand.

## 2. What is owed, and what is emphatically not

### Not owed: a global population authority

The withdrawn contract asked for *"one exact `BoundaryUse` record per static
lowering event"* with a choke-point API and a planned-set-vs-emitted-ledger
comparison. **Do not build that, and do not widen `PlannedEffectSeat` into it.**

`PlannedEffectSeat` is discharged **for its own domain**. Its key, its Need/Avail
vocabulary and its choke point are intentionally effect-specific and do not
extend to either residual class. Widening it repeats the exact domain conflation
ruled out in `evt_1v9m7t4m9dmj7` — the confusion `D7` was built to prevent.

**There is no missing universal authority.** Lowering deliberately uses separate
exact authorities for separate semantic populations: host-effect seats,
aggregate allocation occurrences, continuation source slots, continuation
specializations and call identities, join plans, typed declared-unit calls.
Security comes from **the exact domain-specific producer plus its checked
consumption boundary**, not from one global token vocabulary. A proposal to
unify them is out of scope here regardless of its merits.

### Owed: the two live consumer positions

Both variants still select `RecursiveDescent`. That is the work, and it is all
of the work.

## 3. The invariant that survives — outcome (b)

Unchanged, and it is the reason this node is hard:

> **Invocation-local activation, resume and return-hole state never enters ABI
> data.** Only ordinary typed values cross a unit boundary. Static continuation
> and callee identity remain planner- and compiler-owned. Any open, escaping or
> ambiguous case **refuses before allocation or call emission**.

**Planning must reject, not degrade.** A case you cannot prove lawful is a
refusal at planning time, never a silent fall back to the retained lane and
never a partial emission you clean up afterwards. Validate first, allocate
second.

## 4. Deliverables

### `D0` — re-census on the pinned base

Confirm both variants are still live and still selected, and record the exact
classification sites by phrase. **Preserve the exhaustive two-variant selector
and enumerator through `D0`-`D2` and every commit up to the final
variant-removal commit** — that is the window in which it discriminates, and
the window in which its evidence must be captured.

**It does NOT stay discriminating through the whole transition, and this
sentence said it did until 2026-08-08.** `D3` empties the population by design;
see the reconciliation under `D3` and the rewritten `AC-6`.

If a variant is already unreachable at your base, that is a finding: record it
with the evidence and route it. Do not delete it on that basis alone.

### `D1` — the activation probe, one discriminating witness per position

**This is the deliverable that sizes the node.**

Under a **test-only per-variant selector exclusion**, run **one discriminating
executable witness for each position** and record the **first real functionized
outcome**. The question `D1` answers:

> Does the landed continuation machinery already close either class for free?

- A witness must **discriminate** — it distinguishes this position's transport
  working from it not working. A compile-time refusal that never executes is not
  an outcome; route it as a refusal and say so.
- Record the first **real** outcome, not the first thing that goes red. A red
  from the exclusion harness itself measures the harness.

**Post the `D1` result before starting `D2`.** It may close the node, halve it,
or trigger the hard stop below.

### `D2` — only for a class `D1` shows does not close for free

Add **only the narrow consumer-port authority that class's failure proves
necessary.**

- **Reuse the existing continuation specialization / call identity and typed
  value transport wherever they already name the edge.**
- If a genuinely new fact is required, it must be a **domain-specific,
  planner-owned binding for that exact recursor consumer occurrence and its
  static downstream continuation or suffix.**
- It must **not** be `BoundaryUse`, must **not** be `PlannedEffectSeat`, must
  **not** be a runtime selector, and must **not** be a lowering-minted token.

> #### `D1` ANSWERED — settled decisions only
>
> **THE `D2` TECHNIQUE IS IN FLIGHT AND IS DELIBERATELY NOT CANONIZED HERE.**
>
> **Settled, and safe to rely on:**
>
> - `D1` came back **asymmetric** (checkpoint `2e5e6a8b`). Position B
>   `LexicalCallArgumentRecursor` **closes for free for the exact executable
>   witness `D1` measured** — that witness's functionized lane executes and
>   yields the same decoded `RuntimeObservation`, and that result stands.
>
>   > **WITHDRAWN 2026-08-08 AS A CLASS-WIDE CLAIM — Architect
>   > `evt_5w09dcwbf7k70`, on `D3` hard stop 4.** This bullet previously read
>   > *"position B closes for free"* full stop, with no scope on it, and the
>   > Steward propagated that unscoped form into this frame, the briefing and
>   > three watchdogs.
>   >
>   > **The evidence against it:** at `D3`, six previously-green semantic
>   > controls fail closed. **Five of them — eight expressions across five test
>   > families — enumerate exactly `{LexicalCallArgumentRecursor}`**, position
>   > B's own population, and two of those refuse with the exact string `D2`'s
>   > A/B reproduces.
>   >
>   > ⇒ **True of the one witness `D1` measured, false of the class.** Position
>   > B's population is repaired by [[RT-LEXICAL-RECURSOR-CONSUMERS]], which
>   > gates `D3`.
>   >
>   > **A SECOND WITHDRAWAL, same day — "all six reach the lane through
>   > `host_result_closure_match`" is ALSO false**, and three seats restated it
>   > before anyone opened the sixth fixture. Row 6 (`d8d`) builds an ordinary
>   > `Match` whose scrutinee is `px8j_deferred_recursive_field_fixture()` —
>   > **position A's shape** — and enumerates exactly
>   > `{MatchScrutineeRecursor}`. It was never in position B's population, and
>   > it is owned by [[RT-MATCH-RECURSOR-CONSUMERS]], which also gates `D3`.
>   > Census of record `evt_16cmej481q7ns`; partition `evt_3r4j14fv1jtj2`.
>   >
>   > **Both withdrawals are one defect at two scales:** a summary looser than
>   > the measurement under it, repeated by readers who did not open the
>   > underlying object. The first generalized one green witness to a class; the
>   > second generalized five fixtures' shared helper onto a sixth nobody read.
>   >
>   > **How it survived:** one green witness was read as a property of the
>   > class. Nothing in `D1` would have caught it — a passing case cannot
>   > report the cases it never exercised. The implementer surfaced it against
>   > its own interest and labelled it as such, which is the only reason it was
>   > caught before the retirement landed.
> - **`D2` is `MatchScrutineeRecursor` alone.**
> - **One node.** The fold survives, but its *"same mechanism / would build the
>   same transport twice"* justification is **withdrawn** — `D1` disproved it,
>   since B needs no transport at all. Splitting B out would produce a deletion
>   that cannot fail.
> - **Hard stop 1 remains neither triggered nor cleared.** It presupposes two
>   transports and one position needs none. Do not record it as
>   considered-and-cleared.
> - **Hard stop 2 is UNANSWERED GLOBALLY.** It is *not* triggered by the
>   generated-context population — `contexts=[]` by itself can never be that
>   trigger, because generated contexts are intentionally a strict subset of
>   specialization calls. That is a narrowing, not an answer.
> - **Sizing: `M`.** Scope halved, variance concentrated in position A.
>
> ⇒ **NO PRODUCTION `D2` EDIT IS AUTHORIZED** as of 2026-08-08 ~09:5xZ. Not a
> generated context, not a specialization redirect, not a `StaticWorker` port.
> Only a test-only causal trace on the `D1` witness. `c715e692` is **held
> evidence, not a production candidate.**
>
> **This frame deliberately does NOT restate the `D2` technique, and that is a
> correction of my own error.** I previously canonized one here from Architect
> ruling `evt_46yzde84ky6ax`. **Its site premise had already been withdrawn**
> at `evt_5yeh0tfp4gwwb` when exact instrumentation measured that only the
> `carried_inner` IH seat is reached, `composed_recursive_argument_binding`
> entry count is **zero**, and the specialized composed target lookup never
> runs. I froze a superseded mechanism into the contract, which is worse than
> leaving it in the thread, because a frame reads as authoritative.
>
> ⇒ **The `D2` technique is a live, moving object and its home is the WP
> thread.** To find the current boundary, **read the LATEST Architect ruling in
> this thread — not the first one that answers your question.** As at
> 2026-08-08 ~09:5xZ that is **`evt_5we1eh4k2hhry`**, which authorizes one
> ordered, correlated test-only continuation-consumption trace and nothing
> else. **Assume that event id is stale; verify it is the latest before
> acting.**
>
> **One fact worth keeping, scoped precisely.** `Lowered::Constructor`'s
> `occurrence` is a planner-issued `AggregateOccurrenceId`, and
> `aggregate_record_view` yields `producer_origin()` and `shape()` — so the
> in-tree comment claiming the real producer origin needs a lowering signature
> change is **false**. **This is a fact about the UNREACHED specialized seat.
> It is not `D2`'s active mechanism, and correcting that comment is not
> currently owed by anyone.**

### `D3` — joint retirement

> #### `D3` IS GATED ON TWO SUCCESSOR NODES — 2026-08-08, hard stop 4
>
> **Do not resume `D3` from `10369776252861e8b15e613576256a3682c70066`.** That
> checkpoint is **held evidence only**: it is not a candidate, not the repair
> base, and not to be continued. Architect ruling `evt_5w09dcwbf7k70`, corrected
> at `evt_ykbnr68eb3bs`, partitioned at `evt_3r4j14fv1jtj2`.
>
> **What happened.** The retirement is applied and `AC-2b` is fully
> dispositioned at 22 items, but six previously-green semantic controls then
> fail closed across five distinct refusal boundaries. `D3` alone would turn six
> durable controls red and spend the only probes for five guards.
>
> **The census (`evt_16cmej481q7ns`) partitioned them into two populations with
> two distinct activation seams** — nine expressions measured individually, none
> of them A+B:
>
> | rows | population | seam | owner |
> |---|---|---|---|
> | 1-5, eight expressions | exactly `{LexicalCallArgumentRecursor}` | B-only exclusion | [[RT-LEXICAL-RECURSOR-CONSUMERS]] |
> | 6, `d8d` | exactly `{MatchScrutineeRecursor}` | **A**-only exclusion | [[RT-MATCH-RECURSOR-CONSUMERS]] |
>
> **The order, and `D3` is last in it:**
>
> 1. The `D2` **record correction** lands — a bounded child over `8efdfdb3` with
>    no production or test-logic change, narrowing every class-wide claim to the
>    exact `D1` witness. **`8efdfdb3` itself does not land**: its record claims
>    *"position A closes"*, and row 6 is an A-only counterexample **on that same
>    object**. Approval was withdrawn mid-publish, `dec_6nsrbyw1wjpb` is void,
>    PR #1609 closed before merge, `main` never modified.
> 2. [[RT-MATCH-RECURSOR-CONSUMERS]] — **first**, because it closes the
>    Position-A claim that correction narrows.
> 3. [[RT-LEXICAL-RECURSOR-CONSUMERS]] — rows 1-5.
> 4. **`D3` resumes from the resulting `main`**, jointly retires, replays the
>    `AC-2b` dispositions, and proves all six rows green **without** exclusion.
>
> **Two things `D3` may not do when it resumes, and they are the tempting
> shortcuts:** **no new `#[ignore]`** — quarantining these six is ruled out, not
> merely discouraged — and **no reshaping of a fixture or absorption of a
> refusal** to make a row pass. The six rows stay enabled and unchanged in
> meaning.

**Only after both executable positions are green**, retire the two residual
variants and their test-only selector hooks.

The lane itself is **not** yours. [[RT-DESCENT-RETIRE]] owns the
`RecursiveDescent` lane, its selector, its enum and its authority. Retiring
these two variants is what unblocks that node; performing its deletion here is
banned scope.

> #### RECONCILED 2026-08-08 — `D3` EMPTIES THE ENUM; THE FRAME DENIED IT
>
> **Authority: Architect ruling `evt_4tf1hhp51nyh0`**, on the implementer's
> `D3` stop `evt_zgs93z3p3mfx`. The stop was correct and the contradiction was
> in this contract, not in the retirement.
>
> These two variants are the **entire live `RecursiveDescentResidual`
> population**. Removing both leaves the enum **uninhabited**: the classifier
> answers `None` for every program, the enumerator returns the empty set for
> every program, and the complete enumerator and `ShortCircuitLikeTheSelector`
> agree on `{}` for every input. **The exact-set control therefore cannot
> discriminate after `D3`, and calling it green would be exactly the `0/0`
> witness this frame bans.** `AC-6` promised the opposite, so it was
> unsatisfiable; it is rewritten below as a lifecycle boundary.
>
> **This is not a four-row perimeter.** `D3` owns retiring or rewriting **every**
> control whose subject disappears with these two variants. Known at
> `8efdfdb3`, and explicitly **not a bound**:
>
> - `the_body_authority_selector_narrows_only_completed_ports_and_stays_fail_closed`
> - `retained_authority_residual_is_the_typed_selector_accounting`
> - `d1_each_residual_variant_is_observable`
> - `d1_the_enumerator_reports_every_variant_not_the_first`
> - `d3_the_exact_set_control_still_reds_under_short_circuiting`
> - `d3_the_previously_masked_classes_are_now_reported_directly`
> - every corpus, entry, wrapper and production-site row that treats an empty
>   enumeration as evidence of classifier completeness
>
> **Re-sweep the whole of `crates/ken-runtime/src` by type, by enumerator, by
> mutation, and by both variant names.** A list handed to you is a floor.
>
> **Four things you may not do to keep a test non-empty**, because each tests a
> population production does not have:
>
> 1. **Retain either production variant.**
> 2. **Invent a `cfg(test)` ghost variant.**
> 3. **Preserve a per-variant selector-exclusion hook** solely to keep a
>    population inhabited.
> 4. **Move the exact-set test unchanged to [[RT-DESCENT-RETIRE]].** With an
>    empty subject it is vacuous *there* too. That node's own `D1` zero reading
>    plus its `D2` temporary-positive control is the lawful successor evidence,
>    and **a later one-shot positive control cannot retroactively make this
>    node's final empty population discriminating.**
>
> **What survives, and what it is allowed to claim:**
>
> - **The two executable position witnesses stay, with their real
>   discriminators.** At final `D3` they must select `FunctionizedUnits`, execute
>   to their accepted observations, and **still red when their respective
>   transport is disabled.** That is what discharges `AC-2` — without any
>   residual-population fiction.
> - **The production-site `Some(empty)` hook row may remain only as a
>   site-execution sentinel** — it distinguishes *"the selector site ran"* from
>   *"it never ran."* **Its doc comment and its assertion must both say it proves
>   neither completeness nor the absence of missed residuals.** A sentinel that
>   does not disclaim what it cannot see is read as coverage.
> - **Keep the dead lane, the selector/enumerator scaffolding, the now-empty
>   enum, and the authority** for [[RT-DESCENT-RETIRE]]. Deleting them here stays
>   banned. Remove **only** the two variants, their production
>   constructor/insertion sites, and the variant-specific test hooks.
> - **Keep both collector walks.** The arm must still descend into scrutinee and
>   case bodies so a future variant is findable there.
>
> **Two obligations on the handoff:**
>
> - **Call the enumerator oracle `spent`/`dormant`, never `discriminating`.**
> - **Record the nondegenerate historical evidence at exact `D2` `8efdfdb3`**
>   — where the population was still inhabited. **Do not reconstruct it after
>   retirement**; there is nothing left to reconstruct it from.

## 5. Acceptance criteria

- **AC-1 — both variants are gone from the `RecursiveDescentResidual` enum and
  from the classifier**, at the final SHA.
  *Control:* grep the whole of `lowering/` for each variant name; zero
  production hits. Not a line-number check.
- **AC-2 — each retired position has a committed executable witness that
  discriminates.**
  *Control:* the witness fails when the transport is disabled and passes when it
  is enabled, both from the committed tree. A hand-run mutation does not
  discharge this.
  **At final `D3` both witnesses must additionally select `FunctionizedUnits`
  and execute to their accepted observations** — still redding when their
  respective transport is disabled. **These two are what carry `AC-2` once the
  residual population is empty**, so they are the controls that must survive the
  sweep intact.
- **AC-2b — every control whose subject the retirement removes is retired or
  rewritten, and none is left to pass vacuously.**
  *Control:* the handback enumerates each such control with its disposition
  (retired, rewritten to an explicit emptiness assertion, or preserved with a
  stated reason it is still probative), **and states the sweep that found them**
  — across `crates/ken-runtime/src`, on **six** axes:
  1. by **type**;
  2. by **enumerator**;
  3. by **mutation**;
  4. by **variant name**, CamelCase;
  5. by **lane** — `BodyEmissionAuthority::RecursiveDescent`;
  6. by **fixture spelling** — the snake_case form of each retired class.
  **The list under `D3` is a floor, not the perimeter.** A control left green
  over an empty population is the failure this AC exists to catch, and it is
  invisible to a suite count.

  > **Axes 5 and 6 were added 2026-08-08 because the original four missed a
  > control and it cost a red.** `d5_c3_a_second_residual_retains_recursive_descent`
  > matches none of the first four: it never names the type, never calls the
  > enumerator, arms no mutation, and reaches the class **only through a
  > snake_case fixture name that a CamelCase grep cannot match.** It was found
  > by running the suite, which is the expensive way.
  >
  > **The axes are candidate selectors, not a closure argument.** Six axes miss
  > less than four; they do not prove a perimeter. State the result as a floor.
- **AC-3 — outcome (b) holds at every new boundary.** No invocation-local
  activation, resume or return-hole state in ABI data.
  *Control:* name the ABI payload for each new crossing and show its fields are
  ordinary typed values.
- **AC-4 — every unlawful case refuses before allocation or call emission**, and
  the refusal is reachable.
  *Control:* a committed negative witness per refusal path. A negative check
  passes for any reason, so each needs a positive control proving the path is
  reached at all.
- **AC-5 — no widening of `PlannedEffectSeat` and no `BoundaryUse` revival.**
  *Control:* `BoundaryUse` stays at zero production hits; `PlannedEffectSeat`'s
  key, vocabulary and choke point are blob-identical to base unless the Architect
  rules otherwise on the record.
- **AC-6 — the exact-set enumerator stays discriminating up to the final
  variant-removal commit, and `D3` deliberately spends it there.**
  **Rewritten 2026-08-08 per Architect `evt_4tf1hhp51nyh0`. It previously read
  *"stays discriminating through the transition, including at intermediate
  commits"* — which the final state cannot satisfy, because `D3` empties the
  population.** See the reconciliation under `D3`.
  *Control, in two parts:*
  1. **Through `D2` and every commit before the final variant-removal
     commit**, the control still reds under `ShortCircuitLikeTheSelector` on a
     compound firing both variants. This is the part that must hold, and it is
     where the evidence is captured.
  2. **At the final commit** the oracle is **spent**, and the handoff says so in
     those words. The zero reading passes to [[RT-DESCENT-RETIRE]], which makes
     it probative through its own `D2` temporary-positive control. **A green
     exact-set assertion at final `D3` fails this AC** — it is the banned `0/0`
     witness wearing the AC's name.
- **AC-7 — the candidate contains NO tracker `status:` change.** This node's flip
  is the Steward's, post-merge, at `merge-procedure.md` M7. **Do not close
  [[RT-DESCENT-RETIRE]] or any other node.**
  *Control:* `git diff` over `docs/program/issues/` on the candidate is empty of
  `status:` lines; discharged by the handback stating you made none.
- **AC-8 — CI green** on the merge. Not a local `--workspace` run, which is
  banned (`COORDINATION §12`).

## 6. Banned scope

- **No `BoundaryUse` record, and no universal per-lowering-event authority.**
- **No widening of `PlannedEffectSeat`** beyond host-effect.
- **No runtime selector and no lowering-minted token** as the new fact.
- **No deletion of the `RecursiveDescent` lane, selector, enum or authority** —
  that is [[RT-DESCENT-RETIRE]].
- **No resume, reset, or cherry-pick of `07ce6ef1`** or any preserved freeze ref.
  See Base below.
- **No tracker `status:` change** (`AC-7`).
- **No weakening of an existing gate** to make a witness pass — including
  `boundary_transfer_admissibility`. If a gate blocks the lawful path, that is a
  finding to route.
- **No `0/0` witness.** A control that observes an empty population measures
  nothing.
  **This bans the CLAIM, not the state.** After `D3` the population is
  legitimately empty, and that is the point of the node — what is banned is a
  control that goes green over it and is then read as coverage. An emptiness
  assertion is fine when it says it is one; the same assertion inherited from
  when the population was inhabited is not.
- **No retaining a variant, ghost `cfg(test)` variant, or per-variant hook to
  keep a population inhabited** — see the reconciliation under `D3`.

## 7. Hard stops

Stop and route to the Steward; do not improvise:

1. **`D1` shows the two positions require materially different transports.** The
   "one mechanism" fold is then wrong. **Do not preserve that claim merely
   because both variants mention an active recursor** — re-size or re-fold is
   the Steward's call with the Architect.
2. **A class requires a new planner or ABI population** rather than a narrow
   binding over the existing continuation machinery. This is the trigger that
   makes the node `L` again, and it is the Architect's to rule on.
3. **A lawful case cannot be made to refuse before allocation** — outcome (b) is
   then in question, which is a soundness route, not a workaround.
4. **A newly reachable shape trips a fail-closed invariant** — campaign Trap 2.
   This is **expected** as classes retire. Route it as its own node; do not
   absorb it and do not adjust the lane around it.
   [[RT-FNUNIT-RESULT-TOKEN]] is the precedent: it was routed this way on
   2026-08-08 and now gates [[RT-DESCENT-RETIRE]].

## 8. Base

**Branch after [[RT-CONTSPEC-WITNESS]] merges, from that then-current `main`.
Pin your base SHA in your first checkpoint post.**

**`07ce6ef1` is not the repair base and must not be resumed.** It is **not an
ancestor of `d9b2eb38`** and survives only on preserved and old `D7` branches.
Measured: its `StaticRecursorWorker` prototype has **36 crate hits there and
zero on current `main`**, and the four core files have diverged by
**`+58,582/-17,365`** — `git diff --numstat 07ce6ef1 837f9296 --` over
`lowering/core.rs`, `lowering/core/tests/control.rs`, `lowering/mod.rs`,
`planning/static_transition.rs`. Continuing or cherry-picking it **would
overwrite the landed continuation-specialization, ownership, ABI and ledger
architecture.**

It may be cited as **historical refusal and design evidence only**. Every
mechanism claim must be re-derived on your base.

## 9. Contention

Runtime is single-threaded. This node follows [[RT-CONTSPEC-WITNESS]] and touches
the same `lowering/` files that seam moves, which is why the base is pinned at
pickup rather than named here.

**Targeted builds only — never `--workspace`** (`COORDINATION §12`; operator hard
rule). Use `scripts/ken-cargo` scoped with `-p ken-runtime`. Check disk headroom
before taking the build lock, and do not reclaim scratch while another seat holds
the build turn.

## 10. Sizing

**`M`, provisional.** The `L` is withdrawn: it was sized against inventing the
continuation machinery, which has landed.

The provisional part is real. `D1` is a genuine open question, and hard stops 1
and 2 both make the node bigger. **Post the `D1` outcome as its own checkpoint
before starting `D2`** — that is the point at which the Steward re-sizes if
needed.

**`M` re-affirmed 2026-08-08 with `D0`-`D2` complete and the `D3` reconciliation
applied.** `D1` came back asymmetric (position B closed for free **on its exact
witness** — the class-wide reading is withdrawn, see `D2`), `D2` landed as a
narrow guard, and the implementer reports the `D3` production retirement as two
files and six sites, building clean. **The added `D3` work is the control sweep,
not more mechanism** — retiring or rewriting every control whose subject the
retirement removes. That is bounded by the existing suite and does not reopen
sizing.

> **THE RE-AFFIRMATION ABOVE IS SUPERSEDED FOR `D3`, 2026-08-08 — hard stop 4.**
> The `D3` retirement exposed six previously-green semantic controls that fail
> closed across five refusal boundaries, all through position B's shape. That
> repair is **not** in this node: it is [[RT-LEXICAL-RECURSOR-CONSUMERS]], which
> now holds a `blocks` edge onto this one.
>
> **`D0`-`D2` is unaffected and lands on its own** (approved object `8efdfdb3`,
> Decision `dec_6nsrbyw1wjpb`). `D3` resumes from a `main` that already carries
> the repair, and must then prove the same six rows green **without** any
> exclusion hook and **without a single new `#[ignore]`.**
>
> The sizing statement that did not survive is *"the added `D3` work is the
> control sweep, not more mechanism."* There is more mechanism — it is simply
> owned by another node.

**What would:** if the sweep finds a control that another node's *merged*
evidence depends on and that cannot be honestly rewritten inside this node, that
is a Steward call on where the evidence lives — route it rather than absorbing
it. The implementer was right to refuse that call on `2026-08-08`; the frame
now answers it, so it should not need routing twice.

Checkpoints, exact SHA posted at each:

1. `D0` re-census and pinned base.
2. `D1` activation probe, both positions — **including a "closes for free"
   result, which is the best outcome and must not be quietly folded into `D2`.**
3. `D2` consumer port, per class that needs one.
4. `D3` joint retirement.

Target roughly an hour per implementer turn: a releasable increment or a genuine
hard stop. Both are good outcomes; neither-of-those is the bad one.
