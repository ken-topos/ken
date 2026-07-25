# RT-FNSPLIT-B2A-S — retained-body selection defunctionalization

**Owner:** Runtime · **Size:** M
**Depends on:** `RT-FNSPLIT-B2A-C` (merged, `2db29abe`)
**Blocks:** `RT-FNSPLIT-B2F`
**Anchors re-derived on `origin/main` = `4c5afda6`.** ⛔ They are **not** copied
from the pre-`B2A-C` frame — every line number in the old draft was stale.
If an anchor does not hold on your base, that is a **hard-stop**.

> ## ⛔ READ FIRST — THIS UNIT DELIBERATELY CROSSES A LANDED NEGATIVE PIN
>
> `RT-FNSPLIT-B2A-C` landed **N3: no plan `origin -> expr` lookup from a
> lowering/selection consumer.** ⭐ **B2A-S's entire job is to introduce exactly
> that lookup.** So N3 is **not** a constraint you inherit — it is a pin you
> **delete and replace** with the sole-dispatcher pin (AC-4 below).
>
> ⛔ **Do not treat the crossing as a violation, and do not preserve N3.** Say in
> your handoff that N3 is retired by design, and name what replaces it. A reviewer
> who reads the new lookup against B2A-C's AC list without this will reject a
> correct diff.
>
> Equally, the adversary's verified tripwire on `2db29abe` was: *"the first read
> of the origin's **value** in a decision."* **That read is this unit's D2.**

## Objective

**Make the static origin the sole authority for selecting a retained closure
body, and remove the body carrier in the same diff.**

★ **The recognition criterion is that the consumer is the SOLE point of
consumption.** A tag with a second consumption path is not a defunctionalization;
it is a tag with a leak.

## What B2A-C already did for you — do NOT rebuild it

1. **The origin is in scope at every construction site.** `plan_expr` returns
   `PlannedExpr { entry, occurrence }`; correspondence is threaded through
   `lower_expr` (`core.rs:4255`), `SourceMachineState`, the pending-expression
   frames, `SourceContinuation` and `SourcePrefixTemplate`.
2. ⭐ **The body and its origin are ALREADY CO-LOCATED** —
   `OwnedSourceOccurrence { expr: RuntimeExpr, static_origin: StaticOriginId }`
   at `mod.rs:238-241`, and both closure variants carry it:
   `Lowered::Closure { captures, params, body: OwnedSourceOccurrence }`
   (`mod.rs:460-464`) and `Lowered::DeclarationClosure { symbol, captures,
   params, body: OwnedSourceOccurrence }` (`mod.rs:465-470`).
3. **Three origin accessors exist, all planner-side and positional:**
   `child_static_origin` (`static_transition.rs:945`), `root_static_origin`
   (`:958`), `declaration_occurrence_origin` (`:971`). ⛔ `origin_of` is
   planner-private; no consumer may mint an origin.
4. **The origin is currently provenance only** — verified by the adversary on
   `2db29abe`: `static_origin` is **never compared, never a map key, never
   branched on** in production `lowering/`.

## Deliverables

**D1 — ⭐ THE CARRIER LOSES ITS BODY. This is the whole slice, and it is now a
one-field statement.** Because B2A-C co-located them, D1 is precisely:

> **`OwnedSourceOccurrence` drops `expr: RuntimeExpr`, leaving `static_origin`
> as the sole identity.**

⛔ **In the same diff.** A struct retaining `expr` beside the origin preserves two
authorities and **fails the slice** — that is what makes the origin a *selector*
rather than provenance. ⚠ This is the requirement most likely to be softened
under pressure and the one that must not be. **If it cannot be completed for some
member of the population, that is a hard-stop, not a partial landing.**

**D2 — one closed consumer, and it is the FIRST read of the origin's value in a
decision.** Install a single resolver — `lower_static_origin(static_origin,
dynamic_env)` or as you prefer; the *shape* is ruled, not the name. It resolves
the occurrence **from the plan, solely by the tag**, then invokes the existing
recursive lowering. **Every** retained closure application / resume in scope
routes through it; **none** selects a body directly.

**D3 — key on `.occurrence`, NEVER on `.entry`.** The adversary's second verified
tripwire: every production use of `.entry` today is an **edge endpoint** or
`next = planned.entry`, never a key. ⛔ A collection keyed by `.entry`
reintroduces the #8 category error, because nested `ComputationalMatch`
occurrences **share a scheduling entry**. Occurrences are safe as keys —
B1R enforces the `origin.0 == planned_node.0` bijection.

**D4 — retire N3 and install the sole-dispatcher pin in its place.** B2A-C's N3
asserted no `origin -> expr` lookup exists. Delete that assertion and replace it
with a pin asserting **exactly one** such lookup, reachable from **exactly one**
consumer. ⇒ The count goes `0 -> 1`, not `0 -> unbounded`.

**D5 — decide the population EXPLICITLY and say which.** `Lowered::Closure`,
`Lowered::DeclarationClosure`, and `Lowered::ComputationalRecursorClosure` are one
family. **State for each whether it is in the covered population.** ⚠ The
declaration case is asymmetric: its origin is reachable **by symbol**
(`declaration_occurrence_origin`), so it may not need the same treatment — decide
and justify, do not assume.

**D6 — controls, each red at a named artifact.**

- **swap two equal-shaped origin entries** → wrong-body / reject control
  **reddens**;
