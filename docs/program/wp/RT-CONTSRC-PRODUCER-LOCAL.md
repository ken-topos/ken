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
     unit-body boundary. **That does not make it latent.** A defect masked by a
     second defect is not lower priority when the mask is itself scheduled work.

     ⛔ **The ANCHOR moved and the old one is dead.** This clause used to read
     *"[[RT-UNIT-CLOSURE-CONVERT]] exists to clear that boundary, so landing it
     is what unmasks this defect — the correction lands before or with that
     node."* **That node is `closed` and will never land**, so the sentence
     named a scheduling anchor that cannot arrive. ⛔ Do not restore it from an
     older revision. **The mask is cleared IN THIS NODE, at `D5`** — the
     unit-body boundary turned out to be a real functionized-unit environment
     transport failure, recut as `D5a`/`D5b`, not a substrate gap. The severity
     ruling is unchanged; only what clears the mask has moved.

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
  the two function-local target tables are accepted and preserved.

  > ⛔ **THE DIAGNOSIS THIS PARAGRAPH CARRIED WAS FALSE, and it was mine.** It
  > said the mixed binding pair sits in the environment *"without ever calling
  > the selected recursive argument"*, so the corpus never consumes
  > `raw_worker_calls` and the mutation *"resolve `RawWorker` through
  > `worker_calls`"* survives **for want of a call**.
  >
  > **Measured at `400bddde`: the selected argument IS called.** The raw route
  > emits — `raw 1, supplied 1`, no suffix, against declared callee 30 — and
  > `raw_worker_calls` **is** consumed. **The mutation survives for a different
  > reason entirely:** at a body that was never retargeted, `worker_calls` and
  > `raw_worker_calls` lawfully hold **the same declared target**, so the swap is
  > byte-for-byte the same lookup and callee.
  >
  > ⇒ **Building the witness was never going to make that mutation red.** The
  > requirement rested on the wrong cause, not on a missing fixture. Rulings:
  > leader `evt_31bj4wm8064xr`, Architect `evt_4125xc5qw8g0x`.

  ⛔ **Do not conflate the two, here or anywhere else in this frame.** *"The
  mechanism is unreached"* is a real gap and may never be accepted as delivered
  semantics. *"Both tables hold the same entry, so there is no callee-selection
  decision at this seat"* is **the absence of a distinction** — nothing is left
  unproved, and no mutation can discriminate what the representation does not
  encode.

  The witness is a test fixture built from **existing source constructs**.
  Ordinary source syntax in a fixture is authorized; **a new production
  occurrence kind, authority, fabricated identity, population mechanism, or
  hand-constructed plan is NOT.** It must prove: same exact closure/body
  provenance yields a `GeneratedContext` IH and a `RawWorker` selected
  argument; the argument is invoked at the planner-derived binder position
  with its real raw arity and captures; the emitted raw event carries **no**
  generated-context suffix; and the existing `GeneratedContext`
  append-nothing mutation **stays red**. All four hold at `08f1c2f7`.

  **The *"selected recursive argument is actually called"* conjunct is ALREADY
  DELIVERED — credit it, do not rebuild it.** Architect `evt_6grnfx2psztcn`.
  Through ordinary production planning and lowering, the selected recursive
  argument **is** called at the **source-machine** seat with its exact raw
  arguments and captures, and its result is consumed in the same continuation
  with no closure-valued unit result. That is the accepted **`D8e`/`D8j`**
  evidence, and it is `D6b`'s positive for this conjunct.

  > **Do NOT read the *"`D6b` IS MIS-SIZED"* section below as closing the
  > source-machine path.** `evt_6azsr4xrch1he` classified the then-current
  > refusal as a **production representation/consumer gap** and **prescribed
  > the repair** — install the target-derived `StaticWorkerBinding` at the
  > selected recursive position, then consume it at the source-machine callee
  > seat ahead of the value-only `Var` path. **`D8d` and `D8e` delivered
  > exactly that** (execution order rows 3 and 4/9, both DISCHARGED). The
  > section describes the **pre-repair** state.

  **What `08f1c2f7` adds is a NARROWER negative, and it is retained as a local
  fail-closed control.** The **ordinary-unit copy** of that case body carries
  only a **word** at this position, so using it as a specialized callee must
  fail closed — measured as `Unsupported(BoundaryCarrier, ...)`. **It does not
  generalize over the composed source-machine population and cannot retire the
  already-proved `D8e` positive.**

  **WITHDRAWN from this witness — one conjunct: *"resolves through
  `raw_worker_calls`"* read as an ATTRIBUTION claim.** The emission does
  resolve through that table; what no fixture can prove is that it is *that*
  table rather than the other. See the `D6b` LIMITS block below for why that is
  a stated proof limit and not missing instrumentation.

  ⛔ **WITHDRAWN: *"substituting `worker_calls` for that lookup reddens it."***
  It is **structurally unsatisfiable at an equal-table seat** — see the
  correction above. **The lawful law is ASYMMETRIC and conditional on actual
  retargeting**, per Architect `evt_4125xc5qw8g0x`:

  | # | clause |
  |---|---|
  | 1 | `RawWorker` consumes the raw operand run, appends **no** context suffix, resolves through `raw_worker_calls` |
  | 2 | `GeneratedContext` consumes the **same raw prefix plus its exact planned suffix**, resolves through `worker_calls` |
  | 3 | **Where retargeting makes the table answers differ**, swapping the context route to the raw table must refuse **at the earliest structural guard**. ⇐ **this existing live negative IS the callee-selection discriminator** |
  | 4 | **Where no retarget occurred and the entries are equal**, a table swap is **observational identity — not a missing negative** |

  ⛔ **Do NOT narrow `worker_calls` to context-only targets to manufacture a
  red.** It would replace the established *"declared worker targets, selectively
  overwritten by retargeting"* contract at all three population sites
  (`units.rs:1404`, `:2042`, `:3236`), create new absence semantics for ordinary
  and generated bodies, and change production solely so a test can fail. **No
  production consumer requires it.**

  ⛔ **Fail closed on the population edge:** if the ordinary fixture reaches a
  template-only raw body and `raw_worker_calls` cannot be populated without
  declaring an undefined function, **stop and report that exact edge.** Do not
  force-declare a body to make the test green.

  This belongs to `D6b`, **not** `D6c` — `D6c` owns the refusal closure *after*
  this positive is established.

  ###### `D6b` LIMIT — the ONE thing no fixture delivers, and why it is permanent

  > **Scope, stated first because an earlier draft of this block got it wrong.**
  > This limit covers the **retargeted raw-body / raw-table attribution**
  > conjunct and **nothing else**. The *"selected recursive argument is actually
  > called"* conjunct is **NOT** a limit — it is **delivered** by `D8e`/`D8j`
  > (Architect `evt_6grnfx2psztcn`). Do not extend what follows to it.

  **`D6b` previously owed the raw-target declared-call-table representation**,
  on the reading that `D6a` leaves it honestly inert: the body-origin lookup is
  overwritten by the generated context and the raw body is removed from the
  executable population, so a raw route is not consumable in a retargeted
  specialization. **The observation is true. The obligation is WITHDRAWN**, and
  what follows is why it cannot be re-derived.

  **The only known mechanism is BANNED, on measurement.** Making that lookup
  answerable means retaining the raw body as a **declared-and-defined
  `Function`**. That was built and measured: it defines a standalone `Function`
  whose result is a `Constructor` containing a raw `Closure`, and **25 newly-red
  rows** all stop at the permanent unit-result closure boundary, causally
  isolated to one retention predicate (`741/2` unarmed, `716/27` armed). It is
  **not a missing predicate** — it proves the retention **reopens the exact
  boundary the generated-context design exists to avoid.** Architect
  `evt_3dcafs581921e` Finding 2 withdrew `D7a2`'s raw-body executable-set
  equality on that measurement.

  **Do NOT** flip the retention predicate, force-declare a body, or substitute
  the existing IH `GeneratedContext` — it has different semantics and operands.
  **Do NOT read the `D7a`-`D7e` block as a live route**: that whole cut is
  SPENT (superseded by `D8a`-`D8g`), and its surviving prose is the reason this
  mechanism kept looking available.

  ###### The attribution limit — an ABSENCE OF DISTINCTION, not a coverage hole

  Measured at `08f1c2f7`, at the body-definition seat with no mutation armed:

  ```
  spec 0  worker body 36  retargeted  worker_calls {5,33,36,40}
                                      raw_worker_calls {5,33,40}
  spec 1  worker body 33  plain       both tables {5,33,40}
  ```

  **The two tables disagree exactly where a retarget happened, and nowhere
  else.** So at the only body that emits a raw call the tables are **equal** —
  the other one answers identically — and at the only body where they **differ**
  no raw call can be emitted at all, because body 36 is in the emittable
  population and absent from the executable one.

  ⇒ **Raw-table attribution is unprovable, structurally.** That is clause 4 of
  the asymmetric law above: where the entries are equal a table swap is
  **observational identity**. No mutation can discriminate what the
  representation does not encode, so nothing here is left unproved.

  **Nothing retires this limit — do not label it a transition sentinel.** A
  sentinel names the obligation that clears it; this has none. The mechanism
  that would clear it is banned on measurement, and the attribution consequence
  is a property of the representation rather than a gap in it. Labelling it as
  pending leaves a later reader waiting for a checkpoint that will never be
  cut, which is precisely how the spent `D7a2` pointer stayed live.
  **`08f1c2f7`'s "transition sentinel" wording must become this permanent
  limit** (Architect `evt_6grnfx2psztcn`).

  **What these rows DO own:** which bodies each table answers **for**. A defect
  keeping both key sets exact and changing the `FuncRef` one entry holds is
  invisible to them — the same family as `D8g`'s residual, caught at the emitter
  if at all.

  ###### `D8g`'s LANDED EVIDENCE for the clauses above — read before building

  **`D8g` discharged at `400bddde` and measured things `D6b` would otherwise
  re-derive.** Two of them cut in opposite directions, so take both:

  1. **A raw route DOES reach emission.** On `px8tr` the two routes emit against
     **distinct** declared callees — `FuncRef` 33 for the generated context,
     30 for raw — where the **body origin is identical on both by design**
     (that is `D6a`). ⇒ **Do not build the emission witness as though no raw
     route ever emits**, and do not use body origin to tell the routes apart.
  2. **The premise above was INTERROGATED against that evidence and is NOT
     contradicted.** `D8g`'s `WrongTable` mutation refuses with *"no
     `raw_worker_calls` target for body origin"* — which is what "the raw body
     is removed from the executable population" predicts for the **retargeted**
     body. ⛔ **This is an inference from a refusal message, not a direct
     measurement of the retargeted population. Re-measure before building, and
     if the representation turns out already present, CLOSE the obligation and
     say so — do not manufacture the work to match this paragraph.**

  **`D8g` also settled where a refusal may land** — see `TWO PROOF PLANES`.
  `D6b`'s *"substituting `worker_calls` for that lookup reddens it"* is
  discharged at the **earliest structural guard**, pre-emission, with an empty
  emission set. It does **not** owe a post-emission relation failure, and the
  refusal must be matched on the **typed** error — category plus discriminating
  reason — never `is_err()` or a `contains` over formatted text.

  **`D6c` — the refusal set, pre-emission.**

  > ⛔ **NOT `D8f`'s refusal set, despite opening with the same two words.**
  > `D8f` owes *omission, duplicate, transplant, wrong occurrence* about **which
  > call consumes a pending checked-IH marker.** `D6c` owes the list below about
  > **selecting the raw/IH target and its selected-recursive-argument member,
  > pre-emission.** ⛔ **Different subjects, overlapping vocabulary, same frame,
  > both owed.** Discharging one discharges **nothing** of the other; say which
  > you mean whenever you write "the omission refusal" on this node.

  Refuse on omission, duplicate,
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

  ##### `D6b` IS MIS-SIZED — the two lowering paths DIFFER at the recursive field

  > **The substrate claim in this heading named `D7`, which is now SPENT.** It
  > read *"`D7` is the substrate, and it RUNS BEFORE `D6b` CLOSES."* The
  > substrate work was recut as `D8a`-`D8g` (Architect `evt_3dcafs581921e`) and
  > is discharged through `D8g` at `400bddde`, so **there is no longer a
  > predecessor pending between here and `D6b`'s close.**
  >
  > ⇒ **The ruling below describes a gap that has since been REPAIRED. It is
  > the history of that repair, NOT a live prohibition.** `evt_6azsr4xrch1he`
  > classified the refusal as a production representation/consumer gap and
  > **prescribed** the fix; **`D8d` and `D8e` delivered it** (rows 3 and 4/9,
  > both DISCHARGED), and `D8e`/`D8j` is now the accepted positive that the
  > selected recursive argument **is** called at the source-machine seat.
  >
  > **Do NOT cite this section to argue the source-machine path is closed.**
  > Doing so converts a discharged repair back into a limit and would retire an
  > already-proved positive. Architect `evt_6grnfx2psztcn` corrected exactly
  > that misreading.

  **Architect ruling `evt_6azsr4xrch1he`, 2026-08-05**, accepting the composed-path
  hard stop on preserved exact `d86be55d`. `d86be55d` is preserved as **accepted
  partial progress**, not a residual.

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
  the only source shape that would exercise the raw table is **refused earlier**.
  ⛔ **Do not accept the raw table as a permanently unwitnessed residual**;
  calling `D6b` complete would turn an unreachable mechanism into delivered
  semantics.

  > ⚠ **SUPERSEDED CLAUSE — this paragraph used to add *"which is exactly why
  > its wrong-table mutation stays green."* That causal claim is FALSE and is
  > removed.** Measured at `400bddde`: the raw route **is** reached and emits,
  > and the mutation stays green because at an unretargeted body both tables
  > hold the **same** declared target. **The unreachability finding above is
  > unaffected — it is about a different source shape.** Do not re-derive
  > "the raw table is unreached" from the mutation's greenness. ⛔ **And do not
  add a carrier callable/helper route** — worker body, arity, captures and route
  are already compiler/planner facts, so encoding callable identity or a template
  descriptor in the carrier would widen the ABI/runtime boundary, duplicate the
  static authority, and violate the compiler-only closure-capsule ruling.

  ###### The cut — RECUT TWICE. Read the `D8` block below, not this paragraph.

  **In-node checkpoints, not a predecessor node** (Steward sizing call, which the
  ruling leaves to me). It lands on the same branch in the same candidate, so a
  node adds a tracker object and a critical-path entry while buying **no** merge
  boundary — the same reasoning that closed [[RT-UNIT-CLOSURE-CONVERT]].

  **The first cut was `D7a`-`D7e`; it is spent.** The governing cut is
  `D8a`-`D8g`, immediately below, on Architect ruling `evt_3dcafs581921e`,
  **extended by `D8h`-`D8k` on ruling `evt_37fa3rdegb3yc`** — see *"`D8e`'s
  causal obligation is real and gets a SECOND DISCHARGE FORM"* below.

  ###### GOVERNING — `D8e`'s causal obligation is real and gets a SECOND DISCHARGE FORM. Architect `evt_37fa3rdegb3yc`, outcome (c).

  **Ruled on preserved exact `89e36ec1`, which stays as non-candidate,
  load-bearing evidence.** The witness is lawful, the `D8d` installation and
  `D8e` source-machine consumption are real, and the whole-node stop was
  correct. **Both mechanisms I offered were refused, and the refusals are the
  useful part:**

  | offered | why refused |
  |---|---|
  | (a) a claim seat reachable from the composed path | `claim_and_call_continuation` resolves the specialization target, emits its direct call and records the decoded callee. The composed path has **already** invoked the selected raw worker and handed its result to the exact source-machine continuation. Emitting the specialization too would **execute the continuation twice** — so moving the call site is not a repair |
  | (b) exempt composed producers from result-edge projection | the causal identity is planner-issued and **real**. Exempting would erase an obligation merely because one consumer cannot express its discharge — and would make the same five-field specialization both create and not create a causal obligation depending on which projection is queried |

  ⇒ **The false law is "every planned causal identity has exactly one execution
  form."** The fix is a typed alternative discharge of the *same exact* causal
  identity, not an exemption and not a second call:

  ```
  ContinuationDischarge = DirectSpecializationCall | ComposedSourceContinuation
  ```

  **The projected identity remains in the planned population exactly once.**

  ###### SIZING RULING — four in-node checkpoints, NOT a predecessor node

  The Architect left this to me and named four separable obligations. **They
  are `D8h`-`D8k` on this node.** Two grounded reasons, per
  `agent/playbooks/federation/steward.md` §4's preference order relax → fold →
  cut, where **fold succeeds**:

  1. **The work cannot stand alone.** All four build on `D8a`/`D8b`/`D8d`,
     which are **unmerged on this branch**. A predecessor node's `depends_on`
     would name unmerged checkpoints *of its own successor* — the shape that
     needed three paragraphs of "this is not a deadlock" prose when
     [[RT-UNIT-CLOSURE-CONVERT]] was in it. Do not recreate it.
  2. **This node is the sole gate on the seven-node `RecursiveDescent`
     retirement campaign.** A node adds a merge boundary, a candidate and a
     critical-path entry; checkpoints add none of those and buy nothing here.

  **The constraint that would have demanded a node is "this node is getting
  long."** That is graph aesthetics, not a spec rule, a `docs/PRINCIPLES.md`
  commitment, a measured capability gap or a ruling — so under §4 it is **not
  grounded** and does not get a node.

  ###### `D8h` — planner pairing. The authority, issued where it is known.

  Pair each `ComposedCallTarget` with the **exact opaque
  `ContinuationCallIdentity`** selected by that target's own causal coordinate.
  The pairing is available without reconstruction, from the same five fields as
  the `D8a` selector: emission owner, producer `Construct` origin, continuation
  origin, producer alternative, recursive position. **The existing planner
  lookup supplies the opaque call-site sequence inside the identity.**

  ⛔ **Lowering must not derive the identity from body, symbol, arity, source
  position, or a same-shaped constructor.** This ban is what closes the
  same-symbol shortcut `89e36ec1` correctly refused — **no constructor-symbol
  equality may participate in the new authority.**

  ###### `D8i` — transport it on the binding, as a SEPARATE closed facet

  `D8d`'s target-derived `StaticWorkerBinding` carries the opaque discharge
  authority. ⛔ **It is NOT part of `StaticWorkerCallRoute`.** Route decides
  callee plus operand run; **causal discharge says which continuation
  obligation this one composed consumption satisfies.** Two different questions,
  and collapsing them is the error to avoid.

  **Ordinary static-worker bindings get an EXPLICIT non-composed arm** — not a
  default, not a missing field, not an `Option` read as absence.

  ###### `D8j` — verified discharge at the existing seat, and its own relation

  `D8e` consumes the authority **only** at its existing exact source-machine
  `StaticWorker` callee seat. For a non-empty argument run it must survive
  through `SourceContinuation::CallArgument` and be consumed **only when the
  common static-worker emitter has emitted the call and its result is returned
  into the unchanged source-machine control.**

  ⛔ **Installing the binding, beginning argument evaluation, or observing a
  worker-shaped value is NOT discharge.** Three near-misses, named because each
  is locally plausible.

  ⛔ **Do NOT insert the raw-worker instruction into
  `function_local.continuation_emissions`.** That map's invariant is that
  decoding the instruction yields `identity.target()`, and the composed
  instruction **lawfully targets the raw worker instead**. Add a **separate**
  function-local typed relation for composed discharges, held to the same
  evidence standard as the direct form:

  1. the opaque causal identity came from the exact planner target;
  2. the claiming function is `identity.emission_owner()`;
  3. the **finished CLIF** contains the recorded raw-worker call;
  4. the decoded callee and operand contract agree with the exact `D8b`/`D8d`
     target;
  5. the call result is returned into the same source-machine continuation.

  **Only after that function-local verification may the whole-pass ledger
  record the composed discharge.**

  ###### `D8j` 4b needs its OWN non-degenerate control. Architect `evt_447qfaj9ddd3s`.

  **Check 4 is two checks and only the callee half had a live
  discriminator.** At `f854bd78` the operand-run comparison executed **only on
  the degenerate equality `1 == 1`** — the sole root-owned CLIF witness has two
  workers with identical declared runs (arity one, zero captures). Caught by
  `runtime-leader`; the implementer did not claim otherwise.

  ⛔ **`D8k` MAY NOT INHERIT THIS DISCRIMINATOR**, and the reason generalizes
  past this node: *"a defective or vacuous operand comparator could admit the
  wrong local discharge and `D8k`'s set equalities could still close
  perfectly."* **Operand-contract agreement is a LOCAL ADMISSION NET of `D8j`,
  not a global-ledger property of `D8k`.** A partition cannot see that one of
  its members was admitted on a wrong fact.

  The required control, and its four properties:

  1. a real root-owned source-machine composed call, emitted through the
     ordinary planner/lowering path;
  2. exact causal identity, paired target, claiming owner, recorded
     instruction, decoded callee and downstream result all remain **correct**;
  3. **only the supplied-run evidence disagrees** with that exact target's
     declared raw run;
  4. **verification 4b is the FIRST refusal**, and the composed-discharge
     relation remains empty.

  **Narrowest lawful form:** a `cfg(test)`-only mutation of
  `StaticWorkerEmission.supplied_operands` **after a real call has assembled
  and emitted its operand vector**. The positive path must keep deriving that
  field from the actual assembled `inputs.len()` adjacent to the emitter.
  ⭐ **The mutation changes an INPUT handed to the verifier, never the
  comparison itself.** A lawful differing-arity/capture witness is also
  acceptable.

  ⛔ **Substituting the whole other target is NOT a control** — verification 1
  or 4a refuses first and **masks** 4b. This is the short-circuit trap: a probe
  that trips an earlier guard measures that guard, not the one under test.

  **If the bounded control cannot reach 4b without an earlier refusal,
  preserve the attempt and route that exact refusal** — do not fabricate a
  count, instruction, identity or plan. No production mechanism change is
  authorized by this ruling.

  ###### `D8k` — the global law becomes a PARTITION, not a weakened count

  ```
  planned = resolved = declared = claimed
  planned = direct-emitted ⊎ composed-consumed
  direct claims   = decoded direct-specialization emissions
  composed claims = verified composed source-continuation consumptions
  ```

  **The union is disjoint and equality is over exact identity sets.** Existing
  declaration of planner targets may remain over the full planned set — **an
  unused declaration is not an emitted call.** The direct-emission closure scan
  is **unchanged**.

  `continuation_result_edges_owned_by` is also **unchanged**. The detached-result
  seat filters identities already discharged by **either** verified form. ⇒ The
  `D8e` witness has no residual edge, **while a program that merely skips the
  projection or suppresses either evidence still fails global closure.** That
  second clause is the whole point: this must not become a way to opt out.

  ###### Required discriminators — the recut must make these NON-VACUOUS

  Architect-specified, and they are acceptance rather than suggestions:

  | perturbation | required outcome |
  |---|---|
  | suppress the composed discharge after the real raw call | refusal |
  | present another exact identity, **including the same-symbol shortcut** | refusal |
  | redirect or suppress the raw-worker instruction while recording discharge | **finished-CLIF** refusal |
  | claim one identity once by **each** form | duplicate / disjointness refusal |
  | present the composed authority under the wrong emission owner | refusal |
  | discharge an **ordinary** `StaticWorker` binding as composed | refusal |

  **Retain the present neighbouring-IH and value-position controls** — they are
  not superseded by these.

  ###### `D8e`'s disposition, stated exactly

  **`D8d` and the `D8e` consumer are accepted progress. `D8e` is NOT
  discharged**, because its positive program still cannot close the causal
  ledger. It discharges at row 9 of the execution order, once `D8h`-`D8k`
  land — **not before, and not by re-measuring `89e36ec1`.**

  ###### GOVERNING — the `D7` cut is MIS-SIZED. The work is `D8a`-`D8g`.

  **Architect ruling `evt_3dcafs581921e`, 2026-08-05, outcome (c).** Ruled on
  preserved exact `f3427dae` (`D7a`) and its child `9f21ff0e` (`D7a2`). Both are
  preserved as **non-candidate evidence**. `D7b`-`D7e`, `D6b` closeout, `D6c`,
  QA, candidate, `D6` closure and downstream are **frozen**.

  **The `D8` series is allocated so that label order IS execution order.** The
  `D7` letters were allocation order and misled twice, which is why this frame
  needed a separate ordering table at all. Do not allocate a `D8x2`; if a
  checkpoint needs splitting, renumber the tail.

  **Everything below this block about `D7a`, `D7a2`, `D7b` and `D7e` is spent
  except where a `D8` checkpoint names it as preserved.**

  ###### Finding 1 — `f3427dae` is BLOCKED: the four-field selector is owner-incomplete

  Adding `producer_construct_origin` was necessary and its provenance re-checks
  are sound. It is **not sufficient as the final identity.**

  The planner already distinguishes discoveries by `enclosing_specialization`,
  interns specialization keys with `emission_owner`, and **deliberately interns
  distinct generated contexts for the same raw worker reached under different
  continuation identities.** The lowerer retains `defining_emission_owner` as an
  independent fact. Yet `composed_worker_view` searches all units on the four
  source coordinates alone, and **its own contract admits multiple answers
  differing only in emission owner.**

  **That multiplicity is not harmless.** A generated route carries
  `GeneratedContextIssued(context_id)`, so two enclosing specialization owners
  can name **different exact callees** while sharing all four source
  coordinates. The caller holds the owner needed to tell them apart. Collapsing
  them and accepting only when their complete views happen to agree makes
  **agreement, rather than causal identity, the selector.**

  **The false step was mine.** I justified the four-field selector as "the same
  causal coordinate `continuation_call_binding_for` already uses". That lookup
  may **fail closed on duplicate call tokens**; the composed view instead
  **projects an owner-specific generated-route answer**. The two are not
  analogous, and the analogy is what made four fields look sufficient.

  ###### Finding 2 — `9f21ff0e` FALSIFIES `D7a2`'s own execution premise

  The hard stop is **accepted**. Retaining the required raw body defines a
  standalone `Function` whose result is a `Constructor` containing a raw
  `Closure`; all 25 newly-red rows stop at the permanent unit-result closure
  boundary, and the toggle is causally isolated to one retention predicate
  (`741/2` unarmed, `716/27` armed).

  **This is not a missing predicate.** It proves that *"make the raw body a
  declared-and-defined `Function`"* **reopens the exact boundary the
  generated-context design exists to avoid.**

  **Withdrawn, and all three are withdrawals of my own frame text:**

  | withdrawn | where I wrote it |
  |---|---|
  | `D7a2`'s raw-body **executable-set equality** | the four-step population closure, from `evt_7x6knchb4rb1n` §2 |
  | `D7e`'s *"prove the raw target is both declared and defined"* | same |
  | the four-field selector as **final identity** | the `D7a` recut published at `f0167927` |

  **Banned, restated because the measurement makes each one look plausible:** no
  closure carrier, no ABI/runtime lane, no boundary exemption. Do **not** flip
  the test-only retention predicate. Do **not** substitute the existing IH
  `GeneratedContext` — it has **different semantics and operands**.

  ###### The lawful mechanism family: a planner-issued COMPOSED-CALL TARGET

  It is **distinct from both** the standalone `RawWorker` `Function` and the
  existing IH `GeneratedContext`. It must:

  - **preserve** the selected recursive argument's raw argument and capture
    semantics;
  - **consume its result in the exact source-machine continuation**, with the
    closure-valued result **never crossing a unit boundary**;
  - carry an **owner-qualified** identity, and an **occurrence-qualified** one
    wherever more than one source call can consume the binding.

  No first-call, shape, arity, or *"whichever target exists"* rule is lawful.

  ###### EXECUTION ORDER — and label order NO LONGER tracks it. Read the numbers.

  | # | checkpoint | owner layer | status |
  |---|---|---|---|
  | 1 | `D8a` — owner-qualified composed selector | planner projection | DISCHARGED `e02ef413` |
  | 2 | `D8b` — composed-call target, planner representation | planner | DISCHARGED `e4b4c26c` |
  | 3 | `D8d` — install the one target-derived environment binding | lowering environment | DISCHARGED `c2e8314f` |
  | 4 | `D8e` — consumer at the source-machine callee seat | lowering consumer | consumer DISCHARGED `70171a99`; **witness proved `89e36ec1`; NOT discharged** |
  | 5 | `D8h` — planner pairing: target to exact opaque causal identity | planner | **DISCHARGED** `a435d180` |
  | 6 | `D8i` — transport the discharge authority on the binding | lowering environment | **DISCHARGED** `abfd67ff`. Clause 2 is **self-correlated** and is not evidence for specialization-body owner correctness; re-measured under `D8o`'s plan-derived authority at `5154d94a` |
  | 7 | `D8j` — verified source-machine discharge + function-local relation | lowering consumer + proof | **DISCHARGED** `86fe8bc6`. Verification 2 carries the same ambient-agreement limit; re-measured at `5154d94a`, positive and negative |
  | 8 | `D8k` — partitioned global closeout | proof | **DISCHARGED** `372097de` |
  | 9 | `D8e` DISCHARGES — causal obligation dischargeable and discharged | — | **DISCHARGED, Steward ruling below** |
  | 10 | `D8l1` — MEASURE the envelope frontier | planner | ANSWERED at `aaef1772`: **not structural** |
  | 11 | `D8l2` — repair `ordinary_envelope`'s nonrecursive population | planner | DISCHARGED `1f9a2020`+`abe46dda`, **QA-approved** |
  | 12 | `D8n` — per-`Function` consumption lifecycle | lowering compiler state | **DISCHARGED** `3a63fe64`, after two bounded evidence corrections |
  | 13 | `D8o` — ambient body authority | lowering compiler state | **DISCHARGED** `3aadbaae`, after two evidence blocks (`5154d94a`, `d3f7e32c`) |
  | 14 | `D8m` — preserve the source frame through the bridge | lowering preservation | **DISCHARGED** `01263e44` (`760/2/1`), after a matrix block and two fidelity children |
  | 15 | `D8p` — cross-body checked-application binding contract | planner + unit definition + lowering | **DISCHARGED** `2be7adb0` (Architect `evt_6sk9hbp17893t`), after a mechanism-accepted evidence block at `198b7c14` |
  | 16 | `D8f` — checked-marker occupancy | integration | **DISCHARGED** at exact `04c40d4c` (Architect `evt_6cka0ecwnd25t`). Three-case disposition partial accepted at `c49d0652`; the four refusal controls then took **five** bounded evidence holds, closing at `d94a3df9` with both wrong-occurrence origins named from planner authority. `04c40d4c` is a whitespace-only identity child of it |
  | 17 | `D8g` — non-vacuous closeout, over TWO populations | proof | **Owner re-measurement DISCHARGED** `1b367065` (Architect `evt_7v6ve6rsc8d39`) — **outcome (1): the owner VARIES but SEPARATES nothing**, so the prohibition stands and no owner positive is required. **Closeout then HARD-STOPPED** (`evt_9yrrhnjp6dfd`): its two facts live in disjoint witness families. **RECUT over both populations** by Architect disposition `evt_6afmy0398n847`; frame clauses replaced 2026-08-06. Closeout then ran four review rounds — `b0614513`, `337ee9d9`, `6d884090`, `90f96c00`, `0665b9d5` all preserved as held progress. **Two review requirements were withdrawn as STRONGER THAN THE FRAME**: post-emission-FIRST refusal for the mutations (`evt_5vwdtrznf3km4` — earliest structural guard is accepted, and is the stronger evidence), and the independent `(defining Function, call origin)` expectation (`evt_399j195rn8fpa` — **no authority exists to derive it from**; see `EXACT-KEY LIMIT`). **SECOND RECUT landed 2026-08-06.** ⇒ **DISCHARGED at exact `400bddde`** (Architect `evt_4mzks855xpw8h`) |
  | 18 | `D6b` closeout | proof | **DISCHARGED at exact `c47d94a7`** (Architect `evt_61y1n1y7kpj3f`), reached as `08f1c2f7` → `b7d34bf7` → `c47d94a7`. `08f1c2f7` was the accepted bounded evidence block, not the completion object (Architect `evt_6grnfx2psztcn`; three commits on `400bddde`, `771/2/1`, no production change). **ONE conjunct retired as a permanent LIMIT**: *"resolves through `raw_worker_calls`"* read as attribution — the tables are equal wherever a raw call emits, the body where they differ emits none, and the retention that would change it is banned on measurement (`evt_3dcafs581921e` Finding 2). **The *"selected recursive argument is actually called"* conjunct is DELIVERED, not limited** — `D8e`/`D8j` is the accepted positive on the composed source-machine path; `08f1c2f7`'s `BoundaryCarrier` row is only the **ordinary-unit copy** and is retained as a local fail-closed control. The remaining work was a **comment/failure-text-only fidelity chain** and it landed: `b7d34bf7` retired two false source contracts, `c47d94a7` named the discriminator the row actually has — proved by inversion, substituting the identifier back reproduces the parent blob byte for byte |
  | 19 | `D6c` — the refusal set, pre-emission | proof | **DISCHARGED at exact `bd79daa7`** (Architect `evt_41j4y5rpy4zfc`). **Entirely a refusal set, so the refusal plane is its whole proof** — see `TWO PROOF PLANES`. Mechanism accepted at `93424655`: a checked, producer-local, exhaustive canonical-run postcondition in `continuation_case_binder_run`, five refusal rows, two positives, the `px8tr` order discriminator and the governed equal-value limit. `e7390f2e` → `93424655` → `8ec921aa` → `bd79daa7`, the last two comment-only |
  | 20 | `D9` — assemble the ordinary run from the planner envelope | lowering producer | **NEXT.** Found by node QA, not by a checkpoint review: `AC-1` had no owning deliverable. Classified in-node with no new planner/ABI authority (Architect `evt_1y7h08xd7ermp`) |

  ⛔ **`D8h`-`D8k` execute BEFORE `D8f`/`D8g`, despite sorting after them.**
  This heading previously read *"label order, for the first time on this
  node."* That is now false and the tidier fix was rejected on purpose.

  **Why not renumber `D8f`/`D8g` down and keep the invariant.** Neither has ever
  been released or handed off, so renaming them costs nothing *outside* the
  frame — but both already carry public meaning in the thread. *"`D8f`/`D8g`
  remain held"* appears in the `D8d`, `D8e`, witness-continuation and hard-stop
  posts, and in the Architect's ruling. Reassigning the labels would silently
  redefine every one of those sentences after the fact. **This node already
  ruled on exactly this trade at `D8c`: a deliberate gap beats reusing a spent
  label for different content.** The same reasoning forbids the reuse here, so
  the ordering invariant is what gives way.

  ###### STEWARD RULING — the `D6` cut is CLOSED; QA then BLOCKED. See `D9`.

  > **AMENDED 2026-08-06 after node QA. Two claims below were WRONG when I made
  > them and are corrected in place, not appended to.** QA blocked the candidate
  > on `AC-1` (`evt_28qa9ee7wx1sk`), which **no deliverable had ever been
  > assigned to close.** So *"the whole ledger is discharged"* was true only of
  > the `D`-checkpoints, and *"next unit: node QA"* skipped a unit that did not
  > yet exist. **`D9` below is that unit.** Everything about the `D6` cut and
  > the two candidate gates is unaffected and still holds.

  **Issued 2026-08-06, answering `runtime-leader`'s `evt_9gq3s2n9rcj2` and the
  Architect's disposition. The leader's frame read was correct and its refusal
  to infer a unit was right** — the `D6` cut defines exactly `D6a`, `D6b`,
  `D6c` and nothing else, so there was no closure action to find.

  **"`D6` closure" in the held-language of this frame names the cut COMPLETING,
  not a fourth checkpoint.** All three are discharged — `D6a` `70d4e8d9`,
  `D6b` `c47d94a7`, `D6c` `bd79daa7` — so `D6` is closed by that fact alone.
  Nothing further is owed at `D6` and **no `D6d` may be allocated.**

  **Every `D`-checkpoint is discharged.** `D0`-`D4` landed; `D5`'s work is the
  `D6a`/`D6b`/`D6c` cut above (`D5a-1`/`D5a-2` spent, GOVERNING block); the
  `D8` series is complete across all nineteen execution-order rows. **That is
  not the same as the node being acceptable** — see the amendment above and
  `D9`.

  **The two candidate gates this frame sets are both MET, and I checked each
  against its own text rather than against the discharge table:**

  | gate | where | status |
  |---|---|---|
  | `D8o` plus the bounded remeasurement | *"`D8o`'s candidate gate"* | `D8o` discharged `3aadbaae`; the remeasurement is recorded at `5154d94a` — `D8i` clause 2 and `D8j` verification 2 both hold positive and negative, with the specialization-body composed-claim population recorded **empty rather than fabricated**, which is what that gate demanded |
  | the retroactive stale-owner question | *"`D8o` — the RETROACTIVE question"* | ANSWERED `evt_t4e294qz932v`: nothing invalidated, `D8a`-`D8k` not reopened, one self-correlated control identified and now carried as a stated limit on rows 6 and 7 |

  **"Correct and unreachable" is the DEFAULT still binds the candidate** and is
  not discharged by this ruling — it is QA's to evidence, per the standing
  section above. `D8e`'s own layer is exercised end to end, which is what
  distinguishes it from the `D6b` limit.

  ⇒ **Next unit: `D9`.** Then QA **resumes at `AC-2`** — `AC-1`'s siblings were
  never reached and do not need re-running. `AC-1` and `AC-1b` are **per row,
  never a total**, and `AC-1`'s row refuses at a different site from the other
  five, so report the two populations separately. `AC-4` is workspace green
  **in CI**, never a local `--workspace` run (`COORDINATION §12`).

  ###### `D8c` IS RETIRED — folded into `D8e`. Architect `evt_nwgvvr4vaf7y`.

  **Outcome (c) at the CHECKPOINT boundary, not the whole-node boundary. The
  node remains well-sized** — `D8c` was simply ordered before the mechanisms
  that make its law meaningful. Preserve exact `e4b4c26c`; **build neither form
  the implementer returned.**

  **My error, stated plainly: `D8c`'s consumption statement is an INTEGRATION
  PROPERTY, not a predecessor mechanism.** I read the mechanism family's three
  properties as three checkpoints, when the third is a property **of the
  composition of the other two**. Only `D8d` + `D8e` together can establish it:
  `D8d` owns binding, `D8e` owns consumption, and the promised fact — exact
  arguments and captures consumed in the source-machine continuation with no
  closure-valued unit result — is what their composition demonstrates.

  **Both returned forms are unlawful, and each fails against a different item of
  this frame's own ban list:**

  | form | why it is not lawful |
  |---|---|
  | resolve the `D8b` target in `source_call_state` from a threaded `D8a` selector | a **second target-selection authority** in the consumer. It bypasses the binding authority `D8d` requires and duplicates the `D8e` contract; the carried word cannot validate the selector, so the late lookup is **not self-authenticating** |
  | install a target-derived `Lowered::Closure` as `Value` | a **second callable representation** beside `StaticWorkerBinding`. It lets the template enter **value positions** and bypasses the route-selected emitter `D8e` requires |

  **The corrected version of form 2 is exactly `D8d`** — install a
  target-derived `StaticWorkerBinding`, not a specialized closure value. **That
  binding is intentionally unreadable until `D8e` supplies its sole callee
  consumer**, which is precisely why it could never have discharged a `D8c`
  standing on its own.

  **Banned in the recut:** a temporary `Value(Closure)` bridge, and any
  consumer-side target lookup that `D8d`/`D8e` later replace. A scaffold that
  the next checkpoint deletes is not a checkpoint.

  **The label `D8c` is not reused.** The gap is deliberate — label order still
  ascends, and reusing a spent label for different content is worse than a gap.

  **Still orthogonal and still required:** the queued deletion of `D8b`'s
  unreachable owner-collision guard. It moves to the `D8d` handoff. It does not
  make anything executable and was never load-bearing for `D8c`.

  **Still in-node checkpoints, not a predecessor node** (Steward sizing call).
  The reasoning is unchanged and the new finding does not touch it: this lands
  on the same branch in the same candidate, so a node buys **no** merge boundary
  while adding a tracker object and a critical-path entry. Same reasoning that
  closed [[RT-UNIT-CLOSURE-CONVERT]].

  ###### The base, and what it still carries

  **Base for `D8a` is preserved exact `9f21ff0e`**, not `f3427dae` (Steward
  call). `D8b` names specific `9f21ff0e` work as preserved — the split of
  `composed_worker_view`, the selector-agreement law — and rebasing to
  `f3427dae` would throw that away to re-do it.

  **So the branch carries withdrawn machinery through `D8a`, deliberately.** The
  raw-body executable-set equality and the test-only retention predicate are
  **removed in `D8b`**, where their replacement is decided. Nothing merges
  meanwhile: `9f21ff0e` is non-candidate evidence and the candidate is held, so
  the only cost is that the branch is briefly inconsistent with its own frame.
  **Do not remove them in `D8a`** — the selector correction is what `D8a` is,
  and bundling the removal is the mistake that spent `D5a` twice.

  **`D8a` — the owner-qualified composed selector.**

  ```rust
  pub(in crate::cranelift_backend) fn composed_worker_view(
      &self,
      emission_owner: /* the existing owner fact, not a new identity */,
      producer_construct_origin: StaticOriginId,
      continuation_origin: StaticOriginId,
      producer_alternative: u32,
      recursive_position: u32,
  ) -> Result<ComposedWorkerView, CraneliftBackendError>
  ```

  **This does not require changing every four-field lookup** (Architect, same
  ruling). It is the composed view that projects an owner-specific answer.

  **Preserved from `f3427dae` — do not rebuild:** the
  `producer_construct_origin` field itself, the three-field collision evidence,
  the independent field-drop controls, the construct-origin transplant that
  selects the other layer's actual worker, and the body-child and
  ordered-capture provenance checks. All sound, and all still required to be
  **independently live**.

  **What `f3427dae`'s controls do NOT establish:** they prove uniqueness only in
  **the two present fixture populations.** The owner discriminator needs its
  own — the same four source fields under **two distinct emission owners**
  resolve **separately**, and **transplanting the owner refuses**.

  **The fork, and the implementer measures it rather than picking it:** if the
  planner can prove that two-owner population **structurally impossible**, then
  **encode that invariant and mutation-prove it** instead of building a
  discriminator. **Either answer discharges `D8a`.** What does not discharge it
  is the present evidence — the code **explicitly says the population is
  possible**, so a control that never instantiates a second owner cannot settle
  the question either way.

  ###### `D8a` STATUS — DISCHARGED at exact `e02ef413`, on the STRUCTURAL branch

  `742 passed / 2 failed / 1 ignored` (`+1` row), both failures the `d86be55d`
  baseline pair; production and test profiles clean. **The fork resolved to
  "structurally impossible", and it was measured before it was chosen.**

  **Reason one — the walks are disjoint.** `continuation_result_origins` does
  not descend into `Closure`/`LexicalClosure`; every descent root is
  `worker.body_origin`. Origins form a tree, so two descent roots are
  nested-or-disjoint and a producer `Construct` is reached by exactly one
  discovery. Its emission owner is fixed by where it sits.

  **Reason two, and this is the one that makes it structural rather than an
  artifact of one walk's shape:** a test-only hook pushed every descent a second
  time with `enclosing_specialization: None` — **removing reason one exactly and
  nothing else** — and still yielded no second owner. Planning refuses first, on
  both plans, with *"a continuation coordinate is not present in the lexical
  environment in force at the emission seat"*. That is the `D5a` availability
  law standing behind the traversal. The disarmed run is the positive control.

  **Side effect, and it is an improvement:** owner-qualifying **moves where a
  cross-layer transplant is caught**. Under four fields a wrong body and a
  transplanted construct origin both reached selector agreement — one defect
  from two sides. Now a transplant pairs one layer's owner with another's
  construct, a pair no unit carries, so the **selector** refuses it one step
  earlier, before any worker is compared.

  > **WATCH, and it binds every later checkpoint: the owner buys NO
  > discrimination on any current population.** It is correctness insurance and
  > an earlier transplant catch — **not a key that separates anything today.**
  > **If a later checkpoint assumes the owner is doing selection work, that
  > assumption is false as measured.** In particular, `D8g` must not attempt a
  > positive that demonstrates owner-based separation; there is no population to
  > demonstrate it on, and manufacturing one is the fabrication this node bans.
  >
  > ⛔ **"AS MEASURED" MEANS AT `D8a`, AND THE POPULATION HAS CHANGED SINCE.**
  > `D8h`-`D8p` added ten checkpoints of new planner facts. **`D8g` re-measures
  > this claim before obeying it** — see *"`D8g` — RE-MEASURE the
  > owner-separation prohibition before obeying it."* The ban on **fabricating**
  > a population is not in question; only the factual claim that **none exists**
  > is.

  **`D8b` — the composed-call target, planner side.** Mint the target as a
  planner fact under the `D8a` selector, carrying the **full
  `ComposedWorkerView` provenance**.

  **The `D7a2` requirement object's shape is retained as scaffolding** and was
  accepted as such: unconstructible outside the planner, one per exact selector,
  carrying the **whole view rather than a bare body origin**, so a consumer
  compares provenance rather than an origin that two layers could both
  plausibly name. Demand is the existence of a specialization at a selector,
  which is an interned planner fact.

  **Do not rebuild the `D7a`/`D7a2` circularity.** It bit this node once
  already: `D7a` refused the very target `D7b` was required to make callable, so
  `D7b` could never lawfully start. **`D8b`'s target must not gate on an
  executability check that `D8c` is what satisfies.** Split the question exactly
  as `9f21ff0e` already did — unreconciled resolution separate from the
  executability question — and state, per check, which side it lives on.

  **Its executability is established by `D8c`, not asserted by `D8b`.** No
  standalone `Function`, no declared-and-defined raw-body population, no
  `EmittableCallEdge` before `D8e` emits a call, no forced declaration in
  lowering, no source re-walk.

  **Preserved from `9f21ff0e` — do not rebuild:** the split of
  `composed_worker_view` into unreconciled resolution plus the executability
  question, and the **selector-agreement** law.

  **Do not restore `required_body_origin()`'s self-comparison.** The implementer
  deleted it rather than repairing it because it was **defined as**
  `worker.body_origin` and so compared a value with itself and could not fail.
  Selector agreement is the real law and catches **both** minting defects,
  because they are one defect seen from two sides: a demand attributed to a
  selector that does not resolve to it.

  **Honest residual carried forward, and this is its second carry:** the
  reconciliation gate's **third** check (declaration/definition agreement) was
  **unexercised** at `9f21ff0e` — deleting it left the row green — because every
  required body on these plans has an emittable descriptor by construction and
  the only reaching perturbation trips plane bounds first, proving the wrong
  guard. **If `D8b` retains any form of that check it must be exercised or
  deleted. It does not get a third carry.**

  **Also delete in `D8b`: the owner-collision refusal in `composed_worker_view`
  (Steward, on the `D8a` report).** The implementer measured it **unreachable
  and unexercised — deleting it reds nothing** — and labelled it defence in
  depth against a future change to the walk. `D8a` resolved the fork to the
  structural branch, and the ruling authorized encoding the invariant
  **instead** of a discriminator, not alongside a dead guard. The invariant is
  the witnessed mechanism: its duplication-hook control is mutation-proved (hook
  made a no-op reds the `D8a` row). **Keeping an unexercised second guard puts
  untested code in the TCB to defend an invariant that already has a live
  control** — `docs/PRINCIPLES.md`, small auditable TCB.

  **This is NOT the `D6b` unwitnessed-residual case, and the difference is why
  one is deleted and the other was forbidden.** `D6b`'s raw table was
  unreachable **because the mechanism was incomplete** — the source shape that
  would exercise it was refused earlier by the very gap under repair, so
  accepting it would have turned an unreachable mechanism into delivered
  semantics.

  > ⚠ **Read that in the past tense, and do not extend it to the table swap.**
  > At `400bddde` the raw route **is** reached and emits. A **third** category
  > exists that this contrast does not name: `D6b`'s Raw→`worker_calls` swap is
  > inert because **both tables lawfully hold the same target** at an
  > unretargeted body. That is neither an incomplete mechanism nor an impossible
  > population — it is **the absence of a distinction**, and it is the only one
  > of the three that requires no repair and no prohibition.
  >
  > **A fourth case sits under the same words and belongs to the IMPOSSIBLE-
  > POPULATION category, not the incomplete-mechanism one:** `raw_worker_calls`
  > answering for the **retargeted** body. That body is in the emittable
  > population and absent from the executable one by design, and the only
  > mechanism that would change it is **banned on measurement** (`evt_3dcafs581921e`
  > Finding 2). ⇒ **Do not read "the mechanism was incomplete" as an invitation
  > to complete it.** See `D6b` LIMITS.

  The owner-collision refusal is unreachable **because the planner
  proves the population impossible**, which is `D8a`'s finding rather than a
  symptom of one. Do not cite `D6b`'s prohibition against this deletion, and do
  not cite this deletion as precedent for accepting an unwitnessed mechanism.

  **The owner's selector role is separately live and stays:** supplying an owner
  no unit carries reaches the zero-answer refusal, and dropping the owner from
  the selector reds two `D7a` rows. **Delete the collision guard, not the
  field.**

  ###### `D8b` STATUS — DISCHARGED at exact `e4b4c26c`

  `ComposedCallTarget` minted; the withdrawn raw-body retention machinery
  removed. **One well-founded target form, no fork returned.**

  **The one candidate fork dissolved, and it stays dissolved under the `D8c`
  retirement:** whether the target names the raw body or the route-resolved
  callee. It is not a fork because **the view already carries route
  eligibility**, so the target is a **representation, not a route decision**.
  That reasoning originally cited *"`D8c` owns consumption"*; consumption is now
  `D8e`, and the conclusion is unchanged — **do not re-open it** on the ground
  that the checkpoint it named was retired.

  **`D8c` — RETIRED, folded into `D8e`.** See the retirement block above. Its
  law is stated as `D8e`'s acceptance, where the mechanisms that make it
  meaningful exist.

  **`D8d` — one environment authority, not two.** When the composed and
  source-machine case environment installs the constructor-argument segment, the
  selected recursive position installs that exact compiler-only
  `StaticWorkerBinding` at its **source-order binder position**, routed to the
  `D8b` composed-call target rather than to a standalone `RawWorker` `Function`.
  Nonrecursive arguments stay `Value`; the IH prefix and outer frame are
  unchanged. No parallel side map, no carrier facet. Value-position use of the
  closure capsule **continues to fail closed**, exactly as in the functionized
  unit.

  ###### `D8d` STATUS — DISCHARGED at exact `c2e8314f`

  `742 passed / 2 failed / 1 ignored` (`+1` row), both failures the `d86be55d`
  baseline pair; both profiles clean. One compiler-only `StaticWorkerBinding`
  at the selected recursive source-order position, derived from the exact `D8b`
  target under the `D8a` five-field selector. **Not a specialized
  `Value(Closure)`** — the capsule has no value representation, so
  value-position use stays fail-closed at `value_at`, which is the property
  `D8e`'s consumer will be the sole lawful way around. Identity and shape from
  the target, capture **operands** from the lowered closure at that position —
  the `D6a` split, with the constructor re-checking the two. Route is
  `RawWorker` by `D6a`'s law, **not selected**. The queued owner-collision guard
  is deleted and the `emission_owner` selector role re-measured live afterwards.

  > **The binding is correct and MEASURABLY NEVER INSTALLED** — the two
  > preconditions do not coincide anywhere in the suite. The implementer pinned
  > this as a **sentinel** rather than leaving it in a handoff, and used **two
  > counters** because *"unreadable by design"* and *"never built"* are
  > indistinguishable from outside. That sentinel is `D8e`'s inheritance.

  ###### STANDING — on this node, "correct and unreachable" is the DEFAULT, and the WITNESS is the deliverable

  **The pattern is the thing to plan around rather than rediscover:** a
  checkpoint that builds a mechanism ahead of its consumer lands it **correct
  and unreached**. `D8d`'s binding is the clean instance.

  > **TWO of the three examples this paragraph once listed have been
  > RECLASSIFIED, and citing them as unreached mechanisms now teaches the exact
  > conflation this frame bans elsewhere.**
  >
  > - **`D6b`'s raw table is a LIMIT, not an unreached mechanism.** The raw
  >   route **is** reached and emits. Attribution is unprovable because the two
  >   tables are equal wherever a raw call is emitted — an **absence of
  >   distinction**. See `D6b` LIMITS above.
  > - **`D7a2`'s retention never landed at all.** It was built, measured to red
  >   **25 rows** at the permanent unit-result closure boundary, and
  >   **withdrawn and banned** (Architect `evt_3dcafs581921e` Finding 2). A
  >   banned mechanism is not an unreached one.
  >
  > ⇒ The standing lesson below is unchanged and still right. Only these two
  > citations were wrong, and a reader who took them at face value would price
  > a limit as owed work.

  On this node:

  - **The mechanism is the cheap half. The witness that reaches it is the
    deliverable**, and a checkpoint that lands a mechanism with no witness has
    delivered its easier half.
  - **A green suite is not evidence about an unreached mechanism.** ⚠ **The
    example this line used to give — `D6b`'s wrong-table mutation — is
    RETRACTED.** That mutation stays green because both tables lawfully hold the
    same target at an unretargeted body, **not** because anything is unreached
    (`400bddde`; Architect `evt_4125xc5qw8g0x`). The rule stands; **it was
    illustrated with a case that turned out to be the other failure mode**, and
    that mis-attribution is itself the thing to watch for.
  - **Two counters, not one**, whenever *"unreadable by design"* and *"never
    built"* would look identical from outside.

  **If `D8e` cannot build its witness through the ordinary production
  planner/lowering path, that is a WHOLE-NODE finding, not a checkpoint
  finding** — it would mean the composed path cannot exercise this substrate at
  all. Hard-stop and route it to me, not around it. **Hand-constructing a plan
  to make the witness exist is the fabrication this node bans**, and it would
  convert an unreachable mechanism into delivered semantics, which is the exact
  thing the `D6b` ruling forbade.

  **`D8e` — source-machine callee consumer, AND the consumption law closed over
  the composed path.** An exact `Var` callee resolving to that binding is
  consumed **before** the value-only `Var` path. Arguments are still evaluated
  under the existing source-machine control and phase, then handed to the
  **same** route-selected static-worker emitter direct descent uses. No
  duplicated target or operand assembly; no shortcut through `lower_expr` that
  bypasses source control.

  **The folded `D8c` law is `D8e`'s acceptance, not a separate checkpoint.** The
  selected recursive argument's result is consumed **in the exact source-machine
  continuation**, the closure-valued result **never crosses a unit boundary**,
  and raw argument and capture semantics are preserved exactly. `9f21ff0e`'s
  trace names the seat it must avoid:

  ```text
  UNIT-BODY entry function=PredeclaredFunctionId(2) origin=StaticOriginId(36)
    UNIT-RESULT transfer origin=StaticOriginId(36) value=Constructor
    BOUNDARY-REFUSAL first closure child variant=Closure
  ```

  **Positive:** that refusal is **not reached**, and the target is consumed with
  the selected argument's exact raw operands.

  **Why this is one checkpoint and not two** (Steward, the ruling leaves the
  fold to me): the law is what `D8e`'s own positive has to demonstrate anyway —
  a callee consumer cannot be shown to work without showing the closure result
  did not cross the boundary. Split off, it would have been a checkpoint that
  asserts what its predecessor already proved. **If it turns out to carry real
  independent work, that is a genuine hard stop and a re-cut** — say so rather
  than absorbing it silently.

  ###### `D8e` PART ONE DONE at exact `70171a99` — consumer complete, WITNESS NOT BUILT

  **Not discharged, and the leader said so plainly rather than implying
  otherwise.** `742/2/1` unchanged, both profiles clean. **The implementer ran
  out of runway, not into a wall** — that distinction is load-bearing for
  whoever picks it up, and it is why this is not yet the whole-node finding the
  section above describes.

  **The consumer, which is complete and correct:** in the source machine's
  `Call` arm, an exact `Var` callee resolving to a `D8d` binding takes the
  static-worker route, placed **ahead of the callee's own evaluation**. That
  placement is the mechanism, not a convenience — a `Var` callee evaluated
  first goes through the machine's value arm, which calls `value_at` and fails
  closed on a static worker by design. **So the binding is consumed there or
  refused everywhere; there is no third outcome.** A `Var` resolving to `Value`
  falls through untouched. Arguments evaluate under the machine's own control
  and phase through the existing `CallArgument` continuation; only the
  completion differs. `call_static_worker` is split at the argument phase so
  direct descent and this consumer share `call_static_worker_with_inputs` —
  **no duplicate operand assembly**, arity moved into the shared half.
  `SourceCallee` is a **sum, not a widened operand slot**: widening it to hold a
  static worker would undo the fail-closed property `D8d` installed it for.

  **The sentinel now carries a THIRD counter** and records consumption at zero
  alongside installation, so the gap stays pinned rather than decaying into an
  assumption.

  > **THE REMAINING WORK IS THE WITNESS, and its required conjunction is now
  > known.** The composed deferred-constructor path fires only under
  > `requires_heterogeneous_deforestation` **plus** an immediate-binder
  > eliminator, and that must coincide with **functionized-unit definition**
  > **and** an **interned specialization at the exact selector**. All four at
  > once.
  >
  > **This is a fixture-design problem, and it is the deliverable** — see the
  > STANDING section above. **Build it through the ordinary production
  > planner/lowering path.** Hand-constructing a plan, or fabricating a target
  > or identity to make the conjunction hold, is banned.
  >
  > **If the four cannot be made to coincide lawfully, THAT is the whole-node
  > finding** — the composed path cannot exercise this substrate — and it
  > hard-stops to me rather than being worked around. No production mechanism
  > change is authorized by the witness continuation.

  ###### `D8e` WITNESS BUILT at exact `89e36ec1` — POSITIVE ROUTE PROVED, and a WHOLE-NODE HARD STOP

  **`745 passed / 2 failed / 1 ignored`**, up exactly the three new rows from
  `70171a99`; no new reds, both failures the unchanged `d86be55d` baseline pair;
  production and test profiles checked separately; **test-only, no production
  file touched.**

  **The four facts DO coincide, lawfully, through the ordinary production
  path.** The witness proves one `D8d` binding, one `D8e` consumption before the
  value-only path, and an emitted `RawWorker` call with the exact raw run — one
  source argument, zero captures, no generated-context suffix. **The emitted
  call is read back from `WorkerCallEmitted`** — the emitter's own log, written
  **after the instruction exists**, not the binding that requested it. Because
  the call has a non-empty argument run, that same event is the evidence the
  arguments went through `CallArgument`; a zero-arg call would bypass it.

  **Two discriminating controls.** Moving the callee one index onto the
  neighbouring induction hypothesis — a **real, live, adjacent binding of the
  same call arity**, not a fabricated index — leaves installation at 1 and
  consumption at 0, so consumption is attributable to **the exact binding**
  rather than to "some callable in scope". Swapping the bridge to an ordinary
  `Match`, so `lower_expr` lowers the same call, installs the same binding and
  **refuses in value position** — `D8d`'s fail-closed guard surviving `D8e`.
  Mutation-proved after the real work was committed: disarming the consumer reds
  only the positive row at `(1, 0)`; disarming the installation reds all three at
  `(0, 0)`. **The two mutations red different clauses**, which is what makes the
  rows separable rather than one aggregate.

  > ###### CORRECTION — my stated whole-node condition did NOT fire. A DIFFERENT one did.
  >
  > The block above says the whole-node finding is *"the four cannot be made to
  > coincide lawfully."* **They did coincide.** The blocker is **not the
  > conjunction — it is what the conjunction NECESSARILY CREATES.**
  >
  > **Interning the specialization that supplies the `D8a` target necessarily
  > projects a causal call onto the same emitting unit.**
  > `continuation_result_edges_owned_by` is keyed on the emission owner and
  > admits **every** projected call, so the edge is **not optional — it is the
  > same act as satisfying fact 4.**
  >
  > That edge has exactly two discharges and the composed deferred-constructor
  > path can perform **neither**:
  >
  > | discharge | why the composed path cannot |
  > |---|---|
  > | a claim | `claim_and_call_continuation` has **one** call site in the crate, on the ordinary producer branch of the same `Construct` arm the composed path returns **before**. The early return to the selected field is what makes it unreachable |
  > | a unit result that **is** the planned producer constructor | the composed path exists precisely to **eliminate that constructor in place** |
  >
  > So the compile refuses at the **`D5a` detached-result seat**, and the row
  > asserts that exact refusal. **The outer raw-body unit-result closure refusal
  > — the one my law named — is independently NOT reached.**
  >
  > **My condition was necessary but not exhaustive.** I anticipated failing to
  > reach the conjunction and wrote only that trigger; the real failure is
  > downstream of a conjunction that succeeded. **A reader matching against the
  > written condition alone would conclude no whole-node finding occurred.**

  **THE FABRICATION THAT WOULD HAVE GONE GREEN, found and refused.** Giving the
  unit a **different occurrence of the same constructor** as its result reaches
  the second discharge — **the identity check is per-symbol** — while emitting a
  specialization call for the **wrong occurrence**. Recorded so nobody
  rediscovers it as a fix. A witness bought that way is worse than the stop.

  **Three shape constraints, each measured and recorded at the fixture:** the
  declaration body may not **be** the `ComputationalMatch` (the source root is
  the planned seed node, which for that shape is the producer `Construct`, so
  the continuation value-environment walk starts **below its own
  continuation**); the wrapper may not be a `Match` (that is the
  `MatchScrutineeRecursor` residual, which selects `RecursiveDescent`, and that
  lane **defines no units**, so fact 3 fails **silently**); and the selected
  field's arms must be **statically selectable**.

  **`D8d`'s sentinel is RE-SCOPED, not deleted.** Its wider claim — the
  preconditions *"do not coincide anywhere in this suite"* — is now false, and
  the retirement is recorded in the row with a note **not to restore the
  wording**. What survives is the narrower fact about the two populations `D8d`
  landed with: **neither crosses over**. That is the control keeping `D8e`'s
  witness a **construction rather than an inheritance**.

  **The refusal assertion is a labelled sentinel: it reds the moment the
  composed path acquires a lawful discharge**, and when it does, the three
  counter clauses above it are already the positive route, green.

  ###### STANDING, third occurrence — SATISFYING a required fact can CREATE an undischargeable obligation

  `D7a` refused the very target `D7b` had to make callable. `D7a2`'s retention
  reopened the exact boundary the generated context exists to avoid. Now fact 4
  **projects an edge the composed path cannot discharge**. **Each surfaced only
  when something downstream tried to use the thing**, never when the fact itself
  was established.

  **So a checkpoint that establishes a required fact owes a statement of what
  that fact OBLIGES, not only what it provides.** On this node, ask it at the
  point the fact is minted rather than at the point it is consumed.

  **`D8f` — checked-marker occupancy.** ⛔ **SUPERSEDED IN PART — read the
  `D8f` RECUT section before acting on this paragraph.** `D8p` measured the
  occupancy ordering on a live witness, so the first three sentences below are
  **established, not owed**, and `D8f`'s remaining scope is the four refusals
  each on its own live path. Its release is gated on the Architect classifying
  the double-discharge closure `D8p` exposed.

  The witness places an ordinary
  selected-argument call **before** the checked IH call inside one checked
  wrapper. So *"a marker is pending"* **cannot** mean *"the next static-worker
  call consumes it."* The selected-argument call **leaves the marker pending**;
  only the exact planner-issued checked call occurrence may consume it. Use the
  existing checked **occurrence authority** or a faithful projection of it,
  never route, arity, binder-index coincidence, or first-call order. Omission,
  duplicate, transplant and wrong occurrence must all refuse.

  **`D8g` — non-vacuous closeout, over TWO populations. RECUT 2026-08-06.**

  **The former text assigned both required facts to the `px8tr` A/B witness,
  and it is WITHDRAWN.** It demanded a same-program conjunction — a same-body
  `GeneratedContext` IH **and** a selected-argument emission through the
  composed-call target, on one witness — that **no population in reach
  satisfies**, and that **was never derived from a production contract.** The
  implementer measured that edge and hard-stopped rather than fabricating a
  route (`evt_9yrrhnjp6dfd`); the Architect accepted the stop and ruled this
  recut (`evt_6afmy0398n847`). **The grounded clause was always `D8l`'s** —
  composed emission belongs to a **composed** witness, never to the A/B one.
  **Do not construct a combined witness.**

  **The two populations join at one exact PRODUCTION SEAM, not at their
  outputs. That is what makes the recut non-vacuous.**

  | # | population | what it owns | required relation |
  |---|---|---|---|
  | 1 | **ordinary / functionized — `px8tr`** | the table-choice and suffix contract; it already holds the mixed route population — a same-body `GeneratedContext` IH and a `RawWorker` route through ordinary production lowering | the **composed post-emission evidence set** of the `EXACT-KEY LIMIT` below, and **only** that set. Wrong-table mutation **red**; context-suffix mutation **stays red** — each at its **earliest structural guard**, per `evt_5vwdtrznf3km4` |
  | 2 | **composed — the lawful `D8e`/`D8m`/`D8n`/`D8f` family** | the composed ingress | the exact selected recursive argument reaches its `D8b` composed-call target, then **the same** production `call_static_worker_with_inputs` emitter. Compare exact target, application occurrence, **actual defining body kind**, decoded raw callee, operand run, and composed discharge against independent planner/unit-definition authority |

  **Both relations must NAME `call_static_worker_with_inputs` as the
  convergence seam, and prove that NEITHER INGRESS DUPLICATES target or operand
  assembly.** Never two output bags. **Population 2 is a keyed exact relation.
  Population 1's ACTUAL side is exactly keyed and rejects duplicates, but its
  INDEPENDENT side is bounded by the limit below — do not read "keyed exact
  relation" as reaching population 1's expectation.**

  ##### `D8g` EXACT-KEY LIMIT — the per-call expectation has NO AUTHORITY

  **Ruled `evt_399j195rn8fpa`, 2026-08-06.**

  ⛔ **WITHDRAWN: the clause requiring an independent expectation keyed by
  `(defining Function, call origin)`.** It was written as though satisfiable.
  **It is not, and the absence is structural — not an accessor anyone missed.**

  The required population is

  ```
  (exact defining body, source Call occurrence)
      -> (worker target, route/table, arity, captures, context suffix)
  ```

  and **no planner or unit authority names it.** Measured at
  `0665b9d5`, four surfaces were checked and each falls short in a different way:

  | surface | why it does not name the population |
  |---|---|
  | `EmittableCallEdge` | carries caller, callee and call-site origin, but its **closed** `EmittableCallKind` is `StaticBody \| Declaration` (`static_transition.rs:9458-9470`). The local `StaticWorker` applications at issue are **not members** |
  | `ContinuationCallView` | names planner-issued **causal continuation calls** and their specialization target (`:11350-11392`) — never the source `RuntimeExpr::Call` that later invokes a static-worker binding |
  | `ContinuationUnitView` / `ContinuationContextView` | independently name worker contracts and suffixes, but `continuation_context_for(enclosing, worker_body_origin)` **requires those two facts from its caller** (`:11298-11347`); it projects no body/call edge |
  | `D8oBodyKey` | lawfully joins an emitted `FuncId` to a planner body descriptor, and the source-origin walk names syntax — but neither **resolves that syntax's de Bruijn callee to the worker binding installed in that body** |

  **That last resolution happens during lowering, not planning.** The
  specialization pass decides the IH route from the retarget outcome and
  constructs the binding (`units.rs:1688-1705`); the binder run maps into the
  lowering environment (`:1764-1802`); only then does
  `call_static_worker_with_inputs` consume the binding and record
  `(Function, call_origin)` **after emission** (`core.rs:8079-8306`). **The
  planner retains no per-body application record spanning those seams.**

  ⇒ **An accessor added now could do only one of two unsound things:** package
  observed emission keys and call them independent — **the self-selection defect
  already held four times** — or reimplement the lowering environment walk beside
  production, **a second authority that can reproduce the same bug.** Neither is
  authorized by `D8g`'s evidence-only scope.

  ###### What `D8g` proves INSTEAD — and what it must never be CALLED

  Actual-key duplicate rejection; independently validated `D8o` body membership;
  independently named source occurrence; planner/unit contract and
  generated-context suffix checks; emitted-`FuncRef` evidence; and the two exact
  structural mutation refusals at their earliest guard with an empty emission set.

  ⛔ **Two things this composition must NEVER be called**, because both are the
  withdrawn clause wearing the shape of the fix:

  1. **an independently enumerated expected key set** — its keys come from the
     observations, and a key set derived from what it checks is not evidence;
  2. **an independently selected target** — the observed target still chooses
     which record it is compared against.

  **State the residual plainly in the closeout:** a defect that moves target and
  route **together** to another lawful same-shaped worker is **not** caught by
  population 1. That is the exact hole, it is admitted rather than papered over,
  and `D8j`'s finished-CLIF decode plus the `D8o` body authority bound how far it
  can travel.

  ###### Why this is a LIMIT and not a NEW NODE — the constraint was interrogated

  Option 1 was available: make static-worker application resolution a
  planner-owned unmintable population keyed by the `D8oBodyKey` domain plus exact
  source `Call` origin, and make lowering consume it. **It is refused here on one
  factual finding, not on cost.**

  **`D6c` does not consume it.** `D6c` is the **pre-emission refusal set** about
  *selecting* the raw/IH target and its selected-recursive-argument member. Every
  obligation in it — omission, duplicate, wrong source position, wrong
  closure/body, wrong capture run, wrong order, fabricated availability,
  cross-routing raw and IH targets, and the route/suffix exactness law — is
  discharged by **a perturbation and a guard at the selection seam**
  (`units.rs:1688-1705`), which is the proof shape ruled adequate at
  `evt_5vwdtrznf3km4`. **A refusal needs a guard, never an independently
  enumerated expectation population.**

  ⇒ Creating a second planning population **solely to satisfy a test** violates
  **subsume-don't-proliferate** (`docs/PRINCIPLES.md`). If a later checkpoint
  genuinely needs the fact **in production**, option 1 is the right move and
  makes it structural with one owner — **but it must be raised as its own node
  with that consumer named, never folded into `D8g` and never disguised as a test
  accessor.**

  **What would reopen this:** a named production consumer of the per-call
  application record. **Not** a reviewer wanting a stronger `D8g` row — that is
  the pressure this limit exists to absorb.

  **Why two populations suffice, stated so it is not re-litigated.** `px8tr`
  already exercises coexistence of the two route/table/suffix arms, and the
  composed witness adds a **distinct ingress to that same emitter**. Putting
  both ingress shapes in one source program would cover **no additional route
  contract** — it would manufacture the ungrounded conjunction this recut
  exists to remove.

  **Every independent side stays subject to the `D8f` rules** — see **WHAT
  DISQUALIFIES A SIDE**: no different-program mechanism read, no
  complement-derived subject, no shared collector; and any difference proof
  withholds the sole producer of its observable.

  **Not authorized:** a combined fixture, an owner-positive population, or any
  change to production mechanism, identity, ABI, route/selection authority,
  binding classification, or the ledger.

  **The declared-and-defined standalone `Function` clause is withdrawn** — see
  Finding 2. `D6c`'s refusal set follows only after this positive closes.

  **Watch, carried from the implementer's own handoff:** the reconciliation gate
  **refuses in production by design** at `9f21ff0e`. Anything that starts
  calling it before the route is settled will read that refusal as a
  **regression** rather than as the checkpoint's own finding.

  ###### `D8g` — RE-MEASURE the owner-separation prohibition before obeying it

  **`D8g`'s FIRST deliverable is a measurement, for the same reason `D8l1`'s
  was.** Not a fixture, not a positive — an answer to a question about its own
  premise.

  **The premise.** The `D8a` WATCH above forbids `D8g` from attempting a
  positive that demonstrates owner-based separation, on the stated ground that
  *"the owner buys NO discrimination on any current population"* and *"there is
  no population to demonstrate it on, and manufacturing one is the fabrication
  this node bans."*

  **Why it cannot simply be inherited.** That claim is **undated and says "as
  measured"** — it was measured at `D8a`, and **the population is exactly what
  `D8h`-`D8p` have been changing** ever since: planner pairing to opaque causal
  identity, the ambient body authority and its supplied body kind, the
  per-`Function` consumption lifecycle, the transported source frame, and now a
  keyed cross-body binding at two emission seats. **Ten checkpoints of new
  planner facts is precisely the event that would falsify a "no population
  exists" claim**, and nothing has re-run it.

  **So the prohibition is load-bearing in BOTH directions and that is why it
  must be measured, not assumed.**

  | if the re-measurement says | then |
  |---|---|
  | the owner still separates nothing | the prohibition **stands unchanged**, and `D8g` is now standing on a checked premise instead of an inherited one |
  | the owner now separates a real population | the prohibition is **forbidding a positive that is both available and worth having**, and it is retired against the measurement — never against an argument |

  ⛔ **Neither outcome licenses manufacturing a population.** The ban on
  fabrication is not what is being questioned; **only the factual claim that no
  population exists** is. If the measurement finds none, that is the answer and
  `D8g` proceeds without an owner positive.

  ⛔ **Do not size any `D8g` owner work before that answer exists.** Sizing a
  repair ahead of its own premise measurement is the error this node has
  already paid for three times — `RT-UNIT-CLOSURE-CONVERT`, the `D7` cut, and
  `D8l`.

  **Held, not released.** `D8g` sits behind `D8f`, which is itself gated on the
  Architect's classification of the double-discharge closure. This section
  exists so that `D8g` is shovel-ready the moment it becomes the frontier — not
  so that anyone starts it now.

  ###### STEWARD RULING — `D8e` CLOSES. The envelope frontier is `D8l`.

  **`D8h`-`D8k` are complete and the causal-projection repair is proved.** The
  `D8e` witness passes the former `D5a` detached-result seat: a causal call
  answered by a composed source continuation **is not detached**, because the
  producer's constructor was eliminated in place and the obligation was met by
  the raw-worker call the source machine made. `continuation_result_edges_owned_by`
  is unchanged; the filter is what moved.

  **`D8e` closes on its own law, which is proved.** Consume the selected
  recursive argument in the exact source-machine continuation with exact raw
  arguments and captures, closure-valued result never crossing a unit boundary
  — all of it demonstrated at `89e36ec1` and preserved through `D8k`: one
  binding installed, one consumption before the value path, an emitted
  `RawWorker` call carrying the exact raw run, read from the emitter's own log.

  ⛔ **It does NOT close on "a compiling composed program," and the difference
  is not a technicality.** The witness now stops later and elsewhere — in
  **specialization emission**, building the specialization's case binder run,
  because the ordinary envelope carries no nonrecursive field at the selected
  field's source position. **No `D8e` release ever put specialization emission
  in scope.** Closing `D8e` on end-to-end compilation would retroactively widen
  its law to cover a mechanism it never owned, which is the mis-ordering that
  retired `D8c`.

  ⭐ **`D8e` closing is NOT the node closing.** The standing section above —
  *"correct and unreachable" is the DEFAULT* — still binds the **candidate**:
  this node must not ship a composed mechanism no program can reach. `D8e`'s
  own layer is genuinely exercised end to end, which is what distinguishes it
  from `D6b`, where the mechanism was never reached at all. The node's closure
  obligation moves to `D8l`.

  ###### `D8l` — the ordinary-envelope frontier. FIRST deliverable is a MEASUREMENT.

  **`D8l` precedes `D8f` and `D8g`** (same label-order caveat as `D8h`-`D8k`).
  `D8g` requires *"a selected-argument emission through the composed-call
  target"* and `D8f`'s marker occupancy is emission-adjacent, so **both are
  blocked until a composed witness reaches emission.** This is not an optional
  tidy-up.

  > **DISCHARGED — and this paragraph is the clause the `D8g` recut restored.**
  > `D8l2` landed (`1f9a2020`+`abe46dda`) and `D8f` discharged at `04c40d4c`, so
  > the block above is history, not a live hold. **Its wording is nonetheless
  > operative**: it assigns composed emission to a **composed** witness. The
  > `D8g` paragraph above once assigned the same fact to the `px8tr` A/B
  > witness; the Architect ruled **this** clause the grounded execution
  > dependency (`evt_6afmy0398n847`) — *it is the checkpoint that made the
  > composed family reach emission* — and the `D8g` conjunction the defect.
  > **Do not re-derive a same-program conjunction from the sentence above.**

  ⛔ **`D8l1` is: is the refusal STRUCTURAL to the composed shape, or a fixture
  accident? Do not size the repair before that answer exists.** Flagged by the
  implementer and it is the right flag: **both** witnesses hit the identical
  refusal from `continuation_case_binder_run`, and both place a `Match` at the
  selected field's source position — **which is exactly what
  `requires_heterogeneous_deforestation` demands.** If the shape that triggers
  the composed path is the same shape the envelope cannot carry, this is a
  mechanism question for the Architect and potentially a second whole-node
  finding, not a fixture fix.

  **Sizing the repair before that measurement is the error this node has
  already paid for twice** — `RT-UNIT-CLOSURE-CONVERT` was framed on a premise
  measurement retired, and `D7` was cut before its own premise was checked.

  ###### `D8l1` ANSWERED — NOT structural. A real planner defect, fixture-exposed.

  **Measured at exact `aaef1772`, nothing committed.** Two witnesses identical
  except the order of the producer `Construct`'s two fields:

  | producer | recursive position | outcome |
  |---|---|---|
  | `Wrap(worker, field)` | 0 — nonrecursive field **after** it | refuses: ordinary envelope has no nonrecursive field at source position 1 |
  | `Wrap(field, worker)` | 1 — nonrecursive field **before** it | **compiles end to end** |

  Both place a `Match` at the selected field, so **the deforestable shape is
  not what is being rejected — the only variable is field order.**

  **The defect, exactly.** `ContinuationUnitView::ordinary_envelope`
  (`planning/static_transition.rs:1755`) emits nonrecursive roles from a
  **dense loop index**, while `continuation_case_binder_run`
  (`lowering/units.rs:1198`) looks them up by the producer's **true source
  position**. The two agree only while every nonrecursive field precedes the
  selected recursive position — **omitting a later position does not renumber
  the earlier ones, but omitting an earlier one renumbers the later ones.**

  ⭐ **The method's own doc comment states the correct rule** — *"nonrecursive
  producer-`Construct` fields in producer source order with the selected
  recursive position omitted"* — **and the loop does not implement it.** It
  emits the envelope *index*, not the source position it stands for. `px8tr`
  has the recursive position last, so the two coincide and the defect has
  **never been reachable** until now.

  ⇒ A real production defect with a live witness, reachable by **any** producer
  whose selected recursive position is not last — composed or not.

  ###### `D8l2` — the bounded repair. Architect `evt_5bs9fxyxww8gy`.

  ⛔ **Repair ONLY `ordinary_envelope`'s nonrecursive-field population.** The
  view already has sufficient closed authority: `ordinary_parameters`, the exact
  ordered worker-capture run, and the selected `recursive_position`.

  After checked subtraction of captures, let `N` be the nonrecursive
  source-field count; the source constructor has `N + 1` fields in this
  single-selected-worker projection. **Require `recursive_position < N + 1`,
  then enumerate source positions `0..N + 1` in source order EXCLUDING that
  exact selected position**, emitting one `NonrecursiveConstructorField {
  source_position }` per remaining position. Append the existing worker-capture
  roles unchanged in capture-ordinal order. **Not a reverse source walk, and it
  adds no identity** — the selected position and the count are already
  immutable planner facts. Reconcile to the exact `Parameter`-slot count.

  ⛔ **Do NOT change `continuation_case_binder_run`** — it already performs the
  correct lookup, and **its refusal is what exposed the bad planner record.**
  No renumbering in lowering, no inferring a missing role from the case body, no
  padding, no ABI slot count/order change, and no touching `D8h`-`D8k`'s
  pairing, transport, verification, filter or partition laws. **No planner/ABI
  redesign is authorized: this is a population correction inside the existing
  representation.**

  **Required to close (Architect's list, verbatim in substance):**

  1. Planner-level exact populations — selected first of two ->
     `[source_position: 1]`; selected last -> `[source_position: 0]`; selected
     mid of at least three -> `[0, 2, ...]` in source order.
  2. ⛔ **SUPERSEDED — this clause asked for a refusal at the WRONG PLANE.**
     Replaced by item 2' below. Architect `evt_2ffwkpgnmr8xd`, its own error,
     transcribed by me without catching it.
  3. The capture tail stays **byte-for-byte ordered** after the corrected
     nonrecursive prefix, header and slot counts unchanged.
  4. Both field-order witnesses reach specialization emission. ⛔ **Their
     ordinary fields must carry DISTINGUISHABLE values and the observable
     answer must depend on the selected source-position mapping — mere
     compilation with an unused field is not evidence of correct binding.**
  5. The formerly refusing recursive-position-0 witness **compiles and
     executes**, and the whole-pass causal ledger closes with a **non-empty
     composed half**. ⭐ Add this reach assertion **beside** `D8k`'s law-level
     ledger row, **never in place of it**.
  6. The old dense-prefix implementation, or an equivalent mutation, makes the
     recursive-position-0 row **red** while the exact correction makes **both**
     orientations green. All `D8h`-`D8k` local and global controls preserved.

  **Two additions of mine, both from the implementer's own measured warnings:**

  ⛔ **7. Enumerate every consumer of `ordinary_envelope` and state, per
  consumer, whether the renumbering changes what it sees.** The implementer
  measured that **no current fixture would red on this fix** — a change that
  silently alters a numbering every consumer reads, with no existing test
  catching it, is exactly the shape that surfaces three checkpoints later. The
  enumeration is reviewable output, not a step.

  ⛔ **8. Attribute the compiling witness's THREE consumptions against ONE
  discharge.** The implementer flagged this rather than rounding it off, and
  was right to: the reasoning that the other two are ordinary direct-facet
  bindings is sound (two claims of one identity would refuse as a double
  discharge) but it is **reasoned, not measured**. Confirm the two carry
  `DirectSpecializationCall`. **This adds no mechanism** — it converts a
  reasoned claim into a measured one.

  ###### `D8l2` EVIDENCE COMPLETION — item 2 was at the wrong plane. `evt_2ffwkpgnmr8xd`.

  **The production repair at `1f9a2020` is the RIGHT repair** and is preserved
  as accepted partial progress, **not a candidate**: the planner now emits true
  source positions, the formerly refusing selected-first witness compiles, and
  the non-empty composed half reaches whole-pass closure. Items 1, 5, 6, 7 and
  8 are discharged. **The three reported gaps do not share one disposition.**

  ###### Item 2' — the producer/consumer PLANE split

  ⛔ **Item 2 asked for a wrong-order refusal at the consumer, and that was
  never satisfiable.** `ordinary_envelope` produces a **slot-ordered
  authority**; `continuation_case_binder_run` **consumes** it by finding the
  slot carrying a given source position, so it **follows any internally
  consistent permutation it is handed**. The landed
  `checked_computational_ih_binder_run_admits_a_permuted_envelope` row is
  **correct and stays unchanged**. Making that consumer independently
  reconstruct source order would add exactly the **second slot-order
  authority** this release forbids.

  ⭐ **That does NOT make a wrong-order planner population lawful, and the
  record must not say it does.** The distinction is the whole of item 2':

  | plane | rule |
  |---|---|
  | **producer** (`ordinary_envelope`) | **required** to emit the exact source-ordered vector |
  | **consumer** (`continuation_case_binder_run`) | follows a self-consistent permutation faithfully; this is by design |

  **Item 2' replaces item 2:**

  - out-of-range selection **refuses in planning**;
  - omission, duplication and dense-prefix substitution reach their **existing
    exact refusal**;
  - **wrong order fails the PLANNER-LEVEL exact-population equality** — item 1
    already carries the net, since selected-middle must be exactly `[0, 2]` and
    reversing the producer order makes that assertion red. **No downstream
    refusal is required or authorized.**

  ⛔ **Fidelity correction to the `D8l2` record:** it must **not** say broadly
  that wrong order is *"admitted by design"* — that is true of the **consumer
  presented with** a permutation and false of the **planner producer**. Remove
  or reframe the `WrongOrder` no-op entry in the defect matrix. **Add no
  production validator and no second representation.**

  ###### Item 3' — the capture tail. NOT waived; bounded test work.

  **The repair changes the prefix immediately before the capture tail, so a
  capture-free witness cannot prove the join.** Every worker currently in reach
  is capture-free, which makes the present evidence **construction-only, not
  measured**.

  Extend the existing `contspec_multiple_worker_captures_fixture` ordinary-path
  shape — or add its narrow `D8l` sibling — so it **also has a non-empty
  nonrecursive prefix**. For **both** orientations assert the complete
  `WorkerCapture` tail — **ordinal, owner, closure origin, source and
  lifetime** — equals the immutable worker-capture provenance **in order**,
  plus the full envelope length, `header.parameters`, and the actual
  Parameter-slot count. ⛔ **At least two captures; a length-only or empty-tail
  assertion is insufficient.**

  **Planner/test evidence only — the production capture loop stays
  byte-identical.**

  ###### Item 4' — the observable answer. NOT waived; bounded execution fixture.

  ⛔ **The current fields carry different integers but the program never makes
  its result depend on them. That is the exact vacuity the clause exists to
  prevent** — populations and reach were proved, an answer was not.

  The test crate already has a linked execution harness:
  `artifact::compile_expr_for_lowering_tests(...).run(None)`. Route the matched
  ordinary payload through the **real composed call** and return it as a ground
  `Int`. Selected-first and selected-last must carry **distinguishable
  payloads** and assert their **exact returned observations**.

  ⛔ **The value must be CONSUMED — not merely present in the envelope, and not
  used only to select a case.** Presence and case-selection are the two ways
  this goes vacuous while looking green.

  The exact repair must green **both** orientations; **`DensePrefix` must still
  red the selected-first orientation.** Preserve the existing end-to-end
  compile, non-empty composed-discharge and `D8h`-`D8k` ledger assertions
  **beside** this differential.

  ###### Scope of the completion, stated so it is not read as a re-opening

  **One bounded `D8l2` evidence completion after a fidelity correction** — not
  `D8f`, not a new node, not a planner/ABI redesign. ⛔ **Apart from correcting
  stale comments and records and adding the two fixtures and controls above, no
  production edit beyond `1f9a2020` is authorized.**

  ###### `D8m` — PRESERVE the source frame through the bridge. `evt_7vghm2dzh0k4w`.

  **`D8m` executes immediately BEFORE `D8f`.** Exact `e80fd42f` is accepted as
  `D8f` **hard-stop evidence only** — not a candidate, and `D8f` is **not
  discharged**.

  **Both ways out were refused, and both refusals are load-bearing.**

  ⛔ **The gate may NOT be accepted without a non-vacuous witness.** The
  always-admit mutation leaving the full suite green **proves zero trust
  delta**: the production population does not distinguish the new law from its
  predecessor. That cannot establish omission, duplicate, transplant or
  wrong-occurrence refusal. ⭐ **This is the fourth occurrence on this node of
  the standing pattern above — "correct and unreachable" is the DEFAULT and the
  WITNESS is the deliverable.** The implementer's distinction is worth keeping:
  the gate is **unwitnessABLE at this checkpoint**, which is a different claim
  from unwitnessed.

  ⛔ **A lowering-synthesized bridge must NOT receive a newly minted
  `CheckedSubcontinuationFrame` identity.** The oriented plan currently proves a
  **bijection** between checked frame entries and source Runtime frame markers.
  An unmarked synthetic frame **weakens that bijection**; borrowing a frame by
  fingerprint, body shape, origin coincidence or uniqueness **restores the
  already-banned inference**.

  ⇒ **The bridge is not a new semantic frame.** It is an optimization of the
  exact source `ComputationalMatch` that `immediate_binder_eliminator` consumes.
  **So preserve that source frame through the optimization** — the new work only
  stops the fusion from dropping an identity the source already issued.

  **The mechanism, in five parts:**

  1. Extend the immediate-binder recognition result into a **closed bridge
     descriptor** recognizing exactly: the existing direct
     `ComputationalMatch`; the existing direct ordinary `Match`; or exactly
     `CheckedSubcontinuationFrame { frame_id, body: ComputationalMatch }`.
     ⛔ **It must not generically peel wrappers.**
  2. For the checked form retain **two** facts in the same descriptor: the
     marker's exact `frame_id`, and the wrapped match's exact source
     occurrence, which is **child 0 of the marker occurrence**. ⛔ **The
     wrapper origin is NOT the match origin.**
  3. **Only after** the existing immediate-binder and heterogeneous-
     deforestation gates have selected the bridge, run the **existing**
     checked-frame entry and consumption path on that exact marker and exact
     `cases`/`default`. Preserve the current plan lookup, runtime-frame
     fingerprint check and affine `(invocation, frame)` consumption. ⛔ **No
     second validator.**
  4. Construct the bridge `ComputationalEliminatorFrame` with the **same tuple
     a normally lowered checked computational match receives** —
     `checked_frame_id`, `checked_invocation_id`, `checked_invocation_source`,
     `checked_invocation_depth` — plus the wrapped match's exact source origin.
     An **unwrapped** bridge retains the current all-`None` tuple. ⛔ **Presence
     of a checked frame does not change `answer_route`.**
  5. ⛔ **No plan-frame kind, marker kind, runtime/ABI field, synthetic frame
     id, body/fingerprint lookup, or fallback.** The source marker remains the
     **sole identity authority**.

  **Required proof before `D8f` resumes:**

  - a checked immediate-binder bridge **consumes its exact source marker once**
    and reaches `computational_ih_slots_for_case` with the exact frame named by
    its slot templates;
  - **suppressing the transported tuple reproduces the detached-frame
    refusal**;
  - marker omission, stale/wrong frame, fingerprint change, transplant and
    duplicate consumption **still refuse through the existing plan/marker
    laws**;
  - ⭐ **substituting the wrapper origin for the wrapped match origin is caught
    on a same-shaped, DISTINCT-OCCURRENCE control** — a same-shaped control is
    what makes this non-vacuous, since the two origins coincide in shape;
  - an **unchecked** computational bridge remains unchecked, and the ordinary
    bridge population is **unchanged**;
  - **plan frame count and Runtime marker count remain identical** — the
    bijection above is what this protects.

  **Then `D8f` resumes on the preserved gate**, and its specialization-only
  witness must show the ordinary selected-argument call **leaves the marker
  pending** while only the exact planner-issued checked-call occurrence consumes
  it. ⛔ **Both always-admit AND never-admit must be red**, and omission,
  duplicate, transplant and wrong-occurrence must **each have its own live
  refusal**.

  ###### `D8m` sizing — in-node, with ONE named condition that would change it

  **A bounded lowering-preservation checkpoint in this node, immediately before
  `D8f` — not a new substrate node.** The Architect attached one condition, and
  it is a real escape hatch rather than a formality: **unless the
  implementation census shows the identity must cross another producer/consumer
  family.** If `D8m`'s own census finds that, **hard-stop and return it to me**
  — that is the fact that would convert this into a node, and it is not a
  decision to absorb quietly.

  ⭐ **RESOLVED — the census RAN and did NOT trigger the hatch** (Architect
  `evt_502e30ws16ary`, reviewing `6c711bf5`). **The in-node sizing holds and
  this conditional is spent.** Do not re-run the census to re-open it.

  ###### `D8m` at `6c711bf5` was BLOCKED — two findings worth carrying forward

  **The mechanism is right** — descriptor family, exact marker-child-0 origin,
  existing enter/consume path, unchanged `answer_route`, all-`None` unwrapped
  branch. `6c711bf5` is preserved as partial progress. Two acceptance findings
  blocked it, and both are shapes this node has hit before:

  **1. A partial tuple that reaches the guard anyway.** The checked branch
  retained `checked_frame_id` but wrote `checked_invocation_id: None`,
  `checked_invocation_source: None`, `checked_invocation_depth: 0`. The normal
  path derives those from `active_recursive_invocations` — at a checked root
  `checked_invocation_id` is `Some(0)`, not `None` — and they feed
  selected-scope, delimiter, splice-edge and invocation validation.
  ⛔ **Reaching the missing-slot guard proves only that `checked_frame_id`
  survived; it does NOT prove preservation of the required tuple.** Share or
  exactly reuse the normal derivation; do not hard-code three fields and do not
  add another validator.

  **2. Controls that passed for the wrong reason.** ⛔ **The "distinct
  occurrence" control was not distinct** — `d8m_witness(..., second_frame)`
  ended with `let _ = second_frame`, so the second occurrence was never
  constructed. And the unmarked and one-shape-off controls compiled with **no
  oriented plan**, so plan/marker validation could reject them **before** the
  bridge descriptor or the borrowing hazard was reached. ⭐ **Assertions that
  merely exclude the later slot-marker text cannot distinguish that masking** —
  the control refuses, but at an earlier guard than the one under test. The
  fingerprint-change and duplicate-consumption controls were absent entirely.

  **This is a correction within `D8m`** — not a new node, and **not permission
  to alter `D8f`**, which stays held until the corrected exact object is
  reviewed.

  ###### `D8n` — per-Function consumption lifecycle. A REAL defect. `evt_2v8pzbp9ek8r2`.

  **`D8n` precedes the `D8m` witness.** Preserve exact `7d7f248b` as partial
  `D8m` progress — its shared four-field checked-computational derivation is
  accepted and the dead `second_frame` axis removal is correct.

  **The defect.** The source marker is **one** source identity, but the ordinary
  declaration `Function` and the generated specialization `Function` are
  **distinct emitted code bodies**. Each lowers its own copy of that source body
  and each must validate **one** consumption of the same semantic
  `(invocation_id, frame_id)` pair. ⛔ **A compile-wide set conflates those two
  function-local validations, because both function roots currently use
  invocation id 0.**

  ⭐ **This is NOT the dynamic-branch case, and the distinction is the whole
  argument.** Branch successors **rejoin**, so `CheckedFrameBranchScope` unions
  their consumptions and correctly rejects a later post-join revisit. **Distinct
  emitted functions do not rejoin in one lowering control-flow graph, so their
  consumption sets must not be unioned.**

  **This is the function-splitting collision the earlier pair-versus-triple
  ruling left CONDITIONAL** — that ruling rejected a dormant
  `PredeclaredFunctionId` key salt but required a fresh ruling if the same pair
  collided across function splitting. **The collision now exists, and its lawful
  repair is lifecycle scoping, not restoring the salt.**

  **Ruled mechanism.** ⛔ **Keep the live element type exactly `(u64, u64)` — do
  NOT add emission owner, `FuncId`, `PredeclaredFunctionId` or any other salt to
  the key.** Make `consumed_subcontinuation_frames` a **per-emitted-`Function`
  lowering scope**:

  1. enter a **fresh empty** pair set before lowering each source-bearing
     generated `Function`;
  2. use that same set — **including the existing branch transactions** — for
     the whole function body;
  3. require **no active checked marker to cross the function boundary**;
  4. restore the enclosing compiler state after the function build; ⛔ **do not
     union one function's consumptions into another's**;
  5. apply the boundary **uniformly** to ordinary unit bodies, continuation
     specialization bodies and generated continuation-context bodies. The
     recursive-descent root remains one function and therefore one scope.

  ⛔ **One structural function-body transaction around the EXISTING validator** —
  not an ad hoc clear at the second lowering, and not a second validator. The
  source plan/Runtime-marker bijection stays source-level and unchanged.

  **Required discriminators, before the remaining `D8m` proof resumes:**

  - the lawful witness records **two** defining function identities/emission
    owners and the **same colliding pair**, and **each function consumes it
    exactly once**;
  - **both** functions reach the exact checked-IH slot reconciliation named by
    the plan;
  - **restoring the old compile-wide lifecycle reproduces the second-function
    duplicate refusal**;
  - duplicating the marker on one executable path **inside** either function
    still produces the existing consumed-more-than-once refusal;
  - ⛔ **a template-only single-function form is green but is NOT the sole
    positive** — it removes one side of the collision and would leave the
    production defect intact;
  - branch-scope duplicate/transplant controls unchanged;
  - pair key shape, plan-frame count, Runtime-marker count, fingerprints and the
    full transported tuple all unchanged.

  ###### `D8n` sizing — in-node, and BRANCH TOPOLOGY is what decides it

  **An ordered in-node predecessor checkpoint, not a separate substrate node.**

  ⚠ **The honest case for a node is real and I am recording it rather than
  brushing past it:** this is a **production lifecycle change to a law that
  predates this node**, applying uniformly across three body kinds, and unlike
  `D8h`-`D8k` it **would stand alone** — it repairs a defect reachable by any
  program splitting a checked body across a declaration and a specialization,
  independently of anything `D8` built. By the standalone test I used for
  `D8h`-`D8k`, this one passes.

  ⛔ **What decides it against a node is that the fix must land on the HELD
  SHARED BRANCH.** A separate node could only get there by (a) cutting from
  `main`, merging, then **rebasing** `wp/RT-DECL-CLOSURE-PORT-typed-units` onto
  it — which **destroys every preserved exact checkpoint SHA on that branch**,
  and this node's whole evidence chain is preserved-SHA-addressed — or (b)
  landing on the same shared branch anyway, which recreates the
  "two nodes, one branch, one candidate" shape [[RT-UNIT-CLOSURE-CONVERT]]
  needed three paragraphs to explain was not a deadlock.

  ⇒ **Neither is acceptable, so the classification concern loses to the
  topology.** Recorded so a later reader knows the substrate argument was
  weighed and what beat it — **if this node's branch ever merges before the
  lifecycle work is done, the answer flips to a node.**

  ###### `D8o` — ambient body authority. A SECOND defect. `evt_d2yrj4fqdfjt`.

  **Found by `D8n`'s evidence probe at exact `ef41b513`.** Not an
  observation-only curiosity, and ⛔ **it must not be folded into the bounded
  `D8n` correction.**

  **The defect.** `define_continuation_bodies` opens each specialization
  `FuncId` with `open_aggregate_events(id)` (`units.rs:1361`), which sets
  **only** `defining_function_id`. **That loop never binds
  `defining_emission_owner` before lowering its source body.** Ordinary unit
  bodies bind `Predeclared(unit.function)` (`units.rs:3172`) and generated
  contexts bind `Specialization(context.enclosing)` (`units.rs:2100`).
  ⇒ **A specialization body inherits whichever emission owner the prior pass
  left behind.** The probe measured distinct specialization and ordinary
  `Function` consumptions **under the same stale predeclared owner**.

  ⛔ **This field is active production authority, not diagnostics.** It is read
  by continuation claiming/calling, **composed-worker selection and authority
  checks**, **composed-discharge ownership**, carried-context resolution, and
  synthesized aggregate occurrence/root/dynamic reconciliation —
  `core.rs:6788, 8398, 8426, 9373, 9584-9587, 9766`; `mod.rs:10051, 10486,
  10557`. **A stale value can select or validate against the wrong
  planner-owner population; `None` can also silently decline some paths.**

  ⛔ **`defining_unit` is a neighbouring ambient field, likewise unbound in the
  specialization loop** — so a one-field patch **without a read/write census is
  unsafe.**

  **Mechanism.** Emitted-body ambient authority must be a **structural
  body-lifetime binding, not inherited mutable residue.** For a specialization
  body the planner-issued owner is
  `ContinuationEmissionOwner::Specialization(unit.id)`; ordinary and
  generated-context bindings keep their existing exact domains. **Entry binds
  the body facts before ANY source lowering; exit restores/clears them so no
  later `Function` can inherit them.**

  ⛔ **First inventory every production reader of BOTH
  `defining_emission_owner` and `defining_unit`**, then prove each of the three
  source-bearing body kinds sees its exact planner-issued authority and that
  **cross-body inheritance is red**. ⛔ **Do not infer the owner from `FuncId`,
  raw body origin, or whichever field was last set.**

  ###### `D8o` — the RETROACTIVE question I am raising, not assuming away

  ⚠ **`D8a`'s selector is FIVE-FIELD and its first field is the emission
  owner.** `D8i`'s wrong-owner refusal and `D8j`'s verification clause 2 —
  *"the claiming function equals `identity.emission_owner()`"* — are both
  owner-keyed, and `D8j` clause 2 is precisely the check that would compare a
  **stale ambient owner** against a planner identity.

  ⇒ **Open question for @architect, and it is candidate-blocking rather than
  academic: does any `D8a`-`D8k` control derive BOTH sides from the stale
  ambient owner, and therefore pass vacuously?** A control whose two sides share
  a defective source agrees with itself regardless of the defect. **I am not
  asserting the evidence is bad — I am refusing to assume it is fine**, because
  the field the defect is in is the field that chain is keyed on.

  **If the answer is that some rows need re-derivation after `D8o`, that is a
  finding about the evidence chain, not a new mechanism** — and it lands as
  re-measurement on preserved SHAs, not as reopened checkpoints.

  ###### `D8o` sizing and sequencing

  **In-node, on the same branch-topology grounds as `D8n`** — the fix must land
  on the held shared branch, and the rebase that a separate node would require
  destroys the preserved exact SHAs this node's evidence is addressed by.

  **Sequencing, per the Architect:** it does **not** mechanically block `D8n`'s
  evidence-only edit, **but it must be resolved before a candidate or any
  downstream proof that treats specialization-owner checks as authoritative.**
  ⇒ `D8o` runs after `D8n` closes and **before** the `D8m` witness, since that
  witness spans both function kinds and its ownership facts must be trustworthy
  when measured.

  ###### ANSWERED — nothing invalidated; ONE control is self-correlated. `evt_t4e294qz932v`.

  ⛔ **`D8a`-`D8k` are NOT reopened.** The stale specialization owner does not
  explain any accepted positive, `D8j` wrong-owner red, or `D8k` partition
  result. Why, per checkpoint:

  | checkpoint | why the stale field is not an input |
  |---|---|
  | `D8a`, `D8b`, `D8h` | owner, selector, target and identity populations derive entirely from planner-issued `ContinuationUnitView` / composed-target records |
  | `D8k` law-level row | obtains opaque identities from the plan and passes `identity.emission_owner()` **explicitly**; no specialization-body composed accumulation site exists |
  | `D8i` composed positive | its witness constructs and consumes in an **ordinary declaration-owned unit body**, whose pass binds `Predeclared(unit.function)` **before** source lowering |
  | `D8j` positive | **root-owned**; the claim is finalized in that ordinary root body, after the exact predeclared owner is bound. The later specialization emission is **downstream** of the measured relation |
  | `D8j` `WrongClaimingOwner` | substitutes **`None`** — it does not derive a second agreeing value from the stale field |

  ###### The ONE self-correlated control — `D8i` clause 2

  ⭐ **This is the finding, and it is exactly the shape worth having asked
  about.** Clause 2's mutation chooses a "foreign" identity by asking for a
  planner target whose owner **differs from `self.defining_emission_owner`**;
  the production guard then compares that identity's owner **back to the same
  `self.defining_emission_owner`.**

  ⇒ **It proves the guard rejects disagreement RELATIVE TO THE AMBIENT VALUE.
  It cannot prove the ambient value is the planner-correct owner of the defining
  `Function`. If the field were stale, the mutation and the guard would still
  agree about which identity is "foreign."**

  **It is NOT a false green today** — on the current witness the refusal occurs
  during the earlier ordinary-unit pass, where the owner is explicitly bound.
  ⛔ **But it must no longer be cited as evidence for specialization-body owner
  correctness.**

  **`D8j` verification 2 carries the same limitation** as a generic
  implementation guard: it proves **agreement with the ambient field**, not the
  independent `Function`-to-owner mapping. Its root-owned row **remains valid**;
  it simply **says nothing about the stale specialization population.**

  ###### `D8o`'s candidate gate, and the BOUNDED remeasurement it enables

  ⛔⛔ **CORRECTED 2026-08-06 — THIS GATE UNDERSPECIFIED THE COMPARISON, AND
  THAT IS MY DEFECT.** It read *"observe the defining `Function` and compare
  its live `defining_unit` / `defining_emission_owner` **pair** to the
  planner-issued body authority."* **That asks for a PAIR COMPARISON.** The
  ring built one, twice, and both were blocked (`evt_6gk0n7p2eazn2`,
  `evt_4y1q8s6cq5m11`). **What is required is a KEYED RELATION**, and the
  corrected statement is below. ⛔ Do not restore the old wording.

  **`D8o` must supply the independent missing premise as a RELATION, not a
  multiset:**

  - **the domain is a closed, independently meaningful exact body key** —
    ordinary unit, continuation specialization, or generated context, **each
    carrying its exact planner descriptor identity**. An equivalent independent
    `Function`-to-body mapping is lawful;
  - **compare the complete `exact body key → live (owner, unit)` relation**
    against a planner-derived expectation;
  - **assert each of the three body kinds has a NON-EMPTY observed
    population** — the kind must be present *in the compared value*, not
    inferred beside it;
  - **cross-body inheritance stays red.**

  ⛔ **The discriminating test the frame previously could not state:
  SWAPPING TWO LAWFUL PAIRS BETWEEN TWO BODIES MUST RED.** Under an unkeyed
  multiset it does not — the multiset is unchanged and the `FuncId`s stay
  distinct.

  ⛔ **The expected pair comes from the body descriptor/pass input — never from
  the ambient field, `FuncId` alone, raw origin, or a selected composed
  identity.** A `FuncId` may *label* the observed `Function` only if its
  expected pair is joined through the independent declaration/body mapping.
  Deriving the expectation from the thing under test is the defect this whole
  section is about.

  **Then re-run and record, on the `D8o` descendant:** `D8i` clause 2, and
  `D8j` verification 2's positive **and** negative rows.

  ⭐ **This is bounded REMEASUREMENT, not rework** — it composes their
  **unchanged guard laws** with `D8o`'s newly independent body-owner premise.
  ⛔ **Preserve `D8i`'s and `D8j`'s prior exact SHAs as baselines; do not
  rewrite or reopen them.** `D8a`/`D8b`/`D8h` and `D8k` need **ordinary
  regression coverage only**, not owner-source re-derivation.

  ⛔ **If no lawful specialization-body composed claim exists, record that
  population as EMPTY — do not fabricate one.** `D8o`'s three-body authority
  evidence is the proof for that body kind.

  **Candidate stays held until `D8o` AND this bounded remeasurement are both
  recorded.** `D8m` remains correctly sequenced after `D8o`, because it does
  span ordinary and specialization `Function`s.

  ###### `D8p` — cross-body checked-application binding. `D8f`'s predecessor.

  **Architect ruling `evt_6jay1wesbfhbw` on preserved exact `0eb04397`, parent
  `01263e44`. The `D8f` HARD STOP IS ACCEPTED and `D8f` is NOT discharged.**

  ⛔ **This is NOT a bounded continuation of `D8f`.** `D8f` owns **one**
  question: when two real static-worker calls occur under one pending marker,
  **does exact application occurrence distinguish the ordinary call from the
  checked call?** ⛔ **The measured program never reaches that two-call state**,
  and making it do so requires a relation that **does not exist on this exact
  object**.

  **The missing contract:**

  ```text
  (exact defining body, exact checked application occurrence,
   checked call template + slot)
      -> function-local callable binding and operand authority
  ```

  **Producer: planning / unit definition. Consumer: call emission in the
  ordinary/composed unit `Function`.** ⚠ **The existing checked template names
  the application, but the `Function` that ENTERS the marker has no callable
  binding at that occurrence** — so supplying one changes **planner population,
  unit-body binding, and lowering emission** *before* `D8f` can ask its
  question. That is why it cannot fold in.

  **The lawful form:** project the **existing exact checked application
  authority** into the defining `Function` and **emit the exact call there**.
  Call target and operand run must be **function-local declarations derived
  from that same planner record**.

  ⛔ **Refused:** transporting a specialized value across the unit boundary;
  widening an ABI; inferring a target from worker/body shape; selecting by
  route, arity, binder index, call order, or **whichever target exists**.

  **All three present refusals stay load-bearing and must be preserved:**

  1. **Do not weaken the checked-segment law.** An inferred semantic IH layer
     cannot coexist with checked layers without its **own checked invocation
     authority**.
  2. **Do not let `finish_checked_computational_ih_marker` accept a pending
     marker.** ⇒ That **erases** the exact-call obligation instead of
     satisfying it.
  3. **Do not bypass the source-open/dynamic-parent comparison.** ⇒ That
     attaches a real call to the **wrong dynamic invocation**.

  **The required proof, per exact defining body AND application occurrence:**
  zero/one/multiple binding behaviour; agreement of application origin, call
  template, slot, target and operand contract; a **real function-local emitted
  call**; and preservation of all three refusals above.

  **Only then may `D8f` resume** and prove the earlier ordinary call leaves the
  marker pending while the exact checked occurrence consumes it.

  ⛔ **`0eb04397` is HARD-STOP EVIDENCE ONLY.** Its sentinel and corrected
  post-`D8m` explanation are **sound evidence, not a candidate mechanism.**

  ⚠ **Mechanical, for any descendant:** `git diff --check 01263e44..0eb04397`
  reports a **new blank line at EOF in `control.rs`**. It does not affect the
  ruling but **must not transfer into a candidate.**

  ###### `D8p` sizing — IN-NODE, on my own recorded flip condition

  **An ordered in-node predecessor checkpoint, executing BEFORE `D8f`.** Label
  order does not track execution order on this node; read the numbers in the
  execution-order table.

  ⚠ **The case for a separate node is stronger here than at `D8n`, and I am
  recording it rather than brushing past it.** The Architect called it a
  **predecessor substrate** and it crosses **three planes** — planner
  population, unit-definition binding, lowering emission. It **would stand
  alone**, and this node is already large. By the standalone test alone, it
  passes.

  ⛔ **What decides it is the condition I NAMED at `D8n` and must now apply to
  myself:** *"if this node's branch ever merges before the lifecycle work is
  done, the answer flips to a node."* **`wp/RT-DECL-CLOSURE-PORT-typed-units`
  is UNMERGED and 168+ commits ahead of `origin/main`.** ⇒ **The condition is
  NOT met, so the topology argument stands unchanged:** a separate node reaches
  the held branch only by rebasing it — **destroying every preserved exact
  checkpoint SHA this node's evidence chain is addressed by**, including the
  `0eb04397` the Architect just ruled on — or by landing on the same branch
  anyway, which recreates "two nodes, one branch, one candidate".

  ⭐ **Applying my own recorded criterion rather than re-deriving one is the
  point.** A sizing rule that is re-argued at each use is not a rule.

  ⚠ **What this does NOT settle, and it is the OPERATOR's:** whether the whole
  campaign should be restructured now that this node has absorbed `D8h`-`D8p`.
  **That is the standing campaign-sizing question, not a per-checkpoint call**,
  and no in-node ruling should be read as answering it.

  ###### `D8f` RECUT — `D8p` moved the frontier. Do not re-ask what it proved.

  **Grounded on the `D8p` handoff `evt_14tv58v4k5wxr` and the leader's review
  request `evt_68ry7r5bz86n5`, at exact `198b7c14`.**

  > ### CORRECTION — my escape condition was BINARY and the real verdict was neither branch
  >
  > This section originally said it was *"conditional on the `D8p` accept — if
  > that review blocks on **mechanism** rather than on evidence fidelity, this
  > section moves with it."* **The Architect's verdict `evt_w6wc0xt9gkqf` is
  > neither branch.** It **BLOCKS on evidence fidelity while explicitly
  > accepting the production mechanism** — and one of its three findings
  > **retires a claim this section quotes.**
  >
  > ⛔ **A two-branch condition has no cell for a fidelity block that lands on
  > the evidence the section leans on.** That is the same defect shape this
  > section exists to fix in `D8f`, committed in the fix itself. **Re-derived
  > below rather than patched around.**

  **What the verdict SETTLES, and it is the load-bearing half.** The production
  mechanism is **accepted**: both source-machine `StaticWorker` seats consult
  the existing checked-IH seam after their exact operand run is available; the
  exact source occurrence and carried binder ordinal identify the pending
  application; **no new identity, validator, target lookup, ABI widening, or
  route/shape/order inference**; the zero/one/declined population is real, the
  ordinary body is reached, and **all three refusals remain load-bearing.**
  ⇒ **`D8p`'s mechanism is not in question and `D8f`'s dependence on it is
  sound.** What is in question is what the committed **proof** may be said to
  have established.

  **What `D8p` already established. Quoted narrowly, because the temptation is
  to round it up.** On the witness carrying an ordinary call on the same
  selected recursive argument **inside the pending marker's own application**:
  the first emitted call carries **no** consumption, and the second consumes at
  the plan's **exact occurrence**. Same worker, same arity, same frame — so
  **route, arity, binder index and call order are all blind, and the occurrence
  decides.** The **always-admit mutation reds.**

  That last clause retires a claim that was standing in production. The gate's
  comment read *"UNWITNESSED, and measured to be so — mutating it to ALWAYS
  admit leaves the whole suite green."* **True until `D8p`, false now**;
  `198b7c14` states the current measurement and keeps the old one as dated
  history.

  ⛔ **DO NOT read the quoted paragraph as more than the verdict allows.** The
  Architect's finding 3 is that **the permutation is not meaningful on the
  measured population** — both bodies presently observe **the same** occurrence
  and target values, so swapping them is a **no-op**. That is lawful under this
  frame's own equal-value limit, but the handoff prose **called the occurrences
  distinct and the permutation discriminating**, and that prose is retired
  against the measurement.

  **What survives, stated at the strength it was actually earned:**

  | claim | status after `evt_w6wc0xt9gkqf` |
  |---|---|
  | the seam is consulted at both source-machine emission seats, on the exact occurrence and carried binder ordinal | **accepted** — mechanism |
  | the always-admit mutation reds, retiring the standing "UNWITNESSED" production comment | **stands** — a mutation result, untouched by all three findings |
  | route, arity, binder index and call order are blind **as a cross-body permutation discriminator** | **RETIRED** — the two bodies observe the same values, so the swap is a no-op |
  | the two-call ordering: the ordinary call leaves the marker pending and the exact checked occurrence consumes it | **NOT settled by this verdict either way.** It is a within-body claim and finding 3 is a cross-body one, so the verdict neither endorses nor retires it |

  **So the instruction to `D8f` is narrower than this section first stated.**
  `D8f` **must not re-run the always-admit mutation** — that result stands. But
  it **may not treat the two-call ordering as discharged** on `D8p`'s evidence,
  because the corrected `D8p` child has not yet been reviewed and the only
  finding touching a permutation retired it as a no-op.

  ⛔ **The trap that would follow from the original wording:** a reader takes
  *"do not re-ask the occupancy ordering"* as licence to skip the within-body
  proof, on the strength of a cross-body discriminator that has just been
  measured **absent**. **That is a proof skipped on evidence that does not
  cover it** — worse than the duplicate fixture the instruction was written to
  prevent.

  ###### RE-DERIVED against accepted `2be7adb0`. The table above STANDS.

  **The instruction one paragraph up is DISCHARGED, not pending.** `D8p` is
  **discharged** at exact `2be7adb0` (Architect `evt_6sk9hbp17893t`), and the
  accepted object confirms both rows that were in doubt:

  - **The permutation stays a no-op, now on the record rather than in retired
    prose.** Finding 4 of the accept: both defining bodies bind
    `StaticOriginId(21)` and call `StaticOriginId(29)` **with the same
    contract**, so the swap is **explicitly** a no-op and the text claims no
    present discriminator. ⇒ **The "retired" row is correct as written**, and
    the corrected child states the absence rather than papering it.
  - **The two-call ordering is still NOT discharged, and it is now reachable.**
    The leader's classification gate `evt_z0k71pe8006d` states it directly:
    *"`D8f`'s actual two-call occupancy state is now reachable, but it refuses
    downstream."* ⇒ **Reachable is not proved.** The row stands.

  ⭐ **So the correction was load-bearing, and this is the check that says so.**
  Had the original wording survived, `D8f` would have been released with licence
  to skip a within-body proof that the accepted object shows is **reachable and
  unproved** — the exact failure the re-derivation was scheduled to catch.

  ⛔ **Nothing further re-derives here.** `D8f`'s remaining gate is the
  classification, not this section.

  ###### `D8f` CLASSIFIED — bounded and IN-NODE. No further predecessor.

  **Architect ruling `evt_3ynqrahtzngaz`, on accepted exact `2be7adb0`.**
  ⭐ **The cheap outcome: the `D8n` flip condition never had to be applied**,
  because no new checkpoint is cut. `D8p` is not reopened.

  **Why it is bounded, and it is worth reading because it is nearly the
  opposite of what the refusal suggests.** The two calls are **already**
  separated by an exact authority at the right seat:
  `consume_checked_ih_marker_at_static_worker_call` compares the live
  `static_origin` with the pending marker's planner-issued `application_origin`
  **before either call is emitted**, and on the measured witness it **correctly**
  leaves the earlier ordinary call pending and consumes only at the exact
  checked occurrence.

  ⇒ **The refusal is DOWNSTREAM of a decision that is already right.** Both
  seats call `claim_composed_discharge` **regardless of that decision**, and it
  treats the binding-level `ComposedSourceContinuation(identity)` facet as
  sufficient call-site authority. So the ordinary call claims the identity
  first, the checked call claims it again, and the **unchanged** Function-local
  affine ledger **correctly** rejects the duplicate.

  ⛔ **Nothing is missing** — no planner population, ABI field, target lookup,
  causal identity, route, or validator. **This is the integration boundary
  failing to carry forward an authority decision it had already made.**

  **The required law — a closed disposition, three cases that may NOT be
  collapsed:**

  | case | required behaviour |
  |---|---|
  | no checked application pending | preserve `D8j` — a composed binding may claim normally |
  | pending at a **different** occurrence | emit the ordinary call, **leave the marker pending**, and do **not** claim its composed causal identity |
  | the **exact** checked application consumed | emit the call and claim that same planner-issued identity **exactly once** |

  ⛔ **A `bound: bool` gate is specifically wrong.** `false` currently conflates
  *"no marker exists"* with *"a different occurrence owns the pending marker"*,
  and those two must diverge. **Names and representation are Runtime's; the
  three cases are not.**

  ⛔ **Banned in the same breath:** rewriting the fixture, reclassifying a
  composed binding as direct, adding occurrence to the causal identity, minting
  a second identity, weakening or resetting or special-casing the
  Function-local affine ledger, and selecting by route, arity, binder index,
  call order, target shape or availability.

  **The proof, on the real two-call witness, derived INDEPENDENTLY of the live
  occurrence decision:** the first emitted call has a **pending-nonmatching**
  disposition and makes **no** composed claim, while the exact occurrence emits
  and makes **exactly one**. **Mutating the declined first call into an eligible
  claim must reproduce the current duplicate-discharge refusal**, and a lawful
  **no-marker `D8j` composed-call positive** must be retained so that an
  over-broad only-checked-discharge repair **reds**. Direct bindings still make
  no composed claim.

  ⚠ **The baseline duplicate is evidence only through explicit mutation — do
  not leave the positive witness red.**

  ⭐ **This confirms the correction above rather than superseding it.** The
  release requires the two-call ordering to be **proved**, which is exactly what
  the table records as *not settled*; and it carries the four refusals forward
  as **still owed on separate live paths**, which is the residue this section
  named. **The frame is being executed as written.**

  **What `D8f` still owes.** The four refusals — **omission, duplicate,
  transplant, wrong occurrence** — **each on its own live path**, plus **the
  two-call ordering itself**, which the row above records as *not settled either
  way* by `evt_w6wc0xt9gkqf`. `D8p`'s evidence puts none of the four on a path
  of its own.

  ⛔ **These are about WHICH CALL CONSUMES A PENDING CHECKED-IH MARKER.** `D6c`
  owes a **different** refusal set that also opens *omission, duplicate* — its
  subject is **selecting the raw/IH target and its selected-recursive-argument
  member, pre-emission.** **Discharging one discharges nothing of the other.**

  ⚠ **This paragraph originally said the scope was "smaller than the original
  cut" and named only the four refusals.** That rested on the two-call ordering
  being discharged, which the verdict does not support. **The scope is smaller
  than the original cut only by the always-admit mutation** — the one result
  that survives all three findings intact.

  **The closure `D8p` exposed, which the previous `D8f` text does not name.**
  The implementer stated it as the handoff's honest limit and did **not** claim
  `D8f`: the two-call witness now refuses **downstream** at *"one causal
  identity was discharged twice in a single function"*, because **the ordinary
  call re-uses a binding carrying a composed authority.** By the implementer's
  own reading that is a question about **the ordinary call's discharge**, not
  about occupancy, and it is recorded in the row and at the gate so the binding
  evidence is not read as more than it is.

  **Whose that is: the Architect's, and the routing is deliberate.** Whether the
  double-discharge refusal is an **in-`D8f` fixture question** or a **further
  mechanism boundary** is a design classification of exactly the kind that
  produced `D8p` from `D8f`'s previous hard stop. **It is not mine to rule and
  not the ring's to absorb.** It routes once the `D8p` verdict lands, so it
  costs one message rather than an interrupt to a review in progress.

  **`D8f`'s release is gated on that classification.** Not on the `D8p` accept
  alone.

  | classification | what happens to `D8f` |
  |---|---|
  | in-fixture | `D8f` absorbs it and is released on the residue above |
  | mechanism boundary | a further ordered predecessor, and the `D8n` flip condition is applied to it as it was to `D8p` — not re-derived |

  **Why this section exists at all, stated once.** This node already records the
  rule it is an instance of: *a checkpoint that establishes a required fact owes
  a statement of what that fact OBLIGES, not only what it provides.* `D8p`
  establishes the binding and **obliges** a disposition of the discharge
  question it exposed. **The previous defect was mine** — `D8f`'s section read
  as though occupancy were the only question, so releasing it unchanged would
  have sent the implementer at a witness **already measured to refuse for a
  reason the frame does not name.** That is what the `0eb04397` round cost, and
  it is not to be paid twice.

  **What this does NOT settle.** It does not discharge `D8f`, does not accept
  `D8p`, and does not touch the standing operator question of whether the
  **campaign** should be restructured now that this node has absorbed
  `D8h`-`D8p`.

  ###### GOVERNING — name the INDEPENDENT SIDE, and KEY the relation.

  **Binding on `D8p`, `D8f` and `D8g`. Author their controls against this
  before writing them, not after review sends them back.**

  Two Architect verdicts blocked `D8o`'s evidence — `evt_6gk0n7p2eazn2` and
  `evt_4y1q8s6cq5m11` — and with the earlier rounds and `D8f`'s three, this
  shape has now surfaced **eight** times in this node:

  | # | where | how the two sides agreed with each other |
  |---|---|---|
  | 1 | `D8j` 4b | a degenerate `1 == 1` comparator |
  | 2 | `D8n` | two rows green on a population the compile never reached |
  | 3 | `D8i` clause 2 | mutation picks "foreign" **relative to** `defining_emission_owner`; the guard compares **back to that same field** |
  | 4 | `D8o` round 1 | the authority row records the **bind arguments**, and the EMPTY specialization population is filtered by **the field under validation** |
  | 5 | `D8o` round 2 | timing fixed, but the proof **erases the body key** — an unkeyed pair multiset, so **swapping two lawful pairs between two bodies stays green** |
  | 6 | `D8f` `f42fff78` | the "independent side" is an expected **error substring**, and the final comparison is a `BTreeSet` of four formatted errors — a **bag of final errors**. Messages say where compilation stopped, never which call did what |
  | 7 | `D8f` `8671825c` | keyed at last, but the two planning "sides" **share one marker collector**, so the independent side is built from the thing it is checking |
  | 8 | `D8f` `cf770717` | origins read from the mechanism **on a lawful comparison compile**, and the second subject derived as the **complement** of the observed set |

  **The rule, in two halves. The second half is the one that took two rounds.**

  1. **Name the INDEPENDENT SIDE.** A control has to name which of its two
     sides is independent of the mechanism under test, and that side must be
     derivable without consulting the mechanism. If a checkpoint cannot name
     it, the control is not yet evidence, however green and however specific.
  2. **KEY THE RELATION.** When the property is *"each X has its own Y"*, the
     control must compare **`exact key → value` as a relation**. Comparing the
     **bag of values** proves only that the right values exist *somewhere*.

  ⛔ **Two corollaries, both of which look like they close the gap and do
  not:**

  - **A component's DISTINCTNESS is not a key on the relation.** Asserting
    every observed `FuncId` is distinct alongside a pair-multiset equality
    **does not restore the missing join** — the pairs are still unattached to
    the bodies that produced them.
  - **A VARIANT is not a KIND.** `D8O_COMPOSED_CLAIM_BODIES` recorded
    `(FuncId, ContinuationEmissionOwner)` and the assertion was named
    `from_specialization` — but **a generated context and a specialization
    body both lawfully carry a `Specialization(...)` owner.** Classifying the
    owner variant cannot answer *"which body kind made this claim?"* ⇒ **When
    a kind must be proved, the kind has to be PRESENT IN THE COMPARED VALUE**,
    not recoverable beside it.

  ⭐ **The cheap discriminating question to ask of any such control, which the
  frame could not state until now:** *what lawful permutation of my observed
  data leaves this assertion green?* If swapping two records between two
  subjects passes, the relation is unkeyed.

  ##### WHAT DISQUALIFIES A SIDE — from `D8f`'s three holds and `D8g`'s four

  Half 1 says the independent side must be **derivable without consulting the
  mechanism**. That was precise enough for the Architect to adjudicate every
  time, and the implementer conceded each finding — so the rule is not
  ambiguous. **It is nonetheless satisfiable SIX ways that feel independent and
  are not.** 1-3 are `D8f`'s (rows 6-8, one instance each); **4-6 are `D8g`'s
  closeout, where 4 alone accounts for THREE of its four holds.** Author against
  this list, because "I did not call the mechanism" is not the test:

  1. **Distance is not independence.** Reading the mechanism under test on a
     **different program** — a lawful comparison compile — leaves the side
     dependent. A consistent *wrong* classification in both runs keeps the row
     green. The lawful run is the same mechanism, one program removed.
  2. **A complement is not a derivation.** Naming subject A independently and
     defining subject B as *"the other one"* pins one origin, not two. The
     second subject must be named by its own authority.
  3. **A shared collector re-joins two sides that look separate.** Two
     populations built from one collector agree because they have a common
     ancestor, not because the property holds.
  4. ⛔ **A COARSER-KEYED independent side lets the observation select its own
     expectation.** This is the one that cost three holds, in three disguises:
     the observed `route` chose which branch to validate against; then a
     `worker_body_origin`-keyed map was compared against `(Function,
     call_origin)` observations; then the observed **target** picked which
     independent record it was checked against. **If the independent side is
     keyed more coarsely than the observation, the observation supplies the
     selector and the relation validates itself** — and a defect moving two
     fields **together** stays green. **Key both sides at the same granularity,
     require the KEY SETS EQUAL, and only then compare values.**
  5. **An expectation relabelled as an observation.** `worker.body_origin` was
     recorded as a "decoded raw callee". It is the binding's **expected** origin,
     read **before** emission, and raw and generated-context routes **share it by
     design** — so no wrong-table mutation could ever move it. **A rename is not
     a measurement:** say which side of the seam a value was read from.
  6. **An order surrogate is not a causal selector.** `.first()`,
     `.position(...)`, `.iter().next()` name *whichever sorted first*, not the
     subject. Injectivity makes the choice **unique without making it
     derived**. **Prove the population is a singleton, or name the subject from
     its own authority.**

  **What discharged it** (`d94a3df9`): both origins named by walking
  `StaticTransitionPlan::child_static_origin` — the planner's sole child-origin
  production point — down a positional path spelled from the witness's own
  shape, with the two occurrences asserted distinct so a collapsed path cannot
  leave the control with one subject. **Exact equalities against a named key,
  never filters over the observed set.**

  ⇒ **Binding on `D8g` and `D6c`, which inherit this rule.** `D6c`'s refusal
  set is the next control family large enough to repeat this arc.

  ###### THE TWO PROOF PLANES — ruled `evt_5vwdtrznf3km4`. `D6c` reads this FIRST.

  **A positive and a refusal are proved at different seams, and requiring one
  to meet the other's bar is how `D8g` lost a round.**

  | plane | where it must land |
  |---|---|
  | **positive** | the **keyed post-emission** relation with its exact actual-call evidence. ⛔ **For population 1 the independent side is bounded by the `EXACT-KEY LIMIT` — read it before quoting this row.** `evt_5vwdtrznf3km4` predates `evt_399j195rn8fpa` and did not know the authority was absent |
  | **refusal** | the **FIRST existing production guard that owns the moved input**, before any instruction exists, with an **empty emission set** |

  ⛔ **An earlier fail-closed guard is STRONGER, not weaker.** Arranging a
  bad-but-emittable call so a downstream observer can reject it is **not
  authorized** — that weakens production to satisfy a test. **Do not describe a
  pre-emission refusal as a post-emission relation failure; they are distinct
  planes.** A refusal row owes: the mutation **fired** (application count), the
  **typed** error matched exhaustively — **category plus discriminating reason**,
  never `is_err()` or a `contains` over formatted text — **zero** post-emission
  records, and the exact positive rerun separately.

  ⇒ **`D6c` is entirely a refusal set, so the refusal plane is its whole
  proof.** It owes **no** independently enumerated expectation population; a
  perturbation and a guard at the selection seam is the shape.

  ⛔ **LIMIT ON THAT QUESTION — added 2026-08-06 from the `D8m` partial, and it
  is a correction to the rule as I first wrote it.** **The swap discriminator
  requires the two subjects to hold DISTINCT values.** Where two subjects
  lawfully observe **the same** value, swapping them is a **no-op that neither
  the bag nor the keyed relation can see**, and no such swap exists to build.

  Measured on `D8m`'s current witness: its two bodies observe the **same**
  plan-named frame and slot, so the permutation is constructible and **not
  discriminating**.

  ⛔ **The right response is NOT to skip the clause and NOT to claim a
  rejection that does not happen.** Assert the equality explicitly and **name
  what would change it** — which is what the implementer did rather than
  reporting a discrimination it did not have. ⭐ **A control that says "this
  permutation is a no-op here, and here is the witness property that would make
  it bite" is stronger than one that quietly omits the check** — the first goes
  red when the witness gains a distinct second occurrence; the second never
  does.

  ⇒ **Where the values coincide, the discriminating power has to come from
  somewhere else** — on `D8m` that is the real distinct occurrence.

  ###### STANDING PRE-REPORT CHECK — this tell has now cost the node THREE times

  **Before reporting a clean compile as a MISSING REFUSAL, check whether the two
  subjects lawfully hold the SAME value.** A clean compile has at least three
  explanations and only the first is a gap:

  | explanation | what it actually means |
  |---|---|
  | production has no guard that owns the moved input | a real gap — the finding |
  | the two subjects hold **equal values** | an **observational identity**; there is no distinction to discriminate, and no such mutation exists to build |
  | the mutation **never applied** | an **instrument defect**; count applications separately from outcome |

  **Three occurrences, each reported as a gap before being reclassified:**

  1. **`D8m`'s permutation** — same plan-named frame and slot (above).
  2. **`D6b`'s raw-to-`worker_calls` mirror** — both tables lawfully hold the
     same declared target at an unretargeted body. Cost a release and a frame
     recut.
  3. **`D6c`'s wrong-order swap** — the governed witness's two members hold
     **identical route and body origin**; the census showed the permutation
     fires (`hits=3`) and is not discriminating. Where the members **are**
     distinct (`px8tr`), production **refuses**. ⇒ The unguarded set was **two,
     not three**.

  **The discriminator between explanations 2 and 3 is cheap and must be
  recorded separately:** application count proves the mutation **fired**;
  a structural read of the reshaped subject proves it **changed what it
  claimed to change**. Counting entry to an arm proves neither. Explanation 2
  is then settled by reading the two subjects' values — **not** by re-running
  the compile.

  ⇒ **State the subjects' values in the report, every time.** A finding of the
  form *"mutation X compiles clean, therefore no guard owns it"* is
  **incomplete** until it also says the two subjects differ. Two of the three
  above would have been caught before they reached a leader.

  ⭐ **RESOLVED at `e5ad0e10`, and the mechanism is worth copying.** The second
  occurrence is **two structurally identical checked composed bridges side by
  side in one scope, with distinct frame ids and distinct slot templates but
  DELIBERATELY EQUAL FINGERPRINTS.** ⇒ The pair is **invisible to every
  shape-keyed check**, leaving the transported identity as the only thing that
  tells them apart — and equal fingerprints are also **what lets a transplant
  reach lowering at all**, since planning compares them and admits the
  exchange. ⇒ **The permutation is now ENFORCED rather than recorded.**

  ###### SIXTH sub-shape — a difference proof needs a SOLE producer

  ⛔ **Binding on `D8f` and `D8g`.** Found at `e5ad0e10` by mutation, in
  evidence **already accepted at `7d7f248b`**:
  `d8m_the_source_frame_identity_survives_the_bridge` **clause 1 was green for
  the wrong reason.**

  It was written as `D8m`'s difference proof, reasoning that the slot-marker
  guard is **unreachable while the bridge carries `None`**. ⛔ **That reasoning
  is false for that witness: the same match is ALSO lowered by the DIRECT
  path**, which carried the identity before `D8m` and still does. ⇒ **The
  refusal arrives whether or not the bridge transports anything.**

  ⭐ **How it was caught, and it is the cheap test:** reverting the bridge to
  its complete pre-`D8m` all-`None` tuple **reds four of the five rows and
  leaves clause 1 GREEN.** A difference proof that survives the removal of the
  difference is not a difference proof.

  ⇒ **The rule: a difference proof requires the withheld mechanism to be the
  ONLY producer of the observable.** Before claiming one, ask **what else in
  production yields this same outcome** — a second lowering path, a fallback, an
  earlier guard. ⛔ **The independent side can be perfectly chosen and the proof
  still vacuous, because the defect is in the SUBJECT, not in the comparison.**

  ⚠ **Recording ORDER is part of this.** The `D8m` pair is recorded **before**
  the binding law runs. **Recorded after it, the two components could not
  disagree and the comparison would be agreeing with itself** — the same
  self-correlation as rows 3 and 4 above, reached through sequencing rather
  than through field choice.

  ⭐ **Two disciplines from that same delivery, both worth copying:** every
  refusal row is **labelled with the PLANE that catches it** (planning versus
  the consumption seam), because *"it refuses"* and *"it refuses at the bridge"*
  are different findings and **only the second is about `D8m`**; and where a
  control **refuses identically without** the new witness, that is **stated as
  a bound on what the witness buys** rather than quietly counted as coverage.

  ⛔ **CORRECTING A CLAIM HAS TWO SURFACES, AND THE EXECUTABLE ONE IS THE ONE
  THAT GETS MISSED.** Architect `evt_58ggvqc2d771q` blocked `e5ad0e10` on
  exactly this: the doc block and the new clause 1b **correctly** establish that
  withholding the bridge tuple leaves the refusal unchanged — while the
  **executable row still read** `Clause 1 — the identity arrives` and *"the
  bridge must carry ... and the proof is WHICH guard is reached"*. ⇒ **The
  refuted inference was still the assertion's own label and failure message**,
  which is what a reader sees when it fires and what a maintainer greps.

  ⇒ **When a checkpoint refutes a claim, sweep the CLAUSE LABELS and ASSERTION
  MESSAGES, not only the doc comment.** A correction that lands only in prose
  leaves the operative text still issuing the claim. **Same shape as
  [[a-later-note-saying-a-deliverable-is-false-does-not-replace-the-deliverable]]**,
  reached through test code rather than through a frame.

  ⚠ **The second block finding is the `D8o` zero again, in a new place:**
  `d8m_the_bridge_arm_populations_move_only_with_the_marker` runs **only**
  `ComputationalMatch` sites, so **no fixture in the row reaches
  `D8mBridgeArm::Ordinary`.** It observes zero ordinary sites before and after
  **on a population that contains no ordinary bridge.** ⛔ **A zero measured on
  an empty population is not the framed "unchanged" proof** — it needs a lawful
  ordinary bridge that reaches the same production recorder, with **the
  expected arm derived from the source descriptor, not from the recorder under
  test.**

  Three sub-shapes have now appeared, and all three read as rigorous:

  1. **The observation sits on the input side of the assignment under test.**
     `AmbientBodyAuthority::bind` recorded `owner`/`unit` **before** installing
     them, so the row proved *what the writer intended to install* — never what
     a reader sees. **Observe after the write, through the same field
     production reads.**
  2. **The key is dropped from a membership test.** Comparing recorded pairs
     against an **unkeyed union** while ignoring the recorded `function` admits
     cross-body substitution: hand a specialization any pair belonging to some
     other lawful body and membership still passes. **One-to-one against a
     per-body expectation, never set membership.** This is `D8n`'s
     `owners.len() == 2` over `BTreeSet<Option<FuncId>>` in a second costume.
  3. **An emptiness claim established through the field under validation.**
     Filtering `d8i_discharges()` by the claimed identity's emission owner and
     reading a `Specialization` owner as proof the construction happened inside
     a specialization body **is the very guard being remeasured.** A
     predeclared-owned identity observed under a wrongly bound specialization
     body classifies as predeclared and **preserves the asserted zero.**

  **A zero needs a stronger independent side than a positive, not a weaker
  one.** The briefing called this population *"the one place where 'we checked'
  means there was nothing to check."* That was the right place to look and it
  **understated the problem** — the population is not merely unexercised, it is
  **circularly established**, so the zero would survive the defect that makes it
  false.

  **The tell that the shape is present, and it is subtle:** the ordinary-unit
  call records its `function` **before** `open_aggregate_events(id)`, so that
  field was already wrong on the landed object. **The test never caught it
  because the test never used the field** — the weak assertion was itself the
  reason the defect stayed invisible. A control that ignores a field cannot
  report that the field is broken, so **an unused recorded field is not
  harmless slack; it is unmeasured surface.**

  ###### `D8k`'s ledger row is proved ON THE LAW, not on reach — and that is owed

  Recorded because the implementer stated it plainly rather than letting it
  pass: **no composed witness reaches `close_continuation_claim_ledger`** —
  both stop at the envelope frontier. The row exercises the ledger with real
  planner identities through the ordinary projection, so it **proves the law**;
  it does **not** prove any program reaches it.

  ⇒ **Whoever closes `D8l` owes the end-to-end assertion, ADDED BESIDE this row
  rather than replacing it.** A law-level proof and a reach-level proof fail for
  different reasons, and collapsing them loses the one that still holds.

  **Two things `D8k` established that must not be re-litigated.** The residual
  filter's soundness rests on **pass ORDER**: the seat runs while the function
  is still being built, so it reads the claim as well as the verified relation
  (which is populated only once the CLIF is finished). That is safe **only**
  because verification runs before publication and fails the whole compile on
  any failed claim — so no artifact can exist in which an unverified claim
  suppressed a residual. ⛔ **Neither pass may move without re-deriving this.**
  And `record_composed` claims through the **same slot** a direct emission
  claims, so the partition is disjoint **at the point of claim**, not only at
  closeout.

  ###### SPENT — the `D7a`-`D7e` cut. Superseded by `D8a`-`D8g` above.

  Retained because `D8a` and `D8b` name specific pieces of it as preserved. The
  ordering table below is **no longer authoritative**; the `D8` table above is.

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

  > ⚠ **A THIRD clause of this superseded paragraph is now also qualified.**
  > *"Make the wrong-table mutation red"* holds **only where retargeting makes
  > the two tables differ** — see `D6b`'s asymmetric-law table. At an
  > equal-table seat it is unsatisfiable, and it was inherited forward from here
  > exactly like the two below.

  > ### THIS IS WHERE THE `D8g` DEFECT CAME FROM. Two clauses are WITHDRAWN.
  >
  > It sits in the superseded region, so it governs nothing — but it states two
  > retired requirements **affirmatively**, and it is the text `D8g`'s closeout
  > was derived from by copy. Anyone tracing *why* `D8g` was recut lands here.
  >
  > | clause | status |
  > |---|---|
  > | **one witness reaches BOTH** a same-body `GeneratedContext` IH and the composed/raw selected-argument emission | **WITHDRAWN 2026-08-06** — no population in reach satisfies it, and it was **never derived from a production contract** (Architect `evt_6afmy0398n847`). `D8g` is recut over the **two actual populations**, joined at `call_static_worker_with_inputs` |
  > | the raw target is **both declared and defined** | **WITHDRAWN** — the standalone `Function` clause, see Finding 2 |
  >
  > **The provenance is the lesson.** The conjunction was **inherited, not
  > derived** — it rode from `D7e` into `D8g` unchanged while the `D8l` clause
  > independently assigned composed emission to a **composed** witness, and the
  > two were never reconciled. It cost a full implementer turn to a hard stop.
  > **A requirement you carried forward by copy has not been grounded by the
  > checkpoint that copied it.**

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

- **`D9` — assemble the continuation's ordinary run from the PLANNER ENVELOPE,
  not from the nonrecursive fields alone. ADDED 2026-08-06; this is the node's
  LAST deliverable and it is what `AC-1` was always asking for.**

  **Why it exists, stated plainly: this is a framing defect of mine.** `AC-1`
  named an outcome and **no deliverable was ever assigned to produce it.** Node
  QA blocked on it at `evt_28qa9ee7wx1sk` — correctly, and it is the first
  reader that could have, because **QA checks ACs against the artifact while
  every checkpoint review checks a checkpoint against a ruling.** A pass/fail
  total would have hidden it, which is exactly what `AC-1b` exists to prevent.

  **It is NOT a regression.** The row
  `buffer_allocate_malformed_capacity_narrows_to_invalid_bounds`
  has been red since `D0` at `179af863`. What moved is where it dies: a
  front-end `Match: scrutinee is not a constructor value` at `D0`, and now a
  fail-closed arity refusal at emission.

  ###### The measurement — Architect `evt_1y7h08xd7ermp`, on exact `bd79daa7`

  Taken with a temporary diagnostic in a detached worktree, since deleted; the
  candidate was never touched. The failing call is the **direct continuation
  emission at `core.rs:7621`, for `ContinuationSpecializationId(0)`**:

  | fact | value |
  |---|---|
  | callee target | origin `12`, declared **6 `Parameter` + 2 `Capture`** |
  | caller supplied | `1` ordinary + `2` projected continuation inputs = **3** |
  | planner ordinary envelope | one `NonrecursiveConstructorField(0)`, then **five exact `WorkerCapture` roles** |
  | selected recursive value | closure occurrence `381`, body `375`, arity `1`, **five captures already present** in `Lowered::Closure::captures` |

  **The identity and the planner envelope independently name the same closure,
  body, arity, five capture ordinals, owner, source origins and lifetimes.** So
  the callee descriptor is not inventing operands and the ABI is not missing a
  slot. **The caller builds only the nonrecursive field run — despite its own
  comment and the planner contract both requiring the selected worker's capture
  run to follow.** The five values are already available at that exact
  source-bearing seat through this node's landed phase-bearing closure-capture
  representation.

  **No hard stop fires.** The Architect classified this as a bounded omission in
  existing producer-side assembly: no environment lookup, new locator, new
  carrier, new slot kind or widened ABI. Section 7 clauses 1 and 2 are **not**
  triggered, which is what makes this framable in-node at all.

  ###### The repair surface — exactly three things

  1. Assemble the continuation's ordinary run from the **planner-issued
     `ContinuationOrdinaryEnvelopeRole` sequence.**
  2. Take **nonrecursive roles from their exact lowered source positions** and
     **worker-capture roles from the exact selected closure's existing ordered
     capture vector.**
  3. **Fail closed before emission** on closure occurrence/body/arity, capture
     ordinal/count/provenance, role order, or final header-cardinality
     disagreement.

  ###### BANNED repairs — each discards authority that already exists

  - weakening the six-parameter descriptor;
  - padding from `producer_env`;
  - treating capture ordinals as **generic environment indices**.

  The last two reintroduce the **cross-plane aliasing this node removed**. A
  green `AC-1` obtained by any of the three is not a discharge and will be
  rejected on review.

  ###### The controls, and the rule they are authored against

  **Author them against the GOVERNING block — *name the INDEPENDENT SIDE, and
  KEY the relation* — BEFORE writing them, not after review sends them back.**
  That shape has now cost this node **eight** holds, and `D9`'s proof is the
  same shape yet again: it compares an assembled run against a planner
  expectation.

  - **The independent side is the planner envelope**, which exists before any
    lowering assembly and is derivable without consulting the assembler.
    **Deriving the expectation from the assembled run is the defect** the whole
    GOVERNING section is about.
  - **Key the relation by exact role position.** A multiset of five capture
    values proves only that the right values exist *somewhere*. Ask the cheap
    discriminating question: *what lawful permutation of my observed data
    leaves this assertion green?* **Swapping two capture ordinals between two
    roles must RED.**
  - **The positive is `AC-1`'s row itself** — it reaches the real producer and
    returns `InvalidBounds` at the exact `264 -> 262 / position 1` consumer,
    with shared-host dispatch count **zero**. Removing the carried-capacity arm
    recreates the refusal at that exact seat.
  - **Report `AC-1` separately from the five-row `ComputationalMatch`
    population, always.** They refuse at different sites and greening one says
    nothing about the other.

  ###### Sizing

  One turn. The Architect's measurement means nothing needs discovering — the
  values, the authority and the contract all exist and are named above. **If the
  refusal set alone grows past a turn, split `D9` into `D9a` (assembly plus the
  `AC-1` positive) and `D9b` (the refusal set) rather than hard-stopping** — but
  do not split the guard away from the assembly it guards by default.

## 4. Acceptance criteria

- **`AC-1` — the linked row closes. OWNED BY `D9`.** Until 2026-08-06 this AC
  named an outcome **no deliverable was assigned to produce**, and node QA is
  what found it. **Every AC in this section must name its owning deliverable;
  if one cannot, that is the finding.** The `D0` row
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

  **This ban does NOT reach the `D8b` composed-call target** (Steward, on
  Architect ruling `evt_3dcafs581921e`). The bullet forbids an authority that
  **selects among admissible edges at emission**. The composed-call target is a
  **planner-issued identity**, minted under the `D8a` selector from provenance
  the planner already holds, and it is exactly what removes the need to choose
  at emission. The same reading already applies to the landed
  `StaticWorkerCallRoute`, accepted at `D6a` under this bullet. Do not cite this
  bullet against `D8b` or `D8d` — the ruling that authorizes them postdates it,
  and a first-call, shape, arity or *"whichever target exists"* rule stays
  banned by `D8b`'s own text.
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
