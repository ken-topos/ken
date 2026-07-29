---
id: RT-FNSPLIT-RECUR-PORT
title: "emission-port completion — the governed nested-bracket family (recursive ComputationalMatch + trap arms) must select FunctionizedUnits, so RT-SCALE-B can measure the completed population"
status: merged
owner: runtime
size: XL
gate: none
depends_on: [RT-FNSPLIT-B2F]
blocks: [RT-SCALE-B]
github: null
origin: RT-SCALE-B frame-unsatisfied hard stop #13, raised by runtime-implementer (evt_1wszmnre17w1e) and ruled structurally by the Architect (evt_14eq3v2g0v1hm, stop order evt_4yg9kzq46sn4b). Escalated to the Steward by runtime-leader (evt_3fmxnax486b52) because the repair is outside RT-SCALE-B and RT-FNSPLIT-B2F is merged. Steward-filed 2026-07-28 (agents cannot create tracked work per COORDINATION §2); Steward ruling evt_37fwa49tk6dhj.
---

> ## ⛔⛔ WHY THIS NODE EXISTS — `RT-SCALE-B` CANNOT LAWFULLY MEASURE WITHOUT IT
>
> **`RT-SCALE-B` is the node that decides whether the whole `RT-NATIVE-FNSPLIT`
> effort worked.** It is paused clean at `466a9fa7` with no edits, and it stays
> paused until this lands.
>
> **The determination is structural, not empirical** (Architect
> `evt_14eq3v2g0v1hm`). On `origin/main = 466a9fa7`:
>
> 1. `planning/static_transition.rs::nested_resource_bracket` builds each bracket
>    as a `RuntimeExpr::ComputationalMatch` with `recursive_positions: vec![0]`,
>    carrying trap arms.
> 2. `lowering/core.rs::requires_recursive_descent_authority` returns true for any
>    non-empty `recursive_positions`, **and independently** for any
>    `RuntimeExpr::Trap` reached in the source tree.
> 3. `select_body_emission_authority` therefore selects
>    `BodyEmissionAuthority::RecursiveDescent`.
> 4. `compile_expr_into_module` declares and defines `UnitBundle` bodies **only**
>    for `FunctionizedUnits`; `RecursiveDescent` executes the retained monolithic
>    `compiler.lower_expr` root instead.
>
> ⇒ ⭐ **`B2F`'s functionized population is never produced for the family the
> scaling gate names.** Both remaining options are unlawful as `D1`/`D2`:
> reporting retained-authority metrics as the *completed representation* is a
> **fabricated population**, and reporting missing functionized metrics as
> `could_not_determine` is an **`AC-B1` failure**. ✅ Runtime refused both.

> ## ⛔ `RT-FNSPLIT-B2F` IS NOT DEFECTIVE — DO NOT REOPEN IT
>
> **The residual was approved as part of `B2F`.** Decision `dec_5y5ekba06d1t7`
> says so in terms:
>
> > *"the closed `RecursiveDescent` residual covers recursive computational
> > matches, `Match`-with-`Call` scrutinees, traps, seed `Closure` calls, and
> > top-level transparent closure declarations."*
>
> ⇒ **`B2F` landed exactly its approved scope.** The gap is that `RT-SCALE-B`'s
> prescribed discriminator family lives **inside that approved residual** — two
> nodes each individually correct and **jointly inconsistent**.
>
> ⚠ **That inconsistency is the Steward's**, not the ring's: `RT-SCALE-B` was
> released as shovel-ready without checking its prescribed family against the
> authority `B2F` was scoped to deliver. ⛔ Recorded so no one re-derives it as a
> `B2F` defect and reopens a merged node.

> ## ⛔ THE BENCHMARK IS NOT BEING AMENDED — this was a real fork, decided
>
> ⚠⚠ **AMENDED BY THE `#14` RULING — read the ruling block below with this
> one.** The *semantics* are still not being amended and the fork below is still
> decided the same way. ⛔ But the family's **spelling** is now corrected: the
> one-operand `BufferFreeze(Var(0))` was malformed and `D7` replaces it. ⭐ The
> line that follows — *"deleting recursion or traps would be a different
> benchmark"* — is untouched and still binds.
>
> `runtime-leader` correctly surfaced two paths (`evt_3fmxnax486b52`): repair the
> port, **or** obtain a scope amendment changing the benchmark. ⭐ **The port
> side is taken.**
>
> **The Architect already ruled the substitution inadequate on the merits:**
>
> > *"A family made functionizable merely by deleting recursion or traps would be
> > a different benchmark and could not answer this frame's question."*
>
> ⭐⭐ **And there is a harder reason than measurement fidelity.** This chain
> exists because `PX8-ERRID-ALLOC` died on `Cranelift backend failure: Code for
> function is too large`. ⇒ **If recursive `ComputationalMatch` and trap arms
> still lower through the retained monolithic root, the wall is not cleared for
> those constructs — regardless of what we choose to measure.** Amending the
> benchmark would move the *thermometer*, ⛔ not the *fever*.
>
> ✅ **MEASURED AND AFFIRMATIVE — `S0`, 2026-07-28.** The `PX8-ERRID-ALLOC`
> failing fixture **does** route through `RecursiveDescent`:
> `authority=RecursiveDescent function=ken_nc23_entrypoint`, measured on clean
> `b6bac1a8` with an environment-gated probe after
> `select_body_emission_authority`, then reversed. ⇒ **The retained authority is
> itself on the critical path to the Cranelift wall — `D1`/`D2` are load-bearing
> for `PX8` itself, not only for the measurement. No re-scope.**

