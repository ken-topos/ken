# RT-CONTSRC-PRODUCER-LOCAL — the producer-local continuation source coordinate

**A value created mid-body is a third availability class. Continuation
specialization can name entry values and generated-context captures, and cannot
name a host-effect result or a `Match` case binder — so the environment for
those edges never closes and the specialization is never committed. This node
adds that coordinate domain.**

**Owner:** Team Runtime. **Size:** L.
**Node:** `docs/program/issues/RT-CONTSRC-PRODUCER-LOCAL.md`.
**Risk:** medium-high — it widens a planner/ABI representation, and the
Architect has already ruled the naive shape unlawful.

**Authority:** Steward ruling 2026-08-05 at [[RT-DECL-CLOSURE-PORT]] checkpoint
`1f`, on measurements `evt_5kws532ac99c9` and `evt_5ngh190h9b1k5` and the
Architect representation gate `evt_75k8cydbj5127`.

---

## 1. Base and fixed inputs

**Governing base: exact `179af86350ba7191935fcc9ff902bb166c954339`**, on branch
`wp/RT-DECL-CLOSURE-PORT-typed-units`. **Continue that branch.** It is not on
`main`, so a fresh branch cut from it gains no independent mergeability and
risks losing the proved lineage. `D7` checkpoints 1, `1b` and `1c` are proved
substrate and are preserved byte-identically.

⛔ **Rebase, merge or cherry-pick of `fb8fd881`, `430798bf`, `548682c3`,
`42ccd8ec` remains banned** — competing historical implementations; importing
one reintroduces the role/disposition-derived schema the host-effect ruling
ruled false.

| path | blob at `179af863` |
|---|---|
| `crates/ken-runtime/src/cranelift_backend/planning/static_transition.rs` | `e66e423c9991a406694a9e1a59d58906f3f94929` |
| `crates/ken-runtime/src/cranelift_backend/lowering/core.rs` | `949a4bea2cd53c840ba63f3320dbfc3f2eb5550a` |
| `crates/ken-runtime/src/cranelift_backend/lowering/units.rs` | `fc6981119d0f2bb3023d4a4abc6b46fabfcb2771` |
| `crates/ken-cli/tests/rt_parity_native.rs` | `b2df2bbd00644b907cae5d05efa76edd9df1b3f2` |

**Baselines at that base, per-row and not as totals:** `ken-runtime` lib
**718 passed / 2 failed** (two standing reds), `ken-elaborator` lib **108 / 0**,
`D0` parity **1 passed / 6 failed** with `buffer_freeze` the passing row.

## 2. The measured gap

`ContinuationInputSource` (`static_transition.rs:410`) is `Parameter` /
`LexicalCapture` / `SeedCapture`, and its enclosing record requires an
**entry-ABI coordinate**. `continuation_owner_entry_sources` enumerates exactly
`parameters + captures`; every carrier, ownership, storage-owner, affinity and
equality check derives from that exact `AbiSlot`.

A host-effect result or a `Match` case binder is created **after entry**. It is
neither a parameter/capture nor the unit's outgoing `Result` convention slot.
The emission seam agrees independently: its exhaustive two-class resolution
covers an entry value in its root owner and a value captured by a generated
context.

**Closure-edge census at the base:** 34 case-binder-only, 4
effect-result-plus-case-binder, 1 `Construct`-only. The four mixed edges span
all six failing `D0` rows.

## 3. Deliverables

- **`D0` — the delta-free baseline.** Before any delta, record which rows are
  green at `179af863`, per row. ⛔ A measurement carrying your own delta cannot
  produce it.

- **`D1` — the two coordinate domains, separated.** Not a fourth enum arm.
  1. **Entry ABI source** — existing owner + parameter/capture position and its
     slot-derived contract, **unchanged**.
  2. **Producer-local source** — an exact structural binding identity in the
     producer body, with planner-derived carrier / ownership / storage /
     affinity, **and a separate immediate-availability projection (`D2b`)**.

  The source-position type is a **closed sum**: an entry position must not be
  representable as a local binding, or the reverse. ⛔ No default arm.

  > ⛔ **THIS CLAUSE WAS AMBIGUOUS AND IT COST A CHECKPOINT. Corrected
  > 2026-08-05.** It used to read "an exact emission-time locator into the
  > environment that actually contains it" — and **never said which
  > environment.** `D2` reasonably read it as the *semantic* environment and
  > populated a scope-relative `(environment_origin, environment_index)`. The
  > emission seam indexes something else entirely. Those are two coordinate
  > spaces, and a term that silently spans both is the defect.
  >
  > ⭐ **Root identity and immediate availability are DIFFERENT FACTS** and this
  > frame now names them separately. `ProducerLocalBinding` and its value
  > contract are **never** rewritten as an ABI position.
  >
  > This is a Steward framing defect, and a different kind from this campaign's
  > earlier four: not a false law, but a load-bearing term left spanning two
  > spaces. Nobody could have discharged it as written.

