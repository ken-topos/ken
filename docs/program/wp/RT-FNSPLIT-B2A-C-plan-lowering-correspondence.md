# RT-FNSPLIT-B2A-C — plan↔lowering occurrence correspondence

**Owner:** Runtime · **Size:** L (raised from M at hard-stop #8)
**Depends on:** `RT-FNSPLIT-B1R` (landed)
**Blocks:** `RT-FNSPLIT-B2A-S` (selection) → `RT-FNSPLIT-B2F` (functionization)
**Anchors pinned on `origin/main` = `70bd2c74`.** A Steward doc-only publish
(PR #939) is in flight; it touches `docs/program/**` only and cannot move any
anchor below. ⛔ If any anchor does not hold, that is a **hard-stop**, not a
reinterpretation — see §Hard-stop.

> ## ⛔ AMENDED IN PLACE — 2026-07-25, hard-stop #8. RE-READ D9, D2, D3.
>
> **`RT-FNSPLIT-B2A-C` was AMENDED, not re-sliced** (Steward's call; the
> Architect explicitly left WP scope and the count to me). The finding repairs
> **this unit's own producer**, so it is entry-3 scope — entries 1 and 2 stay
> open and the Q3 boundary is untouched.
>
> - ⭐ **NEW `D9` — the dual planner result.** Build it **first**; D2 and D3 are
>   only well-defined once it exists.
> - **`D2`'s premise was wrong** — "the child's origin" must read "the child's
>   `.occurrence`".
> - **`D3` gains a SECOND AXIS.** Its probe found the defect; its vocabulary
>   mis-named it an "ordinal disagreement" when it is a **category error**.
> - **New `AC-11`–`AC-17`** carry the ruled closure.
> - **Size raised M → L, deliberately and in the open** — the #7 lesson was that
>   a unit which grows silently is how a frame becomes unsatisfiable.
>
> ⛔ **Nothing in D1/D4–D8 or N1–N4 changes.**

## Objective, in one sentence

**Transport each occurrence's already-preallocated `StaticOriginId` to the
lowering site where it is currently out of scope — and change nothing else.**

⛔ **This unit does NOT select a body, dispatch, functionize, or remove the
existing carrier.** It makes the static name *available*. Selection is
`RT-FNSPLIT-B2A-S`; functionization is `RT-FNSPLIT-B2F`.

## Why this unit exists (read this before the deliverables)

`RT-NATIVE-FNSPLIT` symptom-inventory **entry 3** is a *vacancy*, and entries 1
and 2 are the surrogates it forced. The planner preallocates an origin for every
occurrence; **the lowering walk is an independent traversal with no carried
correspondence**, so `lower_expr` has no static name for what it is lowering.
Pointer identity (entry 1) and whole-configuration (entry 2) were reached for
**because the static value is absent at the site.**

★ **The census distinction that makes this unit separable: existence is TOTAL;
what is absent is RECOVERABILITY AT THE SITE.** The machine holds clones
stripped of position. Two structurally identical closures at different source
positions have different origins, and **a clone cannot distinguish them.**

## Settled inputs — do NOT re-derive these

1. **Totality + injectivity are MEASURED** (`evt_4tqj93ctj24z2`, type-driven over
   `ir.rs:337–461`, both cfgs, definitions included). Every `Closure` /
   `LexicalClosure` occurrence `lower_expr` can reach — **including via
   `source_call_state` / `SourceMachineState::Eval`** — has exactly one planned
   `StaticOriginId`, **uniformly over all input programs.**
   ⛔ Do not re-run this as a precondition. **Do** re-run it as D5's committed
   guard, which is a different artifact for a different purpose.
2. **The origin IS the node ordinal** — `origin: StaticOriginId(planned_node.0)`
   at `semantic_ir.rs:194` (expression) and `:231` (control). You are **not**
   minting an identity space.
3. ⭐ **The positional child-origin table ALREADY EXISTS on landed `main`** —
   this is requirement D2's producer and it is not yours to build:
   - `child_origins: Vec<StaticOriginId>` on the arena (`semantic_ir.rs:324`),
     pushed **positionally** at `:173`
     (`arena.child_origins.push(StaticOriginId(child.0))`);
   - `child_origins: DenseRange` per plane record (`:417`), and
     `children: DenseRange` on the node (`:155`);
   - the **checked** invariant `material.len + children.len ==
     source_material_elements` (`:143`), enforced via `children_since` (`:374`);
   - the declaration states the mechanism outright at `:248–249`: *"A syntax
     child is not an atom. Child positions live in the record's positional
     child-origin range, so **child k is recoverable as child k**."*
4. ⭐ **`plan_expr` ALREADY THREADS A POSITIONAL CHILD ORDINAL** — `plan_expr`
   (`static_transition.rs:493`) takes an ordinal parameter, and the call sites
   *state the ordering*: `scrutinee` → `0` (`:566`, `:589`, `:656`),
   `then_expr` → `1` and `else_expr` → `2` (`:560`, `:561`), `plan_sequence`
   passes `ordinal as u32` (`:454`, `:481`), roots pass `0` (`:1410`, `:1415`).
5. **Visibility.** `StaticOriginId` is `pub(super)` (`semantic_ir.rs:18`);
   `StaticNodeId` is **private** (`static_transition.rs:23`). Per the hard-stop-#5
   ruling: widen `StaticOriginId` **only** to
   `pub(in crate::cranelift_backend)`, spell the carrier field **`static_origin`**
   (⛔ never bare `origin` — `RecursorProducerOriginId` already spells that word
   on these records), and do **not** widen `StaticNodeId`.
6. **The scoped `could_not_determine` is deliberate and binding.** The *partition*
   — which occurrences are machine-only — is **program-dependent and not
   statically enumerable.** ⛔ **D1 forbids enumerating a guessed machine-only
   subset.** Thread uniformly instead.

## Deliverables

**D1 — thread uniformly over the WHOLE traversal.** Carry `StaticOriginId`
through `lower_expr`, `SourceMachineState` (`mod.rs:1962`), **every**
pending-expression frame, `SourceContinuation`, and the **cloneable**
`SourcePrefixTemplate`, **uniformly over direct descent and the source-machine
fallback** (`source_call_state`, `core.rs:3542`). ⛔ **No guessed "machine-only"
subset.**

⭐ **The delegation point is `core.rs:2078`** — `other => …lower_expr(builder,
&other, &env)`. That fallback arm hands `lower_expr` **every form the machine's
own 11-arm dispatcher does not handle, closures included**, which is exactly why
a subset-based threading cannot be sound. ⚠ `SourcePrefixTemplate` being
**cloneable** is the D4 pressure point: a clone that drops the origin
reintroduces the vacancy silently.

**D2 — derive child origins ONLY from the checked positional table + the
source-field ordinal.** At structural descent, a child's origin comes from the
current occurrence's positional child-origin range indexed by the source-field
ordinal — nothing else. ⛔ **No pointer / content / hash / clone-order /
visit-order recovery, no arithmetic minting, and no second identity map.**

> ### ⛔ AMENDED 2026-07-25 (hard-stop #8) — D2's PREMISE WAS WRONG
>
> **D2 said "the child's origin" as if `plan_expr`'s return value *were* that
> origin. It is not.** See **D9**: `plan_expr` returns a *scheduling entry*, and
> for `ComputationalMatch` that is a **different node** from the one carrying the
> occurrence record. ⇒ **Read D2 as "the child's `.occurrence`", never its
> `.entry`.** Build D9 first; D2 is only well-defined once it exists.

**D3 — ⭐ ESTABLISH THE ORDINAL CORRESPONDENCE PER VARIANT. This is the real
work and it is the one thing D2 silently presupposes.** The positional table is
usable **only if the lowering's per-variant child order agrees with the
planner's ordinals.** That agreement is **not currently written down anywhere**,
and nothing in the type system enforces it.

⇒ For **every** `RuntimeExpr` variant with expression-typed fields, state the
planner's ordinal for each field (read off the `plan_expr` call sites in
§Settled-4) **beside** the lowering's traversal position, and show they agree.
Where they disagree, **the planner's ordinal is authoritative** and the lowering
adapts — ⛔ do not renumber the planner to match the lowering, because the
plane's records are already laid out against those ordinals.

⚠ **Report a disagreement as a finding, not a silent fix.** A mismatch here is
precisely the class of defect that produced entries 1–3, and I want to see it.

> ### ⭐ AMENDED 2026-07-25 (hard-stop #8) — D3 NEEDS A **SECOND AXIS**
>
> **D3's probe worked: it found the defect. D3's vocabulary did not: it named it
> an "ordinal disagreement" when it is a CATEGORY ERROR.** That mis-naming is the
> tell that this frame had **one axis where it needed two.**
>
> **The source-field positions D3 states are CORRECT** (`0` scrutinee, `1+i` case
> body) and stand. But ⭐ **positional agreement does NOT imply entry/occurrence
> identity** — a second, independent axis. ⇒ D3's table must record **both**: the
> source-field ordinal *and*, per variant, whether `entry == occurrence.node`.
>
> **All non-`ComputationalMatch` expression results have
> `entry == occurrence.node`. `ComputationalMatch` is the SOLE explicit split.**
> State that in the table as the invariant it is, so the next author cannot
> re-conflate the two.

**D9 — ⭐ THE DUAL PLANNER RESULT. Build this FIRST; D2 and D3 depend on it.**
Ruled at `evt_308azmr4cszd7`. One `StaticNodeId` is currently made to mean two
different things:

- **`entry`** — the first node the transfer graph *schedules* for the expression;
- **`occurrence`** — the node on which `SemanticSourceSeed::expression`
  registered that `RuntimeExpr`, and from which its positional child-origin
  record is read.

They coincide for ordinary forms and **deliberately do not** for
`ComputationalMatch`. Verified on exact `96e66c9f`: `:628` pushes the
`SourceReturnResume`; `:672` registers the occurrence **on `resume`**; `:673`
returns **`scrutinee`**. The plane derives an origin from that record node
(`semantic_ir.rs:171–203`) and later indexes child positions by it (`:645–695`).

Make the recursive result carry **both already-created facts, structurally and
without a lookup**:

```rust
#[derive(Clone, Copy)]
struct PlannedExpr {
    entry: StaticNodeId,
    occurrence: StaticOriginId,
}
```

`plan_expr` returns `Result<PlannedExpr, _>`. An ordinary node returns
`{ entry: node, occurrence: origin_of(node) }`; `ComputationalMatch` returns
`{ entry: scrutinee.entry, occurrence: origin_of(resume) }`. **`origin_of` is
formed only inside the planner**, from the node allocated for that semantic seed
— ⛔ no lowering consumer may mint it.

**The two fields have DISJOINT consumers, and that is the whole mechanism:**

1. **Transfer topology consumes only `.entry`** — every `edge`, successor chain,
   case-test target, sequence entry, and `plan.entries` element. ⇒ the approved
   Boundary-A graph stays **topologically unchanged**: a parent still schedules
   the computational scrutinee, **never** the resume.
2. **Source correspondence consumes only `.occurrence`** — every positional child
   list handed to `expression_node` / `expression_seed` records the child's
   `.occurrence`, never its `.entry`.
3. `plan_sequence` / `plan_cases` keep their chain/dispatch **entry**, but their
   positional result is the children's **occurrence** origins: edge to each
   child's `.entry`, place its `.occurrence` at the source position.
4. **Root and transparent-declaration seeding preserve the same distinction.**
   `plan.entries` stays scheduling entries; `root_static_origin()` returns the
   root result's `.occurrence`; `declaration_entry_origin` (rename to
   `declaration_occurrence_origin` permitted) stores the declaration result's
   `.occurrence`. **Store these at planning time — ⛔ never recover occurrence
   from entry later.**