> ## ✅ THIS IS THE FRAME'S OWN PRESCRIBED HANDLING, not a Steward invention
>
> `docs/program/wp/RT-SCALE-B-emission-scaling-verdict.md` armed this exact
> symptom at release:
>
> > *"A new entry that reduces to that same predicate means the **port** is
> > incomplete — it routes back to the emission-port work rather than being ruled
> > here, and ⛔ it does **not** become a verdict of super-linearity."*
>
> ⇒ ⭐ **The armed symptom fired and the frame's prescribed route is this node.**

> ## ⛔⛔ HARD STOP `#14` — 2026-07-28, after `S1`. SCOPE RULED: **THIS FOLDS IN.
> ## IT IS NOT A NEW NODE.**
>
> **Reported by `runtime-implementer` (`evt_1qfevzbf089rj`), escalated by
> `runtime-leader` (`evt_3ncs7m7syn7ds`).** Branch clean, no commit, no
> prohibited substitution, no `B2F` surface touched. ✅ **Stopping there was
> right.**
>
> ### What `S1` proved before it stopped — ⭐ this part is a WIN, keep it
>
> At depth 3 the governed source selects `FunctionizedUnits`, the recursive
> lexical-closure position resolves to its planner-declared body unit and emits
> a **direct unit call**, and re-entry closes as a **static-origin-keyed CFG
> backedge** rather than an inline re-lowering. ⇒ **`AC-2`'s growth property is
> demonstrated.** The stop is downstream of it.
>
> ### The stop
>
> ```text
> BoundaryCarrier: a host-effect operand is a specialized-only surface and a
> carried boundary word has no compile-time template for it to read; the
> carrier's ruled route is an emitted helper call
> ```
>
> `lower_process_host_effect` lowers its operands and then calls
> `specialized_env_at(&lowered, "a host-effect operand")`. A functionized
> closure-body unit necessarily receives `buffer` through the `B2R` frame as
> **`Carried`**, so the governed `BufferFreeze(Var(0))` cannot cross that
> boundary. ⚠ **Independent of `D1` recursive positions and `D2` trap arms** —
> it is a third, unframed port.
>
> ### ⭐ WHY IT FOLDS RATHER THAN FORKING
>
> **`AC-1` requires the family to select `FunctionizedUnits` *and produce a
> complete `UnitBundle`*; `AC-6` requires `RT-SCALE-B` to actually collect every
> metric.** Compilation halting at `BoundaryCarrier` means no complete bundle
> and no collectable measurement. ⇒ ⛔ **This node's own acceptance criteria are
> undischargeable without the carried host-effect operand port.** A new node
> would put a hard prerequisite *in front of* an `active` node on the critical
> path and lengthen it for nothing.
>
> ⚠ **The node is now larger than the `L` it was priced at.** ⇒ **Ruled and
> re-sized: see the ruling block below.** `D6` and `D7` are added, the size is
> `XL`, and `S6`/`S7` are appended to the frame's slicing order.
>
> ### ⛔ THE OPERAND-SHAPE MISMATCH IS THE ARCHITECT'S, NOT THE RING'S
>
> The governed constructor spells `BufferFreeze` with **one** operand (`Var(0)`)
> while the production wire encoder requires **four** — buffer, start, length,
> span-origin.
>
> ⚠ **`PROHIBITION 1` does not literally forbid reconciling that spelling** — it
> forbids deleting recursion, dropping trap arms, and substituting a non-bracket
> synthetic, none of which is this. ⛔ **But do not infer permission from that.**
> Whether changing the governed constructor's operand shape preserves the
> benchmark's meaning is *precisely* the question the Architect answered for
> recursion and traps (*"a different benchmark could not answer this frame's
> question"*), and it must be answered explicitly for this shape too.
>
> ⇒ ⛔ **Runtime does not reconcile the governed source's operand shape on its
> own reading of `PROHIBITION 1`.** ✅ **RULED** at `evt_3629v1gy7fwqq` — and
> the answer went the other way: the **encoder is right and stays strict at
> four**; the one-operand *source* is malformed and is replaced. See `D7`.
>
> ### Bookkeeping
>
> **Count of record 13 → 14.** ⛔ No research pull fires at `#14`; ⚠ **`#15` is
> armed, so the next stop pulls research.** Symptom inventory `ENTRIES` 4 → 5 is
> owed by the **Architect**, and entry 5 is the last before the 6th, which must
> answer the shared-predicate question.
>
> ⚠ **The `§5a` line on `main` read `11` when this stop was raised — two stops
> stale.** The implementer read the authoritative line exactly as instructed and
> was right to flag the disagreement. Repaired in the same publish as this block.

> ## ⚖️⚖️ ARCHITECT RULING ON `#14` — `evt_3629v1gy7fwqq`, 2026-07-28
>
> **Outcome: the port is admitted, but NARROWLY — and the malformed side turned
> out to be the governed SOURCE, not the encoder.**
>
> ⛔⛔ **READ THIS BEFORE `PROHIBITION 1`. IT AMENDS IT.** The ruling requires
> changing the governed family, which `PROHIBITION 1` reads as forbidden. ⇒ ⭐
> **From this ruling forward, "the unmodified governed family" means the
> corrected canonical helper in `D7`. The malformed one-operand helper is
> RETIRED.** Every other clause of `PROHIBITION 1` stands untouched: ⛔ deleting
> recursion, dropping trap arms, and substituting a non-bracket synthetic remain
> forbidden and still fail `AC-1`.
>
> ⚠ **Why this is a correction and not a substitution** (the Architect's
> grounding, and the thing that keeps `PROHIBITION 1` coherent): the one-operand
> spelling is a **planning-only raw `RuntimeExpr` that could not state its own
> named contract.** After the recursive `Let`, `Var(0)` denotes the *recursive
> call result*, not the allocated buffer — and one operand cannot state start,
> length, or the span provenance `PX8-SPAN-PROV` requires. ⇒ It was never a
> well-formed instance of the benchmark it claims to be.
>
> ### `D6` — the representation boundary
>
> `lower_process_host_effect` must **retain `LoweringOperand`s until the host
> operation and operand seat are known.** ⛔ It must not bulk-call
> `specialized_env_at` before reaching the `BufferFreeze` branch.
>
> **For `BufferFreeze` only** — seats 0 (buffer) and 3 (span-origin) are
> phase-bearing:
>
> | seat | admitted |
> |---|---|
> | 0, 3 | `Specialized(Lowered::ResourceToken { value })` — the existing specialized value |
> | 0, 3 | `Carried(word)` — **only** after validating the existing `InvocationBorrowed` / `BorrowedOpaque` carrier representation, then projecting through the **existing** emitted carrier-scalar helper |
> | 0, 3 | ⛔ every other specialized shape and every other represented shape **fails closed** |
> | 1, 2 | remain specialized `Int` operands, continuing through the existing narrowing path |
>
> ⭐ **The checked host operation plus operand position supplies the semantic
> resource-token role; the carrier validation supplies the runtime
> representation class.** ⛔ **Do not rederive a source type dynamically.**
>
> ### ⛔ WHAT THE LICENSE DOES NOT PERMIT — a closed list
>
> ⛔ Treating every `Carried` host operand as a scalar · reconstructing or
> fabricating `Lowered::ResourceToken` · adding a `Lowered` variant that
> contains a carrier · minting a new carrier tag, class, identity, ABI field,
> service, envelope field, or ingress lane · synthesizing bounds or provenance
> inside the wire encoder · widening **any other** host operation without an
> independently demonstrated carried seat.
>
> ⭐ This **composes** the already-landed `B2R` carrier with the existing
> capability / borrowed-opaque projection precedent. ✅ It changes **no** `B2F`
> ABI / services / envelope / ingress surface — `AC-5` is unaffected.
>
> ### `D7` — the corrected canonical governed family
>
> The encoder stays strict at four operands:
> `BufferFreeze(buffer, start, length, span_origin)`.
>
> ```text
> bracket(0) = unit
>
> bracket(n) =
>   BufferAllocate(1) match
>     Err -> trap
>     Ok(buffer) ->
>       ComputationalMatch
>         Scope(
>           lexical closure λ buffer.
>             let _ = bracket(n - 1) in
>             BufferFreeze(buffer, 0, 1, buffer) match
>               Err -> trap
>               Ok(_) -> unit
>         )
>       case Scope(ih) -> ih(buffer)
>       default -> trap
> ```
>
> **The allocation result is the induction-hypothesis argument.** Inside the
> closure both resource seats resolve to **that closure parameter**, not to the
> `Let` result: seats 0 and 3 carry the **same acquisition token**, and seats 1
> and 2 are exactly `Int(0)` and `Int(1)`.
>
> ⛔⛔ **IMPLEMENT BY SEMANTIC BINDER ROLES, THEN AUDIT THE GENERATED INDICES.**
> ⛔ **Do not copy the raw `Var` indices out of the ruling** — they are
> illustrative, and copying them is how the current defect got here.
>
> ⭐ **Still load-bearing and unchanged:** the recursive `ComputationalMatch`,
> `recursive_positions = [0]`, **all** trap arms, the `n=3..7` family, and LIFO
> bracket behavior.

> ## ⛔⛔ HARD STOP `#15` — 2026-07-28, after `S6/D7`. ⚠ **THE ARMED RESEARCH
> ## CONSULT HAS FIRED.** Scope: **it folds in. Still not a new node.**
>
> **Reported by `runtime-implementer` (`evt_70nwtht1kf0aq`), escalated by
> `runtime-leader` (`evt_7wb1y9a8rdrza`), Steward disposition
> `evt_43c6tspcx0xg3`.** ✅ `S6/D7` committed alone at `f1306864`; `S7/D6` held
> as a one-file patch. No prohibited change made.
>
> ### What `S6`/`D7` proved before it stopped — ⭐ `AC-7` did its job
>
> Structural `n=3..7` pin **green**. The span-origin-role evasion **red at the
> intended role assertion** and was restored. ⭐ The generated case scope was
> audited against the real three-role inventory — allocated buffer, original
> `Scope` argument, induction hypothesis — so `ih(buffer)` uses the **allocation
> result**, not the original closure. ⇒ **The index audit caught exactly what
> copying raw `Var` indices out of the ruling would have missed.**
>
> ### The stop
>
> ```text
> BoundaryCarrier: Match merges native scalar lanes and has no carried lane; a
> boundary word cannot cross it until that join carries the phase
> ```
>
> With the corrected family **and** `D6`'s `BufferFreeze` seats open, the
> governed compile reaches **past both carried resource operands** and then
> refuses at `specialized_join_arm("Match")`: the corrected closure body returns
> the recursive bracket through an ordinary `Match`, and after the host-seat port
> that result is phase-bearing. ⛔ **`D6`'s closed license does not admit a
> carried ordinary `Match` result join** — opening one is a new executable
> representation mechanism. ✅ Runtime made no such change.
>
> ### ⭐ WHY IT FOLDS — re-derived, not cited
>
> **Identical to `#14` and deliberately re-argued rather than inherited.**
> `AC-1` requires a complete `UnitBundle`; `AC-6` requires the metrics to be
> collectable. Compilation still halts — now at the `Match` join instead of the
> host-effect operand — so **both remain undischargeable inside this node's own
> acceptance criteria.** ⛔ A new node would again insert a hard prerequisite in
> front of an `active` critical-path node.
>
> ⛔ **No `D8` and no re-pricing until the Architect rules.** The mechanism
> determines the size, exactly as at `#14`.
>
> ### ⛔⛔ THE PATTERN MATTERS MORE THAN THIS STOP
>
> **`#14` and `#15` are the same shape — a specialized-only surface meeting a
> carried boundary word.** ⇒ ⚠ **Each narrowly-ruled seat opens the path far
> enough to reveal the next one**, and this node has now been re-priced twice on
> one underlying shape.
>
> ⇒ ⛔ **The consult question is not "may we open a carried `Match` join
> lane?"** It is **whether one general carried-phase representation boundary
> subsumes both, and how many instances remain before a complete `UnitBundle`.**
> ⭐ The named §5a-ii predicate is `executable-boundary closure`, and **entry 6
> is the mandated predicate check** — it falls on exactly the right entry.
> ⛔ §5a-ii: the Architect names it, not the Steward.
>
> ### ✅✅ RESOLVED — it folds as `D8`. ⚠ The Steward's fork ruling was REVERSED
>
> ⚠ **Steward `evt_5bbpc6wyj31zb` ruled this a fork and filed a node
> `RT-FNSPLIT-JOIN-CLOSURE`. That ruling is withdrawn** and the node is not
> filed. **The Architect's `#15` ruling (`evt_4a34wkyk6bmm5`) sequences the join
> closure *inside* this node — after `D6`, before the remaining selector/trap
> slices — and that is correct on the merits.**
>
> ⛔ **The reason, stated so it is not re-derived:** the join work happens on
> **this node's branch, between this node's slices, in one merge.** A separate
> node would sit in `depends_on` — which encodes *"merged before"* — while never
> merging independently. ⇒ ⭐ **A node boundary that does not correspond to a
> merge boundary is a label, not a node.**
>
> ⚠ **The concern that drove the fork is real and is answered differently:** an
> `XL` node carrying four mechanisms has an unreviewable acceptance surface.
> ⇒ **The answer is deliverable and AC numbering** — `D8` and `AC-11`–`AC-16`
> below — ⛔ not a graph split.
>
> **Three independent lines converged on the mechanism**, which is why the
> ruling was fast:
>
> | source | finding |
> |---|---|
> | research advisory (`evt_3ywrfbxn9gm6t`) | the general boundary is **already ruled** at `C1 §2h`, which names *branch/join forwarding* explicitly and forbids a reachability whitelist |
> | Architect entry 6 (`cbd5848b`) | entries 4/5/6 share `executable-boundary closure`; *"stop ruling seat-by-seat"* |
> | Steward scope review | a lowering-wide join convention is not a seat, and a fourth mechanism makes this node's ACs unreviewable |
>
> ⭐⭐ **`D6` WAS RIGHT AND STAYS RIGHT.** The advisory separates two **roles**:
> a **semantic consumer** seat (this node's `D6`) — where per-seat admission
> **converges**, because the consumer assigns meaning to the word — and a
> **forwarding join** — where it **diverges**, because each newly reachable join
> re-reveals the same missing result convention. ⇒ ⛔ The error would have been
> applying `D6`'s technique to joins.
>
> ⚠ **`AC-2` remains a preservation obligation across this too** — the corrected
> family and the eventual join convention both sit under the mutation that
> proves recursive positions lower as declared unit calls.

> ## ⚖️⚖️ ARCHITECT RULING ON `#15` — `evt_4a34wkyk6bmm5`, 2026-07-28. THIS IS `D8`.
>
> **Adopt result-directed join signature conversion across the COMPLETE pending
> join family.** ⛔ **Do not add a one-off carried `Match` arm, and do not widen
> host consumers generically.**
>
> ⭐ **One general `executable-boundary closure`, but its ROLES are not
> interchangeable.** `BufferFreeze` is a **semantic consumer** and keeps `D6`'s
> narrow operation+seat admission. An ordinary `Match` is a **forwarding join**
> and must use one statically selected result representation shared by every
> continuing predecessor.
>
> ### 1. A pre-emission phase plan — one entry per join occurrence
>
> Before lowering emits **any** arm, semantic planning assigns each source join
> occurrence, **keyed by its `StaticOriginId`**, exactly one closed contract:
>
> ```text
> JoinResultRepresentation =
>   NativeScalarPair
>   | CarrierWord
> ```
>
> ⚠ **The names are not ruled; the two-way shape is.**
>
> ⛔ **This is a COMPILE-TIME plan, never a runtime `{phase, tag, payload}`
> sum.** It is derived from the semantic occurrence/owner graph that already
> owns unit boundaries and static-body calls, using a **monotone phase
> summary**:
>
> ```text
> SpecializedOnly < CarrierRequired
> ```
>
> **Seed `CarrierRequired`** from declared-unit call results, carried ABI
> inputs/captures, carrier projections, and existing one-way carrier producers.
> **Propagate** through environment insertion, recursive calls, forwarding
> results, and joins.
>
> - A join is **`CarrierWord`** when **any** non-trapping predecessor may carry,
>   **or** its result flows through a carrier-required forwarding edge.
> - It is **`NativeScalarPair`** only when **every** continuing predecessor and
>   result edge is **structurally proven** specialized-only **inside the same
>   unit**.
> - **Trap predecessors contribute no result edge.** ⭐ A join with no continuing
>   predecessor **emits no merge**.
>
> **The analysis is total over the semantic source variants, exhaustive with no
> wildcard, and validated before lowering.** ⇒ A bijection: every source join
> occurrence has **exactly one** plan entry, every plan entry names an
> **existing** occurrence, and lowering consumes the entry for that origin
> **exactly once**. ⛔ **`construct: &'static str` stays a diagnostic label and
> is NEVER an identity key.**
>
> ⛔⛔ **`select_terminal_result_origins` may supply a result-position SEED, but
> it is NOT the closure.** ⚠ **Using that set alone would be exactly the
> manually maintained reachability boundary `C1 §2h` rejects.** The
> authoritative result is the transitive phase plan produced beside the semantic
> owner/occurrence graph.
>
> ### 2. Lowering consumes a typed token BEFORE creating the merge
>
> | planned | emission |
> |---|---|
> | `NativeScalarPair` | retain the existing two-`i64` `(tag, payload)` merge and the existing scalar-kind agreement checks. ⛔ A `Carried` predecessor here is an internal plan/implementation disagreement and **fails the function definition** |
> | `CarrierWord` | create **exactly one** `i64` block parameter. `Carried(word)` passes **unchanged**; a supported `Specialized(lowered)` crosses **exactly once** through the existing `transfer_into_carrier` / `carried_join_arm` one-way producer. ⭐ The merged result is `LoweringOperand::Carried` |
> | `Specialized(Trap)` | call the existing `seal_source_trap_branch`; **emit no jump to the merge.** ⭐ Trap is a **control-flow edge property**, not a carrier materialization case |
>
> ⭐⭐ **The held `lower_dynamic_host_result_match` must select `CarrierWord`
> EVEN THOUGH ITS SCRUTINEE IS SPECIALIZED.**
>
> ⛔ **Forbidden selectors, all four:** scrutinee phase · first-arm phase · case
> order · a runtime *"if any arm is carried"* test. ⚠ **None of them declares
> the merge signature before arm emission**, which is the whole property.
>
> ⛔ An unsupported specialized→carrier materialization **aborts the function
> before `define_function` / `UnitBundle` publication** and must not leave a
> published partial CFG. ⛔ **No `Carried → Lowered` conversion.**
>
> ### 3. ⛔ CLOSE THE FAMILY, NOT THIS CALL SITE
>
> The held `a5f35d8d` census is the **review population**:
>
> | helper | direct lowering callers |
> |---|---|
> | `merge_branch_value` | 4 |
> | `merge_scalar_branch` | 10 |
> | `merge_planned_scalar_branch` | 1 |
> | **total** | **15** |
>
> ⛔⛔ **This is a STATIC REVIEW POPULATION, NOT a claim that the governed input
> reaches 15 more stops.** ⭐ Only `lower_dynamic_host_result_match` is presently
> **proven reachable**.
>
> ⛔ **No helper may retain an unplanned "choose after lowering the arms"
> route.** ⭐ **A new join caller must be UNABLE TO COMPILE without supplying and
> consuming the typed plan token** — ⛔ do not preserve a catch-all legacy helper
> that bypasses it.
>
> ### 4. ⛔ What stays narrow and what stays frozen
>
> **`D6` is unchanged, exactly as ruled:** only `BufferFreeze` seats 0 and 3
> admit the validated carried `InvocationBorrowed` / `BorrowedOpaque` resource
> representation; seats 1 and 2 stay specialized `Int`; every other host
> operation/seat stays exhaustively classified and **fails closed** unless it
> has its own emitted semantic-consumer route.
>
> ⛔ **This ruling adds NONE of:** a carrier tag, class, identity, metadata
> field, runtime phase discriminant, reverse decode, ABI field, service,
> envelope field, ingress lane, or parallel ownership lane. ⭐
> **`CarriedBoundaryWord` stays one SSA word**, and the external two-pointer
> generated-function ABI, two-field services record, and two-field call-frame
> envelope are **byte-for-byte unchanged**.

## Objective

Make the **same governed** `nested_resource_bracket` family — recursive
`ComputationalMatch` with trap arms, unmodified — select
`BodyEmissionAuthority::FunctionizedUnits` and produce a complete functionized
unit population, so `RT-SCALE-B` can collect `D1`–`D3`/`D5` evidence on the
representation whose growth the chain exists to bound.

⛔ **Not in scope:** deleting recursion or traps from the family; introducing a
non-bracket synthetic; changing what `RT-SCALE-B` measures; reopening `B2F`'s
landed envelope, ABI, services record, or static ingress declaration.

## Deliverables

| id | deliverable |
|---|---|
| `D1` | functionized emission for `ComputationalMatch` with non-empty `recursive_positions` — recursive positions become declared unit calls, not an inlined re-lowering |
| `D2` | functionized emission for `RuntimeExpr::Trap` arms, so a reached `Trap` no longer forces the whole body to the retained authority |
| `D3` | `requires_recursive_descent_authority` narrowed to what genuinely remains unportable, with each removed condition individually justified — ⛔ **the selector stays closed, exhaustive and fail-closed** |
| `D4` | the governed `nested_resource_bracket` family selects `FunctionizedUnits` at every `n` in `3..7`, with the existing selector control updated to assert the new selection **positively** |
| `D5` | the retained-authority residual re-stated: whatever still selects `RecursiveDescent` after this node, named explicitly and closed |
| `D6` | **the narrow carried resource-token seat** — `lower_process_host_effect` retains `LoweringOperand`s until operation + seat are known, and admits a validated `Carried` carrier at `BufferFreeze` seats 0/3 only, failing closed on every other shape (ruling `evt_3629v1gy7fwqq`) |
| `D7` | **the corrected canonical governed family** — the malformed one-operand `BufferFreeze(Var(0))` replaced by the four-seat operational bracket, built from semantic binder roles, with recursion / `recursive_positions = [0]` / trap arms / `n=3..7` / LIFO all retained |
| `D8` | **result-directed join signature conversion, across the WHOLE pending join family** — a pre-emission per-join phase plan keyed by `StaticOriginId` selecting one closed result contract, and a typed plan token that lowering consumes before creating the merge (ruling `evt_4a34wkyk6bmm5`). ⛔ **Not** a one-off carried `Match` arm |

## Acceptance criteria

- **`AC-1`** — the unmodified `nested_resource_bracket(n)` for every `n` in
  `3..7` selects `FunctionizedUnits` and produces a complete `UnitBundle`.
  ⛔ A control that mutates the family to make it pass does not discharge this.
- **`AC-2`** — recursive positions lower as **declared unit calls**. ⛔ A control
  must red if a recursive position is re-lowered inline into the caller's body,
  since that is the growth this chain exists to bound.
- **`AC-3`** — a `Trap` arm inside an otherwise functionizable body does not by
  itself force `RecursiveDescent`. ⛔ Needs a positive control: the same body
  with and without the trap arm, both functionized.
- **`AC-4`** — the selector remains **closed and fail-closed**: an unhandled or
  unknown source shape still selects the retained authority rather than being
  admitted by default. ⛔ **`D3` narrows the condition; it must not invert the
  default.**
- **`AC-5`** — `B2F`'s landed mechanism is unchanged: two-pointer ABI, two-field
  services record, two-field call-frame envelope, three-field root ingress
  consumed only by the public adapter, role-keyed static ingress. ⛔ Inventory
  controls must still pass unmodified.
- **`AC-6`** — `RT-SCALE-B`'s harness can produce every `D2` metric for every
  `n` in `3..7` on this family. ⭐ **This is the node's real exit condition** —
  the point is not that the selector flipped, but that the measurement is now
  collectable.

### Added by the `#14` ruling — `evt_3629v1gy7fwqq`

⛔ **These are the Architect's required discriminators, and they bind exactly as
`AC-1`–`AC-6` do.**

- **`AC-7`** — a **structural fixture control** pins the corrected family: the
  induction-hypothesis argument **is** the allocation result; `BufferFreeze`
  has exactly **four** operands; seats 0 and 3 are the **same closure
  parameter**; seats 1 and 2 are literals `0` and `1`; recursion and **every**
  trap arm are retained. ⭐ This is what stops `D7` from drifting into the
  substitution `PROHIBITION 1` still forbids.
- **`AC-8`** — mutating **either** `BufferFreeze` resource seat back to
  specialized-only **must red** on the governed carried route. ⛔ A control that
  only proves the carried route works cannot distinguish a live port from a
  path that never needed one.
- **`AC-9`** — the new seat **fails closed**: a carried **wrong-class /
  non-`BorrowedOpaque`** value must fail **before any host request is issued**,
  and a **carried `start` or `length`** must fail closed. ⭐ Same discipline
  `AC-4` imposes on the selector, now on the operand port.
- **`AC-10`** — the existing **`PX8-SPAN-PROV` same-shape / two-buffer
  discriminator stays green**, and substituting a **distinct span-origin token
  must red**. ⛔ **The encoder must not derive seat 3 from seat 0** — if it
  does, seat 3 is decoration and the provenance discriminator is vacuous.

⭐ **Three of the ruling's seven discriminators are already ACs and are not
restated:** `n=3..7` selecting `FunctionizedUnits` with complete `UnitBundle`s
is **`AC-1`**; `S1`'s direct-unit-call / static-origin-backedge mutation
remaining discriminating is **`AC-2`**; the `B2F` inventory controls remaining
unchanged is **`AC-5`**. ⚠ `AC-2` is now a **preservation** obligation — `D7`
rewrites the family `S1` proved it on, so the mutation must be re-run against
the corrected source, not assumed to carry.

### Added by the `#15` ruling — `evt_4a34wkyk6bmm5`, the `D8` discriminators

⛔ **Six required discriminators, and they bind exactly as `AC-1`–`AC-10` do.**

- **`AC-11` — static contract, ORDER-INDEPENDENT.** On the **same specialized
  scrutinee**: one arm returns a declared-unit **carried** result, one returns a
  supported **specialized** result. ⭐ The plan must read `CarrierWord`
  **before arm emission**. **Swapping arm order leaves the plan and the emitted
  signature unchanged.** ⛔ Mutating the selector to scrutinee-based or
  first-arm-based **must red at the plan assertion**.
- **`AC-12` — uniform predecessor conversion.** The carried arm reaches the
  merge **unchanged**; the specialized sibling invokes the existing producer
  **exactly once**. ⛔ **Both mutations must red independently** — re-producing
  the carried word, and bypassing production for the specialized value.
- **`AC-13` — trap topology.** A trapping sibling seals/returns and contributes
  **no merge predecessor**. ⛔ Mutating it to jump to the merge must red on the
  predecessor/CFG assertion. ⭐ An **all-trap join creates no merge block**.
- **`AC-14` — closed population.** A structural census pins the **three helper
  families and the 4 / 10 / 1 direct-caller population** at the held checkpoint.
  ⛔ Removing a required typed-plan consumption, **or adding a caller through an
  unplanned helper**, must fail compilation or a structural control.
- **`AC-15` — fail before publication.** An unsupported specialized→carrier
  materialization returns the producer's own failure and publishes **no
  function and no partial `UnitBundle`**.
- **`AC-16` — governed causal route.** The corrected `n=3..7` family **reaches
  this carrier-result join** and produces complete `UnitBundle`s, ⭐ **while
  preserving every existing `AC-1`–`AC-10` discriminator** — the `D6` seat
  mutations, `PX8-SPAN-PROV`, the direct-unit-call / static-origin backedge, the
  selector's fail-closed default, and the unchanged `B2F` ABI inventories.

## ✅ CLOSED — the open question that sized this node

> **Does the `PX8-ERRID-ALLOC` failing fixture route through
> `RecursiveDescent`?** ✅ **YES.** Measured at `S0` on 2026-07-28 (above).
> ⇒ The retained authority is on the critical path to the Cranelift wall and
> `D1`/`D2` are load-bearing for `PX8` itself. **No re-scope; the node stands.**
>
> ⚠ **Size moved for a different reason.** `L` → **`XL`** comes from the `#14`
> ruling adding `D6` and `D7`, not from this measurement. ⭐ `XL` is a new value
> in this corpus and means exactly one thing: **beyond `L`, and it would have
> been split had it not already been `active` on the critical path.**
>
> ⭐ **Size reconciliation after `#15` (owed by the Architect's ruling): it stays
> `XL`.** `D8` adds a pre-emission analysis pass and a lowering-wide join
> refactor across 3 helper families — ⚠ genuinely large, and the reason the
> Steward first ruled a fork. ⛔ **But `XL` already means "beyond `L`, unsplit
> only because it is `active` on the critical path", and that statement is
> unchanged.** ⇒ Inventing a further tier would encode nothing a reader can act
> on. **The reviewability answer is `D6`/`D7`/`D8` and `AC-11`–`AC-16` being
> separately numbered and separately mutable, not a bigger label.**

## Bookkeeping

- **Hard-stop count of record: 15.** ⚠⚠ **`#15` HAS FIRED — the armed research
  consult is OPEN.** The armed multiples are `#15`, `#18`, `#21`; the next is
  `#18`. The authoritative counter is
  `docs/program/issues/RT-NATIVE-FNSPLIT.md`'s **`COUNT OF RECORD` block at the
  head of `§5a`**, which wins on any disagreement — ⛔ read it at the point of a
  stop, never a count transcribed into a frame (**including this line**).
- **Symptom inventory: `ENTRIES` 4 → 5** for `#14`, appended by the
  **Architect** at commit `9db7991f` **before** it ruled, as the protocol
  requires. ⚠⚠ **`ENTRIES` 5 → 6 is now owed for `#15` — and entry 6 IS `NEXT
  PREDICATE CHECK`.** ⇒ ⛔ **The Architect must answer whether the entries share
  a predicate BEFORE the `#15` ruling issues.** ⭐ The predicate it already
  named is `executable-boundary closure` (`evt_55bzwnhjpwjrs`); ⛔ §5a-ii —
  naming it is the Architect's, never the Steward's.
- ⛔ **§5a-ii: the shared predicate is the Architect's to name**, and it named it
  at `evt_55bzwnhjpwjrs`. The Steward does not restate it.

## Standing constraints

⛔ **Targeted `scripts/ken-cargo` only — never `--workspace`.** Workspace-green
and `--locked` mean **green in CI** (`agent/COORDINATION.md` §12).
⚠ The C3 archive leak is live: ~700 MB of `/tmp` per full `-p ken-runtime` run,
surfacing as an unrelated `No space left on device`. ⛔ Triage on the **error
production raised**, never on the test names.
