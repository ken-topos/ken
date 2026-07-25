---
id: RT-FNSPLIT-B2A-C
title: "plan↔lowering occurrence correspondence — transport the preallocated StaticOriginId to the site where it is out of scope"
status: merged
owner: runtime
size: L
gate: none
depends_on: [RT-FNSPLIT-B1R]
blocks: [RT-FNSPLIT-B2A-S]
github: 940
origin: Architect ruling evt_1jdh8pn8y96z on RT-NATIVE-FNSPLIT hard-stop #7 (2026-07-25), deciding on the runtime-implementer's TOTAL + injective census evt_4tqj93ctj24z2, which the Steward gated the ruling behind (evt_3ne9d2wkee0mx, evt_3qtyhp53v5g7x). Steward-filed; Steward owns the frame, scope, and AC/control placement.
---

> ## ✅ MERGED — `origin/main` = `2db29abe` (PR #940), 2026-07-25. RETROS IN (3/3).
>
> Content-verified on the **landed tree** (a squash lands under a new SHA):
> `struct PlannedExpr` ✅ · `occurrence: StaticOriginId` ✅ ·
> `declaration_occurrence_origin` ✅ · **`declaration_entry_origin` = 0 hits** ·
> seed API `children: &[StaticOriginId]` ✅ · attested `cranelift_backend.rs` OID
> == ledger row ⇒ gate green. Adversary notified `evt_1axyemv8kyj2y`.
> Retros: leader `evt_4w03bgrftv3nz`, QA `evt_4kq3s08q5hg6v`, implementer
> `evt_3g9vt4edd1ce4`.
>
> **Closes inventory entry 3 ONLY** — the recoverability vacancy, which was the
> *cause* of this chain's repeated defeats. ⛔ Entry 1 waits for
> `RT-FNSPLIT-B2A-S`; entry 2 for `RT-FNSPLIT-B2F`; `RT-NATIVE-FNSPLIT` stays
> `active`.
>
> ### ⭐ THE STEWARD LESSON FROM #8 — my census question was UNDER-SPECIFIED
>
> I gated the Architect's #7 ruling on **totality** ("every reachable occurrence
> has exactly one origin"). The census answered `TOTAL` and was **true**. But the
> mechanism needed **closure under parent→child reachability** — a different
> property. `ComputationalMatch` filed its occurrence on a *different node* from
> the entry its parent points at, so totality held while composition failed, and
> **#8 was predictable from the question I asked.**
> ⇒ **When gating a ruling on a measured property, write down the mechanism
> obligation it is meant to discharge and check the implication.** Rigour does not
> supply relevance. The cheap probe: **compose the accessor with itself, per
> variant, on a real instance** before threading it through N call sites.
>
> ## ⛔ (history) AMENDED AT HARD-STOP #8 — 2026-07-25. Still `active`; ring HELD pending routing.
>
> **Amended in place, not re-sliced** (Steward's call — the Architect ruled the
> mechanism at `evt_308azmr4cszd7` and explicitly left WP scope and the count to
> me). `plan_expr` returns a *scheduling entry*, which for `ComputationalMatch`
> is a **different node** from the one carrying the occurrence record — a
> **category error**, not an ordinal disagreement. New **D9** (the
> `PlannedExpr { entry, occurrence }` dual result), **D2**'s premise corrected,
> **D3** given a second axis, **AC-11–AC-17** added, **size M → L**.
>
> ★ **D3's probe found the defect and D3's vocabulary mis-named it** — the tell
> that the frame had one axis where it needed two.
>
> ## ✅ ACTIVE — kicked to the Runtime ring 2026-07-25 (`evt_3cka2t192dx10`)
>
> Full §2c gate ran: retros in from all three seats, quiescent, **both**
> contention axes checked (file axis disjoint from the in-flight doc publish;
> ledger axis found `cranelift_backend.rs` attested and the frame warns about
> it), compactions started in one pass and **each drop verified on the
> `Context compacted` marker** — implementer reached `ctx 0%` (its `ctx 33%`
> footer was the lagging read mid-compaction, not a failed drop). All three
> confirmed `Working` before this flip.
>
> ## ✅ FRAMED — 2026-07-25
>
> **The shovel-ready frame is
> `docs/program/wp/RT-FNSPLIT-B2A-C-plan-lowering-correspondence.md`.**
> Read that, not this file. It carries the six ruled mechanism requirements as
> D1–D8, the four negative-boundary pins N1–N4, ten ACs, and anchors pinned on
> `origin/main` = `70bd2c74`.
>
> ⭐ **D3 is the deliverable the ruling only implies: the per-variant ORDINAL
> CORRESPONDENCE between the planner's child ordinals and the lowering's
> traversal order.** D2's positional derivation silently presupposes it, nothing
> in the type system enforces it, and it is written down nowhere. ⭐ **D5 is the
> longest-lived: a coverage guard that must redden on a NEWLY ADDED expression
> field, not merely enumerate today's variants** — that is what stops entry 3
> recurring when `RuntimeExpr` grows.

