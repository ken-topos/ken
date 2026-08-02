# WP frame — `RT-CONTSPEC-ACTIVATE` (ContinuationSpecialization seam 2 of 4)

Node: `docs/program/issues/RT-CONTSPEC-ACTIVATE.md`. Campaign:
`docs/program/16-recursive-descent-retirement.md`. Owner: runtime ring.
Authority: Architect ownership/sizing ruling `evt_1yymw1gdszpbs`, outcome (c),
seam 2. **Recut 2026-08-02** on the Architect's interface ruling
`evt_3xj4eqwqmn46n` as corrected by `evt_66t42tapvdbsj`, and the Steward's
baseline measurement `evt_2zhx69f2fw07w`.

This is the seam that turns the mechanism on. Seam 1 put the accepted helper on
the landed substrate and proved it changed nothing. This seam makes the
already-planned continuation population **reachable by the emitter**, and proves
that exact population became emission.

> ## RECUT NOTICE — the 37-row census population is RETIRED as this seam's subject
>
> The first cut of this frame told you to select 37 rows from the corrected `D4`
> census and activate against them. **That instruction was wrong and it is
> withdrawn.** Measured at `origin/main = 68c72c75`:
>
> - `active_emission_owner` occurs nowhere in the repository, not even in docs.
> - `lowering boundary use has no active emitted owner` and
>   `JoinArm boundary-use ledger missing` occur **only** in this frame and in
>   `docs/program/wp/RT-CONTSPEC-LOWER-D8.md`. Zero hits under `crates/`.
> - The census's test names do resolve — 24 `fn b2f_d9_*` / `fn c1_d3_*` in
>   `crates/ken-runtime/src/cranelift_backend/lowering/core/tests/constructors.rs`
>   — and seam 1's `AC-2` run at `767e7795`, which is in `main`'s lineage, was
>   **589 passed, 0 failed, 1 ignored**.
>
> ⇒ **Those rows are green on the lawful base.** They are not a backlog of
> failures for an activation to fix.
>
> **Why the first cut got there.** The 138-row census records first refusals from
> a run against the held `1aef3192` lineage, which carried an
> `active_emission_owner` mechanism that was never merged and that seam 1
> forbids merging. Seam 1 correctly treated that lineage as an **oracle, not a
> base, for its code**. This frame then used the same lineage's **census** as a
> live source-level selector — the identical error one level up. `D4` is a
> historical first-refusal record. It can tell you what once failed and why; it
> **cannot name a current source authority**.
>
> **Checkpoint 1's `751fa18d` stands and is not redone.** Its tree equals its
> base exactly, it contains no production delta, and it is an accurate record of
> what the census says. It is preserved as history. It is **not** this seam's
> population.

## The live subject, stated positively

The lawful base plans a continuation population and then deliberately declines to
emit it. `contspec_planner_closes_ordered_keys_units_and_causal_edges_dormantly`
(`planning/static_transition.rs:11770`) proves both halves in one test:

| fact | where |
|---|---|
| `plan.continuation_specializations.len() == 2` | the test, line 11772 |
| `plan.continuation_specialization_calls.len() == 2` | the test, line 11773 |
| 2 distinct call targets, none orphaned or conflated | the test's `D5` assertion |
| `abi.continuation_descriptors.len()` equals the specialization count | the test's `D1` assertion |
| `emittable_units()` iterates `self.abi.descriptors` **only** | `static_transition.rs:4788` |
| `emittable_call_edges()` excludes the planned continuation-call population | `static_transition.rs:4743` |

So the descriptors are built, validated, and held in a **separate vector**
(`continuation_descriptors`, `planning/static_transition/abi.rs:456`) that no
emitter accessor projects.

⇒ **The live unfinished behaviour is this: a planner-issued continuation
specialization and its causal call both exist and are validated, but the emitter
can declare no target for them and can emit no direct call before the producer
result loses callable identity at the join.**

This seam closes exactly that, and nothing else.