- **perturb a borrowed address without changing any ordinal** → **no identity
  change** (⭐ the chain's predicate as an executable test);
- **replace a tag lookup with a pointer or content lookup** → **fails loudly**;
- **duplicate / missing / out-of-range origin** → **loud planner failure**
  (`PlannerInvariant`, per `RT-PLANNER-ATTRIB-K` — an invariant violation is a
  compiler bug, not a capacity limit);
- ⭐ **a second consumption path** → **reddens.** This is D2's recognition
  criterion as a control, and it is the one a happy-path suite omits.

**D7 — carry the adversary's N1 recipe deliverable (≈3 lines, cheap).** The AC-11
topology differential's provenance is **verified** (7/7 rows reproduce from
`70bd2c74`) but **the recipe is not in the tree**. ★ The asserted property is
*equality against committed constants*, so **a post-change re-capture would have
produced byte-identical values — nothing distinguishes a genuine baseline from a
re-recording.** ⇒ Record in the comment: the base SHA, the probe function names
(`b2ac_topology_digest`, `b2ac_topology_fixtures`), and the
`git worktree add --detach <path> <base>` + test invocation. **Demonstrate the
binding; do not testify to it.**

**D8 — fix the queued wording nit** *only because you are already in that file*:
`px8ta_oriented_subcontinuation.rs` says the wrapper "makes the harness match the
product" while granting **256 MiB, 32×** the product's 8 MiB. Correct to *"at
least the product's headroom."* ⛔ Not worth a commit of its own.

## Acceptance criteria

- **AC-1 — `expr` is GONE from the carrier**, shown structurally, not asserted.
- **AC-2 — the resolver is the SOLE consumption point.** D6's second-path control
  reddens. ⛔ A grep is not sufficient — pin it at a named artifact.
- **AC-3 — identity is the ordinal, provably.** D6's address-perturbation control.
- **AC-4 — the `origin -> expr` lookup count is exactly 1**, replacing B2A-C's
  zero-count pin. State both the old and new assertion so the transition is
  auditable.
- **AC-5 — no collection is keyed by `.entry`**; a mutation introducing one
  reddens.
- **AC-6 — the population is stated per variant** (D5), with the declaration
  asymmetry decided and justified.
- **AC-7 — inventory honesty: this claims entry 1 ONLY.** ⛔ Not entry 2 (waits
  for `RT-FNSPLIT-B2F`); entry 3 was closed by `B2A-C`. State them separately —
  this is a standing ruling, not a formality.
- **AC-8 — no regression, GREEN IN CI.** ⛔ Never a local `--workspace` run.
- **AC-9 — D7's recipe is in the tree.**

Each mutation applied at its **natural production site**, restored
**byte-identically**, verified with `git diff --quiet` (⚠ `--stat` always exits 0
and is not an emptiness test).

## ⚠ Cost and measurement discipline, learned the hard way in `B2A-C`

- ⭐ **Before threading or removing a derived identity across N sites, compose the
  accessor with itself, per variant, on a real instance.** `B2A-C` threaded ~60
  call sites before checking that the identity a parent hands down owns the next
  level's entries; it did not, and that was hard-stop #8. **Totality of a table
  says nothing about composition.**
- ⚠ **`core.rs` mentions `static_origin` 165 times.** That is a *consumer* count.
  ⛔ **Do not size this WP from it** — the #7 defect was sizing by consumers. Size
  by what must be **true at each construction site** after `expr` is gone.
- ⛔ **Never select a build artifact from an accumulating directory by name or
  mtime**, and **never pipe a gate through `tail`** — it discards the evidence
  *and* replaces the exit code. Three confident-wrong measurements came from this
  in one session.
- ⚠ **If a resource cliff (stack, RSS, timeout) fires, measure the BASE's MARGIN,
  not just pass/fail.** Attribution needs the margin. And **fixing a cliff by
  raising a limit spends a detector — name which one and where its replacement
  belongs.**

## Build discipline

⛔ **Targeted builds only — NEVER `--workspace`** (operator hard rule,
`COORDINATION §12`): `-p ken-runtime`, or `--test <name>`. Workspace, `--locked`
and conformance run **in CI**. ⚠ Changes reaching the reifier need the full
`-p ken-interp` suite. ⚠ **`ken-cli` integration tests live in another shard and
are invisible to `-p ken-runtime`** — that is how `B2A-C` went red after a green
targeted run.

⚠ **`crates/ken-runtime/src/cranelift_backend.rs` is cited in
`library/SOURCE-ATTESTATIONS`.** It is the module **root**; your files are
submodules and none are attested. An edit there reddens
`registered_record_validation_gates_run` for reasons that look unrelated. Prefer
not touching it; if you must, **say so** and ⛔ do not re-attest.

## Hard-stop

**Count of record is 8** (`RT-NATIVE-FNSPLIT` holds it; that line wins any
disagreement).
⚠⚠ **#9 IS THE VERY NEXT STOP AND IT FIRES A RESEARCH PULL** — the Steward
dispatches research **before** the Architect rules. Do not soften a
deliverable to avoid a stop; **the last two stops were Steward framing defects
that the ring caught by stopping.**

Stop and escalate if: an anchor fails; **D1 cannot be completed for some member**;
the sole-consumption property cannot be pinned mechanically; or removing `expr`
forces functionization work (that is `B2F`'s atomic boundary and must not leak
here).

⭐ **Report the measurement and name the discriminating property — do not infer a
boundary from a size.** That was the ring's own #7/#8 lesson, and it is the one
that keeps costing this chain.