> ## ⭐ WHY THIS NODE EXISTS — it is the CAUSE the chain kept working around
>
> `RT-NATIVE-FNSPLIT` symptom inventory **entry 3** is the vacancy that forced
> entries 1 and 2. The planner walk preallocates a `StaticOriginId` for every
> occurrence; the **lowering walk is an independent traversal of the same source
> with no carried correspondence**, so the occurrence being lowered has *no
> static name*. Every prior attempt reached for a dynamic surrogate — pointer
> identity (entry 1), then whole-configuration (entry 2) — **because the static
> value is out of scope at the site, not because it was mis-chosen.**
>
> Hard-stop #5 ruled the origin **carrier** onto planner records, and the
> Architect and the Steward *both* treated "the carrier exists" as sufficient.
> **It is not: a field on a planner record does not put a value in `lower_expr`'s
> scope.** Hard-stop #7 is where that came due.
>
> ⛔ **This node is therefore sized by what PRODUCES the origin at the site.**
> `RT-FNSPLIT-B2A-S` was sized by what *consumes* it and became unsatisfiable
> inside its own boundary — the framing defect that caused #7. Do not repeat it.

## The deciding measurement (do not re-litigate it; do re-run its guard)

`runtime-implementer`, `evt_4tqj93ctj24z2` — verdict **`TOTAL`, with
injectivity**, established **type-driven over the `RuntimeExpr` declaration**
(`ir.rs:337–461`), not by grep:

- **Leg 1** enumerate every expression-typed field of every variant;
- **Leg 2** check each of `plan_expr`'s arms covers each field — *the leg that
  matters, because those arms use `..` patterns*;
- **Leg 3** close the outside: can any input to `Lowering` hand `lower_expr` an
  expr the planner never walked?

The two fields easiest to drop behind a `..` — `Effect`'s `capability.value` and
`LexicalClosure`'s `captures: Vec<RuntimeExpr>` — are **both planned
explicitly**. No production site synthesizes a `RuntimeExpr` in either cfg.
`NativeSeedEnvironment` holds `RuntimeGroundValue`, which has **no closure
variant**, so `lower_seed_capture` can never yield one. Only `Transparent`
declarations carry a body and `lower_declaration_ref` rejects every other kind.
Injectivity holds because `plan_expr` is a tree traversal **and
`positioned_sources` rejects duplicate/missing origins on every compile** —
checked, not argued.

⇒ **Every `Closure`/`LexicalClosure` occurrence `lower_expr` can reach — by
direct descent *or* only via `source_call_state`/`SourceMachineState::Eval` —
corresponds to exactly one planned `StaticOriginId`, uniformly over all input
programs.**

⚠ **Scoped `could_not_determine`, and it is deliberate:** the *partition* (which
occurrences are machine-only) is **program-dependent and not statically
enumerable**. Totality is determined; the enumeration is not. ⛔ **Do not read
"TOTAL" as "and here is the partition", and do not enumerate a guessed
machine-only subset** — the Architect ruled that explicitly (req. 1 below).

★ **The distinction the census produced, and it is the whole reason this unit is
separable: existence is total; what is absent is *recoverability at the site*.**
The machine holds **clones stripped of position**. An origin exists for each, but
recovering *which* one needs a carried tag or a forbidden key. Two structurally
identical closures at different source positions have different origins and **a
clone cannot distinguish them.**

## Why this is separable from the atomic functionization boundary

**Architect, `evt_1jdh8pn8y96z`, verbatim in substance:** threading **does not
choose static identity, invent an identity space, or define behaviour for an
unplanned occurrence — it transports an already-settled fact to a site where it
is currently out of scope.** That makes correspondence **production plumbing,
not Q3 functionization authority.** It introduces no target code unit, ABI,
dispatch, indirect call, semantic-plane execution, switch-over, or alternate
emission path.

⇒ **The Q3 atomic boundary remains intact:** functionization + live switch +
differential equivalence + old-authority removal are still **one** boundary
(`RT-FNSPLIT-B2F`).

## The six ruled mechanism requirements (transcribed, authoritative)

`5c7eae26`'s D1–D3 **may be carried but are not sufficient alone.** The unit must
close correspondence over the **whole traversal**:

1. **Thread `StaticOriginId`** through `lower_expr`, `SourceMachineState`, **every
   pending-expression frame**, `SourceContinuation`, and `SourcePrefixTemplate`,
   **uniformly over direct descent and the source-machine fallback.** ⛔ **Do not
   enumerate a guessed "machine-only" subset** — the census proved totality
   *without* determining that program-dependent partition.
2. **At structural descent, derive each child's origin ONLY from the current
   occurrence's checked positional child-origin table and the source-field
   ordinal.** ⛔ No pointer/content/hash/clone/visit-order recovery, no
   arithmetic minting, no second identity map.
3. **Whenever an owned `RuntimeExpr` is cloned into a pending frame/template,
   clone/carry its already-known origin in the same constructor.** The pair may
   be represented explicitly; the origin is **provenance only** in this unit.
4. **Seed root and transparent-declaration occurrences from their
   planner-assigned origins.** Declaration references remain **childless
   leaves**; declaration bodies retain their single separately planned entry.
5. **Re-run the type-driven coverage proof as a COMMITTED STRUCTURAL GUARD** —
   every expression-typed field of every `RuntimeExpr` variant must have exactly
   one origin-threading arm. ⭐ **A wildcard / `..` must not make a newly added
   expression field silently originless.**
6. **Prove correspondence at the three closure-construction sites:** the current
   occurrence origin is in scope for `Closure` and `LexicalClosure`; the
   declaration-entry origin is in scope for `DeclarationClosure`. **This unit
   need not yet store the tag in `Lowered::Closure`.**

## ⛔ The negative boundary — what keeps this behaviour-preserving

**During this unit, selection and lowering still consume the existing
`RuntimeExpr` carrier exactly as before.** The threaded origin may be used
**only** to derive/pass child correspondence and to validate coverage. It must
**not** select a body, call a dispatcher, alter a branch, index executable
semantics, or affect emitted CLIF.

Pinned **mechanically**, not by prose:

- production census remains the existing **one** root `FunctionBuilder::new` /
  **one** root `define_function` path;
- **zero** new `Module::declare_function` / `define_function`, call, dispatch, or
  compiled output;
- **no plan `origin -> expr` lookup from a lowering/selection consumer** in this
  unit;
- **focused equivalence:** the pre/post emitted function and observable results
  are unchanged for the closure / source-machine discriminator set.

★ **This is why carrying provenance beside an existing source term is NOT "two
authorities": only the source term is consumed; the origin is not yet a
selector.** The next unit is where that changes.

## Sequencing and the disposition of `5c7eae26`

```
RT-FNSPLIT-B1R  →  RT-FNSPLIT-B2A-C  →  RT-FNSPLIT-B2A-S  →  RT-FNSPLIT-B2F
 (landed)          correspondence        selection            functionization
                   THIS NODE             (atomic: tag as      (atomic: Q3)
                                         selector + remove
                                         body + sole
                                         dispatcher, one diff)
```

⛔ **`5c7eae26` IS NOT MERGEABLE ALONE — the Architect CONFIRMED the Steward's
reading.** Q2's permission was conditional on the complete tag-plus-sole-dispatch
conditions, and D1–D3 without correspondence and without D4 are not a standalone
checkpoint. **Preserve it as durable input and transplant/fold it into this
unit.** It is on `origin/wp/RT-FNSPLIT-B2A-S-selection-defunctionalization`.

## Inventory semantics — state these SEPARATELY

**Architect, ruled:** closing the cause's transport seam is **not** closing
either downstream symptom.

| inventory entry | closed by |
|---|---|
| **3** — recoverability vacancy (the CAUSE) | **this unit** |
| **1** — cloned-body / pointer identity | the later **complete selection** unit (`RT-FNSPLIT-B2A-S`) |
| **2** — whole-configuration specialization | the **atomic functionization / switch** boundary (`RT-FNSPLIT-B2F`) |

⛔ Do not claim entry 1 or 2 on this unit's landing.

## Steward notes for the frame author (me)

- ⭐ **Size by the PRODUCER.** The consumer census (24 sites) understated hard-stop
  #7's real cost by ~100×; the work was in *producing the value at the 3
  construction sites*, which pulls in the descent edits plus the machine-frame
  touch points. A compiler-enumerated census answers "what breaks", never "what
  must now be threaded."
- **Requirement 5 is the durable one** — it is the guard that stops this whole
  class recurring when a new `RuntimeExpr` field is added. Make it an AC with a
  named red artifact, not a note.
- **Requirement 2's "checked positional child-origin table"** must be identified
  against the landed planner before the frame pins anchors; `static_transition.rs`
  and `semantic_ir.rs` are the files.