- **`D2` — coverage of both binding kinds. The IDENTITY AND VALUE CONTRACT are
  ACCEPTED PRESERVATION at exact `e6d4f085`** (Architect `evt_38yd5sd1ht0kk`).
  The host-effect result **and** the exact `Match` case binder, as **distinct**
  structural bindings. A later common local-binding representation may subsume
  them; this node does not assume one.

  ⛔ **`D2` IS NOT COMPLETE. It is REOPENED as an availability boundary**
  (Architect `evt_44k69b55vhek2`, 2026-08-05) — see `D2b`, which `D3` waits
  behind. ⛔ This frame said "COMPLETE at exact `e6d4f085`" for about fifteen
  minutes; that claim is **deleted, not qualified**, because a reader who found
  it would take `D3` as unblocked.

  ⛔ **A `ComputationalMatch` case binder run is NOT homogeneous, and this frame
  did not say so.** The case environment is ordered `[recursive IH binders,
  constructor argument binders, outer environment]`. For
  `ordinal < recursive_positions.len()` the binder is a **recursive IH**; the
  rest are **constructor arguments**, whose carrier is `abi::result_carrier` on
  the **scrutinee's** shape through `slot_referent_affinity`. Identity stays
  `(case body, binder ordinal)` — the ordinal's **role** is read off the case
  header. ⛔ There is no blanket `ValueWord` rule and no `ResultPhase` to
  `AbiCarrier` map; the first `D2` candidate invented one and was blocked
  (`evt_9krmbv834z9p`).

  ⛔ **The recursive-IH prefix takes no contract here and stays `Open`.** An IH
  is a compiler-only `LoweringEnvironmentBinding::StaticWorker` with no runtime
  word, tag, descriptor or carrier. Leaving it `Open` is **declining to
  represent**, not defaulting. It is [[RT-CONTSRC-CALLABLE-CONTRACT]]'s scope.

- **`D2b` — the closed IMMEDIATE-AVAILABILITY projection. NEW, and `D3` waits
  behind it.** Authority: Architect `evt_44k69b55vhek2` on the measured D3 stop
  `evt_72pyjkamqsewc`. ⛔ This is **not** the callable-contract successor, **not**
  `D4` admission, and **not** a new carrier or convention lane.

  **The measured defect.** `D1`'s root coordinate is not consumable at emission:
  no planner-issued projection carries a producer-local binding through
  intervening lexical shifts to the exact retained emission seat, nor proves its
  capture into an ABI-only generated emission context.

  ⛔ **Do NOT pin "`producer_env` is always the emitting function's ABI operand
  run."** The 61 seam records prove the shape of the **currently admitted**
  population only. There are **two materially different consumers**: the
  retained-frame seat passes the current `LoweringEnvironmentBinding` run, and
  the detached/generated-context seats read a function-local ABI operand run.
  ⭐ The `Specialization` row is what discriminates — `immediate_slot` 1 and 2
  against `source_abi_position` 0 and 1, so the index is the generated context's
  **own** operand position, not a root ABI position.

  **One planner-issued, closed projection, separate from root provenance, with
  two arms:**

  1. **Current lexical availability** — keyed to the exact causal
     producer/emission occurrence, carrying the exact environment origin **plus
     post-shift index**, derived by the forward semantic environment walk.
     Lowering may consume it **only at that exact seat while holding that exact
     lexical environment**.
  2. **Generated-context capture availability** — keyed to the exact generated
     emission owner/context, carrying its **declared immediate capture slot**.
     ⛔ It exists **only after** the full producer-local root coordinate is proved
     present in that context's ordered capture projection.

  Entry ABI availability remains its existing case, untouched.

  **Lawfulness of a generated-context capture:** carrying a producer-local value
  as a declared capture is lawful **only when the caller's exact current-lexical
  projection proves the value is already available at that call seat**. This does
  **not** widen the predeclared function's entry ABI and does not claim the
  mid-body value existed there.

  ⛔ **If no exact current-lexical source is available, planning DECLINES or
  REJECTS** per the existing candidate/program boundary. Lowering must not
  reverse-search, infer a shift, reuse `Result`, or fabricate a capture.

  **Fail closed on all five:** wrong emission origin · wrong post-shift index ·
  wrong generated owner/context · missing full-coordinate capture membership ·
  wrong immediate slot.

  **Two discriminators, and the first exists to defeat a specific vacuity:**
  - one with **at least one intervening binder**, so "introduction index equals
    emission index" cannot satisfy the controls by coincidence;
  - one **generated-context** case where root and immediate positions **differ**.

  ⭐ **The implementer withheld six of nine consumer sites rather than land
  them**, on the grounds that the seam cannot reach empty while emission is
  blocked and a partial would leave a seam no longer naming its own remaining
  work. That judgement is correct and is now frame law: ⛔ **do not land a
  partial `D3`.**