> ### The subject is WRONG-BUT-GREEN, so a green suite proves nothing here
>
> The baseline is expected to be green overall. The dormancy is deliberate and
> asserted. ⇒ **Every acceptance criterion below is written against a control
> that can fail on the candidate**, and the population counts are the discriminator:
>
> ```text
> base:       planned units = 2, planned calls = 2, emittable continuation units = 0, emittable continuation calls = 0
> candidate:  planned units = 2, planned calls = 2, emittable continuation units = 2, emittable continuation calls = 2
> ```
>
> The planned side must not move. If it does, this seam changed the plan, which
> is seam 3 and 4 territory and is banned here.

> ### The 37/37 collision — still live, and still not the same hazard
>
> "37" names two disjoint populations in this campaign: `producer callable
> identity is not a Closure` is 37 rows owned by `CONTSPEC planner` in the
> census, and the Architect's matrix separately assigns 37 rows to lower. They
> share a number and nothing else. **Neither is this seam's population.** The
> warning is retained only so nobody re-derives a selection from either.

## Fixed inputs

Measured at `origin/main = ab6b89fc`.

| input | measured value |
|---|---|
| seam 1 | `RT-CONTSPEC-ASSEMBLY`, `merged` |
| preserved history | `751fa18dee9b51155d6337a2223459e36e2c16a6` — the 37-row census record. Preserved, not consumed |
| the planned population | `PlannedContinuationSpecialization` (`static_transition.rs:622`), `PlannedContinuationSpecializationCall` (`static_transition.rs:640`) — both **private** |
| the call token | carries `producer_owner`, `producer_result_origin`, `producer_construct_origin`, `producer_alternative`, `call_site_sequence`, `target`, `worker` |
| the dormancy witness | `contspec_planner_closes_ordered_keys_units_and_causal_edges_dormantly`, `static_transition.rs:11770` |
| the emitter projections | `emittable_units()` `:4788`, `emittable_call_edges()` `:4743` |
| lowering consumption site | `crates/ken-runtime/src/cranelift_backend/lowering/units.rs` |
| the accepted helper | `CheckedFrameBranchScope`, landed by seam 1; it changes no emission authority |
| baseline suite | `scripts/ken-cargo test -p ken-runtime --lib` |

Reproduce, read-only:

```sh
git rev-parse HEAD
git grep -n 'fn emittable_units\|fn emittable_call_edges' -- 'crates/**'
git grep -n 'continuation_descriptors' -- 'crates/ken-runtime/src/cranelift_backend/planning/static_transition/abi.rs'
```

## Two preconditions on every suite run, carried from seam 1

Both produced a false hard stop on seam 1 (`evt_3q972fhrnsr0b`, ruled
`evt_1pt7rmmw2k5d0`). Neither is optional here.

1. **Prove the tree in the same shell as the run.** `git rev-parse HEAD`
   immediately before the suite; quote its output as the base. A `git switch`
   onto a branch held by another worktree fails **silently** inside an `&&`
   chain and the chain runs the suite in the old tree.
2. **Build before you test.** `crates/ken-runtime` is
   `crate-type = ["rlib", "staticlib"]` and `cargo test --lib` never emits
   `libken_runtime.a`. Without it, `ken_runtime_staticlib()`
   (`object_linker_packaging.rs:1211`) finds no archive and ~40 rows fail with a
   `Toolchain` error whose text names ken-host. Run:

   ```sh
   scripts/ken-cargo build -p ken-runtime
   scripts/ken-cargo test  -p ken-runtime --lib
   ```

⇒ A `Toolchain`-stage `ObjectLinkerPackagingError` is an environment finding,
not a baseline finding.

## The two bounded exemptions to the prior-slice freeze

Seam 1 froze six prior-slice surfaces. **That freeze holds here with exactly two
narrow exemptions.** Both are for reachability, not for authority.

### Exemption 1 — the activation projection

Granted by the Architect at `evt_3xj4eqwqmn46n` item 1, because the planned
population is private and lowering cannot otherwise reach it without re-deriving
planner facts:

> `planning/static_transition.rs` may add an **activation projection only**.

### Exemption 2 — the facade route, added by the 05:12 recut

Granted at `evt_4hrnfc81h6t81`. **The first cut of this frame froze
`planning.rs` while `D2` required the typed identity to survive into lowering.
Those two are mutually unsatisfiable, and hard stop 2 fired on exactly that.**

