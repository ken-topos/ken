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
and enumerator** — it stays discriminating through the whole transition.

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
>   `LexicalCallArgumentRecursor` **closes for free** — the functionized lane
>   executes and yields the same decoded `RuntimeObservation`.
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

**Only after both executable positions are green**, retire the two residual
variants and their test-only selector hooks.

The lane itself is **not** yours. [[RT-DESCENT-RETIRE]] owns the
`RecursiveDescent` lane, its selector, its enum and its authority. Retiring
these two variants is what unblocks that node; performing its deletion here is
banned scope.

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
- **AC-6 — the exact-set enumerator stays discriminating through the
  transition**, including at intermediate commits.
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

Checkpoints, exact SHA posted at each:

1. `D0` re-census and pinned base.
2. `D1` activation probe, both positions — **including a "closes for free"
   result, which is the best outcome and must not be quietly folded into `D2`.**
3. `D2` consumer port, per class that needs one.
4. `D3` joint retirement.

Target roughly an hour per implementer turn: a releasable increment or a genuine
hard stop. Both are good outcomes; neither-of-those is the bad one.