⭐ **CHANGE THE `expression_node` / `expression_seed` API TO TAKE
`&[StaticOriginId]`.** The Architect states this as a preference; **I am making
it a requirement**, because it makes the type prevent this exact conflation
instead of relying on call-site discipline — the same reason D5 is a structural
guard rather than a note. ⚠ **If there is a concrete obstruction to the signature
change, report it as a finding and I will relax this to the Architect's
preference** — do not silently fall back to discipline.

⛔ **This is NOT a second identity space and NOT an `entry -> record` map.** Both
values are outputs of the same recursive visit; `occurrence` is the **existing**
`StaticOriginId` already assigned to the **existing** semantic seed. It adds **no
node, no origin, no search, no arithmetic.** Specifically: ⛔ do **not** add a
plane accessor that searches by entry, do **not** walk `ruled_children`, do
**not** seed the match onto the scrutinee, and do **not** return the resume as
the transfer entry.

⚠ **The synthesized-`Trap` finding stands as corrected:** it is a
lowering-internal leaf derived from a match-default atom, **not** a newly planned
source occurrence. It needs no child correspondence and ⛔ **does not justify
broadening the planned-origin population claim.**

**D4 — carry provenance in the SAME constructor as the clone.** Whenever an owned
`RuntimeExpr` is cloned into a pending frame or template, clone/carry its
already-known origin **in that same constructor**. The pair may be represented
explicitly. ⛔ In this unit the origin is **provenance only**.

