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

**D1 — ⭐ THE RETAINED-CLOSURE CARRIER LOSES ITS BODY.**

> **For the covered population only, the retained body ceases to be carried and
> `static_origin` becomes the sole identity.**

> ### ⛔ CORRECTED 2026-07-25 — D1's original phrasing was a STEWARD DEFECT
>
> It read *"`OwnedSourceOccurrence` drops `expr: RuntimeExpr`"* — a **struct-level**
> statement for a **population-level** requirement. **That struct backs 17
> construction sites**, not just the closure family: it also carries the source
> machine's in-flight frames (`SourceContinuation` / `SourcePrefixTemplate` /
> pending-expression, `mod.rs:1799–2077`).
>
> ⛔ **And one site provably cannot resolve from a tag:** `core.rs:3529` builds
> `OwnedSourceOccurrence { expr: RuntimeExpr::Trap(default.clone()), static_origin }`
> — the **one synthesized term in the whole lowering**, whose tag resolves to the
> **match**, not the trap. Dropping `expr` there substitutes a match for a trap —
> exactly the wrong-body defect **D6's swap control exists to redden** — and
> minting an origin is barred (`origin_of` is planner-private).
> ⚠ The adversary independently reached the same placement on `2db29abe`
> (`evt_7mve56d192pv6`): a **lowering-internal leaf**, *not* a planned source
> occurrence, which "does not justify broadening the planned-origin population."
>
> ⇒ **Ruled (a) at `evt_2eap269sgnavm`: the population is the retained-closure
> variants.** Under (a), D1 clears **every** member with no softening — all
> retained closure bodies are `cloned(<planned occurrence>)` and resolve by tag.
> ★ **D1's original phrasing also contradicted D5/AC-6**, which delegates the
> population decision. **D5 was the correct half.**

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
- **AC-4 — the `origin -> expr` lookup count is exactly 1** — ✅ **CLOSED at
  `3c273a38` by TOKENIZATION, not by more spellings.** Stands as framed; the
  defect was the needle, not the property. See the correction below.
- **AC-5′ — narrowed to the three statements a mechanism can enforce, with the
  residual recorded as a REVIEW property.** (Supersedes AC-5.)
  1. **An entry is not nameable outside the planner** — compiler-enforced
     (`PlannedExpr`/`StaticNodeId` carry no `pub`; re-export is `E0364`).
  2. **Entry-keying resolves the WRONG body** — behavioural, with a
     non-vacuity guard.
  3. **The sanctioned selection table REFUSES re-keying onto entries** —
     mechanical.
  ⛔ **Residual, stated not hidden:** an independently maintained entry-keyed
  collection *inside* the two planner files is a **review** property, not a
  mechanical one. Inside the planner, entry-keying is the planner's own job and
  is **not** prohibited.
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

