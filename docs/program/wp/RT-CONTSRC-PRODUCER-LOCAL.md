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

  ⛔ **READ THE LAW WITH ITS RATIONALE — 2026-08-05, Steward.** The banned thing
  is **a fragment that leaves the seam no longer naming its own remaining
  work.** It is *not* a ban on cutting `D3` into smaller whole deliverables. A
  landing that **retains** `entry_abi_pending_producer_local` with a non-empty,
  truthful enumeration does not commit the defect this law exists to prevent.
  ⇒ `D3` is recut into `D3a` and `D3b` below; **each is whole**, and the law
  stands unweakened over both. A blanket imperative whose rationale names a
  narrower defect is a frame-authoring defect of mine, not a constraint to
  route around silently.

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

  ### ⛔ `D3` IS RECUT INTO `D3a` AND `D3b` — Steward ruling, 2026-08-05

  **Grounds: the correspondence probe** (`evt_gp162jb84s8b`, exact `2bd724cd`,
  reverted byte-identical). 58 of 58 predeclared-emitter records **match** — the
  positive result the first `D3` attempt lacked, ruling out a different base, a
  reversal and a constant offset. But **every match is at depth zero**:
  `env_len == seat_len == 2`, the producer owner's entry parameter count, across
  2 construct origins and 1 owner. No reaching emission sits under an
  intervening binder, and **`post_shift_index` is distinguishable from the root
  ABI position only when a binder intervenes.**

  ⇒ **The post-shift axis is UNMEASURED, not passed.** Confirmed from the other
  side: all 85 reaching records carry `availability=EntryAbi`; **zero
  producer-local availabilities reach lowering at all**, because the `D2` gate
  declines every candidate before projection.

  ⛔ **THE SEQUENCING DEFECT IS MINE.** `D3`'s lowering half consumes a
  population **only `D4` creates**, and this frame ordered `D3` before `D4`.
  `D3` as cut bundled two consumer groups with **different measurability** into
  one all-or-nothing deliverable. That is a sizing defect in my cut, not a
  Runtime execution problem, and the recut is the fix.

  ### ⛔ THE BINDING ORDER IS FOUR CHECKPOINTS — Architect `evt_7vc8zh0rvqyps`

  ⛔ **This supersedes the Steward's own two-checkpoint recut** (`D3a`/`D3b`
  with `D3b` "with or after `D4`"), which was **directionally right and
  under-specified in the one place that decides the work.** `D4` as a single
  unit cannot both *create* the nonzero-depth population and *prove* the final
  partition, so "with or after `D4`" never named what would produce `D3b`'s
  evidence. `D4a` is that missing mechanism. ⭐ Same defect class as the `D1`
  clause: a load-bearing sequencing term left ambiguous across two things.

  ⛔ **Atomic scope and mechanism are UNCHANGED. No new node, selector,
  coordinate, ABI field, or fallback is authorized.**

  1. **`D3a` — non-lowering closure.** Land the tagged ABI provenance
     authority, the validator changes, the ABI/view checks, and the
     full-coordinate generated-context lookup. Keep **both** lowering consumers
     explicitly refusing `CurrentLexical` **and** `GeneratedContextCapture`;
     keep the seam and the pending population **visible**.
     ⛔ **`entry_abi_pending_producer_local` is RETAINED, and that is
     COMPLIANCE, not an exception** — the release conditioned its deletion on
     *"only when its live enumeration is empty"* (`evt_6zr8a4h90c7rp`). After
     `D3a` it enumerates exactly the two lowering arms, so it still names its
     own remaining work.

  2. **`D4a` — bounded admission and MEASUREMENT.** Admit the census-bound `V`
     population using the **existing** authority, while retaining `D3a`'s
     refusals. ⭐ **This checkpoint MAY BE DELIBERATELY RED.** Its purpose is to
     produce the real reaching producer-local emissions and measure
     nonzero-depth `CurrentLexical` correspondence. `R` remains declined.
     ⛔ A red here is the instrument working, not a regression to chase.

     **`D4a` ran two extension rounds. Round 1 hard-stopped, structurally.**
     Admission landed at `52422da5` and produced exactly one reaching
     `CurrentLexical` emission — at depth 1, but with
     `post_shift_index == locator.environment_index == 0`, so the pass-through
     defect stays observationally identical. The Architect's bounded extension
     (`evt_tkzyc61rmd3`) sent the durable shifted fixture
     `contsrc_d2_both_binding_kinds_fixture` through real lowering; it emits
     **zero seam records**. Its `Let`-bound effect is `HostOpV1::ConsoleRead`,
     absent from the fixed 13-element `CRANELIFT_HOST_EFFECT_CONSUMERS_V1`, so
     `lower_process_host_effect` refuses it as an unavailable lane before the
     emission seam. ⭐ **The fixture is shifted precisely BY the construct that
     makes it unlowerable** — it has always been planner-level. Hard stop
     `evt_7xwdw87mgf1q3`, QA-verified `evt_5nd65hwfh941k`.

     **Round 2 — a lowerable shifted fixture is AUTHORIZED** (Steward ruling
     `evt_28xx7t69z7j76`). This lifts exactly one prohibition from the bounded
     extension, *"do not add a new population member"*, and lifts nothing else:
     alternate lowering routes, ABI/lane widening, selectors, fallbacks,
     permanent side maps and direct construction all stand. Three constraints:

     - ⭐ **The fixture supplies the POPULATION; the MUTATION supplies the
       discrimination.** A fixture built to exhibit
       `post_shift_index != locator.environment_index` and then observed to
       exhibit it measures nothing. Soundness rests on requirement 5 of
       `evt_tkzyc61rmd3` — the bounded wrong-index/swap mutation must flip the
       discriminator. ⛔ **If that row cannot be written, the fixture is not
       worth adding**, and the result is a hard stop rather than a green suite.
     - ⛔ **Do not inherit the effect lane from `D2b`'s fixture.** The
       requirement is a shifted lowerable producer-local emission; whether the
       shifted value is a host-effect result or a case binder is
       **unconstrained**, and case binders are already in `V`. If no admitted
       lane's result can be `Let`-bound *and* no non-effect route shifts, that
       is a larger fact to report, not to work around.
     - ⛔ **`contsrc_d2_both_binding_kinds_fixture` is untouched**, discriminator
       included. The new fixture is additive.

  3. **`D3b` — lowering closure.** Implement the two lowering arms **only after
     that evidence exists**:
     - `CurrentLexical` requires the matching predeclared emitter plus exact
       emission origin, lexical-environment origin, and post-shift index.
     - `GeneratedContextCapture` requires the matching specialization context,
       owner, and declared immediate-capture slot.
     - Cross-domain pairings, missing identity, wrong owner, wrong index/slot,
       and ambiguity **reject**.

     Delete the seam only when its closed population is empty.

  4. **`D4b` — admission closeout.** Prove the framed final partition and
     controls: `interned = V`, `declined = R`, with no extra route modality and
     no special case.

  ⛔ **Option 2 as offered is INVALID, not merely worse:** `D4` cannot safely
  admit the population before the lowering consumers are explicitly fail-closed.
  That is why `D3a` precedes `D4a` rather than the reverse.

  ⛔ **OPTION 3 IS REJECTED, and not for cost.** "Accept direct-construction
  testing of the lowering arm with the unmeasured plane correspondence
  recorded" would write `producer_env.get(post_shift_index)` resting on the two
  planes agreeing at depth zero. If that inference is wrong **the seam does not
  refuse — it reads a different value, silently.** A silent wrong value at an
  emission seam is precisely the failure class `D2b`'s guards were built to
  prevent. ⛔ Do not re-propose it as a schedule recovery.

  ### Finding 2 — ANSWERED. Architect confirming gate `evt_6p6vf0aqnjn3g`

  All 27 of 27 specialization-emitter records are non-corresponding — 26
  out-of-range, 1 pointing at a different owner's parameter (`env_len=3`
  against `seat_len=1`) — a perfect separation by emitter class with zero
  exceptions. ⭐ The implementer read it as correct-by-design but **declined to
  classify a mismatch as the benign one on its own authority**, because the
  ruling said any mismatch is a hard stop. That refusal was correct and it is
  what produced the ruling below.

  **The mismatch is lawful and expected.** `CurrentLexical` is an authority over
  the **retained lexical environment at an exact predeclared emission
  occurrence**. A specialization emitter owns a **generated-context operand
  run**; the same numeric index has no authority to name a value in the retained
  lexical environment.

  ⇒ **Seam 1 must REJECT `CurrentLexical` whenever the active emitter is a
  specialization, before indexing any operand run.** The lawful local arm there
  is `GeneratedContextCapture`, matching the exact generated context,
  specialization owner, full root-coordinate membership, and declared
  immediate-capture slot. **Conversely a predeclared emitter must reject
  `GeneratedContextCapture`.** ⛔ No conversion, offset, fallback, or
  "same value" inference crosses the domains.

  ⛔ **The Architect corrected its own earlier "any mismatch is a hard stop" as
  OVERBROAD.** The hard-stop comparison applies to a `CurrentLexical` authority
  **at its matching predeclared-emitter domain**; applying that comparison to a
  specialization emitter is **itself a category error**. ⇒ Do not carry the
  older phrasing forward from any earlier post in the thread.

  ⛔ This binds `D3b` only. `D3a` does not wait on it.

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

  ⛔ **`C` = 83 and `V` = 80 are MEASUREMENTS AT `e6d4f085`, not invariants —
  `R` is the invariant** (Steward ruling `evt_28xx7t69z7j76`, 2026-08-05). The
  discharge condition above says *post-admission* census for a reason: it is a
  procedure re-run at the base being closed, which is also why the program
  fingerprint is part of the identity. Adding a corpus fixture therefore moves
  `C` and `V` and does **not** violate this deliverable.

  What is pinned is `R`'s **three named causes** — `OPEN[ih-binder]`,
  `OPEN[let-value:Construct]`, `AMBIG2[let-value:If]`. A new fixture that adds a
  member to `R` is a real finding about the contract domain: stop and report it.
  One that adds only to `V` leaves the partition intact.

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