Measured at base `86c7cedf`:

| fact | value |
|---|---|
| `planning.rs` blob at base | `0b2fdb22a05fbf58bd0cfed22f308011b69fb969` |
| `planning.rs:14` | `mod static_transition;` — **private** |
| `ContinuationSpecializationId` | `pub(in crate::cranelift_backend)`, `static_transition.rs:406` |

The item's visibility is backend-wide, but it sits inside a **private module**,
so a sibling `lowering` cannot *name* it by path.
`UnitBundle::continuation_functions: BTreeMap<ContinuationSpecializationId,
FuncId>` therefore has no lawful route without a facade. ⇒ **Deleting the re-export while keeping the typed map is not
a repair; it is the contradiction restated.**

> `planning.rs` may perform **namespace wiring only** — re-exporting the opaque
> typed activation identities and views that lowering needs. **No derivation,
> selection, construction, validation, or mutation may move there.**

### What both exemptions still exclude

`planning/static_transition/abi.rs`, semantic IR, `boundary_value.rs`,
`boundary_value_clif.rs`, and planner derivation/interning/key/ABI construction
inside `static_transition.rs` — all frozen at their `main` blobs.

⇒ **The projection may expose what the planner already decided. It may not
decide anything, and the facade may not compute anything.**

## The integration boundary, ruled at `evt_4hrnfc81h6t81`

Once the typed projection is lawfully routable, the seam is exactly this and
stops here:

1. `static_transition.rs` completes the read-only projection **from already
   validated facts only**.
2. `units.rs` **declares and defines** each projected continuation target from
   its exact projected definition, descriptor, slot, and input contract, and
   resolves each projected causal token to that declared `FuncId`.
3. Ordinary unit body emission carries its actual `PredeclaredFunctionId` as
   **function-scoped defining-unit context** into the producer-alternative
   lowering path. That context is compared with the token's `producer_owner`.
   ⇒ It is **not** a resurrected global historical selector.
4. At the exact producer construct, alternative, and sequence named by the
   token, lowering **removes the exact claim once**, emits the resolved direct
   call before the identity-erasing join, and rejects wrong owner, absent or
   duplicate claim, wrong target, and leftover claims.
5. Completion compares the exact **planned / declared / defined / called** sets.

**Still forbidden:** positional ordinals, symbol parsing, source re-enumeration,
and any second arm authority. `D3` necessarily integrates `lowering/units.rs`,
`lowering/core.rs`, and lowering's per-function emission context — that is
in scope and is not a finding.

## Deliverables

- **D1 — the activation projection.** In `planning/static_transition.rs`, a
  read-only, unmintable view of:
  - each already-validated continuation unit with its exact
    `ContinuationSpecializationId`, its immutable planner key facts, and its
    already-validated ABI descriptor, slots, and input authority; and
  - each already-validated continuation call token with the full producer tuple
    and exact target.

  It **revalidates plan/ABI agreement and fails closed.** Lowering may neither
  enumerate source syntax to reconstruct the population nor invent an id, owner,
  descriptor, or call.

  ⛔ **`D1` is NOT complete at `4a058dbb`** (ruled `evt_4hrnfc81h6t81` item 4).
  That projection exposes only `id()` on the unit view and `target()` plus
  `call_site_sequence()` on the call view. **The immutable specialization key
  facts, the validated descriptor/slots/input authority, and the call's full
  producer tuple are all still owed.** Without them the current source cannot
  define the continuation body, locate the exact producer alternative, compare
  the actual defining owner, or consume the exact claim — and every one of those
  is `D3`'s content. A checkpoint that declares symbols has not discharged this.

