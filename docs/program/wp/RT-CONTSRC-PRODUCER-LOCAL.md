# RT-CONTSRC-PRODUCER-LOCAL — the producer-local continuation source coordinate

**A value created mid-body is a third ROOT-COORDINATE domain. Continuation
specialization can name entry values and generated-context captures, and cannot
name a host-effect result or a `Match` case binder — so the environment for
those edges never closes and the specialization is never committed. This node
adds that coordinate domain, and separately replaces the availability
representation the domain is consumed through.**

⛔ **This summary said "a third AVAILABILITY class" until 2026-08-05. That was
the false coupling itself** — it makes a root source kind imply an availability
class. Root provenance answers *which value*; availability answers *where this
consumer holds it*; neither determines the other (`D3b`, on the `D3c`
measurement).

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
     affinity, **and separate consumer-specific availability claims (`D2b`)**.

  ⭐ **`unchanged` on item 1 is CORRECT and stays.** The root-identity sum is
  exactly what the `D3c` correction preserves — no added source domain,
  position, offset or fallback. ⛔ Do not confuse it with the retired "Entry ABI
  *availability* remains its existing case", which was a claim about
  availability and was false. Root identity: unchanged. Availability: replaced.

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

  ⛔ **THE LAW BELOW WAS REPLACED 2026-08-05 on the `D3c` measurement.** What
  stood here was "one planner-issued, closed projection, separate from root
  provenance, with two arms" — current-lexical availability and
  generated-context capture availability — plus "Entry ABI availability remains
  its existing case, untouched." **Both are false and neither is retained.** The
  two-arm shape is root-adjacent: it reads an availability off the root
  coordinate. The entry-ABI clause is the specific premise `D3c` destroyed —
  an entry-ABI root's availability is **not** its root ABI position, and at
  nonzero lexical depth production silently read the wrong operand.

  **Availability is a CONSUMER-SPECIFIC planner-issued claim.** The projection
  is not one availability with arms; it is a claim selected by the environment
  the consumer actually holds, over two closed environment classes:

  1. **`CurrentLexical`** — exact predeclared emission owner, producer/result
     and emission occurrence, lexical-environment origin, and the **lexical index
     issued by the nearest-exact-singleton-alias law** (see `D3b`), derived by
     the forward semantic environment walk. Lowering may consume it **only at
     that exact seat while holding that exact lexical environment**.
  2. **`EntryFrame`** — an exact frame identity plus declared slot: a
     predeclared function frame, or a generated context frame identified by its
     `ContinuationContextId` and enclosing `ContinuationSpecializationId` with
     that pairing revalidated. ⛔ It exists **only after** the full root
     coordinate is proved present in that frame's ordered projection **exactly
     once**.

  ⛔ **`GeneratedContextCapture` is SUBSUMED into the generated-context
  `EntryFrame` case.** It is not a third class and not a surviving second name
  for the same environment class.

  ⛔ **A projection consumed at direct emission and later reused as a
  generated-context capture exposes TWO SEPARATELY VALIDATED VIEWS**, never one
  availability reused twice — direct emission reads `producer_env`, capture
  append reads `defining_abi_operands`, and one unqualified index cannot be
  authority for both. An unkeyed vector, "first matching availability", or one
  generic `immediate_slot` is unlawful.

  ⛔ **Either root arm may take either environment class**, subject to the
  membership proofs above. Root provenance answers *which value*; availability
  answers *where this consumer holds it*. The one asymmetry is substrate, not
  domain: a `ProducerLocal` member of a **predeclared** `EntryFrame` cannot be
  invented and stays unavailable unless a separately authorized substrate later
  declares it.

  ⇒ `D3b` carries the full ruled statement, the four retired clauses and the
  control list. **This paragraph is the law, not a pointer to it.**

  **Lawfulness of a generated-context capture:** carrying a producer-local value
  as a declared capture is lawful **only when the caller's exact current-lexical
  projection proves the value is already available at that call seat**. This does
  **not** widen the predeclared function's entry ABI and does not claim the
  mid-body value existed there.

  ⛔ **If no exact current-lexical source is available, planning DECLINES or
  REJECTS** per the existing candidate/program boundary. Lowering must not
  reverse-search, infer a shift, reuse `Result`, or fabricate a capture.

  **Fail closed on all seven** (⛔ **the old list of five named `wrong immediate
  slot`, a field now retired, and had no arm for a claim presented to the wrong
  consumer — the defect `D3c` measured**): wrong emission owner/origin · wrong
  lexical-environment root · wrong selected lexical index · wrong frame identity
  (predeclared frame, or context id and enclosing specialization id) · **missing
  eligible membership, or an ambiguous `Closed([S, T])` where an exact singleton
  is required** · wrong declared slot · **a direct-emission claim presented to
  the ABI-frame consumer, and the converse**.

  ⛔ **"Duplicate" is NOT a refusal arm in the LEXICAL environment** (Architect
  ruling on the `D3b` hard stop at exact `456ec7e6`, 2026-08-05). Two lexical
  positions each holding exactly `Closed([S])` are **proved aliases of one
  semantic source**, and the law selects the nearest. ⛔ Duplicate membership
  **still refuses in a frame's ordered capture projection**. `D3b` carries the
  full statement; **do not collapse the two cases.**

  **Why the asymmetry is principled, not an oversight** (Architect confirmation,
  2026-08-05, on the landed recut — recorded here because a later `exactly once`
  sweep will find the retained clauses and be tempted to unify them):

  | | licences duplicates? | why |
  |---|---|---|
  | lexical environment | **yes** | `Closed([S])` from the forward semantic walk **proves** two bindings denote one source, and de Bruijn order supplies a canonical nearest binding |
  | frame ordered projection | **no** | it declares ordered **ABI slots** — it carries no semantic-walk alias proof and has no "nearest lexical binder" ordering |

  ⇒ Two frame members carrying one full coordinate make the **declared source
  slot non-unique**, so they must refuse. ⛔ **Selecting one of them would
  reintroduce the unkeyed first-member rule at the ABI boundary** — the exact
  thing the ban exists to prevent, arriving under the alias law's name.

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
     ⛔ **NAMING ONLY, 2026-08-05:** `D3a` is discharged and the sentence above
     is a true record of what it landed, so it is retained rather than rewritten.
     But `GeneratedContextCapture` is now **subsumed** into the generated-context
     `EntryFrame` case (`D3b`). ⛔ Do not carry that name into new work, and do
     not read this line as authority for it being a separate class.
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

  3. **`D3b` — lowering closure. RE-CUT 2026-08-05 on the `D3c` result**
     (Architect ruling at the `D3c` disposition; Steward fidelity recut). The
     landing at `bc371f13` is **preservation-only evidence, not a candidate.**

     **The law it landed is FALSE and is REPLACED, not qualified.**
     `EntryAbi + CurrentLexical` is not a crossed pair — it is the **measured
     lawful answer at nonzero lexical depth**, and an `EntryAbi` root may also
     be available through a generated-context capture. **Root provenance
     answers *which value*; availability answers *where this consumer holds
     it*. Neither determines the other.**

     **Why a fourth pairing does not fix it.**
     `ContinuationInputProjection::availability` is reused by consumers that do
     not hold the same environment: direct continuation-call emission reads
     `producer_env`, the exact environment at the producer/emission seat, while
     generated-context capture append reads
     `function_local.defining_abi_operands`, an entry-frame operand run. **One
     unqualified index cannot be authority for both.**

     **The ruled representation.** `ContinuationSourceCoordinate` stays
     unchanged as the closed root-identity sum — no added source domain,
     position, offset or fallback. What is replaced is the single root-coupled
     availability, which becomes **consumer-specific planner-issued
     availability claims** over two closed environment classes:

     1. **`CurrentLexical`** — exact predeclared emission owner,
        producer/result and emission occurrence, lexical-environment origin,
        and the **lexical index issued by the nearest-exact-singleton-alias
        law** below.
     2. **`EntryFrame`** — an exact frame identity plus declared slot, either a
        predeclared function frame, or a generated context frame identified by
        its `ContinuationContextId` and enclosing `ContinuationSpecializationId`
        with that pairing revalidated.

     **`GeneratedContextCapture` is subsumed into the generated-context
     `EntryFrame` case.** Do not retain two names for one environment class.

     A projection consumed at direct emission and later reused as a
     generated-context capture exposes **two separately validated views**, never
     one availability reused twice. A fixed pair of consumer views, or a closed
     claim record keyed by those two consumer kinds, is lawful; **an unkeyed
     vector, "first matching availability", or one generic `immediate_slot` is
     not.**

     **Availability is selected from the environment actually held, never from
     the root-coordinate arm:**
     - at the measured predeclared direct-emission seat, **both** `EntryAbi` and
       `ProducerLocal` roots use `CurrentLexical`. The existing forward semantic
       environment walk issues the position by the **nearest-exact-singleton-alias
       law** below. ⛔ **The old requirement that the walk find the full
       coordinate "exactly once" is FALSE and is retired** — it conflated *does
       this position certainly hold `S`* with *is it the only position that holds
       `S`*, and `D3b` needs only the first;
     - at a generated-context operand consumer, **either** root arm may use the
       generated-context `EntryFrame`, but only when that exact context
       descriptor's ordered capture projection contains the full coordinate
       exactly once and the declared slot agrees;
     - at a predeclared `EntryFrame` consumer, either root arm is representable
       only when that exact predeclared descriptor has a planner-issued member
       for the full coordinate. An ordinary `EntryAbi` member can satisfy this
       today. **A `ProducerLocal` member cannot be invented** — it stays
       unavailable unless a separately authorized substrate later declares it.

     **Preserved non-candidate checkpoint: exact `456ec7e6`** (`722 passed / 10
     failed` — the 7 prior reds unchanged, 3 new ones sharing the one cause this
     law fixes). Resume the bounded repair from it; it is not a candidate and does
     not go to QA. ⛔ `41d2b1e5` appears in two posts and **is not an object on
     the branch** — it was quoted from memory and corrected twice.

     Two results from that turn stand and are not to be re-measured: **caller-frame
     multiplicity is negative** (direct owner = emission owner 40/40; capture
     indexed frame = enclosing spec's emission owner 20/20, and structurally so,
     since `emission_owner` is a field of `ContinuationSpecializationKey`), and
     the **capture consumer's source-frame defect is fixed**. ⛔ The earlier
     specialization census that suggested multiplicity was a **per-compile-id
     artifact** — its author retired it; do not reintroduce it as evidence.

     ### The nearest-exact-singleton-alias law (lexical positions)

     Architect ruling on the hard stop at exact `456ec7e6`, 2026-08-05. It
     **replaces** coordinate-containment plus exact-once-position. The old law
     was a mis-stated precondition — not a missing coordinate domain, SSA
     identity oracle, caller-edge key, unit-frame edit, or new node.

     **Why duplicates are lawful.** One `ContinuationValueSourceAuthority`
     describes **one semantic value**, and `join` unions and deduplicates its
     complete `ContinuationSourceSlotAuthority` records. So `Closed([S])` means
     every represented path yields exact source slot `S`, while `Closed([S, T])`
     means the value is ambiguous between distinct sources. ⇒ **Two positions
     each holding `Closed([S])` are proved aliases of the same semantic source**,
     even where lowering assigns them different SSA names. The measured
     `let y = x` at indices 0 and 2 is exactly that case.

     ⛔ **The `If` concern does not defeat this.** An `If` that can yield `S` or
     `T` joins to `Closed([S, T])` and is **not** an exact alias. If both branches
     yield `S` the join stays `Closed([S])` — which is precisely the proof
     required. No SSA-equality instrument or alias tag is needed, and adding one
     would duplicate a semantic-value fact the planner authority already states.

     **The rule, in four steps:**
     1. Derive against the **complete requested source-slot authority `S`** —
        coordinate, carrier, ownership, storage owner, and referent affinity.
     2. A lexical position is eligible **iff** its held authority is exactly
        `Closed([S])`. ⛔ Merely containing `S.coordinate` is not eligibility,
        and neither is `Closed([S, T])`.
     3. Among eligible positions select the **minimum de Bruijn index** — the
        nearest/innermost exact alias in the ordered environment.
     4. Record that index in the existing exact-seat `CurrentLexical` claim. The
        consumer **rederives** the same result from the same complete `S` and
        exact seat before indexing.

     ⛔ **This is NOT the banned "first matching availability."** It is one typed
     `CurrentLexical` claim chosen by a **total** rule over an ordered semantic
     environment, applied only **after** exact singleton equality has proved
     every eligible position is an alias of one value. The banned forms stay
     banned: first coordinate-containing member, an unkeyed availability vector,
     fallback between consumer views, reverse search, shape or offset inference,
     and any ambiguous source set.

     **RETIRE, do not annotate around:** `RootIsImmediate`; the "three lawful /
     three crossed" coordinate-product table; the equality requirement
     `immediate_slot == source_abi_position`;
     `ContinuationImmediateResolution::root`, which duplicates the coordinate
     domain after the caller already supplied the coordinate; and **the
     exact-once-lexical-position precondition together with its "present at two
     positions refuses" clause.**

     **The bounded re-cut is exactly five things:**
     1. planner construction of the two consumer-specific availability views;
     2. the direct-emission resolver and the ABI-only context-capture resolver;
     3. the corresponding ABI/view agreement checks and durable controls;
     4. **deliberate replacement of `D3c`'s old-defect observatory with the
        corrected invariant** — see the `watch` note under `D3c`;
     5. the **two-stage `EntryFrame` construction** below.

     ### The two-stage `EntryFrame` obligation stays in `D3b`

     ⛔ **It is NOT moved to `D4b`** (Architect, 2026-08-05). `D3b` must mint all
     context skeletons, resolve **every** structural generated-frame requirement
     to exactly one `(ContinuationContextId, ContinuationSpecializationId)`,
     publish only **immutable final** claims, and refuse zero-or-multiple
     resolution. `ContinuationContextId` does not exist while
     `exact_continuation_projection` interns a specialization key, so that phase
     cannot build the final claim and later mutation of an interned projection is
     unlawful. `(enclosing_specialization, worker_body_origin)` is a
     **provisional interning key only**.

     **Direct planner controls may prove that construction now**; `D4b` later
     supplies behavioral activation. ⛔ The measured **0/60** generated-owner
     consumptions explains the present evidence boundary and **does not authorize
     a half-stamped accepted plan.**

     **The generated `EntryFrame` ID names the SOURCE/CALLER frame** whose
     `defining_abi_operands` are indexed — never the target `context` argument to
     `call_declared_context` merely because that ID is already in hand.
     `Predeclared(owner)` is that exact entry frame; `Specialization(owner)` is
     the exact generated caller context resolved from the planner-owned selected
     worker-body key.

     **The `D3c` witness must then prove**, on the same seat: `EntryAbi` root
     position 0 remains root provenance; its direct-emission `CurrentLexical`
     position is 1; the emitted operand is the entry oracle's exact SSA value;
     and substituting root position 0 for the planner-issued lexical position
     **fails at the consumer boundary** while both positions stay in bounds and
     same-shape. The zero-depth rows remain positive agreement evidence, and
     **no equality law is derived from them.**

     **Additional load-bearing controls, each reaching the real consumer before
     refusing:** swap or reuse the other input's availability claim; wrong
     emission owner / origin / environment root / selected lexical index; wrong
     predeclared frame; wrong context id, enclosing-specialization id, capture
     membership, or declared slot; **absent eligible membership, and duplicate
     membership in a frame's ordered capture projection**; and presenting a
     direct-emission claim to the ABI-frame consumer and the converse.

     **The alias law's own six required controls** (Architect, on `456ec7e6`).
     The repair must prove all of them:

     1. the measured `EntryAbi` source at lexical indices 0 and 2 **selects index
        0** and reaches the real consumer;
     2. perturbing that claim to index 2 is **refused by consumer revalidation**
        — this proves *canonicality*, ⛔ **not** that the outer alias is a
        different value;
     3. an **inner `Closed([S, T])` plus an outer `Closed([S])` selects the outer
        singleton** — this is the control that proves the rule is not first
        coordinate-containment, and it is the one most easily omitted;
     4. `Closed([S, T])` with **no** singleton `S` refuses;
     5. the same coordinate with a **different carrier, ownership, storage owner,
        or affinity does not qualify** — eligibility is the complete authority
        `S`, not the coordinate;
     6. the existing **zero-depth and shifted-index discriminators remain live**.

     ⛔ **Control 3 is not a variant of control 1.** Selecting the *outer*
     position in 3 and the *inner* in 1 is what distinguishes a total rule over
     eligibility from a positional shortcut; a suite carrying only 1 and 2 passes
     under either.

     **Preserve the accepted `ProducerLocal` `CurrentLexical` and
     generated-context evidence from `bc371f13`, retyped under this
     representation.** The consumer-mutation apparatus and the
     membership machinery are not work to redo from zero — the premise is what
     was false, not their fidelity.

     No reverse search in lowering, numeric shift, constant offset, padding,
     same-shape or same-value inference, or consumer-side fallback is lawful.

     Delete the seam only when its closed population is empty.

  4. **`D4b` — admission closeout.** Prove the framed final partition and
     controls: `interned = V`, `declined = R`, with no extra route modality and
     no special case.

  5. **`D3c` — the `EntryAbi` immediate-availability measurement. DISCHARGED
     2026-08-05 at QA-approved exact `f5e4fa9f` over preserved `bc371f13`.**
     Inserted per Architect `evt_56jh63qntwtfe` and Steward recut
     `evt_7he9qv8wbv1yq`; numbered last only because renumbering would have
     broken live references. It ran **before** further `D3b` work, which is the
     whole reason `D3b`'s false law was caught before anything was built on it.

     **THE RESULT: the position MOVES.** At a real predeclared emission seat
     under one intervening binder:

     ```
     source_abi_position          = 0
     defining_abi_operands[0]     = specialized-scalar(v15)   <- the entry oracle
     producer_env                 = [ specialized-scalar(v44),   <- index 0
                                      specialized-scalar(v15),   <- index 1
                                      specialized-scalar(v21) ]
     measured immediate position  = 1
     ```

     Production reads `producer_env[0]` and gets **`v44`**, the producer-local
     the binder pushed, where the entry parameter `v15` belongs. **Nothing at
     that seam can see it:** the index is in bounds, the lowering shape is
     identical, and `D3b`'s own consistency law `immediate_slot ==
     source_abi_position` **passes**, because both are 0. The seam emits a
     well-formed operand of exactly the right contract carrying the wrong
     value — the precise class this checkpoint's Option 3 rejection names.

     **Attribution, which is the half that makes it a finding rather than a
     divergence:** the same armed window compiles the `D5a`
     `px8tr_nested_post_effect` witness at **zero** binder depth, where the two
     agree position for position, and the control refuses to proceed if that
     agreement set is empty. So the oracle is correct exactly where the
     projection's assumption holds and divergent exactly where a binder was
     pushed. The two coincide **only at zero binder depth**, and every
     pre-`D4a` population was at zero depth — which is why nothing ever had to
     tell them apart. No production edit; every added line is `#[cfg(test)]`,
     and no fixture was authored — the seat was selected from the existing
     production population by the property, never by ordinal.

     **The severity question, ruled by the Steward rather than left open.** The
     implementer correctly noted this is not shown reachable in a program that
     compiles green today, because the population already dies downstream at the
     unit-body boundary. **That does not make it latent.**
     [[RT-UNIT-CLOSURE-CONVERT]] exists to clear that boundary, so **landing it
     is what unmasks this defect** — the correction lands before or with that
     node. A defect masked by a second defect is not lower priority when the
     mask is itself scheduled work.

     **`f5e4fa9f` is preservation-only.** The correction is `D3b`'s re-cut
     above, and it is *not* a fourth pairing.

     **watch — this checkpoint's own control is scheduled for deliberate
     replacement.** The `D3c` observatory pins a relation, not an index or SSA
     word, so it is durable by construction: **if the representation is
     corrected so the two agree, this control must be re-cut deliberately, and
     its going red is the correction announcing itself.** That replacement is
     item 4 of `D3b`'s bounded re-cut. Do not read its red as a regression.

  ⛔⛔ **SUPERSEDED 2026-08-05 — [[RT-UNIT-CLOSURE-CONVERT]] IS `closed` AND
  THIS GATE IS NOW `D5` BELOW.** The paragraph that stood here called the five
  `Var: no runtime binding` reds "a substrate boundary, not a bounded repair
  here." ⛔ **That is FALSE and is retired by measurement**, not merely
  re-scoped: `D1`/`D1b`/`D1c` on that node measured the runtime closure-
  conversion substrate **complete**, production's capture basis **total by
  construction**, and the five failing units **never on the production path at
  all** — they are literals in `test_objects.rs`. The Architect then ruled the
  contract (`evt_5g7kaec1xzaf6`): `LexicalClosure.captures` must be **total**
  for its body's ambient lexical demand, so those fixtures are **malformed**.
  ⇒ It is a bounded repair here after all, and it is `D5`. Do not restore the
  old reading, and do not read the five reds as evidence of a substrate gap.

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

  ⇒ **Seam 1 must REJECT a `CurrentLexical` claim for a predeclared retained
  environment whenever the consumer holds a generated-context frame, before
  indexing any operand run**, and conversely a generated-context frame claim
  must not index a predeclared lexical environment. The lawful local claim at a
  specialization consumer is the **generated-context `EntryFrame`**, matching
  the exact context id, enclosing specialization id, full root-coordinate
  membership exactly once, and declared slot. ⛔ No conversion, offset,
  fallback, or "same value" inference crosses the domains.

  ⛔ **RESTATED 2026-08-05 — the rejection is by CONSUMER ENVIRONMENT IDENTITY,
  not by root domain.** This finding's substance survives the `D3c` correction
  intact; what changes is its basis. The earlier phrasing keyed the refusal to
  the *emitter class* standing in for the root domain, and that coupling is
  exactly the false premise `D3c` destroyed. **Do not carry the emitter-class
  phrasing forward** — a root arm no longer implies an environment class, so a
  refusal justified by the root domain is unsound even where its verdict
  happens to be right.

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