- **`D3` — the consumers, each handled explicitly. THE COUNT IS TEN, NOT
  THREE.** ⛔ **HELD until `D2b` is populated and mutation-proved.** The
  `entry_abi_pending_producer_local` seam stays intact until then.
  ⛔ **Corrected 2026-08-05 by `evt_1srfqjmkp5eh8`; the original three
  below were a Steward guess and `D3` must not be cut on that number.**

  ⭐ **`D3`'s exhaustiveness is scoped to the CURRENT closed value-slot contract
  version, deliberately** (Architect gate `evt_38yd5sd1ht0kk`). Build against
  one contract. [[RT-CONTSRC-CALLABLE-CONTRACT]] adds a second arm later and
  **owns revisiting every exhaustive consumer** — that is named in its scope, so
  `D3` is not incomplete for stopping here. ⛔ Do not pre-build a second arm, and
  do not write `D3`'s exhaustiveness as if the value-slot contract were the
  permanent whole.

  The three this frame named:
  - `validate_continuation_source_slot` re-derives the same arm and contract.
    ⛔ No exemption for the new arm — it is the only exact validator.
  - Generated-context capture lookup compares the **full** source coordinate.
  - The emission resolver handles the local arm **explicitly**. ⛔ No default,
    no fallthrough.

  The seven the ring measured beyond them: the ABI plane's
  `append_continuation_descriptor`,
  `append_continuation_context_descriptor` and
  `AbiPlane::validate_continuation_specializations`, and the two
  view-agreement checks.

  **The live enumeration is in the tree, not in this list.** The seam function
  `entry_abi_pending_producer_local` exists to be `grep`ed: it *is* what `D3`
  owes, and it is deleted when the list empties. ⇒ **Size `D3` from that grep,
  never from this frame's prose.** A frame-side count of a code-side population
  goes stale the moment the code moves, which is how it was wrong the first
  time.

- **`D4` — broad admission, stated as SET EQUALITY over the census's explicit
  unit.** ⛔ **Recut 2026-08-05 by Architect gate `evt_38yd5sd1ht0kk` on the
  census `evt_qttaeebtzjkt`. The earlier wording — "all newly representable
  candidates may lawfully intern" — is REPLACED, not qualified.**

  **`D4` is not an all-programs / all-source-kinds closure theorem.** It
  completes over the closed contract domain this node owns, with an exact named
  declined set. The unit is **one call to
  `exact_continuation_source_environment`**, identified by **program fingerprint
  + consumer owner + continuation origin + producer construct origin + recursive
  position + closure origin**.

  | set | contents |
  |---|---|
  | `C` | all **83** `(identity, full required vector)` instances |
  | `V` | the **80** whose entire required vector is closed under the current value-slot authority, including the two empty vectors |
  | `R = C \ V` | exactly **3**: `OPEN[ih-binder]`, `OPEN[let-value:Construct]`, `AMBIG2[let-value:If]` |

  **`D4` discharges when the post-admission census proves `interned = V` and
  `declined = R`** — no additional residual, ⛔ no member/corpus/closure
  predicate, ⛔ no first-`Open` classification. The full-vector walk remains the
  authority.

  ⛔ **The program fingerprint is load-bearing, not decoration.**
  `StaticOriginId`s are allocated per compile, so without it edges from
  different fixtures collide on identity and the census silently undercounts —
  measured: a first pass reported 58 identities of which six were collisions.

  ⛔ **Call the three "outside-this-contract-domain residuals", never
  "unrepresentable"** (Architect, same gate). `Construct` and joined-`If` are
  simply not authorized by this node; nothing here claims no future authority
  can represent them.

  ⭐ **All 17 parity instances are in `V`** — that is the critical-path fact.

## 4. Acceptance criteria