- **D2 — the lowering consumption.** `lowering/units.rs` consumes those views to
  forward-declare one continuation target per planned specialization and resolve
  the exact direct call per causal token, **emitted in the producer alternative
  before the identity-erasing join.**
  Keep continuation identity typed: do **not** alias a
  `ContinuationSpecializationId` to an ordinal, and do **not** fabricate a
  `PredeclaredFunctionId`. The typed identity reaches lowering through the
  exemption-2 facade, which is now permitted.

  ⛔ **`D2` is NOT complete because symbols were declared.** `declare_unit_bundle`
  declaring continuation symbols while `define_unit_bodies` still iterates only
  `emittable_units()` leaves the continuation map with **no lawful defining or
  call-resolution consumer**. That is the expected consequence of `D1` being
  incomplete — it is **not** authority to invent a consumer in lowering. `D2` is
  discharged when each projected target is declared **and defined** from its
  projected contract, and each causal token resolves to that declared `FuncId`.

- **D3 — affine consumption and owner agreement.** Exactly **one** affine
  consumption of the exact planned continuation-call token. "Active emitted
  owner" means the exact unit currently being defined, carried as emission
  context and compared against the producer owner on the planner claim.
  ⇒ **Do not resurrect the historical `active_emission_owner` field, and do not
  add a lowering-only `JoinArm` token beside the call token.** The token's
  producer construct, alternative, and sequence already name the causal arm; a
  second ledger would duplicate authority. This is the Architect's item 5.

- **D4 — the behavioural fixture at the emission seam.** Derived from the
  existing `contspec_plan` shape, observing at emission rather than by
  source-text inspection:
  1. the exact nonzero planned unit and call population;
  2. one declared and defined continuation target per planned specialization;
  3. one direct call per exact causal token, in the producer alternative, before
     the join;
  4. an answer that **depends on the selected target** — redirecting a token to
     the other same-shaped target changes or fails the result;
  5. affine rejection when the same token is consumed twice, and rejection under
     the wrong producer owner.

- **D5 — the before/after population disposition.** The four counts above on
  this seam's own base and on the candidate, plus a single aggregate line
  stating that the rest of the suite's pass/fail set is unchanged.

## Acceptance criteria

- **AC-1 — the planned population is unchanged and the emittable population
  moved.** On the candidate: planned units 2, planned calls 2, emittable
  continuation units 2, emittable continuation calls 2. On the base, the last
  two are 0.
  *Control:* `D5`'s four counts, both trees, `git rev-parse HEAD` shown in the
  same block. **A planned count that moves fails this AC** — that is a plan
  change, not an activation.

- **AC-2 — the target discriminator fails when the target is wrong.** Redirect a
  causal token to the other same-shaped target; the `D4` fixture must change or
  fail its answer.
  *Control:* the mutation run and shown red, then reverted. **A fixture that
  stays green under target redirection is not observing emission** — it is
  observing that something was declared, which is `AC-1`'s job.

- **AC-3 — the affine and owner controls fail when mutated.** Consume the same
  token twice; the affine control must go red. Emit under the wrong producer
  owner; the owner control must go red.
  *Control:* both mutations run and shown red, then reverted. Commit the real
  fix before any mutation proof and reset after.

- **AC-4 — the frozen surfaces are blob-identical to the merge base, with two
  named and blob-bounded exemptions.**
  - **Frozen, blob-equal, no exceptions:** `planning/static_transition/abi.rs`,
    `planning/static_transition/semantic_ir.rs`, `boundary_value.rs`,
    `boundary_value_clif.rs`.
    *Control:* `git rev-parse <candidate>:<path>` against
    `git rev-parse <merge-base>:<path>` for each.
  - **Exemption 1 — `planning/static_transition.rs`**, activation projection
    only. *Control:* the underlying **specialization and call populations are
    byte-or-claim identical before and after** — the diff adds a view and
    changes no derived value.
  - **Exemption 2 — `planning.rs`**, facade namespace wiring only. Base blob is
    `0b2fdb22a05fbf58bd0cfed22f308011b69fb969`.
    *Control:* the whole diff against that blob is `pub use` / `pub(in …) use`
    re-export lines and nothing else. ⛔ **Any `fn`, `impl`, `struct`, `enum`,
    control flow, or literal in `planning.rs`'s delta fails this AC outright** —
    the facade routes names and computes nothing.

- **AC-5 — no test outside the `D4` fixture changes status in either
  direction.** A row that starts failing is a regression; a row that starts
  passing means the activation reached outside its population.
  *Control:* the full pass/fail set on this seam's base against the candidate's,
  differenced. **The symmetric difference must be exactly the `D4` fixture.**