- **`D5` — correct the two MALFORMED `LexicalClosure` fixtures. ADDED
  2026-08-05; this is the node's LAST deliverable and it clears the candidate
  gate.** Folded in from `closed` [[RT-UNIT-CLOSURE-CONVERT]], whose substrate
  premise three measurement passes retired.

  **The governing contract, Architect ruling `evt_5g7kaec1xzaf6`:** a
  `RuntimeExpr::LexicalClosure` is well formed only if its explicit ordered
  `captures` run covers **every** ambient lexical de Bruijn reference its `body`
  can reach, after accounting for (1) declared parameters, (2) binders
  introduced inside the body, and (3) separately declared compiler-private or
  generated unit bindings under their own contracts. ⛔ **There is no lawful
  fourth source — an undeclared enclosing/caller environment tail.**

  ⭐ **It is a TOTALITY ruling, not a MINIMALITY one.** A conservative capture
  run **may be larger** than the body's actually-used free-variable set; it may
  never be **shorter** than the body's ambient lexical demand. Do not "tighten"
  a capture run to its used set on the strength of this deliverable.

  ⛔⛔ **THE ANCHOR IS `planning/static_transition.rs:12038`, NOT
  `test_objects.rs`. Corrected 2026-08-05 after a hard stop; the wrong anchor
  rode through THREE artifacts before anyone probed it.**

  The site is the `LexicalClosure { captures: Vec::new(), params: ["buffer"],
  body: closure_body }` construction inside **`governed_nested_resource_bracket`**
  (`planning/static_transition.rs:11949`, construction at `:12038`). That fixture
  carries a **real de Bruijn scope tracker** (`bind`/`var`, `:11962`–`:11979`)
  which emits `RuntimeExpr::Var` by role lookup, and it is the fixture behind
  **exactly the five failing `Var` rows**.

  ⛔ **`test_objects.rs:176` and `:220` are CORRECT and must not be touched.**
  Their bodies are **genuinely closed — zero `Var` nodes** (`:176` is a `Let`
  over an `Effect` and a `Construct`; `:220` wraps `:176`). Their ambient
  lexical demand is nil, so `captures: Vec::new()` is right there, and adding
  captures would **fabricate** them — violating the very totality ruling this
  deliverable rests on, which constrains captures to *ambient demand* and is
  silent where demand is zero.

  ⭐ **How the wrong anchor survived, because the shape recurs:** `D1c` found
  empty-capture literals in `test_objects.rs` and asserted they were the failing
  units' source **without checking that they produce origins 88/14** — naming a
  producer from a grep rather than a probe. The Architect's ruling and my `D5`
  release then each inherited it as measured. **A `file:line` quoted in a code
  fence reads as measured and is only a claim.** Cf.
  [[agreement-is-not-corroboration-when-a-premise-was-inherited]]. `D1c`'s
  *mechanism* conclusion is unaffected: the population is still a hand-written
  fixture literal, not elaborator output.

  **Three requirements, all of them, and the second is the one that matters:**

  1. Either **explicitly capture** the intended enclosing value at closure
     construction, or make the body **genuinely closed** if that is the
     fixture's semantic intent.
  2. ⛔ **Retain a discriminator proving the corrected fixture still reaches its
     intended `D4a` boundary** — not merely that it went green. **Going green by
     avoiding the boundary is the failure mode this deliverable exists to
     prevent**, and a passing suite cannot distinguish the two.
  3. Retain a **negative** showing an undeclared ambient lexical reference does
     **not** acquire a caller tail or a fabricated capture.

  ### ⛔⛔⛔ THE GOVERNING TEXT IS HERE — everything below this block is SUPERSEDED

  **Architect ruling `evt_5gvgzp6bzj64t`, 2026-08-05.** `D5` is **real
  unit-environment transport work**, not a fixture edit. Both the
  "malformed fixture" conclusion and the fixture scope-track edit are
  **WITHDRAWN**. ⛔ Do not touch the fixture.

  **The premise that defeated two earlier recuts:** "nothing at the match level
  binds `AllocatedBuffer`" counted only the inner `ComputationalMatch`'s binders
  and **omitted the enclosing ordinary `Match`** over `BufferAllocate`, whose
  `Result::Ok` case has `binders: 1` and **is** the governed buffer. The
  nearest-first environment is
  `[InductionHypothesis, ScopeArgument, AllocatedBuffer, ...outer]`, so
  **`Var(2)` is well founded** — local under three binders. Confirmed by the
  production walk `required_surrounding_environment_prefix`, by both lowering
  constructions, and by the fixture's own committed structural assertion.

  ⇒ ⛔ **The five reds are a REAL DEFECT — a functionized-unit environment
  transport failure.** The emitted unit keeps the inner IH and constructor
  argument and **loses the outer ordinary-`Match` success binder** in scope at
  the source case body. ⛔ **Do not restore the "malformed fixture / no Ken
  defect" reading; I published it and it is false.**

  ⭐ **Unchanged and still standing:** the `LexicalClosure.captures` **totality**
  law; the `Scope`-constructor closure is **innocent** with a correct empty
  capture run; the `StaticWorker` is **non-causal**. ⭐ **The per-depth
  compounding worry is DEAD** — each recursive level builds its own outer
  `BufferAllocate` match and success binder, so a fresh `BinderScope::default()`
  accurately describes that level. It is **not** permission to discard the
  enclosing ordinary-case binder during unit formation.

  #### `D5a` — planner side: transport the binding

  1. Derive the outer ordinary-case binding from the enclosing `Match`
     **success-binder provenance**.
  2. Represent it as an **explicit typed unit-environment member** — ⛔ never an
     implicit caller tail.
  3. Retain the closed order **`IHs ++ arguments ++ outer frame`**.
  4. **Fail closed before emission** on omission, redirection, wrong provenance,
     wrong order, or fabricated availability.

  ⛔ **`D5a` HARD-STOPS to the Steward if representing that member requires a
  new planner or ABI authority** rather than an added typed member on an
  existing one. That is substrate expansion, the Architect's to authorize, and
  must not be discovered mid-repair.

  ##### GOVERNING — `D5a-1`/`D5a-2` are SPENT; the work is `D6a`/`D6b`/`D6c`

  **Architect ruling `evt_760m5azkrdrzq`, 2026-08-05.** Everything in the
  `D5a-1`/`D5a-2` subsection below is **refuted**. Those two labels are spent:
  their text names a mechanism the measurement disproved, so do not implement
  from them and do not reuse the labels. **Hold exact `e27d297a`.**

  **The law in this frame was FALSE and is corrected.** The semantic case
  environment is `[IHs, ALL constructor arguments in source order, outer
  frame]`. For the governed case: `[IH, ScopeArgument, BufferAllocate success
  payload]`. The functionized construction **replaces the selected recursive
  argument with its IH** — that is the defect. Source comments carry the same
  false law and are the implementer's to correct.

  **Three things are refuted, and each would have gone green:**

  | refuted | why |
  |---|---|
  | the Steward's *"new member at ordinal 2, not 3"* | it is `ScopeArgument` at ordinal **1**; `ContinuationInput(0)` moves to 2. At ordinal 2, `Var(2)` still resolves to a success payload while `Var(1)` is silently wrong |
  | the continuation-input projection | `ContinuationInput(0)` **already is** the outer success binder with exact case-body provenance (origin 45's `Result::Ok` body **is** the `ComputationalMatch` at 50); `required_input_count` is already 1. A second input is the banned fabricated availability |
  | a new `ContinuationOrdinaryEnvelopeRole` + ABI `Parameter(ValueWord)` | that envelope is the exact runtime Parameter-slot run. The selected recursive argument is a **closure capsule** with no lawful `ValueWord` representation, deliberately non-transferable. Widening `ordinary_parameters` reintroduces the closure-boundary violation under a new name |

  **Authorized representation.** One explicit **compiler-only unit-environment
  member** for the selected recursive constructor argument, derived from the
  unit's already-exact `recursive_position` and `ContinuationWorkerProvenance`.
  ⛔ It is **not** a new source occurrence, continuation input, ABI slot,
  carrier, tag, or runtime descriptor. It is installed as a compiler-only
  static-worker binding built from the same exact closure occurrence, body,
  arity and ordered capture provenance the unit already carries; its captures
  remain the existing transported worker-capture operands. **The closure capsule
  does not cross the ABI.**

  Closed binder plan: (1) one IH member for the selected recursive position;
  (2) one constructor-argument member for **every** source position —
  nonrecursive through the existing ordinary envelope, the selected recursive
  one through the new typed member; (3) the existing continuation-input run
  **unchanged**. Exact run for the governed row:

  ```
  InductionHypothesis
  SelectedRecursiveArgument { source_position: recursive_position }
  ContinuationInput(0)
  ```

  **The call-route distinction is load-bearing, and it is CONDITIONAL.** A
  closed compiler-only call-route discriminator is authorized, keyed to the
  planner-issued raw-body versus generated-context target; it may live on the
  static-worker binding and in the function-local declared-call table. ⛔ It
  must **not** be inferred from body shape, arity, use site, environment
  length, or which target is available, and no `FuncRef` may cross a function
  boundary. The current body-origin-only `worker_calls` lookup and
  `generated_context_captures` guard cannot distinguish the two bindings, which
  is why it is authorized.

  > ⛔ **CORRECTED 2026-08-05 by measurement (Architect `evt_3hx267n11sm9k`).**
  > This paragraph used to say, unconditionally, *"the IH calls the exact
  > generated continuation context and appends that context's
  > continuation-input suffix; the selected recursive argument calls the raw
  > worker and appends none."* **The first half is false.** The exact law:
  >
  > 1. `SelectedRecursiveArgument` **always** carries `RawWorker`.
  > 2. `InductionHypothesis` carries `GeneratedContext` **iff** the planner
  >    issued and this unit resolved that generated context; **otherwise it
  >    lawfully carries `RawWorker`.**
  >
  > ⇒ The governed fixture's **Raw/Raw** pair is **lawful**, not a defect. The
  > route field separates call semantics **where a generated context exists**;
  > it does **not** make every IH/argument pair different callables. Suffix
  > presence attaches to an IH route that **resolved** a generated context, not
  > to every IH.

  ⇒ This **narrows** the old statement that `StaticWorkerBinding` carries no
  call identity: it still carries no runtime callable value, but it must carry
  the compiler-only exact call route that separates these two lawful bindings.

  ###### The cut: `D6a`, `D6b`, `D6c`

  Sized for one turn each. This is a planner/lowering **representation
  expansion**, so it is three checkpoints, not one — the same mis-sizing that
  spent `D5a` twice.

  **`D6a` — representation only.** The `SelectedRecursiveArgument` typed member
  with its derivation from `recursive_position` + `ContinuationWorkerProvenance`,
  **and** the closed call-route discriminator. Both are what the plan *says*;
  ⛔ no lowering consumption in this turn.

  **`D6b` — consumption and the positive.** Lowering consumes both members.
  Governed positive: `Var(0)` is the IH, `Var(1)` is the exact `ScopeArgument`,
  `Var(2)` is the existing `BufferAllocate` success payload, and execution
  advances to the original `D4a` boundary.

  ⛔ **The discriminating control must run on the MIXED witness, not the
  governed one.** The governed fixture is **Raw/Raw** and is therefore
  **degenerate on route** — it cannot separate the two bindings. The
  **landed-object fixture's `GeneratedContext`/`RawWorker` pair is the
  nondegenerate discriminator.** Assert the **exact mixed route pair
  directly**; ⛔ do **not** infer from equal rendered routes that one binding
  was reused.

  ⚠ **Assert the whole run, never `Var(2)` alone** — but state the real reason.
  **The earlier rationale is RETRACTED:** a tail-appended member does **not**
  silently pass. The typed worker binding **refuses in value position** and
  five rows redden (measured). The whole-run control is still required; it is
  not required by the silent-pass story this frame used to tell.

  ⛔ **`D6b` also owes a NON-VACUOUS raw-call emission witness** (Architect
  `evt_21ytnjgdw147`, on partial progress `d86be55d`). The route consumer and
  the two function-local target tables are accepted and preserved — but **the
  landed-object row proves the mixed binding pair is in the environment without
  ever calling the selected recursive argument.** The corpus therefore never
  consumes `raw_worker_calls` in the retargeted case, and the mutation *"resolve
  `RawWorker` through `worker_calls`"* **survives**. ⚠ **A green suite cannot
  discharge a callee-selection law whose wrong table is observationally inert.**

  The witness is a test fixture built from **existing source constructs on the
  ordinary planner/lowering path**, where the selected recursive argument is
  actually **called**. Ordinary source syntax in a fixture is authorized; ⛔ a
  new production occurrence kind, authority, fabricated identity, population
  mechanism, or hand-constructed plan is not. It must prove: same exact
  closure/body provenance yields a `GeneratedContext` IH and a `RawWorker`
  selected argument; the argument is invoked at the planner-derived binder
  position with its real raw arity and captures; the emitted raw event carries
  **no** generated-context suffix and resolves through `raw_worker_calls`;
  substituting `worker_calls` for that lookup **reddens** it; and the existing
  `GeneratedContext` append-nothing mutation **stays red**.

  ⛔ **Fail closed on the population edge:** if the ordinary fixture reaches a
  template-only raw body and `raw_worker_calls` cannot be populated without
  declaring an undefined function, **stop and report that exact edge.** Do not
  force-declare a body to make the test green.

  This belongs to `D6b`, **not** `D6c` — `D6c` owns the refusal closure *after*
  this positive is established.

  ⚠ **`D6b` also owes the raw-target declared-call-table representation**,
  which `D6a` leaves honestly inert: the existing body-origin lookup is
  overwritten by the generated context and the raw body is removed from the
  executable population, so a raw route is not yet consumable in a retargeted
  specialization.

  **`D6c` — the refusal set, pre-emission.** Refuse on omission, duplicate,
  wrong source position, wrong closure/body, wrong capture run, wrong order,
  fabricated availability, or cross-routing the raw and IH targets. Plus:
  exactly one selected-recursive-argument member for the unit's exact recursive
  position and worker provenance; exact binder-run cardinality `IH count +
  argument_binders + continuation_inputs`; no added ABI Parameter/Capture/Result
  slot and unchanged continuation-input count; raw-route versus
  generated-context-route exactness, with **suffix presence only on an IH route
  that resolved a generated context** — ⛔ not on every IH, per the corrected
  conditional law above.

  ###### `D6a` STATUS — DISCHARGED at exact `70d4e8d9`

  **Architect fidelity verdict `evt_t1g8hrrtw4vn`**, parent `625b7860`, tree
  `5d868b6a`. The comment-only child touched exactly the three authorized files;
  after removing comment lines from the zero-context diff, the only changed Rust
  tokens are **two assertion-message literals** — conditions, predicates,
  expected vectors, route enum and construction, binder plan, provenance checks
  and control reachability all unchanged. A repository-wide phrase-family sweep
  found **no surviving** unconditional-IH, equal-route-means-reuse, or
  silent-tail-append statement. `736 / 2 / 1`, both profiles clean.

  ⇒ `D6b` is the next sequenced checkpoint, released at `evt_4hn520k8c5z5y`.

  The acceptance record below is retained as the mechanism's provenance.

  ###### `D6a` mechanism acceptance — exact `625b7860` (superseded by the discharge above)

  Architect review `evt_3hx267n11sm9k`, parent `e27d297a`, tree `80471947`.
  Measured `730 passed / 7 failed / 1 ignored` to **`736 / 2 / 1`** — five rows
  repaired, including the governed `Var(2)` failure, whose run is now
  `[IH RawWorker, SelectedRecursiveArgument RawWorker, ContinuationInput(BufferAllocate Ok)]`
  at ordinals 0/1/2, reaching the original `D4a` boundary. Both profiles clean.
  The repaired rows are a consequence of installing the missing binder, **not**
  an unauthorized route consumer: production call emission does not read the
  route, and the only read is the `cfg(test)` environment trace.

  ⛔ **`625b7860` does NOT discharge `D6a`** — its comments state the false
  unconditional route law this frame carried. A **comment-only child** of exact
  `625b7860` is authorized, correcting at minimum: `lowering/units.rs` (the
  `InductionHypothesis` docs, the segment-2 *"differing only in call route"*
  statement, the recursive-argument construction comment); `lowering/mod.rs`
  (the route-type prose asserting the two bindings are always different
  callables, and the `GeneratedContext` arm prose); and
  `lowering/core/tests/control.rs` (the withdrawn *"tail append silently leaves
  a plausible value"* explanation — keep the control, state its real
  discriminator). ⛔ **No executable change is authorized by this correction.**
  Preserve the tests, binder plan, route enum and construction, provenance
  checks, and the held `D6b` boundary.

  **Preserved, unchanged:** the existing hard stop for any additional recursive
  position for which the unit projects no worker. ⛔ Do **not** generalize this
  ruling into a multi-worker population.

  ##### ⛔ `D6b` IS MIS-SIZED. `D7` is the substrate, and it RUNS BEFORE `D6b` CLOSES.

  **Architect ruling `evt_6azsr4xrch1he`, 2026-08-05**, accepting the composed-path
  hard stop on preserved exact `d86be55d`. ⚠ **`D7`'s number is allocation order,
  not execution order** — it precedes `D6b` closeout. `D6b`, `D6c`, QA, candidate,
  `D6` closure and downstream are **frozen** meanwhile; `d86be55d` is preserved as
  **accepted partial progress**, not a residual.

  **The false claim was this frame's, and it is corrected: the two lowering paths
  do NOT build the same segments at the recursive field.**

  | path | what it installs at the selected recursive field |
  |---|---|
  | composed / source-machine (`core.rs:2929`, `:3467`, `:4649`) | `extend_specialized(..., args)` — **every** constructor field enters as `LoweringEnvironmentBinding::Value` |
  | functionized specialization (`units.rs:1696-1764`) | **alone** replaces the selected recursive field with the planner-derived `StaticWorker` |

  Direct descent has an exact-`Var` static-worker callee consumer
  (`core.rs:10375-10404`); the source machine's `Var` is **value-only** and its
  call state requires a specialized callable template.

  ⇒ ⛔ **This is a production representation/consumer gap, NOT a missing fixture.**
  The bounded witness repair released for `D6b` could not have discharged it —
  the only source shape that would exercise the raw table is **refused earlier**,
  which is exactly why its wrong-table mutation stays green. ⛔ **Do not accept
  the raw table as a permanently unwitnessed residual**; calling `D6b` complete
  would turn an unreachable mechanism into delivered semantics. ⛔ **And do not
  add a carrier callable/helper route** — worker body, arity, captures and route
  are already compiler/planner facts, so encoding callable identity or a template
  descriptor in the carrier would widen the ABI/runtime boundary, duplicate the
  static authority, and violate the compiler-only closure-capsule ruling.

  ###### The cut: `D7a`–`D7e`, ordered

  **In-node checkpoints, not a predecessor node** (Steward sizing call, which the
  ruling leaves to me). It lands on the same branch in the same candidate, so a
  node adds a tracker object and a critical-path entry while buying **no** merge
  boundary — the same reasoning that closed [[RT-UNIT-CLOSURE-CONVERT]]. Five
  checkpoints because the interface has five pieces with real dependencies, and
  because bundling is what spent `D5a` twice and `D6b` once.

  ###### EXECUTION ORDER — read this table, never the letters

  ⚠ **Labels are allocation order. This table is execution order.** It is the
  only ordering statement in this frame that is authoritative.

  | # | checkpoint | owner layer |
  |---|---|---|
  | 1 | `D7a` — composed worker view, **four-field** selector | planner projection |
  | 2 | `D7a2` — raw-target requirement / population closure | planner population |
  | 3 | `D7b` — one environment authority | lowering environment |
  | 4 | `D7c` — source-machine callee consumer | lowering consumer |
  | 5 | `D7d` — checked-marker occupancy | integration |
  | 6 | `D7e` — non-vacuous closeout, both paths | proof |
  | 7 | `D6b` closeout, then `D6c` refusal set | — |

  **`D7a` — planner-issued composed worker view.** From the exact
  computational-frame origin, selected alternative and recursive source position
  **already in scope**, planning exposes the same full worker provenance the
  continuation unit uses: closure occurrence, raw body, declared arity, ordered
  capture provenance, route eligibility. ⛔ Lowering may **not** rediscover it
  from closure shape, body shape, whichever target exists, or a source re-walk.
  Refuse before emission on zero answers, conflicting full identities, wrong
  position/body/capture provenance, or an unexecutable raw target.

  > ⛔ **NOT DISCHARGED — the selector is FOUR fields, not three** (Architect
  > `evt_7x6knchb4rb1n`, on preserved partial progress exact `ab741989`).
  > `(continuation_origin, producer_alternative, recursive_position)` **is not an
  > identity** — it names a source-level frame *position*, and **two dynamic
  > recursion layers may instantiate the same one.** Both current plans prove it:
  > each triple has **two distinct worker answers**, separated exactly by the
  > producer `Construct` origin.
  >
  > ```rust
  > pub(in crate::cranelift_backend) fn composed_worker_view(
  >     &self,
  >     producer_construct_origin: StaticOriginId,
  >     continuation_origin: StaticOriginId,
  >     producer_alternative: u32,
  >     recursive_position: u32,
  > ) -> Result<ComposedWorkerView, CraneliftBackendError>
  > ```
  >
  > This is the **same causal coordinate `continuation_call_binding_for` already
  > uses** — no tag, sequence, owner heuristic, or second identity. The composed
  > path supplies the first field directly as `deferred.construct_origin`.
  >
  > ⭐ **Different workers under different `producer_construct_origin` are NOT
  > conflicting answers — they are different questions the old selector
  > collapsed.** *"Conflicting full identities"* stays a refusal only when two
  > different workers answer the **same four-field** selector. ⛔ Do not choose
  > first/lowest; do not add emission owner, specialization id, or call sequence.
  >
  > The correction replaces **every** three-field source-contract statement and
  > control, and must prove: the measured three-field groups **collide**; every
  > corresponding four-field group resolves **exactly once**; dropping or
  > transplanting the construct-origin field **refuses, independently** of the
  > other three selector-field controls; and body-child and ordered-capture
  > provenance checks remain **independently live**.

  **`D7a2` — the raw-target requirement, and it is why `D7b` cannot start
  without it.** ⛔ **Do NOT delete or weaken the unexecutable-target refusal.**
  Against the current post-`D5a` executable population it is **telling the
  truth**: the outer raw body has a descriptor but no declared-and-defined
  `Function`.

  ⛔ **But that population predates the contract `D7b` adds, and the result is a
  CIRCULARITY: `D7a` refuses the very target `D7b` is required to make callable,
  so `D7b` can never lawfully start.** `D7b` does not merely expose an inert
  value — it installs a compiler-only `SelectedRecursiveArgument` whose route is
  **unconditionally `RawWorker`**, and a callable binding must resolve to a
  declared-and-defined target **even if a particular source body never invokes
  it**. That is a new planner-known **raw-target requirement**.

  The planner closes the population **before** environment installation:
  (1) mint **one unconstructible** raw-target requirement under the same exact
  four-field selector and full worker provenance; (2) treat it as a **surviving
  raw route** for `template_only_worker_bodies` / `executable_units`, retaining
  that raw body in the one declared-and-defined `Function` population;
  (3) re-run `composed_worker_view`'s executability check against that **final**
  population; (4) only then let `D7b` install the binding.

  ⛔ This is an **executable-target requirement, not an emitted-call event.** Do
  not invent an `EmittableCallEdge` before `D7c` emits a call, do not
  force-declare in lowering, do not retain every template globally, and do not
  infer demand from a reached lowerer or a source re-walk.

  **Non-vacuous controls, all five:** exact-set equality between four-field
  composed-worker requirements and the planner records retaining their raw
  targets; omission, wrong body and cross-construct transplant **refuse before
  lowering**; declaration and definition populations retain the **same exact**
  required body with **no undefined phantom**; a body with **no** such
  requirement **stays template-only** (proving this is not a global suppression
  rollback); and after reconciliation the current outer layer answers positively
  **while a deliberate requirement-omission mutation reaches the preserved
  unexecutable-target refusal**.

  **Why `D7a2` is its own checkpoint and not a `D7a` substep** (Steward sizing
  call, which the ruling leaves to me): the four-field correction is projection
  and stays in `D7a`, but the requirement **changes the executable-unit
  population**, which the released *"`D7a` projection only"* boundary excludes.
  Folding it in would repeat the bundling that spent `D5a` twice, `D6b` once,
  and `D7a` now.

  **`D7b` — one environment authority, not two.** When the composed /
  source-machine case environment installs the constructor-argument segment, the
  selected recursive position installs that exact compiler-only
  `StaticWorkerBinding` at its **source-order binder position**. Nonrecursive
  arguments stay `Value`; the IH prefix and outer frame are unchanged. ⛔ No
  parallel side map, no carrier facet. Value-position use of the closure capsule
  **continues to fail closed**, exactly as in the functionized unit.

  **`D7c` — source-machine callee consumer.** An exact `Var` callee resolving to
  that binding is consumed **before** the value-only `Var` path. Arguments are
  still evaluated under the existing source-machine control and phase, then handed
  to the **same** route-selected static-worker emitter direct descent uses. ⛔ No
  duplicated target/operand assembly; no shortcut through `lower_expr` that
  bypasses source control.

  **`D7d` — checked-marker coexistence.** The witness places an ordinary
  selected-argument call **before** the checked IH call inside one checked
  wrapper. ⇒ ⛔ *"A marker is pending"* therefore **cannot** mean *"the next
  static-worker call consumes it."* The selected-argument call **leaves the marker
  pending**; only the exact planner-issued checked call occurrence may consume it.
  Use the existing checked **occurrence authority** (or a faithful projection),
  ⛔ never route, arity, binder-index coincidence, or first-call order. Omission,
  duplicate, transplant and wrong occurrence must all refuse.

  **`D7e` — non-vacuous closeout.** Re-run the ordinary A/B source witness through
  **both** composed and functionized paths. It must reach a same-body
  `GeneratedContext` IH and `RawWorker` selected-argument emission, make the
  wrong-table mutation **red**, keep the context-suffix mutation **red**, and prove
  the raw target is **both declared and defined**. ⛔ `D6c`'s refusal set follows
  only after this positive closes.

  ##### SPENT — `D5a` cut into `D5a-1`/`D5a-2` (Steward, `evt_62ee5f2dvmvp1`). REFUTED, see above.

  **The hard stop above is ANSWERED: no.** The representation already exists —
  `ContinuationCaseBinderSource::ContinuationInput` at `lowering/units.rs:1022`,
  ordered by `continuation_case_binder_run` (consumed at `units.rs:1615`), whose
  own stated contract is *"the IH prefix, the constructor arguments in source
  order, then this frame's continuation inputs"* — the required closed order,
  already ruled and already load-bearing. The outer-frame member rides as a
  **continuation input**, so it is an explicit typed member structurally, not by
  convention. No substrate expansion; no Architect authorization owed.

  **The omission, re-derived at the producer with an instrument** (not from the
  fixture) for `governed_nested_brackets_n3`:

  ```
  ihs=1  arg_binders=1  cont_inputs=1  run_len=2  envelope_len=0
  ```

  Run = `[InductionHypothesis, ContinuationInput(0)]`, length **2**, matching
  `D1b`'s independently measured lowering environment `[worker | carried(v10)]`,
  `env_len=2`. Two independent measurements agree, so the run **is** the
  environment and the producer is the seam. Against the well-founded source
  scope, `Var(2)` needs a **third** member: `cont_inputs = 1` omits the outer
  `BufferAllocate` `Result::Ok` success binder. Segment 3 is a bare
  `0..continuation_inputs` count, so the omission is upstream in **what the
  planner projects as this frame's continuation inputs** — not in the ordering.

  **Why the cut:** the implementer ended two consecutive turns on `D5a` out of
  working budget with no edit. Both turns delivered what was asked (the
  authority audit, then the instrumented re-derivation), so the pace was not the
  problem — `D5a` as written is a planner projection change **plus** five
  fail-closed laws **plus** their controls, which is not a one-hour turn. **The
  sizing was the Steward's defect.**

  **`D5a-1` — the projection.** At the producer of `continuation_inputs` for
  functionized frames, derive the outer `BufferAllocate` `Result::Ok` success
  binder from its exact success-binder provenance and include it as an explicit
  `ContinuationInput`, preserving `IHs ++ arguments ++ outer frame`.
  **Positive:** the run for `governed_nested_brackets_n3` goes **2 to 3**,
  `Var(2)` resolves to the actual `BufferAllocate` success payload, and the
  original `D4a` boundary is still reached.

  ⚠ **`arg_binders=1` does not mean an argument is present in the run.** The
  sole argument binder is a recursive position, so **segment 2 contributes
  zero** and the new member lands at ordinal **2**, not 3. Placing it at 3 is
  exactly the "wrong order" case that `D5a-2` refuses.

  **`D5a-2` — the five pre-emission refusals.** Omission, redirection, wrong
  provenance, wrong order, fabricated availability — each with its control, plus
  the negative: drop or redirect **only** that planned outer-frame member and
  observe the **pre-emission** refusal.

  **The split drops nothing.** Both checkpoints land on the same branch in the
  same candidate, and neither is a merge boundary — the Architect's fail-closed
  obligation is discharged by `D5a-2` before anything is routed for review.

  #### `D5b` — lowering side: consume it, and prove it

  5. Lowering consumes **that exact planned member**.
  6. **Positive:** `Var(2)` is the actual `BufferAllocate` success payload,
     **and the original `D4a` boundary is still reached.**
  7. **Negative:** drop or redirect **only** that planned outer-frame member and
     observe the **pre-emission** refusal.

  ⛔ **Banned:** no capture edit, no source `Var` rewrite, no padding, no
  shifting, no synthesized capture, no caller-tail recovery.

  ⭐ **This vindicates [[RT-UNIT-CLOSURE-CONVERT]]'s GAP STATEMENT** — the unit
  environment does not carry what its body needs — while its **mechanism**
  (capture slots) stays wrong. That node stays `closed` and capture conversion
  remains banned as the repair.

  ### ⛔ SUPERSEDED 2026-08-05 — the capture literal is innocent (kept: that part is still true)

  **The "correct the capture literal" edit is WITHDRAWN.** Measured at
  `:12038`: `closure_scope` is built from an **empty** `BinderScope::default()`
  and binds only `AllocatedBuffer`, which **is** the closure's own `"buffer"`
  parameter, so its `Var(1)` is that parameter and the closure's ambient demand
  is **zero**. ⇒ `captures: Vec::new()` is **correct there too**, and adding a
  capture would fabricate one.

  **The defect is outside the closure**, twenty lines on:
  `allocation_scope = BinderScope::default().bind(AllocatedBuffer)` presumes an
  ambient `AllocatedBuffer` **at the `ComputationalMatch` level that nothing
  there binds** — the case declares `argument_binders: 1,
  recursive_positions: [0]`, binding only `ScopeArgument` and
  `InductionHypothesis`, while the buffer is bound *inside* the closure. So
  `bracket_case_scope.var(AllocatedBuffer)` yields the failing `Var(2)`.

  ⭐ **This is short by exactly one OUTERMOST binding, matching `D1b`'s
  measurement**, and confirms once more that the `StaticWorker` at de Bruijn 0
  is non-causal: it sits innermost, the missing binding is outermost.

  ⭐⭐ **THE CONCLUSION FOR THE CAMPAIGN: the five reds are the runtime
  CORRECTLY REJECTING a malformed fixture.** `Var: no runtime binding` is the
  right answer to an IR referencing a binder nobody bound. ⛔ **No Ken defect is
  implicated**, and nothing here is evidence of a substrate gap.

  **Pending an Architect semantics ruling** (routed `evt_1e3bz0c973egh`): should
  the governed buffer be in scope at the match level (bind `AllocatedBuffer`
  around it), or should `allocation_scope` be corrected so the case body does not
  reference it? Both change what the fixture **means**, which is why it is not
  the Steward's call. ⛔ Do not repair before that ruling.

  ⚠ **The repair must also establish, and it is UNMEASURED:** the construction
  recurs at every depth and a nested call builds indices from a **fresh
  `BinderScope::default()`** while landing under enclosing binders. Whether that
  compounds the same defect per level, or the single unbound `AllocatedBuffer` is
  the whole of it, belongs to the repair's evidence — ⛔ not to another
  measurement round.

  ### Superseded: the recursion question — the capture list is DERIVED, not chosen

  `governed_nested_resource_bracket` recurses on `depth - 1`, so one construction
  site produces a closure at every depth, and the ring asked whether the fix is
  one capture list or a depth-dependent one.

  ⛔ **It is neither, as a decision.** The Architect's ruling makes the capture
  run **exactly the body's ambient lexical demand at that construction**, so
  uniform-versus-depth-dependent is an **outcome to be measured, not an intent to
  be picked.** Derive it from the fixture's own `bind`/`var` scope tracker —
  which already models the scope and is the fixture's own statement of what its
  body reaches — and let the answer fall out. ⛔ Do **not** settle it by trying a
  uniform list and seeing whether the suite goes green; that selects the shape
  that passes rather than the shape the contract requires.

  ⛔ **HARD-STOP CASE, and it is the live one:** if the tracker's own model says
  the body should **not** reach outside the closure, then the `Var` is the defect
  and the capture list is innocent — **the opposite repair.** The ruling settled
  that `captures` must be total for ambient demand; ⛔ **it did NOT rule that
  these bodies' `Var` indices are correct**, and that question has been open
  since `D1b` measured the shortfall as exactly one outermost position. Stop and
  return it to me rather than choosing.

  ⛔ **Banned, unchanged and restated because the measurement makes one of them
  look plausible:** no `CaptureSlot` identity field, no backend-synthesized
  capture, no padding or shifting of the unit environment, no caller-tail copy.
  `D1b` measured a `StaticWorker` at de Bruijn 0 in exactly the failing units,
  which makes shifting `Var`s look like the obvious fix — it is not; both
  failures are `index == env_len` and removing the worker makes the shortfall
  **worse**.

  ⚠ **Noted and deliberately NOT filed as a node:** an end-to-end `ken-cli`
  corpus audit of capture producers. The Architect was explicit that it is **not
  required to decide this contract**, since no observed producer can make an
  undeclared ambient tail lawful. Do not derive a node from this line.

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
- **`AC-2` — the closed sums are enforced by the type, not by convention.** ⛔
  **"`D3`'s three consumers" is SUPERSEDED — the count is ten** (`D3` header,
  corrected at `evt_1srfqjmkp5eh8`); do not discharge this against three.

  ⛔ **REPLACED 2026-08-05 on the `D3c` correction: there are now TWO closed
  sums and this AC binds both.** A new **root source kind** must be unable to
  compile until every one of `D3`'s ten consumers assigns it; and a new
  **availability claim kind** must be unable to compile until both
  consumer-specific views — direct emission and ABI-only context capture —
  assign it. ⛔ No wildcard arm in either.

  ⛔ **The type cannot enforce the part that matters most, so say so here rather
  than let it read as covered.** Exhaustiveness proves every consumer *handles*
  a claim; it cannot prove a consumer holds the environment its claim names.
  That is the defect `D3c` measured — a well-typed, in-bounds, same-shape read
  of the wrong value — and it is guarded by the per-consumer claim-kind, frame
  and seat checks in `D3b`, with `D3b`'s control list as the evidence. A green
  exhaustive match is not evidence for it.
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
- ⛔ **Exempting the new arm from `validate_continuation_source_slot`.**
- ⛔ **One generic availability index serving both consumers.** ⛔ **RESTATED
  2026-08-05 — the old wording "using `immediate_slot` alone and discarding root
  provenance" named a field that is now RETIRED**, and its rationale was the
  weaker half. The ban is not merely that root provenance must be kept; it is
  that a **single unqualified index cannot be authority for two consumers
  holding different environments.** An unkeyed vector, a "first matching
  availability" search, or one generic `immediate_slot` is unlawful even when
  root provenance is faithfully retained alongside it.

  ⛔ **This ban does NOT reach the nearest-exact-singleton-alias law** (Architect,
  2026-08-05). That law is a **total** rule over an ordered semantic environment,
  applied only after exact `Closed([S])` equality has proved every eligible
  position is an alias of one semantic value, and it yields **one typed
  `CurrentLexical` claim per consumer** — not one index shared across consumers.
  ⇒ **The discriminator is eligibility, not ordering.** "First match" is banned
  because it selects from candidates that were never proved equivalent; selecting
  the minimum index from a set already proved to be aliases is not that. Do not
  cite this bullet against it.
- ⛔ **A fourth pairing added to the retired coordinate-product table**, or any
  numeric equality, constant offset, padding, reverse search, same-shape or
  same-value inference, or consumer-side fallback bridging the domains.
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