**D5 — the coverage guard, committed, structural, and RED at a named artifact.**
Re-run the type-driven coverage proof **as a test that ships**: every
expression-typed field of every `RuntimeExpr` variant has **exactly one**
origin-threading arm.

⭐ **The guard's whole purpose is that a wildcard / `..` must not let a NEWLY
ADDED expression field become silently originless.** So it must fail on an
*added* field, not merely on the current set — a test that enumerates today's
variants and passes is not this guard. ⇒ **State how your guard reacts to a new
expression-typed field, and demonstrate it** (add one in a scratch mutation,
observe red, revert byte-identically, `git diff --quiet`).

⚠ This is the deliverable with the longest half-life in the whole chain: it is
what stops entry 3 recurring the next time `RuntimeExpr` grows a field.

**D6 — seed the roots.** Root and transparent-declaration occurrences take their
planner-assigned origins (`:1410`, `:1415`). Declaration references remain
**childless leaves**; a declaration body keeps its **single** separately planned
entry. ⇒ Inlining at reference sites adds **no** second planning.

**D7 — prove correspondence in scope at all three closure-construction sites.**
The current occurrence's origin is in scope at `RuntimeExpr::Closure`
(`core.rs:4211`) and `::LexicalClosure` (`:4226`); the **declaration-entry**
origin is in scope for `DeclarationClosure`. ⛔ **This unit need NOT store the
tag in `Lowered::Closure`** — and per the negative boundary it must not *use* it.