- **`AC-1` — the linked row closes.** The `D0` row
  `buffer_allocate_malformed_capacity_narrows_to_invalid_bounds` reaches the
  real producer and returns `InvalidBounds` at the exact `264 -> 262 /
  position 1` consumer, with shared-host dispatch count **zero**. Removing the
  carried-capacity arm recreates the refusal at that exact seat.

  ⛔⛔ **THE SIX RED ROWS ARE TWO POPULATIONS, NOT ONE — measured at `D0`
  (`evt_1srfqjmkp5eh8`).** The `AC-1` row refuses at a different site from the
  other five:

  | rows | refusal |
  |---|---|
  | `buffer_allocate_malformed_capacity_narrows_to_invalid_bounds` (`AC-1`) | `Match: scrutinee is not a constructor value` |
  | the other five | `ComputationalMatch: tree-producing match scrutinee is not Bool or a constructor` |

  ⇒ **Greening the five does NOT discharge `AC-1`, and greening `AC-1` says
  nothing about the five.** Five-and-one at two distinct sites is invisible in
  the `1 passed / 6 failed` total, which is precisely what a total is for.
  Report the two populations separately, always.
- **`AC-1b` — per row, never a total.** Every row green in `D0` is still green,
  stated per row. ⛔ A pass/fail count is not evidence: it reads identically
  before and after, and that is what hid two of this campaign's false laws —
  and, per `AC-1` above, it also hides that the red rows are two populations.
- **`AC-2` — the closed sum is enforced by the type, not by convention.** A new
  source kind must be unable to compile until every one of `D3`'s three
  consumers assigns it. ⛔ No wildcard arm.
- **`AC-3` — every instance in `V` is accounted for individually.** ⛔ **The
  "34 newly-interning edges" this AC used to name is SUPERSEDED.** That figure
  predated `D2` and counted only declining edges; the census
  (`evt_qttaeebtzjkt`) enumerates the **whole candidate population** at the unit
  `D4` states, which is why 83 replaces 39 — and it found an instance the old
  partition did not contain at all, the `AMBIG2[let-value:If]`.

  Name each of the **80** instances in `V` and show for each that interning is
  lawful. ⛔ An aggregate "no regressions" claim does not discharge this — a
  differential over an aggregate passes while one of N contributors defects.

  ⛔ **A case-binder position is not one population.** It resolves to a recursive
  IH or to a constructor argument, and those take different contracts (`D2`).
  An account that treats the case-binder run as homogeneous is the exact defect
  the `D2` gate blocked, one level up.
- **`AC-4` (no-regression).** Workspace green **in CI** — ⛔ never a local
  `--workspace` run (`COORDINATION §12`).
- **`AC-5` — `1c`'s converse survives. DISCHARGED AT `D4`, NOT BEFORE.** The
  interned-to-member law and its four mutation controls remain intact and
  **non-vacuous**: show each still fails when its target is mutated.

  **The timing is not bookkeeping.** `AC-5` exists because **broad admission
  changes the interned population**, which is the condition under which a
  control silently goes vacuous. Until `D4` admits something, that condition is
  unreachable and a "controls still green" report would be **true and
  meaningless** — it would measure an unchanged population and read as having
  cleared the risk. ⛔ Do not accept `AC-5` from any deliverable that admits
  nothing; `D1` correctly declined to claim it.

## 5. Banned scope

- ⛔ **A fourth `ContinuationInputSource` case** while the enclosing record
  still requires an entry-ABI coordinate. The Architect rejected this shape
  explicitly.
- ⛔ **Claiming a mid-body value exists at function entry** — widening
  parameters/captures to seat it, inventing an entry position, or reusing
  `AbiSlotKind::Result` (a different boundary direction).
- ⛔ **Exempting the new arm from `validate_continuation_source_slot`**, or
  using `immediate_slot` alone and discarding root provenance.
- ⛔ **Any route-modality or edge-selection authority.** Broad admission
  dissolves the need. If you find yourself needing one, that is a finding about
  `D4`'s scope — hard-stop and return it.
- ⛔ **Corpus identity, closure identity, first-`Open` reason, or planned-member
  status as a predicate.** All four are forbidden substitutes for a real
  authority, and `member=true` is measured constant across all 612 declines and
  all 489 interns, so it discriminates nothing.
- ⛔ **Special-casing closure `381`** or any named closure.

## 6. The standing methodological requirement

**Validate the full required environment as a vector. First-`Open`
classification is not a population oracle.**

This is not general advice; it is the specific defect that produced a false
minimality ruling on 2026-08-05. "6 effect edges equal the 6 failing rows" was a
pair count short-circuited at the first `Open` position, compared against a 161
that was in a different unit. The effect-result-only population is **zero**.

⇒ Every census this node produces **states its unit** and answers *what does
this edge require*, never *where did it first stop*.

## 7. Hard stop

Stop and report, with the concrete edge, if:

- a lawful producer-local coordinate cannot be expressed without one of the five
  exits the Architect closed;
- broad admission turns out to require an edge-selection authority after all; or
- closing the case-binder binding perturbs a row that `D0` recorded green, in a
  way the per-row evidence cannot account for.

⛔ Do not absorb any of these and do not work around them.