> ## ⛔ AC CORRECTION 2026-07-25 — AC-4 AND AC-5 WERE STEWARD DEFECTS
>
> **Authority: Steward, frame author.** The Architect blocked `d99d223d`
> (`evt_1p11krxny4wny`) on AC-4 and AC-5 for the **third consecutive round**,
> each time by a compile-preserving evasion, while affirming both times that
> *the production mechanism is coherent and the fold changes no production
> bytes*. ⭐ **Three defeats of the same two pins is not a ring failure — it is
> the tell that the pins ask for something unenforceable.** The Architect named
> the alternative explicitly: *"route a frame correction that narrows AC-4/AC-5
> to properties the mechanism can enforce. Do not add spellings to the scans."*
> This is that correction. ⚠ It is **not** a hard-stop — count of record stays
> **8**, and the Architect's own call (*"another review fold, not #9"*) governs.
>
> ### ⚠ AC-4 — MY OWN FIRST DRAFT OF THIS CORRECTION WAS WRONG, AND IS WITHDRAWN
>
> At `12:2xZ` I drafted this correction concluding *"the census is neither
> necessary nor sufficient ⇒ retire it."* **The implementer's fold 2
> (`evt_2ve8wt25s24bk`), authored in parallel, falsifies that.** Recorded here
> rather than quietly replaced, because the mistake is the instructive part.
>
> **The actual cause of all three AC-4 defeats was a LAYOUT-SCOPED NEEDLE.**
> `line.contains(".source_occurrence(")` is a claim about *formatting*: the
> Architect's mutation split the token across lines, so **no line contained the
> string**. ⇒ ⭐ **The property was enforceable all along; the needle was
> line-oriented.** Fold 2 replaces the mechanism with **tokenization** — strip
> line comments, split on every non-identifier character, match **whole tokens**,
> count the *identifier* rather than a call shape. By construction: no formatting
> can hide a mention, `source_occurrences` is not conflated with
> `source_occurrence`, and path-form/aliased calls are caught because *a method
> cannot be called without naming it*. Verified reddening on **both** demonstrated
> evasions. ⇒ **AC-4 stands as framed and is CLOSED.**
>
> ⛔ **The lesson against myself: I inferred "unenforceable property" from three
> failures of one mechanism, when the mechanism had a single shared defect.**
> "Three defeats ⇒ the default branch is wrong" correctly says *stop repairing the
> detector* — it does **not** license concluding the property cannot be enforced.
> **Ask what all the failures share first.** Here they shared a line boundary.
>
> ⚠ **Residual, NON-BLOCKING** (do not open a fourth fold for it): the exported
> inventory pins each item only up to `(`, so an *existing* accessor changed to
> return `SourceOccurrence<'src>` — which contains a term — keeps both the name
> list and the `-> Result<&'src RuntimeExpr` count green. That is an undemonstrated
> hole in a neighbouring property (AC-1/AC-2 territory, and AC-1 is closed).
> **Record it; do not chase it.** Fold 2's stated macro-synthesis limit is
> likewise accepted as a recorded limit.
>
> ### AC-5′ — RULED: adopt the implementer's narrowing. The scan was chasing a ghost
>
> **Ruled authoritative** (`evt_2ve8wt25s24bk`, and it is the branch I had reached
> independently — this is corroboration, not inheritance, since we measured
> different things). The three enforceable statements above are AC-5's discharge;
> the in-planner residual is a **review** property.
>
> **Why no test can close the global negative.** ⭐ The implementer did the thing
> the frame demands and *checked whether a mechanism could enforce it* rather than
> assuming: a `SchedulingEntry` newtype without `Ord`/`Hash` and with a private
> field. **It fails**, because `edge()` must read the raw ordinal
> (`self.plan.nodes[from.0 as usize].owner`) to index the node table — so any
> wrapper must expose the ordinal to code in **the same module as the mutation
> site**. ⇒ **The property is enforceable exactly at module boundaries and nowhere
> inside one module.** That is a real result, not a concession.
>
> **The boundary half was genuinely missing and is now pinned.** Measured: the
> entry types carry **no `pub`**, so no consumer can *name* an entry, hence none
> can key on one — surface **12 backend files → 2**. ⭐ **And the reduction is the
> compiler's, not the test's:** widening the declaration reddens the pin, while
> re-exporting the type **does not compile** — `E0364: StaticNodeId is private,
> and cannot be re-exported`. ⚠ That is *stronger* than the export-inventory pin I
> had drafted, which is why my version is dropped: **I proposed a test for
> something the compiler already refuses.**
>
> ⚠ **And the scan could never be made sound:** `.entry(` is `BTreeMap`'s std API.
> **Every** non-planner `.entry` hit in the whole backend is a std map call
> (`lowering/mod.rs:1297`, `planning.rs:373/379/385/406/435/456`). Tightening the
> needle buys false positives, not closure.
>
> ### ⭐ The class of MY defect, so the next frame does not repeat it
>
> Both surviving blocks came from **a pin claiming more than its mechanism could
> see** — the implementer's own summary, and it is exact. My share, across this
> chain, is one shape repeated three times: **I stated a requirement in terms of
> the artifact I had most recently looked at.** D1 named a struct when the
> requirement was a population; AC-4 named a *line-matchable call shape* when the
> requirement was an identifier count; AC-5 named a global spelling class when the
> requirement was a module boundary.
>
> ⇒ **Write the pin as the PROPERTY, then ask which mechanism already enforces it
> — the compiler is a legitimate answer and often the strongest one** (`E0364`
> here did more than any test I could have specified). ⇒ **And when a pin must be
> narrowed, narrow it to what is enforceable AND record the residual in the
> source next to the enforced statements**, so the next reader inherits the limit
> instead of the overclaim. **An AC taxonomy with no cell for the honest answer
> reads as complete.**

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