⚠ **The three sites are not symmetric, and the asymmetry is load-bearing:**
`DeclarationClosure` is built in `lower_declaration_ref`, where the origin **is
already reachable by symbol** — so it needs no threading, only the seeding in D6.
The two `lower_expr` arms are the ones with nothing in scope. ⇒ Do not let the
easy third site suggest the other two are close to done.

**D8 — fold `5c7eae26`.** D1–D3 of the halted B2A-S slice (the dense
compile-local table, its lifetime on `StaticTransitionPlan<'src>`, the
visibility widening) are **durable input**, on
`origin/wp/RT-FNSPLIT-B2A-S-selection-defunctionalization`. Transplant what
survives contact with D1–D3 above; ⛔ **do not merge that SHA as a unit** (the
Architect confirmed Q2's permission does not fire for it). Say explicitly what
you kept and what you dropped.

## ⛔ The negative boundary — this is what makes the unit reviewable

**Selection and lowering still consume the existing `RuntimeExpr` carrier exactly
as before.** The threaded origin may be used **only** to derive/pass child
correspondence and to validate coverage. It must **not** select a body, call a
dispatcher, alter a branch, index executable semantics, or affect emitted CLIF.

★ **This is why provenance beside an existing source term is NOT "two
authorities": only the source term is consumed; the origin is not yet a
selector.**

Pin it **mechanically**:

- **N1** production census stays **exactly one** root `FunctionBuilder::new`
  (`core.rs:140`) and **one** root `define_function` (`core.rs:202`);
- **N2** **zero** new `Module::declare_function` / `define_function`, call,
  dispatch, or compiled output;
- **N3** **no plan `origin -> expr` lookup from any lowering/selection
  consumer** in this unit;
- **N4** emitted function and observable results **unchanged** for the closure /
  source-machine discriminator set.

## Acceptance criteria

- **AC-1 — uniform threading, shown not asserted.** A structural test pins that
  the origin is carried on `SourceMachineState`, every pending-expression frame,
  `SourceContinuation`, and `SourcePrefixTemplate`. ⛔ A prose claim that "the
  fallback is covered" does not discharge this.
- **AC-2 — ⭐ the ordinal correspondence is WRITTEN DOWN, per variant.** D3's
  field↔ordinal table ships in the source as a comment or test table, so the
  next author cannot re-derive it wrongly. Any disagreement found is reported.
- **AC-3 — ⭐ the coverage guard reddens on a NEW field.** D5 demonstrated
  against an added expression-typed field, restored byte-identically
  (`git diff --quiet`). ⛔ A guard that only enumerates today's variants **fails
  this AC**.
- **AC-4 — child-origin derivation is positional, provably.** Control: **swap two
  same-shaped children of one variant** → the derived origins swap with them.
  Control: **perturb a borrowed address without changing any ordinal** → **no
  identity change.** ⭐ The second is the chain's predicate as an executable
  test: if identity moves when only the address moved, the tag is not
  authoritative.
- **AC-5 — forbidden keys fail loudly.** Replacing a positional derivation with a
  pointer or content lookup **fails** at a named artifact.
- **AC-6 — duplicate / missing / out-of-range origin is a LOUD planner failure**
  (`PlannerInvariant`, per `RT-PLANNER-ATTRIB-K`) — an invariant violation is a
  **compiler bug, not a capacity limit**.
- **AC-7 — N1–N4 each pinned by a committed check**, not by review reading. State
  the artifact for each.
- **AC-8 — behaviour preserved.** N4's focused equivalence runs green, and the
  no-regression criterion is **green in CI**, never a local `--workspace` run
  (`COORDINATION §12`).
- **AC-9 — inventory honesty.** The landing claims **entry 3 only**. ⛔ It must
  **not** claim entry 1 (waits for `B2A-S`) or entry 2 (waits for `B2F`). Say so
  explicitly in the handoff.
- **AC-10 — the D8 fold is stated.** What was kept from `5c7eae26`, what was
  dropped, and why.

### AC-11..AC-16 — the ruled closure for D9 (added 2026-07-25, hard-stop #8)

Each must be **mechanically proved**, not reviewed:

- **AC-11 — every transfer edge is unchanged and consumes `.entry`.** The
  Boundary-A graph is topologically identical.
- **AC-12 — every semantic child position consumes `.occurrence`.**
- **AC-13 — the split is exactly one variant.** All non-`ComputationalMatch`
  expression results satisfy `entry == occurrence.node`; `ComputationalMatch` is
  the **sole** explicit split (entry = its scrutinee entry, occurrence = its
  resume seed).
- **AC-14 — nested computational matches remain INJECTIVE even when multiple
  occurrences share a scheduling entry.** ⭐ This is the one that would silently
  pass on a shallow test — a shared entry is exactly the condition under which a
  wrong key still looks unique.
- **AC-15 — root and transparent-declaration `ComputationalMatch` bodies receive
  the RESUME occurrence origin, not the scrutinee origin.** A root or declaration
  whose body is a `ComputationalMatch` is a **required discriminator** — the frame
  names it because it is the case a happy-path test omits.
- **AC-16 — the two swap controls redden in OPPOSITE places.** Replacing a
  positional `.occurrence` with `.entry` → reddens a **named
  `ComputationalMatch` discriminator**; replacing a transfer `.entry` with
  `.occurrence` → reddens the **Boundary-A topology check**. ⭐ Two controls that
  fail in the *same* place would not distinguish the two axes, which is the whole
  point of D9.
- **AC-17 — the existing 28-test computational population is green** without
  weakening D2 or any of N1–N4.

Each mutation applied at its **natural production site**, restored
**byte-identically**, verified with `git diff --quiet` (⚠ `--stat` always
exits 0 and is not an emptiness test).

## ⚠ ONE ATTESTED SOURCE SITS IN YOUR SUBSYSTEM — Steward contention finding

**`crates/ken-runtime/src/cranelift_backend.rs` is cited in
`library/SOURCE-ATTESTATIONS`**, at blob OID `8508a01c`, which is **exactly its
current OID**. Nothing else in `crates/ken-runtime/**` is attested.

⇒ **If you edit that file — even a locator-only change like adding a `mod` line
or a re-export — its OID moves, the ledger row goes stale, and
`registered_record_validation_gates_run` REDDENS IN CI** for reasons that will
look unrelated to your change. This exact class cost a Steward PR its first
attempt on 2026-07-24.

**Your scope should not need it:** the widening is to
`pub(in crate::cranelift_backend)`, and the files in play are
`lowering/core.rs`, `lowering/mod.rs`, `planning/static_transition.rs`, and
`planning/static_transition/semantic_ir.rs` — all **submodules**, none attested.

⇒ **Prefer not touching `cranelift_backend.rs`.** If D1's threading genuinely
requires it, **say so in your handoff** and do **not** silently re-attest:
`scripts/gen-source-attestations.sh` writes a `.proposed` and **refuses to
install it**, because re-attesting *asserts* the currency claim. Re-attestation
is the Steward's call and requires diffing each cited anchor across both OIDs.

## Build discipline

⛔ **Targeted builds only — NEVER `--workspace`** (operator hard rule,
`COORDINATION §12`). Use `scripts/ken-cargo` scoped `-p ken-runtime`, or
`--test <name>`. The full build, the `--locked` gate, and conformance run **in
CI**. ⚠ Changes under `lowering/` reaching the reifier need the **full
`-p ken-interp` suite**, and a cross-crate declaration-text oracle is invisible
to targeted builds — flag if you touch that surface.

## Hard-stop

The chain's hard-stop count of record is **7** (`RT-NATIVE-FNSPLIT` holds it;
that line wins any disagreement). **Next research pull is #9.**

**Stop and escalate — do not reinterpret a deliverable to fit what is
buildable** — if any of these holds:

- an anchor in §Settled does not hold on your base;
- **D3 finds a per-variant ordinal disagreement you cannot resolve in the
  lowering** (planner ordinals are authoritative; if adapting the lowering is not
  behaviour-preserving, that is a hard-stop);
- D5's guard cannot be made to redden on an added field without production
  changes beyond declarative types;
- threading forces any N1–N4 pin to move.

⭐ **Report the measurement and name the discriminating property; do not infer a
boundary from a size.** That is the ring's own lesson from #7 — it inferred
"collapses into B2F" from the *size* of the carrier when the deciding property
was *totality*, and retracted it unprompted. **A big number is not a structural
property.**

**A member you cannot cover is a hard-stop, not a partial landing.**