- **AC-6 — CI green** on the merge. Workspace-green means green in CI, never a
  local `--workspace` run.

## Banned scope

- **No change to the planned population.** No new specialization, call, key, or
  descriptor, and no change to any existing one. This seam makes the plan
  reachable; it does not extend it.
- **No planner or ABI semantic repair.** A planner- or ABI-worded refusal on the
  lawful assembly is a **new interface fact**: route it, do not repair it.
- **No edit to `planning/static_transition/abi.rs`**, semantic IR,
  `boundary_value.rs`, or `boundary_value_clif.rs` (`AC-4`). `planning.rs` is
  open **only** for re-export lines under exemption 2.
- **No second authority in `lowering`.** No reconstruction of the population
  from source syntax, no minted id, owner, descriptor, or call, and no
  lowering-only ledger beside the planner's call token.
- **No D7 population expansion.** The 17 ledger and representation rows are seam
  3.
- **No selection from the `D4` census**, and no reuse of `751fa18d` as a
  population. It is preserved history.
- **No merge, rebase, or wholesale cherry-pick of any preserved object.**
- **No test asserting facts about source or documentation lines** (operator test
  policy). `D5` is a review artifact; `D4` is a behavioural fixture and must
  observe emission, not source text.

## Contention

Runtime is single-threaded. This seam edits `lowering/` plus the one bounded
projection in `planning/static_transition.rs`. Take the shared build lock for
`AC-1`/`AC-5`; probe without blocking first. **Targeted only:**
`scripts/ken-cargo test -p ken-runtime --lib`. **Never `--workspace`** — the
full-workspace build, the `--locked` gate and conformance run in CI.

## Sizing

**Size `M`.** The recut is materially smaller than the first cut: the population
is 2 units and 2 calls that already exist and are already validated, not 37 rows
to be selected and moved. The work is a projection, a consumption, and a fixture.

⇒ **Commit at these checkpoints and post the exact SHA at each.** The list is
restated by the 05:12 recut, because the first two were accepted before their
deliverables were complete:

1. **`D1` complete** — the facade route, plus a projection carrying the
   specialization key facts, the validated descriptor/slots/input authority, and
   the call's **full producer tuple**. Not just `id()` / `target()` /
   `call_site_sequence()`.
2. **`D2` complete** — each projected target declared **and defined** from its
   projected contract, and each causal token resolved to that declared `FuncId`.
   Declaring symbols alone is not this checkpoint.
3. `D3` affine and owner controls, `D4` fixture, their mutation proofs, `D5`.

⛔ **Do not accept a checkpoint against its label. Accept it against the
deliverable's text.** Checkpoints 1 and 2 were both accepted on a reading of
their headline rather than their content, and hard stop 2 is what surfaced it
two checkpoints later.

Work already on the branch is preserved and reusable: `4a058dbb` (partial
projection), `d7291746` (typed identity restored). Neither is discarded; both
are extended.

If any single checkpoint runs past an hour, stop and route. That is a sizing
finding and the recut is the Steward's.

**Expect to end your turn at each checkpoint.** Post the SHA and wait for the
leader's acceptance rather than assuming one turn spans all three.

## Hard stops

Stop and route to the Steward, do not improvise, if any of these hold:

1. **The projection cannot expose the population without deciding something** —
   without minting an id, choosing a descriptor, or re-deriving a planner key.
   That is an interface fact and it is the most valuable thing this seam can
   discover. Report the exact fact needed.
2. **The activation cannot be made without editing a frozen surface other than
   the one exempted `static_transition.rs` projection.** Not a small exception.
3. **A planner- or ABI-worded refusal appears on the lawful assembly.** New
   interface fact; route it, do not repair it.
4. **A planned count moves** (`AC-1`). The projection changed the plan and the
   boundary is wrong.
5. **A `D4` control stays green under its own mutation** (`AC-2`, `AC-3`). The
   control is not measuring what it claims and the activation is unproved.
6. **A row outside the `D4` fixture changes status in either direction**
   (`AC-5`).
